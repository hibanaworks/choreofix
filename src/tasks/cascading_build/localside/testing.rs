//! The single physical worker returns actual input-bound test receipts.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(endpoint: &mut Endpoint<TEST_SERVICE>, domain: &D) -> ShellResult<()> {
    let mut results = [Value::Null, Value::Null, Value::Null];
    // Three initial tests precede a known aggregate send outside that choice.
    for _ in 0..3 {
        let branch = endpoint.offer().await?;
        let (index, request) = match branch.label() {
            TestA::LOGICAL_LABEL => (0, imp::decode(branch.recv::<TestA>().await?)?),
            TestB::LOGICAL_LABEL => (1, imp::decode(branch.recv::<TestB>().await?)?),
            TestC::LOGICAL_LABEL => (2, imp::decode(branch.recv::<TestC>().await?)?),
            _ => return Err(ShellError::message("unexpected initial test request")),
        };
        if request.as_object().is_none_or(|fields| fields.len() != 2) {
            return Err(ShellError::message(
                "test needs arguments and an actual input object",
            ));
        }
        let arguments = imp::arguments(request["arguments"].clone(), index)?;
        imp::validate_object(&request["object"], index)?;
        let result = domain
            .execute("TEST_RUNNER", "run_build_tests", arguments)
            .await
            .map_err(ShellError::message)?;
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
    endpoint.send::<AllForCompiler>(&bytes.as_slice()).await?;
    loop {
        let branch = endpoint.offer().await?;
        let (index, request) = match branch.label() {
            TestA::LOGICAL_LABEL => (0, imp::decode(branch.recv::<TestA>().await?)?),
            TestB::LOGICAL_LABEL => (1, imp::decode(branch.recv::<TestB>().await?)?),
            TestC::LOGICAL_LABEL => (2, imp::decode(branch.recv::<TestC>().await?)?),
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
        if request.as_object().is_none_or(|fields| fields.len() != 2) {
            return Err(ShellError::message(
                "test needs arguments and an actual input object",
            ));
        }
        let arguments = imp::arguments(request["arguments"].clone(), index)?;
        imp::validate_object(&request["object"], index)?;
        let result = domain
            .execute("TEST_RUNNER", "run_build_tests", arguments)
            .await
            .map_err(ShellError::message)?;
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
        endpoint.send::<AllForCompiler>(&bytes.as_slice()).await?;
    }
}
