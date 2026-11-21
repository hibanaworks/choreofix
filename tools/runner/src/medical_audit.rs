//! Bind actual patient inputs, received requests and results to effects.
//! Dynamic protocol order and affine custody are checked by EPF.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn completed<'a>(calls: &[&'a Call], actor: &str, tool: &str, before: usize) -> Result<&'a Value> {
    let call = one(calls.iter().copied().filter(|call| {
        call.request["agent"] == actor && call.request["tool"] == tool && call.result.0 < before
    }))?;
    Ok(call.result.1)
}

// Reconstruct actual current panel contents, not an expected-next-operation cursor.
fn panel(calls: &[&Call], actor: &str, before: usize) -> Result<Value> {
    let mut panel = serde_json::Map::new();
    for call in calls.iter().copied().filter(|call| {
        call.request["agent"] == actor
            && call.request["tool"] == "order_tests"
            && call.result.0 < before
    }) {
        let result = call.result.1;
        for test in result["data"]["tests"]
            .as_array()
            .ok_or("actual panel tests")?
        {
            panel.insert(
                test.as_str().ok_or("actual test name")?.to_owned(),
                result.clone(),
            );
        }
    }
    Ok(Value::Object(panel))
}

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty medical evidence")?["message"];
    if compiled["task"] != "5E" || fixture["task"] != "5E" || fixture["type"] != "fixtures" {
        return Err("medical task differs".into());
    }
    let journal = crate::journal::read(rows, artifact, dynamic)?;

    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u8> {
        compiled["labels"][name]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| format!("compiled label {name}").into())
    };
    let records_actor = role("RECORDS")?;
    let crate::journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if calls.len() < 6
        || !simulator_complete
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe consultation outcome".into());
    }
    let business = |frame: &Frame| {
        compiled["labels"].as_object().is_some_and(|labels| {
            labels
                .values()
                .any(|label| label.as_u64() == Some(u64::from(frame.logical_label)))
        })
    };
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let general_calls = ordered
        .iter()
        .filter(|call| {
            call.request["agent"] == "ER_DOCTOR" && call.request["tool"] == "order_tests"
        })
        .count();
    let cardiac_calls = ordered
        .iter()
        .filter(|call| {
            call.request["agent"] == "CARDIOLOGIST" && call.request["tool"] == "order_tests"
        })
        .count();
    if general_calls == 0 || cardiac_calls == 0 {
        return Err("missing actual patient panel".into());
    }
    for (name, count) in [
        ("AssessmentRequest", 1),
        ("AssessmentResult", 1),
        ("GeneralTests", general_calls),
        ("GeneralTestResults", general_calls),
        ("CardiacTests", cardiac_calls),
        ("CardiacTestResults", cardiac_calls),
        ("ImagingIntent", 1),
        ("ImagingRequest", 1),
        ("ImagingResult", 1),
        ("CtLoan", 2),
        ("Referral", 1),
        ("ReferralForOwner", 1),
        ("GeneralFindings", 1),
        ("FindingsForOwner", 1),
        ("DiagnosisRequest", 1),
        ("DiagnosisResult", 1),
        ("CardiacFindings", 1),
        ("ReviewRequest", 1),
        ("ReviewResult", 1),
        ("MedicationReview", 1),
        ("GeneralPanelLoan", general_calls),
        ("GeneralPanelReturn", general_calls),
        ("GeneralPanelCompleted", 1),
        ("GeneralTestsPartial", general_calls - 1),
        ("GeneralTestsComplete", 1),
        ("CardiacPanelLoan", cardiac_calls),
        ("CardiacPanelReturn", cardiac_calls),
        ("CardiacPanelCompleted", 1),
        ("CardiacTestsPartial", cardiac_calls - 1),
        ("CardiacTestsComplete", 1),
        ("CardiacPanelReport", 1),
        ("FileAssessmentResult", 1),
        ("GeneralFileRequest", general_calls),
        ("GeneralAdmitted", general_calls),
        ("FileGeneralResult", general_calls),
        ("ImagingAdmitted", 1),
        ("FileImagingResult", 1),
        ("FileReferral", 1),
        ("FileFindings", 1),
        ("FileDiagnosisResult", 1),
        ("FileReviewResult", 1),
        ("Summary", 1),
        ("CardiacFileRequest", cardiac_calls),
        ("CardiacAdmitted", cardiac_calls),
        ("FileCardiacResult", cardiac_calls),
        ("GeneralCompletedData", 1),
        ("CardiacCompletedData", 1),
        ("CompletedConsultation", 1),
    ] {
        let number = label(name)?;
        if frames
            .values()
            .filter(|frame| business(frame) && frame.logical_label == number)
            .count()
            != count
        {
            return Err(format!("missing or extra {name} occurrence").into());
        }
    }
    let mut scored = scored_events.iter().collect::<Vec<_>>();
    let mut used_frames = BTreeSet::new();
    for call in &ordered {
        let actor = call.request["agent"].as_str().ok_or("effect actor")?;
        let tool = call.tool;
        let arguments = &call.request["arguments"];
        let (_, result) = call.result;
        let (ended, _) = call.ended;
        if arguments["patient_id"] != "patient_1"
            || result["success"] != true
            || result["data"]["patient_id"] != "patient_1"
            || !result["data"]["artifact_id"].is_string()
        {
            return Err("effect lacks its successful assigned-patient artifact".into());
        }
        let messages: &[(&str, &str, &str, bool)] = match (actor, tool) {
            ("ER_DOCTOR", "assess_patient") => &[
                ("ASSESS_FILE", "GENERAL_SERVICE", "AssessmentRequest", false),
                (
                    "GENERAL_SERVICE",
                    "ASSESS_FILE",
                    "FileAssessmentResult",
                    true,
                ),
                ("GENERAL_SERVICE", "ER_DOCTOR", "AssessmentResult", true),
            ],
            ("ER_DOCTOR", "order_tests") => &[
                (
                    "GENERAL_FILE",
                    "GENERAL_SERVICE",
                    "GeneralFileRequest",
                    false,
                ),
                ("GENERAL_SERVICE", "ER_DOCTOR", "GeneralAdmitted", false),
                ("GENERAL_PANEL", "GENERAL_SERVICE", "GeneralTests", false),
                (
                    "GENERAL_SERVICE",
                    "GENERAL_PANEL",
                    "GeneralTestResults",
                    true,
                ),
                ("GENERAL_SERVICE", "GENERAL_FILE", "FileGeneralResult", true),
            ],
            ("CARDIOLOGIST", "order_tests") => &[
                (
                    "CARDIAC_FILE",
                    "CARDIAC_SERVICE",
                    "CardiacFileRequest",
                    false,
                ),
                ("CARDIAC_SERVICE", "CARDIOLOGIST", "CardiacAdmitted", false),
                ("CARDIAC_PANEL", "CARDIAC_SERVICE", "CardiacTests", false),
                (
                    "CARDIAC_SERVICE",
                    "CARDIAC_PANEL",
                    "CardiacTestResults",
                    true,
                ),
                ("CARDIAC_SERVICE", "CARDIAC_FILE", "FileCardiacResult", true),
            ],
            ("ER_DOCTOR", "perform_imaging") => &[
                ("IMAGING_FILE", "IMAGING_SERVICE", "ImagingIntent", false),
                ("IMAGING_SERVICE", "ER_DOCTOR", "ImagingAdmitted", false),
                ("CT_SCANNER", "IMAGING_SERVICE", "ImagingRequest", false),
                ("IMAGING_SERVICE", "CT_SCANNER", "ImagingResult", true),
                ("IMAGING_SERVICE", "IMAGING_FILE", "FileImagingResult", true),
            ],
            ("CARDIOLOGIST", "diagnose") => &[
                (
                    "DIAGNOSIS_FILE",
                    "CARDIAC_SERVICE",
                    "DiagnosisRequest",
                    false,
                ),
                (
                    "CARDIAC_SERVICE",
                    "DIAGNOSIS_FILE",
                    "FileDiagnosisResult",
                    true,
                ),
                ("CARDIAC_SERVICE", "CARDIOLOGIST", "DiagnosisResult", true),
            ],
            ("PHARMACIST", "review_prescriptions") => &[
                ("REVIEW_FILE", "PHARMACY_SERVICE", "ReviewRequest", false),
                ("PHARMACY_SERVICE", "REVIEW_FILE", "FileReviewResult", true),
                ("PHARMACY_SERVICE", "PHARMACIST", "ReviewResult", true),
            ],
            _ => return Err("unassigned clinical effect".into()),
        };
        call.take_score(&mut scored, arguments)?;
        for &(from, to, name, after) in messages {
            let value = if after { result } else { arguments };
            let (from, to, number) = (role(from)?, role(to)?, label(name)?);
            let candidates = frames.iter().filter(|(id, frame)| {
                !used_frames.contains(*id)
                    && (frame.direction(from, to)
                        || (after
                            && matches!(name, "FileGeneralResult" | "FileCardiacResult")
                            && frame.direction(records_actor, to)))
                    && frame.logical_label == number
                    && if after {
                        frame.sent > ended
                    } else {
                        frame.received < call.emitted
                    }
            });
            let (id, frame) = if after {
                candidates.min_by_key(|(_, f)| f.sent)
            } else {
                candidates.max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing actual {name}"))?;
            if frame.value()? != *value {
                return Err(format!("altered actual {name}").into());
            }
            used_frames.insert(*id);
        }
    }
    if !scored.is_empty() {
        return Err("unused common-world clinical effect".into());
    }
    for (id, frame) in &frames {
        if used_frames.contains(id) || !business(frame) {
            continue;
        }
        let name = compiled["labels"]
            .as_object()
            .ok_or("clinical label names")?
            .iter()
            .find(|(_, number)| number.as_u64() == Some(u64::from(frame.logical_label)))
            .map(|(name, _)| name.as_str())
            .ok_or("unknown clinical communication")?;
        if matches!(
            name,
            "CtLoan"
                | "GeneralPanelLoan"
                | "GeneralPanelReturn"
                | "GeneralPanelCompleted"
                | "CardiacPanelLoan"
                | "CardiacPanelReturn"
                | "CardiacPanelCompleted"
        ) {
            // Static projection and EPF validate the actual owned carrier; it
            // is not a JSON address, receipt or another application owner map.
            used_frames.insert(*id);
            continue;
        }
        let expected = match name {
            "Referral" | "ReferralForOwner" | "FileReferral" => {
                completed(&ordered, "ER_DOCTOR", "assess_patient", frame.sent)?.clone()
            }
            "GeneralFindings" | "FindingsForOwner" | "FileFindings" => json!({
                "assessment":completed(&ordered,"ER_DOCTOR","assess_patient",frame.sent)?,
                "tests":panel(&ordered,"ER_DOCTOR",frame.sent)?,
                "imaging":completed(&ordered,"ER_DOCTOR","perform_imaging",frame.sent)?,
                "cardiac_tests":panel(&ordered,"CARDIOLOGIST",frame.sent)?,
            }),
            "CardiacFindings" => {
                completed(&ordered, "CARDIOLOGIST", "diagnose", frame.sent)?.clone()
            }
            "MedicationReview" => {
                completed(&ordered, "PHARMACIST", "review_prescriptions", frame.sent)?.clone()
            }
            "GeneralTestsPartial" | "GeneralTestsComplete" => {
                panel(&ordered, "ER_DOCTOR", frame.sent)?
            }
            "CardiacTestsPartial" | "CardiacTestsComplete" | "CardiacPanelReport" => {
                panel(&ordered, "CARDIOLOGIST", frame.sent)?
            }
            "GeneralCompletedData" | "CardiacCompletedData" => {
                let actor = if name == "GeneralCompletedData" {
                    "ER_DOCTOR"
                } else {
                    "CARDIOLOGIST"
                };
                let last = ordered
                    .iter()
                    .filter(|call| {
                        call.request["agent"] == actor
                            && call.request["tool"] == "order_tests"
                            && call.result.0 < frame.sent
                    })
                    .max_by_key(|call| call.result.0)
                    .ok_or("completed panel has no actual last result")?;
                json!({"tests":panel(&ordered,actor,frame.sent)?,"attempt":last.result.1})
            }
            "Summary" | "CompletedConsultation" => json!({
                "clinical":{"findings":{
                    "assessment":completed(&ordered,"ER_DOCTOR","assess_patient",frame.sent)?,
                    "tests":panel(&ordered,"ER_DOCTOR",frame.sent)?,
                    "imaging":completed(&ordered,"ER_DOCTOR","perform_imaging",frame.sent)?,
                    "cardiac_tests":panel(&ordered,"CARDIOLOGIST",frame.sent)?,
                },"diagnosis":completed(&ordered,"CARDIOLOGIST","diagnose",frame.sent)?},
                "medication_review":completed(&ordered,"PHARMACIST","review_prescriptions",frame.sent)?,
            }),
            _ => return Err(format!("unused or extra {name} frame").into()),
        };
        if frame.value()? != expected {
            return Err(format!("{name} differs from actual completed patient data").into());
        }
        used_frames.insert(*id);
    }
    let mut overlaps = 0;
    let mut panel_imaging_overlaps = 0;
    for (index, a) in ordered.iter().enumerate() {
        for b in &ordered[index + 1..] {
            if a.started.1 < b.ended.1 && b.started.1 < a.ended.1 {
                if a.tool == "perform_imaging" && b.tool == "perform_imaging" {
                    return Err("overlapping actual CT effects".into());
                }
                overlaps += 1;
                if matches!(
                    (a.tool, b.tool),
                    ("order_tests", "perform_imaging") | ("perform_imaging", "order_tests")
                ) {
                    panel_imaging_overlaps += 1;
                }
            }
        }
    }
    Ok(
        json!({"dynamic_protocol":dynamic,"status":"passed","task":"5E","domain_calls":calls.len(),"violations":0,
        "simulator_complete":true,"common_causal_score":"passed","independent_effect_overlaps":overlaps,
        "panel_imaging_overlaps":panel_imaging_overlaps,
        "scope":"one-use typed file and resource receive commits, actual effects, results and readable findings; EPF owns dynamic order",
        "effect_receives_bound_to_checked_occurrences":true,
        "absent_domain_effects_certified":false}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let (artifact, _) = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = crate::projection::check_dynamic(&artifact, &rows)?;
    check(&rows, &artifact, &dynamic)?;
    let journal = crate::journal::read(&rows, &artifact, &dynamic)?;
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let selected = journal
        .frames
        .iter()
        .filter_map(|(id, frame)| {
            let wire = frame.identity["frame_label"]
                .as_u64()
                .and_then(|wire| u16::try_from(wire).ok())?;
            let header = contracts.get(&wire)?;
            (header[4] == 0
                && artifact["compiled"]["labels"]
                    .as_object()?
                    .values()
                    .any(|label| label.as_u64() == Some(u64::from(frame.logical_label))))
            .then_some(*id)
        })
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let verdicts = crate::journal::json_frame_mutants(&rows, &selected, |changed| {
        check(changed, &artifact, &dynamic)
    })?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,
        "scope":"saved-record clinical effect audit, not runtime attacks or EPF mutation replay","model_calls":0,
        "verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
