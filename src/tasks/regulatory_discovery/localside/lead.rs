use super::super::{global::*, imp, reanalysis::Reanalysis};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
use std::cell::Cell;
#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, LEAD>,
    mut study: Endpoint<'run, STUDY>,
    (mut structure_authority, selection, domain): (
        Endpoint<'run, STRUCTURE_VIEW>,
        &Cell<Value>,
        &impl Domain,
    ),
) -> ShellResult<()> {
    let reference = async {
        let [mut synthesis, mut culture, mut reference] = endpoint.par()?;
        async {
            drop(synthesis.recv::<Loan<SYNTHESIS_VIEW>>().await?);
        };
        async {
            drop(culture.recv::<Loan<CULTURE_VIEW>>().await?);
        };
        let mut mass = reference.recv::<Loan<MASS>>().await?;
        mass.send::<Use<REFERENCE>>(&()).await?;
        let receipt = imp::decode(mass.recv::<Reply<REFERENCE>>().await?)?;
        reference.send::<Loan<MASS>>(mass).await?;
        let bytes = imp::encode(&receipt)?;
        reference
            .send::<Reply<REFERENCE>>(&bytes.as_slice())
            .await?;
        imp::artifact(&receipt)
    }
    .await?;
    let (mut assay, mut purity) = async {
        let [mut assay, mut purity] = endpoint.par()?;
        async { assay.recv::<Loan<ASSAY_VIEW>>().await };
        async { purity.recv::<Loan<PURITY_VIEW>>().await };
    }
    .await?;
    let (mut stability, mut toxicity) = async {
        let [mut stability, mut toxicity] = endpoint.par()?;
        async { stability.recv::<Loan<STABILITY_VIEW>>().await };
        async { toxicity.recv::<Loan<TOXICITY_VIEW>>().await };
    }
    .await?;
    let (mut bio, mut tox) = async {
        let [mut bio, mut tox] = endpoint.par()?;
        async {
            let report = imp::decode(bio.recv::<BioReport>().await?)?;
            let bytes = imp::encode(&report)?;
            bio.send::<BioReport>(&bytes.as_slice()).await?;
            Ok(report)
        };
        async {
            let report = imp::decode(tox.recv::<ToxReport>().await?)?;
            let bytes = imp::encode(&report)?;
            tox.send::<ToxReport>(&bytes.as_slice()).await?;
            Ok(report)
        };
    }
    .await?;
    let compound = imp::compound(&bio["compound"])?;
    let ((mut structural, mut structure_authority), characterization) = async {
        let [mut confirmation, mut characterization] = endpoint.par()?;
        async {
            let result = loop {
                let mut instrument = confirmation.recv::<Loan<HPLC>>().await?;
                instrument.send::<Use<STRUCTURE>>(&()).await?;
                confirmation.send::<Loan<HPLC>>(instrument).await?;
                confirmation
                    .send::<Loan<STRUCTURE_VIEW>>(structure_authority)
                    .await?;
                let branch = confirmation.offer().await?;
                let receipt = match branch.label() {
                    Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                    Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected structural confirmation {label}"
                        )));
                    }
                };
                imp::inputs(&receipt, &json!({"compound":compound}))?;
                let bytes = imp::encode(&receipt)?;
                confirmation
                    .send::<Reply<STRUCTURE>>(&bytes.as_slice())
                    .await?;
                structure_authority = confirmation.recv::<Loan<STRUCTURE_VIEW>>().await?;
                if receipt["success"] == true {
                    break (imp::artifact(&receipt)?, structure_authority);
                }
            };
            Ok(result)
        };
        async {
            let mut mass = characterization.recv::<Loan<MASS>>().await?;
            mass.send::<Use<CHARACTERIZATION>>(&()).await?;
            let receipt = imp::decode(mass.recv::<Reply<CHARACTERIZATION>>().await?)?;
            characterization.send::<Loan<MASS>>(mass).await?;
            let bytes = imp::encode(&receipt)?;
            characterization
                .send::<Reply<CHARACTERIZATION>>(&bytes.as_slice())
                .await?;
            imp::inputs(
                &receipt,
                &json!({"compound":compound,"reference":reference}),
            )?;
            imp::artifact(&receipt)
        };
    }
    .await?;
    loop {
        if bio["compound"] != tox["compound"]
            || imp::compound(&bio["compound"])? != compound
            || tox["toxicity"]["data"]["inputs"]["prepared_cells"] != imp::artifact(&bio["cells"])?
        {
            return Err(ShellError::message("study reports name different material"));
        }
        let arguments = endpoint.recv::<Request<DECISION>>().await?.into_value();
        let decision = domain
            .execute("LEAD_SCIENTIST", "submit_decision", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::actual(&decision, ("LEAD_SCIENTIST", "submit_decision", &arguments))?;
        imp::inputs(
            &decision,
            &json!({
                "compound":compound, "bioassay":imp::artifact(&bio["assay"])?,
                "stability":imp::artifact(&bio["stability"])?, "toxicity":imp::artifact(&tox["toxicity"])?,
                "structural":structural, "characterization":characterization
            }),
        )?;
        let bytes = imp::encode(&decision)?;
        endpoint.send::<Reply<DECISION>>(&bytes.as_slice()).await?;
        study.send::<Decision>(&bytes.as_slice()).await?;
        endpoint.send::<Loan<STUDY>>(study).await?;
        let branch = endpoint.offer().await?;
        let receipt = match branch.label() {
            ApprovedForLead::LOGICAL_LABEL => imp::decode(branch.recv::<ApprovedForLead>().await?)?,
            ComplianceRejected::LOGICAL_LABEL => {
                imp::decode(branch.recv::<ComplianceRejected>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected regulatory review {label}"
                )));
            }
        };
        imp::inputs(
            &receipt,
            &json!({"compound":compound,"decision":imp::artifact(&decision)?}),
        )?;
        let bytes = imp::encode(&receipt)?;
        endpoint.send::<ReviewResult>(&bytes.as_slice()).await?;
        if receipt["success"] == true {
            return Ok(());
        }
        study = endpoint.recv::<Loan<STUDY>>().await?;
        let requested = endpoint.recv::<Request<REANALYSIS>>().await?.into_value();
        let choice = u8::from(requested["biology"] == true)
            | (u8::from(requested["toxicology"] == true) << 1);
        let bytes = imp::encode(&requested)?;
        selection.set(requested);
        drop(study.offer().await?);
        match choice {
            0 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cells] = study.par()?;
                    async {
                        let plan = Reanalysis::<0, BIOLOGIST>::from_bytes(&bytes)?;
                        bio.send::<Plan<0, BIOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<0, TOXICOLOGIST>::from_bytes(&bytes)?;
                        tox.send::<Plan<0, TOXICOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<0, HPLC_OWNER>::from_bytes(&bytes)?;
                        hplc.send::<Plan<0, HPLC_OWNER>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<0, CELL_OWNER>::from_bytes(&bytes)?;
                        cells.send::<Plan<0, CELL_OWNER>>(&plan).await?;
                    };
                    Ok(())
                }
                .await?
            }
            1 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cells] = study.par()?;
                    async {
                        let plan = Reanalysis::<1, BIOLOGIST>::from_bytes(&bytes)?;
                        bio.send::<Plan<1, BIOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<1, TOXICOLOGIST>::from_bytes(&bytes)?;
                        tox.send::<Plan<1, TOXICOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<1, HPLC_OWNER>::from_bytes(&bytes)?;
                        hplc.send::<Plan<1, HPLC_OWNER>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<1, CELL_OWNER>::from_bytes(&bytes)?;
                        cells.send::<Plan<1, CELL_OWNER>>(&plan).await?;
                    };
                    Ok(())
                }
                .await?
            }
            2 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cells] = study.par()?;
                    async {
                        let plan = Reanalysis::<2, BIOLOGIST>::from_bytes(&bytes)?;
                        bio.send::<Plan<2, BIOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<2, TOXICOLOGIST>::from_bytes(&bytes)?;
                        tox.send::<Plan<2, TOXICOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<2, HPLC_OWNER>::from_bytes(&bytes)?;
                        hplc.send::<Plan<2, HPLC_OWNER>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<2, CELL_OWNER>::from_bytes(&bytes)?;
                        cells.send::<Plan<2, CELL_OWNER>>(&plan).await?;
                    };
                    Ok(())
                }
                .await?
            }
            3 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cells] = study.par()?;
                    async {
                        let plan = Reanalysis::<3, BIOLOGIST>::from_bytes(&bytes)?;
                        bio.send::<Plan<3, BIOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<3, TOXICOLOGIST>::from_bytes(&bytes)?;
                        tox.send::<Plan<3, TOXICOLOGIST>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<3, HPLC_OWNER>::from_bytes(&bytes)?;
                        hplc.send::<Plan<3, HPLC_OWNER>>(&plan).await?;
                    };
                    async {
                        let plan = Reanalysis::<3, CELL_OWNER>::from_bytes(&bytes)?;
                        cells.send::<Plan<3, CELL_OWNER>>(&plan).await?;
                    };
                    Ok(())
                }
                .await?
            }
            _ => unreachable!(),
        }
        selection.set(Value::Null);

        match choice {
            0 => (),
            1 => {
                async {
                    let [mut bioassay, mut stable] = endpoint.par()?;
                    async {
                        bioassay.send::<Loan<ASSAY_VIEW>>(assay).await?;
                    };
                    async {
                        stable.send::<Loan<STABILITY_VIEW>>(stability).await?;
                    };
                    Ok(())
                }
                .await?;
                assay = endpoint.recv::<Loan<ASSAY_VIEW>>().await?;
                stability = endpoint.recv::<Loan<STABILITY_VIEW>>().await?;
                bio = imp::decode(endpoint.recv::<BioReport>().await?)?;
                let bytes = imp::encode(&bio)?;
                endpoint.send::<BioReport>(&bytes.as_slice()).await?;
            }
            2 => {
                async {
                    let [mut pure, mut toxic] = endpoint.par()?;
                    async {
                        pure.send::<Loan<PURITY_VIEW>>(purity).await?;
                    };
                    async {
                        toxic.send::<Loan<TOXICITY_VIEW>>(toxicity).await?;
                    };
                    Ok(())
                }
                .await?;
                purity = endpoint.recv::<Loan<PURITY_VIEW>>().await?;
                toxicity = endpoint.recv::<Loan<TOXICITY_VIEW>>().await?;
                tox = imp::decode(endpoint.recv::<ToxReport>().await?)?;
                let bytes = imp::encode(&tox)?;
                endpoint.send::<ToxReport>(&bytes.as_slice()).await?;
            }
            3 => {
                async {
                    let [mut bioassay, mut stable, mut pure, mut toxic] = endpoint.par()?;
                    async {
                        bioassay.send::<Loan<ASSAY_VIEW>>(assay).await?;
                    };
                    async {
                        stable.send::<Loan<STABILITY_VIEW>>(stability).await?;
                    };
                    async {
                        pure.send::<Loan<PURITY_VIEW>>(purity).await?;
                    };
                    async {
                        toxic.send::<Loan<TOXICITY_VIEW>>(toxicity).await?;
                    };
                    Ok(())
                }
                .await?;
                (assay, purity) = async {
                    let [mut first, mut second] = endpoint.par()?;
                    async { first.recv::<Loan<ASSAY_VIEW>>().await };
                    async { second.recv::<Loan<PURITY_VIEW>>().await };
                }
                .await?;
                (stability, toxicity) = async {
                    let [mut first, mut second] = endpoint.par()?;
                    async { first.recv::<Loan<STABILITY_VIEW>>().await };
                    async { second.recv::<Loan<TOXICITY_VIEW>>().await };
                }
                .await?;
                (bio, tox) = async {
                    let [mut bio, mut tox] = endpoint.par()?;
                    async {
                        let report = imp::decode(bio.recv::<BioReport>().await?)?;
                        let bytes = imp::encode(&report)?;
                        bio.send::<BioReport>(&bytes.as_slice()).await?;
                        Ok(report)
                    };
                    async {
                        let report = imp::decode(tox.recv::<ToxReport>().await?)?;
                        let bytes = imp::encode(&report)?;
                        tox.send::<ToxReport>(&bytes.as_slice()).await?;
                        Ok(report)
                    };
                }
                .await?;
            }
            _ => unreachable!(),
        }
        (structural, structure_authority) = loop {
            let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
            instrument.send::<Use<STRUCTURE>>(&()).await?;
            endpoint.send::<Loan<HPLC>>(instrument).await?;
            endpoint
                .send::<Loan<STRUCTURE_VIEW>>(structure_authority)
                .await?;
            let branch = endpoint.offer().await?;
            let receipt = match branch.label() {
                Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                Failed::<REANALYSIS_FAILED>::LOGICAL_LABEL => {
                    imp::decode(branch.recv::<Failed<REANALYSIS_FAILED>>().await?)?
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected structural confirmation {label}"
                    )));
                }
            };
            imp::inputs(&receipt, &json!({"compound":compound}))?;
            let bytes = imp::encode(&receipt)?;
            endpoint.send::<Reply<STRUCTURE>>(&bytes.as_slice()).await?;
            structure_authority = endpoint.recv::<Loan<STRUCTURE_VIEW>>().await?;
            if receipt["success"] == true {
                break (imp::artifact(&receipt)?, structure_authority);
            }
        };
    }
}
