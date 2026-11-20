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

// Drive one operation at a time so an idle peer's attempted read cannot
// mask what the chosen developer's merge file actually admitted.
async fn drive_requested_order(
    guest: &process::Guest,
    (review_b_first, tests_b_first, merge_b_first): (bool, bool, bool),
    separate_preparations: bool,
) -> Result<()> {
    let agents = ["DEVELOPER_A", "DEVELOPER_B"];
    let features = ["user_authentication", "payment_processing"];
    let suites = [
        "full_test_suite_after_auth",
        "full_test_suite_after_payment",
    ];
    let order = |b_first: bool| if b_first { [1usize, 0] } else { [0, 1] };
    let read = async |agent: &str, path: String| -> Result<()> {
        let reply = guest.command(agent, format!("cat /service/{path}")).await?;
        let value = response(&reply).map_err(|error| format!("{agent} reading {path}: {error}"))?;
        assert_eq!(value["success"], true, "{agent} {path}: {value}");
        Ok(())
    };
    let call = async |agent: &str, path: String, args: Value| -> Result<()> {
        let reply = guest
            .command(
                agent,
                format!("echo '{args}' > /service/{path}; cat /service/{path}"),
            )
            .await?;
        let value =
            response(&reply).map_err(|error| format!("{agent} submitting {path}: {error}"))?;
        assert_eq!(value["success"], true, "{agent} {path}: {value}");
        Ok(())
    };
    let batches = if separate_preparations {
        order(merge_b_first).map(|i| vec![i]).to_vec()
    } else {
        vec![vec![0, 1]]
    };
    for batch in batches {
        for &i in &batch {
            call(
                agents[i],
                "implement_feature".into(),
                json!({"feature_name":features[i]}),
            )
            .await?;
        }
        for &i in &batch {
            read(agents[1 - i], "interface_change".into()).await?;
        }
        for i in order(tests_b_first)
            .into_iter()
            .filter(|i| batch.contains(i))
        {
            let f = features[i];
            read("TEST_WRITER", format!("code_ready/{f}")).await?;
            call(
                "TEST_WRITER",
                format!("write_tests/{f}"),
                json!({"target":f}),
            )
            .await?;
        }
        for i in order(review_b_first)
            .into_iter()
            .filter(|i| batch.contains(i))
        {
            let f = features[i];
            read("REVIEWER", format!("code_ready/{f}")).await?;
            call("REVIEWER", format!("code_review/{f}"), json!({"pr_id":f})).await?;
            read(agents[i], "review".into()).await?;
            read(agents[1 - i], "peer_review".into()).await?;
            read("TEST_WRITER", format!("review/{f}")).await?;
        }
        for i in order(merge_b_first)
            .into_iter()
            .filter(|i| batch.contains(i))
        {
            let f = features[i];
            read(agents[i], "merge_ready".into()).await?;
            call(agents[i], "merge_code".into(), json!({"branch":f})).await?;
            if separate_preparations && batch.len() == 1 && i == order(merge_b_first)[0] {
                let other = features[1 - i];
                let unavailable = guest
                    .command(
                        "CI_RUNNER",
                        format!("dd if=/service/publication/{other} iflag=nonblock"),
                    )
                    .await?;
                let output = unavailable["stdout"].as_str().ok_or("CI probe output")?;
                assert!(
                    output.contains("Resource temporarily unavailable")
                        || output.contains("Permission denied"),
                    "{unavailable}"
                );
            }
            let published = guest
                .command("CI_RUNNER", format!("cat /service/publication/{f}"))
                .await?;
            assert_eq!(response(&published)?["data"]["feature"], f, "{published}");
            call(
                "CI_RUNNER",
                format!("run_tests/{f}"),
                json!({"suite":suites[i]}),
            )
            .await?;
            for agent in agents {
                read(agent, "integration".into()).await?;
            }
            read("TEST_WRITER", "integration".into()).await?;
            read("REVIEWER", format!("integration/{f}")).await?;
        }
    }
    for agent in [
        "DEVELOPER_A",
        "DEVELOPER_B",
        "CI_RUNNER",
        "TEST_WRITER",
        "REVIEWER",
    ] {
        assert_eq!(
            guest.command(agent, "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(())
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
    let ordinary = crate::qualification::FEATURE_DELIVERY_INITIAL_ORDERS;
    let count = ordinary.len() + 2;
    let selected = match std::env::var("CHOREOFIX_FEATURE_DELIVERY_CASE") {
        Ok(value) => {
            let index = value.parse::<usize>()?;
            if index >= count {
                return Err("feature-delivery condition is outside the declared matrix".into());
            }
            index..index + 1
        }
        Err(std::env::VarError::NotPresent) => 0..count,
        Err(error) => return Err(error.into()),
    };
    let expected_conditions = selected.len();
    let full_matrix = expected_conditions == count;
    save(
        &directory.join("input-domain.json"),
        &json!({"declared_conditions":count,"selected_conditions":selected.clone().collect::<Vec<_>>(),
            "full_matrix":full_matrix,"model_calls":0}),
    )?;
    for index in selected {
        let separate_preparations = index >= ordinary.len();
        let (review_b_first, tests_b_first, merge_b_first) =
            if let Some(&order) = ordinary.get(index) {
                order
            } else {
                let b_first = index == ordinary.len() + 1;
                (b_first, b_first, b_first)
            };
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "6M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"6M","depth":0,"review_b_first":review_b_first,
            "separate_preparations":separate_preparations,"tests_b_first":tests_b_first,"merge_b_first":merge_b_first,"model_calls":0,"scored_benchmark":false,"build":receipt,
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
        let result = if separate_preparations || review_b_first != merge_b_first {
            hibana_runtime::race(
                async {
                    async {
                        drive_requested_order(
                            &guest,
                            (review_b_first, tests_b_first, merge_b_first),
                            separate_preparations,
                        )
                        .await?;
                    };
                    runtime.await
                },
                hibana_runtime::host::deadline(Duration::from_secs(45)),
            )
            .await?
        } else {
            let (_,(),(),(),result) = hibana_runtime::race(async {
            async {
                let guest = &guest;
                for (agent, feature) in [("DEVELOPER_A","user_authentication"),("DEVELOPER_B","payment_processing")] {
                    async move {
                        let implemented = guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                        assert_eq!(response(&implemented)?["success"],true,"{implemented}");
                        let changed = guest.command(agent,"cat /service/interface_change".into()).await?;
                        assert_eq!(response(&changed)?["success"],true,"{changed}");
                        let first = (feature == "payment_processing") == merge_b_first;
                        let reviewed_first = (feature == "payment_processing") == review_b_first;
                        let feedbacks = if reviewed_first {["review","peer_review"]} else {["peer_review","review"]};
                        for file in feedbacks {
                            let feedback=guest.command(agent,format!("cat /service/{file}")).await?;
                            assert_eq!(response(&feedback)?["success"],true,"{feedback}");
                        }
                        if !first {
                            let previous=guest.command(agent,"cat /service/integration".into()).await?;
                            assert_eq!(response(&previous)?["success"],true,"{previous}");
                        }
                        let ready=guest.command(agent,"cat /service/merge_ready".into()).await?;
                        assert_eq!(response(&ready)?["success"],true,"{ready}");
                        let merged=guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/merge_code",json!({"branch":feature}))).await?;
                        assert_eq!(response(&merged)?["success"],true,"{merged}");
                        let integrated=guest.command(agent,"cat /service/integration".into()).await?;
                        assert_eq!(response(&integrated)?["success"],true,"{integrated}");
                        if first {
                            let other=guest.command(agent,"cat /service/integration".into()).await?;
                            assert_eq!(response(&other)?["success"],true,"{other}");
                        }
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                        Result::Ok(())
                    };
                }
            };
            async {
                let targets=if tests_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in targets {
                    let ready=guest.command("TEST_WRITER",format!("cat /service/code_ready/{target}")).await?;
                    assert_eq!(response(&ready)?["success"],true,"{ready}");
                    let reply=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                    assert_eq!(response(&reply)?["success"],true,"{reply}");
                }
                for target in ["user_authentication","payment_processing"] {
                    let review=guest.command("TEST_WRITER",format!("cat /service/review/{target}")).await?;
                    assert_eq!(response(&review)?["success"],true,"{review}");
                }
                for _ in 0..2 {
                    let ci=guest.command("TEST_WRITER","cat /service/integration".into()).await?;
                    assert_eq!(response(&ci)?["success"],true,"{ci}");
                }
                assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let targets=if review_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in targets {
                    let ready=guest.command("REVIEWER",format!("cat /service/code_ready/{target}")).await?;
                    assert_eq!(response(&ready)?["success"],true,"{ready}");
                    let reply=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                    assert_eq!(response(&reply)?["success"],true,"{reply}");
                }
                for target in if merge_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]} {
                    let ci=guest.command("REVIEWER",format!("cat /service/integration/{target}")).await?;
                    assert_eq!(response(&ci)?["success"],true,"{ci}");
                }
                assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let suites=if merge_b_first {[("payment_processing","full_test_suite_after_payment"),("user_authentication","full_test_suite_after_auth")]} else {[("user_authentication","full_test_suite_after_auth"),("payment_processing","full_test_suite_after_payment")]};
                for (target,suite) in suites {
                    let publication=guest.command("CI_RUNNER",format!("cat /service/publication/{target}")).await?;
                    assert_eq!(response(&publication)?["success"],true,"{publication}");
                    let reply=guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests/{target}; cat /service/run_tests/{target}",json!({"suite":suite}))).await?;
                    assert_eq!(response(&reply)?["success"],true,"{reply}");
                }
                assert_eq!(guest.command("CI_RUNNER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
            result
        };
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
        let events = result["score"]["events"]
            .as_array()
            .ok_or("scored 6M events")?;
        for (tool, key, b_first) in [
            ("code_review", "target", review_b_first),
            ("write_tests", "component", tests_b_first),
            ("merge_code", "branch", merge_b_first),
        ] {
            let observed: Vec<_> = events
                .iter()
                .filter(|event| event["tool"] == tool)
                .map(|event| event["args"][key].as_str().ok_or("scored target"))
                .collect::<std::result::Result<_, _>>()?;
            let expected = if b_first {
                ["payment_processing", "user_authentication"]
            } else {
                ["user_authentication", "payment_processing"]
            };
            assert_eq!(observed, expected, "observed {tool} effect order");
        }
        if separate_preparations {
            let tools = events
                .iter()
                .map(|e| e["tool"].as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                tools,
                [
                    "implement_feature",
                    "write_tests",
                    "code_review",
                    "merge_code",
                    "run_tests"
                ]
                .map(Some)
                .repeat(2)
            );
        }
        let mutations = if index == 0 {
            crate::feature_delivery_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutations"),
            )?
        } else {
            Value::Null
        };
        let audit = if index == 0 {
            mutations["audit"].clone()
        } else {
            crate::feature_delivery_audit::audit(&output.join("events.jsonl"))?
        };
        save(&output.join("audit.json"), &audit)?;
        outcomes.push(json!({"case":index,"audit":audit,"record_mutations":mutations}));
        save(
            &directory.join("progress.json"),
            &json!({"completed_conditions":outcomes.len(),"expected_conditions":expected_conditions,
                "declared_conditions":count,"full_matrix":full_matrix,"model_calls":0,"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("outcomes.json"),
        &json!({"status":"passed","task":"6M","conditions":outcomes.len(),
            "declared_conditions":count,"full_matrix":full_matrix,
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
        let (_,(),(),(),result)=hibana_runtime::race(async {
            async {
                let guest=&guest;
                for (agent,feature) in [("DEVELOPER_A","user_authentication"),("DEVELOPER_B","payment_processing")] {
                    async move {
                        for command in [
                            format!("echo '{}' > /service/merge_code",json!({"branch":feature})),
                            r#"echo '{"feature_name":"unassigned_feature"}' > /service/implement_feature"#.into(),
                            r#"echo '{"success":true}' > /service/review"#.into(),
                        ] {
                            let refused=guest.command(agent,command).await?;
                            assert!(refused["stdout"].as_str().ok_or("refused guest output")?.contains("wash:"),"{refused}");
                        }
                        let implemented=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                        assert_eq!(response(&implemented)?["success"],true,"{implemented}");
                        let first=(feature=="payment_processing")==review_b_first;
                        let peer=guest.command(agent,"cat /service/interface_change".into()).await?;
                        assert_eq!(response(&peer)?["success"],true,"{peer}");
                        if !first {
                            loop {
                                let peer=guest.command(agent,"cat /service/peer_review".into()).await?;
                                if response(&peer)?["success"].as_bool().ok_or("actual peer review")? { break; }
                                let updated=guest.command(agent,"cat /service/interface_change".into()).await?;
                                assert_eq!(response(&updated)?["success"],true,"{updated}");
                            }
                        }
                        loop {
                            let review=guest.command(agent,"cat /service/review".into()).await?;
                            if response(&review)?["success"].as_bool().ok_or("actual own review")? {break;}
                            let revised=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                            assert_eq!(response(&revised)?["success"],true,"{revised}");
                        }
                        if first {
                            loop {
                                let peer=guest.command(agent,"cat /service/peer_review".into()).await?;
                                if response(&peer)?["success"].as_bool().ok_or("actual peer review")? {break;}
                                let updated=guest.command(agent,"cat /service/interface_change".into()).await?;
                                assert_eq!(response(&updated)?["success"],true,"{updated}");
                            }
                        } else {
                            loop {
                                let previous=guest.command(agent,"cat /service/integration".into()).await?;
                                if response(&previous)?["success"].as_bool().ok_or("actual prior CI")? {break;}
                                loop {
                                    let updated=guest.command(agent,"cat /service/interface_change".into()).await?;
                                    assert_eq!(response(&updated)?["success"],true,"{updated}");
                                    let peer=guest.command(agent,"cat /service/peer_review".into()).await?;
                                    if response(&peer)?["success"].as_bool().ok_or("actual peer repair review")? {break;}
                                }
                            }
                        }
                        loop {
                            let ready=guest.command(agent,"cat /service/merge_ready".into()).await?;
                            assert_eq!(response(&ready)?["success"],true,"{ready}");
                            let merged=guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/merge_code",json!({"branch":feature}))).await?;
                            assert_eq!(response(&merged)?["success"],true,"{merged}");
                            let integrated=guest.command(agent,"cat /service/integration".into()).await?;
                            if response(&integrated)?["success"].as_bool().ok_or("actual own CI")? {break;}
                            let revised=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                            assert_eq!(response(&revised)?["success"],true,"{revised}");
                            let early=guest.command(agent,"cat /service/merge_ready".into()).await?;
                            assert!(early["stdout"].as_str().ok_or("early repair merge-ready output")?.contains("Permission denied"),"{early}");
                            let command=if failures.is_empty() {
                                "cat /service/peer_review; echo peer_status:$?; cat /service/interface_change; echo interface_status:$?"
                            } else {
                                "cat /service/peer_review; echo peer_status:$?"
                            };
                            let peer=guest.command(agent,command.into()).await?;
                            let output=peer["stdout"].as_str().ok_or("retired initial-review output")?;
                            assert!(output.contains("peer_status:2")
                                && output.contains("Permission denied"),"{peer}");
                            if failures.is_empty() {
                                assert!(output.contains("interface_status:2"),"{peer}");
                            }
                            loop {
                                let review=guest.command(agent,"cat /service/review".into()).await?;
                                if response(&review)?["success"].as_bool().ok_or("actual repair review")? {break;}
                                let revised=guest.command(agent,format!("echo '{}' > /service/implement_feature; cat /service/implement_feature",json!({"feature_name":feature}))).await?;
                                assert_eq!(response(&revised)?["success"],true,"{revised}");
                            }
                        }
                        if first {
                            let other=guest.command(agent,"cat /service/integration".into()).await?;
                            assert_eq!(response(&other)?["success"],true,"{other}");
                        }
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                        Result::Ok(())
                    };
                }
            };
            async {
                let refused=guest.command("TEST_WRITER",r#"echo '{"target":"payment_processing"}' > /service/write_tests/user_authentication"#.into()).await?;
                assert!(refused["stdout"].as_str().ok_or("wrong test target output")?.contains("wash:"),"{refused}");
                let targets=if tests_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in targets {
                    let ready=guest.command("TEST_WRITER",format!("cat /service/code_ready/{target}")).await?;
                    assert_eq!(response(&ready)?["success"],true,"{ready}");
                    let tested=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                    assert_eq!(response(&tested)?["success"],true,"{tested}");
                }
                let order=if review_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                for target in order {
                    loop {
                        let review=guest.command("TEST_WRITER",format!("cat /service/review/{target}")).await?;
                        if response(&review)?["success"].as_bool().ok_or("actual test writer review input")? {break;}
                        let ready=guest.command("TEST_WRITER",format!("cat /service/code_ready/{target}")).await?;
                        assert_eq!(response(&ready)?["success"],true,"{ready}");
                        let tested=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                        assert_eq!(response(&tested)?["success"],true,"{tested}");
                    }
                }
                for target in order {
                    loop {
                        let integrated=guest.command("TEST_WRITER","cat /service/integration".into()).await?;
                        let actual=response(&integrated)?;
                        assert_eq!(actual["data"]["feature"],target,"{actual}");
                        if actual["success"].as_bool().ok_or("actual test writer CI input")? {break;}
                        loop {
                            let ready=guest.command("TEST_WRITER",format!("cat /service/code_ready/{target}")).await?;
                            assert_eq!(response(&ready)?["success"],true,"{ready}");
                            let tested=guest.command("TEST_WRITER",format!("echo '{}' > /service/write_tests/{target}; cat /service/write_tests/{target}",json!({"target":target}))).await?;
                            assert_eq!(response(&tested)?["success"],true,"{tested}");
                            let review=guest.command("TEST_WRITER",format!("cat /service/review/{target}")).await?;
                            if response(&review)?["success"].as_bool().ok_or("actual repaired review")? {break;}
                        }
                    }
                }
                assert_eq!(guest.command("TEST_WRITER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let refused=guest.command("REVIEWER",r#"echo '{"pr_id":"payment_processing"}' > /service/code_review/user_authentication"#.into()).await?;
                assert!(refused["stdout"].as_str().ok_or("wrong review target output")?.contains("wash:"),"{refused}");
                let targets=if review_b_first {["payment_processing","user_authentication"]} else {["user_authentication","payment_processing"]};
                if failures.is_empty() {
                    for target in targets {
                        let ready=guest.command("REVIEWER",format!("cat /service/code_ready/{target}")).await?;
                        assert_eq!(response(&ready)?["success"],true,"{ready}");
                        let reviewed=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                        assert_eq!(response(&reviewed)?["success"],false,"{reviewed}");
                    }
                }
                for target in targets {
                    loop {
                        let ready=guest.command("REVIEWER",format!("cat /service/code_ready/{target}")).await?;
                        assert_eq!(response(&ready)?["success"],true,"{ready}");
                        let reviewed=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                        if response(&reviewed)?["success"].as_bool().ok_or("actual review")? {break;}
                    }
                }
                for target in targets {
                    loop {
                        let ci=guest.command("REVIEWER",format!("cat /service/integration/{target}")).await?;
                        if response(&ci)?["success"].as_bool().ok_or("actual reviewer CI input")? {break;}
                        loop {
                            let ready=guest.command("REVIEWER",format!("cat /service/code_ready/{target}")).await?;
                            assert_eq!(response(&ready)?["success"],true,"{ready}");
                            let reviewed=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{target}; cat /service/code_review/{target}",json!({"pr_id":target}))).await?;
                            if response(&reviewed)?["success"].as_bool().ok_or("actual repair review")? {break;}
                        }
                    }
                }
                assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
                Result::Ok(())
            };
            async {
                let refused=guest.command("CI_RUNNER",r#"echo '{"suite":"full_test_suite_after_payment"}' > /service/run_tests/user_authentication"#.into()).await?;
                assert!(refused["stdout"].as_str().ok_or("wrong CI suite output")?.contains("wash:"),"{refused}");
                let suites=if review_b_first {[("payment_processing","full_test_suite_after_payment"),("user_authentication","full_test_suite_after_auth")]} else {[("user_authentication","full_test_suite_after_auth"),("payment_processing","full_test_suite_after_payment")]};
                let (target,suite)=suites[0];
                let refused=guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests/{target}",json!({"suite":suite}))).await?;
                assert!(refused["stdout"].as_str().ok_or("early CI output")?.contains("wash:"),"{refused}");
                for (target,suite) in suites {
                    loop {
                        let publication=guest.command("CI_RUNNER",format!("cat /service/publication/{target}")).await?;
                        assert_eq!(response(&publication)?["success"],true,"{publication}");
                        let integrated=guest.command("CI_RUNNER",format!("echo '{}' > /service/run_tests/{target}; cat /service/run_tests/{target}",json!({"suite":suite}))).await?;
                        if response(&integrated)?["success"].as_bool().ok_or("actual CI result")? {break;}
                    }
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
        let mutations = crate::feature_delivery_audit::check_mutants(
            &output.join("events.jsonl"),
            &output.join("record-mutations"),
        )?;
        let audit = &mutations["audit"];
        save(&output.join("audit.json"), audit)?;
        outcomes.push(json!({"case":index,"depth":depth,"domain_calls":audit["domain_calls"],
            "capture_records":audit["dynamic_protocol"]["captured_records"],"audit_report":format!("case{index}/audit.json"),
            "record_mutants_rejected":mutations["count"],"rejected_guest_requests":10,"rejected_recovery_reviews":failed_recovery_reviews}));
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
