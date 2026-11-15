//! Actual review results choose the replies allowed by each source continuation.
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
        let inputs = imp::decode(task.recv::<ReviewCandidateA>().await?)?;
        imp::review_inputs(&inputs, 0)?;
        let arguments = task.recv::<ReviewRequestA>().await?.into_value();
        let value = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::reviewed(&value, &inputs["candidate"], 0)?;
        if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
            return Err(ShellError::message("review preceded its actual tests"));
        }
        let bytes = imp::encode(&value)?;
        actual.set(Some(value));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
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
        let ci = imp::decode(task.recv::<CiUpdateA>().await?)?;
        imp::ci_notice(&ci, 0)?;
        if ci["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateA>().await?)?;
            imp::review_inputs(&inputs, 0)?;
            let arguments = task.recv::<ReviewRequestA>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 0)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
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
        }
    }
    loop {
        let stage = imp::decode(task.recv::<StageUpdateA>().await?)?;
        imp::stage_notice(&stage, 0)?;
        if stage["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateA>().await?)?;
            imp::review_inputs(&inputs, 0)?;
            let arguments = task.recv::<ReviewRequestA>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 0)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    task.send::<StageRepairApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                    break;
                }
                DecisionArm::Right => {
                    task.send::<StageRepairRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                }
            }
        }
        loop {
            let ci = imp::decode(task.recv::<CiUpdateA>().await?)?;
            imp::ci_notice(&ci, 0)?;
            if ci["success"] == true {
                break;
            }
            loop {
                let inputs = imp::decode(task.recv::<ReviewCandidateA>().await?)?;
                imp::review_inputs(&inputs, 0)?;
                let arguments = task.recv::<ReviewRequestA>().await?.into_value();
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 0)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                let bytes = imp::encode(&value)?;
                actual.set(Some(value));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        task.send::<StageCiApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        task.send::<StageCiRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultA>(&bytes.as_slice()).await?;
                    }
                }
            }
        }
    }
    if task.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("staged integration failed"));
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn feature_b<D: Domain>(
    task: &mut Endpoint<REVIEW_B>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    loop {
        let inputs = imp::decode(task.recv::<ReviewCandidateB>().await?)?;
        imp::review_inputs(&inputs, 1)?;
        let arguments = task.recv::<ReviewRequestB>().await?.into_value();
        let value = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::reviewed(&value, &inputs["candidate"], 1)?;
        if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
            return Err(ShellError::message("review preceded its actual tests"));
        }
        let bytes = imp::encode(&value)?;
        actual.set(Some(value));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
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
        let ci = imp::decode(task.recv::<CiUpdateB>().await?)?;
        imp::ci_notice(&ci, 1)?;
        if ci["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateB>().await?)?;
            imp::review_inputs(&inputs, 1)?;
            let arguments = task.recv::<ReviewRequestB>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 1)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
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
        }
    }
    loop {
        let stage = imp::decode(task.recv::<StageUpdateB>().await?)?;
        imp::stage_notice(&stage, 1)?;
        if stage["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateB>().await?)?;
            imp::review_inputs(&inputs, 1)?;
            let arguments = task.recv::<ReviewRequestB>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 1)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    task.send::<StageRepairApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                    break;
                }
                DecisionArm::Right => {
                    task.send::<StageRepairRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                }
            }
        }
        loop {
            let ci = imp::decode(task.recv::<CiUpdateB>().await?)?;
            imp::ci_notice(&ci, 1)?;
            if ci["success"] == true {
                break;
            }
            loop {
                let inputs = imp::decode(task.recv::<ReviewCandidateB>().await?)?;
                imp::review_inputs(&inputs, 1)?;
                let arguments = task.recv::<ReviewRequestB>().await?.into_value();
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 1)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                let bytes = imp::encode(&value)?;
                actual.set(Some(value));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        task.send::<StageCiApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        task.send::<StageCiRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultB>(&bytes.as_slice()).await?;
                    }
                }
            }
        }
    }
    if task.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("staged integration failed"));
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn feature_c<D: Domain>(
    task: &mut Endpoint<REVIEW_C>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    loop {
        let inputs = imp::decode(task.recv::<ReviewCandidateC>().await?)?;
        imp::review_inputs(&inputs, 2)?;
        let arguments = task.recv::<ReviewRequestC>().await?.into_value();
        let value = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::reviewed(&value, &inputs["candidate"], 2)?;
        if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
            return Err(ShellError::message("review preceded its actual tests"));
        }
        let bytes = imp::encode(&value)?;
        actual.set(Some(value));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
            DecisionArm::Left => {
                task.send::<ApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                break;
            }
            DecisionArm::Right => {
                task.send::<RejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
            }
        }
    }
    loop {
        let ci = imp::decode(task.recv::<CiUpdateC>().await?)?;
        imp::ci_notice(&ci, 2)?;
        if ci["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateC>().await?)?;
            imp::review_inputs(&inputs, 2)?;
            let arguments = task.recv::<ReviewRequestC>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 2)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    task.send::<RepairApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                    break;
                }
                DecisionArm::Right => {
                    task.send::<RepairRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                }
            }
        }
    }
    loop {
        let stage = imp::decode(task.recv::<StageUpdateC>().await?)?;
        imp::stage_notice(&stage, 2)?;
        if stage["success"] == true {
            break;
        }
        loop {
            let inputs = imp::decode(task.recv::<ReviewCandidateC>().await?)?;
            imp::review_inputs(&inputs, 2)?;
            let arguments = task.recv::<ReviewRequestC>().await?.into_value();
            let value = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&value, &inputs["candidate"], 2)?;
            if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                return Err(ShellError::message("review preceded its actual tests"));
            }
            let bytes = imp::encode(&value)?;
            actual.set(Some(value));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    task.send::<StageRepairApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                    break;
                }
                DecisionArm::Right => {
                    task.send::<StageRepairRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                }
            }
        }
        loop {
            let ci = imp::decode(task.recv::<CiUpdateC>().await?)?;
            imp::ci_notice(&ci, 2)?;
            if ci["success"] == true {
                break;
            }
            loop {
                let inputs = imp::decode(task.recv::<ReviewCandidateC>().await?)?;
                imp::review_inputs(&inputs, 2)?;
                let arguments = task.recv::<ReviewRequestC>().await?.into_value();
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 2)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                let bytes = imp::encode(&value)?;
                actual.set(Some(value));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        task.send::<StageCiApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        task.send::<StageCiRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        task.send::<FileReviewResultC>(&bytes.as_slice()).await?;
                    }
                }
            }
        }
    }
    if task.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("staged integration failed"));
    }
    Ok(())
}
