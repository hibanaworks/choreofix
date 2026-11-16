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
        &json!({"task":"6H","depth":0,"model_calls":0,"scored_benchmark":false,
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
                    // The live agent read only its own approval, leaving the peer
                    // outcomes and preceding CI unread. Neither request may skip
                    // those actual incoming operations. The same guest recovers.
                    for command in [format!("echo '{}' > /service/merge_code",json!({"branch":feature})),
                                    "cat /service/release/user_authentication".into()] {
                        let early=guest.command(agent,command).await?;
                        assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied")),"unread prerequisites were skipped: {early}");
                    }
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
            for (feature,suite) in FEATURES.into_iter().zip(["full_suite_after_auth","full_suite_after_payment","full_suite_after_notification"]) {
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
    assert_eq!(result["error"], Value::Null, "{result}");
    save(&directory.join("runtime-result.json"), &result)?;
    let audit = crate::staged_integration_audit::audit(&directory.join("events.jsonl"))?;
    assert_eq!(audit["domain_calls"], 21);
    save(
        &directory.join("boundary.json"),
        &json!({"task":"6H","conditions":1,"model_calls":0,
        "late_edit_rejected":true,"premature_merge_rejected_then_retried":true,
        "own_review_does_not_replace_peer_outcomes":true,"unread_ci_prevents_later_stage_read":true,
        "malformed_json_close_identified_then_corrected":true,
        "rejected_probes":5,"scored_benchmark":false,"audit":audit}),
    )?;
    println!("{}", directory.display());
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
    let ((),_,(),(),(),(),result) = hibana_runtime::race(async {
        let guest = &guest;
        async {
            for peer in &FEATURES[1..] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/interface/{peer}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            let mut expected_probe_errors = [2, 2, 2, 2, 6].into_iter();
            loop {
                let value = guest.command("DEVELOPER_A",
                    "echo '{\"feature_name\":\"user_authentication\"}' > /service/implement_feature; cat /service/implement_feature".into()).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
                let review = guest.command("DEVELOPER_A","cat /service/review/user_authentication".into()).await?;
                if response(&review)?["success"] != true {continue;}
                // After a completed elastic visit, an older in-flight suffix
                // remains possible. Its absence is EAGAIN, not permission to
                // claim that the whole public receive contract is impossible.
                let premature = guest.command("DEVELOPER_A",
                    "dd if=/service/release/user_authentication iflag=nonblock".into()).await?;
                let errno = expected_probe_errors.next().expect("unexpected extra approval");
                assert!(premature["stdout"].as_str().is_some_and(|text| text.contains(&format!("os error {errno}"))),
                    "unexpected release-stream admission: {premature}");
                let merged = guest.command("DEVELOPER_A",
                    "echo '{\"branch\":\"user_authentication\"}' > /service/merge_code; cat /service/merge_code".into()).await?;
                assert_eq!(response(&merged)?["success"],true,"{merged}");
                let ci = guest.command("DEVELOPER_A","cat /service/integration/user_authentication".into()).await?;
                if response(&ci)?["success"] != true {continue;}
                let stage = guest.command("DEVELOPER_A","cat /service/release/user_authentication".into()).await?;
                if response(&stage)?["success"] == true {break;}
            }
            assert!(expected_probe_errors.next().is_none(), "recovery skipped a declared probe");
            for feature in &FEATURES[1..] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/review/{feature}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }
            for feature in &FEATURES[1..] {for kind in ["integration","release"] {
                let value = guest.command("DEVELOPER_A",format!("cat /service/{kind}/{feature}")).await?;
                assert_eq!(response(&value)?["success"],true,"{value}");
            }}
            assert_eq!(guest.command("DEVELOPER_A","exit 0".into()).await?["exit_status"],0);
            Result::Ok(())
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
            for (target,suite) in FEATURES.into_iter().zip([
                "full_suite_after_auth","full_suite_after_payment","full_suite_after_notification"
            ]) {
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
    let audit = crate::staged_integration_audit::audit(&directory.join("events.jsonl"))?;
    assert_eq!(audit["domain_calls"], 53, "{audit}");
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
