//! Physical imaging owners; requests are checked before the actual effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn ct<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, CT_SERVICE>,
    mut instrument: Endpoint<'r, CT_SCANNER>,
    effects: &D,
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
                let admitted = branch.recv::<CtPerformIntentA>().await?;

                endpoint.send::<CtAssessmentQueryA>(&admitted).await?;
                assessment_a = imp::decode(endpoint.recv::<CtAssessmentA>().await?)?;
                imp::actual_result(&assessment_a, "patient_a")?;
                endpoint.send::<CtPerformAdmittedA>(&admitted).await?;
                let admitted = admitted.into_value();
                endpoint.send::<CtLoan>(instrument).await?;
                let request = endpoint.recv::<CtPerformRequestA>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted)?;
                let result = effects.apply(request).await?;
                imp::actual_result(&result, "patient_a")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_a["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                async {
                    let [mut resource, mut file] = endpoint.par()?;
                    async {
                        resource.send::<CtImageA>(&bytes.as_slice()).await?;
                    };
                    async {
                        file.send::<FileCtImageA>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                image_a = result;
            }
            CtInterpretIntentA::LOGICAL_LABEL => {
                let admitted = branch.recv::<CtInterpretIntentA>().await?;

                imp::actual_result(&image_a, "patient_a")?;
                endpoint.send::<CtInterpretAdmittedA>(&admitted).await?;
                let admitted = admitted.into_value();
                endpoint.send::<CtLoan>(instrument).await?;
                let request = endpoint.recv::<CtInterpretRequestA>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted)?;
                let result = effects.apply(request).await?;
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
                async {
                    let [mut resource, mut file] = endpoint.par()?;
                    async {
                        resource.send::<CtReportA>(&bytes.as_slice()).await?;
                    };
                    async {
                        file.send::<FileCtReportA>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                endpoint.send::<CtFindingsA>(&bytes.as_slice()).await?;
            }
            CtPerformIntentB::LOGICAL_LABEL => {
                let admitted = branch.recv::<CtPerformIntentB>().await?;

                endpoint.send::<CtAssessmentQueryB>(&admitted).await?;
                assessment_b = imp::decode(endpoint.recv::<CtAssessmentB>().await?)?;
                imp::actual_result(&assessment_b, "patient_b")?;
                endpoint.send::<CtPerformAdmittedB>(&admitted).await?;
                let admitted = admitted.into_value();
                endpoint.send::<CtLoan>(instrument).await?;
                let request = endpoint.recv::<CtPerformRequestB>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted)?;
                let result = effects.apply(request).await?;
                imp::actual_result(&result, "patient_b")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment_b["data"]["artifact_id"],
                )?;
                let bytes = imp::encode(&result)?;
                async {
                    let [mut resource, mut file] = endpoint.par()?;
                    async {
                        resource.send::<CtImageB>(&bytes.as_slice()).await?;
                    };
                    async {
                        file.send::<FileCtImageB>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                image_b = result;
            }
            CtInterpretIntentB::LOGICAL_LABEL => {
                let admitted = branch.recv::<CtInterpretIntentB>().await?;

                imp::actual_result(&image_b, "patient_b")?;
                endpoint.send::<CtInterpretAdmittedB>(&admitted).await?;
                let admitted = admitted.into_value();
                endpoint.send::<CtLoan>(instrument).await?;
                let request = endpoint.recv::<CtInterpretRequestB>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted)?;
                let result = effects.apply(request).await?;
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
                async {
                    let [mut resource, mut file] = endpoint.par()?;
                    async {
                        resource.send::<CtReportB>(&bytes.as_slice()).await?;
                    };
                    async {
                        file.send::<FileCtReportB>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                instrument = endpoint.recv::<CtLoan>().await?;
                endpoint.send::<CtFindingsB>(&bytes.as_slice()).await?;
            }
            _ => return Err(ShellError::message("unexpected CT request")),
        }
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn mri<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, MRI_SERVICE>,
    mut instrument: Endpoint<'r, MRI>,
    effects: &D,
) -> ShellResult<()> {
    let admitted = endpoint.recv::<MriPerformIntent>().await?;

    endpoint.send::<MriAssessmentQuery>(&admitted).await?;
    let assessment = imp::decode(endpoint.recv::<MriAssessment>().await?)?;
    imp::actual_result(&assessment, "patient_b")?;
    endpoint.send::<MriPerformAdmitted>(&admitted).await?;
    let admitted = admitted.into_value();
    endpoint.send::<MriLoan>(instrument).await?;
    let request = endpoint.recv::<MriPerformRequest>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::same(&arguments, &admitted)?;
    let image = effects.apply(request).await?;
    imp::actual_result(&image, "patient_b")?;
    imp::same(
        &image["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&image)?;
    async {
        let [mut resource, mut file] = endpoint.par()?;
        async {
            resource.send::<MriImage>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileMriImage>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    instrument = endpoint.recv::<MriLoan>().await?;

    let admitted = endpoint.recv::<MriInterpretIntent>().await?;

    endpoint.send::<MriInterpretAdmitted>(&admitted).await?;
    let admitted = admitted.into_value();
    endpoint.send::<MriLoan>(instrument).await?;
    let request = endpoint.recv::<MriInterpretRequest>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::same(&arguments, &admitted)?;
    let report = effects.apply(request).await?;
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
    async {
        let [mut resource, mut file] = endpoint.par()?;
        async {
            resource.send::<MriReport>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileMriReport>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let _instrument = endpoint.recv::<MriLoan>().await?;
    endpoint.send::<MriFindingsB>(&bytes.as_slice()).await?;
    Ok(())
}
