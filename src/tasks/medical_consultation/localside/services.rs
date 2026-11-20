//! Actual domain effects follow direct projected requests and CT custody.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn general<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, GENERAL_SERVICE>,
    (mut panel, actual): (Endpoint<'r, GENERAL_PANEL>, &Cell<Value>),
    effects: &D,
) -> ShellResult<()> {
    let request = endpoint.recv::<AssessmentRequest>().await?;
    let assessment = effects.apply(request).await?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    async {
        let [mut clinical, mut file] = endpoint.par()?;
        async {
            clinical.send::<AssessmentResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileAssessmentResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;

    loop {
        endpoint.send::<GeneralPanelLoan>(panel).await?;
        let admitted = endpoint.recv::<GeneralFileRequest>().await?;
        endpoint.send::<GeneralAdmitted>(&admitted).await?;
        let request = endpoint.recv::<GeneralTests>().await?;
        let arguments = json::decode(request.as_bytes())?;
        json::same(&arguments, &admitted.0.into_value())?;
        patient::tests(&arguments, &["vitals", "basic_labs"])?;
        let result = effects.apply(request).await?;
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
        if patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
            let data = serde_json::json!({"tests":receipts,"attempt":result});
            let bytes = json::encode(&data)?;
            endpoint
                .send::<GeneralCompletedData>(&bytes.as_slice())
                .await?;
            let tests = json::encode(&receipts)?;
            endpoint
                .send::<GeneralTestsComplete>(&tests.as_slice())
                .await?;
            break;
        }
        let bytes = json::encode(&result)?;
        endpoint
            .send::<FileGeneralResult>(&bytes.as_slice())
            .await?;
        let bytes = json::encode(&receipts)?;
        endpoint
            .send::<GeneralTestsPartial>(&bytes.as_slice())
            .await?;
    }
    endpoint.send::<GeneralPanelCompleted>(panel).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn cardiac<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, CARDIAC_SERVICE>,
    (mut panel, actual): (Endpoint<'r, CARDIAC_PANEL>, &Cell<Value>),
    effects: &D,
) -> ShellResult<()> {
    let referral = json::decode(endpoint.recv::<ReferralForOwner>().await?)?;
    patient::actual_result(&referral)?;
    loop {
        endpoint.send::<CardiacPanelLoan>(panel).await?;
        let admitted = endpoint.recv::<CardiacFileRequest>().await?;
        endpoint.send::<CardiacAdmitted>(&admitted).await?;
        let request = endpoint.recv::<CardiacTests>().await?;
        let arguments = json::decode(request.as_bytes())?;
        json::same(&arguments, &admitted.0.into_value())?;
        patient::tests(&arguments, &["ecg", "cardiac_enzymes"])?;
        let result = effects.apply(request).await?;
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
        if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
            let data = serde_json::json!({"tests":receipts,"attempt":result});
            let bytes = json::encode(&data)?;
            endpoint
                .send::<CardiacCompletedData>(&bytes.as_slice())
                .await?;
            let tests = json::encode(&receipts)?;
            async {
                let [mut specialist, mut doctor] = endpoint.par()?;
                async {
                    specialist
                        .send::<CardiacTestsComplete>(&tests.as_slice())
                        .await?;
                };
                async {
                    doctor.send::<CardiacPanelReport>(&tests.as_slice()).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await?;
            break;
        }
        let bytes = json::encode(&result)?;
        endpoint
            .send::<FileCardiacResult>(&bytes.as_slice())
            .await?;
        let bytes = json::encode(&receipts)?;
        endpoint
            .send::<CardiacTestsPartial>(&bytes.as_slice())
            .await?;
    }
    endpoint.send::<CardiacPanelCompleted>(panel).await?;

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
    let request = endpoint.recv::<DiagnosisRequest>().await?;
    let result = effects.apply(request).await?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    async {
        let [mut clinical, mut file] = endpoint.par()?;
        async {
            clinical.send::<DiagnosisResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileDiagnosisResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn pharmacy<D: ReceivedEffects>(
    endpoint: &mut Endpoint<PHARMACY_SERVICE>,
    effects: &D,
) -> ShellResult<()> {
    let request = endpoint.recv::<ReviewRequest>().await?;
    let result = effects.apply(request).await?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    async {
        let [mut clinical, mut file] = endpoint.par()?;
        async {
            clinical.send::<ReviewResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileReviewResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn imaging<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, IMAGING_SERVICE>,
    ct: Endpoint<'r, CT_SCANNER>,
    effects: &D,
) -> ShellResult<()> {
    let request = endpoint.recv::<ImagingIntent>().await?;
    endpoint.send::<ImagingAdmitted>(&request).await?;
    let admitted = request.0.into_value();
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|value| !value.eq_ignore_ascii_case("CT"))
    {
        return Err(ShellError::message("expected CT imaging"));
    }
    endpoint.send::<CtLoan>(ct).await?;
    let request = endpoint.recv::<ImagingRequest>().await?;
    let arguments = json::decode(request.as_bytes())?;
    json::same(&arguments, &admitted)?;
    let result = effects.apply(request).await?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    async {
        let [mut resource, mut file] = endpoint.par()?;
        async {
            resource.send::<ImagingResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileImagingResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let _ct = endpoint.recv::<CtLoan>().await?;
    Ok(())
}
