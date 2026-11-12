//! Saved actual effects and affine resource requests; EPF owns dynamic ordering.
use crate::{
    evidence::{Result, digest},
    journal::{check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty coordination evidence")?["message"];
    if compiled["task"] != "4E" || fixture["task"] != "4E" || fixture["type"] != "fixtures" {
        return Err("coordination task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("coordination participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("BACKEND_DEV"), Some("FRONTEND_DEV"), Some("TESTER")])
    {
        return Err("original coordination participants differ".into());
    }
    let names = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let store = names("roles", "STORE")?;
    let repo = names("roles", "REPO")?;
    let environment = names("roles", "TEST_ENV")?;
    let session = compiled["session"].as_u64().ok_or("compiled session")?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if frames.values().any(|f| f.identity["session"] != session)
        || score["complete"] != true
        || score["violations"] != json!([])
        || calls.is_empty()
    {
        return Err("coordination completion or session differs".into());
    }
    let pairs = [
        (
            names("roles", "FRONTEND_DEV")?,
            names("roles", "FRONTEND_IO")?,
        ),
        (
            names("roles", "BACKEND_DEV")?,
            names("roles", "BACKEND_IO")?,
        ),
        (names("roles", "TESTER")?, names("roles", "TESTER_IO")?),
    ];
    check_close_pairs(&frames, &pairs)?;
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("original score events")?
        .clone();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("effect participant")?;
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        let tool = call.tool()?;
        let expected = one(actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .filter(|c| c["tool"] == tool))?;
        let arguments = &call.request["arguments"];
        if *arguments != expected["arguments"] {
            return Err("unassigned coordination arguments".into());
        }
        let (result_at, result) = call.result()?;
        if result["success"] != true {
            return Err("unexpected coordination tool failure".into());
        }
        let score_args = if tool == "implement_feature" {
            json!({"feature":arguments["feature_name"]})
        } else {
            arguments.clone()
        };
        let index = scored
            .iter()
            .position(|e| {
                e["agent"] == agent
                    && e["tool"] == tool
                    && e["args"] == score_args
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("original score differs from actual coordination effect")?;
        scored.remove(index);
        let (request_name, reply_name, resource, io) = match (agent, tool) {
            ("BACKEND_DEV", "define_api") => ("DefineApi", "ApiDefined", repo, "BACKEND_IO"),
            ("BACKEND_DEV", "implement_feature") => {
                ("ImplementBackend", "BackendImplemented", repo, "BACKEND_IO")
            }
            ("FRONTEND_DEV", "implement_feature") => (
                "ImplementFrontend",
                "FrontendImplemented",
                repo,
                "FRONTEND_IO",
            ),
            ("TESTER", "run_tests") => ("RunTests", "EnvironmentTested", environment, "TESTER_IO"),
            _ => return Err("unassigned coordination effect".into()),
        };
        let request_label = names("labels", request_name)? as u8;
        let reply_label = names("labels", reply_name)? as u8;
        let (request_id, request) = frames
            .iter()
            .filter(|(_, f)| {
                f.direction(resource, store)
                    && f.logical_label == request_label
                    && f.received.is_some_and(|at| at < call.emitted)
            })
            .max_by_key(|(_, f)| f.received)
            .ok_or("effect lacks a resource request")?;
        if request.value()? != *arguments || !used_frames.insert(*request_id) {
            return Err("resource request altered or reused".into());
        }
        let mut before = request.sent;
        if tool == "implement_feature" && agent == "BACKEND_DEV" {
            let label = names("labels", "BackendIntent")? as u8;
            let vm = names("roles", agent)?;
            let (id, intent) = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(vm, store)
                        && f.logical_label == label
                        && f.sent < request.sent
                        && f.received.is_some_and(|at| at < call.emitted)
                })
                .max_by_key(|(_, f)| f.received)
                .ok_or("backend request without admitted input")?;
            if intent.value()? != *arguments || !used_frames.insert(*id) {
                return Err("backend intent altered or reused".into());
            }
            before = before.min(intent.sent);
        }
        if tool == "run_tests" {
            let label = names("labels", "ReadForTests")? as u8;
            let (id, read) = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(repo, store)
                        && f.logical_label == label
                        && f.received.is_some_and(|at| at < call.emitted)
                })
                .max_by_key(|(_, f)| f.received)
                .ok_or("test effect lacks repository receipt")?;
            if read.value()? != *arguments || !used_frames.insert(*id) {
                return Err("test repository request altered or reused".into());
            }
            before = before.min(read.sent);
        }
        close_arguments(
            &frames,
            &mut used_closes,
            names("roles", agent)?,
            names("roles", io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
        let mut replies = vec![(resource, reply_label)];
        if tool == "run_tests" {
            replies.extend([
                (repo, names("labels", "RepositoryTested")? as u8),
                (
                    names("roles", "BACKEND_DEV")?,
                    names("labels", "BackendResult")? as u8,
                ),
                (
                    names("roles", "FRONTEND_DEV")?,
                    names("labels", "FrontendResult")? as u8,
                ),
            ]);
        }
        for (target, label) in replies {
            let (id, reply) = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(store, target) && f.logical_label == label && f.sent > result_at
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or("actual effect reply missing")?;
            if reply.value()? != *result || !used_frames.insert(*id) {
                return Err("actual effect reply altered or reused".into());
            }
            reply.received()?;
        }
    }
    if !scored.is_empty() {
        return Err("unmatched original score event".into());
    }
    let effect_labels = [
        "DefineApi",
        "ApiDefined",
        "ImplementBackend",
        "BackendImplemented",
        "ImplementFrontend",
        "FrontendImplemented",
        "ReadForTests",
        "RunTests",
        "RepositoryTested",
        "EnvironmentTested",
        "BackendResult",
        "FrontendResult",
        "BackendIntent",
    ]
    .into_iter()
    .map(|key| names("labels", key).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, f)| effect_labels.contains(&f.logical_label) && !used_frames.contains(id))
    {
        return Err("unmatched resource request, admitted input or actual reply".into());
    }
    let finalize = names("labels", "Finalize")? as u8;
    let final_request = one(frames
        .values()
        .filter(|f| f.direction(repo, store) && f.logical_label == finalize))?;
    let latest = |agent: &str, tool: &str| -> Result<Value> {
        let call = ordered
            .iter()
            .filter(|c| {
                c.request["agent"] == agent
                    && c.request["tool"] == tool
                    && c.result
                        .as_ref()
                        .is_some_and(|(at, _)| *at < final_request.sent)
            })
            .max_by_key(|c| c.emitted)
            .ok_or("final candidate lacks actual implementation")?;
        Ok(call.result()?.1.clone())
    };
    if final_request.value()?
        != json!({"api":latest("BACKEND_DEV", "define_api")?,
        "backend":latest("BACKEND_DEV", "implement_feature")?,
        "frontend":latest("FRONTEND_DEV", "implement_feature")?})
    {
        return Err("final candidate differs from actual current results".into());
    }
    Ok(
        json!({"status":"passed","task":"4E","domain_calls":calls.len(),"simulator_complete":true,
        "violations":0,"effect_scope":"actual received resource requests, admitted arguments, original results, current candidate and completed guests"}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}
