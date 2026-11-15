//! Direct file admission, actual resource requests, effect and object handoff.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    endpoint: &mut Endpoint<ROLE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let (agent, module) = match ROLE {
        BUILDER_A => ("BUILDER_A", "module_a"),
        BUILDER_B => ("BUILDER_B", "module_b"),
        _ => return Err(ShellError::message("unknown builder")),
    };
    loop {
        let incoming = endpoint.offer().await?;
        let admitted = match incoming.label() {
            CompileA::LOGICAL_LABEL if ROLE == BUILDER_A => {
                incoming.recv::<CompileA>().await?.into_value()
            }
            CompileB::LOGICAL_LABEL if ROLE == BUILDER_B => {
                incoming.recv::<CompileB>().await?.into_value()
            }
            _ => return Err(ShellError::message("unexpected builder request")),
        };
        let (mut core, mut types) = async {
            let [mut library, mut definitions] = endpoint.par()?;
            async { library.recv::<CoreLoan>().await };
            async { definitions.recv::<TypesLoan>().await };
        }
        .await?;
        let bytes = imp::encode(&admitted)?;
        let (library, definitions) = async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                async {
                    core.send::<CompileCore>(&bytes.as_slice()).await?;
                };
                imp::decode(a.recv::<CompileCore>().await?)
            };
            async {
                async {
                    types.send::<CompileTypes>(&bytes.as_slice()).await?;
                };
                imp::decode(b.recv::<CompileTypes>().await?)
            };
        }
        .await?;
        if library != admitted || definitions != admitted {
            return Err(ShellError::message(
                "compile resources differ from admitted arguments",
            ));
        }
        let result = domain
            .execute(agent, "compile_module", admitted)
            .await
            .map_err(ShellError::message)?;
        imp::compiled(&result, agent, module)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                async {
                    a.send::<CoreCompiled>(&bytes.as_slice()).await?;
                };
                let result = core.recv::<CoreCompiled>().await?;
                if result[..] != bytes {
                    return Err(ShellError::message(
                        "core result differs from the actual compile",
                    ));
                }
                Ok(())
            };
            async {
                async {
                    b.send::<TypesCompiled>(&bytes.as_slice()).await?;
                };
                let result = types.recv::<TypesCompiled>().await?;
                if result[..] != bytes {
                    return Err(ShellError::message(
                        "types result differs from the actual compile",
                    ));
                }
                Ok(())
            };
            Ok(())
        }
        .await?;
        endpoint.send::<Compiled>(&bytes.as_slice()).await?;
        if ROLE == BUILDER_A {
            async {
                let [mut notification, mut handoff] = endpoint.par()?;
                async {
                    notification.send::<BuiltA>(&bytes.as_slice()).await?;
                };
                async {
                    handoff.send::<TypesUpdated>(&bytes.as_slice()).await?;
                    let [mut library, mut definitions] = handoff.par()?;
                    async {
                        library.send::<CoreLoan>(core).await?;
                    };
                    async {
                        definitions.send::<TypesLoan>(types).await?;
                    };
                    Ok(())
                };
                Ok(())
            }
            .await?;
            break;
        }
        let next = endpoint.offer().await?;
        match next.label() {
            CoreLoan::LOGICAL_LABEL | TypesLoan::LOGICAL_LABEL => {
                drop(next);
                async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async {
                        library.send::<CoreLoan>(core).await?;
                    };
                    async {
                        definitions.send::<TypesLoan>(types).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            BuiltB::LOGICAL_LABEL | ReturnCore::LOGICAL_LABEL | ReturnTypes::LOGICAL_LABEL => {
                drop(next);
                async {
                    let [mut notification, mut library, mut definitions] = endpoint.par()?;
                    async {
                        notification.send::<BuiltB>(&bytes.as_slice()).await?;
                    };
                    async {
                        library.send::<ReturnCore>(core).await?;
                    };
                    async {
                        definitions.send::<ReturnTypes>(types).await?;
                    };
                    Ok(())
                }
                .await?;
                break;
            }
            _ => return Err(ShellError::message("unexpected builder continuation")),
        }
    }
    Ok(())
}
