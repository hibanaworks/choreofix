//! The independent verification and result-dependent analytical continuation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ANALYST>) -> ShellResult<Value> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            AnalystBatch::LOGICAL_LABEL => {
                let batch = branch.recv::<AnalystBatch>().await?.into_value();
                let material = imp::decode(endpoint.recv::<AnalystMaterial>().await?)?;
                imp::same_value(&material["synthesis"], &batch)?;
                let quality_packet = json!({"arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"},"material":material});
                imp::quality_input(&quality_packet)?;
                let bytes = imp::encode(&material)?;
                endpoint.send::<FileAnalystBatch>(&bytes.as_slice()).await?;
                let (verification, analytical) = async {
                    let [mut verify, mut analyze] = endpoint.par()?;
                    async {
                        let arguments = verify.recv::<FileVerification>().await?.into_value();
                        let packet = json!({"arguments":arguments,"material":material});
                        imp::instrument_inputs::<8>(&packet)?;
                        let bytes = imp::encode(&packet)?;
                        verify.send::<Verification>(&bytes.as_slice()).await?;
                        let mut cent = verify.recv::<Loan<CENTRIFUGE>>().await?;
                        cent.send::<Verification>(&bytes.as_slice()).await?;
                        let actual = imp::decode(cent.recv::<VerificationResult>().await?)?;
                        imp::instrument_result::<8>(&actual, &packet)?;
                        verify.send::<Loan<CENTRIFUGE>>(cent).await?;
                        let bytes = imp::encode(&actual)?;
                        verify.send::<FileVerificationResult>(&bytes.as_slice()).await?;
                        Ok(actual)
                    };
                    async {
                        let branch = analyze.offer().await?;
                        match branch.label() {
                            FilePublishAnalyst::LOGICAL_LABEL => {
                                let publication = branch.recv::<FilePublishAnalyst>().await?.into_value();
                                imp::same_value(&publication["artifact_id"], &material["bioassay"]["data"]["artifact_id"])?;
                                let bytes = imp::encode(&material)?;
                                analyze.send::<FilePublishedAnalyst>(&bytes.as_slice()).await?;
                                let arguments = analyze.recv::<FileChromatography>().await?.into_value();
                                let packet = json!({"arguments":arguments,"material":material});
                                imp::instrument_inputs::<6>(&packet)?;
                                let bytes = imp::encode(&packet)?;
                                analyze.send::<Chromatography>(&bytes.as_slice()).await?;
                                let mut chroma = analyze.recv::<Loan<CHROMATOGRAPH>>().await?;
                                chroma.send::<Chromatography>(&bytes.as_slice()).await?;
                                let chromatography = imp::decode(chroma.recv::<ChromatographyResult>().await?)?;
                                imp::instrument_result::<6>(&chromatography, &packet)?;
                                analyze.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                                let bytes = imp::encode(&chromatography)?;
                                analyze.send::<FileChromatographyResult>(&bytes.as_slice()).await?;
                                let arguments = analyze.recv::<FileMolecular>().await?.into_value();
                                let packet = json!({"arguments":arguments,"material":material,"chromatography":chromatography});
                                imp::instrument_inputs::<7>(&packet)?;
                                let bytes = imp::encode(&packet)?;
                                analyze.send::<Molecular>(&bytes.as_slice()).await?;
                                let mut spec = analyze.recv::<Loan<SPECTROMETER>>().await?;
                                spec.send::<Molecular>(&bytes.as_slice()).await?;
                                let molecular = imp::decode(spec.recv::<MolecularResult>().await?)?;
                                imp::instrument_result::<7>(&molecular, &packet)?;
                                analyze.send::<Loan<SPECTROMETER>>(spec).await?;
                                let bytes = imp::encode(&molecular)?;
                                analyze.send::<FileMolecularResult>(&bytes.as_slice()).await?;
                                let branch = analyze.offer().await?;
                                let quality = match branch.label() {
                                    AnalystApproved::LOGICAL_LABEL => imp::decode(branch.recv::<AnalystApproved>().await?)?,
                                    AnalystRejected::LOGICAL_LABEL => imp::decode(branch.recv::<AnalystRejected>().await?)?,
                                    label => return Err(ShellError::message(format!("unexpected analytical QC result {label}"))),
                                };
                                imp::quality_result(&quality, &quality_packet)?;
                                Ok(json!({"quality":quality,"chromatography":chromatography,"molecular":molecular}))
                            }
                            AnalystApproved::LOGICAL_LABEL => {
                                let quality = imp::decode(branch.recv::<AnalystApproved>().await?)?;
                                imp::quality_result(&quality, &quality_packet)?;
                                let publication = analyze.recv::<FilePublishAnalyst>().await?.into_value();
                                imp::same_value(&publication["artifact_id"], &material["bioassay"]["data"]["artifact_id"])?;
                                let bytes = imp::encode(&material)?;
                                analyze.send::<FilePublishedAnalyst>(&bytes.as_slice()).await?;
                                let arguments = analyze.recv::<FileChromatography>().await?.into_value();
                                let packet = json!({"arguments":arguments,"material":material});
                                imp::instrument_inputs::<6>(&packet)?;
                                let bytes = imp::encode(&packet)?;
                                analyze.send::<Chromatography>(&bytes.as_slice()).await?;
                                let mut chroma = analyze.recv::<Loan<CHROMATOGRAPH>>().await?;
                                chroma.send::<Chromatography>(&bytes.as_slice()).await?;
                                let chromatography = imp::decode(chroma.recv::<ChromatographyResult>().await?)?;
                                imp::instrument_result::<6>(&chromatography, &packet)?;
                                analyze.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                                let bytes = imp::encode(&chromatography)?;
                                analyze.send::<FileChromatographyResult>(&bytes.as_slice()).await?;
                                let arguments = analyze.recv::<FileMolecular>().await?.into_value();
                                let packet = json!({"arguments":arguments,"material":material,"chromatography":chromatography});
                                imp::instrument_inputs::<7>(&packet)?;
                                let bytes = imp::encode(&packet)?;
                                analyze.send::<Molecular>(&bytes.as_slice()).await?;
                                let mut spec = analyze.recv::<Loan<SPECTROMETER>>().await?;
                                spec.send::<Molecular>(&bytes.as_slice()).await?;
                                let molecular = imp::decode(spec.recv::<MolecularResult>().await?)?;
                                imp::instrument_result::<7>(&molecular, &packet)?;
                                analyze.send::<Loan<SPECTROMETER>>(spec).await?;
                                let bytes = imp::encode(&molecular)?;
                                analyze.send::<FileMolecularResult>(&bytes.as_slice()).await?;
                                Ok(json!({"quality":quality,"chromatography":chromatography,"molecular":molecular}))
                            }
                            AnalystRejected::LOGICAL_LABEL => {
                                let quality = imp::decode(branch.recv::<AnalystRejected>().await?)?;
                                imp::quality_result(&quality, &quality_packet)?;
                                Ok(json!({"quality":quality,"chromatography":null,"molecular":null}))
                            }
                            label => Err(ShellError::message(format!("unexpected analytical publication {label}"))),
                        }
                    };
                }.await?;
                let report = json!({"material":material,"quality":analytical["quality"],
                    "chromatography":analytical["chromatography"],"molecular":analytical["molecular"],"verification":verification});
                imp::check_report(&report)?;
                let bytes = imp::encode(&report)?;
                async {
                    let [mut chemist, mut qc] = endpoint.par()?;
                    async {
                        chemist.send::<AnalystReport>(&bytes.as_slice()).await?;
                    };
                    async {
                        qc.send::<AnalystReportToQc>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    AnalystReSynthesize::LOGICAL_LABEL => {
                        let cause = imp::decode(branch.recv::<AnalystReSynthesize>().await?)?;
                        imp::same_value(&cause, &report["quality"])?;
                        let bytes = imp::encode(&cause)?;
                        endpoint.send::<FileAnalystBatch>(&bytes.as_slice()).await?;
                    }
                    AnalystFinalApproved::LOGICAL_LABEL => {
                        let approved = imp::decode(branch.recv::<AnalystFinalApproved>().await?)?;
                        imp::same_value(&approved, &report["quality"])?;
                        let bytes = imp::encode(&approved)?;
                        endpoint.send::<FileAnalystBatch>(&bytes.as_slice()).await?;
                        return Ok(report);
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected analyst continuation {label}"
                        )));
                    }
                }
            }
            AnalystSynthesisFailed::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<AnalystSynthesisFailed>().await?)?;
                let cause = imp::decode(endpoint.recv::<AnalystReSynthesize>().await?)?;
                imp::same_value(&cause, &failed)?;
                let bytes = imp::encode(&cause)?;
                endpoint.send::<FileAnalystBatch>(&bytes.as_slice()).await?;
            }
            AnalystReagentsExhausted::LOGICAL_LABEL => {
                let exhausted = branch
                    .recv::<AnalystReagentsExhausted>()
                    .await?
                    .into_value();
                let terminal = endpoint.recv::<AnalystFinalExhausted>().await?.into_value();
                imp::same_value(&terminal, &exhausted)?;
                return Ok(terminal);
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected analyst batch {label}"
                )));
            }
        }
    }
}
