//! Actual neurologist referral, tests, shared findings and diagnosis.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<NEUROLOGIST>) -> ShellResult<()> {
    let assessment = json::decode(endpoint.recv::<NeuroReferral>().await?)?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    async {
        let [mut p0, mut p1, mut p2, mut p3, mut p4] = endpoint.par()?;
        async {
            p0.send::<ReferralForOwner>(&bytes.as_slice()).await?;
        };
        async {
            p1.send::<MriOrder>(&bytes.as_slice()).await?;
        };
        async {
            p2.send::<MriOrderForOwner>(&bytes.as_slice()).await?;
        };
        async {
            p3.send::<FileMriOrder>(&bytes.as_slice()).await?;
        };
        async {
            p4.send::<FileNeuroReferral>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let tests = loop {
        let mut panel = endpoint.recv::<PanelLoan<NEURO_PANEL>>().await?;
        let arguments = endpoint.recv::<NeuroAdmitted>().await?;
        panel.send::<TestRequest<3>>(&arguments.into()).await?;
        let attempt = json::decode(panel.recv::<TestResult>().await?)?;
        patient::actual_result(&attempt)?;
        endpoint.send::<PanelReturn<NEURO_PANEL>>(panel).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            PanelPartial::LOGICAL_LABEL => {
                let data = json::decode(branch.recv::<PanelPartial>().await?)?;
                if patient::panel_complete(&data, &["neuro_exam", "nerve_conduction"])? {
                    return Err(ShellError::message(
                        "partial reply contains completed panel",
                    ));
                }
            }
            PanelComplete::LOGICAL_LABEL => {
                let data = json::decode(branch.recv::<PanelComplete>().await?)?;
                if !patient::panel_complete(&data, &["neuro_exam", "nerve_conduction"])? {
                    return Err(ShellError::message("completed reply lacks actual tests"));
                }
                break data;
            }
            _ => return Err(ShellError::message("unexpected actual panel result")),
        }
    };
    let (general, other_tests, ct, mri) = async {
        let [mut g, mut other, mut c, mut m] = endpoint.par()?;
        async { json::decode(g.recv::<GeneralForNeuro>().await?) };
        async { json::decode(other.recv::<CardPanelForNeuro>().await?) };
        async { json::decode(c.recv::<CtForNeuro>().await?) };
        async { json::decode(m.recv::<MriForNeuro>().await?) };
    }
    .await?;
    json::same(&general["assessment"], &assessment)?;
    if !patient::panel_complete(&general["tests"], &["vitals", "basic_labs"])? {
        return Err(ShellError::message("general findings lack actual tests"));
    }
    if !patient::panel_complete(&other_tests, &["ecg", "cardiac_enzymes"])? {
        return Err(ShellError::message("shared specialist panel is incomplete"));
    }
    patient::actual_result(&ct)?;
    patient::actual_result(&mri)?;
    let findings =
        json!({"general":general,"own_tests":tests,"other_tests":other_tests,"CT":ct,"MRI":mri});
    let bytes = json::encode(&findings)?;
    endpoint
        .send::<FileNeuroFindings>(&bytes.as_slice())
        .await?;
    let diagnosis = json::decode(endpoint.recv::<NeuroDiagnosisResult>().await?)?;
    patient::actual_result(&diagnosis)?;
    for test in &["neuro_exam", "nerve_conduction"] {
        json::same(
            &diagnosis["data"]["inputs"][format!("test:{test}")],
            &findings["own_tests"][test]["data"]["artifact_id"],
        )?;
    }
    Ok(())
}
