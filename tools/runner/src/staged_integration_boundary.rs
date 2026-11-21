//! Free-shell operation stimuli through seven actual Wasm guests; no model calls.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const FEATURES: [&str; 3] = [
    "user_authentication",
    "payment_processing",
    "notification_system",
];
const SUITES: [&str; 3] = [
    "full_suite_after_auth",
    "full_suite_after_payment",
    "full_suite_after_notification",
];
fn response(reply: &Value) -> Result<Value> {
    let stdout = reply["stdout"].as_str().ok_or("guest stdout")?;
    let at = stdout
        .find('{')
        .ok_or_else(|| format!("missing guest result: {reply}"))?;
    serde_json::Deserializer::from_str(&stdout[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty guest result")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 6H runtime, source receipt, wash and neutral simulator"]
async fn seven_actual_wasm_guests_complete_staged_publication() -> Result<()> {
    clean_publication(Schedule::Concurrent).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the fixed 6H executable, actual wash and common simulator"]
async fn each_release_can_finish_before_the_next_developer_starts() -> Result<()> {
    clean_publication(Schedule::StageEach).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 6H executable, wash and common simulator"]
async fn every_merge_order_preserves_required_deployment_order() -> Result<()> {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        clean_publication(Schedule::MergeBeforeStaging(order)).await?;
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Schedule {
    Concurrent,
    StageEach,
    MergeBeforeStaging([usize; 3]),
}

#[hibana_runtime::scope]
async fn clean_publication(schedule: Schedule) -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("staged-integration evidence root")?,
    )
    .join(format!(
        "staged-integration-wasm-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_STAGED_INTEGRATION_WASI_BINARY").ok_or("6H binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_STAGED_INTEGRATION_BUILD_RECEIPT").ok_or("6H build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    crate::projection::archive(&binary, &directory, "6H")?;
    save(
        &directory.join("case.json"),
        &json!({"task":"6H","depth":0,"model_calls":0,"scored_benchmark":false,"developer_starts":match schedule {
            Schedule::Concurrent => json!("concurrent"),
            Schedule::StageEach => json!("after preceding staging validation"),
            Schedule::MergeBeforeStaging(order) => json!({"all_approved_before_merge_order":order.map(|index| FEATURES[index])}),
        },
        "build":receipt,"simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"delay_multiplier":0.01,
        "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
        "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
        "harness_binary_sha256":digest(&std::env::current_exe()?)?,
        "harness_source_sha256":digest(Path::new("tools/runner/src/staged_integration_boundary.rs"))?,
        "audit_source_sha256":digest(Path::new("tools/runner/src/staged_integration_audit.rs"))?}),
    )?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let (simulator, _) = oracle::Simulator::new("6H", 0, 0.01, trace.clone(), &[])?;
    let (guest, runtime) = process::guest(&binary, module, &["6H"], &directory, trace, simulator)?;
    let result = if schedule != Schedule::Concurrent {
        let ((), result) = hibana_runtime::race(
            async {
                async { staged_one_at_a_time(&guest, schedule).await };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(60)),
        )
        .await?;
        result
    } else {
        let (_,(),(),(),(),result)=hibana_runtime::race(async {
        let guest=&guest;
        for (agent,own) in [("DEVELOPER_A",0),("DEVELOPER_B",1),("DEVELOPER_C",2)] {
            async move {
                let feature=FEATURES[own];
                if own == 0 {
                    for (path, expected) in [
                        ("/service", vec!["implement_feature", "integration", "interface", "merge_code", "release", "review"]),
                        ("/service/review", vec!["notification_system", "payment_processing", "user_authentication"]),
                        ("/service/interface", vec!["notification_system", "payment_processing"]),
                    ] {
                        let listing=guest.command(agent,format!("ls {path}")).await?;
                        let actual=listing["stdout"].as_str().ok_or("directory output")?
                            .strip_suffix("wash> ").ok_or("directory command did not finish")?
                            .split_whitespace().collect::<Vec<_>>();
                        assert_eq!(actual,expected,"{path}: {listing}");
                    }
                    // Match the live shell mistake: a complete first JSON object
                    // followed by another chunk, then submission through cp.
                    let malformed=guest.command(agent,
                        "printf '%s' '{\"feature_name\":\"user_authentication\"}' > /work/bad_payload; printf '%s' 'trailing' >> /work/bad_payload; cp /work/bad_payload /service/implement_feature".into()).await?;
                    let error=malformed["stdout"].as_str().ok_or("malformed request reply")?;
                    assert!(error.contains("close output /service/implement_feature: Invalid argument"),"missing failed-operation context: {malformed}");
                }
                let implemented=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                assert_eq!(response(&implemented)?["success"],true,"{agent}: {implemented}");
                for other in FEATURES.into_iter().filter(|other| *other != feature) {
                    let value=guest.command(agent,format!("cat /service/interface/{other}")).await?;
                    assert_eq!(response(&value)?["success"],true,"{agent}: {value}");
                }
                if own == 1 {
                    let early=guest.command(agent,format!("echo '{}' > /service/merge_code",json!({"branch":feature}))).await?;
                    assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied")),"premature merge was not rejected: {early}");
                }
                if own == 2 {
                    let review=guest.command(agent,format!("cat /service/review/{feature}")).await?;
                    assert_eq!(response(&review)?["success"],true,"{agent}: {review}");
                    let early=guest.command(agent,"cat /service/release/user_authentication".into()).await?;
                    assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied")),"unread CI was skipped: {early}");
                }
                let initial_reviews=if own == 0 {&FEATURES[..1]} else {&FEATURES[..]};
                for target in initial_reviews {
                        if own == 2 && *target == feature {continue;}
                        let review=guest.command(agent,format!("cat /service/review/{target}")).await?;
                        assert_eq!(response(&review)?["success"],true,"{agent}: {review}");
                }
                for target in FEATURES {
                    if target == feature {
                        let merged=guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/merge_code",json!({"branch":feature}))).await?;
                        assert_eq!(response(&merged)?["success"],true,"{agent}: {merged}");
                    }
                    for kind in ["integration","release"] {
                        let value=guest.command(agent,format!("cat /service/{kind}/{target}")).await?;
                        assert_eq!(response(&value)?["success"],true,"{agent}/{kind}: {value}");
                    }
                    if own == 0 && target == feature {
                        let late=guest.command(agent,format!("echo '{}' > /service/implement_feature",json!({"feature_name":feature,"source":"late edit after positive staging"}))).await?;
                        assert!(late["stdout"].as_str().is_some_and(|text|text.contains("Permission denied")),"late editing was not rejected: {late}");
                        for peer in &FEATURES[1..] {
                            let review=guest.command(agent,format!("cat /service/review/{peer}")).await?;
                            assert_eq!(response(&review)?["success"],true,"{agent}: {review}");
                        }
                    }
                }
                assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
        }
        async {
            for target in FEATURES {
                let candidate=guest.command("TEST_WRITER",format!("cat /service/candidate/{target}")).await?;
                assert_eq!(response(&candidate)?["success"],true,"{candidate}");
                let tested=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                assert_eq!(response(&tested)?["success"],true,"{tested}");
            }
            for target in FEATURES {
                let review=guest.command("TEST_WRITER",format!("cat /service/review/{target}")).await?;
                assert_eq!(response(&review)?["success"],true,"{review}");
            }
            for target in FEATURES { for kind in ["integration","release"] {
                let value=guest.command("TEST_WRITER",format!("cat /service/{kind}/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{kind}: {value}");
            }}
            assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            for target in FEATURES {
                let candidate=guest.command("REVIEWER",format!("cat /service/candidate/{target}")).await?;
                assert!(response(&candidate)?["tests"]["success"] == true,"{candidate}");
                let review=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                assert_eq!(response(&review)?["success"],true,"{review}");
            }
            for target in FEATURES { for kind in ["integration","release"] {
                let value=guest.command("REVIEWER",format!("cat /service/{kind}/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{kind}: {value}");
            }}
            assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            for (feature,suite) in FEATURES.into_iter().zip(SUITES) {
                let publication=guest.command("CI_RUNNER",format!("cat /service/publication/{feature}")).await?;
                assert_eq!(response(&publication)?["success"],true,"{publication}");
                let ci=guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests/{feature}; cat /service/run_tests/{feature}",json!({"suite":suite}))).await?;
                assert_eq!(response(&ci)?["success"],true,"{ci}");
                let staged=guest.command("CI_RUNNER",format!("cat /service/release/{feature}")).await?;
                assert_eq!(response(&staged)?["success"],true,"{staged}");
            }
            assert_eq!(guest.command("CI_RUNNER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            let early=guest.command("RELEASE_MANAGER",format!("echo '{}' > /service/deploy/payment_processing",json!({"service":FEATURES[1]}))).await?;
            assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied")),"deployment B skipped its required prior stage: {early}");
            for target in FEATURES {
                let ready=guest.command("RELEASE_MANAGER",format!("cat /service/release/{target}")).await?;
                assert_eq!(response(&ready)?["ci"]["success"],true,"{ready}");
                for operation in ["deploy","validate"] {
                    let value=guest.command("RELEASE_MANAGER",format!("echo '{}' > /service/{operation}/{target}; cat /service/{operation}/{target}",json!({"service":target}))).await?;
                    assert_eq!(response(&value)?["success"],true,"{operation}: {value}");
                }
            }
            assert_eq!(guest.command("RELEASE_MANAGER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {runtime.await};
    },async {hibana_runtime::host::sleep(Duration::from_secs(240)).await?;Err("seven-guest staged-integration timeout".into())}).await?;
        result
    };
    assert_eq!(result["error"], Value::Null, "{result}");
    save(&directory.join("runtime-result.json"), &result)?;
    let audit = crate::journal::Evidence::open(&directory.join("events.jsonl"))?.audit("6H")?;
    assert_eq!(audit["domain_calls"], 21);
    let mut boundary = json!({"task":"6H","conditions":1,"model_calls":0,
        "scored_benchmark":false,"audit":audit});
    if schedule == Schedule::StageEach {
        let events = result["score"]["events"]
            .as_array()
            .ok_or("simulator events")?;
        let (stages, remainder) = events.as_chunks::<7>();
        assert!(remainder.is_empty());
        assert_eq!(stages.len(), FEATURES.len());
        for (feature, chunk) in FEATURES.into_iter().zip(stages) {
            let tools = [
                "implement_feature",
                "write_tests",
                "code_review",
                "merge_code",
                "run_tests",
                "deploy_service",
                "validate_staging",
            ];
            for (event, tool) in chunk.iter().zip(tools) {
                assert_eq!(event["tool"], tool);
                assert_eq!(event["success"], true);
            }
            assert_eq!(chunk[0]["args"]["feature"], feature);
            assert_eq!(chunk[6]["args"]["service"], feature);
        }
        boundary["developer_starts"] = json!("after preceding staging validation");
    } else if let Schedule::MergeBeforeStaging(order) = schedule {
        let events = result["score"]["events"]
            .as_array()
            .ok_or("simulator events")?;
        let actual = |tool: &str, field: &str| -> Result<Vec<&str>> {
            events
                .iter()
                .filter(|event| event["tool"] == tool)
                .map(|event| {
                    if event["success"] != true {
                        return Err("a scheduled operation failed".into());
                    }
                    event["args"][field]
                        .as_str()
                        .ok_or_else(|| "missing original operation target".into())
                })
                .collect()
        };
        assert_eq!(
            actual("merge_code", "branch")?,
            order.map(|index| FEATURES[index])
        );
        assert_eq!(
            actual("run_tests", "suite")?,
            order.map(|index| SUITES[index])
        );
        assert_eq!(actual("deploy_service", "service")?, FEATURES);
        assert_eq!(actual("validate_staging", "service")?, FEATURES);
        let last_ci = events
            .iter()
            .rposition(|e| e["tool"] == "run_tests")
            .ok_or("missing CI")?;
        let first_deploy = events
            .iter()
            .position(|e| e["tool"] == "deploy_service")
            .ok_or("missing deployment")?;
        assert!(last_ci < first_deploy);
        boundary["merge_order"] = json!(actual("merge_code", "branch")?);
        boundary["ci_order"] = json!(actual("run_tests", "suite")?);
        boundary["deployment_order"] = json!(actual("deploy_service", "service")?);
    } else {
        boundary["late_edit_rejected"] = json!(true);
        boundary["premature_merge_rejected_then_retried"] = json!(true);
        boundary["deployment_b_before_validated_a_rejected"] = json!(true);
        boundary["unread_ci_prevents_later_stage_read"] = json!(true);
        boundary["malformed_json_close_identified_then_corrected"] = json!(true);
        boundary["rejected_probes"] = json!(5);
    }
    save(&directory.join("boundary.json"), &boundary)?;
    println!("{}", directory.display());
    Ok(())
}

// Only the test sets the submission times. Every read and write goes through wash.
async fn staged_one_at_a_time(guest: &process::Guest, schedule: Schedule) -> Result<()> {
    let read = async |agent: &str, path: String| -> Result<Value> {
        let reply = guest.command(agent, format!("cat /service/{path}")).await?;
        response(&reply).map_err(|error| format!("{agent} reading {path}: {error}").into())
    };
    let call = async |agent: &str, path: String, args: Value| -> Result<()> {
        let command = format!("echo '{args}' > /service/{path} && cat /service/{path}");
        let reply = guest.command(agent, command).await?;
        assert_eq!(
            response(&reply)?["success"],
            true,
            "{agent} {path}: {reply}"
        );
        Ok(())
    };
    let developers = ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"];
    let stage = async |feature: &str| -> Result<()> {
        assert_eq!(
            read("RELEASE_MANAGER", format!("release/{feature}")).await?["ci"]["success"],
            true
        );
        for operation in ["deploy", "validate"] {
            call(
                "RELEASE_MANAGER",
                format!("{operation}/{feature}"),
                json!({"service":feature}),
            )
            .await?;
        }
        for agent in developers
            .into_iter()
            .chain(["TEST_WRITER", "REVIEWER", "CI_RUNNER"])
        {
            assert_eq!(
                read(agent, format!("release/{feature}")).await?["success"],
                true
            );
        }
        Ok(())
    };
    let prepare = async |index: usize| -> Result<()> {
        let (feature, developer) = (FEATURES[index], developers[index]);
        call(
            developer,
            "implement_feature".into(),
            json!({"feature_name":feature}),
        )
        .await?;
        for peer in developers.into_iter().filter(|peer| *peer != developer) {
            assert_eq!(
                read(peer, format!("interface/{feature}")).await?["success"],
                true
            );
        }
        assert_eq!(
            read("TEST_WRITER", format!("candidate/{feature}")).await?["success"],
            true
        );
        call(
            "TEST_WRITER",
            format!("write_tests/{feature}"),
            json!({"target":feature}),
        )
        .await?;
        assert_eq!(
            read("REVIEWER", format!("candidate/{feature}")).await?["tests"]["success"],
            true
        );
        call(
            "REVIEWER",
            format!("code_review/{feature}"),
            json!({"pr_id":feature}),
        )
        .await?;
        for agent in developers.into_iter().chain(["TEST_WRITER"]) {
            assert_eq!(
                read(agent, format!("review/{feature}")).await?["success"],
                true
            );
        }
        Ok(())
    };
    let integrate = async |index: usize| -> Result<()> {
        let (feature, developer, suite) = (FEATURES[index], developers[index], SUITES[index]);
        call(developer, "merge_code".into(), json!({"branch":feature})).await?;
        if let Schedule::MergeBeforeStaging(order) = schedule
            && index == order[0]
        {
            let unavailable = guest
                .command(
                    "CI_RUNNER",
                    format!(
                        "dd if=/service/publication/{} iflag=nonblock",
                        FEATURES[order[1]]
                    ),
                )
                .await?;
            let output = unavailable["stdout"].as_str().ok_or("CI probe output")?;
            assert!(
                output.contains("Resource temporarily unavailable")
                    || output.contains("Permission denied"),
                "{unavailable}"
            );
        }
        let publication = read("CI_RUNNER", format!("publication/{feature}")).await?;
        assert_eq!(publication["success"], true);
        assert_eq!(publication["data"]["feature"], feature);
        call(
            "CI_RUNNER",
            format!("run_tests/{feature}"),
            json!({"suite":suite}),
        )
        .await?;
        for agent in developers.into_iter().chain(["TEST_WRITER", "REVIEWER"]) {
            assert_eq!(
                read(agent, format!("integration/{feature}")).await?["success"],
                true
            );
        }
        Ok(())
    };
    if let Schedule::MergeBeforeStaging(order) = schedule {
        for index in 0..FEATURES.len() {
            prepare(index).await?;
        }
        for index in order {
            integrate(index).await?;
        }
        for feature in FEATURES {
            stage(feature).await?;
        }
    } else {
        for (index, feature) in FEATURES.into_iter().enumerate() {
            prepare(index).await?;
            integrate(index).await?;
            stage(feature).await?;
        }
    }
    for agent in
        developers
            .into_iter()
            .chain(["TEST_WRITER", "REVIEWER", "CI_RUNNER", "RELEASE_MANAGER"])
    {
        assert_eq!(
            guest.command(agent, "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 6H runtime, source receipt, wash and neutral simulator"]
async fn seven_actual_wasm_guests_repair_review_ci_and_staging_failures() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("staged-integration evidence root")?,
    )
    .join(format!(
        "staged-integration-recovery-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_STAGED_INTEGRATION_WASI_BINARY").ok_or("6H binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_STAGED_INTEGRATION_BUILD_RECEIPT").ok_or("6H build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    crate::projection::archive(&binary, &directory, "6H")?;
    save(
        &directory.join("case.json"),
        &json!({
            "task":"6H","depth":2,"model_calls":0,"scored_benchmark":false,
            "build":receipt,"simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,
            "delay_multiplier":0.01,"schedule":"A recovers while B/C initial material is retained",
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/staged_integration_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/staged_integration_audit.rs"))?
        }),
    )?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let (simulator, _) = oracle::Simulator::new("6H", 2, 0.01, trace.clone(), &[])?;
    let (guest, runtime) = process::guest(&binary, module, &["6H"], &directory, trace, simulator)?;
    let (release_probes,_,(),(),(),(),result) = hibana_runtime::race(async {
        let guest = &guest;
        async {
            for peer in &FEATURES[1..] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/interface/{peer}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            let mut release_probes = Vec::new();
            loop {
                let value = guest.command("DEVELOPER_A",
                    "echo '{\"feature_name\":\"user_authentication\"}' > /service/implement_feature; cat /service/implement_feature".into()).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let review = guest.command("DEVELOPER_A","cat /service/review/user_authentication".into()).await?;
                if response(&review)?["success"] != true {continue;}
                // The approval arrived from REPO on the results FIFO. An older
                // release cannot follow it, and the next release needs A's merge.
                let premature = guest.command("DEVELOPER_A",
                    "dd if=/service/release/user_authentication iflag=nonblock".into()).await?;
                let output = premature["stdout"].as_str().ok_or("missing nonblocking release reply")?;
                if response(&premature).is_ok() {
                    return Err(format!("unexpected release before merge: {premature}").into());
                }
                release_probes.push(output.to_owned());
                let merged = guest.command("DEVELOPER_A",
                    "echo '{\"branch\":\"user_authentication\"}' > /service/merge_code; cat /service/merge_code".into()).await?;
                assert_eq!(response(&merged)?["success"],true,"{merged}");
                let ci = guest.command("DEVELOPER_A","cat /service/integration/user_authentication".into()).await?;
                if response(&ci)?["success"] != true {continue;}
                let stage = guest.command("DEVELOPER_A","cat /service/release/user_authentication".into()).await?;
                if response(&stage)?["success"] == true {break;}
            }
            for feature in &FEATURES[1..] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/review/{feature}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            for feature in &FEATURES[1..] {for kind in ["integration","release"] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/{kind}/{feature}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }}
            assert_eq!(guest.command("DEVELOPER_A","exit 0".into()).await?["exit_status"],0);
            Result::Ok(release_probes)
        };
        for (agent,feature,peer) in [
            ("DEVELOPER_B",FEATURES[1],FEATURES[2]),
            ("DEVELOPER_C",FEATURES[2],FEATURES[1]),
        ] {
            async move {
                let value = guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let value = guest.command(agent,format!("cat /service/interface/{peer}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                loop {
                    let value = guest.command(agent,"cat /service/interface/user_authentication".into()).await?;
                    assert_eq!(response(&value)?["success"],true,"{value}");
                    let review = guest.command(agent,"cat /service/review/user_authentication".into()).await?;
                    if response(&review)?["success"] != true {continue;}
                    let ci = guest.command(agent,"cat /service/integration/user_authentication".into()).await?;
                    if response(&ci)?["success"] != true {continue;}
                    let stage = guest.command(agent,"cat /service/release/user_authentication".into()).await?;
                    if response(&stage)?["success"] == true {break;}
                }
                for target in &FEATURES[1..] {
                    let value = guest.command(agent,format!("cat /service/review/{target}")).await?;
                    assert_eq!(response(&value)?["success"],true,"{value}");
                }
                for target in &FEATURES[1..] {
                    if *target == feature {
                        let value = guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/merge_code",json!({"branch":feature}))).await?;
                        assert_eq!(response(&value)?["success"],true,"{value}");
                    }
                    for kind in ["integration","release"] {
                        let value = guest.command(agent,format!("cat /service/{kind}/{target}")).await?;
                        assert_eq!(response(&value)?["success"],true,"{value}");
                    }
                }
                assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
        }
        async {
            loop {
                let value = guest.command("TEST_WRITER","cat /service/candidate/user_authentication".into()).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let value = guest.command("TEST_WRITER",
                    "echo '{\"target\":\"user_authentication\"}' > /service/write_tests/user_authentication; cat /service/write_tests/user_authentication".into()).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let review = guest.command("TEST_WRITER","cat /service/review/user_authentication".into()).await?;
                if response(&review)?["success"] != true {continue;}
                let ci = guest.command("TEST_WRITER","cat /service/integration/user_authentication".into()).await?;
                if response(&ci)?["success"] != true {continue;}
                let stage = guest.command("TEST_WRITER","cat /service/release/user_authentication".into()).await?;
                if response(&stage)?["success"] == true {break;}
            }
            for target in &FEATURES[1..] {
                let value = guest.command("TEST_WRITER",format!("cat /service/candidate/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let value = guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            for target in &FEATURES[1..] {
                let value = guest.command("TEST_WRITER",format!("cat /service/review/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            for target in &FEATURES[1..] {for kind in ["integration","release"] {
                let value = guest.command("TEST_WRITER",format!("cat /service/{kind}/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }}
            assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            loop {
                let value = guest.command("REVIEWER","cat /service/candidate/user_authentication".into()).await?;
                assert_eq!(response(&value)?["tests"]["success"],true,"{value}");
                let review = guest.command("REVIEWER",
                    "echo '{\"pr_id\":\"user_authentication\"}' > /service/code_review/user_authentication; cat /service/code_review/user_authentication".into()).await?;
                if response(&review)?["success"] != true {continue;}
                let ci = guest.command("REVIEWER","cat /service/integration/user_authentication".into()).await?;
                if response(&ci)?["success"] != true {continue;}
                let stage = guest.command("REVIEWER","cat /service/release/user_authentication".into()).await?;
                if response(&stage)?["success"] == true {break;}
            }
            for target in &FEATURES[1..] {
                let value = guest.command("REVIEWER",format!("cat /service/candidate/{target}")).await?;
                assert_eq!(response(&value)?["tests"]["success"],true,"{value}");
                let value = guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            for target in &FEATURES[1..] {for kind in ["integration","release"] {
                let value = guest.command("REVIEWER",format!("cat /service/{kind}/{target}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }}
            assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            for (target,suite) in FEATURES.into_iter().zip(SUITES) {
                loop {
                    let value = guest.command("CI_RUNNER",format!("cat /service/publication/{target}")).await?;
                    assert_eq!(response(&value)?["success"],true,"{value}");
                    let ci = guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests/{target}; cat /service/run_tests/{target}",json!({"suite":suite}))).await?;
                    if response(&ci)?["success"] != true {continue;}
                    let stage = guest.command("CI_RUNNER",format!("cat /service/release/{target}")).await?;
                    if response(&stage)?["success"] == true {break;}
                }
            }
            assert_eq!(guest.command("CI_RUNNER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {
            for target in FEATURES {
                loop {
                    let ready = guest.command("RELEASE_MANAGER",format!("cat /service/release/{target}")).await?;
                    assert_eq!(response(&ready)?["ci"]["success"],true,"{ready}");
                    let deployed = guest.command("RELEASE_MANAGER",format!("echo '{}' > /service/deploy/{target}; cat /service/deploy/{target}",json!({"service":target}))).await?;
                    assert_eq!(response(&deployed)?["success"],true,"{deployed}");
                    let stage = guest.command("RELEASE_MANAGER",format!("echo '{}' > /service/validate/{target}; cat /service/validate/{target}",json!({"service":target}))).await?;
                    if response(&stage)?["success"] == true {break;}
                    let rollback = guest.command("RELEASE_MANAGER",format!("echo '{}' > /service/rollback/{target}; cat /service/rollback/{target}",json!({"service":target}))).await?;
                    assert_eq!(response(&rollback)?["success"],true,"{rollback}");
                }
            }
            assert_eq!(guest.command("RELEASE_MANAGER","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
        };
        async {runtime.await};
    },async {
        hibana_runtime::host::sleep(Duration::from_secs(240)).await?;
        Err("seven-guest staged-integration recovery timeout".into())
    }).await?;
    assert_eq!(result["error"], Value::Null, "{result}");
    save(&directory.join("runtime-result.json"), &result)?;
    let audit = crate::journal::Evidence::open(&directory.join("events.jsonl"))?.audit("6H")?;
    assert_eq!(audit["domain_calls"], 53, "{audit}");
    save(&directory.join("effect-audit.json"), &audit)?;
    save(
        &directory.join("premerge-release-probes.json"),
        &json!(release_probes),
    )?;
    assert_eq!(
        release_probes.len(),
        5,
        "each approved candidate tested before merging"
    );
    for (index, probe) in release_probes.iter().enumerate() {
        assert!(
            probe.contains("Permission denied (os error 2)"),
            "premature release read {}: {probe}",
            index + 1
        );
    }
    save(
        &directory.join("boundary.json"),
        &json!({
            "task":"6H","conditions":1,"model_calls":0,"scored_benchmark":false,
            "review_rejections":2,"ci_failures":2,"staging_failures":2,"actual_rollbacks":2,
            "audit":audit
        }),
    )?;
    println!("{}", directory.display());
    Ok(())
}
