//! Actual Wasm inputs are test schedules, not common simulator permissions.
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
        .ok_or("laboratory terminal output")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("missing laboratory reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty laboratory reply")?
        .map_err(Into::into)
}
async fn chemist(guest: &process::Guest, adversarial: bool) -> Result<()> {
    if adversarial {
        let reply=guest.command("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "invalid_arguments", "{reply}");
    }
    let reply = guest
        .command(
            "CHEMIST",
            "echo '{\"sample\":\"raw_materials\"}' > /service/run_analysis; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("CHEMIST","echo '{\"material\":\"compound_purification\"}' > /service/run_separation; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "CHEMIST",
            "echo '{\"sample\":\"final_validation\"}' > /service/run_analysis; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    if adversarial {
        let reply=guest.command("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis; cat /service/result".into()).await?;
        assert_eq!(response(&reply)?["status"], "out_of_order", "{reply}");
    }
    assert_eq!(
        guest.command("CHEMIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn biologist(guest: &process::Guest, cell_type: Option<&str>) -> Result<()> {
    let reply=guest.command("BIOLOGIST","echo '{\"material\":\"cell_cultures\"}' > /service/run_separation; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "BIOLOGIST",
            "echo '{\"sample\":\"fluorescence\"}' > /service/run_analysis; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let mut arguments = json!({"compound":"compound_purification"});
    if let Some(value) = cell_type {
        arguments["cell_type"] = json!(value);
    }
    let reply = guest
        .command(
            "BIOLOGIST",
            format!("echo '{arguments}' > /service/run_bioassay; cat /service/result"),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        response(&reply)?["data"]["cell_type"],
        cell_type.unwrap_or("")
    );
    assert_eq!(
        guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn analyst(guest: &process::Guest) -> Result<()> {
    let reply = guest
        .command(
            "ANALYST",
            "echo '{\"sample\":\"molecular_detail\"}' > /service/run_analysis; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "ANALYST",
            "echo '{\"material\":\"verification\"}' > /service/run_separation; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command("ANALYST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current laboratory runtime, receipt, wash and common simulator"]
async fn actual_wasi_specimen_causality_and_instrument_custody() -> Result<()> {
    run().await
}
#[hibana_runtime::scope]
async fn run() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("laboratory evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "lab-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_LAB_WASI_BINARY").ok_or("laboratory binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_BUILD_RECEIPT").ok_or("laboratory source receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let long_type = "prepared_culture_".repeat(24);
    let cases = [
        (true, Some("cell_cultures"), 0.01),
        (false, None, 0.01),
        (false, Some(long_type.as_str()), 1.0),
    ];
    let mut outcomes = Vec::new();
    for (index, (adversarial, cell_type, delay)) in cases.into_iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "13E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"13E","adversarial_inputs":adversarial,"cell_type":cell_type,"delay_multiplier":delay,
            "runtime_binary_sha256":digest(&binary)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "guest_wasm_sha256":digest(module)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"build":receipt,"model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("13E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["13E"], &output, trace, simulator)?;
        let ((), (), (), result) = hibana_runtime::race(
            async {
                async { chemist(&guest, adversarial).await };
                async { biologist(&guest, cell_type).await };
                async { analyst(&guest).await };
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
                .ok_or("actual laboratory events")?
                .len(),
            8
        );
        let audit = crate::lab_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        if delay == 1.0 {
            assert!(
                audit["independent_instrument_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0),
                "{audit}"
            );
        }
        if index == 0 {
            let mutants = crate::lab_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
            assert_eq!(mutants["mutants_rejected"], 87);
        }
        outcomes.push(json!({"case":index,"audit":audit,"result":result}));
        save(
            &directory.join("progress.json"),
            &json!({"completed":outcomes.len(),"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","task":"13E","conditions":outcomes.len(),"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
