//! Actual cell/material data and candidate custody drive every biological retry.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run(biologist: &mut Endpoint<BIOLOGIST>) -> ShellResult<()> {
    let arguments = biologist.recv::<FileCells>().await?.into_value();
    let bytes = imp::encode(&arguments)?;
    biologist.send::<Preparation>(&bytes.as_slice()).await?;
    let mut cent = biologist.recv::<CentLoan>().await?;
    cent.send::<CentRequest>(&bytes.as_slice()).await?;
    let cells = imp::decode(cent.recv::<InstrumentResult>().await?)?;
    let cells_id = imp::record(
        &cells,
        ("run_separation", "BIOLOGIST", &arguments),
        &json!({}),
    )?;
    biologist.send::<CentLoan>(cent).await?;
    let bytes = imp::encode(&cells)?;
    biologist.send::<FileCellsResult>(&bytes.as_slice()).await?;
    let arguments = biologist.recv::<FileCalibration>().await?.into_value();
    let bytes = imp::encode(&json!({"arguments":arguments,"cells":cells}))?;
    biologist.send::<Calibration>(&bytes.as_slice()).await?;
    let mut spec = biologist.recv::<SpecLoan>().await?;
    spec.send::<SpecRequest>(&bytes.as_slice()).await?;
    let calibration = imp::decode(spec.recv::<InstrumentResult>().await?)?;
    imp::record(
        &calibration,
        ("run_analysis", "BIOLOGIST", &arguments),
        &json!({"prepared_cells":cells_id}),
    )?;
    biologist.send::<SpecLoan>(spec).await?;
    let bytes = imp::encode(&calibration)?;
    biologist
        .send::<FileCalibrationResult>(&bytes.as_slice())
        .await?;
    let sample = imp::decode(biologist.recv::<Sample>().await?)?;
    imp::specimen(&sample)?;
    let bytes = imp::encode(&sample)?;
    biologist.send::<FileSample>(&bytes.as_slice()).await?;
    let mut materials = sample;
    materials["cells"] = cells;
    materials["calibration"] = calibration;
    let mut biology = biologist.recv::<BioAccess>().await?;
    let branch = biologist.offer().await?;
    let requested = match branch.label() {
        FirstChemist::LOGICAL_LABEL => {
            let args = imp::decode(branch.recv::<FirstChemist>().await?)?;
            let bytes = imp::encode(&args)?;
            super::super::arguments::Arguments::<9>::try_from(bytes.as_slice())?;
            args
        }
        FirstBiologist::LOGICAL_LABEL => {
            let args = imp::decode(branch.recv::<FirstBiologist>().await?)?;
            let bytes = imp::encode(&args)?;
            super::super::arguments::Arguments::<10>::try_from(bytes.as_slice())?;
            args
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected requested scientific QC {label}"
            )));
        }
    };
    let result = loop {
        let arguments = biology.recv::<FileBioassay>().await?.into_value();
        let bytes = imp::encode(&json!({"arguments":arguments,"materials":materials}))?;
        biologist.send::<BioRequest>(&bytes.as_slice()).await?;
        biologist.send::<BioAccess>(biology).await?;
        let branch = biologist.offer().await?;
        let result = match branch.label() {
            BioPassed::LOGICAL_LABEL => imp::decode(branch.recv::<BioPassed>().await?)?,
            BioFailed::LOGICAL_LABEL => imp::decode(branch.recv::<BioFailed>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected biological result {label}"
                )));
            }
        };
        imp::bio_result(&result, &arguments, &materials)?;
        let bytes = imp::encode(&result)?;
        biologist
            .send::<FileBioassayResult>(&bytes.as_slice())
            .await?;
        if result["success"] == true {
            break result;
        }
        biology = biologist.recv::<BioAccess>().await?;
    };
    materials["bioassay"] = result;

    if requested["scientist"] == "BIOLOGIST" {
        let bytes = imp::encode(&materials)?;
        biologist.send::<BiologyReport>(&bytes.as_slice()).await?;
    }
    let qc_arguments = json!({"scientist":"BIOLOGIST","data_type":"bioassay"});
    let mut approved_biology = Value::Null;
    loop {
        let branch = biologist.offer().await?;
        match branch.label() {
            FirstChemist::LOGICAL_LABEL => {
                branch.recv::<FirstChemist>().await?;
            }
            NextChemist::LOGICAL_LABEL => {
                branch.recv::<NextChemist>().await?;
            }
            FirstBiologist::LOGICAL_LABEL
            | NextBiologist::LOGICAL_LABEL
            | LastBiologist::LOGICAL_LABEL => {
                let selected = match branch.label() {
                    FirstBiologist::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<FirstBiologist>().await?)?
                    }
                    NextBiologist::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<NextBiologist>().await?)?
                    }
                    _ => imp::decode(branch.recv::<LastBiologist>().await?)?,
                };
                if selected != qc_arguments {
                    return Err(ShellError::message(
                        "QC requested another actual biological subject",
                    ));
                }
                let bytes = imp::encode(&materials)?;
                biologist.send::<BiologyReport>(&bytes.as_slice()).await?;
            }
            BiologistRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<BiologistRejected>().await?)?;
                imp::qc_result(
                    &result,
                    &qc_arguments,
                    &imp::qc_inputs(&qc_arguments, &materials)?,
                )?;
                let mut biology = biologist.recv::<BioAccess>().await?;
                let bytes = imp::encode(&result)?;
                biologist.send::<FileFeedback>(&bytes.as_slice()).await?;
                let branch = biologist.offer().await?;
                let requested = match branch.label() {
                    NextChemist::LOGICAL_LABEL => {
                        let args = imp::decode(branch.recv::<NextChemist>().await?)?;
                        let bytes = imp::encode(&args)?;
                        super::super::arguments::Arguments::<9>::try_from(bytes.as_slice())?;
                        args
                    }
                    NextBiologist::LOGICAL_LABEL => {
                        let args = imp::decode(branch.recv::<NextBiologist>().await?)?;
                        let bytes = imp::encode(&args)?;
                        super::super::arguments::Arguments::<10>::try_from(bytes.as_slice())?;
                        args
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected requested scientific QC {label}"
                        )));
                    }
                };
                let result = loop {
                    let arguments = biology.recv::<FileBioassay>().await?.into_value();
                    let bytes = imp::encode(&json!({"arguments":arguments,"materials":materials}))?;
                    biologist.send::<Retest>(&bytes.as_slice()).await?;
                    biologist.send::<BioAccess>(biology).await?;
                    let branch = biologist.offer().await?;
                    let result = match branch.label() {
                        RetestPassed::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RetestPassed>().await?)?
                        }
                        RetestFailed::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RetestFailed>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected biological result {label}"
                            )));
                        }
                    };
                    imp::bio_result(&result, &arguments, &materials)?;
                    let bytes = imp::encode(&result)?;
                    biologist
                        .send::<FileBioassayResult>(&bytes.as_slice())
                        .await?;
                    if result["success"] == true {
                        break result;
                    }
                    biology = biologist.recv::<BioAccess>().await?;
                };
                materials["bioassay"] = result;

                if requested["scientist"] == "BIOLOGIST" {
                    let bytes = imp::encode(&materials)?;
                    biologist.send::<BiologyReport>(&bytes.as_slice()).await?;
                }
            }
            LastBiologistRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<LastBiologistRejected>().await?)?;
                imp::qc_result(
                    &result,
                    &qc_arguments,
                    &imp::qc_inputs(&qc_arguments, &materials)?,
                )?;
                let mut biology = biologist.recv::<BioAccess>().await?;
                let bytes = imp::encode(&result)?;
                biologist.send::<FileFeedback>(&bytes.as_slice()).await?;
                let result = loop {
                    let arguments = biology.recv::<FileBioassay>().await?.into_value();
                    let bytes = imp::encode(&json!({"arguments":arguments,"materials":materials}))?;
                    biologist.send::<LastRetest>(&bytes.as_slice()).await?;
                    biologist.send::<BioAccess>(biology).await?;
                    let branch = biologist.offer().await?;
                    let result = match branch.label() {
                        LastRetestPassed::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<LastRetestPassed>().await?)?
                        }
                        LastRetestFailed::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<LastRetestFailed>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected biological result {label}"
                            )));
                        }
                    };
                    imp::bio_result(&result, &arguments, &materials)?;
                    let bytes = imp::encode(&result)?;
                    biologist
                        .send::<FileBioassayResult>(&bytes.as_slice())
                        .await?;
                    if result["success"] == true {
                        break result;
                    }
                    biology = biologist.recv::<BioAccess>().await?;
                };
                materials["bioassay"] = result;
            }
            BiologistApproved::LOGICAL_LABEL | LastBiologistApproved::LOGICAL_LABEL => {
                let result = if branch.label() == BiologistApproved::LOGICAL_LABEL {
                    imp::decode(branch.recv::<BiologistApproved>().await?)?
                } else {
                    imp::decode(branch.recv::<LastBiologistApproved>().await?)?
                };
                imp::qc_result(
                    &result,
                    &qc_arguments,
                    &imp::qc_inputs(&qc_arguments, &materials)?,
                )?;
                let bytes = imp::encode(&result)?;
                biologist.send::<FileFeedback>(&bytes.as_slice()).await?;
                approved_biology = result;
            }
            ChemistApproved::LOGICAL_LABEL => {
                branch.recv::<ChemistApproved>().await?;
            }
            ChemistRejected::LOGICAL_LABEL => {
                branch.recv::<ChemistRejected>().await?;
            }
            LastChemistApproved::LOGICAL_LABEL => {
                branch.recv::<LastChemistApproved>().await?;
            }
            LastChemistRejected::LOGICAL_LABEL => {
                branch.recv::<LastChemistRejected>().await?;
            }
            ApprovedResults::LOGICAL_LABEL => {
                let mut approved = imp::decode(branch.recv::<ApprovedResults>().await?)?;
                if approved["biologist_qc"] != approved_biology {
                    return Err(ShellError::message(
                        "final materials substitute the actual biological approval",
                    ));
                }
                for key in ["synthesis", "initial_analysis", "purification"] {
                    if approved["materials"][key] != materials[key] {
                        return Err(ShellError::message(
                            "final materials describe another actual biological specimen",
                        ));
                    }
                }
                for key in ["cells", "calibration", "bioassay"] {
                    approved["materials"][key] = materials[key].clone();
                }
                imp::approved_inputs(&approved)?;
                let bytes = imp::encode(&approved)?;
                biologist.send::<ApprovedBiology>(&bytes.as_slice()).await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected biological communication {label}"
                )));
            }
        }
    }
}
