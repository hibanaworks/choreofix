//! Scripted shell inputs drive the actual WASI guests and original environment.
use crate::{
    evidence::{Result, Trace, digest, save},
    oracle, philosophers_audit, process, snapshot,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("terminal output")?;
    let start = text.find('{').ok_or("service result missing")?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service result")?
        .map_err(Into::into)
}

async fn participant(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    (actor, adversarial): (Value, bool),
) -> Result<Value> {
    let agent = actor["agent"].as_str().ok_or("participant")?;
    let count = |events: &[Value]| {
        events
            .iter()
            .filter(|event| event["agent"] == agent)
            .count()
    };
    let private = guest
        .command(
            agent,
            "x=3; expr $x + 4 > /work/note; cat /missing || cat /work/note".into(),
        )
        .await?;
    assert!(
        private["stdout"]
            .as_str()
            .ok_or("private output")?
            .contains("7\n")
    );
    if adversarial {
        for arguments in [json!({}), json!({"philosopher":"any display name"})] {
            let early = guest
                .command(agent, format!("echo '{arguments}' > /service/eat"))
                .await?;
            assert!(
                early["stdout"]
                    .as_str()
                    .ok_or("early output")?
                    .contains("wash:"),
                "{early}"
            );
            assert_eq!(count(&simulator.observed_events().await?), 0);
        }
    }
    let mut results = Vec::new();
    for call in actor["calls"].as_array().ok_or("original calls")? {
        let tool = call["tool"].as_str().ok_or("tool")?;
        if adversarial {
            for arguments in [json!({}), json!({"philosopher":42})] {
                let before = count(&simulator.observed_events().await?);
                let rejected = guest
                    .command(agent, format!("echo '{arguments}' > /service/{tool}"))
                    .await?;
                assert!(
                    rejected["stdout"]
                        .as_str()
                        .ok_or("rejection output")?
                        .contains("wash:"),
                    "{rejected}"
                );
                assert_eq!(count(&simulator.observed_events().await?), before);
            }
        }
        if adversarial && tool == "think" {
            // Rejected thoughts must not enable a meal even after multiple attempts.
            let early = guest
                .command(
                    agent,
                    "echo '{\"philosopher\":\"ready\"}' > /service/eat".into(),
                )
                .await?;
            assert!(
                early["stdout"]
                    .as_str()
                    .ok_or("early meal output")?
                    .contains("wash:"),
                "{early}"
            );
            assert_eq!(count(&simulator.observed_events().await?), 0);
        }
        // This original argument is a display value; it must not change the
        // participant or its actual fork pair. Padding crosses a guest chunk.
        let arguments = json!({"philosopher":"ANOTHER_DISPLAY_NAME"});
        let accepted = guest
            .command(
                agent,
                format!(
                    "printf '%s' '{}{arguments}' > /service/{tool}; cat /service/{tool}",
                    " ".repeat(512)
                ),
            )
            .await?;
        let result = response(&accepted)?;
        assert_eq!(result["success"], true, "{agent}/{tool}: {result}");
        results.push(result);
        if adversarial {
            let before = count(&simulator.observed_events().await?);
            let command = format!("echo '{arguments}' > /service/{tool}");
            if tool == "think" {
                let reply = guest
                    .command(agent, format!("{command} && cat /service/think"))
                    .await?;
                let thought = response(&reply)?;
                assert_eq!(thought["success"], true, "{reply}");
                assert_eq!(thought["data"]["philosopher"], agent);
                assert_eq!(count(&simulator.observed_events().await?), before + 1);
            } else {
                let duplicate = guest.command(agent, command).await?;
                assert!(
                    duplicate["stdout"]
                        .as_str()
                        .ok_or("duplicate output")?
                        .contains("wash:"),
                    "{duplicate}"
                );
                assert_eq!(count(&simulator.observed_events().await?), before);
            }
        }
    }
    assert_eq!(
        count(&simulator.observed_events().await?),
        2 + usize::from(adversarial)
    );
    let private = guest.command(agent, "cat /work/note".into()).await?;
    assert!(
        private["stdout"]
            .as_str()
            .ok_or("saved private output")?
            .contains("7\n")
    );
    let exit = guest.command(agent, "exit 0".into()).await?;
    assert_eq!(exit["exit_status"], 0);
    Ok(json!({"kind":"participant","agent":agent,"results":results}))
}

#[hibana_runtime::scope]
async fn inputs<const N: usize>(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    (actors, adversarial): ([Value; N], bool),
) -> Result<Vec<Value>> {
    let mut results: [Value; N] = core::array::from_fn(|_| Value::Null);
    let mut actors = actors.into_iter();
    let inputs = results
        .each_mut()
        .map(|output| (actors.next().expect("one input per participant"), output));
    async {
        for (actor, output) in inputs {
            async move {
                *output = participant(guest, simulator, (actor, adversarial)).await?;
            };
        }
        Ok(())
    }
    .await?;
    Ok(results.into_iter().collect())
}

