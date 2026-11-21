//! Independent saved-evidence checks. No experiment execution or task authority.
use crate::{
    evidence::{Result, digest},
    model, snapshot,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::{Component, Path},
};

fn check_resource_assignments(task: &str, fixture: &Value, metadata: &Value) -> Result<()> {
    let field = if task.starts_with('9') {
        "forks"
    } else if matches!(task, "1E" | "1M" | "1H") {
        "modules"
    } else if matches!(task, "2E" | "2M" | "2H") {
        "sections"
    } else {
        "resources"
    };
    for actor in fixture["fixtures"].as_array().ok_or("fixture actors")? {
        let mut actual = actor[field]
            .as_array()
            .ok_or("physical resource assignments")?
            .iter()
            .map(|v| v.as_str().ok_or("resource name"))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut expected = if matches!(task, "7H" | "8M" | "15E") {
            let mut names = BTreeSet::new();
            for operation in actor["operations"]
                .as_array()
                .ok_or("original operations")?
            {
                names.extend(crate::oracle::resources_for_call(
                    metadata,
                    actor["agent"].as_str().ok_or("participant")?,
                    &json!({"tool":operation["name"]}),
                )?);
            }
            names.into_iter().collect::<Vec<_>>()
        } else if field == "resources" && !matches!(task, "11M" | "14H" | "16M") {
            // The tools' physical resources are data, not a list of known task IDs.
            let mut names = Vec::new();
            for call in actor["calls"].as_array().ok_or("participant assignments")? {
                names.extend(crate::oracle::resources_for_call(
                    metadata,
                    actor["agent"].as_str().ok_or("participant")?,
                    call,
                )?);
            }
            names.sort_unstable();
            names.dedup();
            names
        } else {
            metadata["agent_resources"][actor["agent"].as_str().ok_or("actor")?]
                .as_array()
                .ok_or("original participant resources")?
                .iter()
                .map(|v| v.as_str().ok_or("resource name"))
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        actual.sort_unstable();
        expected.sort_unstable();
        if actual != expected {
            return Err("physical resource assignment changed".into());
        }
    }
    Ok(())
}

fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn records(path: &Path) -> Result<Vec<Value>> {
    BufReader::new(fs::File::open(path)?)
        .lines()
        .map(|v| Ok(serde_json::from_str(&v?)?))
        .collect()
}
fn unique<'a>(rows: &'a [Value], direction: &str, method: &str) -> Result<&'a Value> {
    let values: Vec<_> = rows
        .iter()
        .filter(|v| v["direction"] == direction && v["message"]["method"] == method)
        .collect();
    if values.len() != 1 {
        return Err(format!("expected one {direction} {method}").into());
    }
    Ok(values[0])
}

fn reply<'a>(rows: &'a [Value], request: &Value) -> Result<&'a Value> {
    let id = &request["message"]["id"];
    if id.is_null() {
        return Err("RPC request has no ID".into());
    }
    let replies: Vec<_> = rows
        .iter()
        .filter(|row| {
            row["direction"] == "receive"
                && row["message"]["id"] == *id
                && row["message"].get("method").is_none()
        })
        .collect();
    if replies.len() != 1 {
        return Err("expected one reply to the recorded RPC request".into());
    }
    Ok(&replies[0]["message"]["result"])
}
fn tokens(value: &Value) -> Result<u64> {
    let input = value["inputTokens"].as_u64().ok_or("input tokens")?;
    let output = value["outputTokens"].as_u64().ok_or("output tokens")?;
    let total = value["totalTokens"].as_u64().ok_or("inclusive tokens")?;
    if input == 0 || output == 0 || total != input.checked_add(output).ok_or("token overflow")? {
        return Err("invalid inclusive token accounting".into());
    }
    Ok(total)
}

