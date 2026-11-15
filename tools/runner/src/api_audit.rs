//! Match EPF-checked file bytes and resource receives to actual effects and replies.
use crate::{evidence::Result, journal};
use hibana::runtime::wire::{Payload, WirePayload};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};

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
fn validate<P: WirePayload>(frame: &journal::Frame) -> Result<()> {
    P::validate_payload(Payload::new(&frame.bytes))
        .map_err(|error| format!("WASI payload: {error:?}").into())
}
pub fn audit(path: &Path) -> Result<Value> {
    let records = journal::records(path)?;
    let projection = crate::projection::load(path)?;
    let mut result = check(&records, &projection)?;
    result["source_events_sha256"] = json!(crate::evidence::digest(path)?);
    result["compiled_projection_sha256"] = json!(crate::evidence::digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}
fn check(records: &[Value], projection: &Value) -> Result<Value> {
    let compiled = &projection["compiled"];
    let labels = crate::projection::frames(projection, records)?;
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("API actors")?;
    if compiled["task"] != "8E" || fixture["task"] != "8E" || actors.len() != 3 {
        return Err("unexpected API task".into());
    }
    let depth = fixture["depth"]
        .as_u64()
        .filter(|n| *n <= 2)
        .ok_or("API depth")?;
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let service_fd = u8::try_from(compiled["service_fd"].as_u64().ok_or("operation fd")?)?;
    let outcome_fd = u8::try_from(compiled["outcome_fd"].as_u64().ok_or("outcome fd")?)?;
    let dynamic = crate::projection::check_dynamic(projection, records)?;
    let mut journal = journal::read(records, &labels, actors)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let frames = &journal.frames;
    let mut effects = BTreeMap::new();
    let mut closes = BTreeSet::new();
    let mut returned = BTreeSet::new();
    let mut resource_requests = 0;
    let mut resource_returns = 0;
    let mut result_reads = 0;
    let mut rejections = 0;
    for (agent, file_name, controller_name, vm_name, resource) in [
        (
            "DB_ADMIN",
            "ADMIN_FILE",
            "ADMIN_CLOSE",
            "ADMIN_VM",
            Some(("DATABASE", "DATABASE_RESOURCE", 18, 20, 21)),
        ),
        (
            "BACKEND_DEV",
            "BACKEND_FILE",
            "BACKEND_CLOSE",
            "BACKEND_VM",
            None,
        ),
        (
            "TESTER",
            "TESTER_FILE",
            "TESTER_CLOSE",
            "TESTER_VM",
            Some(("TEST_ENV", "TEST_RESOURCE", 19, 22, 23)),
        ),
    ] {
        let owner = role(agent)?;
        let file = role(file_name)?;
        let controller = role(controller_name)?;
        let vm = role(vm_name)?;
        let actor = journal::one(actors.iter().filter(|actor| actor["agent"] == agent))?;
        let original = journal::one(actor["calls"].as_array().ok_or("original calls")?.iter())?;
        let tool = original["tool"].as_str().ok_or("original tool")?;
        let (&call_id, call) = journal::one(
            journal
                .calls
                .iter()
                .filter(|(_, call)| call.request["agent"] == agent),
        )?;
        let (result_at, result) = call.result()?;
        if call.tool()? != tool
            || call.request["arguments"] != original["arguments"]
            || result["tool_name"] != tool
            || result["success"] != (agent != "TESTER" || depth == 0)
        {
            return Err("actual API call differs from the original assignment".into());
        }
        let (&authority_id, _) = journal::one(
            frames
                .iter()
                .filter(|(_, frame)| frame.direction(owner, file) && frame.logical_label == 165),
        )?;
        if receipts[&authority_id]["receive_commit"]["actor"] != vm {
            return Err("actual close authority did not reach its guest".into());
        }
        let exit = journal::one(
            frames
                .values()
                .filter(|f| f.direction(file, owner) && f.logical_label == 241),
        )?;
        if exit.bytes != 0i32.to_be_bytes() {
            return Err("file owner received failed exit".into());
        }
        journal::one(frames.values().filter(|f| {
            f.direction(vm, owner)
                && f.logical_label == 10
                && f.bytes.first().copied().map(u64::from) == Some(file)
        }))?
        .received()?;
        returned.insert(file);

        let mut requests = frames
            .iter()
            .filter(|(_, frame)| {
                (frame.direction(file, owner)
                    && matches!(
                        frame.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT
                            | protocol::LABEL_WASI_FD_READ
                            | protocol::LABEL_WASI_FD_CLOSE
                    ))
                    || (frame.direction(file, controller)
                        && frame.logical_label == protocol::LABEL_WASI_FD_CLOSE_OBJECT)
            })
            .map(|(&id, frame)| (frame.received, id, frame))
            .collect::<Vec<_>>();
        let mut replies = frames
            .iter()
            .filter(|(_, frame)| {
                (frame.direction(owner, file)
                    && matches!(
                        frame.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN_RET
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT_RET
                            | protocol::LABEL_WASI_FD_READ_RET
                            | protocol::LABEL_WASI_FD_CLOSE_RET
                    ))
                    || (frame.direction(controller, file)
                        && matches!(
                            frame.logical_label,
                            protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                                | protocol::LABEL_WASI_FD_CLOSE_REJECTED
                        ))
            })
            .map(|(&id, frame)| (frame.received, id, frame))
            .collect::<Vec<_>>();
        requests.sort_by_key(|&(at, _, _)| at);
        replies.sort_by_key(|&(at, _, _)| at);
        if requests.len() != replies.len() {
            return Err("unpaired operation-file requests and replies".into());
        }
        let mut bytes = Vec::new();
        let mut offset = 0usize;
        let mut flags = 0u16;
        for ((_, request_id, request), (_, reply_id, reply)) in requests.into_iter().zip(replies) {
            if receipts[&request_id]["send_actor"] != vm
                || receipts[&request_id]["receive_commit"]["actor"] != owner
                || receipts[&reply_id]["send_actor"] != owner
                || receipts[&reply_id]["receive_commit"]["actor"] != vm
            {
                return Err("operation file crossed a physical participant boundary".into());
            }
            match (request.logical_label, reply.logical_label) {
                (protocol::LABEL_WASI_PATH_OPEN, protocol::LABEL_WASI_PATH_OPEN_RET) => {
                    validate::<protocol::PathOpenReq>(request)?;
                    validate::<protocol::PathOpenedRet>(reply)?;
                    if reply.bytes[1..3] == [0, 0] {
                        if reply.bytes[0] != service_fd
                            || request.bytes[0] != 3
                            || request.bytes[14..] != *format!("service/{tool}").as_bytes()
                        {
                            return Err("operation open names another object".into());
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
                    if request.bytes[0] != service_fd
                        || reply.bytes[0] != service_fd
                        || count > request.bytes.len() - 2
                        || (errno != 0 && count != 0)
                    {
                        return Err("operation write fd or accepted prefix differs".into());
                    }
                    if errno == 0 {
                        let start = if flags & 1 != 0 { bytes.len() } else { offset };
                        let end = start.checked_add(count).ok_or("file offset overflow")?;
                        bytes.resize(bytes.len().max(end), 0);
                        bytes[start..end].copy_from_slice(&request.bytes[2..2 + count]);
                        offset = end;
                    }
                }
                (protocol::LABEL_WASI_FD_READ, protocol::LABEL_WASI_FD_READ_RET) => {
                    validate::<protocol::FdReadReq>(request)?;
                    validate::<protocol::FdReadDoneRet>(reply)?;
                    let end = offset
                        .saturating_add(usize::from(request.bytes[1]))
                        .min(bytes.len());
                    if request.bytes[0] != service_fd
                        || reply.bytes[0] != service_fd
                        || reply.bytes[1..3] != [0, 0]
                        || bytes.get(offset..end) != Some(&reply.bytes[4..])
                    {
                        return Err("operation read differs from actual file data".into());
                    }
                    if bytes == serde_json::to_vec(result)? {
                        result_reads += 1;
                    }
                    offset = end;
                }
                (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                    validate::<protocol::FdCloseReq>(request)?;
                    validate::<protocol::FdClosedRet>(reply)?;
                    if request.bytes != [service_fd] || reply.bytes != [service_fd, 0, 0] {
                        return Err("ordinary close fd/result mismatch".into());
                    }
                }
                (protocol::LABEL_WASI_FD_CLOSE_OBJECT, protocol::LABEL_WASI_FD_CLOSE_REJECTED) => {
                    validate::<protocol::FdCloseObjectReq>(request)?;
                    validate::<protocol::FdCloseRejectedRet>(reply)?;
                    if request.bytes != [service_fd]
                        || reply.bytes != [service_fd, 0, 28]
                        || serde_json::from_slice::<Value>(&bytes)
                            .is_ok_and(|v| v == original["arguments"])
                        || request.received()? < call.emitted && call.emitted < reply.received()?
                    {
                        return Err(
                            "rejected close admitted an effect or rejected valid input".into()
                        );
                    }
                    rejections += 1;
                }
                (
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT,
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET,
                ) => {
                    validate::<protocol::FdCloseObjectReq>(request)?;
                    validate::<protocol::FdClosedRet>(reply)?;
                    if request.bytes != [service_fd]
                        || reply.bytes != [service_fd, 0, 0]
                        || serde_json::from_slice::<Value>(&bytes)? != call.request["arguments"]
                        || request.received()? >= call.emitted
                        || result_at >= reply.sent
                        || effects
                            .insert(call_id, (request_id, authority_id))
                            .is_some()
                    {
                        return Err("accepted file bytes, effect or actual reply differ".into());
                    }
                    closes.insert(request_id);
                    bytes = serde_json::to_vec(result)?;
                    offset = 0;
                }
                _ => return Err("operation file request/reply types differ".into()),
            }
        }
        if let Some((resource_owner, resource_name, intent_label, request_label, reply_label)) =
            resource
        {
            let resource_owner = role(resource_owner)?;
            let resource = role(resource_name)?;
            let intent = journal::one(frames.values().filter(|f| {
                f.direction(controller, resource_owner) && f.logical_label == intent_label
            }))?;
            let (&request_id, request) = journal::one(frames.iter().filter(|(_, f)| {
                f.direction(resource, controller) && f.logical_label == request_label
            }))?;
            let (&reply_id, reply) = journal::one(frames.iter().filter(|(_, f)| {
                f.direction(resource, resource_owner) && f.logical_label == reply_label
            }))?;
            if intent.value()? != call.request["arguments"]
                || request.value()? != call.request["arguments"]
                || request.received()? >= call.emitted
                || reply.value()? != *result
                || reply.sent <= result_at
                || receipts[&request_id]["send_actor"] != resource_owner
                || receipts[&request_id]["receive_commit"]["actor"] != owner
                || receipts[&reply_id]["send_actor"] != owner
                || receipts[&reply_id]["receive_commit"]["actor"] != resource_owner
            {
                return Err("physical resource request/effect/result differs".into());
            }
            journal::one(frames.values().filter(|f| {
                f.direction(controller, resource_owner)
                    && f.logical_label == 10
                    && f.bytes.first().copied().map(u64::from) == Some(resource)
            }))?
            .received()?;
            resource_requests += 1;
            resource_returns += 1;
        }
    }
    if effects.len() != 3
        || effects.len() != journal.calls.len()
        || closes.len() != 3
        || returned.len() != 3
    {
        return Err("missing or unmatched API effect/close/return".into());
    }
    let (_, test) = journal::one(
        journal
            .calls
            .iter()
            .filter(|(_, call)| call.request["agent"] == "TESTER"),
    )?;
    let (finished, test_result) = test.result()?;
    let mut complete_reads = BTreeMap::new();
    let mut early_reads = 0;
    let tester_controller = role("TESTER_CLOSE")?;
    for (owner_name, file_name, read_name, vm_name, labels) in [
        (
            "ADMIN_OUTCOME",
            "ADMIN_OUTCOME_FILE",
            "ADMIN_OUTCOME_READ",
            "ADMIN_VM",
            [50, 60],
        ),
        (
            "BACKEND_OUTCOME",
            "BACKEND_OUTCOME_FILE",
            "BACKEND_OUTCOME_READ",
            "BACKEND_VM",
            [51, 61],
        ),
    ] {
        let owner = role(owner_name)?;
        let file = role(file_name)?;
        let read = role(read_name)?;
        let vm = role(vm_name)?;
        let notice = journal::one(frames.values().filter(|f| {
            f.direction(tester_controller, owner) && labels.contains(&f.logical_label)
        }))?;
        if notice.value()? != *test_result
            || notice.sent <= finished
            || notice.logical_label != labels[usize::from(depth != 0)]
        {
            return Err("outcome notification differs from the actual test result".into());
        }
        let mut requests = frames
            .values()
            .filter(|f| {
                (f.direction(file, owner) && f.logical_label == protocol::LABEL_WASI_PATH_OPEN)
                    || (f.direction(file, read) && f.logical_label == protocol::LABEL_WASI_FD_READ)
            })
            .collect::<Vec<_>>();
        let mut replies = frames
            .values()
            .filter(|f| {
                (f.direction(owner, file) && f.logical_label == protocol::LABEL_WASI_PATH_OPEN_RET)
                    || (f.direction(read, file)
                        && f.logical_label == protocol::LABEL_WASI_FD_READ_RET)
            })
            .collect::<Vec<_>>();
        requests.sort_by_key(|f| f.received);
        replies.sort_by_key(|f| f.received);
        if requests.len() != replies.len() {
            return Err("unpaired outcome file I/O".into());
        }
        let canonical = serde_json::to_vec(test_result)?;
        let mut offset = 0usize;
        let mut complete = 0;
        for (request, reply) in requests.into_iter().zip(replies) {
            if request.logical_label == protocol::LABEL_WASI_PATH_OPEN {
                validate::<protocol::PathOpenReq>(request)?;
                validate::<protocol::PathOpenedRet>(reply)?;
                if reply.bytes[1..3] != [0, 0]
                    || reply.bytes[0] != outcome_fd
                    || request.bytes[14..] != *b"service/outcome"
                {
                    return Err("outcome open fd/path mismatch".into());
                }
                offset = 0;
            } else {
                validate::<protocol::FdReadReq>(request)?;
                validate::<protocol::FdReadDoneRet>(reply)?;
                let end = offset
                    .saturating_add(usize::from(request.bytes[1]))
                    .min(canonical.len());
                if request.bytes[0] != outcome_fd
                    || reply.bytes[0] != outcome_fd
                    || reply.bytes[1..3] != [0, 0]
                    || reply.sent <= finished
                    || canonical.get(offset..end) != Some(&reply.bytes[4..])
                {
                    return Err("outcome bytes differ from actual test result".into());
                }
                if request.sent < finished {
                    early_reads += 1;
                }
                if offset < canonical.len() && end == canonical.len() {
                    complete += 1;
                }
                offset = end;
            }
        }
        if complete == 0 {
            return Err("actual outcome was never completely read".into());
        }
        complete_reads.insert(owner, complete);
        journal::one(frames.values().filter(|f| {
            f.direction(vm, owner)
                && f.logical_label == 10
                && f.bytes.first().copied().map(u64::from) == Some(file)
        }))?
        .received()?;
    }
    let events = journal.score["events"]
        .as_array()
        .ok_or("actual API score events")?;
    if events.len() != 3
        || journal.score["complete"] != (depth == 0)
        || journal.score["violations"] != json!([])
    {
        return Err("API task score differs from actual terminal result".into());
    }
    for call in journal.calls.values() {
        let (_, result) = call.result()?;
        let event = journal::one(events.iter().filter(|event| {
            event["agent"] == call.request["agent"] && event["tool"] == call.request["tool"]
        }))?;
        if event["args"] != call.request["arguments"]
            || event["success"] != result["success"]
            || event["result"] != result["data"]
        {
            return Err("API task score differs from the actual effect".into());
        }
    }
    let mut ordered = journal.calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    if ordered
        .iter()
        .map(|call| call.tool())
        .collect::<Result<Vec<_>>>()?
        != [
            "migrate_schema",
            "implement_backend",
            "run_integration_tests",
        ]
        || ordered
            .windows(2)
            .any(|pair| pair[0].result().is_ok_and(|(at, _)| at >= pair[1].emitted))
    {
        return Err("actual schema/backend/test data causality differs".into());
    }
    Ok(
        json!({"task":"8E","depth":depth,"status":"passed","participants":3,
        "domain_calls":journal.calls.len(),"actual_resource_endpoint_requests":resource_requests,
        "actual_resource_endpoint_returns":resource_returns,"complete_result_reads":complete_reads,
        "early_result_reads":early_reads,"observed_operation_result_reads":result_reads,
        "rejected_object_closes":rejections,"effect_receipts":effects,"violations":0,
        "simulator_complete":journal.score["complete"],"expected_terminal_test_failure":depth!=0,
        "dynamic_protocol":dynamic}),
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
                                && m["action"] == "send"
                                && m["source"] == 17
                                && m["target"] == 20
                                && m["payload"] == json!([4])
                        }
                        "altered_frame" => m["type"] == "protocol_frame" && m["action"] == "send",
                        "forged_arguments" => m["type"] == "execute",
                        "forged_result" => m["type"] == "simulator_result",
                        "missing_notification" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 22
                                && m["target"] == 11
                        }
                        "missing_read" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 23
                                && m["target"] == 13
                                && m["payload"].as_array().is_some_and(|b| b.len() > 5)
                        }
                        "missing_exit" => m["type"] == "exit",
                        "false_score" => m["type"] == "score",
                        "missing_loan" | "wrong_loan_resource" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 3
                                && m["target"] == 20
                        }
                        "missing_resource_request" | "forged_resource_request" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 15
                                && m["target"] == 20
                        }
                        "forged_resource_result" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 15
                                && m["target"] == 3
                        }
                        "missing_resource_return" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 20
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
