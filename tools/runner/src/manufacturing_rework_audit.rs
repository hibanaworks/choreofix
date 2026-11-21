//! Match workshop effects to individual file and equipment receives in the recorded execution.
use crate::{
    evidence::{Result, digest},
    journal::{self, Frame},
    projection,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(crate) fn audit(path: &Path) -> Result<Value> {
    let rows = journal::records(path)?;
    let artifact = projection::load(path)?;
    let labels = projection::frames(&artifact, &rows)?;
    let epf = projection::check_dynamic(&artifact, &rows)?;
    let mut report = verify(&artifact, &rows, &labels, &epf)?;
    report["compiled_projection_sha256"] = json!(digest(&path.with_file_name("projection.json"))?);
    report["source_events_sha256"] = json!(digest(path)?);
    Ok(report)
}

pub(super) fn verify(
    artifact: &Value,
    rows: &[Value],
    labels: &BTreeMap<u64, u8>,
    epf: &Value,
) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let inputs = &rows.first().ok_or("missing workshop journal")?["message"];
    let actors = inputs["fixtures"]
        .as_array()
        .ok_or("workshop participants")?;
    if inputs["task"] != "11M"
        || compiled["task"] != "11M"
        || actors.len() != 5
        || actors
            .iter()
            .filter_map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                "DISPATCHER",
                "INSPECTOR",
                "WORKER_A",
                "WORKER_B",
                "WORKER_C",
            ])
    {
        return Err("workshop task or participants differ".into());
    }
    let mut history = journal::read(rows, labels, actors)?;
    history.bind_receives(epf)?;
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("missing projected workshop role {name}").into())
    };
    let dispatcher = role("DISPATCHER")?;
    let inspector = role("INSPECTOR")?;
    let bench = role("BENCH")?;
    let gauge = role("GAUGE")?;
    let readout = role("READOUT")?;
    let depot = role("DEPOT")?;
    let world = history.score["events"]
        .as_array()
        .ok_or("workshop score events")?;
    if history.score["violations"] != json!([])
        || history.score["complete"].as_bool().is_none()
        || world.len() != history.calls.len()
    {
        return Err("workshop effect count, score or violations differ".into());
    }
    let mut calls = history.calls.iter().collect::<Vec<_>>();
    calls.sort_by_key(|(_, c)| c.emitted);
    let mut spent = BTreeSet::new();
    let mut scored = BTreeSet::new();
    let mut request_edges = BTreeSet::new();
    let mut evidence = Vec::new();
    // A part, gauge reading and work order are real simulator outputs kept for correspondence.
    let mut parts: [Option<(&Value, usize)>; 6] = [None; 6];
    let mut readings: [Option<(&Value, usize)>; 6] = [None; 6];
    let mut orders: [Option<(&Value, usize)>; 6] = [None; 6];
    let mut intervals = Vec::new();
    for (id, call) in calls {
        let args = &call.request["arguments"];
        let job = args["job_id"].as_str().ok_or("manufacturing job")?;
        let (index, agent, station, file, worker) = match job {
            "alpha_job_a" => (0u8, "WORKER_A", "ALPHA", "ALPHA", "WORKER_A"),
            "beta_job_b" => (1, "WORKER_B", "BETA", "BETA", "WORKER_B"),
            "gamma_job_c" => (2, "WORKER_C", "GAMMA", "GAMMA", "WORKER_C"),
            "beta_job_a" => (3, "WORKER_A", "BETA", "BETA_A", "BETA_SERVICE"),
            "gamma_job_b" => (4, "WORKER_B", "GAMMA", "GAMMA_B", "GAMMA_SERVICE"),
            "alpha_job_c" => (5, "WORKER_C", "ALPHA", "ALPHA_C", "ALPHA_SERVICE"),
            _ => return Err("unknown manufacturing job".into()),
        };
        let i = usize::from(index);
        let worker = role(worker)?;
        let file = role(file)?;
        let machine = role(&format!("{station}_MACHINE"))?;
        let (reply_at, result) = call.result()?;
        let tool = call.tool()?;
        let expected_agent = match tool {
            "do_job" => agent,
            "inspect_quality" => "INSPECTOR",
            "dispatch_rework" => "DISPATCHER",
            _ => return Err("unexpected workshop effect".into()),
        };
        if call.request["agent"] != expected_agent || result["tool_name"] != tool {
            return Err("workshop effect executed for the wrong participant".into());
        }
        let (score_idx, _) = journal::one(world.iter().enumerate().filter(|(_, s)| {
            s["agent"] == expected_agent
                && s["tool"] == tool
                && s["args"] == *args
                && s["success"] == result["success"]
                && s["result"] == result["data"]
                && s["violations"] == json!([])
        }))?;
        if !scored.insert(score_idx) {
            return Err("workshop score reused".into());
        }
        let mut matched = Vec::new();
        // Pick the next unconsumed occurrence on this logical edge. Equal arguments can recur.
        let mut frame = |from: u64,
                         to: u64,
                         label: u8,
                         value: Option<&Value>,
                         after: Option<usize>,
                         before: Option<usize>|
         -> Result<&Frame> {
            let edge = (from, to, label);
            request_edges.insert(edge);
            let (fid, f) = history
                .frames
                .iter()
                .filter(|(n, f)| {
                    !spent.contains(*n)
                        && f.direction(from, to)
                        && f.logical_label == label
                        && match value {
                            Some(expected) => f.value().is_ok_and(|v| v == *expected),
                            None => f.bytes.is_empty(),
                        }
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or_else(|| format!("missing {job} {tool} frame {edge:?}"))?;
            if after.is_some_and(|a| f.sent <= a)
                || before.is_some_and(|b| f.received.is_none_or(|r| r >= b))
            {
                return Err(format!("out-of-order {job} {tool} frame {edge:?}").into());
            }
            spent.insert(*fid);
            matched.push(*fid);
            Ok(f)
        };
        match tool {
            "do_job" => {
                let part = &result["data"]["part"];
                let previous = parts[i];
                if args != &json!({"job_id":job})
                    || result["success"] != true
                    || part["job_id"] != job
                    || part["worker"] != agent
                    || part["station"] != format!("STATION_{station}")
                    || part["attempt"].as_u64()
                        != Some(
                            previous
                                .map_or(0, |(p, _)| p["attempt"].as_u64().unwrap_or(0))
                                .checked_add(1)
                                .ok_or("part attempt overflow")?,
                        )
                    || part["artifact_id"] != format!("part/{job}/{}", part["attempt"])
                    || (previous.is_some() && orders[i].is_none())
                    || result["data"]["rework_order"]
                        != *orders[i]
                            .map(|(o, _)| &o["data"]["order"])
                            .unwrap_or(&Value::Null)
                {
                    return Err(
                        "manufactured part differs from the submission or its rework order".into(),
                    );
                }
                let admission = frame(
                    file,
                    worker,
                    35,
                    Some(args),
                    orders[i].map(|(_, at)| at),
                    Some(call.emitted),
                )?
                .received()?;
                let claim = frame(
                    worker,
                    role(&format!("{station}_OWNER"))?,
                    index,
                    Some(args),
                    Some(admission),
                    Some(call.emitted),
                )?
                .received()?;
                if index < 3 {
                    frame(
                        machine,
                        depot,
                        10,
                        Some(args),
                        Some(claim),
                        Some(call.emitted),
                    )?;
                    let tool_role = [role("TOOL_0")?, role("TOOL_1")?];
                    let (slot, _) =
                        journal::one(tool_role.iter().enumerate().filter(|(slot, t)| {
                            history.frames.values().any(|f| {
                                f.direction(**t, depot)
                                    && f.logical_label == 3 + *slot as u8
                                    && f.sent > admission
                                    && f.received.is_some_and(|at| at < call.emitted)
                                    && f.value().is_ok_and(|v| v == *args)
                            })
                        }))?;
                    frame(
                        tool_role[slot],
                        depot,
                        3 + slot as u8,
                        Some(args),
                        Some(admission),
                        Some(call.emitted),
                    )?;
                    let tool_reply = frame(
                        depot,
                        tool_role[slot],
                        5 + slot as u8,
                        Some(result),
                        Some(reply_at),
                        None,
                    )?
                    .received()?;
                    frame(
                        depot,
                        machine,
                        machine.try_into()?,
                        Some(result),
                        Some(reply_at),
                        None,
                    )?;
                    frame(worker, file, 21, Some(result), Some(tool_reply), None)?;
                    frame(worker, inspector, index, Some(part), Some(tool_reply), None)?;
                } else {
                    let drive = role(&format!("{station}_DRIVE"))?;
                    frame(
                        machine,
                        drive,
                        20,
                        Some(args),
                        Some(claim),
                        Some(call.emitted),
                    )?;
                    let returned = frame(drive, machine, 21, Some(result), Some(reply_at), None)?
                        .received()?;
                    frame(worker, file, 21, Some(result), Some(returned), None)?;
                    frame(worker, inspector, index, Some(part), Some(returned), None)?;
                }
                parts[i] = Some((part, reply_at));
                readings[i] = None;
                orders[i] = None;
                intervals.push((station, index < 3, call.started()?.1, call.ended()?.1));
            }
            "inspect_quality" => {
                let (part, produced) = parts[i].ok_or("inspection without a manufactured part")?;
                if produced >= call.emitted
                    || args != &json!({"job_id":job})
                    || result["data"]["input_part"] != *part
                    || readings[i].is_some()
                    || result["data"]["receipt"].as_str().is_none_or(str::is_empty)
                {
                    return Err("wrong or repeated inspected part".into());
                }
                let submitted = frame(
                    inspector,
                    worker,
                    index,
                    Some(args),
                    None,
                    Some(call.emitted),
                )?
                .received()?;
                let presented = frame(
                    worker,
                    bench,
                    index,
                    Some(args),
                    Some(submitted),
                    Some(call.emitted),
                )?
                .received()?;
                frame(
                    gauge,
                    readout,
                    index,
                    Some(args),
                    Some(presented),
                    Some(call.emitted),
                )?;
                let actual =
                    frame(readout, gauge, 34, Some(result), Some(reply_at), None)?.received()?;
                frame(gauge, inspector, 34, Some(result), Some(actual), None)?;
                let approved = result["success"].as_bool().ok_or("missing gauge verdict")?;
                let verdict_label = match (index < 3, approved) {
                    (true, _) => index,
                    (false, true) => 25,
                    (false, false) => 26,
                };
                let measured = frame(
                    bench,
                    worker,
                    verdict_label,
                    Some(result),
                    Some(actual),
                    None,
                )?
                .received()?;
                if approved {
                    frame(worker, file, 36, Some(result), Some(measured), None)?;
                }
                let delivered = frame(
                    worker,
                    dispatcher,
                    26,
                    (!approved).then_some(result),
                    Some(measured),
                    None,
                )?
                .received()?;
                readings[i] = Some((result, delivered));
            }
            "dispatch_rework" => {
                let (failed, delivered) = readings[i].ok_or("dispatch without inspection")?;
                if failed["success"] != false
                    || orders[i].is_some()
                    || result["success"] != true
                    || result["data"]["input_part"] != failed["data"]["input_part"]
                    || result["data"]["inspection"] != failed["data"]["receipt"]
                    || result["data"]["worker"] != agent
                    || result["data"]["reason"] != *args.get("reason").unwrap_or(&json!(""))
                    || result["data"]["order"].as_str().is_none_or(str::is_empty)
                {
                    return Err("wrong or reused dispatch verdict".into());
                }
                frame(
                    dispatcher,
                    worker,
                    27,
                    Some(args),
                    Some(delivered),
                    Some(call.emitted),
                )?;
                frame(
                    worker,
                    dispatcher,
                    index,
                    Some(result),
                    Some(reply_at),
                    None,
                )?;
                let delivered =
                    frame(worker, file, 36, Some(result), Some(reply_at), None)?.received()?;
                orders[i] = Some((result, delivered));
            }
            _ => unreachable!(),
        }
        evidence.push(
            json!({"rpc_id":id,"agent":expected_agent,"tool":tool,"job":job,"frames":matched}),
        );
    }
    for (id, f) in &history.frames {
        let edge = (
            f.identity["source"].as_u64().ok_or("frame source")?,
            f.identity["target"].as_u64().ok_or("frame target")?,
            f.logical_label,
        );
        if !spent.contains(id)
            && (request_edges.contains(&edge) || f.value().is_ok_and(|v| v.is_object()))
        {
            return Err("unmatched workshop request or reply".into());
        }
    }
    for &(station, tool, start, _) in &intervals {
        let active = intervals
            .iter()
            .filter(|(_, _, a, b)| *a <= start && start < *b)
            .collect::<Vec<_>>();
        if active.iter().filter(|(s, _, _, _)| *s == station).count() > 1
            || (tool && active.iter().filter(|(_, t, _, _)| *t).count() > 2)
        {
            return Err("overlapping effects exceed workshop capacity".into());
        }
    }
    if history.score["complete"]
        != readings
            .iter()
            .all(|r| r.is_some_and(|(r, _)| r["success"] == true))
    {
        return Err("business score differs from the six gauged parts".into());
    }
    Ok(
        json!({"status":"passed","task":"11M","simulator_complete":history.score["complete"],"violations":0,"domain_calls":evidence.len(),"effects":evidence,"captured_records":epf["captured_records"],"runtime_binary_sha256":artifact["runtime_binary_sha256"]}),
    )
}
