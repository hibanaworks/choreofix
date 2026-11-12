//! EPF source conformance and independent section-effect/result correspondence.
use crate::evidence::Result;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
fn bytes(message: &Value) -> Result<Vec<u8>> {
    message["payload"]
        .as_array()
        .ok_or("frame bytes")?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "frame byte".into())
        })
        .collect()
}
fn pair(value: &Value) -> Option<Vec<&str>> {
    let mut pair: Vec<_> = value["sections"]
        .as_str()?
        .split(',')
        .map(str::trim)
        .collect();
    pair.sort();
    Some(pair)
}
pub fn audit(path: &Path) -> Result<Value> {
    let records: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let projection = crate::projection::load(path)?;
    let mut result = check(&records, &projection)?;
    result["source_events_sha256"] = json!(crate::evidence::digest(path)?);
    result["compiled_projection_sha256"] = json!(crate::evidence::digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json"),
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}

fn check(records: &[Value], projection: &Value) -> Result<Value> {
    let labels = crate::projection::frames(projection, records)?;
    let dynamic = crate::projection::check_dynamic(projection, records)?;
    if projection["compiled"]["task"] != records.first().ok_or("empty evidence")?["message"]["task"]
    {
        return Err("compiled projection belongs to a different task".into());
    }
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?;
    let names: &[&str] = match fixture["task"].as_str() {
        Some("2E") => &["INTRO_SECTION", "METHODS_SECTION", "RESULTS_SECTION"],
        Some("2M") => &[
            "BACKGROUND_SECTION",
            "INTRO_SECTION",
            "METHODS_SECTION",
            "EXPERIMENTS_SECTION",
            "RESULTS_SECTION",
        ],
        Some("2H") => &[
            "BACKGROUND_SECTION",
            "INTRO_SECTION",
            "RELATED_SECTION",
            "METHODS_SECTION",
            "EXPERIMENTS_SECTION",
            "RESULTS_SECTION",
            "DISCUSSION_SECTION",
        ],
        _ => return Err("unknown writing task".into()),
    };
    if actors.len() != names.len() {
        return Err("participant/section count".into());
    }
    let terminal = fixture["terminal_review_failure"]
        .as_bool()
        .ok_or("terminal review setting")?;
    let assignments: BTreeMap<_, _> = actors
        .iter()
        .enumerate()
        .map(|(i, a)| Ok((a["agent"].as_str().ok_or("participant")?, (i as u64, a))))
        .collect::<Result<_>>()?;
    let sections: BTreeMap<_, _> = names
        .iter()
        .enumerate()
        .map(|(i, &n)| (n, (7 + i as u64, 28 + i as u64)))
        .collect();
    let mut closes = BTreeSet::new();
    let mut admissions = BTreeMap::<u64, Value>::new();
    let mut requests = BTreeMap::<u64, Vec<(u64, Value)>>::new();
    let mut loan_counts = BTreeMap::<u64, usize>::new();
    let mut return_counts = BTreeMap::<u64, usize>::new();
    let mut calls = BTreeMap::<&str, usize>::new();
    let mut outstanding = BTreeMap::<u64, Value>::new();
    let mut responses = BTreeMap::<(u64, String), Value>::new();
    let mut delivered = BTreeMap::<(u64, String), Vec<Value>>::new();
    let mut active = BTreeMap::<u64, BTreeSet<String>>::new();
    let mut maximum_writes = 0;
    let journal = crate::journal::read(records, &labels, actors)?;
    let receipts = journal.receipts(&dynamic)?;
    let mut frame_rows = BTreeMap::<usize, Vec<(u32, &crate::journal::Frame, bool)>>::new();
    for (&id, frame) in &journal.frames {
        frame_rows
            .entry(frame.sent)
            .or_default()
            .push((0, frame, false));
        let committed = usize::try_from(
            receipts[&id]["receive_commit"]["journal_row"]
                .as_u64()
                .ok_or("receive commit observation")?,
        )?;
        let ordinal = u32::try_from(
            receipts[&id]["receive_commit"]["ordinal"]
                .as_u64()
                .ok_or("receive commit ordinal")?,
        )?;
        frame_rows
            .entry(committed)
            .or_default()
            .push((ordinal, frame, true));
    }
    // One capture row can contain several commits. Preserve producer order,
    // rather than ordering them by the transport's unrelated frame IDs.
    for observed in frame_rows.values_mut() {
        observed.sort_by_key(|&(ordinal, _, _)| ordinal);
    }
    for (index, row) in records.iter().enumerate() {
        let m = &row["message"];
        if let Some(observed) = frame_rows.get(&index) {
            for &(_, frame, received) in observed {
                if frame.identity["session"] != 18 {
                    return Err("wrong session".into());
                }
                let from = frame.identity["source"].as_u64().ok_or("source")?;
                let to = frame.identity["target"].as_u64().ok_or("target")?;
                let data = &frame.bytes;
                let owner = sections
                    .values()
                    .find(|&&(owner, _)| from == owner || to == owner)
                    .copied();
                if let Some((owner, resource)) = owner {
                    if data.len() != 3 || u64::from(data[0]) != resource {
                        return Err("invalid resource envelope".into());
                    }
                    let vm = if from == owner { to } else { from };
                    let actor = actors
                        .get(vm.checked_sub(14).ok_or("VM role")? as usize)
                        .ok_or("VM role")?;
                    let assigned = actor["sections"].as_array().ok_or("assigned sections")?;
                    if !assigned.iter().any(|n| {
                        sections
                            .get(n.as_str().unwrap_or(""))
                            .is_some_and(|&(_, r)| r == resource)
                    }) {
                        return Err("loan to unassigned writer".into());
                    }
                    if !received {
                        if to == owner {
                            *return_counts.entry(resource).or_default() += 1;
                        } else {
                            *loan_counts.entry(resource).or_default() += 1;
                        }
                    }
                }
                if received && to < actors.len() as u64 && from == to + 14 && data.as_slice() == [4]
                {
                    closes.insert(to);
                }
                if received
                    && from < actors.len() as u64
                    && to == from + 14
                    && let Ok(value) = serde_json::from_slice::<Value>(data)
                {
                    if let Some(tool) = value["tool_name"].as_str() {
                        delivered
                            .entry((from, tool.into()))
                            .or_default()
                            .push(value);
                    } else if value["sections"].is_string()
                        && admissions.insert(from, value).is_some()
                    {
                        return Err("duplicate admitted write".into());
                    }
                }
                if received
                    && to < actors.len() as u64
                    && sections.values().any(|&(_, resource)| resource == from)
                {
                    let arguments: Value = serde_json::from_slice(data)?;
                    if admissions.get(&to) != Some(&arguments) {
                        return Err("section request differs from actual admitted arguments".into());
                    }
                    requests.entry(to).or_default().push((from, arguments));
                }
                if received
                    && from < actors.len() as u64
                    && sections.values().any(|&(_, resource)| resource == to)
                {
                    let result: Value = serde_json::from_slice(data)?;
                    if result["tool_name"] != "write_to_document" {
                        return Err("unexpected section result tool".into());
                    }
                    delivered
                        .entry((from, "write_to_document".into()))
                        .or_default()
                        .push(result);
                }
            }
        }
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("execute")) => {
                let agent = m["agent"].as_str().ok_or("effect agent")?;
                let &(role, actor) = assignments.get(agent).ok_or("unknown agent")?;
                let count = calls.entry(agent).or_default();
                let expected = actor["calls"]
                    .get(*count)
                    .ok_or("effect after completion")?;
                let tool = m["tool"].as_str().ok_or("effect tool")?;
                let matches = if tool == "write_to_document" {
                    pair(&expected["arguments"]).is_some()
                        && pair(&expected["arguments"]) == pair(&m["arguments"])
                } else {
                    expected["arguments"] == m["arguments"]
                };
                if expected["tool"] != tool || !matches || !closes.remove(&role) {
                    return Err(
                        "effect differs from original assignment or has no WASI close".into(),
                    );
                }
                for prior in actor["calls"]
                    .as_array()
                    .ok_or("calls")?
                    .iter()
                    .take(*count)
                {
                    let prior = prior["tool"].as_str().ok_or("prior tool")?;
                    if !delivered.contains_key(&(role, prior.into())) {
                        return Err("effect before previous result".into());
                    }
                }
                if tool == "write_to_document" {
                    let required = actor["sections"]
                        .as_array()
                        .ok_or("sections")?
                        .iter()
                        .map(|n| {
                            sections
                                .get(n.as_str().ok_or("section")?)
                                .map(|&(_, r)| r)
                                .ok_or_else(|| "unknown section".into())
                        })
                        .collect::<Result<BTreeSet<_>>>()?;
                    let seen = requests
                        .remove(&role)
                        .ok_or("write without resource requests")?;
                    if seen.len() != 2
                        || seen.iter().map(|&(r, _)| r).collect::<BTreeSet<_>>() != required
                        || seen.iter().any(|(_, args)| args != &m["arguments"])
                    {
                        return Err(
                            "write lacks both actual section requests with matching arguments"
                                .into(),
                        );
                    }
                    let names = actor["sections"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|n| n.as_str().unwrap().to_owned())
                        .collect::<BTreeSet<_>>();
                    if active.values().any(|a| !a.is_disjoint(&names)) {
                        return Err("overlapping physical sections".into());
                    }
                    active.insert(m["id"].as_u64().ok_or("effect ID")?, names);
                    maximum_writes = maximum_writes.max(active.len());
                }
                *count += 1;
                if outstanding
                    .insert(m["id"].as_u64().ok_or("effect ID")?, m.clone())
                    .is_some()
                {
                    return Err("duplicate effect ID".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = m["id"].as_u64().ok_or("result ID")?;
                let call = outstanding.remove(&id).ok_or("unmatched result")?;
                let &(role, _) = assignments.get(call["agent"].as_str().unwrap()).unwrap();
                let tool = call["tool"].as_str().unwrap();
                if m["result"]["tool_name"] != tool
                    || m["result"]["success"] != !(terminal && tool == "review_own_work")
                    || responses
                        .insert((role, tool.into()), m["result"].clone())
                        .is_some()
                {
                    return Err("wrong original result".into());
                }
                active.remove(&id);
            }
            _ => {}
        }
    }
    let score = &journal.score;
    for (&agent, &(role, actor)) in &assignments {
        if calls.get(agent) != Some(&4) || actor["calls"].as_array().ok_or("calls")?.len() != 4 {
            return Err("incomplete original work".into());
        }
        for call in actor["calls"].as_array().unwrap() {
            let tool = call["tool"].as_str().unwrap();
            let result = responses
                .get(&(role, tool.into()))
                .ok_or("missing result")?;
            let values = delivered
                .remove(&(role, tool.into()))
                .ok_or("missing delivered result")?;
            if values.len() != if tool == "write_to_document" { 2 } else { 1 }
                || values.iter().any(|v| v != result)
            {
                return Err("delivered result differs from original".into());
            }
            let events = score["events"]
                .as_array()
                .ok_or("events")?
                .iter()
                .filter(|e| e["agent"] == agent && e["tool"] == tool)
                .collect::<Vec<_>>();
            if events.len() != 1 {
                return Err("original event count".into());
            }
            let event = events[0];
            let effect_args = if tool == "write_to_document" {
                json!({"sections":call["arguments"]["sections"].as_str().ok_or("sections")?.split(',').map(str::trim).collect::<Vec<_>>()})
            } else if matches!(tool, "draft_content" | "review_own_work") {
                json!({"section":call["arguments"]["section_name"]})
            } else {
                call["arguments"].clone()
            };
            let args_match = if tool == "write_to_document" {
                let mut actual = event["args"]["sections"]
                    .as_array()
                    .ok_or("written sections")?
                    .iter()
                    .map(|v| v.as_str().ok_or("section name"))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                actual.sort();
                Some(actual) == pair(&call["arguments"])
            } else {
                event["args"] == effect_args
            };
            if !args_match
                || event["result"] != result["data"]
                || event["success"] != result["success"]
            {
                return Err("original event/result mismatch".into());
            }
        }
    }
    if !outstanding.is_empty()
        || !requests.is_empty()
        || !delivered.is_empty()
        || !active.is_empty()
        || sections
            .values()
            .any(|&(_, r)| loan_counts.get(&r) != Some(&2) || return_counts.get(&r) != Some(&2))
        || score["complete"] != !terminal
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("events")?.len() != 4 * actors.len()
        || maximum_writes == 0
        || maximum_writes > actors.len() / 2
    {
        return Err("incomplete section-delegation evidence".into());
    }
    Ok(
        json!({"task":fixture["task"],"depth":fixture["depth"],"kind":"actual_wasi_section_delegation","status":"passed","scored_benchmark":false,"model_calls":null,
        "participants":actors.len(),"domain_calls":4*actors.len(),"actual_resource_endpoint_requests":2*actors.len(),"actual_resource_endpoint_returns":2*actors.len(),
        "carrier_frames":journal.frames.len(),"max_disjoint_writes":maximum_writes,"simulator_complete":score["complete"],"violations":0,"guest_exits":actors.len(),"expected_terminal_review_failure":terminal,"dynamic_protocol":dynamic}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    let projection = crate::projection::load(path)?;
    fs::create_dir(output)?;
    let rows: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let mut results = Vec::new();
    for kind in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_loan",
        "wrong_loan_resource",
        "missing_request",
        "forged_request",
        "missing_reply",
        "forged_reply",
        "missing_return",
        "overlapping_loans",
        "missing_admission",
        "missing_close",
        "forged_arguments",
        "forged_original_result",
        "missing_exit",
        "false_score",
    ] {
        let mut copy = rows.clone();
        if matches!(kind, "wrong_frame_label" | "wrong_lane") {
            let frame = copy
                .iter()
                .find(|row| row["message"]["type"] == "protocol_frame")
                .ok_or("frame for header mutation")?["message"]
                .clone();
            let field = if kind == "wrong_frame_label" {
                "frame_label"
            } else {
                "lane"
            };
            let replacement = if kind == "wrong_frame_label" {
                65535
            } else {
                (frame["lane"].as_u64().ok_or("observed lane")? + 1) % 256
            };
            for row in &mut copy {
                if row["message"]["type"] == "protocol_frame"
                    && row["message"]["frame_id"] == frame["frame_id"]
                {
                    row["message"][field] = json!(replacement);
                }
            }
        } else {
            match kind {
                "missing_loan"
                | "wrong_loan_resource"
                | "missing_request"
                | "forged_request"
                | "missing_reply"
                | "forged_reply"
                | "missing_return"
                | "missing_admission" => {
                    let (from, to) = match kind {
                        "missing_loan" | "wrong_loan_resource" => (7, 14),
                        "missing_request" | "forged_request" => (28, 0),
                        "missing_reply" | "forged_reply" => (0, 28),
                        "missing_return" => (14, 7),
                        "missing_admission" => (0, 14),
                        _ => unreachable!(),
                    };
                    let frame = copy
                        .iter()
                        .find(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["source"] == from
                                && r["message"]["target"] == to
                                && (kind != "missing_admission"
                                    || serde_json::from_slice::<Value>(
                                        &bytes(&r["message"]).unwrap(),
                                    )
                                    .is_ok_and(|v| v["sections"].is_string()))
                        })
                        .ok_or("mutation frame")?["message"]["frame_id"]
                        .clone();
                    if kind.starts_with("missing_") {
                        copy.retain(|r| {
                            !(r["message"]["type"] == "protocol_frame"
                                && r["message"]["frame_id"] == frame)
                        });
                    } else {
                        for row in copy.iter_mut().filter(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["frame_id"] == frame
                        }) {
                            if kind == "wrong_loan_resource" {
                                row["message"]["payload"][0] = json!(29);
                            } else {
                                let data = if kind == "forged_request" {
                                    json!({"sections":"forged"})
                                } else {
                                    json!({"success":true,"data":"forged"})
                                };
                                row["message"]["payload"] = json!(serde_json::to_vec(&data)?);
                            }
                        }
                    }
                }
                "overlapping_loans" => {
                    let next_id = copy
                        .iter()
                        .find(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["source"] == 7
                                && r["message"]["target"] != 14
                        })
                        .ok_or("second loan")?["message"]["frame_id"]
                        .clone();
                    let is_next_loan = |r: &Value| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == next_id
                    };
                    let next: Vec<_> = copy.iter().filter(|r| is_next_loan(r)).cloned().collect();
                    if next.is_empty() {
                        return Err("second loan mutation target".into());
                    }
                    copy.retain(|r| !is_next_loan(r));
                    let position = copy
                        .iter()
                        .position(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["action"] == "receive"
                                && r["message"]["source"] == 7
                                && r["message"]["target"] == 14
                        })
                        .ok_or("first loan receipt")?;
                    copy.splice(position + 1..position + 1, next);
                }
                "missing_close" => {
                    let ids: Vec<_> = copy
                        .iter()
                        .filter(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["source"] == 14
                                && r["message"]["target"] == 0
                                && r["message"]["payload"] == json!([4])
                        })
                        .map(|r| r["message"]["frame_id"].clone())
                        .collect();
                    if ids.is_empty() {
                        return Err("close mutation target".into());
                    }
                    copy.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && ids.contains(&r["message"]["frame_id"]))
                    });
                }
                "missing_exit" => {
                    let index = copy
                        .iter()
                        .position(|r| r["message"]["type"] == "exit")
                        .ok_or("exit mutation target")?;
                    copy.remove(index);
                }
                "forged_arguments" => {
                    let row = copy
                        .iter_mut()
                        .find(|r| r["message"]["type"] == "execute")
                        .ok_or("effect mutation target")?;
                    row["message"]["arguments"] = json!({"sections":"forged"});
                }
                "forged_original_result" => {
                    let row = copy
                        .iter_mut()
                        .find(|r| r["message"]["type"] == "simulator_result")
                        .ok_or("result mutation target")?;
                    row["message"]["result"]["data"] = json!({"forged":true});
                }
                "false_score" => {
                    let row = copy
                        .iter_mut()
                        .find(|r| r["message"]["type"] == "score")
                        .ok_or("score mutation target")?;
                    row["message"]["complete"] = json!(
                        !row["message"]["complete"]
                            .as_bool()
                            .ok_or("score outcome")?
                    );
                }
                _ => unreachable!(),
            }
            if copy == rows {
                return Err(format!("mutation {kind} did not change the evidence").into());
            }
        }
        let mutant = output.join(format!("{kind}.jsonl"));
        use std::io::Write;
        let mut file = std::io::BufWriter::new(fs::File::create_new(&mutant)?);
        for row in &copy {
            serde_json::to_writer(&mut file, row)?;
            writeln!(file)?;
        }
        file.flush()?;
        drop(file);
        let error = check(&copy, &projection)
            .err()
            .ok_or_else(|| format!("accepted mutant {kind}"))?;
        results.push(json!({"kind":kind,"status":"rejected","error":error.to_string()}));
    }
    let result = json!({"control":"passed","mutants":results});
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}
