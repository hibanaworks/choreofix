use crate::{
    drug_discovery_boundary::{read, write},
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 3M executable, build receipt and wash Wasm"]
async fn actual_wasm_publishes_three_independent_chapters() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_RESEARCH_PAPER_WASI_BINARY").ok_or("3M binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_RESEARCH_PAPER_BUILD_RECEIPT").ok_or("3M build receipt")?,
    );
    let condition = std::env::var("CHOREOFIX_RESEARCH_PAPER_CASE")?.parse::<usize>()?;
    let directory =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("3M evidence root")?)
            .join(format!("research-paper-wasi-{condition}"));
    let failures: &[(&str, u32)] = match condition {
        0 => &[],
        1 => &[("fact_check", 0), ("review_sections", 0)],
        _ => return Err("unknown 3M case".into()),
    };
    fs::create_dir_all(&directory)?;
    let build = archive_build_receipt(&binary, &receipt, &directory)?;
    projection::archive(&binary, &directory, "3M")?;
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    save(
        &directory.join("case.json"),
        &json!({
            "task":"3M", "condition":condition, "depth":0, "delay_multiplier":0.10, "model_calls":0, "scored_benchmark":false,
            "failure_overrides":failures,
            "runtime_binary_sha256":digest(&binary)?, "guest_wasm_sha256":digest(&module)?, "build":build,
            "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
        }),
    )?;
    let events = directory.join("events.jsonl");
    let trace = Trace::create(&events)?;
    let (simulator, fixture) = oracle::Simulator::new("3M", 0, 0.10, trace.clone(), failures)?;
    save(&directory.join("fixture.json"), &fixture)?;
    let (guest, runtime) = process::guest(&binary, &module, &[], &directory, trace, simulator)?;
    let ((), (), (), outcome) = hibana_runtime::race(async {
        async {
            let guest = &guest;
            for (agent, topic, section) in [
                ("RESEARCHER_A", "subtopic_a", "section_a"),
                ("RESEARCHER_B", "subtopic_b", "section_b"),
                ("RESEARCHER_C", "subtopic_c", "section_c"),
            ] {
                async move {
                    if !failures.is_empty() && section == "section_b" {
                        for topic in ["reference provenance", "alternative survey methods"] {
                            let path = "/service/supplemental_research";
                            write(guest, agent, path, json!({"topic":topic})).await?;
                            let supplemental = read(guest, agent, path).await?;
                            assert_eq!(supplemental["success"], true);
                            assert_eq!(supplemental["data"]["topic"], topic);
                        }
                    }
                    write(guest, agent, "/service/research_topic", json!({"topic":topic})).await?;
                    let research = read(guest, agent, "/service/research_topic").await?;
                    assert_eq!(research["data"]["topic"], topic);
                    if !failures.is_empty() && section == "section_b" {
                        let path = "/service/supplemental_research";
                        let topic = "cross-check after assigned research";
                        write(guest, agent, path, json!({"topic":topic})).await?;
                        assert_eq!(read(guest, agent, path).await?["data"]["topic"], topic);
                    }
                    let draft = json!({"section_name":section,"content_notes":format!("Evidence gathered for {topic}.")});
                    let references = json!({"section_name":section});
                    if section == "section_b" {
                        write(guest, agent, "/service/update_references", references).await?;
                        write(guest, agent, "/service/write_section", draft).await?;
                    } else {
                        write(guest, agent, "/service/write_section", draft).await?;
                        write(guest, agent, "/service/update_references", references).await?;
                    }
                    assert_eq!(read(guest, agent, "/service/write_section").await?["success"], true);
                    assert_eq!(read(guest, agent, "/service/update_references").await?["success"], true);
                    loop {
                        let actual = read(guest, agent, "/service/review").await?;
                        if actual["success"] == true {
                            break;
                        }
                        let tool = actual["tool_name"].as_str().ok_or("rejection source")?;
                        assert!(matches!(tool, "fact_check" | "review_sections"));
                        let feedback = format!("Revise this draft in light of the {tool} result.");
                        write(guest, agent, "/service/revise_section", json!({"section_name":section,"feedback":feedback})).await?;
                        assert_eq!(read(guest, agent, "/service/revise_section").await?["data"]["artifact"]["feedback"], feedback);
                    }
                    let denied = guest.command(agent, format!(
                        "echo '{{\"section_name\":\"{section}\",\"feedback\":\"late edit\"}}' > /service/revise_section"
                    )).await?;
                    assert_eq!(denied["stdout"], "wash: close output /service/revise_section: Permission denied (os error 2)\nwash> ");
                    assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                };
            }
            Ok(())
        };
        async {
            for section in ["section_b", "section_a", "section_c"] {
                let path = format!("/service/fact_check/{section}");
                // The fixture injects an editorial rejection on its first
                // chapter, so that chapter needs two positive fact checks.
                let mut approvals = 1 + usize::from(!failures.is_empty() && section == "section_b");
                while approvals != 0 {
                    let snapshot = read(&guest, "FACT_CHECKER", &format!("/service/section/{section}")).await?;
                    write(&guest, "FACT_CHECKER", &path, json!({"section_name":section})).await?;
                    let actual = read(&guest, "FACT_CHECKER", &path).await?;
                    assert_eq!(actual["data"]["input_snapshot"], snapshot);
                    approvals -= usize::from(actual["success"] == true);
                }
            }
            for section in ["section_c", "section_a", "section_b"] {
                let actual = read(&guest, "FACT_CHECKER", &format!("/service/approved/{section}")).await?;
                assert_eq!(actual["success"], true);
            }
            assert_eq!(guest.command("FACT_CHECKER", "exit 0".into()).await?["exit_status"], 0);
        };
        async {
            for section in ["section_b", "section_a", "section_c"] {
                let path = format!("/service/review_sections/{section}");
                loop {
                    let material = read(&guest, "EDITOR", &format!("/service/section/{section}")).await?;
                    write(&guest, "EDITOR", &path, json!({"section_name":section})).await?;
                    let actual = read(&guest, "EDITOR", &path).await?;
                    assert_eq!(actual["data"]["input_snapshot"], material["snapshot"]);
                    if actual["success"] == true {
                        break;
                    }
                }
            }
            write(&guest, "EDITOR", "/service/combine_sections", json!({"sections":"section_a,section_b,section_c"})).await?;
            assert_eq!(read(&guest, "EDITOR", "/service/combine_sections").await?["success"], true);
            assert_eq!(guest.command("EDITOR", "exit 0".into()).await?["exit_status"], 0);
        };
        async { runtime.await };
    }, async {
        hibana_runtime::host::sleep(Duration::from_secs(55)).await?;
        Err("3M actual-Wasm deadline".into())
    }).await?;
    save(&directory.join("runtime-result.json"), &outcome)?;
    assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
    assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
    assert_eq!(outcome["score"]["complete"], true, "{outcome}");
    assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
    assert_eq!(
        outcome["score"]["events"]
            .as_array()
            .ok_or("3M effects")?
            .len(),
        if failures.is_empty() { 16 } else { 24 }
    );
    let audit = crate::journal::Evidence::open(&events)?.audit("3M")?;
    assert_eq!(audit["status"], "passed");
    save(&directory.join("effect-audit.json"), &audit)?;
    Ok(())
}
