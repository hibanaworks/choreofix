//! The physical repository owns candidates, tests, reviews and publications.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use crate::shell::{global::Loan, imp::review_reply::ReviewReply};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<D: Domain>(owner: &mut Endpoint<REPO_OWNER>, domain: &D) -> ShellResult<()> {
    let mut repository = owner.recv::<RepoLoan>().await?;
    let mut requests = owner.recv::<Loan<MERGE_REQUESTS>>().await?;
    async {
        loop {
            let request = requests.offer().await?;
            match request.label() {
                MergeRequestA::LOGICAL_LABEL => {
                    let value = request.recv::<MergeRequestA>().await?;
                    owner.send::<AdmitA>(&value).await?;
                    loop {
                        let verdict = owner.offer().await?;
                        match verdict.label() {
                            PublicationPassedA::LOGICAL_LABEL => {
                                verdict.recv::<PublicationPassedA>().await?;
                                break;
                            }
                            PublicationFailedA::LOGICAL_LABEL => {
                                verdict.recv::<PublicationFailedA>().await?;
                                let repaired = requests.recv::<MergeRequestA>().await?;
                                owner.send::<AdmitA>(&repaired).await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected publication outcome {label}"
                                )));
                            }
                        }
                    }
                }
                CheckStageA::LOGICAL_LABEL => {
                    let arguments = request.recv::<CheckStageA>().await?;
                    owner.send::<LatestCheckedA>(&arguments).await?;
                    owner.recv::<InstalledA>().await?;
                }
                MergeRequestB::LOGICAL_LABEL => {
                    let value = request.recv::<MergeRequestB>().await?;
                    owner.send::<AdmitB>(&value).await?;
                    loop {
                        let verdict = owner.offer().await?;
                        match verdict.label() {
                            PublicationPassedB::LOGICAL_LABEL => {
                                verdict.recv::<PublicationPassedB>().await?;
                                break;
                            }
                            PublicationFailedB::LOGICAL_LABEL => {
                                verdict.recv::<PublicationFailedB>().await?;
                                let repaired = requests.recv::<MergeRequestB>().await?;
                                owner.send::<AdmitB>(&repaired).await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected publication outcome {label}"
                                )));
                            }
                        }
                    }
                }
                CheckStageB::LOGICAL_LABEL => {
                    let arguments = request.recv::<CheckStageB>().await?;
                    owner.send::<LatestCheckedB>(&arguments).await?;
                    owner.recv::<InstalledB>().await?;
                }
                MergeRequestC::LOGICAL_LABEL => {
                    let value = request.recv::<MergeRequestC>().await?;
                    owner.send::<AdmitC>(&value).await?;
                    loop {
                        let verdict = owner.offer().await?;
                        match verdict.label() {
                            PublicationPassedC::LOGICAL_LABEL => {
                                verdict.recv::<PublicationPassedC>().await?;
                                break;
                            }
                            PublicationFailedC::LOGICAL_LABEL => {
                                verdict.recv::<PublicationFailedC>().await?;
                                let repaired = requests.recv::<MergeRequestC>().await?;
                                owner.send::<AdmitC>(&repaired).await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected publication outcome {label}"
                                )));
                            }
                        }
                    }
                }
                CheckStageC::LOGICAL_LABEL => {
                    let arguments = request.recv::<CheckStageC>().await?;
                    owner.send::<LatestCheckedC>(&arguments).await?;
                    owner.recv::<InstalledC>().await?;
                }
                TaskExit::LOGICAL_LABEL => {
                    if request.recv::<TaskExit>().await? != 0 {
                        return Err(ShellError::message("staging failed"));
                    }
                    return Ok::<(), ShellError>(());
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected repository request {label}"
                    )));
                }
            }
        }
    };
    let mut candidates = [Value::Null, Value::Null, Value::Null];
    let mut tests = [Value::Null, Value::Null, Value::Null];
    let mut reviews = [Value::Null, Value::Null, Value::Null];
    let mut stages = [Value::Null, Value::Null, Value::Null];
    let mut publications = [Value::Null, Value::Null, Value::Null];
    let mut stage_inputs = [Value::Null, Value::Null, Value::Null];
    let mut deployments = [Value::Null, Value::Null, Value::Null];
    let mut ci_results = [Value::Null, Value::Null, Value::Null];
    loop {
        let branch = repository.offer().await?;
        match branch.label() {
            ImplementA::LOGICAL_LABEL => {
                let arguments = branch.recv::<ImplementA>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_A", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&value, 0)?;
                let bytes = imp::encode(&value)?;
                candidates[0] = value;
                repository
                    .send::<ImplementationA>(&bytes.as_slice())
                    .await?;
                repository.send::<TestCandidateA>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateA>(&bytes.as_slice())
                    .await?;
                repository.send::<InterfaceAToB>(&bytes.as_slice()).await?;
                repository.send::<InterfaceAToC>(&bytes.as_slice()).await?;
            }
            TestedA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestedA>().await?)?;
                imp::tested(&value, &candidates[0], 0)?;
                tests[0] = value;
                let inputs = json!({"candidate":candidates[0],"tests":tests[0]});
                imp::review_inputs(&inputs, 0)?;
                let bytes = imp::encode(&inputs)?;
                repository
                    .send::<ReviewCandidateA>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<FileReviewCandidateA>(&bytes.as_slice())
                    .await?;
            }
            ApprovedA::LOGICAL_LABEL
            | RejectedA::LOGICAL_LABEL
            | RepairApprovedA::LOGICAL_LABEL
            | RepairRejectedA::LOGICAL_LABEL
            | StageRepairApprovedA::LOGICAL_LABEL
            | StageRepairRejectedA::LOGICAL_LABEL
            | StageCiApprovedA::LOGICAL_LABEL
            | StageCiRejectedA::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    ApprovedA::LOGICAL_LABEL => branch.recv::<ApprovedA>().await?.to_vec(),
                    RejectedA::LOGICAL_LABEL => branch.recv::<RejectedA>().await?.to_vec(),
                    RepairApprovedA::LOGICAL_LABEL => {
                        branch.recv::<RepairApprovedA>().await?.to_vec()
                    }
                    RepairRejectedA::LOGICAL_LABEL => {
                        branch.recv::<RepairRejectedA>().await?.to_vec()
                    }
                    StageRepairApprovedA::LOGICAL_LABEL => {
                        branch.recv::<StageRepairApprovedA>().await?.to_vec()
                    }
                    StageRepairRejectedA::LOGICAL_LABEL => {
                        branch.recv::<StageRepairRejectedA>().await?.to_vec()
                    }
                    StageCiApprovedA::LOGICAL_LABEL => {
                        branch.recv::<StageCiApprovedA>().await?.to_vec()
                    }
                    StageCiRejectedA::LOGICAL_LABEL => {
                        branch.recv::<StageCiRejectedA>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::reviewed(&value, &candidates[0], 0)?;
                if imp::receipt(&value)? <= imp::receipt(&tests[0])? {
                    return Err(ShellError::message(
                        "review preceded the actual completed tests",
                    ));
                }
                reviews[0] = value;
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
                repository
                    .send::<ReviewFeedbackA>(&bytes.as_slice())
                    .await?;
            }
            AdmitA::LOGICAL_LABEL => {
                let arguments = branch.recv::<AdmitA>().await?.into_value();
                imp::merge_inputs(&arguments, (0, &candidates[0], &tests[0], &reviews[0]))?;
                let value = domain
                    .execute("DEVELOPER_A", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::merge_result(
                    &value,
                    (0, &candidates[0], &tests[0], &reviews[0]),
                    &ci_results,
                )?;
                let bytes = imp::encode(&value)?;
                publications[0] = value;
                repository.send::<MergedA>(&bytes.as_slice()).await?;
                imp::ci_inputs(&publications[0], &tests, 0)?;
                let bytes = imp::encode(&json!({"publication":publications[0],"tests":tests}))?;
                if stages[0]["success"] == false {
                    repository
                        .send::<RepairPublicationA>(&bytes.as_slice())
                        .await?;
                } else {
                    repository.send::<PublicationA>(&bytes.as_slice()).await?;
                }
            }
            CiPassedA::LOGICAL_LABEL
            | CiFailedA::LOGICAL_LABEL
            | StageCiPassedA::LOGICAL_LABEL
            | StageCiFailedA::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    CiPassedA::LOGICAL_LABEL => branch.recv::<CiPassedA>().await?.to_vec(),
                    CiFailedA::LOGICAL_LABEL => branch.recv::<CiFailedA>().await?.to_vec(),
                    StageCiPassedA::LOGICAL_LABEL => {
                        branch.recv::<StageCiPassedA>().await?.to_vec()
                    }
                    StageCiFailedA::LOGICAL_LABEL => {
                        branch.recv::<StageCiFailedA>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::ci_result(&value, &publications[0], &tests, 0)?;
                imp::ci_notice(&value, 0)?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateA>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    repository
                        .send::<PublicationPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                } else {
                    repository
                        .send::<PublicationFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                }
                ci_results[0] = value;
            }
            LatestCheckedA::LOGICAL_LABEL => {
                let arguments = branch.recv::<LatestCheckedA>().await?.into_value();
                let mut head = None;
                for (index, ci) in ci_results
                    .iter()
                    .enumerate()
                    .filter(|(_, ci)| !ci.is_null())
                {
                    let occurrence = imp::receipt(ci)?;
                    if head.is_none_or(|(previous, _)| occurrence > previous) {
                        head = Some((occurrence, index));
                    }
                }
                let (_, index) =
                    head.ok_or_else(|| ShellError::message("deployment lacks tested repository"))?;
                let inputs = json!({"publication":publications[0],"integrated_publication":publications[index],"ci":ci_results[index]});
                let prior = core::array::from_fn(
                    |j| json!({"deployment":deployments[j],"validation":stages[j]}),
                );
                imp::deploy_input(&arguments, &inputs, (0, &prior))?;
                let bytes = imp::encode(&inputs)?;
                repository.send::<StageInputsA>(&bytes.as_slice()).await?;
                let bytes = repository.recv::<InstalledA>().await?.to_vec();
                let installed = imp::decode(&bytes)?;
                imp::deployed(&installed, &inputs, (0, &prior))?;
                stage_inputs[0] = inputs;
                deployments[0] = installed;
                repository.send::<InstalledA>(&bytes.as_slice()).await?;
            }
            StagePassedA::LOGICAL_LABEL | StageFailedA::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    StagePassedA::LOGICAL_LABEL => branch.recv::<StagePassedA>().await?.to_vec(),
                    StageFailedA::LOGICAL_LABEL => branch.recv::<StageFailedA>().await?.to_vec(),
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::publication_stage(&value, &stage_inputs[0], 0)?;
                stages[0] = value;
            }
            RollbackReceiptA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<RollbackReceiptA>().await?)?;
                imp::rollback_notice(&value, &stages[0], &stage_inputs[0], 0)?;
            }
            StageUpdateA::LOGICAL_LABEL => {
                imp::stage_notice(&stages[0], 0)?;
                let bytes = imp::encode(&stages[0])?;
                branch.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateA>(&bytes.as_slice()).await?;
            }
            ImplementB::LOGICAL_LABEL => {
                let arguments = branch.recv::<ImplementB>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_B", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&value, 1)?;
                let bytes = imp::encode(&value)?;
                candidates[1] = value;
                repository
                    .send::<ImplementationB>(&bytes.as_slice())
                    .await?;
                repository.send::<TestCandidateB>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateB>(&bytes.as_slice())
                    .await?;
                repository.send::<InterfaceBToA>(&bytes.as_slice()).await?;
                repository.send::<InterfaceBToC>(&bytes.as_slice()).await?;
            }
            TestedB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestedB>().await?)?;
                imp::tested(&value, &candidates[1], 1)?;
                tests[1] = value;
                let inputs = json!({"candidate":candidates[1],"tests":tests[1]});
                imp::review_inputs(&inputs, 1)?;
                let bytes = imp::encode(&inputs)?;
                repository
                    .send::<ReviewCandidateB>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<FileReviewCandidateB>(&bytes.as_slice())
                    .await?;
            }
            ApprovedB::LOGICAL_LABEL
            | RejectedB::LOGICAL_LABEL
            | RepairApprovedB::LOGICAL_LABEL
            | RepairRejectedB::LOGICAL_LABEL
            | StageRepairApprovedB::LOGICAL_LABEL
            | StageRepairRejectedB::LOGICAL_LABEL
            | StageCiApprovedB::LOGICAL_LABEL
            | StageCiRejectedB::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    ApprovedB::LOGICAL_LABEL => branch.recv::<ApprovedB>().await?.to_vec(),
                    RejectedB::LOGICAL_LABEL => branch.recv::<RejectedB>().await?.to_vec(),
                    RepairApprovedB::LOGICAL_LABEL => {
                        branch.recv::<RepairApprovedB>().await?.to_vec()
                    }
                    RepairRejectedB::LOGICAL_LABEL => {
                        branch.recv::<RepairRejectedB>().await?.to_vec()
                    }
                    StageRepairApprovedB::LOGICAL_LABEL => {
                        branch.recv::<StageRepairApprovedB>().await?.to_vec()
                    }
                    StageRepairRejectedB::LOGICAL_LABEL => {
                        branch.recv::<StageRepairRejectedB>().await?.to_vec()
                    }
                    StageCiApprovedB::LOGICAL_LABEL => {
                        branch.recv::<StageCiApprovedB>().await?.to_vec()
                    }
                    StageCiRejectedB::LOGICAL_LABEL => {
                        branch.recv::<StageCiRejectedB>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::reviewed(&value, &candidates[1], 1)?;
                if imp::receipt(&value)? <= imp::receipt(&tests[1])? {
                    return Err(ShellError::message(
                        "review preceded the actual completed tests",
                    ));
                }
                reviews[1] = value;
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
                repository
                    .send::<ReviewFeedbackB>(&bytes.as_slice())
                    .await?;
            }
            AdmitB::LOGICAL_LABEL => {
                let arguments = branch.recv::<AdmitB>().await?.into_value();
                imp::merge_inputs(&arguments, (1, &candidates[1], &tests[1], &reviews[1]))?;
                let value = domain
                    .execute("DEVELOPER_B", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::merge_result(
                    &value,
                    (1, &candidates[1], &tests[1], &reviews[1]),
                    &ci_results,
                )?;
                let bytes = imp::encode(&value)?;
                publications[1] = value;
                repository.send::<MergedB>(&bytes.as_slice()).await?;
                imp::ci_inputs(&publications[1], &tests, 1)?;
                let bytes = imp::encode(&json!({"publication":publications[1],"tests":tests}))?;
                if stages[1]["success"] == false {
                    repository
                        .send::<RepairPublicationB>(&bytes.as_slice())
                        .await?;
                } else {
                    repository.send::<PublicationB>(&bytes.as_slice()).await?;
                }
            }
            CiPassedB::LOGICAL_LABEL
            | CiFailedB::LOGICAL_LABEL
            | StageCiPassedB::LOGICAL_LABEL
            | StageCiFailedB::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    CiPassedB::LOGICAL_LABEL => branch.recv::<CiPassedB>().await?.to_vec(),
                    CiFailedB::LOGICAL_LABEL => branch.recv::<CiFailedB>().await?.to_vec(),
                    StageCiPassedB::LOGICAL_LABEL => {
                        branch.recv::<StageCiPassedB>().await?.to_vec()
                    }
                    StageCiFailedB::LOGICAL_LABEL => {
                        branch.recv::<StageCiFailedB>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::ci_result(&value, &publications[1], &tests, 1)?;
                imp::ci_notice(&value, 1)?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateB>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    repository
                        .send::<PublicationPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                } else {
                    repository
                        .send::<PublicationFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                }
                ci_results[1] = value;
            }
            LatestCheckedB::LOGICAL_LABEL => {
                let arguments = branch.recv::<LatestCheckedB>().await?.into_value();
                let mut head = None;
                for (index, ci) in ci_results
                    .iter()
                    .enumerate()
                    .filter(|(_, ci)| !ci.is_null())
                {
                    let occurrence = imp::receipt(ci)?;
                    if head.is_none_or(|(previous, _)| occurrence > previous) {
                        head = Some((occurrence, index));
                    }
                }
                let (_, index) =
                    head.ok_or_else(|| ShellError::message("deployment lacks tested repository"))?;
                let inputs = json!({"publication":publications[1],"integrated_publication":publications[index],"ci":ci_results[index]});
                let prior = core::array::from_fn(
                    |j| json!({"deployment":deployments[j],"validation":stages[j]}),
                );
                imp::deploy_input(&arguments, &inputs, (1, &prior))?;
                let bytes = imp::encode(&inputs)?;
                repository.send::<StageInputsB>(&bytes.as_slice()).await?;
                let bytes = repository.recv::<InstalledB>().await?.to_vec();
                let installed = imp::decode(&bytes)?;
                imp::deployed(&installed, &inputs, (1, &prior))?;
                stage_inputs[1] = inputs;
                deployments[1] = installed;
                repository.send::<InstalledB>(&bytes.as_slice()).await?;
            }
            StagePassedB::LOGICAL_LABEL | StageFailedB::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    StagePassedB::LOGICAL_LABEL => branch.recv::<StagePassedB>().await?.to_vec(),
                    StageFailedB::LOGICAL_LABEL => branch.recv::<StageFailedB>().await?.to_vec(),
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::publication_stage(&value, &stage_inputs[1], 1)?;
                stages[1] = value;
            }
            RollbackReceiptB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<RollbackReceiptB>().await?)?;
                imp::rollback_notice(&value, &stages[1], &stage_inputs[1], 1)?;
            }
            StageUpdateB::LOGICAL_LABEL => {
                imp::stage_notice(&stages[1], 1)?;
                let bytes = imp::encode(&stages[1])?;
                branch.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateB>(&bytes.as_slice()).await?;
            }
            ImplementC::LOGICAL_LABEL => {
                let arguments = branch.recv::<ImplementC>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_C", "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&value, 2)?;
                let bytes = imp::encode(&value)?;
                candidates[2] = value;
                repository
                    .send::<ImplementationC>(&bytes.as_slice())
                    .await?;
                repository.send::<TestCandidateC>(&bytes.as_slice()).await?;
                repository
                    .send::<FileTestCandidateC>(&bytes.as_slice())
                    .await?;
                repository.send::<InterfaceCToA>(&bytes.as_slice()).await?;
                repository.send::<InterfaceCToB>(&bytes.as_slice()).await?;
            }
            TestedC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestedC>().await?)?;
                imp::tested(&value, &candidates[2], 2)?;
                tests[2] = value;
                let inputs = json!({"candidate":candidates[2],"tests":tests[2]});
                imp::review_inputs(&inputs, 2)?;
                let bytes = imp::encode(&inputs)?;
                repository
                    .send::<ReviewCandidateC>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<FileReviewCandidateC>(&bytes.as_slice())
                    .await?;
            }
            ApprovedC::LOGICAL_LABEL
            | RejectedC::LOGICAL_LABEL
            | RepairApprovedC::LOGICAL_LABEL
            | RepairRejectedC::LOGICAL_LABEL
            | StageRepairApprovedC::LOGICAL_LABEL
            | StageRepairRejectedC::LOGICAL_LABEL
            | StageCiApprovedC::LOGICAL_LABEL
            | StageCiRejectedC::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    ApprovedC::LOGICAL_LABEL => branch.recv::<ApprovedC>().await?.to_vec(),
                    RejectedC::LOGICAL_LABEL => branch.recv::<RejectedC>().await?.to_vec(),
                    RepairApprovedC::LOGICAL_LABEL => {
                        branch.recv::<RepairApprovedC>().await?.to_vec()
                    }
                    RepairRejectedC::LOGICAL_LABEL => {
                        branch.recv::<RepairRejectedC>().await?.to_vec()
                    }
                    StageRepairApprovedC::LOGICAL_LABEL => {
                        branch.recv::<StageRepairApprovedC>().await?.to_vec()
                    }
                    StageRepairRejectedC::LOGICAL_LABEL => {
                        branch.recv::<StageRepairRejectedC>().await?.to_vec()
                    }
                    StageCiApprovedC::LOGICAL_LABEL => {
                        branch.recv::<StageCiApprovedC>().await?.to_vec()
                    }
                    StageCiRejectedC::LOGICAL_LABEL => {
                        branch.recv::<StageCiRejectedC>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::reviewed(&value, &candidates[2], 2)?;
                if imp::receipt(&value)? <= imp::receipt(&tests[2])? {
                    return Err(ShellError::message(
                        "review preceded the actual completed tests",
                    ));
                }
                reviews[2] = value;
            }
            ReviewFeedbackC::LOGICAL_LABEL => {
                let bytes = imp::encode(&reviews[2])?;
                branch.send::<ReviewFeedbackC>(&bytes.as_slice()).await?;
                repository
                    .send::<ReviewFeedbackC>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackC>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackC>(&bytes.as_slice())
                    .await?;
                repository
                    .send::<ReviewFeedbackC>(&bytes.as_slice())
                    .await?;
            }
            AdmitC::LOGICAL_LABEL => {
                let arguments = branch.recv::<AdmitC>().await?.into_value();
                imp::merge_inputs(&arguments, (2, &candidates[2], &tests[2], &reviews[2]))?;
                let value = domain
                    .execute("DEVELOPER_C", "merge_code", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::merge_result(
                    &value,
                    (2, &candidates[2], &tests[2], &reviews[2]),
                    &ci_results,
                )?;
                let bytes = imp::encode(&value)?;
                publications[2] = value;
                repository.send::<MergedC>(&bytes.as_slice()).await?;
                imp::ci_inputs(&publications[2], &tests, 2)?;
                let bytes = imp::encode(&json!({"publication":publications[2],"tests":tests}))?;
                if stages[2]["success"] == false {
                    repository
                        .send::<RepairPublicationC>(&bytes.as_slice())
                        .await?;
                } else {
                    repository.send::<PublicationC>(&bytes.as_slice()).await?;
                }
            }
            CiPassedC::LOGICAL_LABEL
            | CiFailedC::LOGICAL_LABEL
            | StageCiPassedC::LOGICAL_LABEL
            | StageCiFailedC::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    CiPassedC::LOGICAL_LABEL => branch.recv::<CiPassedC>().await?.to_vec(),
                    CiFailedC::LOGICAL_LABEL => branch.recv::<CiFailedC>().await?.to_vec(),
                    StageCiPassedC::LOGICAL_LABEL => {
                        branch.recv::<StageCiPassedC>().await?.to_vec()
                    }
                    StageCiFailedC::LOGICAL_LABEL => {
                        branch.recv::<StageCiFailedC>().await?.to_vec()
                    }
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::ci_result(&value, &publications[2], &tests, 2)?;
                imp::ci_notice(&value, 2)?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                repository.send::<CiUpdateC>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    repository
                        .send::<PublicationPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                } else {
                    repository
                        .send::<PublicationFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                }
                ci_results[2] = value;
            }
            LatestCheckedC::LOGICAL_LABEL => {
                let arguments = branch.recv::<LatestCheckedC>().await?.into_value();
                let mut head = None;
                for (index, ci) in ci_results
                    .iter()
                    .enumerate()
                    .filter(|(_, ci)| !ci.is_null())
                {
                    let occurrence = imp::receipt(ci)?;
                    if head.is_none_or(|(previous, _)| occurrence > previous) {
                        head = Some((occurrence, index));
                    }
                }
                let (_, index) =
                    head.ok_or_else(|| ShellError::message("deployment lacks tested repository"))?;
                let inputs = json!({"publication":publications[2],"integrated_publication":publications[index],"ci":ci_results[index]});
                let prior = core::array::from_fn(
                    |j| json!({"deployment":deployments[j],"validation":stages[j]}),
                );
                imp::deploy_input(&arguments, &inputs, (2, &prior))?;
                let bytes = imp::encode(&inputs)?;
                repository.send::<StageInputsC>(&bytes.as_slice()).await?;
                let bytes = repository.recv::<InstalledC>().await?.to_vec();
                let installed = imp::decode(&bytes)?;
                imp::deployed(&installed, &inputs, (2, &prior))?;
                stage_inputs[2] = inputs;
                deployments[2] = installed;
                repository.send::<InstalledC>(&bytes.as_slice()).await?;
            }
            StagePassedC::LOGICAL_LABEL | StageFailedC::LOGICAL_LABEL => {
                let bytes = match branch.label() {
                    StagePassedC::LOGICAL_LABEL => branch.recv::<StagePassedC>().await?.to_vec(),
                    StageFailedC::LOGICAL_LABEL => branch.recv::<StageFailedC>().await?.to_vec(),
                    _ => unreachable!(),
                };
                let value = imp::decode(&bytes)?;
                imp::publication_stage(&value, &stage_inputs[2], 2)?;
                stages[2] = value;
            }
            RollbackReceiptC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<RollbackReceiptC>().await?)?;
                imp::rollback_notice(&value, &stages[2], &stage_inputs[2], 2)?;
            }
            StageUpdateC::LOGICAL_LABEL => {
                imp::stage_notice(&stages[2], 2)?;
                let bytes = imp::encode(&stages[2])?;
                branch.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
                repository.send::<StageUpdateC>(&bytes.as_slice()).await?;
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
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
