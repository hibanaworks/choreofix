//! Actual spectrometer data, its affine loans and result-bound final validation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run<D: Domain>(endpoint: &mut Endpoint<SPEC_SERVICE>, domain: &D) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<SpecLoan>().await?;
    let admitted = imp::decode(endpoint.recv::<InitialAnalysis>().await?)?;
    let arguments = imp::arguments::<1>(&admitted["arguments"])?;
    let synthesis = admitted["synthesis"].clone();
    let synthesis_id = imp::synthesis_id(&synthesis)?;
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let mut analysis = domain
        .execute("CHEMIST", "run_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(
        &analysis,
        ("run_analysis", "CHEMIST", &arguments),
        &json!({"synthesis":synthesis_id}),
    )?;
    let initial_analysis = analysis.clone();
    let bytes = imp::encode(&analysis)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<SpecLoan>().await?;
    endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
    let admitted = imp::decode(endpoint.recv::<Calibration>().await?)?;
    let arguments = imp::arguments::<4>(&admitted["arguments"])?;
    let cells = imp::record(
        &admitted["cells"],
        (
            "run_separation",
            "BIOLOGIST",
            &json!({"material":"cell_cultures"}),
        ),
        &json!({}),
    )?;
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let calibration = domain
        .execute("BIOLOGIST", "run_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(
        &calibration,
        ("run_analysis", "BIOLOGIST", &arguments),
        &json!({"prepared_cells":cells}),
    )?;
    let bytes = imp::encode(&calibration)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<SpecLoan>().await?;
    endpoint.send::<CalibrationProof>(&bytes.as_slice()).await?;
    let mut biology = Value::Null;
    let first_approval = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            FirstChemist::LOGICAL_LABEL => {
                let selected = imp::decode(branch.recv::<FirstChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
                let (observed, returned) = async {
                    let [mut trials, mut qc] = endpoint.par()?;
                    async {
                        let observed = loop {
                            let branch = trials.offer().await?;
                            match branch.label() {
                                BioSpecPassed::LOGICAL_LABEL => {
                                    break imp::biological_proof(
                                        branch.recv::<BioSpecPassed>().await?,
                                    )?;
                                }
                                BioSpecFailed::LOGICAL_LABEL => {
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
            FirstBiologist::LOGICAL_LABEL => {
                let selected = imp::decode(branch.recv::<FirstBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;
                let observed = loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        BioSpecPassed::LOGICAL_LABEL => {
                            break imp::biological_proof(branch.recv::<BioSpecPassed>().await?)?;
                        }
                        BioSpecFailed::LOGICAL_LABEL => {
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
            NextChemist::LOGICAL_LABEL | NextBiologist::LOGICAL_LABEL => {
                let (selected, scientist) = match branch.label() {
                    NextChemist::LOGICAL_LABEL => {
                        (imp::decode(branch.recv::<NextChemist>().await?)?, "CHEMIST")
                    }
                    NextBiologist::LOGICAL_LABEL => (
                        imp::decode(branch.recv::<NextBiologist>().await?)?,
                        "BIOLOGIST",
                    ),
                    _ => unreachable!(),
                };
                imp::qc_arguments(&selected, scientist)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
            }
            Reanalysis::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Reanalysis>().await?)?;
                let arguments = imp::arguments::<1>(&admitted["arguments"])?;
                imp::same_value(&admitted["synthesis"], &synthesis)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
                imp::same_value(&request, &admitted)?;
                let result = domain
                    .execute("CHEMIST", "run_analysis", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", &arguments),
                    &serde_json::json!({"synthesis":synthesis_id}),
                )?;
                analysis = result.clone();
                let bytes = imp::encode(&result)?;
                endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
                endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
            }
            ChemistApproved::LOGICAL_LABEL => {
                break imp::decode(branch.recv::<ChemistApproved>().await?)?;
            }
            BiologistApproved::LOGICAL_LABEL => {
                break imp::decode(branch.recv::<BiologistApproved>().await?)?;
            }
            ChemistRejected::LOGICAL_LABEL => {
                branch.recv::<ChemistRejected>().await?;
            }
            BiologistRejected::LOGICAL_LABEL => {
                branch.recv::<BiologistRejected>().await?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    NextChemist::LOGICAL_LABEL => {
                        let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                        imp::qc_arguments(&selected, "CHEMIST")?;
                        let (observed, returned) = async {
                            let [mut trials, mut qc] = endpoint.par()?;
                            async {
                                let observed = loop {
                                    let branch = trials.offer().await?;
                                    match branch.label() {
                                        RetestSpecPassed::LOGICAL_LABEL => {
                                            break imp::biological_proof(
                                                branch.recv::<RetestSpecPassed>().await?,
                                            )?;
                                        }
                                        RetestSpecFailed::LOGICAL_LABEL => {
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
                    NextBiologist::LOGICAL_LABEL => {
                        let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                        imp::qc_arguments(&selected, "BIOLOGIST")?;
                        let observed = loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                RetestSpecPassed::LOGICAL_LABEL => {
                                    break imp::biological_proof(
                                        branch.recv::<RetestSpecPassed>().await?,
                                    )?;
                                }
                                RetestSpecFailed::LOGICAL_LABEL => {
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
            LastRetestSpecPassed::LOGICAL_LABEL => {
                biology = imp::biological_proof(branch.recv::<LastRetestSpecPassed>().await?)?;
            }
            LastRetestSpecFailed::LOGICAL_LABEL => {
                biology = imp::biological_proof(branch.recv::<LastRetestSpecFailed>().await?)?;
            }
            LastChemist::LOGICAL_LABEL | LastBiologist::LOGICAL_LABEL => {
                let (selected, scientist) = match branch.label() {
                    LastChemist::LOGICAL_LABEL => {
                        (imp::decode(branch.recv::<LastChemist>().await?)?, "CHEMIST")
                    }
                    LastBiologist::LOGICAL_LABEL => (
                        imp::decode(branch.recv::<LastBiologist>().await?)?,
                        "BIOLOGIST",
                    ),
                    _ => unreachable!(),
                };
                imp::qc_arguments(&selected, scientist)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
            }
            LastReanalysis::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<LastReanalysis>().await?)?;
                let arguments = imp::arguments::<1>(&admitted["arguments"])?;
                imp::same_value(&admitted["synthesis"], &synthesis)?;
                endpoint.send::<SpecLoan>(instrument).await?;
                let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
                imp::same_value(&request, &admitted)?;
                let result = domain
                    .execute("CHEMIST", "run_analysis", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", &arguments),
                    &serde_json::json!({"synthesis":synthesis_id}),
                )?;
                analysis = result.clone();
                let bytes = imp::encode(&result)?;
                endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<SpecLoan>().await?;
                endpoint.send::<AnalysisProof>(&bytes.as_slice()).await?;
            }
            LastChemistApproved::LOGICAL_LABEL => {
                break imp::decode(branch.recv::<LastChemistApproved>().await?)?;
            }
            LastBiologistApproved::LOGICAL_LABEL => {
                break imp::decode(branch.recv::<LastBiologistApproved>().await?)?;
            }
            LastChemistRejected::LOGICAL_LABEL => {
                branch.recv::<LastChemistRejected>().await?;
            }
            LastBiologistRejected::LOGICAL_LABEL => {
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

    if biology["result"]["success"] != true {
        return Err(ShellError::message(
            "quality lacks an actual successful biological trial",
        ));
    }
    imp::biological_proof(&imp::encode(&biology)?)?;
    let admitted = imp::decode(endpoint.recv::<Molecular>().await?)?;
    let arguments = imp::arguments::<6>(&admitted["arguments"])?;
    let approved = &admitted["approved"];
    let inputs = imp::approved_inputs(approved)?;
    imp::same_value(&approved["chemist_qc"], &chemist_qc)?;
    imp::same_value(&approved["biologist_qc"], &biologist_qc)?;
    imp::same_value(&approved["materials"]["synthesis"], &synthesis)?;
    imp::same_value(
        &approved["materials"]["initial_analysis"],
        &initial_analysis,
    )?;
    imp::same_value(&approved["materials"]["analysis"], &analysis)?;
    imp::same_value(&approved["materials"]["calibration"], &calibration)?;
    for key in ["purification", "cells", "bioassay"] {
        imp::same_value(&approved["materials"][key], &biology["materials"][key])?;
    }
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let molecular = domain
        .execute("ANALYST", "run_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(&molecular, ("run_analysis", "ANALYST", &arguments), &inputs)?;
    let bytes = imp::encode(&molecular)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<SpecLoan>().await?;
    let (verification, quality) = async {
        let [mut verification, mut quality] = endpoint.par()?;
        async { imp::decode(verification.recv::<VerificationProof>().await?) };
        async { imp::decode(quality.recv::<QualityForOwner>().await?) };
    }
    .await?;
    imp::same_value(&quality["chemist_qc"], &chemist_qc)?;
    imp::same_value(&quality["biologist_qc"], &biologist_qc)?;
    imp::same_value(&quality["materials"], &approved["materials"])?;
    imp::same_value(&quality["molecular"], &molecular)?;
    imp::same_value(&quality["verification"], &verification)?;
    let inputs = imp::final_inputs(&quality)?;
    let admitted = imp::decode(endpoint.recv::<FinalValidation>().await?)?;
    let arguments = imp::arguments::<8>(&admitted["arguments"])?;
    imp::same_value(&admitted["quality"], &quality)?;
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let result = domain
        .execute("CHEMIST", "run_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(&result, ("run_analysis", "CHEMIST", &arguments), &inputs)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<SpecLoan>().await?;
    endpoint.send::<SpecLoan>(instrument).await?;
    Ok(())
}
