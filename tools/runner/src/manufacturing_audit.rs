//! Actual file/resource receives and real effects; source execution belongs to EPF.
use crate::{
    evidence::{Result, save},
    journal::Frame,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn take_frame<'a>(
    frames: &'a BTreeMap<u64, Frame<'a>>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<(u64, &'a Frame<'a>)> {
    let mut found = frames.iter().filter(|(_, frame)| matches(frame));
    let (&id, frame) = found.next().ok_or("missing manufacturing occurrence")?;
    if found.next().is_some() || !used.insert(id) {
        return Err("duplicate or reused manufacturing occurrence".into());
    }
    Ok((id, frame))
}
pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = journal.execution;
    let compiled = &journal.projection["compiled"];

    let name =
        |group: &str, key: &str| crate::projection::named_id(&compiled[group], key).map(u64::from);
    let supervisor = name("roles", "SUPERVISOR")?;
    let crate::journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if calls.len() != 5
        || !simulator_complete
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("manufacturing completion, session or effect population differs".into());
    }
    let mut used = BTreeSet::new();
    let mut done_receipts = Vec::new();
    let mut actual_jobs = Vec::new();
    let mut file_requests = 0;
    let mut resource_requests = 0;
    for (agent, job, machine, file, request, resource_request, resource_reply, file_reply, done) in [
        (
            "TECHNICIAN_A",
            "lathe_job_a",
            "LATHE",
            "LATHE_FILE_A",
            "RequestLatheA",
            "LatheJobA",
            "LatheResult",
            "FileLatheResultA",
            "LatheDoneA",
        ),
        (
            "TECHNICIAN_B",
            "lathe_job_b",
            "LATHE",
            "LATHE_FILE_B",
            "RequestLatheB",
            "LatheJobB",
            "LatheResult",
            "FileLatheResultB",
            "LatheDoneB",
        ),
        (
            "TECHNICIAN_A",
            "mill_job_a",
            "MILL",
            "MILL_FILE_A",
            "RequestMillA",
            "MillJobA",
            "MillResult",
            "FileMillResultA",
            "MillDoneA",
        ),
        (
            "TECHNICIAN_B",
            "mill_job_b",
            "MILL",
            "MILL_FILE_B",
            "RequestMillB",
            "MillJobB",
            "MillResult",
            "FileMillResultB",
            "MillDoneB",
        ),
    ] {
        let matches = calls
            .values()
            .filter(|call| {
                call.request["agent"] == agent
                    && call.tool == "do_job"
                    && call.request["arguments"]["job_id"] == job
            })
            .collect::<Vec<_>>();
        let [call] = matches.as_slice() else {
            return Err("missing or repeated job effect".into());
        };
        let arguments = json!({"job_id":job});
        let (at, result) = call.result;
        if call.request["arguments"] != arguments
            || result["tool_name"] != "do_job"
            || result["success"] != true
            || result["data"]
                != json!({"job_id":job,"agent":agent,"machine":machine,"status":"completed"})
        {
            return Err("manufacturing job arguments or actual result differ".into());
        }
        let actor = name("roles", agent)?;
        let file = name("roles", file)?;
        let resource = name("roles", machine)?;
        let request_label = name("labels", request)? as u8;
        let (_, admitted) = take_frame(&frames, &mut used, |frame| {
            frame.direction(file, actor) && frame.logical_label == request_label
        })?;
        if admitted.received >= call.emitted || admitted.value()? != arguments {
            return Err("job effect lacks its unique actual file admission".into());
        }
        file_requests += 1;
        let resource_label = name("labels", resource_request)? as u8;
        let (_, received) = take_frame(&frames, &mut used, |frame| {
            frame.direction(resource, actor) && frame.logical_label == resource_label
        })?;
        if received.received >= call.emitted || received.value()? != arguments {
            return Err("job resource differs from admitted arguments".into());
        }
        resource_requests += 1;
        for (target, label) in [
            (resource, resource_reply),
            (file, file_reply),
            (supervisor, done),
        ] {
            let label = name("labels", label)? as u8;
            let (_, frame) = take_frame(&frames, &mut used, |frame| {
                frame.direction(actor, target) && frame.logical_label == label
            })?;
            if frame.sent <= at || frame.value()? != *result {
                return Err("job reply or report differs from the actual effect".into());
            }
            let received = frame.received;
            if target == supervisor {
                done_receipts.push(received);
            }
        }
        actual_jobs.push(result["data"].clone());
    }
    actual_jobs.sort_by_key(|job| {
        job["job_id"]
            .as_str()
            .expect("checked job identity")
            .to_owned()
    });
    let reports = calls
        .values()
        .filter(|call| call.request["agent"] == "SUPERVISOR" && call.tool == "collect_reports")
        .collect::<Vec<_>>();
    let [call] = reports.as_slice() else {
        return Err("missing or repeated report effect".into());
    };
    let (at, result) = call.result;
    if call.request["arguments"] != json!({})
        || result["tool_name"] != "collect_reports"
        || result["success"] != true
        || result["data"]["agent"] != "SUPERVISOR"
        || result["data"]["scope"] != "all_jobs"
        || result["data"]["status"] != "collected"
        || done_receipts.len() != 4
        || done_receipts
            .iter()
            .any(|received| *received >= call.emitted)
    {
        return Err("report effect lacks its four actual completed job deliveries".into());
    }
    let mut reported = result["data"]["jobs"]
        .as_array()
        .cloned()
        .ok_or("actual reported jobs")?;
    if reported.iter().any(|job| !job["job_id"].is_string()) {
        return Err("report job identity missing".into());
    }
    reported.sort_by_key(|job| {
        job["job_id"]
            .as_str()
            .expect("checked report identity")
            .to_owned()
    });
    if reported != actual_jobs {
        return Err("report does not contain the four real job results".into());
    }
    let file = name("roles", "SUPERVISOR_FILE")?;
    let request = name("labels", "CollectReports")? as u8;
    let (_, admitted) = take_frame(&frames, &mut used, |frame| {
        frame.direction(file, supervisor) && frame.logical_label == request
    })?;
    if admitted.received >= call.emitted || admitted.value()? != json!({}) {
        return Err("report lacks its actual file request".into());
    }
    let reply = name("labels", "Reports")? as u8;
    let (_, delivered) = take_frame(&frames, &mut used, |frame| {
        frame.direction(supervisor, file) && frame.logical_label == reply
    })?;
    if delivered.sent <= at || delivered.value()? != *result {
        return Err("report file reply differs from actual effect".into());
    }
    let mut scored = scored_events.iter().collect::<Vec<_>>();
    for call in calls.values() {
        // The neutral simulator records collect_reports' declared default,
        // whereas its actual accepted request omits that optional argument.
        let scored_arguments = if call.tool == "collect_reports" {
            json!({"scope":"all_jobs"})
        } else {
            call.request["arguments"].clone()
        };
        call.take_score(&mut scored, &scored_arguments)?;
    }
    let business = [
        "RequestLatheA",
        "RequestLatheB",
        "RequestMillA",
        "RequestMillB",
        "LatheJobA",
        "LatheJobB",
        "MillJobA",
        "MillJobB",
        "LatheResult",
        "MillResult",
        "LatheDoneA",
        "LatheDoneB",
        "MillDoneA",
        "MillDoneB",
        "FileLatheResultA",
        "FileLatheResultB",
        "FileMillResultA",
        "FileMillResultB",
        "CollectReports",
        "Reports",
    ]
    .into_iter()
    .map(|label| name("labels", label).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if !scored.is_empty()
        || frames
            .iter()
            .any(|(id, frame)| business.contains(&frame.logical_label) && !used.contains(id))
    {
        return Err("unmatched manufacturing effect or business occurrence".into());
    }
    let return_labels = [
        name("labels", "ReturnLathe")? as u8,
        name("labels", "ReturnMill")? as u8,
    ];
    let returns = frames
        .values()
        .filter(|frame| return_labels.contains(&frame.logical_label))
        .count();
    if returns != 4 {
        return Err("incomplete machine return coverage".into());
    }
    let jobs = scored_events
        .iter()
        .filter(|event| event["tool"] == "do_job")
        .collect::<Vec<_>>();
    let mut overlaps = 0;
    for (index, left) in jobs.iter().enumerate() {
        for right in &jobs[index + 1..] {
            if left["result"]["machine"] != right["result"]["machine"]
                && left["timestamp"].as_f64().ok_or("job start time")?
                    < right["completed_at"]
                        .as_f64()
                        .ok_or("job completion time")?
                && right["timestamp"].as_f64().ok_or("job start time")?
                    < left["completed_at"].as_f64().ok_or("job completion time")?
            {
                overlaps += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"11E","domain_calls":calls.len(),"simulator_complete":true,"violations":0,
        "actual_jobs":4,"actual_reports":1,"matched_effect_frame_ids":used,
        "actual_typed_file_requests":file_requests+1,"actual_resource_endpoint_requests":resource_requests,
        "actual_resource_endpoint_returns":returns,"distinct_machine_overlap_pairs":overlaps,"dynamic_protocol":dynamic,
        "effect_scope":"unique committed file/resource requests, actual job replies and four-job report; EPF owns source execution and delegation"}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let evidence = crate::journal::Evidence::open(path)?;
    let baseline = evidence.audit("11E")?;
    let (rows, artifact) = (evidence.rows, evidence.projection);
    let dynamic = &baseline["dynamic_protocol"];
    let ids = baseline["matched_effect_frame_ids"]
        .as_array()
        .ok_or("manufacturing JSON ids")?
        .iter()
        .map(|id| id.as_u64().ok_or_else(|| "manufacturing frame id".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = crate::journal::json_frame_mutants(&rows, &ids, |changed| {
        crate::journal::check_effects(changed, &artifact, dynamic)
    })?;
    let first = *ids.first().ok_or("manufacturing frame")?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        let mut observations = 0;
        for row in &mut changed {
            let message = &mut row["message"];
            if message["type"] == "protocol_frame" && message["frame_id"] == first {
                message[field] = json!(
                    message[field].as_u64().ok_or("wire field")?
                        ^ if field == "lane" { 255 } else { 65535 }
                );
                observations += 1;
            }
        }
        if observations != 1 {
            return Err("manufacturing header mutation must change one actual acceptance".into());
        }
        let error = crate::journal::check_effects(&changed, &artifact, dynamic)
            .err()
            .ok_or("coherently altered manufacturing wire header accepted")?;
        checked.push(
            json!({"kind":format!("coherent_wrong_{field}"),"frame_id":first,"status":"rejected","error":error.to_string()}),
        );
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,"scope":"record mutations, not runtime attacks; unchanged source capture checked once"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
