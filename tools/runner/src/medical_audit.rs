//! Bind actual patient inputs, received requests and results to effects.
//! Dynamic protocol order and affine custody are checked by EPF.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn completed<'a>(calls: &[&'a Call], actor: &str, tool: &str, before: usize) -> Result<&'a Value> {
    let call = one(calls.iter().copied().filter(|call| {
        call.request["agent"] == actor
            && call.request["tool"] == tool
            && call.result.as_ref().is_some_and(|(at, _)| *at < before)
    }))?;
    Ok(call.result()?.1)
}

// Reconstruct actual current panel contents, not an expected-next-operation cursor.
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

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty medical evidence")?["message"];
    if compiled["task"] != "5E" || fixture["task"] != "5E" || fixture["type"] != "fixtures" {
        return Err("medical task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("medical participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|actor| actor["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("ER_DOCTOR"), Some("CARDIOLOGIST"), Some("PHARMACIST")])
    {
        return Err("medical participants differ".into());
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
    if calls.len() < 6
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe consultation outcome".into());
    }
    let pairs = [
        ("ER_DOCTOR", "ER_IO"),
        ("CARDIOLOGIST", "CARDIO_IO"),
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
    let mut scored = score["events"]
        .as_array()
        .ok_or("common-world effects")?
        .clone();
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
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
            return Err("effect lacks its successful assigned-patient artifact".into());
        }
        let (io, from, owner, request, result_target, reply) = match (actor, tool) {
            ("ER_DOCTOR", "assess_patient") => (
                "ER_IO",
                "ER_DOCTOR",
                "GENERAL_SERVICE",
                "AssessmentRequest",
                "ER_DOCTOR",
                "AssessmentResult",
            ),
            ("ER_DOCTOR", "order_tests") => (
                "ER_IO",
                "GENERAL_PANEL",
                "GENERAL_SERVICE",
                "GeneralTests",
                "GENERAL_PANEL",
                "GeneralTestResults",
            ),
            ("CARDIOLOGIST", "order_tests") => (
                "CARDIO_IO",
                "CARDIAC_PANEL",
                "CARDIAC_SERVICE",
                "CardiacTests",
                "CARDIAC_PANEL",
                "CardiacTestResults",
            ),
            ("ER_DOCTOR", "perform_imaging") => (
                "ER_IO",
                "CT_SCANNER",
                "IMAGING_SERVICE",
                "ImagingRequest",
                "CT_SCANNER",
                "ImagingResult",
            ),
            ("CARDIOLOGIST", "diagnose") => (
                "CARDIO_IO",
                "CARDIOLOGIST",
                "CARDIAC_SERVICE",
                "DiagnosisRequest",
                "CARDIOLOGIST",
                "DiagnosisResult",
            ),
            ("PHARMACIST", "review_prescriptions") => (
                "PHARMACY_IO",
                "PHARMACIST",
                "PHARMACY_SERVICE",
                "ReviewRequest",
                "PHARMACIST",
                "ReviewResult",
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
            .ok_or("clinical effect differs from its common-world occurrence")?;
        scored.remove(index);
        let mut before = call.emitted;
        for (from, to, name, value, after) in [
            (from, owner, request, arguments, false),
            (owner, result_target, reply, result, true),
        ] {
            let (from, to, number) = (role(from)?, role(to)?, label(name)?);
            let candidates = frames.iter().filter(|(id, frame)| {
                !used_frames.contains(*id)
                    && frame.direction(from, to)
                    && frame.logical_label == number
                    && if after {
                        frame.sent > ended
                    } else {
                        frame.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, frame) = if after {
                candidates.min_by_key(|(_, frame)| frame.sent)
            } else {
                candidates.max_by_key(|(_, frame)| frame.received)
            }
            .ok_or_else(|| format!("missing actual {name}"))?;
            if frame.value()? != *value {
                return Err(format!("altered actual {name}").into());
            }
            frame.received()?;
            used_frames.insert(*id);
            if !after {
                before = before.min(frame.sent);
            }
        }
        if tool == "perform_imaging" {
            let number = label("ImagingIntent")?;
            let candidates = frames
                .iter()
                .filter(|(id, frame)| {
                    !used_frames.contains(*id) && frame.logical_label == number && business(frame)
                })
                .collect::<Vec<_>>();
            let &(id, intent) = one(candidates.iter())?;
            if intent.value()? != *arguments || intent.received()? >= call.emitted {
                return Err("CT effect differs from its admitted intent".into());
            }
            used_frames.insert(*id);
            before = before.min(intent.sent);
        }
        close_arguments(
            &frames,
            &mut used_closes,
            role(actor)?,
            role(io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
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
            frame.received()?;
            used_frames.insert(*id);
            continue;
        }
        let expected = match name {
            "Referral" | "ReferralForOwner" => {
                completed(&ordered, "ER_DOCTOR", "assess_patient", frame.sent)?.clone()
            }
            "GeneralFindings" | "FindingsForOwner" => json!({
                "assessment":completed(&ordered,"ER_DOCTOR","assess_patient",frame.sent)?,
                "tests":panel(&ordered,"ER_DOCTOR",frame.sent)?,
                "imaging":completed(&ordered,"ER_DOCTOR","perform_imaging",frame.sent)?,
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
            "CardiacTestsPartial" | "CardiacTestsComplete" => {
                panel(&ordered, "CARDIOLOGIST", frame.sent)?
            }
            _ => return Err(format!("unused or extra {name} frame").into()),
        };
        if frame.value()? != expected {
            return Err(format!("{name} differs from actual completed patient data").into());
        }
        frame.received()?;
        used_frames.insert(*id);
    }
    let mut overlaps = 0;
    let mut panel_imaging_overlaps = 0;
    for (index, a) in ordered.iter().enumerate() {
        for b in &ordered[index + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                if a.tool()? == "perform_imaging" && b.tool()? == "perform_imaging" {
                    return Err("overlapping actual CT effects".into());
                }
                overlaps += 1;
                if matches!(
                    (a.tool()?, b.tool()?),
                    ("order_tests", "perform_imaging") | ("perform_imaging", "order_tests")
                ) {
                    panel_imaging_overlaps += 1;
                }
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"5E","domain_calls":calls.len(),"violations":0,
        "simulator_complete":true,"common_causal_score":"passed","independent_effect_overlaps":overlaps,
        "panel_imaging_overlaps":panel_imaging_overlaps,
        "scope":"one-use actual WASI admissions, clinical requests, findings and actual results; EPF owns dynamic order",
        "absent_domain_effects_certified":false}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
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
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,
        "scope":"saved-record clinical effect audit, not runtime attacks or EPF mutation replay","model_calls":0,
        "verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
