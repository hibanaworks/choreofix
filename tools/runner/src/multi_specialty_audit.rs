//! Actual clinical admissions, resource requests and effects; EPF owns progress.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

// The complete frame contract includes its logical endpoints. Metadata/private
// I/O can reuse a business label, and remains checked by EPF rather than by
// this patient-data auditor.
fn clinical_frame(frame: &Frame, compiled: &Value) -> bool {
    let infrastructure = compiled["roles"].as_object().is_some_and(|roles| {
        roles.iter().any(|(name, role)| {
            (name.ends_with("_DATA") || name.ends_with("_IO") || name.ends_with("_VM"))
                && (*role == frame.identity["source"] || *role == frame.identity["target"])
        })
    });
    !infrastructure
        && compiled["labels"].as_object().is_some_and(|labels| {
            labels
                .values()
                .any(|value| value.as_u64() == Some(u64::from(frame.logical_label)))
        })
}

fn completed<'a>(
    calls: &[&'a Call],
    actor: &str,
    tool: &str,
    modality: Option<&str>,
    before: usize,
) -> Result<&'a Value> {
    one(calls.iter().copied().filter(|call| {
        call.request["agent"] == actor
            && call.request["tool"] == tool
            && modality.is_none_or(|m| {
                call.request["arguments"]["modality"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case(m))
            })
            && call.result.as_ref().is_some_and(|(at, _)| *at < before)
    }))?
    .result()
    .map(|(_, value)| value)
}
fn panel(calls: &[&Call], actor: &str, before: usize) -> Result<Value> {
    let mut panel = serde_json::Map::new();
    for call in calls.iter().copied().filter(|call| {
        call.request["agent"] == actor
            && call.request["tool"] == "order_tests"
            && call.result.as_ref().is_some_and(|(at, _)| *at < before)
    }) {
        let result = call.result()?.1;
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
fn case(calls: &[&Call], before: usize) -> Result<Value> {
    Ok(
        json!({"assessment":completed(calls,"ER_DOCTOR","assess_patient",None,before)?,
        "general":panel(calls,"ER_DOCTOR",before)?,
        "CT":completed(calls,"RADIOLOGIST","interpret_imaging",Some("CT"),before)?,
        "MRI":completed(calls,"RADIOLOGIST","interpret_imaging",Some("MRI"),before)?,
        "cardiac":{"result":completed(calls,"CARDIOLOGIST","diagnose",None,before)?,"tests":panel(calls,"CARDIOLOGIST",before)?},
        "neurology":{"result":completed(calls,"NEUROLOGIST","diagnose",None,before)?,"tests":panel(calls,"NEUROLOGIST",before)?},
        "review":completed(calls,"PHARMACIST","review_prescriptions",None,before)?}),
    )
}
pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty multi-specialty evidence")?["message"];
    if compiled["task"] != "5M" || fixture["task"] != "5M" || fixture["type"] != "fixtures" {
        return Err("multi-specialty task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("medical participants")?;
    if actors.len() != 5
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("ER_DOCTOR"),
                Some("CARDIOLOGIST"),
                Some("NEUROLOGIST"),
                Some("RADIOLOGIST"),
                Some("PHARMACIST"),
            ])
    {
        return Err("multi-specialty participants differ".into());
    }
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
    let labels = crate::projection::frames(artifact, rows)?;
    let journal = crate::journal::read(rows, &labels, actors, dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if calls.len() < 12
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe multi-specialty outcome".into());
    }
    let business = |frame: &Frame| clinical_frame(frame, compiled);
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let tests = ordered
        .iter()
        .filter(|c| c.request["tool"] == "order_tests")
        .count();
    for actor in ["ER_DOCTOR", "CARDIOLOGIST", "NEUROLOGIST"] {
        if !ordered
            .iter()
            .any(|c| c.request["agent"] == actor && c.request["tool"] == "order_tests")
        {
            return Err("missing actual panel".into());
        }
    }
    let fixed = [
        "CtOrder",
        "MriOrder",
        "CtOrderForOwner",
        "MriOrderForOwner",
        "CtPerformIntent",
        "MriPerformIntent",
        "CtPerformRequest",
        "MriPerformRequest",
        "CtImageResult",
        "MriImageResult",
        "CtInterpretIntent",
        "MriInterpretIntent",
        "CtInterpretRequest",
        "MriInterpretRequest",
        "CtReportResult",
        "MriReportResult",
        "GeneralForCard",
        "GeneralForNeuro",
        "GeneralForCardOwner",
        "GeneralForNeuroOwner",
        "GeneralForConference",
        "CardPanelForNeuro",
        "NeuroPanelForCard",
        "CtForCard",
        "CtForNeuro",
        "CtForCardOwner",
        "CtForNeuroOwner",
        "CtForER",
        "CtForConference",
        "MriForCard",
        "MriForNeuro",
        "MriForCardOwner",
        "MriForNeuroOwner",
        "MriForER",
        "MriForConference",
        "CardDiagnosisRequest",
        "CardDiagnosisResult",
        "NeuroDiagnosisRequest",
        "NeuroDiagnosisResult",
        "CardDiagnosisForER",
        "NeuroDiagnosisForER",
        "CardDiagnosisForConference",
        "NeuroDiagnosisForConference",
        "AssessmentRequest",
        "AssessmentResult",
        "ReviewRequest",
        "ReviewResult",
        "ReviewForER",
        "ReviewForConference",
        "ConferenceIntent",
        "RoomRequest",
        "RoomResult",
        "FinalForER",
        "FinalForCard",
        "FinalForNeuro",
        "FinalForRad",
        "FileAssessmentResult",
        "CtPerformAdmitted",
        "CtInterpretAdmitted",
        "FileCtImageResult",
        "FileCtReportResult",
        "MriPerformAdmitted",
        "MriInterpretAdmitted",
        "FileMriImageResult",
        "FileMriReportResult",
        "FileCardDiagnosisResult",
        "FileNeuroDiagnosisResult",
        "FileReviewResult",
        "FileConferenceResult",
        "ConferenceAdmitted",
        "FileCardReferral",
        "FileNeuroReferral",
        "FileCtOrder",
        "FileMriOrder",
        "FileCardFindings",
        "FileNeuroFindings",
        "Candidate",
        "GeneralCompletedData",
        "CardiacCompletedData",
        "NeuroCompletedData",
    ];
    for (name, count) in fixed.into_iter().map(|name| (name, 1)).chain([
        ("PanelLoan", tests),
        ("PanelReturn", tests),
        ("PanelCompleted", 3),
        ("TestRequest", tests),
        ("TestResult", tests),
        ("PanelPartial", tests.checked_sub(3).ok_or("panel count")?),
        ("PanelComplete", 3),
        ("CardReferral", 1),
        ("NeuroReferral", 1),
        ("ReferralForOwner", 2),
        ("CtLoan", 4),
        ("MriLoan", 4),
        ("RoomLoan", 2),
    ]) {
        let n = label(name)?;
        if frames
            .values()
            .filter(|f| business(f) && f.logical_label == n)
            .count()
            != count
        {
            return Err(format!("missing or excess {name}").into());
        }
    }
    for (actor, request, admitted, result) in [
        (
            "ER_DOCTOR",
            "GeneralFileRequest",
            "GeneralAdmitted",
            "FileGeneralResult",
        ),
        (
            "CARDIOLOGIST",
            "CardiacFileRequest",
            "CardiacAdmitted",
            "FileCardiacResult",
        ),
        (
            "NEUROLOGIST",
            "NeuroFileRequest",
            "NeuroAdmitted",
            "FileNeuroResult",
        ),
    ] {
        let expected = ordered
            .iter()
            .filter(|call| call.request["agent"] == actor && call.request["tool"] == "order_tests")
            .count();
        for name in [request, admitted, result] {
            if frames
                .values()
                .filter(|f| {
                    business(f) && f.logical_label == label(name).expect("declared file label")
                })
                .count()
                != expected
            {
                return Err(format!("missing or excess {name}").into());
            }
        }
    }
    let mut scored = score["events"].as_array().ok_or("world effects")?.clone();
    if scored.len() != ordered.len() {
        return Err("world and domain effect counts differ".into());
    }
    let mut used = BTreeSet::new();
    for call in &ordered {
        let actor = call.request["agent"].as_str().ok_or("effect actor")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let (_, result) = call.result()?;
        let (ended, _) = call.ended()?;
        if arguments["patient_id"] != "patient_1"
            || result["success"] != true
            || result["data"]["patient_id"] != "patient_1"
            || !result["data"]["artifact_id"].is_string()
        {
            return Err("effect lacks assigned-patient artifact".into());
        }
        let messages: &[(&str, &str, &str, bool)] = match (actor, tool) {
            ("ER_DOCTOR", "assess_patient") => &[
                ("ER_FILE", "GENERAL_SERVICE", "AssessmentRequest", false),
                ("GENERAL_SERVICE", "ER_FILE", "FileAssessmentResult", true),
                ("GENERAL_SERVICE", "ER_DOCTOR", "AssessmentResult", true),
            ],
            ("ER_DOCTOR", "order_tests") => &[
                ("ER_FILE", "GENERAL_SERVICE", "GeneralFileRequest", false),
                ("GENERAL_SERVICE", "ER_DOCTOR", "GeneralAdmitted", false),
                ("GENERAL_PANEL", "GENERAL_SERVICE", "TestRequest", false),
                ("GENERAL_SERVICE", "GENERAL_PANEL", "TestResult", true),
                ("GENERAL_SERVICE", "ER_FILE", "FileGeneralResult", true),
            ],
            ("CARDIOLOGIST", "order_tests") => &[
                (
                    "CARDIO_FILE",
                    "CARDIAC_SERVICE",
                    "CardiacFileRequest",
                    false,
                ),
                ("CARDIAC_SERVICE", "CARDIOLOGIST", "CardiacAdmitted", false),
                ("CARDIAC_PANEL", "CARDIAC_SERVICE", "TestRequest", false),
                ("CARDIAC_SERVICE", "CARDIAC_PANEL", "TestResult", true),
                ("CARDIAC_SERVICE", "CARDIO_FILE", "FileCardiacResult", true),
            ],
            ("NEUROLOGIST", "order_tests") => &[
                ("NEURO_FILE", "NEURO_SERVICE", "NeuroFileRequest", false),
                ("NEURO_SERVICE", "NEUROLOGIST", "NeuroAdmitted", false),
                ("NEURO_PANEL", "NEURO_SERVICE", "TestRequest", false),
                ("NEURO_SERVICE", "NEURO_PANEL", "TestResult", true),
                ("NEURO_SERVICE", "NEURO_FILE", "FileNeuroResult", true),
            ],
            ("CARDIOLOGIST", "diagnose") => &[
                (
                    "CARDIO_FILE",
                    "CARDIAC_SERVICE",
                    "CardDiagnosisRequest",
                    false,
                ),
                (
                    "CARDIAC_SERVICE",
                    "CARDIO_FILE",
                    "FileCardDiagnosisResult",
                    true,
                ),
                (
                    "CARDIAC_SERVICE",
                    "CARDIOLOGIST",
                    "CardDiagnosisResult",
                    true,
                ),
            ],
            ("NEUROLOGIST", "diagnose") => &[
                (
                    "NEURO_FILE",
                    "NEURO_SERVICE",
                    "NeuroDiagnosisRequest",
                    false,
                ),
                (
                    "NEURO_SERVICE",
                    "NEURO_FILE",
                    "FileNeuroDiagnosisResult",
                    true,
                ),
                ("NEURO_SERVICE", "NEUROLOGIST", "NeuroDiagnosisResult", true),
            ],
            ("PHARMACIST", "review_prescriptions") => &[
                ("PHARMACY_FILE", "PHARMACY_SERVICE", "ReviewRequest", false),
                (
                    "PHARMACY_SERVICE",
                    "PHARMACY_FILE",
                    "FileReviewResult",
                    true,
                ),
                ("PHARMACY_SERVICE", "PHARMACIST", "ReviewResult", true),
            ],
            ("RADIOLOGIST", "perform_imaging" | "interpret_imaging") => match (
                arguments["modality"]
                    .as_str()
                    .map(str::to_ascii_uppercase)
                    .as_deref(),
                tool,
            ) {
                (Some("CT"), "perform_imaging") => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtPerformIntent", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtPerformAdmitted", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtPerformRequest", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtImageResult", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtImageResult", true),
                ],
                (Some("CT"), "interpret_imaging") => &[
                    ("RADIOLOGY_FILE", "CT_SERVICE", "CtInterpretIntent", false),
                    ("CT_SERVICE", "RADIOLOGIST", "CtInterpretAdmitted", false),
                    ("CT_SCANNER", "CT_SERVICE", "CtInterpretRequest", false),
                    ("CT_SERVICE", "CT_SCANNER", "CtReportResult", true),
                    ("CT_SERVICE", "RADIOLOGY_FILE", "FileCtReportResult", true),
                ],
                (Some("MRI"), "perform_imaging") => &[
                    ("RADIOLOGY_FILE", "MRI_SERVICE", "MriPerformIntent", false),
                    ("MRI_SERVICE", "RADIOLOGIST", "MriPerformAdmitted", false),
                    ("MRI", "MRI_SERVICE", "MriPerformRequest", false),
                    ("MRI_SERVICE", "MRI", "MriImageResult", true),
                    ("MRI_SERVICE", "RADIOLOGY_FILE", "FileMriImageResult", true),
                ],
                (Some("MRI"), "interpret_imaging") => &[
                    ("RADIOLOGY_FILE", "MRI_SERVICE", "MriInterpretIntent", false),
                    ("MRI_SERVICE", "RADIOLOGIST", "MriInterpretAdmitted", false),
                    ("MRI", "MRI_SERVICE", "MriInterpretRequest", false),
                    ("MRI_SERVICE", "MRI", "MriReportResult", true),
                    ("MRI_SERVICE", "RADIOLOGY_FILE", "FileMriReportResult", true),
                ],
                _ => return Err("unassigned imaging modality".into()),
            },
            ("ER_DOCTOR", "conduct_case_conference") => &[
                ("ER_FILE", "CONFERENCE_SERVICE", "ConferenceIntent", false),
                (
                    "CONFERENCE_SERVICE",
                    "ER_DOCTOR",
                    "ConferenceAdmitted",
                    false,
                ),
                (
                    "CONSULTATION_ROOM",
                    "CONFERENCE_SERVICE",
                    "RoomRequest",
                    false,
                ),
                (
                    "CONFERENCE_SERVICE",
                    "CONSULTATION_ROOM",
                    "RoomResult",
                    true,
                ),
                (
                    "CONFERENCE_SERVICE",
                    "ER_FILE",
                    "FileConferenceResult",
                    true,
                ),
            ],
            _ => return Err("unassigned multi-specialty effect".into()),
        };
        let i = scored
            .iter()
            .position(|e| call.matches_score(e, arguments))
            .ok_or("effect differs from common-world occurrence")?;
        scored.remove(i);
        for &(from, to, name, after) in messages {
            let value = if name == "RoomRequest" {
                json!({"arguments":arguments,"case":case(&ordered,call.emitted)?})
            } else if after {
                result.clone()
            } else {
                arguments.clone()
            };
            let (from, to, number) = (role(from)?, role(to)?, label(name)?);
            let candidates = frames.iter().filter(|(id, frame)| {
                !used.contains(*id)
                    && (frame.direction(from, to)
                        || (after
                            && matches!(
                                name,
                                "FileGeneralResult" | "FileCardiacResult" | "FileNeuroResult"
                            )
                            && frame.direction(
                                role("CONFERENCE_SERVICE").expect("compiled record owner"),
                                to,
                            )))
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
            if frame.value()? != value {
                return Err(format!("altered actual {name}").into());
            }
            used.insert(*id);
        }
        if tool != "assess_patient"
            && tool != "review_prescriptions"
            && result["data"]["inputs"]["assessment"]
                != completed(&ordered, "ER_DOCTOR", "assess_patient", None, call.emitted)?["data"]["artifact_id"]
        {
            return Err("effect refers to another assessment".into());
        }
        if tool == "interpret_imaging"
            && result["data"]["inputs"]["image"]
                != completed(
                    &ordered,
                    "RADIOLOGIST",
                    "perform_imaging",
                    arguments["modality"].as_str(),
                    call.emitted,
                )?["data"]["artifact_id"]
        {
            return Err("interpretation uses another actual image".into());
        }
        if tool == "diagnose" {
            let panel = panel(&ordered, actor, call.emitted)?;
            for (test, receipt) in panel.as_object().ok_or("diagnosis panel")? {
                if result["data"]["inputs"][format!("test:{test}")]
                    != receipt["data"]["artifact_id"]
                {
                    return Err("diagnosis uses another actual test".into());
                }
            }
            for modality in ["CT", "MRI"] {
                if result["data"]["inputs"][format!("imaging:{modality}")]
                    != completed(
                        &ordered,
                        "RADIOLOGIST",
                        "interpret_imaging",
                        Some(modality),
                        call.emitted,
                    )?["data"]["artifact_id"]
                {
                    return Err("diagnosis uses another actual report".into());
                }
            }
        }
        if tool == "conduct_case_conference" {
            for doctor in ["CARDIOLOGIST", "NEUROLOGIST"] {
                if result["data"]["inputs"][format!("diagnosis:{doctor}")]
                    != completed(&ordered, doctor, "diagnose", None, call.emitted)?["data"]["artifact_id"]
                {
                    return Err("conference uses another actual diagnosis".into());
                }
            }
        }
    }
    if !scored.is_empty() {
        return Err("unused world effect".into());
    }
    for (id, f) in &frames {
        if used.contains(id) || !business(f) {
            continue;
        }
        let name = compiled["labels"]
            .as_object()
            .ok_or("message labels")?
            .iter()
            .find(|(_, n)| n.as_u64() == Some(u64::from(f.logical_label)))
            .map(|(name, _)| name.as_str())
            .ok_or("unknown business label")?;
        if matches!(
            name,
            "PanelLoan" | "PanelReturn" | "PanelCompleted" | "CtLoan" | "MriLoan" | "RoomLoan"
        ) {
            used.insert(*id);
            continue;
        }
        let expected = match name {
            "CardReferral" | "NeuroReferral" | "ReferralForOwner" | "CtOrder" | "MriOrder"
            | "CtOrderForOwner" | "MriOrderForOwner" | "FileCardReferral" | "FileNeuroReferral"
            | "FileCtOrder" | "FileMriOrder" => {
                completed(&ordered, "ER_DOCTOR", "assess_patient", None, f.sent)?.clone()
            }
            "PanelPartial" | "PanelComplete" => {
                let owner = f.identity["source"].as_u64().ok_or("panel owner")?;
                let actor = if owner == role("GENERAL_SERVICE")? {
                    "ER_DOCTOR"
                } else if owner == role("CARDIAC_SERVICE")? {
                    "CARDIOLOGIST"
                } else if owner == role("NEURO_SERVICE")? {
                    "NEUROLOGIST"
                } else {
                    return Err("unknown panel owner".into());
                };
                panel(&ordered, actor, f.sent)?
            }
            "GeneralForCard"
            | "GeneralForNeuro"
            | "GeneralForCardOwner"
            | "GeneralForNeuroOwner"
            | "GeneralForConference" => {
                json!({"assessment":completed(&ordered,"ER_DOCTOR","assess_patient",None,f.sent)?,"tests":panel(&ordered,"ER_DOCTOR",f.sent)?})
            }
            "CardPanelForNeuro" => panel(&ordered, "CARDIOLOGIST", f.sent)?,
            "NeuroPanelForCard" => panel(&ordered, "NEUROLOGIST", f.sent)?,
            "CtForCard" | "CtForNeuro" | "CtForCardOwner" | "CtForNeuroOwner" | "CtForER"
            | "CtForConference" => completed(
                &ordered,
                "RADIOLOGIST",
                "interpret_imaging",
                Some("CT"),
                f.sent,
            )?
            .clone(),
            "MriForCard" | "MriForNeuro" | "MriForCardOwner" | "MriForNeuroOwner" | "MriForER"
            | "MriForConference" => completed(
                &ordered,
                "RADIOLOGIST",
                "interpret_imaging",
                Some("MRI"),
                f.sent,
            )?
            .clone(),
            "CardDiagnosisForER" | "CardDiagnosisForConference" => {
                json!({"result":completed(&ordered,"CARDIOLOGIST","diagnose",None,f.sent)?,"tests":panel(&ordered,"CARDIOLOGIST",f.sent)?})
            }
            "NeuroDiagnosisForER" | "NeuroDiagnosisForConference" => {
                json!({"result":completed(&ordered,"NEUROLOGIST","diagnose",None,f.sent)?,"tests":panel(&ordered,"NEUROLOGIST",f.sent)?})
            }
            "ReviewForER" | "ReviewForConference" => {
                completed(&ordered, "PHARMACIST", "review_prescriptions", None, f.sent)?.clone()
            }
            "GeneralCompletedData" | "CardiacCompletedData" | "NeuroCompletedData" => {
                let actor = match name {
                    "GeneralCompletedData" => "ER_DOCTOR",
                    "CardiacCompletedData" => "CARDIOLOGIST",
                    _ => "NEUROLOGIST",
                };
                let last = ordered
                    .iter()
                    .filter(|call| {
                        call.request["agent"] == actor
                            && call.request["tool"] == "order_tests"
                            && call.result.as_ref().is_some_and(|(at, _)| *at < f.sent)
                    })
                    .max_by_key(|call| call.result.as_ref().map(|(at, _)| *at))
                    .ok_or("completed record has no actual last test")?;
                json!({"tests":panel(&ordered,actor,f.sent)?,"attempt":last.result()?.1})
            }
            "Candidate" => case(&ordered, f.sent)?,
            "FileCardFindings" | "FileNeuroFindings" => {
                let (own, other) = if name == "FileCardFindings" {
                    ("CARDIOLOGIST", "NEUROLOGIST")
                } else {
                    ("NEUROLOGIST", "CARDIOLOGIST")
                };
                json!({"general":{"assessment":completed(&ordered,"ER_DOCTOR","assess_patient",None,f.sent)?,"tests":panel(&ordered,"ER_DOCTOR",f.sent)?},"own_tests":panel(&ordered,own,f.sent)?,"other_tests":panel(&ordered,other,f.sent)?,"CT":completed(&ordered,"RADIOLOGIST","interpret_imaging",Some("CT"),f.sent)?,"MRI":completed(&ordered,"RADIOLOGIST","interpret_imaging",Some("MRI"),f.sent)?})
            }
            "FinalForER" | "FinalForCard" | "FinalForNeuro" | "FinalForRad" => {
                json!({"case":case(&ordered,f.sent)?,"conference":completed(&ordered,"ER_DOCTOR","conduct_case_conference",None,f.sent)?})
            }
            _ => return Err(format!("unused {name}").into()),
        };
        if f.value()? != expected {
            return Err(format!("{name} differs from actual patient data").into());
        }
        used.insert(*id);
    }
    let mut overlaps = 0;
    let mut cross_scanner_overlaps = 0;
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                overlaps += 1;
                if a.request["agent"] == "RADIOLOGIST"
                    && b.request["agent"] == "RADIOLOGIST"
                    && a.tool()? == b.tool()?
                    && matches!(a.tool()?, "perform_imaging" | "interpret_imaging")
                    && a.request["arguments"]["modality"] != b.request["arguments"]["modality"]
                {
                    cross_scanner_overlaps += 1;
                }
            }
        }
    }
    Ok(
        json!({"dynamic_protocol":dynamic,"status":"passed","task":"5M","domain_calls":calls.len(),"violations":0,"simulator_complete":true,"common_causal_score":"passed","independent_effect_overlaps":overlaps,"cross_scanner_effect_overlaps":cross_scanner_overlaps,"scope":"one-use typed operation-file and resource receive commits, actual effects and readable findings; EPF owns dynamic order","effect_receives_bound_to_checked_occurrences":true,"absent_domain_effects_certified":false}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let (artifact, _) = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = crate::projection::check_dynamic(&artifact, &rows)?;
    check(&rows, &artifact, &dynamic)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let actors = rows.first().ok_or("fixture")?["message"]["fixtures"]
        .as_array()
        .ok_or("participants")?;
    let journal = crate::journal::read(&rows, &labels, actors, &dynamic)?;
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let selected = journal
        .frames
        .iter()
        .filter_map(|(id, f)| {
            let wire = u16::try_from(f.identity["frame_label"].as_u64()?).ok()?;
            let header = contracts.get(&wire)?;
            (header[4] == 0
                && clinical_frame(f, &artifact["compiled"])
                && artifact["compiled"]["labels"]
                    .as_object()?
                    .values()
                    .any(|n| n.as_u64() == Some(u64::from(f.logical_label))))
            .then_some(*id)
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
