//! Initial review hands its actual service views to CI repair; successful CI retires them.
use super::super::{actual_outcome, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn initial<D: Domain>(
    owner: &mut Endpoint<REVIEW_INITIAL>,
    (domain, actual_a, actual_b): (&D, &Cell<Option<Value>>, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut a = owner.recv::<crate::shell::global::Loan<REVIEW_A>>().await?;
    let mut b = owner.recv::<crate::shell::global::Loan<REVIEW_B>>().await?;
    let [mut control_a, mut control_b] = owner.par()?;
    async {
        loop {
            let candidate = imp::decode(a.recv::<ReadyReviewA>().await?)?;
            imp::candidate(&candidate, FEATURE_A)?;
            let arguments = a.recv::<ReviewRequestA>().await?.into_value();
            let result = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&result, &candidate, FEATURE_A)?;
            let bytes = imp::encode(&result)?;
            actual_a.set(Some(result));
            match actual_outcome(actual_a).map_err(|e| ShellError::message(format!("{e:?}")))? {
                DecisionArm::Left => {
                    a.send::<ApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    control_a
                        .send::<crate::shell::global::Loan<REVIEW_A>>(a)
                        .await?;
                    break;
                }
                DecisionArm::Right => {
                    a.send::<RejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    a.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                }
            }
        }
    };
    async {
        loop {
            let candidate = imp::decode(b.recv::<ReadyReviewB>().await?)?;
            imp::candidate(&candidate, FEATURE_B)?;
            let arguments = b.recv::<ReviewRequestB>().await?.into_value();
            let result = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&result, &candidate, FEATURE_B)?;
            let bytes = imp::encode(&result)?;
            actual_b.set(Some(result));
            match actual_outcome(actual_b).map_err(|e| ShellError::message(format!("{e:?}")))? {
                DecisionArm::Left => {
                    b.send::<ApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    control_b
                        .send::<crate::shell::global::Loan<REVIEW_B>>(b)
                        .await?;
                    break;
                }
                DecisionArm::Right => {
                    b.send::<RejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    b.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                }
            }
        }
    };
    Ok(())
}

#[hibana_runtime::scope]
pub async fn feature_a<D: Domain>(
    owner: &mut Endpoint<REVIEW_REPAIR_A>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let approved = owner.recv::<ApprovedA>().await?.to_vec();
    let mut task = owner.recv::<crate::shell::global::Loan<REVIEW_A>>().await?;
    task.send::<ApprovedA>(&ReviewReply::<true>::from_bytes(&approved)?)
        .await?;
    task.send::<FileReviewResultA>(&approved.as_slice()).await?;
    task.send::<MergeReadyA>(&approved.as_slice()).await?;
    loop {
        let result = imp::decode(owner.recv::<CiResultA>().await?)?;
        if result["data"]["feature"] != FEATURE_A {
            return Err(ShellError::message("unrelated CI data"));
        }
        let bytes = imp::encode(&result)?;
        actual.set(Some(result.clone()));
        match result["success"].as_bool() {
            Some(true) => {
                task.send::<CiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                owner.send::<RetiredReviewA>(task).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                return Ok(());
            }
            Some(false) => {
                task.send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                loop {
                    let candidate = imp::decode(task.recv::<ReadyReviewA>().await?)?;
                    imp::candidate(&candidate, FEATURE_A)?;
                    let arguments = task.recv::<ReviewRequestA>().await?.into_value();
                    let result = domain
                        .execute("REVIEWER", "code_review", arguments)
                        .await
                        .map_err(ShellError::message)?;
                    imp::reviewed(&result, &candidate, FEATURE_A)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(Some(result));
                    match actual_outcome(actual)
                        .map_err(|e| ShellError::message(format!("{e:?}")))?
                    {
                        DecisionArm::Left => {
                            task.send::<RepairApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                                .await?;
                            task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                            task.send::<MergeReadyA>(&bytes.as_slice()).await?;
                            break;
                        }
                        DecisionArm::Right => {
                            task.send::<RepairRejectedA>(&ReviewReply::<false>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                            task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                        }
                    }
                }
            }
            None => return Err(ShellError::message("CI has no actual outcome")),
        }
    }
}

#[hibana_runtime::scope]
pub async fn feature_b<D: Domain>(
    owner: &mut Endpoint<REVIEW_REPAIR_B>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let approved = owner.recv::<ApprovedB>().await?.to_vec();
    let mut task = owner.recv::<crate::shell::global::Loan<REVIEW_B>>().await?;
    task.send::<ApprovedB>(&ReviewReply::<true>::from_bytes(&approved)?)
        .await?;
    task.send::<FileReviewResultB>(&approved.as_slice()).await?;
    task.send::<MergeReadyB>(&approved.as_slice()).await?;
    loop {
        let result = imp::decode(owner.recv::<CiResultB>().await?)?;
        if result["data"]["feature"] != FEATURE_B {
            return Err(ShellError::message("unrelated CI data"));
        }
        let bytes = imp::encode(&result)?;
        actual.set(Some(result.clone()));
        match result["success"].as_bool() {
            Some(true) => {
                task.send::<CiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                owner.send::<RetiredReviewB>(task).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                return Ok(());
            }
            Some(false) => {
                task.send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                owner.send::<CiUpdate>(&bytes.as_slice()).await?;
                loop {
                    let candidate = imp::decode(task.recv::<ReadyReviewB>().await?)?;
                    imp::candidate(&candidate, FEATURE_B)?;
                    let arguments = task.recv::<ReviewRequestB>().await?.into_value();
                    let result = domain
                        .execute("REVIEWER", "code_review", arguments)
                        .await
                        .map_err(ShellError::message)?;
                    imp::reviewed(&result, &candidate, FEATURE_B)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(Some(result));
                    match actual_outcome(actual)
                        .map_err(|e| ShellError::message(format!("{e:?}")))?
                    {
                        DecisionArm::Left => {
                            task.send::<RepairApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                                .await?;
                            task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                            task.send::<MergeReadyB>(&bytes.as_slice()).await?;
                            break;
                        }
                        DecisionArm::Right => {
                            task.send::<RepairRejectedB>(&ReviewReply::<false>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                            task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                        }
                    }
                }
            }
            None => return Err(ShellError::message("CI has no actual outcome")),
        }
    }
}
