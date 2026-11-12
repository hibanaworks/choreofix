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
            let duplicate = guest
                .command(agent, format!("echo '{arguments}' > /service/{tool}"))
                .await?;
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
    assert_eq!(count(&simulator.observed_events().await?), 2);
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

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified dining runtime, wash and unchanged original environments"]
async fn all_dining_levels_use_both_resource_endpoints_through_wasi() -> Result<()> {
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
    let mut summary = Vec::new();
    for (task, adversarial) in [
        ("9E", false),
        ("9M", false),
        ("9H", false),
        ("9E", true),
        ("9M", true),
        ("9H", true),
    ] {
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
        let audit = philosophers_audit::audit(&case.join("events.jsonl"))?;
        assert_eq!(audit["max_disjoint_eaters"], actors.len() / 2);
        assert_eq!(audit["actual_resource_endpoint_returns"], 2 * actors.len());
        save(&case.join("audit.json"), &audit)?;
        let mutants =
            philosophers_audit::check_mutants(&case.join("events.jsonl"), &case.join("mutants"))?;
        assert_eq!(
            mutants["mutants"]
                .as_array()
                .ok_or("mutation verdicts")?
                .len(),
            19
        );
        summary.push(
            json!({"task":task,"model_calls":0,"scored_benchmark":false,"audit":audit,
            "mutants":mutants,"events_sha256":digest(&case.join("events.jsonl"))?}),
        );
        save(&root.join("summary.json"), &json!(summary))?;
    }
    println!("{}", root.display());
    Ok(())
}
