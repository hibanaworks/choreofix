//! One-use clinical admissions, actual resource requests and effect data.
//! EPF validates source execution and affine custody; this module never advances it.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn completed<'a>(
    calls: &[&'a Call],
    key: (&str, &str, &str, Option<&str>),
    before: usize,
) -> Result<&'a Value> {
    one(calls.iter().copied().filter(|call| {
        call.request["agent"] == key.0
            && call.request["tool"] == key.1
            && call.request["arguments"]["patient_id"] == key.2
            && key.3.is_none_or(|m| {
                call.request["arguments"]["modality"]
                    .as_str()
                    .is_some_and(|v| v.eq_ignore_ascii_case(m))
            })
            && call.result.as_ref().is_some_and(|(at, _)| *at < before)
    }))?
    .result()
    .map(|(_, value)| value)
}
fn panel(calls: &[&Call], key: (&str, &str), before: usize) -> Result<Value> {
    let mut values = serde_json::Map::new();
    for call in calls.iter().copied().filter(|call| {
        call.request["agent"] == key.0
            && call.request["tool"] == "order_tests"
            && call.request["arguments"]["patient_id"] == key.1
            && call.result.as_ref().is_some_and(|(at, _)| *at < before)
    }) {
        let result = call.result()?.1;
        for test in result["data"]["tests"]
            .as_array()
            .ok_or("actual test names")?
        {
            values.insert(
                test.as_str().ok_or("actual test name")?.into(),
                result.clone(),
            );
        }
    }
    Ok(Value::Object(values))
}
fn findings(calls: &[&Call], patient: &str, before: usize) -> Result<Value> {
    let doctor = if patient == "patient_a" {
        "ER_DOCTOR_A"
    } else if patient == "patient_b" {
        "ER_DOCTOR_B"
    } else {
        return Err("unknown actual patient".into());
    };
    let mut value = json!({"assessment":completed(calls,(doctor,"assess_patient",patient,None),before)?,
        "tests":panel(calls,(doctor,patient),before)?,
        "ct":completed(calls,("RADIOLOGIST","interpret_imaging",patient,Some("CT")),before)?});
    if patient == "patient_b" {
        value["mri"] = completed(
            calls,
            ("RADIOLOGIST", "interpret_imaging", patient, Some("MRI")),
            before,
        )?
        .clone();
    }
    Ok(value)
}
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty critical-care evidence")?["message"];
    if compiled["task"] != "5H" || fixture["task"] != "5H" || fixture["type"] != "fixtures" {
        return Err("critical-care task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("clinical participants")?;
    if actors.len() != 7
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("ER_DOCTOR_A"),
                Some("ER_DOCTOR_B"),
                Some("RADIOLOGIST"),
                Some("SURGEON"),
                Some("ANESTHESIOLOGIST"),
                Some("ICU_COORDINATOR"),
                Some("PHARMACIST"),
            ])
    {
        return Err("critical-care participants differ".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u8> {
        compiled["labels"][name]
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .ok_or_else(|| format!("compiled label {name}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() < 16
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe critical-care outcome".into());
    }
    let pairs = [
        ("ER_DOCTOR_A", "ER_A_IO"),
        ("ER_DOCTOR_B", "ER_B_IO"),
        ("RADIOLOGIST", "RADIOLOGY_IO"),
        ("SURGEON", "SURGERY_IO"),
        ("ANESTHESIOLOGIST", "ANESTHESIA_IO"),
        ("ICU_COORDINATOR", "ICU_IO"),
        ("PHARMACIST", "PHARMACY_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let business = |frame: &Frame| {
        !pairs
            .iter()
            .any(|(vm, io)| frame.direction(*vm, *io) || frame.direction(*io, *vm))
    };
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let tests = ordered
        .iter()
        .filter(|call| call.request["tool"] == "order_tests")
        .count();
    for doctor in ["ER_DOCTOR_A", "ER_DOCTOR_B"] {
        if !ordered
            .iter()
            .any(|call| call.request["agent"] == doctor && call.request["tool"] == "order_tests")
        {
            return Err("missing an actual patient panel".into());
        }
    }
    for (name, number) in compiled["labels"]
        .as_object()
        .ok_or("compiled label names")?
    {
        let number = number
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or("compiled logical label")?;
        let expected = match name.as_str() {
            "PanelLoan" | "PanelReturn" | "TestRequest" | "TestResult" => tests,
            "PanelCompleted" | "PanelComplete" | "ResourceLoan" | "ResourceReturn" => 2,
            "PanelPartial" => tests.checked_sub(2).ok_or("two actual patient panels")?,
            "CtLoan" => 8,
            "MriLoan" => 4,
            "AnesthesiaLoan" | "SurgeryLoan" => 2,
            _ => 1,
        };
        if frames
            .values()
            .filter(|f| business(f) && f.logical_label == number)
            .count()
            != expected
        {
            return Err(format!("missing or excess {name}").into());
        }
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("common-world effects")?
        .clone();
    if scored.len() != ordered.len() {
        return Err("world and domain counts differ".into());
    }
    let (mut used, mut closes) = (BTreeSet::new(), BTreeSet::new());
    for call in &ordered {
        let actor = call.request["agent"].as_str().ok_or("effect actor")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let patient = arguments["patient_id"].as_str().ok_or("effect patient")?;
        let (_, result) = call.result()?;
        let (ended, _) = call.ended()?;
        if result["success"] != true
            || result["data"]["patient_id"] != patient
            || !result["data"]["artifact_id"].is_string()
        {
            return Err("effect lacks an actual patient artifact".into());
        }
        let modality = arguments["modality"].as_str().map(str::to_ascii_uppercase);
        let (io, from, owner, request, target, reply, intent, second) =
            match (actor, tool, patient, modality.as_deref()) {
                ("ER_DOCTOR_A", "assess_patient", "patient_a", _) => (
                    "ER_A_IO",
                    "ER_DOCTOR_A",
                    "PATIENT_A_SERVICE",
                    "AssessmentRequestA",
                    "ER_DOCTOR_A",
                    "AssessmentResultA",
                    None,
                    None,
                ),
                ("ER_DOCTOR_B", "assess_patient", "patient_b", _) => (
                    "ER_B_IO",
                    "ER_DOCTOR_B",
                    "PATIENT_B_SERVICE",
                    "AssessmentRequestB",
                    "ER_DOCTOR_B",
                    "AssessmentResultB",
                    None,
                    None,
                ),
                ("ER_DOCTOR_A", "order_tests", "patient_a", _) => (
                    "ER_A_IO",
                    "PANEL_A",
                    "PATIENT_A_SERVICE",
                    "TestRequest",
                    "PANEL_A",
                    "TestResult",
                    None,
                    None,
                ),
                ("ER_DOCTOR_B", "order_tests", "patient_b", _) => (
                    "ER_B_IO",
                    "PANEL_B",
                    "PATIENT_B_SERVICE",
                    "TestRequest",
                    "PANEL_B",
                    "TestResult",
                    None,
                    None,
                ),
                ("ER_DOCTOR_A", "diagnose", "patient_a", _) => (
                    "ER_A_IO",
                    "ER_DOCTOR_A",
                    "PATIENT_A_SERVICE",
                    "DiagnosisRequestA",
                    "ER_DOCTOR_A",
                    "DiagnosisResultA",
                    None,
                    None,
                ),
                ("ER_DOCTOR_B", "diagnose", "patient_b", _) => (
                    "ER_B_IO",
                    "ER_DOCTOR_B",
                    "PATIENT_B_SERVICE",
                    "DiagnosisRequestB",
                    "ER_DOCTOR_B",
                    "DiagnosisResultB",
                    None,
                    None,
                ),
                ("RADIOLOGIST", "perform_imaging", "patient_a", Some("CT")) => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtPerformRequestA",
                    "CT_SCANNER",
                    "CtImageA",
                    Some("CtPerformIntentA"),
                    None,
                ),
                ("RADIOLOGIST", "interpret_imaging", "patient_a", Some("CT")) => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtInterpretRequestA",
                    "CT_SCANNER",
                    "CtReportA",
                    Some("CtInterpretIntentA"),
                    None,
                ),
                ("RADIOLOGIST", "perform_imaging", "patient_b", Some("CT")) => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtPerformRequestB",
                    "CT_SCANNER",
                    "CtImageB",
                    Some("CtPerformIntentB"),
                    None,
                ),
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("CT")) => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtInterpretRequestB",
                    "CT_SCANNER",
                    "CtReportB",
                    Some("CtInterpretIntentB"),
                    None,
                ),
                ("RADIOLOGIST", "perform_imaging", "patient_b", Some("MRI")) => (
                    "RADIOLOGY_IO",
                    "MRI",
                    "MRI_SERVICE",
                    "MriPerformRequest",
                    "MRI",
                    "MriImage",
                    Some("MriPerformIntent"),
                    None,
                ),
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("MRI")) => (
                    "RADIOLOGY_IO",
                    "MRI",
                    "MRI_SERVICE",
                    "MriInterpretRequest",
                    "MRI",
                    "MriReport",
                    Some("MriInterpretIntent"),
                    None,
                ),
                ("ANESTHESIOLOGIST", "prepare_anesthesia", "patient_a", _) => (
                    "ANESTHESIA_IO",
                    "OPERATING_ROOM",
                    "ANESTHESIA_SERVICE",
                    "AnesthesiaRequest",
                    "OPERATING_ROOM",
                    "AnesthesiaResult",
                    Some("AnesthesiaIntent"),
                    None,
                ),
                ("SURGEON", "perform_surgery", "patient_a", _) => (
                    "SURGERY_IO",
                    "OPERATING_ROOM",
                    "SURGERY_SERVICE",
                    "SurgeryRequest",
                    "OPERATING_ROOM",
                    "SurgeryResult",
                    Some("SurgeryIntent"),
                    None,
                ),
                ("ICU_COORDINATOR", "manage_icu", "patient_b", _) => (
                    "ICU_IO",
                    "ICU_BEDS",
                    "ICU_SERVICE",
                    "BedRequest",
                    "ICU_BEDS",
                    "BedResult",
                    Some("IcuIntent"),
                    Some((
                        "VENTILATOR",
                        "VentilatorRequest",
                        "VENTILATOR",
                        "VentilatorResult",
                    )),
                ),
                ("PHARMACIST", "review_prescriptions", "patient_A_and_B", _) => (
                    "PHARMACY_IO",
                    "PHARMACIST",
                    "PHARMACY_SERVICE",
                    "ReviewRequest",
                    "PHARMACIST",
                    "ReviewResult",
                    None,
                    None,
                ),
                _ => return Err("unassigned clinical effect".into()),
            };
        let index = scored
            .iter()
            .position(|event| {
                event["agent"] == actor
                    && event["tool"] == tool
                    && event["args"] == *arguments
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("effect differs from its common-world occurrence")?;
        scored.remove(index);
        let mut before = call.emitted;
        for (from, request, target, reply) in
            std::iter::once((from, request, target, reply)).chain(second)
        {
            for (from, to, name, value, after) in [
                (from, owner, request, arguments, false),
                (owner, target, reply, result, true),
            ] {
                let (from, to, n) = (role(from)?, role(to)?, label(name)?);
                let matches = frames.iter().filter(|(id, f)| {
                    !used.contains(*id)
                        && f.direction(from, to)
                        && f.logical_label == n
                        && if after {
                            f.sent > ended
                        } else {
                            f.received.is_some_and(|at| at < call.emitted)
                        }
                });
                let (id, frame) = if after {
                    matches.min_by_key(|(_, f)| f.sent)
                } else {
                    matches.max_by_key(|(_, f)| f.received)
                }
                .ok_or_else(|| format!("missing actual {name}"))?;
                if frame.value()? != *value {
                    return Err(format!("altered {name}").into());
                }
                frame.received()?;
                used.insert(*id);
                if !after {
                    before = before.min(frame.sent);
                }
            }
        }
        if let Some(name) = intent {
            let (sender, recipient, logical_label) = (role(actor)?, role(owner)?, label(name)?);
            let candidates = frames
                .iter()
                .filter(|(id, f)| {
                    !used.contains(*id)
                        && f.direction(sender, recipient)
                        && f.logical_label == logical_label
                })
                .collect::<Vec<_>>();
            let (id, frame) = *one(candidates.iter())?;
            if frame.value()? != *arguments || frame.received()? >= call.emitted {
                return Err("intent differs from its actual resource request".into());
            }
            used.insert(*id);
            before = before.min(frame.sent);
        }
        close_arguments(
            &frames,
            &mut closes,
            role(actor)?,
            role(io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
        if !["assess_patient", "review_prescriptions"].contains(&tool) {
            let doctor = if patient == "patient_a" {
                "ER_DOCTOR_A"
            } else {
                "ER_DOCTOR_B"
            };
            if result["data"]["inputs"]["assessment"]
                != completed(
                    &ordered,
                    (doctor, "assess_patient", patient, None),
                    call.emitted,
                )?["data"]["artifact_id"]
            {
                return Err("effect uses another patient assessment".into());
            }
        }
        if tool == "interpret_imaging"
            && result["data"]["inputs"]["image"]
                != completed(
                    &ordered,
                    (
                        "RADIOLOGIST",
                        "perform_imaging",
                        patient,
                        modality.as_deref(),
                    ),
                    call.emitted,
                )?["data"]["artifact_id"]
        {
            return Err("interpretation uses another actual image".into());
        }
        if tool == "diagnose" {
            let actual = findings(&ordered, patient, call.emitted)?;
            let expected: &[&str] = if patient == "patient_a" {
                &["trauma_panel", "blood_type", "coag"]
            } else {
                &["stroke_panel", "ct_angio"]
            };
            let panel = actual["tests"].as_object().ok_or("actual complete panel")?;
            if !expected.iter().all(|test| panel.contains_key(*test)) {
                return Err("diagnosis has an incomplete actual panel".into());
            }
            for test in expected {
                if result["data"]["inputs"][format!("test:{test}")]
                    != panel[*test]["data"]["artifact_id"]
                {
                    return Err("diagnosis uses another actual test".into());
                }
            }
            for (modality, field) in [("CT", "ct"), ("MRI", "mri")] {
                if field == "mri" && patient == "patient_a" {
                    continue;
                }
                if result["data"]["inputs"][format!("imaging:{modality}")]
                    != actual[field]["data"]["artifact_id"]
                {
                    return Err("diagnosis uses another actual report".into());
                }
            }
        }
        if ["prepare_anesthesia", "perform_surgery", "manage_icu"].contains(&tool) {
            let doctor = if patient == "patient_a" {
                "ER_DOCTOR_A"
            } else {
                "ER_DOCTOR_B"
            };
            if result["data"]["inputs"][format!("diagnosis:{doctor}")]
                != completed(&ordered, (doctor, "diagnose", patient, None), call.emitted)?["data"]["artifact_id"]
            {
                return Err("care uses another actual diagnosis".into());
            }
        }
        if tool == "perform_surgery"
            && result["data"]["inputs"]["anesthesia"]
                != completed(
                    &ordered,
                    ("ANESTHESIOLOGIST", "prepare_anesthesia", "patient_a", None),
                    call.emitted,
                )?["data"]["artifact_id"]
        {
            return Err("surgery uses another preparation".into());
        }
    }
    if !scored.is_empty() {
        return Err("unused common-world effect".into());
    }
    for (id, frame) in &frames {
        if used.contains(id) || !business(frame) {
            continue;
        }
        let name = compiled["labels"]
            .as_object()
            .ok_or("message names")?
            .iter()
            .find(|(_, n)| n.as_u64() == Some(u64::from(frame.logical_label)))
            .map(|(name, _)| name.as_str())
            .ok_or("unknown business label")?;
        if matches!(
            name,
            "PanelLoan"
                | "PanelReturn"
                | "PanelCompleted"
                | "CtLoan"
                | "MriLoan"
                | "AnesthesiaLoan"
                | "PreparedRoom"
                | "SurgeryLoan"
                | "ResourceLoan"
                | "ResourceReturn"
        ) {
            frame.received()?;
            used.insert(*id);
            continue;
        }
        let expected = match name {
            "CtAssessmentQueryA" | "CtAssessmentQueryB" | "MriAssessmentQuery" => {
                let patient = if name == "CtAssessmentQueryA" {
                    "patient_a"
                } else {
                    "patient_b"
                };
                let modality = if name == "MriAssessmentQuery" {
                    "MRI"
                } else {
                    "CT"
                };
                one(ordered.iter().copied().filter(|call| {
                    call.request["agent"] == "RADIOLOGIST"
                        && call.request["tool"] == "perform_imaging"
                        && call.request["arguments"]["patient_id"] == patient
                        && call.request["arguments"]["modality"]
                            .as_str()
                            .is_some_and(|m| m.eq_ignore_ascii_case(modality))
                }))?
                .request["arguments"]
                    .clone()
            }
            "CtAssessmentA" => completed(
                &ordered,
                ("ER_DOCTOR_A", "assess_patient", "patient_a", None),
                frame.sent,
            )?
            .clone(),
            "CtAssessmentB" | "MriAssessment" => completed(
                &ordered,
                ("ER_DOCTOR_B", "assess_patient", "patient_b", None),
                frame.sent,
            )?
            .clone(),
            "CtFindingsA" => completed(
                &ordered,
                ("RADIOLOGIST", "interpret_imaging", "patient_a", Some("CT")),
                frame.sent,
            )?
            .clone(),
            "CtFindingsB" => completed(
                &ordered,
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("CT")),
                frame.sent,
            )?
            .clone(),
            "MriFindingsB" => completed(
                &ordered,
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("MRI")),
                frame.sent,
            )?
            .clone(),
            "FindingsA" => findings(&ordered, "patient_a", frame.sent)?,
            "FindingsB" => findings(&ordered, "patient_b", frame.sent)?,
            "DiagnosisForAnesthesia" | "DiagnosisForSurgery" => json!({
                "result":completed(&ordered,("ER_DOCTOR_A","diagnose","patient_a",None),frame.sent)?,
                "findings":findings(&ordered,"patient_a",frame.sent)?}),
            "DiagnosisForIcu" => json!({
                "result":completed(&ordered,("ER_DOCTOR_B","diagnose","patient_b",None),frame.sent)?,
                "findings":findings(&ordered,"patient_b",frame.sent)?}),
            "PreparationForSurgery" => completed(
                &ordered,
                ("ANESTHESIOLOGIST", "prepare_anesthesia", "patient_a", None),
                frame.sent,
            )?
            .clone(),
            "SurgeryForER" => completed(
                &ordered,
                ("SURGEON", "perform_surgery", "patient_a", None),
                frame.sent,
            )?
            .clone(),
            "IcuForER" => completed(
                &ordered,
                ("ICU_COORDINATOR", "manage_icu", "patient_b", None),
                frame.sent,
            )?
            .clone(),
            "ReviewForA" | "ReviewForB" => completed(
                &ordered,
                (
                    "PHARMACIST",
                    "review_prescriptions",
                    "patient_A_and_B",
                    None,
                ),
                frame.sent,
            )?
            .clone(),
            "PanelPartial" | "PanelComplete" => {
                let source = frame.identity["source"].as_u64().ok_or("panel source")?;
                let (actor, patient) = if source == role("PATIENT_A_SERVICE")? {
                    ("ER_DOCTOR_A", "patient_a")
                } else if source == role("PATIENT_B_SERVICE")? {
                    ("ER_DOCTOR_B", "patient_b")
                } else {
                    return Err("unknown panel source".into());
                };
                panel(&ordered, (actor, patient), frame.sent)?
            }
            _ => return Err(format!("unused business {name}").into()),
        };
        if frame.value()? != expected {
            return Err(format!("{name} differs from actual patient data").into());
        }
        frame.received()?;
        used.insert(*id);
    }
    let mut overlaps = 0;
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                overlaps += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"5H","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
        "common_causal_score":"passed","independent_effect_overlaps":overlaps,
        "scope":"one-use WASI admissions, both ICU resource requests, actual patient inputs and results; EPF owns dynamic order",
        "absent_domain_effects_certified":false}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    check(&rows, &artifact)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let actors = rows.first().ok_or("fixture")?["message"]["fixtures"]
        .as_array()
        .ok_or("participants")?;
    let journal = crate::journal::read(&rows, &labels, actors)?;
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let selected = journal
        .frames
        .iter()
        .filter_map(|(id, frame)| {
            let wire = u16::try_from(frame.identity["frame_label"].as_u64()?).ok()?;
            let header = contracts.get(&wire)?;
            (header[4] == 0
                && artifact["compiled"]["labels"]
                    .as_object()?
                    .values()
                    .any(|n| n.as_u64() == Some(u64::from(frame.logical_label))))
            .then_some(*id)
        })
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,
        "scope":"saved-record effect audit, not runtime attacks or EPF mutation replay","model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
