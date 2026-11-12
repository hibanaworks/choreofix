//! Actual clinical admissions, resource requests and effects; EPF owns progress.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

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
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
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
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() < 12
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe multi-specialty outcome".into());
    }
    let pairs = [
        ("ER_DOCTOR", "ER_IO"),
        ("CARDIOLOGIST", "CARDIO_IO"),
        ("NEUROLOGIST", "NEURO_IO"),
        ("RADIOLOGIST", "RADIOLOGY_IO"),
        ("PHARMACIST", "PHARMACY_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let business = |f: &Frame| {
        !pairs
            .iter()
            .any(|(vm, io)| f.direction(*vm, *io) || f.direction(*io, *vm))
    };
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
    let mut scored = score["events"].as_array().ok_or("world effects")?.clone();
    if scored.len() != ordered.len() {
        return Err("world and domain effect counts differ".into());
    }
    let (mut used, mut closes) = (BTreeSet::new(), BTreeSet::new());
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
        let (io, from, owner, request, target, reply, intent) = match (actor, tool) {
            ("ER_DOCTOR", "assess_patient") => (
                "ER_IO",
                "ER_DOCTOR",
                "GENERAL_SERVICE",
                "AssessmentRequest",
                "ER_DOCTOR",
                "AssessmentResult",
                None,
            ),
            ("ER_DOCTOR", "order_tests") => (
                "ER_IO",
                "GENERAL_PANEL",
                "GENERAL_SERVICE",
                "TestRequest",
                "GENERAL_PANEL",
                "TestResult",
                None,
            ),
            ("CARDIOLOGIST", "order_tests") => (
                "CARDIO_IO",
                "CARDIAC_PANEL",
                "CARDIAC_SERVICE",
                "TestRequest",
                "CARDIAC_PANEL",
                "TestResult",
                None,
            ),
            ("NEUROLOGIST", "order_tests") => (
                "NEURO_IO",
                "NEURO_PANEL",
                "NEURO_SERVICE",
                "TestRequest",
                "NEURO_PANEL",
                "TestResult",
                None,
            ),
            ("CARDIOLOGIST", "diagnose") => (
                "CARDIO_IO",
                "CARDIOLOGIST",
                "CARDIAC_SERVICE",
                "CardDiagnosisRequest",
                "CARDIOLOGIST",
                "CardDiagnosisResult",
                None,
            ),
            ("NEUROLOGIST", "diagnose") => (
                "NEURO_IO",
                "NEUROLOGIST",
                "NEURO_SERVICE",
                "NeuroDiagnosisRequest",
                "NEUROLOGIST",
                "NeuroDiagnosisResult",
                None,
            ),
            ("PHARMACIST", "review_prescriptions") => (
                "PHARMACY_IO",
                "PHARMACIST",
                "PHARMACY_SERVICE",
                "ReviewRequest",
                "PHARMACIST",
                "ReviewResult",
                None,
            ),
            ("RADIOLOGIST", "perform_imaging" | "interpret_imaging") => match (
                arguments["modality"]
                    .as_str()
                    .map(str::to_ascii_uppercase)
                    .as_deref(),
                tool,
            ) {
                (Some("CT"), "perform_imaging") => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtPerformRequest",
                    "CT_SCANNER",
                    "CtImageResult",
                    Some("CtPerformIntent"),
                ),
                (Some("CT"), "interpret_imaging") => (
                    "RADIOLOGY_IO",
                    "CT_SCANNER",
                    "CT_SERVICE",
                    "CtInterpretRequest",
                    "CT_SCANNER",
                    "CtReportResult",
                    Some("CtInterpretIntent"),
                ),
                (Some("MRI"), "perform_imaging") => (
                    "RADIOLOGY_IO",
                    "MRI",
                    "MRI_SERVICE",
                    "MriPerformRequest",
                    "MRI",
                    "MriImageResult",
                    Some("MriPerformIntent"),
                ),
                (Some("MRI"), "interpret_imaging") => (
                    "RADIOLOGY_IO",
                    "MRI",
                    "MRI_SERVICE",
                    "MriInterpretRequest",
                    "MRI",
                    "MriReportResult",
                    Some("MriInterpretIntent"),
                ),
                _ => return Err("unassigned imaging modality".into()),
            },
            ("ER_DOCTOR", "conduct_case_conference") => (
                "ER_IO",
                "CONSULTATION_ROOM",
                "CONFERENCE_SERVICE",
                "RoomRequest",
                "CONSULTATION_ROOM",
                "RoomResult",
                Some("ConferenceIntent"),
            ),
            _ => return Err("unassigned multi-specialty effect".into()),
        };
        let i = scored
            .iter()
            .position(|e| {
                e["agent"] == actor
                    && e["tool"] == tool
                    && e["args"] == *arguments
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("effect differs from common-world occurrence")?;
        scored.remove(i);
        let value = if tool == "conduct_case_conference" {
            json!({"arguments":arguments,"case":case(&ordered,call.emitted)?})
        } else {
            arguments.clone()
        };
        let mut before = call.emitted;
        for (from, to, name, value, after) in [
            (from, owner, request, &value, false),
            (owner, target, reply, result, true),
        ] {
            let (from, to, n) = (role(from)?, role(to)?, label(name)?);
            let candidates = frames.iter().filter(|(id, f)| {
                !used.contains(*id)
                    && f.direction(from, to)
                    && f.logical_label == n
                    && if after {
                        f.sent > ended
                    } else {
                        f.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, f) = if after {
                candidates.min_by_key(|(_, f)| f.sent)
            } else {
                candidates.max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing actual {name}"))?;
            if f.value()? != *value {
                return Err(format!("altered {name}").into());
            }
            f.received()?;
            used.insert(*id);
            if !after {
                before = before.min(f.sent);
            }
        }
        if let Some(intent) = intent {
            let (sender, to, n) = (
                role(if tool == "conduct_case_conference" {
                    "ER_DOCTOR"
                } else {
                    "RADIOLOGIST"
                })?,
                role(owner)?,
                label(intent)?,
            );
            let candidates = frames
                .iter()
                .filter(|(id, f)| {
                    !used.contains(*id) && f.direction(sender, to) && f.logical_label == n
                })
                .collect::<Vec<_>>();
            let (id, f) = *one(candidates.iter())?;
            if f.value()? != value || f.received()? >= call.emitted {
                return Err("intent differs from real resource request".into());
            }
            used.insert(*id);
            before = before.min(f.sent);
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
            f.received()?;
            used.insert(*id);
            continue;
        }
        let expected = match name {
            "CardReferral" | "NeuroReferral" | "ReferralForOwner" | "CtOrder" | "MriOrder"
            | "CtOrderForOwner" | "MriOrderForOwner" => {
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
            "FinalForER" | "FinalForCard" | "FinalForNeuro" | "FinalForRad" => {
                json!({"case":case(&ordered,f.sent)?,"conference":completed(&ordered,"ER_DOCTOR","conduct_case_conference",None,f.sent)?})
            }
            _ => return Err(format!("unused {name}").into()),
        };
        if f.value()? != expected {
            return Err(format!("{name} differs from actual patient data").into());
        }
        f.received()?;
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
        json!({"status":"passed","task":"5M","domain_calls":calls.len(),"violations":0,"simulator_complete":true,"common_causal_score":"passed","independent_effect_overlaps":overlaps,"scope":"one-use actual WASI admissions, real resource requests and completed clinical findings; EPF owns dynamic order","absent_domain_effects_certified":false}),
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
        .filter_map(|(id, f)| {
            let wire = u16::try_from(f.identity["frame_label"].as_u64()?).ok()?;
            let header = contracts.get(&wire)?;
            (header[4] == 0
                && artifact["compiled"]["labels"]
                    .as_object()?
                    .values()
                    .any(|n| n.as_u64() == Some(u64::from(f.logical_label))))
            .then_some(*id)
        })
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,"scope":"saved-record effect audit, not runtime attacks or EPF mutation replay","model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
