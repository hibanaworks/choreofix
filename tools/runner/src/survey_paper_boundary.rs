//! Drive the seven actual wash guests and their common environment.
use crate::{
    drug_discovery_boundary::{read, write},
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection, survey_paper_audit,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[hibana_runtime::main]
#[test]
#[ignore = "requires the 3H executable, build receipt and wash Wasm"]
async fn seven_wasm_guests_publish_the_current_survey() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_SURVEY_PAPER_WASI_BINARY").ok_or("3H binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_SURVEY_PAPER_BUILD_RECEIPT").ok_or("3H build receipt")?,
    );
    let output =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("3H evidence root")?);
    let depth: u8 = std::env::var("CHOREOFIX_SURVEY_PAPER_DEPTH")?.parse()?;
    let directory = output.join(format!("survey-paper-wasi-{depth}"));
    fs::create_dir_all(&output)?;
    fs::create_dir(&directory)?;
    let build = archive_build_receipt(&binary, &receipt, &directory)?;
    projection::archive(&binary, &directory, "3H")?;
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    save(
        &directory.join("case.json"),
        &json!({"task":"3H","depth":depth,"delay_multiplier":0.1,"model_calls":0,
        "scored_benchmark":false,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
        "build":build,"compiled_projection_sha256":digest(&directory.join("projection.json"))?}),
    )?;
    let events = directory.join("events.jsonl");
    let trace = Trace::create(&events)?;
    let (simulator, fixture) = oracle::Simulator::new("3H", depth, 0.10, trace.clone(), &[])?;
    save(&directory.join("fixture.json"), &fixture)?;
    let (guest, runtime) = process::guest(&binary, &module, &[], &directory, trace, simulator)?;

    let ((), (), (), (), outcome) = hibana_runtime::race(async {
        async {
            let guest = &guest;
            for (agent, topic, section, references_first) in [
                ("RESEARCHER_A","subtopic_a","section_a",false),
                ("RESEARCHER_B","subtopic_b","section_b",true),
                ("RESEARCHER_C","subtopic_c","section_c",true),
                ("RESEARCHER_D","subtopic_d","section_d",false),
            ] {
                async move {
                    write(guest, agent, "/service/research_topic", json!({"topic":topic})).await?;
                    assert_eq!(read(guest, agent, "/service/research_topic").await?["data"]["topic"], topic);
                    let steps = [("write_section",json!({"section_name":section,"content_notes":format!("Primary sources and a taxonomy of {topic}.")})),
                                 ("update_references",json!({"section_name":section}))];
                    for index in if references_first { [1,0] } else { [0,1] } {
                        let (tool, args) = &steps[index];
                        let path = format!("/service/{tool}");
                        write(guest, agent, &path, args.clone()).await?;
                        assert_eq!(read(guest, agent, &path).await?["success"],true);
                    }
                    loop {
                        let review = read(guest, agent, "/service/review_result").await?;
                        assert_eq!(review["data"]["scope"],section);
                        if review["success"] == true {
                            break;
                        }
                        let feedback = format!("Address evidence limits in {}",review["data"]["review_id"]);
                        let path = "/service/revise_section";
                        write(guest, agent, path, json!({"section_name":section,"feedback":feedback})).await?;
                        assert_eq!(read(guest, agent, path).await?["data"]["artifact"]["feedback"],feedback);
                    }
                    let denied = guest.command(agent, format!("echo '{{\"section_name\":\"{section}\",\"feedback\":\"late edit\"}}' > /service/revise_section")).await?;
                    assert!(denied["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")), "{denied}");
                    assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                };
            }
            Ok(())
        };
        async {
            let agent = "DATA_ANALYST";
            let sections = if depth == 0 {["section_c","section_a","section_d","section_b"]}
                           else {["section_b","section_d","section_a","section_c"]};
            for section in sections {
                let base = format!("/service/{section}");
                let research = read(&guest,agent,&format!("{base}/research")).await?;
                assert_eq!(research["success"],true);
                let path = format!("{base}/create_figure");
                write(&guest,agent,&path,json!({"description":format!("Evidence categories and gaps for {section}")})).await?;
                let figure = read(&guest,agent,&path).await?;
                assert_eq!(figure["success"],true);
                let input = read(&guest,agent,&format!("{base}/insertion_input")).await?;
                assert_eq!(input["figures"][section],figure["data"]["artifact"]);
                let path = format!("{base}/insert_figure");
                write(&guest,agent,&path,json!({"section_name":section,"figure_id":figure["data"]["figure_id"]})).await?;
                assert_eq!(read(&guest,agent,&path).await?["data"]["artifact"],figure["data"]["artifact"]);
            }
            assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
        };
        async {
            let agent = "REVIEWER";
            let sections = if depth == 0 {["section_c","section_a","section_d","section_b"]}
                           else {["section_b","section_d","section_a","section_c"]};
            for section in sections {
                let base = format!("/service/{section}");
                let path = format!("{base}/peer_review");
                loop {
                    let input = read(&guest,agent,&format!("{base}/review_input")).await?;
                    write(&guest,agent,&path,json!({"manuscript":section})).await?;
                    let result = read(&guest,agent,&path).await?;
                    assert_eq!(result["data"]["input_snapshot"],input);
                    if result["success"] == true {
                        break;
                    }
                }
            }
            assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
        };
        async {
            let agent = "EDITOR_IN_CHIEF";
            let input = read(&guest,agent,"/service/accepted_sections").await?;
            let path = "/service/make_editorial_decision";
            write(&guest,agent,path,json!({"review_summary":"Use the approved four-section evidence and bibliography."})).await?;
            let decision = read(&guest,agent,path).await?;
            assert_eq!(decision["data"]["input_snapshot"],input["input_snapshot"]);
            let path = "/service/combine_sections";
            write(&guest,agent,path,json!({"sections":"section_d,section_a, section_c, section_b"})).await?;
            let result = read(&guest,agent,path).await?;
            assert_eq!(result["data"]["editorial_receipts"]["finalize"],decision["data"]["decision_id"]);
            assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
        };
        async { runtime.await };
    }, async {
        hibana_runtime::host::sleep(Duration::from_secs(55)).await?;
        Err("3H actual-Wasm deadline".into())
    }).await?;
    save(&directory.join("runtime-result.json"), &outcome)?;
    assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
    assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
    assert_eq!(outcome["score"]["complete"], true, "{outcome}");
    assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
    assert_eq!(
        outcome["score"]["events"]
            .as_array()
            .ok_or("3H effects")?
            .len(),
        26 + 2 * usize::from(depth)
    );
    save(
        &directory.join("effect-audit.json"),
        &survey_paper_audit::audit(&events)?,
    )?;
    Ok(())
}
