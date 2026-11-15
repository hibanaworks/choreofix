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
                let initial_reviews=if own == 0 {&FEATURES[..1]} else {&FEATURES[..]};
                for target in initial_reviews {
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
        "late_edit_rejected":true,"premature_merge_rejected_then_retried":true,"scored_benchmark":false,"audit":audit}),
    )?;
    println!("{}", directory.display());
    Ok(())
}
