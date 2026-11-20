//! QC owns both scientific authorities until its actual result permits rework.
use super::super::{
    global::*,
    imp::{self, SynthesisTerminal},
};
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<QC>) -> ShellResult<Value> {
    let actual = loop {
        let branch = endpoint.offer().await?;
        let (actual, chemistry, biology, stock) = match branch.label() {
            SynthesisFinished::LOGICAL_LABEL => {
                let batch = branch.recv::<SynthesisFinished>().await?.into_value();
                let chemistry = endpoint.recv::<Loan<CHEMIST>>().await?;
                let stock = endpoint.recv::<Loan<REAGENT_SUPPLY>>().await?;
                let material = imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                imp::same_value(&material["synthesis"], &batch)?;
                let biology = endpoint.recv::<Loan<BIOLOGIST>>().await?;
                let publication = endpoint.recv::<FilePublishQc>().await?.into_value();
                imp::same_value(
                    &publication["artifact_id"],
                    &material["bioassay"]["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&material)?;
                endpoint.send::<FilePublishedQc>(&bytes.as_slice()).await?;
                endpoint.send::<FileQcInput>(&bytes.as_slice()).await?;
                let arguments = endpoint.recv::<FileQuality>().await?.into_value();
                let packet = json!({"arguments":arguments,"material":material});
                imp::quality_input(&packet)?;
                let bytes = imp::encode(&packet)?;
                async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async {
                        fume.send::<QualityAdmission>(&bytes.as_slice()).await?;
                    };
                    async {
                        spec.send::<SpecQualityAdmission>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                let (mut fume, mut spec) = async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async { fume.recv::<Loan<FUME_HOOD>>().await };
                    async { spec.recv::<Loan<SPECTROMETER>>().await };
                }
                .await?;
                let (from_fume, from_spec) = async {
                    async {
                        fume.send::<FumeQuality>(&bytes.as_slice()).await?;
                        let branch = fume.offer().await?;
                        match branch.label() {
                            FumeQualityApproved::LOGICAL_LABEL => {
                                imp::decode(branch.recv::<FumeQualityApproved>().await?)
                            }
                            FumeQualityRejected::LOGICAL_LABEL => {
                                imp::decode(branch.recv::<FumeQualityRejected>().await?)
                            }
                            label => {
                                Err(ShellError::message(format!("unexpected fume QC {label}")))
                            }
                        }
                    };
                    async {
                        spec.send::<SpecQuality>(&bytes.as_slice()).await?;
                        let branch = spec.offer().await?;
                        match branch.label() {
                            SpecQualityApproved::LOGICAL_LABEL => {
                                imp::decode(branch.recv::<SpecQualityApproved>().await?)
                            }
                            SpecQualityRejected::LOGICAL_LABEL => {
                                imp::decode(branch.recv::<SpecQualityRejected>().await?)
                            }
                            label => Err(ShellError::message(format!(
                                "unexpected spectrum QC {label}"
                            ))),
                        }
                    };
                }
                .await?;
                imp::same_value(&from_fume, &from_spec)?;
                imp::quality_result(&from_fume, &packet)?;
                async {
                    let [mut fume_owner, mut spec_owner] = endpoint.par()?;
                    async {
                        fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                    };
                    async {
                        spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
                    };
                    Ok(())
                }
                .await?;
                let bytes = imp::encode(&from_fume)?;
                if from_fume["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    async {
                        let [mut bio, mut chroma] = endpoint.par()?;
                        async {
                            bio.send::<BiologyApproved>(&reply).await?;
                        };
                        async {
                            chroma.send::<ChromaQualityApproved>(&reply).await?;
                            let receipt =
                                imp::decode(chroma.recv::<ChromaQualityReceipt>().await?)?;
                            imp::same_value(&receipt, &from_fume)
                        };
                        Ok(())
                    }
                    .await?;
                    endpoint
                        .send::<FileQualityResult>(&bytes.as_slice())
                        .await?;
                    endpoint.send::<AnalystApproved>(&reply).await?;
                } else {
                    let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                    async {
                        let [mut bio, mut chroma] = endpoint.par()?;
                        async {
                            bio.send::<BiologyRejected>(&reply).await?;
                        };
                        async {
                            chroma.send::<ChromaQualityRejected>(&reply).await?;
                            let receipt =
                                imp::decode(chroma.recv::<ChromaQualityReceipt>().await?)?;
                            imp::same_value(&receipt, &from_fume)
                        };
                        Ok(())
                    }
                    .await?;
                    endpoint
                        .send::<FileQualityResult>(&bytes.as_slice())
                        .await?;
                    endpoint.send::<AnalystRejected>(&reply).await?;
                }
                let report = imp::decode(endpoint.recv::<AnalystReportToQc>().await?)?;
                imp::same_value(&report["material"], &material)?;
                imp::same_value(&report["quality"], &from_fume)?;
                imp::check_report(&report)?;
                (from_fume, chemistry, biology, stock)
            }
            SynthesisFailed::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<SynthesisFailed>().await?)?;
                if failed["data"]["status"] != "synthesis_failed" {
                    return Err(ShellError::message(
                        "failed synthesis has another actual outcome",
                    ));
                }
                let (chemistry, biology) = async {
                    let [mut chemistry, mut biology] = endpoint.par()?;
                    async { chemistry.recv::<Loan<CHEMIST>>().await };
                    async { biology.recv::<Loan<BIOLOGIST>>().await };
                }
                .await?;
                let stock = endpoint.recv::<Loan<REAGENT_SUPPLY>>().await?;
                (failed, chemistry, biology, stock)
            }
            ReagentsExhausted::LOGICAL_LABEL => {
                let exhausted = branch.recv::<ReagentsExhausted>().await?.into_value();
                let (chemistry, biology) = async {
                    let [mut chemistry, mut biology] = endpoint.par()?;
                    async { chemistry.recv::<Loan<CHEMIST>>().await };
                    async { biology.recv::<Loan<BIOLOGIST>>().await };
                }
                .await?;
                let stock = endpoint.recv::<Loan<REAGENT_SUPPLY>>().await?;
                (exhausted, chemistry, biology, stock)
            }
            label => return Err(ShellError::message(format!("unexpected QC batch {label}"))),
        };
        let bytes = imp::encode(&actual)?;
        if actual["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            endpoint.send::<FinalApproved>(&reply).await?;
            async {
                let [
                    mut bio,
                    mut analyst,
                    mut fume,
                    mut spec,
                    mut cent,
                    mut chroma,
                ] = endpoint.par()?;
                async {
                    bio.send::<BiologyFinalApproved>(&reply).await?;
                };
                async {
                    analyst.send::<AnalystFinalApproved>(&reply).await?;
                };
                async {
                    fume.send::<FumeFinalApproved>(&reply).await?;
                };
                async {
                    spec.send::<SpecFinalApproved>(&reply).await?;
                };
                async {
                    cent.send::<CentFinalApproved>(&reply).await?;
                };
                async {
                    chroma.send::<ChromaFinalApproved>(&reply).await?;
                };
                Ok(())
            }
            .await?;
            let validation = imp::decode(endpoint.recv::<FinalValidated>().await?)?;
            if validation["success"] != true
                || validation["tool_name"] != "run_analysis"
                || validation["data"]["sample"] != "final_validation"
                || !actual["data"]["artifact_id"].is_u64()
                || validation["data"]["inputs"]["bioassay_qc"] != actual["data"]["artifact_id"]
            {
                return Err(ShellError::message(
                    "final validation is not for the approved batch",
                ));
            }
            async {
                let [mut chem, mut bio, mut reagent] = endpoint.par()?;
                async {
                    chem.send::<Loan<CHEMIST>>(chemistry).await?;
                };
                async {
                    bio.send::<Loan<BIOLOGIST>>(biology).await?;
                };
                async {
                    reagent.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                };
                Ok(())
            }
            .await?;
            break validation;
        } else if actual["data"]["status"] == "reagent_exhausted" {
            let terminal = SynthesisTerminal::<false>::try_from(bytes.as_slice())?;
            endpoint.send::<FinalExhausted>(&terminal).await?;
            async {
                let [
                    mut bio,
                    mut analyst,
                    mut fume,
                    mut spec,
                    mut cent,
                    mut chroma,
                ] = endpoint.par()?;
                async {
                    bio.send::<BiologyFinalExhausted>(&terminal).await?;
                };
                async {
                    analyst.send::<AnalystFinalExhausted>(&terminal).await?;
                };
                async {
                    fume.send::<FumeFinalExhausted>(&terminal).await?;
                };
                async {
                    spec.send::<SpecFinalExhausted>(&terminal).await?;
                };
                async {
                    cent.send::<CentFinalExhausted>(&terminal).await?;
                };
                async {
                    chroma.send::<ChromaFinalExhausted>(&terminal).await?;
                };
                Ok(())
            }
            .await?;
            async {
                let [mut bio, mut analyst, mut qc] = endpoint.par()?;
                async {
                    bio.send::<FileBioBatch>(&bytes.as_slice()).await?;
                };
                async {
                    analyst.send::<FileAnalystBatch>(&bytes.as_slice()).await?;
                };
                async {
                    qc.send::<FileQcQuality>(&bytes.as_slice()).await?;
                };
                Ok(())
            }
            .await?;
            async {
                let [mut chem, mut bio, mut reagent] = endpoint.par()?;
                async {
                    chem.send::<Loan<CHEMIST>>(chemistry).await?;
                };
                async {
                    bio.send::<Loan<BIOLOGIST>>(biology).await?;
                };
                async {
                    reagent.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                };
                Ok(())
            }
            .await?;
            break actual;
        } else {
            let reply = ReviewReply::<false>::from_bytes(&bytes)?;
            if actual["data"]["status"] == "synthesis_failed" {
                endpoint.send::<SynthesisFailed>(&reply).await?;
            } else {
                endpoint.send::<ReSynthesize>(&reply).await?;
            }
            async {
                let [
                    mut bio,
                    mut analyst,
                    mut fume,
                    mut spec,
                    mut cent,
                    mut chroma,
                ] = endpoint.par()?;
                async {
                    bio.send::<BiologyReSynthesize>(&reply).await?;
                };
                async {
                    analyst.send::<AnalystReSynthesize>(&reply).await?;
                };
                async {
                    fume.send::<FumeReSynthesize>(&reply).await?;
                };
                async {
                    spec.send::<SpecReSynthesize>(&reply).await?;
                };
                async {
                    cent.send::<CentReSynthesize>(&reply).await?;
                };
                async {
                    chroma.send::<ChromaReSynthesize>(&reply).await?;
                };
                Ok(())
            }
            .await?;
            async {
                let [mut chem, mut bio, mut reagent] = endpoint.par()?;
                async {
                    chem.send::<Loan<CHEMIST>>(chemistry).await?;
                };
                async {
                    bio.send::<Loan<BIOLOGIST>>(biology).await?;
                };
                async {
                    reagent.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                };
                Ok(())
            }
            .await?;
        }
    };
    Ok(actual)
}
