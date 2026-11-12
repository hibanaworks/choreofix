//! Actual wash/WASI requests against the unchanged codebase simulator.
//! Scripted stimuli test the boundary and do not count as model experiments.
use crate::{
    evidence::{Result, Trace, digest, save},
    oracle, process, snapshot,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("terminal stdout")?;
    let start = text.find('{').ok_or("service result missing")?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service result")?
        .map_err(Into::into)
}

fn count(events: &[Value], agent: &str, tool: &str) -> usize {
    events
        .iter()
        .filter(|event| event["agent"] == agent && event["tool"] == tool)
        .count()
}

async fn developer(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    actor: Value,
    task: &str,
) -> Result<Vec<Value>> {
    let agent = actor["agent"].as_str().ok_or("original participant")?;
    let calls = actor["calls"].as_array().ok_or("original operations")?;
    assert_eq!(calls.len(), 4);

    // Ordinary private work and failures must not consume a shared operation.
    let private = guest.command(agent,
        "total=0; for x in 1 2 3; do expr $total + $x | read total; done; printf '%s' \"$total\" > /work/count; cat /missing || cat /work/count".into()).await?;
    assert!(
        private["stdout"]
            .as_str()
            .ok_or("private stdout")?
            .contains('6')
    );
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "design_feature"),
        0
    );
    let early = guest
        .command(
            agent,
            format!("echo '{}' > /service/commit_changes", calls[2]["arguments"]),
        )
        .await?;
    assert!(
        early["stdout"]
            .as_str()
            .ok_or("early stdout")?
            .contains("wash:")
    );
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "commit_changes"),
        0
    );

    let mut feedback = Vec::new();
    for call in calls {
        let tool = call["tool"].as_str().ok_or("original tool")?;
        for arguments in [
            "not-json".to_string(),
            "{}".to_string(),
            if tool == "commit_changes" {
                "{\"modules\":\"AUTH_MODULE,AUTH_MODULE\"}".into()
            } else {
                "{\"feature_name\":\"another_participants_feature\"}".into()
            },
        ] {
            let before = count(&simulator.observed_events().await?, agent, tool);
            let invalid = guest
                .command(
                    agent,
                    format!("echo '{arguments}' > /service/{tool}; cat /service/result"),
                )
                .await?;
            assert_eq!(response(&invalid)?["status"], "invalid_arguments");
            assert_eq!(
                count(&simulator.observed_events().await?, agent, tool),
                before
            );
        }
        // This exceeds one guest write chunk and exercises ABI normalization.
        let padding = " ".repeat(512);
        let valid = guest
            .command(
                agent,
                format!(
                    "printf '%s' '{padding}{}' > /service/{tool}; cat /service/result",
                    call["arguments"]
                ),
            )
            .await?;
        let result = response(&valid)?;
        if tool != "run_local_tests" {
            assert_eq!(result["success"], true, "{agent}/{tool}: {result}");
        }
        feedback.push(result);
        let before = count(&simulator.observed_events().await?, agent, tool);
        assert_eq!(before, 1, "{agent}/{tool} effect count");
        let duplicate = guest
            .command(
                agent,
                format!("echo '{}' > /service/{tool}", call["arguments"]),
            )
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate stdout")?
                .contains("wash:")
        );
        assert_eq!(
            count(&simulator.observed_events().await?, agent, tool),
            before
        );
    }
    if task == "1H" {
        while feedback.last().ok_or("test result")?["success"] == false {
            let private=guest.command(agent,
                "note=reconsider; printf '%s' \"$note\" > /work/revision; cat /work/revision; echo '{}' > /service/implement_code; cat /service/result".into()).await?;
            assert_eq!(response(&private)?["status"], "invalid_arguments");
            for call in &calls[1..] {
                let tool = call["tool"].as_str().ok_or("revision operation")?;
                let before = count(&simulator.observed_events().await?, agent, tool);
                let reply = guest
                    .command(
                        agent,
                        format!(
                            "echo '{}' > /service/{tool}; cat /service/result",
                            call["arguments"]
                        ),
                    )
                    .await?;
                let result = response(&reply)?;
                if tool != "run_local_tests" {
                    assert_eq!(result["success"], true);
                }
                assert_eq!(
                    count(&simulator.observed_events().await?, agent, tool),
                    before + 1
                );
                feedback.push(result);
            }
        }
        assert_eq!(feedback.last().ok_or("test result")?["success"], true);
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(feedback)
}

