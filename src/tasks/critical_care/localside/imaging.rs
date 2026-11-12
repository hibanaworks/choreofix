//! Physical imaging owners; requests are checked before the actual effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn ct<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CT_SERVICE>,
    mut instrument: Endpoint<'r, CT_SCANNER>,
    domain: &D,
) -> ShellResult<()> {
    let mut assessment_a = Value::Null;
    let mut assessment_b = Value::Null;
    let mut image_a = Value::Null;
    let mut image_b = Value::Null;
    // Four actual requests; their legal order is selected by the global's routes.
    for _ in 0..4 {
        let branch = endpoint.offer().await?;
        match branch.label() {
            CtPerformIntentA::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<CtPerformIntentA>().await?)?;
                imp::imaging(&admitted, ("patient_a", "CT"))?;
                let bytes = imp::encode(&admitted)?;
                endpoint
                    .send::<CtAssessmentQueryA>(&bytes.as_slice())
                    .await?;
                assessment_a = imp::decode(endpoint.recv::<CtAssessmentA>().await?)?;
                imp::actual_result(&assessment_a, "patient_a")?;
                endpoint.send::<CtLoan>(instrument).await?;
                let arguments = imp::decode(endpoint.recv::<CtPerformRequestA>().await?)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("RADIOLOGIST", "perform_imaging", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::actual_result(&result, "patient_a")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_a["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CtImageA>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                image_a = result;
            }
            CtInterpretIntentA::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<CtInterpretIntentA>().await?)?;
                imp::imaging(&admitted, ("patient_a", "CT"))?;
                imp::actual_result(&image_a, "patient_a")?;
                endpoint.send::<CtLoan>(instrument).await?;
                let arguments = imp::decode(endpoint.recv::<CtInterpretRequestA>().await?)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("RADIOLOGIST", "interpret_imaging", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::actual_result(&result, "patient_a")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_a["data"]["artifact_id"],
                )?;
                imp::same(
                    &result["data"]["inputs"]["image"],
                    &image_a["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CtReportA>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                endpoint.send::<CtFindingsA>(&bytes.as_slice()).await?;
            }
            CtPerformIntentB::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<CtPerformIntentB>().await?)?;
                imp::imaging(&admitted, ("patient_b", "CT"))?;
                let bytes = imp::encode(&admitted)?;
                endpoint
                    .send::<CtAssessmentQueryB>(&bytes.as_slice())
                    .await?;
                assessment_b = imp::decode(endpoint.recv::<CtAssessmentB>().await?)?;
                imp::actual_result(&assessment_b, "patient_b")?;
                endpoint.send::<CtLoan>(instrument).await?;
                let arguments = imp::decode(endpoint.recv::<CtPerformRequestB>().await?)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("RADIOLOGIST", "perform_imaging", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::actual_result(&result, "patient_b")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_b["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CtImageB>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                image_b = result;
            }
            CtInterpretIntentB::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<CtInterpretIntentB>().await?)?;
                imp::imaging(&admitted, ("patient_b", "CT"))?;
                imp::actual_result(&image_b, "patient_b")?;
                endpoint.send::<CtLoan>(instrument).await?;
                let arguments = imp::decode(endpoint.recv::<CtInterpretRequestB>().await?)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute("RADIOLOGIST", "interpret_imaging", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::actual_result(&result, "patient_b")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_b["data"]["artifact_id"],
                )?;
                imp::same(
                    &result["data"]["inputs"]["image"],
                    &image_b["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<CtReportB>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                endpoint.send::<CtFindingsB>(&bytes.as_slice()).await?;
            }
            _ => return Err(ShellError::message("unexpected CT request")),
        }
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn mri<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, MRI_SERVICE>,
    mut instrument: Endpoint<'r, MRI>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = imp::decode(endpoint.recv::<MriPerformIntent>().await?)?;
    imp::imaging(&admitted, ("patient_b", "MRI"))?;
    let bytes = imp::encode(&admitted)?;
    endpoint
        .send::<MriAssessmentQuery>(&bytes.as_slice())
        .await?;
    let assessment = imp::decode(endpoint.recv::<MriAssessment>().await?)?;
    imp::actual_result(&assessment, "patient_b")?;
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = imp::decode(endpoint.recv::<MriPerformRequest>().await?)?;
    imp::same(&arguments, &admitted)?;
    let image = domain
        .execute("RADIOLOGIST", "perform_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&image, "patient_b")?;
    imp::same(
        &image["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&image)?;
    endpoint.send::<MriImage>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<MriLoan>().await?;

    let admitted = imp::decode(endpoint.recv::<MriInterpretIntent>().await?)?;
    imp::imaging(&admitted, ("patient_b", "MRI"))?;
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = imp::decode(endpoint.recv::<MriInterpretRequest>().await?)?;
    imp::same(&arguments, &admitted)?;
    let report = domain
        .execute("RADIOLOGIST", "interpret_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&report, "patient_b")?;
    imp::same(
        &report["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    imp::same(
        &report["data"]["inputs"]["image"],
        &image["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&report)?;
    endpoint.send::<MriReport>(&bytes.as_slice()).await?;
    let _instrument = endpoint.recv::<MriLoan>().await?;
    endpoint.send::<MriFindingsB>(&bytes.as_slice()).await?;
    Ok(())
}
