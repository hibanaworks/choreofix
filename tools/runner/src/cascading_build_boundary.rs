//! Actual five-wash path; protocol and common-world evidence are kept distinct.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("build terminal output")?;
    let offset = text
        .find('{')
        .ok_or_else(|| format!("missing build reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[offset..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty build reply")?
        .map_err(Into::into)
}

async fn compile(
    guest: &process::Guest,
    (agent, module): (&str, &str),
    invalid: bool,
) -> Result<Value> {
    if invalid {
        let bad = guest
            .command(
                agent,
                "echo '{\"module_name\":\"unassigned\"}' > /service/compile_module".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("refused file output")?
                .contains("wash:"),
            "{bad}"
        );
        let private = guest
            .command(
                agent,
                "echo private-data > /work/notes; cat /work/notes".into(),
            )
            .await?;
        assert!(
            private["stdout"]
                .as_str()
                .ok_or("private output")?
                .contains("private-data")
        );
    }
    let arguments = json!({"module_name":module});
    let actual = loop {
        let reply = guest.command(agent, format!(
            "printf '%s' '{arguments}' > /service/compile_module; cat /service/compile_module"
        )).await?;
        if !reply["stdout"]
            .as_str()
            .ok_or("compile file output")?
            .contains("wash:")
        {
            break response(&reply)?;
        }
        let update = guest
            .command(agent, "dd if=/service/updates iflag=nonblock".into())
            .await?;
        if !update["stdout"]
            .as_str()
            .ok_or("update file output")?
            .contains("wash:")
        {
            let actual = response(&update)?;
            assert!(
                actual["tool_name"] == "compile_module" || actual["module"].is_string(),
                "{actual}"
            );
        }
    };
    assert_eq!(actual["tool_name"], "compile_module");
    assert_eq!(actual["success"], true);
    assert_eq!(actual["data"]["module"], module);
    Ok(actual)
}

