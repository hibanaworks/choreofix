//! One-use scientific admissions and actual effects; EPF owns source progress.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, check_close_pairs, close_arguments, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn step(call: &Call) -> Result<u8> {
    let a = &call.request["arguments"];
    Ok(match (call.request["agent"].as_str(), call.tool()?) {
        (Some("CHEMIST"), "run_hplc_analysis") if a == &json!({"sample":"synthesis_qc"}) => 0,
        (Some("BIOLOGIST"), "run_cell_assay") if a == &json!({"assay_type":"culture_prep"}) => 1,
        (Some("TOXICOLOGIST"), "run_mass_spec_analysis")
            if a == &json!({"sample":"baseline_markers"}) =>
        {
            2
        }
        (Some("CHEMIST"), "run_mass_spec_analysis") if a == &json!({"sample":"molecular_id"}) => 3,
        (Some("FORMULATION_SCIENTIST"), "prepare_formulation")
            if a.as_object().is_some_and(|v| v.len() == 1) && a["compound"].is_string() =>
        {
            4
        }
        (Some("BIOLOGIST"), "run_cell_assay") if a == &json!({"assay_type":"bioassay"}) => 5,
        (Some("BIOLOGIST"), "run_hplc_analysis") if a == &json!({"sample":"stability"}) => 6,
        (Some("TOXICOLOGIST"), "run_hplc_analysis") if a == &json!({"sample":"purity_check"}) => 7,
        (Some("TOXICOLOGIST"), "run_cell_assay") if a == &json!({"assay_type":"cytotoxicity"}) => 8,
        (Some("CLINICAL_LEAD"), "review_clinical_data")
            if a == &json!({"data_type":"combined_review"}) =>
        {
            9
        }
        (Some("REGULATORY_SPECIALIST"), "verify_regulatory_compliance")
            if a == &json!({"review_type":"initial_review"}) =>
        {
            10
        }
        (Some("REGULATORY_SPECIALIST"), "verify_regulatory_compliance")
            if a == &json!({"review_type":"hold_review"}) =>
        {
            11
        }
        (Some("PROJECT_DIRECTOR"), "submit_project_decision")
            if a == &json!({"outcome":"continue"}) =>
        {
            12
        }
        (Some("PROJECT_DIRECTOR"), "submit_project_decision")
            if a == &json!({"outcome":"retry"}) =>
        {
            13
        }
        (Some("PROJECT_DIRECTOR"), "submit_project_decision")
            if a == &json!({"outcome":"abort"}) =>
        {
            14
        }
        (Some("CHEMIST"), "run_hplc_analysis") if a == &json!({"sample":"documentation"}) => 15,
        _ => return Err("unassigned scientific effect or changed arguments".into()),
    })
}

