//! Five actual wash guests and the same common 14M world used by every method.
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
        .ok_or("scientific terminal output")?;
    let at = text.find('{').ok_or("missing scientific response")?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty scientific response")?
        .map_err(Into::into)
}
// A shell request/response is test I/O. Protocol permission remains in the
// production guest's directly written Endpoint operations.
async fn call(
    guest: &process::Guest,
    (agent, tool): (&str, &str),
    arguments: Value,
) -> Result<Value> {
    let reply = guest
        .command(
            agent,
            format!("echo '{arguments}' > /service/{tool}; cat /service/result"),
        )
        .await?;
    response(&reply)
}
async fn chemist(guest: &process::Guest) -> Result<()> {
    loop {
        if call(
            guest,
            ("CHEMIST", "run_hplc_analysis"),
            json!({"sample":"synthesis_qc"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    assert_eq!(
        call(
            guest,
            ("CHEMIST", "run_mass_spec_analysis"),
            json!({"sample":"molecular_id"})
        )
        .await?["success"],
        true
    );
    loop {
        if call(
            guest,
            ("CHEMIST", "run_hplc_analysis"),
            json!({"sample":"documentation"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    assert_eq!(
        guest.command("CHEMIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn biologist(guest: &process::Guest) -> Result<()> {
    loop {
        if call(
            guest,
            ("BIOLOGIST", "run_cell_assay"),
            json!({"assay_type":"culture_prep"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    loop {
        if call(
            guest,
            ("BIOLOGIST", "run_cell_assay"),
            json!({"assay_type":"bioassay"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    loop {
        if call(
            guest,
            ("BIOLOGIST", "run_hplc_analysis"),
            json!({"sample":"stability"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    loop {
        let result = call(guest, ("BIOLOGIST", "await_review"), json!({})).await?;
        if let Some(plan) = result.get("reanalysis") {
            if plan["biology"] == true {
                loop {
                    if call(
                        guest,
                        ("BIOLOGIST", "run_cell_assay"),
                        json!({"assay_type":"bioassay"}),
                    )
                    .await?["success"]
                        == true
                    {
                        break;
                    }
                }
                loop {
                    if call(
                        guest,
                        ("BIOLOGIST", "run_hplc_analysis"),
                        json!({"sample":"stability"}),
                    )
                    .await?["success"]
                        == true
                    {
                        break;
                    }
                }
            }
        } else {
            assert_eq!(result["tool_name"], "verify_regulatory_compliance");
            assert_eq!(result["success"], true);
            break;
        }
    }
    assert_eq!(
        guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn toxicologist(guest: &process::Guest) -> Result<()> {
    assert_eq!(
        call(
            guest,
            ("TOXICOLOGIST", "run_mass_spec_analysis"),
            json!({"sample":"baseline_markers"})
        )
        .await?["success"],
        true
    );
    loop {
        if call(
            guest,
            ("TOXICOLOGIST", "run_hplc_analysis"),
            json!({"sample":"purity_check"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    loop {
        if call(
            guest,
            ("TOXICOLOGIST", "run_cell_assay"),
            json!({"assay_type":"cytotoxicity"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    loop {
        let result = call(guest, ("TOXICOLOGIST", "await_review"), json!({})).await?;
        if let Some(plan) = result.get("reanalysis") {
            if plan["toxicology"] == true {
                loop {
                    if call(
                        guest,
                        ("TOXICOLOGIST", "run_hplc_analysis"),
                        json!({"sample":"purity_check"}),
                    )
                    .await?["success"]
                        == true
                    {
                        break;
                    }
                }
                loop {
                    if call(
                        guest,
                        ("TOXICOLOGIST", "run_cell_assay"),
                        json!({"assay_type":"cytotoxicity"}),
                    )
                    .await?["success"]
                        == true
                    {
                        break;
                    }
                }
            }
        } else {
            assert_eq!(result["tool_name"], "verify_regulatory_compliance");
            assert_eq!(result["success"], true);
            break;
        }
    }
    assert_eq!(
        guest.command("TOXICOLOGIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn lead(guest: &process::Guest, go: bool, mass_first: bool, plans: &[u8]) -> Result<()> {
    assert_eq!(
        call(
            guest,
            ("LEAD_SCIENTIST", "run_mass_spec_analysis"),
            json!({"sample":"reference_methods"})
        )
        .await?["success"],
        true
    );
    if mass_first {
        assert_eq!(
            call(
                guest,
                ("LEAD_SCIENTIST", "run_mass_spec_analysis"),
                json!({"sample":"characterization"})
            )
            .await?["success"],
            true
        );
    }
    loop {
        if call(
            guest,
            ("LEAD_SCIENTIST", "run_hplc_analysis"),
            json!({"sample":"structural_confirmation"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    if !mass_first {
        assert_eq!(
            call(
                guest,
                ("LEAD_SCIENTIST", "run_mass_spec_analysis"),
                json!({"sample":"characterization"})
            )
            .await?["success"],
            true
        );
    }
    let mut rejections = 0;
    loop {
        let result = call(
            guest,
            ("LEAD_SCIENTIST", "submit_decision"),
            json!({"go":go}),
        )
        .await?;
        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["go"], go);
        if result["regulatory_review"]["success"] == true {
            break;
        }
        assert_eq!(result["regulatory_review"]["success"], false);
        let mask = plans[rejections % plans.len()];
        rejections += 1;
        assert_eq!(
            call(
                guest,
                ("LEAD_SCIENTIST", "reconsider"),
                json!({"biology":mask&1!=0,"toxicology":mask&2!=0})
            )
            .await?["success"],
            true
        );
    }
    assert_eq!(
        guest.command("LEAD_SCIENTIST", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
async fn regulator(guest: &process::Guest) -> Result<()> {
    loop {
        if call(
            guest,
            ("REGULATORY_SPECIALIST", "verify_regulatory_compliance"),
            json!({"decision":"go_no_go"}),
        )
        .await?["success"]
            == true
        {
            break;
        }
    }
    assert_eq!(
        guest
            .command("REGULATORY_SPECIALIST", "exit 0".into())
            .await?["exit_status"],
        0
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the fixed-source regulatory-discovery runtime, receipt, wash and common world"]
async fn five_actual_wasm_guests_complete_both_decisions_and_reconsiderations() -> Result<()> {
    run().await
}
#[hibana_runtime::scope]
async fn run() -> Result<()> {
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_WASI_BINARY")
            .ok_or("regulatory binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_BUILD_RECEIPT")
            .ok_or("regulatory build receipt")?,
    );
    let root = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("regulatory evidence root")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "regulatory-discovery-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let assigned = std::env::var("CHOREOFIX_REGULATORY_DISCOVERY_CASE")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?;
    let mut reports = Vec::new();
    for index in 0..12 {
        if assigned.is_some_and(|selected| selected != index) {
            continue;
        }
        let go = index != 1;
        let mass_first = index % 2 == 1;
        let plans: Vec<u8> = if (2..=5).contains(&index) {
            vec![(index - 2) as u8]
        } else {
            vec![0, 1, 2, 3]
        };
        let failures: Vec<(&str, u32)> = match index {
            2..=5 => vec![("verify_regulatory_compliance", 0)],
            6 => vec![
                ("run_hplc_analysis", 1),
                ("run_hplc_analysis", 2),
                ("run_hplc_analysis", 3),
            ],
            7 => vec![
                ("verify_regulatory_compliance", 0),
                ("verify_regulatory_compliance", 1),
                ("verify_regulatory_compliance", 2),
            ],
            8 => vec![
                ("run_hplc_analysis", 0),
                ("run_hplc_analysis", 2),
                ("run_cell_assay", 0),
                ("run_cell_assay", 2),
            ],
            _ => vec![],
        };
        let depth = if index >= 10 { (index - 9) as u8 } else { 0 };
        let delay = if index == 9 { 1.0 } else { 0.01 };
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        let artifact = crate::projection::archive(&binary, &output, "14M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"14M","condition":index,"go":go,"mass_first":mass_first,
   "plans":plans,"failure_overrides":failures,"depth":depth,"delay_multiplier":delay,"build":receipt,
   "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
   "compiled_projection_sha256":digest(&output.join("projection.json"))?,"model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("14M", depth, delay, trace.clone(), &failures)?;
        let (guest, runtime) =
            process::guest(&binary, module, &["14M"], &output, trace, simulator)?;
        let ((), (), (), (), (), result) = hibana_runtime::race(
            async {
                async { chemist(&guest).await };
                async { biologist(&guest).await };
                async { toxicologist(&guest).await };
                async { lead(&guest, go, mass_first, &plans).await };
                async { regulator(&guest).await };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(240)),
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
        let effects = crate::regulatory_discovery_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(effects["simulator_complete"], result["score"]["complete"]);
        assert_eq!(
            effects["runtime_binary_sha256"],
            artifact["runtime_binary_sha256"]
        );
        assert_eq!(
            effects["compiled_projection_sha256"],
            digest(&output.join("projection.json"))?
        );
        let dynamic = &effects["dynamic_protocol"];
        save(&output.join("protocol-audit.json"), dynamic)?;
        save(&output.join("effect-audit.json"), &effects)?;
        if index == 9 {
            assert!(
                effects["max_concurrent_effects"]
                    .as_u64()
                    .is_some_and(|n| n >= 3)
            );
        }
        let mutants = if matches!(index, 0 | 5 | 7 | 11) {
            let result = crate::regulatory_discovery_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
            assert!(result["mutants_rejected"].as_u64().is_some_and(|n| n > 0));
            result["mutants_rejected"].clone()
        } else {
            json!(0)
        };
        reports.push(json!({"condition":index,"score":result["score"],"protocol_audit":dynamic,"effect_audit":effects,"record_mutants_rejected":mutants,"application_effects_certified":false}));
    }
    if reports.is_empty() {
        return Err("no regulatory condition selected".into());
    }
    save(
        &directory.join("qualification.json"),
        &json!({"task":"14M","declared_conditions":12,"assigned":assigned,"cases":reports,
  "model_calls":0,"scored_runs":0,"application_effects_certified":false}),
    )?;
    Ok(())
}
