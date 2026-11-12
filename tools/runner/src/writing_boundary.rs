//! Shell stimuli for actual section-delegating Wasm guests and original tasks.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, writing_audit,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("stdout")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("no JSON result: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty result")?
        .map_err(Into::into)
}
async fn participant(guest: &process::Guest, actor: Value, terminal: bool) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("agent")?;
    let early = guest
        .command(agent, "echo '{}' > /service/write_to_document".into())
        .await?;
    assert!(early["stdout"].as_str().ok_or("stdout")?.contains("wash:"));
    let private = guest
        .command(
            agent,
            "x=3; expr $x + 4 > /work/note; cat /work/note".into(),
        )
        .await?;
    assert!(private["stdout"].as_str().unwrap().contains("7\n"));
    for call in actor["calls"].as_array().ok_or("calls")? {
        let tool = call["tool"].as_str().ok_or("tool")?;
        let rejected = guest
            .command(
                agent,
                format!("echo '{{}}' > /service/{tool}; cat /service/result"),
            )
            .await?;
        assert_eq!(response(&rejected)?["status"], "invalid_arguments");
        let key = call["arguments"]
            .as_object()
            .ok_or("arguments")?
            .keys()
            .next()
            .ok_or("key")?;
        let wrong = serde_json::to_string(&json!({key:"a_different_object"}))?;
        let rejected = guest
            .command(
                agent,
                format!("echo '{wrong}' > /service/{tool}; cat /service/result"),
            )
            .await?;
        assert_eq!(response(&rejected)?["status"], "invalid_arguments");
        let mut arguments = call["arguments"].clone();
        if tool == "write_to_document" {
            let duplicate = json!({"sections":format!("{},{}",arguments["sections"].as_str().ok_or("sections")?,actor["sections"][0].as_str().ok_or("section")?)});
            let rejected = guest
                .command(
                    agent,
                    format!(
                        "echo '{}' > /service/{tool}; cat /service/result",
                        serde_json::to_string(&duplicate)?
                    ),
                )
                .await?;
            assert_eq!(response(&rejected)?["status"], "invalid_arguments");
            let mut names: Vec<_> = arguments["sections"]
                .as_str()
                .unwrap()
                .split(',')
                .map(str::trim)
                .collect();
            names.reverse();
            arguments["sections"] = json!(names.join(" , "));
        }
        let encoded = format!("{}{}", " ".repeat(512), serde_json::to_string(&arguments)?)
            .replace('\'', "'\"'\"'");
        let reply = guest
            .command(
                agent,
                format!("printf '%s' '{encoded}' > /service/{tool}; cat /service/result"),
            )
            .await?;
        assert_eq!(
            response(&reply)?["success"],
            !(terminal && tool == "review_own_work")
        );
        let duplicate = guest
            .command(
                agent,
                format!(
                    "echo '{}' > /service/{tool}",
                    serde_json::to_string(&arguments)?
                ),
            )
            .await?;
        assert!(duplicate["stdout"].as_str().unwrap().contains("wash:"));
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
#[hibana_runtime::scope]
async fn inputs<const N: usize>(
    guest: &process::Guest,
    actors: [Value; N],
    terminal: bool,
) -> Result<()> {
    for actor in actors {
        async move {
            participant(guest, actor, terminal).await?;
        };
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified writing binary, actual wash and unchanged 3/5/7-person simulators"]
async fn all_writing_levels_use_actual_section_endpoints() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_WRITING_WASI_BINARY").ok_or("writing binary missing")?,
    );
    let receipt_path = PathBuf::from(
        std::env::var_os("CHOREOFIX_WRITING_BUILD_RECEIPT").ok_or("build receipt missing")?,
    );
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "writing-sections-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let receipt = archive_build_receipt(&runtime, &receipt_path, &directory)?;
    let mut cases = Vec::new();
    for task in ["2E", "2M", "2H"] {
        for (name, depth, delay, terminal) in [
            ("depth-0", 0, 0.01, false),
            ("depth-1", 1, 0.01, false),
            ("depth-2", 2, 0.01, false),
            ("terminal-review-failure", 0, 0.01, true),
            ("original-delay", 0, 1.0, false),
        ] {
            let output = directory.join(format!("{task}-{name}"));
            fs::create_dir(&output)?;
            let projection = crate::projection::archive(&runtime, &output, task)?;
            let trace = Trace::create(&output.join("events.jsonl"))?;
            let failures = if terminal {
                vec![("review_own_work", 0)]
            } else {
                vec![]
            };
            let (simulator, fixture) =
                oracle::Simulator::new(task, depth, delay, trace.clone(), &failures)?;
            save(
                &output.join("case.json"),
                &json!({"task":task,"depth":depth,"domain_delay_multiplier":delay,"external_failure_specs":failures,
                "test_barriers":"none","model_calls":0,"scored_benchmark":false,"build":receipt,"wasm_sha256":digest(wasm)?,
                "runtime_binary_sha256":projection["runtime_binary_sha256"],
                "compiled_projection_sha256":digest(&output.join("projection.json"))?,
                "harness_binary_sha256":digest(&std::env::current_exe()?)?,"harness_source_sha256":digest(Path::new("tools/runner/src/writing_boundary.rs"))?,
                "oracle_source_sha256":digest(Path::new("tools/runner/src/oracle.rs"))?,"audit_source_sha256":digest(Path::new("tools/runner/src/writing_audit.rs"))?}),
            )?;
            let (guest, runtime_loop) =
                process::guest(&runtime, wasm, &[task], &output, trace, simulator)?;
            let actors = fixture["fixtures"].as_array().ok_or("actors")?.clone();
            let result = hibana_runtime::race(
                async {
                    async {
                        match task {
                            "2E" => {
                                inputs(
                                    &guest,
                                    <[Value; 3]>::try_from(actors)
                                        .map_err(|_| "writing actor count")?,
                                    terminal,
                                )
                                .await?
                            }
                            "2M" => {
                                inputs(
                                    &guest,
                                    <[Value; 5]>::try_from(actors)
                                        .map_err(|_| "writing actor count")?,
                                    terminal,
                                )
                                .await?
                            }
                            "2H" => {
                                inputs(
                                    &guest,
                                    <[Value; 7]>::try_from(actors)
                                        .map_err(|_| "writing actor count")?,
                                    terminal,
                                )
                                .await?
                            }
                            _ => return Err("unknown writing task".into()),
                        };
                    };
                    let result = runtime_loop.await?;
                    Ok(result)
                },
                hibana_runtime::host::deadline(Duration::from_secs(120)),
            )
            .await?;
            assert_eq!(result["runtime_exit_code"], 0, "{result}");
            assert_eq!(result["protocol"]["status"], "complete");
            assert_eq!(result["score"]["complete"], !terminal);
            assert_eq!(result["score"]["violations"], json!([]));
            let audit = writing_audit::audit(&output.join("events.jsonl"))?;
            if delay == 1.0 {
                assert_eq!(
                    audit["max_disjoint_writes"],
                    match task {
                        "2E" => 1,
                        "2M" => 2,
                        "2H" => 3,
                        _ => unreachable!(),
                    },
                    "{audit}"
                );
            }
            let mutants = writing_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("mutants"),
            )?;
            save(&output.join("audit.json"), &audit)?;
            save(&output.join("result.json"), &result)?;
            cases.push(
                json!({"task":task,"case":name,"audit":audit,"mutants":mutants,"runtime":result}),
            );
            save(
                &directory.join("summary.json"),
                &json!({"cases":cases,"model_calls":0,"scored_benchmark":false}),
            )?;
        }
    }
    println!("{}", directory.display());
    Ok(())
}
