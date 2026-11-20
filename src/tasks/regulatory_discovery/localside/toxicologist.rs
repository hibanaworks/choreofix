use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, TOXICOLOGIST>,
    purity_authority: Endpoint<'run, PURITY_VIEW>,
    toxicity_authority: Endpoint<'run, TOXICITY_VIEW>,
) -> ShellResult<()> {
    let mut mass = endpoint.recv::<Loan<MASS>>().await?;
    mass.send::<Use<BASELINE>>(&()).await?;
    let baseline = imp::decode(mass.recv::<Reply<BASELINE>>().await?)?;
    endpoint.send::<Loan<MASS>>(mass).await?;
    let bytes = imp::encode(&baseline)?;
    endpoint.send::<Reply<BASELINE>>(&bytes.as_slice()).await?;
    let baseline_id = imp::artifact(&baseline)?;
    let compound = imp::decode(endpoint.recv::<ToxicologySample>().await?)?;
    let compound_id = imp::compound(&compound)?;
    let bytes = imp::encode(&compound)?;
    endpoint.send::<ToxicologySample>(&bytes.as_slice()).await?;

    let mut authority = purity_authority;
    let purity = loop {
        let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
        instrument.send::<Use<PURITY>>(&()).await?;
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.send::<Loan<PURITY_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected purity result {label}"
                )));
            }
        };
        imp::inputs(&result, &json!({"compound":compound_id}))?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Reply<PURITY>>(&bytes.as_slice()).await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<PURITY_VIEW>>().await?;
    };

    let mut authority = toxicity_authority;
    let toxicity = loop {
        let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
        instrument.send::<Use<CYTOTOXICITY>>(&()).await?;
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.send::<Loan<TOXICITY_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected toxicity result {label}"
                )));
            }
        };
        imp::inputs(
            &result,
            &json!({"compound":compound_id,"baseline":baseline_id,"purity":imp::artifact(&purity)?,"prepared_cells": result["data"]["inputs"]["prepared_cells"].as_u64().ok_or_else(|| ShellError::message("toxicity lacks prepared cells"))?}),
        )?;
        let bytes = imp::encode(&result)?;
        endpoint
            .send::<Reply<CYTOTOXICITY>>(&bytes.as_slice())
            .await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<TOXICITY_VIEW>>().await?;
    };

    let report =
        json!({"compound":compound,"baseline":baseline,"purity":purity,"toxicity":toxicity});
    let bytes = imp::encode(&report)?;
    endpoint.send::<ToxReport>(&bytes.as_slice()).await?;
    loop {
        let branch = endpoint.offer().await?;
        let plan = match branch.label() {
            ApprovedForTox::LOGICAL_LABEL => {
                let approval = imp::decode(branch.recv::<ApprovedForTox>().await?)?;
                imp::artifact(&approval)?;
                let bytes = imp::encode(&approval)?;
                endpoint.send::<StudyNotice>(&bytes.as_slice()).await?;
                return Ok(());
            }
            0 => imp::decode(branch.recv::<Plan<0, TOXICOLOGIST>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, TOXICOLOGIST>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, TOXICOLOGIST>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, TOXICOLOGIST>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected study notice {label}"
                )));
            }
        };
        if plan["toxicology"] == true {
            let bytes = imp::encode(&plan)?;
            endpoint.send::<StudyNotice>(&bytes.as_slice()).await?;
            let (purity_authority, toxicity_authority) = async {
                let [mut first, mut second] = endpoint.par()?;
                async { first.recv::<Loan<PURITY_VIEW>>().await };
                async { second.recv::<Loan<TOXICITY_VIEW>>().await };
            }
            .await?;

            let mut authority = purity_authority;
            let revised_purity = loop {
                let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
                instrument.send::<Use<PURITY>>(&()).await?;
                endpoint.send::<Loan<HPLC>>(instrument).await?;
                endpoint.send::<Loan<PURITY_VIEW>>(authority).await?;
                let branch = endpoint.offer().await?;
                let result = match branch.label() {
                    Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                    Failed::<REANALYSIS_FAILED>::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<Failed<REANALYSIS_FAILED>>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected purity result {label}"
                        )));
                    }
                };
                imp::inputs(&result, &json!({"compound":compound_id}))?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<Reply<PURITY>>(&bytes.as_slice()).await?;
                if result["success"] == true {
                    break result;
                }
                authority = endpoint.recv::<Loan<PURITY_VIEW>>().await?;
            };

            let mut authority = toxicity_authority;
            let revised_toxicity = loop {
                let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
                instrument.send::<Use<CYTOTOXICITY>>(&()).await?;
                endpoint.send::<Loan<CELL>>(instrument).await?;
                endpoint.send::<Loan<TOXICITY_VIEW>>(authority).await?;
                let branch = endpoint.offer().await?;
                let result = match branch.label() {
                    Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                    Failed::<REANALYSIS_FAILED>::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<Failed<REANALYSIS_FAILED>>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected toxicity result {label}"
                        )));
                    }
                };
                imp::inputs(
                    &result,
                    &json!({"compound":compound_id,"baseline":baseline_id,"purity":imp::artifact(&revised_purity)?,"prepared_cells": result["data"]["inputs"]["prepared_cells"].as_u64().ok_or_else(|| ShellError::message("toxicity lacks prepared cells"))?}),
                )?;
                let bytes = imp::encode(&result)?;
                endpoint
                    .send::<Reply<CYTOTOXICITY>>(&bytes.as_slice())
                    .await?;
                if result["success"] == true {
                    break result;
                }
                authority = endpoint.recv::<Loan<TOXICITY_VIEW>>().await?;
            };

            let report = json!({"compound":compound,"baseline":baseline,"purity":revised_purity,"toxicity":revised_toxicity});
            let bytes = imp::encode(&report)?;
            endpoint.send::<ToxReport>(&bytes.as_slice()).await?;
        }
    }
}
