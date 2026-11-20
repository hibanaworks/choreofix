//! The synthesis/QC receiver owns the actual fume hood and resident reagent access.
use super::super::{
    global::*,
    imp::{self, SynthesisTerminal},
};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<FUME_OWNER>,
    domain: &D,
    (synthesis_actual, quality_actual): (&Cell<Value>, &Cell<Value>),
) -> ShellResult<()> {
    let (mut fume, stock) = async {
        let [mut fume, mut stock] = endpoint.par()?;
        async { fume.recv::<Loan<FUME_HOOD>>().await };
        async { stock.recv::<Loan<REAGENT_SUPPLY>>().await };
    }
    .await?;
    // This Option owns the affine Endpoint while it is physically resident here.
    let mut stock = Some(stock);
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            SynthesisAdmission::LOGICAL_LABEL => {
                let admitted = branch.recv::<SynthesisAdmission>().await?.into_value();
                let reagent = stock
                    .take()
                    .ok_or_else(|| ShellError::message("missing actual reagent Endpoint"))?;
                async {
                    let [mut hood, mut reagent_owner] = endpoint.par()?;
                    async {
                        hood.send::<Loan<FUME_HOOD>>(fume).await?;
                    };
                    async {
                        reagent_owner.send::<Loan<REAGENT_SUPPLY>>(reagent).await?;
                    };
                    Ok(())
                }
                .await?;
                let (hood, reagent) = async {
                    let [mut hood, mut reagent] = endpoint.par()?;
                    async { Ok(hood.recv::<FumeSynthesis>().await?.into_value()) };
                    async { Ok(reagent.recv::<ReagentSynthesis>().await?.into_value()) };
                }
                .await?;
                imp::same_value(&hood, &admitted)?;
                imp::same_value(&reagent, &admitted)?;
                let actual = domain
                    .execute("CHEMIST", "run_synthesis", admitted.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::synthesis_result(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                async {
                    let [mut hood, mut reagent] = endpoint.par()?;
                    async {
                        hood.send::<FumeSynthesisResult>(&bytes.as_slice()).await?;
                    };
                    async {
                        reagent
                            .send::<ReagentSynthesisResult>(&bytes.as_slice())
                            .await?;
                    };
                    Ok(())
                }
                .await?;
                let (returned_fume, returned_stock) = async {
                    let [mut hood, mut reagent] = endpoint.par()?;
                    async { hood.recv::<Loan<FUME_HOOD>>().await };
                    async { reagent.recv::<Loan<REAGENT_SUPPLY>>().await };
                }
                .await?;
                fume = returned_fume;
                stock = Some(returned_stock);
                synthesis_actual.set(actual);
            }
            FumeInspection::LOGICAL_LABEL => {
                branch.recv::<FumeInspection>().await?;
                endpoint.send::<Loan<FUME_HOOD>>(fume).await?;
                fume = endpoint.recv::<Loan<FUME_HOOD>>().await?;
            }
            SynthesisFinished::LOGICAL_LABEL => {
                let actual = synthesis_actual.take();
                let bytes = imp::encode(&actual)?;
                let terminal = SynthesisTerminal::<true>::try_from(bytes.as_slice())?;
                synthesis_actual.set(actual);
                branch.send::<SynthesisFinished>(&terminal).await?;
                endpoint.send::<SynthesisFinished>(&terminal).await?;
                async {
                    let [mut bio, mut analyst, mut spec, mut cent, mut chroma] = endpoint.par()?;
                    async {
                        bio.send::<BiologyBatch>(&terminal).await?;
                    };
                    async {
                        analyst.send::<AnalystBatch>(&terminal).await?;
                    };
                    async {
                        spec.send::<SpecBatch>(&terminal).await?;
                    };
                    async {
                        cent.send::<CentBatch>(&terminal).await?;
                    };
                    async {
                        chroma.send::<ChromaBatch>(&terminal).await?;
                    };
                    Ok(())
                }
                .await?;
                let reagent = stock
                    .take()
                    .ok_or_else(|| ShellError::message("missing actual reagent Endpoint"))?;
                endpoint.send::<Loan<REAGENT_SUPPLY>>(reagent).await?;
            }
            SynthesisFailed::LOGICAL_LABEL => {
                let actual = synthesis_actual.take();
                if actual["data"]["status"] != "synthesis_failed" {
                    return Err(ShellError::message(
                        "synthesis failure differs from actual result",
                    ));
                }
                let bytes = imp::encode(&actual)?;
                let failed = ReviewReply::<false>::from_bytes(&bytes)?;
                synthesis_actual.set(actual);
                branch.send::<SynthesisFailed>(&failed).await?;
                endpoint.send::<SynthesisFailed>(&failed).await?;
                async {
                    let [mut bio, mut analyst, mut spec, mut cent, mut chroma] = endpoint.par()?;
                    async {
                        bio.send::<BiologySynthesisFailed>(&failed).await?;
                    };
                    async {
                        analyst.send::<AnalystSynthesisFailed>(&failed).await?;
                    };
                    async {
                        spec.send::<SpecSynthesisFailed>(&failed).await?;
                    };
                    async {
                        cent.send::<CentSynthesisFailed>(&failed).await?;
                    };
                    async {
                        chroma.send::<ChromaSynthesisFailed>(&failed).await?;
                    };
                    Ok(())
                }
                .await?;
                let reagent = stock
                    .take()
                    .ok_or_else(|| ShellError::message("missing actual reagent Endpoint"))?;
                endpoint.send::<Loan<REAGENT_SUPPLY>>(reagent).await?;
            }
            ReagentsExhausted::LOGICAL_LABEL => {
                let actual = synthesis_actual.take();
                let bytes = imp::encode(&actual)?;
                let terminal = SynthesisTerminal::<false>::try_from(bytes.as_slice())?;
                synthesis_actual.set(actual);
                branch.send::<ReagentsExhausted>(&terminal).await?;
                endpoint.send::<ReagentsExhausted>(&terminal).await?;
                async {
                    let [mut bio, mut analyst, mut spec, mut cent, mut chroma] = endpoint.par()?;
                    async {
                        bio.send::<BiologyReagentsExhausted>(&terminal).await?;
                    };
                    async {
                        analyst.send::<AnalystReagentsExhausted>(&terminal).await?;
                    };
                    async {
                        spec.send::<SpecReagentsExhausted>(&terminal).await?;
                    };
                    async {
                        cent.send::<CentReagentsExhausted>(&terminal).await?;
                    };
                    async {
                        chroma.send::<ChromaReagentsExhausted>(&terminal).await?;
                    };
                    Ok(())
                }
                .await?;
                let reagent = stock
                    .take()
                    .ok_or_else(|| ShellError::message("missing actual reagent Endpoint"))?;
                endpoint.send::<Loan<REAGENT_SUPPLY>>(reagent).await?;
            }
            QualityAdmission::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<QualityAdmission>().await?)?;
                imp::quality_input(&admitted)?;
                let synthesis = synthesis_actual.take();
                imp::same_value(&admitted["material"]["synthesis"], &synthesis)?;
                synthesis_actual.set(synthesis);
                endpoint.send::<Loan<FUME_HOOD>>(fume).await?;
                let (hood, spec) = async {
                    let [mut hood, mut spec] = endpoint.par()?;
                    async { imp::decode(hood.recv::<FumeQuality>().await?) };
                    async { imp::decode(spec.recv::<SpecQuality>().await?) };
                }
                .await?;
                imp::same_value(&hood, &admitted)?;
                imp::same_value(&spec, &admitted)?;
                let actual = domain
                    .execute(
                        "QC_CONTROLLER",
                        "verify_result",
                        admitted["arguments"].clone(),
                    )
                    .await
                    .map_err(ShellError::message)?;
                imp::quality_result(&actual, &admitted)?;
                quality_actual.set(actual);
                drop(endpoint.offer().await?);
                let actual = quality_actual.take();
                let bytes = imp::encode(&actual)?;
                if actual["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    async {
                        let [mut hood, mut spec] = endpoint.par()?;
                        async {
                            hood.send::<FumeQualityApproved>(&reply).await?;
                        };
                        async {
                            spec.send::<SpecQualityApproved>(&reply).await?;
                        };
                        Ok(())
                    }
                    .await?;
                } else {
                    let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                    async {
                        let [mut hood, mut spec] = endpoint.par()?;
                        async {
                            hood.send::<FumeQualityRejected>(&reply).await?;
                        };
                        async {
                            spec.send::<SpecQualityRejected>(&reply).await?;
                        };
                        Ok(())
                    }
                    .await?;
                }
                fume = endpoint.recv::<Loan<FUME_HOOD>>().await?;
            }
            FumeReSynthesize::LOGICAL_LABEL => {
                let cause = imp::decode(branch.recv::<FumeReSynthesize>().await?)?;
                let synthesis = synthesis_actual.take();
                imp::batch_cause(&cause, &synthesis)?;
                synthesis_actual.set(synthesis);
                let reagent = endpoint.recv::<Loan<REAGENT_SUPPLY>>().await?;
                if stock.replace(reagent).is_some() {
                    return Err(ShellError::message("duplicate actual reagent Endpoint"));
                }
            }
            FumeFinalApproved::LOGICAL_LABEL => {
                let cause = imp::decode(branch.recv::<FumeFinalApproved>().await?)?;
                imp::batch_cause(&cause, &synthesis_actual.take())?;
                return Ok(());
            }
            FumeFinalExhausted::LOGICAL_LABEL => {
                let cause = branch.recv::<FumeFinalExhausted>().await?.into_value();
                imp::same_value(&cause, &synthesis_actual.take())?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected fume operation {label}"
                )));
            }
        }
    }
}
