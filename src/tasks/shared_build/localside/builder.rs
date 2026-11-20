//! Direct file admission, actual resource requests, effect and object handoff.
use super::super::{global::*, imp, wire::Arguments};
use crate::shell::{ShellError, ShellResult, imp::service::PairedEffects};
use hibana::{Endpoint, g::Message};
#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    endpoint: &mut Endpoint<ROLE>,
    effects: &impl PairedEffects,
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
                imp::decode(incoming.recv::<CompileA>().await?.as_bytes())?
            }
            CompileB::LOGICAL_LABEL if ROLE == BUILDER_B => {
                imp::decode(incoming.recv::<CompileB>().await?.as_bytes())?
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
        let request = Arguments::<ROLE, true>::from_bytes(&bytes)?;
        let result = {
            let [mut a, mut b] = endpoint.par()?;
            let (library, definitions) = async {
                async {
                    async {
                        core.send::<CompileCore<ROLE>>(&request).await?;
                    };
                    a.recv::<CompileCore<ROLE>>().await
                };
                async {
                    async {
                        types.send::<CompileTypes<ROLE>>(&request).await?;
                    };
                    b.recv::<CompileTypes<ROLE>>().await
                };
            }
            .await?;
            if imp::decode(library.as_bytes())? != admitted
                || imp::decode(definitions.as_bytes())? != admitted
            {
                return Err(ShellError::message(
                    "compile resources differ from admitted arguments",
                ));
            }
            effects.apply_pair(library, definitions).await?
        };
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
