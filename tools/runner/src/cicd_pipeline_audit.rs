//! Check 16M effects against the selected executable's committed requests and replies.
use crate::{
    evidence::{Result, digest},
    journal::{self, one},
    projection,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ops::Range, path::Path};

pub fn audit(path: &Path) -> Result<Value> {
    let rows = journal::records(path)?;
    let artifact = projection::load(path)?;
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("missing pipeline fixture")?["message"];
    if compiled["task"] != "16M" || fixture["task"] != "16M" {
        return Err("foreign pipeline task".into());
    }
    let participants = fixture["fixtures"]
        .as_array()
        .ok_or("pipeline participants")?;
    let headers = projection::frames(&artifact, &rows)?;
    let dynamic = projection::check_dynamic(&artifact, &rows)?;
    let mut journal = journal::read(&rows, &headers, participants)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let score = journal.score["events"]
        .as_array()
        .ok_or("pipeline score events")?;
    if journal.score["violations"] != json!([]) || score.len() != journal.calls.len() {
        return Err("pipeline score and actual effects disagree".into());
    }
    let role = |name: &str| compiled["roles"][name].as_u64().ok_or("pipeline role");
    let mut used = BTreeSet::new();
    let mut take = |(from, to, label, actor): (&str, &str, &str, &str),
                    window: Range<usize>,
                    expected: &Value|
     -> Result<u64> {
        let (from, to, actor) = (role(from)?, role(to)?, role(actor)?);
        let logical = compiled["labels"][label]
            .as_u64()
            .ok_or("pipeline message")?;
        let (&id, _) = one(journal.frames.iter().filter(|(id, frame)| {
            !used.contains(*id)
                && frame.direction(from, to)
                && u64::from(frame.logical_label) == logical
                && frame.received.is_some_and(|at| window.contains(&at))
                && receipts
                    .get(id)
                    .is_some_and(|receipt| receipt["receive_commit"]["actor"] == actor)
                && frame.value().is_ok_and(|value| value == *expected)
        }))
        .map_err(|error| format!("{label} {from}->{to}: {error}"))?;
        used.insert(id);
        Ok(id)
    };

    let mut effects = Vec::new();
    let mut scored = BTreeSet::new();
    for call in journal.calls.values() {
        let agent = call.request["agent"].as_str().ok_or("pipeline agent")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let (result_at, result) = call.result()?;
        let receipt = result["data"]["receipt"]
            .as_u64()
            .ok_or("common effect receipt")?;
        let observed = one(score
            .iter()
            .filter(|event| event["result"]["receipt"] == receipt))
        .map_err(|_| "common score receipt is missing or duplicated")?;
        if !scored.insert(receipt)
            || observed["agent"] != agent
            || observed["tool"] != tool
            || observed["args"] != *arguments
            || observed["completed_at"].as_f64().is_none()
            || observed["success"] != result["success"]
            || observed["result"] != result["data"]
        {
            return Err("common score changed the actual pipeline effect".into());
        }

        let (file, vm) = match agent {
            "FRONTEND_DEV" => ("FRONTEND_FILE", "FRONTEND_VM"),
            "BACKEND_DEV" => ("BACKEND_FILE", "BACKEND_VM"),
            "QA_ENGINEER" => ("QA_FILE", "QA_VM"),
            "RELEASE_ENGINEER" => ("RELEASE_FILE", "RELEASE_VM"),
            _ => return Err("unexpected pipeline effect actor".into()),
        };
        let target = arguments["target"].as_str();
        let (resource, input, holder, output) = match (tool, target) {
            ("compile_code", _) => ("BUILD_SERVER", "Compile", agent, "Compiled"),
            ("run_unit_tests", _) => ("BUILD_SERVER", "UnitTest", agent, "UnitTested"),
            ("pull_artifacts", Some("test_harness")) => {
                ("ARTIFACT_STORE", "PullForQa", "REGISTRY_OWNER", "QaPulled")
            }
            ("pull_artifacts", Some("staging_deploy")) => (
                "ARTIFACT_STORE",
                "PullForRelease",
                "REGISTRY_OWNER",
                "ReleasePulled",
            ),
            ("run_integration_tests", _) => ("TEST_ENV", "Integration", "QA", "Integrated"),
            ("run_e2e_tests", _) => ("TEST_ENV", "E2e", "QA", "E2eTested"),
            ("deploy_staging", _) => ("TEST_ENV", "Stage", "RELEASE", "Staged"),
            ("run_smoke_tests", Some("staging")) => {
                ("TEST_ENV", "SmokeStaging", "RELEASE", "StagingResult")
            }
            ("deploy_production", _) => ("PROD_ENV", "Deploy", "RELEASE", "Deployed"),
            ("run_smoke_tests", Some("production")) => {
                ("PROD_ENV", "SmokeProduction", "RELEASE", "ProductionSmoke")
            }
            ("publish_artifact", _) => (
                "REGISTRY_OWNER",
                "RegisterArtifact",
                "REGISTRY_OWNER",
                "Published",
            ),
            _ => return Err(format!("unknown pipeline effect {tool} {target:?}").into()),
        };

        let (requests, replies) = if tool == "publish_artifact" {
            let receipt = take((file, agent, "Publish", agent), 0..call.emitted, arguments)?;
            let workspace = take(
                ("BUILD_SERVER", resource, "ReadBuild", holder),
                journal.frames[&receipt].received()?..call.emitted,
                arguments,
            )?;
            let registry = take(
                ("ARTIFACT_STORE", resource, input, holder),
                journal.frames[&receipt].received()?..call.emitted,
                arguments,
            )?;
            let real = take(
                (resource, agent, output, agent),
                result_at..rows.len(),
                result,
            )?;
            let delivered = take(
                (agent, file, output, vm),
                journal.frames[&real].received()?..rows.len(),
                result,
            )?;
            (vec![receipt, workspace, registry], vec![real, delivered])
        } else {
            let received = take((file, resource, input, holder), 0..call.emitted, arguments)?;
            let delivered = take((resource, file, output, vm), result_at..rows.len(), result)?;
            (vec![received], vec![delivered])
        };
        effects.push(json!({"agent":agent,"tool":tool,"arguments":arguments,
                            "request_frames":requests,"reply_frames":replies}));
    }
    Ok(json!({
        "status": "passed",
        "task": "16M",
        "simulator_complete": journal.score["complete"],
        "violations": 0,
        "compiled_projection_sha256": digest(&path.with_file_name("projection.json"))?,
        "runtime_binary_sha256": artifact["runtime_binary_sha256"],
        "dynamic": dynamic,
        "effects": effects,
    }))
}
