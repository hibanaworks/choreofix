//! Actual admitted and resource requests meet at the sole effect owner.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, BUILD_SERVICE>,
    (core, types): (Endpoint<'r, CORE_LIB>, Endpoint<'r, SHARED_TYPES>),
    domain: &D,
) -> ShellResult<()> {
    // Select the global route before borrowing the chosen pair of fragments.
    drop(endpoint.offer().await?);
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
    // Actual compiled data, not permissions, owners or completion flags.
    let mut objects = [Value::Null, Value::Null];
    let (_core, _types) = loop {
        let branch = endpoint.offer().await?;
        let (agent, module, arguments) = match branch.label() {
            CompileA::LOGICAL_LABEL => (
                "BUILDER_A",
                "module_a",
                imp::decode(branch.recv::<CompileA>().await?)?,
            ),
            CompileB::LOGICAL_LABEL => (
                "BUILDER_B",
                "module_b",
                imp::decode(branch.recv::<CompileB>().await?)?,
            ),
            ReturnCore::LOGICAL_LABEL | ReturnTypes::LOGICAL_LABEL => {
                drop(branch);
                break async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<ReturnCore>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<ReturnTypes>().await?) };
                }
                .await?;
            }
            _ => return Err(ShellError::message("unexpected shared build request")),
        };
        let arguments = imp::compile_arguments(arguments, module)?;
        let (core, types) = async {
            let [mut library, mut definitions] = endpoint.par()?;
            async { imp::decode(library.recv::<CompileCore>().await?) };
            async { imp::decode(definitions.recv::<CompileTypes>().await?) };
        }
        .await?;
        if core != arguments || types != arguments {
            return Err(ShellError::message(
                "compile resources differ from admitted arguments",
            ));
        }
        let result = domain
            .execute(agent, "compile_module", arguments)
            .await
            .map_err(ShellError::message)?;
        let object = imp::compiled(&result, agent, module)?;
        objects[usize::from(module == "module_b")] = object;
        let bytes = imp::encode(&result)?;
        async {
            let [mut library, mut definitions] = endpoint.par()?;
            async {
                library.send::<CoreCompiled>(&bytes.as_slice()).await?;
            };
            async {
                definitions.send::<TypesCompiled>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
    };
    if objects[0]["inputs"]["SHARED_TYPES"] != objects[1]["inputs"]["SHARED_TYPES"] {
        return Err(ShellError::message(
            "current modules were built against different types",
        ));
    }
    let arguments = imp::link_arguments(imp::decode(endpoint.recv::<Link>().await?)?)?;
    let result = domain
        .execute("INTEGRATOR", "link_modules", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::linked(&result, &objects)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Linked>(&bytes.as_slice()).await?;
    Ok(())
}