pub(crate) fn shell_exchanges(
    rows: &[Value],
    expected: &Value,
    can_collect: bool,
) -> Result<Vec<(String, Value)>> {
    // Each guest command occupies one position in submission order, even if its
    // terminal output is later collected through another model RPC.
    let mut commands = Vec::<(&Value, Option<Value>)>::new();
    let mut interrupted = BTreeSet::<usize>::new();
    let mut active = BTreeMap::<String, (&Value, Option<usize>, Option<&Value>)>::new();
    let mut rpc_ids = BTreeSet::new();
    let mut item_ids = BTreeSet::new();
    for row in rows {
        let message = &row["message"];
        if row["direction"] == "send" && message.get("method").is_none() {
            let rpc = message["id"].to_string();
            if let Some((_, _, sent)) = active.get_mut(&rpc) {
                if sent.replace(&message["result"]).is_some() {
                    return Err("duplicate shell response".into());
                }
            } else if rpc_ids.contains(&rpc) {
                return Err("reply sent after model closed shell call".into());
            }
            continue;
        }
        if row["direction"] != "receive" {
            continue;
        }
        model::check_model_item(message)?;
        let params = &message["params"];
        if message["method"] == "item/tool/call" {
            let rpc = message["id"].to_string();
            let item = params["callId"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("missing tool callId")?;
            if params["tool"] != "shell" || !rpc_ids.insert(rpc.clone()) || !item_ids.insert(item) {
                return Err("duplicate or unexpected model tool".into());
            }
            let args = params["arguments"].as_object().ok_or("shell arguments")?;
            let ordinal = if can_collect && args.is_empty() {
                interrupted.pop_first()
            } else if args.len() == 1
                && let Some(command) = params["arguments"]["command"].as_str()
            {
                if command.len() <= 4096 {
                    commands.push((params, None));
                    Some(commands.len() - 1)
                } else {
                    None
                }
            } else {
                return Err("unexpected shell arguments".into());
            };
            active.insert(rpc, (params, ordinal, None));
        } else if message["method"] == "item/completed"
            && params["item"]["type"] == "dynamicToolCall"
        {
            let rpc = active
                .iter()
                .find(|(_, (request, _, _))| model::same_tool_call(request, params))
                .map(|(id, _)| id.clone())
                .ok_or("model completed an unknown shell call")?;
            let (request, ordinal, sent) = active.remove(&rpc).unwrap();
            let cancelled = model::cancelled_tool(params);
            if cancelled {
                if let Some(i) = ordinal {
                    interrupted.insert(i);
                }
            } else {
                model::check_tool_completion(
                    request,
                    sent.ok_or("shell call closed before its reply")?,
                    params,
                )?;
            }
            if let Some(sent) = sent {
                if sent["success"] != true {
                    return Err("failed shell dispatch".into());
                }
                let items = sent["contentItems"].as_array().ok_or("tool reply")?;
                let collect = request["arguments"]
                    .as_object()
                    .is_some_and(|a| a.is_empty());
                let recovered = collect && ordinal.is_some();
                if items.len() != if recovered { 2 } else { 1 }
                    || items.iter().any(|item| item["type"] != "inputText")
                {
                    return Err("tool reply shape".into());
                }
                let response: Value =
                    serde_json::from_str(items[0]["text"].as_str().ok_or("tool text")?)?;
                if let Some(i) = ordinal {
                    let (original, recorded) = &mut commands[i];
                    if recovered {
                        let origin: Value = serde_json::from_str(
                            items[1]["text"]
                                .as_str()
                                .ok_or("original command receipt")?,
                        )?;
                        if origin
                            != json!({"original_call":original["callId"],"command":original["arguments"]["command"]})
                        {
                            return Err("terminal output names a different original call".into());
                        }
                    }
                    if recorded
                        .as_ref()
                        .is_some_and(|previous| *previous != response)
                    {
                        return Err("terminal output changed between deliveries".into());
                    }
                    *recorded = Some(response);
                } else {
                    let expected = if collect {
                        json!({"status":"no_interrupted_command"})
                    } else {
                        json!({"status":"script_too_long","limit_utf8_bytes":4096})
                    };
                    if response != expected {
                        return Err("unexpected local shell response".into());
                    }
                }
            }
        }
    }
    if !active.is_empty()
        || !interrupted.is_empty()
        || *expected != rpc_ids.len()
        || rpc_ids.is_empty()
    {
        return Err("incomplete tool exchanges".into());
    }
    commands
        .into_iter()
        .map(|(request, out)| {
            Ok((
                request["arguments"]["command"].as_str().unwrap().to_owned(),
                out.ok_or("missing terminal output")?,
            ))
        })
        .collect()
}

fn participant(root: &Path, agent: &str, runtime: &[Value]) -> Result<Value> {
    let directory = root.join(agent);
    let report = load(&directory.join("participant.json"))?;
    if report["status"] != "completed"
        || !report["error"].is_null()
        || report["requested_model"] != model::MODEL
        || report["returned_model"] != model::MODEL
        || report["requested_effort"] != model::EFFORT
        || report["returned_effort"] != model::EFFORT
        || report["raw_events_sha256"] != digest(&directory.join("events.jsonl"))?
    {
        return Err("failed or altered participant report".into());
    }
    let rows = records(&directory.join("events.jsonl"))?;
    let thread_request = unique(&rows, "send", "thread/start")?;
    let thread = &thread_request["message"]["params"];
    let instructions = fs::read_to_string(root.join(format!("{agent}-prompt.txt")))?;
    if thread["model"] != model::MODEL
        || thread["allowProviderModelFallback"] != false
        || thread["config"]["model_reasoning_effort"] != model::EFFORT
        || thread["config"]["features.shell_tool"] != false
        || thread["config"]["features.multi_agent"] != false
        || thread["baseInstructions"]
            != format!(
                "{instructions}\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response."
            )
        || thread["dynamicTools"]
            .as_array()
            .is_none_or(|a| a.len() != 1 || a[0]["name"] != "shell")
    {
        return Err("model request settings or prompt differ".into());
    }
    let metadata = reply(&rows, thread_request)?;
    if metadata["model"] != model::MODEL || metadata["reasoningEffort"] != model::EFFORT {
        return Err("actual model/effort not proven".into());
    }
    let turn = unique(&rows, "send", "turn/start")?;
    if turn["message"]["params"]["model"] != model::MODEL
        || turn["message"]["params"]["effort"] != model::EFFORT
    {
        return Err("turn settings differ".into());
    }
    let completed = unique(&rows, "receive", "turn/completed")?;
    if completed["message"]["params"]["turn"]["status"] != "completed"
        || !completed["message"]["params"]["turn"]["error"].is_null()
    {
        return Err("turn did not complete".into());
    }
    let usage = rows
        .iter()
        .rev()
        .find(|v| {
            v["direction"] == "receive" && v["message"]["method"] == "thread/tokenUsage/updated"
        })
        .ok_or("missing raw usage")?;
    let usage = &usage["message"]["params"]["tokenUsage"]["total"];
    if *usage != report["usage"] {
        return Err("usage differs from raw event".into());
    }
    let total = tokens(usage)?;
    let exchanges = shell_exchanges(
        &rows,
        &report["tool_calls"],
        thread["dynamicTools"][0]["inputSchema"]
            .get("required")
            .is_none(),
    )?;
    let mut commands = BTreeMap::new();
    let mut delivered = Vec::new();
    let mut exited = false;
    for row in runtime {
        let message = &row["message"];
        if message["agent"] != agent {
            continue;
        }
        if row["direction"] == "to_runtime" && message["type"] == "command" {
            if exited
                || commands
                    .insert(message["id"].to_string(), delivered.len())
                    .is_some()
            {
                return Err("duplicate or after-exit command".into());
            }
            delivered.push((
                message["command"]
                    .as_str()
                    .ok_or("runtime command")?
                    .to_owned(),
                Value::Null,
            ));
        } else if row["direction"] == "runtime"
            && ["terminal", "exit", "terminal_closed"]
                .contains(&message["type"].as_str().unwrap_or(""))
            && !message["id"].is_null()
        {
            let position = commands
                .remove(&message["id"].to_string())
                .ok_or("terminal reply without command")?;
            let reply = match message["type"].as_str() {
                Some("terminal") => json!({"stdout":message["stdout"]}),
                Some("exit") => {
                    exited = true;
                    json!({"stdout":message["stdout"],"exit_status":message["status"]})
                }
                _ => json!({"status":"guest_closed"}),
            };
            delivered[position].1 = reply;
        }
    }
    if !commands.is_empty() || !exited || delivered != exchanges {
        return Err("model commands/replies differ from actual guest transport".into());
    }
    Ok(
        json!({"participant":agent,"inclusive_tokens":total,"usage":usage,"tool_calls":report["tool_calls"],
        "turn_start_unix_ns":turn["unix_ns"],"turn_end_unix_ns":completed["unix_ns"]}),
    )
}

pub fn run(root: &Path) -> Result<Value> {
    let result = load(&root.join("result.json"))?;
    let versions = load(&root.join("versions.json"))?;
    let task = result["task"].as_str().ok_or("task")?;
    if result["versions"] != versions
        || !result["error"].is_null()
        || result["scored_benchmark"] != false
        || result["events_sha256"] != digest(&root.join("events.jsonl"))?
    {
        return Err("failed or altered case result".into());
    }
    let binaries = versions["binaries"].as_object().ok_or("binary hashes")?;
    let required = ["runtime", "wash.wasm", "runner"];
    if binaries.len() != required.len() || required.iter().any(|name| !binaries.contains_key(*name))
    {
        return Err("missing or unexpected executable in the evidence bundle".into());
    }
    for (name, hash) in binaries {
        if name.contains('/') || json!(digest(&root.join(name))?) != *hash {
            return Err("binary differs".into());
        }
    }
    let manifest = load(&root.join("source-manifest.json"))?;
    if versions["source_manifest_sha256"] != digest(&root.join("source-manifest.json"))? {
        return Err("source manifest hash differs".into());
    }
    for (name, hash) in manifest.as_object().ok_or("source manifest")? {
        if Path::new(name)
            .components()
            .any(|v| !matches!(v, Component::Normal(_)))
            || json!(digest(&root.join(name))?) != *hash
        {
            return Err(format!("source differs: {name}").into());
        }
    }
    let benchmark = choreofix_inputs::verify_benchmark(&root.join("source"))?;
    let receipt = load(&root.join("build-receipt.json"))?;
    let original = load(&root.join("input-build-receipt.json"))?;
    let input_hash = digest(&root.join("input-build-receipt.json"))?;
    let source_hash = digest(&root.join("build-source.sha256"))?;
    let mut normalized = original.clone();
    normalized["source_manifest"] = json!("build-source.sha256");
    normalized["input_receipt_sha256"] = json!(input_hash);
    if versions["format_version"] != 5
        || versions["runtime_build_receipt"] != receipt
        || versions["build_receipt_sha256"] != digest(&root.join("build-receipt.json"))?
        || versions["input_build_receipt_sha256"] != input_hash
        || versions["runtime_source_manifest_sha256"] != source_hash
        || original["source_manifest_sha256"] != source_hash
        || original["source_unchanged"] != true
        || !original["source_git"].is_object()
        || versions["binaries"]["runtime"] != original["binary_sha256"]
        || receipt != normalized
    {
        return Err("runtime, source or build receipt differs".into());
    }
    snapshot::runtime_sources(root, &root.join("build-source.sha256"))?;
    let rows = records(&root.join("events.jsonl"))?;
    let fixture = &rows.first().ok_or("no runtime evidence")?["message"];
    let metadata = load(
        &root
            .join("source/benchmark/descriptions")
            .join(task)
            .join("metadata.json"),
    )?;
    let config = load(&root.join("source/experiment.json"))?;
    let actual_agents: Vec<_> = fixture["fixtures"]
        .as_array()
        .ok_or("fixture actors")?
        .iter()
        .map(|a| a["agent"].clone())
        .collect();
    if json!(actual_agents) != metadata["agents"]
        || fixture["task"] != result["task"]
        || fixture["depth"] != result["depth"]
        || fixture["seed"] != config["python_hash_seed"]
        || fixture["delay_multiplier"] != config["domain_tool_delay_multiplier"]
        || fixture["simulator_contract"] != benchmark["simulator_contract"]
    {
        return Err("task identity, depth or participant set differs from original inputs".into());
    }
    check_resource_assignments(task, fixture, &metadata)?;
    if matches!(task, "7H" | "8M" | "15E" | "11M" | "14H" | "16M") {
        let schemas = load(&root.join(format!("source/benchmark/descriptions/{task}/tools.json")))?;
        if fixture["tool_schemas"] != schemas {
            return Err("tool schemas differ from the common task".into());
        }
        let tools = schemas.as_array().ok_or("tool schemas")?;
        for actor in fixture["fixtures"].as_array().ok_or("fixture actors")? {
            let expected = tools
                .iter()
                .map(|tool| &tool["function"])
                .filter(|tool| {
                    tool["agent_ids"]
                        .as_array()
                        .is_some_and(|agents| agents.contains(&actor["agent"]))
                })
                .collect::<Vec<_>>();
            if (expected.is_empty() && !(task == "16M" && actor["agent"] == "BUILD_MASTER"))
                || actor["operations"] != json!(expected)
            {
                return Err("participant operations differ from the common task".into());
            }
        }
    }
    if fixture["adapter"] != "rust_upstream_api"
        || fixture["test_delays"] != json!({})
        || fixture["delay_multiplier"] != 1.0
        || fixture["failure_overrides"] != json!([])
    {
        return Err("model fixture settings differ".into());
    }
    if versions["model_connection"]["model"] != model::MODEL
        || versions["model_connection"]["effort"] != model::EFFORT
        || versions["model_connection"]["client_version"]
            .as_str()
            .is_none_or(str::is_empty)
        || versions["model_connection"]["client_sha256"]
            .as_str()
            .is_none_or(|hash| {
                hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
    {
        return Err("missing or inconsistent model client identity".into());
    }
    let mut participants = Vec::new();
    for actor in fixture["fixtures"].as_array().ok_or("fixture actors")? {
        let expected = crate::prompt(
            &root.join("source"),
            fixture["task"].as_str().ok_or("task")?,
            actor,
        )?;
        if fs::read_to_string(root.join(format!(
            "{}-prompt.txt",
            actor["agent"].as_str().ok_or("actor")?
        )))? != expected
        {
            return Err("saved prompt differs from frozen original inputs and interface".into());
        }
        participants.push(participant(
            root,
            actor["agent"].as_str().ok_or("actor")?,
            &rows,
        )?);
    }
    let mut times = Vec::new();
    for p in &participants {
        let start: u128 = p["turn_start_unix_ns"]
            .as_str()
            .ok_or("turn start time")?
            .parse()?;
        let end: u128 = p["turn_end_unix_ns"]
            .as_str()
            .ok_or("turn end time")?
            .parse()?;
        if start >= end {
            return Err("invalid model interval".into());
        }
        times.push((start, 1i32));
        times.push((end, -1i32));
    }
    times.sort();
    let (mut active, mut maximum) = (0, 0);
    for (_, change) in times {
        active += change;
        maximum = maximum.max(active);
    }
    let fixture_task = fixture["task"].clone();
    // Participant checks have consumed this parsed history. The protocol
    // auditor owns its own parse; do not retain both copies during EPF search.
    drop(rows);
    let protocol = match task {
        "1E" | "1M" | "1H" => crate::codebase_audit::audit(&root.join("events.jsonl"))?,
        "2E" | "2M" | "2H" => crate::writing_audit::audit(&root.join("events.jsonl"))?,
        "3E" => crate::report_audit::audit(&root.join("events.jsonl"))?,
        "3M" => crate::research_paper_audit::audit(&root.join("events.jsonl"))?,
        "3H" => crate::survey_paper_audit::audit(&root.join("events.jsonl"))?,
        "4E" => crate::coordination_audit::audit(&root.join("events.jsonl"))?,
        "4M" => crate::refactor_audit::audit(&root.join("events.jsonl"))?,
        "13E" => crate::lab_audit::audit(&root.join("events.jsonl"))?,
        "13M" => crate::lab_quality_audit::audit(&root.join("events.jsonl"))?,
        "13H" => crate::lab_cascading_audit::audit(&root.join("events.jsonl"))?,
        "14E" => crate::drug_discovery_audit::audit(&root.join("events.jsonl"))?,
        "14M" => crate::regulatory_discovery_audit::audit(&root.join("events.jsonl"))?,
        "14H" => crate::multi_round_discovery_audit::audit(&root.join("events.jsonl"))?,
        "5E" => crate::medical_audit::audit(&root.join("events.jsonl"))?,
        "5M" => crate::multi_specialty_audit::audit(&root.join("events.jsonl"))?,
        "5H" => crate::critical_care_audit::audit(&root.join("events.jsonl"))?,
        "6E" => crate::review_audit::audit(&root.join("events.jsonl"))?,
        "6M" => crate::feature_delivery_audit::audit(&root.join("events.jsonl"))?,
        "6H" => crate::staged_integration_audit::audit(&root.join("events.jsonl"))?,
        "7E" => crate::document_audit::audit(&root.join("events.jsonl"))?,
        "7M" => crate::figure_document_audit::audit(&root.join("events.jsonl"))?,
        "7H" => crate::large_document_audit::audit(&root.join("events.jsonl"))?,
        "10E" => crate::build_audit::audit(&root.join("events.jsonl"))?,
        "11M" => crate::manufacturing_rework_audit::audit(&root.join("events.jsonl"))?,
        "11E" => crate::manufacturing_audit::audit(&root.join("events.jsonl"))?,
        "12E" => crate::kitchen_audit::audit(&root.join("events.jsonl"))?,
        "10M" => crate::cascading_build_audit::audit(&root.join("events.jsonl"))?,
        "8E" => crate::api_audit::audit(&root.join("events.jsonl"))?,
        "8M" => crate::full_stack_api_audit::audit(&root.join("events.jsonl"))?,
        "15E" => crate::wafer_audit::audit(&root.join("events.jsonl"))?,
        "16E" => crate::cicd_audit::audit(&root.join("events.jsonl"))?,
        "16M" => crate::cicd_pipeline_audit::audit(&root.join("events.jsonl"))?,
        "9E" | "9M" | "9H" => crate::philosophers_audit::audit(&root.join("events.jsonl"))?,
        _ => return Err("task has no delegated WASI implementation".into()),
    };
    if protocol["compiled_projection_sha256"] != versions["compiled_projection_sha256"]
        || protocol["runtime_binary_sha256"] != versions["binaries"]["runtime"]
    {
        return Err("projection differs from the frozen executable and artifact".into());
    }
    let runtime = result["outcomes"]
        .as_array()
        .ok_or("run outcomes")?
        .iter()
        .filter(|o| o["kind"] == "guest_runtime")
        .collect::<Vec<_>>();
    if runtime.len() != 1 {
        return Err("expected one actual guest runtime outcome".into());
    }
    let runtime = &runtime[0]["result"];
    if protocol["status"] != "passed"
        || protocol["task"] != fixture_task
        || protocol["simulator_complete"] != runtime["score"]["complete"]
        || runtime["score"]["complete"].as_bool().is_none()
        || protocol["violations"] != 0
        || runtime["score"]["violations"] != json!([])
        || runtime["protocol"]["status"] != "complete"
        || runtime["runtime_exit_code"] != 0
        || runtime["backend_errors"] != json!([])
    {
        return Err("protocol verification differs from the actual task outcome".into());
    }
    let total: u64 = participants
        .iter()
        .try_fold(0u64, |total, v| {
            total.checked_add(v["inclusive_tokens"].as_u64()?)
        })
        .ok_or("inclusive usage overflow or missing usage")?;
    Ok(
        json!({"status":"passed","task":fixture_task,"kind":"unscored_direct_wasi_model_preflight",
        "scored_benchmark":false,"participants":participants,"participant_turns":participants.len(),"max_simultaneous_model_turns":maximum,
        "inclusive_tokens":total,"protocol":protocol,"events_sha256":result["events_sha256"],
        "simulator_contract":benchmark["simulator_contract"],
        "simulator_manifest_sha256":digest(&root.join("source/benchmark/simulator-manifest.json"))?,
        "auditor_sha256":digest(&std::env::current_exe()?)?}),
    )
}

pub fn check_mutants(root: &Path, output: &Path) -> Result<Value> {
    let baseline = run(root)?;
    fs::create_dir(output)?;
    if output.canonicalize()?.starts_with(root.canonicalize()?) {
        return Err("mutant output must be outside original evidence".into());
    }
    fn copy(from: &Path, to: &Path) -> Result<()> {
        fs::create_dir(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let target = to.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy(&entry.path(), &target)?;
            } else {
                fs::copy(entry.path(), target)?;
            }
        }
        Ok(())
    }
    let actors = records(&root.join("events.jsonl"))?;
    let agent = actors[0]["message"]["fixtures"][0]["agent"]
        .as_str()
        .ok_or("first actor")?;
    let mut checked = Vec::new();
    for kind in [
        "forged_model",
        "incorrect_inclusive_usage",
        "forged_command",
        "missing_tool_reply",
        "false_simulator_score",
        "changed_upstream_source",
        "unqualified_runtime",
        "missing_guest_artifact",
        "wrong_participant_set",
        "simulator_contract_mismatch",
        "artificial_delay_in_model_fixture",
    ] {
        let case = output.join(kind);
        copy(root, &case)?;
        let events = case.join(agent).join("events.jsonl");
        let mut rows = records(&events)?;
        let mut participant = load(&case.join(agent).join("participant.json"))?;
        let mut result = load(&case.join("result.json"))?;
        match kind {
            "forged_model" => {
                participant["requested_model"] = json!("other");
                participant["returned_model"] = json!("other");
                let thread_id = unique(&rows, "send", "thread/start")?["message"]["id"].clone();
                for row in &mut rows {
                    let received = row["direction"] == "receive";
                    let message = &mut row["message"];
                    if ["thread/start", "turn/start"]
                        .contains(&message["method"].as_str().unwrap_or(""))
                    {
                        message["params"]["model"] = json!("other");
                    }
                    if received && message["id"] == thread_id && message.get("method").is_none() {
                        message["result"]["model"] = json!("other");
                    }
                }
            }
            "incorrect_inclusive_usage" => {
                let total = participant["usage"]["totalTokens"]
                    .as_u64()
                    .ok_or("usage")?
                    + 1;
                participant["usage"]["totalTokens"] = json!(total);
                for row in &mut rows {
                    if row["message"]["method"] == "thread/tokenUsage/updated" {
                        row["message"]["params"]["tokenUsage"]["total"]["totalTokens"] =
                            json!(total);
                    }
                }
            }
            "forged_command" => {
                rows.iter_mut()
                    .find(|r| r["message"]["method"] == "item/tool/call")
                    .ok_or("tool call")?["message"]["params"]["arguments"]["command"] =
                    json!("echo forged");
            }
            "missing_tool_reply" => {
                let id = rows
                    .iter()
                    .find(|r| r["message"]["method"] == "item/tool/call")
                    .ok_or("tool call")?["message"]["id"]
                    .clone();
                // Server call IDs and client request IDs may overlap. Remove
                // only the tool response, preserving client requests with that ID.
                let before = rows.len();
                rows.retain(|r| {
                    !(r["direction"] == "send"
                        && r["message"]["id"] == id
                        && r["message"].get("method").is_none())
                });
                if before - rows.len() != 1 {
                    return Err("expected one actual tool response to remove".into());
                }
            }
            "false_simulator_score" => {
                let mut runtime = records(&case.join("events.jsonl"))?;
                let score = runtime
                    .iter_mut()
                    .find(|r| r["direction"] == "simulator" && r["message"]["type"] == "score")
                    .ok_or("score")?;
                let complete = score["message"]["complete"]
                    .as_bool()
                    .ok_or("score outcome")?;
                score["message"]["complete"] = json!(!complete);
                fs::write(
                    case.join("events.jsonl"),
                    runtime.iter().map(|r| format!("{r}\n")).collect::<String>(),
                )?;
                result["events_sha256"] = json!(digest(&case.join("events.jsonl"))?);
            }
            "changed_upstream_source" => {
                let mut manifest = load(&case.join("source-manifest.json"))?;
                let relative = format!(
                    "benchmark/descriptions/{}/description.md",
                    result["task"].as_str().ok_or("task")?
                );
                fs::write(case.join("source").join(&relative), "changed input\n")?;
                manifest[format!("source/{relative}")] =
                    json!(digest(&case.join("source").join(&relative))?);
                crate::evidence::save(&case.join("source-manifest.json"), &manifest)?;
                let mut versions = load(&case.join("versions.json"))?;
                versions["source_manifest_sha256"] =
                    json!(digest(&case.join("source-manifest.json"))?);
                crate::evidence::save(&case.join("versions.json"), &versions)?;
                result["versions"] = versions;
            }
            "unqualified_runtime" => {
                let path = case.join("runtime");
                let mut bytes = fs::read(&path)?;
                bytes[0] ^= 1;
                fs::write(&path, bytes)?;
                let mut versions = load(&case.join("versions.json"))?;
                versions["binaries"]["runtime"] = json!(digest(&path)?);
                crate::evidence::save(&case.join("versions.json"), &versions)?;
                result["versions"] = versions;
            }
            "missing_guest_artifact" => {
                fs::remove_file(case.join("wash.wasm"))?;
                let mut versions = load(&case.join("versions.json"))?;
                versions["binaries"]
                    .as_object_mut()
                    .ok_or("binary hashes")?
                    .remove("wash.wasm");
                crate::evidence::save(&case.join("versions.json"), &versions)?;
                result["versions"] = versions;
            }
            "wrong_participant_set" | "simulator_contract_mismatch" => {
                let mut runtime = records(&case.join("events.jsonl"))?;
                if kind == "wrong_participant_set" {
                    runtime[0]["message"]["fixtures"]
                        .as_array_mut()
                        .ok_or("actors")?
                        .pop();
                } else {
                    runtime[0]["message"]["simulator_contract"] = json!("unmatched-world-contract");
                }
                fs::write(
                    case.join("events.jsonl"),
                    runtime.iter().map(|r| format!("{r}\n")).collect::<String>(),
                )?;
                result["events_sha256"] = json!(digest(&case.join("events.jsonl"))?);
            }
            "artificial_delay_in_model_fixture" => {
                let mut runtime = records(&case.join("events.jsonl"))?;
                runtime[0]["message"]["test_delays"] = json!({"synthetic_barrier":true});
                fs::write(
                    case.join("events.jsonl"),
                    runtime
                        .iter()
                        .map(|row| format!("{row}\n"))
                        .collect::<String>(),
                )?;
                result["events_sha256"] = json!(digest(&case.join("events.jsonl"))?);
            }
            _ => unreachable!(),
        }
        fs::write(
            &events,
            rows.iter().map(|r| format!("{r}\n")).collect::<String>(),
        )?;
        participant["raw_events_sha256"] = json!(digest(&events)?);
        crate::evidence::save(&case.join(agent).join("participant.json"), &participant)?;
        crate::evidence::save(&case.join("result.json"), &result)?;
        let error = run(&case)
            .expect_err("altered evidence passed independent audit")
            .to_string();
        checked.push(json!({"mutation":kind,"status":"rejected","reason":error,"outer_hashes_recomputed":true}));
        fs::remove_dir_all(case)?;
    }
    let report = json!({"status":"passed","baseline":baseline,"checked":checked,
        "auditor_sha256":digest(&std::env::current_exe()?)?,
        "original_events_sha256":digest(&root.join("events.jsonl"))?});
    crate::evidence::save(&output.join("report.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_guest_output_must_reach_the_same_model_call() {
        let params = json!({"tool":"shell","arguments":{"command":"cat /service/material"},"threadId":"t","turnId":"u","callId":"exec"});
        let request = json!({"direction":"receive","message":{"method":"item/tool/call","id":15,"params":params}});
        let items = json!([{"type":"inputText","text":"{\"stdout\":\"manuscript\"}"}]);
        let reply = json!({"direction":"send","message":{"id":15,"result":{"success":true,"contentItems":items}}});
        let ack = json!({"direction":"receive","message":{"method":"item/completed","params":{"threadId":"t","turnId":"u","item":{"type":"dynamicToolCall","id":"exec","tool":"shell","arguments":params["arguments"],"status":"completed","success":true,"contentItems":items}}}});
        let rows = [request, reply, ack];
        let count = json!(1);
        assert_eq!(
            shell_exchanges(&rows, &count, true).unwrap(),
            vec![(
                "cat /service/material".to_owned(),
                json!({"stdout":"manuscript"})
            )]
        );
        assert!(shell_exchanges(&rows[..2], &count, true).is_err());
        let mut early = rows.clone();
        early.swap(1, 2);
        assert!(shell_exchanges(&early, &count, true).is_err());
        for (field, value) in [
            ("contentItems", json!([])),
            ("id", json!("other")),
            ("arguments", json!({"command":"other"})),
        ] {
            let mut changed = rows.clone();
            changed[2]["message"]["params"]["item"][field] = value;
            assert!(shell_exchanges(&changed, &count, true).is_err(), "{field}");
        }
    }

    #[test]
    fn physical_resources_come_from_all_assigned_tools_without_a_task_allowlist() {
        let metadata: Value = serde_json::from_str(include_str!(
            "../../../benchmark/descriptions/5M/metadata.json"
        ))
        .unwrap();
        let fixture = json!({"fixtures":[
            {"agent":"ER_DOCTOR","resources":["CONSULTATION_ROOM"],"calls":[
                {"tool":"assess_patient"},{"tool":"order_tests"},{"tool":"conduct_case_conference"}]},
            {"agent":"RADIOLOGIST","resources":["MRI","CT_SCANNER"],"calls":[
                {"tool":"perform_imaging"},{"tool":"interpret_imaging"}]}
        ]});
        check_resource_assignments("5M", &fixture, &metadata).unwrap();
        let mut missing = fixture.clone();
        missing["fixtures"][1]["resources"] = json!(["CT_SCANNER"]);
        assert!(check_resource_assignments("5M", &missing, &metadata).is_err());
        let mut replaced = fixture.clone();
        replaced["fixtures"][0]["resources"] = json!(["ICU_BEDS"]);
        assert!(check_resource_assignments("5M", &replaced, &metadata).is_err());
        let mut duplicated = fixture;
        duplicated["fixtures"][1]["resources"] = json!(["CT_SCANNER", "MRI", "MRI"]);
        assert!(check_resource_assignments("5M", &duplicated, &metadata).is_err());
    }

    #[test]
    fn each_participant_uses_the_reply_to_its_own_thread_request() {
        let mut rows = Vec::new();
        for id in [2, 3, 4] {
            rows.push(
                json!({"direction":"receive","message":{"id":id,"result":{"participant":id}}}),
            );
        }
        for id in [2, 3, 4] {
            let request = json!({"direction":"send","message":{"id":id,"method":"thread/start"}});
            assert_eq!(reply(&rows, &request).unwrap()["participant"], id);
            let mut missing = rows.clone();
            missing.retain(|row| row["message"]["id"] != id);
            missing
                .push(json!({"direction":"send","message":{"id":id,"result":{"participant":id}}}));
            missing.push(json!({"direction":"receive","message":{"id":id,"method":"notification","result":{"participant":id}}}));
            assert!(reply(&missing, &request).is_err());
            let mut duplicate = rows.clone();
            duplicate.push(
                json!({"direction":"receive","message":{"id":id,"result":{"participant":id}}}),
            );
            assert!(reply(&duplicate, &request).is_err());
        }
        assert!(reply(&rows, &json!({"message":{"method":"thread/start"}})).is_err());
    }
}
