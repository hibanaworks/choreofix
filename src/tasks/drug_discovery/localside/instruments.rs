//! Physical instrument owners receive and compare actual data before effects.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::json;
pub async fn hplc<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, HPLC_SERVICE>,
    mut instrument: Endpoint<'r, HPLC>,
    domain: &D,
) -> ShellResult<()> {
    let qc = {
        let admitted = imp::decode(endpoint.recv::<HplcIntent>().await?)?;
        imp::assigned(&admitted, "sample", "synthesis_qc")?;
        endpoint.send::<HplcLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<HplcRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_hplc_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<HplcResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<HplcLoan>().await?;
        result
    };
    let purity = {
        let admitted = imp::decode(endpoint.recv::<HplcIntent>().await?)?;
        imp::assigned(&admitted, "sample", "purity_check")?;
        endpoint.send::<HplcLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<HplcRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("TOXICOLOGIST", "run_hplc_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<HplcResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<HplcLoan>().await?;
        result
    };
    let stability = {
        let admitted = imp::decode(endpoint.recv::<HplcIntent>().await?)?;
        imp::assigned(&admitted, "sample", "stability")?;
        endpoint.send::<HplcLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<HplcRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_hplc_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<HplcResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<HplcLoan>().await?;
        result
    };
    let structural = {
        let admitted = imp::decode(endpoint.recv::<HplcIntent>().await?)?;
        imp::assigned(&admitted, "sample", "structural_confirmation")?;
        endpoint.send::<HplcLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<HplcRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("LEAD_SCIENTIST", "run_hplc_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<HplcResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<HplcLoan>().await?;
        result
    };
    let mut decision = imp::decode(endpoint.recv::<DecisionForOwner>().await?)?;
    if !decision["go"].is_boolean() {
        return Err(ShellError::message("decision has no actual go/no-go value"));
    }
    imp::same(&decision["compound"]["qc"], &qc)?;
    imp::same(&decision["bio"]["stability"], &stability)?;
    imp::same(&decision["tox"]["purity"], &purity)?;
    imp::same(&decision["analyses"]["structural"], &structural)?;
    let submitted = domain
        .execute(
            "LEAD_SCIENTIST",
            "submit_decision",
            json!({"go":decision["go"]}),
        )
        .await
        .map_err(ShellError::message)?;
    imp::decision_receipt(&decision, &submitted)?;
    let bytes = imp::encode(&submitted)?;
    endpoint
        .send::<DecisionSubmitted>(&bytes.as_slice())
        .await?;
    decision["receipt"] = submitted;
    let admitted = imp::decode(endpoint.recv::<HplcIntent>().await?)?;
    imp::assigned(&admitted["arguments"], "sample", "documentation")?;
    imp::same(&admitted["decision"], &decision)?;
    endpoint.send::<HplcLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<HplcRequest>().await?)?;
    imp::same(&request, &admitted)?;
    let result = domain
        .execute("CHEMIST", "run_hplc_analysis", request["arguments"].clone())
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<HplcResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<HplcLoan>().await?;
    drop(instrument);
    Ok(())
}
pub async fn mass<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, MASS_SERVICE>,
    mut instrument: Endpoint<'r, MASS_SPEC>,
    domain: &D,
) -> ShellResult<()> {
    let _result = {
        let admitted = imp::decode(endpoint.recv::<MassIntent>().await?)?;
        imp::assigned(&admitted, "sample", "baseline_markers")?;
        endpoint.send::<MassLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("TOXICOLOGIST", "run_mass_spec_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<MassLoan>().await?;
        result
    };
    let _result = {
        let admitted = imp::decode(endpoint.recv::<MassIntent>().await?)?;
        imp::assigned(&admitted, "sample", "reference_methods")?;
        endpoint.send::<MassLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("LEAD_SCIENTIST", "run_mass_spec_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<MassLoan>().await?;
        result
    };
    let _result = {
        let admitted = imp::decode(endpoint.recv::<MassIntent>().await?)?;
        imp::assigned(&admitted, "sample", "molecular_id")?;
        endpoint.send::<MassLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_mass_spec_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<MassLoan>().await?;
        result
    };
    let _result = {
        let admitted = imp::decode(endpoint.recv::<MassIntent>().await?)?;
        imp::assigned(&admitted, "sample", "characterization")?;
        endpoint.send::<MassLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("LEAD_SCIENTIST", "run_mass_spec_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<MassLoan>().await?;
        result
    };
    drop(instrument);
    Ok(())
}
pub async fn cells<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CELL_SERVICE>,
    instrument: Endpoint<'r, CELL_LAB>,
    domain: &D,
) -> ShellResult<()> {
    endpoint.send::<CellLoan>(instrument).await?;
    let _prep = {
        let admitted = imp::decode(endpoint.recv::<CellIntent>().await?)?;
        imp::assigned(&admitted, "assay_type", "culture_prep")?;

        let request = imp::decode(endpoint.recv::<CellRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_cell_assay", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<CellResult>(&bytes.as_slice()).await?;

        result
    };
    let _assay = {
        let admitted = imp::decode(endpoint.recv::<CellIntent>().await?)?;
        imp::assigned(&admitted, "assay_type", "bioassay")?;

        let request = imp::decode(endpoint.recv::<CellRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_cell_assay", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<CellResult>(&bytes.as_slice()).await?;

        result
    };
    let instrument = endpoint.recv::<CellLoan>().await?;
    endpoint.send::<CellLoan>(instrument).await?;
    let _tox = {
        let admitted = imp::decode(endpoint.recv::<CellIntent>().await?)?;
        imp::assigned(&admitted, "assay_type", "cytotoxicity")?;

        let request = imp::decode(endpoint.recv::<CellRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("TOXICOLOGIST", "run_cell_assay", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<CellResult>(&bytes.as_slice()).await?;

        result
    };
    let instrument = endpoint.recv::<CellLoan>().await?;
    drop(instrument);
    Ok(())
}
