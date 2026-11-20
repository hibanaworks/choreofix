use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::json;

pub async fn hplc(
    endpoint: &mut Endpoint<'_, HPLC_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    loop {
        let arguments = endpoint.recv::<Request<SYNTHESIS_QC>>().await?.into_value();
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.recv::<Use<SYNTHESIS_QC>>().await?;
        let receipt = domain
            .execute("CHEMIST", "run_hplc_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("CHEMIST", "run_hplc_analysis", &arguments))?;
        instrument = endpoint.recv::<Loan<HPLC>>().await?;
        let mut material = endpoint.recv::<Loan<SYNTHESIS_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<SYNTHESIS_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<SYNTHESIS_VIEW>>(material).await?;
    }
    loop {
        let arguments = endpoint.recv::<Request<PURITY>>().await?.into_value();
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.recv::<Use<PURITY>>().await?;
        let receipt = domain
            .execute("TOXICOLOGIST", "run_hplc_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("TOXICOLOGIST", "run_hplc_analysis", &arguments))?;
        instrument = endpoint.recv::<Loan<HPLC>>().await?;
        let mut material = endpoint.recv::<Loan<PURITY_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<PURITY_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<PURITY_VIEW>>(material).await?;
    }
    loop {
        let arguments = endpoint.recv::<Request<STABILITY>>().await?.into_value();
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.recv::<Use<STABILITY>>().await?;
        let receipt = domain
            .execute("BIOLOGIST", "run_hplc_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("BIOLOGIST", "run_hplc_analysis", &arguments))?;
        instrument = endpoint.recv::<Loan<HPLC>>().await?;
        let mut material = endpoint.recv::<Loan<STABILITY_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<STABILITY_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<STABILITY_VIEW>>(material).await?;
    }
    loop {
        let arguments = endpoint.recv::<Request<STRUCTURE>>().await?.into_value();
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.recv::<Use<STRUCTURE>>().await?;
        let receipt = domain
            .execute("LEAD_SCIENTIST", "run_hplc_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(
            &receipt,
            ("LEAD_SCIENTIST", "run_hplc_analysis", &arguments),
        )?;
        instrument = endpoint.recv::<Loan<HPLC>>().await?;
        let mut material = endpoint.recv::<Loan<STRUCTURE_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<STRUCTURE_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<STRUCTURE_VIEW>>(material).await?;
    }
    let approval = loop {
        let branch = endpoint.offer().await?;
        let plan = match branch.label() {
            ApprovedForHplc::LOGICAL_LABEL => {
                let approval = imp::decode(branch.recv::<ApprovedForHplc>().await?)?;
                imp::artifact(&approval)?;
                break approval;
            }
            0 => imp::decode(branch.recv::<Plan<0, HPLC_OWNER>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, HPLC_OWNER>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, HPLC_OWNER>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, HPLC_OWNER>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected study notice {label}"
                )));
            }
        };
        if plan["toxicology"] == true {
            loop {
                let arguments = endpoint.recv::<Request<PURITY>>().await?.into_value();
                endpoint.send::<Loan<HPLC>>(instrument).await?;
                endpoint.recv::<Use<PURITY>>().await?;
                let receipt = domain
                    .execute("TOXICOLOGIST", "run_hplc_analysis", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::result(&receipt, ("TOXICOLOGIST", "run_hplc_analysis", &arguments))?;
                instrument = endpoint.recv::<Loan<HPLC>>().await?;
                let mut material = endpoint.recv::<Loan<PURITY_VIEW>>().await?;
                let bytes = imp::encode(&receipt)?;
                if receipt["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    material.send::<Passed>(&reply).await?;
                    endpoint.send::<Loan<PURITY_VIEW>>(material).await?;
                    break;
                }
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                material.send::<Failed<REANALYSIS_FAILED>>(&reply).await?;
                endpoint.send::<Loan<PURITY_VIEW>>(material).await?;
            }
        }
        if plan["biology"] == true {
            loop {
                let arguments = endpoint.recv::<Request<STABILITY>>().await?.into_value();
                endpoint.send::<Loan<HPLC>>(instrument).await?;
                endpoint.recv::<Use<STABILITY>>().await?;
                let receipt = domain
                    .execute("BIOLOGIST", "run_hplc_analysis", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::result(&receipt, ("BIOLOGIST", "run_hplc_analysis", &arguments))?;
                instrument = endpoint.recv::<Loan<HPLC>>().await?;
                let mut material = endpoint.recv::<Loan<STABILITY_VIEW>>().await?;
                let bytes = imp::encode(&receipt)?;
                if receipt["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    material.send::<Passed>(&reply).await?;
                    endpoint.send::<Loan<STABILITY_VIEW>>(material).await?;
                    break;
                }
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                material.send::<Failed<REANALYSIS_FAILED>>(&reply).await?;
                endpoint.send::<Loan<STABILITY_VIEW>>(material).await?;
            }
        }
        loop {
            let arguments = endpoint.recv::<Request<STRUCTURE>>().await?.into_value();
            endpoint.send::<Loan<HPLC>>(instrument).await?;
            endpoint.recv::<Use<STRUCTURE>>().await?;
            let receipt = domain
                .execute("LEAD_SCIENTIST", "run_hplc_analysis", arguments.clone())
                .await
                .map_err(ShellError::message)?;
            imp::result(
                &receipt,
                ("LEAD_SCIENTIST", "run_hplc_analysis", &arguments),
            )?;
            instrument = endpoint.recv::<Loan<HPLC>>().await?;
            let mut material = endpoint.recv::<Loan<STRUCTURE_VIEW>>().await?;
            let bytes = imp::encode(&receipt)?;
            if receipt["success"] == true {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                material.send::<Passed>(&reply).await?;
                endpoint.send::<Loan<STRUCTURE_VIEW>>(material).await?;
                break;
            }
            let reply = ReviewReply::<false>::from_bytes(&bytes)?;
            material.send::<Failed<REANALYSIS_FAILED>>(&reply).await?;
            endpoint.send::<Loan<STRUCTURE_VIEW>>(material).await?;
        }
    };
    let regulatory = imp::artifact(&approval)?;
    let compound = approval["data"]["inputs"]["compound"]
        .as_u64()
        .ok_or_else(|| ShellError::message("approval lacks compound"))?;
    let decision = approval["data"]["inputs"]["decision"]
        .as_u64()
        .ok_or_else(|| ShellError::message("approval lacks decision"))?;
    loop {
        let arguments = endpoint
            .recv::<Request<DOCUMENTATION>>()
            .await?
            .into_value();
        let submitted = imp::decode(endpoint.recv::<ApprovalNotice>().await?)?;
        if submitted != approval {
            return Err(ShellError::message(
                "documentation names a different regulatory approval",
            ));
        }
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.recv::<Use<DOCUMENTATION>>().await?;
        let receipt = domain
            .execute("CHEMIST", "run_hplc_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("CHEMIST", "run_hplc_analysis", &arguments))?;
        imp::inputs(
            &receipt,
            &json!({"compound":compound,"decision":decision,"regulatory":regulatory}),
        )?;
        instrument = endpoint.recv::<Loan<HPLC>>().await?;
        let mut study = endpoint.recv::<Loan<STUDY>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            study.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<STUDY>>(study).await?;
            return Ok(());
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        study.send::<DocumentationFailed>(&reply).await?;
        endpoint.send::<Loan<STUDY>>(study).await?;
    }
}

pub async fn cell(
    endpoint: &mut Endpoint<'_, CELL_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
    loop {
        let arguments = endpoint.recv::<Request<CULTURE_PREP>>().await?.into_value();
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.recv::<Use<CULTURE_PREP>>().await?;
        let receipt = domain
            .execute("BIOLOGIST", "run_cell_assay", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("BIOLOGIST", "run_cell_assay", &arguments))?;
        instrument = endpoint.recv::<Loan<CELL>>().await?;
        let mut material = endpoint.recv::<Loan<CULTURE_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<CULTURE_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<CULTURE_VIEW>>(material).await?;
    }
    loop {
        let arguments = endpoint.recv::<Request<BIOASSAY>>().await?.into_value();
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.recv::<Use<BIOASSAY>>().await?;
        let receipt = domain
            .execute("BIOLOGIST", "run_cell_assay", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("BIOLOGIST", "run_cell_assay", &arguments))?;
        instrument = endpoint.recv::<Loan<CELL>>().await?;
        let mut material = endpoint.recv::<Loan<ASSAY_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<ASSAY_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<ASSAY_VIEW>>(material).await?;
    }
    loop {
        let arguments = endpoint.recv::<Request<CYTOTOXICITY>>().await?.into_value();
        endpoint.send::<Loan<CELL>>(instrument).await?;
        endpoint.recv::<Use<CYTOTOXICITY>>().await?;
        let receipt = domain
            .execute("TOXICOLOGIST", "run_cell_assay", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::result(&receipt, ("TOXICOLOGIST", "run_cell_assay", &arguments))?;
        instrument = endpoint.recv::<Loan<CELL>>().await?;
        let mut material = endpoint.recv::<Loan<TOXICITY_VIEW>>().await?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            material.send::<Passed>(&reply).await?;
            endpoint.send::<Loan<TOXICITY_VIEW>>(material).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        material.send::<Failed<ANALYSIS_FAILED>>(&reply).await?;
        endpoint.send::<Loan<TOXICITY_VIEW>>(material).await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        let plan = match branch.label() {
            ApprovedForCells::LOGICAL_LABEL => {
                let approval = imp::decode(branch.recv::<ApprovedForCells>().await?)?;
                imp::artifact(&approval)?;
                break;
            }
            0 => imp::decode(branch.recv::<Plan<0, CELL_OWNER>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, CELL_OWNER>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, CELL_OWNER>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, CELL_OWNER>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected study notice {label}"
                )));
            }
        };
        if plan["biology"] == true {
            loop {
                let arguments = endpoint.recv::<Request<BIOASSAY>>().await?.into_value();
                endpoint.send::<Loan<CELL>>(instrument).await?;
                endpoint.recv::<Use<BIOASSAY>>().await?;
                let receipt = domain
                    .execute("BIOLOGIST", "run_cell_assay", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::result(&receipt, ("BIOLOGIST", "run_cell_assay", &arguments))?;
                instrument = endpoint.recv::<Loan<CELL>>().await?;
                let mut material = endpoint.recv::<Loan<ASSAY_VIEW>>().await?;
                let bytes = imp::encode(&receipt)?;
                if receipt["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    material.send::<Passed>(&reply).await?;
                    endpoint.send::<Loan<ASSAY_VIEW>>(material).await?;
                    break;
                }
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                material.send::<Failed<REANALYSIS_FAILED>>(&reply).await?;
                endpoint.send::<Loan<ASSAY_VIEW>>(material).await?;
            }
        }
        if plan["toxicology"] == true {
            loop {
                let arguments = endpoint.recv::<Request<CYTOTOXICITY>>().await?.into_value();
                endpoint.send::<Loan<CELL>>(instrument).await?;
                endpoint.recv::<Use<CYTOTOXICITY>>().await?;
                let receipt = domain
                    .execute("TOXICOLOGIST", "run_cell_assay", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::result(&receipt, ("TOXICOLOGIST", "run_cell_assay", &arguments))?;
                instrument = endpoint.recv::<Loan<CELL>>().await?;
                let mut material = endpoint.recv::<Loan<TOXICITY_VIEW>>().await?;
                let bytes = imp::encode(&receipt)?;
                if receipt["success"] == true {
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    material.send::<Passed>(&reply).await?;
                    endpoint.send::<Loan<TOXICITY_VIEW>>(material).await?;
                    break;
                }
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                material.send::<Failed<REANALYSIS_FAILED>>(&reply).await?;
                endpoint.send::<Loan<TOXICITY_VIEW>>(material).await?;
            }
        }
    }
    Ok(())
}

pub async fn mass(
    endpoint: &mut Endpoint<'_, MASS_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<MASS>>().await?;
    {
        let arguments = endpoint.recv::<Request<BASELINE>>().await?.into_value();
        endpoint.send::<Loan<MASS>>(instrument).await?;
        endpoint.recv::<Use<BASELINE>>().await?;
        let receipt = domain
            .execute("TOXICOLOGIST", "run_mass_spec_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::actual(
            &receipt,
            ("TOXICOLOGIST", "run_mass_spec_analysis", &arguments),
        )?;
        let bytes = imp::encode(&receipt)?;
        endpoint.send::<Reply<BASELINE>>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Loan<MASS>>().await?;
    }
    {
        let arguments = endpoint.recv::<Request<REFERENCE>>().await?.into_value();
        endpoint.send::<Loan<MASS>>(instrument).await?;
        endpoint.recv::<Use<REFERENCE>>().await?;
        let receipt = domain
            .execute(
                "LEAD_SCIENTIST",
                "run_mass_spec_analysis",
                arguments.clone(),
            )
            .await
            .map_err(ShellError::message)?;
        imp::actual(
            &receipt,
            ("LEAD_SCIENTIST", "run_mass_spec_analysis", &arguments),
        )?;
        let bytes = imp::encode(&receipt)?;
        endpoint.send::<Reply<REFERENCE>>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Loan<MASS>>().await?;
    }
    {
        let arguments = endpoint.recv::<Request<MOLECULAR_ID>>().await?.into_value();
        endpoint.send::<Loan<MASS>>(instrument).await?;
        endpoint.recv::<Use<MOLECULAR_ID>>().await?;
        let receipt = domain
            .execute("CHEMIST", "run_mass_spec_analysis", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::actual(&receipt, ("CHEMIST", "run_mass_spec_analysis", &arguments))?;
        let bytes = imp::encode(&receipt)?;
        endpoint
            .send::<Reply<MOLECULAR_ID>>(&bytes.as_slice())
            .await?;
        instrument = endpoint.recv::<Loan<MASS>>().await?;
    }
    {
        let arguments = endpoint
            .recv::<Request<CHARACTERIZATION>>()
            .await?
            .into_value();
        endpoint.send::<Loan<MASS>>(instrument).await?;
        endpoint.recv::<Use<CHARACTERIZATION>>().await?;
        let receipt = domain
            .execute(
                "LEAD_SCIENTIST",
                "run_mass_spec_analysis",
                arguments.clone(),
            )
            .await
            .map_err(ShellError::message)?;
        imp::actual(
            &receipt,
            ("LEAD_SCIENTIST", "run_mass_spec_analysis", &arguments),
        )?;
        let bytes = imp::encode(&receipt)?;
        endpoint
            .send::<Reply<CHARACTERIZATION>>(&bytes.as_slice())
            .await?;
        instrument = endpoint.recv::<Loan<MASS>>().await?;
    }
    drop(endpoint.recv::<Loan<STUDY>>().await?);
    drop(instrument);
    Ok(())
}
