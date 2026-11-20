//! Free-shell stimuli through actual Wasm, operation files and module Endpoints.
//! Test input scheduling never chooses production guest actions or permissions.
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

struct Case {
    task: &'static str,
    name: &'static str,
    depth: u8,
    delay: f64,
    failures: &'static [(&'static str, u32)],
    adversarial: bool,
    order: Option<Vec<&'static str>>,
}
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("stdout")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("missing actual result: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty result")?
        .map_err(Into::into)
}
async fn operation(guest: &process::Guest, actor: &Value, call: &Value) -> Result<Value> {
    let agent = actor["agent"].as_str().ok_or("agent")?;
    let tool = call["tool"].as_str().ok_or("tool")?;
    let mut arguments = call["arguments"].clone();
    if tool == "commit_changes" {
        let mut names = arguments["modules"]
            .as_str()
            .ok_or("modules")?
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>();
        names.reverse();
        arguments["modules"] = json!(names.join(" , "));
    }
    let encoded = format!("{}{}", " ".repeat(512), serde_json::to_string(&arguments)?)
        .replace('\'', "'\"'\"'");
    let reply = guest
        .command(
            agent,
            format!("printf '%s' '{encoded}' > /service/{tool}; cat /service/{tool}"),
        )
        .await?;
    response(&reply)
}
async fn invalid(guest: &process::Guest, actor: &Value, call: &Value) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("agent")?;
    let tool = call["tool"].as_str().ok_or("tool")?;
    for arguments in [
        "not-json".to_owned(),
        "{}".to_owned(),
        serde_json::to_string(&if tool == "commit_changes" {
            json!({"modules":"AUTH_MODULE,AUTH_MODULE"})
        } else {
            json!({"feature_name":"another_feature"})
        })?,
    ] {
        let reply = guest
            .command(agent, format!("echo '{arguments}' > /service/{tool}"))
            .await?;
        assert!(
            reply["stdout"].as_str().ok_or("stdout")?.contains("wash:"),
            "{reply}"
        );
    }
    Ok(())
}
async fn preparation(guest: &process::Guest, actor: &Value, case: &Case) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("agent")?;
    let calls = actor["calls"].as_array().ok_or("calls")?;
    if case.adversarial {
        for call in &calls[1..] {
            let reply = guest
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
                reply["stdout"].as_str().ok_or("stdout")?.contains("wash:"),
                "{reply}"
            );
        }
        let private=guest.command(agent,"total=0; for x in 1 2 3; do expr $total + $x | read total; done; printf '%s' \"$total\" > /work/count; cat /missing || cat /work/count".into()).await?;
        assert!(private["stdout"].as_str().ok_or("stdout")?.contains('6'));
    }
    for call in &calls[..2] {
        if case.adversarial {
            invalid(guest, actor, call).await?;
        }
        assert_eq!(operation(guest, actor, call).await?["success"], true);
        if case.adversarial {
            let duplicate = guest
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
                duplicate["stdout"]
                    .as_str()
                    .ok_or("stdout")?
                    .contains("wash:"),
                "{duplicate}"
            );
        }
    }
    Ok(())
}
async fn finish(guest: &process::Guest, actor: &Value, case: &Case) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("agent")?;
    let calls = actor["calls"].as_array().ok_or("calls")?;
    loop {
        if case.adversarial {
            invalid(guest, actor, &calls[2]).await?;
        }
        assert_eq!(operation(guest, actor, &calls[2]).await?["success"], true);
        if case.adversarial {
            let duplicate = guest
                .command(
                    agent,
                    format!(
                        "echo '{}' > /service/commit_changes",
                        serde_json::to_string(&calls[2]["arguments"])?
                    ),
                )
                .await?;
            assert!(
                duplicate["stdout"]
                    .as_str()
                    .ok_or("stdout")?
                    .contains("wash:"),
                "{duplicate}"
            );
            invalid(guest, actor, &calls[3]).await?;
        }
        let verdict = operation(guest, actor, &calls[3]).await?;
        if case.adversarial {
            let duplicate = guest
                .command(
                    agent,
                    format!(
                        "echo '{}' > /service/run_local_tests",
                        serde_json::to_string(&calls[3]["arguments"])?
                    ),
                )
                .await?;
            assert!(
                duplicate["stdout"]
                    .as_str()
                    .ok_or("stdout")?
                    .contains("wash:"),
                "{duplicate}"
            );
        }
        if case.task != "1H" || verdict["success"] == true {
            break;
        }
        if case.adversarial {
            invalid(guest, actor, &calls[1]).await?;
        }
        assert_eq!(operation(guest, actor, &calls[1]).await?["success"], true);
    }
    if case.adversarial {
        for call in calls {
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
    case: &Case,
) -> Result<()> {
    if let Some(order) = &case.order {
        for &name in order {
            let actor = actors
                .iter()
                .find(|a| a["agent"] == name)
                .ok_or("ordered participant")?;
            preparation(guest, actor, case).await?;
            finish(guest, actor, case).await?;
        }
        return Ok(());
    }
    if case.adversarial {
        async {
            for actor in actors {
                async move {
                    preparation(guest, &actor, case).await?;
                    finish(guest, &actor, case).await?;
                };
            }
            Ok(())
        }
        .await?;
    } else {
        // Only this capacity stimulus synchronizes actual prepared results.
        // It neither withholds a world effect nor adds a production barrier.
        async {
            for actor in actors.clone() {
                async move {
                    preparation(guest, &actor, case).await?;
                };
            }
            Ok(())
        }
        .await?;
        async {
            for actor in actors {
                async move {
                    finish(guest, &actor, case).await?;
                };
            }
            Ok(())
        }
        .await?;
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the fixed-source codebase binary, real wash and common 3/5/7-person worlds"]
async fn direct_files_keep_original_module_effects_and_retries() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let runtime =
        PathBuf::from(std::env::var_os("CHOREOFIX_CODEBASE_WASI_BINARY").ok_or("codebase binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_CODEBASE_BUILD_RECEIPT").ok_or("codebase receipt")?,
    );
    let auditor =
        PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("auditor")?).canonicalize()?;
    let root = PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "codebase-direct-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let receipt = archive_build_receipt(&runtime, &receipt, &directory)?;
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut assignments = Vec::new();
    for task in ["1E", "1M", "1H"] {
        for (name, depth, delay, failures, adversarial) in [
            ("depth-0", 0, 0.01, &[][..], true),
            ("depth-1", 1, 0.01, &[][..], true),
            ("depth-2", 2, 0.01, &[][..], true),
            (
                "external-test-failures",
                if task == "1H" { 2 } else { 0 },
                0.01,
                if task == "1H" {
                    &[("run_local_tests", 2), ("run_local_tests", 3)][..]
                } else {
                    &[("run_local_tests", 0)][..]
                },
                true,
            ),
            ("original-delay", 0, 1.0, &[][..], false),
        ] {
            assignments.push(Case {
                task,
                name,
                depth,
                delay,
                failures,
                adversarial,
                order: None,
            });
        }
    }
    let [a, b, c, d, e, f, g] = [
        "DEVELOPER_A",
        "DEVELOPER_B",
        "DEVELOPER_C",
        "DEVELOPER_D",
        "DEVELOPER_E",
        "DEVELOPER_F",
        "DEVELOPER_G",
    ];
    for (task, name, order) in [
        ("1E", "abc", vec![a, b, c]),
        ("1E", "acb", vec![a, c, b]),
        ("1E", "bac", vec![b, a, c]),
        ("1E", "bca", vec![b, c, a]),
        ("1E", "cab", vec![c, a, b]),
        ("1E", "cba", vec![c, b, a]),
        ("1M", "abcde", vec![a, b, c, d, e]),
        ("1M", "bcdea", vec![b, c, d, e, a]),
        ("1M", "cdeab", vec![c, d, e, a, b]),
        ("1M", "deabc", vec![d, e, a, b, c]),
        ("1M", "eabcd", vec![e, a, b, c, d]),
        ("1M", "edcba", vec![e, d, c, b, a]),
        ("1H", "abcdefg", vec![a, b, c, d, e, f, g]),
        ("1H", "bcdefga", vec![b, c, d, e, f, g, a]),
        ("1H", "cdefgab", vec![c, d, e, f, g, a, b]),
        ("1H", "defgabc", vec![d, e, f, g, a, b, c]),
        ("1H", "efgabcd", vec![e, f, g, a, b, c, d]),
        ("1H", "fgabcde", vec![f, g, a, b, c, d, e]),
        ("1H", "gabcdef", vec![g, a, b, c, d, e, f]),
        ("1H", "gfedcba", vec![g, f, e, d, c, b, a]),
    ] {
        assignments.push(Case {
            task,
            name,
            depth: 0,
            delay: 0.01,
            failures: &[],
            adversarial: false,
            order: Some(order),
        });
    }
    assignments.push(Case {
        task: "1H",
        name: "b-repairs-first",
        depth: 0,
        delay: 0.01,
        failures: &[("run_local_tests", 0)],
        adversarial: false,
        order: Some(vec![b, c, d, e, f, g, a]),
    });
    save(&directory.join("assignments.json"),&json!(assignments.iter().map(|c|json!({"task":c.task,"name":c.name,"depth":c.depth,"delay":c.delay,"failures":c.failures,"adversarial_inputs":c.adversarial,"participant_order":c.order})).collect::<Vec<_>>()))?;
    // Complementary invocations retain the full inventory and name their
    // tested range. A selected range cannot claim the whole qualification.
    let selected = std::env::var_os("CHOREOFIX_CODEBASE_CASES")
        .map(|value| {
            value
                .into_string()
                .map_err(|_| "codebase range must be UTF-8")
        })
        .transpose()?
        .map(|value| value.split(',').map(str::to_owned).collect::<Vec<_>>());
    if let Some(selected) = &selected {
        if selected.iter().enumerate().any(|(index, name)| {
            selected[..index].contains(name)
                || !assignments
                    .iter()
                    .any(|case| *name == format!("{}-{}", case.task, case.name))
        }) {
            return Err("codebase range contains an unknown or repeated assignment".into());
        }
        save(&directory.join("selected-cases.json"), &json!(selected))?;
    }
    let expected = selected.as_ref().map_or(assignments.len(), Vec::len);
    let mut outcomes = Vec::new();
    for case in &assignments {
        if selected
            .as_ref()
            .is_some_and(|selected| !selected.contains(&format!("{}-{}", case.task, case.name)))
        {
            continue;
        }
        let output = directory.join(format!("{}-{}", case.task, case.name));
        fs::create_dir(&output)?;
        let projection = crate::projection::archive(&runtime, &output, case.task)?;
        save(
            &output.join("case.json"),
            &json!({"task":case.task,"depth":case.depth,"domain_delay_multiplier":case.delay,"external_failure_specs":case.failures,"adversarial_inputs":case.adversarial,"participant_order":case.order,"test_barriers":if case.order.is_some(){"each developer completes before the next starts designing"}else if case.adversarial{"none"}else{"shell stimuli wait for actual design and implementation results before submitting commits"},"model_calls":0,"scored_benchmark":false,"build":receipt,"wasm_sha256":digest(wasm)?,"runtime_binary_sha256":projection["runtime_binary_sha256"],"compiled_projection_sha256":digest(&output.join("projection.json"))?,"auditor_binary_sha256":digest(&auditor)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,"harness_source_sha256":digest(Path::new("tools/runner/src/codebase_boundary.rs"))?,"audit_source_sha256":digest(Path::new("tools/runner/src/codebase_audit.rs"))?}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) = oracle::Simulator::new(
            case.task,
            case.depth,
            case.delay,
            trace.clone(),
            case.failures,
        )?;
        let (guest, run) = process::guest(&runtime, wasm, &[case.task], &output, trace, simulator)?;
        let actors = fixture["fixtures"].as_array().ok_or("actors")?.clone();
        let result = hibana_runtime::race(
            async {
                async {
                    match case.task {
                        "1E" => {
                            inputs(
                                &guest,
                                <[Value; 3]>::try_from(actors).map_err(|_| "actor count")?,
                                case,
                            )
                            .await?
                        }
                        "1M" => {
                            inputs(
                                &guest,
                                <[Value; 5]>::try_from(actors).map_err(|_| "actor count")?,
                                case,
                            )
                            .await?
                        }
                        "1H" => {
                            inputs(
                                &guest,
                                <[Value; 7]>::try_from(actors).map_err(|_| "actor count")?,
                                case,
                            )
                            .await?
                        }
                        _ => return Err("unknown codebase task".into()),
                    };
                };
                let result = run.await?;
                Ok(result)
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(result["runtime_exit_code"], 0, "{result}");
        assert_eq!(result["protocol"]["status"], "complete");
        assert_eq!(
            result["score"]["complete"],
            case.task == "1H" || case.failures.is_empty()
        );
        assert_eq!(result["score"]["violations"], json!([]));
        if let Some(order) = &case.order {
            let events = result["score"]["events"].as_array().ok_or("score events")?;
            let mut expected = Vec::new();
            for &name in order {
                expected.extend(
                    ["design_feature", "implement_code", "commit_changes"]
                        .map(|tool| (name, tool, true)),
                );
                if case.name == "b-repairs-first" {
                    expected.push((name, "run_local_tests", false));
                    expected.extend(
                        ["implement_code", "commit_changes"].map(|tool| (name, tool, true)),
                    );
                }
                expected.push((name, "run_local_tests", true));
            }
            let actual = events
                .iter()
                .map(|event| {
                    Ok((
                        event["agent"].as_str().ok_or("agent")?,
                        event["tool"].as_str().ok_or("effect")?,
                        event["success"].as_bool().ok_or("effect outcome")?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(actual, expected);
        }
        let status = Command::new(&auditor)
            .arg("--check-protocol-mutants")
            .arg(output.join("events.jsonl"))
            .arg(output.join("mutants"))
            .stdout(Stdio::null())
            .stderr(Stdio::from(fs::File::create(
                output.join("mutants-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!("codebase audit failed ({status}): {}", output.display()).into());
        }
        fs::copy(output.join("mutants/audit.json"), output.join("audit.json"))?;
        let audit: Value = serde_json::from_slice(&fs::read(output.join("audit.json"))?)?;
        let mutants: Value =
            serde_json::from_slice(&fs::read(output.join("mutants/summary.json"))?)?;
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["dynamic_protocol"]["status"], "passed");
        if !case.adversarial {
            assert_eq!(
                audit["max_disjoint_commits"],
                match case.task {
                    _ if case.order.is_some() => 1,
                    "1E" => 1,
                    "1M" => 2,
                    "1H" => 3,
                    _ => unreachable!(),
                },
                "{audit}"
            );
        }
        outcomes.push(json!({"task":case.task,"case":case.name,"audit":audit,"record_mutations_rejected":mutants["record_mutations_rejected"],"runtime":result}));
        save(
            &directory.join("progress.json"),
            &json!({"cases":outcomes,"model_calls":0,"scored_benchmark":false}),
        )?;
    }
    assert_eq!(outcomes.len(), expected);
    save(
        &directory.join("summary.json"),
        &json!({"status":if expected == assignments.len(){"passed"}else{"passed_selected_cases"},"required_total":assignments.len(),"tested_count":expected,"cases":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    println!("{}", directory.display());
    Ok(())
}
