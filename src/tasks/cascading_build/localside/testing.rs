//! The single physical worker returns actual input-bound test receipts.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run(
    endpoint: &mut Endpoint<TEST_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut results = [Value::Null, Value::Null, Value::Null];
    // Three initial tests precede a known aggregate send outside that choice.
    for _ in 0..3 {
        let branch = endpoint.offer().await?;
        let (index, received) = match branch.label() {
            TestA::LOGICAL_LABEL => (0, branch.recv::<TestA>().await?),
            TestB::LOGICAL_LABEL => (1, branch.recv::<TestB>().await?),
            TestC::LOGICAL_LABEL => (2, branch.recv::<TestC>().await?),
            _ => return Err(ShellError::message("unexpected initial test request")),
        };
        let request = imp::decode(received.as_bytes())?;
        if request["object"]["module"] != imp::MODULES[index] {
            return Err(ShellError::message(
                "test message differs from its input module",
            ));
        }
        let result = effects.apply(received).await?;
        imp::test_result(&result, index)?;
        if result["data"]["input_object"] != request["object"] {
            return Err(ShellError::message(
                "test executed a different object from the received input",
            ));
        }
        results[index] = result.clone();
        let bytes = imp::encode(&result)?;
        async {
            let [mut tester, mut builder] = endpoint.par()?;
            async {
                tester.send::<Tested>(&bytes.as_slice()).await?;
            };
            async {
                builder.send::<Feedback>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    }
    let bytes = imp::encode(&serde_json::json!(results))?;
    async {
        let [mut compiler, mut integrator] = endpoint.par()?;
        async {
            compiler.send::<AllForCompiler>(&bytes.as_slice()).await?;
        };
        async {
            integrator
                .send::<AllForIntegrator>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    loop {
        let branch = endpoint.offer().await?;
        let (index, received) = match branch.label() {
            TestA::LOGICAL_LABEL => (0, branch.recv::<TestA>().await?),
            TestB::LOGICAL_LABEL => (1, branch.recv::<TestB>().await?),
            TestC::LOGICAL_LABEL => (2, branch.recv::<TestC>().await?),
            FinishedService::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<FinishedService>().await?)?;
                if result["tool_name"] != "link_modules"
                    || result["success"] != true
                    || result["data"]["status"] != "linked"
                {
                    return Err(ShellError::message(
                        "test worker did not receive the actual linked outcome",
                    ));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected test-worker continuation")),
        };
        let request = imp::decode(received.as_bytes())?;
        if request["object"]["module"] != imp::MODULES[index] {
            return Err(ShellError::message(
                "test message differs from its input module",
            ));
        }
        let result = effects.apply(received).await?;
        imp::test_result(&result, index)?;
        if result["data"]["input_object"] != request["object"] {
            return Err(ShellError::message(
                "test executed a different object from the received input",
            ));
        }
        results[index] = result.clone();
        let bytes = imp::encode(&result)?;
        async {
            let [mut tester, mut builder] = endpoint.par()?;
            async {
                tester.send::<Tested>(&bytes.as_slice()).await?;
            };
            async {
                builder.send::<Feedback>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        let bytes = imp::encode(&serde_json::json!(results))?;
        async {
            let [mut compiler, mut integrator] = endpoint.par()?;
            async {
                compiler.send::<AllForCompiler>(&bytes.as_slice()).await?;
            };
            async {
                integrator
                    .send::<AllForIntegrator>(&bytes.as_slice())
                    .await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    }
}