async fn assigned_operations(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    actor: Value,
    _task: &str,
) -> Result<Vec<Value>> {
    let agent = actor["agent"].as_str().ok_or("original participant")?;
    let calls = actor["calls"].as_array().ok_or("original operations")?;
    assert_eq!(calls.len(), 4);
    let mut feedback = Vec::new();
    for call in calls {
        let tool = call["tool"].as_str().ok_or("original tool")?;
        let reply = guest
            .command(
                agent,
                format!(
                    "echo '{}' > /service/{tool}; cat /service/result",
                    call["arguments"]
                ),
            )
            .await?;
        let result = response(&reply)?;
        assert_eq!(result["success"], true, "{agent}/{tool}: {result}");
        assert_eq!(count(&simulator.observed_events().await?, agent, tool), 1);
        feedback.push(result);
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(feedback)
}

// Fixture setup and evidence checks contain no application Endpoint operations.
fn output(label: &str) -> Result<std::path::PathBuf> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "{label}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    Ok(directory)
}

#[hibana_runtime::scope]
async fn case<const N: usize>(
    directory: &Path,
    task: &str,
    depth: u8,
    delay: f64,
    failures: &[(&str, u32)],
    drive: impl AsyncFn(&process::Guest, &oracle::Simulator, Value, &str) -> Result<Vec<Value>>,
    held_commit_agents: &[&str],
) -> Result<Value> {
    let case = directory.join(format!("{task}-depth{depth}"));
    fs::create_dir(&case)?;
    let versions = snapshot::freeze(task, &case)?;
    save(
        &case.join("case.json"),
        &json!({"task":task,"depth":depth,
        "kind":"deterministic_rust_wasi_boundary","model_calls":0,"scored_benchmark":false,"versions":versions,
        "test_barriers":held_commit_agents, "domain_delay_multiplier":delay,"external_failure_specs":failures,
        "participant_driver":std::any::type_name_of_val(&drive),
        }),
    )?;
    let trace = Trace::create(&case.join("events.jsonl"))?;
    let (simulator, fixture) = oracle::Simulator::new(task, depth, delay, trace.clone(), failures)?;
    save(&case.join("simulator-fixture.json"), &fixture)?;
    if !held_commit_agents.is_empty() {
        assert_eq!(
            simulator
                .call(json!({"type":"hold_disjoint_commits","agents":held_commit_agents}))
                .await?["status"],
            "configured"
        );
    }
    let actors: [Value; N] = fixture["fixtures"]
        .as_array()
        .ok_or("original actors")?
        .clone()
        .try_into()
        .map_err(|_| "fixture participant count differs from the selected protocol")?;
    let (guest, runtime_loop) = process::guest(
        &case.join("runtime").canonicalize()?,
        &case.join("wash.wasm").canonicalize()?,
        &[task],
        &case,
        trace,
        simulator.clone(),
    )?;
    let mut feedback: [Vec<Value>; N] = core::array::from_fn(|_| Vec::new());
    let mut actors = actors.into_iter();
    let participants = feedback
        .each_mut()
        .map(|output| (actors.next().expect("one fixture per participant"), output));
    let result = hibana_runtime::race(
        async {
            let drive = &drive;
            let guest = &guest;
            let simulator = &simulator;

            for (actor, output) in participants {
                async move {
                    *output = drive(guest, simulator, actor, task).await?;
                };
            }
            let result = runtime_loop.await?;
            Ok(result)
        },
        hibana_runtime::host::deadline(Duration::from_secs(90)),
    )
    .await?;
    save(&case.join("result.json"), &result)?;
    let audit = crate::codebase_audit::audit(&case.join("events.jsonl"))?;
    save(&case.join("audit.json"), &audit)?;
    let mutants =
        crate::codebase_audit::check_mutants(&case.join("events.jsonl"), &case.join("mutants"))?;
    assert_eq!(
        result["protocol"],
        json!({"type":"run_result","status":"complete"})
    );
    assert_eq!(result["runtime_exit_code"], 0);
    assert_eq!(result["backend_errors"], json!([]));
    assert_eq!(
        result["score"]["complete"],
        task == "1H" || failures.is_empty()
    );
    assert_eq!(result["score"]["violations"], json!([]));
    assert_eq!(
        result["score"]["events"]
            .as_array()
            .ok_or("original events")?
            .len(),
        audit["domain_calls"]
            .as_u64()
            .ok_or("audited effect count")? as usize
    );
    if !failures.is_empty() && task != "1H" {
        for participant in &feedback {
            assert_eq!(participant.last().ok_or("test verdict")?["success"], false);
        }
    }
    Ok(
        json!({"task":task,"depth":depth,"result":result,"feedback":feedback.as_slice(),"audit":audit,
        "mutants":mutants,"events_sha256":digest(&case.join("events.jsonl"))?,
        "kind":"deterministic_rust_wasi_boundary","model_calls":0,"scored_benchmark":false}),
    )
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires seven actual WASI guests and the unchanged original simulator"]
async fn seven_guests_reconsider_failures_through_delegated_module_endpoints() -> Result<()> {
    let directory = output("codebase-rust-reconsider")?;
    let mut outcomes = Vec::new();
    for depth in [0, 1, 2] {
        outcomes.push(case::<7>(&directory, "1H", depth, 0.01, &[], developer, &[]).await?);
        save(&directory.join("summary.json"), &json!(outcomes))?;
    }
    let repeated = directory.join("two-failed-tests");
    fs::create_dir(&repeated)?;
    let outcome = case::<7>(
        &repeated,
        "1H",
        0,
        0.01,
        &[("run_local_tests", 0), ("run_local_tests", 1)],
        developer,
        &[],
    )
    .await?;
    assert!(
        outcome["audit"]["attempts"]
            .as_object()
            .ok_or("attempts")?
            .values()
            .all(|count| count.as_u64() == Some(3))
    );
    outcomes.push(outcome);
    let probes = directory.join("normal-delays-admission-probes");
    fs::create_dir(&probes)?;
    outcomes.push(case::<7>(&probes, "1H", 0, 1.0, &[], developer, &[]).await?);
    save(&directory.join("summary.json"), &json!(outcomes))?;
    let normal = directory.join("normal-delays");
    fs::create_dir(&normal)?;
    outcomes.push(case::<7>(&normal, "1H", 0, 1.0, &[], assigned_operations, &[]).await?);
    save(&directory.join("summary.json"), &json!(outcomes))?;
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires real WASI; the fixture withholds three independent commit effects"]
async fn three_disjoint_commits_are_admitted_before_any_response() -> Result<()> {
    let directory = output("codebase-rust-held-commits")?;
    let agents = ["DEVELOPER_A", "DEVELOPER_C", "DEVELOPER_E"];
    let outcome = case::<7>(&directory, "1H", 0, 1.0, &[], assigned_operations, &agents).await?;
    let events = fs::read_to_string(directory.join("1H-depth0/events.jsonl"))?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<Vec<Value>, _>>()?;
    let release = events
        .iter()
        .position(|row| row["message"]["type"] == "commits_released")
        .ok_or("held commits were not released")?;
    for agent in agents {
        assert_eq!(
            events[..release]
                .iter()
                .filter(|row| row["direction"] == "test_control"
                    && row["message"]["type"] == "commit_held"
                    && row["message"]["request"]["agent"] == agent)
                .count(),
            1
        );
    }
    assert!(!events[..release].iter().any(
        |row| row["direction"] == "domain_start" && row["message"]["tool"] == "commit_changes"
    ));
    assert_eq!(outcome["audit"]["max_disjoint_commits"], 3);
    save(&directory.join("summary.json"), &outcome)?;
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the built codebase WASI runtime, wash and original simulator environment"]
async fn typed_requests_gate_effects_and_keep_shell_errors_recoverable() -> Result<()> {
    let directory = output("codebase-rust-boundary")?;
    let mut outcomes = Vec::new();
    for task in ["1E", "1M"] {
        for depth in [0, 1, 2] {
            let outcome = match task {
                "1E" => case::<3>(&directory, task, depth, 0.01, &[], developer, &[]).await?,
                "1M" => case::<5>(&directory, task, depth, 0.01, &[], developer, &[]).await?,
                _ => unreachable!(),
            };
            outcomes.push(outcome);
            save(&directory.join("summary.json"), &json!(outcomes))?;
        }
    }
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires real WASI and uses only the upstream FailureSpec environment API"]
async fn terminal_test_errors_close_without_retries() -> Result<()> {
    let directory = output("codebase-rust-terminal-tests")?;
    let mut outcomes = Vec::new();
    for task in ["1E", "1M"] {
        let outcome = match task {
            "1E" => {
                case::<3>(
                    &directory,
                    task,
                    0,
                    0.01,
                    &[("run_local_tests", 0)],
                    developer,
                    &[],
                )
                .await?
            }
            "1M" => {
                case::<5>(
                    &directory,
                    task,
                    0,
                    0.01,
                    &[("run_local_tests", 0)],
                    developer,
                    &[],
                )
                .await?
            }
            _ => unreachable!(),
        };
        outcomes.push(outcome);
        save(&directory.join("summary.json"), &json!(outcomes))?;
    }
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires real WASI and original normal domain delays, without barriers"]
async fn normal_delays_allow_two_disjoint_actual_wasi_commits() -> Result<()> {
    let directory = output("codebase-rust-normal-delays")?;
    let outcome = case::<5>(&directory, "1M", 0, 1.0, &[], developer, &[]).await?;
    save(&directory.join("summary.json"), &outcome)?;
    assert_eq!(outcome["audit"]["max_disjoint_commits"], 2);
    println!("{}", directory.display());
    Ok(())
}
