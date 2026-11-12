//! Real four-guest trials; these schedules are not simulator permissions.
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

#[hibana_runtime::main]
#[test]
#[ignore = "requires current 13M runtime, source receipt, wash and common simulator"]
async fn actual_wasi_reconsideration_and_two_instrument_qc() -> Result<()> {
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
        "lab-quality-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_QUALITY_WASI_BINARY").ok_or("laboratory binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_QUALITY_BUILD_RECEIPT")
            .ok_or("laboratory source receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    let cases = [
        (
            0u8,
            &[("CHEMIST", true), ("BIOLOGIST", true)] as &[(&str, bool)],
            &[] as &[(&str, u32)],
            0.01,
        ),
        (0, &[("BIOLOGIST", true), ("CHEMIST", true)], &[], 0.01),
        (
            1,
            &[("CHEMIST", false), ("CHEMIST", true), ("BIOLOGIST", true)],
            &[],
            0.01,
        ),
        (
            1,
            &[("BIOLOGIST", false), ("BIOLOGIST", true), ("CHEMIST", true)],
            &[],
            0.01,
        ),
        (
            2,
            &[
                ("CHEMIST", false),
                ("CHEMIST", false),
                ("CHEMIST", true),
                ("BIOLOGIST", true),
            ],
            &[],
            0.01,
        ),
        (
            2,
            &[
                ("BIOLOGIST", false),
                ("BIOLOGIST", false),
                ("BIOLOGIST", true),
                ("CHEMIST", true),
            ],
            &[],
            1.0,
        ),
        (
            0,
            &[
                ("CHEMIST", true),
                ("BIOLOGIST", false),
                ("BIOLOGIST", false),
                ("BIOLOGIST", true),
            ],
            &[("verify_result", 1), ("verify_result", 2)],
            0.01,
        ),
        (
            0,
            &[
                ("BIOLOGIST", true),
                ("CHEMIST", false),
                ("CHEMIST", false),
                ("CHEMIST", true),
            ],
            &[("verify_result", 1), ("verify_result", 2)],
            0.01,
        ),
        (
            0,
            &[
                ("CHEMIST", false),
                ("BIOLOGIST", false),
                ("CHEMIST", false),
                ("BIOLOGIST", false),
                ("CHEMIST", true),
                ("BIOLOGIST", true),
            ],
            &[
                ("verify_result", 0),
                ("verify_result", 1),
                ("verify_result", 2),
                ("verify_result", 3),
            ],
            1.0,
        ),
        (
            0,
            &[
                ("BIOLOGIST", false),
                ("CHEMIST", false),
                ("BIOLOGIST", false),
                ("CHEMIST", false),
                ("BIOLOGIST", true),
                ("CHEMIST", true),
            ],
            &[
                ("verify_result", 0),
                ("verify_result", 1),
                ("verify_result", 2),
                ("verify_result", 3),
            ],
            0.01,
        ),
    ];
    for (index, &(depth, qc_requests, failures, delay)) in cases.iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "13M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"13M","depth":depth,"qc_requests":qc_requests,"injected_failures":failures,
            "delay_multiplier":delay,"runtime_binary_sha256":digest(&binary)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "guest_wasm_sha256":digest(module)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"build":receipt,
            "model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("13M", depth, delay, trace.clone(), failures)?;
        let (guest, runtime) =
            process::guest(&binary, module, &["13M"], &output, trace, simulator)?;
        let ((), (), (), (), result) = hibana_runtime::race(async {
            async {
                // The guest receives actual QC feedback with the submitted operation.
                for (tool, arguments) in [
                    ("run_synthesis", json!({"compound":"novel_compound"})),
                    ("run_analysis", json!({"sample":"synthesized_compound"})),
                ] {
                    let reply = guest.command("CHEMIST", format!("echo '{arguments}' > /service/{tool}; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["success"], true, "{reply}");
                }
                let reply = guest.command("CHEMIST", "echo '{\"material\":\"compound_purification\"}' > /service/run_separation; cat /service/result".into()).await?;
                let mut actual = response(&reply)?;
                assert_eq!(actual["operation"]["success"], true, "{actual}");
                while actual["verification"]["success"] == false {
                    let reply = guest.command("CHEMIST", "echo '{\"sample\":\"synthesized_compound\"}' > /service/run_analysis; cat /service/result".into()).await?;
                    actual = response(&reply)?;
                    assert_eq!(actual["operation"]["success"], true, "{actual}");
                }
                assert_eq!(actual["verification"]["success"], true, "{actual}");
                let reply = guest.command("CHEMIST", "echo '{\"sample\":\"final_validation\"}' > /service/run_analysis; cat /service/result".into()).await?;
                assert_eq!(response(&reply)?["success"], true, "{reply}");
                assert_eq!(guest.command("CHEMIST", "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            };
            async {
                for (tool, arguments) in [
                    ("run_separation", json!({"material":"cell_cultures"})),
                    ("run_analysis", json!({"sample":"fluorescence"})),
                ] {
                    let reply = guest.command("BIOLOGIST", format!("echo '{arguments}' > /service/{tool}; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["success"], true, "{reply}");
                }
                loop {
                    let reply = guest.command("BIOLOGIST", "echo '{\"compound\":\"novel_compound\",\"cell_type\":\"cell_cultures\"}' > /service/run_bioassay; cat /service/result".into()).await?;
                    let actual = response(&reply)?;
                    if actual["success"] == false { continue; }
                    assert_eq!(actual["operation"]["success"], true, "{actual}");
                    match actual["verification"]["success"].as_bool() {
                        Some(true) => break,
                        Some(false) => continue,
                        None => return Err(format!("missing actual QC decision: {actual}").into()),
                    }
                }
                assert_eq!(guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            };
            async {
                for (tool, arguments) in [
                    ("run_analysis", json!({"sample":"molecular_detail"})),
                    ("run_separation", json!({"material":"verification"})),
                ] {
                    let reply = guest.command("ANALYST", format!("echo '{arguments}' > /service/{tool}; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["success"], true, "{reply}");
                }
                assert_eq!(guest.command("ANALYST", "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            };
            async {
                for &(scientist, expected_success) in qc_requests {
                    let arguments = json!({"scientist":scientist,"data_type":if scientist == "CHEMIST" {"purification"} else {"bioassay"}});
                        let reply = guest.command("QC_CONTROLLER", format!("echo '{arguments}' > /service/verify_result; cat /service/result")).await?;
                        let actual = response(&reply)?;
                        assert_eq!(actual["tool_name"], "verify_result", "{actual}");
                        assert_eq!(actual["success"], expected_success, "{actual}");
                }
                assert_eq!(guest.command("QC_CONTROLLER", "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            };
            async { runtime.await };
        }, hibana_runtime::host::deadline(Duration::from_secs(300))).await?;
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
        outcomes.push(
            json!({"case":index,"depth":depth,"qc_requests":qc_requests,"injected_failures":failures,
            "domain_calls":result["score"]["events"].as_array().ok_or("actual effects")?.len(),
            "dynamic_and_effect_audit_qualified":false,"model_calls":0}),
        );
        save(
            &directory.join("progress.json"),
            &json!({"runtime_completed":outcomes.len(),"completed":0,"outcomes":outcomes}),
        )?;
    }
    // Persist the complete runtime matrix before the independent record audits.
    // A slow or interrupted audit must not require reexecuting unchanged guests.
    // The test still succeeds only after every saved case passes both audits.
    for index in 0..outcomes.len() {
        let output = directory.join(format!("case{index}"));
        let audit = crate::lab_quality_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        if index == 0 {
            let mutants = crate::lab_quality_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
            assert!(mutants["mutants_rejected"].as_u64().is_some_and(|n| n > 0));
        }
        if index == 8 {
            assert!(
                audit["biological_qc_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0),
                "{audit}"
            );
        }
        outcomes[index]["audit"] = audit;
        outcomes[index]["dynamic_and_effect_audit_qualified"] = json!(true);
        save(
            &directory.join("progress.json"),
            &json!({"runtime_completed":outcomes.len(),"completed":index + 1,"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","task":"13M","conditions":outcomes.len(),
        "dynamic_and_effect_audit_qualified":true,"model_calls":0,"scored_benchmark":false,"outcomes":outcomes}),
    )?;
    Ok(())
}
