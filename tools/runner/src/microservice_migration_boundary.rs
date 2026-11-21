//! Exercise each service through the seven migration participants’ real shells.
use crate::{
    evidence::{Result, Trace, digest, save},
    journal, oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::Duration,
};

fn response(reply: Value) -> Result<Value> {
    let stdout = reply["stdout"].as_str().ok_or("missing guest stdout")?;
    let (_, body) = stdout
        .split_once('{')
        .ok_or_else(|| format!("no simulator response in {reply}"))?;
    Ok(serde_json::Deserializer::from_str(&format!("{{{body}"))
        .into_iter::<Value>()
        .next()
        .ok_or("empty simulator result")??)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 4H application, real wash, and shared simulator"]
async fn wash_runs_independent_schema_migrations_and_all_suites() -> Result<()> {
    const A: &str = "microservice_a";
    const B: &str = "microservice_b";
    const C: &str = "microservice_c";
    let orders = [
        [A, B, C],
        [A, C, B],
        [B, A, C],
        [B, C, A],
        [C, A, B],
        [C, B, A],
    ];
    for migration in orders {
        for tests in orders {
            run_wash_order(migration, tests, None).await?;
        }
    }

    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 4H application, real wash, and shared simulator"]
async fn wash_reviews_repaired_candidates_after_test_failure() -> Result<()> {
    let [a, b, c] = ["microservice_a", "microservice_b", "microservice_c"];
    for failed in 0..3 {
        for rejected in [0, 2] {
            run_wash_order([c, b, a], [c, a, b], Some((failed, rejected))).await?;
        }
    }
    Ok(())
}

#[hibana_runtime::scope]
async fn run_wash_order(
    order: [&str; 3],
    test_order: [&str; 3],
    recovery: Option<(u32, u32)>,
) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(root)?;
    let initials = |services: [&str; 3]| {
        services
            .map(|s| s.trim_start_matches("microservice_"))
            .join("")
    };
    let name = format!("{}-tests-{}", initials(order), initials(test_order));
    let failed_test = recovery.map(|(index, _)| index);
    let rejected_reviews = recovery.map_or(0, |(_, count)| count);
    let mut suites = test_order.to_vec();
    if let Some((failed, rejected)) = recovery {
        let index = failed as usize;
        let retry_at = if rejected == 0 {
            suites.len()
        } else {
            index + 1
        };
        suites.insert(retry_at, test_order[index]);
    }
    let name = match recovery {
        Some((index, count)) => format!("{name}-fail-test-{index}-reject-{count}"),
        None => name,
    };
    let output =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("4H evidence root")?)
            .join(name);
    fs::create_dir(&output)?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_MICROSERVICE_MIGRATION_WASI_BINARY").ok_or("4H executable")?,
    )
    .canonicalize()?;
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let artifact = projection::archive(&runtime, &output, "4H")?;
    save(
        &output.join("case.json"),
        &json!({"task":"4H","scope":"tested_services_on_staging","migration_order":order,"test_order":suites,"failed_test_index":failed_test,"rejected_repair_reviews":rejected_reviews,"depth":0,"delay_multiplier":0.01,
       "model_calls":0,"scored_benchmark":false,"runtime_binary_sha256":digest(&runtime)?,"guest_wasm_sha256":digest(wasm)?,
       "compiled_projection_sha256":digest(&output.join("projection.json"))?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,
       "harness_source_sha256":digest(Path::new("tools/runner/src/microservice_migration_boundary.rs"))?}),
    )?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let mut failures: Vec<_> = failed_test
        .map(|index| ("run_tests", index))
        .into_iter()
        .collect();
    failures.extend((3..3 + rejected_reviews).map(|index| ("code_review", index)));
    let (world, _) = oracle::Simulator::new("4H", 0, 0.01, trace.clone(), failures.as_slice())?;
    let (guest, runtime_exit) = process::guest(&runtime, wasm, &[], &output, trace, world)?;
    let ((),(),(),(),result)=hibana_runtime::race(async {
        async {
            let early_result = guest.command("DEVOPS","dd if=/service/test_result/microservice_a iflag=nonblock".into()).await?;
            let text = early_result["stdout"].as_str().ok_or("premature staging candidate read")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{early_result}");
            let early = guest.command("DEVOPS", r#"echo '{"migration_name":"microservice_b_schema"}' | dd of=/service/migrate_schema/microservice_b oflag=nonblock"#.into()).await?;
            let text = early["stdout"].as_str().ok_or("early schema request output")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{early}");
            let reply=response(guest.command("ARCHITECT",r#"echo '{"title":"migration_plan","dependencies":{"microservice_a":["microservice_c"]}}' > /service/design_proposal; cat /service/design_proposal"#.into()).await?)?;
            assert_eq!(reply["data"]["dependencies"]["microservice_a"],json!(["microservice_c"]));
            let plan=response(guest.command("DEVOPS","cat /service/plan".into()).await?)?;
            assert_eq!(plan,reply);
            for service in order {
                let path=format!("/service/migrate_schema/{service}");
                if service=="microservice_b" {
                    let refused=guest.command("DEVOPS",format!(r#"echo '{{"migration_name":"microservice_a_schema"}}' > {path}"#)).await?;
                    let text=refused["stdout"].as_str().ok_or("wrong-target schema request output")?;
                    assert!(text.contains("Invalid argument") || text.contains("Permission denied"), "{refused}");
                }
                let name=format!("{service}_schema");
                let args=json!({"migration_name":name});
                guest.command("DEVOPS",format!("echo '{args}' > {path}")).await?;
                if service == order[0] {
                    let other = order[1];
                    let args = json!({"migration_name":format!("{other}_schema")});
                    let pending = guest.command("DEVOPS",format!("echo '{args}' | dd of=/service/migrate_schema/{other} oflag=nonblock")).await?;
                    let text = pending["stdout"].as_str().ok_or("concurrent migration response")?;
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{pending}");
                }
                let migrated=response(guest.command("DEVOPS",format!("cat {path}")).await?)?;
                assert_eq!(migrated["success"],true);
                assert_eq!(migrated["data"]["service"],service);
                assert_eq!(migrated["data"]["migration_name"],name);
                assert_eq!(migrated["data"]["plan"],plan["data"]["receipt"]);
                if service=="microservice_b" {
                    let duplicate=guest.command("DEVOPS",format!("echo '{args}' > {path}")).await?;
                    let text=duplicate["stdout"].as_str().ok_or("repeated schema request output")?;
                    assert!(text.contains("Permission denied"), "{duplicate}");
                }
                if order[0] == test_order[0] && service == order[0] {
                    let tested=response(guest.command("DEVOPS",format!("cat /service/test_result/{service}")).await?)?;
                    assert_eq!(tested["success"],true);
                    assert_eq!(tested["data"]["service"],service);
                    assert_eq!(tested["data"]["plan"],plan["data"]["receipt"]);
                }
            }
            for service in test_order {
                if order[0] == test_order[0] && service == order[0] { continue; }
                let tested=response(guest.command("DEVOPS",format!("cat /service/test_result/{service}")).await?)?;
                assert_eq!(tested["success"],true);
                assert_eq!(tested["data"]["service"],service);
                assert_eq!(tested["data"]["plan"],plan["data"]["receipt"]);
            }
            let args=json!({"service":"microservice_a","environment":"staging"});
            let blocked=response(guest.command("DEVOPS",format!("echo '{args}' > /service/deploy_service/microservice_a && cat /service/deploy_service/microservice_a")).await?)?;
            assert_eq!(blocked,json!({"success":false,"status":"dependency_pending","service":"microservice_c"}));
            let deploy_order = if test_order[0] == "microservice_b" {
                ["microservice_b","microservice_c","microservice_a"]
            } else { ["microservice_c","microservice_a","microservice_b"] };
            for service in deploy_order {
                let args=json!({"service":service,"environment":"staging"});
                let deployed=response(guest.command("DEVOPS",format!("echo '{args}' > /service/deploy_service/{service} && cat /service/deploy_service/{service}")).await?)?;
                assert_eq!(deployed["success"],true);
                assert_eq!(deployed["data"]["service"],service);
                assert_eq!(deployed["data"]["plan"],plan["data"]["receipt"]);
            }
            assert_eq!(guest.command("DEVOPS","exit 0".into()).await?["exit_status"],0);
            assert_eq!(guest.command("ARCHITECT","exit 0".into()).await?["exit_status"],0);
            Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
        };
        async {
            let peer=&guest;
            for (name,service) in [("DEVELOPER_A","microservice_a"),("DEVELOPER_B","microservice_b"),("DEVELOPER_C","microservice_c")] {
                async move {
                    let plan=response(peer.command(name,"cat /service/plan".into()).await?)?;
                    let args=json!({"feature_name":service});
                    let made=response(peer.command(name,format!("echo '{args}' > /service/implement_feature; cat /service/implement_feature")).await?)?;
                    assert_eq!(made["data"]["service"],service);
                    assert_eq!(made["data"]["plan"],plan["data"]["receipt"]);
                    let reviewed=response(peer.command(name,"cat /service/review_result".into()).await?)?;
                    assert_eq!(reviewed["data"]["candidate"],made["data"]["receipt"]);
                    let tested=response(peer.command(name,"cat /service/test_result".into()).await?)?;
                    assert_eq!(tested["data"]["candidate"], made["data"]["receipt"]);
                    assert_eq!(tested["data"]["review"], reviewed["data"]["receipt"]);
                    if tested["success"] == false {
                        let mut prior = made;
                        for attempt in 0..=rejected_reviews {
                            let fixed=response(peer.command(name,format!("echo '{args}' > /service/implement_feature; cat /service/implement_feature")).await?)?;
                            assert_eq!(fixed["data"]["service"], service);
                            assert_eq!(fixed["data"]["plan"], prior["data"]["plan"]);
                            assert_ne!(fixed["data"]["receipt"], prior["data"]["receipt"]);
                            let review=response(peer.command(name,"cat /service/review_result".into()).await?)?;
                            assert_eq!(review["success"], attempt == rejected_reviews);
                            assert_eq!(review["data"]["candidate"],fixed["data"]["receipt"]);
                            prior = fixed;
                        }
                        let retested=response(peer.command(name,"cat /service/test_result".into()).await?)?;
                        assert_eq!(retested["success"],true);
                        assert_eq!(retested["data"]["candidate"],prior["data"]["receipt"]);
                    }
                    assert_eq!(peer.command(name,"exit 0".into()).await?["exit_status"],0);
                };
            }
            Ok(())
        };
        async {
            for service in ["microservice_c","microservice_b","microservice_a"] {
                let made=response(guest.command("REVIEWER",format!("cat /service/candidate/{service}")).await?)?;
                let args=json!({"pr_id":service});
                let reviewed=response(guest.command("REVIEWER",format!("echo '{args}' > /service/code_review/{service}; cat /service/code_review/{service}")).await?)?;
                assert_eq!(reviewed["success"],true);
                assert_eq!(reviewed["data"]["candidate"],made["data"]["receipt"]);
            }
            for (index, service) in test_order.into_iter().enumerate() {
                let path = format!("/service/candidate/{service}");
                let mut next = response(guest.command("REVIEWER",format!("cat {path}")).await?)?;
                assert_eq!(next["data"]["service"], service);
                if failed_test != Some(index as u32) {
                    assert_eq!(next["data"]["status"], "tests passed");
                    continue;
                }
                let args = json!({"pr_id":service});
                for attempt in 0..=rejected_reviews {
                    assert_eq!(next["data"]["status"], "implemented");
                    let reviewed = response(guest.command("REVIEWER",format!("echo '{args}' > /service/code_review/{service}; cat /service/code_review/{service}")).await?)?;
                    assert_eq!(reviewed["success"],attempt == rejected_reviews);
                    assert_eq!(reviewed["data"]["candidate"],next["data"]["receipt"]);
                    if attempt < rejected_reviews {
                        let previous = next;
                        next = response(guest.command("REVIEWER",format!("cat {path}")).await?)?;
                        assert_ne!(previous["data"]["receipt"],next["data"]["receipt"]);
                    }
                }
            }
            if let Some(index) = failed_test {
                let service = test_order[index as usize];
                let passed = response(guest.command("REVIEWER",format!("cat /service/candidate/{service}")).await?)?;
                assert_eq!(passed["data"]["service"], service);
                assert_eq!(passed["data"]["status"], "tests passed");
            }
            assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
            Ok(())
        };
        async {
            for (index, service) in suites.iter().copied().enumerate() {
                let approval=response(guest.command("TESTER",format!("cat /service/candidate/{service}")).await?)?;
                if service=="microservice_c" {
                    let bad=guest.command("TESTER",r#"echo '{"suite":"microservice_b"}' > /service/run_tests/microservice_c"#.into()).await?;
                    let text=bad["stdout"].as_str().ok_or("missing failure output")?;
                    assert!(text.contains("Invalid argument")||text.contains("Permission denied"),"{bad}");
                }
                let args=json!({"suite":service});
                let tested=response(guest.command("TESTER",format!("echo '{args}' > /service/run_tests/{service}; cat /service/run_tests/{service}")).await?)?;
                assert_eq!(tested["success"],failed_test != Some(index as u32));
                assert_eq!(tested["data"]["suite"],service);
                assert_eq!(tested["data"]["review"],approval["data"]["receipt"]);
                assert_eq!(tested["data"]["candidate"],approval["data"]["candidate"]);
                assert_eq!(tested["data"]["plan"],approval["data"]["plan"]);
                if tested["success"] == false {
                    let early = guest.command("TESTER",format!("echo '{args}' | dd of=/service/run_tests/{service} oflag=nonblock")).await?;
                    let text = early["stdout"].as_str().ok_or("missing retest refusal")?;
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{early}");
                }
            }
            assert_eq!(guest.command("TESTER","exit 0".into()).await?["exit_status"],0);
            Ok(())
        };
        async { runtime_exit.await };
    },hibana_runtime::host::deadline(Duration::from_secs(120))).await?;
    save(&output.join("result.json"), &result)?;
    assert_eq!(result["runtime_exit_code"], 0);
    assert_eq!(result["backend_errors"], json!([]));
    assert_eq!(result["protocol"]["scope"], "tested_services_on_staging");
    assert_eq!(result["protocol"]["status"], "complete");
    assert_eq!(result["score"]["complete"], false);
    assert_eq!(result["score"]["violations"], json!([]));
    let effects = result["score"]["events"]
        .as_array()
        .ok_or("score lacks effects")?;
    assert_eq!(
        effects.len(),
        16 + recovery.map_or(0, |(_, rejected)| 3 + 2 * rejected as usize)
    );
    let observed_suites: Vec<_> = effects
        .iter()
        .filter(|e| e["tool"] == "run_tests")
        .map(|e| e["result"]["suite"].as_str())
        .collect();
    assert_eq!(
        observed_suites,
        suites.iter().copied().map(Some).collect::<Vec<_>>()
    );
    let actual = effects
        .iter()
        .filter(|e| e["tool"] == "migrate_schema")
        .collect::<Vec<_>>();
    assert_eq!(
        actual
            .iter()
            .map(|e| e["result"]["service"].as_str())
            .collect::<Vec<_>>(),
        order.map(Some)
    );
    for pair in actual.windows(2) {
        assert!(
            pair[0]["completed_at"]
                .as_f64()
                .ok_or("schema completion time")?
                <= pair[1]["timestamp"].as_f64().ok_or("schema start time")?
        );
    }
    let rows = BufReader::new(fs::File::open(output.join("events.jsonl"))?)
        .lines()
        .map(|s| Ok(serde_json::from_str::<Value>(&s?)?))
        .collect::<Result<Vec<_>>>()?;
    let (observed, _) = projection::load(&output.join("events.jsonl"))?;
    assert_eq!(observed, artifact);
    let audit = projection::check_dynamic(&artifact, &rows)?;
    save(&output.join("epf.json"), &audit)?;
    let history = journal::read(&rows, &artifact, &audit)?;
    let roles = &artifact["compiled"]["roles"];
    let effects = crate::microservice_migration_audit::preparation(&history, roles)?;
    save(&output.join("effect-receipts.json"), &effects)?;
    if recovery == Some((1, 2)) {
        let failed = journal::one(
            history
                .calls
                .iter()
                .filter(|(_, c)| c.tool == "run_tests" && c.result.1["success"] == false),
        )?;
        let (&retry_id, retry) = journal::one(history.calls.iter().filter(|(_, c)| {
            c.tool == "run_tests"
                && c.request["arguments"] == failed.1.request["arguments"]
                && c.emitted > failed.1.emitted
        }))?;
        let correspondence = effects["effects"].as_array().ok_or("4H effect receipts")?;
        let tested = journal::one(correspondence.iter().filter(|c| c["rpc"] == retry_id))?;
        let forward = tested["effect_request"]
            .as_u64()
            .ok_or("4H test environment request")?;
        let ready = history.frames[&forward].received;
        let mut premature = rows.clone();
        let rpc = premature.remove(retry.emitted);
        premature.insert(ready, rpc);
        let execution = projection::check_dynamic(&artifact, &premature)?;
        let err = crate::journal::check_effects(&premature, &artifact, &execution)
            .expect_err("a previous test request cannot admit a premature retry");
        assert!(err.to_string().contains("missing matching frame"), "{err}");
        drop(premature);

        let mut missing = rows.clone();
        missing.retain(|row| {
            !(row["message"]["id"] == *failed.0
                && (row["message"]["type"] == "execute"
                    || row["message"]["type"] == "simulator_result"
                    || row["direction"] == "domain_end"))
        });
        let (_, outcome) = failed.1.result;
        let score =
            journal::one(missing.iter_mut().filter(|row| {
                row["direction"] == "simulator" && row["message"]["type"] == "score"
            }))?;
        score["message"]["events"]
            .as_array_mut()
            .ok_or("score events")?
            .retain(|e| e["result"]["receipt"] != outcome["data"]["receipt"]);
        let execution = projection::check_dynamic(&artifact, &missing)?;
        let err = crate::journal::check_effects(&missing, &artifact, &execution)
            .expect_err("a dropped RPC and score must leave an unmatched request");
        assert!(
            err.to_string().contains("multiple matching frames")
                || err.to_string().contains("unmatched 4H"),
            "{err}"
        );
    }
    let file = roles["DEVOPS_FILE"].as_u64().ok_or("DevOps file role")?;
    let schemas = roles["STAGING_OWNER"]
        .as_u64()
        .ok_or("schema effect role")?;
    let staging = roles["STAGING_ENV"].as_u64().ok_or("staging resource")?;
    let architect = roles["ARCHITECT"].as_u64().ok_or("staging issuer")?;
    let vm = roles["DEVOPS_VM"].as_u64().ok_or("DevOps Wasm role")?;
    let loan = |from, to| {
        journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(from, to)
                && f.logical_label == 47
                && f.bytes.len() == 3
                && f.bytes[0] as u64 == staging
        }))
    };
    let (&issued, issued_frame) = loan(architect, schemas)?;
    let (&returned, returned_frame) = loan(schemas, architect)?;
    assert_eq!(
        history.frames[&issued].receipt["receive_commit"]["actor"],
        schemas
    );
    assert_eq!(
        history.frames[&returned].receipt["receive_commit"]["actor"],
        architect
    );
    let mut test_results = Vec::new();
    for call in history
        .calls
        .values()
        .filter(|call| call.tool == "run_tests")
    {
        let (completed, value) = call.result;
        let suffix = match value["data"]["service"].as_str() {
            Some("microservice_a") => "A",
            Some("microservice_b") => "B",
            Some("microservice_c") => "C",
            _ => return Err("test result lacks the service".into()),
        };
        let role = |name: String| {
            roles[&name]
                .as_u64()
                .ok_or("missing role in compiled projection")
        };
        let service = role(format!("SERVICE_{suffix}"))?;
        let developer = role(format!("DEVELOPER_{suffix}"))?;
        let developer_file = role(format!("DEVELOPER_{suffix}_FILE"))?;
        let tester = role(format!("TESTER_{suffix}"))?;
        let file = role("TEST_FILE".to_owned())?;
        let owner = role("ENV_OWNER".to_owned())?;
        let environment = role("TEST_ENV".to_owned())?;
        let test_vm = role("TEST_VM".to_owned())?;
        let review = journal::one(history.calls.values().filter(|previous| {
            previous.tool == "code_review"
                && previous.result.1["data"]["receipt"] == value["data"]["review"]
        }))?;
        let (reviewed_at, approved) = review.result;
        assert_eq!(approved["success"], true);
        assert_eq!(approved["data"]["candidate"], value["data"]["candidate"]);
        let (&approval, notice) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(service, file)
                && f.logical_label as u64 == service
                && reviewed_at < f.sent
                && f.value().is_ok_and(|v| v == *approved)
        }))?;
        let admitted = notice.received;
        let arguments = &call.request["arguments"];
        assert_eq!(arguments, &json!({"suite":value["data"]["service"]}));
        let (&command, request) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(file, owner)
                && f.logical_label as u64 == tester
                && admitted < f.sent
                && f.received < call.emitted
                && f.value().is_ok_and(|v| v == *arguments)
        }))?;
        let accepted = request.received;
        let (&forward, forwarded) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(environment, tester)
                && f.logical_label as u64 == tester
                && accepted < f.sent
                && f.received < call.emitted
                && f.value().is_ok_and(|v| v == *arguments)
        }))?;
        let is_environment = |f: &journal::Frame| {
            f.logical_label == 47 && f.bytes.len() == 3 && f.bytes[0] as u64 == environment
        };
        let (&checkout, grant) = journal::one(history.frames.iter().filter(|(_, f)| {
            is_environment(f)
                && f.direction(owner, tester)
                && forwarded.sent < f.sent
                && f.received < call.emitted
        }))?;
        assert!(forwarded.received < grant.received);
        assert_eq!(history.frames[&command].receipt["send_actor"], test_vm);
        assert_eq!(history.frames[&forward].receipt["send_actor"], owner);
        assert_eq!(
            history.frames[&forward].receipt["receive_commit"]["actor"],
            tester
        );
        let (&reply, shell_reply) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(owner, file)
                && f.logical_label as u64 == tester
                && completed < f.sent
                && f.value().is_ok_and(|v| v == *value)
        }))?;
        let (&returned, _) = journal::one(history.frames.iter().filter(|(_, f)| {
            is_environment(f)
                && f.direction(tester, owner)
                && completed < f.sent
                && f.received < shell_reply.sent
        }))?;
        assert_eq!(
            history.frames[&reply].receipt["receive_commit"]["actor"],
            test_vm
        );
        let positive = value["success"].as_bool().ok_or("missing test success")?;
        let (&tested, result_frame) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(service, developer)
                && f.logical_label == if positive { 93 } else { 94 }
                && f.sent > completed
                && f.value().is_ok_and(|v| v == *value)
        }))?;
        let (&reported, report) = journal::one(history.frames.iter().filter(|(_, f)| {
            f.direction(developer, developer_file)
                && f.logical_label == 95
                && f.value().is_ok_and(|v| v == *value)
        }))?;
        assert!(result_frame.received < report.sent);
        assert_eq!(history.frames[&tested].receipt["send_actor"], tester);
        assert_eq!(
            history.frames[&reported].receipt["receive_commit"]["actor"],
            role(format!("{suffix}_VM"))?
        );
        let mut entry = json!({"test":value, "approval_read":history.frames[&approval].receipt,
                "test_command":history.frames[&command].receipt, "environment_request":history.frames[&forward].receipt,
                "environment_loan":history.frames[&checkout].receipt, "environment_return":history.frames[&returned].receipt,
                "tester_read":history.frames[&reply].receipt, "outcome":history.frames[&tested].receipt, "shell_read":history.frames[&reported].receipt});
        let review_file = role("REVIEW_FILE".to_owned())?;
        let reviewer_vm = role("REVIEW_VM".to_owned())?;
        let reviewer = role(format!("REVIEWER_{suffix}"))?;
        if !positive {
            let (&loan_id, loan) = journal::one(history.frames.iter().filter(|(_, f)| {
                f.direction(tester, developer)
                    && f.logical_label == 47
                    && f.bytes.first().copied().map(u64::from) == Some(service)
            }))?;
            assert_eq!(
                history.frames[&loan_id].receipt["receive_commit"]["actor"],
                developer
            );
            let repairs: Vec<_> = history
                .calls
                .values()
                .filter(|c| {
                    c.tool == "implement_feature"
                        && c.request["arguments"]["feature_name"] == value["data"]["service"]
                        && c.emitted > completed
                })
                .collect();
            assert_eq!(repairs.len(), 1 + rejected_reviews as usize);

            let mut after = loan.received;
            let mut revisions = Vec::new();
            for repair in repairs {
                let (finished, fixed) = repair.result;
                let (&request_id, _) = journal::one(history.frames.iter().filter(|(_, f)| {
                    f.direction(developer_file, service)
                        && f.logical_label == 82
                        && after < f.received
                        && f.received < repair.emitted
                        && f.value().is_ok_and(|v| v == repair.request["arguments"])
                }))?;
                assert_eq!(
                    history.frames[&request_id].receipt["receive_commit"]["actor"],
                    developer
                );
                let (&reply_id, _) = journal::one(history.frames.iter().filter(|(_, f)| {
                    f.direction(service, developer_file)
                        && f.sent > finished
                        && f.value().is_ok_and(|v| v == *fixed)
                }))?;
                let (&submission_id, submission) =
                    journal::one(history.frames.iter().filter(|(_, f)| {
                        f.direction(service, review_file)
                            && u64::from(f.logical_label) == service
                            && f.sent > finished
                            && f.value().is_ok_and(|v| v == *fixed)
                    }))?;
                let submitted = submission.received;
                assert_eq!(
                    history.frames[&submission_id].receipt["send_actor"],
                    reviewer
                );
                assert_eq!(
                    history.frames[&submission_id].receipt["receive_commit"]["actor"],
                    reviewer_vm
                );
                assert_eq!(fixed["data"]["plan"], value["data"]["plan"]);
                assert_ne!(fixed["data"]["receipt"], value["data"]["candidate"]);
                let review = journal::one(history.calls.values().filter(|c| {
                    c.tool == "code_review"
                        && c.emitted > submitted
                        && c.request["arguments"]["pr_id"] == value["data"]["service"]
                        && c.result.1["data"]["candidate"] == fixed["data"]["receipt"]
                }))?;
                let (ended, reviewed) = review.result;
                let (&review_request, _) = journal::one(history.frames.iter().filter(|(_, f)| {
                    f.direction(review_file, service)
                        && u64::from(f.logical_label) == service
                        && f.sent > submitted
                        && f.received < review.emitted
                        && f.value().is_ok_and(|v| v == review.request["arguments"])
                }))?;
                assert_eq!(
                    history.frames[&review_request].receipt["receive_commit"]["actor"],
                    reviewer
                );
                let (&review_reply, _) = journal::one(history.frames.iter().filter(|(_, f)| {
                    f.direction(service, review_file)
                        && f.sent > ended
                        && f.value().is_ok_and(|v| v == *reviewed)
                }))?;
                assert_eq!(
                    history.frames[&review_reply].receipt["receive_commit"]["actor"],
                    reviewer_vm
                );
                let (&developer_reply, decision) =
                    journal::one(history.frames.iter().filter(|(_, f)| {
                        f.direction(developer, developer_file)
                            && f.sent > ended
                            && f.value().is_ok_and(|v| v == *reviewed)
                    }))?;
                after = decision.received;
                assert_eq!(
                    history.frames[&developer_reply].receipt["receive_commit"]["actor"],
                    role(format!("{suffix}_VM"))?
                );
                revisions.push(json!({"request":history.frames[&request_id].receipt,"build_reply":history.frames[&reply_id].receipt,
                    "reviewer_read":history.frames[&submission_id].receipt,"candidate":fixed,"review_request":history.frames[&review_request].receipt,
                    "review":reviewed,"review_reply":history.frames[&review_reply].receipt,"developer_read":history.frames[&developer_reply].receipt}));
            }
            entry["recovery"] =
                json!({"returned_service":history.frames[&loan_id].receipt,"revisions":revisions});
        } else {
            let (&id, _) = journal::one(history.frames.iter().filter(|(_, f)| {
                f.direction(reviewer, review_file)
                    && u64::from(f.logical_label) == service
                    && f.sent > completed
                    && f.value().is_ok_and(|v| v == *value)
            }))?;
            assert_eq!(history.frames[&id].receipt["send_actor"], reviewer);
            assert_eq!(
                history.frames[&id].receipt["receive_commit"]["actor"],
                reviewer_vm
            );
            entry["reviewer_read"] = history.frames[&id].receipt.clone();
            let checked = journal::one(
                effects["effects"]
                    .as_array()
                    .ok_or("effect receipts")?
                    .iter()
                    .filter(|e| {
                        e["tool"] == "run_tests" && e["receipt"] == value["data"]["receipt"]
                    }),
            )?;
            let mut staging = serde_json::Map::new();
            for field in ["test_result", "service_loan", "devops_shell_read"] {
                let frame = checked["staging_candidate"][field]
                    .as_u64()
                    .ok_or("staging handoff frame")?;
                staging.insert(field.into(), history.frames[&frame].receipt.clone());
            }
            entry["staging_candidate"] = Value::Object(staging);
        }
        test_results.push(entry);
    }
    assert_eq!(test_results.len(), suites.len());
    assert_eq!(
        test_results
            .iter()
            .filter(|entry| entry.get("recovery").is_some())
            .count(),
        usize::from(failed_test.is_some())
    );
    save(&output.join("test-receipts.json"), &json!(test_results))?;
    let mut migrations = Vec::new();
    for (id, call) in &history.calls {
        if call.tool != "migrate_schema" {
            continue;
        }
        let args = &call.request["arguments"];
        let (returned_at, result) = call.result;
        let (&request, frame) = journal::one(history.frames.iter().filter(|(_, frame)| {
            frame.direction(file, schemas)
                && frame.received < call.emitted
                && frame.value().is_ok_and(|v| v == *args)
        }))?;
        let (&response, reply) = journal::one(history.frames.iter().filter(|(_, reply)| {
            reply.direction(schemas, file)
                && reply.sent > returned_at
                && reply.logical_label == frame.logical_label
                && reply.value().is_ok_and(|v| v == *result)
        }))?;
        assert!(issued_frame.received < frame.received && reply.sent < returned_frame.sent);
        assert_eq!(history.frames[&request].receipt["send_actor"], vm);
        assert_eq!(
            history.frames[&request].receipt["receive_commit"]["actor"],
            schemas
        );
        assert_eq!(history.frames[&response].receipt["send_actor"], schemas);
        assert_eq!(
            history.frames[&response].receipt["receive_commit"]["actor"],
            vm
        );
        migrations.push(json!({"call_id":id, "arguments":args, "simulator_receipt":result["data"]["receipt"],
            "request":history.frames[&request].receipt["receive_commit"], "reply":history.frames[&response].receipt["receive_commit"]}));
    }
    assert_eq!(migrations.len(), 3);
    save(
        &output.join("schema-receipts.json"),
        &json!({"migrations":migrations,"staging_loan":history.frames[&issued].receipt, "staging_return":history.frames[&returned].receipt, "model_calls":0, "scored_benchmark":false}),
    )?;
    Ok(())
}
