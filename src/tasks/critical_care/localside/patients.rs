//! Actual clinical data owners; all communication is visible at its effect boundary.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn patient_a<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, PATIENT_A_SERVICE>,
    (mut panel, actual): (Endpoint<'r, PANEL_A>, &Cell<Value>),
    effects: &D,
) -> ShellResult<()> {
    let request = endpoint.recv::<AssessmentRequestA>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::patient(&arguments, "patient_a")?;
    let assessment = effects.apply(request).await?;
    imp::actual_result(&assessment, "patient_a")?;
    let bytes = imp::encode(&assessment)?;
    async {
        let [mut clinical, mut file] = endpoint.par()?;
        async {
            clinical
                .send::<AssessmentResultA>(&bytes.as_slice())
                .await?;
        };
        async {
            file.send::<FileAssessmentA>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let (receipts, ct) = async {
        let [mut tests, mut ct] = endpoint.par()?;
        async {
            loop {
                tests.send::<PanelLoan<PANEL_A>>(panel).await?;
                let admitted = tests.recv::<PanelFileRequestA>().await?;
                tests.send::<PanelAdmittedA>(&admitted).await?;
                let request = tests.recv::<TestRequest<2>>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted.into_value())?;
                imp::tests(
                    &arguments,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                )?;
                let result = effects.apply(request).await?;
                imp::actual_result(&result, "patient_a")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment["data"]["artifact_id"],
                )?;
                let mut receipts = actual.take();
                let update = imp::record_panel(
                    &mut receipts,
                    &result,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                );
                actual.set(receipts.clone());
                update?;
                let bytes = imp::encode(&result)?;
                tests.send::<TestResult>(&bytes.as_slice()).await?;
                panel = tests.recv::<PanelReturn<PANEL_A>>().await?;
                let bytes = imp::encode(&receipts)?;
                if imp::panel_complete(
                    &receipts,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                )? {
                    tests.send::<PanelCompleted<PANEL_A>>(panel).await?;
                    tests.send::<PanelComplete>(&bytes.as_slice()).await?;
                    break Ok(receipts);
                }
                let attempt = imp::encode(&result)?;
                tests.send::<FilePanelResultA>(&attempt.as_slice()).await?;
                tests.send::<PanelPartial>(&bytes.as_slice()).await?;
            }
        };
        async {
            let query = ct.recv::<CtAssessmentQueryA>().await?.into_value();
            imp::imaging(&query, ("patient_a", "CT"))?;
            let bytes = imp::encode(&assessment)?;
            ct.send::<CtAssessmentA>(&bytes.as_slice()).await?;
            let report = imp::decode(ct.recv::<CtFindingsA>().await?)?;
            imp::actual_result(&report, "patient_a")?;
            imp::same(
                &report["data"]["inputs"]["assessment"],
                &assessment["data"]["artifact_id"],
            )?;
            Ok(report)
        };
    }
    .await?;
    let findings = json!({"assessment":assessment, "tests":receipts, "ct":ct});
    let bytes = imp::encode(&findings)?;
    endpoint.send::<FindingsA>(&bytes.as_slice()).await?;
    let request = endpoint.recv::<DiagnosisRequestA>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::patient(&arguments, "patient_a")?;
    imp::optional_text(&arguments, "symptoms")?;
    let result = effects.apply(request).await?;
    imp::actual_result(&result, "patient_a")?;
    imp::same(
        &result["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    for test in ["trauma_panel", "blood_type", "coag"] {
        imp::same(
            &result["data"]["inputs"][format!("test:{test}")],
            &receipts[test]["data"]["artifact_id"],
        )?;
    }
    imp::same(
        &result["data"]["inputs"]["imaging:CT"],
        &ct["data"]["artifact_id"],
    )?;
    let result_bytes = imp::encode(&result)?;
    let record_bytes = imp::encode(&json!({"result":result,"findings":findings}))?;
    async {
        let [
            mut clinical,
            mut file,
            mut anesthesia,
            mut surgery,
            mut anesthesia_file,
            mut surgery_file,
        ] = endpoint.par()?;
        async {
            clinical
                .send::<DiagnosisResultA>(&result_bytes.as_slice())
                .await?;
        };
        async {
            file.send::<FileDiagnosisResultA>(&result_bytes.as_slice())
                .await?;
        };
        async {
            anesthesia
                .send::<DiagnosisForAnesthesia>(&record_bytes.as_slice())
                .await?;
        };
        async {
            surgery
                .send::<DiagnosisForSurgery>(&record_bytes.as_slice())
                .await?;
        };
        async {
            anesthesia_file
                .send::<FileDiagnosisAnesthesia>(&record_bytes.as_slice())
                .await?;
        };
        async {
            surgery_file
                .send::<FileDiagnosisSurgery>(&record_bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn patient_b<'r, D: ReceivedEffects>(
    endpoint: &mut Endpoint<'r, PATIENT_B_SERVICE>,
    (mut panel, actual): (Endpoint<'r, PANEL_B>, &Cell<Value>),
    effects: &D,
) -> ShellResult<()> {
    let request = endpoint.recv::<AssessmentRequestB>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::patient(&arguments, "patient_b")?;
    let assessment = effects.apply(request).await?;
    imp::actual_result(&assessment, "patient_b")?;
    let bytes = imp::encode(&assessment)?;
    async {
        let [mut clinical, mut file] = endpoint.par()?;
        async {
            clinical
                .send::<AssessmentResultB>(&bytes.as_slice())
                .await?;
        };
        async {
            file.send::<FileAssessmentB>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let (receipts, ct, mri) = async {
        let [mut tests, mut ct, mut mri] = endpoint.par()?;
        async {
            loop {
                tests.send::<PanelLoan<PANEL_B>>(panel).await?;
                let admitted = tests.recv::<PanelFileRequestB>().await?;
                tests.send::<PanelAdmittedB>(&admitted).await?;
                let request = tests.recv::<TestRequest<3>>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&arguments, &admitted.into_value())?;
                imp::tests(&arguments, ("patient_b", &["stroke_panel", "ct_angio"]))?;
                let result = effects.apply(request).await?;
                imp::actual_result(&result, "patient_b")?;
                imp::same(
                    &result["data"]["inputs"]["assessment"],
                    &assessment["data"]["artifact_id"],
                )?;
                let mut receipts = actual.take();
                let update = imp::record_panel(
                    &mut receipts,
                    &result,
                    ("patient_b", &["stroke_panel", "ct_angio"]),
                );
                actual.set(receipts.clone());
                update?;
                let bytes = imp::encode(&result)?;
                tests.send::<TestResult>(&bytes.as_slice()).await?;
                panel = tests.recv::<PanelReturn<PANEL_B>>().await?;
                let bytes = imp::encode(&receipts)?;
                if imp::panel_complete(&receipts, ("patient_b", &["stroke_panel", "ct_angio"]))? {
                    tests.send::<PanelCompleted<PANEL_B>>(panel).await?;
                    tests.send::<PanelComplete>(&bytes.as_slice()).await?;
                    break Ok(receipts);
                }
                let attempt = imp::encode(&result)?;
                tests.send::<FilePanelResultB>(&attempt.as_slice()).await?;
                tests.send::<PanelPartial>(&bytes.as_slice()).await?;
            }
        };
        async {
            let query = ct.recv::<CtAssessmentQueryB>().await?.into_value();
            imp::imaging(&query, ("patient_b", "CT"))?;
            let bytes = imp::encode(&assessment)?;
            ct.send::<CtAssessmentB>(&bytes.as_slice()).await?;
            let report = imp::decode(ct.recv::<CtFindingsB>().await?)?;
            imp::actual_result(&report, "patient_b")?;
            imp::same(
                &report["data"]["inputs"]["assessment"],
                &assessment["data"]["artifact_id"],
            )?;
            Ok(report)
        };
        async {
            let query = mri.recv::<MriAssessmentQuery>().await?.into_value();
            imp::imaging(&query, ("patient_b", "MRI"))?;
            let bytes = imp::encode(&assessment)?;
            mri.send::<MriAssessment>(&bytes.as_slice()).await?;
            let report = imp::decode(mri.recv::<MriFindingsB>().await?)?;
            imp::actual_result(&report, "patient_b")?;
            imp::same(
                &report["data"]["inputs"]["assessment"],
                &assessment["data"]["artifact_id"],
            )?;
            Ok(report)
        };
    }
    .await?;
    let findings = json!({"assessment":assessment, "tests":receipts, "ct":ct, "mri":mri});
    let bytes = imp::encode(&findings)?;
    endpoint.send::<FindingsB>(&bytes.as_slice()).await?;
    let request = endpoint.recv::<DiagnosisRequestB>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::patient(&arguments, "patient_b")?;
    imp::optional_text(&arguments, "symptoms")?;
    let result = effects.apply(request).await?;
    imp::actual_result(&result, "patient_b")?;
    imp::same(
        &result["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    for test in ["stroke_panel", "ct_angio"] {
        imp::same(
            &result["data"]["inputs"][format!("test:{test}")],
            &receipts[test]["data"]["artifact_id"],
        )?;
    }
    imp::same(
        &result["data"]["inputs"]["imaging:CT"],
        &ct["data"]["artifact_id"],
    )?;
    imp::same(
        &result["data"]["inputs"]["imaging:MRI"],
        &mri["data"]["artifact_id"],
    )?;
    let result_bytes = imp::encode(&result)?;
    let record_bytes = imp::encode(&json!({"result":result,"findings":findings}))?;
    async {
        let [mut clinical, mut file, mut icu, mut icu_file] = endpoint.par()?;
        async {
            clinical
                .send::<DiagnosisResultB>(&result_bytes.as_slice())
                .await?;
        };
        async {
            file.send::<FileDiagnosisResultB>(&result_bytes.as_slice())
                .await?;
        };
        async {
            icu.send::<DiagnosisForIcu>(&record_bytes.as_slice())
                .await?;
        };
        async {
            icu_file
                .send::<FileDiagnosisIcu>(&record_bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
