//! Direct request receives, affine resource custody and actual clinical effects.
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
    let arguments = endpoint.recv::<AssessmentRequest>().await?.into_value();
    let assessment = domain
        .execute("ER_DOCTOR", "assess_patient", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<AssessmentResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1
                .send::<FileAssessmentResult>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let receipts = loop {
        endpoint.send::<PanelLoan<GENERAL_PANEL>>(panel).await?;
        let admitted = endpoint.recv::<GeneralFileRequest>().await?;
        endpoint.send::<GeneralAdmitted>(&admitted).await?;
        let arguments = endpoint.recv::<TestRequest<1>>().await?.into_value();
        json::same(&arguments, &admitted.into_value())?;
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
        if patient::panel_complete(&receipts, &["vitals", "basic_labs"])? {
            let data = json::encode(&json!({"tests":receipts,"attempt":result}))?;
            endpoint
                .send::<GeneralCompletedData>(&data.as_slice())
                .await?;
            let bytes = json::encode(&receipts)?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint
            .send::<FileGeneralResult>(&bytes.as_slice())
            .await?;
        let bytes = json::encode(&receipts)?;
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    endpoint
        .send::<PanelCompleted<GENERAL_PANEL>>(panel)
        .await?;
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
        Ok::<(), ShellError>(())
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
        let admitted = endpoint.recv::<CardiacFileRequest>().await?;
        endpoint.send::<CardiacAdmitted>(&admitted).await?;
        let arguments = endpoint.recv::<TestRequest<2>>().await?.into_value();
        json::same(&arguments, &admitted.into_value())?;
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
        if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])? {
            let data = json::encode(&json!({"tests":receipts,"attempt":result}))?;
            endpoint
                .send::<CardiacCompletedData>(&data.as_slice())
                .await?;
            let bytes = json::encode(&receipts)?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint
            .send::<FileCardiacResult>(&bytes.as_slice())
            .await?;
        let bytes = json::encode(&receipts)?;
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    endpoint
        .send::<PanelCompleted<CARDIAC_PANEL>>(panel)
        .await?;
    let bytes = json::encode(&receipts)?;
    let (general, ct, mri, ()) = async {
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
        return Err(ShellError::message("general panel is incomplete"));
    }
    for (modality, result) in [("CT", &ct), ("MRI", &mri)] {
        patient::actual_result(result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        if result["data"]["modality"] != modality {
            return Err(ShellError::message("report has another modality"));
        }
    }
    let arguments = endpoint.recv::<CardDiagnosisRequest>().await?.into_value();
    let result = domain
        .execute("CARDIOLOGIST", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    for test in &["ecg", "cardiac_enzymes"] {
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
    let file_result = json::encode(&result)?;
    endpoint
        .send::<CardDiagnosisResult>(&file_result.as_slice())
        .await?;
    let bytes = json::encode(&json!({"result":result,"tests":receipts}))?;
    async {
        let [mut branch0, mut branch1, mut branch2] = endpoint.par()?;
        async {
            branch0
                .send::<FileCardDiagnosisResult>(&file_result.as_slice())
                .await?;
        };
        async {
            branch1
                .send::<CardDiagnosisForER>(&bytes.as_slice())
                .await?;
        };
        async {
            branch2
                .send::<CardDiagnosisForConference>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
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
        let admitted = endpoint.recv::<NeuroFileRequest>().await?;
        endpoint.send::<NeuroAdmitted>(&admitted).await?;
        let arguments = endpoint.recv::<TestRequest<3>>().await?.into_value();
        json::same(&arguments, &admitted.into_value())?;
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
        if patient::panel_complete(&receipts, &["neuro_exam", "nerve_conduction"])? {
            let data = json::encode(&json!({"tests":receipts,"attempt":result}))?;
            endpoint
                .send::<NeuroCompletedData>(&data.as_slice())
                .await?;
            let bytes = json::encode(&receipts)?;
            endpoint.send::<PanelComplete>(&bytes.as_slice()).await?;
            break receipts;
        }
        endpoint.send::<FileNeuroResult>(&bytes.as_slice()).await?;
        let bytes = json::encode(&receipts)?;
        endpoint.send::<PanelPartial>(&bytes.as_slice()).await?;
    };
    endpoint.send::<PanelCompleted<NEURO_PANEL>>(panel).await?;
    let bytes = json::encode(&receipts)?;
    let (general, ct, mri, ()) = async {
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
        return Err(ShellError::message("general panel is incomplete"));
    }
    for (modality, result) in [("CT", &ct), ("MRI", &mri)] {
        patient::actual_result(result)?;
        json::same(
            &result["data"]["inputs"]["assessment"],
            &assessment["data"]["artifact_id"],
        )?;
        if result["data"]["modality"] != modality {
            return Err(ShellError::message("report has another modality"));
        }
    }
    let arguments = endpoint.recv::<NeuroDiagnosisRequest>().await?.into_value();
    let result = domain
        .execute("NEUROLOGIST", "diagnose", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    for test in &["neuro_exam", "nerve_conduction"] {
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
    let file_result = json::encode(&result)?;
    endpoint
        .send::<NeuroDiagnosisResult>(&file_result.as_slice())
        .await?;
    let bytes = json::encode(&json!({"result":result,"tests":receipts}))?;
    async {
        let [mut branch0, mut branch1, mut branch2] = endpoint.par()?;
        async {
            branch0
                .send::<FileNeuroDiagnosisResult>(&file_result.as_slice())
                .await?;
        };
        async {
            branch1
                .send::<NeuroDiagnosisForER>(&bytes.as_slice())
                .await?;
        };
        async {
            branch2
                .send::<NeuroDiagnosisForConference>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
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
    let admitted = endpoint.recv::<CtPerformIntent>().await?;
    endpoint.send::<CtPerformAdmitted>(&admitted).await?;
    endpoint.send::<CtLoan>(instrument).await?;
    let arguments = endpoint.recv::<CtPerformRequest>().await?.into_value();
    json::same(&arguments, &admitted.into_value())?;
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
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<CtImageResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<FileCtImageResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    instrument = endpoint.recv::<CtLoan>().await?;
    let admitted = endpoint.recv::<CtInterpretIntent>().await?;
    endpoint.send::<CtInterpretAdmitted>(&admitted).await?;
    endpoint.send::<CtLoan>(instrument).await?;
    let arguments = endpoint.recv::<CtInterpretRequest>().await?.into_value();
    json::same(&arguments, &admitted.into_value())?;
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
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<CtReportResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1
                .send::<FileCtReportResult>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
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
        Ok::<(), ShellError>(())
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
    let admitted = endpoint.recv::<MriPerformIntent>().await?;
    endpoint.send::<MriPerformAdmitted>(&admitted).await?;
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = endpoint.recv::<MriPerformRequest>().await?.into_value();
    json::same(&arguments, &admitted.into_value())?;
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
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<MriImageResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1
                .send::<FileMriImageResult>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    instrument = endpoint.recv::<MriLoan>().await?;
    let admitted = endpoint.recv::<MriInterpretIntent>().await?;
    endpoint.send::<MriInterpretAdmitted>(&admitted).await?;
    endpoint.send::<MriLoan>(instrument).await?;
    let arguments = endpoint.recv::<MriInterpretRequest>().await?.into_value();
    json::same(&arguments, &admitted.into_value())?;
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
    async {
        let [mut branch0, mut branch1] = endpoint.par()?;
        async {
            branch0.send::<MriReportResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1
                .send::<FileMriReportResult>(&bytes.as_slice())
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
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
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn pharmacy<D: Domain>(
    endpoint: &mut Endpoint<PHARMACY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<ReviewRequest>().await?.into_value();
    let result = domain
        .execute("PHARMACIST", "review_prescriptions", arguments)
        .await
        .map_err(ShellError::message)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    async {
        let [mut branch0, mut branch1, mut branch2, mut branch3] = endpoint.par()?;
        async {
            branch0.send::<ReviewResult>(&bytes.as_slice()).await?;
        };
        async {
            branch1.send::<ReviewForER>(&bytes.as_slice()).await?;
        };
        async {
            branch2
                .send::<ReviewForConference>(&bytes.as_slice())
                .await?;
        };
        async {
            branch3.send::<FileReviewResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
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
    let (
        (
            (_general_panel, general_record),
            (_cardiac_panel, cardiac_record),
            (_neuro_panel, neuro_record),
            general,
            ct,
            mri,
            cardiac,
            neuro,
        ),
        review,
    ) = async {
        let [mut clinical, mut pharmacy] = endpoint.par()?;
        async {
            let (general_record, cardiac_record, neuro_record) = async {
                let [mut general, mut cardiac, mut neuro] = clinical.par()?;
                async {
                    let data = json::decode(general.recv::<GeneralCompletedData>().await?)?;
                    if !patient::panel_complete(&data["tests"], &["vitals", "basic_labs"])? {
                        return Err(ShellError::message("completed record lacks actual tests"));
                    }
                    patient::actual_result(&data["attempt"])?;
                    if !data["tests"]
                        .as_object()
                        .is_some_and(|values| values.values().any(|r| r == &data["attempt"]))
                    {
                        return Err(ShellError::message(
                            "last attempt is absent from completed records",
                        ));
                    }
                    let panel = general.recv::<PanelCompleted<GENERAL_PANEL>>().await?;
                    let bytes = json::encode(&data["attempt"])?;
                    general.send::<FileGeneralResult>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>((panel, data))
                };
                async {
                    let data = json::decode(cardiac.recv::<CardiacCompletedData>().await?)?;
                    if !patient::panel_complete(&data["tests"], &["ecg", "cardiac_enzymes"])? {
                        return Err(ShellError::message("completed record lacks actual tests"));
                    }
                    patient::actual_result(&data["attempt"])?;
                    if !data["tests"]
                        .as_object()
                        .is_some_and(|values| values.values().any(|r| r == &data["attempt"]))
                    {
                        return Err(ShellError::message(
                            "last attempt is absent from completed records",
                        ));
                    }
                    let panel = cardiac.recv::<PanelCompleted<CARDIAC_PANEL>>().await?;
                    let bytes = json::encode(&data["attempt"])?;
                    cardiac.send::<FileCardiacResult>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>((panel, data))
                };
                async {
                    let data = json::decode(neuro.recv::<NeuroCompletedData>().await?)?;
                    if !patient::panel_complete(
                        &data["tests"],
                        &["neuro_exam", "nerve_conduction"],
                    )? {
                        return Err(ShellError::message("completed record lacks actual tests"));
                    }
                    patient::actual_result(&data["attempt"])?;
                    if !data["tests"]
                        .as_object()
                        .is_some_and(|values| values.values().any(|r| r == &data["attempt"]))
                    {
                        return Err(ShellError::message(
                            "last attempt is absent from completed records",
                        ));
                    }
                    let panel = neuro.recv::<PanelCompleted<NEURO_PANEL>>().await?;
                    let bytes = json::encode(&data["attempt"])?;
                    neuro.send::<FileNeuroResult>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>((panel, data))
                };
            }
            .await?;
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
            Ok::<_, ShellError>((
                general_record,
                cardiac_record,
                neuro_record,
                general,
                ct,
                mri,
                cardiac,
                neuro,
            ))
        };
        async { json::decode(pharmacy.recv::<ReviewForConference>().await?) };
    }
    .await?;
    json::same(&general["tests"], &general_record["tests"])?;
    json::same(&cardiac["tests"], &cardiac_record["tests"])?;
    json::same(&neuro["tests"], &neuro_record["tests"])?;
    for value in [
        &general["assessment"],
        &ct,
        &mri,
        &cardiac["result"],
        &neuro["result"],
        &review,
    ] {
        patient::actual_result(value)?;
    }
    let case = json!({"assessment":general["assessment"],"general":general["tests"],"CT":ct,"MRI":mri,"cardiac":cardiac,"neurology":neuro,"review":review});
    let admitted = endpoint.recv::<ConferenceIntent>().await?;
    endpoint.send::<ConferenceAdmitted>(&admitted).await?;
    endpoint.send::<RoomLoan>(room).await?;
    let request = endpoint.recv::<RoomRequest>().await?.into_value();
    json::same(&request["case"], &case)?;
    json::same(&request["arguments"], &admitted.into_value())?;
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
    let response = json::encode(&result)?;
    endpoint.send::<RoomResult>(&response.as_slice()).await?;
    let _room = endpoint.recv::<RoomLoan>().await?;
    let bytes = json::encode(&json!({"case":case,"conference":result}))?;
    async {
        let [
            mut branch0,
            mut branch1,
            mut branch2,
            mut branch3,
            mut branch4,
        ] = endpoint.par()?;
        async {
            branch0
                .send::<FileConferenceResult>(&response.as_slice())
                .await?;
        };
        async {
            branch1.send::<FinalForER>(&bytes.as_slice()).await?;
        };
        async {
            branch2.send::<FinalForCard>(&bytes.as_slice()).await?;
        };
        async {
            branch3.send::<FinalForNeuro>(&bytes.as_slice()).await?;
        };
        async {
            branch4.send::<FinalForRad>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
