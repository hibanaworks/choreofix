use crate::{
    drug_discovery_boundary::{read, write},
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

async fn request(
    guest: &process::Guest,
    agent: &str,
    path: &str,
    arguments: Value,
) -> Result<Value> {
    write(guest, agent, path, arguments).await?;
    read(guest, agent, path).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 14H executable, build receipt and wash Wasm"]
async fn actual_wasm_runs_multi_round_study() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_WASI_BINARY").ok_or("14H binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_BUILD_RECEIPT")
            .ok_or("14H build receipt")?,
    );
    let directory =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("14H evidence directory")?);
    let condition = std::env::var("CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE")?.parse::<u8>()?;
    let (rounds, hold, abort, count) = match condition {
        0 => (1, false, false, 13),
        1 => (1, true, false, 14),
        2 => (2, false, true, 21),
        _ => return Err("unknown 14H condition".into()),
    };
    let output = directory.join(format!("multi-round-discovery-wasi-{condition}"));
    fs::create_dir_all(&directory)?;
    fs::create_dir(&output)?;
    let build = archive_build_receipt(&binary, &receipt, &output)?;
    projection::archive(&binary, &output, "14H")?;
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let failures: &[(&str, u32)] = if hold {
        &[("verify_regulatory_compliance", 0)]
    } else {
        &[]
    };
    save(
        &output.join("case.json"),
        &json!({
            "task":"14H", "condition":condition, "model_calls":0, "scored_benchmark":false,
            "failure_overrides":failures, "delay_multiplier":0.10, "build":build,
            "runtime_binary_sha256":digest(&binary)?, "guest_wasm_sha256":digest(&module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
        }),
    )?;
    let events = output.join("events.jsonl");
    let trace = Trace::create(&events)?;
    let (simulator, fixture) = oracle::Simulator::new("14H", 0, 0.10, trace.clone(), failures)?;
    save(&output.join("fixture.json"), &fixture)?;
    let (guest, runtime) = process::guest(&binary, &module, &[], &output, trace, simulator)?;
    let ((), outcome) = hibana_runtime::race(async {
        async {
            async {
                for (path, sample) in [
                    ("/service/run_hplc_analysis/synthesis_qc", "synthesis_qc"),
                    ("/service/run_mass_spec_analysis/molecular_id", "molecular_id"),
                ] {
                    assert_eq!(request(&guest, "CHEMIST", path, json!({"sample":sample})).await?["success"], true);
                }
                let outcome = read(&guest, "CHEMIST", "/service/outcome").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                let actual = request(&guest, "CHEMIST", "/service/run_hplc_analysis/documentation", json!({"sample":"documentation"})).await?;
                assert_eq!(actual["success"], true);
                assert_eq!(actual["data"]["inputs"]["decision"], outcome["decision"]["data"]["event_id"]);
                assert_eq!(guest.command("CHEMIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                assert_eq!(request(&guest, "BIOLOGIST", "/service/run_cell_assay/culture_prep", json!({"assay_type":"culture_prep"})).await?["success"], true);
                for round in 0..rounds {
                    let actual = read(&guest, "BIOLOGIST", "/service/study").await?;
                    assert_eq!(actual["success"], true);
                    let (path, arg) = (
                        "/service/run_hplc_analysis/stability",
                        json!({"sample":"stability"}),
                    );
                    if condition != 1 {
                        assert_eq!(request(&guest, "BIOLOGIST", path, arg.clone()).await?["success"], true);
                    }
                    let bio = request(&guest, "BIOLOGIST", "/service/run_cell_assay/bioassay", json!({"assay_type":"bioassay"})).await?;
                    if round == 0 { assert_eq!(bio["success"], true); }
                    write(&guest, "BIOLOGIST", "/service/submit_evidence", json!({"evidence":"bioassay"})).await?;
                    if condition == 1 {
                        assert_eq!(request(&guest, "BIOLOGIST", path, arg).await?["success"], true);
                    }
                }
                let outcome = read(&guest, "BIOLOGIST", "/service/study").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                assert_eq!(guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                assert_eq!(request(&guest, "TOXICOLOGIST", "/service/run_mass_spec_analysis/baseline_markers", json!({"sample":"baseline_markers"})).await?["success"], true);
                for round in 0..rounds {
                    let actual = read(&guest, "TOXICOLOGIST", "/service/study").await?;
                    assert_eq!(actual["success"], true);
                    assert_eq!(request(&guest, "TOXICOLOGIST", "/service/run_hplc_analysis/purity_check", json!({"sample":"purity_check"})).await?["success"], true);
                    let tox = request(&guest, "TOXICOLOGIST", "/service/run_cell_assay/cytotoxicity", json!({"assay_type":"cytotoxicity"})).await?;
                    if round == 0 { assert_eq!(tox["success"], true); }
                    write(&guest, "TOXICOLOGIST", "/service/submit_evidence", json!({"evidence":"cytotoxicity"})).await?;
                }
                let outcome = read(&guest, "TOXICOLOGIST", "/service/study").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                assert_eq!(guest.command("TOXICOLOGIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                for _ in 0..rounds {
                    let brief = read(&guest, "FORMULATION_SCIENTIST", "/service/study").await?;
                    assert_eq!(brief["compound"]["success"], true);
                    let compound = brief["compound"]["data"]["compound"].as_str().ok_or("actual compound identity")?;
                    let actual = request(&guest, "FORMULATION_SCIENTIST", "/service/prepare_formulation", json!({"compound":compound})).await?;
                    assert_eq!(actual["success"], true);
                }
                let outcome = read(&guest, "FORMULATION_SCIENTIST", "/service/study").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                assert_eq!(guest.command("FORMULATION_SCIENTIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                for round in 0..rounds {
                    let inputs = read(&guest, "CLINICAL_LEAD", "/service/study").await?;
                    assert_eq!(inputs["formulation"]["success"], true);
                    let actual = request(&guest, "CLINICAL_LEAD", "/service/review_clinical_data", json!({"data_type":"combined_review"})).await?;
                    assert_eq!(actual["success"], true);
                    assert_eq!(actual["data"]["assessment"], if round == 0 {"supported"} else {"insufficient_data"});
                }
                let outcome = read(&guest, "CLINICAL_LEAD", "/service/study").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                assert_eq!(guest.command("CLINICAL_LEAD", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                for round in 0..rounds {
                    let actual = read(&guest, "REGULATORY_SPECIALIST", "/service/study").await?;
                    assert_eq!(actual["success"], true);
                    let review = request(&guest, "REGULATORY_SPECIALIST", "/service/verify_regulatory_compliance/initial_review", json!({"review_type":"initial_review"})).await?;
                    assert_eq!(review["success"], !hold && round == 0);
                    if hold {
                        let review = request(&guest, "REGULATORY_SPECIALIST", "/service/verify_regulatory_compliance/hold_review", json!({"review_type":"hold_review"})).await?;
                        assert_eq!(review["success"], true);
                    }
                    write(&guest, "REGULATORY_SPECIALIST", "/service/submit_evidence", json!({"evidence":"regulatory"})).await?;
                }
                let outcome = read(&guest, "REGULATORY_SPECIALIST", "/service/study").await?;
                assert_eq!(outcome["outcome"], if abort { "abort" } else { "continue" });
                assert_eq!(guest.command("REGULATORY_SPECIALIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                for round in 0..rounds {
                    let review = read(&guest, "PROJECT_DIRECTOR", "/service/study").await?;
                    assert_eq!(review["success"], round == 0);
                    let outcome = if round + 1 < rounds {"retry"} else if abort {"abort"} else {"continue"};
                    let actual = request(&guest, "PROJECT_DIRECTOR", "/service/submit_project_decision", json!({"outcome":outcome})).await?;
                    assert_eq!(actual["success"], true);
                    assert_eq!(actual["data"]["outcome"], outcome);
                    assert_eq!(actual["data"]["inputs"]["regulatory"], review["data"]["event_id"]);
                }
                assert_eq!(guest.command("PROJECT_DIRECTOR", "exit 0".into()).await?["exit_status"], 0);
            };
            Ok(())
        };
        async { runtime.await };
    }, async {
        hibana_runtime::host::sleep(Duration::from_secs(55)).await?;
        Err(format!("14H actual Wasm deadline, condition {condition}").into())
    }).await?;
    save(&output.join("runtime-result.json"), &outcome)?;
    assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
    assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
    assert_eq!(outcome["score"]["complete"], true, "{outcome}");
    assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
    assert_eq!(
        outcome["score"]["events"]
            .as_array()
            .ok_or("14H scored effects")?
            .len(),
        count
    );
    let audit = crate::journal::Evidence::open(&events)?
        .audit(crate::multi_round_discovery_audit::check)?;
    assert_eq!(audit["domain_calls"], count);
    assert_eq!(
        audit["terminal_outcome"],
        if abort { "abort" } else { "continue" }
    );
    assert_eq!(audit["samples_consumed"], if abort { 3 } else { 2 });
    save(&output.join("audit.json"), &audit)?;
    Ok(())
}
