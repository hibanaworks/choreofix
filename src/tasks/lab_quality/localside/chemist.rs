//! The scientific candidate moves to QC; only an actual rejection returns editing.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(chemist: &mut Endpoint<CHEMIST>) -> ShellResult<()> {
    let mut chemistry = chemist.recv::<ChemAccess>().await?;
    let synthesis_args = chemistry.recv::<FileSynthesis>().await?.into_value();
    let bytes = imp::encode(&synthesis_args)?;
    chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
    let mut fume = chemistry.recv::<FumeLoan>().await?;
    fume.send::<SynthesisRequest>(&bytes.as_slice()).await?;
    let synthesis = imp::decode(fume.recv::<InstrumentResult>().await?)?;
    let synthesis_id = imp::record(
        &synthesis,
        ("run_synthesis", "CHEMIST", &synthesis_args),
        &json!({}),
    )?;
    chemistry.send::<FumeLoan>(fume).await?;
    let bytes = imp::encode(&synthesis)?;
    chemistry
        .send::<FileSynthesisResult>(&bytes.as_slice())
        .await?;

    let arguments = chemistry.recv::<FileAnalysis>().await?.into_value();
    let bytes = imp::encode(&json!({"arguments":arguments,"synthesis":synthesis}))?;
    chemistry.send::<InitialAnalysis>(&bytes.as_slice()).await?;
    let mut spec = chemistry.recv::<SpecLoan>().await?;
    spec.send::<SpecRequest>(&bytes.as_slice()).await?;
    let initial_analysis = imp::decode(spec.recv::<InstrumentResult>().await?)?;
    let analysis_id = imp::record(
        &initial_analysis,
        ("run_analysis", "CHEMIST", &arguments),
        &json!({"synthesis":synthesis_id}),
    )?;
    chemistry.send::<SpecLoan>(spec).await?;
    let bytes = imp::encode(&initial_analysis)?;
    chemistry
        .send::<FileAnalysisResult>(&bytes.as_slice())
        .await?;

    let arguments = chemistry.recv::<FilePurification>().await?.into_value();
    let bytes = imp::encode(
        &json!({"arguments":arguments,"synthesis":synthesis,"analysis":initial_analysis}),
    )?;
    chemistry.send::<Purification>(&bytes.as_slice()).await?;
    let mut cent = chemistry.recv::<CentLoan>().await?;
    cent.send::<CentRequest>(&bytes.as_slice()).await?;
    let purification = imp::decode(cent.recv::<InstrumentResult>().await?)?;
    imp::record(
        &purification,
        ("run_separation", "CHEMIST", &arguments),
        &json!({"synthesis":synthesis_id,"analysis":analysis_id}),
    )?;
    chemistry.send::<CentLoan>(cent).await?;
    let bytes = imp::encode(&purification)?;
    chemistry
        .send::<FilePurificationResult>(&bytes.as_slice())
        .await?;
    let mut sample = json!({"synthesis":synthesis,"initial_analysis":initial_analysis,"purification":purification,"analysis":initial_analysis});
    imp::specimen(&sample)?;
    let bytes = imp::encode(&sample)?;
    chemist.send::<Sample>(&bytes.as_slice()).await?;
    chemist.send::<ChemAccess>(chemistry).await?;
    let qc_arguments = json!({"scientist":"CHEMIST","data_type":"purification"});
    let mut approved = Value::Null;
    loop {
        let branch = chemist.offer().await?;
        match branch.label() {
            FirstChemist::LOGICAL_LABEL
            | NextChemist::LOGICAL_LABEL
            | LastChemist::LOGICAL_LABEL => {
                let selected = match branch.label() {
                    FirstChemist::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<FirstChemist>().await?)?
                    }
                    NextChemist::LOGICAL_LABEL => imp::decode(branch.recv::<NextChemist>().await?)?,
                    _ => imp::decode(branch.recv::<LastChemist>().await?)?,
                };
                if selected != qc_arguments {
                    return Err(ShellError::message(
                        "QC selected a different actual chemistry request",
                    ));
                }
                let bytes = imp::encode(&sample)?;
                chemist.send::<ChemistReport>(&bytes.as_slice()).await?;
            }
            FirstBiologist::LOGICAL_LABEL => {
                branch.recv::<FirstBiologist>().await?;
            }
            NextBiologist::LOGICAL_LABEL => {
                branch.recv::<NextBiologist>().await?;
            }
            ChemistRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ChemistRejected>().await?)?;
                let inputs = imp::qc_inputs(&qc_arguments, &sample)?;
                imp::qc_result(&result, &qc_arguments, &inputs)?;
                let mut chemistry = chemist.recv::<ChemAccess>().await?;
                let bytes = imp::encode(&result)?;
                chemist.send::<FileFeedback>(&bytes.as_slice()).await?;
                let arguments = chemistry.recv::<FileAnalysis>().await?.into_value();
                let bytes =
                    imp::encode(&json!({"arguments":arguments,"synthesis":sample["synthesis"]}))?;
                chemistry.send::<Reanalysis>(&bytes.as_slice()).await?;
                let mut spec = chemistry.recv::<SpecLoan>().await?;
                spec.send::<SpecRequest>(&bytes.as_slice()).await?;
                let result = imp::decode(spec.recv::<InstrumentResult>().await?)?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", &arguments),
                    &json!({"synthesis":synthesis_id}),
                )?;
                chemistry.send::<SpecLoan>(spec).await?;
                let bytes = imp::encode(&result)?;
                chemistry
                    .send::<FileAnalysisResult>(&bytes.as_slice())
                    .await?;
                sample["analysis"] = result;
                chemist.send::<ChemAccess>(chemistry).await?;
            }
            LastChemistRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<LastChemistRejected>().await?)?;
                let inputs = imp::qc_inputs(&qc_arguments, &sample)?;
                imp::qc_result(&result, &qc_arguments, &inputs)?;
                let mut chemistry = chemist.recv::<ChemAccess>().await?;
                let bytes = imp::encode(&result)?;
                chemist.send::<FileFeedback>(&bytes.as_slice()).await?;
                let arguments = chemistry.recv::<FileAnalysis>().await?.into_value();
                let bytes =
                    imp::encode(&json!({"arguments":arguments,"synthesis":sample["synthesis"]}))?;
                chemistry.send::<LastReanalysis>(&bytes.as_slice()).await?;
                let mut spec = chemistry.recv::<SpecLoan>().await?;
                spec.send::<SpecRequest>(&bytes.as_slice()).await?;
                let result = imp::decode(spec.recv::<InstrumentResult>().await?)?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", &arguments),
                    &json!({"synthesis":synthesis_id}),
                )?;
                chemistry.send::<SpecLoan>(spec).await?;
                let bytes = imp::encode(&result)?;
                chemistry
                    .send::<FileAnalysisResult>(&bytes.as_slice())
                    .await?;
                sample["analysis"] = result;
                chemist.send::<ChemAccess>(chemistry).await?;
            }
            ChemistApproved::LOGICAL_LABEL | LastChemistApproved::LOGICAL_LABEL => {
                let result = if branch.label() == ChemistApproved::LOGICAL_LABEL {
                    imp::decode(branch.recv::<ChemistApproved>().await?)?
                } else {
                    imp::decode(branch.recv::<LastChemistApproved>().await?)?
                };
                let inputs = imp::qc_inputs(&qc_arguments, &sample)?;
                imp::qc_result(&result, &qc_arguments, &inputs)?;
                let bytes = imp::encode(&result)?;
                chemist.send::<FileFeedback>(&bytes.as_slice()).await?;
                approved = result;
            }
            BiologistApproved::LOGICAL_LABEL => {
                branch.recv::<BiologistApproved>().await?;
            }
            BiologistRejected::LOGICAL_LABEL => {
                branch.recv::<BiologistRejected>().await?;
            }
            LastBiologistApproved::LOGICAL_LABEL => {
                branch.recv::<LastBiologistApproved>().await?;
            }
            LastBiologistRejected::LOGICAL_LABEL => {
                branch.recv::<LastBiologistRejected>().await?;
            }
            Quality::LOGICAL_LABEL => {
                let quality = imp::decode(branch.recv::<Quality>().await?)?;
                for key in ["synthesis", "initial_analysis", "purification", "analysis"] {
                    if quality["materials"][key] != sample[key] {
                        return Err(ShellError::message(
                            "quality substituted the chemist's actual batch or measurement",
                        ));
                    }
                }
                if quality["chemist_qc"] != approved {
                    return Err(ShellError::message(
                        "quality substituted the actual positive chemistry QC result",
                    ));
                }
                let inputs = imp::final_inputs(&quality)?;
                let bytes = imp::encode(&quality)?;
                chemist.send::<FileQuality>(&bytes.as_slice()).await?;
                let arguments = chemist.recv::<FileFinal>().await?.into_value();
                let bytes = imp::encode(&json!({"arguments":arguments,"quality":quality}))?;
                chemist.send::<FinalValidation>(&bytes.as_slice()).await?;
                let mut spec = chemist.recv::<SpecLoan>().await?;
                spec.send::<SpecRequest>(&bytes.as_slice()).await?;
                let result = imp::decode(spec.recv::<InstrumentResult>().await?)?;
                imp::record(&result, ("run_analysis", "CHEMIST", &arguments), &inputs)?;
                chemist.send::<SpecLoan>(spec).await?;
                let bytes = imp::encode(&result)?;
                chemist.send::<FileFinalResult>(&bytes.as_slice()).await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected chemistry message {label}"
                )));
            }
        }
    }
}
