//! One-use scientific request/effect receipts. EPF alone checks source progress.
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

fn actual(calls: &[&Call], at: usize, query: (&str, &str, Option<(&str, &str)>)) -> Result<Value> {
    let (agent, tool, argument) = query;
    let call = calls
        .iter()
        .copied()
        .filter(|c| {
            c.request["agent"] == agent
                && c.request["tool"] == tool
                && argument.is_none_or(|(key, value)| c.request["arguments"][key] == value)
                && c.result.as_ref().is_some_and(|(end, _)| *end < at)
        })
        .max_by_key(|c| c.emitted)
        .ok_or("missing prior scientific effect")?;
    let result = call.result()?.1;
    if result["success"] != true {
        return Err("a failed scientific result cannot supply an artifact".into());
    }
    Ok(result.clone())
}
fn compound(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(
        json!({"qc":actual(calls,at,("CHEMIST","run_hplc_analysis",Some(("sample","synthesis_qc"))))?,
        "identity":actual(calls,at,("CHEMIST","run_mass_spec_analysis",Some(("sample","molecular_id"))))?}),
    )
}
fn biology(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"compound":compound(calls,at)?,
        "preparation":actual(calls,at,("BIOLOGIST","run_cell_assay",Some(("assay_type","culture_prep"))))?,
        "assay":actual(calls,at,("BIOLOGIST","run_cell_assay",Some(("assay_type","bioassay"))))?,
        "stability":actual(calls,at,("BIOLOGIST","run_hplc_analysis",Some(("sample","stability"))))?}))
}
fn toxicology(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"compound":compound(calls,at)?,
        "baseline":actual(calls,at,("TOXICOLOGIST","run_mass_spec_analysis",Some(("sample","baseline_markers"))))?,
        "purity":actual(calls,at,("TOXICOLOGIST","run_hplc_analysis",Some(("sample","purity_check"))))?,
        "cytotoxicity":actual(calls,at,("TOXICOLOGIST","run_cell_assay",Some(("assay_type","cytotoxicity"))))?}))
}
fn candidate(calls: &[&Call], at: usize, go: &Value) -> Result<Value> {
    if !go.is_boolean() {
        return Err("decision must be a boolean".into());
    }
    Ok(
        json!({"go":go,"compound":compound(calls,at)?,"bio":biology(calls,at)?,"tox":toxicology(calls,at)?,
        "analyses":{"reference":actual(calls,at,("LEAD_SCIENTIST","run_mass_spec_analysis",Some(("sample","reference_methods"))))?,
        "structural":actual(calls,at,("LEAD_SCIENTIST","run_hplc_analysis",Some(("sample","structural_confirmation"))))?,
        "characterization":actual(calls,at,("LEAD_SCIENTIST","run_mass_spec_analysis",Some(("sample","characterization"))))?}}),
    )
}
fn submission(calls: &[&Call], at: usize) -> Result<Value> {
    let decision = calls
        .iter()
        .copied()
        .filter(|c| {
            c.tool().is_ok_and(|t| t == "submit_decision")
                && c.result.as_ref().is_some_and(|(end, _)| *end < at)
        })
        .max_by_key(|c| c.emitted)
        .ok_or("missing actual decision submission")?;
    Ok(
        json!({"decision":candidate(calls,decision.emitted,&decision.request["arguments"]["go"])?,
        "receipt":decision.result()?.1}),
    )
}
fn take<'a>(
    frames: &'a BTreeMap<u64, Frame>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<&'a Frame> {
    let (&id, frame) = frames
        .iter()
        .filter(|(id, f)| !used.contains(id) && matches(f))
        .min_by_key(|(_, f)| f.sent)
        .ok_or("missing one-use scientific frame")?;
    used.insert(id);
    Ok(frame)
}
fn same(a: &Value, b: &Value) -> Result<()> {
    if a != b {
        return Err("scientific value differs from admitted or actual data".into());
    }
    Ok(())
}
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty scientific evidence")?["message"];
    if compiled["task"] != "14M" || fixture["task"] != "14M" || fixture["type"] != "fixtures" {
        return Err("regulatory study source or fixture differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("scientific participants")?;
    if actors.len() != 5
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("CHEMIST"),
                Some("BIOLOGIST"),
                Some("TOXICOLOGIST"),
                Some("LEAD_SCIENTIST"),
                Some("REGULATORY_SPECIALIST"),
            ])
    {
        return Err("scientific participant population differs".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() < 14
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe regulatory study".into());
    }
    let pairs = [
        ("CHEMIST", "CHEMIST_IO"),
        ("BIOLOGIST", "BIOLOGIST_IO"),
        ("TOXICOLOGIST", "TOXICOLOGIST_IO"),
        ("LEAD_SCIENTIST", "LEAD_IO"),
        ("REGULATORY_SPECIALIST", "REGULATOR_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut results = Vec::new();
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("scientist")?;
        let vm = role(agent)?;
        let io = pairs
            .iter()
            .find(|(v, _)| *v == vm)
            .ok_or("scientific I/O")?
            .1;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let documentation = tool == "run_hplc_analysis" && arguments["sample"] == "documentation";
        // These are the application's message data contracts, not a second
        // workflow: sealed wire rows and EPF decide their static/dynamic use.
        let (
            resource,
            service,
            request_label,
            intent_source,
            intent_label,
            reply_source,
            reply_label,
        ) = match tool {
            "run_hplc_analysis" if !documentation => {
                let cap = match (agent, arguments["sample"].as_str()) {
                    ("CHEMIST", Some("synthesis_qc")) => "QC",
                    ("BIOLOGIST", Some("stability")) => "STABILITY",
                    ("TOXICOLOGIST", Some("purity_check")) => "PURITY",
                    ("LEAD_SCIENTIST", Some("structural_confirmation")) => "STRUCTURAL",
                    _ => return Err("unassigned HPLC effect".into()),
                };
                (
                    role("HPLC")?,
                    role("HPLC_SERVICE")?,
                    2,
                    role(cap)?,
                    1,
                    role(cap)?,
                    if call.result()?.1["success"] == true {
                        3
                    } else {
                        4
                    },
                )
            }
            "run_cell_assay" => {
                let cap = match (agent, arguments["assay_type"].as_str()) {
                    ("BIOLOGIST", Some("culture_prep")) => "CULTURE",
                    ("BIOLOGIST", Some("bioassay")) => "BIOASSAY",
                    ("TOXICOLOGIST", Some("cytotoxicity")) => "TOXICITY",
                    _ => return Err("unassigned cell assay".into()),
                };
                (
                    role("CELL_LAB")?,
                    role("CELL_SERVICE")?,
                    2,
                    role(cap)?,
                    1,
                    role(cap)?,
                    if call.result()?.1["success"] == true {
                        3
                    } else {
                        4
                    },
                )
            }
            "run_mass_spec_analysis" => {
                if !matches!(
                    (agent, arguments["sample"].as_str()),
                    ("CHEMIST", Some("molecular_id"))
                        | ("TOXICOLOGIST", Some("baseline_markers"))
                        | (
                            "LEAD_SCIENTIST",
                            Some("reference_methods" | "characterization")
                        )
                ) {
                    return Err("unassigned mass analysis".into());
                }
                (
                    role("MASS_SPEC")?,
                    role("MASS_SERVICE")?,
                    11,
                    vm,
                    10,
                    role("MASS_SERVICE")?,
                    12,
                )
            }
            "submit_decision" if agent == "LEAD_SCIENTIST" => (
                role("STUDY")?,
                role("HPLC_SERVICE")?,
                23,
                role("STUDY")?,
                23,
                role("HPLC_SERVICE")?,
                24,
            ),
            "verify_regulatory_compliance" if agent == "REGULATORY_SPECIALIST" => (
                role("STUDY")?,
                role("REVIEW_SERVICE")?,
                25,
                role("STUDY")?,
                25,
                role("STUDY")?,
                0,
            ),
            "run_hplc_analysis" if documentation && agent == "CHEMIST" => (
                role("HPLC")?,
                role("HPLC_SERVICE")?,
                29,
                role("STUDY")?,
                29,
                role("STUDY")?,
                if call.result()?.1["success"] == true {
                    3
                } else {
                    4
                },
            ),
            _ => return Err("unexpected scientific effect".into()),
        };
        let request = take(&frames, &mut used, |f| {
            f.direction(resource, service)
                && f.logical_label == request_label
                && f.received.is_some_and(|at| at < call.emitted)
        })?;
        let value = request.value()?;
        let expected = if tool == "submit_decision" {
            candidate(&ordered, call.emitted, &arguments["go"])?
        } else if tool == "verify_regulatory_compliance" || documentation {
            let mut expected = submission(&ordered, call.emitted)?;
            expected["arguments"] = arguments.clone();
            if documentation {
                expected["success"] = json!(true);
                expected["approval"] = actual(
                    &ordered,
                    call.emitted,
                    (
                        "REGULATORY_SPECIALIST",
                        "verify_regulatory_compliance",
                        None,
                    ),
                )?;
            }
            expected
        } else {
            arguments.clone()
        };
        same(&value, &expected)?;
        let intent = if request_label == 23 || request_label == 25 {
            request
        } else {
            let f = take(&frames, &mut used, |f| {
                f.direction(intent_source, service)
                    && f.logical_label == intent_label
                    && f.received.is_some_and(|at| at < request.sent)
            })?;
            same(&f.value()?, &value)?;
            f
        };
        close_arguments(
            &frames,
            &mut closes,
            vm,
            io,
            intent.sent,
            &format!("service/{tool}"),
            arguments,
        )?;
        let (result_at, result) = call.result()?;
        if result["data"]["scientist"] != agent || !result["success"].is_boolean() {
            return Err("scientific result belongs to another participant".into());
        }
        if matches!(
            tool,
            "run_hplc_analysis" | "run_cell_assay" | "run_mass_spec_analysis"
        ) {
            let grant = take(&frames, &mut used, |f| {
                f.direction(service, vm)
                    && f.logical_label == 5
                    && f.sent > intent.sent
                    && f.received.is_some_and(|at| at < request.sent)
            })?;
            let returned = take(&frames, &mut used, |f| {
                f.direction(vm, service) && f.logical_label == 5 && f.sent > request.sent
            })?;
            if grant.received()? >= returned.sent
                || (!matches!(tool, "run_mass_spec_analysis")
                    && returned.received()? >= call.emitted)
            {
                return Err("instrument receipt does not bracket its effect".into());
            }
            if tool == "run_mass_spec_analysis" && returned.sent <= result_at {
                return Err("mass instrument returned before actual response".into());
            }
        }
        if tool == "verify_regulatory_compliance" {
            let regulator = role("REGULATORY_SPECIALIST")?;
            let forwarded = take(&frames, &mut used, |f| {
                f.direction(resource, regulator)
                    && f.logical_label == 23
                    && f.received.is_some_and(|at| at < request.sent)
            })?;
            same(&forwarded.value()?, &submission(&ordered, request.sent)?)?;
            let approved = result["success"] == true;
            let deliveries = if approved {
                vec![
                    ("REGULATORY_SPECIALIST", 26),
                    ("LEAD_SCIENTIST", 40),
                    ("BIOLOGIST", 41),
                    ("TOXICOLOGIST", 42),
                    ("HPLC_SERVICE", 43),
                    ("CELL_SERVICE", 44),
                ]
            } else {
                vec![("REGULATORY_SPECIALIST", 27), ("LEAD_SCIENTIST", 27)]
            };
            for (dest, label) in deliveries {
                let to = role(dest)?;
                let f = take(&frames, &mut used, |f| {
                    f.direction(resource, to) && f.logical_label == label && f.sent > result_at
                })?;
                same(&f.value()?, result)?;
            }
            if approved {
                let to = role("CHEMIST")?;
                let f = take(&frames, &mut used, |f| {
                    f.direction(resource, to) && f.logical_label == 45 && f.sent > result_at
                })?;
                same(
                    &f.value()?,
                    &json!({"success":true,"approval":result,"decision":value["decision"],"receipt":value["receipt"]}),
                )?;
            }
            same(arguments, &json!({"decision":"go_no_go"}))?;
            same(
                &result["data"]["inputs"]["decision"],
                &value["receipt"]["data"]["artifact_id"],
            )?;
            same(
                &result["data"]["inputs"]["compound"],
                &value["decision"]["compound"]["identity"]["data"]["artifact_id"],
            )?;
        } else {
            let reply_target = if tool == "run_mass_spec_analysis" {
                resource
            } else if tool == "submit_decision" {
                role("STUDY")?
            } else {
                vm
            };
            let f = take(&frames, &mut used, |f| {
                f.direction(reply_source, reply_target)
                    && f.logical_label == reply_label
                    && f.sent > result_at
            })?;
            same(&f.value()?, result)?;
        }
        if tool == "submit_decision" {
            for (scientist, label, report) in [
                ("BIOLOGIST", 21, &value["bio"]),
                ("TOXICOLOGIST", 22, &value["tox"]),
            ] {
                let from = role(scientist)?;
                let lead = role("LEAD_SCIENTIST")?;
                let delivered = take(&frames, &mut used, |f| {
                    f.direction(from, lead)
                        && f.logical_label == label
                        && f.received.is_some_and(|at| at < request.sent)
                })?;
                same(&delivered.value()?, report)?;
            }
            same(arguments, &json!({"go":value["go"]}))?;
            same(&result["data"]["go"], &value["go"])?;
            for (key, receipt) in [
                ("compound", &value["compound"]["identity"]),
                ("bioassay", &value["bio"]["assay"]),
                ("stability", &value["bio"]["stability"]),
                ("toxicity", &value["tox"]["cytotoxicity"]),
                ("structural", &value["analyses"]["structural"]),
                ("characterization", &value["analyses"]["characterization"]),
            ] {
                same(
                    &result["data"]["inputs"][key],
                    &receipt["data"]["artifact_id"],
                )?;
            }
        }
        if documentation {
            same(
                &result["data"]["inputs"]["decision"],
                &value["receipt"]["data"]["artifact_id"],
            )?;
            same(
                &result["data"]["inputs"]["regulatory"],
                &value["approval"]["data"]["artifact_id"],
            )?;
        }
        results.push(json!({"effect_id":call.request["id"],"request_at":request.received()?,"effect_at":call.emitted,"actual_result_at":result_at}));
    }
    let mut plans = BTreeMap::<usize, BTreeSet<u64>>::new();
    for (&id, f) in &frames {
        if used.contains(&id)
            || pairs
                .iter()
                .any(|(vm, io)| f.direction(*vm, *io) || f.direction(*io, *vm))
        {
            continue;
        }
        if f.logical_label == 6 {
            continue;
        } // Affine payload/custody is checked by EPF, not decoded here.
        let from = f.identity["source"].as_u64().ok_or("frame source")?;
        let to = f.identity["target"].as_u64().ok_or("frame target")?;
        let expected = match f.logical_label {
            20 if f.direction(role("CHEMIST")?, role("BIOLOGIST")?) => compound(&ordered, f.sent)?,
            31 if f.direction(role("CHEMIST")?, role("TOXICOLOGIST")?) => {
                compound(&ordered, f.sent)?
            }
            0..=3 if from == role("STUDY")? => {
                let value = f.value()?;
                let mask = u8::from(value["biology"] == true)
                    | (u8::from(value["toxicology"] == true) << 1);
                if value.as_object().is_none_or(|o| o.len() != 2)
                    || !value["biology"].is_boolean()
                    || !value["toxicology"].is_boolean()
                    || mask != f.logical_label
                {
                    return Err("reanalysis does not match its data contract".into());
                }
                let mut closed = BTreeSet::new();
                close_arguments(
                    &frames,
                    &mut closed,
                    role("LEAD_SCIENTIST")?,
                    role("LEAD_IO")?,
                    f.sent,
                    "service/reconsider",
                    &value,
                )?;
                let at = *closed.first().ok_or("plan admission")?;
                if !plans.entry(at).or_default().insert(to) {
                    return Err("duplicate plan delivery".into());
                }
                value
            }
            _ => {
                return Err(format!(
                    "unused or unrecognized scientific frame {id}: {from}->{to} label {}",
                    f.logical_label
                )
                .into());
            }
        };
        same(&f.value()?, &expected)?;
        used.insert(id);
    }
    let recipients = ["BIOLOGIST", "TOXICOLOGIST", "HPLC_SERVICE", "CELL_SERVICE"]
        .into_iter()
        .map(role)
        .collect::<Result<BTreeSet<_>>>()?;
    if plans
        .values()
        .any(|destinations| *destinations != recipients)
    {
        return Err("missing or extra plan delivery".into());
    }
    let rejected = ordered
        .iter()
        .filter(|c| {
            c.request["tool"] == "verify_regulatory_compliance"
                && c.result
                    .as_ref()
                    .is_some_and(|(_, v)| v["success"] == false)
        })
        .count();
    if plans.len() != rejected {
        return Err("missing or extra reanalysis plan occurrence".into());
    }
    for (to, label) in [(role("BIOLOGIST")?, 20), (role("TOXICOLOGIST")?, 31)] {
        let from = role("CHEMIST")?;
        if frames
            .values()
            .filter(|f| f.direction(from, to) && f.logical_label == label)
            .count()
            != 1
        {
            return Err("missing or extra compound delivery".into());
        }
    }
    let mut overlaps = 0;
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                if a.tool()? == b.tool()?
                    && matches!(
                        a.tool()?,
                        "run_hplc_analysis" | "run_cell_assay" | "run_mass_spec_analysis"
                    )
                {
                    return Err("overlapping exclusive instrument effects".into());
                }
                overlaps += 1;
            }
        }
    }
    let mut maximum = 0;
    for call in &ordered {
        let at = call.started()?.1;
        let mut active = 0;
        for other in &ordered {
            if other.started()?.1 <= at && at < other.ended()?.1 {
                active += 1;
            }
        }
        maximum = maximum.max(active);
    }
    Ok(
        json!({"status":"passed","task":"14M","simulator_complete":score["complete"],"domain_calls":calls.len(),"actual_effect_receipts":results,
        "effect_closes":closes.len(),"effect_close_ordinals":closes,"reanalysis_plans":plans.len(),"independent_effect_overlaps":overlaps,"max_concurrent_effects":maximum,"violations":0,
        "scope":"one-use actual WASI admissions, instrument request/return, scientific data, regulatory verdict and documentation; EPF checks dynamic source execution"}),
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
            .ok_or("scientific evidence directory")?
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
        .ok_or("scientists")?;
    let journal = crate::journal::read(&rows, &labels, actors)?;
    let ios = [
        "CHEMIST_IO",
        "BIOLOGIST_IO",
        "TOXICOLOGIST_IO",
        "LEAD_IO",
        "REGULATOR_IO",
    ]
    .into_iter()
    .map(|n| artifact["compiled"]["roles"][n].as_u64().ok_or("I/O role"))
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let mut ids = Vec::new();
    for (id, frame) in &journal.frames {
        let source = frame.identity["source"].as_u64().ok_or("frame source")?;
        let target = frame.identity["target"].as_u64().ok_or("frame target")?;
        if !ios.contains(&source)
            && !ios.contains(&target)
            && frame.logical_label != 5
            && frame.logical_label != 6
        {
            ids.push(*id);
        }
    }
    let mut verdicts =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let absent_label = (0..=u16::MAX)
        .find(|label| !contracts.contains_key(label))
        .ok_or("no unused wire label")?;
    for id in &ids {
        for kind in ["coherent_wrong_wire_label", "coherent_wrong_lane"] {
            let mut changed = rows.clone();
            for row in &mut changed {
                let message = &mut row["message"];
                if message["type"] == "protocol_frame" && message["frame_id"] == *id {
                    if kind == "coherent_wrong_wire_label" {
                        message["frame_label"] = json!(absent_label);
                    } else {
                        message["lane"] =
                            json!((message["lane"].as_u64().ok_or("wire lane")? + 1) % 256);
                    }
                }
            }
            let error = check(&changed, &artifact)
                .err()
                .ok_or("accepted altered scientific wire header")?;
            verdicts
                .push(json!({"frame_id":id,"kind":kind,"rejected":true,"error":error.to_string()}));
        }
    }
    for ordinal in baseline["effect_close_ordinals"]
        .as_array()
        .ok_or("effect close inventory")?
    {
        let at = ordinal
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("close ordinal")?;
        let (&close_id, closed) = journal
            .frames
            .iter()
            .find(|(_, f)| f.sent == at)
            .ok_or("actual close frame")?;
        let vm = closed.identity["target"].as_u64().ok_or("close target")?;
        let io = closed.identity["source"].as_u64().ok_or("close source")?;
        let (&request_id, _) = journal
            .frames
            .iter()
            .filter(|(_, f)| {
                f.direction(vm, io)
                    && f.logical_label == hibana_wasip1_runtime::protocol::LABEL_WASI_FD_CLOSE
                    && f.sent < closed.sent
            })
            .max_by_key(|(_, f)| f.sent)
            .ok_or("actual close request")?;
        let changed = rows
            .iter()
            .filter(|row| {
                !(row["message"]["type"] == "protocol_frame"
                    && (row["message"]["frame_id"] == close_id
                        || row["message"]["frame_id"] == request_id))
            })
            .cloned()
            .collect::<Vec<_>>();
        let error = check(&changed, &artifact)
            .err()
            .ok_or("accepted missing repeated scientific close pair")?;
        verdicts.push(json!({"frame_id":close_id,"kind":"missing_effect_close_pair","rejected":true,"error":error.to_string()}));
    }
    for &id in journal.calls.keys() {
        let mut changed = rows.clone();
        for row in &mut changed {
            if matches!(row["direction"].as_str(), Some("domain_end" | "simulator"))
                && row["message"]["id"] == id
                && row["message"].get("result").is_some()
            {
                row["message"]["result"]["unadmitted_value"] = json!(true);
            }
        }
        let error = check(&changed, &artifact)
            .err()
            .ok_or("accepted substituted coherent domain result")?;
        verdicts.push(json!({"effect_id":id,"kind":"coherent_domain_result_substitution","rejected":true,"error":error.to_string()}));
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"verdicts":verdicts,
        "source_events_sha256":digest(path)?,"scope":"saved-record effect audit mutations, not runtime attacks","model_calls":0});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
