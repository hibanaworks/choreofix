//! Actual assessment, independent resource operations and final findings.
use super::super::global::*;
use crate::{
    imp::{json, patient},
    shell::{ShellError, ShellResult},
};
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ER_DOCTOR>) -> ShellResult<()> {
    let (clinical, medication) = async {
        let [mut clinical, mut pharmacy] = endpoint.par()?;
        async {
            let assessment = json::decode(clinical.recv::<AssessmentResult>().await?)?;
            patient::actual_result(&assessment)?;
            let (tests, image, cardiac_tests) = async {
                let [mut tests, mut scan, mut referral] = clinical.par()?;
                async {
                    loop {
                        let mut panel = tests.recv::<GeneralPanelLoan>().await?;
                        let arguments = tests.recv::<GeneralAdmitted>().await?;
                        panel.send::<GeneralTests>(&arguments.0).await?;
                        let result = json::decode(panel.recv::<GeneralTestResults>().await?)?;
                        patient::actual_result(&result)?;
                        tests.send::<GeneralPanelReturn>(panel).await?;
                        let branch = tests.offer().await?;
                        match branch.label() {
                            22 => {
                                let receipts =
                                    json::decode(branch.recv::<GeneralTestsPartial>().await?)?;
                                if patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
                                    return Err(ShellError::message(
                                        "partial general reply contains a complete panel",
                                    ));
                                }
                            }
                            26 => {
                                let receipts=json::decode(branch.recv::<GeneralTestsComplete>().await?)?;
                                if !patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
                                    return Err(ShellError::message(
                                        "general panel lacks actual completed results",
                                    ));
                                }
                                break Ok::<_, ShellError>(receipts);
                            }
                            _ => return Err(ShellError::message("unexpected general result")),
                        }
                    }
                };
                async {
                    let arguments = scan.recv::<ImagingAdmitted>().await?;
                    let mut ct = scan.recv::<CtLoan>().await?;
                    ct.send::<ImagingRequest>(&arguments.0).await?;
                    let image = json::decode(ct.recv::<ImagingResult>().await?)?;
                    patient::actual_result(&image)?;
                    scan.send::<CtLoan>(ct).await?;
                    Ok::<_, ShellError>(image)
                };
                async {
                    let bytes = json::encode(&assessment)?;
                    async {
                        let [mut doctor, mut owner, mut file] = referral.par()?;
                        async {
                            doctor.send::<Referral>(&bytes.as_slice()).await?;
                        };
                        async {
                            owner.send::<ReferralForOwner>(&bytes.as_slice()).await?;
                        };
                        async {
                            file.send::<FileReferral>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    json::decode(referral.recv::<CardiacPanelReport>().await?)
                };
            }
            .await?;
            let findings = json!({"assessment":assessment,"tests":tests,"imaging":image,"cardiac_tests":cardiac_tests});
            let bytes = json::encode(&findings)?;
            async {
                let [mut doctor, mut owner, mut file] = clinical.par()?;
                async {
                    doctor.send::<GeneralFindings>(&bytes.as_slice()).await?;
                };
                async {
                    owner.send::<FindingsForOwner>(&bytes.as_slice()).await?;
                };
                async {
                    file.send::<FileFindings>(&bytes.as_slice()).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await?;
            let diagnosis = json::decode(clinical.recv::<CardiacFindings>().await?)?;
            patient::actual_result(&diagnosis)?;
            Ok::<_, ShellError>(json!({"findings":findings,"diagnosis":diagnosis}))
        };
        async {
            let result = json::decode(pharmacy.recv::<MedicationReview>().await?)?;
            patient::actual_result(&result)?;
            Ok::<_, ShellError>(result)
        };
    }
    .await?;
    let summary = json!({"clinical":clinical,"medication_review":medication});
    let bytes = json::encode(&summary)?;
    endpoint
        .send::<CompletedConsultation>(&bytes.as_slice())
        .await?;
    Ok(())
}
