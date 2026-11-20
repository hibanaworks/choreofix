//! One physical test environment; requests use its actual Endpoint.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<D: ReceivedEffects>(
    owner: &mut Endpoint<TEST_OWNER>,
    effects: &D,
) -> ShellResult<()> {
    let mut environment = owner.recv::<TestLoan>().await?;
    let mut candidates = [Value::Null, Value::Null, Value::Null];
    loop {
        let branch = environment.offer().await?;
        match branch.label() {
            TestCandidateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestCandidateA>().await?)?;
                imp::candidate(&value, 0)?;
                candidates[0] = value;
            }
            TestRequestA::LOGICAL_LABEL => {
                let request = branch.recv::<TestRequestA>().await?;
                imp::candidate(&candidates[0], 0)?;
                let value = effects.apply(request).await?;
                imp::tested(&value, &candidates[0], 0)?;
                let bytes = imp::encode(&value)?;
                environment.send::<TestedA>(&bytes.as_slice()).await?;
                environment.send::<FileTestedA>(&bytes.as_slice()).await?;
            }
            ReviewFeedbackA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewFeedbackA>().await?)?;
                imp::review_notice(&value, 0)?;
            }
            CiUpdateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<CiUpdateA>().await?)?;
                imp::ci_notice(&value, 0)?;
            }
            StageUpdateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateA>().await?)?;
                imp::stage_notice(&value, 0)?;
            }
            TestCandidateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestCandidateB>().await?)?;
                imp::candidate(&value, 1)?;
                candidates[1] = value;
            }
            TestRequestB::LOGICAL_LABEL => {
                let request = branch.recv::<TestRequestB>().await?;
                imp::candidate(&candidates[1], 1)?;
                let value = effects.apply(request).await?;
                imp::tested(&value, &candidates[1], 1)?;
                let bytes = imp::encode(&value)?;
                environment.send::<TestedB>(&bytes.as_slice()).await?;
                environment.send::<FileTestedB>(&bytes.as_slice()).await?;
            }
            ReviewFeedbackB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewFeedbackB>().await?)?;
                imp::review_notice(&value, 1)?;
            }
            CiUpdateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<CiUpdateB>().await?)?;
                imp::ci_notice(&value, 1)?;
            }
            StageUpdateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateB>().await?)?;
                imp::stage_notice(&value, 1)?;
            }
            TestCandidateC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<TestCandidateC>().await?)?;
                imp::candidate(&value, 2)?;
                candidates[2] = value;
            }
            TestRequestC::LOGICAL_LABEL => {
                let request = branch.recv::<TestRequestC>().await?;
                imp::candidate(&candidates[2], 2)?;
                let value = effects.apply(request).await?;
                imp::tested(&value, &candidates[2], 2)?;
                let bytes = imp::encode(&value)?;
                environment.send::<TestedC>(&bytes.as_slice()).await?;
                environment.send::<FileTestedC>(&bytes.as_slice()).await?;
            }
            ReviewFeedbackC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<ReviewFeedbackC>().await?)?;
                imp::review_notice(&value, 2)?;
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
                    "unexpected test environment operation {label}"
                )));
            }
        }
    }
}
