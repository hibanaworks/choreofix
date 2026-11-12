//! CI owns its input data and follows the projected retry branches directly.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CI_SERVICE>,
    build: Endpoint<'r, BUILD_SERVER>,
    domain: &D,
) -> ShellResult<()> {
    endpoint.send::<BuildLoan>(build).await?;
    let mut tests_a = Value::Null;
    let mut tests_b = Value::Null;
    let mut publication = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewApproved::LOGICAL_LABEL => {
                branch.recv::<ReviewApproved>().await?;
            }
            ReviewRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ReviewRejected>().await?)?;
                match result["data"]["target"].as_str() {
                    Some(FEATURE_A) => {
                        tests_a = imp::decode(endpoint.recv::<TestsForCiA>().await?)?;
                    }
                    Some(FEATURE_B) => {
                        tests_b = imp::decode(endpoint.recv::<TestsForCiB>().await?)?;
                    }
                    _ => return Err(ShellError::message("review result names another feature")),
                }
            }
            TestsForCiA::LOGICAL_LABEL => {
                tests_a = imp::decode(branch.recv::<TestsForCiA>().await?)?;
            }
            TestsForCiB::LOGICAL_LABEL => {
                tests_b = imp::decode(branch.recv::<TestsForCiB>().await?)?;
            }
            Merged::LOGICAL_LABEL => break imp::decode(branch.recv::<Merged>().await?)?,
            _ => {
                return Err(ShellError::message(
                    "expected preparation or the selected publication",
                ));
            }
        }
    };
    loop {
        match publication["data"]["feature"].as_str() {
            Some(FEATURE_A) => {
                imp::merged(&publication, FEATURE_A)?;
                let admitted = endpoint.recv::<CiIntentA>().await?.into_value();
                imp::arguments(&admitted, ("suite", SUITE_A))?;
                imp::ci_tests(&publication, (&tests_a, &tests_b))?;
                let (authority, arguments) = async {
                    let [mut feature, mut build] = endpoint.par()?;
                    async {
                        Ok::<_, ShellError>(feature.recv::<RunCiOnFeatureA>().await?.into_value())
                    };
                    async { Ok::<_, ShellError>(build.recv::<RunCiA>().await?.into_value()) };
                }
                .await?;
                imp::same(&authority, &admitted)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, &publication, (&tests_a, &tests_b))?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                let branch = endpoint.offer().await?;
                publication = match branch.label() {
                    LastCiPassedA::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<LastCiPassedA>().await?)?,
                            &result,
                        )?;
                        drop(endpoint.recv::<BuildReturn>().await?);
                        return Ok(());
                    }
                    CiPassedA::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiPassedA>().await?)?, &result)?;
                        imp::decode(endpoint.recv::<Merged>().await?)?
                    }
                    CiFailedA::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiFailedA>().await?)?, &result)?;
                        tests_a = imp::decode(endpoint.recv::<TestsForCiA>().await?)?;
                        loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                RecoveryReviewApproved::LOGICAL_LABEL => {
                                    let review = imp::decode(
                                        branch.recv::<RecoveryReviewApproved>().await?,
                                    )?;
                                    imp::arguments(&review["data"], ("target", FEATURE_A))?;
                                    break;
                                }
                                RecoveryReviewRejected::LOGICAL_LABEL => {
                                    let review = imp::decode(
                                        branch.recv::<RecoveryReviewRejected>().await?,
                                    )?;
                                    imp::arguments(&review["data"], ("target", FEATURE_A))?;
                                    tests_a = imp::decode(endpoint.recv::<TestsForCiA>().await?)?;
                                }
                                _ => {
                                    return Err(ShellError::message(
                                        "expected the recovery review result",
                                    ));
                                }
                            }
                        }
                        imp::decode(endpoint.recv::<Merged>().await?)?
                    }
                    _ => {
                        return Err(ShellError::message(
                            "expected the original feature A CI outcome",
                        ));
                    }
                };
            }
            Some(FEATURE_B) => {
                imp::merged(&publication, FEATURE_B)?;
                let admitted = endpoint.recv::<CiIntentB>().await?.into_value();
                imp::arguments(&admitted, ("suite", SUITE_B))?;
                imp::ci_tests(&publication, (&tests_a, &tests_b))?;
                let (authority, arguments) = async {
                    let [mut feature, mut build] = endpoint.par()?;
                    async {
                        Ok::<_, ShellError>(feature.recv::<RunCiOnFeatureB>().await?.into_value())
                    };
                    async { Ok::<_, ShellError>(build.recv::<RunCiB>().await?.into_value()) };
                }
                .await?;
                imp::same(&authority, &admitted)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("CI_RUNNER", "run_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::ci_result(&result, &publication, (&tests_a, &tests_b))?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                endpoint.send::<CiResult>(&bytes.as_slice()).await?;
                let branch = endpoint.offer().await?;
                publication = match branch.label() {
                    LastCiPassedB::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<LastCiPassedB>().await?)?,
                            &result,
                        )?;
                        drop(endpoint.recv::<BuildReturn>().await?);
                        return Ok(());
                    }
                    CiPassedB::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiPassedB>().await?)?, &result)?;
                        imp::decode(endpoint.recv::<Merged>().await?)?
                    }
                    CiFailedB::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiFailedB>().await?)?, &result)?;
                        tests_b = imp::decode(endpoint.recv::<TestsForCiB>().await?)?;
                        loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                RecoveryReviewApproved::LOGICAL_LABEL => {
                                    let review = imp::decode(
                                        branch.recv::<RecoveryReviewApproved>().await?,
                                    )?;
                                    imp::arguments(&review["data"], ("target", FEATURE_B))?;
                                    break;
                                }
                                RecoveryReviewRejected::LOGICAL_LABEL => {
                                    let review = imp::decode(
                                        branch.recv::<RecoveryReviewRejected>().await?,
                                    )?;
                                    imp::arguments(&review["data"], ("target", FEATURE_B))?;
                                    tests_b = imp::decode(endpoint.recv::<TestsForCiB>().await?)?;
                                }
                                _ => {
                                    return Err(ShellError::message(
                                        "expected the recovery review result",
                                    ));
                                }
                            }
                        }
                        imp::decode(endpoint.recv::<Merged>().await?)?
                    }
                    _ => {
                        return Err(ShellError::message(
                            "expected the original feature B CI outcome",
                        ));
                    }
                };
            }
            _ => {
                return Err(ShellError::message(
                    "expected a publication for a known feature",
                ));
            }
        }
    }
}
