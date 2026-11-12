//! Actual wash/common-world inputs. These scripts do not authorize task effects.
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
const JOBS: [&str; 4] = ["lathe_job_a", "lathe_job_b", "mill_job_a", "mill_job_b"];
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"]
        .as_str()
        .ok_or("manufacturing terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing manufacturing reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty manufacturing reply")?
        .map_err(Into::into)
}
// File I/O only: test scheduling observes actual recorded data, never app progress flags.
async fn recorded(path: &Path, matches: impl Fn(&Value) -> bool) -> Result<Value> {
    loop {
        for row in crate::journal::records(path)? {
            if matches(&row) {
                return Ok(row);
            }
        }
        hibana_runtime::host::sleep(Duration::from_millis(1)).await?;
    }
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified manufacturing CLI, actual wash and common world"]
async fn all_twenty_four_orders_and_four_recovery_inputs_use_actual_wasm() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("manufacturing evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "manufacturing-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_WASI_BINARY").ok_or("manufacturing CLI")?,
    )
    .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_BUILD_RECEIPT")
            .ok_or("manufacturing build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut orders = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let order = [a, b, c, d];
                    let mut sorted = order;
                    sorted.sort_unstable();
                    if sorted == [0, 1, 2, 3] {
                        orders.push((order, false));
                    }
                }
            }
        }
    }
    assert_eq!(orders.len(), 24);
    orders.extend([
        ([0, 2, 1, 3], true),
        ([2, 0, 3, 1], true),
        ([1, 3, 0, 2], true),
        ([3, 1, 2, 0], true),
    ]);
    let mut outcomes = Vec::new();
    for (order, adversarial) in orders {
        let output = directory.join(format!("case{}", outcomes.len()));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "11E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"11E","depth":0,"job_order":order.map(|j| JOBS[j]),"adversarial_inputs":adversarial,"domain_delay_multiplier":0.01,"model_calls":0,"scored_benchmark":false,"build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,"test_barriers":"none; serial public command input"}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("11E", 0, 0.01, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["11E"], &output, trace, simulator)?;
        let ((), result)=hibana_runtime::race(async {
            async {
                for (position,job) in order.into_iter().enumerate() {
                    let agent=if job.is_multiple_of(2) {"TECHNICIAN_A"} else {"TECHNICIAN_B"};
                    let arguments=json!({"job_id":JOBS[job]});
                    if adversarial {
                        let invalid=match job {
                            0 => r#"{"job_id":"lathe_job_b"}"#,
                            1 => r#"{"job_id":0}"#,
                            2 => r#"{"job_id":"mill_job_a","owner":"TECHNICIAN_A"}"#,
                            _ => "{",
                        };
                        let bad=guest.command(agent,format!("printf '%s' '{invalid}' > /service/do_job; cat /service/result")).await?;
                        assert_eq!(response(&bad)?["status"],"invalid_arguments","{bad}");
                        if position==0 {
                            let private=guest.command(agent,"printf '%s' 'private computation' > /work/notes; cat /work/notes".into()).await?;
                            assert!(private["stdout"].as_str().ok_or("private output")?.contains("private computation"));
                        }
                    }
                    let padding=if adversarial && position==0 {" ".repeat(1200)} else {String::new()};
                    let actual=guest.command(agent,format!("printf '%s' '{arguments}{padding}' > /service/do_job; cat /service/result")).await?;
                    let actual=response(&actual)?;
                    assert_eq!(actual["tool_name"],"do_job"); assert_eq!(actual["success"],true); assert_eq!(actual["data"]["job_id"],JOBS[job]);
                    if adversarial {
                        let duplicate=guest.command(agent,format!("echo '{arguments}' > /service/do_job; cat /service/result")).await?;
                        assert_eq!(response(&duplicate)?["status"],"out_of_order","{duplicate}");
                    }
                }
                if adversarial {
                    let bad=guest.command("SUPERVISOR","echo '{\"scope\":\"all_jobs\"}' > /service/collect_reports; cat /service/result".into()).await?;
                    assert_eq!(response(&bad)?["status"],"invalid_arguments");
                }
                let report=guest.command("SUPERVISOR","echo '{}' > /service/collect_reports; cat /service/result".into()).await?;
                let report=response(&report)?; assert_eq!(report["success"],true); assert_eq!(report["data"]["jobs"].as_array().ok_or("four actual report jobs")?.len(),4);
                if adversarial {
                    let duplicate=guest.command("SUPERVISOR","echo '{}' > /service/collect_reports; cat /service/result".into()).await?;
                    assert_eq!(response(&duplicate)?["status"],"out_of_order");
                }
                for agent in ["TECHNICIAN_A","TECHNICIAN_B","SUPERVISOR"] { assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0); }
                Ok::<(), Box<dyn std::error::Error+Send+Sync>>(())
            };
            async { runtime.await };
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let checked = crate::manufacturing_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &checked)?;
        let mutants = crate::manufacturing_audit::check_mutants(
            &output.join("events.jsonl"),
            &output.join("record-mutants"),
        )?;
        outcomes.push(json!({"case":output,"job_order":order.map(|j|JOBS[j]),"adversarial_inputs":adversarial,"protocol_complete":true,"common_score_complete":true,"audit":checked,"record_mutants_rejected":mutants["checked"].as_array().ok_or("manufacturing mutants")?.len()}));
    }
    assert_eq!(outcomes.len(), 28);
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,"scored_benchmark":false,"cases":outcomes}),
    )?;
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires actual wash and normal-delay common world; controlled schedule, not a model trial"]
async fn all_eight_busy_choices_and_late_first_retries_use_the_free_machine() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("manufacturing evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "manufacturing-busy-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_WASI_BINARY").ok_or("manufacturing CLI")?,
    )
    .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_BUILD_RECEIPT").ok_or("manufacturing receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (first, waiting, early_retry) in [(0usize, 1usize), (1, 0), (2, 3), (3, 2)]
        .into_iter()
        .flat_map(|(first, waiting)| {
            [false, true].map(move |early_retry| (first, waiting, early_retry))
        })
    {
        let other = waiting ^ 2;
        let first_agent = if first.is_multiple_of(2) {
            "TECHNICIAN_A"
        } else {
            "TECHNICIAN_B"
        };
        let waiting_agent = if waiting.is_multiple_of(2) {
            "TECHNICIAN_A"
        } else {
            "TECHNICIAN_B"
        };
        let output = directory.join(format!("case{}", outcomes.len()));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "11E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"11E","depth":0,"first_job":JOBS[first],"initial_busy_job":JOBS[waiting],"switched_job":JOBS[other],"retry_before_other_job":early_retry,"domain_delay_multiplier":1.0,"model_calls":0,"scored_benchmark":false,"build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,"test_barriers":"second technician released after actual first domain_start; remaining jobs retried after the other first job's actual terminal result"}),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, _) = oracle::Simulator::new("11E", 0, 1.0, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["11E"], &output, trace, simulator)?;
        let ((), result) = hibana_runtime::race(
            async {
                async {
                    async {
                        let args = json!({"job_id":JOBS[first]});
                        let reply = guest
                            .command(
                                first_agent,
                                format!("echo '{args}' > /service/do_job; cat /service/result"),
                            )
                            .await?;
                        assert_eq!(response(&reply)?["data"]["job_id"], JOBS[first]);
                        recorded(&events, |r| {
                            r["direction"] == "runtime"
                                && r["message"]["type"] == "terminal"
                                && r["message"]["agent"] == waiting_agent
                                && response(&r["message"]).is_ok_and(|v| {
                                    v["tool_name"] == "do_job" && v["data"]["job_id"] == JOBS[other]
                                })
                        })
                        .await?;
                        let args = json!({"job_id":JOBS[first ^ 2]});
                        loop {
                            let reply = guest
                                .command(
                                    first_agent,
                                    format!("echo '{args}' > /service/do_job; cat /service/result"),
                                )
                                .await?;
                            let value = response(&reply)?;
                            if value["tool_name"] == "do_job" {
                                assert_eq!(value["data"]["job_id"], JOBS[first ^ 2]);
                                break;
                            }
                            assert_eq!(value["status"], "busy");
                            hibana_runtime::host::sleep(Duration::from_millis(10)).await?;
                        }
                        assert_eq!(
                            guest.command(first_agent, "exit 0".into()).await?["exit_status"],
                            0
                        );
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    };
                    async {
                        recorded(&events, |r| {
                            r["direction"] == "domain_start"
                                && r["message"]["arguments"]["job_id"] == JOBS[first]
                        })
                        .await?;
                        let args = json!({"job_id":JOBS[waiting]});
                        let busy = guest
                            .command(
                                waiting_agent,
                                format!("echo '{args}' > /service/do_job; cat /service/result"),
                            )
                            .await?;
                        assert_eq!(response(&busy)?["status"], "busy", "{busy}");
                        if early_retry {
                            let retry = guest
                                .command(
                                    waiting_agent,
                                    format!("echo '{args}' > /service/do_job; cat /service/result"),
                                )
                                .await?;
                            assert_eq!(response(&retry)?["status"], "busy");
                        }
                        let args = json!({"job_id":JOBS[other]});
                        let free = guest
                            .command(
                                waiting_agent,
                                format!("echo '{args}' > /service/do_job; cat /service/result"),
                            )
                            .await?;
                        assert_eq!(response(&free)?["data"]["job_id"], JOBS[other]);
                        recorded(&events, |r| {
                            r["direction"] == "runtime"
                                && r["message"]["type"] == "terminal"
                                && r["message"]["agent"] == first_agent
                                && response(&r["message"]).is_ok_and(|v| {
                                    v["tool_name"] == "do_job" && v["data"]["job_id"] == JOBS[first]
                                })
                        })
                        .await?;
                        let args = json!({"job_id":JOBS[waiting]});
                        loop {
                            let reply = guest
                                .command(
                                    waiting_agent,
                                    format!("echo '{args}' > /service/do_job; cat /service/result"),
                                )
                                .await?;
                            let value = response(&reply)?;
                            if value["tool_name"] == "do_job" {
                                assert_eq!(value["data"]["job_id"], JOBS[waiting]);
                                break;
                            }
                            assert_eq!(value["status"], "busy");
                            hibana_runtime::host::sleep(Duration::from_millis(10)).await?;
                        }
                        assert_eq!(
                            guest.command(waiting_agent, "exit 0".into()).await?["exit_status"],
                            0
                        );
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    };
                    async {
                        let report = guest
                            .command(
                                "SUPERVISOR",
                                "echo '{}' > /service/collect_reports; cat /service/result".into(),
                            )
                            .await?;
                        assert_eq!(
                            response(&report)?["data"]["jobs"]
                                .as_array()
                                .ok_or("actual completed jobs")?
                                .len(),
                            4
                        );
                        assert_eq!(
                            guest.command("SUPERVISOR", "exit 0".into()).await?["exit_status"],
                            0
                        );
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    };
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(result["protocol"]["status"], "complete");
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let audit = crate::manufacturing_audit::audit(&events)?;
        save(&output.join("audit.json"), &audit)?;
        assert!(
            audit["distinct_machine_overlap_pairs"]
                .as_u64()
                .ok_or("actual machine overlap")?
                >= 1
        );
        let mutants =
            crate::manufacturing_audit::check_mutants(&events, &output.join("record-mutants"))?;
        outcomes.push(json!({"case":output,"first_job":JOBS[first],"initial_busy_job":JOBS[waiting],"switched_job":JOBS[other],"retry_before_other_job":early_retry,"audit":audit,"record_mutants_rejected":mutants["checked"].as_array().ok_or("manufacturing mutants")?.len()}));
    }
    assert_eq!(outcomes.len(), 8);
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,"scored_benchmark":false,"controlled_schedule":true,"cases":outcomes}),
    )?;
    Ok(())
}
