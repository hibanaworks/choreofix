use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub(crate) const CASES: [&str; 2] = ["reference-first-no-go", "baseline-first-go"];

pub(super) async fn read(guest: &process::Guest, agent: &str, path: &str) -> Result<Value> {
    let unavailable = format!("wash: read {path}: Permission denied (os error 2)\nwash> ");
    loop {
        let reply = guest.command(agent, format!("cat {path}")).await?;
        let stdout = reply["stdout"].as_str().ok_or("missing discovery stdout")?;
        if stdout != unavailable {
            return serde_json::Deserializer::from_str(stdout)
                .into_iter()
                .next()
                .ok_or("missing discovery reply")?
                .map_err(Into::into);
        }
        hibana_runtime::host::sleep(Duration::from_millis(20)).await?;
    }
}

pub(super) async fn write(
    guest: &process::Guest,
    agent: &str,
    path: &str,
    arguments: Value,
) -> Result<()> {
    let unavailable = format!("wash: close output {path}: Permission denied (os error 2)\nwash> ");
    let command = format!("echo '{arguments}' > {path}");
    loop {
        let reply = guest.command(agent, command.clone()).await?;
        if reply["stdout"] != unavailable {
            assert_eq!(reply["stdout"], "wash> ", "{agent} {path}: {reply}");
            return Ok(());
        }
        hibana_runtime::host::sleep(Duration::from_millis(20)).await?;
    }
}

