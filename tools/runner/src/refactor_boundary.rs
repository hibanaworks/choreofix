//! Five actual wash guests; scripted choices test the ABI without model calls.
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
    let text = reply["stdout"].as_str().ok_or("refactor terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing service response: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service response")?
        .map_err(Into::into)
}

async fn architect(guest: &process::Guest, reverse: bool) -> Result<()> {
    let modules = if reverse {
        ["module_b_refactor", "module_a_refactor"]
    } else {
        ["module_a_refactor", "module_b_refactor"]
    };
    for title in modules {
        let reply = guest
            .command(
                "ARCHITECT",
                format!(
                    "echo '{}' > /service/design_proposal; cat /service/result",
                    json!({"title":title})
                ),
            )
            .await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
    }
    assert_eq!(
        guest.command("ARCHITECT", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn developer(guest: &process::Guest, agent: &str, reverse: bool) -> Result<()> {
    let module = if agent == "DEVELOPER_A" {
        "module_a"
    } else {
        "module_b"
    };
    let targets = if reverse {
        ["SHARED_LIBRARY", module]
    } else {
        [module, "SHARED_LIBRARY"]
    };
    for target in targets {
        let reply = guest
            .command(
                agent,
                format!(
                    "echo '{}' > /service/refactor_code; cat /service/result",
                    json!({"target":target})
                ),
            )
            .await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
    }
    let reply = guest
        .command(
            agent,
            format!(
                "echo '{}' > /service/merge_code; cat /service/result",
                json!({"branch":module})
            ),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn reviewer(guest: &process::Guest, reverse: bool) -> Result<()> {
    let modules = if reverse {
        ["module_b", "module_a"]
    } else {
        ["module_a", "module_b"]
    };
    for target in modules {
        let reply = guest
            .command(
                "REVIEWER",
                format!(
                    "echo '{}' > /service/code_review; cat /service/result",
                    json!({"pr_id":target})
                ),
            )
            .await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
    }
    assert_eq!(
        guest.command("REVIEWER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn tester(guest: &process::Guest, choices: u8) -> Result<()> {
    for bit in [4, 5] {
        let suites = if choices & (1 << bit) != 0 {
            ["module_b_tests", "module_a_tests"]
        } else {
            ["module_a_tests", "module_b_tests"]
        };
        for suite in suites {
            let reply = guest
                .command(
                    "TESTER",
                    format!(
                        "echo '{}' > /service/run_tests; cat /service/result",
                        json!({"suite":suite})
                    ),
                )
                .await?;
            assert_eq!(response(&reply)?["success"], true, "{reply}");
        }
    }
    assert_eq!(
        guest.command("TESTER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires current refactor WASI binary, receipt, wash and common simulator"]
async fn actual_wasi_refactor_orders_and_parallel_effects() -> Result<()> {
    run(vec![
        (0, 0.01),
        (63, 0.01),
        (21, 0.01),
        (42, 0.01),
        (0, 1.0),
    ])
    .await
}

#[hibana_runtime::scope]
async fn run(cases: Vec<(u8, f64)>) -> Result<()> {
    if cases.is_empty() || cases.iter().any(|(choice, _)| *choice >= 64) {
        return Err("refactor qualification cannot execute an empty or invalid input".into());
    }
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("refactor evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "refactor-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_REFACTOR_WASI_BINARY").ok_or("refactor binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_REFACTOR_BUILD_RECEIPT").ok_or("refactor receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (index, (choices, delay)) in cases.into_iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "4M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"4M","choices":choices,"delay_multiplier":delay,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"model_calls":0,"scored_benchmark":false,"build":receipt,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/refactor_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/refactor_audit.rs"))?}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("4M", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["4M"], &output, trace, simulator)?;
        let ((), (), (), (), (), result) = hibana_runtime::race(
            async {
                async { architect(&guest, choices & 1 != 0).await };
                async { developer(&guest, "DEVELOPER_A", choices & 2 != 0).await };
                async { developer(&guest, "DEVELOPER_B", choices & 4 != 0).await };
                async { reviewer(&guest, choices & 8 != 0).await };
                async { tester(&guest, choices).await };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
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
                .ok_or("actual effects")?
                .len(),
            14
        );
        let audit = crate::refactor_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        // Exercise every business frame once. Other cases retain full positive
        // wire/effect/EPF checks without repeating the same mutation mechanism.
        let mutations = if index == 0 {
            crate::refactor_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutations"),
            )?
        } else {
            Value::Null
        };
        if delay == 1.0 {
            assert!(
                audit["review_test_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0),
                "{audit}"
            );
            // The independent second review can finish before tests permit
            // merging. Preserve its measured overlap, including zero. The
            // controlled native Endpoint schedules test merge/review progress.
        }
        outcomes.push(json!({"case":index,"choices":choices,"audit":audit,"mutations":mutations,"result":result}));
        save(
            &directory.join("progress.json"),
            &json!({"completed":outcomes.len(),"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","task":"4M","conditions":outcomes.len(),"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
