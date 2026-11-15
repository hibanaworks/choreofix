//! Real wash requests and the neutral machine world; no application permissions.
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
fn agent(job: usize) -> &'static str {
    if job.is_multiple_of(2) {
        "TECHNICIAN_A"
    } else {
        "TECHNICIAN_B"
    }
}
fn file(job: usize) -> &'static str {
    if job < 2 {
        "/service/do_lathe_job"
    } else {
        "/service/do_mill_job"
    }
}
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"]
        .as_str()
        .ok_or("manufacturing terminal output")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("missing manufacturing reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty reply")?
        .map_err(Into::into)
}
async fn write_job(guest: &process::Guest, job: usize, padding: &str) -> Result<()> {
    let arguments = json!({"job_id":JOBS[job]});
    loop {
        let reply = guest
            .command(
                agent(job),
                format!("printf '%s' '{arguments}{padding}' > {}", file(job)),
            )
            .await?;
        let stdout = reply["stdout"].as_str().ok_or("job write output")?;
        if !stdout.contains("wash:") {
            return Ok(());
        }
        if !stdout.contains("(os error 2)") && !stdout.contains("(os error 6)") {
            return Err(format!("unexpected job write failure: {reply}").into());
        }
        hibana_runtime::host::sleep(Duration::from_millis(5)).await?;
    }
}
async fn read_job(guest: &process::Guest, job: usize) -> Result<()> {
    let reply = guest
        .command(agent(job), format!("cat {}", file(job)))
        .await?;
    let actual = response(&reply)?;
    assert_eq!(actual["tool_name"], "do_job");
    assert_eq!(actual["success"], true, "{reply}");
    assert_eq!(actual["data"]["job_id"], JOBS[job]);
    Ok(())
}
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
#[ignore = "requires source-qualified operation files, actual wash and neutral world"]
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
        std::env::var_os("CHOREOFIX_MANUFACTURING_WASI_BINARY").ok_or("manufacturing binary")?,
    )
    .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_BUILD_RECEIPT").ok_or("manufacturing receipt")?,
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
            &json!({"task":"11E","depth":0,"job_order":order.map(|job|JOBS[job]),
            "adversarial_inputs":adversarial,"domain_delay_multiplier":0.01,"model_calls":0,"scored_benchmark":false,
            "build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,"response_read_order":"reverse_of_requests",
            "test_barriers":"none; actual job writes precede all result reads"}),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, _) = oracle::Simulator::new("11E", 0, 0.01, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["11E"], &output, trace, simulator)?;
        let ((), result) = hibana_runtime::race(
            async {
                async {
                    for (position, job) in order.iter().copied().enumerate() {
                        if adversarial {
                            let invalid = match job {
                                0 => r#"{"job_id":"lathe_job_b"}"#,
                                1 => r#"{"job_id":0}"#,
                                2 => r#"{"job_id":"mill_job_a","owner":"TECHNICIAN_A"}"#,
                                _ => "{",
                            };
                            let bad = guest
                                .command(
                                    agent(job),
                                    format!("printf '%s' '{invalid}' > {}", file(job)),
                                )
                                .await?;
                            assert!(
                                bad["stdout"]
                                    .as_str()
                                    .ok_or("invalid argument output")?
                                    .contains("(os error 28)"),
                                "{bad}"
                            );
                            if position == 0 {
                                let private = guest
                                    .command(
                                        agent(job),
                                        "echo 'private computation' > /work/notes; cat /work/notes"
                                            .into(),
                                    )
                                    .await?;
                                assert!(
                                    private["stdout"]
                                        .as_str()
                                        .ok_or("private output")?
                                        .contains("private computation")
                                );
                            }
                        }
                        let padding = if adversarial && position == 0 {
                            " ".repeat(1200)
                        } else {
                            String::new()
                        };
                        write_job(&guest, job, &padding).await?;
                    }
                    for job in order.into_iter().rev() {
                        read_job(&guest, job).await?;
                        if adversarial {
                            let arguments = json!({"job_id":JOBS[job]});
                            let again = guest
                                .command(agent(job), format!("echo '{arguments}' > {}", file(job)))
                                .await?;
                            assert!(
                                again["stdout"]
                                    .as_str()
                                    .ok_or("duplicate output")?
                                    .contains("wash:"),
                                "{again}"
                            );
                        }
                    }
                    if adversarial {
                        let bad = guest
                            .command(
                                "SUPERVISOR",
                                "echo '{\"scope\":\"all_jobs\"}' > /service/collect_reports".into(),
                            )
                            .await?;
                        assert!(
                            bad["stdout"]
                                .as_str()
                                .ok_or("invalid report output")?
                                .contains("(os error 28)")
                        );
                    }
                    let report = guest
                        .command(
                            "SUPERVISOR",
                            "echo '{}' > /service/collect_reports; cat /service/collect_reports"
                                .into(),
                        )
                        .await?;
                    let report = response(&report)?;
                    assert_eq!(report["success"], true);
                    assert_eq!(
                        report["data"]["jobs"]
                            .as_array()
                            .ok_or("actual reports")?
                            .len(),
                        4
                    );
                    if adversarial {
                        let again = guest
                            .command("SUPERVISOR", "echo '{}' > /service/collect_reports".into())
                            .await?;
                        assert!(
                            again["stdout"]
                                .as_str()
                                .ok_or("duplicate report")?
                                .contains("wash:")
                        );
                    }
                    for actor in ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"] {
                        assert_eq!(
                            guest.command(actor, "exit 0".into()).await?["exit_status"],
                            0
                        );
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let mutants =
            crate::manufacturing_audit::check_mutants(&events, &output.join("record-mutants"))?;
        let checked = &mutants["baseline"];
        save(&output.join("audit.json"), checked)?;
        assert_eq!(checked["status"], "passed");
        assert_eq!(checked["actual_resource_endpoint_requests"], 4);
        assert_eq!(checked["actual_resource_endpoint_returns"], 4);
        outcomes.push(json!({"case":output,"task":"11E","job_order":order.map(|job|JOBS[job]),"adversarial_inputs":adversarial,
            "audit_file":"audit.json","audit_sha256":digest(&output.join("audit.json"))?,"events_sha256":digest(&events)?,
            "dynamic_protocol":checked["dynamic_protocol"]["status"],"common_score_complete":true,
            "record_mutants_rejected":mutants["checked"].as_array().ok_or("manufacturing mutations")?.len()}));
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
#[ignore = "requires actual wash and normal-delay machine effects; tests refusal then free-machine work"]
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
        std::env::var_os("CHOREOFIX_MANUFACTURING_WASI_BINARY").ok_or("manufacturing binary")?,
    )
    .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_BUILD_RECEIPT").ok_or("manufacturing receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (first, early_retry) in
        (0usize..4).flat_map(|first| [false, true].map(move |retry| (first, retry)))
    {
        let waiting = first ^ 1;
        let other = waiting ^ 2;
        let remaining = first ^ 2;
        let output = directory.join(format!("case{}", outcomes.len()));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "11E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"11E","depth":0,"first_job":JOBS[first],"initial_busy_job":JOBS[waiting],
            "switched_job":JOBS[other],"retry_before_other_job":early_retry,"domain_delay_multiplier":1.0,"model_calls":0,
            "scored_benchmark":false,"build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "test_barriers":"wait only for the actual first domain_start before trying the occupied machine"}),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, _) = oracle::Simulator::new("11E", 0, 1.0, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["11E"], &output, trace, simulator)?;
        let ((), result) = hibana_runtime::race(
            async {
                async {
                    write_job(&guest, first, "").await?;
                    recorded(&events, |row| {
                        row["direction"] == "domain_start"
                            && row["message"]["arguments"]["job_id"] == JOBS[first]
                    })
                    .await?;
                    for _ in 0..if early_retry { 2 } else { 1 } {
                        let args = json!({"job_id":JOBS[waiting]});
                        let busy = guest
                            .command(
                                agent(waiting),
                                format!(
                                    "printf '%s' '{args}' | dd of={} oflag=nonblock",
                                    file(waiting)
                                ),
                            )
                            .await?;
                        let stdout = busy["stdout"].as_str().ok_or("busy request output")?;
                        assert!(
                            stdout.contains("(os error 2)") || stdout.contains("(os error 6)"),
                            "occupied machine must refuse this unadmitted request: {busy}"
                        );
                    }
                    async {
                        async {
                            read_job(&guest, first).await?;
                        };
                        async {
                            write_job(&guest, other, "").await?;
                            read_job(&guest, other).await?;
                        };
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }
                    .await?;
                    async {
                        async {
                            write_job(&guest, waiting, "").await?;
                            read_job(&guest, waiting).await?;
                        };
                        async {
                            write_job(&guest, remaining, "").await?;
                            read_job(&guest, remaining).await?;
                        };
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }
                    .await?;
                    let report = guest
                        .command(
                            "SUPERVISOR",
                            "echo '{}' > /service/collect_reports; cat /service/collect_reports"
                                .into(),
                        )
                        .await?;
                    assert_eq!(
                        response(&report)?["data"]["jobs"]
                            .as_array()
                            .ok_or("four reports")?
                            .len(),
                        4
                    );
                    for actor in ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"] {
                        assert_eq!(
                            guest.command(actor, "exit 0".into()).await?["exit_status"],
                            0
                        );
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        assert_eq!(result["backend_errors"], json!([]));
        let mutants =
            crate::manufacturing_audit::check_mutants(&events, &output.join("record-mutants"))?;
        let checked = &mutants["baseline"];
        save(&output.join("audit.json"), checked)?;
        assert!(
            checked["distinct_machine_overlap_pairs"]
                .as_u64()
                .ok_or("actual machine overlap")?
                > 0,
            "the other machine must perform real overlapping work"
        );
        outcomes.push(json!({"case":output,"first_job":JOBS[first],"retry_before_other_job":early_retry,"audit_file":"audit.json",
            "audit_sha256":digest(&output.join("audit.json"))?,"events_sha256":digest(&events)?,"dynamic_protocol":checked["dynamic_protocol"]["status"],
            "distinct_machine_overlap_pairs":checked["distinct_machine_overlap_pairs"],"record_mutants_rejected":mutants["checked"].as_array().ok_or("mutants")?.len()}));
    }
    assert_eq!(outcomes.len(), 8);
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,"scored_benchmark":false,"cases":outcomes}),
    )?;
    Ok(())
}
