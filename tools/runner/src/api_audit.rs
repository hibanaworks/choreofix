//! Match EPF-checked file bytes and resource receives to actual effects and replies.
use crate::journal::file::FileContents;
use crate::{evidence::Result, journal};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
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
pub(crate) fn check(records: &[Value], projection: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &projection["compiled"];
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let journal = crate::journal::read(records, projection, dynamic)?;
    let actors = journal.participants;
    if compiled["task"] != "8E" || fixture["task"] != "8E" {
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
    let frames = &journal.frames;
    let mut effects = BTreeMap::new();
    let mut closes = BTreeSet::new();
    let mut returned = BTreeSet::new();
    let mut resource_requests = 0;
    let mut resource_returns = 0;
    let mut result_reads = 0;
    for (agent, file_name, controller_name, vm_name, file_label, resource) in [
        (
            "DB_ADMIN",
            "ADMIN_FILE",
            "ADMIN_CLOSE",
            "ADMIN_VM",
            18,
            Some(("DATABASE", "DATABASE_RESOURCE", 18, 20, 21)),
        ),
        (
            "BACKEND_DEV",
            "BACKEND_FILE",
            "BACKEND_CLOSE",
            "BACKEND_VM",
            24,
            None,
        ),
        (
            "TESTER",
            "TESTER_FILE",
            "TESTER_CLOSE",
            "TESTER_VM",
            19,
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
        let (result_at, result) = call.result;
        if call.tool != tool
            || call.request["arguments"] != original["arguments"]
            || result["tool_name"] != tool
            || result["success"] != (agent != "TESTER" || depth == 0)
        {
            return Err("actual API call differs from the original assignment".into());
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
        }))?;
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
                    || (frame.direction(file, controller) && frame.logical_label == file_label)
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
                    || (frame.direction(owner, file) && frame.logical_label == 165)
            })
            .map(|(&id, frame)| (frame.received, id, frame))
            .collect::<Vec<_>>();
        requests.sort_by_key(|&(at, _, _)| at);
        replies.sort_by_key(|&(at, _, _)| at);
        if requests.len() != replies.len() {
            return Err("unpaired operation-file requests and replies".into());
        }
        let mut contents = FileContents::new(service_fd);
        for ((_, request_id, request), (_, reply_id, reply)) in requests.into_iter().zip(replies) {
            let receiver = if request.logical_label == file_label {
                controller
            } else {
                owner
            };
            if journal.frames[&request_id].receipt["send_actor"] != vm
                || journal.frames[&request_id].receipt["receive_commit"]["actor"] != receiver
                || journal.frames[&reply_id].receipt["send_actor"] != owner
                || journal.frames[&reply_id].receipt["receive_commit"]["actor"] != vm
            {
                return Err("operation file crossed a physical participant boundary".into());
            }
            match (request.logical_label, reply.logical_label) {
                (protocol::LABEL_WASI_PATH_OPEN, protocol::LABEL_WASI_PATH_OPEN_RET) => {
                    contents.open(request, reply, |actual| {
                        actual == format!("service/{tool}").as_bytes()
                    })?;
                }
                (
                    protocol::LABEL_WASI_FD_WRITE_OBJECT,
                    protocol::LABEL_WASI_FD_WRITE_OBJECT_RET,
                ) => {
                    contents.write(request, reply)?;
                }
                (protocol::LABEL_WASI_FD_READ, protocol::LABEL_WASI_FD_READ_RET) => {
                    contents.read(request, reply)?;
                    if contents.bytes == serde_json::to_vec(result)? {
                        result_reads += 1;
                    }
                }
                (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                    contents.close(request, reply)?;
                }
                (label, 165) if label == file_label => {
                    let (&result_id, published) =
                        journal::one(frames.iter().filter(|(_, f)| {
                            f.direction(controller, owner) && f.logical_label == 42
                        }))?;
                    if request.bytes != contents.bytes
                        || request.value()? != call.request["arguments"]
                        || request.received >= call.emitted
                        || !reply.bytes.is_empty()
                        || published.value()? != *result
                        || result_at >= published.sent
                        || published.received >= reply.sent
                        || journal.frames[&result_id].receipt["send_actor"] != controller
                        || journal.frames[&result_id].receipt["receive_commit"]["actor"] != owner
                        || effects
                            .insert(call_id, (request_id, result_id, reply_id))
                            .is_some()
                    {
                        return Err("submitted file bytes, Endpoint receive, effect or published result differ".into());
                    }
                    closes.insert(request_id);
                    contents.bytes = serde_json::to_vec(result)?;
                    contents.offset = 0;
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
                || request.received >= call.emitted
                || reply.value()? != *result
                || reply.sent <= result_at
                || journal.frames[&request_id].receipt["send_actor"] != resource_owner
                || journal.frames[&request_id].receipt["receive_commit"]["actor"] != controller
                || journal.frames[&reply_id].receipt["send_actor"] != controller
                || journal.frames[&reply_id].receipt["receive_commit"]["actor"] != resource_owner
            {
                return Err("physical resource request/effect/result differs".into());
            }
            let loan = journal::one(frames.values().filter(|f| {
                f.direction(resource_owner, controller)
                    && f.logical_label == 10
                    && f.bytes.first().copied().map(u64::from) == Some(resource)
            }))?;
            let returned_loan = journal::one(frames.values().filter(|f| {
                f.direction(controller, resource_owner)
                    && f.logical_label == 10
                    && f.bytes.first().copied().map(u64::from) == Some(resource)
            }))?;
            if loan.received >= call.emitted || returned_loan.received <= reply.received {
                return Err("resource loan or return did not bracket its effect and reply".into());
            }
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
    let (finished, test_result) = test.result;
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
                request.validate::<protocol::PathOpenReq>()?;
                reply.validate::<protocol::PathOpenedRet>()?;
                if reply.bytes[1..3] != [0, 0]
                    || reply.bytes[0] != outcome_fd
                    || request.bytes[14..] != *b"service/outcome"
                {
                    return Err("outcome open fd/path mismatch".into());
                }
                offset = 0;
            } else {
                request.validate::<protocol::FdReadReq>()?;
                reply.validate::<protocol::FdReadDoneRet>()?;
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
        }))?;
    }
    let events = journal.scored_events;
    if events.len() != 3 || journal.simulator_complete != (depth == 0) {
        return Err("API task score differs from actual terminal result".into());
    }
    for call in journal.calls.values() {
        let event = journal::one(events.iter().filter(|event| {
            event["agent"] == call.request["agent"] && event["tool"] == call.request["tool"]
        }))?;
        if !call.matches_score(event, &call.request["arguments"]) {
            return Err("API task score differs from the actual effect".into());
        }
    }
    let mut ordered = journal.calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    if ordered.iter().map(|call| call.tool).collect::<Vec<_>>()
        != [
            "migrate_schema",
            "implement_backend",
            "run_integration_tests",
        ]
        || ordered
            .windows(2)
            .any(|pair| pair[0].result.0 >= pair[1].emitted)
    {
        return Err("actual schema/backend/test data causality differs".into());
    }
    Ok(
        json!({"task":"8E","depth":depth,"status":"passed","participants":3,
        "domain_calls":journal.calls.len(),"actual_resource_endpoint_requests":resource_requests,
        "actual_resource_endpoint_returns":resource_returns,"complete_result_reads":complete_reads,
        "early_result_reads":early_reads,"observed_operation_result_reads":result_reads,
        "effect_receipts":effects,"violations":0,
        "simulator_complete":journal.simulator_complete,"expected_terminal_test_failure":depth!=0,
        "dynamic_protocol":dynamic}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let evidence = crate::journal::Evidence::open(path)?;
    evidence.audit("8E")?;
    let rows = evidence.rows;
    fs::create_dir(output)?;
    let mut reports = Vec::new();
    for kind in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_file_request",
        "changed_file_bytes",
        "changed_published_result",
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
                        "missing_file_request" | "changed_file_bytes" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 18
                                && m["target"] == 21
                        }
                        "changed_published_result" => {
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == 21
                                && m["target"] == 1
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
                "missing_notification" | "missing_read" | "missing_exit" => {
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
                "missing_file_request"
                | "missing_loan"
                | "missing_resource_request"
                | "missing_resource_return" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    copy.retain(|row| {
                        !(row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id)
                    });
                }
                "changed_file_bytes"
                | "changed_published_result"
                | "wrong_loan_resource"
                | "forged_resource_request"
                | "forged_resource_result" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    let mut bytes = payload(&copy[index]["message"])?;
                    match kind {
                        // Equivalent JSON, different bytes from the guest's accepted writes.
                        "changed_file_bytes" => bytes.push(b' '),
                        "wrong_loan_resource" => bytes[0] = 16,
                        "forged_resource_request" => {
                            bytes = serde_json::to_vec(&json!({"migration_name":"wrong_target"}))?
                        }
                        "changed_published_result" | "forged_resource_result" => {
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
        let error = crate::journal::reject_mutant(path, &file, "8E")?;
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
