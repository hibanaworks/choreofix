use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub(crate) async fn run(
    control: &mut Endpoint<ADMIN_CLOSE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let admitted = control.recv::<MigrationIntent>().await?;
    control.send::<MigrationIntent>(&admitted).await?;
    let request = control.recv::<MigrationRequest>().await?;
    if request.as_bytes() != admitted.0 {
        return Err(ShellError::message(
            "resource request differs from admitted migration",
        ));
    }
    let effect = effects.apply(request);
    let mut database = control.recv::<Loan<DATABASE_RESOURCE>>().await?;
    let result = effect.await?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    database
        .send::<MigrationCompleted>(&bytes.as_slice())
        .await?;
    control.send::<Loan<DATABASE_RESOURCE>>(database).await?;
    let [mut ready, mut file] = control.par()?;
    async {
        ready.send::<MigrationDone>(&()).await?;
    };
    async {
        file.send::<AppliedResult>(&bytes.as_slice()).await?;
    };
    Ok(())
}
