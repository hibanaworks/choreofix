//! Each test consumes the actual received object and a typed file request.
use super::super::wire::BuildRequest;
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<TESTER>) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        let report = match branch.label() {
            TestObjectA::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectA>().await?)?,
            TestObjectB::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectB>().await?)?,
            TestObjectC::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectC>().await?)?,
            FinishedTester::LOGICAL_LABEL => {
                let bytes = branch.recv::<FinishedTester>().await?.to_vec();
                endpoint.send::<ObjectForTesting>(&bytes.as_slice()).await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected actual build to test")),
        };
        let index = imp::module(&report["data"]["module"])?;
        let object = imp::object(&report, index)?;
        let bytes = imp::encode(&report)?;
        endpoint.send::<ObjectForTesting>(&bytes.as_slice()).await?;
        let arguments = match index {
            0 => endpoint.recv::<TestFileA>().await?.into_value(),
            1 => endpoint.recv::<TestFileB>().await?.into_value(),
            _ => endpoint.recv::<TestFileC>().await?.into_value(),
        };
        let bytes = imp::encode(&json!({"arguments":arguments,"object":object}))?;
        match index {
            0 => endpoint.send::<TestA>(&BuildRequest(&bytes)).await?,
            1 => endpoint.send::<TestB>(&BuildRequest(&bytes)).await?,
            _ => endpoint.send::<TestC>(&BuildRequest(&bytes)).await?,
        }
        let result = imp::decode(endpoint.recv::<Tested>().await?)?;
        imp::test_result(&result, index)?;
        if result["data"]["input_object"] != object {
            return Err(ShellError::message(
                "test result names a different received object",
            ));
        }
        let bytes = imp::encode(&result)?;
        endpoint.send::<FileTested>(&bytes.as_slice()).await?;
    }
}