#[hibana_runtime::scope]
async fn actual_wasi_dining(task: &str, adversarial: bool) -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?)
            .join(format!(
                "philosophers-rust-boundary-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
            ));
    fs::create_dir_all(&root)?;
    let case = root.join(format!(
        "{task}-{}",
        if adversarial { "adversarial" } else { "normal" }
    ));
    fs::create_dir(&case)?;
    let versions = snapshot::freeze(task, &case)?;
    save(
        &case.join("case.json"),
        &json!({"task":task,"depth":0,"model_calls":0,
            "kind":"scripted_actual_wasi_boundary","scored_benchmark":false,"adversarial_inputs":adversarial,
            "domain_delay_multiplier":1.0,"test_barriers":"none","versions":versions}),
    )?;
    let trace = Trace::create(&case.join("events.jsonl"))?;
    let (simulator, fixture) = oracle::Simulator::new(task, 0, 1.0, trace.clone(), &[])?;
    let (guest, runtime) = process::guest(
        &case.join("runtime").canonicalize()?,
        &case.join("wash.wasm").canonicalize()?,
        &[task],
        &case,
        trace,
        simulator.clone(),
    )?;
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?;
    let (runtime, mut outcomes) = hibana_runtime::race(
        async {
            async { (runtime).await };
            async {
                match task {
                    "9E" => {
                        inputs(
                            &guest,
                            &simulator,
                            (
                                <[Value; 3]>::try_from(actors.clone())
                                    .map_err(|_| "dining actor count")?,
                                adversarial,
                            ),
                        )
                        .await
                    }
                    "9M" => {
                        inputs(
                            &guest,
                            &simulator,
                            (
                                <[Value; 5]>::try_from(actors.clone())
                                    .map_err(|_| "dining actor count")?,
                                adversarial,
                            ),
                        )
                        .await
                    }
                    "9H" => {
                        inputs(
                            &guest,
                            &simulator,
                            (
                                <[Value; 7]>::try_from(actors.clone())
                                    .map_err(|_| "dining actor count")?,
                                adversarial,
                            ),
                        )
                        .await
                    }
                    _ => Err("unknown dining task".into()),
                }
            };
        },
        hibana_runtime::host::deadline(Duration::from_secs(120)),
    )
    .await?;
    outcomes.push(json!({"kind":"runtime","result":runtime}));
    save(&case.join("outcomes.json"), &json!(outcomes))?;
    assert_eq!(outcomes.len(), actors.len() + 1);
    assert_eq!(runtime["runtime_exit_code"], 0);
    assert_eq!(runtime["protocol"]["status"], "complete");
    assert_eq!(runtime["score"]["complete"], true);
    assert_eq!(runtime["score"]["violations"], json!([]));
    assert_eq!(runtime["backend_errors"], json!([]));
    let (audit, mutants) =
        philosophers_audit::check_mutants(&case.join("events.jsonl"), &case.join("mutants"))?;
    assert_eq!(audit["max_disjoint_eaters"], actors.len() / 2);
    assert_eq!(audit["actual_resource_endpoint_returns"], 2 * actors.len());
    save(&case.join("audit.json"), &audit)?;
    assert_eq!(
        mutants["mutants"]
            .as_array()
            .ok_or("mutation verdicts")?
            .len(),
        20
    );
    save(
        &root.join("summary.json"),
        &json!([{"task":task,"adversarial_inputs":adversarial,"model_calls":0,
            "scored_benchmark":false,"audit":audit,"mutants":mutants,
            "events_sha256":digest(&case.join("events.jsonl"))?}]),
    )?;
    println!("{}", root.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn three_philosophers_normal() -> Result<()> {
    actual_wasi_dining("9E", false).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn five_philosophers_normal() -> Result<()> {
    actual_wasi_dining("9M", false).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn seven_philosophers_normal() -> Result<()> {
    actual_wasi_dining("9H", false).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn three_philosophers_adversarial() -> Result<()> {
    actual_wasi_dining("9E", true).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn five_philosophers_adversarial() -> Result<()> {
    actual_wasi_dining("9M", true).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and original environment"]
async fn seven_philosophers_adversarial() -> Result<()> {
    actual_wasi_dining("9H", true).await
}

/// Each diner finishes before the next guest starts thinking. A fixed priority
/// or an all-thoughts barrier would stall a legal schedule in this test.
#[hibana_runtime::main]
#[test]
#[ignore = "requires the dining executable and wash Wasm"]
async fn three_philosophers_all_meal_orders() -> Result<()> {
    verify_action_words(
        "9E",
        "three-diner-orders",
        crate::qualification::DINING_9E_ORDERS.iter().copied(),
    )
    .await
}

/// With every other guest idle, each seat must be able to finish first.
/// Rotations cover both relative orders of every neighboring pair.
#[hibana_runtime::main]
#[test]
#[ignore = "requires the dining executable and wash Wasm"]
async fn five_philosophers_any_first_seat() -> Result<()> {
    verify_action_words(
        "9M",
        "five-first-seats",
        crate::qualification::DINING_9M_ORDERS.iter().copied(),
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the dining executable and wash Wasm"]
async fn seven_philosophers_any_first_seat() -> Result<()> {
    verify_action_words(
        "9H",
        "seven-first-seats",
        crate::qualification::DINING_9H_ORDERS.iter().copied(),
    )
    .await
}

// In the upstream observer, lower case denotes a thought and upper case a meal.
#[hibana_runtime::scope]
pub(super) async fn verify_action_words<'a>(
    task: &str,
    folder: &str,
    schedules: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(root)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence directory")?);
    fs::create_dir_all(&evidence)?;
    let output = evidence.join(folder);
    fs::create_dir(&output)?;
    let executable = PathBuf::from(
        std::env::var_os("CHOREOFIX_PHILOSOPHERS_WASI_BINARY").ok_or("dining executable")?,
    )
    .canonicalize()?;
    let wash = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let count = match task {
        "9E" => 3,
        "9M" => 5,
        "9H" => 7,
        _ => return Err(format!("unknown dining task {task}").into()),
    };
    let diners = &crate::philosopher_identity::AGENTS[..count];
    crate::projection::archive(&executable, &output, task)?;
    let program = digest(&executable)?;
    let module = digest(wash)?;
    let projection = digest(&output.join("projection.json"))?;
    let harness = digest(&std::env::current_exe()?)?;
    let world_hash = digest(Path::new("benchmark/simulator-manifest.json"))?;
    for (name, word) in schedules {
        let steps: Vec<_> = word
            .chars()
            .map(|action| match action {
                'a'..='g' => Ok((action.to_ascii_uppercase(), "think")),
                'A'..='G' => Ok((action, "eat")),
                _ => Err("unknown dining action"),
            })
            .collect::<std::result::Result<_, _>>()?;
        let order: String = word.chars().filter(char::is_ascii_uppercase).collect();
        let case = output.join(name);
        fs::create_dir(&case)?;
        fs::hard_link(output.join("projection.json"), case.join("projection.json"))?;
        save(
            &case.join("case.json"),
            &json!({"task":task,"depth":0,"meal_order":order,"actions":word,
            "model_calls":0,"scored_benchmark":false,"runtime_binary_sha256":program,
            "compiled_projection_sha256":projection,"guest_wasm_sha256":module,
            "harness_binary_sha256":harness,"simulator_manifest_sha256":world_hash,
            "domain_delay_multiplier":0.01}),
        )?;
        let events = case.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (world, _) = oracle::Simulator::new(task, 0, 0.01, trace.clone(), &[])?;
        let (guest, trial) = process::guest(&executable, wash, &[task], &case, trace, world)?;
        let ((), result) = hibana_runtime::race(async {
            async {
                for &(seat, tool) in &steps {
                    let diner = format!("PHILOSOPHER_{seat}");
                    let reply = guest.command(&diner, format!(
                            "echo '{{\"philosopher\":\"{diner}\"}}' > /service/{tool}; cat /service/{tool}"
                        )).await?;
                    let effect = response(&reply)?;
                    assert_eq!(effect["success"], true, "{diner}/{tool}: {reply}");
                    assert_eq!(effect["data"]["philosopher"], diner);
                }
                for &diner in diners {
                    assert_eq!(guest.command(diner, "exit 0".into()).await?["exit_status"], 0);
                }
                Ok(())
            };
            async { trial.await };
        }, hibana_runtime::host::deadline(Duration::from_secs(60))).await?;
        save(&case.join("result.json"), &result)?;
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["protocol"]["status"], "complete");
        assert_eq!(result["backend_errors"], json!([]));
        let expected: Vec<_> = steps
            .iter()
            .map(|(seat, tool)| (format!("PHILOSOPHER_{seat}"), *tool))
            .collect();
        let scored = result["score"]["events"]
            .as_array()
            .ok_or("simulator events")?;
        assert!(
            scored
                .iter()
                .map(|e| (e["agent"].as_str(), e["tool"].as_str()))
                .eq(expected
                    .iter()
                    .map(|(agent, tool)| (Some(agent.as_str()), Some(*tool)))),
            "{name}: {scored:?}"
        );
        let audit = philosophers_audit::audit(&events)?;
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        assert_eq!(audit["actual_resource_endpoint_returns"], 2 * count);
        assert_eq!(audit["violations"], 0);
        save(&case.join("audit.json"), &audit)?;
    }
    Ok(())
}
