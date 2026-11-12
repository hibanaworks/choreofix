//! Actual API effects and outcome-file bytes; EPF checks source execution.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const OUTCOME_FD: u8 = 16;
fn send(output: &mut impl Write, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    Ok(())
}
fn payload(message: &Value) -> Result<Vec<u8>> {
    message["payload"]
        .as_array()
        .ok_or("frame bytes")?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "invalid frame byte".into())
        })
        .collect()
}
pub fn audit(path: &Path) -> Result<Value> {
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|l| Ok(serde_json::from_str::<Value>(&l?)?))
        .collect::<Result<Vec<_>>>()?;
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
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("API actors")?;
    if projection["compiled"]["task"] != "8E" || fixture["task"] != "8E" || actors.len() != 3 {
        return Err("unexpected API task".into());
    }
    let depth = fixture["depth"]
        .as_u64()
        .filter(|n| *n <= 2)
        .ok_or("API depth")?;
    let passes = depth == 0;
    let binding = |agent: &str| -> Result<(u64, u64)> {
        match agent {
            "DB_ADMIN" => Ok((0, 5)),
            "BACKEND_DEV" => Ok((1, 6)),
            "TESTER" => Ok((2, 7)),
            _ => Err("unknown API participant".into()),
        }
    };
    let mut journal = crate::journal::read(records, &labels, actors)?;
    let dynamic = crate::projection::check_dynamic(projection, records)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if frames.values().any(|frame| frame.identity["session"] != 28)
        || calls.len() != 3
        || score["complete"] != passes
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("score events")?.len() != 3
    {
        return Err("incomplete API effect or score evidence".into());
    }
    let mut observed = Vec::new();
    for (&id, frame) in &frames {
        observed.push((frame.sent, 0, false, frame));
        observed.push((
            frame.received()?,
            receipts[&id]["receive_commit"]["ordinal"]
                .as_u64()
                .ok_or("receive ordinal")?,
            true,
            frame,
        ));
    }
    observed.sort_by_key(|(at, ordinal, _, _)| (*at, *ordinal));
    let mut notification = BTreeMap::<u64, (usize, Value)>::new();
    let mut delivered = BTreeMap::new();
    let mut open_sent = BTreeMap::new();
    let mut first_reply = BTreeMap::new();
    let mut pending_read = BTreeSet::new();
    let mut read_bytes = BTreeMap::<u64, Vec<u8>>::new();
    let mut complete_reads = BTreeMap::<u64, usize>::new();
    for (at, _, received, frame) in observed {
        let source = frame.identity["source"].as_u64().ok_or("source")?;
        let target = frame.identity["target"].as_u64().ok_or("target")?;
        let bytes = &frame.bytes;
        if !received {
            if matches!((source, target), (13, 11) | (14, 12)) {
                open_sent.entry(target).or_insert(at);
            }
            if matches!((source, target), (11, 13) | (12, 14)) {
                first_reply.entry(source).or_insert(at);
                if pending_read.remove(&source) {
                    if bytes.len() < 4
                        || usize::from(bytes[3]) != bytes.len() - 4
                        || bytes[0] != OUTCOME_FD
                        || bytes[1..3] != [0, 0]
                    {
                        return Err("invalid actual outcome read".into());
                    }
                    let expected = &notification.get(&source).ok_or("read without outcome")?.1;
                    let canonical = serde_json::to_vec(expected)?;
                    let current = read_bytes.entry(source).or_default();
                    current.extend_from_slice(&bytes[4..]);
                    if !canonical.starts_with(current) {
                        return Err("outcome bytes differ from actual notification".into());
                    }
                    if *current == canonical {
                        *complete_reads.entry(source).or_default() += 1;
                        current.clear();
                    }
                }
            }
        } else {
            if matches!((source, target), (0, 15) | (1, 6) | (2, 16))
                && let Ok(value) = serde_json::from_slice::<Value>(bytes)
                && value["tool_name"].is_string()
                && delivered.insert(source, value).is_some()
            {
                return Err("duplicate delivered API result".into());
            }
            if source == 2
                && [11, 12].contains(&target)
                && notification.insert(target, (at, frame.value()?)).is_some()
            {
                return Err("duplicate outcome notification".into());
            }
            if matches!((source, target), (13, 11) | (14, 12))
                && bytes.len() == 2
                && (bytes[0] != OUTCOME_FD || !pending_read.insert(target))
            {
                return Err("overlapping or invalid outcome read".into());
            }
        }
    }
    let mut effects = BTreeMap::new();
    let mut completed = BTreeMap::new();
    let mut used_closes = BTreeSet::new();
    let mut responses = BTreeMap::new();
    for call in calls.values() {
        let agent = call.request["agent"].as_str().ok_or("effect actor")?;
        let (role, vm) = binding(agent)?;
        let original = crate::journal::one(actors.iter().filter(|actor| actor["agent"] == agent))?;
        let tool = call.tool()?;
        if original["calls"][0]["tool"] != tool
            || original["calls"][0]["arguments"] != call.request["arguments"]
            || effects.insert(tool, call.emitted).is_some()
        {
            return Err("API effect differs from assignment".into());
        }
        let close = frames
            .values()
            .filter(|f| {
                f.direction(vm, role)
                    && f.bytes == [4]
                    && f.received.is_some_and(|at| at < call.emitted)
            })
            .max_by_key(|f| f.received)
            .ok_or("effect without committed close")?;
        if !used_closes.insert(close.sent) {
            return Err("close reused for another API effect".into());
        }
        let (at, result) = call.result()?;
        if result["success"] != (tool != "run_integration_tests" || passes)
            || delivered.remove(&role).as_ref() != Some(result)
            || responses.insert(agent, (call, result)).is_some()
        {
            return Err("actual API result mismatch".into());
        }
        completed.insert(tool, at);
        let event = crate::journal::one(
            score["events"]
                .as_array()
                .ok_or("score events")?
                .iter()
                .filter(|event| event["agent"] == agent && event["tool"] == tool),
        )?;
        if event["args"] != call.request["arguments"]
            || event["success"] != result["success"]
            || event["result"] != result["data"]
        {
            return Err("API score differs from actual effect".into());
        }
    }
    let effect = |tool: &str| effects.get(tool).copied().ok_or("missing effect");
    let done = |tool: &str| completed.get(tool).copied().ok_or("missing completion");
    if done("migrate_schema")? >= effect("implement_backend")?
        || done("implement_backend")? >= effect("run_integration_tests")?
    {
        return Err("actual API data dependency differs".into());
    }
    for (resource, agent, service) in [(15, "DB_ADMIN", 0), (16, "TESTER", 2)] {
        let (call, result) = responses.get(agent).ok_or("missing API operation")?;
        let request =
            crate::journal::one(frames.values().filter(|f| f.direction(resource, service)))?;
        let reply =
            crate::journal::one(frames.values().filter(|f| f.direction(service, resource)))?;
        if request.value()? != call.request["arguments"]
            || reply.value()? != **result
            || request.received()? >= call.emitted
            || call.result()?.0 >= reply.sent
        {
            return Err("actual resource request/effect/result differs".into());
        }
    }
    let test_result = responses.get("TESTER").ok_or("tester result")?.1;
    let finished = done("run_integration_tests")?;
    for owner in [11, 12] {
        let (notified, value) = notification
            .get(&owner)
            .ok_or("missing outcome notification")?;
        if value != test_result
            || *notified <= finished
            || first_reply.get(&owner).is_none_or(|at| at <= notified)
            || complete_reads.get(&owner).is_none_or(|n| *n < 1)
        {
            return Err("outcome file differs from actual test result".into());
        }
        if fixture["require_early_result_open"] == true
            && open_sent.get(&owner).is_none_or(|at| *at >= finished)
        {
            return Err("early-read stimulus was not exercised".into());
        }
    }
    if responses.len() != 3
        || !delivered.is_empty()
        || !pending_read.is_empty()
        || read_bytes.values().any(|v| !v.is_empty())
    {
        return Err("incomplete API result delivery".into());
    }
    Ok(
        json!({"task":"8E","depth":depth,"kind":"actual_wasi_resource_delegation","model_calls":null,
        "scored_benchmark":false,"status":"passed","participants":3,"domain_calls":calls.len(),
        "actual_resource_endpoint_requests":2,
        "actual_resource_endpoint_returns":frames.values().filter(|f|matches!((f.identity["source"].as_u64(),f.identity["target"].as_u64()),(Some(5),Some(3))|(Some(7),Some(4)))&&f.bytes.len()==3).count(),
        "simulator_complete":score["complete"],"expected_terminal_test_failure":!passes,
        "carrier_frames":frames.len(),"result_receipts":2,"read_dependency_observed":true,
        "result_opens":open_sent.len(),"early_result_opens":open_sent.values().filter(|at|**at<finished).count(),
        "complete_result_reads":complete_reads,"violations":0,"guest_exits":actors.len(),
        "original_test_delays":fixture["test_delays"],"dynamic_protocol":dynamic}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let rows = BufReader::new(File::open(path)?)
        .lines()
        .map(|l| Ok(serde_json::from_str::<Value>(&l?)?))
        .collect::<Result<Vec<_>>>()?;
    let mut reports = Vec::new();
    for kind in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_close",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_notification",
        "missing_read",
        "missing_exit",
        "false_score",
        "missing_loan",
        "wrong_loan_resource",
        "missing_resource_request",
        "forged_resource_request",
        "forged_resource_result",
        "missing_resource_return",
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
            let index = copy
                .iter()
                .position(|r| {
                    let m = &r["message"];
                    match kind {
                        "missing_close" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "receive"
                                && m["source"] == 5
                                && m["target"] == 0
                                && m["payload"] == json!([4])
                        }
                        "altered_frame" => {
                            m["type"] == "protocol_frame" && m["action"] == "receive"
                        }
                        "forged_arguments" => m["type"] == "execute",
                        "forged_result" => m["type"] == "simulator_result",
                        "missing_notification" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "receive"
                                && m["source"] == 2
                                && m["target"] == 11
                        }
                        "missing_read" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 11
                                && m["target"] == 13
                                && m["payload"].as_array().is_some_and(|b| b.len() > 5)
                        }
                        "missing_exit" => m["type"] == "exit",
                        "false_score" => m["type"] == "score",
                        "missing_loan" | "wrong_loan_resource" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 3
                                && m["target"] == 5
                        }
                        "missing_resource_request" | "forged_resource_request" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 15
                                && m["target"] == 0
                        }
                        "forged_resource_result" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 0
                                && m["target"] == 15
                        }
                        "missing_resource_return" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 5
                                && m["target"] == 3
                        }
                        _ => false,
                    }
                })
                .ok_or("mutation target")?;
            match kind {
                "missing_close" | "missing_notification" | "missing_read" | "missing_exit" => {
                    copy.remove(index);
                }
                "altered_frame" => copy[index]["message"]["source"] = json!(254),
                "forged_arguments" => {
                    copy[index]["message"]["arguments"] = json!({"component":"wrong"})
                }
                "forged_result" => {
                    copy[index]["message"]["result"]["data"] = json!({"forged":true})
                }
                "false_score" => {
                    copy[index]["message"]["complete"] =
                        json!(!copy[index]["message"]["complete"].as_bool().unwrap())
                }
                "missing_loan" | "missing_resource_request" | "missing_resource_return" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    copy.retain(|row| {
                        !(row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id)
                    });
                }
                "wrong_loan_resource" | "forged_resource_request" | "forged_resource_result" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    let mut bytes = payload(&copy[index]["message"])?;
                    match kind {
                        "wrong_loan_resource" => bytes[0] = 16,
                        "forged_resource_request" => {
                            bytes = serde_json::to_vec(&json!({"migration_name":"wrong_target"}))?
                        }
                        "forged_resource_result" => {
                            let mut response: Value = serde_json::from_slice(&bytes)?;
                            response["data"]["forged"] = json!(true);
                            bytes = serde_json::to_vec(&response)?;
                        }
                        _ => unreachable!(),
                    }
                    for row in &mut copy {
                        if row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id
                        {
                            row["message"]["payload"] = json!(bytes);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        let file = output.join(format!("{kind}.jsonl"));
        let mut out = std::io::BufWriter::new(File::create(&file)?);
        for r in &copy {
            send(&mut out, r)?
        }
        out.flush()?;
        let error = crate::journal::reject_mutant(path, &file, "--audit-api")?;
        let hash = crate::evidence::digest(&file)?;
        fs::remove_file(&file)?;
        reports.push(json!({"mutation":kind,"rejected":true,"sha256":hash,"error":error}));
    }
    let report = json!({"mutants":reports,"original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}
