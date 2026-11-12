//! Scripted 4E inputs through three real wash guests and the original simulator.
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
    let text = reply["stdout"]
        .as_str()
        .ok_or("coordination terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing service response: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service response")?
        .map_err(Into::into)
}
async fn frontend(guest: &process::Guest, visits: usize) -> Result<()> {
    for _ in 0..visits {
        let reply = guest.command("FRONTEND_DEV", "echo '{\"api_name\":\"feature_api_spec\"}' > /service/agree_api; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
        let reply = guest.command("FRONTEND_DEV", "echo '{\"feature_name\":\"frontend_component\"}' > /service/implement_feature; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
    }
    let reply = guest
        .command(
            "FRONTEND_DEV",
            "echo '{\"api_name\":\"feature_api_spec\"}' > /service/agree_api; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["tool_name"], "run_tests", "{reply}");
    assert_eq!(
        guest.command("FRONTEND_DEV", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn backend(guest: &process::Guest, frontend_first: &[bool]) -> Result<()> {
    for &first in frontend_first {
        let reply = guest.command("BACKEND_DEV", "echo '{\"api_name\":\"feature_api_spec\"}' > /service/define_api; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
        if first {
            let reply = guest.command("BACKEND_DEV", "echo '{\"feature_name\":\"frontend_component\"}' > /service/loan_frontend; cat /service/result".into()).await?;
            assert_eq!(response(&reply)?["status"], "accepted", "{reply}");
        }
        let reply = guest.command("BACKEND_DEV", "echo '{\"feature_name\":\"backend_endpoint\"}' > /service/implement_feature; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["success"], true, "{reply}");
    }
    let reply = guest
        .command(
            "BACKEND_DEV",
            "echo '{\"api_name\":\"feature_api_spec\"}' > /service/finalize; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["status"], "accepted", "{reply}");
    let reply = guest
        .command(
            "BACKEND_DEV",
            "echo '{\"suite\":\"integration_tests\"}' > /service/await_tests; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["tool_name"], "run_tests", "{reply}");
    assert_eq!(
        guest.command("BACKEND_DEV", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn tester(guest: &process::Guest) -> Result<()> {
    let reply = guest
        .command(
            "TESTER",
            "echo '{\"suite\":\"integration_tests\"}' > /service/run_tests; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command("TESTER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current coordination WASI binary, receipt, wash and original simulator"]
async fn actual_wasi_coordination_orders_and_repeated_amendments() -> Result<()> {
    run(vec![
        (vec![false], 0.01),
        (vec![true], 0.01),
        (vec![false, true], 0.01),
        (vec![true, false, true, false], 0.01),
        (vec![false], 1.0),
    ])
    .await
}

#[hibana_runtime::scope]
async fn run(cases: Vec<(Vec<bool>, f64)>) -> Result<()> {
    if cases.is_empty() || cases.iter().any(|(orders, _)| orders.is_empty()) {
        return Err("coordination qualification cannot execute an empty input".into());
    }
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("coordination evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "coordination-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_COORDINATION_WASI_BINARY").ok_or("coordination binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_COORDINATION_BUILD_RECEIPT").ok_or("coordination receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (index, (orders, delay)) in cases.into_iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "4E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"4E","frontend_first":orders,
            "delay_multiplier":delay,"model_calls":0,"scored_benchmark":false,"build":receipt,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/coordination_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/coordination_audit.rs"))?}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("4E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["4E"], &output, trace, simulator)?;
        let ((), (), (), result) = hibana_runtime::race(
            async {
                async { frontend(&guest, orders.len()).await };
                async { backend(&guest, &orders).await };
                async { tester(&guest).await };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
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
                .ok_or("original effects")?
                .len(),
            3 * orders.len() + 1
        );
        let audit = crate::coordination_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        save(&output.join("result.json"), &result)?;
        outcomes.push(json!({"case":index,"frontend_first":orders,"audit":audit,"result":result}));
        save(
            &directory.join("progress.json"),
            &json!({"completed":outcomes.len(),"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","task":"4E","conditions":outcomes.len(),"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
