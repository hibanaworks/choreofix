//! One affine repository view owns actual candidates, approvals and publications.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(owner: &mut Endpoint<REPO_OWNER>, domain: &D) -> ShellResult<()> {
    let mut repository = owner.recv::<RepoLoan>().await?;
    let mut submissions = owner
        .recv::<crate::shell::global::Loan<MERGE_REQUESTS>>()
        .await?;
    async {
        for _ in 0..2 {
            let request = submissions.offer().await?;
            match request.label() {
                MergeA::LOGICAL_LABEL => {
                    let arguments = request.recv::<MergeA>().await?;
                    owner.send::<InitialMergeA>(&arguments).await?;
                    loop {
                        let decision = owner.offer().await?;
                        match decision.label() {
                            CiPassedA::LOGICAL_LABEL => {
                                let _ = decision.recv::<CiPassedA>().await?;
                                break;
                            }
                            CiFailedA::LOGICAL_LABEL => {
                                let _ = decision.recv::<CiFailedA>().await?;
                                let repaired = submissions.recv::<MergeA>().await?;
                                owner.send::<InitialMergeA>(&repaired).await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected integration decision {label}"
                                )));
                            }
                        }
                    }
                }
                MergeB::LOGICAL_LABEL => {
                    let arguments = request.recv::<MergeB>().await?;
                    owner.send::<InitialMergeB>(&arguments).await?;
                    loop {
                        let decision = owner.offer().await?;
                        match decision.label() {
                            CiPassedB::LOGICAL_LABEL => {
                                let _ = decision.recv::<CiPassedB>().await?;
                                break;
                            }
                            CiFailedB::LOGICAL_LABEL => {
                                let _ = decision.recv::<CiFailedB>().await?;
                                let repaired = submissions.recv::<MergeB>().await?;
                                owner.send::<InitialMergeB>(&repaired).await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected integration decision {label}"
                                )));
                            }
                        }
                    }
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected merge submission {label}"
                    )));
                }
            }
        }
        Ok::<(), ShellError>(())
    };
    let mut candidates = [Value::Null, Value::Null];
    let mut reviews = [Value::Null, Value::Null];
    let mut tests = [Value::Null, Value::Null];
    let mut publication = Value::Null;
    let mut prior_ci = Value::Null;
    let mut retired_a = None;
    let mut retired_b = None;
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
                repository.send::<InterfaceA>(&bytes.as_slice()).await?;
                repository.send::<ReadyReviewA>(&bytes.as_slice()).await?;
                repository
                    .send::<FileReviewCandidateA>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyTestA>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateA>(&bytes.as_slice())
                    .await?;
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
            InitialMergeA::LOGICAL_LABEL => {
                let arguments = branch.recv::<InitialMergeA>().await?.into_value();
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
            CiPassedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<CiPassedA>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
                // The sender already owns its actual CI result. Its service
                // view must arrive here before any file observes that success.
            }
            RetiredReviewA::LOGICAL_LABEL => {
                retired_a = Some(branch.recv::<RetiredReviewA>().await?);
                let bytes = imp::encode(&prior_ci)?;
                repository
                    .send::<CiPassedA>(
                        &crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?,
                    )
                    .await?;
            }
            CiFailedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<CiFailedA>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
                repository
                    .send::<CiFailedA>(
                        &crate::shell::imp::review_reply::ReviewReply::<false>::from_bytes(&bytes)?,
                    )
                    .await?;
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
                repository.send::<InterfaceB>(&bytes.as_slice()).await?;
                repository.send::<ReadyReviewB>(&bytes.as_slice()).await?;
                repository
                    .send::<FileReviewCandidateB>(&bytes.as_slice())
                    .await?;
                repository.send::<ReadyTestB>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateB>(&bytes.as_slice())
                    .await?;
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
            InitialMergeB::LOGICAL_LABEL => {
                let arguments = branch.recv::<InitialMergeB>().await?.into_value();
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
            CiPassedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<CiPassedB>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
            }
            RetiredReviewB::LOGICAL_LABEL => {
                retired_b = Some(branch.recv::<RetiredReviewB>().await?);
                let bytes = imp::encode(&prior_ci)?;
                repository
                    .send::<CiPassedB>(
                        &crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?,
                    )
                    .await?;
            }
            CiFailedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<CiFailedB>().await?.to_vec();
                let result = imp::decode(&bytes)?;
                imp::ci_result(&result, &publication, (&tests[0], &tests[1]))?;
                prior_ci = result;
                repository
                    .send::<CiFailedB>(
                        &crate::shell::imp::review_reply::ReviewReply::<false>::from_bytes(&bytes)?,
                    )
                    .await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected repository operation {label}"
                )));
            }
        }
        if let (Some(review_a), Some(review_b)) = (&mut retired_a, &mut retired_b) {
            repository.send::<TaskExit>(&0).await?;
            repository.send::<TaskExit>(&0).await?;
            if review_a.recv::<TaskExit>().await? != 0 {
                return Err(ShellError::message("delivery failed"));
            }
            repository.send::<TaskExit>(&0).await?;
            if review_b.recv::<TaskExit>().await? != 0 {
                return Err(ShellError::message("delivery failed"));
            }
            repository.send::<TaskExit>(&0).await?;
            return Ok(());
        }
    }
}
