use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, BIOLOGIST>,
    cells_authority: Endpoint<'run, CULTURE_VIEW>,
    assay_authority: Endpoint<'run, ASSAY_VIEW>,
    stability_authority: Endpoint<'run, STABILITY_VIEW>,
) -> ShellResult<()> {
    let mut authority = cells_authority;
    let cells = loop {
        let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
        instrument.send::<Use<CULTURE_PREP>>(&()).await?;
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.send::<Loan<CULTURE_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected cells result {label}"
                )));
            }
        };

        let bytes = imp::encode(&result)?;
        endpoint
            .send::<Reply<CULTURE_PREP>>(&bytes.as_slice())
            .await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<CULTURE_VIEW>>().await?;
    };

    let cells_id = imp::artifact(&cells)?;
    let compound = imp::decode(endpoint.recv::<BiologicalSample>().await?)?;
    let compound_id = imp::compound(&compound)?;
    let bytes = imp::encode(&compound)?;
    endpoint.send::<BiologicalSample>(&bytes.as_slice()).await?;

    let mut authority = assay_authority;
    let assay = loop {
        let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
        instrument.send::<Use<BIOASSAY>>(&()).await?;
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.send::<Loan<ASSAY_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected assay result {label}"
                )));
            }
        };
        imp::inputs(
            &result,
            &json!({"compound":compound_id,"prepared_cells":cells_id}),
        )?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Reply<BIOASSAY>>(&bytes.as_slice()).await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<ASSAY_VIEW>>().await?;
    };

    let mut authority = stability_authority;
    let stability = loop {
        let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
        instrument.send::<Use<STABILITY>>(&()).await?;
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.send::<Loan<STABILITY_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected stability result {label}"
                )));
            }
        };
        imp::inputs(
            &result,
            &json!({"compound":compound_id,"medium":imp::artifact(&assay)?}),
        )?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Reply<STABILITY>>(&bytes.as_slice()).await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<STABILITY_VIEW>>().await?;
    };

    let report = json!({"compound":compound,"cells":cells,"assay":assay,"stability":stability});
    let bytes = imp::encode(&report)?;
    endpoint.send::<BioReport>(&bytes.as_slice()).await?;
    loop {
        let branch = endpoint.offer().await?;
        let plan = match branch.label() {
            ApprovedForBio::LOGICAL_LABEL => {
                let approval = imp::decode(branch.recv::<ApprovedForBio>().await?)?;
                imp::artifact(&approval)?;
                let bytes = imp::encode(&approval)?;
                endpoint.send::<StudyNotice>(&bytes.as_slice()).await?;
                return Ok(());
            }
            0 => imp::decode(branch.recv::<Plan<0, BIOLOGIST>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, BIOLOGIST>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, BIOLOGIST>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, BIOLOGIST>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected study notice {label}"
                )));
            }
        };
        if plan["biology"] == true {
            let bytes = imp::encode(&plan)?;
            endpoint.send::<StudyNotice>(&bytes.as_slice()).await?;
            let (assay_authority, stability_authority) = async {
                let [mut first, mut second] = endpoint.par()?;
                async { first.recv::<Loan<ASSAY_VIEW>>().await };
                async { second.recv::<Loan<STABILITY_VIEW>>().await };
            }
            .await?;

            let mut authority = assay_authority;
            let revised_assay = loop {
                let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
                instrument.send::<Use<BIOASSAY>>(&()).await?;
                endpoint.send::<Loan<CELL>>(instrument).await?;
                endpoint.send::<Loan<ASSAY_VIEW>>(authority).await?;
                let branch = endpoint.offer().await?;
                let result = match branch.label() {
                    Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                    Failed::<REANALYSIS_FAILED>::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<Failed<REANALYSIS_FAILED>>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected assay result {label}"
                        )));
                    }
                };
                imp::inputs(
                    &result,
                    &json!({"compound":compound_id,"prepared_cells":cells_id}),
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<Reply<BIOASSAY>>(&bytes.as_slice()).await?;
                if result["success"] == true {
                    break result;
                }
                authority = endpoint.recv::<Loan<ASSAY_VIEW>>().await?;
            };

            let mut authority = stability_authority;
            let revised_stability = loop {
                let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
                instrument.send::<Use<STABILITY>>(&()).await?;
                endpoint.send::<Loan<HPLC>>(instrument).await?;
                endpoint.send::<Loan<STABILITY_VIEW>>(authority).await?;
                let branch = endpoint.offer().await?;
                let result = match branch.label() {
                    Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
                    Failed::<REANALYSIS_FAILED>::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<Failed<REANALYSIS_FAILED>>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected stability result {label}"
                        )));
                    }
                };
                imp::inputs(
                    &result,
                    &json!({"compound":compound_id,"medium":imp::artifact(&revised_assay)?}),
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<Reply<STABILITY>>(&bytes.as_slice()).await?;
                if result["success"] == true {
                    break result;
                }
                authority = endpoint.recv::<Loan<STABILITY_VIEW>>().await?;
            };

            let report = json!({"compound":compound,"cells":cells,"assay":revised_assay,"stability":revised_stability});
            let bytes = imp::encode(&report)?;
            endpoint.send::<BioReport>(&bytes.as_slice()).await?;
        }
    }
}
