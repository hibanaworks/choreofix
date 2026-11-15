//! Three independent review participants driven by actual requests and results.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn feature_a<D: Domain>(
    task: &mut Endpoint<REVIEW_A>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut inputs = Value::Null;
    loop {
        let branch = task.offer().await?;
        match branch.label() {
            ReviewCandidateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewCandidateA>().await?)?;
                imp::review_inputs(&value, 0)?;
                inputs = value;
            }
            ReviewRequestA::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReviewRequestA>().await?.into_value();
                imp::review_inputs(&inputs, 0)?;
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 0)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                actual.set(Some(value));
            }
            ApprovedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<ApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RejectedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairApprovedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairRejectedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairApprovedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairRejectedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiApprovedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiApprovedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiRejectedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiRejectedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            FileReviewResultA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch.send::<FileReviewResultA>(&bytes.as_slice()).await?;
            }
            CiUpdateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<CiUpdateA>().await?)?;
                imp::ci_notice(&value, 0)?;
            }
            StageUpdateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateA>().await?)?;
                imp::stage_notice(&value, 0)?;
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected feature A review operation {label}"
                )));
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn feature_b<D: Domain>(
    task: &mut Endpoint<REVIEW_B>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut inputs = Value::Null;
    loop {
        let branch = task.offer().await?;
        match branch.label() {
            ReviewCandidateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewCandidateB>().await?)?;
                imp::review_inputs(&value, 1)?;
                inputs = value;
            }
            ReviewRequestB::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReviewRequestB>().await?.into_value();
                imp::review_inputs(&inputs, 1)?;
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 1)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                actual.set(Some(value));
            }
            ApprovedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<ApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RejectedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairApprovedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairRejectedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairApprovedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairRejectedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiApprovedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiApprovedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiRejectedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiRejectedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            FileReviewResultB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch.send::<FileReviewResultB>(&bytes.as_slice()).await?;
            }
            CiUpdateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<CiUpdateB>().await?)?;
                imp::ci_notice(&value, 1)?;
            }
            StageUpdateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateB>().await?)?;
                imp::stage_notice(&value, 1)?;
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected feature B review operation {label}"
                )));
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn feature_c<D: Domain>(
    task: &mut Endpoint<REVIEW_C>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut inputs = Value::Null;
    loop {
        let branch = task.offer().await?;
        match branch.label() {
            ReviewCandidateC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewCandidateC>().await?)?;
                imp::review_inputs(&value, 2)?;
                inputs = value;
            }
            ReviewRequestC::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReviewRequestC>().await?.into_value();
                imp::review_inputs(&inputs, 2)?;
                let value = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&value, &inputs["candidate"], 2)?;
                if imp::receipt(&value)? <= imp::receipt(&inputs["tests"])? {
                    return Err(ShellError::message("review preceded its actual tests"));
                }
                actual.set(Some(value));
            }
            ApprovedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<ApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RejectedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairApprovedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            RepairRejectedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<RepairRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairApprovedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageRepairRejectedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageRepairRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiApprovedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiApprovedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
            }
            StageCiRejectedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch
                    .send::<StageCiRejectedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
            }
            FileReviewResultC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                branch.send::<FileReviewResultC>(&bytes.as_slice()).await?;
            }
            CiUpdateC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<CiUpdateC>().await?)?;
                imp::ci_notice(&value, 2)?;
            }
            StageUpdateC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateC>().await?)?;
                imp::stage_notice(&value, 2)?;
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected feature C review operation {label}"
                )));
            }
        }
    }
}