async fn builder(
    guest: &process::Guest,
    ((agent, module), mut actual): ((&str, &str), Value),
    invalid: bool,
) -> Result<()> {
    loop {
        let feedback = loop {
            let reply = guest
                .command(agent, "dd if=/service/test_result iflag=nonblock".into())
                .await?;
            if !reply["stdout"]
                .as_str()
                .ok_or("test feedback output")?
                .contains("wash:")
            {
                break response(&reply)?;
            }
            let update = guest
                .command(agent, "dd if=/service/updates iflag=nonblock".into())
                .await?;
            if !update["stdout"]
                .as_str()
                .ok_or("update file output")?
                .contains("wash:")
            {
                let actual = response(&update)?;
                assert!(
                    actual["tool_name"] == "compile_module" || actual["module"].is_string(),
                    "{actual}"
                );
            }
        };
        assert_eq!(feedback["data"]["input_object"], actual["data"]["object"]);
        let reconsider = 'incoming: loop {
            for path in ["reconsideration", "outcome", "updates"] {
                let reply = guest
                    .command(agent, format!("dd if=/service/{path} iflag=nonblock"))
                    .await?;
                let text = reply["stdout"].as_str().ok_or("incoming file output")?;
                if text.contains("wash:") {
                    continue;
                }
                let data = response(&reply)?;
                match path {
                    "reconsideration" => {
                        assert_eq!(data["module"], module);
                        assert_eq!(data["object"], actual["data"]["object"]);
                        break 'incoming Some(data);
                    }
                    "outcome" => {
                        assert_eq!(data["tool_name"], "link_modules");
                        assert_eq!(data["success"], true);
                        assert_eq!(data["data"]["status"], "linked");
                        break 'incoming None;
                    }
                    _ => assert!(
                        data["tool_name"] == "compile_module" || data["module"].is_string(),
                        "{data}"
                    ),
                }
            }
        };
        if reconsider.is_none() {
            break;
        }
        actual = compile(guest, (agent, module), false).await?;
    }
    if invalid {
        let arguments = json!({"module_name":module});
        let bad = guest
            .command(
                agent,
                format!("echo '{arguments}' > /service/compile_module"),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("duplicate output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

async fn tester(guest: &process::Guest, invalid: bool) -> Result<()> {
    if invalid {
        let bad = guest
            .command(
                "TEST_RUNNER",
                "echo '{\"module_name\":\"unassigned\"}' > /service/run_build_tests_a".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("refused test output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    loop {
        let report = response(
            &guest
                .command("TEST_RUNNER", "cat /service/next_object".into())
                .await?,
        )?;
        if report["tool_name"] == "link_modules" {
            assert_eq!(report["data"]["status"], "linked");
            break;
        }
        assert_eq!(report["tool_name"], "compile_module");
        let module = report["data"]["module"]
            .as_str()
            .ok_or("actual test module")?;
        let suffix = module.strip_prefix("module_").ok_or("module object name")?;
        let arguments = json!({"module_name":module});
        let reply=guest.command("TEST_RUNNER",format!("printf '%s' '{arguments}' > /service/run_build_tests_{suffix}; cat /service/run_build_tests_{suffix}")).await?;
        let actual = response(&reply)?;
        assert_eq!(actual["tool_name"], "run_build_tests");
        assert_eq!(actual["data"]["module"], module);
        assert_eq!(actual["data"]["input_object"], report["data"]["object"]);
    }
    if invalid {
        let bad = guest
            .command(
                "TEST_RUNNER",
                "echo '{\"module_name\":\"module_a\"}' > /service/run_build_tests_a".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("duplicate test output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    assert_eq!(
        guest.command("TEST_RUNNER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

async fn integrator(guest: &process::Guest, invalid: bool, priority: &str) -> Result<()> {
    if invalid {
        let bad = guest
            .command(
                "INTEGRATOR",
                "echo '{\"modules\":\"module_a,module_a\"}' > /service/link_modules".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("refused link output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    let mut try_unknown = invalid;
    let mut try_current = invalid;
    let candidate = loop {
        let ready = guest
            .command(
                "INTEGRATOR",
                "dd if=/service/candidate iflag=nonblock".into(),
            )
            .await?;
        if !ready["stdout"]
            .as_str()
            .ok_or("ready candidate output")?
            .contains("wash:")
        {
            break response(&ready)?;
        }
        let repair = guest
            .command("INTEGRATOR", "dd if=/service/rebuild iflag=nonblock".into())
            .await?;
        if repair["stdout"]
            .as_str()
            .ok_or("repair offer output")?
            .contains("wash:")
        {
            hibana_runtime::host::sleep(Duration::from_millis(5)).await?;
            continue;
        }
        let offered = response(&repair)?;
        let eligible = offered["rebuild"]
            .as_array()
            .ok_or("current repair candidates")?;
        let module = priority
            .chars()
            .map(|letter| format!("module_{}", letter.to_ascii_lowercase()))
            .find(|name| eligible.iter().any(|value| value == name))
            .ok_or("no eligible builder")?;
        let current = ["module_a", "module_b", "module_c"]
            .into_iter()
            .find(|module| !eligible.iter().any(|value| value == module));
        let invalid_choices = [
            try_unknown.then_some("unknown"),
            current.filter(|_| try_current),
        ];
        for rejected in invalid_choices.into_iter().flatten() {
            guest
                .command("INTEGRATOR", format!("echo {rejected} > /service/rebuild"))
                .await?;
            let retry = response(
                &guest
                    .command("INTEGRATOR", "cat /service/rebuild".into())
                    .await?,
            )?;
            assert_eq!(retry["rebuild"], offered["rebuild"]);
            assert_eq!(retry["objects"], offered["objects"]);
            assert_eq!(retry["tests"], offered["tests"]);
            assert_eq!(retry["error"], "choose a module listed in rebuild");
        }
        try_unknown = false;
        if current.is_some() {
            try_current = false;
        }
        let reply = guest
            .command("INTEGRATOR", format!("echo {module} > /service/rebuild"))
            .await?;
        assert!(
            !reply["stdout"]
                .as_str()
                .ok_or("repair choice output")?
                .contains("wash:"),
            "{reply}"
        );
    };
    assert_eq!(candidate["success"], true);
    assert_eq!(
        candidate["objects"]
            .as_array()
            .ok_or("candidate objects")?
            .len(),
        3
    );
    assert_eq!(
        candidate["tests"]
            .as_array()
            .ok_or("candidate tests")?
            .len(),
        3
    );
    let result=response(&guest.command("INTEGRATOR","echo '{\"modules\":\"module_c,module_a,module_b\"}' > /service/link_modules; cat /service/link_modules".into()).await?)?;
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["status"], "linked");
    assert_eq!(result["data"]["issues"], json!([]));
    if invalid {
        let bad = guest
            .command(
                "INTEGRATOR",
                "echo '{\"modules\":\"module_a,module_b,module_c\"}' > /service/link_modules"
                    .into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("duplicate link output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    assert_eq!(
        guest.command("INTEGRATOR", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified cascading build, actual wash and common world"]
async fn actual_five_wasm_build_test_repair_and_link() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("build evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "cascading-build-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_CASCADING_BUILD_WASI_BINARY")
            .ok_or("cascading build binary")?,
    )
    .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_CASCADING_BUILD_BUILD_RECEIPT")
            .ok_or("cascading build receipt")?,
    );
    let auditor =
        PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("frozen protocol auditor")?)
            .canonicalize()?;
    archive_build_receipt(&binary, &input, &directory)?;
    let receipt_hash = digest(&directory.join("build-receipt.json"))?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    let reuse = std::env::var_os("CHOREOFIX_CASCADING_BUILD_REUSE").map(PathBuf::from);
    if reuse.is_some() && std::env::var_os("CI").is_some() {
        return Err("CI must execute its own full Wasm matrix, without reused records".into());
    }
    projection::archive(&binary, &directory, "10M")?;
    let orders = ["ABC", "ACB", "BAC", "BCA", "CAB", "CBA"];
    let injected = &[
        ("run_build_tests", 0),
        ("run_build_tests", 1),
        ("run_build_tests", 2),
    ][..];
    let multi_failure = orders
        .into_iter()
        .zip(orders.into_iter().rev())
        .map(|(order, priority)| (0, order, priority, true, 0.01, injected));
    for (depth, order, priority, invalid, delay, failures) in
        [(0, "ABC", "ABC", true, 0.01, injected)]
            .into_iter()
            .chain(multi_failure)
            .chain((0..=2).flat_map(|depth| {
                orders
                    .into_iter()
                    .zip(orders.into_iter().rev())
                    .map(move |(order, priority)| (depth, order, priority, false, 0.01, &[][..]))
            }))
            .chain([
                (0, "ABC", "CBA", false, 1.0, &[][..]),
                (0, "CBA", "ABC", false, 1.0, &[][..]),
            ])
    {
        let case_index = outcomes.len();
        let output = directory.join(format!("case{case_index}"));
        fs::create_dir(&output)?;
        fs::hard_link(
            directory.join("projection.json"),
            output.join("projection.json"),
        )?;
        save(
            &output.join("case.json"),
            &json!({"task":"10M","depth":depth,"initial_order":order,"repair_preference":priority,
            "model_calls":0,"scored_benchmark":false,"build_receipt":"../build-receipt.json",
            "build_receipt_sha256":receipt_hash,"invalid_input_recovery":invalid,
            "delay_multiplier":delay,"failure_overrides":failures,
            "initial_order_control":"actual_shell_commands",
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,"test_barriers":"none"}),
        )?;
        let previous = reuse
            .as_ref()
            .map(|root| root.join(format!("case{case_index}")))
            .filter(|path| path.exists());
        let result = if let Some(previous) = &previous {
            let old: Value = serde_json::from_slice(&fs::read(previous.join("case.json"))?)?;
            let current: Value = serde_json::from_slice(&fs::read(output.join("case.json"))?)?;
            for key in [
                "task",
                "depth",
                "initial_order",
                "repair_preference",
                "initial_order_control",
                "invalid_input_recovery",
                "delay_multiplier",
                "failure_overrides",
                "runtime_binary_sha256",
                "guest_wasm_sha256",
                "compiled_projection_sha256",
                "test_barriers",
                "model_calls",
            ] {
                if old[key] != current[key] {
                    return Err(format!("saved case differs at {key}").into());
                }
            }
            let trace = Trace::create(&output.join("environment-check.jsonl"))?;
            let (_, fixture) = oracle::Simulator::new("10M", depth, delay, trace, failures)?;
            let old_fixture: Value =
                serde_json::from_slice(&fs::read(previous.join("simulator-fixture.json"))?)?;
            if fixture != old_fixture {
                return Err("saved case has a different common environment or inputs".into());
            }
            fs::create_dir(output.join("reused-source"))?;
            archive_build_receipt(
                &binary,
                &previous
                    .parent()
                    .ok_or("old case parent")?
                    .join("build-receipt.json"),
                &output.join("reused-source"),
            )?;
            fs::copy(previous.join("case.json"), output.join("reused-case.json"))?;
            for name in [
                "events.jsonl",
                "runtime-result.json",
                "runtime-stderr.log",
                "simulator-fixture.json",
            ] {
                fs::hard_link(previous.join(name), output.join(name))?;
            }
            save(
                &output.join("reuse.json"),
                &json!({"original_case":previous,
                "events_sha256":digest(&output.join("events.jsonl"))?,"wasm_rerun":false,
                "runtime_and_guest_identical":true,"original_environment_and_inputs_identical":true}),
            )?;
            serde_json::from_slice(&fs::read(output.join("runtime-result.json"))?)?
        } else {
            let trace = Trace::create(&output.join("events.jsonl"))?;
            let (simulator, fixture) =
                oracle::Simulator::new("10M", depth, delay, trace.clone(), failures)?;
            save(&output.join("simulator-fixture.json"), &fixture)?;
            let (guest, runtime) =
                process::guest(&binary, module, &["10M"], &output, trace, simulator)?;
            let ((), result) = hibana_runtime::race(
                async {
                    async {
                        async { tester(&guest, invalid).await };
                        async { integrator(&guest, invalid, priority).await };
                        // Only this model-free fixture chooses its command order.
                        // The executable receives no order or resolver selection.
                        let [first, second, third] =
                            <[u8; 3]>::try_from(order.as_bytes())?.map(|role| match role {
                                b'A' => ("BUILDER_A", "module_a"),
                                b'B' => ("BUILDER_B", "module_b"),
                                b'C' => ("BUILDER_C", "module_c"),
                                _ => unreachable!("matrix orders are permutations of ABC"),
                            });
                        let first_result = compile(&guest, first, invalid).await?;
                        async { builder(&guest, (first, first_result), invalid).await };
                        // The third builder is not issuing its compile yet,
                        // but must still consume its actual shared-input update.
                        loop {
                            let reply = guest
                                .command(third.0, "dd if=/service/updates iflag=nonblock".into())
                                .await?;
                            if !reply["stdout"]
                                .as_str()
                                .ok_or("third builder update")?
                                .contains("wash:")
                            {
                                assert_eq!(response(&reply)?["data"]["module"], first.1);
                                break;
                            }
                        }
                        let second_result = compile(&guest, second, invalid).await?;
                        async { builder(&guest, (second, second_result), invalid).await };
                        loop {
                            let reply = guest
                                .command(third.0, "dd if=/service/updates iflag=nonblock".into())
                                .await?;
                            if !reply["stdout"]
                                .as_str()
                                .ok_or("third builder update")?
                                .contains("wash:")
                            {
                                assert_eq!(response(&reply)?["data"]["module"], second.1);
                                break;
                            }
                        }
                        let third_result = compile(&guest, third, invalid).await?;
                        async { builder(&guest, (third, third_result), invalid).await };
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    };
                    async { runtime.await };
                },
                hibana_runtime::host::deadline(Duration::from_secs(180)),
            )
            .await?;
            save(&output.join("runtime-result.json"), &result)?;
            result
        };
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let events = result["score"]["events"]
            .as_array()
            .ok_or("actual build events")?;
        for tool in ["compile_module", "run_build_tests"] {
            let mut calls: Vec<_> = events
                .iter()
                .filter(|event| event["tool"] == tool)
                .collect();
            calls.sort_by(|a, b| {
                a["timestamp"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&b["timestamp"].as_f64().unwrap())
            });
            for pair in calls.windows(2) {
                assert!(
                    pair[0]["completed_at"].as_f64().unwrap()
                        <= pair[1]["timestamp"].as_f64().unwrap()
                );
            }
        }
        let overlaps = events
            .iter()
            .filter(|test| test["tool"] == "run_build_tests")
            .flat_map(|test| {
                events.iter().filter(move |compile| {
                    compile["tool"] == "compile_module"
                        && compile["timestamp"].as_f64().unwrap()
                            < test["completed_at"].as_f64().unwrap()
                        && test["timestamp"].as_f64().unwrap()
                            < compile["completed_at"].as_f64().unwrap()
                })
            })
            .count();
        if delay == 1.0 {
            assert!(
                overlaps > 0,
                "normal delays must exhibit actual build/test overlap"
            );
        }
        if !failures.is_empty() {
            let mut tests: Vec<_> = events
                .iter()
                .filter(|event| event["tool"] == "run_build_tests")
                .collect();
            tests.sort_by(|a, b| {
                a["timestamp"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&b["timestamp"].as_f64().unwrap())
            });
            // FailureSpec counts fresh assessments. Stale-object checks may
            // interleave differently with builds; match the actual injected
            // results instead of assuming fixed global call positions.
            let injected = tests
                .iter()
                .filter(|test| test["result"]["status"] == "injected_failure")
                .collect::<Vec<_>>();
            assert_eq!(injected.len(), 3);
            assert!(injected.iter().all(|test| test["success"] == false));
            for agent in ["BUILDER_A", "BUILDER_B", "BUILDER_C"] {
                assert!(
                    events
                        .iter()
                        .filter(|event| event["agent"] == agent)
                        .count()
                        >= 2
                );
            }
        }
        // Audit each already-completed record in its own process. All source
        // and input checks remain enabled; workspace is released between cases.
        let status = Command::new(&auditor)
            .arg("--check-cascading-build-mutants")
            .arg(output.join("events.jsonl"))
            .arg(output.join("record-mutants"))
            .stdout(Stdio::null())
            .stderr(Stdio::from(fs::File::create_new(
                output.join("mutant-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!(
                "saved record mutation failed ({status}) at {}",
                output.display()
            )
            .into());
        }
        let mutants: Value =
            serde_json::from_slice(&fs::read(output.join("record-mutants/report.json"))?)?;
        let checked = &mutants["baseline"];
        assert_eq!(checked["status"], "passed");
        assert_eq!(checked["dynamic_protocol"]["status"], "passed");
        assert_eq!(checked["dynamic_protocol_checked"], true);
        save(&output.join("audit.json"), checked)?;
        outcomes.push(
            json!({"case":output,"task":"10M","depth":depth,"initial_order":order,"repair_preference":priority,
            "invalid_input_recovery":invalid,"delay_multiplier":delay,"failure_overrides":failures,
            "protocol_complete":true,"common_score_complete":true,"actual_build_test_overlaps":overlaps,
            "audit_file":"audit.json","audit_sha256":digest(&output.join("audit.json"))?,"auditor_sha256":digest(&auditor)?,
            "dynamic_protocol":checked["dynamic_protocol"]["status"],
            "captured_records":checked["dynamic_protocol"]["captured_records"],
            "application_effect_audit_complete":true,"effect_audit_sha256":digest(&output.join("audit.json"))?,
            "record_mutants_rejected":mutants["checked"].as_array().ok_or("mutants")?.len(),
            "wasm_execution_reused":previous.is_some()}),
        );
    }
    if outcomes.len() != 27 {
        return Err("all 27 declared Wasm conditions must be checked".into());
    }
    save(
        &directory.join("report.json"),
        &json!({"status":"passed_partial_qualification",
        "model_calls":0,"scored_benchmark":false,"cases":outcomes}),
    )?;
    Ok(())
}
