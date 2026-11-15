//! Actual publication inputs and CI results drive the two integration branches.
use super::super::{actual_outcome, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message, runtime::resolver::DecisionArm};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<CI_OWNER>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut build = owner.recv::<BuildLoan>().await?;
    let first = build.offer().await?;
    match first.label() {
        PublicationA::LOGICAL_LABEL => {
            let mut inputs = imp::decode(first.recv::<PublicationA>().await?)?;
            loop {
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_A)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestA>().await?.into_value();
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                actual.set(Some(result));
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        build
                            .send::<CiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        build
                            .send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                    }
                }
                inputs = imp::decode(build.recv::<PublicationA>().await?)?;
            }
            let mut inputs = imp::decode(build.recv::<PublicationB>().await?)?;
            loop {
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_B)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestB>().await?.into_value();
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                actual.set(Some(result));
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        build
                            .send::<LastCiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        build
                            .send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                    }
                }
                inputs = imp::decode(build.recv::<PublicationB>().await?)?;
            }
        }
        PublicationB::LOGICAL_LABEL => {
            let mut inputs = imp::decode(first.recv::<PublicationB>().await?)?;
            loop {
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_B)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestB>().await?.into_value();
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                actual.set(Some(result));
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        build
                            .send::<CiPassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        build
                            .send::<CiFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultB>(&bytes.as_slice()).await?;
                    }
                }
                inputs = imp::decode(build.recv::<PublicationB>().await?)?;
            }
            let mut inputs = imp::decode(build.recv::<PublicationA>().await?)?;
            loop {
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_A)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestA>().await?.into_value();
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                actual.set(Some(result));
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        build
                            .send::<LastCiPassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        build
                            .send::<CiFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                            .await?;
                        build.send::<FileCiResultA>(&bytes.as_slice()).await?;
                    }
                }
                inputs = imp::decode(build.recv::<PublicationA>().await?)?;
            }
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected initial publication {label}"
            )));
        }
    }
    if build.recv::<TaskExit>().await? != 0 {
        return Err(ShellError::message("delivery failed"));
    }
    Ok(())
}
