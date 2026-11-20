//! Actual chemistry data and affine synthesis authority, through final validation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, CHEM_CONTROL>,
    mut chemistry: Endpoint<'r, CHEMIST>,
) -> ShellResult<Value> {
    loop {
        let admitted = chemistry.recv::<FileSynthesis>().await?;
        chemistry.send::<SynthesisAdmission>(&admitted).await?;
        let (mut fume, mut stock) = async {
            let [mut fume, mut stock] = chemistry.par()?;
            async { fume.recv::<Loan<FUME_HOOD>>().await };
            async { stock.recv::<Loan<REAGENT_SUPPLY>>().await };
        }
        .await?;
        let (actual, reagent_actual) = async {
            async {
                fume.send::<FumeSynthesis>(&admitted).await?;
                imp::decode(fume.recv::<FumeSynthesisResult>().await?)
            };
            async {
                stock.send::<ReagentSynthesis>(&admitted).await?;
                imp::decode(stock.recv::<ReagentSynthesisResult>().await?)
            };
        }
        .await?;
        imp::same_value(&actual, &reagent_actual)?;
        imp::synthesis_result(&actual, &admitted.into_value())?;
        let bytes = imp::encode(&actual)?;
        async {
            let [mut hood, mut reagent, mut file] = chemistry.par()?;
            async {
                hood.send::<Loan<FUME_HOOD>>(fume).await?;
            };
            async {
                reagent.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            };
            async {
                file.send::<FileSynthesisResult>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
        let branch = chemistry.offer().await?;
        match branch.label() {
            SynthesisFinished::LOGICAL_LABEL => {
                let synthesis = branch.recv::<SynthesisFinished>().await?.into_value();
                imp::same_value(&synthesis, &actual)?;
                endpoint.send::<Loan<CHEMIST>>(chemistry).await?;
                let mut material = json!({"synthesis":synthesis});
                let arguments = endpoint.recv::<FileAnalysis>().await?.into_value();
                let packet = json!({"arguments":arguments,"material":material});
                imp::instrument_inputs::<1>(&packet)?;
                let bytes = imp::encode(&packet)?;
                endpoint.send::<Analysis>(&bytes.as_slice()).await?;
                let mut spec = endpoint.recv::<Loan<SPECTROMETER>>().await?;
                spec.send::<Analysis>(&bytes.as_slice()).await?;
                let analysis = imp::decode(spec.recv::<AnalysisResult>().await?)?;
                imp::instrument_result::<1>(&analysis, &packet)?;
                endpoint.send::<Loan<SPECTROMETER>>(spec).await?;
                let bytes = imp::encode(&analysis)?;
                endpoint
                    .send::<FileAnalysisResult>(&bytes.as_slice())
                    .await?;
                material["analysis"] = analysis;
                let arguments = endpoint.recv::<FilePurification>().await?.into_value();
                let packet = json!({"arguments":arguments,"material":material});
                imp::instrument_inputs::<2>(&packet)?;
                let bytes = imp::encode(&packet)?;
                endpoint.send::<Purification>(&bytes.as_slice()).await?;
                let mut cent = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
                cent.send::<Purification>(&bytes.as_slice()).await?;
                let purification = imp::decode(cent.recv::<PurificationResult>().await?)?;
                imp::instrument_result::<2>(&purification, &packet)?;
                endpoint.send::<Loan<CENTRIFUGE>>(cent).await?;
                let bytes = imp::encode(&purification)?;
                endpoint
                    .send::<FilePurificationResult>(&bytes.as_slice())
                    .await?;
                material["purification"] = purification;
                let bytes = imp::encode(&material)?;
                endpoint.send::<PurifiedMaterial>(&bytes.as_slice()).await?;
                let report = imp::decode(endpoint.recv::<AnalystReport>().await?)?;
                for key in ["synthesis", "analysis", "purification"] {
                    imp::same_value(&report["material"][key], &material[key])?;
                }
                imp::check_report(&report)?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    ReSynthesize::LOGICAL_LABEL => {
                        let cause = imp::decode(branch.recv::<ReSynthesize>().await?)?;
                        imp::same_value(&cause, &report["quality"])?;
                        let bytes = imp::encode(&cause)?;
                        endpoint.send::<FileChemQuality>(&bytes.as_slice()).await?;
                        chemistry = endpoint.recv::<Loan<CHEMIST>>().await?;
                    }
                    FinalApproved::LOGICAL_LABEL => {
                        let approved = imp::decode(branch.recv::<FinalApproved>().await?)?;
                        imp::same_value(&approved, &report["quality"])?;
                        let bytes = imp::encode(&approved)?;
                        endpoint.send::<FileChemQuality>(&bytes.as_slice()).await?;
                        let arguments = endpoint.recv::<FileFinal>().await?.into_value();
                        let packet = json!({"arguments":arguments,"report":report});
                        imp::instrument_inputs::<10>(&packet)?;
                        let bytes = imp::encode(&packet)?;
                        endpoint.send::<FinalValidation>(&bytes.as_slice()).await?;
                        let mut spec = endpoint.recv::<Loan<SPECTROMETER>>().await?;
                        spec.send::<FinalValidation>(&bytes.as_slice()).await?;
                        let actual = imp::decode(spec.recv::<FinalResult>().await?)?;
                        imp::instrument_result::<10>(&actual, &packet)?;
                        endpoint.send::<Loan<SPECTROMETER>>(spec).await?;
                        let bytes = imp::encode(&actual)?;
                        async {
                            let [mut file, mut quality] = endpoint.par()?;
                            async {
                                file.send::<FileFinalResult>(&bytes.as_slice()).await?;
                            };
                            async {
                                quality.send::<FinalValidated>(&bytes.as_slice()).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        return Ok(actual);
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected chemistry continuation {label}"
                        )));
                    }
                }
            }
            SynthesisFailed::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<SynthesisFailed>().await?)?;
                imp::same_value(&failed, &actual)?;
                endpoint.send::<Loan<CHEMIST>>(chemistry).await?;
                let cause = imp::decode(endpoint.recv::<SynthesisFailed>().await?)?;
                imp::same_value(&cause, &failed)?;
                chemistry = endpoint.recv::<Loan<CHEMIST>>().await?;
            }
            ReagentsExhausted::LOGICAL_LABEL => {
                let exhausted = branch.recv::<ReagentsExhausted>().await?.into_value();
                imp::same_value(&exhausted, &actual)?;
                endpoint.send::<Loan<CHEMIST>>(chemistry).await?;
                let terminal = endpoint.recv::<FinalExhausted>().await?.into_value();
                imp::same_value(&terminal, &exhausted)?;
                return Ok(terminal);
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected synthesis result {label}"
                )));
            }
        }
    }
}
