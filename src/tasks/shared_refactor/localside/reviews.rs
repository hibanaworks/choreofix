//! Each module's actual review can run independently of the other module.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(reviewer: &mut Endpoint<REVIEWER>, domain: &D) -> ShellResult<()> {
    async {
        let [mut a, mut b] = reviewer.par()?;
        async {
            let current = imp::decode(a.recv::<ReadyReviewA>().await?)?;
            let args = a.recv::<ReviewRequestA>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", args)
                .await
                .map_err(ShellError::message)?;
            imp::validation(&value, &current, "module_a")?;
            let bytes = imp::encode(&value)?;
            a.send::<ReviewedA>(&bytes.as_slice()).await?;
            a.send::<FileReviewedA>(&bytes.as_slice()).await?;
        };
        async {
            let current = imp::decode(b.recv::<ReadyReviewB>().await?)?;
            let args = b.recv::<ReviewRequestB>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", args)
                .await
                .map_err(ShellError::message)?;
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
