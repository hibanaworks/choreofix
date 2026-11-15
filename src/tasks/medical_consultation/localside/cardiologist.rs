//! Actual referral and the specialist's independently requested test panel.
use super::super::global::*;
use crate::{
    imp::{json, patient},
    shell::{ShellError, ShellResult},
};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<CARDIOLOGIST>) -> ShellResult<()> {
    let referral = json::decode(endpoint.recv::<Referral>().await?)?;
    patient::actual_result(&referral)?;
    loop {
        let mut panel = endpoint.recv::<CardiacPanelLoan>().await?;
        let arguments = endpoint.recv::<CardiacAdmitted>().await?;
        panel.send::<CardiacTests>(&arguments).await?;
        let result = json::decode(panel.recv::<CardiacTestResults>().await?)?;
        patient::actual_result(&result)?;
        endpoint.send::<CardiacPanelReturn>(panel).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            30 => {
                let receipts = json::decode(branch.recv::<CardiacTestsPartial>().await?)?;
                if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
                    return Err(ShellError::message(
                        "partial cardiac reply contains a complete panel",
                    ));
                }
            }
            32 => {
                let receipts = json::decode(branch.recv::<CardiacTestsComplete>().await?)?;
                if !patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
                    return Err(ShellError::message(
                        "cardiac panel lacks actual completed results",
                    ));
                }
                break;
            }
            _ => return Err(ShellError::message("unexpected cardiac result")),
        }
    }
    let findings = json::decode(endpoint.recv::<GeneralFindings>().await?)?;
    json::same(&findings["assessment"], &referral)?;
    let diagnosis = json::decode(endpoint.recv::<DiagnosisResult>().await?)?;
    patient::actual_result(&diagnosis)?;
    let bytes = json::encode(&diagnosis)?;
    endpoint.send::<CardiacFindings>(&bytes.as_slice()).await?;
    Ok(())
}
