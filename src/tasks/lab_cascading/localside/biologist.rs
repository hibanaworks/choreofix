//! Preparation, real material reception and assay custody follow the global.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, BIO_CONTROL>,
    mut biology: Endpoint<'r, BIOLOGIST>,
) -> ShellResult<Value> {
    let arguments = endpoint.recv::<FileCells>().await?.into_value();
    let packet = json!({"arguments":arguments,"material":{}});
    let bytes = imp::encode(&packet)?;
    endpoint.send::<Cells>(&bytes.as_slice()).await?;
    let mut cent = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
    cent.send::<Cells>(&bytes.as_slice()).await?;
    let cells = imp::decode(cent.recv::<CellsResult>().await?)?;
    imp::instrument_result::<3>(&cells, &packet)?;
    endpoint.send::<Loan<CENTRIFUGE>>(cent).await?;
    let bytes = imp::encode(&cells)?;
    endpoint.send::<FileCellsResult>(&bytes.as_slice()).await?;
    let arguments = endpoint.recv::<FileCalibration>().await?.into_value();
    let packet = json!({"arguments":arguments,"material":{"cells":cells}});
    let bytes = imp::encode(&packet)?;
    endpoint.send::<Calibration>(&bytes.as_slice()).await?;
    let mut spec = endpoint.recv::<Loan<SPECTROMETER>>().await?;
    spec.send::<Calibration>(&bytes.as_slice()).await?;
    let calibration = imp::decode(spec.recv::<CalibrationResult>().await?)?;
    imp::instrument_result::<4>(&calibration, &packet)?;
    endpoint.send::<Loan<SPECTROMETER>>(spec).await?;
    let bytes = imp::encode(&calibration)?;
    endpoint
        .send::<FileCalibrationResult>(&bytes.as_slice())
        .await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            BiologyBatch::LOGICAL_LABEL => {
                let batch = branch.recv::<BiologyBatch>().await?.into_value();
                let mut material = imp::decode(endpoint.recv::<PurifiedMaterial>().await?)?;
                imp::same_value(&material["synthesis"], &batch)?;
                let bytes = imp::encode(&material)?;
                endpoint.send::<FileBioBatch>(&bytes.as_slice()).await?;
                material["cells"] = cells.clone();
                material["calibration"] = calibration.clone();
                let arguments = biology.recv::<FileBioassay>().await?.into_value();
                let packet = json!({"arguments":arguments,"material":material});
                imp::instrument_inputs::<5>(&packet)?;
                let bytes = imp::encode(&packet)?;
                biology.send::<Assay>(&bytes.as_slice()).await?;
                let assay = imp::decode(biology.recv::<AssayResult>().await?)?;
                imp::instrument_result::<5>(&assay, &packet)?;
                material["bioassay"] = assay.clone();
                let bytes = imp::encode(&material)?;
                let assay_bytes = imp::encode(&assay)?;
                async {
                    let [mut analyst, mut qc, mut file] = biology.par()?;
                    async {
                        analyst.send::<AnalystMaterial>(&bytes.as_slice()).await?;
                    };
                    async {
                        qc.send::<BiologyReport>(&bytes.as_slice()).await?;
                    };
                    async {
                        file.send::<FileBioassayResult>(&assay_bytes.as_slice())
                            .await?;
                    };
                    Ok(())
                }
                .await?;
                endpoint.send::<Loan<BIOLOGIST>>(biology).await?;
                let packet = json!({"arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"},"material":material});
                let branch = endpoint.offer().await?;
                match branch.label() {
                    BiologyApproved::LOGICAL_LABEL => {
                        let approved = imp::decode(branch.recv::<BiologyApproved>().await?)?;
                        imp::quality_result(&approved, &packet)?;
                        let bytes = imp::encode(&approved)?;
                        endpoint.send::<FileBioBatch>(&bytes.as_slice()).await?;
                        // This approval is not the later loop's finalization
                        // choice. Receive that actual projected choice.
                        let final_result = imp::decode(
                            endpoint
                                .offer()
                                .await?
                                .recv::<BiologyFinalApproved>()
                                .await?,
                        )?;
                        imp::same_value(&final_result, &approved)?;
                        return Ok(final_result);
                    }
                    BiologyRejected::LOGICAL_LABEL => {
                        let rejected = imp::decode(branch.recv::<BiologyRejected>().await?)?;
                        imp::quality_result(&rejected, &packet)?;
                        let cause = imp::decode(endpoint.recv::<BiologyReSynthesize>().await?)?;
                        imp::same_value(&cause, &rejected)?;
                        let bytes = imp::encode(&cause)?;
                        endpoint.send::<FileBioBatch>(&bytes.as_slice()).await?;
                        biology = endpoint.recv::<Loan<BIOLOGIST>>().await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected biology QC {label}"
                        )));
                    }
                }
            }
            BiologySynthesisFailed::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<BiologySynthesisFailed>().await?)?;
                endpoint.send::<Loan<BIOLOGIST>>(biology).await?;
                let cause = imp::decode(endpoint.recv::<BiologyReSynthesize>().await?)?;
                imp::same_value(&cause, &failed)?;
                let bytes = imp::encode(&cause)?;
                endpoint.send::<FileBioBatch>(&bytes.as_slice()).await?;
                biology = endpoint.recv::<Loan<BIOLOGIST>>().await?;
            }
            BiologyReagentsExhausted::LOGICAL_LABEL => {
                let exhausted = branch
                    .recv::<BiologyReagentsExhausted>()
                    .await?
                    .into_value();
                endpoint.send::<Loan<BIOLOGIST>>(biology).await?;
                let terminal = endpoint.recv::<BiologyFinalExhausted>().await?.into_value();
                imp::same_value(&terminal, &exhausted)?;
                return Ok(terminal);
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected biology batch {label}"
                )));
            }
        }
    }
}
