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

async fn builder(
    guest: &process::Guest,
    (agent, module): (&str, &str),
    invalid: bool,
) -> Result<()> {
    if invalid {
        let reply = guest.command(agent,
            "echo '{\"module_name\":\"unassigned\"}' > /service/compile_module; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "invalid_arguments");
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
    loop {
        let arguments = json!({"module_name":module});
        let reply = guest
            .command(
                agent,
                format!("printf '%s' '{arguments}' > /service/compile_module; cat /service/result"),
            )
            .await?;
        let result = response(&reply)?;
        assert_eq!(result["tool_name"], "compile_module", "{reply}");
        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["module"], module);
        let next = &result["coordination"]["continuation"];
        if next["tool_name"] == "link_modules" {
            assert_eq!(next["data"]["status"], "linked");
            break;
        }
        assert_eq!(next["module"], module);
        assert_eq!(next["object"], result["data"]["object"]);
    }
    if invalid {
        let arguments = json!({"module_name":module});
        let reply = guest
            .command(
                agent,
                format!("echo '{arguments}' > /service/compile_module; cat /service/result"),
            )
            .await?;
        assert_eq!(response(&reply)?["status"], "out_of_order");
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

async fn tester(guest: &process::Guest, first_module: &str, invalid: bool) -> Result<()> {
    if invalid {
        let reply = guest.command("TEST_RUNNER",
            "echo '{\"module_name\":\"unassigned\"}' > /service/run_build_tests; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "invalid_arguments");
    }
    let mut module = first_module.to_owned();
    loop {
        let arguments = json!({"module_name":module});
        let reply = guest
            .command(
                "TEST_RUNNER",
                format!(
                    "printf '%s' '{arguments}' > /service/run_build_tests; cat /service/result"
                ),
            )
            .await?;
        let result = response(&reply)?;
        assert_eq!(result["tool_name"], "run_build_tests", "{reply}");
        assert_eq!(result["data"]["module"], module);
        let next = &result["coordination"]["next"];
        if next["tool_name"] == "link_modules" {
            assert_eq!(next["data"]["status"], "linked");
            break;
        }
        assert_eq!(next["tool_name"], "compile_module");
        module = next["data"]["module"]
            .as_str()
            .ok_or("next actual test module")?
            .to_owned();
    }
    if invalid {
        let reply = guest.command("TEST_RUNNER",
            "echo '{\"module_name\":\"module_a\"}' > /service/run_build_tests; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "out_of_order");
    }
    assert_eq!(
        guest.command("TEST_RUNNER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

async fn integrator(guest: &process::Guest, invalid: bool) -> Result<()> {
    if invalid {
        let reply = guest.command("INTEGRATOR",
            "echo '{\"modules\":\"module_a,module_a\"}' > /service/link_modules; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "invalid_arguments");
    }
    let reply = guest.command("INTEGRATOR",
        "echo '{\"modules\":\"module_c,module_a,module_b\"}' > /service/link_modules; cat /service/result".into()).await?;
    let result = response(&reply)?;
    assert_eq!(result["success"], true, "{reply}");
    assert_eq!(result["data"]["status"], "linked");
    assert_eq!(result["data"]["issues"], json!([]));
    if invalid {
        let reply = guest.command("INTEGRATOR",
            "echo '{\"modules\":\"module_a,module_b,module_c\"}' > /service/link_modules; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "out_of_order");
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
    let orders = [
        ("ABC", "module_a"),
        ("ACB", "module_a"),
        ("BAC", "module_b"),
        ("BCA", "module_b"),
        ("CAB", "module_c"),
        ("CBA", "module_c"),
    ];
    for (depth, (order, first_module), invalid, delay, failures) in (0..=2)
        .flat_map(|depth| orders.map(move |order| (depth, order, false, 0.01, &[][..])))
        .chain(orders.map(|order| {
            (
                0,
                order,
                true,
                0.01,
                &[
                    ("run_build_tests", 0),
                    ("run_build_tests", 1),
                    ("run_build_tests", 2),
                ][..],
            )
        }))
        .chain([
            (0, orders[0], false, 1.0, &[][..]),
            (0, orders[5], false, 1.0, &[][..]),
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
            &json!({"task":"10M","depth":depth,"initial_order":order,
            "model_calls":0,"scored_benchmark":false,"build_receipt":"../build-receipt.json",
            "build_receipt_sha256":receipt_hash,"invalid_input_recovery":invalid,
            "delay_multiplier":delay,"failure_overrides":failures,
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
                process::guest(&binary, module, &["10M", order], &output, trace, simulator)?;
            let ((), result) = hibana_runtime::race(
                async {
                    async {
                        async { builder(&guest, ("BUILDER_A", "module_a"), invalid).await };
                        async { builder(&guest, ("BUILDER_B", "module_b"), invalid).await };
                        async { builder(&guest, ("BUILDER_C", "module_c"), invalid).await };
                        async { tester(&guest, first_module, invalid).await };
                        async { integrator(&guest, invalid).await };
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
            // The world counts only fresh assessments for FailureSpec. The
            // first two global tests are stale-input rejections, so eligible
            // calls 0/1/2 correspond to actual test occurrences 2/3/4 here.
            for index in [2, 3, 4] {
                assert_eq!(tests[index]["success"], false);
                assert_eq!(tests[index]["result"]["status"], "injected_failure");
            }
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
        let audit_file = fs::File::create_new(output.join("audit.json"))?;
        let status = Command::new(&auditor)
            .arg("--audit-protocol")
            .arg(output.join("events.jsonl"))
            .stdout(Stdio::from(audit_file))
            .stderr(Stdio::from(fs::File::create_new(
                output.join("audit-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!(
                "saved protocol audit failed ({status}); retain {}",
                output.display()
            )
            .into());
        }
        let checked: Value = serde_json::from_slice(&fs::read(output.join("audit.json"))?)?;
        assert_eq!(checked["status"], "passed_protocol_audit");
        assert_eq!(checked["dynamic_protocol"]["status"], "passed");
        assert_eq!(checked["application_effects_certified"], false);
        let status = Command::new(&auditor)
            .arg("--audit-cascading-build")
            .arg(output.join("events.jsonl"))
            .stdout(Stdio::from(fs::File::create_new(
                output.join("effect-audit.json"),
            )?))
            .stderr(Stdio::from(fs::File::create_new(
                output.join("effect-audit-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!(
                "saved effect audit failed ({status}) at {}",
                output.display()
            )
            .into());
        }
        let effects: Value = serde_json::from_slice(&fs::read(output.join("effect-audit.json"))?)?;
        assert_eq!(effects["status"], "passed");
        let status = Command::new(&auditor)
            .arg("--check-cascading-build-mutants")
            .arg(output.join("events.jsonl"))
            .arg(output.join("record-mutants"))
            .stdout(Stdio::from(fs::File::create_new(
                output.join("mutant-result.json"),
            )?))
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
        outcomes.push(
            json!({"case":output,"task":"10M","depth":depth,"initial_order":order,
            "invalid_input_recovery":invalid,"delay_multiplier":delay,"failure_overrides":failures,
            "protocol_complete":true,"common_score_complete":true,"actual_build_test_overlaps":overlaps,
            "audit_file":"audit.json","audit_sha256":digest(&output.join("audit.json"))?,"auditor_sha256":digest(&auditor)?,
            "dynamic_protocol":checked["dynamic_protocol"]["status"],
            "captured_records":checked["dynamic_protocol"]["captured_records"],
            "application_effect_audit_complete":true,"effect_audit_sha256":digest(&output.join("effect-audit.json"))?,
            "record_mutants_rejected":mutants["checked"].as_array().ok_or("mutants")?.len(),
            "wasm_execution_reused":previous.is_some()}),
        );
    }
    if outcomes.len() != 26 {
        return Err("all 26 declared Wasm conditions must be checked".into());
    }
    save(
        &directory.join("report.json"),
        &json!({"status":"passed_partial_qualification",
        "model_calls":0,"scored_benchmark":false,"cases":outcomes}),
    )?;
    Ok(())
}
