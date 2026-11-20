use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub(crate) async fn run(
    control: &mut Endpoint<BACKEND_CLOSE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    control.recv::<MigrationDone>().await?;
    let result = effects
        .apply(control.recv::<BackendRequest>().await?)
        .await?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    let [mut ready, mut file] = control.par()?;
    async {
        ready.send::<BackendReady>(&()).await?;
    };
    async {
        file.send::<AppliedResult>(&bytes.as_slice()).await?;
    };
    Ok(())
}
