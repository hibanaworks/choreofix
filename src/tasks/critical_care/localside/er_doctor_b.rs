//! Direct patient requests, completed panel custody and actual care records.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ER_B>) -> ShellResult<()> {
    let ((assessment, _panel, tests, findings, diagnosis, care), review) = async {
        let [mut clinical, mut review] = endpoint.par()?;
        async {
            let assessment = imp::decode(clinical.recv::<AssessmentResultB>().await?)?;
            imp::actual_result(&assessment, "patient_b")?;
            let (completed, tests) = loop {
                let mut panel = clinical.recv::<PanelLoan<PANEL_B>>().await?;
                let admitted = clinical.recv::<PanelAdmittedB>().await?;
                panel
                    .send::<TestRequest<3>>(&imp::EffectRequest(admitted))
                    .await?;
                let attempt = imp::decode(panel.recv::<TestResult>().await?)?;
                imp::actual_result(&attempt, "patient_b")?;
                clinical.send::<PanelReturn<PANEL_B>>(panel).await?;
                let branch = clinical.offer().await?;
                match branch.label() {
                    PanelPartial::LOGICAL_LABEL => {
                        let data = imp::decode(branch.recv::<PanelPartial>().await?)?;
                        if imp::panel_complete(&data, ("patient_b", &["stroke_panel", "ct_angio"]))?
                        {
                            return Err(ShellError::message(
                                "partial result contains complete panel",
                            ));
                        }
                    }
                    PanelCompleted::<PANEL_B>::LOGICAL_LABEL => {
                        let panel = branch.recv::<PanelCompleted<PANEL_B>>().await?;
                        let data = imp::decode(clinical.recv::<PanelComplete>().await?)?;
                        if !imp::panel_complete(
                            &data,
                            ("patient_b", &["stroke_panel", "ct_angio"]),
                        )? {
                            return Err(ShellError::message("completed panel lacks actual tests"));
                        }
                        let bytes = imp::encode(&attempt)?;
                        clinical.send::<FilePanelResultB>(&bytes.as_slice()).await?;
                        break (panel, data);
                    }
                    _ => return Err(ShellError::message("unexpected actual panel result")),
                }
            };
            let findings = imp::decode(clinical.recv::<FindingsB>().await?)?;
            imp::same(&findings["assessment"], &assessment)?;
            imp::same(&findings["tests"], &tests)?;
            let bytes = imp::encode(&findings)?;
            clinical.send::<FileFindingsB>(&bytes.as_slice()).await?;
            let diagnosis = imp::decode(clinical.recv::<DiagnosisResultB>().await?)?;
            imp::actual_result(&diagnosis, "patient_b")?;
            let care = imp::decode(clinical.recv::<IcuForER>().await?)?;
            imp::actual_result(&care, "patient_b")?;
            Ok::<_, ShellError>((assessment, completed, tests, findings, diagnosis, care))
        };
        async { imp::decode(review.recv::<ReviewForB>().await?) };
    }
    .await?;
    imp::actual_result(&review, "patient_A_and_B")?;
    let bytes = imp::encode(
        &json!({"success":true,"assessment":assessment,"tests":tests,"findings":findings,"diagnosis":diagnosis,"care":care,"prescription_review":review}),
    )?;
    endpoint.send::<SummaryB>(&bytes.as_slice()).await?;
    Ok(())
}
