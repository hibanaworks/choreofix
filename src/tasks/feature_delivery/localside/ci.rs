//! The CI service receives publications, computes actual results and hands them to review.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(
    build: &mut Endpoint<BUILD_SERVER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    loop {
        let branch = build.offer().await?;
        match branch.label() {
            PublicationA::LOGICAL_LABEL => {
                let inputs = imp::decode(branch.recv::<PublicationA>().await?)?;
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_A)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationA>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestA>().await?;
                let result = effects.apply(arguments).await?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                let admitted = match result["success"].as_bool() {
                    Some(true) => {
                        build
                            .send::<CiPassedA>(
                                &crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(
                                    &bytes,
                                )?,
                            )
                            .await?;
                        build.send::<CiResultA>(&bytes.as_slice()).await?;
                        build.recv::<CiPassedA>().await?.to_vec()
                    }
                    Some(false) => {
                        build
                            .send::<CiFailedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                            .await?;
                        build.send::<CiResultA>(&bytes.as_slice()).await?;
                        build.recv::<CiFailedA>().await?.to_vec()
                    }
                    None => return Err(ShellError::message("missing checked CI outcome")),
                };
                if admitted != bytes {
                    return Err(ShellError::message(
                        "CI admission changed the actual result",
                    ));
                }
                build.send::<FileCiResultA>(&admitted.as_slice()).await?;
            }
            PublicationB::LOGICAL_LABEL => {
                let inputs = imp::decode(branch.recv::<PublicationB>().await?)?;
                let publication = &inputs["publication"];
                let tests = (&inputs["tests"][0], &inputs["tests"][1]);
                imp::merged(publication, FEATURE_B)?;
                imp::ci_tests(publication, tests)?;
                let bytes = imp::encode(publication)?;
                build.send::<FilePublicationB>(&bytes.as_slice()).await?;
                let arguments = build.recv::<CiRequestB>().await?;
                let result = effects.apply(arguments).await?;
                imp::ci_result(&result, publication, tests)?;
                let bytes = imp::encode(&result)?;
                let admitted = match result["success"].as_bool() {
                    Some(true) => {
                        build
                            .send::<CiPassedB>(
                                &crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(
                                    &bytes,
                                )?,
                            )
                            .await?;
                        build.send::<CiResultB>(&bytes.as_slice()).await?;
                        build.recv::<CiPassedB>().await?.to_vec()
                    }
                    Some(false) => {
                        build
                            .send::<CiFailedB>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                            .await?;
                        build.send::<CiResultB>(&bytes.as_slice()).await?;
                        build.recv::<CiFailedB>().await?.to_vec()
                    }
                    None => return Err(ShellError::message("missing checked CI outcome")),
                };
                if admitted != bytes {
                    return Err(ShellError::message(
                        "CI admission changed the actual result",
                    ));
                }
                build.send::<FileCiResultB>(&admitted.as_slice()).await?;
            }
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("delivery failed"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected CI operation {label}"
                )));
            }
        }
    }
}
