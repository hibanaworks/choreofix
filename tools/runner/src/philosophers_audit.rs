//! Bind EPF-checked receive occurrences to actual file data, effects and replies.
//! Source eligibility and affine custody are checked by EPF, not reimplemented.
use crate::{evidence::Result, journal};
use hibana::runtime::wire::{Payload, WirePayload};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::Path,
};

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
            .join("projection.json"),
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}

fn check(records: &[Value], projection: &Value) -> Result<Value> {
    use crate::philosopher_identity::{AGENTS, roles};
    let labels = crate::projection::frames(projection, records)?;
    let dynamic = crate::projection::check_dynamic(projection, records)?;
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    if projection["compiled"]["task"] != fixture["task"]
        || fixture["terminal_review_failure"] != false
    {
        return Err("different task or invalid dining configuration".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("fixtures")?;
    let count = match fixture["task"].as_str() {
        Some("9E") => 3,
        Some("9M") => 5,
        Some("9H") => 7,
        _ => return Err("unknown dining task".into()),
    };
    // Physical integration identities, shared with the actual application.
    // This is not a source execution or a current-holder table.
    let layout = [
        (
            "FORK_A",
            [
                roles::A,
                roles::FORK_A,
                roles::A_WASI,
                roles::A_IO,
                roles::FORK_A_RESOURCE,
                roles::A_THINK_FILE,
                roles::THINK_A,
                roles::A_EAT_FILE,
            ],
        ),
        (
            "FORK_B",
            [
                roles::B,
                roles::FORK_B,
                roles::B_WASI,
                roles::B_IO,
                roles::FORK_B_RESOURCE,
                roles::B_THINK_FILE,
                roles::THINK_B,
                roles::B_EAT_FILE,
            ],
        ),
        (
            "FORK_C",
            [
                roles::C,
                roles::FORK_C,
                roles::C_WASI,
                roles::C_IO,
                roles::FORK_C_RESOURCE,
                roles::C_THINK_FILE,
                roles::THINK_C,
                roles::C_EAT_FILE,
            ],
        ),
        (
            "FORK_D",
            [
                roles::D,
                roles::FORK_D,
                roles::D_WASI,
                roles::D_IO,
                roles::FORK_D_RESOURCE,
                roles::D_THINK_FILE,
                roles::THINK_D,
                roles::D_EAT_FILE,
            ],
        ),
        (
            "FORK_E",
            [
                roles::E,
                roles::FORK_E,
                roles::E_WASI,
                roles::E_IO,
                roles::FORK_E_RESOURCE,
                roles::E_THINK_FILE,
                roles::THINK_E,
                roles::E_EAT_FILE,
            ],
        ),
        (
            "FORK_F",
            [
                roles::F,
                roles::FORK_F,
                roles::F_WASI,
                roles::F_IO,
                roles::FORK_F_RESOURCE,
                roles::F_THINK_FILE,
                roles::THINK_F,
                roles::F_EAT_FILE,
            ],
        ),
        (
            "FORK_G",
            [
                roles::G,
                roles::FORK_G,
                roles::G_WASI,
                roles::G_IO,
                roles::FORK_G_RESOURCE,
                roles::G_THINK_FILE,
                roles::THINK_G,
                roles::G_EAT_FILE,
            ],
        ),
    ];
    let layout = &layout[..count];
    if actors.len() != count
        || actors
            .iter()
            .zip(AGENTS)
            .any(|(actor, name)| actor["agent"] != name)
    {
        return Err("dining participant identities".into());
    }
    let journal = journal::read(records, &labels, actors)?;
    let receipts = journal.receipts(&dynamic)?;
    if dynamic["sessions"]
        .as_array()
        .ok_or("EPF sessions")?
        .iter()
        .any(|session| session["session"] != crate::philosopher_identity::SESSION)
    {
        return Err("different dining session".into());
    }
    let committed = |id: u64| -> Result<usize> {
        usize::try_from(
            receipts.get(&id).ok_or("frame receipt")?["receive_commit"]["journal_row"]
                .as_u64()
                .ok_or("receive commit observation")?,
        )
        .map_err(Into::into)
    };
    let forks: BTreeMap<_, _> = layout
        .iter()
        .map(|(name, ids)| (*name, (u64::from(ids[1]), u64::from(ids[4]))))
        .collect();
    let mut effects = BTreeMap::<u64, (usize, u64)>::new();
    let mut rejected_closes = 0;
    let mut read_results = 0;
    for (index, (_, ids)) in layout.iter().enumerate() {
        let [eat_owner, _, vm, _, _, think_view, think_owner, eat_view] = ids.map(u64::from);
        let agent = AGENTS[index];
        for (tool, view, owner, fd) in [
            ("think", think_view, think_owner, 4),
            ("eat", eat_view, eat_owner, 5),
        ] {
            let initial = journal::one(journal.frames.values().filter(|frame| {
                frame.direction(owner, vm)
                    && frame.logical_label == 10
                    && frame.bytes.first() == Some(&(view as u8))
            }))?;
            if initial.bytes.len() != 3 {
                return Err("file view was not transferred as an actual Endpoint".into());
            }
            let exit = journal::one(
                journal
                    .frames
                    .values()
                    .filter(|frame| frame.direction(view, owner) && frame.logical_label == 241),
            )?;
            if exit.bytes != 0i32.to_be_bytes() {
                return Err("file owner received a nonzero guest exit".into());
            }
            let mut requests = journal
                .frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(view, owner)
                        && matches!(
                            frame.logical_label,
                            protocol::LABEL_WASI_PATH_OPEN
                                | protocol::LABEL_WASI_FD_WRITE_OBJECT
                                | protocol::LABEL_WASI_FD_READ
                                | protocol::LABEL_WASI_FD_CLOSE_OBJECT
                                | protocol::LABEL_WASI_FD_CLOSE
                        )
                })
                .map(|(&id, frame)| Ok((committed(id)?, id, frame)))
                .collect::<Result<Vec<_>>>()?;
            let mut replies = journal
                .frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(owner, view)
                        && matches!(
                            frame.logical_label,
                            protocol::LABEL_WASI_PATH_OPEN_RET
                                | protocol::LABEL_WASI_FD_WRITE_OBJECT_RET
                                | protocol::LABEL_WASI_FD_READ_RET
                                | protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                                | protocol::LABEL_WASI_FD_CLOSE_REJECTED
                                | protocol::LABEL_WASI_FD_CLOSE_RET
                        )
                })
                .map(|(&id, frame)| Ok((committed(id)?, id, frame)))
                .collect::<Result<Vec<_>>>()?;
            requests.sort_by_key(|&(at, _, _)| at);
            replies.sort_by_key(|&(at, _, _)| at);
            if requests.len() != replies.len() {
                return Err("unpaired object I/O observations".into());
            }
            // Reconstruct descriptor data only. EPF owns the dynamic protocol
            // interpretation; no stage, next-operation or owner state is here.
            let mut bytes = Vec::<u8>::new();
            let mut offset = 0usize;
            let mut fd_flags = 0u16;
            let mut previous_reply = None;
            for ((at, request_id, request), (replied, reply_id, reply)) in
                requests.into_iter().zip(replies)
            {
                if at >= replied
                    || previous_reply.is_some_and(|previous| previous >= at)
                    || receipts[&request_id]["send_actor"] != vm
                    || receipts[&request_id]["receive_commit"]["actor"] != owner
                    || receipts[&reply_id]["send_actor"] != owner
                    || receipts[&reply_id]["receive_commit"]["actor"] != vm
                {
                    return Err("object I/O request/reply occurrence mismatch".into());
                }
                previous_reply = Some(replied);
                match (request.logical_label, reply.logical_label) {
                    (protocol::LABEL_WASI_PATH_OPEN, protocol::LABEL_WASI_PATH_OPEN_RET) => {
                        validate::<protocol::PathOpenReq>(request)?;
                        validate::<protocol::PathOpenedRet>(reply)?;
                        let errno = u16::from_be_bytes(reply.bytes[1..3].try_into()?);
                        if errno == 0 {
                            if reply.bytes[0] != fd
                                || request.bytes[0] != 3
                                || request.bytes[14..] != *format!("service/{tool}").as_bytes()
                            {
                                return Err("open was rebound to a different object".into());
                            }
                            if u16::from_be_bytes(request.bytes[9..11].try_into()?) & 8 != 0 {
                                bytes.clear();
                            }
                            fd_flags = u16::from_be_bytes(request.bytes[11..13].try_into()?);
                            offset = 0;
                        }
                    }
                    (
                        protocol::LABEL_WASI_FD_WRITE_OBJECT,
                        protocol::LABEL_WASI_FD_WRITE_OBJECT_RET,
                    ) => {
                        validate::<protocol::FdWriteObjectReq>(request)?;
                        validate::<protocol::FdWriteDoneRet>(reply)?;
                        if request.bytes[0] != fd || reply.bytes[0] != fd {
                            return Err("write reply belongs to a different fd".into());
                        }
                        let length = usize::from(reply.bytes[1]);
                        let errno = u16::from_be_bytes(reply.bytes[2..4].try_into()?);
                        if length > request.bytes.len() - 2 || (errno != 0 && length != 0) {
                            return Err("invalid accepted write prefix".into());
                        }
                        if errno == 0 {
                            let start = if fd_flags & 1 != 0 {
                                bytes.len()
                            } else {
                                offset
                            };
                            let end = start.checked_add(length).ok_or("write offset overflow")?;
                            bytes.resize(bytes.len().max(end), 0);
                            bytes[start..end].copy_from_slice(&request.bytes[2..2 + length]);
                            offset = end;
                        }
                    }
                    (protocol::LABEL_WASI_FD_READ, protocol::LABEL_WASI_FD_READ_RET) => {
                        validate::<protocol::FdReadReq>(request)?;
                        validate::<protocol::FdReadDoneRet>(reply)?;
                        let end = offset
                            .saturating_add(usize::from(request.bytes[1]))
                            .min(bytes.len());
                        if request.bytes[0] != fd
                            || reply.bytes[0] != fd
                            || reply.bytes[1..3] != [0, 0]
                            || bytes.get(offset..end) != Some(&reply.bytes[4..])
                        {
                            return Err("read bytes differ from the actual file/result".into());
                        }
                        if !bytes.is_empty() {
                            read_results += 1;
                        }
                        offset = end;
                    }
                    (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                        validate::<protocol::FdCloseReq>(request)?;
                        validate::<protocol::FdClosedRet>(reply)?;
                        if request.bytes != [fd] || reply.bytes != [fd, 0, 0] {
                            return Err("ordinary close fd/result mismatch".into());
                        }
                    }
                    (
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT,
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                        | protocol::LABEL_WASI_FD_CLOSE_REJECTED,
                    ) => {
                        validate::<protocol::FdCloseObjectReq>(request)?;
                        if request.bytes != [fd] || reply.bytes.first() != Some(&fd) {
                            return Err("object close fd mismatch".into());
                        }
                        let calls = journal
                            .calls
                            .iter()
                            .filter(|(_, call)| {
                                call.request["agent"] == agent
                                    && call.request["tool"] == tool
                                    && at < call.emitted
                                    && call.emitted < replied
                            })
                            .collect::<Vec<_>>();
                        let arguments = serde_json::from_slice::<Value>(&bytes).ok();
                        let valid = arguments.as_ref().is_some_and(|v| {
                            v.as_object()
                                .is_some_and(|v| v.get("philosopher").is_some_and(Value::is_string))
                        });
                        if reply.logical_label == protocol::LABEL_WASI_FD_CLOSE_REJECTED {
                            validate::<protocol::FdCloseRejectedRet>(reply)?;
                            if valid || !calls.is_empty() || reply.bytes != [fd, 0, 28] {
                                return Err(
                                    "rejected close admitted an effect or rejected valid arguments"
                                        .into(),
                                );
                            }
                            rejected_closes += 1;
                        } else {
                            validate::<protocol::FdClosedRet>(reply)?;
                            if reply.bytes != [fd, 0, 0] || !valid || calls.len() != 1 {
                                return Err("successful close lacks one admitted effect".into());
                            }
                            let (&call_id, call) = calls[0];
                            let (result_at, result) = call.result()?;
                            if result_at >= replied
                                || call.request["arguments"]
                                    != arguments.ok_or("invalid arguments")?
                                || effects.insert(call_id, (at, request_id)).is_some()
                            {
                                return Err(
                                    "file arguments/effect/result occurrence mismatch".into()
                                );
                            }
                            bytes = serde_json::to_vec(result)?;
                            offset = 0;
                        }
                    }
                    _ => return Err("object request and reply types differ".into()),
                }
            }
        }
    }
    if effects.len() != journal.calls.len() || effects.len() != 2 * count {
        return Err("unmatched or missing object effect occurrence".into());
    }
    let mut used = BTreeSet::new();
    let mut meals = Vec::new();
    let mut effect_receipts = Vec::new();
    for (index, (_, ids)) in layout.iter().enumerate() {
        let owner = u64::from(ids[0]);
        let agent = AGENTS[index];
        let assignment = &actors[index];
        for tool in ["think", "eat"] {
            let call =
                journal::one(journal.calls.values().filter(|call| {
                    call.request["agent"] == agent && call.request["tool"] == tool
                }))?;
            let id = call.request["id"].as_u64().ok_or("effect ID")?;
            let &(close_at, close_id) = effects.get(&id).ok_or("unbound effect")?;
            let (result_at, result) = call.result()?;
            let mut resources = Vec::new();
            if result["success"] != true || result["data"]["philosopher"] != agent {
                return Err("unexpected original dining result".into());
            }
            if tool == "eat" {
                let required = assignment["forks"]
                    .as_array()
                    .filter(|v| v.len() == 2)
                    .ok_or("two physical forks")?;
                let mut names = BTreeSet::new();
                for name in required {
                    let name = name.as_str().ok_or("fork name")?;
                    if !names.insert(name) {
                        return Err("same physical fork twice".into());
                    }
                    let &(fork_owner, resource) = forks.get(name).ok_or("unknown physical fork")?;
                    let find = |source, target, label| -> Result<(u64, &journal::Frame)> {
                        let sender = if source == resource {
                            if target == owner { fork_owner } else { owner }
                        } else {
                            source
                        };
                        let (&id, frame) = journal::one(journal.frames.iter().filter(
                            |(frame_id, frame)| {
                                frame.direction(source, target)
                                    && u64::from(frame.logical_label) == label
                                    && (label != 10
                                        || frame.bytes.first() == Some(&(resource as u8)))
                                    && receipts[*frame_id]["send_actor"] == sender
                            },
                        ))
                        .map_err(|error| {
                            format!(
                                "{agent}/{tool} frame {source}->{target}, label {label}: {error}"
                            )
                        })?;
                        Ok((id, frame))
                    };
                    let (intent_id, intent) = find(owner, fork_owner, resource)?;
                    let (request_id, request) = find(resource, owner, resource)?;
                    let (loan_id, loan) = find(fork_owner, owner, 10)?;
                    let (reply_id, reply) = find(resource, fork_owner, resource)?;
                    let (return_id, returned) = find(owner, fork_owner, 10)?;
                    if intent.value()? != call.request["arguments"]
                        || request.value()? != call.request["arguments"]
                        || reply.value()? != *result
                        || loan.bytes.len() != 3
                        || returned.bytes.len() != 3
                        || !(close_at < committed(intent_id)?
                            && committed(intent_id)? < committed(request_id)?
                            && committed(request_id)? < committed(loan_id)?
                            && committed(loan_id)? < call.emitted
                            && result_at < committed(reply_id)?
                            && committed(reply_id)? < committed(return_id)?)
                        || receipts[&request_id]["send_actor"] != fork_owner
                        || receipts[&request_id]["receive_commit"]["actor"] != owner
                        || receipts[&loan_id]["receive_commit"]["actor"] != owner
                        || receipts[&reply_id]["send_actor"] != owner
                        || receipts[&reply_id]["receive_commit"]["actor"] != fork_owner
                    {
                        return Err("effect lacks its two committed resource requests, Endpoints or actual replies".into());
                    }
                    for id in [intent_id, request_id, loan_id, reply_id, return_id] {
                        if !used.insert(id) {
                            return Err("resource occurrence reused by another effect".into());
                        }
                    }
                    resources.push(json!({"resource":resource,"intent_frame":intent_id,
                        "request_frame":request_id,"request_receive":receipts[&request_id]["receive_commit"],
                        "endpoint_frame":loan_id,"endpoint_receive":receipts[&loan_id]["receive_commit"],
                        "result_frame":reply_id,"return_frame":return_id}));
                }
                meals.push((call.started()?.1, call.ended()?.1, names));
            }
            let events = journal.score["events"]
                .as_array()
                .ok_or("original events")?
                .iter()
                .filter(|event| event["agent"] == agent && event["tool"] == tool)
                .collect::<Vec<_>>();
            let expected_args = if tool == "think" {
                json!({})
            } else {
                json!({"left_fork":assignment["forks"][0],"right_fork":assignment["forks"][1]})
            };
            if events.len() != 1
                || events[0]["args"] != expected_args
                || events[0]["result"] != result["data"]
                || events[0]["success"] != result["success"]
            {
                return Err("original simulator score and actual effect differ".into());
            }
            effect_receipts.push(json!({"effect_id":id,"agent":agent,"tool":tool,
                "object_frame":close_id,"object_receive":receipts[&close_id]["receive_commit"],
                "arguments":call.request["arguments"],"resources":resources}));
        }
    }
    for (&id, frame) in &journal.frames {
        if forks.values().any(|&(owner, resource)| {
            frame.identity["source"] == owner
                || frame.identity["target"] == owner
                || frame.identity["source"] == resource
                || frame.identity["target"] == resource
        }) && !used.contains(&id)
        {
            return Err("unused or excessive physical resource frame".into());
        }
    }
    let mut maximum_eaters = 0;
    for (index, (start, end, names)) in meals.iter().enumerate() {
        let mut overlaps = 0;
        for (other_index, (other_start, other_end, other_names)) in meals.iter().enumerate() {
            if index != other_index
                && start < other_end
                && other_start < end
                && !names.is_disjoint(other_names)
            {
                return Err("effects overlap on the same physical fork".into());
            }
            if other_start <= start && start < other_end {
                overlaps += 1;
            }
        }
        maximum_eaters = maximum_eaters.max(overlaps);
    }
    if journal.score["complete"] != true
        || journal.score["violations"] != json!([])
        || journal.score["events"]
            .as_array()
            .ok_or("score events")?
            .len()
            != 2 * count
        || maximum_eaters == 0
        || maximum_eaters > count / 2
    {
        return Err("incomplete dining outcome or invalid physical overlap".into());
    }
    Ok(
        json!({"task":fixture["task"],"kind":"bound_choreofs_resource_effect_receipts",
        "model_calls":null,"scored_benchmark":false,"status":"passed","participants":count,
        "domain_calls":journal.calls.len(),"carrier_frames":journal.frames.len(),
        "max_disjoint_eaters":maximum_eaters,"simulator_complete":true,
        "actual_resource_endpoint_requests":2*count,"actual_resource_endpoint_returns":2*count,
        "object_effect_receipts":effects.len(),"rejected_object_closes":rejected_closes,
        "effect_receipts":effect_receipts,
        "observed_result_reads":read_results,"violations":0,"guest_exits":actors.len(),
        "dynamic_protocol":dynamic,"original_test_delays":fixture["test_delays"]}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let baseline = audit(path)?;
    let projection = crate::projection::load(path)?;
    let records = journal::records(path)?;
    let labels = crate::projection::frames(&projection, &records)?;
    let first_eat = baseline["effect_receipts"]
        .as_array()
        .ok_or("effect receipts")?
        .iter()
        .find(|receipt| receipt["agent"] == "PHILOSOPHER_A" && receipt["tool"] == "eat")
        .ok_or("first eating effect receipt")?;
    fs::create_dir(output)?;
    let mut checked = Vec::new();
    for mutation in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_request",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_exit",
        "false_score",
        "effect_before_second_request",
        "consistent_unknown_private_role",
        "consistent_wrong_session",
        "coherent_arguments",
        "coherent_write_bytes",
        "coherent_read_result",
        "missing_object_close",
        "missing_capture",
        "missing_checkpoint",
        "duplicate_effect_fresh_id",
        "missing_resource_return",
    ] {
        let mut copy = records.clone();
        match mutation {
            "wrong_frame_label"
            | "wrong_lane"
            | "consistent_unknown_private_role"
            | "consistent_wrong_session" => {
                let frame = records
                    .iter()
                    .find(|row| row["message"]["type"] == "protocol_frame")
                    .ok_or("header mutation frame")?["message"]
                    .clone();
                let (field, replacement) = match mutation {
                    "wrong_frame_label" => ("frame_label", 65535),
                    "wrong_lane" => ("lane", (frame["lane"].as_u64().ok_or("lane")? + 1) % 256),
                    "consistent_unknown_private_role" => ("source", 254),
                    "consistent_wrong_session" => (
                        "session",
                        u64::from(crate::philosopher_identity::SESSION) + 1,
                    ),
                    _ => unreachable!(),
                };
                for row in &mut copy {
                    if row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == frame["frame_id"]
                    {
                        row["message"][field] = json!(replacement);
                    }
                }
            }
            "missing_request" | "missing_object_close" | "missing_resource_return" => {
                let id = match mutation {
                    "missing_request" => &first_eat["resources"][0]["request_frame"],
                    "missing_object_close" => &first_eat["object_frame"],
                    "missing_resource_return" => &first_eat["resources"][0]["return_frame"],
                    _ => unreachable!(),
                };
                let before = copy.len();
                copy.retain(|row| {
                    !(row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == *id)
                });
                if before == copy.len() {
                    return Err("missing frame mutation assignment".into());
                }
            }
            "altered_frame" => {
                let row = copy
                    .iter_mut()
                    .find(|row| {
                        row["message"]["type"] == "protocol_frame"
                            && row["message"]["action"] == "receive"
                    })
                    .ok_or("receive observation")?;
                row["message"]["source"] = json!(254);
            }
            "forged_arguments" => {
                let row = copy
                    .iter_mut()
                    .find(|row| {
                        row["direction"] == "runtime" && row["message"]["type"] == "execute"
                    })
                    .ok_or("effect observation")?;
                row["message"]["arguments"] = json!({"philosopher":42});
            }
            "coherent_arguments" => {
                let id = first_eat["effect_id"].as_u64().ok_or("effect ID")?;
                for row in &mut copy {
                    if row["message"]["id"] == id
                        && ((row["direction"] == "runtime" && row["message"]["type"] == "execute")
                            || row["direction"] == "domain_start")
                    {
                        row["message"]["arguments"]["philosopher"] =
                            json!("UNADMITTED_DISPLAY_NAME");
                    }
                }
            }
            "coherent_write_bytes" | "coherent_read_result" => {
                let label = if mutation == "coherent_write_bytes" {
                    protocol::LABEL_WASI_FD_WRITE_OBJECT
                } else {
                    protocol::LABEL_WASI_FD_READ_RET
                };
                let frame = records
                    .iter()
                    .find(|row| {
                        let m = &row["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["frame_id"]
                                .as_u64()
                                .is_some_and(|id| labels.get(&id) == Some(&label))
                            && m["payload"].as_array().is_some_and(|bytes| {
                                bytes.len() > 5 && matches!(bytes[0].as_u64(), Some(4 | 5))
                            })
                    })
                    .ok_or("file data mutation frame")?["message"]
                    .clone();
                let offset = if mutation == "coherent_write_bytes" {
                    2
                } else {
                    4
                };
                let byte = frame["payload"][offset].as_u64().ok_or("file byte")?;
                for row in &mut copy {
                    if row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == frame["frame_id"]
                    {
                        row["message"]["payload"][offset] = json!(byte ^ 1);
                    }
                }
            }
            "forged_result" => {
                let row = copy
                    .iter_mut()
                    .find(|row| {
                        row["direction"] == "simulator"
                            && row["message"]["type"] == "simulator_result"
                    })
                    .ok_or("original effect result")?;
                row["message"]["result"]["data"] = json!({"forged":true});
            }
            "missing_exit" | "missing_capture" | "missing_checkpoint" => {
                let kind = match mutation {
                    "missing_exit" => "exit",
                    "missing_capture" => "protocol_capture",
                    "missing_checkpoint" => "protocol_checkpoint",
                    _ => unreachable!(),
                };
                let index = copy
                    .iter()
                    .position(|row| row["message"]["type"] == kind)
                    .ok_or("missing record mutation assignment")?;
                copy.remove(index);
            }
            "false_score" => {
                let row = copy
                    .iter_mut()
                    .find(|row| row["message"]["type"] == "score")
                    .ok_or("score observation")?;
                row["message"]["complete"] = json!(false);
            }
            "effect_before_second_request" => {
                let before = first_eat["resources"]
                    .as_array()
                    .ok_or("two resource receipts")?
                    .iter()
                    .filter_map(|resource| resource["request_receive"]["journal_row"].as_u64())
                    .max()
                    .ok_or("second resource receipt")?;
                let before = before as usize;
                let old = copy
                    .iter()
                    .position(|row| {
                        row["direction"] == "runtime"
                            && row["message"]["type"] == "execute"
                            && row["message"]["id"] == first_eat["effect_id"]
                    })
                    .ok_or("effect observation")?;
                if before >= old {
                    return Err("baseline effect precedes its resource receipt".into());
                }
                let row = copy.remove(old);
                copy.insert(before, row);
            }
            "duplicate_effect_fresh_id" => {
                let id = first_eat["effect_id"].as_u64().ok_or("effect ID")?;
                let fresh = records
                    .iter()
                    .filter_map(|row| row["message"]["id"].as_u64())
                    .max()
                    .ok_or("RPC IDs")?
                    .checked_add(1)
                    .ok_or("fresh RPC ID overflow")?;
                let mut extra = records
                    .iter()
                    .filter(|row| {
                        row["message"]["id"] == id
                            && ((row["direction"] == "runtime"
                                && row["message"]["type"] == "execute")
                                || row["direction"] == "domain_start"
                                || row["direction"] == "domain_end"
                                || (row["direction"] == "simulator"
                                    && row["message"]["type"] == "simulator_result"))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if extra.len() != 4 {
                    return Err("complete effect lifecycle mutation assignment".into());
                }
                for row in &mut extra {
                    row["message"]["id"] = json!(fresh);
                }
                let before = records
                    .iter()
                    .position(|row| {
                        row["direction"] == "simulator"
                            && row["message"]["type"] == "simulator_result"
                            && row["message"]["id"] == id
                    })
                    .ok_or("effect result observation")?
                    + 1;
                copy.splice(before..before, extra);
            }
            _ => unreachable!(),
        }
        // These mutations keep the complete source execution valid. Their
        // rejection demonstrates the additional, application-data obligation.
        let epf_still_valid = matches!(
            mutation,
            "coherent_arguments"
                | "coherent_write_bytes"
                | "coherent_read_result"
                | "effect_before_second_request"
                | "duplicate_effect_fresh_id"
        );
        if epf_still_valid {
            crate::projection::check_dynamic(&projection, &copy)?;
        }
        let mutant_path = output.join(format!("{mutation}.jsonl"));
        let mut file = std::io::BufWriter::new(File::create_new(&mutant_path)?);
        for row in &copy {
            serde_json::to_writer(&mut file, row)?;
            writeln!(file)?;
        }
        file.flush()?;
        let error = check(&copy, &projection)
            .err()
            .ok_or_else(|| format!("audit accepted {mutation}"))?
            .to_string();
        checked.push(json!({"mutation":mutation,"rejected":true,"epf_execution_preserved":epf_still_valid.then_some(true),"error":error}));
    }
    let result = json!({"mutants":checked,"original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok(result)
}
