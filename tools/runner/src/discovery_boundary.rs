//! Four real wash guests; input scripts are test data, not simulator policy.
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
        .ok_or("instrument terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing service response {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty instrument reply")?
        .map_err(Into::into)
}
async fn chemist(guest: &process::Guest) -> Result<()> {
    let reply=guest.command("CHEMIST","echo '{\"sample\":\"synthesis_qc\"}' > /service/run_hplc_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("CHEMIST","echo '{\"sample\":\"molecular_id\"}' > /service/run_mass_spec_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("CHEMIST","echo '{\"sample\":\"documentation\"}' > /service/run_hplc_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command("CHEMIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn biologist(guest: &process::Guest) -> Result<()> {
    let reply=guest.command("BIOLOGIST","echo '{\"assay_type\":\"culture_prep\"}' > /service/run_cell_assay; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "BIOLOGIST",
            "echo '{\"assay_type\":\"bioassay\"}' > /service/run_cell_assay; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "BIOLOGIST",
            "echo '{\"sample\":\"stability\"}' > /service/run_hplc_analysis; cat /service/result"
                .into(),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn toxicologist(guest: &process::Guest) -> Result<()> {
    let reply=guest.command("TOXICOLOGIST","echo '{\"sample\":\"baseline_markers\"}' > /service/run_mass_spec_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("TOXICOLOGIST","echo '{\"sample\":\"purity_check\"}' > /service/run_hplc_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("TOXICOLOGIST","echo '{\"assay_type\":\"cytotoxicity\"}' > /service/run_cell_assay; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(
        guest.command("TOXICOLOGIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn lead(guest: &process::Guest, go: bool) -> Result<()> {
    let reply=guest.command("LEAD_SCIENTIST","echo '{\"sample\":\"reference_methods\"}' > /service/run_mass_spec_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("LEAD_SCIENTIST","echo '{\"sample\":\"structural_confirmation\"}' > /service/run_hplc_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply=guest.command("LEAD_SCIENTIST","echo '{\"sample\":\"characterization\"}' > /service/run_mass_spec_analysis; cat /service/result".into()).await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    let reply = guest
        .command(
            "LEAD_SCIENTIST",
            format!(
                "echo '{}' > /service/submit_decision; cat /service/result",
                json!({"go":go})
            ),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    assert_eq!(response(&reply)?["data"]["go"], go, "{reply}");
    assert_eq!(
        guest.command("LEAD_SCIENTIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current discovery runtime, source receipt, wash and common simulator"]
async fn actual_wasi_instrument_custody_and_both_decisions() -> Result<()> {
    run(vec![(true, 0.01), (false, 0.01), (true, 1.0)]).await
}
#[hibana_runtime::scope]
async fn run(cases: Vec<(bool, f64)>) -> Result<()> {
    if cases.is_empty() {
        return Err("empty instrument qualification".into());
    }
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("instrument evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "discovery-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_DISCOVERY_WASI_BINARY").ok_or("instrument binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_DISCOVERY_BUILD_RECEIPT").ok_or("instrument build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (index, (go, delay)) in cases.into_iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "14E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"14E","go":go,"delay_multiplier":delay,"build":receipt,
   "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,
   "harness_binary_sha256":digest(&std::env::current_exe()?)?,"model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("14E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["14E"], &output, trace, simulator)?;
        let ((), (), (), (), result) = hibana_runtime::race(
            async {
                async { chemist(&guest).await };
                async { biologist(&guest).await };
                async { toxicologist(&guest).await };
                async { lead(&guest, go).await };
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
                .ok_or("actual instrument effects")?
                .len(),
            13
        );
        let audit = crate::discovery_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        assert_eq!(audit["go"], go, "{audit}");
        if index == 0 {
            let mutants = crate::discovery_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
            assert_eq!(mutants["mutants_rejected"], 129);
        }
        if delay == 1.0 {
            assert!(
                audit["independent_instrument_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0),
                "{audit}"
            );
        }
        outcomes.push(json!({"case":index,"go":go,"audit":audit,"result":result}));
        save(
            &directory.join("progress.json"),
            &json!({"completed":outcomes.len(),"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","task":"14E","conditions":outcomes.len(),"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
