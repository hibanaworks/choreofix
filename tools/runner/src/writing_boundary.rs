//! Shell stimuli for actual section-delegating Wasm guests and original tasks.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
    for call in actor["calls"].as_array().ok_or("calls")?.iter().skip(1) {
        let early = guest
            .command(
                agent,
                format!(
                    "echo '{}' > /service/{}",
                    serde_json::to_string(&call["arguments"])?,
                    call["tool"].as_str().ok_or("tool")?
                ),
            )
            .await?;
        assert!(
            early["stdout"].as_str().ok_or("stdout")?.contains("wash:"),
            "{early}"
        );
    }
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
            .command(agent, format!("echo '{{}}' > /service/{tool}"))
            .await?;
        assert!(
            rejected["stdout"]
                .as_str()
                .ok_or("stdout")?
                .contains("wash:"),
            "{rejected}"
        );
        let key = call["arguments"]
            .as_object()
            .ok_or("arguments")?
            .keys()
            .next()
            .ok_or("key")?;
        let wrong = serde_json::to_string(&json!({key:"a_different_object"}))?;
        let rejected = guest
            .command(agent, format!("echo '{wrong}' > /service/{tool}"))
            .await?;
        assert!(
            rejected["stdout"]
                .as_str()
                .ok_or("stdout")?
                .contains("wash:"),
            "{rejected}"
        );
        let mut arguments = call["arguments"].clone();
        if tool == "write_to_document" {
            let duplicate = json!({"sections":format!("{},{}",arguments["sections"].as_str().ok_or("sections")?,actor["sections"][0].as_str().ok_or("section")?)});
            let rejected = guest
                .command(
                    agent,
                    format!(
                        "echo '{}' > /service/{tool}",
                        serde_json::to_string(&duplicate)?
                    ),
                )
                .await?;
            assert!(
                rejected["stdout"]
                    .as_str()
                    .ok_or("stdout")?
                    .contains("wash:"),
                "{rejected}"
            );
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
                format!("printf '%s' '{encoded}' > /service/{tool}; cat /service/{tool}"),
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
    for call in actor["calls"].as_array().ok_or("calls")? {
        let late = guest
            .command(
                agent,
                format!(
                    "echo '{}' > /service/{}",
                    serde_json::to_string(&call["arguments"])?,
                    call["tool"].as_str().ok_or("tool")?
                ),
            )
            .await?;
        assert!(
            late["stdout"].as_str().ok_or("stdout")?.contains("wash:"),
            "{late}"
        );
    }
    let private = guest
        .command(
            agent,
            "expr 8 + 5 > /work/finished; cat /work/finished".into(),
        )
        .await?;
    assert!(private["stdout"].as_str().ok_or("stdout")?.contains("13\n"));
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
    controlled_overlap: bool,
) -> Result<()> {
    if controlled_overlap {
        // Test stimuli synchronize completed preparation, so rejected-input
        // processing cannot obscure the actual independent effect intervals.
        // Production guests and localsides have no test barrier.
        async {
            for actor in actors.clone() {
                async move {
                    let agent = actor["agent"].as_str().ok_or("agent")?;
                    for call in actor["calls"].as_array().ok_or("calls")?.iter().take(2) {
                        let tool = call["tool"].as_str().ok_or("tool")?;
                        let reply = guest
                            .command(
                                agent,
                                format!(
                                    "echo '{}' > /service/{tool}; cat /service/{tool}",
                                    serde_json::to_string(&call["arguments"])?
                                ),
                            )
                            .await?;
                        assert_eq!(response(&reply)?["success"], true, "{reply}");
                    }
                };
            }
            Ok(())
        }
        .await?;
        async {
            for actor in actors {
                async move {
                    let agent = actor["agent"].as_str().ok_or("agent")?;
                    for call in actor["calls"].as_array().ok_or("calls")?.iter().skip(2) {
                        let tool = call["tool"].as_str().ok_or("tool")?;
                        let reply = guest
                            .command(
                                agent,
                                format!(
                                    "echo '{}' > /service/{tool}; cat /service/{tool}",
                                    serde_json::to_string(&call["arguments"])?
                                ),
                            )
                            .await?;
                        assert_eq!(response(&reply)?["success"], true, "{reply}");
                    }
                    assert_eq!(
                        guest.command(agent, "exit 0".into()).await?["exit_status"],
                        0
                    );
                };
            }
            Ok(())
        }
        .await?;
        return Ok(());
    }
    for actor in actors {
        async move {
            participant(guest, actor, terminal).await?;
        };
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified writing binary, actual wash and common 3/5/7-person worlds"]
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
    let auditor = PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("writing auditor")?)
        .canonicalize()?;
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
                "test_barriers":if delay == 1.0 { "shell stimuli wait for all actual literature and draft results before submitting writes" } else { "none" },"model_calls":0,"scored_benchmark":false,"build":receipt,"wasm_sha256":digest(wasm)?,
                "auditor_binary_sha256":digest(&auditor)?,
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
                                    delay == 1.0,
                                )
                                .await?
                            }
                            "2M" => {
                                inputs(
                                    &guest,
                                    <[Value; 5]>::try_from(actors)
                                        .map_err(|_| "writing actor count")?,
                                    terminal,
                                    delay == 1.0,
                                )
                                .await?
                            }
                            "2H" => {
                                inputs(
                                    &guest,
                                    <[Value; 7]>::try_from(actors)
                                        .map_err(|_| "writing actor count")?,
                                    terminal,
                                    delay == 1.0,
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
            save(&output.join("result.json"), &result)?;
            let status = Command::new(&auditor)
                .arg("--check-writing-mutants")
                .arg(output.join("events.jsonl"))
                .arg(output.join("mutants"))
                .stdout(Stdio::null())
                .stderr(Stdio::from(fs::File::create(
                    output.join("mutants-stderr.log"),
                )?))
                .status()?;
            if !status.success() {
                return Err(
                    format!("writing audit failed ({status}): {}", output.display()).into(),
                );
            }
            fs::copy(output.join("mutants/audit.json"), output.join("audit.json"))?;
            let audit: Value = serde_json::from_slice(&fs::read(output.join("audit.json"))?)?;
            assert_eq!(audit["status"], "passed");
            assert_eq!(audit["dynamic_protocol"]["status"], "passed");
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
            let mutants: Value =
                serde_json::from_slice(&fs::read(output.join("mutants/summary.json"))?)?;
            cases.push(
                json!({"task":task,"case":name,"audit":audit,"mutants":mutants,"runtime":result}),
            );
            save(
                &directory.join("summary.json"),
                &json!({"cases":cases,"model_calls":0,"scored_benchmark":false}),
            )?;
        }
    }
    assert_eq!(cases.len(), 15);
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","cases":cases,"model_calls":0,"scored_benchmark":false}),
    )?;
    println!("{}", directory.display());
    Ok(())
}

