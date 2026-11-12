//! Actual spectrometer measurements and the physical QC instrument's custody.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, SPEC_SERVICE>,
    mut instrument: Endpoint<'r, SPECTROMETER>,
    domain: &D,
) -> ShellResult<()> {
    let analysis = {
        let admitted = imp::decode(endpoint.recv::<InitialAnalysis>().await?)?;
        imp::assigned(&admitted, "sample", "synthesized_compound")?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
        result
    };
    let calibration = {
        let admitted = imp::decode(endpoint.recv::<Calibration>().await?)?;
        imp::assigned(&admitted, "sample", "fluorescence")?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        endpoint.send::<CalibrationProof>(&bytes.as_slice()).await?;
        result
    };
    let mut biology = Value::Null;
    let first_approval = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            50 => {
                let selected = imp::decode(branch.recv::<FirstChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
                let (observed, returned) = async {
                    let [mut trials, mut qc] = endpoint.par()?;
                    async {
                        let observed = loop {
                            let branch = trials.offer().await?;
                            match branch.label() {
                                37 => break imp::decode(branch.recv::<BioSpecPassed>().await?)?,
                                38 => {
                                    branch.recv::<BioSpecFailed>().await?;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected biological observation {label}"
                                    )));
                                }
                            }
                        };
                        Ok(observed)
                    };
                    async {
                        qc.send::<SpecLoan>(instrument).await?;
                        Ok(qc.recv::<SpecLoan>().await?)
                    };
                }
                .await?;
                biology = observed;
                instrument = returned;
            }
            51 => {
                let selected = imp::decode(branch.recv::<FirstBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;
                let observed = loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        37 => break imp::decode(branch.recv::<BioSpecPassed>().await?)?,
                        38 => {
                            branch.recv::<BioSpecFailed>().await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected biological observation {label}"
                            )));
                        }
                    }
                };
                biology = observed;
                endpoint.send::<SpecLoan>(instrument).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
            }
            52 | 53 => {
                let (selected, scientist) = match branch.label() {
                    52 => (imp::decode(branch.recv::<NextChemist>().await?)?, "CHEMIST"),
                    53 => (
                        imp::decode(branch.recv::<NextBiologist>().await?)?,
                        "BIOLOGIST",
                    ),
                    _ => unreachable!(),
                };
                imp::qc_arguments(&selected, scientist)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
            }
            15 => {
                let admitted = imp::decode(branch.recv::<Reanalysis>().await?)?;
                imp::assigned(&admitted, "sample", "synthesized_compound")?;
                endpoint.send::<SpecLoan>(instrument).await?;
                let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
                imp::same(&request, &admitted)?;
                let result = domain
                    .execute("CHEMIST", "run_analysis", request)
                    .await
                    .map_err(ShellError::message)?;
                imp::successful(&result)?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
                endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
            }
            80 => {
                break imp::decode(branch.recv::<ChemistApproved>().await?)?;
            }
            82 => {
                break imp::decode(branch.recv::<BiologistApproved>().await?)?;
            }
            81 => {
                branch.recv::<ChemistRejected>().await?;
            }
            83 => {
                branch.recv::<BiologistRejected>().await?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    52 => {
                        let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                        imp::qc_arguments(&selected, "CHEMIST")?;
                        let (observed, returned) = async {
                            let [mut trials, mut qc] = endpoint.par()?;
                            async {
                                let observed = loop {
                                    let branch = trials.offer().await?;
                                    match branch.label() {
                                        105 => {
                                            break imp::decode(
                                                branch.recv::<RetestSpecPassed>().await?,
                                            )?;
                                        }
                                        106 => {
                                            branch.recv::<RetestSpecFailed>().await?;
                                        }
                                        label => {
                                            return Err(ShellError::message(format!(
                                                "unexpected biological observation {label}"
                                            )));
                                        }
                                    }
                                };
                                Ok(observed)
                            };
                            async {
                                qc.send::<SpecLoan>(instrument).await?;
                                Ok(qc.recv::<SpecLoan>().await?)
                            };
                        }
                        .await?;
                        biology = observed;
                        instrument = returned;
                    }
                    53 => {
                        let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                        imp::qc_arguments(&selected, "BIOLOGIST")?;
                        let observed = loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                105 => {
                                    break imp::decode(branch.recv::<RetestSpecPassed>().await?)?;
                                }
                                106 => {
                                    branch.recv::<RetestSpecFailed>().await?;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected biological observation {label}"
                                    )));
                                }
                            }
                        };
                        biology = observed;
                        endpoint.send::<SpecLoan>(instrument).await?;
                        instrument = endpoint.recv::<SpecLoan>().await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected QC selection after biological rejection {label}"
                        )));
                    }
                }
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected spectrometer operation {label}"
                )));
            }
        }
    };
    let last_approval = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            115 => {
                biology = imp::decode(branch.recv::<LastRetestSpecPassed>().await?)?;
            }
            116 => {
                biology = imp::decode(branch.recv::<LastRetestSpecFailed>().await?)?;
            }
            54 | 55 => {
                let (selected, scientist) = match branch.label() {
                    54 => (imp::decode(branch.recv::<LastChemist>().await?)?, "CHEMIST"),
                    55 => (
                        imp::decode(branch.recv::<LastBiologist>().await?)?,
                        "BIOLOGIST",
                    ),
                    _ => unreachable!(),
                };
                imp::qc_arguments(&selected, scientist)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
            }
            117 => {
                let admitted = imp::decode(branch.recv::<LastReanalysis>().await?)?;
                imp::assigned(&admitted, "sample", "synthesized_compound")?;
                endpoint.send::<SpecLoan>(instrument).await?;
                let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
                imp::same(&request, &admitted)?;
                let result = domain
                    .execute("CHEMIST", "run_analysis", request)
                    .await
                    .map_err(ShellError::message)?;
                imp::successful(&result)?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
                endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
            }
            84 => {
                break imp::decode(branch.recv::<LastChemistApproved>().await?)?;
            }
            85 => {
                break imp::decode(branch.recv::<LastBiologistApproved>().await?)?;
            }
            86 => {
                branch.recv::<LastChemistRejected>().await?;
            }
            87 => {
                branch.recv::<LastBiologistRejected>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected final spectrometer operation {label}"
                )));
            }
        }
    };
    let (chemist_qc, biologist_qc) = match first_approval["data"]["scientist"].as_str() {
        Some("CHEMIST") => (first_approval, last_approval),
        Some("BIOLOGIST") => (last_approval, first_approval),
        _ => {
            return Err(ShellError::message(
                "actual QC approval has no scientific subject",
            ));
        }
    };
    imp::successful(&biology)?;
    let molecular = {
        let admitted = imp::decode(endpoint.recv::<Molecular>().await?)?;
        imp::assigned(&admitted["arguments"], "sample", "molecular_detail")?;
        imp::same(&admitted["biology"], &biology)?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("ANALYST", "run_analysis", request["arguments"].clone())
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        result
    };
    let (verification, quality) = async {
        let [mut separated, mut report] = endpoint.par()?;
        async { imp::decode(separated.recv::<VerificationProof>().await?) };
        async { imp::decode(report.recv::<QualityForOwner>().await?) };
    }
    .await?;
    imp::same(&quality["biology"]["result"], &biology)?;
    imp::same(&quality["biology"]["calibration"], &calibration)?;
    imp::same(&quality["biology"]["qc"]["chemist"], &chemist_qc)?;
    imp::same(&quality["biology"]["qc"]["biologist"], &biologist_qc)?;
    imp::same(&quality["biology"]["sample"]["analysis"], &analysis)?;
    imp::same(&quality["molecular"], &molecular)?;
    imp::same(&quality["separation"], &verification)?;
    let admitted = imp::decode(endpoint.recv::<FinalValidation>().await?)?;
    imp::assigned(&admitted["arguments"], "sample", "final_validation")?;
    imp::same(&admitted["quality"], &quality)?;
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same(&request, &admitted)?;
    let result = domain
        .execute("CHEMIST", "run_analysis", request["arguments"].clone())
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<SpecLoan>().await?;
    drop(instrument);
    Ok(())
}
