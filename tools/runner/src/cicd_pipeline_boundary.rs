use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

fn result(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("pipeline shell output")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("missing actual result: {text}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty result")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 16M runtime, build receipt and wash guest"]
async fn actual_wash_pipeline_recovers_builds_qa_and_deployment() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(root)?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_PIPELINE_WASI_BINARY").ok_or("16M executable")?,
    );
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_PIPELINE_BUILD_RECEIPT").ok_or("16M build receipt")?,
    );
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("16M evidence root")?);
    fs::create_dir_all(&evidence)?;
    let output = evidence.join("cicd-pipeline-wasi");
    fs::create_dir(&output)?;
    let build = archive_build_receipt(&runtime, &receipt, &output)?;
    let runtime_sha = digest(&runtime)?;
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let wasm_sha = digest(wasm)?;
    let mut cases = Vec::new();
    for (case, failures, first, expected_effects, expected_production, repair) in [
        ("backend-first", &[][..], "backend", 14, 1, None),
        (
            "staging-deploy-failure",
            &[("deploy_staging", 0)][..],
            "backend",
            11,
            0,
            None,
        ),
        (
            "staging-rejection",
            &[("run_smoke_tests", 0)][..],
            "frontend",
            12,
            0,
            None,
        ),
        (
            "production-smoke-failure",
            &[("run_smoke_tests", 1)][..],
            "frontend",
            14,
            1,
            None,
        ),
        (
            "both-compile-recovery",
            &[("compile_code", 0)][..],
            "frontend",
            16,
            1,
            None,
        ),
        (
            "both-unit-recovery",
            &[("run_unit_tests", 0)][..],
            "backend",
            18,
            1,
            None,
        ),
        (
            "repeated-compile-recovery",
            &[("compile_code", 0), ("compile_code", 1)][..],
            "frontend",
            18,
            1,
            None,
        ),
        (
            "qa-repair-frontend",
            &[("run_integration_tests", 0)][..],
            "backend",
            20,
            1,
            Some(("frontend", true)),
        ),
        (
            "qa-repair-backend",
            &[("run_e2e_tests", 0)][..],
            "frontend",
            20,
            1,
            Some(("backend", true)),
        ),
        (
            "qa-repair-both",
            &[("run_e2e_tests", 0)][..],
            "backend",
            23,
            1,
            Some(("both", true)),
        ),
        (
            "qa-repair-rejected",
            &[("run_integration_tests", 0), ("run_integration_tests", 1)][..],
            "frontend",
            15,
            0,
            Some(("backend", false)),
        ),
        (
            "build-and-qa-recovery",
            &[
                ("compile_code", 0),
                ("run_unit_tests", 0),
                ("run_integration_tests", 0),
                ("run_e2e_tests", 0),
            ][..],
            "frontend",
            26,
            1,
            Some(("backend", true)),
        ),
    ] {
        let directory = output.join(case);
        fs::create_dir(&directory)?;
        let trace = Trace::create(&directory.join("events.jsonl"))?;
        projection::archive(&runtime, &directory, "16M")?;
        let (simulator, fixture) = oracle::Simulator::new("16M", 0, 0.01, trace.clone(), failures)?;
        let complete = repair.map_or_else(
            || {
                failures
                    .iter()
                    .all(|(tool, _)| matches!(*tool, "compile_code" | "run_unit_tests"))
            },
            |(_, passed)| passed,
        );
        save(
            &directory.join("case.json"),
            &json!({
                "task":"16M","runtime_binary_sha256":runtime_sha,
                "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
                "wasm_sha256":wasm_sha,"fixture":fixture,"model_calls":0,
                "scored_benchmark":false,"expected_complete":complete,"failure_overrides":failures,
                "qa_repair":repair.map(|(component, _)| component)
            }),
        )?;
        let (guest, runtime_loop) =
            process::guest(&runtime, wasm, &[], &directory, trace, simulator)?;
        let verdict = hibana_runtime::race(
            async {
                let guest = &guest;
                for (agent, component, artifact, wrong) in [
                    ("FRONTEND_DEV", "frontend", "frontend_image", "backend_image"),
                    ("BACKEND_DEV", "backend", "backend_image", "frontend_image"),
                ] {
                    async move {
                        guest.command(agent, "cat /service/start".into()).await?;
                        for attempt in 0..=usize::from(repair.is_some_and(|(target, _)| target == component || target == "both")) {
                        if attempt != 0 {
                            let requested = result(&guest.command(agent, "cat /service/qa_revise".into()).await?)?;
                            assert_eq!(requested["success"], false);
                            assert!(requested["integration"]["success"] == false || requested["e2e"]["success"] == false);
                            if case == "build-and-qa-recovery" {
                                let early = guest.command(agent, "dd if=/service/qa_done iflag=nonblock".into()).await?;
                                assert!(early["stdout"].as_str().is_some_and(|text|
                                    text.contains("nonblocking read /service/qa_done: Permission denied") && !text.contains('{')
                                ), "early QA result must leave the shell usable: {early}");
                            }
                        }
                        let revision_path = if attempt == 0 { "revise" } else { "qa_revise" };
                        loop {
                            guest.command(agent, "cat /service/workspace".into()).await?;
                            let compiled = result(&guest.command(agent, format!(
                                "printf '%s' '{{\"component\":\"{component}\"}}' > /service/compile_code; cat /service/compile_code"
                            )).await?)?;
                            if compiled["success"] == false {
                                let revision = guest.command(agent, format!("cat /service/{revision_path}")).await?;
                                assert_eq!(result(&revision)?, compiled);
                                continue;
                            }
                            let tested = result(&guest.command(agent, format!(
                                "printf '%s' '{{\"component\":\"{component}\"}}' > /service/run_unit_tests; cat /service/run_unit_tests"
                            )).await?)?;
                            if tested["success"] == false {
                                let revision = guest.command(agent, format!("cat /service/{revision_path}")).await?;
                                assert_eq!(result(&revision)?, tested);
                                continue;
                            }
                            let denied = guest.command(agent, format!(
                                "printf '%s' '{{\"artifact\":\"{wrong}\"}}' > /service/publish_artifact"
                            )).await?;
                            assert!(denied["stdout"].as_str().is_some_and(|text| text.contains("wash:")), "{denied}");
                            let observed = guest.command(agent, format!(
                                "printf '%s' '{{\"artifact\":\"{artifact}\"}}' > /service/publish_artifact; cat /service/publish_artifact"
                            )).await?;
                            let published = result(&observed)?;
                            assert_eq!(published["success"], true);
                            assert_eq!(published["data"]["artifact"], artifact);
                            let verified = guest.command(agent, "cat /service/build_verified".into()).await?;
                            assert_eq!(result(&verified)?, published);
                            break;
                        }
                        }
                        if repair.is_some_and(|(target, _)| target != component && target != "both") {
                            let standby = result(&guest.command(agent, "cat /service/build_verified".into()).await?)?;
                            assert_eq!(standby["data"]["artifact"], artifact);
                        }
                        let tested = result(&guest.command(agent, "cat /service/qa_done".into()).await?)?;
                        assert_eq!(tested["success"], repair.is_none_or(|(_, passed)| passed));
                        assert_eq!(result(&guest.command(agent, "cat /service/result".into()).await?)?["success"], complete);
                        assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    };
                }
                async {
                    let agent = "BUILD_MASTER";
                    let denied = guest.command(agent, format!(
                        "printf '%s' '{{\"first\":\"{first}\"}}' > /service/build_{first}_first; cat /service/build_{first}_first"
                    )).await?;
                    assert!(denied["stdout"].as_str().is_some_and(|text| text.contains("Permission denied")), "{denied}");
                    let initial = guest.command(agent, "cat /service/qa_report".into()).await?;
                    let report = result(&initial)?;
                    assert_eq!(report["success"], repair.is_none());
                    assert_eq!(report["success"], report["integration"]["success"] == true && report["e2e"]["success"] == true);
                    if let Some((component, passed)) = repair {
                        let invalid = guest.command(agent,
                            r#"printf '%s' '{"component":"neither"}' > /service/repair"#.into()
                        ).await?;
                        assert!(invalid["stdout"].as_str().is_some_and(|text| text.contains("wash:")), "{invalid}");
                        let denied = guest.command(agent, format!(
                            "printf '%s' '{{\"component\":\"{component}\"}}' > /service/repair; cat /service/repair"
                        )).await?;
                        assert!(denied["stdout"].as_str().is_some_and(|text| text.contains("Permission denied")), "{denied}");
                        let report = result(&guest.command(agent, "cat /service/qa_report".into()).await?)?;
                        assert_eq!(report["success"], passed);
                        assert_eq!(report["success"], report["integration"]["success"] == true && report["e2e"]["success"] == true);
                    }
                    assert_eq!(result(&guest.command(agent, "cat /service/result".into()).await?)?["success"], complete);
                    assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async {
                    let agent = "QA_ENGINEER";
                    for attempt in 0..=u32::from(repair.is_some()) {
                    guest.command(agent, "cat /service/start".into()).await?;
                    let reply = guest.command(agent,
                        r#"printf '%s' '{"target":"test_harness"}' > /service/pull_artifacts; cat /service/pull_artifacts"#.into()
                    ).await?;
                    assert_eq!(result(&reply)?["success"], true);
                    let mut checks = [("run_integration_tests","api_integration"),("run_e2e_tests","user_flows")];
                    if (first == "backend") != (attempt != 0) {
                        checks.reverse();
                    }
                    for (operation, suite) in checks {
                        let reply = guest.command(agent, format!(
                            "printf '%s' '{{\"suite\":\"{suite}\"}}' > /service/{operation}; cat /service/{operation}"
                        )).await?;
                        assert_eq!(result(&reply)?["success"], !failures.contains(&(operation, attempt)));
                    }
                    }
                    assert_eq!(result(&guest.command(agent, "cat /service/result".into()).await?)?["success"], complete);
                    assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async {
                    let agent = "RELEASE_ENGINEER";
                    let gate = result(&guest.command(agent, "cat /service/start".into()).await?)?;
                    assert_eq!(gate["success"], repair.is_none_or(|(_, passed)| passed));
                    if gate["success"] == false {
                        let denied = guest.command(agent,
                            r#"printf '%s' '{"component":"full_stack"}' | dd of=/service/deploy_staging oflag=nonblock"#.into()
                        ).await?;
                        assert!(denied["stdout"].as_str().is_some_and(|text| text.contains("wash:")), "{denied}");
                        assert_eq!(result(&guest.command(agent, "cat /service/result".into()).await?)?, gate);
                        assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                        return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
                    }
                    let pulled = guest.command(agent,
                        r#"printf '%s' '{"target":"staging_deploy"}' > /service/pull_artifacts; cat /service/pull_artifacts"#.into()
                    ).await?;
                    assert_eq!(result(&pulled)?["success"], true);
                    let deployed = result(&guest.command(agent,
                        r#"printf '%s' '{"component":"full_stack"}' > /service/deploy_staging; cat /service/deploy_staging"#.into()
                    ).await?)?;
                    let staging = if deployed["success"] == true {
                        let readiness = guest.command(agent, "cat /service/staging_ready".into()).await?;
                        assert_eq!(result(&readiness)?, deployed);
                        result(&guest.command(agent,
                            r#"printf '%s' '{"target":"staging"}' > /service/run_staging_smoke; cat /service/run_staging_smoke"#.into()
                        ).await?)?
                    } else {
                        deployed
                    };
                    let promote = staging["success"] == true;
                    let notice = if promote { "promote" } else { "abort" };
                    let reply = guest.command(agent, format!("cat /service/{notice}")).await?;
                    assert_eq!(result(&reply)?, staging);
                    if promote {
                        for (file, arguments) in [
                            ("deploy_production", r#"{"component":"full_stack"}"#),
                            ("run_production_smoke", r#"{"target":"production"}"#),
                        ] {
                            let reply = guest.command(agent, format!(
                                "printf '%s' '{arguments}' > /service/{file}; cat /service/{file}"
                            )).await?;
                            assert_eq!(result(&reply)?["success"], file == "deploy_production" || complete);
                        }
                    }
                    assert_eq!(result(&guest.command(agent, "cat /service/result".into()).await?)?["success"], complete);
                    assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                runtime_loop.await
            },
            hibana_runtime::host::deadline(Duration::from_secs(45)),
        ).await?;
        save(&directory.join("result.json"), &verdict)?;
        assert_eq!(verdict["runtime_exit_code"], 0, "{verdict}");
        assert_eq!(verdict["protocol"]["status"], "complete");
        assert_eq!(verdict["score"]["complete"], complete);
        assert_eq!(verdict["score"]["violations"], json!([]));
        let effects = verdict["score"]["events"]
            .as_array()
            .ok_or("pipeline effects")?;
        assert_eq!(effects.len(), expected_effects);
        assert_eq!(
            effects
                .iter()
                .filter(|row| row["tool"] == "deploy_production")
                .count(),
            expected_production
        );
        let compiles = effects
            .iter()
            .filter(|effect| effect["tool"] == "compile_code")
            .map(|effect| {
                effect["args"]["component"]
                    .as_str()
                    .expect("assigned component")
            })
            .collect::<Vec<_>>();
        let other = if first == "frontend" {
            "backend"
        } else {
            "frontend"
        };
        if let Some((target, passed)) = repair {
            let expected_compiles: &[_] = if case == "build-and-qa-recovery" {
                &[
                    first, other, "frontend", "frontend", "backend", "backend", "backend",
                ]
            } else {
                match target {
                    "frontend" => &[first, other, "frontend"],
                    "backend" => &[first, other, "backend"],
                    "both" => &[first, other, "frontend", "backend"],
                    _ => unreachable!(),
                }
            };
            assert_eq!(compiles, expected_compiles);
            let pulls: Vec<_> = effects
                .iter()
                .filter(|e| e["tool"] == "pull_artifacts" && e["agent"] == "QA_ENGINEER")
                .collect();
            assert_eq!(pulls.len(), 2);
            for component in ["frontend", "backend"] {
                let artifact = format!("{component}_image");
                assert_eq!(
                    pulls[0]["result"]["artifacts"][&artifact]
                        != pulls[1]["result"]["artifacts"][&artifact],
                    target == component || target == "both",
                );
            }
            if !passed {
                assert!(!effects.iter().any(|e| e["agent"] == "RELEASE_ENGINEER"));
            }
        } else {
            match case {
                "both-compile-recovery" => {
                    assert_eq!(compiles, [first, other, first, other]);
                    assert_eq!(
                        effects
                            .iter()
                            .filter(|effect| effect["success"] == false)
                            .count(),
                        2
                    );
                }
                "both-unit-recovery" => {
                    assert_eq!(compiles, [first, other, other, first]);
                    assert_eq!(
                        effects
                            .iter()
                            .filter(|effect| effect["success"] == false)
                            .count(),
                        2
                    );
                }
                "repeated-compile-recovery" => {
                    assert_eq!(compiles, [first, other, first, first, other, other]);
                    assert_eq!(
                        effects
                            .iter()
                            .filter(|effect| effect["success"] == false)
                            .count(),
                        4
                    );
                }
                _ => assert_eq!(compiles, [first, other]),
            }
        }
        let audit =
            crate::journal::Evidence::open(&directory.join("events.jsonl"))?.audit("16M")?;
        assert_eq!(audit["dynamic"]["status"], "passed");
        assert_eq!(audit["runtime_binary_sha256"], runtime_sha);
        assert_eq!(audit["simulator_complete"], complete);
        assert_eq!(
            audit["effects"]
                .as_array()
                .ok_or("checked pipeline effects")?
                .len(),
            effects.len()
        );
        save(&directory.join("audit.json"), &audit)?;
        cases.push(json!({
            "case": case,
            "effects": expected_effects,
            "simulator_complete": complete,
            "captured_records": audit["dynamic"]["captured_records"],
            "events_sha256": digest(&directory.join("events.jsonl"))?,
            "result_sha256": digest(&directory.join("result.json"))?,
            "audit_sha256": digest(&directory.join("audit.json"))?,
        }));
    }
    save(
        &output.join("boundary.json"),
        &json!({
            "status": "passed",
            "task": "16M",
            "build": build,
            "runtime_binary_sha256": runtime_sha,
            "wasm_sha256": wasm_sha,
            "model_calls": 0,
            "scored_benchmark": false,
            "cases": cases,
        }),
    )
}
