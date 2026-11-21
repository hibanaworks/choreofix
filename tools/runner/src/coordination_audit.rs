//! EPF owns execution/custody; this reader checks file bytes and actual effects.
use crate::journal::file::{FileContents, same_path};
use crate::{evidence::Result, journal};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn number(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .filter(|n| *n <= 255)
        .ok_or_else(|| "invalid compiled file field".into())
}

pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = journal.execution;
    let compiled = &journal.projection["compiled"];
    let actors = journal.participants;

    let role = |name: &str| number(&compiled["roles"][name]);
    let label =
        |name: &str| -> Result<u8> { Ok(u8::try_from(number(&compiled["labels"][name])?)?) };
    let store = role("STORE")?;
    let repo = role("REPO")?;
    let environment = role("TEST_ENV")?;
    let frames = &journal.frames;
    if frames
        .values()
        .any(|f| f.identity["session"] != compiled["session"])
        || !journal.simulator_complete
        || journal.calls.is_empty()
    {
        return Err("coordination completion or session differs".into());
    }
    let contract_label = label("Contract")?;
    let ready_label = label("Ready")?;
    let candidate_before = |before: usize| -> Result<Value> {
        let latest = |agent: &str, tool: &str| -> Result<Value> {
            let call = journal
                .calls
                .values()
                .filter(|c| {
                    c.request["agent"] == agent && c.request["tool"] == tool && c.result.0 < before
                })
                .max_by_key(|c| c.result.0)
                .ok_or("missing actual candidate input")?;
            Ok(call.result.1.clone())
        };
        Ok(
            json!({"api":latest("BACKEND_DEV","define_api")?,"backend":latest("BACKEND_DEV","implement_feature")?,"frontend":latest("FRONTEND_DEV","implement_feature")?}),
        )
    };
    let mut used_calls = BTreeSet::new();
    let mut used_frames = BTreeSet::new();
    let mut completed_files = BTreeSet::new();
    let mut accepted_closes = 0;
    let mut rejected_closes = 0;
    let mut resource_receipts = BTreeMap::<u64, usize>::new();
    let mut scored = journal.scored_events.iter().collect::<Vec<_>>();
    let operations = compiled["operation_files"]
        .as_array()
        .ok_or("compiled operation files")?;
    if operations.len() != 5 {
        return Err("operation binding population differs".into());
    }
    let mut operation_names = BTreeSet::new();
    for binding in operations {
        let name = binding["name"].as_str().ok_or("operation name")?;
        if !matches!(
            name,
            "DEFINE" | "BACKEND" | "FRONTEND" | "AGREEMENT" | "TESTER"
        ) || !operation_names.insert(name)
        {
            return Err("unknown or repeated operation binding".into());
        }
        let file = number(&binding["file"])?;
        let metadata = number(&binding["metadata"])?;
        let control = number(&binding["control"])?;
        let guest = number(&binding["guest"])?;
        let fd = u8::try_from(number(&binding["fd"])?)?;
        let path = binding["path"].as_str().ok_or("operation path")?;
        let agent = binding["agent"].as_str().ok_or("operation agent")?;
        let actor = journal::one(actors.iter().filter(|a| a["agent"] == agent))?;
        let tool = binding["tool"].as_str();
        let assigned = tool
            .map(|tool| {
                journal::one(
                    actor["calls"]
                        .as_array()
                        .ok_or("original assignments")?
                        .iter()
                        .filter(|c| c["tool"] == tool),
                )
            })
            .transpose()?;
        let mut requests = frames
            .iter()
            .filter(|(_, f)| {
                (f.direction(file, metadata)
                    && matches!(
                        f.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT
                            | protocol::LABEL_WASI_FD_READ
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
                (f.direction(metadata, file)
                    && matches!(
                        f.logical_label,
                        protocol::LABEL_WASI_PATH_OPEN_RET
                            | protocol::LABEL_WASI_FD_WRITE_OBJECT_RET
                            | protocol::LABEL_WASI_FD_READ_RET
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
        if requests.len() != replies.len() || requests.is_empty() {
            return Err("unpaired file operations".into());
        }
        let mut contents = FileContents::new(fd);
        for ((_, request_id, request), (_, reply_id, reply)) in requests.into_iter().zip(replies) {
            let request_at = request.received;
            let reply_at = frames[&reply_id].first_poll;
            let receiver = metadata;
            if journal.frames[&request_id].receipt["send_actor"] != guest
                || journal.frames[&request_id].receipt["receive_commit"]["actor"] != receiver
                || journal.frames[&reply_id].receipt["send_actor"] != receiver
                || journal.frames[&reply_id].receipt["receive_commit"]["actor"] != guest
                || reply_at <= request.received
            {
                return Err(
                    "file request/reply crossed its actual participant or occurrence".into(),
                );
            }
            match (request.logical_label, reply.logical_label) {
                (protocol::LABEL_WASI_PATH_OPEN, protocol::LABEL_WASI_PATH_OPEN_RET) => {
                    contents.open(request, reply, |actual| same_path(actual, path))?;
                }
                (
                    protocol::LABEL_WASI_FD_WRITE_OBJECT,
                    protocol::LABEL_WASI_FD_WRITE_OBJECT_RET,
                ) => {
                    contents.write(request, reply)?;
                }
                (protocol::LABEL_WASI_FD_READ, protocol::LABEL_WASI_FD_READ_RET) => {
                    contents.read(request, reply)?;
                }
                (protocol::LABEL_WASI_FD_CLOSE, protocol::LABEL_WASI_FD_CLOSE_RET) => {
                    contents.close(request, reply)?;
                }
                (protocol::LABEL_WASI_FD_CLOSE_OBJECT, protocol::LABEL_WASI_FD_CLOSE_REJECTED) => {
                    request.validate::<protocol::FdCloseObjectReq>()?;
                    reply.validate::<protocol::FdCloseRejectedRet>()?;
                    let value = serde_json::from_slice::<Value>(&contents.bytes).ok();
                    let valid = if let Some(original) = assigned {
                        value.as_ref() == Some(&original["arguments"])
                    } else {
                        value.as_ref().is_some_and(|v| match name {
                            "AGREEMENT" => {
                                v.as_object().is_some_and(|o| o.len() == 2)
                                    && v["api_name"] == "feature_api_spec"
                                    && matches!(
                                        v["first"].as_str(),
                                        Some("BACKEND_DEV" | "FRONTEND_DEV")
                                    )
                            }
                            _ => false,
                        })
                    };
                    if request.bytes != [fd] || reply.bytes != [fd, 0, 28] || valid {
                        return Err("invalid rejected file close".into());
                    }
                    rejected_closes += 1;
                }
                (
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT,
                    protocol::LABEL_WASI_FD_CLOSE_OBJECT_RET,
                ) => {
                    request.validate::<protocol::FdCloseObjectReq>()?;
                    reply.validate::<protocol::FdClosedRet>()?;
                    if request.bytes != [fd] || reply.bytes != [fd, 0, 0] {
                        return Err("invalid accepted close fd/result".into());
                    }
                    let arguments: Value = serde_json::from_slice(&contents.bytes)?;
                    let result = if let Some(tool) = tool {
                        let (&call_id, call) =
                            journal::one(journal.calls.iter().filter(|(_, c)| {
                                c.request["agent"] == agent
                                    && c.request["tool"] == tool
                                    && c.emitted > request_at
                                    && c.result.0 < reply_at
                            }))?;
                        let (result_at, result) = call.result;
                        if !used_calls.insert(call_id)
                            || arguments != call.request["arguments"]
                            || assigned.ok_or("missing original assignment")?["arguments"]
                                != arguments
                            || result["success"] != true
                        {
                            return Err(
                                "accepted file input, effect or original result differs".into()
                            );
                        }
                        let (intent_name, resource_rows): (&str, Vec<(u64, &str, &str)>) =
                            match name {
                                "DEFINE" => {
                                    ("DefineIntent", vec![(repo, "DefineApi", "ApiDefined")])
                                }
                                "BACKEND" => (
                                    "BackendIntent",
                                    vec![(repo, "ImplementBackend", "BackendImplemented")],
                                ),
                                "FRONTEND" => (
                                    "FrontendIntent",
                                    vec![(repo, "ImplementFrontend", "FrontendImplemented")],
                                ),
                                "TESTER" => (
                                    "TestIntent",
                                    vec![
                                        (repo, "ReadForTests", "RepositoryTested"),
                                        (environment, "RunTests", "EnvironmentTested"),
                                    ],
                                ),
                                _ => return Err("unknown operation binding".into()),
                            };
                        let intent_label = label(intent_name)?;
                        let (&intent_id, intent) = journal::one(frames.iter().filter(|(_, f)| {
                            f.direction(control, store)
                                && f.logical_label == intent_label
                                && f.received > request_at
                                && f.received < call.emitted
                        }))?;
                        let input = intent.value()?;
                        if name == "TESTER" {
                            let candidate = candidate_before(call.emitted)?;
                            if input != arguments {
                                return Err(
                                    "test input differs from current actual candidate".into()
                                );
                            }
                            let (&ready_id, ready) = frames
                                .iter()
                                .filter(|(_, f)| {
                                    f.direction(store, file)
                                        && f.logical_label == ready_label
                                        && f.value().is_ok_and(|v| v == candidate)
                                        && f.received < request_at
                                })
                                .max_by_key(|(_, f)| f.received)
                                .ok_or("missing current candidate delivery")?;
                            if ready.value()? != candidate || !used_frames.insert(ready_id) {
                                return Err("test candidate delivery differs or is reused".into());
                            }
                        } else if input != arguments {
                            return Err("admitted file input differs from intent".into());
                        }
                        if !used_frames.insert(intent_id) {
                            return Err("reused admitted intent".into());
                        }
                        for (resource, request_name, reply_name) in resource_rows {
                            let request_label = label(request_name)?;
                            let reply_label = label(reply_name)?;
                            let (&id, resource_request) =
                                journal::one(frames.iter().filter(|(_, f)| {
                                    f.direction(resource, store)
                                        && f.logical_label == request_label
                                        && f.received > request_at
                                        && f.received < call.emitted
                                }))?;
                            if resource_request.value()? != arguments
                                || !used_frames.insert(id)
                                || journal.frames[&id].receipt["send_actor"] != metadata
                                || journal.frames[&id].receipt["receive_commit"]["actor"] != store
                            {
                                return Err("actual resource request differs or is reused".into());
                            }
                            let (&id, response) = journal::one(frames.iter().filter(|(id, f)| {
                                f.direction(store, resource)
                                    && f.logical_label == reply_label
                                    && frames[*id].first_poll > result_at
                                    && frames[*id].first_poll < reply_at
                            }))?;
                            if response.value()? != *result
                                || !used_frames.insert(id)
                                || journal.frames[&id].receipt["send_actor"] != store
                                || journal.frames[&id].receipt["receive_commit"]["actor"]
                                    != metadata
                            {
                                return Err("actual resource reply differs or is reused".into());
                            }
                            *resource_receipts.entry(resource).or_default() += 1;
                        }
                        if name == "TESTER" {
                            let (&id, notice) = journal::one(frames.iter().filter(|(id, f)| {
                                f.direction(control, file)
                                    && f.logical_label == ready_label
                                    && frames[*id].first_poll > result_at
                                    && frames[*id].first_poll > reply_at
                            }))?;
                            if notice.value()? != *result || !used_frames.insert(id) {
                                return Err(
                                    "tester received a changed or reused test result".into()
                                );
                            }
                        }
                        let score_args = if tool == "implement_feature" {
                            json!({"feature":arguments["feature_name"]})
                        } else {
                            arguments.clone()
                        };
                        call.take_score(&mut scored, &score_args)?;
                        result.clone()
                    } else if name == "AGREEMENT" {
                        if !arguments.as_object().is_some_and(|o| o.len() == 2)
                            || arguments["api_name"] != "feature_api_spec"
                        {
                            return Err("invalid agreement bytes".into());
                        }
                        let order = match arguments["first"].as_str() {
                            Some("BACKEND_DEV") => label("BackendFirst")?,
                            Some("FRONTEND_DEV") => label("FrontendFirst")?,
                            _ => return Err("invalid implementation choice".into()),
                        };
                        let (&order_id, order_frame) =
                            journal::one(frames.iter().filter(|(id, f)| {
                                f.direction(control, store)
                                    && f.logical_label == order
                                    && frames[*id].first_poll > request_at
                                    && frames[*id].first_poll < reply_at
                            }))?;
                        let (&contract_id, contract) = frames
                            .iter()
                            .filter(|(id, f)| {
                                f.direction(store, file)
                                    && f.logical_label == contract_label
                                    && !used_frames.contains(*id)
                                    && f.received < request_at
                            })
                            .max_by_key(|(_, f)| f.received)
                            .ok_or("missing actual API contract")?;
                        let actual = contract.value()?;
                        if order_frame.value()? != arguments
                            || !used_frames.insert(order_id)
                            || !used_frames.insert(contract_id)
                        {
                            return Err("agreement contract differs or is reused".into());
                        }
                        let result = json!({"success":true,"api_contract":actual,"first":arguments["first"]});
                        let order_received = order_frame.received;
                        let accepted_label = label("AgreementAccepted")?;
                        let (&accepted_id, accepted) =
                            journal::one(frames.iter().filter(|(_, f)| {
                                f.direction(store, control)
                                    && f.logical_label == accepted_label
                                    && f.received > order_received
                                    && f.received < reply_at
                            }))?;
                        if accepted.value()? != result || !used_frames.insert(accepted_id) {
                            return Err("actual agreement acceptance differs or is reused".into());
                        }
                        let next_close_at = frames
                            .values()
                            .filter(|f| {
                                f.direction(file, control)
                                    && f.logical_label == protocol::LABEL_WASI_FD_CLOSE_OBJECT
                            })
                            .map(|f| f.received)
                            .filter(|at| *at > request_at)
                            .min();
                        let (&reply_id, actual_reply) =
                            journal::one(frames.iter().filter(|(id, f)| {
                                f.direction(control, file)
                                    && f.logical_label == contract_label
                                    && frames[*id].first_poll > reply_at
                                    && next_close_at.is_none_or(|at| frames[*id].first_poll < at)
                                    && !used_frames.contains(id)
                            }))?;
                        if actual_reply.value()? != result || !used_frames.insert(reply_id) {
                            return Err("incoming agreement response differs or is reused".into());
                        }
                        let backend_file = role("BACKEND_OUTCOME_FILE")?;
                        let backend_notice = label("BackendResult")?;
                        let (&id, notice) = journal::one(frames.iter().filter(|(id, f)| {
                            f.direction(control, backend_file)
                                && f.logical_label == backend_notice
                                && frames[*id].first_poll > request_at
                                && frames[*id].first_poll < reply_at
                        }))?;
                        if notice.value()? != result || !used_frames.insert(id) {
                            return Err("backend agreement notice differs or is reused".into());
                        }
                        result
                    } else {
                        return Err("unknown control file".into());
                    };
                    accepted_closes += 1;
                    if name != "AGREEMENT" {
                        contents.bytes = serde_json::to_vec(&result)?;
                    }
                    contents.offset = 0;
                }
                _ => return Err("file request/reply types differ".into()),
            }
        }
        let exit = journal::one(
            frames
                .values()
                .filter(|f| f.direction(file, metadata) && f.logical_label == 241),
        )?;
        if exit.bytes != 0i32.to_be_bytes() {
            return Err("operation file has failed guest exit".into());
        }

        if !completed_files.insert(file) {
            return Err("duplicate operation binding".into());
        }
    }
    if used_calls.len() != journal.calls.len() || !scored.is_empty() {
        return Err("unmatched actual effect or score event".into());
    }
    for (&resource, &count) in &resource_receipts {
        let returned = frames
            .values()
            .filter(|f| {
                f.identity["target"] == store
                    && f.logical_label == 10
                    && f.bytes.first().copied().map(u64::from) == Some(resource)
            })
            .count();
        if returned != count {
            return Err("unfinished actual resource returns".into());
        }
    }
    let test = journal::one(
        journal
            .calls
            .values()
            .filter(|c| c.request["tool"] == "run_tests"),
    )?;
    let (test_result_at, actual_result) = test.result;
    let outcomes = compiled["outcome_files"]
        .as_array()
        .ok_or("compiled outcome bindings")?;
    if outcomes.len() != 2 {
        return Err("outcome binding population differs".into());
    }
    let mut received_result_messages = 0;
    for binding in outcomes {
        let file = number(&binding["file"])?;
        let owner = number(&binding["metadata"])?;
        let guest = number(&binding["guest"])?;
        let result_label = if owner == role("BACKEND_OUTCOME")? {
            label("BackendResult")?
        } else if owner == role("FRONTEND_OUTCOME")? {
            label("FrontendResult")?
        } else {
            return Err("unknown outcome binding".into());
        };
        let (&id, result) = journal::one(
            frames
                .iter()
                .filter(|(_, f)| f.direction(store, file) && f.logical_label == result_label),
        )?;
        if result.value()? != *actual_result
            || frames[&id].first_poll <= test_result_at
            || !used_frames.insert(id)
            || journal.frames[&id].receipt["receive_commit"]["actor"] != guest
        {
            return Err("incoming outcome differs from the actual test result".into());
        }
        // EPF binds the received message to this file view. Its chunked WASI
        // projection is implemented and boundary-tested in the SDK.
        received_result_messages += 1;
        let exit = journal::one(
            frames
                .values()
                .filter(|f| f.direction(file, owner) && f.logical_label == 241),
        )?;
        if exit.bytes != 0i32.to_be_bytes() {
            return Err("outcome guest failed".into());
        }

        if !completed_files.insert(file) {
            return Err("duplicate outcome binding".into());
        }
    }
    let tester_file = role("TESTER_FILE")?;
    for (&id, frame) in frames
        .iter()
        .filter(|(_, f)| f.direction(store, tester_file) && f.logical_label == ready_label)
    {
        if !used_frames.contains(&id) {
            if frame.value()? != candidate_before(frames[&id].first_poll)? {
                return Err("candidate notification differs from actual completed work".into());
            }
            used_frames.insert(id);
        }
    }
    let effect_labels = [
        "DefineIntent",
        "DefineApi",
        "ApiDefined",
        "Contract",
        "BackendFirst",
        "FrontendFirst",
        "AgreementAccepted",
        "BackendIntent",
        "ImplementBackend",
        "BackendImplemented",
        "FrontendIntent",
        "ImplementFrontend",
        "FrontendImplemented",
        "Ready",
        "TestIntent",
        "ReadForTests",
        "RunTests",
        "RepositoryTested",
        "EnvironmentTested",
        "BackendResult",
        "FrontendResult",
    ]
    .into_iter()
    .map(label)
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, f)| effect_labels.contains(&f.logical_label) && !used_frames.contains(id))
    {
        return Err("unmatched effect data/receipt".into());
    }
    if completed_files.len() != 7 {
        return Err("missing mounted file termination".into());
    }
    Ok(
        json!({"status":"passed","task":"4E","domain_calls":used_calls.len(),"accepted_closes":accepted_closes,
        "rejected_closes":rejected_closes,"completed_files":completed_files.len(),"received_result_messages":received_result_messages,
        "simulator_complete":true,"violations":0,"dynamic_protocol":dynamic,
        "effect_scope":"EPF-checked receives, actual mounted-file bytes, unique admitted values/resource requests/effects/replies, current test candidate and common score"}),
    )
}