fn prior<'a>(calls: &[&'a Call], at: usize, steps: &[u8]) -> Result<&'a Value> {
    calls
        .iter()
        .copied()
        .filter(|c| {
            step(c).is_ok_and(|s| steps.contains(&s))
                && c.result.as_ref().is_some_and(|(end, _)| *end < at)
        })
        .max_by_key(|c| c.result.as_ref().map(|(end, _)| *end))
        .ok_or("missing actual prior scientific result")?
        .result()
        .map(|(_, v)| v)
}
fn event(calls: &[&Call], at: usize, s: u8) -> Result<Value> {
    let value = &prior(calls, at, &[s])?["data"]["event_id"];
    if !value.is_u64() {
        return Err("scientific occurrence has no actual event ID".into());
    }
    Ok(value.clone())
}
fn lineage(calls: &[&Call], at: usize, s: u8) -> Result<Value> {
    let keys: &[(&str, u8)] = match s {
        0..=2 => &[],
        3 => &[("qc", 0)],
        4 => &[("compound", 3)],
        5 | 6 => &[("formulation", 4), ("cells", 1)],
        7 => &[("formulation", 4)],
        8 => &[
            ("formulation", 4),
            ("cells", 1),
            ("baseline", 2),
            ("purity", 7),
        ],
        9 => &[
            ("formulation", 4),
            ("bioassay", 5),
            ("stability", 6),
            ("purity", 7),
            ("toxicity", 8),
        ],
        10 | 11 => &[("clinical", 9)],
        12..=14 => &[("compound", 3), ("formulation", 4), ("clinical", 9)],
        15 => &[("compound", 3), ("formulation", 4), ("clinical", 9)],
        _ => return Err("unknown scientific lineage".into()),
    };
    let mut value = json!({});
    for &(key, s) in keys {
        value[key] = event(calls, at, s)?;
    }
    if s >= 12 {
        value["regulatory"] = prior(calls, at, &[10, 11])?["data"]["event_id"].clone();
    }
    if s == 15 {
        value["decision"] = prior(calls, at, &[12, 13, 14])?["data"]["event_id"].clone();
    }
    Ok(value)
}
fn outcome(calls: &[&Call], at: usize) -> Result<Value> {
    let decision = prior(calls, at, &[12, 13, 14])?;
    Ok(
        json!({"outcome":decision["data"]["outcome"],"compound":prior(calls,at,&[3])?,
        "formulation":prior(calls,at,&[4])?,"clinical":prior(calls,at,&[9])?,
        "regulatory":prior(calls,at,&[10,11])?,"decision":decision}),
    )
}
fn take<'a>(
    frames: &'a BTreeMap<u64, Frame>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<(u64, &'a Frame)> {
    let (&id, f) = frames
        .iter()
        .filter(|(id, f)| !used.contains(id) && matches(f))
        .min_by_key(|(_, f)| f.sent)
        .ok_or("missing one-use scientific frame")?;
    used.insert(id);
    Ok((id, f))
}
fn same(a: &Value, b: &Value) -> Result<()> {
    if a != b {
        return Err("scientific value differs from actual admitted data".into());
    }
    Ok(())
}
fn instruments(s: u8) -> &'static [&'static str] {
    match s {
        0 | 6 | 7 | 15 => &["HPLC"],
        1 => &["CELL_LAB"],
        2 | 3 | 9 => &["MASS_SPEC"],
        4 => &["FORMULATION_SUITE"],
        5 | 8 => &["CELL_LAB", "BIOLOGICAL_SAMPLES"],
        11 => &["HPLC", "MASS_SPEC"],
        _ => &[],
    }
}

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty scientific evidence")?["message"];
    if compiled["task"] != "14H" || fixture["task"] != "14H" || fixture["type"] != "fixtures" {
        return Err("scientific fixture or compiled task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("scientist population")?;
    let pairs = [
        ("CHEMIST", "CHEMIST_IO"),
        ("BIOLOGIST", "BIOLOGIST_IO"),
        ("TOXICOLOGIST", "TOXICOLOGIST_IO"),
        ("FORMULATION_SCIENTIST", "FORMULATOR_IO"),
        ("CLINICAL_LEAD", "CLINICIAN_IO"),
        ("REGULATORY_SPECIALIST", "REGULATOR_IO"),
        ("PROJECT_DIRECTOR", "DIRECTOR_IO"),
    ];
    if actors.len() != pairs.len()
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != pairs.iter().map(|(a, _)| Some(*a)).collect()
    {
        return Err("scientific participant population differs".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("missing compiled role {name}").into())
    };
    let ios = pairs
        .iter()
        .map(|(a, b)| Ok((role(a)?, role(b)?)))
        .collect::<Result<Vec<_>>>()?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.is_empty()
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe scientific study".into());
    }
    check_close_pairs(&frames, &ios)?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let mut scored = score["events"]
        .as_array()
        .ok_or("scientific score events")?
        .clone();
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut receipts = Vec::new();
    for call in &ordered {
        let s = step(call)?;
        let agent = call.request["agent"].as_str().ok_or("scientist")?;
        let vm = role(agent)?;
        let io = ios.iter().find(|(a, _)| *a == vm).ok_or("scientist I/O")?.1;
        let arguments = &call.request["arguments"];
        let (ended, result) = call.result()?;
        let id = result["data"]["event_id"]
            .as_u64()
            .ok_or("scientific result event ID")?;
        if !ids.insert(id)
            || !result["success"].is_boolean()
            || result["data"]["scientist"] != agent
            || (result["success"] == true && result["data"]["artifact_id"] != id)
            || (result["success"] == false && !result["data"]["artifact_id"].is_null())
        {
            return Err("reused, altered or misattributed scientific result".into());
        }
        same(
            &result["data"]["inputs"],
            &lineage(&ordered, call.emitted, s)?,
        )?;
        if s == 4 {
            same(
                arguments,
                &json!({"compound":prior(&ordered,call.emitted,&[3])?["data"]["compound"]}),
            )?;
        }
        let at = scored
            .iter()
            .position(|e| {
                e["agent"] == agent
                    && e["tool"] == call.request["tool"]
                    && e["args"] == *arguments
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("scientific effect differs from common-world result")?;
        scored.remove(at);
        let (service, cap, requests, request_label) = match s {
            0 => ("HPLC_SERVICE", "QC", vec!["HPLC"], 2),
            1 => ("CELL_SERVICE", "CULTURE", vec!["CELL_LAB"], 2),
            2 | 3 => ("MASS_SERVICE", agent, vec!["MASS_SPEC"], 2),
            4 => ("FORM_SERVICE", "FORMULATION", vec!["FORMULATION_SUITE"], 2),
            5 => (
                "CELL_SERVICE",
                "BIOASSAY",
                vec!["CELL_LAB", "BIOLOGICAL_SAMPLES"],
                2,
            ),
            6 => ("HPLC_SERVICE", "STABILITY", vec!["HPLC"], 2),
            7 => ("HPLC_SERVICE", "PURITY", vec!["HPLC"], 2),
            8 => (
                "CELL_SERVICE",
                "TOXICITY",
                vec!["CELL_LAB", "BIOLOGICAL_SAMPLES"],
                2,
            ),
            9 => ("MASS_SERVICE", "STUDY", vec!["MASS_SPEC"], 2),
            10 => ("STUDY_SERVICE", "STUDY", vec!["STUDY"], 73),
            11 => (
                "STUDY_SERVICE",
                "STUDY",
                vec!["STUDY", "HPLC", "MASS_SPEC"],
                74,
            ),
            12..=14 => (
                "STUDY_SERVICE",
                "PROJECT_DIRECTOR",
                vec!["PROJECT_DIRECTOR"],
                72,
            ),
            15 => ("HPLC_SERVICE", "STUDY", vec!["HPLC"], 2),
            _ => return Err("unknown scientific request".into()),
        };
        let service = role(service)?;
        let cap = role(cap)?;
        let mut request_ids = Vec::new();
        let mut before = call.emitted;
        for resource in &requests {
            let source = role(resource)?;
            let label = if *resource == "BIOLOGICAL_SAMPLES" {
                8
            } else if s == 11 && *resource != "STUDY" {
                2
            } else {
                request_label
            };
            let (id, f) = take(&frames, &mut used, |f| {
                f.direction(source, service)
                    && f.logical_label == label
                    && f.received.is_some_and(|at| at < call.emitted)
            })
            .map_err(|error| {
                format!(
                    "effect {} step {s}, request {source}->{service} label {label}: {error}",
                    call.request["id"]
                )
            })?;
            same(&f.value()?, arguments)?;
            before = before.min(f.sent);
            request_ids.push(id);
        }
        if !matches!(s, 10..=14) {
            let (_, f) = take(&frames, &mut used, |f| {
                f.direction(cap, service)
                    && f.logical_label == 1
                    && f.received.is_some_and(|at| at < before)
            })?;
            same(&f.value()?, arguments)?;
            before = before.min(f.sent);
        }
        if matches!(s, 10 | 11) {
            let request_sent = before;
            for target in [role("HPLC_SERVICE")?, role("MASS_SERVICE")?] {
                // These notifications are sent before the review request.
                // Routine review does not wait for the independent equipment
                // owners to consume them. The actual effect requests above
                // still require a completed receive before execution.
                let (_, f) = take(&frames, &mut used, |f| {
                    f.direction(cap, target)
                        && f.logical_label == if s == 10 { 73 } else { 9 }
                        && f.sent < request_sent
                })?;
                same(&f.value()?, arguments)?;
                before = before.min(f.sent);
            }
        }
        close_arguments(
            &frames,
            &mut closes,
            vm,
            io,
            before,
            &format!("service/{}", call.tool()?),
            arguments,
        )?;
        let (from, to, label) = if matches!(s, 2 | 3) {
            (service, role("MASS_SPEC")?, 3)
        } else if matches!(s, 12..=14) {
            (role("STUDY")?, vm, 70)
        } else {
            (cap, vm, if result["success"] == true { 3 } else { 4 })
        };
        let (reply_id, reply) = take(&frames, &mut used, |f| {
            f.direction(from, to) && f.logical_label == label && f.sent > ended
        })?;
        let expected = if matches!(s, 12..=14) {
            outcome(&ordered, reply.sent)?
        } else {
            result.clone()
        };
        same(&reply.value()?, &expected)?;
        receipts.push(
            json!({"effect_id":call.request["id"],"source_requests":request_ids,
            "effect_at":call.emitted,"actual_result_at":ended,"reply_frame":reply_id,"step":s}),
        );
    }
    if !scored.is_empty() || closes.len() != calls.len() {
        return Err("missing scientific effect or admission".into());
    }
    let mut finals = BTreeSet::new();
    for (&id, f) in &frames {
        if used.contains(&id)
            || ios
                .iter()
                .any(|(a, b)| f.direction(*a, *b) || f.direction(*b, *a))
        {
            continue;
        }
        if matches!(f.logical_label, 5..=7) {
            continue;
        } // EPF certifies affine payload custody.
        let target = f.identity["target"].as_u64().ok_or("scientific target")?;
        let value = f.value()?;
        let expected = match f.logical_label {
            60 => prior(&ordered, f.sent, &[3])?.clone(),
            61 => prior(&ordered, f.sent, &[4])?.clone(),
            62 => prior(&ordered, f.sent, &[1])?.clone(),
            63 => prior(&ordered, f.sent, &[2])?.clone(),
            64 => prior(&ordered, f.sent, &[5])?.clone(),
            65 => prior(&ordered, f.sent, &[7])?.clone(),
            66 => prior(&ordered, f.sent, &[6])?.clone(),
            67 => prior(&ordered, f.sent, &[8])?.clone(),
            68 => prior(&ordered, f.sent, &[9])?.clone(),
            69 => prior(&ordered, f.sent, &[10, 11])?.clone(),
            70 if f.direction(role("STUDY_SERVICE")?, role("HPLC_SERVICE")?) => {
                prior(&ordered, f.sent, &[12, 13, 14])?.clone()
            }
            71 => prior(&ordered, f.sent, &[0])?.clone(),
            181 if f.identity["source"] == role("PROJECT_DIRECTOR")? => {
                if !finals.insert(target) {
                    return Err("repeated final scientific outcome".into());
                }
                let expected = outcome(&ordered, f.sent)?;
                if !matches!(expected["outcome"].as_str(), Some("continue" | "abort"))
                    || (expected["outcome"] == "continue"
                        && expected["regulatory"]["success"] != true)
                {
                    return Err("unfinished or unsupported final decision".into());
                }
                expected
            }
            _ => return Err(format!("unused scientific request/result frame {id}").into()),
        };
        same(&value, &expected)?;
    }
    let expected_finals = pairs
        .iter()
        .filter(|(a, _)| *a != "PROJECT_DIRECTOR")
        .map(|(a, _)| role(a))
        .chain(
            [
                "HPLC_SERVICE",
                "MASS_SERVICE",
                "CELL_SERVICE",
                "FORM_SERVICE",
                "STUDY_SERVICE",
            ]
            .into_iter()
            .map(role),
        )
        .collect::<Result<BTreeSet<_>>>()?;
    if finals != expected_finals {
        return Err("missing scientific final outcome delivery".into());
    }
    let mut overlaps = 0;
    let mut maximum = 0;
    for (i, a) in ordered.iter().enumerate() {
        let start = a.started()?.1;
        let mut active = 0;
        for b in &ordered {
            if b.started()?.1 <= start && start < b.ended()?.1 {
                active += 1;
            }
        }
        maximum = maximum.max(active);
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                let b_step = step(b)?;
                if instruments(step(a)?)
                    .iter()
                    .any(|r| instruments(b_step).contains(r))
                {
                    return Err("overlapping actual effects share a physical instrument".into());
                }
                overlaps += 1;
            }
        }
    }
    let mut report = json!({"status":"passed","task":"14H","simulator_complete":score["complete"],"frame_contracts":labels.len(),"domain_calls":calls.len(),"effect_closes":closes.len(),
        "actual_effect_receipts":receipts,"independent_effect_overlaps":overlaps,"max_concurrent_effects":maximum,
        "violations":0,"scope":"one-use actual WASI admissions, resource requests, real results, scientific lineage and final outcomes; EPF owns source eligibility"});
    report["dynamic_protocol"] = crate::projection::check_dynamic(artifact, rows)?;
    let deliveries = report["dynamic_protocol"]["sessions"]
        .as_array()
        .ok_or("checked sessions")?
        .iter()
        .map(|session| {
            session["acceptances"]
                .as_array()
                .ok_or("checked acceptances")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(|delivery| {
            let id = delivery["frame_id"].as_u64().ok_or("checked frame ID")?;
            Ok((id, delivery.clone()))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut received = BTreeSet::new();
    for receipt in report["actual_effect_receipts"]
        .as_array_mut()
        .ok_or("effect receipts")?
    {
        let at = receipt["effect_at"]
            .as_u64()
            .ok_or("effect journal position")?;
        let mut commits = Vec::new();
        for id in receipt["source_requests"]
            .as_array()
            .ok_or("effect source requests")?
        {
            let delivery = deliveries
                .get(&id.as_u64().ok_or("effect request frame")?)
                .ok_or("effect request has no checked acceptance")?;
            let commit = &delivery["receive_commit"];
            let producer = commit["producer"]
                .as_u64()
                .ok_or("effect request lacks its receive commit")?;
            let ordinal = commit["ordinal"].as_u64().ok_or("receive ordinal")?;
            if commit["journal_row"].as_u64().is_none_or(|row| row >= at)
                || commit["occurrence"] != delivery["occurrence"]
                || !received.insert((producer, ordinal))
            {
                return Err("effect precedes or reuses its checked receive occurrence".into());
            }
            commits.push(commit.clone());
        }
        receipt["receive_commits"] = json!(commits);
    }
    report["effect_receives_bound_to_checked_occurrences"] = json!(true);
    Ok(report)
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let baseline = check(&rows, &artifact)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("scientist fixtures")?;
    let journal = crate::journal::read(&rows, &labels, actors)?;
    let ios = [
        "CHEMIST_IO",
        "BIOLOGIST_IO",
        "TOXICOLOGIST_IO",
        "FORMULATOR_IO",
        "CLINICIAN_IO",
        "REGULATOR_IO",
        "DIRECTOR_IO",
    ]
    .map(|key| {
        artifact["compiled"]["roles"][key]
            .as_u64()
            .ok_or("scientist I/O role")
    })
    .into_iter()
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let mut ids = Vec::new();
    for (&id, f) in &journal.frames {
        let source = f.identity["source"].as_u64().ok_or("frame source")?;
        let target = f.identity["target"].as_u64().ok_or("frame target")?;
        if !ios.contains(&source) && !ios.contains(&target) && !matches!(f.logical_label, 5..=7) {
            ids.push(id);
        }
    }
    let mut verdicts =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    for receipt in baseline["actual_effect_receipts"]
        .as_array()
        .ok_or("effect receipts")?
    {
        let emitted = usize::try_from(receipt["effect_at"].as_u64().ok_or("effect position")?)?;
        let before = receipt["receive_commits"]
            .as_array()
            .ok_or("effect receive commits")?
            .iter()
            .map(|commit| commit["journal_row"].as_u64().ok_or("receive position"))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .ok_or("effect has no receive commit")?;
        let before = usize::try_from(before)?;
        if before >= emitted
            || rows[emitted]["direction"] != "runtime"
            || rows[emitted]["message"]["type"] != "execute"
        {
            return Err("invalid effect admission mutation boundary".into());
        }
        let mut changed = Vec::with_capacity(rows.len());
        let mut delayed = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            if index >= before
                && index < emitted
                && row["direction"] == "runtime"
                && row["message"]["type"] == "protocol_capture"
            {
                delayed.push(row.clone());
            } else {
                changed.push(row.clone());
            }
            if index == emitted {
                changed.append(&mut delayed);
            }
        }
        // Preserve every capture's producer order and every non-capture row's
        // relative order. Delay the receive evidence until after the effect;
        // neither source execution nor the transport/effect journal changes.
        crate::projection::check_dynamic(&artifact, &changed)?;
        crate::journal::read(&changed, &labels, actors)?;
        let error = check(&changed, &artifact)
            .err()
            .ok_or("accepted effect before receive commit")?;
        if !error
            .to_string()
            .contains("effect precedes or reuses its checked receive occurrence")
        {
            return Err(format!("effect/receive mutation hit another boundary: {error}").into());
        }
        verdicts.push(json!({"kind":"receive_commit_evidence_after_effect","effect_id":receipt["effect_id"],
            "original_effect_at":emitted,"delayed_capture_suffix_from":before,"rejected":true,"error":error.to_string()}));
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"verdicts":verdicts,
        "source_events_sha256":digest(path)?,"scope":"saved-record audit mutations, not runtime attacks","model_calls":0});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
