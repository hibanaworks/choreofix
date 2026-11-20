//! The build owner follows the actual CI and staging results without a phase table.
use super::super::{actual_outcome, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::ReceivedEffects},
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message, runtime::resolver::DecisionArm};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<D: ReceivedEffects>(
    owner: &mut Endpoint<CI_OWNER>,
    (effects, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut build = owner.recv::<BuildLoan>().await?;
    let mut checked = [Value::Null, Value::Null, Value::Null];
    loop {
        let event = owner.offer().await?;
        match event.label() {
            PublicationA::LOGICAL_LABEL | RepairPublicationA::LOGICAL_LABEL => {
                let submission = event.label();
                let inputs = if submission == RepairPublicationA::LOGICAL_LABEL {
                    imp::decode(event.recv::<RepairPublicationA>().await?)?
                } else {
                    imp::decode(event.recv::<PublicationA>().await?)?
                };
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 0)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
                let request = build.recv::<CiRequestA>().await?;
                let value = effects.apply(request).await?;
                imp::ci_result(&value, publication, &tests, 0)?;
                actual.set(Some(value.clone()));
                let bytes = imp::encode(&value)?;
                match (
                    submission,
                    actual_outcome(actual)
                        .map_err(|error| ShellError::message(format!("{error:?}")))?,
                ) {
                    (PublicationA::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<CiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (PublicationA::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationA::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<StageCiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationA::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<StageCiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    _ => unreachable!("received publication contract"),
                }
                build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    let material = serde_json::json!({"publication":publication,"integrated_publication":publication,"ci":value});
                    imp::staging_inputs(&material, 0)?;
                    let bytes = imp::encode(&material)?;
                    build.send::<StagingReadyA>(&bytes.as_slice()).await?;
                    checked[0] = material;
                }
            }
            StageUpdateA::LOGICAL_LABEL => {
                let stage = imp::decode(event.recv::<StageUpdateA>().await?)?;
                imp::publication_stage(&stage, &checked[0], 0)?;
            }
            PublicationB::LOGICAL_LABEL | RepairPublicationB::LOGICAL_LABEL => {
                let submission = event.label();
                let inputs = if submission == RepairPublicationB::LOGICAL_LABEL {
                    imp::decode(event.recv::<RepairPublicationB>().await?)?
                } else {
                    imp::decode(event.recv::<PublicationB>().await?)?
                };
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 1)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
                let request = build.recv::<CiRequestB>().await?;
                let value = effects.apply(request).await?;
                imp::ci_result(&value, publication, &tests, 1)?;
                actual.set(Some(value.clone()));
                let bytes = imp::encode(&value)?;
                match (
                    submission,
                    actual_outcome(actual)
                        .map_err(|error| ShellError::message(format!("{error:?}")))?,
                ) {
                    (PublicationB::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<CiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (PublicationB::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationB::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<StageCiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationB::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<StageCiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    _ => unreachable!("received publication contract"),
                }
                build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    let material = serde_json::json!({"publication":publication,"integrated_publication":publication,"ci":value});
                    imp::staging_inputs(&material, 1)?;
                    let bytes = imp::encode(&material)?;
                    build.send::<StagingReadyB>(&bytes.as_slice()).await?;
                    checked[1] = material;
                }
            }
            StageUpdateB::LOGICAL_LABEL => {
                let stage = imp::decode(event.recv::<StageUpdateB>().await?)?;
                imp::publication_stage(&stage, &checked[1], 1)?;
            }
            PublicationC::LOGICAL_LABEL | RepairPublicationC::LOGICAL_LABEL => {
                let submission = event.label();
                let inputs = if submission == RepairPublicationC::LOGICAL_LABEL {
                    imp::decode(event.recv::<RepairPublicationC>().await?)?
                } else {
                    imp::decode(event.recv::<PublicationC>().await?)?
                };
                let tests: [Value; 3] = serde_json::from_value(inputs["tests"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
                let publication = &inputs["publication"];
                imp::ci_inputs(publication, &tests, 2)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationC>(&bytes.as_slice()).await?;
                let request = build.recv::<CiRequestC>().await?;
                let value = effects.apply(request).await?;
                imp::ci_result(&value, publication, &tests, 2)?;
                actual.set(Some(value.clone()));
                let bytes = imp::encode(&value)?;
                match (
                    submission,
                    actual_outcome(actual)
                        .map_err(|error| ShellError::message(format!("{error:?}")))?,
                ) {
                    (PublicationC::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<CiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (PublicationC::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<CiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationC::LOGICAL_LABEL, DecisionArm::Left) => {
                        build
                            .send::<StageCiPassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                    }
                    (RepairPublicationC::LOGICAL_LABEL, DecisionArm::Right) => {
                        build
                            .send::<StageCiFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                    }
                    _ => unreachable!("received publication contract"),
                }
                build.send::<FileCiResultC>(&bytes.as_slice()).await?;
                if value["success"] == true {
                    let material = serde_json::json!({"publication":publication,"integrated_publication":publication,"ci":value});
                    imp::staging_inputs(&material, 2)?;
                    let bytes = imp::encode(&material)?;
                    build.send::<StagingReadyC>(&bytes.as_slice()).await?;
                    checked[2] = material;
                }
            }
            StageUpdateC::LOGICAL_LABEL => {
                let stage = imp::decode(event.recv::<StageUpdateC>().await?)?;
                imp::publication_stage(&stage, &checked[2], 2)?;
            }
            TaskExit::LOGICAL_LABEL => {
                if event.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
                return Ok(());
            }
            label => return Err(ShellError::message(format!("unexpected CI input {label}"))),
        }
    }
}
