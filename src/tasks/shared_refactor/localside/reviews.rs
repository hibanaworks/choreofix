//! Each module's actual review can run independently of the other module.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(
    reviewer: &mut Endpoint<REVIEWER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    async {
        let [mut a, mut b] = reviewer.par()?;
        async {
            let current = imp::decode(a.recv::<ReadyReviewA>().await?)?;
            let received = a.recv::<ReviewRequestA>().await?;
            let value = effects.apply(received).await?;
            imp::validation(&value, &current, "module_a")?;
            let bytes = imp::encode(&value)?;
            a.send::<ReviewedA>(&bytes.as_slice()).await?;
            a.send::<FileReviewedA>(&bytes.as_slice()).await?;
        };
        async {
            let current = imp::decode(b.recv::<ReadyReviewB>().await?)?;
            let received = b.recv::<ReviewRequestB>().await?;
            let value = effects.apply(received).await?;
            imp::validation(&value, &current, "module_b")?;
            let bytes = imp::encode(&value)?;
            b.send::<ReviewedB>(&bytes.as_slice()).await?;
            b.send::<FileReviewedB>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    if reviewer.recv::<ReviewExit>().await? != 0 {
        return Err(ShellError::message("failed repository exit"));
    }
    Ok(())
}
