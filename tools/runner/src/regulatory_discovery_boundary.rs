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

pub(crate) const EXPECTED: [(usize, usize, usize); 7] = [
    (14, 0, 0),
    (17, 1, 0),
    (19, 1, 0),
    (19, 1, 0),
    (21, 1, 0),
    (17, 0, 3),
    (28, 2, 0),
];

async fn analyse(guest: &process::Guest, actor: &str, path: &str, arguments: Value) -> Result<()> {
    loop {
        write(guest, actor, path, arguments.clone()).await?;
        let result = read(guest, actor, path).await?;
        if result["success"] == true {
            return Ok(());
        }
        assert_eq!(
            result["data"]["status"], "injected_failure",
            "{actor} {path}: {result}"
        );
    }
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 14M app executable, build receipt and real wash guest"]
async fn actual_wasm_reconsideration() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_WASI_BINARY").ok_or("14M binary")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_BUILD_RECEIPT")
            .ok_or("14M build receipt")?,
    );
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let directory =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("14M evidence directory")?);
    let condition = std::env::var("CHOREOFIX_REGULATORY_CASE")?.parse::<u8>()?;
    let &(effect_count, rejections, analysis_failures) = EXPECTED
        .get(usize::from(condition))
        .ok_or("unknown 14M condition")?;
    let output = directory.join(format!("regulatory-discovery-wasi-{condition}"));
    fs::create_dir_all(&directory)?;
    fs::create_dir(&output)?;
    let build = archive_build_receipt(&binary, &receipt, &output)?;
    projection::archive(&binary, &output, "14M")?;
    let failures: &[(&str, u32)] = match condition {
        0 => &[],
        5 => &[("run_cell_assay", 0), ("run_hplc_analysis", 1)],
        6 => &[
            ("verify_regulatory_compliance", 0),
            ("verify_regulatory_compliance", 1),
        ],
        _ => &[("verify_regulatory_compliance", 0)],
    };
    let plan = if condition == 6 {
        3
    } else {
        condition.saturating_sub(1)
    };
    let selected = json!({"biology":plan & 1 != 0,"toxicology":plan & 2 != 0});
    save(
        &output.join("case.json"),
        &json!({
            "task":"14M","condition":condition,"delay_multiplier":0.10,"failures":failures,
            "model_calls":0,"scored_benchmark":false,"build":build,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
        }),
    )?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let (simulator, fixture) = oracle::Simulator::new("14M", 0, 0.10, trace.clone(), failures)?;
    save(&output.join("fixture.json"), &fixture)?;
    let (guest, runtime) = process::guest(&binary, &module, &[], &output, trace, simulator)?;
    let ((), outcome) = hibana_runtime::race(async {
        async {
            async {
                assert!(guest.command("CHEMIST", "echo '{\"sample\":\"documentation\"}' > /service/run_hplc_analysis/documentation".into()).await?
                    ["stdout"].as_str().is_some_and(|out| out.starts_with("wash: ")));
                analyse(&guest, "CHEMIST", "/service/run_hplc_analysis/synthesis_qc", json!({"sample":"synthesis_qc"})).await?;
                analyse(&guest, "CHEMIST", "/service/run_mass_spec_analysis/molecular_id", json!({"sample":"molecular_id"})).await?;
                assert_eq!(read(&guest, "CHEMIST", "/service/approval").await?["success"], true);
                analyse(&guest, "CHEMIST", "/service/run_hplc_analysis/documentation", json!({"sample":"documentation"})).await?;
                assert_eq!(guest.command("CHEMIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                analyse(&guest, "BIOLOGIST", "/service/run_cell_assay/culture_prep", json!({"assay_type":"culture_prep"})).await?;
                assert_eq!(read(&guest,"BIOLOGIST","/service/compound").await?["identity"]["success"], true);
                analyse(&guest, "BIOLOGIST", "/service/run_cell_assay/bioassay", json!({"assay_type":"bioassay"})).await?;
                analyse(&guest, "BIOLOGIST", "/service/run_hplc_analysis/stability", json!({"sample":"stability"})).await?;
                loop {
                    let notice = read(&guest,"BIOLOGIST","/service/study").await?;
                    if notice["success"] == true { break; }
                    assert_eq!(notice,selected);
                    assert_eq!(notice["biology"], true);
                    analyse(&guest, "BIOLOGIST", "/service/run_cell_assay/bioassay", json!({"assay_type":"bioassay"})).await?;
                    analyse(&guest, "BIOLOGIST", "/service/run_hplc_analysis/stability", json!({"sample":"stability"})).await?;
                }
                assert_eq!(guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                analyse(&guest, "TOXICOLOGIST", "/service/run_mass_spec_analysis/baseline_markers", json!({"sample":"baseline_markers"})).await?;
                assert_eq!(read(&guest,"TOXICOLOGIST","/service/compound").await?["identity"]["success"], true);
                analyse(&guest, "TOXICOLOGIST", "/service/run_hplc_analysis/purity_check", json!({"sample":"purity_check"})).await?;
                analyse(&guest, "TOXICOLOGIST", "/service/run_cell_assay/cytotoxicity", json!({"assay_type":"cytotoxicity"})).await?;
                loop {
                    let notice = read(&guest,"TOXICOLOGIST","/service/study").await?;
                    if notice["success"] == true { break; }
                    assert_eq!(notice,selected);
                    assert_eq!(notice["toxicology"], true);
                    analyse(&guest, "TOXICOLOGIST", "/service/run_hplc_analysis/purity_check", json!({"sample":"purity_check"})).await?;
                    analyse(&guest, "TOXICOLOGIST", "/service/run_cell_assay/cytotoxicity", json!({"assay_type":"cytotoxicity"})).await?;
                }
                assert_eq!(guest.command("TOXICOLOGIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                analyse(&guest, "LEAD_SCIENTIST", "/service/run_mass_spec_analysis/reference_methods", json!({"sample":"reference_methods"})).await?;
                assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/bioactivity").await?["assay"]["success"],true);
                assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/toxicity").await?["toxicity"]["success"],true);
                write(&guest,"LEAD_SCIENTIST","/service/run_hplc_analysis/structural_confirmation",json!({"sample":"structural_confirmation"})).await?;
                write(&guest,"LEAD_SCIENTIST","/service/run_mass_spec_analysis/characterization",json!({"sample":"characterization"})).await?;
                assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/run_hplc_analysis/structural_confirmation").await?["success"],true);
                assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/run_mass_spec_analysis/characterization").await?["success"],true);
                for revision in 0..=rejections {
                    let go = revision % 2 == 0;
                    write(&guest,"LEAD_SCIENTIST","/service/submit_decision",json!({"go":go})).await?;
                    assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/submit_decision").await?["data"]["go"],go);
                    let review = read(&guest,"LEAD_SCIENTIST","/service/review").await?;
                    if revision == rejections {
                        assert_eq!(review["success"],true);
                        break;
                    }
                    assert_eq!(review["data"]["status"],"injected_failure");
                    write(&guest,"LEAD_SCIENTIST","/service/reanalysis",selected.clone()).await?;
                    if selected["biology"] == true {
                        assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/bioactivity").await?["assay"]["success"],true);
                    }
                    if selected["toxicology"] == true {
                        assert_eq!(read(&guest,"LEAD_SCIENTIST","/service/toxicity").await?["toxicity"]["success"],true);
                    }
                    if plan == 0 {
                        let reply = guest.command("LEAD_SCIENTIST",
                            "echo '{\"sample\":\"structural_confirmation\"}' > /service/run_hplc_analysis/structural_confirmation; cat /service/run_hplc_analysis/structural_confirmation".into()).await?;
                        assert_eq!(serde_json::Deserializer::from_str(reply["stdout"].as_str().ok_or("structural reply")?)
                            .into_iter::<Value>().next().ok_or("structural result")??["success"], true);
                    } else {
                        analyse(&guest, "LEAD_SCIENTIST", "/service/run_hplc_analysis/structural_confirmation", json!({"sample":"structural_confirmation"})).await?;
                    }
                }
                assert_eq!(guest.command("LEAD_SCIENTIST", "exit 0".into()).await?["exit_status"], 0);
            };
            async {
                for revision in 0..=rejections {
                    let decision = read(&guest,"REGULATORY_SPECIALIST","/service/decision").await?;
                    assert_eq!(decision["data"]["go"],revision % 2 == 0);
                    write(&guest,"REGULATORY_SPECIALIST","/service/verify_regulatory_compliance",json!({"decision":"go_no_go"})).await?;
                    let review = read(&guest,"REGULATORY_SPECIALIST","/service/verify_regulatory_compliance").await?;
                    assert_eq!(review["success"],revision == rejections);
                }
                assert_eq!(guest.command("REGULATORY_SPECIALIST", "exit 0".into()).await?["exit_status"], 0);
            };
            Ok(())
        };
        async { runtime.await };
    }, async {
        hibana_runtime::host::sleep(Duration::from_secs(50)).await?;
        Err(format!("14M actual Wasm deadline, condition {condition}").into())
    }).await?;
    save(&output.join("runtime-result.json"), &outcome)?;
    assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
    assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
    assert_eq!(outcome["score"]["complete"], true, "{outcome}");
    assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
    let effects = outcome["score"]["events"]
        .as_array()
        .ok_or("14M effect journal")?;
    assert_eq!(effects.len(), effect_count);
    let audit = crate::journal::Evidence::open(&output.join("events.jsonl"))?
        .audit(crate::regulatory_discovery_audit::check)?;
    assert_eq!(audit["domain_calls"], effect_count);
    assert_eq!(audit["rejections"], rejections);
    assert_eq!(audit["analysis_failures"], analysis_failures);
    assert_eq!(
        audit["reanalysis_choices"],
        json!(vec![selected; rejections])
    );
    assert!(
        audit["independent_effect_overlaps"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    save(&output.join("audit.json"), &audit)?;
    Ok(())
}
