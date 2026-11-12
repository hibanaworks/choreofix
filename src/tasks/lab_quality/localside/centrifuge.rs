//! The owner receives actual requests and retains the resulting laboratory data.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CENT_SERVICE>,
    (mut instrument, candidate, actual): (
        Endpoint<'r, CENTRIFUGE>,
        Endpoint<'r, BIOLOGY>,
        &Cell<Value>,
    ),
    domain: &D,
) -> ShellResult<()> {
    let preparation = {
        let admitted = imp::decode(endpoint.recv::<Preparation>().await?)?;
        imp::assigned(&admitted, "material", "cell_cultures")?;
        endpoint.send::<CentLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_separation", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<CentLoan>().await?;
        result
    };
    let purification = {
        let admitted = imp::decode(endpoint.recv::<Purification>().await?)?;
        imp::assigned(&admitted, "material", "compound_purification")?;
        endpoint.send::<CentLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_separation", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<CentLoan>().await?;
        endpoint
            .send::<PurificationProof>(&bytes.as_slice())
            .await?;
        result
    };
    let calibration = imp::decode(endpoint.recv::<CalibrationProof>().await?)?;
    endpoint.send::<BioAccess>(candidate).await?;
    let mut biology = loop {
        let request = imp::decode(endpoint.recv::<BioRequest>().await?)?;
        imp::assay_arguments(&request["arguments"], &purification["data"]["compound"])?;
        imp::same(&request["purification"], &purification)?;
        imp::same(&request["preparation"], &preparation)?;
        imp::same(&request["calibration"], &calibration)?;
        let candidate = endpoint.recv::<BioAccess>().await?;
        let result = domain
            .execute("BIOLOGIST", "run_bioassay", request["arguments"].clone())
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        match imp::decide(actual).map_err(|_| ShellError::message("missing actual trial result"))? {
            hibana::runtime::resolver::DecisionArm::Left => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                endpoint.send::<BioPassed>(&reply).await?;
                async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async {
                        fume.send::<BioFumePassed>(&reply).await?;
                    };
                    async {
                        spec.send::<BioSpecPassed>(&reply).await?;
                    };
                    Ok(())
                }
                .await?;
                endpoint.send::<BioAccess>(candidate).await?;
                break result;
            }
            hibana::runtime::resolver::DecisionArm::Right => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                endpoint.send::<BioFailed>(&reply).await?;
                async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async {
                        fume.send::<BioFumeFailed>(&reply).await?;
                    };
                    async {
                        spec.send::<BioSpecFailed>(&reply).await?;
                    };
                    Ok(())
                }
                .await?;
                endpoint.send::<BioAccess>(candidate).await?;
            }
        }
    };
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            80 => {
                branch.recv::<ChemistApproved>().await?;
            }
            81 => {
                branch.recv::<ChemistRejected>().await?;
            }
            82 => {
                branch.recv::<BiologistApproved>().await?;
            }
            83 => {
                branch.recv::<BiologistRejected>().await?;
            }
            86 => {
                branch.recv::<LastChemistRejected>().await?;
            }
            87 => {
                branch.recv::<LastBiologistRejected>().await?;
            }
            84 => {
                branch.recv::<LastChemistApproved>().await?;
                break;
            }
            85 => {
                branch.recv::<LastBiologistApproved>().await?;
                break;
            }
            100 => {
                let request = imp::decode(branch.recv::<Retest>().await?)?;
                imp::assay_arguments(&request["arguments"], &purification["data"]["compound"])?;
                imp::same(&request["purification"], &purification)?;
                imp::same(&request["preparation"], &preparation)?;
                imp::same(&request["calibration"], &calibration)?;
                let candidate = endpoint.recv::<BioAccess>().await?;
                let result = domain
                    .execute("BIOLOGIST", "run_bioassay", request["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                let bytes = imp::encode(&result)?;
                actual.set(result.clone());
                match imp::decide(actual)
                    .map_err(|_| ShellError::message("missing actual trial result"))?
                {
                    hibana::runtime::resolver::DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                        endpoint.send::<RetestPassed>(&reply).await?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<RetestFumePassed>(&reply).await?;
                            };
                            async {
                                spec.send::<RetestSpecPassed>(&reply).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    hibana::runtime::resolver::DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                        endpoint.send::<RetestFailed>(&reply).await?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<RetestFumeFailed>(&reply).await?;
                            };
                            async {
                                spec.send::<RetestSpecFailed>(&reply).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                }
                biology = result;
            }
            110 => {
                let request = imp::decode(branch.recv::<LastRetest>().await?)?;
                imp::assay_arguments(&request["arguments"], &purification["data"]["compound"])?;
                imp::same(&request["purification"], &purification)?;
                imp::same(&request["preparation"], &preparation)?;
                imp::same(&request["calibration"], &calibration)?;
                let candidate = endpoint.recv::<BioAccess>().await?;
                let result = domain
                    .execute("BIOLOGIST", "run_bioassay", request["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                let bytes = imp::encode(&result)?;
                actual.set(result.clone());
                match imp::decide(actual)
                    .map_err(|_| ShellError::message("missing actual trial result"))?
                {
                    hibana::runtime::resolver::DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                        endpoint.send::<LastRetestPassed>(&reply).await?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<LastRetestFumePassed>(&reply).await?;
                            };
                            async {
                                spec.send::<LastRetestSpecPassed>(&reply).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    hibana::runtime::resolver::DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                        endpoint.send::<LastRetestFailed>(&reply).await?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<LastRetestFumeFailed>(&reply).await?;
                            };
                            async {
                                spec.send::<LastRetestSpecFailed>(&reply).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                }
                biology = result;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected centrifuge operation {label}"
                )));
            }
        }
    }
    let admitted = imp::decode(endpoint.recv::<Separation>().await?)?;
    imp::assigned(&admitted["arguments"], "material", "verification")?;
    imp::same(&admitted["biology"], &biology)?;
    endpoint.send::<CentLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
    imp::same(&request, &admitted)?;
    let result = domain
        .execute("ANALYST", "run_separation", request["arguments"].clone())
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<CentLoan>().await?;
    endpoint
        .send::<VerificationProof>(&bytes.as_slice())
        .await?;
    drop(instrument);
    Ok(())
}