async fn effect(guest: &process::Guest, agent: &str, path: &str, arguments: Value) -> Result<()> {
    write(guest, agent, path, arguments).await?;
    let reply = read(guest, agent, path).await?;
    assert_eq!(reply["success"], true, "{agent} {path}: {reply}");
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires selected 14E executable, build receipt and real wash guest"]
async fn actual_wasm_discovers_and_records_either_decision() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let directory =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?)
            .join("drug-discovery-wasi");
    fs::create_dir_all(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_DRUG_DISCOVERY_WASI_BINARY").ok_or("14E binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_DRUG_DISCOVERY_BUILD_RECEIPT").ok_or("14E build receipt")?,
    );
    let auditor = PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("14E auditor")?)
        .canonicalize()?;
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let build = archive_build_receipt(&binary, &receipt, &directory)?;
    projection::archive(&binary, &directory, "14E")?;
    let mut cases = Vec::new();
    for (index, case) in CASES.into_iter().enumerate() {
        let output = directory.join(case);
        fs::create_dir(&output)?;
        let go = index == 1;
        fs::hard_link(
            directory.join("projection.json"),
            output.join("projection.json"),
        )?;
        save(
            &output.join("case.json"),
            &json!({
                "task":"14E", "depth":0, "case":case, "go":go, "build":build,
                "delay_multiplier":0.10, "model_calls":0, "scored_benchmark":false,
                "runtime_binary_sha256":digest(&binary)?, "guest_wasm_sha256":digest(&module)?,
                "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            }),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, fixture) = oracle::Simulator::new("14E", 0, 0.10, trace.clone(), &[])?;
        save(&output.join("fixture.json"), &fixture)?;
        let (guest, runtime) = process::guest(&binary, &module, &[], &output, trace, simulator)?;
        let ((), outcome) = hibana_runtime::race(async {
            async {
                async {
                    if !go {
                        let refused = guest.command("CHEMIST",
                            "echo '{\"sample\":\"documentation\"}' > /service/run_hplc_analysis/synthesis_qc".into()).await?;
                        assert!(refused["stdout"].as_str().is_some_and(|out| out.starts_with("wash: ")), "{refused}");
                        let premature = guest.command("CHEMIST",
                            "echo '{\"sample\":\"documentation\"}' > /service/run_hplc_analysis/documentation".into()).await?;
                        assert!(premature["stdout"].as_str().is_some_and(|out| out.starts_with("wash: ")), "{premature}");
                    }
                    effect(&guest,"CHEMIST","/service/run_hplc_analysis/synthesis_qc",json!({"sample":"synthesis_qc"})).await?;
                    effect(&guest,"CHEMIST","/service/run_mass_spec_analysis/molecular_id",json!({"sample":"molecular_id"})).await?;
                    let decision = read(&guest,"CHEMIST","/service/decision").await?;
                    assert_eq!(decision["data"]["go"], go);
                    effect(&guest,"CHEMIST","/service/run_hplc_analysis/documentation",json!({"sample":"documentation"})).await?;
                    assert_eq!(guest.command("CHEMIST","exit 0".into()).await?["exit_status"], 0);
                };
                async {
                    effect(&guest,"BIOLOGIST","/service/run_cell_assay/culture_prep",json!({"assay_type":"culture_prep"})).await?;
                    let compound = read(&guest, "BIOLOGIST", "/service/compound").await?;
                    assert_eq!(compound["identity"]["success"], true);
                    effect(&guest,"BIOLOGIST","/service/run_cell_assay/bioassay",json!({"assay_type":"bioassay"})).await?;
                    effect(&guest,"BIOLOGIST","/service/run_hplc_analysis/stability",json!({"sample":"stability"})).await?;
                    assert_eq!(guest.command("BIOLOGIST","exit 0".into()).await?["exit_status"], 0);
                };
                async {
                    if !go { hibana_runtime::host::sleep(Duration::from_millis(150)).await?; }
                    effect(&guest,"TOXICOLOGIST","/service/run_mass_spec_analysis/baseline_markers",json!({"sample":"baseline_markers"})).await?;
                    let compound = read(&guest, "TOXICOLOGIST", "/service/compound").await?;
                    assert_eq!(compound["identity"]["success"], true);
                    effect(&guest,"TOXICOLOGIST","/service/run_hplc_analysis/purity_check",json!({"sample":"purity_check"})).await?;
                    effect(&guest,"TOXICOLOGIST","/service/run_cell_assay/cytotoxicity",json!({"assay_type":"cytotoxicity"})).await?;
                    assert_eq!(guest.command("TOXICOLOGIST","exit 0".into()).await?["exit_status"], 0);
                };
                async {
                    if go { hibana_runtime::host::sleep(Duration::from_millis(150)).await?; }
                    effect(&guest,"LEAD_SCIENTIST","/service/run_mass_spec_analysis/reference_methods",json!({"sample":"reference_methods"})).await?;
                    let reports = [
                        ("/service/bioactivity", "assay"),
                        ("/service/toxicity", "toxicity"),
                    ];
                    for (path, evidence) in reports {
                        assert_eq!(read(&guest,"LEAD_SCIENTIST",path).await?[evidence]["success"], true);
                    }
                    let structural = ("structural_confirmation","/service/run_hplc_analysis/structural_confirmation");
                    let characterization = ("characterization","/service/run_mass_spec_analysis/characterization");
                    let analyses = if go { [structural, characterization] } else { [characterization, structural] };
                    for (sample, path) in analyses {
                        write(&guest, "LEAD_SCIENTIST", path, json!({"sample":sample})).await?;
                    }
                    for (_, path) in analyses {
                        assert_eq!(read(&guest,"LEAD_SCIENTIST",path).await?["success"], true);
                    }
                    write(&guest,"LEAD_SCIENTIST","/service/submit_decision",json!({"go":go})).await?;
                    let result = read(&guest,"LEAD_SCIENTIST","/service/submit_decision").await?;
                    assert_eq!(result["data"]["go"], go);
                    assert_eq!(guest.command("LEAD_SCIENTIST","exit 0".into()).await?["exit_status"], 0);
                };
                Ok(())
            };
            async { runtime.await };
        }, async {
            hibana_runtime::host::sleep(Duration::from_secs(45)).await?;
            Err(format!("actual 14E Wasm deadline: {case}").into())
        }).await?;
        save(&output.join("runtime-result.json"), &outcome)?;
        assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
        assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
        assert_eq!(outcome["score"]["complete"], true, "{outcome}");
        assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
        let checked = Command::new(&auditor)
            .args(["--audit", "14E"])
            .arg(&events)
            .output()?;
        fs::write(output.join("audit-stderr.log"), &checked.stderr)?;
        if !checked.status.success() {
            return Err(format!("{case}: {}", String::from_utf8_lossy(&checked.stderr)).into());
        }
        let audit: Value = serde_json::from_slice(&checked.stdout)?;
        assert_eq!(audit["decision"], go, "{case}: {audit}");
        assert_eq!(audit["domain_calls"], 13, "{case}: {audit}");
        assert!(
            audit["independent_effect_overlaps"]
                .as_u64()
                .is_some_and(|count| count > 0),
            "{case}: {audit}"
        );
        save(&output.join("audit.json"), &audit)?;
        cases.push(json!({
            "case":case, "audit_sha256":digest(&output.join("audit.json"))?, "audit":audit
        }));
    }
    save(
        &directory.join("mechanism.json"),
        &json!({
            "task":"14E", "status":"passed", "cases":cases,
            "auditor_sha256":digest(&auditor)?, "model_calls":0, "scored_benchmark":false,
        }),
    )
}
