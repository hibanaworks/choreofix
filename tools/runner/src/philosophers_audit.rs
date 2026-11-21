//! Bind EPF-checked receive occurrences to actual file data, effects and replies.
//! Source eligibility and affine custody are checked by EPF, not reimplemented.
use crate::journal::file::FileContents;
use crate::{evidence::Result, journal};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    journal.check_session(&json!(crate::philosopher_identity::SESSION))?;
    let dynamic = journal.execution;
    let projection = journal.projection;
    use crate::philosopher_identity::{AGENTS, THINK_COMPLETED, THINK_REQUEST, roles};
    let fixture = journal.fixture();
    if fixture["terminal_review_failure"] != false {
        return Err("different task or invalid dining configuration".into());
    }
    let actors = journal.participants;
    let count = actors.len();
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
                roles::A_EAT_CLOSE,
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
                roles::B_EAT_CLOSE,
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
                roles::C_EAT_CLOSE,
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
                roles::D_EAT_CLOSE,
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
                roles::E_EAT_CLOSE,
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
                roles::F_EAT_CLOSE,
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
                roles::G_EAT_CLOSE,
            ],
        ),
    ];
    let layout = &layout[..count];
    if actors
        .iter()
        .zip(AGENTS)
        .any(|(actor, name)| actor["agent"] != name)
    {
        return Err("dining participant identities".into());
    }
    let table = u64::from(roles::TABLE);
    let requests_role = u64::from(roles::MEAL_REQUESTS);
    let programs = projection["compiled"]["programs"]
        .as_array()
        .ok_or("compiled dining programs")?;
    let selects_meals = programs.iter().any(|program| program["role"] == table);
    let grants_to_forks = crate::projection::contracts(&projection["compiled"])?
        .values()
        .any(|header| header.source == roles::TABLE && header.target == roles::FORK_A);
    let forks: BTreeMap<_, _> = layout
        .iter()
        .map(|(name, ids)| (*name, (u64::from(ids[1]), u64::from(ids[4]))))
        .collect();
    let mut effects = BTreeMap::<u64, (u64, Option<u64>)>::new();
    let mut rejected_closes = 0;
    let mut read_results = 0;
    for (index, (_, ids)) in layout.iter().enumerate() {
        let [
            eat_owner,
            _,
            vm,
            _,
            _,
            think_view,
            think_owner,
            eat_view,
            eat_close,
        ] = ids.map(u64::from);
        let agent = AGENTS[index];
        for (tool, view, owner, controller, fd) in [
            ("think", think_view, think_owner, think_owner, 4),
            ("eat", eat_view, eat_owner, eat_close, 5),
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
            let authority_id = if tool == "think" {
                // This file sends every close directly to its permanent effect owner.
                None
            } else {
                let (&id, _) = journal::one(journal.frames.iter().filter(|(_, frame)| {
                    frame.direction(owner, view)
                        && frame.logical_label == 165
                        && frame.bytes.first().copied().map(u64::from) == Some(controller)
                }))?;
                if journal.frames[&id].receipt["receive_commit"]["actor"] != vm {
                    return Err("close authority was not received by the actual guest".into());
                }
                Some(id)
            };
            let exchanges = crate::journal::file::exchanges(
                &journal.frames,
                |frame| {
                    (frame.direction(view, owner)
                        && matches!(
                            frame.logical_label,
                            protocol::LABEL_WASI_PATH_OPEN
                                | protocol::LABEL_WASI_FD_WRITE_OBJECT
                                | protocol::LABEL_WASI_FD_READ
                                | protocol::LABEL_WASI_FD_CLOSE
                        ))
                        || (frame.direction(view, controller)
                            && (frame.logical_label == protocol::LABEL_WASI_FD_CLOSE_OBJECT
                                || (tool == "think" && frame.logical_label == THINK_REQUEST)))
                },
                |frame| {
                    (frame.direction(owner, view)
                        && matches!(
                            frame.logical_label,
                            protocol::LABEL_WASI_PATH_OPEN_RET
                                | protocol::LABEL_WASI_FD_WRITE_OBJECT_RET
                                | protocol::LABEL_WASI_FD_READ_RET
                                | protocol::LABEL_WASI_FD_CLOSE_RET
                        ))
                        || (frame.direction(controller, view)
                            && matches!(
                                frame.logical_label,
                                protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                                    | protocol::LABEL_WASI_FD_CLOSE_REJECTED
                            ))
                        || (tool == "think"
                            && frame.direction(controller, view)
                            && frame.logical_label == THINK_COMPLETED)
                },
            )?;
            // Reconstruct descriptor data only. EPF owns the dynamic protocol
            // interpretation; no stage, next-operation or owner state is here.
            let mut contents = FileContents::new(fd);
            for ((request_id, request), (reply_id, reply)) in exchanges {
                let at = request.received;
                let replied = reply.received;
                if journal.frames[&request_id].receipt["send_actor"] != vm
                    || journal.frames[&request_id].receipt["receive_commit"]["actor"] != owner
                    || journal.frames[&reply_id].receipt["send_actor"] != owner
                    || journal.frames[&reply_id].receipt["receive_commit"]["actor"] != vm
                {
                    return Err("object I/O request/reply occurrence mismatch".into());
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
                        if !contents.bytes.is_empty() {
                            read_results += 1;
                        }
                    }
                    (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                        contents.close(request, reply)?;
                    }
                    (THINK_REQUEST, THINK_COMPLETED) => {
                        let arguments = request.value()?;
                        if request.bytes != contents.bytes
                            || !arguments.get("philosopher").is_some_and(Value::is_string)
                            || !reply.bytes.is_empty()
                            || authority_id.is_some()
                        {
                            return Err(
                                "typed thought differs from the file contents or completion".into(),
                            );
                        }
                        let (&call_id, call) =
                            journal::one(journal.calls.iter().filter(|(_, call)| {
                                call.request["agent"] == agent
                                    && call.request["tool"] == tool
                                    && at < call.emitted
                                    && call.emitted < replied
                            }))?;
                        let (result_at, result) = call.result;
                        if result_at >= reply.sent
                            || call.request["arguments"] != arguments
                            || effects.insert(call_id, (request_id, None)).is_some()
                        {
                            return Err(
                                "thought request, effect and acknowledgement mismatch".into()
                            );
                        }
                        contents.bytes = serde_json::to_vec(result)?;
                        contents.offset = 0;
                    }
                    (
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT,
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                        | protocol::LABEL_WASI_FD_CLOSE_REJECTED,
                    ) => {
                        request.validate::<protocol::FdCloseObjectReq>()?;
                        if request.bytes != [fd] || reply.bytes.first() != Some(&fd) {
                            return Err("object close fd mismatch".into());
                        }
                        // close_with completes the guest syscall only after its
                        // returned Endpoint arrives. In an arbitrated meal the
                        // initial fd acknowledgement precedes fork acquisition.
                        let arguments = serde_json::from_slice::<Value>(&contents.bytes).ok();
                        let (effect_after, completion) = if selects_meals
                            && tool == "eat"
                            && reply.logical_label == protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET
                        {
                            let (ready_id, ready) =
                                journal::one(journal.frames.iter().filter(|(_, f)| {
                                    f.direction(controller, requests_role)
                                        && u64::from(f.logical_label) == controller
                                }))?;
                            let ready_at = journal.frames[ready_id].received;
                            let args = ready.value()?;
                            let granted_at = if grants_to_forks {
                                let mut latest = ready_at;
                                let pair = actors[index]["forks"]
                                    .as_array()
                                    .filter(|v| v.len() == 2)
                                    .ok_or("two assigned physical forks")?;
                                for (side, name) in pair.iter().enumerate() {
                                    let &(fork, resource) = forks
                                        .get(name.as_str().ok_or("fork name")?)
                                        .ok_or("unknown physical fork")?;
                                    let label = if side == 0 { fork } else { resource };
                                    let (id, grant) =
                                        journal::one(journal.frames.iter().filter(|(_, f)| {
                                            f.direction(table, fork)
                                                && u64::from(f.logical_label) == label
                                        }))?;
                                    if grant.value()? != args || grant.sent <= ready_at {
                                        return Err(
                                            "fork grant precedes or differs from its meal request"
                                                .into(),
                                        );
                                    }
                                    latest = latest.max(journal.frames[id].received);
                                }
                                latest
                            } else {
                                let (grant_id, grant) =
                                    journal::one(journal.frames.iter().filter(|(_, f)| {
                                        f.direction(table, controller)
                                            && u64::from(f.logical_label) == controller
                                    }))?;
                                if grant.value()? != args || grant.sent <= ready_at {
                                    return Err(
                                        "diner grant precedes or differs from its meal request"
                                            .into(),
                                    );
                                }
                                journal.frames[grant_id].received
                            };
                            let returned_at = journal.frames
                                [&authority_id.ok_or("missing meal Endpoint")?]
                                .received;
                            if args != *arguments.as_ref().ok_or("malformed admitted meal")?
                                || at >= ready_at
                                || ready_at >= granted_at
                                || replied >= returned_at
                            {
                                return Err(
                                    "meal request and grant differ in occurrence or arguments"
                                        .into(),
                                );
                            }
                            (granted_at, returned_at)
                        } else {
                            (at, replied)
                        };
                        let calls = journal
                            .calls
                            .iter()
                            .filter(|(_, call)| {
                                call.request["agent"] == agent
                                    && call.request["tool"] == tool
                                    && effect_after < call.emitted
                                    && call.emitted < completion
                            })
                            .collect::<Vec<_>>();
                        let valid = arguments.as_ref().is_some_and(|v| {
                            v.as_object()
                                .is_some_and(|v| v.get("philosopher").is_some_and(Value::is_string))
                        });
                        if reply.logical_label == protocol::LABEL_WASI_FD_CLOSE_REJECTED {
                            reply.validate::<protocol::FdCloseRejectedRet>()?;
                            if valid || !calls.is_empty() || reply.bytes != [fd, 0, 28] {
                                return Err(
                                    "rejected close admitted an effect or rejected valid arguments"
                                        .into(),
                                );
                            }
                            rejected_closes += 1;
                        } else {
                            reply.validate::<protocol::FdClosedRet>()?;
                            if reply.bytes != [fd, 0, 0] || !valid || calls.len() != 1 {
                                return Err("successful close lacks one admitted effect".into());
                            }
                            let (&call_id, call) = calls[0];
                            let (result_at, result) = call.result;
                            if result_at >= completion
                                || call.request["arguments"]
                                    != arguments.ok_or("invalid arguments")?
                                || effects
                                    .insert(call_id, (request_id, authority_id))
                                    .is_some()
                            {
                                return Err(
                                    "file arguments/effect/result occurrence mismatch".into()
                                );
                            }
                            contents.bytes = serde_json::to_vec(result)?;
                            contents.offset = 0;
                        }
                    }
                    _ => return Err("object request and reply types differ".into()),
                }
            }
        }
    }
    if effects.len() != journal.calls.len() {
        return Err("unmatched or missing object effect occurrence".into());
    }
    let mut used = BTreeSet::new();
    let mut meals = Vec::new();
    let mut effect_receipts = Vec::new();
    for (index, (_, ids)) in layout.iter().enumerate() {
        let owner = u64::from(ids[0]);
        let controller = u64::from(ids[8]);
        let agent = AGENTS[index];
        let assignment = &actors[index];
        for tool in ["think", "eat"] {
            let mut calls: Vec<_> = journal
                .calls
                .values()
                .filter(|call| call.request["agent"] == agent && call.request["tool"] == tool)
                .collect();
            calls.sort_by_key(|call| call.emitted);
            let events: Vec<_> = journal
                .scored_events
                .iter()
                .filter(|event| event["agent"] == agent && event["tool"] == tool)
                .collect();
            if calls.is_empty()
                || calls.len() != events.len()
                || (tool == "eat" && calls.len() != 1)
            {
                return Err(
                    "dining effect count differs from the required meals or recorded thoughts"
                        .into(),
                );
            }
            for (call, event) in calls.into_iter().zip(events) {
                let id = call.request["id"].as_u64().ok_or("effect ID")?;
                let &(close_id, authority_id) = effects.get(&id).ok_or("unbound effect")?;
                let (result_at, result) = call.result;
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
                    for (side, name) in required.iter().enumerate() {
                        let name = name.as_str().ok_or("fork name")?;
                        if !names.insert(name) {
                            return Err("same physical fork twice".into());
                        }
                        let &(fork_owner, resource) =
                            forks.get(name).ok_or("unknown physical fork")?;
                        let receiver = if grants_to_forks && side == 1 {
                            owner
                        } else {
                            controller
                        };
                        let find = |source, target, label| -> Result<(u64, &journal::Frame)> {
                            let sender = if source == controller {
                                owner
                            } else if source == resource {
                                if target == receiver {
                                    fork_owner
                                } else {
                                    owner
                                }
                            } else {
                                source
                            };
                            let (&id, frame) = journal::one(journal.frames.iter().filter(
                            |(frame_id, frame)| {
                                frame.direction(source, target)
                                    && u64::from(frame.logical_label) == label
                                    && journal.frames[*frame_id].receipt["send_actor"] == sender
                            },
                        ))
                        .map_err(|error| {
                            format!(
                                "{agent}/{tool} frame {source}->{target}, label {label}: {error}"
                            )
                        })?;
                            Ok((id, frame))
                        };
                        let (intent_id, intent) = if grants_to_forks {
                            find(
                                table,
                                fork_owner,
                                if side == 0 { fork_owner } else { resource },
                            )?
                        } else {
                            find(
                                controller,
                                fork_owner,
                                if side == 0 { fork_owner } else { resource },
                            )?
                        };
                        let (request_id, request) = find(resource, receiver, resource)?;
                        let (loan_id, _) = find(fork_owner, receiver, 10)?;
                        let (reply_id, reply) = find(resource, fork_owner, resource)?;
                        let (return_id, _) = find(receiver, fork_owner, 10)?;
                        if intent.value()? != call.request["arguments"]
                            || request.value()? != call.request["arguments"]
                            || reply.value()? != *result
                            || !(journal.frames[&request_id].received < call.emitted
                                && journal.frames[&loan_id].received < call.emitted
                                && result_at < journal.frames[&reply_id].received)
                            || journal.frames[&request_id].receipt["send_actor"] != fork_owner
                            || journal.frames[&request_id].receipt["receive_commit"]["actor"]
                                != owner
                            || journal.frames[&loan_id].receipt["receive_commit"]["actor"] != owner
                            || journal.frames[&reply_id].receipt["send_actor"] != owner
                            || journal.frames[&reply_id].receipt["receive_commit"]["actor"]
                                != fork_owner
                        {
                            return Err("effect lacks its two committed resource requests, Endpoints or actual replies".into());
                        }
                        for id in [intent_id, request_id, loan_id, reply_id, return_id] {
                            if !used.insert(id) {
                                return Err("resource occurrence reused by another effect".into());
                            }
                        }
                        resources.push(json!({"resource":resource,"intent_frame":intent_id,
                        "request_frame":request_id,"request_receive":journal.frames[&request_id].receipt["receive_commit"],
                        "endpoint_frame":loan_id,"endpoint_receive":journal.frames[&loan_id].receipt["receive_commit"],
                        "result_frame":reply_id,"return_frame":return_id}));
                    }
                    meals.push((call.started.1, call.ended.1, names));
                }
                let expected_args = if tool == "think" {
                    json!({})
                } else {
                    json!({"left_fork":assignment["forks"][0],"right_fork":assignment["forks"][1]})
                };
                if !call.matches_score(event, &expected_args) {
                    return Err("original simulator score and actual effect differ".into());
                }
                effect_receipts.push(json!({"effect_id":id,"agent":agent,"tool":tool,
                "object_frame":close_id,"object_receive":journal.frames[&close_id].receipt["receive_commit"],
                "close_authority_frame":authority_id,"close_authority_receive":authority_id.map(|id| &journal.frames[&id].receipt["receive_commit"]),
                "arguments":call.request["arguments"],"resources":resources}));
            }
        }
    }
    for (&id, frame) in &journal.frames {
        if forks.values().any(|&(owner, resource)| {
            frame.source == owner
                || frame.target == owner
                || frame.source == resource
                || frame.target == resource
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
    if !journal.simulator_complete
        || journal.scored_events.len() != effects.len()
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

pub fn check_mutants(path: &Path, output: &Path) -> Result<(Value, Value)> {
    let evidence = crate::journal::Evidence::open(path)?;
    let task = evidence.projection["compiled"]["task"]
        .as_str()
        .ok_or("dining task")?;
    if !matches!(task, "9E" | "9M" | "9H") {
        return Err("expected dining evidence".into());
    }
    let baseline = evidence.audit(task)?;
    let projection = evidence.projection;
    let records = evidence.rows;
    let labels = crate::projection::frames(&projection, &records)?;
    let first_eat = baseline["effect_receipts"]
        .as_array()
        .ok_or("effect receipts")?
        .iter()
        .find(|receipt| receipt["agent"] == "PHILOSOPHER_A" && receipt["tool"] == "eat")
        .ok_or("first eating effect receipt")?;
    let close_at = records
        .iter()
        .position(|row| {
            row["message"]["type"] == "protocol_frame"
                && row["message"]["action"] == "send"
                && row["message"]["frame_id"] == first_eat["object_frame"]
        })
        .ok_or("admitted close frame")?;
    let close = &records[close_at]["message"];
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
        "missing_close_authority",
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
            "missing_request"
            | "missing_object_close"
            | "missing_resource_return"
            | "missing_close_authority" => {
                let id = match mutation {
                    "missing_request" => &first_eat["resources"][0]["request_frame"],
                    "missing_object_close" => &first_eat["object_frame"],
                    "missing_resource_return" => &first_eat["resources"][0]["return_frame"],
                    "missing_close_authority" => &first_eat["close_authority_frame"],
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
                            && row["message"]["action"] == "send"
                    })
                    .ok_or("transport acceptance")?;
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
                let matches = |row: &&Value| {
                    let m = &row["message"];
                    m["type"] == "protocol_frame"
                        && m["action"] == "send"
                        && m[if mutation == "coherent_write_bytes" {
                            "source"
                        } else {
                            "target"
                        }] == close["source"]
                        && m["frame_id"].as_u64().is_some_and(|id| {
                            labels.get(&id).is_some_and(|wire| wire.label == label)
                        })
                        && m["payload"]
                            .as_array()
                            .is_some_and(|bytes| bytes.len() > 5 && bytes[0] == close["payload"][0])
                };
                // Mutate data used by this effect, rather than an earlier
                // rejected attempt whose bytes were subsequently overwritten.
                let frame = if mutation == "coherent_write_bytes" {
                    records[..close_at].iter().rev().find(matches)
                } else {
                    records[close_at + 1..].iter().find(matches)
                }
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
        let mutant_path = output.join(format!("{mutation}.jsonl"));
        let mut file = std::io::BufWriter::new(File::create_new(&mutant_path)?);
        for row in &copy {
            serde_json::to_writer(&mut file, row)?;
            writeln!(file)?;
        }
        file.flush()?;
        drop(file);
        let error = journal::reject_mutant(
            path,
            &mutant_path,
            baseline["task"].as_str().ok_or("dining task")?,
        )?;
        if epf_still_valid {
            // Release the complete search workspace after each input, just as
            // the effect auditor does. No source implementation is restored.
            #[cfg(test)]
            let auditor =
                std::env::var_os("CHOREOFIX_AUDITOR").ok_or("qualified auditor missing")?;
            #[cfg(not(test))]
            let auditor = std::env::current_exe()?;
            let checked = Command::new(auditor)
                .arg("--audit-protocol")
                .arg(&mutant_path)
                .stdout(Stdio::null())
                .output()?;
            if !checked.status.success() {
                return Err(format!(
                    "effect-data counterexample changed source execution: {}",
                    String::from_utf8(checked.stderr)?
                )
                .into());
            }
        }
        checked.push(json!({"mutation":mutation,"rejected":true,
            "sha256":crate::evidence::digest(&mutant_path)?,
            "epf_execution_preserved":epf_still_valid.then_some(true),"error":error}));
        fs::remove_file(&mutant_path)?;
    }
    let result = json!({"mutants":checked,"original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok((baseline, result))
}
