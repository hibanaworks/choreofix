//! The repository owner lends its sole Endpoint and executes only requests
//! received through that Endpoint.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REPO>,
    (mut repository, control): (Endpoint<'r, REPO_ENDPOINT>, Endpoint<'r, REVIEW>),
    domain: &D,
) -> ShellResult<()> {
    endpoint.send::<ReviewControl>(control).await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            REVIEW => {
                drop(branch.recv::<ReviewControl>().await?);
            }
            69 | 76 | 169 | 176 => {
                let (agent, feature, admitted) = match branch.label() {
                    69 => ("DEVELOPER_A", FEATURE_A, branch.recv::<ImplementA>().await?),
                    76 => ("DEVELOPER_A", FEATURE_A, branch.recv::<ReviseA>().await?),
                    169 => ("DEVELOPER_B", FEATURE_B, branch.recv::<ImplementB>().await?),
                    176 => ("DEVELOPER_B", FEATURE_B, branch.recv::<ReviseB>().await?),
                    _ => unreachable!(),
                };
                let admitted = admitted.into_value();
                endpoint.send::<RepoLoan>(repository).await?;
                let request = endpoint.recv::<ImplementOnRepo>().await?.into_value();
                if request != admitted || request["feature_name"] != feature {
                    return Err(ShellError::message(
                        "repository implementation arguments differ",
                    ));
                }
                let result = domain
                    .execute(agent, "implement_feature", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint.send::<RepoResult>(&bytes.as_slice()).await?;
                repository = endpoint.recv::<RepoLoan>().await?;
            }
            70 | 170 => {
                let (agent, assigned_branch, admitted) = match branch.label() {
                    70 => (
                        "DEVELOPER_A",
                        MERGE_BRANCH_A,
                        branch.recv::<MergeA>().await?,
                    ),
                    170 => (
                        "DEVELOPER_B",
                        MERGE_BRANCH_B,
                        branch.recv::<MergeB>().await?,
                    ),
                    _ => unreachable!(),
                };
                let admitted = admitted.into_value();
                endpoint.send::<RepoLoan>(repository).await?;
                let request = endpoint.recv::<MergeOnRepo>().await?.into_value();
                if request != admitted || request["branch"] != assigned_branch {
                    return Err(ShellError::message("repository merge arguments differ"));
                }
                let result = domain
                    .execute(agent, "merge_code", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint.send::<RepoResult>(&bytes.as_slice()).await?;
                repository = endpoint.recv::<RepoLoan>().await?;
            }
            30 => {
                branch.recv::<ApprovedA>().await?;
            }
            31 => {
                branch.recv::<RejectedA>().await?;
            }
            40 => {
                branch.recv::<ApprovedB>().await?;
            }
            41 => {
                branch.recv::<RejectedB>().await?;
            }
            80 => {
                branch.recv::<FirstA>().await?;
            }
            81 => {
                branch.recv::<FirstB>().await?;
            }
            82 => {
                branch.recv::<AgainABeforeB>().await?;
            }
            83 => {
                branch.recv::<BDuringARevision>().await?;
            }
            84 => {
                branch.recv::<BAfterAMerge>().await?;
            }
            85 => {
                branch.recv::<AgainAAfterB>().await?;
            }
            86 => {
                branch.recv::<AgainBBeforeA>().await?;
            }
            87 => {
                branch.recv::<ADuringBRevision>().await?;
            }
            88 => {
                branch.recv::<AAfterBMerge>().await?;
            }
            89 => {
                branch.recv::<AgainBAfterA>().await?;
            }
            92 => {
                branch.recv::<AgainADuringBRevision>().await?;
            }
            93 => {
                branch.recv::<AgainBDuringARevision>().await?;
            }
            94 => {
                branch.recv::<AgainBBeforeAReview>().await?;
            }
            95 => {
                branch.recv::<AgainABeforeBReview>().await?;
            }
            234 => {
                branch.recv::<RepoExit>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected repository operation {label}"
                )));
            }
        }
    }
}
