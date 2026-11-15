//! The build owner follows the actual CI and staging results without a phase table.
use super::super::{actual_outcome, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<CI_OWNER>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut build = owner.recv::<BuildLoan>().await?;
    {
        let (mut publication, mut checked) = loop {
            let inputs = imp::decode(build.recv::<PublicationA>().await?)?;
            let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let publication = &inputs["publication"];
            imp::ci_inputs(publication, &tests, 0)?;
            let bytes = imp::encode(publication)?;
            build.send::<FilePublicationA>(&bytes.as_slice()).await?;
            let arguments = build.recv::<CiRequestA>().await?.into_value();
            let value = domain
                .execute("CI_RUNNER", "run_tests", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::ci_result(&value, publication, &tests, 0)?;
            let bytes = imp::encode(&value)?;
            actual.set(Some(value.clone()));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    build
                        .send::<CiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                    let context = serde_json::json!({"publication":publication,"ci":value});
                    imp::staging_inputs(&context, 0)?;
                    let bytes = imp::encode(&context)?;
                    build.send::<StagingReadyA>(&bytes.as_slice()).await?;
                    break (publication.clone(), value);
                }
                DecisionArm::Right => {
                    build
                        .send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                }
            }
        };
        loop {
            let stage = imp::decode(build.recv::<StageUpdateA>().await?)?;
            imp::publication_stage(&stage, &publication, &checked, 0)?;
            if stage["success"] == true {
                break;
            }
            (publication, checked) = loop {
                let inputs = imp::decode(build.recv::<PublicationA>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 0)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestA>().await?.into_value();
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, publication, &tests, 0)?;
                let bytes = imp::encode(&value)?;
                actual.set(Some(value.clone()));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        build
                            .send::<StageCiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                        let context = serde_json::json!({"publication":publication,"ci":value});
                        imp::staging_inputs(&context, 0)?;
                        let bytes = imp::encode(&context)?;
                        build.send::<StagingReadyA>(&bytes.as_slice()).await?;
                        break (publication.clone(), value);
                    }
                    DecisionArm::Right => {
                        build
                            .send::<StageCiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                    }
                }
            };
        }
    }
    {
        let (mut publication, mut checked) = loop {
            let inputs = imp::decode(build.recv::<PublicationB>().await?)?;
            let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let publication = &inputs["publication"];
            imp::ci_inputs(publication, &tests, 1)?;
            let bytes = imp::encode(publication)?;
            build.send::<FilePublicationB>(&bytes.as_slice()).await?;
            let arguments = build.recv::<CiRequestB>().await?.into_value();
            let value = domain
                .execute("CI_RUNNER", "run_tests", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::ci_result(&value, publication, &tests, 1)?;
            let bytes = imp::encode(&value)?;
            actual.set(Some(value.clone()));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    build
                        .send::<CiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                    let context = serde_json::json!({"publication":publication,"ci":value});
                    imp::staging_inputs(&context, 1)?;
                    let bytes = imp::encode(&context)?;
                    build.send::<StagingReadyB>(&bytes.as_slice()).await?;
                    break (publication.clone(), value);
                }
                DecisionArm::Right => {
                    build
                        .send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                }
            }
        };
        loop {
            let stage = imp::decode(build.recv::<StageUpdateB>().await?)?;
            imp::publication_stage(&stage, &publication, &checked, 1)?;
            if stage["success"] == true {
                break;
            }
            (publication, checked) = loop {
                let inputs = imp::decode(build.recv::<PublicationB>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 1)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestB>().await?.into_value();
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, publication, &tests, 1)?;
                let bytes = imp::encode(&value)?;
                actual.set(Some(value.clone()));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        build
                            .send::<StageCiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                        let context = serde_json::json!({"publication":publication,"ci":value});
                        imp::staging_inputs(&context, 1)?;
                        let bytes = imp::encode(&context)?;
                        build.send::<StagingReadyB>(&bytes.as_slice()).await?;
                        break (publication.clone(), value);
                    }
                    DecisionArm::Right => {
                        build
                            .send::<StageCiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                    }
                }
            };
        }
    }
    {
        let (mut publication, mut checked) = loop {
            let inputs = imp::decode(build.recv::<PublicationC>().await?)?;
            let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let publication = &inputs["publication"];
            imp::ci_inputs(publication, &tests, 2)?;
            let bytes = imp::encode(publication)?;
            build.send::<FilePublicationC>(&bytes.as_slice()).await?;
            let arguments = build.recv::<CiRequestC>().await?.into_value();
            let value = domain
                .execute("CI_RUNNER", "run_tests", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::ci_result(&value, publication, &tests, 2)?;
            let bytes = imp::encode(&value)?;
            actual.set(Some(value.clone()));
            match actual_outcome(actual)
                .map_err(|error| ShellError::message(format!("{error:?}")))?
            {
                DecisionArm::Left => {
                    build
                        .send::<CiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                    let context = serde_json::json!({"publication":publication,"ci":value});
                    imp::staging_inputs(&context, 2)?;
                    let bytes = imp::encode(&context)?;
                    build.send::<StagingReadyC>(&bytes.as_slice()).await?;
                    break (publication.clone(), value);
                }
                DecisionArm::Right => {
                    build
                        .send::<CiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                        .await?;
                    build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                }
            }
        };
        loop {
            let stage = imp::decode(build.recv::<StageUpdateC>().await?)?;
            imp::publication_stage(&stage, &publication, &checked, 2)?;
            if stage["success"] == true {
                break;
            }
            (publication, checked) = loop {
                let inputs = imp::decode(build.recv::<PublicationC>().await?)?;
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 2)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationC>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestC>().await?.into_value();
                let value = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&value, publication, &tests, 2)?;
                let bytes = imp::encode(&value)?;
                actual.set(Some(value.clone()));
                match actual_outcome(actual)
                    .map_err(|error| ShellError::message(format!("{error:?}")))?
                {
                    DecisionArm::Left => {
                        build
                            .send::<StageCiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                        let context = serde_json::json!({"publication":publication,"ci":value});
                        imp::staging_inputs(&context, 2)?;
                        let bytes = imp::encode(&context)?;
                        build.send::<StagingReadyC>(&bytes.as_slice()).await?;
                        break (publication.clone(), value);
                    }
                    DecisionArm::Right => {
                        build
                            .send::<StageCiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                    }
                }
            };
        }
    }
    if build.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("staged integration failed"));
    }
    Ok(())
}
