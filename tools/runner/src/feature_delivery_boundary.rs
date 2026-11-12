//! Direct shell stimuli for five actual Wasm guests, not simulator permissions.
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

fn response(reply: &Value) -> Result<Value> {
    let stdout = reply["stdout"]
        .as_str()
        .ok_or("feature-delivery terminal output")?;
    let at = stdout
        .find('{')
        .ok_or_else(|| format!("missing feature-delivery response: {reply}"))?;
    serde_json::Deserializer::from_str(&stdout[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty feature-delivery response")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 6M runtime, source receipt, wash and common simulator"]
async fn actual_wasi_initial_orders_and_integrated_ci() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("feature-delivery evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "feature-delivery-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_WASI_BINARY")
            .ok_or("feature-delivery binary")?,
    )
    .canonicalize()?;
    let input_receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_BUILD_RECEIPT")
            .ok_or("feature-delivery source receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input_receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    let cases = crate::qualification::FEATURE_DELIVERY_INITIAL_ORDERS;
    let selected = match std::env::var("CHOREOFIX_FEATURE_DELIVERY_CASE") {
        Ok(value) => {
            let index = value.parse::<usize>()?;
            if index >= cases.len() {
                return Err("feature-delivery condition is outside the declared matrix".into());
            }
            index..index + 1
        }
        Err(std::env::VarError::NotPresent) => 0..cases.len(),
        Err(error) => return Err(error.into()),
    };
    let expected_conditions = selected.len();
    let full_matrix = expected_conditions == cases.len();
    save(
        &directory.join("input-domain.json"),
        &json!({"declared_conditions":cases.len(),"selected_conditions":selected.clone().collect::<Vec<_>>(),
            "full_matrix":full_matrix,"model_calls":0}),
    )?;
    for index in selected {
        let (review_b_first, tests_b_first) = cases[index];
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "6M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"6M","depth":0,"review_b_first":review_b_first,
            "tests_b_first":tests_b_first,"model_calls":0,"scored_benchmark":false,"build":receipt,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"delay_multiplier":0.01,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/feature_delivery_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/feature_delivery_audit.rs"))?}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("6M", 0, 0.01, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["6M"], &output, trace, simulator)?;
        let (_,(),(),(),result) = hibana_runtime::race(async {
            async {
                let guest = &guest;
                for (agent, feature) in [("DEVELOPER_A","user_authentication"),("DEVELOPER_B","payment_processing")] {
                    async move {
                        let implemented = guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/result",json!({"feature_name":feature}))).await?;
                        let implemented = response(&implemented)?;
                        assert_eq!(implemented["success"],true,"{implemented}");
                        assert_eq!(implemented["coordination"]["code_review"]["success"],true,"{implemented}");
                        let merged = guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/result",json!({"branch":feature}))).await?;
                        let merged = response(&merged)?;
                        assert_eq!(merged["success"],true,"{merged}");
                        assert_eq!(merged["coordination"]["ci"]["success"],true,"{merged}");
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                        Result::Ok(())
                    };
                }
            };
            async {
                let targets = if tests_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in targets {
                    let reply = guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests; cat /service/result",json!({"target":target}))).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"],true,"{actual}");
                }
                assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let targets = if review_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in targets {
                    let reply = guest.command("REVIEWER",format!("echo '{}' > /service/code_review; cat /service/result",json!({"pr_id":target}))).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"],true,"{actual}");
                }
                assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let suites = if review_b_first {["full_test_suite_after_payment","full_test_suite_after_auth"]} else {["full_test_suite_after_auth","full_test_suite_after_payment"]};
                for suite in suites {
                    let reply = guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests; cat /service/result",json!({"suite":suite}))).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"],true,"{actual}");
                }
                assert_eq!(guest.command("CI_RUNNER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"}),
            "{result}"
        );
        assert_eq!(result["runtime_exit_code"], 0, "{result}");
        assert_eq!(result["backend_errors"], json!([]), "{result}");
        assert_eq!(result["score"]["complete"], true, "{result}");
        assert_eq!(result["score"]["violations"], json!([]), "{result}");
        assert_eq!(
            result["score"]["events"]
                .as_array()
                .ok_or("actual development effects")?
                .len(),
            10
        );
        let audit = crate::feature_delivery_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        let mutations = if index == 0 {
            crate::feature_delivery_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutations"),
            )?
        } else {
            Value::Null
        };
        outcomes.push(json!({"case":index,"audit":audit,"record_mutations":mutations}));
        save(
            &directory.join("progress.json"),
            &json!({"completed_conditions":outcomes.len(),"expected_conditions":expected_conditions,
                "declared_conditions":cases.len(),"full_matrix":full_matrix,"model_calls":0,"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("outcomes.json"),
        &json!({"status":"passed","task":"6M","conditions":outcomes.len(),
            "declared_conditions":cases.len(),"full_matrix":full_matrix,
            "model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 6M runtime, source receipt, wash and common simulator"]
async fn actual_wasi_review_rejection_and_ci_repair() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("feature-delivery evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "feature-delivery-reconsideration-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_WASI_BINARY")
            .ok_or("feature-delivery binary")?,
    )
    .canonicalize()?;
    let input_receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_BUILD_RECEIPT")
            .ok_or("feature-delivery source receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input_receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    let cases = crate::qualification::FEATURE_DELIVERY_RECOVERY_CASES;
    let selected = match std::env::var("CHOREOFIX_FEATURE_DELIVERY_CASE") {
        Ok(value) => {
            let index = value.parse::<usize>()?;
            if index >= cases.len() {
                return Err("feature-delivery condition is outside the declared matrix".into());
            }
            index..index + 1
        }
        Err(std::env::VarError::NotPresent) => 0..cases.len(),
        Err(error) => return Err(error.into()),
    };
    let expected_conditions = selected.len();
    let full_matrix = expected_conditions == cases.len();
    save(
        &directory.join("input-domain.json"),
        &json!({"declared_conditions":cases.len(),"selected_conditions":selected.clone().collect::<Vec<_>>(),
            "full_matrix":full_matrix,"model_calls":0}),
    )?;
    for index in selected {
        let (depth, review_b_first, tests_b_first, failures) = cases[index];
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "6M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"6M","depth":depth,"review_b_first":review_b_first,
            "tests_b_first":tests_b_first,"failure_overrides":failures,"model_calls":0,"scored_benchmark":false,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"delay_multiplier":0.01,"build":receipt,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/feature_delivery_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/feature_delivery_audit.rs"))?}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("6M", depth, 0.01, trace.clone(), failures)?;
        let (guest, runtime) = process::guest(&binary, module, &["6M"], &output, trace, simulator)?;
        let(_,(),(),(),result)=hibana_runtime::race(async{
            async{
                let guest=&guest;
                for(agent,feature)in[("DEVELOPER_A","user_authentication"),("DEVELOPER_B","payment_processing")]{
                    async move{
                        loop{
                            let reply=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/result",json!({"feature_name":feature}))).await?;
                            let actual=response(&reply)?;
                            assert_eq!(actual["success"],true,"{actual}");
                            match actual["coordination"]["code_review"]["success"].as_bool().ok_or("actual review outcome")?{
                                false=>continue,
                                true=>{
                                    let reply=guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/result",json!({"branch":feature}))).await?;
                                    let actual=response(&reply)?;
                                    assert_eq!(actual["success"],true,"{actual}");
                                    if actual["coordination"]["ci"]["success"].as_bool().ok_or("actual CI outcome")?{break;}
                                }
                            }
                        }
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                        Result::Ok(())
                    };
                }
            };
            async{
                let targets=if tests_b_first{["payment_processing","user_authentication"]}else{["user_authentication","payment_processing"]};
                let mut actual=Value::Null;
                for target in targets{
                    let reply=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests; cat /service/result",json!({"target":target}))).await?;
                    actual=response(&reply)?;
                    assert_eq!(actual["success"],true,"{actual}");
                }
                loop{
                    if !actual["coordination"]["ci"].is_null(){
                        assert_eq!(actual["coordination"]["ci"]["success"],true,"{actual}");
                        break;
                    }
                    let outcome=actual["coordination"]["outcomes"].as_array().and_then(|values|values.last()).ok_or("actual retry outcome for test writer")?;
                    assert_eq!(outcome["success"],false,"{actual}");
                    let target=outcome["data"]["target"].as_str().or_else(||outcome["data"]["feature"].as_str()).ok_or("retry target for test writer")?;
                    let reply=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests; cat /service/result",json!({"target":target}))).await?;
                    actual=response(&reply)?;
                    assert_eq!(actual["success"],true,"{actual}");
                }
                assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async{
                let targets=if review_b_first{["payment_processing","user_authentication"]}else{["user_authentication","payment_processing"]};
                let mut actual=Value::Null;
                for target in targets{
                    let reply=guest.command("REVIEWER",format!("echo '{}' > /service/code_review; cat /service/result",json!({"pr_id":target}))).await?;
                    actual=response(&reply)?;
                    assert!(actual["success"].is_boolean(),"{actual}");
                }
                loop{
                    if !actual["coordination"]["ci"].is_null(){
                        assert_eq!(actual["coordination"]["ci"]["success"],true,"{actual}");
                        break;
                    }
                    let target=if !actual["coordination"]["code_review"].is_null(){
                        actual["coordination"]["code_review"]["data"]["target"].as_str().ok_or("initial retry target")?
                    }else if actual["success"]==false{
                        actual["data"]["target"].as_str().ok_or("rejected recovery target")?
                    }else{
                        let failed=actual["coordination"]["outcomes"].as_array().and_then(|values|values.last()).ok_or("actual failed CI for reviewer")?;
                        assert_eq!(failed["success"],false,"{actual}");
                        failed["data"]["feature"].as_str().ok_or("actual failed publication target")?
                    };
                    let reply=guest.command("REVIEWER",format!("echo '{}' > /service/code_review; cat /service/result",json!({"pr_id":target}))).await?;
                    actual=response(&reply)?;
                    assert!(actual["success"].is_boolean(),"{actual}");
                }
                assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async{
                let suites=if review_b_first{["full_test_suite_after_payment","full_test_suite_after_auth"]}else{["full_test_suite_after_auth","full_test_suite_after_payment"]};
                for suite in suites{
                    loop{
                        let reply=guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests; cat /service/result",json!({"suite":suite}))).await?;
                        let actual=response(&reply)?;
                        if actual["success"].as_bool().ok_or("actual integrated CI result")?{break;}
                    }
                }
                assert_eq!(guest.command("CI_RUNNER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async{runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"}),
            "{result}"
        );
        assert_eq!(result["runtime_exit_code"], 0, "{result}");
        assert_eq!(result["backend_errors"], json!([]), "{result}");
        assert_eq!(result["score"]["complete"], true, "{result}");
        assert_eq!(result["score"]["violations"], json!([]), "{result}");
        let events = result["score"]["events"]
            .as_array()
            .ok_or("actual reconsideration effects")?;
        let first_failed_ci = events
            .iter()
            .position(|event| event["tool"] == "run_tests" && event["success"] == false)
            .ok_or("missing failed CI stimulus")?;
        let failed_recovery_reviews = events
            .iter()
            .skip(first_failed_ci + 1)
            .filter(|event| event["tool"] == "code_review" && event["success"] == false)
            .count();
        assert!(
            failed_recovery_reviews >= failures.len(),
            "required recovery rejection was not exercised: {events:?}"
        );
        let audit = crate::feature_delivery_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        let mutations = crate::feature_delivery_audit::check_mutants(
            &output.join("events.jsonl"),
            &output.join("record-mutations"),
        )?;
        outcomes.push(json!({"case":index,"depth":depth,"domain_calls":audit["domain_calls"],
            "capture_records":audit["dynamic_protocol"]["captured_records"],"audit_report":format!("case{index}/audit.json"),
            "record_mutants_rejected":mutations["mutants_rejected"],"rejected_recovery_reviews":failed_recovery_reviews}));
        save(
            &directory.join("progress.json"),
            &json!({"completed_conditions":outcomes.len(),"expected_conditions":expected_conditions,
                "full_matrix":full_matrix,"model_calls":0,"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("outcomes.json"),
        &json!({"status":"passed","task":"6M","conditions":outcomes.len(),"declared_conditions":cases.len(),
            "full_matrix":full_matrix,"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
