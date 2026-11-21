//! Match the physical 4H simulator effects to the committed requests that supplied them.
use crate::{evidence::Result, journal, projection};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let fixture = &rows.first().ok_or("empty 4H history")?["message"];
    if artifact["compiled"]["task"] != "4H" || fixture["task"] != "4H" {
        return Err("expected a 4H projection and participants".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("4H participants")?;
    let headers = projection::frames(artifact, rows)?;
    let expected = BTreeSet::from([
        "ARCHITECT",
        "DEVELOPER_A",
        "DEVELOPER_B",
        "DEVELOPER_C",
        "REVIEWER",
        "TESTER",
        "DEVOPS",
    ]);
    if actors.len() != expected.len()
        || actors
            .iter()
            .filter_map(|actor| actor["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Err("incomplete 4H participants".into());
    }
    let history = journal::read(rows, &headers, actors, dynamic)?;
    let mut result = preparation(&history, &artifact["compiled"]["roles"])?;
    result["epf_records"] = dynamic["captured_records"].clone();
    result["runtime_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}

pub(super) fn preparation(history: &journal::Journal, roles: &Value) -> Result<Value> {
    let role = |name: &str| {
        roles[name]
            .as_u64()
            .ok_or_else(|| format!("missing 4H role {name}"))
    };
    let scored = history.score["events"]
        .as_array()
        .ok_or("4H score events")?;
    if scored.len() != history.calls.len()
        || history.score["violations"] != json!([])
        || history.score["complete"] != false
    {
        return Err("4H score and effect inventory disagree or contain a violation".into());
    }

    // Each entry is an observed RPC, ordered by issue, including retries with identical arguments.
    let mut calls: Vec<_> = history.calls.iter().collect();
    calls.sort_by_key(|(_, call)| call.emitted);
    let mut used = BTreeSet::new();
    let deployments = deployment_attempts(history, roles, &mut used)?;
    let mut scores = BTreeSet::new();
    let mut correspondence = Vec::new();
    let mut latest: BTreeMap<(&str, &str), (usize, &Value)> = BTreeMap::new();
    let mut request_edges = BTreeSet::new();
    let mut reply_edges = BTreeSet::new();
    for (&id, call) in calls {
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let (completed, result) = call.result()?;
        let data = &result["data"];
        let subject = match tool {
            "design_proposal" => "",
            "implement_feature" => args["feature_name"].as_str().ok_or("build target")?,
            "code_review" => args["pr_id"].as_str().ok_or("review target")?,
            "run_tests" => args["suite"].as_str().ok_or("suite target")?,
            "deploy_service" => args["service"].as_str().ok_or("deployment target")?,
            "migrate_schema" => args["migration_name"]
                .as_str()
                .and_then(|s| s.strip_suffix("_schema"))
                .ok_or("schema target")?,
            _ => return Err(format!("unaccounted 4H effect {tool}").into()),
        };
        let suffix = match subject {
            "microservice_a" => "A",
            "microservice_b" => "B",
            "microservice_c" => "C",
            "" if tool == "design_proposal" => "",
            _ => return Err("unknown 4H service in the received request".into()),
        };
        let developer = format!("DEVELOPER_{suffix}");
        let developer_file = format!("{developer}_FILE");
        let reviewer = format!("REVIEWER_{suffix}");
        let tester = format!("TESTER_{suffix}");
        let service = format!("SERVICE_{suffix}");
        let dev_vm = format!("{suffix}_VM");
        // Logical file, request receiver, executing actor, shell VM, simulator participant.
        let (file, receiver, actor, vm, agent) = match tool {
            "design_proposal" => (
                "ARCHITECT_FILE",
                "ARCHITECT",
                "ARCHITECT",
                "ARCHITECT_VM",
                "ARCHITECT",
            ),
            "implement_feature" => (
                developer_file.as_str(),
                service.as_str(),
                developer.as_str(),
                dev_vm.as_str(),
                developer.as_str(),
            ),
            "code_review" => (
                "REVIEW_FILE",
                service.as_str(),
                reviewer.as_str(),
                "REVIEW_VM",
                "REVIEWER",
            ),
            "run_tests" => ("TEST_FILE", "ENV_OWNER", "ENV_OWNER", "TEST_VM", "TESTER"),
            "migrate_schema" => (
                "DEVOPS_FILE",
                "STAGING_OWNER",
                "STAGING_OWNER",
                "DEVOPS_VM",
                "DEVOPS",
            ),
            "deploy_service" => (
                "DEVOPS_FILE",
                service.as_str(),
                "DEVOPS",
                "DEVOPS_VM",
                "DEVOPS",
            ),
            _ => unreachable!(),
        };
        let (file, receiver, actor, vm) = (role(file)?, role(receiver)?, role(actor)?, role(vm)?);
        let (key, success_status, failure_status) = match tool {
            "design_proposal" => ("title", "proposed", None),
            "implement_feature" => ("feature_name", "implemented", None),
            "code_review" => ("pr_id", "reviewed", Some("changes_requested")),
            "run_tests" => ("suite", "tests passed", Some("tests_failed")),
            "deploy_service" => ("service", "deployed", Some("deployment_failed")),
            "migrate_schema" => ("migration_name", "migrated", None),
            _ => unreachable!(),
        };
        let expected_status = if result["success"]
            .as_bool()
            .ok_or("missing effect verdict")?
        {
            Some(success_status)
        } else {
            failure_status
        };
        if data[key] != args[key]
            || expected_status.is_none()
            || data["status"].as_str() != expected_status
        {
            return Err("4H effect response differs from its requested target or verdict".into());
        }
        let receipt = data["receipt"]
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or("4H simulator receipt")?;
        if call.request["agent"] != agent
            || data["agent"] != agent
            || (!subject.is_empty() && data["service"] != subject)
            || !scores.insert(receipt)
        {
            return Err("effect actor, service or unique simulator receipt differs".into());
        }
        journal::one(
            scored
                .iter()
                .filter(|event| call.matches_score(event, args)),
        )?;

        request_edges.insert((file, receiver));
        reply_edges.insert((receiver, file, tool));
        let mut committed = |from,
                             to,
                             sender,
                             holder,
                             payload: &Value,
                             after: Option<usize>,
                             before: Option<usize>|
         -> Result<(u64, usize)> {
            let (&frame_id, frame) = journal::one(history.frames.iter().filter(|(i, f)| {
                !used.contains(*i)
                    && f.direction(from, to)
                    && {
                        after.is_none_or(|start| start < f.sent)
                            && before.is_none_or(|end| f.received < end)
                    }
                    && f.value().is_ok_and(|v| v == *payload)
            }))?;
            let observed = history
                .frames
                .get(&frame_id)
                .ok_or("missing committed 4H frame")?
                .receipt;
            if observed["send_actor"] != sender || observed["receive_commit"]["actor"] != holder {
                return Err("wrong physical actor for 4H request or reply".into());
            }
            used.insert(frame_id);
            Ok((frame_id, frame.received))
        };
        let (request, effect_request, reply) = if tool == "deploy_service" {
            request_edges.insert((role("STAGING_ENV")?, role("STAGING_SERVER")?));
            reply_edges.insert((role("STAGING_SERVER")?, role("STAGING_ENV")?, tool));
            *deployments.get(&id).ok_or("unbound staging effect")?
        } else {
            let (request, at) =
                committed(file, receiver, vm, actor, args, None, Some(call.emitted))?;
            let effect_request = if tool == "run_tests" {
                request_edges.insert((role("TEST_ENV")?, role(&tester)?));
                committed(
                    role("TEST_ENV")?,
                    role(&tester)?,
                    actor,
                    role(&tester)?,
                    args,
                    Some(at),
                    Some(call.emitted),
                )?
                .0
            } else if tool == "migrate_schema" {
                let env = role("STAGING_ENV")?;
                let server = role("STAGING_SERVER")?;
                request_edges.insert((env, server));
                reply_edges.insert((server, env, tool));
                let (physical, _) = committed(
                    env,
                    server,
                    actor,
                    server,
                    args,
                    Some(at),
                    Some(call.emitted),
                )?;
                committed(server, env, server, actor, result, Some(completed), None)?;
                physical
            } else {
                request
            };
            let (reply, _) = committed(receiver, file, actor, vm, result, Some(completed), None)?;
            (request, effect_request, reply)
        };
        let prior = |kind, name, field: &str| -> Result<&Value> {
            let &(ended, previous) = latest
                .get(&(kind, name))
                .ok_or("missing prerequisite effect")?;
            if ended >= call.emitted
                || previous["success"] != true
                || data[field] != previous["data"]["receipt"]
            {
                return Err(format!("stale {field} on {tool}").into());
            }
            Ok(&previous["data"])
        };
        if tool == "design_proposal" {
            if !latest.is_empty()
                || args["title"] != "migration_plan"
                || data["title"] != args["title"]
            {
                return Err("invalid migration plan receipt".into());
            }
            if data["dependencies"]
                .as_object()
                .is_none_or(|edges| edges.len() != 3)
            {
                return Err("incomplete migration plan dependencies".into());
            }
            for service in ["microservice_a", "microservice_b", "microservice_c"] {
                let required = args["dependencies"].get(service);
                if !data["dependencies"][service].is_array()
                    || !required.map_or_else(
                        || data["dependencies"][service] == json!([]),
                        |deps| deps == &data["dependencies"][service],
                    )
                {
                    return Err("declared dependencies differ from the actual plan".into());
                }
            }
        } else {
            prior("design_proposal", "", "plan")?;
        }
        if tool == "deploy_service" {
            let test = prior("run_tests", subject, "test")?;
            if test["candidate"] != data["candidate"] || args["environment"] != "staging" {
                return Err("deployment for a different candidate or environment".into());
            }
            let design = latest[&("design_proposal", "")].1;
            let mut prerequisites = serde_json::Map::new();
            for dependency in design["data"]["dependencies"][subject]
                .as_array()
                .ok_or("service prerequisites")?
            {
                let name = dependency.as_str().ok_or("dependency name")?;
                let (ended, other) = latest
                    .get(&("deploy_service", name))
                    .ok_or("prerequisite deployment missing")?;
                if *ended >= call.emitted || other["success"] != true {
                    return Err("prerequisite was not deployed when the effect started".into());
                }
                prerequisites.insert(name.to_owned(), other["data"]["receipt"].clone());
            }
            if data["dependencies"].as_object() != Some(&prerequisites) {
                return Err("deployment does not use the authored prerequisite receipts".into());
            }
        }
        if matches!(tool, "code_review" | "run_tests") {
            prior("implement_feature", subject, "candidate")?;
        }
        if tool == "run_tests"
            && prior("code_review", subject, "review")?["candidate"] != data["candidate"]
        {
            return Err("review and test belong to different builds".into());
        }
        if matches!(tool, "code_review" | "run_tests")
            && latest.get(&(tool, subject)).is_some_and(|(_, previous)| {
                previous["success"] == false && previous["data"]["candidate"] == data["candidate"]
            })
        {
            return Err("4H retry reused the rejected candidate".into());
        }
        let mut entry = json!({"rpc":id,"tool":tool,"service":subject,"receipt":receipt,
            "file_request":request,"effect_request":effect_request,"reply":reply});
        if tool == "run_tests" && result["success"] == true {
            let tester = role(&tester)?;
            let service = role(&service)?;
            let devops = role("DEVOPS")?;
            let file = role("DEVOPS_FILE")?;
            let (candidate, informed) = committed(
                tester,
                devops,
                tester,
                devops,
                result,
                Some(completed),
                None,
            )?;
            let (&loan, transfer) = journal::one(history.frames.iter().filter(|(_, f)| {
                f.direction(tester, devops)
                    && f.logical_label == 47
                    && matches!(f.bytes.as_slice(), [view, _, _] if u64::from(*view) == service)
                    && f.sent > history.frames[&candidate].sent
            }))?;
            let granted = transfer.received;
            let (delivery, _) = committed(
                service,
                file,
                devops,
                role("DEVOPS_VM")?,
                result,
                Some(granted),
                None,
            )?;
            let owned = history
                .frames
                .get(&loan)
                .ok_or("missing service transfer commit")?
                .receipt;
            if informed >= granted
                || owned["send_actor"] != tester
                || owned["receive_commit"]["actor"] != devops
                || u64::from(history.frames[&candidate].logical_label) != tester
                || u64::from(history.frames[&delivery].logical_label) != tester
                || !used.insert(loan)
            {
                return Err("test result and service transfer to DevOps disagree".into());
            }
            reply_edges.insert((tester, devops, tool));
            reply_edges.insert((service, file, tool));
            entry["staging_candidate"] =
                json!({"test_result":candidate,"service_loan":loan,"devops_shell_read":delivery});
        }
        latest.insert((tool, subject), (completed, result));
        correspondence.push(entry);
    }
    for (id, frame) in &history.frames {
        if used.contains(id) {
            continue;
        }
        let edge = (
            frame.identity["source"].as_u64().ok_or("frame source")?,
            frame.identity["target"].as_u64().ok_or("frame target")?,
        );
        if let Ok(value) = frame.value()
            && value.is_object()
            && (request_edges.contains(&edge)
                || value["tool_name"]
                    .as_str()
                    .is_some_and(|tool| reply_edges.contains(&(edge.0, edge.1, tool))))
        {
            return Err(format!("unmatched 4H request or reply at frame {id}").into());
        }
    }
    for suffix in ["a", "b", "c"] {
        let service = format!("microservice_{suffix}");
        if [
            "implement_feature",
            "code_review",
            "run_tests",
            "migrate_schema",
            "deploy_service",
        ]
        .into_iter()
        .any(|tool| {
            latest
                .get(&(tool, service.as_str()))
                .is_none_or(|(_, result)| result["success"] != true)
        }) {
            return Err("incomplete 4H preparation effects".into());
        }
    }
    Ok(
        json!({"scope":"tested_services_on_staging","effects":correspondence,
        "simulator_complete":history.score["complete"],"violations":history.score["violations"]}),
    )
}

/// Follow one service's real file requests, including a refusal followed by identical arguments.
fn deployment_attempts(
    history: &journal::Journal,
    roles: &Value,
    used: &mut BTreeSet<u64>,
) -> Result<BTreeMap<u64, (u64, u64, u64)>> {
    let role = |name: &str| roles[name].as_u64().ok_or("missing staging role");
    let file = role("DEVOPS_FILE")?;
    let devops = role("DEVOPS")?;
    let owner = role("STAGING_OWNER")?;
    let env = role("STAGING_ENV")?;
    let server = role("STAGING_SERVER")?;
    let vm = role("DEVOPS_VM")?;
    let mut effects = BTreeMap::new();
    for suffix in ["A", "B", "C"] {
        let svc = role(&format!("SERVICE_{suffix}"))?;
        let name = format!("microservice_{}", suffix.to_ascii_lowercase());
        let mut requests: Vec<_> = history
            .frames
            .iter()
            .filter(|(_, f)| f.direction(file, svc))
            .collect();
        requests.sort_by_key(|(_, f)| f.sent);
        for (i, &(&id, request)) in requests.iter().enumerate() {
            let stop = requests
                .get(i + 1)
                .map_or(usize::MAX, |(_, next)| next.sent);
            let args = request.value()?;
            let admission = history
                .frames
                .get(&id)
                .ok_or("missing deployment file commit")?
                .receipt;
            if args != json!({"service":name,"environment":"staging"})
                || admission["send_actor"] != vm
                || admission["receive_commit"]["actor"] != devops
                || !used.insert(id)
            {
                return Err("invalid deployment file request".into());
            }
            let evidence = journal::one(history.calls.values().filter_map(|c| {
                let (at, test) = c.result().ok()?;
                (c.tool().ok()? == "run_tests"
                    && at < request.sent
                    && test["data"]["service"] == name
                    && test["success"] == true)
                    .then_some(test)
            }))?;
            let frame = |from,
                         to,
                         label: Option<u8>,
                         bytes: Option<&Value>|
             -> Result<(u64, &journal::Frame)> {
                let (&id, f) = journal::one(history.frames.iter().filter(|(_,f)| {
                    f.direction(from,to) && request.sent < f.sent && f.sent < stop
                        && label.is_none_or(|l| l == f.logical_label)
                        && bytes.is_none_or(|value| f.value().is_ok_and(|v| v == *value))
                        && (label != Some(47) || matches!(f.bytes.as_slice(), [view, _, _] if u64::from(*view) == svc))
                }))?;
                Ok((id, f))
            };
            let (test_id, test) = frame(svc, owner, None, Some(evidence))?;
            let (grant_id, grant) = frame(devops, owner, Some(47), None)?;
            let (forward_id, forward) =
                frame(devops, owner, Some(request.logical_label), Some(&args))?;
            let (reply_id, reply) = frame(svc, file, Some(request.logical_label), None)?;
            let outcome = reply.value()?;
            let (answer_id, answer) = frame(svc, devops, None, Some(&outcome))?;
            let (return_id, returned) = frame(owner, devops, Some(47), None)?;
            if !(request.received < test.sent
                && test.sent < grant.sent
                && test.received < grant.received
                && grant.received < forward.received
                && forward.received < answer.sent
                && answer.sent < returned.sent
                && answer.received < returned.received
                && returned.received < reply.sent)
            {
                return Err("deployment service custody and response are out of order".into());
            }
            for (id, sender, receiver) in [
                (test_id, devops, owner),
                (grant_id, devops, owner),
                (forward_id, devops, owner),
                (answer_id, owner, devops),
                (return_id, owner, devops),
                (reply_id, devops, vm),
            ] {
                let epf = history
                    .frames
                    .get(&id)
                    .ok_or("absent deployment commit")?
                    .receipt;
                if epf["send_actor"] != sender
                    || epf["receive_commit"]["actor"] != receiver
                    || !used.insert(id)
                {
                    return Err("wrong or reused deployment occurrence".into());
                }
            }
            let forwarded_at = forward.received;
            let physical: Vec<_> = history
                .frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(env, server)
                        && request.sent < f.sent
                        && f.sent < stop
                        && f.value().is_ok_and(|v| v == args)
                })
                .collect();
            if outcome["status"] == "dependency_pending" {
                let dependency = outcome["service"]
                    .as_str()
                    .ok_or("missing pending dependency")?;
                let design = journal::one(
                    history
                        .calls
                        .values()
                        .filter(|c| c.tool().ok() == Some("design_proposal")),
                )?
                .result()?
                .1;
                if outcome["success"] != false
                    || !physical.is_empty()
                    || u64::from(answer.logical_label) != svc
                    || !design["data"]["dependencies"][&name]
                        .as_array()
                        .is_some_and(|deps| deps.contains(&json!(dependency)))
                    || history.calls.values().any(|c| {
                        c.tool().ok() == Some("deploy_service")
                            && c.result().is_ok_and(|(at, r)| {
                                at < forwarded_at
                                    && r["success"] == true
                                    && r["data"]["service"] == dependency
                            })
                    })
                {
                    return Err("unjustified deployment deferral".into());
                }
                continue;
            }
            let (&physical_id, admitted) = journal::one(physical.into_iter())?;
            let (&rpc, call) = journal::one(history.calls.iter().filter(|(_, c)| {
                c.tool().ok() == Some("deploy_service")
                    && c.request["arguments"] == args
                    && admitted.received < c.emitted
                    && c.result()
                        .is_ok_and(|(at, result)| at < answer.sent && *result == outcome)
            }))?;
            let (observed_id, observed) = frame(server, env, None, Some(call.result()?.1))?;
            if !(forward.received < admitted.sent
                && call.result()?.0 < observed.sent
                && observed.received < answer.sent)
                || outcome["data"]["test"] != evidence["data"]["receipt"]
                || outcome["data"]["candidate"] != evidence["data"]["candidate"]
                || answer.logical_label != request.logical_label
                || effects.insert(rpc, (id, physical_id, reply_id)).is_some()
            {
                return Err(
                    "deployment does not match the accepted test and staging invocation".into(),
                );
            }
            for (id, sender, receiver) in
                [(physical_id, owner, server), (observed_id, server, owner)]
            {
                let epf = history
                    .frames
                    .get(&id)
                    .ok_or("absent physical staging commit")?
                    .receipt;
                if epf["send_actor"] != sender
                    || epf["receive_commit"]["actor"] != receiver
                    || !used.insert(id)
                {
                    return Err("invalid physical staging occurrence".into());
                }
            }
        }
    }
    Ok(effects)
}
