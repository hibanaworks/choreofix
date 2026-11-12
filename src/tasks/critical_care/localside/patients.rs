//! Actual clinical data owners; all communication is visible at its effect boundary.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn patient_a<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, PATIENT_A_SERVICE>,
    (mut panel, actual): (Endpoint<'r, PANEL_A>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let arguments = imp::decode(endpoint.recv::<AssessmentRequestA>().await?)?;
    imp::patient(&arguments, "patient_a")?;
    let assessment = domain
        .execute("ER_DOCTOR_A", "assess_patient", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&assessment, "patient_a")?;
    let bytes = imp::encode(&assessment)?;
    endpoint
        .send::<AssessmentResultA>(&bytes.as_slice())
        .await?;
    let (receipts, ct) = async {
        let [mut tests, mut ct] = endpoint.par()?;
        async {
            loop {
                tests.send::<PanelLoan<PANEL_A>>(panel).await?;
                let arguments = imp::decode(tests.recv::<TestRequest>().await?)?;
                imp::tests(
                    &arguments,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                )?;
                let result = domain
                    .execute("ER_DOCTOR_A", "order_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
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
                tests.send::<PanelPartial>(&bytes.as_slice()).await?;
            }
        };
        async {
            let query = imp::decode(ct.recv::<CtAssessmentQueryA>().await?)?;
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
    let arguments = imp::decode(endpoint.recv::<DiagnosisRequestA>().await?)?;
    imp::patient(&arguments, "patient_a")?;
    imp::optional_text(&arguments, "symptoms")?;
    let result = domain
        .execute("ER_DOCTOR_A", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
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
    let bytes = imp::encode(&result)?;
    endpoint.send::<DiagnosisResultA>(&bytes.as_slice()).await?;
    let bytes = imp::encode(&json!({"result":result, "findings":findings}))?;
    async {
        let [mut anesthesia, mut surgery] = endpoint.par()?;
        async {
            anesthesia
                .send::<DiagnosisForAnesthesia>(&bytes.as_slice())
                .await?;
        };
        async {
            surgery
                .send::<DiagnosisForSurgery>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn patient_b<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, PATIENT_B_SERVICE>,
    (mut panel, actual): (Endpoint<'r, PANEL_B>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let arguments = imp::decode(endpoint.recv::<AssessmentRequestB>().await?)?;
    imp::patient(&arguments, "patient_b")?;
    let assessment = domain
        .execute("ER_DOCTOR_B", "assess_patient", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&assessment, "patient_b")?;
    let bytes = imp::encode(&assessment)?;
    endpoint
        .send::<AssessmentResultB>(&bytes.as_slice())
        .await?;
    let (receipts, ct, mri) = async {
        let [mut tests, mut ct, mut mri] = endpoint.par()?;
        async {
            loop {
                tests.send::<PanelLoan<PANEL_B>>(panel).await?;
                let arguments = imp::decode(tests.recv::<TestRequest>().await?)?;
                imp::tests(&arguments, ("patient_b", &["stroke_panel", "ct_angio"]))?;
                let result = domain
                    .execute("ER_DOCTOR_B", "order_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
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
                tests.send::<PanelPartial>(&bytes.as_slice()).await?;
            }
        };
        async {
            let query = imp::decode(ct.recv::<CtAssessmentQueryB>().await?)?;
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
            let query = imp::decode(mri.recv::<MriAssessmentQuery>().await?)?;
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
    let arguments = imp::decode(endpoint.recv::<DiagnosisRequestB>().await?)?;
    imp::patient(&arguments, "patient_b")?;
    imp::optional_text(&arguments, "symptoms")?;
    let result = domain
        .execute("ER_DOCTOR_B", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
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
    let bytes = imp::encode(&result)?;
    endpoint.send::<DiagnosisResultB>(&bytes.as_slice()).await?;
    let bytes = imp::encode(&json!({"result":result, "findings":findings}))?;
    endpoint.send::<DiagnosisForIcu>(&bytes.as_slice()).await?;
    Ok(())
}
