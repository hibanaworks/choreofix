//! EPF source conformance and actual request/material/effect/result agreement.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, one, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn pair(value: &Value) -> Result<Vec<&str>> {
    let mut names = value["sections"]
        .as_str()
        .ok_or("section arguments")?
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    names.sort_unstable();
    Ok(names)
}
#[track_caller]
fn take<'a>(
    frames: &'a BTreeMap<u64, Frame>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<&'a Frame> {
    let (id, frame) = one(frames
        .iter()
        .filter(|(id, frame)| !used.contains(*id) && matches(frame)))
    .map_err(|error| {
        format!(
            "{error} at {}:{}",
            std::panic::Location::caller().file(),
            std::panic::Location::caller().line()
        )
    })?;
    used.insert(*id);
    Ok(frame)
}
fn prior<'a>(calls: &'a BTreeMap<u64, Call>, call: &Call, tool: &str) -> Result<&'a Value> {
    let earlier = calls.values().filter(|p| {
        p.request["agent"] == call.request["agent"]
            && p.request["tool"] == tool
            && p.result.as_ref().is_some_and(|(at, _)| *at < call.emitted)
    });
    let p = one(earlier)?;
    Ok(p.result()?.1)
}
fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let fixture = &rows.first().ok_or("empty writing evidence")?["message"];
    let task = fixture["task"].as_str().ok_or("writing task")?;
    let participants = match task {
        "2E" => 3,
        "2M" => 5,
        "2H" => 7,
        _ => return Err("unknown writing task".into()),
    };
    let compiled = &artifact["compiled"];
    let actors = fixture["fixtures"].as_array().ok_or("writing inputs")?;
    if compiled["task"] != task
        || compiled["session"] != 18
        || actors.len() != participants
        || compiled["participants"]
            .as_object()
            .is_none_or(|p| p.len() != participants)
    {
        return Err("writing participant or session differs".into());
    }
    let labels = crate::projection::frames(artifact, rows)?;
    let headers = crate::projection::contracts(compiled)?;
    let mut journal = crate::journal::read(rows, &labels, actors)?;
    journal.bind_receives(dynamic)?;
    let mut used = BTreeSet::new();
    let mut completed = BTreeSet::new();
    let mut score_events = journal.score["events"]
        .as_array()
        .ok_or("original events")?
        .clone();
    let mut reviews_passed = true;
    let mut resource_requests = 0;
    let mut writes = Vec::new();
    for call in journal.calls.values() {
        let agent = call.request["agent"].as_str().ok_or("effect participant")?;
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        let declared = &compiled["participants"][agent];
        let role = declared["role"].as_u64().ok_or("actual native role")?;
        let file = declared["file"].as_u64().ok_or("actual file role")?;
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let assigned = one(actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .filter(|a| a["tool"] == tool))?;
        let args_match = if tool == "write_to_document" {
            pair(args)? == pair(&assigned["arguments"])?
        } else {
            *args == assigned["arguments"]
        };
        if !args_match || !completed.insert((agent, tool)) {
            return Err("wrong or duplicate original work".into());
        }
        let (result_at, result) = call.result()?;
        if result["tool_name"] != tool
            || result["data"]["agent"] != agent
            || !result["success"].is_boolean()
            || (tool != "review_own_work" && result["success"] != true)
        {
            return Err("writing actual effect result differs".into());
        }
        let (request_label, reply_label) = match tool {
            "read_literature" => (1, 5),
            "draft_content" => (2, 6),
            "write_to_document" => (3, 7),
            "review_own_work" => (4, 8),
            _ => return Err("unknown writing effect".into()),
        };
        take(&journal.frames, &mut used, |f| {
            f.direction(file, role)
                && f.logical_label == request_label
                && f.received.is_some_and(|at| at < call.emitted)
                && f.value().is_ok_and(|v| v == *args)
        })?;
        take(&journal.frames, &mut used, |f| {
            f.direction(role, file)
                && f.logical_label == reply_label
                && f.received.is_some_and(|at| at > result_at)
                && f.value().is_ok_and(|v| v == *result)
        })?;
        match tool {
            "read_literature" => {
                if result["data"]["topic"] != args["topic"]
                    || !result["data"]["artifact"]["id"].is_string()
                    || !result["data"]["artifact"]["content"].is_string()
                {
                    return Err("missing actual literature".into());
                }
            }
            "draft_content" => {
                let read = prior(&journal.calls, call, "read_literature")?;
                let topic = read["data"]["topic"]
                    .as_str()
                    .ok_or("actual literature topic")?;
                if result["data"]["section"] != args["section_name"]
                    || result["data"]["artifact"]["literature"]
                        != json!({topic:read["data"]["artifact"]})
                {
                    return Err("draft lacks its actual completed literature".into());
                }
            }
            "write_to_document" => {
                let draft = prior(&journal.calls, call, "draft_content")?;
                if result["data"]["artifact"]["draft"] != draft["data"]["artifact"]
                    || result["data"]["section"] != draft["data"]["section"]
                {
                    return Err("written material lacks its actual completed draft".into());
                }
                let resources = actor["sections"]
                    .as_array()
                    .ok_or("physical sections")?
                    .iter()
                    .map(|name| {
                        compiled["resources"][name.as_str().ok_or("section name")?]["role"]
                            .as_u64()
                            .ok_or_else(|| "compiled physical section".into())
                    })
                    .collect::<Result<Vec<_>>>()?;
                if resources.len() != 2
                    || resources[0] == resources[1]
                    || declared["resources"] != json!(resources)
                {
                    return Err("two distinct section bindings are required".into());
                }
                for &resource in &resources {
                    take(&journal.frames, &mut used, |f| {
                        f.direction(resource, role)
                            && f.logical_label as u64 == resource
                            && f.received.is_some_and(|at| at < call.emitted)
                            && f.value().is_ok_and(|v| v == *args)
                    })?;
                    take(&journal.frames, &mut used, |f| {
                        f.direction(role, resource)
                            && f.logical_label as u64 == resource
                            && f.received.is_some_and(|at| at > result_at)
                            && f.value().is_ok_and(|v| v == *result)
                    })?;
                    resource_requests += 1;
                }
                writes.push((call, resources));
            }
            "review_own_work" => {
                let written = prior(&journal.calls, call, "write_to_document")?;
                if result["data"]["section"] != args["section_name"]
                    || result["data"]["input_snapshot"] != written["data"]["artifact"]
                    || !result["data"]["review_id"].is_string()
                {
                    return Err("review does not name the actual written material".into());
                }
                reviews_passed &= result["success"] == true;
            }
            _ => unreachable!(),
        }
        let index = score_events
            .iter()
            .position(|event| {
                event["agent"] == call.request["agent"]
                    && event["tool"] == tool
                    && event["args"] == *args
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("original score differs from the actual effect")?;
        score_events.remove(index);
    }
    if !score_events.is_empty()
        || completed.len() != 4 * participants
        || journal.score["complete"] != reviews_passed
        || journal.score["violations"] != json!([])
    {
        return Err("incomplete original work or invalid outcome".into());
    }
    let business = compiled["participants"]
        .as_object()
        .ok_or("participants")?
        .values()
        .flat_map(|p| [p["role"].as_u64(), p["file"].as_u64()])
        .chain(
            compiled["resources"]
                .as_object()
                .ok_or("resources")?
                .values()
                .map(|r| r["role"].as_u64()),
        )
        .collect::<Option<BTreeSet<_>>>()
        .ok_or("compiled business role")?;
    for (id, frame) in &journal.frames {
        let wire = u16::try_from(frame.identity["frame_label"].as_u64().ok_or("wire label")?)?;
        let header = headers.get(&wire).ok_or("compiled frame")?;
        if header[4] == 0
            && business.contains(&u64::from(header[1]))
            && business.contains(&u64::from(header[2]))
            && !used.contains(id)
        {
            return Err("extra or unused writing request/result".into());
        }
    }
    // Completion additionally requires both actual returns for each section;
    // EPF already checked their custody. No owner/transfer state is replayed.
    for resource in compiled["resources"]
        .as_object()
        .ok_or("resources")?
        .values()
    {
        let returned = journal
            .frames
            .values()
            .filter(|frame| {
                let Some(wire) = frame.identity["frame_label"]
                    .as_u64()
                    .and_then(|n| u16::try_from(n).ok())
                else {
                    return false;
                };
                headers.get(&wire).is_some_and(|h| {
                    h[4] == 1
                        && resource["role"] == h[5]
                        && frame.identity["target"] == resource["service"]
                })
            })
            .count();
        if returned != 2 {
            return Err("missing actual final section returns".into());
        }
    }
    let mut maximum = 0;
    for (first, resources) in &writes {
        let start = first.started()?.1;
        let mut overlap = 0;
        for (second, other_resources) in &writes {
            if second.started()?.1 <= start && start < second.ended()?.1 {
                overlap += 1;
            }
            if first.request != second.request
                && first.started()?.1.max(second.started()?.1)
                    < first.ended()?.1.min(second.ended()?.1)
                && resources.iter().any(|r| other_resources.contains(r))
            {
                return Err("actual shared-section effects overlap".into());
            }
        }
        maximum = maximum.max(overlap);
    }
    Ok(
        json!({"status":"passed","task":task,"kind":"direct_file_section_requests_and_actual_material","scored_benchmark":false,"participants":participants,"domain_calls":journal.calls.len(),"actual_resource_endpoint_requests":resource_requests,"actual_resource_endpoint_returns":2*participants,"max_disjoint_writes":maximum,"simulator_complete":journal.score["complete"],"violations":0,"guest_exits":participants,"dynamic_protocol":dynamic}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let dynamic = crate::projection::check_dynamic(&projection, &rows)?;
    let mut result = check(&rows, &projection, &dynamic)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("writing directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let dynamic = crate::projection::check_dynamic(&projection, &rows)?;
    let mut audit = check(&rows, &projection, &dynamic)?;
    audit["source_events_sha256"] = json!(digest(path)?);
    audit["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("writing directory")?
            .join("projection.json")
    )?);
    audit["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    fs::create_dir(output)?;
    save(&output.join("audit.json"), &audit)?;
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("writing inputs")?;
    let labels = crate::projection::frames(&projection, &rows)?;
    let journal = crate::journal::read(&rows, &labels, actors)?;
    let ids = journal
        .frames
        .iter()
        .filter(|(_, f)| f.value().is_ok())
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    let mut rejected = crate::journal::json_frame_mutants(&rows, &ids, |changed| {
        check(changed, &projection, &dynamic)
    })?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(if field == "lane" { 255 } else { 65535 });
            }
        }
        let error = check(&changed, &projection, &dynamic)
            .err()
            .ok_or("coherent invalid wire header accepted")?;
        rejected.push(json!({"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result =
        json!({"status":"passed","record_mutations_rejected":rejected.len(),"mutations":rejected});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
