//! The sole repository view receives each actual operation and owns its effect.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(owner: &mut Endpoint<REPO_OWNER>, domain: &D) -> ShellResult<()> {
    let mut repository = owner.recv::<RepoLoan>().await?;
    loop {
        let branch = repository.offer().await?;
        match branch.label() {
            200 => {
                let arguments = branch.recv::<FileImplementA>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_A", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository
                    .send::<FileImplementationA>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<Ready<TARGET_A>>(&bytes.as_slice())
                    .await?;
            }
            202 => {
                let arguments = branch.recv::<FileMergeA>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_A", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository.send::<FileMergedA>(&bytes.as_slice()).await?;
            }
            30 => {
                let bytes = branch.recv::<ApprovedA>().await?.to_vec();
                repository
                    .send::<FileUpdate<A_FILE>>(&bytes.as_slice())
                    .await?;
            }
            31 => {
                let bytes = branch.recv::<RejectedA>().await?.to_vec();
                repository
                    .send::<FileUpdate<A_FILE>>(&bytes.as_slice())
                    .await?;
            }
            201 => {
                let arguments = branch.recv::<FileImplementB>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_B", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository
                    .send::<FileImplementationB>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<Ready<TARGET_B>>(&bytes.as_slice())
                    .await?;
            }
            203 => {
                let arguments = branch.recv::<FileMergeB>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_B", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                repository.send::<FileMergedB>(&bytes.as_slice()).await?;
            }
            40 => {
                let bytes = branch.recv::<ApprovedB>().await?.to_vec();
                repository
                    .send::<FileUpdate<B_FILE>>(&bytes.as_slice())
                    .await?;
            }
            41 => {
                let bytes = branch.recv::<RejectedB>().await?.to_vec();
                repository
                    .send::<FileUpdate<B_FILE>>(&bytes.as_slice())
                    .await?;
            }
            234 => {
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
