//! EPF checks source execution; this auditor checks actual file/effect data.
use crate::{
    evidence::{Result, digest},
    journal::{self, one, records},
};
use hibana::runtime::wire::{Payload, WirePayload};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn field(v: &Value) -> Result<u64> {
    v.as_u64()
        .filter(|v| *v <= 255)
        .ok_or_else(|| "invalid compiled operation field".into())
}
fn validate<P: WirePayload>(f: &journal::Frame) -> Result<()> {
    P::validate_payload(Payload::new(&f.bytes)).map_err(|e| format!("WASI payload: {e:?}").into())
}
fn same_path(bytes: &[u8], path: &str) -> bool {
    use hibana_wasip1_runtime::choreofs::{ChoreoFsFact, ChoreoFsFacts, ObjectId};
    let facts = [ChoreoFsFact::new(path.as_bytes(), ObjectId(0))];
    ChoreoFsFacts::new(&facts).resolve(bytes) == Some(ObjectId(0))
}
pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let mut result = check(&records(path)?, &artifact)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if compiled["task"] != "16E" || fixture["task"] != "16E" || fixture["type"] != "fixtures" {
        return Err("CI/CD task or source differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("DEVELOPER"), Some("TESTER"), Some("DEPLOYER")])
    {
        return Err("CI/CD participants differ".into());
    }
    let dynamic = crate::projection::check_dynamic(artifact, rows)?;
    let labels = crate::projection::frames(artifact, rows)?;
    let mut journal = journal::read(rows, &labels, actors)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let frames = &journal.frames;
    if frames
        .values()
        .any(|f| f.identity["session"] != compiled["session"])
        || journal.calls.len() != 3
        || journal.score["complete"] != true
        || journal.score["violations"] != json!([])
        || journal.score["events"]
            .as_array()
            .ok_or("score events")?
            .len()
            != 3
    {
        return Err("CI/CD completion or score differs".into());
    }
    // Immutable capture indices identify send-poll occurrences. They grant no
    // permission and do not replay another protocol or holder state machine.
    let mut locations = BTreeMap::new();
    let mut ordinals = BTreeMap::<u64, u64>::new();
    for (at, row) in rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["direction"] == "runtime")
    {
        let m = &row["message"];
        if m["type"] != "protocol_capture" {
            continue;
        }
        let producer = m["producer"].as_u64().ok_or("capture producer")?;
        for _ in m["records"].as_array().ok_or("capture records")? {
            let ordinal = ordinals.entry(producer).or_default();
            locations.insert((producer, *ordinal), at);
            *ordinal += 1;
        }
    }
    let mut positions = BTreeMap::new();
    for (&id, receipt) in &receipts {
        let producer = receipt["producer"].as_u64().ok_or("send producer")?;
        let ordinal = receipt["first_poll"].as_u64().ok_or("send first poll")?;
        positions.insert(
            id,
            *locations
                .get(&(producer, ordinal))
                .ok_or("send capture position")?,
        );
    }
    let send_at = |id: u64| positions[&id];
    let operations = compiled["operation_files"]
        .as_array()
        .ok_or("operation files")?;
    if operations.len() != 3 {
        return Err("operation binding population differs".into());
    }
    let mut assigned = BTreeSet::new();
    let mut used_calls = BTreeSet::new();
    let mut results = BTreeMap::new();
    let mut accepted = 0;
    let mut rejected = 0;
    let mut returns = 0;
    for binding in operations {
        let file = field(&binding["file"])?;
        let owner = field(&binding["metadata"])?;
        let control = field(&binding["close"])?;
        let guest = field(&binding["guest"])?;
        let resource = field(&binding["resource"])?;
        let effect_owner = field(&binding["effect_owner"])?;
        let fd = u8::try_from(field(&binding["fd"])?)?;
        let path = binding["path"].as_str().ok_or("file path")?;
        let agent = binding["agent"].as_str().ok_or("agent")?;
        let tool = binding["tool"].as_str().ok_or("tool")?;
        if !assigned.insert(agent) {
            return Err("repeated operation binding".into());
        }
        let original = one(actors.iter().filter(|a| a["agent"] == agent))?;
        let original = one(original["calls"].as_array().ok_or("assigned calls")?.iter())?;
        if original["tool"] != tool {
            return Err("operation differs from assignment".into());
        }
        let mut requests = frames
            .iter()
            .filter(|(_, f)| {
                (f.direction(file, owner)
                    && matches!(
                        f.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT
                            | protocol::LABEL_WASI_FD_CLOSE
                    ))
                    || (f.direction(file, control)
                        && f.logical_label == protocol::LABEL_WASI_FD_CLOSE_OBJECT)
            })
            .map(|(&id, f)| (f.received, id, f))
            .collect::<Vec<_>>();
        let mut replies = frames
            .iter()
            .filter(|(_, f)| {
                (f.direction(owner, file)
                    && matches!(
                        f.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN_RET
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT_RET
                            | protocol::LABEL_WASI_FD_CLOSE_RET
                    ))
                    || (f.direction(control, file)
                        && matches!(
                            f.logical_label,
                            protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                                | protocol::LABEL_WASI_FD_CLOSE_REJECTED
                        ))
            })
            .map(|(&id, f)| (f.received, id, f))
            .collect::<Vec<_>>();
        requests.sort_by_key(|&(at, _, _)| at);
        replies.sort_by_key(|&(at, _, _)| at);
        if requests.is_empty() || requests.len() != replies.len() {
            return Err("unpaired operation file data".into());
        }
        let mut bytes = Vec::new();
        let mut offset = 0usize;
        let mut flags = 0u16;
        for ((_, request_id, request), (_, reply_id, reply)) in requests.into_iter().zip(replies) {
            let request_at = request.received()?;
            let reply_at = send_at(reply_id);
            if receipts[&request_id]["send_actor"] != guest
                || receipts[&request_id]["receive_commit"]["actor"] != owner
                || receipts[&reply_id]["send_actor"] != owner
                || receipts[&reply_id]["receive_commit"]["actor"] != guest
                || reply_at <= request_at
            {
                return Err("operation file request/reply actor or occurrence differs".into());
            }
            match (request.logical_label, reply.logical_label) {
                (protocol::LABEL_WASI_PATH_OPEN, protocol::LABEL_WASI_PATH_OPEN_RET) => {
                    validate::<protocol::PathOpenReq>(request)?;
                    validate::<protocol::PathOpenedRet>(reply)?;
                    if reply.bytes[1..3] == [0, 0] {
                        if reply.bytes[0] != fd
                            || request.bytes[0] != 3
                            || !same_path(&request.bytes[14..], path)
                        {
                            return Err("open names another object".into());
                        }
                        if u16::from_be_bytes(request.bytes[9..11].try_into()?) & 8 != 0 {
                            bytes.clear();
                        }
                        flags = u16::from_be_bytes(request.bytes[11..13].try_into()?);
                        offset = 0;
                    }
                }
                (
                    protocol::LABEL_WASI_FD_WRITE_OBJECT,
                    protocol::LABEL_WASI_FD_WRITE_OBJECT_RET,
                ) => {
                    validate::<protocol::FdWriteObjectReq>(request)?;
                    validate::<protocol::FdWriteDoneRet>(reply)?;
                    let count = usize::from(reply.bytes[1]);
                    let errno = u16::from_be_bytes(reply.bytes[2..4].try_into()?);
                    if request.bytes[0] != fd
                        || reply.bytes[0] != fd
                        || count > request.bytes.len() - 2
                        || (errno != 0 && count != 0)
                    {
                        return Err("write fd or accepted prefix differs".into());
                    }
                    if errno == 0 {
                        let start = if flags & 1 != 0 { bytes.len() } else { offset };
                        let end = start.checked_add(count).ok_or("file offset overflow")?;
                        bytes.resize(bytes.len().max(end), 0);
                        bytes[start..end].copy_from_slice(&request.bytes[2..2 + count]);
                        offset = end;
                    }
                }
                (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                    validate::<protocol::FdCloseReq>(request)?;
                    validate::<protocol::FdClosedRet>(reply)?;
                    if request.bytes != [fd] || reply.bytes != [fd, 0, 0] {
                        return Err("ordinary close differs".into());
                    }
                }
                (protocol::LABEL_WASI_FD_CLOSE_OBJECT, protocol::LABEL_WASI_FD_CLOSE_REJECTED) => {
                    validate::<protocol::FdCloseObjectReq>(request)?;
                    validate::<protocol::FdCloseRejectedRet>(reply)?;
                    if request.bytes != [fd]
                        || reply.bytes != [fd, 0, 28]
                        || serde_json::from_slice::<Value>(&bytes)
                            .is_ok_and(|v| v == original["arguments"])
                    {
                        return Err("rejected file close contains valid assigned arguments".into());
                    }
                    rejected += 1;
                }
                (
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT,
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET,
                ) => {
                    validate::<protocol::FdCloseObjectReq>(request)?;
                    validate::<protocol::FdClosedRet>(reply)?;
                    if request.bytes != [fd] || reply.bytes != [fd, 0, 0] {
                        return Err("accepted close fd or reply differs".into());
                    }
                    let arguments: Value = serde_json::from_slice(&bytes)?;
                    let (&call_id, call) = one(journal.calls.iter().filter(|(_, c)| {
                        c.request["agent"] == agent && c.request["tool"] == tool
                    }))?;
                    let (result_at, result) = call.result()?;
                    if !used_calls.insert(call_id)
                        || arguments != original["arguments"]
                        || arguments != call.request["arguments"]
                        || result["success"] != true
                        || call.emitted <= request_at
                        || result_at >= reply_at
                    {
                        return Err("accepted input/effect/result differs or is reused".into());
                    }
                    let intent_label = u8::try_from(field(&binding["intent_label"])?)?;
                    let request_label = u8::try_from(field(&binding["request_label"])?)?;
                    let result_label = u8::try_from(field(&binding["result_label"])?)?;
                    let intent = one(frames.values().filter(|f| {
                        f.direction(control, effect_owner) && f.logical_label == intent_label
                    }))?;
                    let resource_request = one(frames.values().filter(|f| {
                        f.direction(resource, effect_owner) && f.logical_label == request_label
                    }))?;
                    let (&response_id, response) = one(frames.iter().filter(|(_, f)| {
                        f.direction(effect_owner, resource) && f.logical_label == result_label
                    }))?;
                    if intent.value()? != arguments
                        || resource_request.value()? != arguments
                        || intent.received()? <= request_at
                        || intent.received()? >= call.emitted
                        || resource_request.received()? >= call.emitted
                        || response.value()? != *result
                        || send_at(response_id) <= result_at
                        || response.received()? >= reply_at
                    {
                        return Err(
                            "actual resource request or reply differs from admitted effect".into(),
                        );
                    }
                    one(frames.values().filter(|f| {
                        f.direction(control, effect_owner)
                            && f.logical_label == 10
                            && f.bytes.first().copied().map(u64::from) == Some(resource)
                    }))?;
                    // EPF checks the transfer and receive occurrence. A send
                    // accepting the return does not imply its peer has received
                    // it before the caller's close reply is sent.
                    returns += 1;
                    let event = one(journal.score["events"]
                        .as_array()
                        .ok_or("score events")?
                        .iter()
                        .filter(|e| e["agent"] == agent))?;
                    if event["tool"] != tool
                        || event["args"] != arguments
                        || event["result"] != result["data"]
                        || event["success"] != result["success"]
                        || event["violations"] != json!([])
                    {
                        return Err("common score differs from the actual effect".into());
                    }
                    if results
                        .insert(agent, (result.clone(), result_at, reply_at))
                        .is_some()
                    {
                        return Err("reused operation result".into());
                    }
                    accepted += 1;
                }
                _ => return Err("file request/reply contract differs".into()),
            }
        }
        let exit = one(frames
            .values()
            .filter(|f| f.direction(file, owner) && f.logical_label == 241))?;
        if exit.bytes != [0, 0, 0, 0] {
            return Err("file termination differs".into());
        }
    }
    let role = |name: &str| field(&compiled["roles"][name]);
    let notice = u8::try_from(field(&compiled["labels"]["Notice"])?)?;
    let mut notices = BTreeSet::new();
    for (source, target, kind, agent) in [
        ("DEV_CLOSE", "DEV_FILE", "compile_result", "DEVELOPER"),
        ("DEV_CLOSE", "TEST_FILE", "build_complete", "DEVELOPER"),
        ("TEST_CLOSE", "TEST_FILE", "test_result", "TESTER"),
        ("TEST_CLOSE", "DEPLOY_FILE", "tests_passed", "TESTER"),
        ("DEPLOY_CLOSE", "DEPLOY_FILE", "deploy_result", "DEPLOYER"),
        (
            "DEPLOY_CLOSE",
            "DEV_FILE",
            "deployment_complete",
            "DEPLOYER",
        ),
        ("DEPLOYER", "TEST_FILE", "deployment_complete", "DEPLOYER"),
    ] {
        let source = role(source)?;
        let target = role(target)?;
        let (&id, frame) = one(frames
            .iter()
            .filter(|(_, f)| f.direction(source, target) && f.logical_label == notice))?;
        let (result, result_at, reply_at) = results
            .get(agent)
            .ok_or("notification without actual result")?;
        if !notices.insert(id)
            || frame.value()? != json!({"kind":kind,"result":result})
            || send_at(id) <= *result_at
            || send_at(id) <= *reply_at
        {
            return Err("incoming notification/result differs or is reused".into());
        }
    }
    if accepted != 3
        || used_calls.len() != journal.calls.len()
        || returns != 3
        || frames
            .values()
            .filter(|f| f.logical_label == notice)
            .count()
            != notices.len()
    {
        return Err("unmatched operation/effect/result evidence".into());
    }
    Ok(
        json!({"status":"passed","task":"16E","depth":fixture["depth"],"kind":"actual_endpoint_bound_choreofs",
 "scored_benchmark":false,"model_calls":null,"actual_resource_endpoint_requests":3,"actual_resource_endpoint_returns":returns,
 "domain_calls":journal.calls.len(),"accepted_closes":accepted,"rejected_closes":rejected,"completed_files":3,
 "guest_exits":3,"carrier_frames":frames.len(),"notifications":4,"incoming_result_messages":7,
 "simulator_complete":true,"violations":0,"dynamic_protocol":dynamic,
 "effect_scope":"EPF-checked receives, mounted-file bytes, unique actual resource requests/effects/replies, incoming notifications/results and common score"}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    let artifact = crate::projection::load(path)?;
    let bindings = artifact["compiled"]["operation_files"]
        .as_array()
        .ok_or("operation bindings")?;
    let first = one(bindings.iter().filter(|b| b["agent"] == "DEVELOPER"))?;
    let file = field(&first["file"])?;
    let control = field(&first["close"])?;
    let resource = field(&first["resource"])?;
    let owner = field(&first["effect_owner"])?;
    let request = field(&first["request_label"])?;
    let response = field(&first["result_label"])?;
    let rows = records(path)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    fs::create_dir(output)?;
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
        "forged_notification",
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
            "missing_exit" => {
                let at = copy
                    .iter()
                    .position(|r| r["message"]["type"] == "exit")
                    .ok_or("exit mutation target")?;
                copy.remove(at);
            }
            "false_score" => {
                let row = copy
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "score")
                    .ok_or("score mutation target")?;
                row["message"]["complete"] = json!(false);
            }
            _ => {
                let target = copy
                    .iter()
                    .rev()
                    .find(|row| {
                        let m = &row["message"];
                        if m["type"] != "protocol_frame" {
                            return false;
                        }
                        let logical = m["frame_id"]
                            .as_u64()
                            .and_then(|id| labels.get(&id).copied())
                            .map(u64::from);
                        match kind {
                            "missing_loan" | "wrong_loan_resource" => {
                                m["source"] == owner
                                    && m["target"] == control
                                    && m["payload"][0] == resource
                            }
                            "missing_request" | "forged_request" => {
                                m["source"] == resource
                                    && m["target"] == owner
                                    && logical == Some(request)
                            }
                            "missing_reply" | "forged_reply" => {
                                m["source"] == owner
                                    && m["target"] == resource
                                    && logical == Some(response)
                            }
                            "missing_return" => {
                                m["source"] == control
                                    && m["target"] == owner
                                    && m["payload"][0] == resource
                            }
                            "missing_notification" | "forged_notification" => {
                                m["source"] == control
                                    && m["target"] == artifact["compiled"]["roles"]["TEST_FILE"]
                            }
                            "missing_close" => {
                                m["source"] == file
                                    && m["target"] == control
                                    && m["payload"] == json!([4])
                            }
                            "wrong_frame_label" | "wrong_lane" => true,
                            _ => false,
                        }
                    })
                    .ok_or("frame mutation target")?["message"]["frame_id"]
                    .clone();
                let matches = |r: &Value| {
                    r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == target
                };
                if kind.starts_with("missing_") {
                    copy.retain(|r| !matches(r));
                } else {
                    for row in copy.iter_mut().filter(|r| matches(r)) {
                        match kind {
                            "wrong_frame_label" => row["message"]["frame_label"] = json!(65535),
                            "wrong_lane" => row["message"]["lane"] = json!(255),
                            "wrong_loan_resource" => {
                                row["message"]["payload"][0] =
                                    artifact["compiled"]["roles"]["DEPLOY_ENV"].clone()
                            }
                            "forged_request" => {
                                row["message"]["payload"] =
                                    json!(serde_json::to_vec(&json!({"component":"forged"}))?)
                            }
                            "forged_reply" => {
                                row["message"]["payload"] = json!(serde_json::to_vec(
                                    &json!({"success":true,"data":"forged"})
                                )?)
                            }
                            "forged_notification" => {
                                row["message"]["payload"] = json!(serde_json::to_vec(
                                    &json!({"kind":"build_complete","result":{"success":true,"data":"forged"}})
                                )?)
                            }
                            _ => return Err("unknown record mutation".into()),
                        }
                    }
                }
            }
        }
        let mutant = output.join(format!("{kind}.jsonl"));
        use std::io::Write;
        let mut stream = std::io::BufWriter::new(fs::File::create_new(&mutant)?);
        for row in &copy {
            serde_json::to_writer(&mut stream, row)?;
            writeln!(stream)?;
        }
        stream.flush()?;
        drop(stream);
        let error = journal::reject_mutant(path, &mutant, "--audit-cicd")?;
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
