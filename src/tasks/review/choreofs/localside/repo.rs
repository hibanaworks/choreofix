//! The sole repository view receives each actual operation and owns its effect.
use super::super::global::*;
use crate::shell::imp::service::ReceivedEffects;
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
#[hibana_runtime::scope]
pub async fn run(
    owner: &mut Endpoint<REPO_OWNER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut repository = owner.recv::<RepoLoan>().await?;
    loop {
        let branch = repository.offer().await?;
        match branch.label() {
            FileImplementA::LOGICAL_LABEL => {
                let result = effects
                    .apply(branch.recv::<FileImplementA>().await?)
                    .await?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository
                    .send::<FileImplementationA>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<Ready<TARGET_A>>(&bytes.as_slice())
                    .await?;
            }
            FileMergeA::LOGICAL_LABEL => {
                let result = effects.apply(branch.recv::<FileMergeA>().await?).await?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository.send::<FileMergedA>(&bytes.as_slice()).await?;
            }
            ApprovedLoanA::LOGICAL_LABEL => {
                let mut review = branch.recv::<ApprovedLoanA>().await?;
                let bytes = repository.recv::<ApprovedA>().await?.to_vec();
                repository
                    .send::<FileUpdate<A_FILE>>(&bytes.as_slice())
                    .await?;
                review
                    .send::<FileReviewResult<TARGET_A>>(&bytes.as_slice())
                    .await?;
            }
            RejectedLoanA::LOGICAL_LABEL => {
                let mut review = branch.recv::<RejectedLoanA>().await?;
                let bytes = repository.recv::<RejectedA>().await?.to_vec();
                repository
                    .send::<FileUpdate<A_FILE>>(&bytes.as_slice())
                    .await?;
                review
                    .send::<FileReviewResult<TARGET_A>>(&bytes.as_slice())
                    .await?;
                repository.send::<RejectedLoanA>(review).await?;
            }
            FileImplementB::LOGICAL_LABEL => {
                let result = effects
                    .apply(branch.recv::<FileImplementB>().await?)
                    .await?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository
                    .send::<FileImplementationB>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<Ready<TARGET_B>>(&bytes.as_slice())
                    .await?;
            }
            FileMergeB::LOGICAL_LABEL => {
                let result = effects.apply(branch.recv::<FileMergeB>().await?).await?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository.send::<FileMergedB>(&bytes.as_slice()).await?;
            }
            ApprovedLoanB::LOGICAL_LABEL => {
                let mut review = branch.recv::<ApprovedLoanB>().await?;
                let bytes = repository.recv::<ApprovedB>().await?.to_vec();
                repository
                    .send::<FileUpdate<B_FILE>>(&bytes.as_slice())
                    .await?;
                review
                    .send::<FileReviewResult<TARGET_B>>(&bytes.as_slice())
                    .await?;
            }
            RejectedLoanB::LOGICAL_LABEL => {
                let mut review = branch.recv::<RejectedLoanB>().await?;
                let bytes = repository.recv::<RejectedB>().await?.to_vec();
                repository
                    .send::<FileUpdate<B_FILE>>(&bytes.as_slice())
                    .await?;
                review
                    .send::<FileReviewResult<TARGET_B>>(&bytes.as_slice())
                    .await?;
                repository.send::<RejectedLoanB>(review).await?;
            }
            RepoExit::LOGICAL_LABEL => {
                let status = branch.recv::<RepoExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("review task failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected repository request {label}"
                )));
            }
        }
    }
}