/// Each writer completes through wash before the next writer starts reading.
#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified writing executable and wash Wasm"]
async fn writers_can_finish_in_different_orders() -> Result<()> {
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    let executable = PathBuf::from(
        std::env::var_os("CHOREOFIX_WRITING_WASI_BINARY").ok_or("writing executable")?,
    )
    .canonicalize()?;
    let wash = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let root = PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?)
        .join("writer-orders");
    fs::create_dir_all(root.parent().ok_or("orders directory")?)?;
    fs::create_dir(&root)?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_WRITING_BUILD_RECEIPT").ok_or("writing build receipt")?,
    );
    let receipt = archive_build_receipt(&executable, &receipt, &root)?;
    for (task, orders) in [
        ("2E", &["ABC", "ACB", "BAC", "BCA", "CAB", "CBA"][..]),
        (
            "2M",
            &["ABCDE", "BCDEA", "CDEAB", "DEABC", "EABCD", "EDCBA"][..],
        ),
        (
            "2H",
            &[
                "ABCDEFG", "BCDEFGA", "CDEFGAB", "DEFGABC", "EFGABCD", "FGABCDE", "GABCDEF",
                "GFEDCBA",
            ][..],
        ),
    ] {
        let root = root.join(task);
        fs::create_dir(&root)?;
        let projection = crate::projection::archive(&executable, &root, task)?;
        for &order in orders {
            let case = root.join(order);
            fs::create_dir(&case)?;
            fs::hard_link(root.join("projection.json"), case.join("projection.json"))?;
            let events = case.join("events.jsonl");
            let trace = Trace::create(&events)?;
            let (world, fixture) = oracle::Simulator::new(task, 0, 0.01, trace.clone(), &[])?;
            save(
                &case.join("case.json"),
                &json!({"task":task,"depth":0,"order":order,"model_calls":0,"scored_benchmark":false,"build":receipt,"runtime_binary_sha256":projection["runtime_binary_sha256"],"compiled_projection_sha256":digest(&case.join("projection.json"))?,"guest_wasm_sha256":digest(wash)?,"simulator_manifest_sha256":digest(Path::new("benchmark/simulator-manifest.json"))?,"harness_binary_sha256":digest(&std::env::current_exe()?)?}),
            )?;
            let (guest, trial) = process::guest(&executable, wash, &[task], &case, trace, world)?;
            let ((), result) = hibana_runtime::race(
                async {
                    async {
                        for seat in order.bytes() {
                            participant(
                                &guest,
                                fixture["fixtures"][usize::from(seat - b'A')].clone(),
                                false,
                            )
                            .await?;
                        }
                        Ok(())
                    };
                    async { trial.await };
                },
                hibana_runtime::host::deadline(Duration::from_secs(90)),
            )
            .await?;
            save(&case.join("result.json"), &result)?;
            assert_eq!(result["runtime_exit_code"], 0);
            assert_eq!(result["protocol"]["status"], "complete");
            assert_eq!(result["score"]["complete"], true);
            assert_eq!(result["score"]["violations"], json!([]));
            let expected: Vec<_> = order
                .chars()
                .map(|seat| format!("RESEARCHER_{seat}"))
                .collect();
            let actual: Vec<_> = result["score"]["events"]
                .as_array()
                .ok_or("world events")?
                .iter()
                .filter(|e| e["tool"] == "write_to_document")
                .map(|e| e["agent"].as_str().ok_or("writer"))
                .collect::<std::result::Result<_, _>>()?;
            assert_eq!(actual, expected);
            let audit =
                crate::journal::Evidence::open(&events)?.audit(crate::writing_audit::check)?;
            assert_eq!(audit["actual_resource_endpoint_returns"], 2 * order.len());
            assert_eq!(audit["violations"], 0);
            save(&case.join("audit.json"), &audit)?;
        }
    }
    Ok(())
}
