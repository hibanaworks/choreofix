//! One affine repository view owns actual candidates, approvals and publications.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<REPO_OWNER>,
    (domain, order): (&D, &Cell<Option<[Value; 2]>>),
) -> ShellResult<()> {
    let mut repository = owner.recv::<RepoLoan>().await?;
    let mut candidates = [Value::Null, Value::Null];
    let mut reviews = [Value::Null, Value::Null];
    let mut tests = [Value::Null, Value::Null];
    let mut publication = Value::Null;
    let mut prior_ci = Value::Null;
    loop {
        let branch = repository.offer().await?;
        match branch.label() {
            ImplementA::LOGICAL_LABEL => {
                let arguments = branch.recv::<ImplementA>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_A", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&result, FEATURE_A)?;
                let bytes = imp::encode(&result)?;
                candidates[0] = result;
                repository
                    .send::<ImplementationA>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyReviewA>(&bytes.as_slice()).await?;
                repository
                    .send::<FileReviewCandidateA>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyTestA>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateA>(&bytes.as_slice())
                    .await?;
                repository.send::<InterfaceA>(&bytes.as_slice()).await?;
            }
            ApprovedA::LOGICAL_LABEL
            | RejectedA::LOGICAL_LABEL
            | RepairApprovedA::LOGICAL_LABEL
            | RepairRejectedA::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    ApprovedA::LOGICAL_LABEL => branch.recv::<ApprovedA>().await?.to_vec(),
                    RejectedA::LOGICAL_LABEL => branch.recv::<RejectedA>().await?.to_vec(),
                    RepairApprovedA::LOGICAL_LABEL => {
                        branch.recv::<RepairApprovedA>().await?.to_vec()
                    }
                    RepairRejectedA::LOGICAL_LABEL => {
                        branch.recv::<RepairRejectedA>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let result = imp::decode(&bytes)?;
                imp::reviewed(&result, &candidates[0], FEATURE_A)?;
                reviews[0] = result;
                order.set(Some(reviews.clone()));
            }
            ReviewFeedbackA::LOGICAL_LABEL => {
                let bytes = imp::encode(&reviews[0])?;
                branch.send::<ReviewFeedbackA>(&bytes.as_slice()).await?;
                repository
                    .send::<ReviewFeedbackA>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackA>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackA>(&bytes.as_slice())
                    .await?;
            }
            TestedA::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<TestedA>().await?)?;
                imp::tested(&result, &candidates[0], FEATURE_A)?;
                tests[0] = result;
            }
            MergeReadyA::LOGICAL_LABEL => {
                let bytes = imp::encode(&reviews[0])?;
                branch.send::<MergeReadyA>(&bytes.as_slice()).await?;
            }
            MergeA::LOGICAL_LABEL => {
                let arguments = branch.recv::<MergeA>().await?.into_value();
                imp::merge_inputs(&arguments, (FEATURE_A, &candidates[0], &reviews[0]))?;
                let result = domain
                    .execute("DEVELOPER_A", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::merge_result(&result, (FEATURE_A, &candidates[0], &reviews[0]), &prior_ci)?;
                let bytes = imp::encode(&result)?;
                repository.send::<MergedA>(&bytes.as_slice()).await?;
                let inputs = imp::encode(&serde_json::json!({"publication":result,"tests":tests}))?;
                publication = result;
                repository.send::<PublicationA>(&inputs.as_slice()).await?;
            }
            CiPassedA::LOGICAL_LABEL | CiFailedA::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    CiPassedA::LOGICAL_LABEL => branch.recv::<CiPassedA>().await?.to_vec(),
                    CiFailedA::LOGICAL_LABEL => branch.recv::<CiFailedA>().await?.to_vec(),
                    _ => unreachable!(),
                };
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
            }
            ImplementB::LOGICAL_LABEL => {
                let arguments = branch.recv::<ImplementB>().await?.into_value();
                let result = domain
                    .execute("DEVELOPER_B", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&result, FEATURE_B)?;
                let bytes = imp::encode(&result)?;
                candidates[1] = result;
                repository
                    .send::<ImplementationB>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyReviewB>(&bytes.as_slice()).await?;
                repository
                    .send::<FileReviewCandidateB>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyTestB>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateB>(&bytes.as_slice())
                    .await?;
                repository.send::<InterfaceB>(&bytes.as_slice()).await?;
            }
            ApprovedB::LOGICAL_LABEL
            | RejectedB::LOGICAL_LABEL
            | RepairApprovedB::LOGICAL_LABEL
            | RepairRejectedB::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    ApprovedB::LOGICAL_LABEL => branch.recv::<ApprovedB>().await?.to_vec(),
                    RejectedB::LOGICAL_LABEL => branch.recv::<RejectedB>().await?.to_vec(),
                    RepairApprovedB::LOGICAL_LABEL => {
                        branch.recv::<RepairApprovedB>().await?.to_vec()
                    }
                    RepairRejectedB::LOGICAL_LABEL => {
                        branch.recv::<RepairRejectedB>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let result = imp::decode(&bytes)?;
                imp::reviewed(&result, &candidates[1], FEATURE_B)?;
                reviews[1] = result;
                order.set(Some(reviews.clone()));
            }
            ReviewFeedbackB::LOGICAL_LABEL => {
                let bytes = imp::encode(&reviews[1])?;
                branch.send::<ReviewFeedbackB>(&bytes.as_slice()).await?;
                repository
                    .send::<ReviewFeedbackB>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackB>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackB>(&bytes.as_slice())
                    .await?;
            }
            TestedB::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<TestedB>().await?)?;
                imp::tested(&result, &candidates[1], FEATURE_B)?;
                tests[1] = result;
            }
            MergeReadyB::LOGICAL_LABEL => {
                let bytes = imp::encode(&reviews[1])?;
                branch.send::<MergeReadyB>(&bytes.as_slice()).await?;
            }
            MergeB::LOGICAL_LABEL => {
                let arguments = branch.recv::<MergeB>().await?.into_value();
                imp::merge_inputs(&arguments, (FEATURE_B, &candidates[1], &reviews[1]))?;
                let result = domain
                    .execute("DEVELOPER_B", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::merge_result(&result, (FEATURE_B, &candidates[1], &reviews[1]), &prior_ci)?;
                let bytes = imp::encode(&result)?;
                repository.send::<MergedB>(&bytes.as_slice()).await?;
                let inputs = imp::encode(&serde_json::json!({"publication":result,"tests":tests}))?;
                publication = result;
                repository.send::<PublicationB>(&inputs.as_slice()).await?;
            }
            CiPassedB::LOGICAL_LABEL | CiFailedB::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    CiPassedB::LOGICAL_LABEL => branch.recv::<CiPassedB>().await?.to_vec(),
                    CiFailedB::LOGICAL_LABEL => branch.recv::<CiFailedB>().await?.to_vec(),
                    _ => unreachable!(),
                };
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
            }
            LastCiPassedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<LastCiPassedA>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
                return Ok(());
            }
            LastCiPassedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<LastCiPassedB>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<CiUpdate>(&bytes.as_slice()).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
                repository.send::<TaskExit>(&0).await?;
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
