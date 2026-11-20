//! The unique spectrometer I/O owner consumes real resource requests before effects.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run(
    owner: &mut Endpoint<SPEC_OWNER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut spec = owner.recv::<SpecLoan>().await?;
    let mut raw = Value::Null;
    let mut calibration = Value::Null;
    let mut molecular = Value::Null;
    for _ in 0..4 {
        let branch = spec.offer().await?;
        match branch.label() {
            RawOperation::LOGICAL_LABEL => {
                let received = branch.recv::<RawOperation>().await?;
                let result = effects.apply(received).await?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", "raw_materials"),
                    &json!({}),
                )?;
                raw = result.clone();
                let bytes = imp::encode(&result)?;
                spec.send::<RawResult>(&bytes.as_slice()).await?;
            }
            CalibrationOperation::LOGICAL_LABEL => {
                let received = branch.recv::<CalibrationOperation>().await?;
                let packet = imp::decode(received.as_bytes())?;
                let cells = imp::record(
                    &packet["cells"],
                    ("run_separation", "BIOLOGIST", "cell_cultures"),
                    &json!({}),
                )?;
                let result = effects.apply(received).await?;
                imp::record(
                    &result,
                    ("run_analysis", "BIOLOGIST", "fluorescence"),
                    &json!({"prepared_cells":cells}),
                )?;
                calibration = result.clone();
                let bytes = imp::encode(&result)?;
                spec.send::<CalibrationResult>(&bytes.as_slice()).await?;
            }
            MolecularOperation::LOGICAL_LABEL => {
                let received = branch.recv::<MolecularOperation>().await?;
                let packet = imp::decode(received.as_bytes())?;
                let (compound, assay) = imp::assay(&packet["assay"])?;
                if packet["assay"]["raw"] != raw || packet["assay"]["calibration"] != calibration {
                    return Err(ShellError::message(
                        "molecular request substituted actual spectrometer inputs",
                    ));
                }
                let result = effects.apply(received).await?;
                imp::record(
                    &result,
                    ("run_analysis", "ANALYST", "molecular_detail"),
                    &json!({"compound":compound,"bioassay":assay}),
                )?;
                molecular = result.clone();
                let bytes = imp::encode(&result)?;
                spec.send::<MolecularResult>(&bytes.as_slice()).await?;
            }
            FinalOperation::LOGICAL_LABEL => {
                let received = branch.recv::<FinalOperation>().await?;
                let packet = imp::decode(received.as_bytes())?;
                let inputs = imp::quality(&packet["quality"])?;
                if packet["quality"]["assay"]["raw"] != raw
                    || packet["quality"]["assay"]["calibration"] != calibration
                    || packet["quality"]["molecular"] != molecular
                {
                    return Err(ShellError::message(
                        "final request substituted actual spectrometer results",
                    ));
                }
                let result = effects.apply(received).await?;
                imp::record(
                    &result,
                    ("run_analysis", "CHEMIST", "final_validation"),
                    &inputs,
                )?;
                let bytes = imp::encode(&result)?;
                spec.send::<FinalResult>(&bytes.as_slice()).await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected spectrometer request {label}"
                )));
            }
        }
    }
    owner.send::<SpecLoan>(spec).await?;
    Ok(())
}
