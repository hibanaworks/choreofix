//! One initial issue and final return of the same physical build inputs.
use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, BUILD_SERVICE>,
    (core, types): (Endpoint<'run, CORE_LIB>, Endpoint<'run, SHARED_TYPES>),
) -> ShellResult<()> {
    // The first actual file request selects the source choice before a loan.
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
    async {
        let [mut library, mut definitions] = endpoint.par()?;
        async {
            library.recv::<ReturnCore>().await?;
        };
        async {
            definitions.recv::<ReturnTypes>().await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}
