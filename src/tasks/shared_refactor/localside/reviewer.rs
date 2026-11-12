//! Physical owners consume actual requests before calling the original domain.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let candidate = imp::decode(endpoint.recv::<CandidateForOwner>().await?)?;
    for _ in 0..2 {
        let branch = endpoint.offer().await?;
        let arguments = match branch.label() {
            21 => {
                let arguments =
                    imp::arguments(branch.recv::<ReviewA>().await?, "pr_id", "module_a")?;
                let bytes = imp::encode(&arguments)?;
                endpoint.send::<AcceptedA>(&bytes.as_slice()).await?;
                arguments
            }
            22 => {
                let arguments =
                    imp::arguments(branch.recv::<ReviewB>().await?, "pr_id", "module_b")?;
                let bytes = imp::encode(&arguments)?;
                endpoint.send::<AcceptedB>(&bytes.as_slice()).await?;
                arguments
            }
            _ => return Err(ShellError::message("unexpected review request")),
        };
        let result = domain
            .execute("REVIEWER", "code_review", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let reply = json!({"success":result["success"],"candidate":candidate,"arguments":arguments,"result":result});
        let bytes = imp::encode(&reply)?;
        let reply = crate::shell::imp::review_reply::ReviewReply::from_bytes(&bytes)?;
        endpoint.send::<Approved>(&reply).await?;
        endpoint.send::<Approved>(&reply).await?;
        endpoint.send::<Approved>(&reply).await?;
    }
    Ok(())
}
