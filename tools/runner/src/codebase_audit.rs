//! EPF source conformance and actual data/effect/result correspondence.
//! Observations are consumed once; no holder, phase or workflow is replayed.
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
    let mut names = value["modules"]
        .as_str()
        .ok_or("module arguments")?
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
        .filter(|(id, f)| !used.contains(*id) && matches(f)))
    .map_err(|e| {
        format!(
            "{e} at {}:{}",
            std::panic::Location::caller().file(),
            std::panic::Location::caller().line()
        )
    })?;
    used.insert(*id);
    Ok(frame)
}
fn prior<'a>(calls: &'a BTreeMap<u64, Call>, call: &Call, tool: &str) -> Result<&'a Value> {
    let earlier = calls
        .values()
        .filter(|p| {
            p.request["agent"] == call.request["agent"]
                && p.request["tool"] == tool
                && p.result.as_ref().is_some_and(|(at, _)| *at < call.emitted)
        })
        .max_by_key(|p| p.result.as_ref().map(|(at, _)| *at))
        .ok_or("missing actual completed input")?;
    Ok(earlier.result()?.1)
}
fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let fixture = &rows.first().ok_or("empty codebase evidence")?["message"];
    let task = fixture["task"].as_str().ok_or("codebase task")?;
    let participants = match task {
        "1E" => 3,
        "1M" => 5,
        "1H" => 7,
        _ => return Err("unknown codebase task".into()),
    };
    let compiled = &artifact["compiled"];
    let actors = fixture["fixtures"].as_array().ok_or("original inputs")?;
    if compiled["task"] != task
        || compiled["session"] != 20
        || actors.len() != participants
        || compiled["participants"]
            .as_object()
            .is_none_or(|p| p.len() != participants)
    {
        return Err("codebase participant or session differs".into());
    }
    let labels = crate::projection::frames(artifact, rows)?;
    let headers = crate::projection::contracts(compiled)?;
    let mut journal = crate::journal::read(rows, &labels, actors)?;
    journal.bind_receives(dynamic)?;
    let mut used = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut latest_tests = BTreeMap::new();
    let mut score_events = journal.score["events"]
        .as_array()
        .ok_or("original events")?
        .clone();
    let mut commits = Vec::new();
    let mut expected_returns = BTreeMap::<u64, usize>::new();
    let mut decisions = BTreeMap::<(usize, usize), (u64, u64, u64)>::new();
    for (index, row) in rows.iter().enumerate() {
        if row["message"]["type"] != "protocol_capture" {
            continue;
        }
        for (ordinal, record) in row["message"]["records"]
            .as_array()
            .ok_or("capture records")?
            .iter()
            .enumerate()
        {
            if record[1] != hibana::runtime::tap::RESOLVER_AUDIT {
                continue;
            }
            let causal = record[2].as_u64().ok_or("resolver actor/result")?;
            let packed = record[4].as_u64().ok_or("resolver site/id")?;
            if compiled["participants"]
                .as_object()
                .ok_or("participants")?
                .values()
                .any(|p| p["role"] == causal >> 8 && p["resolver"] == packed & 65535)
            {
                if record[3] != 20 {
                    return Err("resolver session differs".into());
                }
                decisions.insert(
                    (index, ordinal),
                    (causal >> 8, packed & 65535, causal & 255),
                );
            }
        }
    }
    let mut used_decisions = BTreeSet::new();
    for call in journal.calls.values() {
        let agent = call.request["agent"].as_str().ok_or("effect participant")?;
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        let declared = &compiled["participants"][agent];
        let role = declared["role"].as_u64().ok_or("native role")?;
        let file = declared["file"].as_u64().ok_or("file role")?;
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let assigned = one(actor["calls"]
            .as_array()
            .ok_or("assignments")?
            .iter()
            .filter(|a| a["tool"] == tool))?;
        let args_match = if tool == "commit_changes" {
            pair(args)? == pair(&assigned["arguments"])?
        } else {
            *args == assigned["arguments"]
        };
        if !args_match
            || (!seen.insert((agent, tool)) && (task != "1H" || tool == "design_feature"))
        {
            return Err("wrong or duplicate assigned work".into());
        }
        let (result_at, result) = call.result()?;
        if result["tool_name"] != tool
            || result["data"]["agent"] != agent
            || !result["success"].is_boolean()
            || (tool != "run_local_tests" && result["success"] != true)
        {
            return Err("actual codebase result differs".into());
        }
        let (request_label, reply_label) = match tool {
            "design_feature" => (1, 5),
            "implement_code" => (2, 6),
            "commit_changes" => (3, 7),
            "run_local_tests" => (4, 8),
            _ => return Err("unknown effect".into()),
        };
        take(&journal.frames, &mut used, |f| {
            f.direction(file, role)
                && f.logical_label == request_label
                && f.received.is_some_and(|at| at < call.emitted)
                && f.value().is_ok_and(|v| v == *args)
        })?;
        let reply = take(&journal.frames, &mut used, |f| {
            f.direction(role, file)
                && f.logical_label == reply_label
                && f.received.is_some_and(|at| at > result_at)
                && f.value().is_ok_and(|v| v == *result)
        })?;
        let resources = actor["modules"]
            .as_array()
            .ok_or("physical modules")?
            .iter()
            .map(|name| {
                compiled["resources"][name.as_str().ok_or("module name")?]["role"]
                    .as_u64()
                    .ok_or_else(|| "compiled module role".into())
            })
            .collect::<Result<Vec<_>>>()?;
        if resources.len() != 2
            || resources[0] == resources[1]
            || declared["resources"] != json!(resources)
        {
            return Err("two actual module bindings required".into());
        }
        match tool {
            "design_feature" => {
                if result["data"]["feature"] != args["feature_name"]
                    || !result["data"]["artifact"]["id"].is_string()
                    || !result["data"]["artifact"]["content"].is_string()
                {
                    return Err("missing actual design".into());
                }
            }
            "implement_code" => {
                let design = prior(&journal.calls, call, "design_feature")?;
                if result["data"]["feature"] != args["feature_name"]
                    || result["data"]["artifact"]["design"] != design["data"]["artifact"]
                {
                    return Err("implementation lacks its actual design".into());
                }
                let negative = journal
                    .calls
                    .values()
                    .filter(|p| {
                        p.request["agent"] == agent
                            && p.request["tool"] == "run_local_tests"
                            && p.result.as_ref().is_some_and(|(at, v)| {
                                *at < call.emitted && v["data"]["status"] == "tests_failed"
                            })
                    })
                    .max_by_key(|p| p.result.as_ref().map(|(at, _)| *at));
                let repair = if let Some(negative) = negative {
                    let value = negative.result()?.1;
                    json!({"test_id":value["data"]["test_id"],"tested_commit":value["data"]["input_snapshot"]["id"],"diagnostic":value["data"]["diagnostic"]})
                } else {
                    Value::Null
                };
                if result["data"]["artifact"]["repair"] != repair {
                    return Err("repair lacks its actual negative test input".into());
                }
            }
            "commit_changes" => {
                let source = prior(&journal.calls, call, "implement_code")?;
                if result["data"]["artifact"]["source"] != source["data"]["artifact"]
                    || result["data"]["feature"] != source["data"]["feature"]
                {
                    return Err("commit lacks its actual implementation".into());
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
                    *expected_returns.entry(resource).or_default() += 1;
                }
                commits.push((call, resources));
            }
            "run_local_tests" => {
                let committed = prior(&journal.calls, call, "commit_changes")?;
                if result["data"]["feature"] != args["feature_name"]
                    || result["data"]["input_snapshot"] != committed["data"]["artifact"]
                    || !result["data"]["test_id"].is_string()
                {
                    return Err("test lacks its actual committed source".into());
                }
                latest_tests.insert(agent, result["success"] == true);
                if task == "1H" {
                    let resolver = declared["resolver"]
                        .as_u64()
                        .ok_or("compiled test resolver")?;
                    let mut matched = 0;
                    for (id, &(actor, check, decision)) in &decisions {
                        if actor == role
                            && check == resolver
                            && result_at < id.0
                            && id.0 < reply.received()?
                        {
                            if decision != u64::from(result["success"] != true)
                                || !used_decisions.insert(*id)
                            {
                                return Err(
                                    "actual test verdict differs from resolver decision".into()
                                );
                            }
                            matched += 1;
                        }
                    }
                    if matched == 0 {
                        return Err("test result lacks its actual resolver decision".into());
                    }
                    for i in 0..2 {
                        let service = compiled["resources"]
                            [actor["modules"][i].as_str().ok_or("module")?]["service"]
                            .as_u64()
                            .ok_or("module service")?;
                        take(&journal.frames, &mut used, |f| {
                            f.direction(role, service)
                                && f.logical_label as u64 == resources[1 - i]
                                && f.received.is_some_and(|at| at > result_at)
                                && f.value().is_ok_and(|v| v == *result)
                        })?;
                    }
                }
            }
            _ => unreachable!(),
        }
        let index = score_events
            .iter()
            .position(|event| {
                event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == *args
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("score differs from actual effect")?;
        score_events.remove(index);
    }
    let passed = latest_tests.len() == participants && latest_tests.values().all(|v| *v);
    if !score_events.is_empty()
        || seen.len() != 4 * participants
        || latest_tests.len() != participants
        || journal.score["complete"] != passed
        || journal.score["violations"] != json!([])
        || used_decisions.len() != decisions.len()
    {
        return Err("incomplete original work or invalid score/decision evidence".into());
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
                .flat_map(|r| [r["role"].as_u64(), r["service"].as_u64()]),
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
            return Err("extra or unused codebase request/result".into());
        }
    }
    let mut returned = 0;
    for (name, binding) in compiled["resources"].as_object().ok_or("resources")? {
        let role = binding["role"].as_u64().ok_or("resource role")?;
        let count = journal
            .frames
            .values()
            .filter(|f| {
                f.received.is_some()
                    && f.identity["target"] == binding["service"]
                    && f.identity["frame_label"]
                        .as_u64()
                        .and_then(|v| u16::try_from(v).ok())
                        .and_then(|wire| headers.get(&wire))
                        .is_some_and(|h| h[4] == 1 && u64::from(h[5]) == role)
            })
            .count();
        if Some(&count) != expected_returns.get(&role) {
            return Err(format!("missing actual module returns: {name}").into());
        }
        returned += count;
    }
    let mut maximum = 0;
    for (first, resources) in &commits {
        let start = first.started()?.1;
        let mut overlap = 0;
        for (second, other) in &commits {
            if second.started()?.1 <= start && start < second.ended()?.1 {
                overlap += 1;
            }
            if first.request != second.request
                && first.started()?.1.max(second.started()?.1)
                    < first.ended()?.1.min(second.ended()?.1)
                && resources.iter().any(|r| other.contains(r))
            {
                return Err("actual shared-module commits overlap".into());
            }
        }
        maximum = maximum.max(overlap);
    }
    Ok(
        json!({"status":"passed","task":task,"kind":"direct_file_module_requests_and_actual_source","scored_benchmark":false,"participants":participants,"domain_calls":journal.calls.len(),"actual_resource_endpoint_requests":returned,"actual_resource_endpoint_returns":returned,"max_disjoint_commits":maximum,"actual_test_resolver_decisions":used_decisions.len(),"simulator_complete":journal.score["complete"],"violations":0,"guest_exits":participants,"dynamic_protocol":dynamic}),
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
            .ok_or("codebase directory")?
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
            .ok_or("codebase directory")?
            .join("projection.json")
    )?);
    audit["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    fs::create_dir(output)?;
    save(&output.join("audit.json"), &audit)?;
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("codebase inputs")?;
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
    let resolver_ids = projection["compiled"]["participants"]
        .as_object()
        .ok_or("participants")?
        .values()
        .filter_map(|p| p["resolver"].as_u64())
        .collect::<BTreeSet<_>>();
    for kind in [
        "wrong_effect_arguments",
        "wrong_effect_result",
        "false_score",
        "wrong_test_decision",
    ] {
        let mut changed = rows.clone();
        let mut changed_at = None;
        for (index, row) in changed.iter_mut().enumerate() {
            let runtime_record = row["direction"] == "runtime";
            let message = &mut row["message"];
            match kind {
                "wrong_effect_arguments" if runtime_record && message["type"] == "execute" => {
                    message["arguments"] = json!({"feature_name":"unadmitted_feature"});
                    changed_at = Some(index);
                }
                "wrong_effect_result" if message["type"] == "simulator_result" => {
                    message["result"]["data"] = json!({"forged":true});
                    changed_at = Some(index);
                }
                "false_score" if message["type"] == "score" => {
                    message["complete"] =
                        json!(!message["complete"].as_bool().ok_or("original score")?);
                    changed_at = Some(index);
                }
                "wrong_test_decision" if message["type"] == "protocol_capture" => {
                    if let Some(record) = message["records"]
                        .as_array_mut()
                        .ok_or("capture records")?
                        .iter_mut()
                        .find(|r| {
                            r[1] == hibana::runtime::tap::RESOLVER_AUDIT
                                && r[4]
                                    .as_u64()
                                    .is_some_and(|v| resolver_ids.contains(&(v & 65535)))
                        })
                    {
                        record[2] = json!(record[2].as_u64().ok_or("decision result")? ^ 1);
                        changed_at = Some(index);
                    }
                }
                _ => {}
            }
            if changed_at.is_some() {
                break;
            }
        }
        let Some(index) = changed_at else {
            if kind == "wrong_test_decision" && projection["compiled"]["task"] != "1H" {
                continue;
            }
            return Err(format!("no input for record mutation {kind}").into());
        };
        let error = check(&changed, &projection, &dynamic)
            .err()
            .ok_or_else(|| format!("accepted record mutation {kind}"))?;
        rejected
            .push(json!({"kind":kind,"record":index,"rejected":true,"error":error.to_string()}));
    }
    let result =
        json!({"status":"passed","record_mutations_rejected":rejected.len(),"mutations":rejected});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
