//! Match the physical 4H simulator effects to the committed requests that supplied them.
use crate::{evidence::Result, journal, projection};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn audit(events: &Path) -> Result<Value> {
    check(&journal::records(events)?, &projection::load(events)?)
}

pub(super) fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let fixture = &rows.first().ok_or("empty 4H history")?["message"];
    if artifact["compiled"]["task"] != "4H" || fixture["task"] != "4H" {
        return Err("expected a 4H projection and participants".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("4H participants")?;
    let headers = projection::frames(artifact, rows)?;
    let dynamic = projection::check_dynamic(artifact, rows)?;
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
    let mut history = journal::read(rows, &headers, actors)?;
    let receipts = history.bind_receives(&dynamic)?;
    let mut result = preparation(&history, &receipts, &artifact["compiled"]["roles"])?;
    result["epf_records"] = dynamic["captured_records"].clone();
    result["runtime_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}

pub(super) fn preparation(
    history: &journal::Journal,
    receipts: &BTreeMap<u64, &Value>,
    roles: &Value,
) -> Result<Value> {
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
                "STAGING_ENV",
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
        journal::one(scored.iter().filter(|event| {
            event["agent"] == agent
                && event["tool"] == tool
                && event["args"] == *args
                && event["result"] == *data
                && event["success"] == result["success"]
                && event["violations"] == json!([])
        }))?;

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
                    && f.received.is_some_and(|at| {
                        after.is_none_or(|start| start < f.sent)
                            && before.is_none_or(|end| at < end)
                    })
                    && f.value().is_ok_and(|v| v == *payload)
            }))?;
            let observed = receipts
                .get(&frame_id)
                .ok_or("missing committed 4H frame")?;
            if observed["send_actor"] != sender || observed["receive_commit"]["actor"] != holder {
                return Err("wrong physical actor for 4H request or reply".into());
            }
            used.insert(frame_id);
            Ok((frame_id, frame.received()?))
        };
        let (request, at) = committed(file, receiver, vm, actor, args, None, Some(call.emitted))?;
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
        } else {
            request
        };
        let (reply, _) = committed(receiver, file, actor, vm, result, Some(completed), None)?;
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
            let granted = transfer.received()?;
            let (delivery, _) = committed(
                service,
                file,
                devops,
                role("DEVOPS_VM")?,
                result,
                Some(granted),
                None,
            )?;
            let owned = receipts
                .get(&loan)
                .ok_or("missing service transfer commit")?;
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
        json!({"scope":"preparation_tests_and_schema","effects":correspondence,
        "simulator_complete":history.score["complete"],"violations":history.score["violations"]}),
    )
}
