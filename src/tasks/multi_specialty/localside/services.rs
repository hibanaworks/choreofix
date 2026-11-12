//! Actual clinical effects and resource custody, with no application progress state.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn general<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, GENERAL_SERVICE>,
    (mut panel, actual): (Endpoint<'r, GENERAL_PANEL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let arguments = json::decode(endpoint.recv::<AssessmentRequest>().await?)?;
    patient::patient(&arguments)?;
    let assessment = domain
        .execute("ER_DOCTOR", "assess_patient", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    endpoint.send::<AssessmentResult>(&bytes.as_slice()).await?;
    let receipts = loop {
        endpoint.send::<PanelLoan<GENERAL_PANEL>>(panel).await?;
        let arguments = json::decode(endpoint.recv::<TestRequest>().await?)?;
        patient::tests(&arguments, &["vitals", "basic_labs"])?;
        let result = domain
            .execute("ER_DOCTOR", "order_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        patient::actual_result(&result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        let mut receipts = actual.take();
        let update = patient::record_panel(&mut receipts, &result, &["vitals", "basic_labs"]);
        actual.set(receipts.clone());
        update?;
        let bytes = json::encode(&result)?;
        endpoint.send::<TestResult>(&bytes.as_slice()).await?;
        panel = endpoint.recv::<PanelReturn<GENERAL_PANEL>>().await?;
        let bytes = json::encode(&receipts)?;
        if patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
            endpoint
                .send::<PanelCompleted<GENERAL_PANEL>>(panel)
                .await?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    let bytes = json::encode(&json!({"assessment":assessment,"tests":receipts}))?;
    async {
        let [
            mut branch0,
            mut branch1,
            mut branch2,
            mut branch3,
            mut branch4,
        ] = endpoint.par()?;
        async {
            branch0.send::<GeneralForCard>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<GeneralForNeuro>(&bytes.as_slice()).await?;
        };
        async {
            branch2
                .send::<GeneralForCardOwner>(&bytes.as_slice())
                .await?;
        };
        async {
            branch3
                .send::<GeneralForNeuroOwner>(&bytes.as_slice())
                .await?;
        };
        async {
            branch4
                .send::<GeneralForConference>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn cardiac<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CARDIAC_SERVICE>,
    (mut panel, actual): (Endpoint<'r, CARDIAC_PANEL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let assessment = json::decode(endpoint.recv::<ReferralForOwner>().await?)?;
    patient::actual_result(&assessment)?;
    let receipts = loop {
        endpoint.send::<PanelLoan<CARDIAC_PANEL>>(panel).await?;
        let arguments = json::decode(endpoint.recv::<TestRequest>().await?)?;
        patient::tests(&arguments, &["ecg", "cardiac_enzymes"])?;
        let result = domain
            .execute("CARDIOLOGIST", "order_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        patient::actual_result(&result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        let mut receipts = actual.take();
        let update = patient::record_panel(&mut receipts, &result, &["ecg", "cardiac_enzymes"]);
        actual.set(receipts.clone());
        update?;
        let bytes = json::encode(&result)?;
        endpoint.send::<TestResult>(&bytes.as_slice()).await?;
        panel = endpoint.recv::<PanelReturn<CARDIAC_PANEL>>().await?;
        let bytes = json::encode(&receipts)?;
        if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
            endpoint
                .send::<PanelCompleted<CARDIAC_PANEL>>(panel)
                .await?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    let bytes = json::encode(&receipts)?;
    let (general, ct, mri, _) = async {
        let [mut g, mut cross, mut c, mut m] = endpoint.par()?;
        async { json::decode(g.recv::<GeneralForCardOwner>().await?) };
        async { json::decode(c.recv::<CtForCardOwner>().await?) };
        async { json::decode(m.recv::<MriForCardOwner>().await?) };
        async {
            cross.send::<CardPanelForNeuro>(&bytes.as_slice()).await?;
        };
    }
    .await?;
    json::same(&general["assessment"], &assessment)?;
    if !patient::panel_complete(&general["tests"], &["vitals", "basic_labs"])? {
        return Err(ShellError::message("shared general panel is incomplete"));
    }
    for (modality, result) in [("CT", &ct), ("MRI", &mri)] {
        patient::actual_result(result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        if result["data"]["modality"] != modality {
            return Err(ShellError::message(
                "shared image report has another modality",
            ));
        }
    }
    let arguments = json::decode(endpoint.recv::<CardDiagnosisRequest>().await?)?;
    patient::patient(&arguments)?;
    if arguments.get("symptoms").is_some_and(|v| !v.is_string()) {
        return Err(ShellError::message("symptoms must be a string"));
    }
    let result = domain
        .execute("CARDIOLOGIST", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    for test in ["ecg", "cardiac_enzymes"] {
        json::same(
            &result["data"]["inputs"][format!("test:{test}")],
            &receipts[test]["data"]["artifact_id"],
        )?;
    }
    json::same(
        &result["data"]["inputs"]["imaging:CT"],
        &ct["data"]["artifact_id"],
    )?;
    json::same(
        &result["data"]["inputs"]["imaging:MRI"],
        &mri["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&result)?;
    endpoint
        .send::<CardDiagnosisResult>(&bytes.as_slice())
        .await?;
    let bytes = json::encode(&json!({"result":result,"tests":receipts}))?;
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0
                .send::<CardDiagnosisForER>(&bytes.as_slice())
                .await?;
        };
        async {
            branch1
                .send::<CardDiagnosisForConference>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn neurology<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, NEURO_SERVICE>,
    (mut panel, actual): (Endpoint<'r, NEURO_PANEL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let assessment = json::decode(endpoint.recv::<ReferralForOwner>().await?)?;
    patient::actual_result(&assessment)?;
    let receipts = loop {
        endpoint.send::<PanelLoan<NEURO_PANEL>>(panel).await?;
        let arguments = json::decode(endpoint.recv::<TestRequest>().await?)?;
        patient::tests(&arguments, &["neuro_exam", "nerve_conduction"])?;
        let result = domain
            .execute("NEUROLOGIST", "order_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        patient::actual_result(&result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        let mut receipts = actual.take();
        let update =
            patient::record_panel(&mut receipts, &result, &["neuro_exam", "nerve_conduction"]);
        actual.set(receipts.clone());
        update?;
        let bytes = json::encode(&result)?;
        endpoint.send::<TestResult>(&bytes.as_slice()).await?;
        panel = endpoint.recv::<PanelReturn<NEURO_PANEL>>().await?;
        let bytes = json::encode(&receipts)?;
        if patient::panel_complete(&receipts, &["neuro_exam", "nerve_conduction"])? {
            endpoint.send::<PanelCompleted<NEURO_PANEL>>(panel).await?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    let bytes = json::encode(&receipts)?;
    let (general, ct, mri, _) = async {
        let [mut g, mut cross, mut c, mut m] = endpoint.par()?;
        async { json::decode(g.recv::<GeneralForNeuroOwner>().await?) };
        async { json::decode(c.recv::<CtForNeuroOwner>().await?) };
        async { json::decode(m.recv::<MriForNeuroOwner>().await?) };
        async {
            cross.send::<NeuroPanelForCard>(&bytes.as_slice()).await?;
        };
    }
    .await?;
    json::same(&general["assessment"], &assessment)?;
    if !patient::panel_complete(&general["tests"], &["vitals", "basic_labs"])? {
        return Err(ShellError::message("shared general panel is incomplete"));
    }
    for (modality, result) in [("CT", &ct), ("MRI", &mri)] {
        patient::actual_result(result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        if result["data"]["modality"] != modality {
            return Err(ShellError::message(
                "shared image report has another modality",
            ));
        }
    }
    let arguments = json::decode(endpoint.recv::<NeuroDiagnosisRequest>().await?)?;
    patient::patient(&arguments)?;
    if arguments.get("symptoms").is_some_and(|v| !v.is_string()) {
        return Err(ShellError::message("symptoms must be a string"));
    }
    let result = domain
        .execute("NEUROLOGIST", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    for test in ["neuro_exam", "nerve_conduction"] {
        json::same(
            &result["data"]["inputs"][format!("test:{test}")],
            &receipts[test]["data"]["artifact_id"],
        )?;
    }
    json::same(
        &result["data"]["inputs"]["imaging:CT"],
        &ct["data"]["artifact_id"],
    )?;
    json::same(
        &result["data"]["inputs"]["imaging:MRI"],
        &mri["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&result)?;
    endpoint
        .send::<NeuroDiagnosisResult>(&bytes.as_slice())
        .await?;
    let bytes = json::encode(&json!({"result":result,"tests":receipts}))?;
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0
                .send::<NeuroDiagnosisForER>(&bytes.as_slice())
                .await?;
        };
        async {
            branch1
                .send::<NeuroDiagnosisForConference>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn ct<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CT_SERVICE>,
    mut instrument: Endpoint<'r, CT_SCANNER>,
    domain: &D,
) -> ShellResult<()> {
    let assessment = json::decode(endpoint.recv::<CtOrderForOwner>().await?)?;
    patient::actual_result(&assessment)?;
    let admitted = json::decode(endpoint.recv::<CtPerformIntent>().await?)?;
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|v| !v.eq_ignore_ascii_case("CT"))
    {
        return Err(ShellError::message("imaging intent has another modality"));
    }
    endpoint.send::<CtLoan>(instrument).await?;
    let arguments = json::decode(endpoint.recv::<CtPerformRequest>().await?)?;
    json::same(&arguments, &admitted)?;
    let image = domain
        .execute("RADIOLOGIST", "perform_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&image)?;
    json::same(
        &image["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&image)?;
    endpoint.send::<CtImageResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<CtLoan>().await?;

    let admitted = json::decode(endpoint.recv::<CtInterpretIntent>().await?)?;
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|v| !v.eq_ignore_ascii_case("CT"))
    {
        return Err(ShellError::message(
            "interpretation intent has another modality",
        ));
    }
    endpoint.send::<CtLoan>(instrument).await?;
    let arguments = json::decode(endpoint.recv::<CtInterpretRequest>().await?)?;
    json::same(&arguments, &admitted)?;
    let report = domain
        .execute("RADIOLOGIST", "interpret_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&report)?;
    json::same(
        &report["data"]["inputs"]["image"],
        &image["data"]["artifact_id"],
    )?;
    json::same(
        &report["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&report)?;
    endpoint.send::<CtReportResult>(&bytes.as_slice()).await?;
    let _instrument = endpoint.recv::<CtLoan>().await?;
    async {
        let [
            mut branch0,
            mut branch1,
            mut branch2,
            mut branch3,
            mut branch4,
            mut branch5,
        ] = endpoint.par()?;
        async {
            branch0.send::<CtForCard>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<CtForNeuro>(&bytes.as_slice()).await?;
        };
        async {
            branch2.send::<CtForCardOwner>(&bytes.as_slice()).await?;
        };
        async {
            branch3.send::<CtForNeuroOwner>(&bytes.as_slice()).await?;
        };
        async {
            branch4.send::<CtForER>(&bytes.as_slice()).await?;
        };
        async {
            branch5.send::<CtForConference>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn mri<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, MRI_SERVICE>,
    mut instrument: Endpoint<'r, MRI>,
    domain: &D,
) -> ShellResult<()> {
    let assessment = json::decode(endpoint.recv::<MriOrderForOwner>().await?)?;
    patient::actual_result(&assessment)?;
    let admitted = json::decode(endpoint.recv::<MriPerformIntent>().await?)?;
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|v| !v.eq_ignore_ascii_case("MRI"))
    {
        return Err(ShellError::message("imaging intent has another modality"));
    }
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = json::decode(endpoint.recv::<MriPerformRequest>().await?)?;
    json::same(&arguments, &admitted)?;
    let image = domain
        .execute("RADIOLOGIST", "perform_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&image)?;
    json::same(
        &image["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&image)?;
    endpoint.send::<MriImageResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<MriLoan>().await?;

    let admitted = json::decode(endpoint.recv::<MriInterpretIntent>().await?)?;
    patient::patient(&admitted)?;
    if admitted["modality"]
        .as_str()
        .is_none_or(|v| !v.eq_ignore_ascii_case("MRI"))
    {
        return Err(ShellError::message(
            "interpretation intent has another modality",
        ));
    }
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = json::decode(endpoint.recv::<MriInterpretRequest>().await?)?;
    json::same(&arguments, &admitted)?;
    let report = domain
        .execute("RADIOLOGIST", "interpret_imaging", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&report)?;
    json::same(
        &report["data"]["inputs"]["image"],
        &image["data"]["artifact_id"],
    )?;
    json::same(
        &report["data"]["inputs"]["assessment"],
        &assessment["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&report)?;
    endpoint.send::<MriReportResult>(&bytes.as_slice()).await?;
    let _instrument = endpoint.recv::<MriLoan>().await?;
    async {
        let [
            mut branch0,
            mut branch1,
            mut branch2,
            mut branch3,
            mut branch4,
            mut branch5,
        ] = endpoint.par()?;
        async {
            branch0.send::<MriForCard>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<MriForNeuro>(&bytes.as_slice()).await?;
        };
        async {
            branch2.send::<MriForCardOwner>(&bytes.as_slice()).await?;
        };
        async {
            branch3.send::<MriForNeuroOwner>(&bytes.as_slice()).await?;
        };
        async {
            branch4.send::<MriForER>(&bytes.as_slice()).await?;
        };
        async {
            branch5.send::<MriForConference>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn pharmacy<D: Domain>(
    endpoint: &mut Endpoint<PHARMACY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let arguments = json::decode(endpoint.recv::<ReviewRequest>().await?)?;
    patient::patient(&arguments)?;
    if arguments.get("medications").is_some_and(|v| !v.is_string()) {
        return Err(ShellError::message("medications must be a string"));
    }
    let result = domain
        .execute("PHARMACIST", "review_prescriptions", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    endpoint.send::<ReviewResult>(&bytes.as_slice()).await?;
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<ReviewForER>(&bytes.as_slice()).await?;
        };
        async {
            branch1
                .send::<ReviewForConference>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn conference<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CONFERENCE_SERVICE>,
    room: Endpoint<'r, CONSULTATION_ROOM>,
    domain: &D,
) -> ShellResult<()> {
    let ((general, ct, mri, cardiac, neuro), review) = async {
        let [mut clinical, mut pharmacy] = endpoint.par()?;
        async {
            let (general, ct, mri) = async {
                let [mut g, mut c, mut m] = clinical.par()?;
                async { json::decode(g.recv::<GeneralForConference>().await?) };
                async { json::decode(c.recv::<CtForConference>().await?) };
                async { json::decode(m.recv::<MriForConference>().await?) };
            }
            .await?;
            let (cardiac, neuro) = async {
                let [mut c, mut n] = clinical.par()?;
                async { json::decode(c.recv::<CardDiagnosisForConference>().await?) };
                async { json::decode(n.recv::<NeuroDiagnosisForConference>().await?) };
            }
            .await?;
            Ok((general, ct, mri, cardiac, neuro))
        };
        async { json::decode(pharmacy.recv::<ReviewForConference>().await?) };
    }
    .await?;
    for result in [
        &general["assessment"],
        &ct,
        &mri,
        &cardiac["result"],
        &neuro["result"],
        &review,
    ] {
        patient::actual_result(result)?;
    }
    let case = json!({"assessment":general["assessment"],"general":general["tests"],
        "CT":ct,"MRI":mri,"cardiac":cardiac,"neurology":neuro,"review":review});
    let admitted = json::decode(endpoint.recv::<ConferenceIntent>().await?)?;
    json::same(&admitted["case"], &case)?;
    patient::patient(&admitted["arguments"])?;
    if admitted["arguments"]
        .get("attendees")
        .is_some_and(|v| !v.is_string())
    {
        return Err(ShellError::message("attendees must be a string"));
    }
    endpoint.send::<RoomLoan>(room).await?;
    let request = json::decode(endpoint.recv::<RoomRequest>().await?)?;
    json::same(&request, &admitted)?;
    let result = domain
        .execute(
            "ER_DOCTOR",
            "conduct_case_conference",
            request["arguments"].clone(),
        )
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    json::same(
        &result["data"]["inputs"]["diagnosis:CARDIOLOGIST"],
        &case["cardiac"]["result"]["data"]["artifact_id"],
    )?;
    json::same(
        &result["data"]["inputs"]["diagnosis:NEUROLOGIST"],
        &case["neurology"]["result"]["data"]["artifact_id"],
    )?;
    let bytes = json::encode(&result)?;
    endpoint.send::<RoomResult>(&bytes.as_slice()).await?;
    let _room = endpoint.recv::<RoomLoan>().await?;
    let bytes = json::encode(&json!({"case":case,"conference":result}))?;
    async {
        let [mut branch0, mut branch1, mut branch2, mut branch3] = endpoint.par()?;
        async {
            branch0.send::<FinalForER>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<FinalForCard>(&bytes.as_slice()).await?;
        };
        async {
            branch2.send::<FinalForNeuro>(&bytes.as_slice()).await?;
        };
        async {
            branch3.send::<FinalForRad>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}
