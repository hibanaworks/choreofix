//! Actual review results drive the initial and CI-repair continuations directly.
use super::super::{actual_outcome, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn feature_a<D: Domain>(
    task: &mut Endpoint<REVIEW_A>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
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
        match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
            DecisionArm::Left => {
                task.send::<ApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                break;
            }
            DecisionArm::Right => {
                task.send::<RejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
            }
        }
    }
    loop {
        let result = imp::decode(task.recv::<CiUpdate>().await?)?;
        if result["data"]["feature"] != FEATURE_A {
            return Err(ShellError::message("unrelated CI data"));
        }
        match result["success"].as_bool() {
            Some(true) => break,
            Some(false) => loop {
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
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        task.send::<RepairApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        task.send::<RepairRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                    }
                }
            },
            None => return Err(ShellError::message("CI has no actual outcome")),
        }
    }
    if task.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("delivery failed"));
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn feature_b<D: Domain>(
    task: &mut Endpoint<REVIEW_B>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
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
        match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
            DecisionArm::Left => {
                task.send::<ApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                break;
            }
            DecisionArm::Right => {
                task.send::<RejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
            }
        }
    }
    loop {
        let result = imp::decode(task.recv::<CiUpdate>().await?)?;
        if result["data"]["feature"] != FEATURE_B {
            return Err(ShellError::message("unrelated CI data"));
        }
        match result["success"].as_bool() {
            Some(true) => break,
            Some(false) => loop {
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
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        task.send::<RepairApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        task.send::<RepairRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                    }
                }
            },
            None => return Err(ShellError::message("CI has no actual outcome")),
        }
    }
    if task.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("delivery failed"));
    }
    Ok(())
}
