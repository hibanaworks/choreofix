//! EPF source conformance and independent request/effect/result correspondence.
//! These records never authorize an operation in the running experiment.
use crate::evidence::Result;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::Path,
};

fn pair(arguments: &Value) -> Option<Vec<&str>> {
    let mut modules: Vec<_> = arguments["modules"]
        .as_str()?
        .split(',')
        .map(str::trim)
        .collect();
    modules.sort();
    Some(modules)
}

fn original_call(assignment: &Value, count: usize, revising: bool) -> Result<&Value> {
    let index = if revising && count > 0 {
        1 + (count - 1) % 3
    } else {
        count
    };
    assignment["calls"]
        .get(index)
        .ok_or_else(|| "request after completion".into())
}

pub fn audit(path: &Path) -> Result<Value> {
    let records = crate::journal::records(path)?;
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
    let actors = fixture["fixtures"].as_array().ok_or("fixtures")?;
    let revising = fixture["task"] == "1H";
    let terminal_test_failure = fixture["terminal_test_failure"]
        .as_bool()
        .ok_or("terminal test result")?;
    let rows: &[(&str, u64, u64)] = match fixture["task"].as_str() {
        Some("1E") => &[
            ("AUTH_MODULE", 7, 28),
            ("DATABASE_MODULE", 8, 29),
            ("API_MODULE", 9, 30),
        ],
        Some("1M") => &[
            ("AUTH_MODULE", 7, 28),
            ("DATABASE_MODULE", 8, 29),
            ("API_MODULE", 9, 30),
            ("PAYMENT_MODULE", 10, 31),
            ("NOTIFY_MODULE", 11, 32),
        ],
        Some("1H") => &[
            ("AUTH_MODULE", 7, 28),
            ("DATABASE_MODULE", 8, 29),
            ("API_MODULE", 9, 30),
            ("PAYMENT_MODULE", 10, 31),
            ("NOTIFY_MODULE", 11, 32),
            ("ANALYTICS_MODULE", 12, 33),
            ("SEARCH_MODULE", 13, 34),
        ],
        _ => return Err("unknown codebase task".into()),
    };
    if rows.len() != actors.len() {
        return Err("participant count".into());
    }
    let modules: BTreeMap<_, _> = rows
        .iter()
        .map(|&(name, owner, resource)| (name, (owner, resource)))
        .collect();
    let assignments: BTreeMap<_, _> = actors
        .iter()
        .enumerate()
        .map(|(i, a)| Ok((a["agent"].as_str().ok_or("agent name")?, (i as u64, a))))
        .collect::<Result<_>>()?;
    let mut loans = BTreeMap::<u64, usize>::new();
    let mut returns = BTreeMap::<u64, usize>::new();
    let mut requests = BTreeMap::<(u64, String), Vec<(u64, Value)>>::new();
    let mut delivered = BTreeMap::<(u64, String), Vec<Value>>::new();
    let mut calls = BTreeMap::<&str, usize>::new();
    let mut outstanding = BTreeMap::<u64, Value>::new();
    let mut responses = BTreeMap::<(u64, String), Vec<(Value, Value)>>::new();
    let mut test_notifications = BTreeMap::<(u64, u64), Vec<Value>>::new();
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
    for (index, record) in records.iter().enumerate() {
        let message = &record["message"];
        if let Some(observed) = frame_rows.get(&index) {
            for &(_, frame, received) in observed {
                let source = frame.identity["source"].as_u64().ok_or("source")?;
                let target = frame.identity["target"].as_u64().ok_or("target")?;
                let bytes = &frame.bytes;
                let owner_traffic = rows
                    .iter()
                    .any(|&(_, owner, _)| source == owner || target == owner);
                if owner_traffic && bytes.len() != 3 {
                    let result: Value = serde_json::from_slice(bytes)?;
                    let actor = actors.get(source as usize).ok_or("test result sender")?;
                    if !revising
                        || result["tool_name"] != "run_local_tests"
                        || !actor["modules"]
                            .as_array()
                            .ok_or("assigned modules")?
                            .iter()
                            .any(|name| {
                                modules
                                    .get(name.as_str().unwrap_or(""))
                                    .is_some_and(|&(owner, _)| owner == target)
                            })
                    {
                        return Err("unexpected module notification".into());
                    }
                    if received {
                        test_notifications
                            .entry((source, target))
                            .or_default()
                            .push(result);
                    }
                } else if owner_traffic {
                    let resource = u64::from(bytes[0]);
                    let owner = rows
                        .iter()
                        .find(|&&(_, _, r)| r == resource)
                        .ok_or("unknown resource")?
                        .1;
                    let vm = if source == owner {
                        target
                    } else if target == owner {
                        source
                    } else {
                        return Err("wrong resource owner".into());
                    };
                    let actor = actors
                        .get(vm.checked_sub(14).ok_or("guest role")? as usize)
                        .ok_or("guest role")?;
                    if !actor["modules"]
                        .as_array()
                        .ok_or("assigned modules")?
                        .iter()
                        .any(|name| {
                            modules
                                .get(name.as_str().unwrap_or(""))
                                .is_some_and(|&(_, r)| r == resource)
                        })
                    {
                        return Err("Endpoint delivered to unassigned developer".into());
                    }
                    if !received {
                        if target == owner {
                            *returns.entry(resource).or_default() += 1;
                        } else {
                            *loans.entry(resource).or_default() += 1;
                        }
                    }
                }
                if received
                    && target < actors.len() as u64
                    && bytes.first() == Some(&4)
                    && bytes.len() > 1
                {
                    let arguments: Value = serde_json::from_slice(&bytes[1..])?;
                    let actor = &actors[target as usize];
                    let stage = *calls
                        .get(actor["agent"].as_str().ok_or("agent")?)
                        .unwrap_or(&0);
                    let tool = original_call(actor, stage, revising)?["tool"]
                        .as_str()
                        .ok_or("tool")?;
                    if source == target + 14 {
                        if tool == "commit_changes" {
                            return Err("commit bypassed resource Endpoints".into());
                        }
                    } else if rows.iter().any(|&(_, _, resource)| resource == source) {
                        if tool != "commit_changes" {
                            return Err("unexpected module request tool".into());
                        }
                    } else {
                        return Err("unexpected request sender".into());
                    }
                    requests
                        .entry((target, tool.into()))
                        .or_default()
                        .push((source, arguments));
                }
                if received
                    && source < actors.len() as u64
                    && (target == source + 14
                        || rows.iter().any(|&(_, _, resource)| resource == target))
                {
                    let result: Value = serde_json::from_slice(bytes)?;
                    let tool = result["tool_name"]
                        .as_str()
                        .ok_or("result tool")?
                        .to_owned();
                    if rows.iter().any(|&(_, _, resource)| resource == target) {
                        if tool != "commit_changes" {
                            return Err("unexpected module result tool".into());
                        }
                    } else if tool == "commit_changes" {
                        return Err("result bypassed resource Endpoint".into());
                    }
                    delivered.entry((source, tool)).or_default().push(result);
                }
            }
        }
        match (record["direction"].as_str(), message["type"].as_str()) {
            (Some("runtime"), Some("execute")) => {
                let agent = message["agent"].as_str().ok_or("effect agent")?;
                let &(role, assignment) = assignments.get(agent).ok_or("unknown effect agent")?;
                let tool = message["tool"].as_str().ok_or("effect tool")?;
                let count = calls.entry(agent).or_default();
                let expected = original_call(assignment, *count, revising)?;
                if revising && tool == "implement_code" && *count > 1 {
                    let previous = responses
                        .get(&(role, "run_local_tests".into()))
                        .and_then(|values| values.last())
                        .ok_or("revision before test result")?;
                    if previous.1["success"] != false
                        || delivered
                            .get(&(role, "run_local_tests".into()))
                            .and_then(|values| values.last())
                            != Some(&previous.1)
                    {
                        return Err("revision without a received failure result".into());
                    }
                }
                let arguments_match = if tool == "commit_changes" {
                    pair(&expected["arguments"]).is_some()
                        && pair(&expected["arguments"]) == pair(&message["arguments"])
                } else {
                    expected["arguments"] == message["arguments"]
                };
                if expected["tool"] != tool || !arguments_match {
                    return Err("task fidelity mismatch".into());
                }
                let observed = requests
                    .remove(&(role, tool.to_owned()))
                    .ok_or("effect without projected request")?;
                if observed
                    .iter()
                    .any(|(_, args)| args != &message["arguments"])
                {
                    return Err("request/effect argument mismatch".into());
                }
                if tool == "commit_changes" {
                    let required = assignment["modules"]
                        .as_array()
                        .ok_or("modules")?
                        .iter()
                        .map(|name| {
                            modules
                                .get(name.as_str().ok_or("module name")?)
                                .map(|&(_, r)| r)
                                .ok_or_else(|| "unknown module".into())
                        })
                        .collect::<Result<BTreeSet<_>>>()?;
                    if observed.len() != 2
                        || observed.iter().map(|&(r, _)| r).collect::<BTreeSet<_>>() != required
                        || delivered
                            .get(&(role, "implement_code".into()))
                            .map(Vec::len)
                            != Some(if revising { (*count - 2) / 3 + 1 } else { 1 })
                    {
                        return Err(
                            "effect without both actual module requests and completed implementation"
                                .into(),
                        );
                    }
                    let names = assignment["modules"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect::<BTreeSet<_>>();
                    if active.values().any(|other| !other.is_disjoint(&names)) {
                        return Err("overlapping physical modules".into());
                    }
                    active.insert(message["id"].as_u64().ok_or("effect id")?, names);
                    maximum_writes = maximum_writes.max(active.len());
                } else {
                    if observed.len() != 1 || observed[0].0 != role + 14 {
                        return Err("local request count".into());
                    }
                }
                *count += 1;
                if outstanding
                    .insert(message["id"].as_u64().ok_or("effect id")?, message.clone())
                    .is_some()
                {
                    return Err("duplicate effect id".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = message["id"].as_u64().ok_or("result id")?;
                let call = outstanding
                    .remove(&id)
                    .ok_or("unmatched simulator result")?;
                if call["tool"] != message["result"]["tool_name"]
                    || message["result"]["success"].as_bool().is_none()
                    || (!(revising && call["tool"] == "run_local_tests")
                        && message["result"]["success"]
                            != !(terminal_test_failure && call["tool"] == "run_local_tests"))
                {
                    return Err("unexpected original simulator result".into());
                }
                let role = assignments[call["agent"].as_str().unwrap()].0;
                let previous = responses
                    .entry((role, call["tool"].as_str().unwrap().to_owned()))
                    .or_default();
                if !revising && !previous.is_empty() {
                    return Err("duplicate domain result".into());
                }
                previous.push((call, message["result"].clone()));
                active.remove(&id);
            }
            _ => {}
        }
    }
    let score = &journal.score;
    let mut attempts = BTreeMap::new();
    for (&agent, &(role, assignment)) in &assignments {
        let tests = responses
            .get(&(role, "run_local_tests".into()))
            .ok_or("missing test results")?;
        let visits = tests.len();
        if visits == 0
            || (!revising && visits != 1)
            || (revising
                && (tests.last().unwrap().1["success"] != true
                    || tests[..visits - 1]
                        .iter()
                        .any(|(_, result)| result["success"] != false)))
        {
            return Err("invalid reconsideration outcomes".into());
        }
        attempts.insert(agent, visits);
        if calls.get(agent) != Some(&(1 + 3 * visits)) {
            return Err("incomplete revision sequence".into());
        }
        if revising {
            for module in assignment["modules"].as_array().ok_or("modules")? {
                let owner = modules[module.as_str().ok_or("module")?].0;
                let notified = test_notifications
                    .remove(&(role, owner))
                    .ok_or("module owner did not learn test outcome")?;
                if notified.len() != visits
                    || notified
                        .iter()
                        .zip(tests)
                        .any(|(actual, (_, expected))| actual != expected)
                {
                    return Err("module owner learned a different test outcome".into());
                }
            }
        }
        for tool in [
            "design_feature",
            "implement_code",
            "commit_changes",
            "run_local_tests",
        ] {
            let results = responses
                .get(&(role, tool.into()))
                .ok_or("missing effect result")?;
            let values = delivered
                .remove(&(role, tool.into()))
                .ok_or("missing received result")?;
            let copies = if tool == "commit_changes" { 2 } else { 1 };
            let count = if tool == "design_feature" { 1 } else { visits };
            if results.len() != count
                || values.len() != copies * count
                || values
                    .chunks_exact(copies)
                    .zip(results)
                    .any(|(actual, (_, expected))| actual.iter().any(|value| value != expected))
            {
                return Err("effect and delivered result disagree".into());
            }
            let events = score["events"]
                .as_array()
                .ok_or("score events")?
                .iter()
                .filter(|e| e["agent"] == agent && e["tool"] == tool)
                .collect::<Vec<_>>();
            if events.len() != count {
                return Err("original simulator result mismatch".into());
            }
            for (event, (call, result)) in events.into_iter().zip(results) {
                let arguments = if tool == "commit_changes" {
                    json!({"modules":call["arguments"]["modules"].as_str().unwrap().split(',').map(str::trim).collect::<Vec<_>>()})
                } else {
                    json!({"feature":call["arguments"]["feature_name"]})
                };
                if event["args"] != arguments
                    || event["result"] != result["data"]
                    || event["success"] != result["success"]
                {
                    return Err("original simulator result mismatch".into());
                }
            }
        }
        if assignment["calls"]
            .as_array()
            .ok_or("original calls")?
            .len()
            != 4
        {
            return Err("original task changed".into());
        }
    }
    if calls.len() != actors.len()
        || !test_notifications.is_empty()
        || !outstanding.is_empty()
        || !requests.is_empty()
        || !delivered.is_empty()
        || !active.is_empty()
        || rows.iter().any(|&(name, _, r)| {
            let expected: usize = assignments
                .iter()
                .filter(|(_, (_, assignment))| {
                    assignment["modules"]
                        .as_array()
                        .unwrap()
                        .contains(&json!(name))
                })
                .map(|(agent, _)| attempts[agent])
                .sum();
            loans.get(&r) != Some(&expected) || returns.get(&r) != Some(&expected)
        })
        || score["complete"] != (revising || !terminal_test_failure)
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("events")?.len() != calls.values().sum::<usize>()
        || maximum_writes == 0
        || maximum_writes > actors.len() / 2
    {
        return Err("incomplete delegation boundary evidence".into());
    }
    Ok(
        json!({"task":fixture["task"],"kind":"persisted_direct_wasi_delegation_audit","model_calls":Value::Null,"scored_benchmark":false,"status":"passed",
        "participants":actors.len(),"domain_calls":calls.values().sum::<usize>(),"carrier_frames":journal.frames.len(),"max_disjoint_commits":maximum_writes,
        "simulator_complete":score["complete"],"expected_terminal_test_failure":terminal_test_failure,"violations":0,"guest_exits":actors.len(),
        "attempts":attempts,"actual_resource_endpoint_requests":2*attempts.values().sum::<usize>(),"actual_resource_endpoint_returns":2*attempts.values().sum::<usize>(),"original_test_delays":fixture["test_delays"],"dynamic_protocol":dynamic}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    let projection = crate::projection::load(path)?;
    fs::create_dir(output)?;
    let records = crate::journal::records(path)?;
    let mut checked = Vec::new();
    for mutation in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_request",
        "forged_request_arguments",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_exit",
        "false_score",
        "commit_before_second_request",
        "return_before_result",
        "missing_loan",
    ] {
        let mut copy = records.clone();
        if matches!(mutation, "wrong_frame_label" | "wrong_lane") {
            let frame = copy
                .iter()
                .find(|row| row["message"]["type"] == "protocol_frame")
                .ok_or("frame for header mutation")?["message"]
                .clone();
            let field = if mutation == "wrong_frame_label" {
                "frame_label"
            } else {
                "lane"
            };
            let replacement = if mutation == "wrong_frame_label" {
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
            if matches!(
                mutation,
                "commit_before_second_request" | "return_before_result" | "missing_loan"
            ) {
                if mutation == "commit_before_second_request" {
                    let effect = copy
                        .iter()
                        .position(|r| {
                            r["message"]["type"] == "execute"
                                && r["message"]["agent"] == "DEVELOPER_A"
                                && r["message"]["tool"] == "commit_changes"
                        })
                        .ok_or("commit mutation target")?;
                    let effect = copy.remove(effect);
                    let before = copy
                        .iter()
                        .position(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["action"] == "receive"
                                && r["message"]["source"] == 29
                                && r["message"]["target"] == 0
                        })
                        .ok_or("second module request")?;
                    copy.insert(before, effect);
                } else {
                    let (source, target) = if mutation == "missing_loan" {
                        (7, 14)
                    } else {
                        (14, 7)
                    };
                    let id = copy
                        .iter()
                        .find(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["action"] == "send"
                                && r["message"]["source"] == source
                                && r["message"]["target"] == target
                        })
                        .ok_or("loan mutation target")?["message"]["frame_id"]
                        .clone();
                    let moved: Vec<_> = copy
                        .iter()
                        .filter(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["frame_id"] == id
                        })
                        .cloned()
                        .collect();
                    copy.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == id)
                    });
                    if mutation == "return_before_result" {
                        let before = copy
                            .iter()
                            .position(|r| {
                                r["message"]["type"] == "protocol_frame"
                                    && r["message"]["action"] == "send"
                                    && r["message"]["source"] == 0
                                    && r["message"]["target"] == 28
                            })
                            .ok_or("commit reply")?;
                        copy.splice(before..before, moved);
                    }
                }
            } else {
                let index = copy
                    .iter()
                    .position(|r| {
                        let m = &r["message"];
                        match mutation {
                            "missing_request" | "forged_request_arguments" => {
                                r["direction"] == "runtime"
                                    && m["type"] == "protocol_frame"
                                    && m["action"] == "receive"
                                    && m["source"] == 14
                                    && m["target"] == 0
                                    && m["payload"][0] == 4
                            }
                            "altered_frame" => {
                                m["type"] == "protocol_frame" && m["action"] == "receive"
                            }
                            "forged_arguments" => m["type"] == "execute",
                            "forged_result" => {
                                r["direction"] == "simulator" && m["type"] == "simulator_result"
                            }
                            "missing_exit" => m["type"] == "exit",
                            "false_score" => m["type"] == "score",
                            _ => false,
                        }
                    })
                    .ok_or("mutation target")?;
                match mutation {
                    "missing_request" | "missing_exit" => {
                        copy.remove(index);
                    }
                    "forged_request_arguments" => {
                        let id = copy[index]["message"]["frame_id"].clone();
                        for r in &mut copy {
                            if r["message"]["type"] == "protocol_frame"
                                && r["message"]["frame_id"] == id
                            {
                                r["message"]["payload"] =
                                    json!(b"\x04{\"feature_name\":\"forged\"}".as_slice());
                            }
                        }
                    }
                    "altered_frame" => copy[index]["message"]["source"] = json!(254),
                    "forged_arguments" => {
                        copy[index]["message"]["arguments"] = json!({"feature_name":"forged"})
                    }
                    "forged_result" => {
                        copy[index]["message"]["result"]["data"] = json!({"forged":true})
                    }
                    "false_score" => {
                        copy[index]["message"]["complete"] = json!(
                            !copy[index]["message"]["complete"]
                                .as_bool()
                                .ok_or("score completion")?
                        )
                    }
                    _ => unreachable!(),
                }
            }
        }
        let mutated = output.join(format!("{mutation}.jsonl"));
        let mut file = std::io::BufWriter::new(File::create(&mutated)?);
        for record in &copy {
            serde_json::to_writer(&mut file, record)?;
            writeln!(file)?;
        }
        file.flush()?;
        let error = check(&copy, &projection)
            .err()
            .ok_or("audit accepted mutated evidence")?
            .to_string();
        checked.push(json!({"mutation":mutation,"rejected":true,"error":error}));
    }
    let result = json!({"mutants":checked,"original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok(result)
}
