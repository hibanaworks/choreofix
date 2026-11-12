//! Actual CI/CD requests, admitted values and results; EPF checks source execution.
use crate::{
    evidence::{Result, digest},
    journal::{one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

pub fn audit(path: &Path) -> Result<Value> {
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let mut result = check(&rows, &projection)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}

fn check(rows: &[Value], projection: &Value) -> Result<Value> {
    let labels = crate::projection::frames(projection, rows)?;
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if projection["compiled"]["task"] != "16E"
        || fixture["task"] != "16E"
        || fixture["type"] != "fixtures"
        || fixture["depth"].as_u64().is_none_or(|d| d > 2)
    {
        return Err("CI/CD fixture or compiled task".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("original actors")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("DEVELOPER"), Some("TESTER"), Some("DEPLOYER")])
    {
        return Err("CI/CD participants".into());
    }
    let mut journal = crate::journal::read(rows, &labels, actors)?;
    let dynamic = crate::projection::check_dynamic(projection, rows)?;
    journal.bind_receives(&dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if frames.values().any(|f| f.identity["session"] != 16)
        || calls.len() != 3
        || score["complete"] != true
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("score events")?.len() != 3
    {
        return Err("CI/CD completion or score".into());
    }
    let mut used = BTreeSet::new();
    for (agent, vm, service, resource, request_label, result_label) in [
        ("DEVELOPER", 0, 3, 11, 30, 31),
        ("TESTER", 1, 4, 11, 32, 33),
        ("DEPLOYER", 2, 5, 12, 34, 35),
    ] {
        let actor = actors
            .iter()
            .find(|a| a["agent"] == agent)
            .ok_or("CI/CD actor")?;
        let (id, call) = one(calls.iter().filter(|(_, c)| c.request["agent"] == agent))?;
        if !used.insert(*id)
            || actor["calls"].as_array().ok_or("original calls")?.len() != 1
            || actor["calls"][0]["tool"] != call.request["tool"]
            || actor["calls"][0]["arguments"] != call.request["arguments"]
        {
            return Err("CI/CD effect differs from assignment".into());
        }
        let request = one(frames
            .values()
            .filter(|f| f.direction(resource, service) && f.logical_label == request_label))?;
        let reply = one(frames
            .values()
            .filter(|f| f.direction(service, resource) && f.logical_label == result_label))?;
        let admitted = one(frames
            .values()
            .filter(|f| f.direction(service, vm) && f.logical_label == 8))?;
        let close = frames
            .values()
            .filter(|f| {
                f.direction(vm, service)
                    && f.logical_label == 7
                    && f.received.is_some_and(|at| at < admitted.sent)
            })
            .filter(|f| f.bytes == [4])
            .max_by_key(|f| f.received)
            .ok_or("effect without committed close")?;
        let (at, result) = call.result()?;
        if request.value()? != call.request["arguments"]
            || admitted.value()? != call.request["arguments"]
            || reply.value()? != *result
            || result["success"] != true
            || close.received()? >= admitted.sent
            || admitted.received()? >= request.sent
            || request.received()? >= call.emitted
            || at >= reply.sent
        {
            return Err("CI/CD admitted request/effect/result mismatch".into());
        }
        let event = one(score["events"]
            .as_array()
            .ok_or("score events")?
            .iter()
            .filter(|event| event["agent"] == agent))?;
        if event["tool"] != call.request["tool"]
            || event["args"] != call.request["arguments"]
            || event["success"] != result["success"]
            || event["result"] != result["data"]
        {
            return Err("score differs from actual CI/CD effect".into());
        }
    }
    Ok(
        json!({"status":"passed","task":"16E","depth":fixture["depth"],
        "kind":"actual_wasi_resource_delegation","scored_benchmark":false,"model_calls":null,
        "actual_resource_endpoint_requests":3,
        "actual_resource_endpoint_returns":frames.values().filter(|f| matches!(f.identity["source"].as_u64(),Some(0..=2))
            && matches!(f.identity["target"].as_u64(),Some(6|7)) && f.bytes.len()==3).count(),
        "domain_calls":calls.len(),"guest_exits":actors.len(),"carrier_frames":frames.len(),
        "notifications":frames.values().filter(|f| matches!(f.logical_label,20..=23)
            && matches!(f.identity["source"].as_u64(),Some(3..=5))
            && matches!(f.identity["target"].as_u64(),Some(3..=5))).count(),
        "simulator_complete":true,"violations":0,"dynamic_protocol":dynamic}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let rows: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let mut results = Vec::new();
    for kind in [
        "missing_loan",
        "wrong_loan_resource",
        "missing_request",
        "forged_request",
        "missing_reply",
        "forged_reply",
        "missing_return",
        "missing_notification",
        "missing_close",
        "forged_arguments",
        "forged_original_result",
        "missing_exit",
        "false_score",
        "wrong_frame_label",
        "wrong_lane",
    ] {
        let mut copy = rows.clone();
        match kind {
            "wrong_frame_label" | "wrong_lane" => {
                let selected = copy
                    .iter()
                    .find(|row| row["message"]["type"] == "protocol_frame")
                    .ok_or("header mutation frame")?["message"]["frame_id"]
                    .clone();
                let field = if kind == "wrong_frame_label" {
                    "frame_label"
                } else {
                    "lane"
                };
                let changed = if kind == "wrong_frame_label" {
                    65535
                } else {
                    255
                };
                for row in copy.iter_mut().filter(|row| {
                    row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == selected
                }) {
                    row["message"][field] = json!(changed);
                }
                // Change every observation together: send/receive equality alone still holds.
            }
            "missing_loan"
            | "wrong_loan_resource"
            | "missing_request"
            | "forged_request"
            | "missing_reply"
            | "forged_reply"
            | "missing_return"
            | "missing_notification" => {
                let (from, to) = match kind {
                    "missing_loan" | "wrong_loan_resource" => (6, 0),
                    "missing_request" | "forged_request" => (11, 3),
                    "missing_reply" | "forged_reply" => (3, 11),
                    "missing_return" => (0, 6),
                    "missing_notification" => (3, 4),
                    _ => unreachable!(),
                };
                let frame = copy
                    .iter()
                    .find(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["source"] == from
                            && r["message"]["target"] == to
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
                            row["message"]["payload"][0] = json!(12);
                        } else {
                            let data = if kind == "forged_request" {
                                json!({"component":"forged"})
                            } else {
                                json!({"success":true,"data":"forged"})
                            };
                            row["message"]["payload"] = json!(serde_json::to_vec(&data)?);
                        }
                    }
                }
            }
            "missing_close" => {
                let ids: Vec<_> = copy
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["source"] == 0
                            && r["message"]["target"] == 3
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
                row["message"]["arguments"] = json!({"component":"forged"});
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
                row["message"]["complete"] = json!(false);
            }
            _ => unreachable!(),
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
        let error = crate::journal::reject_mutant(path, &mutant, "--audit-cicd")?;
        let hash = digest(&mutant)?;
        fs::remove_file(&mutant)?;
        results.push(json!({"kind":kind,"status":"rejected","sha256":hash,"error":error}));
    }
    let result = json!({"control":"passed","mutants":results});
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}
