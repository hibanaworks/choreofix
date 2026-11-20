//! The owner of the sole test environment performs each actual test.
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
    let mut current = Value::Null;
    loop {
        let branch = environment.offer().await?;
        match branch.label() {
            ReadyTestA::LOGICAL_LABEL => {
                current["module_a"] =
                    imp::decode(branch.recv::<ReadyTestA>().await?)?["module_a"].take();
            }
            TestRequestA::LOGICAL_LABEL => {
                let received = branch.recv::<TestRequestA>().await?;
                let value = effects.apply(received).await?;
                imp::validation(&value, &current, "module_a")?;
                let bytes = imp::encode(&value)?;
                environment.send::<TestedA>(&bytes.as_slice()).await?;
                environment.send::<FileTestedA>(&bytes.as_slice()).await?;
            }
            ReadyTestB::LOGICAL_LABEL => {
                current["module_b"] =
                    imp::decode(branch.recv::<ReadyTestB>().await?)?["module_b"].take();
            }
            TestRequestB::LOGICAL_LABEL => {
                let received = branch.recv::<TestRequestB>().await?;
                let value = effects.apply(received).await?;
                imp::validation(&value, &current, "module_b")?;
                let bytes = imp::encode(&value)?;
                environment.send::<TestedB>(&bytes.as_slice()).await?;
                environment.send::<FileTestedB>(&bytes.as_slice()).await?;
            }
            TestExit::LOGICAL_LABEL => {
                if branch.recv::<TestExit>().await? != 0 {
                    return Err(ShellError::message("failed repository exit"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected test operation {label}"
                )));
            }
        }
    }
}
