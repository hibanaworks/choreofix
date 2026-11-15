//! The build server owns actual publication inputs and returns the real CI result.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<CI_OWNER>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut build = owner.recv::<BuildLoan>().await?;
    let mut inputs = [Value::Null, Value::Null, Value::Null];
    loop {
        let branch = build.offer().await?;
        match branch.label() {
            PublicationA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<PublicationA>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(value["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&value["publication"], &tests, 0)?;
                let bytes = imp::encode(&value["publication"])?;
                inputs[0] = value;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
            }
            CiRequestA::LOGICAL_LABEL => {
                let arguments = branch.recv::<CiRequestA>().await?.into_value();
                let tests: [Value; 3] = serde_json::from_value(inputs[0]["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&inputs[0]["publication"], &tests, 0)?;
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, &inputs[0]["publication"], &tests, 0)?;
                actual.set(Some(value));
            }
            CiPassedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 0)?;
                branch
                    .send::<CiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[0]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 0)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyA>(&bytes.as_slice()).await?;
            }
            CiFailedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 0)?;
                branch
                    .send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultA>(&bytes.as_slice()).await?;
            }
            StageCiPassedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 0)?;
                branch
                    .send::<StageCiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[0]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 0)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyA>(&bytes.as_slice()).await?;
            }
            StageCiFailedA::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 0)?;
                branch
                    .send::<StageCiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultA>(&bytes.as_slice()).await?;
            }
            StageUpdateA::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateA>().await?)?;
                imp::stage_notice(&value, 0)?;
            }
            PublicationB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<PublicationB>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(value["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&value["publication"], &tests, 1)?;
                let bytes = imp::encode(&value["publication"])?;
                inputs[1] = value;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
            }
            CiRequestB::LOGICAL_LABEL => {
                let arguments = branch.recv::<CiRequestB>().await?.into_value();
                let tests: [Value; 3] = serde_json::from_value(inputs[1]["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&inputs[1]["publication"], &tests, 1)?;
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, &inputs[1]["publication"], &tests, 1)?;
                actual.set(Some(value));
            }
            CiPassedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 1)?;
                branch
                    .send::<CiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[1]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 1)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyB>(&bytes.as_slice()).await?;
            }
            CiFailedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 1)?;
                branch
                    .send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultB>(&bytes.as_slice()).await?;
            }
            StageCiPassedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 1)?;
                branch
                    .send::<StageCiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[1]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 1)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyB>(&bytes.as_slice()).await?;
            }
            StageCiFailedB::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 1)?;
                branch
                    .send::<StageCiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultB>(&bytes.as_slice()).await?;
            }
            StageUpdateB::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<StageUpdateB>().await?)?;
                imp::stage_notice(&value, 1)?;
            }
            PublicationC::LOGICAL_LABEL => {
                let value = imp::decode(branch.recv::<PublicationC>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(value["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&value["publication"], &tests, 2)?;
                let bytes = imp::encode(&value["publication"])?;
                inputs[2] = value;
                build.send::<FilePublicationC>(&bytes.as_slice()).await?;
            }
            CiRequestC::LOGICAL_LABEL => {
                let arguments = branch.recv::<CiRequestC>().await?.into_value();
                let tests: [Value; 3] = serde_json::from_value(inputs[2]["tests"].clone())
                    .map_err(|e| ShellError::message(e.to_string()))?;
                imp::ci_inputs(&inputs[2]["publication"], &tests, 2)?;
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, &inputs[2]["publication"], &tests, 2)?;
                actual.set(Some(value));
            }
            CiPassedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 2)?;
                branch
                    .send::<CiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[2]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 2)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyC>(&bytes.as_slice()).await?;
            }
            CiFailedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 2)?;
                branch
                    .send::<CiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultC>(&bytes.as_slice()).await?;
            }
            StageCiPassedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 2)?;
                branch
                    .send::<StageCiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                let context = serde_json::json!({"publication":inputs[2]["publication"],"ci":imp::decode(&bytes)?});
                imp::staging_inputs(&context, 2)?;
                let bytes = imp::encode(&context)?;
                build.send::<StagingReadyC>(&bytes.as_slice()).await?;
            }
            StageCiFailedC::LOGICAL_LABEL => {
                let bytes = imp::actual_bytes(actual)?;
                imp::ci_notice(&imp::decode(&bytes)?, 2)?;
                branch
                    .send::<StageCiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                build.send::<FileCiResultC>(&bytes.as_slice()).await?;
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
                    "unexpected build-server operation {label}"
                )));
            }
        }
    }
}
