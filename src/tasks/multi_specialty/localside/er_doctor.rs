//! Direct ER investigation, actual case data and one conference request.
use super::super::{global::*, imp::Arguments};
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ER_DOCTOR>) -> ShellResult<()> {
    let ((assessment, tests, ct, mri, cardiac, neuro), review) = async {
        let [mut clinical, mut pharmacy] = endpoint.par()?;
        async {
            let assessment = json::decode(clinical.recv::<AssessmentResult>().await?)?;
            patient::actual_result(&assessment)?;
            let (tests, (), ()) = async {
                let [mut own, mut cardiac, mut neuro] = clinical.par()?;
                async {
                    let tests = loop {
                        let mut panel = own.recv::<PanelLoan<GENERAL_PANEL>>().await?;
                        let arguments = own.recv::<GeneralAdmitted>().await?;
                        panel.send::<TestRequest<1>>(&arguments.into()).await?;
                        let attempt = json::decode(panel.recv::<TestResult>().await?)?;
                        patient::actual_result(&attempt)?;
                        own.send::<PanelReturn<GENERAL_PANEL>>(panel).await?;
                        let branch = own.offer().await?;
                        match branch.label() {
                            PanelPartial::LOGICAL_LABEL => {
                                let data = json::decode(branch.recv::<PanelPartial>().await?)?;
                                if patient::panel_complete(&data, &["vitals", "basic_labs"])? {
                                    return Err(ShellError::message(
                                        "partial reply contains completed panel",
                                    ));
                                }
                            }
                            PanelComplete::LOGICAL_LABEL => {
                                let data = json::decode(branch.recv::<PanelComplete>().await?)?;
                                if !patient::panel_complete(&data, &["vitals", "basic_labs"])? {
                                    return Err(ShellError::message(
                                        "completed reply lacks actual tests",
                                    ));
                                }
                                break data;
                            }
                            _ => return Err(ShellError::message("unexpected actual panel result")),
                        }
                    };
                    Ok::<_, ShellError>(tests)
                };
                async {
                    let bytes = json::encode(&assessment)?;
                    cardiac.send::<CardReferral>(&bytes.as_slice()).await?;
                };
                async {
                    let bytes = json::encode(&assessment)?;
                    neuro.send::<NeuroReferral>(&bytes.as_slice()).await?;
                };
            }
            .await?;
            let (ct, mri) = async {
                let [mut c, mut m] = clinical.par()?;
                async { json::decode(c.recv::<CtForER>().await?) };
                async { json::decode(m.recv::<MriForER>().await?) };
            }
            .await?;
            let (cardiac, neuro) = async {
                let [mut c, mut n] = clinical.par()?;
                async { json::decode(c.recv::<CardDiagnosisForER>().await?) };
                async { json::decode(n.recv::<NeuroDiagnosisForER>().await?) };
            }
            .await?;
            for report in [&ct, &mri, &cardiac["result"], &neuro["result"]] {
                patient::actual_result(report)?;
            }
            Ok::<_, ShellError>((assessment, tests, ct, mri, cardiac, neuro))
        };
        async { json::decode(pharmacy.recv::<ReviewForER>().await?) };
    }
    .await?;
    patient::actual_result(&review)?;
    let case = json!({"assessment":assessment,"general":tests,"CT":ct,"MRI":mri,"cardiac":cardiac,"neurology":neuro,"review":review});
    let bytes = json::encode(&case)?;
    endpoint.send::<Candidate>(&bytes.as_slice()).await?;
    let arguments = endpoint.recv::<ConferenceAdmitted>().await?.into_value();
    let mut room = endpoint.recv::<RoomLoan>().await?;
    let request = Arguments::<12>::from_value(json!({"case":case,"arguments":arguments}))?;
    room.send::<RoomRequest>(&request).await?;
    let result = json::decode(room.recv::<RoomResult>().await?)?;
    patient::actual_result(&result)?;
    endpoint.send::<RoomLoan>(room).await?;
    Ok(())
}
