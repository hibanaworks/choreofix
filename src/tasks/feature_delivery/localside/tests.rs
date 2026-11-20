//! The sole test environment owns candidate bytes and executes actual test requests.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run(
    owner: &mut Endpoint<TEST_OWNER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut environment = owner.recv::<TestLoan>().await?;
    let mut candidates = [Value::Null, Value::Null];
    loop {
        let branch = environment.offer().await?;
        match branch.label() {
            ReadyTestA::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ReadyTestA>().await?)?;
                imp::candidate(&result, FEATURE_A)?;
                candidates[0] = result;
            }
            TestRequestA::LOGICAL_LABEL => {
                let arguments = branch.recv::<TestRequestA>().await?;
                imp::candidate(&candidates[0], FEATURE_A)?;
                let result = effects.apply(arguments).await?;
                imp::tested(&result, &candidates[0], FEATURE_A)?;
                let bytes = imp::encode(&result)?;
                environment.send::<TestedA>(&bytes.as_slice()).await?;
                environment.send::<FileTestedA>(&bytes.as_slice()).await?;
            }
            ReviewFeedbackA::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ReviewFeedbackA>().await?)?;
                imp::reviewed(&result, &candidates[0], FEATURE_A)?;
            }
            ReadyTestB::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ReadyTestB>().await?)?;
                imp::candidate(&result, FEATURE_B)?;
                candidates[1] = result;
            }
            TestRequestB::LOGICAL_LABEL => {
                let arguments = branch.recv::<TestRequestB>().await?;
                imp::candidate(&candidates[1], FEATURE_B)?;
                let result = effects.apply(arguments).await?;
                imp::tested(&result, &candidates[1], FEATURE_B)?;
                let bytes = imp::encode(&result)?;
                environment.send::<TestedB>(&bytes.as_slice()).await?;
                environment.send::<FileTestedB>(&bytes.as_slice()).await?;
            }
            ReviewFeedbackB::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ReviewFeedbackB>().await?)?;
                imp::reviewed(&result, &candidates[1], FEATURE_B)?;
            }
            CiUpdate::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<CiUpdate>().await?)?;
                if !result["success"].is_boolean() {
                    return Err(ShellError::message("CI has no actual result"));
                }
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("delivery failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected test environment input {label}"
                )));
            }
        }
    }
}
