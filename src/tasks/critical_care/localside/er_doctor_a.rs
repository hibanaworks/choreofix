//! Direct patient requests, completed panel custody and actual care records.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ER_A>) -> ShellResult<()> {
    let ((assessment, _panel, tests, findings, diagnosis, care), review) = async {
        let [mut clinical, mut review] = endpoint.par()?;
        async {
            let assessment = imp::decode(clinical.recv::<AssessmentResultA>().await?)?;
            imp::actual_result(&assessment, "patient_a")?;
            let (completed, tests) = loop {
                let mut panel = clinical.recv::<PanelLoan<PANEL_A>>().await?;
                let admitted = clinical.recv::<PanelAdmittedA>().await?;
                panel.send::<TestRequest<2>>(&admitted).await?;
                let attempt = imp::decode(panel.recv::<TestResult>().await?)?;
                imp::actual_result(&attempt, "patient_a")?;
                clinical.send::<PanelReturn<PANEL_A>>(panel).await?;
                let branch = clinical.offer().await?;
                match branch.label() {
                    PanelPartial::LOGICAL_LABEL => {
                        let data = imp::decode(branch.recv::<PanelPartial>().await?)?;
                        if imp::panel_complete(
                            &data,
                            ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                        )? {
                            return Err(ShellError::message(
                                "partial result contains complete panel",
                            ));
                        }
                    }
                    PanelCompleted::<PANEL_A>::LOGICAL_LABEL => {
                        let panel = branch.recv::<PanelCompleted<PANEL_A>>().await?;
                        let data = imp::decode(clinical.recv::<PanelComplete>().await?)?;
                        if !imp::panel_complete(
                            &data,
                            ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                        )? {
                            return Err(ShellError::message("completed panel lacks actual tests"));
                        }
                        let bytes = imp::encode(&attempt)?;
                        clinical.send::<FilePanelResultA>(&bytes.as_slice()).await?;
                        break (panel, data);
                    }
                    _ => return Err(ShellError::message("unexpected actual panel result")),
                }
            };
            let findings = imp::decode(clinical.recv::<FindingsA>().await?)?;
            imp::same(&findings["assessment"], &assessment)?;
            imp::same(&findings["tests"], &tests)?;
            let bytes = imp::encode(&findings)?;
            clinical.send::<FileFindingsA>(&bytes.as_slice()).await?;
            let diagnosis = imp::decode(clinical.recv::<DiagnosisResultA>().await?)?;
            imp::actual_result(&diagnosis, "patient_a")?;
            let care = imp::decode(clinical.recv::<SurgeryForER>().await?)?;
            imp::actual_result(&care, "patient_a")?;
            Ok::<_, ShellError>((assessment, completed, tests, findings, diagnosis, care))
        };
        async { imp::decode(review.recv::<ReviewForA>().await?) };
    }
    .await?;
    imp::actual_result(&review, "patient_A_and_B")?;
    let bytes = imp::encode(
        &json!({"success":true,"assessment":assessment,"tests":tests,"findings":findings,"diagnosis":diagnosis,"care":care,"prescription_review":review}),
    )?;
    endpoint.send::<SummaryA>(&bytes.as_slice()).await?;
    Ok(())
}
