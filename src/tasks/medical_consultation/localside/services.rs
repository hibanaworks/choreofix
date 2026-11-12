//! Actual domain effects follow direct projected requests and CT custody.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn general<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, GENERAL_SERVICE>,
    (mut panel, actual): (Endpoint<'r, GENERAL_PANEL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let arguments = json::decode(endpoint.recv::<AssessmentRequest>().await?)?;
    patient::patient(&arguments)?;
    let assessment = domain
        .execute("ER_DOCTOR", "assess_patient", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    endpoint.send::<AssessmentResult>(&bytes.as_slice()).await?;

    loop {
        endpoint.send::<GeneralPanelLoan>(panel).await?;
        let arguments = json::decode(endpoint.recv::<GeneralTests>().await?)?;
        patient::tests(&arguments, &["vitals", "basic_labs"])?;
        let result = domain
            .execute("ER_DOCTOR", "order_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        patient::actual_result(&result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        let mut receipts = actual.take();
        let update = patient::record_panel(&mut receipts, &result, &["vitals", "basic_labs"]);
        actual.set(receipts.clone());
        update?;
        let bytes = json::encode(&result)?;
        endpoint
            .send::<GeneralTestResults>(&bytes.as_slice())
            .await?;
        panel = endpoint.recv::<GeneralPanelReturn>().await?;
        let bytes = json::encode(&receipts)?;
        if patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
            endpoint.send::<GeneralPanelCompleted>(panel).await?;
            endpoint
                .send::<GeneralTestsComplete>(&bytes.as_slice())
                .await?;
            break;
        }
        endpoint
            .send::<GeneralTestsPartial>(&bytes.as_slice())
            .await?;
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn cardiac<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CARDIAC_SERVICE>,
    (mut panel, actual): (Endpoint<'r, CARDIAC_PANEL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let referral = json::decode(endpoint.recv::<ReferralForOwner>().await?)?;
    patient::actual_result(&referral)?;
    loop {
        endpoint.send::<CardiacPanelLoan>(panel).await?;
        let arguments = json::decode(endpoint.recv::<CardiacTests>().await?)?;
        patient::tests(&arguments, &["ecg", "cardiac_enzymes"])?;
        let result = domain
            .execute("CARDIOLOGIST", "order_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        patient::actual_result(&result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &referral["data"]["artifact_id"],
        )?;
        let mut receipts = actual.take();
        let update = patient::record_panel(&mut receipts, &result, &["ecg", "cardiac_enzymes"]);
        actual.set(receipts.clone());
        update?;
        let bytes = json::encode(&result)?;
        endpoint
            .send::<CardiacTestResults>(&bytes.as_slice())
            .await?;
        panel = endpoint.recv::<CardiacPanelReturn>().await?;
        let bytes = json::encode(&receipts)?;
        if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
            endpoint.send::<CardiacPanelCompleted>(panel).await?;
            endpoint
                .send::<CardiacTestsComplete>(&bytes.as_slice())
                .await?;
            break;
        }
        endpoint
            .send::<CardiacTestsPartial>(&bytes.as_slice())
            .await?;
    }

    let findings = json::decode(endpoint.recv::<FindingsForOwner>().await?)?;
    json::same(&findings["assessment"], &referral)?;
    if !patient::panel_complete(&findings["tests"], &["vitals", "basic_labs"])? {
        return Err(ShellError::message("general panel is incomplete"));
    }
    json::same(
        &findings["tests"]["vitals"]["data"]["inputs"]["assessment"],
        &referral["data"]["artifact_id"],
    )?;
    patient::actual_result(&findings["imaging"])?;
    let arguments = json::decode(endpoint.recv::<DiagnosisRequest>().await?)?;
    patient::patient(&arguments)?;
    if arguments
        .get("symptoms")
        .is_some_and(|value| !value.is_string())
    {
        return Err(ShellError::message("symptoms must be a string"));
    }
    let result = domain
        .execute("CARDIOLOGIST", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    endpoint.send::<DiagnosisResult>(&bytes.as_slice()).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn pharmacy<D: Domain>(
    endpoint: &mut Endpoint<PHARMACY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let arguments = json::decode(endpoint.recv::<ReviewRequest>().await?)?;
    patient::patient(&arguments)?;
    if arguments
        .get("medications")
        .is_some_and(|value| !value.is_string())
    {
        return Err(ShellError::message("medications must be a string"));
    }
    let result = domain
        .execute("PHARMACIST", "review_prescriptions", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    endpoint.send::<ReviewResult>(&bytes.as_slice()).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn imaging<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, IMAGING_SERVICE>,
    ct: Endpoint<'r, CT_SCANNER>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = json::decode(endpoint.recv::<ImagingIntent>().await?)?;
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|value| !value.eq_ignore_ascii_case("CT"))
    {
        return Err(ShellError::message("expected CT imaging"));
    }
    endpoint.send::<CtLoan>(ct).await?;
    let arguments = json::decode(endpoint.recv::<ImagingRequest>().await?)?;
    json::same(&arguments, &admitted)?;
    let result = domain
        .execute("ER_DOCTOR", "perform_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    endpoint.send::<ImagingResult>(&bytes.as_slice()).await?;
    let _ct = endpoint.recv::<CtLoan>().await?;
    Ok(())
}
