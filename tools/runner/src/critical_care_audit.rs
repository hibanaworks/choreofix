//! Actual typed file/resource receives and clinical effects. EPF owns dynamic order.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

// Immutable classification of the recorded compiled contracts, never task authority.
fn clinical_frames(compiled: &Value) -> Result<impl Fn(&Frame) -> bool> {
    let mut infrastructure = [false; 256];
    let mut labels = [false; 256];
    for (name, role) in compiled["roles"].as_object().ok_or("compiled roles")? {
        let role = role.as_u64().filter(|r| *r < 256).ok_or("compiled role")? as usize;
        infrastructure[role] =
            name.ends_with("_DATA") || name.ends_with("_IO") || name.ends_with("_VM");
    }
    for label in compiled["labels"]
        .as_object()
        .ok_or("compiled labels")?
        .values()
    {
        labels[label
            .as_u64()
            .filter(|n| *n < 256)
            .ok_or("compiled label")? as usize] = true;
    }
    Ok(move |frame: &Frame| {
        let (Some(from), Some(to)) = (
            frame.identity["source"].as_u64(),
            frame.identity["target"].as_u64(),
        ) else {
            return false;
        };
        from < 256
            && to < 256
            && !infrastructure[from as usize]
            && !infrastructure[to as usize]
            && labels[frame.logical_label as usize]
    })
}
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
pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty critical-care evidence")?["message"];
    if compiled["task"] != "5H" || fixture["task"] != "5H" || fixture["type"] != "fixtures" {
        return Err("critical-care task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("clinical participants")?;
    let expected = BTreeSet::from([
        "ER_DOCTOR_A",
        "ER_DOCTOR_B",
        "RADIOLOGIST",
        "SURGEON",
        "ANESTHESIOLOGIST",
        "ICU_COORDINATOR",
        "PHARMACIST",
    ]);
    if actors.len() != 7
        || actors
            .iter()
            .map(|v| v["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != expected.iter().copied().map(Some).collect()
    {
        return Err("critical-care participants differ".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|r| *r < 256)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u8> {
        compiled["labels"][name]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| format!("compiled label {name}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let journal = crate::journal::read(rows, &labels, actors, dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if calls.len() < 16
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe critical-care outcome".into());
    }
    let business = clinical_frames(compiled)?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let tests = ordered
        .iter()
        .filter(|c| c.request["tool"] == "order_tests")
        .count();
    let tests_a = ordered
        .iter()
        .filter(|c| c.request["agent"] == "ER_DOCTOR_A" && c.request["tool"] == "order_tests")
        .count();
    let tests_b = tests - tests_a;
    if tests_a == 0 || tests_b == 0 {
        return Err("missing actual patient panel".into());
    }
    for (name, number) in compiled["labels"].as_object().ok_or("compiled labels")? {
        let number = number
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .ok_or("logical label")?;
        let count = match name.as_str() {
            "PanelLoan" | "PanelReturn" | "TestRequest" | "TestResult" => tests,
            "PanelCompleted" | "PanelComplete" | "ResourceLoan" | "ResourceReturn" => 2,
            "PanelPartial" => tests - 2,
            "CtLoan" => 8,
            "MriLoan" => 4,
            "AnesthesiaLoan" | "SurgeryLoan" => 2,
            "PanelFileRequestA" | "PanelAdmittedA" | "FilePanelResultA" => tests_a,
            "PanelFileRequestB" | "PanelAdmittedB" | "FilePanelResultB" => tests_b,
            _ => 1,
        };
        let actual = frames
            .values()
            .filter(|f| business(f) && f.logical_label == number)
            .count();
        if actual != count {
            return Err(format!("missing or excess {name}: {actual} != {count}").into());
        }
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("common effects")?
        .iter()
        .collect::<Vec<_>>();
    if scored.len() != ordered.len() {
        return Err("world and domain counts differ".into());
    }
    let mut used = BTreeSet::new();
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
            return Err("effect lacks actual patient artifact".into());
        }
        let modality = arguments["modality"].as_str().map(str::to_ascii_uppercase);
        let relations: &[(&str, &str, &str, bool)] =
            match (actor, tool, patient, modality.as_deref()) {
                ("ER_DOCTOR_A", "assess_patient", "patient_a", _) => &[
                    (
                        "ER_A_FILE",
                        "PATIENT_A_SERVICE",
                        "AssessmentRequestA",
                        false,
                    ),
                    (
                        "PATIENT_A_SERVICE",
                        "ER_DOCTOR_A",
                        "AssessmentResultA",
                        true,
                    ),
                    ("PATIENT_A_SERVICE", "ER_A_FILE", "FileAssessmentA", true),
                ],
                ("ER_DOCTOR_A", "order_tests", "patient_a", _) => &[
                    ("ER_A_FILE", "PATIENT_A_SERVICE", "PanelFileRequestA", false),
                    ("PATIENT_A_SERVICE", "ER_DOCTOR_A", "PanelAdmittedA", false),
                    ("PANEL_A", "PATIENT_A_SERVICE", "TestRequest", false),
                    ("PATIENT_A_SERVICE", "PANEL_A", "TestResult", true),
                ],
                ("ER_DOCTOR_A", "diagnose", "patient_a", _) => &[
                    ("ER_A_FILE", "PATIENT_A_SERVICE", "DiagnosisRequestA", false),
                    ("PATIENT_A_SERVICE", "ER_DOCTOR_A", "DiagnosisResultA", true),
                    (
                        "PATIENT_A_SERVICE",
                        "ER_A_FILE",
                        "FileDiagnosisResultA",
                        true,
                    ),
                ],
                ("ER_DOCTOR_B", "assess_patient", "patient_b", _) => &[
                    (
                        "ER_B_FILE",
                        "PATIENT_B_SERVICE",
                        "AssessmentRequestB",
                        false,
                    ),
                    (
                        "PATIENT_B_SERVICE",
                        "ER_DOCTOR_B",
                        "AssessmentResultB",
                        true,
                    ),
                    ("PATIENT_B_SERVICE", "ER_B_FILE", "FileAssessmentB", true),
                ],
                ("ER_DOCTOR_B", "order_tests", "patient_b", _) => &[
                    ("ER_B_FILE", "PATIENT_B_SERVICE", "PanelFileRequestB", false),
                    ("PATIENT_B_SERVICE", "ER_DOCTOR_B", "PanelAdmittedB", false),
                    ("PANEL_B", "PATIENT_B_SERVICE", "TestRequest", false),
                    ("PATIENT_B_SERVICE", "PANEL_B", "TestResult", true),
                ],
                ("ER_DOCTOR_B", "diagnose", "patient_b", _) => &[
                    ("ER_B_FILE", "PATIENT_B_SERVICE", "DiagnosisRequestB", false),
                    ("PATIENT_B_SERVICE", "ER_DOCTOR_B", "DiagnosisResultB", true),
                    (
                        "PATIENT_B_SERVICE",
                        "ER_B_FILE",
                        "FileDiagnosisResultB",
                        true,
                    ),
                ],
                ("RADIOLOGIST", "perform_imaging", "patient_a", Some("CT")) => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtPerformIntentA", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtPerformAdmittedA", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtPerformRequestA", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtImageA", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtImageA", true),
                ],
                ("RADIOLOGIST", "interpret_imaging", "patient_a", Some("CT")) => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtInterpretIntentA", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtInterpretAdmittedA", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtInterpretRequestA", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtReportA", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtReportA", true),
                ],
                ("RADIOLOGIST", "perform_imaging", "patient_b", Some("CT")) => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtPerformIntentB", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtPerformAdmittedB", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtPerformRequestB", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtImageB", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtImageB", true),
                ],
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("CT")) => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtInterpretIntentB", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtInterpretAdmittedB", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtInterpretRequestB", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtReportB", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtReportB", true),
                ],
                ("RADIOLOGIST", "perform_imaging", "patient_b", Some("MRI")) => &[
                    ("RADIOLOGY_FILE", "MRI_SERVICE", "MriPerformIntent", false),
                    ("MRI_SERVICE", "RADIOLOGIST", "MriPerformAdmitted", false),
                    ("MRI", "MRI_SERVICE", "MriPerformRequest", false),
                    ("MRI_SERVICE", "MRI", "MriImage", true),
                    ("MRI_SERVICE", "RADIOLOGY_FILE", "FileMriImage", true),
                ],
                ("RADIOLOGIST", "interpret_imaging", "patient_b", Some("MRI")) => &[
                    ("RADIOLOGY_FILE", "MRI_SERVICE", "MriInterpretIntent", false),
                    ("MRI_SERVICE", "RADIOLOGIST", "MriInterpretAdmitted", false),
                    ("MRI", "MRI_SERVICE", "MriInterpretRequest", false),
                    ("MRI_SERVICE", "MRI", "MriReport", true),
                    ("MRI_SERVICE", "RADIOLOGY_FILE", "FileMriReport", true),
                ],
                ("ANESTHESIOLOGIST", "prepare_anesthesia", "patient_a", _) => &[
                    (
                        "ANESTHESIA_FILE",
                        "ANESTHESIA_SERVICE",
                        "AnesthesiaIntent",
                        false,
                    ),
                    (
                        "ANESTHESIA_SERVICE",
                        "ANESTHESIOLOGIST",
                        "AnesthesiaAdmitted",
                        false,
                    ),
                    (
                        "OPERATING_ROOM",
                        "ANESTHESIA_SERVICE",
                        "AnesthesiaRequest",
                        false,
                    ),
                    (
                        "ANESTHESIA_SERVICE",
                        "OPERATING_ROOM",
                        "AnesthesiaResult",
                        true,
                    ),
                    (
                        "ANESTHESIA_SERVICE",
                        "ANESTHESIA_FILE",
                        "FileAnesthesiaResult",
                        true,
                    ),
                ],
                ("SURGEON", "perform_surgery", "patient_a", _) => &[
                    ("SURGERY_FILE", "SURGERY_SERVICE", "SurgeryIntent", false),
                    ("SURGERY_SERVICE", "SURGEON", "SurgeryAdmitted", false),
                    ("OPERATING_ROOM", "SURGERY_SERVICE", "SurgeryRequest", false),
                    ("SURGERY_SERVICE", "OPERATING_ROOM", "SurgeryResult", true),
                    ("SURGERY_SERVICE", "SURGERY_FILE", "FileSurgeryResult", true),
                ],
                ("ICU_COORDINATOR", "manage_icu", "patient_b", _) => &[
                    ("ICU_FILE", "ICU_SERVICE", "IcuIntent", false),
                    ("ICU_SERVICE", "ICU_COORDINATOR", "IcuAdmitted", false),
                    ("ICU_BEDS", "ICU_SERVICE", "BedRequest", false),
                    ("VENTILATOR", "ICU_SERVICE", "VentilatorRequest", false),
                    ("ICU_SERVICE", "ICU_BEDS", "BedResult", true),
                    ("ICU_SERVICE", "VENTILATOR", "VentilatorResult", true),
                    ("ICU_SERVICE", "ICU_FILE", "FileIcuResult", true),
                ],
                ("PHARMACIST", "review_prescriptions", "patient_A_and_B", _) => &[
                    ("PHARMACY_FILE", "PHARMACY_SERVICE", "ReviewRequest", false),
                    ("PHARMACY_SERVICE", "PHARMACIST", "ReviewResult", true),
                    (
                        "PHARMACY_SERVICE",
                        "PHARMACY_FILE",
                        "FileReviewResult",
                        true,
                    ),
                ],
                _ => return Err("unassigned clinical effect".into()),
            };
        call.take_score(&mut scored, arguments)?;
        for (from, to, name, after) in relations {
            let (a, b, n) = (role(from)?, role(to)?, label(name)?);
            let matches = frames.iter().filter(|(id, f)| {
                !used.contains(*id)
                    && f.direction(a, b)
                    && f.logical_label == n
                    && if *after {
                        f.sent > ended
                    } else {
                        f.received < call.emitted
                    }
            });
            let (id, frame) = if *after {
                matches.min_by_key(|(_, f)| f.sent)
            } else {
                matches.max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing actual {name}"))?;
            if &frame.value()? != if *after { result } else { arguments } {
                return Err(format!("altered actual {name}").into());
            }
            used.insert(*id);
        }
        if tool == "order_tests" {
            let (svc, er, file, name) = match actor {
                "ER_DOCTOR_A" => (
                    "PATIENT_A_SERVICE",
                    "ER_DOCTOR_A",
                    "ER_A_FILE",
                    "FilePanelResultA",
                ),
                "ER_DOCTOR_B" => (
                    "PATIENT_B_SERVICE",
                    "ER_DOCTOR_B",
                    "ER_B_FILE",
                    "FilePanelResultB",
                ),
                _ => return Err("unassigned panel reply".into()),
            };
            let (a, b, c, n) = (role(svc)?, role(er)?, role(file)?, label(name)?);
            let (id, frame) = frames
                .iter()
                .filter(|(id, f)| {
                    !used.contains(*id)
                        && f.logical_label == n
                        && (f.direction(a, c) || f.direction(b, c))
                        && f.sent > ended
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or("missing actual panel file reply")?;
            if frame.value()? != *result {
                return Err("altered actual panel file reply".into());
            }
            used.insert(*id);
        }
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
            .find(|(_, n)| n.as_u64() == Some(frame.logical_label as u64))
            .map(|(n, _)| n.as_str())
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
                one(ordered.iter().copied().filter(|c| {
                    c.request["agent"] == "RADIOLOGIST"
                        && c.request["tool"] == "perform_imaging"
                        && c.request["arguments"]["patient_id"] == patient
                        && c.request["arguments"]["modality"]
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
            "FindingsA" | "FileFindingsA" => findings(&ordered, "patient_a", frame.sent)?,
            "FindingsB" | "FileFindingsB" => findings(&ordered, "patient_b", frame.sent)?,
            "DiagnosisForAnesthesia"
            | "DiagnosisForSurgery"
            | "FileDiagnosisAnesthesia"
            | "FileDiagnosisSurgery" => {
                json!({"result":completed(&ordered,("ER_DOCTOR_A","diagnose","patient_a",None),frame.sent)?,"findings":findings(&ordered,"patient_a",frame.sent)?})
            }
            "DiagnosisForIcu" | "FileDiagnosisIcu" => {
                json!({"result":completed(&ordered,("ER_DOCTOR_B","diagnose","patient_b",None),frame.sent)?,"findings":findings(&ordered,"patient_b",frame.sent)?})
            }
            "PanelPartial" | "PanelComplete" => {
                let (doctor, patient) = if frame.identity["source"] == role("PATIENT_A_SERVICE")? {
                    ("ER_DOCTOR_A", "patient_a")
                } else if frame.identity["source"] == role("PATIENT_B_SERVICE")? {
                    ("ER_DOCTOR_B", "patient_b")
                } else {
                    return Err("foreign panel data sender".into());
                };
                panel(&ordered, (doctor, patient), frame.sent)?
            }
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
            "SummaryA" | "SummaryB" => {
                let (doctor, patient, care_actor, tool) = if name == "SummaryA" {
                    ("ER_DOCTOR_A", "patient_a", "SURGEON", "perform_surgery")
                } else {
                    ("ER_DOCTOR_B", "patient_b", "ICU_COORDINATOR", "manage_icu")
                };
                let data = findings(&ordered, patient, frame.sent)?;
                json!({"success":true,"assessment":data["assessment"],"tests":data["tests"],"findings":data,"diagnosis":completed(&ordered,(doctor,"diagnose",patient,None),frame.sent)?,"care":completed(&ordered,(care_actor,tool,patient,None),frame.sent)?,"prescription_review":completed(&ordered,("PHARMACIST","review_prescriptions","patient_A_and_B",None),frame.sent)?})
            }
            _ => return Err(format!("unused actual business {name}").into()),
        };
        if frame.value()? != expected {
            return Err(format!("{name} differs from actual patient data").into());
        }
        used.insert(*id);
    }
    let mut overlaps = 0;
    let mut scanners = 0;
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                overlaps += 1;
                if a.request["tool"] == b.request["tool"]
                    && matches!(a.tool()?, "perform_imaging" | "interpret_imaging")
                    && a.request["arguments"]["modality"] != b.request["arguments"]["modality"]
                {
                    scanners += 1;
                }
            }
        }
    }
    Ok(
        json!({"dynamic_protocol":dynamic,"status":"passed","task":"5H","domain_calls":calls.len(),"violations":0,"simulator_complete":true,"common_causal_score":"passed","independent_effect_overlaps":overlaps,"cross_scanner_effect_overlaps":scanners,"effect_receives_bound_to_checked_occurrences":true,"scope":"one-use typed file/resource receive commits, both ICU requests, actual effects and readable findings; EPF owns dynamic order","absent_domain_effects_certified":false}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let (artifact, _) = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = crate::projection::check_dynamic(&artifact, &rows)?;
    check(&rows, &artifact, &dynamic)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("participants")?;
    let journal = crate::journal::read(&rows, &labels, actors, &dynamic)?;
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let business = clinical_frames(&artifact["compiled"])?;
    let selected = journal
        .frames
        .iter()
        .filter_map(|(id, f)| {
            let wire = u16::try_from(f.identity["frame_label"].as_u64()?).ok()?;
            let header = contracts.get(&wire)?;
            (header[4] == 0 && business(f)).then_some(*id)
        })
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let verdicts = crate::journal::json_frame_mutants(&rows, &selected, |changed| {
        check(changed, &artifact, &dynamic)
    })?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,"scope":"saved-record effect audit, not runtime attacks or EPF mutation replay","model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
