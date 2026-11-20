//! Recorded shell prefixes for diagnosing incomplete trials, without a model.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const DOCUMENT_AGENTS: [&str; 7] = [
    "WRITER_A",
    "WRITER_B",
    "WRITER_C",
    "FACT_CHECKER",
    "REVIEWER",
    "FIGURE_CREATOR",
    "EDITOR_IN_CHIEF",
];

#[hibana_runtime::main]
#[test]
#[ignore = "explicit diagnostic of the preserved 5E timeout; not qualification"]
async fn captured_medical_summary_before_assessment() -> Result<()> {
    replay(
        (
            "5E",
            "tools/runner/target/debug/medical-wasi",
            ["ER_DOCTOR", "CARDIOLOGIST", "PHARMACIST"],
        ),
        ("result.json", "error", "timed out"),
        None,
        Duration::from_secs(15),
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "explicit diagnostic of the preserved 6M timeout; not qualification"]
async fn captured_feature_peer_review_after_repair_approval() -> Result<()> {
    replay(
        (
            "6M",
            "tools/runner/target/debug/feature-delivery-wasi",
            [
                "DEVELOPER_A",
                "DEVELOPER_B",
                "REVIEWER",
                "TEST_WRITER",
                "CI_RUNNER",
            ],
        ),
        ("result.json", "error", "timed out"),
        None,
        Duration::from_secs(80),
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "explicit diagnostic of the preserved 13H stale batch read; not qualification"]
async fn captured_qc_batch_read_after_approval() -> Result<()> {
    replay(
        (
            "13H",
            "target/debug/lab-cascading-wasi",
            [
                "CHEMIST",
                "BIOLOGIST",
                "ANALYST",
                "QC_CONTROLLER",
                "SAFETY_OFFICER",
            ],
        ),
        (
            "../stopped.json",
            "status",
            "stopped_obsolete_qc_batch_read",
        ),
        None,
        Duration::from_secs(20),
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "diagnoses the recorded 7H wait after one of two required minor edits"]
async fn captured_document_assessment_before_both_amendments() -> Result<()> {
    let runtime = std::env::var("CHOREOFIX_LARGE_DOCUMENT_WASI_BINARY")?;
    replay(
        ("7H", &runtime, DOCUMENT_AGENTS),
        (
            "../stopped-depth-0.json",
            "status",
            "stopped_without_publication",
        ),
        None,
        Duration::from_secs(95),
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "diagnoses the recorded 7H wait before a renewed review request"]
async fn captured_document_outcome_before_renewed_review() -> Result<()> {
    let runtime = std::env::var("CHOREOFIX_LARGE_DOCUMENT_WASI_BINARY")?;
    replay(
        ("7H", &runtime, DOCUMENT_AGENTS),
        (
            "../interrupted-depth-0.json",
            "status",
            "stopped_after_review_outcome_wait",
        ),
        Some("events.jsonl"),
        Duration::from_secs(85),
    )
    .await
}

#[hibana_runtime::scope]
async fn replay<const N: usize>(
    (task, executable, agents): (&str, &str, [&str; N]),
    (receipt, field, verdict): (&str, &str, &str),
    recorded_effects: Option<&str>,
    deadline: Duration,
) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let captured =
        PathBuf::from(std::env::var_os("CHOREOFIX_TIMEOUT_CAPTURE").ok_or("captured timeout")?);
    let output =
        PathBuf::from(std::env::var_os("CHOREOFIX_TIMEOUT_OUTPUT").ok_or("diagnostic output")?);
    fs::create_dir(&output)?;
    let previous: Value = serde_json::from_slice(&fs::read(captured.join(receipt))?)?;
    let events = captured.join("events.jsonl");
    let original_hash = digest(&events)?;
    if previous["task"] != task
        || previous[field] != verdict
        || previous
            .get("events_sha256")
            .is_some_and(|value| value != &original_hash)
    {
        return Err("the selected input differs from this task's preserved stopped run".into());
    }
    let depth = previous["depth"]
        .as_u64()
        .ok_or("captured depth")?
        .try_into()?;
    let runtime = root.join(executable);
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let runtime_sha = digest(&runtime)?;
    let captured_runtime_sha = digest(&captured.join("runtime"))?;
    let module_sha = digest(&module)?;
    if module_sha != digest(&captured.join("wash.wasm"))? {
        return Err("the shell module changed since the captured timeout".into());
    }
    if runtime_sha != captured_runtime_sha {
        if recorded_effects.is_none() {
            return Err("expected the original executable for simulator replay".into());
        }
        let build_receipt = PathBuf::from(
            std::env::var_os("CHOREOFIX_LARGE_DOCUMENT_BUILD_RECEIPT")
                .ok_or("build receipt for the candidate runtime")?,
        );
        archive_build_receipt(&runtime, &build_receipt, &output)?;
    }
    let scale = 0.1;
    let mut plans = agents.map(|agent| (agent, Vec::<(f64, String)>::new()));
    for line in fs::read_to_string(events)?.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["direction"] == "to_runtime" && row["message"]["type"] == "command" {
            let agent = row["message"]["agent"]
                .as_str()
                .ok_or("captured command agent")?;
            let (_, commands) = plans
                .iter_mut()
                .find(|(who, _)| *who == agent)
                .ok_or("foreign captured agent")?;
            commands.push((
                row["elapsed_seconds"]
                    .as_f64()
                    .ok_or("captured command time")?
                    * scale,
                row["message"]["command"]
                    .as_str()
                    .ok_or("captured command")?
                    .to_owned(),
            ));
        }
    }
    if plans.iter().any(|(_, commands)| commands.is_empty()) {
        return Err("every captured participant must have input".into());
    }
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let (simulator, fixture) = if let Some(recorded) = recorded_effects {
        let recording = captured.join(recorded);
        let hash = digest(&recording)?;
        if previous["events_sha256"].as_str() != Some(hash.as_str()) {
            return Err("recorded effect results must match the stopped run".into());
        }
        (
            oracle::Simulator::recorded_effects(&recording, trace.clone())?,
            json!({"recorded_effects_sha256":hash}),
        )
    } else {
        let config: Value = serde_json::from_slice(&fs::read(root.join("experiment.json"))?)?;
        oracle::Simulator::new(
            task,
            depth,
            config["domain_tool_delay_multiplier"]
                .as_f64()
                .ok_or("domain delay multiplier")?
                * scale,
            trace.clone(),
            &[],
        )?
    };
    save(&output.join("fixture.json"), &fixture)?;
    save(
        &output.join("input.json"),
        &json!({"task":task,"depth":depth,
        "captured_run":captured,"captured_events_sha256":original_hash,
        "runtime":runtime,"runtime_sha256":runtime_sha,
        "captured_runtime_sha256":captured_runtime_sha,"wash_sha256":module_sha,
        "commands":plans.iter().map(|(_, commands)| commands.len()).sum::<usize>(),
        "diagnostic_time_scale":scale,"deadline_seconds":deadline.as_secs(),
        "model_calls":0,"qualification":false,"scored_benchmark":false}),
    )?;
    let (guest, runtime) = process::guest(
        &runtime,
        &module,
        &[task],
        &output,
        trace.clone(),
        simulator,
    )?;
    let started = Instant::now();
    let plans = plans.map(|(agent, commands)| (agent, commands, &guest, &trace));
    let result = hibana_runtime::race(
        async {
            for (agent, commands, guest, trace) in plans {
                async move {
                    for (at, command) in commands {
                        hibana_runtime::host::sleep(
                            Duration::from_secs_f64(at).saturating_sub(started.elapsed()),
                        )
                        .await?;
                        let reply = guest.command(agent, command.clone()).await?;
                        trace.record(
                            "diagnostic_reply",
                            &json!({"agent":agent,"command":command,"reply":reply}),
                        )?;
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
            }
            runtime.await
        },
        hibana_runtime::host::deadline(deadline),
    )
    .await;
    let result = match result {
        Ok(value) => json!({"runtime_result":value}),
        Err(error) => json!({"error":error.to_string()}),
    };
    save(
        &output.join("diagnostic.json"),
        &json!({"result":result,"qualification":false,
        "model_calls":0,"scored_benchmark":false,"note":"Scaled recorded inputs; incomplete prefixes are not EPF-certified completed executions."}),
    )?;
    eprintln!("preserved timeout replay: {result}");
    Ok(())
}
