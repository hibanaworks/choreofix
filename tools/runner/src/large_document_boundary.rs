//! Run the actual 7H preparation guests and correlate their effects.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    journal::{self, one},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[hibana_runtime::main]
#[test]
#[ignore = "requires the versioned 7H preparation executable and wash Wasm"]
async fn chapter_reviews() -> Result<()> {
    run_cases(
        "chapter_reviews",
        [
            (
                "repeated-major-before-part-i",
                0,
                false,
                [0, 0, 0, 0, 0],
                &[][..],
                false,
            ),
            (
                "major-then-accept",
                0,
                false,
                [0, 0, 0, 0, 0],
                &[][..],
                false,
            ),
            ("depth-2", 2, false, [0, 0, 0, 0, 0], &[][..], false),
            ("depth-1", 1, false, [0, 0, 0, 0, 0], &[][..], false),
            (
                "part-ii-fact-and-major-repair",
                0,
                true,
                [0, 0, 0, 0, 0],
                &["CHAPTER_2"][..],
                false,
            ),
            ("published", 0, false, [0, 0, 0, 0, 0], &[][..], false),
        ],
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the versioned 7H preparation executable and wash Wasm"]
async fn editorial_reviews() -> Result<()> {
    run_cases(
        "editorial_reviews",
        [
            (
                "part-ii-minor",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_3", "CHAPTER_4"][..],
                false,
            ),
            (
                "part-ii-amendment-fact-repair",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_3", "CHAPTER_4"][..],
                true,
            ),
            (
                "editorial-repair",
                0,
                false,
                [2, 0, 0, 0, 0],
                &[][..],
                false,
            ),
            (
                "editorial-reassessment",
                0,
                false,
                [1, 0, 0, 1, 1],
                &[][..],
                false,
            ),
            ("fact-repair", 0, false, [0, 2, 0, 0, 0], &[][..], false),
            ("review-repair", 0, false, [0, 0, 2, 0, 0], &[][..], false),
        ],
    )
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the versioned 7H preparation executable and wash Wasm"]
async fn manuscript_amendments() -> Result<()> {
    run_cases(
        "manuscript_amendments",
        [
            (
                "minor-first",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_1"][..],
                false,
            ),
            (
                "minor-both-forward",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_1", "CHAPTER_2"][..],
                false,
            ),
            (
                "major-then-two-amendments",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_1", "CHAPTER_2"][..],
                false,
            ),
            (
                "part-i-amendment-fact-repair",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_1", "CHAPTER_2"][..],
                false,
            ),
            (
                "minor-both-reverse",
                0,
                false,
                [0, 0, 0, 0, 0],
                &["CHAPTER_2", "CHAPTER_1"][..],
                false,
            ),
            (
                "repeated-major-nonblocking-review",
                0,
                false,
                [0, 0, 0, 0, 0],
                &[][..],
                false,
            ),
        ],
    )
    .await
}

type DocumentCase<'a> = (&'a str, u8, bool, [u32; 5], &'a [&'a str], bool);

async fn run_cases(group: &str, cases: [DocumentCase<'_>; 6]) -> Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(workspace)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("7H evidence root")?);
    let base = evidence.join("large-document-wasi");
    fs::create_dir_all(&base)?;
    let root = base.join(group);
    fs::create_dir(&root)?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_LARGE_DOCUMENT_WASI_BINARY").ok_or("7H executable")?,
    );
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_LARGE_DOCUMENT_BUILD_RECEIPT").ok_or("7H receipt")?,
    );
    fs::copy(&runtime, root.join("runtime"))?;
    fs::copy(
        "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
        root.join("wash.wasm"),
    )?;
    projection::archive(&root.join("runtime"), &root, "7H")?;
    let build = archive_build_receipt(&root.join("runtime"), &receipt, &root)?;
    let runtime_sha = digest(&root.join("runtime"))?;
    let mut reports = Vec::with_capacity(cases.len());
    for (case, depth, repair_part_ii, failures, minor_sections, reject_amended_part_ii) in cases {
        let audit = publication_case(
            &root,
            case,
            depth,
            repair_part_ii,
            failures,
            minor_sections,
            reject_amended_part_ii,
        )
        .await?;
        let audit_path = root.join(case).join("preparation-audit.json");
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["runtime_binary_sha256"], runtime_sha);
        assert_eq!(audit["simulator_complete"], true);
        assert_eq!(audit["violations"], 0);
        reports.push(json!({
            "case": case, "depth": depth, "effects":audit["effects"],
            "captured_records":audit["captured_records"],
            "events_sha256":audit["events_sha256"], "audit_sha256":digest(&audit_path)?
        }));
    }
    save(
        &root.join("boundary.json"),
        &json!({
            "task":"7H", "status":"passed", "build":build, "runtime_binary_sha256":runtime_sha,
            "wasm_sha256":digest(&root.join("wash.wasm"))?,
            "cases":reports, "model_calls":0, "scored_benchmark":false
        }),
    )
}

#[hibana_runtime::scope]
async fn publication_case(
    root: &Path,
    case: &str,
    depth: u8,
    repair_part_ii: bool,
    [
        editorial_failures,
        fact_failures,
        review_failures,
        post_edit_fact,
        post_edit_review,
    ]: [u32; 5],
    minor_sections: &[&str],
    reject_amended_part_ii: bool,
) -> Result<Value> {
    let (part_ii_minor, minor_sections): (Vec<_>, Vec<_>) = minor_sections
        .iter()
        .copied()
        .partition(|chapter| matches!(*chapter, "CHAPTER_3" | "CHAPTER_4"));
    let reject_amended_part_i = case == "part-i-amendment-fact-repair";
    let repeated_major = matches!(
        case,
        "repeated-major-before-part-i" | "repeated-major-nonblocking-review"
    );
    let major_then_accept = case == "major-then-accept" || repeated_major;
    let major_first = major_then_accept || case == "major-then-two-amendments";
    let part_ii_major_reviews_per_chapter = if repeated_major {
        2
    } else {
        usize::from(repair_part_ii)
    };
    let part_i_feedback = "Explain “scope” in the opening claim.\nIdentify the evidence.";
    let part_i_review_rejections = review_failures as usize + usize::from(major_first);
    let output = root.join(case);
    fs::create_dir(&output)?;
    let runtime = output.join("runtime");
    let wash = output.join("wash.wasm");
    fs::hard_link(root.join("runtime"), &runtime)?;
    fs::hard_link(root.join("wash.wasm"), &wash)?;
    for artifact in [
        "projection.json",
        "build-source.sha256",
        "input-build-receipt.json",
        "build-receipt.json",
    ] {
        fs::hard_link(root.join(artifact), output.join(artifact))?;
    }
    save(
        &output.join("case.json"),
        &json!({"task":"7H","kind":"scripted_result_driven_recovery",
        "runtime_binary_sha256":digest(&runtime)?,"wash_sha256":digest(&wash)?,
        "build_receipt_sha256":digest(&output.join("build-receipt.json"))?,
        "harness_binary_sha256":digest(&std::env::current_exe()?)?,
        "harness_source_sha256":digest(Path::new("tools/runner/src/large_document_boundary.rs"))?,
        "compiled_projection_sha256":digest(&output.join("projection.json"))?,
        "simulator_manifest_sha256":digest(Path::new("benchmark/simulator-manifest.json"))?,
        "model_calls":0,"scored_benchmark":false,
        "depth":depth,"repair_part_ii":repair_part_ii,"injected_editorial_failures":editorial_failures,
        "injected_part_i_fact_failures":fact_failures,"injected_part_i_review_failures":review_failures,
        "post_edit_fact_failures":post_edit_fact,"post_edit_review_failures":post_edit_review,
        "reviewer_requests_major_first":major_first,
        "part_ii_major_reviews_per_chapter":part_ii_major_reviews_per_chapter,
        "requested_part_i_minor_edits":minor_sections,"requested_part_ii_minor_edits":part_ii_minor,
        "reject_amended_part_ii":reject_amended_part_ii,"reject_amended_part_i":reject_amended_part_i}),
    )?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let part_ii_fact_rejections = 2 * usize::from(repair_part_ii) + depth as usize;
    let part_ii_review_rejections = 2 * part_ii_major_reviews_per_chapter + depth as usize;
    let first_part_i_fact = 2
        + part_ii_fact_rejections as u32
        + part_ii_review_rejections as u32
        + part_ii_minor.len() as u32;
    let first_part_i_review = 2 + part_ii_review_rejections as u32;
    let retries = editorial_failures as usize + depth as usize;
    let first_post_edit_fact = first_part_i_fact
        + fact_failures
        + review_failures
        + 1
        + u32::from(!minor_sections.is_empty());
    let first_post_edit_review = first_part_i_review + review_failures + 1;
    let failures: Vec<_> = (0..editorial_failures)
        .map(|attempt| ("make_editorial_decision", attempt))
        .chain(
            (first_part_i_fact..first_part_i_fact + fact_failures)
                .map(|attempt| ("fact_check", attempt)),
        )
        .chain(
            (first_part_i_review..first_part_i_review + review_failures)
                .map(|attempt| ("review_sections", attempt)),
        )
        .chain(
            (first_post_edit_fact..first_post_edit_fact + post_edit_fact)
                .map(|attempt| ("fact_check", attempt)),
        )
        .chain(
            (first_post_edit_review..first_post_edit_review + post_edit_review)
                .map(|attempt| ("review_sections", attempt)),
        )
        .chain(
            reject_amended_part_ii
                .then_some([("fact_check", 1), ("fact_check", 4)])
                .into_iter()
                .flatten(),
        )
        .chain(reject_amended_part_i.then_some(("fact_check", first_part_i_fact + 1)))
        .chain(
            repair_part_ii
                .then_some([("fact_check", 0), ("fact_check", 3)])
                .into_iter()
                .flatten(),
        )
        .collect();
    let (simulator, fixture) = oracle::Simulator::new("7H", depth, 0.05, trace.clone(), &failures)?;
    let (guest, finish) = process::guest(&runtime, &wash, &["7H"], &output, trace, simulator)?;
    let (result, (), (), (), ()) = hibana_runtime::race(async {
        let response = |reply: Value| -> Result<Value> {
            let text = reply["stdout"].as_str().ok_or("wash output")?;
            let start = text.find('{').ok_or_else(|| format!("missing response: {text}"))?;
            Ok(serde_json::Deserializer::from_str(&text[start..]).into_iter::<Value>()
                .next().ok_or("empty response")??)
        };
        async { finish.await };
        let first = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_1","content_notes":"Introduction framing the stable contribution and evaluation"}' > /service/write_section/CHAPTER_1; cat /service/write_section/CHAPTER_1"#.into()).await?)?;
        assert!(first["success"] == true);
        async {
            let combine = r#"{"sections":"CHAPTER_4,CHAPTER_1,CHAPTER_3,CHAPTER_2"}"#;
            let premature = guest.command("EDITOR_IN_CHIEF", format!(
                "echo '{combine}' | dd of=/service/combine_sections oflag=nonblock"
            )).await?;
            let text = premature["stdout"].as_str().ok_or("premature publication")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
            let mut previous = Value::Null;
            loop {
                let material = response(guest.command("EDITOR_IN_CHIEF", "cat /service/material".into()).await?)?;
                let reviews = material["reviews"].as_array().ok_or("three scoped reviews")?;
                assert_eq!(reviews.len(), 3);
                assert!(reviews.iter().all(|review| review["success"] == true
                    || (review["data"]["status"] == "minor_revision" && review["data"]["scope"].as_str().is_some_and(|scope| material["checked_minor"][scope]["success"] == true))));
                if !previous.is_null() {
                    let old = &previous["input_snapshot"]["sections"];
                    let current = &material["input_snapshot"]["sections"];
                    for chapter in ["CHAPTER_3", "CHAPTER_4"] {
                        assert_eq!(current[chapter], old[chapter]);
                    }
                    assert_eq!(["CHAPTER_1", "CHAPTER_2"].into_iter()
                        .filter(|chapter| current[chapter]["content"] != old[chapter]["content"])
                        .count(), if post_edit_fact + post_edit_review == 0 { 1 } else { 2 });
                }
                let review_id = material["review_receipts"]["CHAPTERS_1_2"].as_str().ok_or("Part I review")?;
                let args = json!({"review_summary":format!("Checked current manuscript and Part I review {review_id}")});
                let decision = response(guest.command("EDITOR_IN_CHIEF", format!(
                    "echo '{args}' > /service/make_editorial_decision; cat /service/make_editorial_decision"
                )).await?)?;
                for field in ["input_snapshot","review_receipts"] {
                    assert_eq!(decision["data"][field], material[field]);
                }
                if decision["success"] == false {
                    let refused = guest.command("EDITOR_IN_CHIEF", format!(
                        "echo '{combine}' | dd of=/service/combine_sections oflag=nonblock"
                    )).await?;
                    let text = refused["stdout"].as_str().ok_or("publication after rejection")?;
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                    previous = material;
                    continue;
                }
                assert_eq!(decision["success"], true);
                let malformed = guest.command("EDITOR_IN_CHIEF", r#"echo '{"sections":"CHAPTER_1,CHAPTER_2,CHAPTER_4,CHAPTER_4"}' > /service/combine_sections"#.into()).await?;
                let text = malformed["stdout"].as_str().ok_or("wrong publication sections")?;
                assert!(text.contains("Invalid argument") || text.contains("Permission denied"), "{text}");
                let published = response(guest.command("EDITOR_IN_CHIEF", format!("echo '{combine}' > /service/combine_sections; cat /service/combine_sections")).await?)?;
                for field in ["input_snapshot","review_receipts","fact_receipts"] {
                    assert_eq!(published["data"][field], material[field]);
                }
                assert_eq!(published["data"]["editorial_receipts"]["final"], decision["data"]["decision_id"]);
                break;
            }
            assert_eq!(guest.command("EDITOR_IN_CHIEF", "exit 0".into()).await?["exit_status"], 0);
            Ok(())
        };
        async {
            let early = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_2","content_notes":"Early background"}' > /service/write_section/CHAPTER_2; cat /service/write_section/CHAPTER_2"#.into()).await?)?;
            assert_eq!(early["data"]["artifact"]["content"], "Early background");
            let refused = guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_2","feedback":"Unfounded revision"}' | dd of=/service/finalize_section/CHAPTER_2 oflag=nonblock"#.into()).await?;
            let text = refused["stdout"].as_str().ok_or("premature Part I submission")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
            let first = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_1"}' > /service/update_references/CHAPTER_1; cat /service/update_references/CHAPTER_1"#.into()).await?)?;
            assert_eq!(first["success"], true);
            let contribution = response(guest.command("WRITER_A", "cat /service/contribution".into()).await?)?;
            let evaluation = response(guest.command("WRITER_A", "cat /service/evaluation".into()).await?)?;
            for (chapter, artifact, original) in [
                ("CHAPTER_3", &contribution, "Stable contribution"),
                ("CHAPTER_4", &evaluation, "Stable evaluation"),
            ] {
                if part_ii_minor.contains(&chapter) {
                    let amended = format!("Clarified {chapter}");
                    if reject_amended_part_ii {
                        let content = artifact["content"].as_str().ok_or("corrected chapter content")?;
                        assert!(content.starts_with(&amended));
                        assert_ne!(content, amended);
                    } else {
                        assert_eq!(artifact["content"], amended);
                        assert!(artifact["minor_review_id"].is_string());
                    }
                } else {
                    assert!(artifact["content"].as_str().ok_or("current chapter content")?.starts_with(original));
                }
            }
            let second = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_2"}' > /service/update_references/CHAPTER_2; cat /service/update_references/CHAPTER_2"#.into()).await?)?;
            assert_eq!(second["success"], true);
            let finalized = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_2","feedback":"Relate the new evaluation to the contribution"}' > /service/finalize_section/CHAPTER_2; cat /service/finalize_section/CHAPTER_2"#.into()).await?)?;
            assert_ne!(finalized["id"], early["data"]["artifact"]["id"]);
            let inputs = json!({"CHAPTER_3":contribution,"CHAPTER_4":evaluation});
            assert_eq!(finalized["input_sections"], inputs);
            assert!(finalized["content"].as_str().ok_or("final background")?.contains("Relate the new evaluation to the contribution"));
            let retained = response(guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_1"}' > /service/finalize_section/CHAPTER_1; cat /service/finalize_section/CHAPTER_1"#.into()).await?)?;
            assert_eq!(retained["input_sections"], inputs);
            loop {
                let outcome = response(guest.command("WRITER_A", "cat /service/editorial_outcome".into()).await?)?;
                if outcome["tool_name"] == "combine_sections" {
                    assert_eq!(outcome["data"]["status"], "combined");
                    break;
                }
                if outcome["data"]["status"] == "minor_revision" {
                    let review = &outcome["data"];
                    let edits = review["minor_replacements"].as_object().ok_or("exact minor changes")?;
                    assert_eq!(edits.len(), minor_sections.len());
                    for chapter in &minor_sections {
                        let expected = edits.get(*chapter).ok_or("requested minor change")?;
                        let forbidden = json!({"section_name":chapter,"feedback":"unreviewed replacement"});
                        let refused = guest.command("WRITER_A", format!(
                            "echo '{forbidden}' > /service/apply_minor_edit/{chapter}"
                        )).await?;
                        let text = refused["stdout"].as_str().ok_or("unreviewed amendment reply")?;
                        assert!(text.contains("Invalid argument") || text.contains("Permission denied"), "{text}");
                        let args = json!({"section_name":chapter,"review_id":review["review_id"]});
                        let revised = response(guest.command("WRITER_A", format!(
                            "echo '{args}' > /service/apply_minor_edit/{chapter}; cat /service/apply_minor_edit/{chapter}"
                        )).await?)?;
                        assert_eq!(revised["success"], true);
                        let artifact = &revised["data"]["artifact"];
                        assert_eq!(artifact["content"], *expected);
                        assert_eq!(artifact["previous_id"], review["input_snapshot"]["sections"][chapter]["id"]);
                        assert_eq!(artifact["minor_review_id"], review["review_id"]);
                        if case == "minor-both-forward" && *chapter == "CHAPTER_1" {
                            let early = guest.command("WRITER_A", "dd if=/service/editorial_outcome iflag=nonblock".into()).await?;
                            let text = early["stdout"].as_str().ok_or("partial amendment assessment")?;
                            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                        }
                        if case == "major-then-two-amendments" && *chapter == "CHAPTER_1" {
                            let early = guest.command("WRITER_A", "cat /service/editorial_outcome".into()).await?;
                            let text = early["stdout"].as_str().ok_or("blocking read after first amendment")?;
                            assert!(text.contains("Permission denied"), "{text}");
                        }
                    }
                    continue;
                }
                if (outcome["tool_name"] == "review_sections" || outcome["tool_name"] == "fact_check")
                    && outcome["success"] == true
                {
                    continue;
                }
                assert!(matches!(outcome["tool_name"].as_str(), Some("make_editorial_decision" | "fact_check" | "review_sections")));
                assert_eq!(outcome["success"], false);
                if major_first && outcome["tool_name"] == "review_sections" {
                    assert_eq!(outcome["data"]["feedback"], part_i_feedback);
                }
                let sections = &outcome["data"]["input_snapshot"]["sections"];
                let introduction = sections["CHAPTER_1"]["content"].as_str().ok_or("rejected introduction")?;
                let (chapter, feedback) = if introduction.contains("Align the introduction terminology") {
                    ("CHAPTER_2", "Explain the relation between the background and evaluation")
                } else {
                    ("CHAPTER_1", "Align the introduction terminology with the contribution")
                };
                let receipt = ["decision_id", "check_id", "review_id"].into_iter()
                    .find_map(|key| outcome["data"][key].as_str()).ok_or("revision verdict receipt")?;
                let args = json!({"section_name":chapter,"feedback":format!("{feedback} ({receipt})")});
                let revised = response(guest.command("WRITER_A", format!(
                    "echo '{args}' > /service/revise_section/{chapter}; cat /service/revise_section/{chapter}"
                )).await?)?;
                assert_eq!(revised["success"], true);
                assert_ne!(revised["data"]["artifact"], sections[chapter]);
            }
            assert_eq!(guest.command("WRITER_A", "exit 0".into()).await?["exit_status"], 0);
            Ok(())
        };
        let refused = guest.command("WRITER_B", r#"echo '{"section_name":"CHAPTER_3"}' | dd of=/service/update_references/CHAPTER_3 oflag=nonblock"#.into()).await?;
        let text = refused["stdout"].as_str().ok_or("premature citation result")?;
        assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
        let refused = guest.command("FACT_CHECKER", r#"echo '{"section_name":"CHAPTER_3"}' | dd of=/service/fact_check/CHAPTER_3 oflag=nonblock"#.into()).await?;
        let text = refused["stdout"].as_str().ok_or("premature fact check")?;
        assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
        let refused = guest.command("REVIEWER", r#"echo '{"section_name":"CHAPTER_3"}' | dd of=/service/review_sections/CHAPTER_3 oflag=nonblock"#.into()).await?;
        let text = refused["stdout"].as_str().ok_or("premature review")?;
        assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
        let contribution = response(guest.command("WRITER_B", r#"echo '{"section_name":"CHAPTER_3","content_notes":"Stable contribution"}' > /service/write_section/CHAPTER_3; cat /service/write_section/CHAPTER_3"#.into()).await?)?;
        assert_eq!(contribution["success"], true);
        let created = response(guest.command("FIGURE_CREATOR", r#"echo '{"description":"Contribution diagram"}' > /service/create_figure/CHAPTER_3; cat /service/create_figure/CHAPTER_3"#.into()).await?)?;
        let figure_id = created["data"]["figure_id"].as_str().ok_or("created figure")?;
        let wrong = json!({"section_name":"CHAPTER_4","figure_id":figure_id});
        let refused = guest.command("FIGURE_CREATOR", format!("echo '{wrong}' > /service/insert_figure/CHAPTER_3")).await?;
        let text = refused["stdout"].as_str().ok_or("mismatched insertion result")?;
        assert!(text.contains("Invalid argument") || text.contains("Permission denied"), "{text}");
        let args = json!({"section_name":"CHAPTER_3","figure_id":figure_id});
        let inserted = response(guest.command("FIGURE_CREATOR", format!("echo '{args}' > /service/insert_figure/CHAPTER_3; cat /service/insert_figure/CHAPTER_3")).await?)?;
        assert_eq!(inserted["success"], true);
        assert_eq!(inserted["data"]["artifact"], created["data"]["artifact"]);
        let cited = response(guest.command("WRITER_C", r#"echo '{"section_name":"CHAPTER_4"}' > /service/update_references/CHAPTER_4; cat /service/update_references/CHAPTER_4"#.into()).await?)?;
        assert_eq!(cited["success"], true);
        let evaluation = response(guest.command("WRITER_C", r#"echo '{"section_name":"CHAPTER_4","content_notes":"Stable evaluation"}' > /service/write_section/CHAPTER_4; cat /service/write_section/CHAPTER_4"#.into()).await?)?;
        assert_eq!(evaluation["data"]["artifact"]["content"], "Stable evaluation");
        let created = response(guest.command("FIGURE_CREATOR", r#"echo '{"description":"Evaluation plot"}' > /service/create_figure/CHAPTER_4; cat /service/create_figure/CHAPTER_4"#.into()).await?)?;
        assert_eq!(created["success"], true);
        let figure_id = created["data"]["figure_id"].as_str().ok_or("created evaluation figure")?;
        let args = json!({"section_name":"CHAPTER_4","figure_id":figure_id});
        let inserted = response(guest.command("FIGURE_CREATOR", format!("echo '{args}' > /service/insert_figure/CHAPTER_4; cat /service/insert_figure/CHAPTER_4")).await?)?;
        assert_eq!(inserted["success"], true);
        assert_eq!(inserted["data"]["artifact"], created["data"]["artifact"]);
        {
            let created = response(guest.command("FIGURE_CREATOR", r#"echo '{"description":"Introduction schematic"}' > /service/create_figure/CHAPTER_1; cat /service/create_figure/CHAPTER_1"#.into()).await?)?;
            assert_eq!(created["success"], true);
            let figure_id = created["data"]["figure_id"].as_str().ok_or("created introduction figure")?;
            let args = json!({"section_name":"CHAPTER_1","figure_id":figure_id});
            let inserted = response(guest.command("FIGURE_CREATOR", format!("echo '{args}' > /service/insert_figure/CHAPTER_1; cat /service/insert_figure/CHAPTER_1")).await?)?;
            assert_eq!(inserted["success"], true);
            assert_eq!(inserted["data"]["artifact"], created["data"]["artifact"]);
            assert_eq!(guest.command("FIGURE_CREATOR", "exit 0".into()).await?["exit_status"], 0);
        }
        for (chapter, author) in [("CHAPTER_4", "WRITER_C"), ("CHAPTER_3", "WRITER_B")] {
            if chapter == "CHAPTER_3" {
                let stable = response(guest.command("WRITER_B", "cat /service/evaluation".into()).await?)?;
                let cited = response(guest.command("WRITER_B", r#"echo '{"section_name":"CHAPTER_3"}' > /service/update_references/CHAPTER_3; cat /service/update_references/CHAPTER_3"#.into()).await?)?;
                assert_eq!(cited["data"]["artifact"]["input_sections"]["CHAPTER_4"], stable);
                assert_eq!(stable["id"] != evaluation["data"]["artifact"]["id"], part_ii_major_reviews_per_chapter > 0 || depth > 0 || part_ii_minor.contains(&"CHAPTER_4"));
            }
            let early = json!({"section_name":chapter,"feedback":"Revision without a refusal"});
            let refused = guest.command(author, format!("echo '{early}' | dd of=/service/revise_section/{chapter} oflag=nonblock")).await?;
            let text = refused["stdout"].as_str().ok_or("early revision result")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
            let early = json!({"section_name":chapter,"review_id":"review:unissued"});
            let refused = guest.command(author, format!("echo '{early}' | dd of=/service/apply_minor_edit/{chapter} oflag=nonblock")).await?;
            let text = refused["stdout"].as_str().ok_or("early minor edit result")?;
            assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
            let mut preceding_revision = None;
            let mut decisions = std::iter::repeat_n("major", part_ii_major_reviews_per_chapter)
                .chain(part_ii_minor.contains(&chapter).then_some("minor"))
                .chain(std::iter::repeat("accept"));
            loop {
                let snapshot = response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?;
                if case == "published" && chapter == "CHAPTER_3" {
                    let early = guest.command("FACT_CHECKER", "dd if=/service/inbox iflag=nonblock".into()).await?;
                    let text = early["stdout"].as_str().ok_or("early assessment read response")?;
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                }
                if let Some(revision) = &preceding_revision {
                    assert_eq!(&snapshot["sections"][chapter], revision);
                }
                let args = json!({"section_name":chapter});
                let checked = response(guest.command("FACT_CHECKER", format!("echo '{args}' > /service/fact_check/{chapter}; cat /service/fact_check/{chapter}")).await?)?;
                assert_eq!(checked["data"]["input_snapshot"], snapshot);
                let evidence = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
                assert_eq!(evidence, checked);
                let mut submitted_feedback = None;
                let outcome = if checked["success"] == true {
                    let decision = decisions.next().ok_or("test review decision")?;
                    let args = if decision == "minor" {
                        json!({"section_name":chapter, "decision":"minor", "minor_edits":[{
                            "section_name":chapter, "before":snapshot["sections"][chapter]["content"],
                            "after":format!("Clarified {chapter}"),
                        }]})
                    } else if decision == "major" {
                        submitted_feedback = Some(format!("Explain the evidence for {chapter}."));
                        json!({"section_name":chapter,"decision":decision,"feedback":submitted_feedback})
                    } else {
                        json!({"section_name":chapter,"decision":decision})
                    };
                    guest.command("REVIEWER", format!("echo '{args}' > /service/review_sections/{chapter}")).await?;
                    response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?
                } else {
                    assert_eq!(checked["data"]["status"], "needs_revision");
                    let args = json!({"section_name":chapter});
                    let refusal = guest.command("REVIEWER", format!(
                        "echo '{args}' | dd of=/service/review_sections/{chapter} oflag=nonblock"
                    )).await?;
                    let text = refusal["stdout"].as_str().ok_or("review of unchecked chapter")?;
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                    response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?
                };
                if checked["success"] == true {
                    assert_eq!(outcome["tool_name"], "review_sections");
                    assert_eq!(outcome["data"]["feedback"].as_str(), submitted_feedback.as_deref());
                    assert_eq!(outcome["data"]["fact_receipts"][chapter], checked["data"]["check_id"]);
                } else {
                    assert_eq!(outcome, checked);
                }
                assert_eq!(outcome["data"]["input_snapshot"], snapshot);
                for (participant, path) in [(author, "editorial_outcome"), ("FACT_CHECKER", "inbox")] {
                    let actual = response(guest.command(participant, format!("cat /service/{path}")).await?)?;
                    assert_eq!(actual, outcome);
                }
                if outcome["success"] == true {
                    break;
                }
                let outcome = if outcome["data"]["status"] == "minor_revision" {
                    let args = json!({"section_name":chapter,"review_id":outcome["data"]["review_id"]});
                    let amended = response(guest.command(author, format!(
                        "echo '{args}' > /service/apply_minor_edit/{chapter}; cat /service/apply_minor_edit/{chapter}"
                    )).await?)?;
                    assert_eq!(amended["success"], true);
                    assert_eq!(amended["data"]["artifact"]["content"], format!("Clarified {chapter}"));
                    assert_eq!(amended["data"]["artifact"]["previous_id"], snapshot["sections"][chapter]["id"]);
                    let amended_snapshot = response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?;
                    assert_eq!(amended_snapshot["sections"][chapter], amended["data"]["artifact"]);
                    let args = json!({"section_name":chapter});
                    let confirmed = response(guest.command("FACT_CHECKER", format!(
                        "echo '{args}' > /service/fact_check/{chapter}; cat /service/fact_check/{chapter}"
                    )).await?)?;
                    assert_eq!(confirmed["success"], !reject_amended_part_ii);
                    assert_eq!(confirmed["data"]["input_snapshot"], amended_snapshot);
                    assert_ne!(confirmed["data"]["check_id"], checked["data"]["check_id"]);
                    for (participant, file) in [(author, "editorial_outcome"), ("FACT_CHECKER", "inbox"), ("REVIEWER", "inbox")] {
                        let actual = response(guest.command(participant, format!("cat /service/{file}")).await?)?;
                        assert_eq!(actual, confirmed);
                    }
                    if confirmed["success"] == true {
                        break;
                    }
                    assert_eq!(confirmed["data"]["status"], "needs_revision");
                    confirmed
                } else {
                    outcome
                };
                let args = json!({"section_name":chapter,"feedback":format!("Correct {chapter} issue {}", outcome["data"]["review_id"].as_str()
                    .or(outcome["data"]["check_id"].as_str()).ok_or("rejected artifact receipt")?)});
                let revised = response(guest.command(author, format!("echo '{args}' > /service/revise_section/{chapter}; cat /service/revise_section/{chapter}")).await?)?;
                assert_eq!(revised["success"], true);
                let updated = revised["data"]["artifact"].clone();
                assert_ne!(updated["id"], snapshot["sections"][chapter]["id"]);
                preceding_revision = Some(updated);
            }
            assert_eq!(guest.command(author, "exit 0".into()).await?["exit_status"], 0);
        }
        async {
            loop {
                let snapshot = response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?;
                if major_then_accept
                    && snapshot["sections"]["CHAPTER_1"]["content"].as_str().ok_or("revised introduction")?
                        .contains("Align the introduction terminology")
                {
                    let early = guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?;
                    let text = early["stdout"].as_str().ok_or("blocking read before renewed fact check")?;
                    assert!(text.contains("Permission denied"), "{text}");
                }
                let checked = response(guest.command("FACT_CHECKER", r#"echo '{"section_name":"CHAPTERS_1_2"}' > /service/fact_check/CHAPTERS_1_2; cat /service/fact_check/CHAPTERS_1_2"#.into()).await?)?;
                assert_eq!(checked["data"]["input_snapshot"], snapshot);
                if checked["success"] == false {
                    assert_eq!(checked["data"]["status"], "needs_revision");
                    let outcome = response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?;
                    assert_eq!(outcome, checked);
                    continue;
                }
                assert_eq!(checked["success"], true);
                let outcome = response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?;
                let outcome = if outcome["tool_name"] == "review_sections" {
                    if matches!(outcome["data"]["status"].as_str(), Some("needs_revision" | "minor_revision")) {
                        continue;
                    }
                    response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?
                } else if outcome["tool_name"] == "fact_check" {
                    assert_eq!(outcome, checked);
                    response(guest.command("FACT_CHECKER", "cat /service/inbox".into()).await?)?
                } else {
                    outcome
                };
                if outcome["tool_name"] == "make_editorial_decision" {
                    assert_eq!(outcome["success"], false);
                    continue;
                }
                assert!(outcome["tool_name"] == "combine_sections" || outcome["reviews"].is_array());
                break;
            }
            assert_eq!(guest.command("FACT_CHECKER", "exit 0".into()).await?["exit_status"], 0);
            Ok(())
        };
        async {
          loop {
            let evidence = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
            if evidence["success"] == false {
                assert_eq!(evidence["tool_name"], "fact_check");
                let refused = guest.command("REVIEWER", r#"echo '{"section_name":"CHAPTERS_1_2"}' | dd of=/service/review_sections/CHAPTERS_1_2 oflag=nonblock"#.into()).await?;
                let text = refused["stdout"].as_str().ok_or("review of failed material")?;
                assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                let verdict = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
                assert_eq!(verdict, evidence);
                continue;
            }
            let request_major = major_first
                && !evidence["data"]["input_snapshot"]["sections"]["CHAPTER_1"]["content"]
                    .as_str().ok_or("reviewed introduction")?
                    .contains("Align the introduction terminology");
            if major_then_accept && !request_major {
                let probe = if case == "repeated-major-nonblocking-review" {
                    "dd if=/service/inbox iflag=nonblock"
                } else {
                    "cat /service/inbox"
                };
                let early = guest.command("REVIEWER", probe.into()).await?;
                let text = early["stdout"].as_str().ok_or("read before renewed review")?;
                if case == "repeated-major-nonblocking-review" {
                    assert!(text.contains("Permission denied") || text.contains("Resource temporarily unavailable"), "{text}");
                } else {
                    assert!(text.contains("Permission denied"), "{text}");
                }
            }
            let refused = guest.command("REVIEWER", r#"echo '{"section_name":"CHAPTERS_1_2","decision":"minor","minor_edits":[]}' > /service/review_sections/CHAPTERS_1_2"#.into()).await?;
            let text = refused["stdout"].as_str().ok_or("malformed minor edits response")?;
            assert!(text.contains("Invalid argument") || text.contains("Permission denied"), "{text}");
            let request_minor = !request_major && !minor_sections.is_empty()
                && minor_sections.iter().all(|chapter|
                    evidence["data"]["input_snapshot"]["sections"][chapter]["minor_review_id"].is_null());
            let args = if request_major { json!({"section_name":"CHAPTERS_1_2","decision":"major","feedback":part_i_feedback}) }
            else if major_then_accept { json!({"section_name":"CHAPTERS_1_2","decision":"accept"}) }
            else if !request_minor { json!({"section_name":"CHAPTERS_1_2"}) }
            else {
                let edits: Vec<_> = minor_sections.iter().map(|chapter| match *chapter {
                    "CHAPTER_1" => json!({"section_name":chapter,"before":"Introduction framing","after":"Introduction presenting"}),
                    "CHAPTER_2" => json!({"section_name":chapter,"before":"Early background","after":"Verified background"}),
                    _ => unreachable!("Part I review fixture section"),
                }).collect();
                json!({"section_name":"CHAPTERS_1_2","decision":"minor","minor_edits":edits})
            };
            guest.command("REVIEWER", format!("echo '{args}' > /service/review_sections/CHAPTERS_1_2")).await?;
            let reviewed = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
            if request_major {
                assert_eq!(reviewed["data"]["status"], "needs_revision");
                assert_eq!(reviewed["data"]["feedback"], part_i_feedback);
            } else if !request_minor {
                assert!(matches!(reviewed["data"]["status"].as_str(), Some("reviewed" | "needs_revision")));
            } else {
                assert_eq!(reviewed["data"]["status"], "minor_revision");
                assert_eq!(reviewed["success"], false);
            }
            assert_eq!(reviewed["data"]["input_snapshot"], evidence["data"]["input_snapshot"]);
            if reviewed["data"]["status"] == "needs_revision" {
                continue;
            }
            if request_minor {
                let confirmation = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
                assert_eq!(confirmation["tool_name"], "fact_check");
                assert_eq!(confirmation["success"], !reject_amended_part_i);
                if confirmation["success"] == false {
                    assert_eq!(confirmation["data"]["status"], "needs_revision");
                    continue;
                }
            }
            let outcome = response(guest.command("REVIEWER", "cat /service/inbox".into()).await?)?;
            if outcome["tool_name"] == "make_editorial_decision" {
                assert_eq!(outcome["success"], false);
                continue;
            }
            assert!(outcome["tool_name"] == "combine_sections" || outcome["reviews"].is_array());
            break;
          }
            assert_eq!(guest.command("REVIEWER", "exit 0".into()).await?["exit_status"], 0);
            Ok(())
        };
    }, hibana_runtime::host::deadline(Duration::from_secs(50))).await?;
    assert_eq!(result["error"], Value::Null);
    assert_eq!(result["protocol"]["status"], "complete");
    save(&output.join("runtime-result.json"), &result)?;

    let events = output.join("events.jsonl");
    let rows = journal::records(&events)?;
    let artifact = projection::load(&events)?;
    let compiled = &artifact["compiled"];
    let headers = projection::frames(&artifact, &rows)?;
    let dynamic = projection::check_dynamic(&artifact, &rows)?;
    let participants = fixture["fixtures"].as_array().ok_or("7H participants")?;
    let participants: Vec<_> = participants
        .iter()
        .filter(|actor| {
            matches!(
                actor["agent"].as_str(),
                Some(
                    "WRITER_A"
                        | "WRITER_B"
                        | "WRITER_C"
                        | "FIGURE_CREATOR"
                        | "FACT_CHECKER"
                        | "REVIEWER"
                        | "EDITOR_IN_CHIEF"
                )
            )
        })
        .cloned()
        .collect();
    assert_eq!(participants.len(), 7);
    let mut journal = journal::read(&rows, &headers, &participants)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let scores = journal.score["events"]
        .as_array()
        .ok_or("7H score events")?;
    let complete = true;
    let effects = 22
        + 2 * usize::from(complete)
        + 2 * part_ii_fact_rejections
        + 3 * part_ii_review_rejections
        + 4 * retries
        + 2 * (fact_failures + post_edit_fact) as usize
        + 3 * (part_i_review_rejections + post_edit_review as usize)
        + 2 * part_ii_minor.len()
        + 6 * usize::from(reject_amended_part_ii)
        + 3 * usize::from(reject_amended_part_i)
        + minor_sections.len()
        + usize::from(!minor_sections.is_empty());
    assert_eq!(scores.len(), effects);
    assert_eq!(journal.calls.len(), effects);
    assert_eq!(journal.score["complete"], complete);
    assert_eq!(journal.score["violations"], json!([]));
    let checked = crate::large_document_audit::check(compiled, &journal, &receipts)?;
    let correspondence = &checked["correspondence"];
    let fact_checks = checked["fact_checks"].as_array().ok_or("checked facts")?;
    let review_verdicts = checked["review_verdicts"]
        .as_array()
        .ok_or("checked reviews")?;
    let part_i = review_verdicts
        .iter()
        .filter(|review| review["scope"] == "CHAPTERS_1_2");
    let expected = std::iter::repeat_n("needs_revision", part_i_review_rejections)
        .chain([if minor_sections.is_empty() {
            "reviewed"
        } else {
            "minor_revision"
        }])
        .chain(reject_amended_part_i.then_some("reviewed"))
        .chain(std::iter::repeat_n(
            "needs_revision",
            post_edit_review as usize,
        ))
        .chain(std::iter::repeat_n("reviewed", retries));
    assert_eq!(
        part_i
            .map(|review| review["status"].as_str())
            .collect::<Vec<_>>(),
        expected.map(Some).collect::<Vec<_>>(),
    );
    let chapter_three_figure = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "insert_figure"
            && call.request["arguments"]["section_name"] == "CHAPTER_3"
    }))?;
    let evaluation_draft = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "write_section"
            && call.request["arguments"]["section_name"] == "CHAPTER_4"
    }))?;
    assert!(chapter_three_figure.ended()?.0 < evaluation_draft.started()?.0);
    let evaluation_creation = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "create_figure"
            && call.request["arguments"]["description"] == "Evaluation plot"
    }))?;
    let (&id, _) = one(journal.frames.iter().filter(|(_, frame)| {
        compiled["labels"]["EvaluationToChapter3"] == u64::from(frame.logical_label)
    }))?;
    let evaluation_received = receipts[&id]["receive_commit"]["journal_row"]
        .as_u64()
        .ok_or("evaluation receipt journal row")?;
    assert!(evaluation_creation.ended()?.0 < evaluation_received as usize);
    let evaluation_figure = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "insert_figure"
            && call.request["arguments"]["section_name"] == "CHAPTER_4"
    }))?;
    let contribution_references = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "update_references"
            && call.request["arguments"]["section_name"] == "CHAPTER_3"
    }))?;
    assert!(evaluation_figure.ended()?.0 < evaluation_received as usize);
    assert!((evaluation_received as usize) < contribution_references.started()?.0);
    let write_first = one(journal.calls.values().filter(|call| {
        call.request["tool"] == "write_section"
            && call.request["arguments"]["section_name"] == "CHAPTER_1"
    }))?;
    let initial = &write_first.result()?.1["data"]["artifact"];
    let mut submissions = Vec::new();
    let file = compiled["roles"]["WRITER_A_FILE"]
        .as_u64()
        .ok_or("writer A file")?;
    let document = compiled["roles"]["DOCUMENT"]
        .as_u64()
        .ok_or("document role")?;
    let drafting = compiled["roles"]["DRAFTING"]
        .as_u64()
        .ok_or("drafting actor")?;
    let writer = compiled["roles"]["WRITER_A_VM"]
        .as_u64()
        .ok_or("writer A actor")?;
    let mut inputs = serde_json::Map::new();
    for (chapter, label) in [
        ("CHAPTER_3", "ContributionToPartI"),
        ("CHAPTER_4", "EvaluationToPartI"),
    ] {
        let (_, accepted) = one(journal.frames.iter().filter(|(_, frame)| {
            frame.identity["target"] == file
                && compiled["labels"][label] == u64::from(frame.logical_label)
        }))?;
        let current = accepted.value()?;
        let review = journal
            .calls
            .values()
            .filter(|call| {
                call.request["tool"] == "review_sections"
                    && call.request["arguments"]["section_name"] == chapter
                    && call.result().is_ok_and(|(_, review)| {
                        matches!(
                            review["data"]["status"].as_str(),
                            Some("reviewed" | "minor_revision")
                        )
                    })
            })
            .max_by_key(|call| call.ended.map(|(row, _)| row))
            .ok_or("current scoped review")?;
        let (review_at, verdict) = review.result()?;
        let (checked_at, checked) = if part_ii_minor.contains(&chapter) && !reject_amended_part_ii {
            assert_eq!(verdict["data"]["status"], "minor_revision");
            one(journal.calls.values().filter(|call| {
                call.request["tool"] == "fact_check"
                    && call.request["arguments"]["section_name"] == chapter
                    && call
                        .result()
                        .is_ok_and(|(at, check)| review_at < at && check["success"] == true)
            }))?
            .result()?
        } else {
            assert_eq!(verdict["data"]["status"], "reviewed");
            (review_at, verdict)
        };
        assert_eq!(
            checked["data"]["input_snapshot"]["sections"][chapter],
            current
        );
        assert!(checked_at < accepted.received()?);
        inputs.insert(chapter.to_owned(), current);
    }
    for (chapter, label) in [("CHAPTER_1", "Finalize1"), ("CHAPTER_2", "Finalize2")] {
        let label = compiled["labels"][label]
            .as_u64()
            .ok_or("Part I submission label")?;
        let (&id, request) = one(journal.frames.iter().filter(|(_, frame)| {
            frame.direction(file, document) && u64::from(frame.logical_label) == label
        }))?;
        let (&reply_id, reply) = one(journal.frames.iter().filter(|(_, frame)| {
            frame.direction(document, file) && u64::from(frame.logical_label) == label
        }))?;
        let request_commit = &receipts[&id]["receive_commit"];
        let reply_commit = &receipts[&reply_id]["receive_commit"];
        assert_eq!(request_commit["actor"], drafting);
        assert_eq!(reply_commit["actor"], writer);
        let actual = reply.value()?;
        assert_eq!(request.value()?["section_name"], chapter);
        assert_eq!(actual["input_sections"].as_object(), Some(&inputs));
        if chapter == "CHAPTER_1" {
            assert_ne!(&actual, initial);
        }
        submissions.push(json!({"section":chapter,"artifact":actual,
            "request_commit":request_commit,"reply_commit":reply_commit}));
    }
    assert_eq!(
        fact_checks.len(),
        3 + retries
            + part_ii_minor.len()
            + 2 * usize::from(reject_amended_part_ii)
            + part_ii_fact_rejections
            + part_ii_review_rejections
            + (fact_failures + post_edit_fact) as usize
            + part_i_review_rejections
            + post_edit_review as usize
            + usize::from(reject_amended_part_i)
            + usize::from(!minor_sections.is_empty())
    );
    assert_eq!(
        review_verdicts.len(),
        3 + retries
            + part_ii_review_rejections
            + 2 * usize::from(reject_amended_part_ii)
            + usize::from(reject_amended_part_i)
            + part_i_review_rejections
            + post_edit_review as usize
    );
    for check in fact_checks
        .iter()
        .chain(review_verdicts)
        .filter(|check| check["scope"] != "CHAPTERS_1_2")
    {
        let end = check["ended_row"].as_u64().ok_or("fact-check completion")?;
        for submission in &submissions {
            assert!(
                end < submission["request_commit"]["journal_row"]
                    .as_u64()
                    .ok_or("Part I submission")?
            );
        }
    }
    let role = |name: &str| {
        compiled["roles"][name]
            .as_u64()
            .ok_or("projected editor role")
    };
    let editor = role("EDITOR")?;
    let owner = role("DOCUMENT_OWNER")?;
    let editor_file = role("EDITOR_FILE")?;
    let evidence_label = compiled["labels"]["PublicationMaterial"]
        .as_u64()
        .ok_or("publication material label")?;
    let mut materials: Vec<_> = journal
        .frames
        .iter()
        .filter(|(_, frame)| {
            frame.direction(document, editor_file)
                && u64::from(frame.logical_label) == evidence_label
        })
        .collect();
    materials.sort_by_key(|(_, frame)| frame.received);
    assert_eq!(materials.len(), 1 + retries);
    let material_commits = &checked["editor_material_commits"];
    let loan_label = compiled["labels"]["Loan"].as_u64().ok_or("loan label")?;
    let fact = role("FACT_PART_I")?;
    let (&fact_grant_id, fact_grant) = one(journal.frames.iter().filter(|(_, frame)| {
        frame.direction(owner, fact) && u64::from(frame.logical_label) == loan_label
    }))?;
    let (&fact_return_id, fact_return) = one(journal.frames.iter().filter(|(_, frame)| {
        frame.direction(fact, owner) && u64::from(frame.logical_label) == loan_label
    }))?;
    assert_eq!(receipts[&fact_grant_id]["receive_commit"]["actor"], fact);
    assert_eq!(receipts[&fact_return_id]["receive_commit"]["actor"], owner);
    let mut initial_part_i: Vec<_> = journal
        .calls
        .values()
        .filter(|call| {
            call.tool().ok() == Some("fact_check")
                && call.request["arguments"]["section_name"] == "CHAPTERS_1_2"
                && fact_grant.received.is_some_and(|at| at < call.emitted)
                && fact_return.received.is_some_and(|at| call.emitted < at)
        })
        .collect();
    initial_part_i.sort_by_key(|call| call.emitted);
    let initial_attempts = 1 + fact_failures as usize + part_i_review_rejections;
    let minor_check = if minor_sections.is_empty() {
        None
    } else {
        assert!(initial_part_i.len() > initial_attempts);
        Some(initial_part_i.remove(initial_attempts))
    };
    assert_eq!(
        initial_part_i.len(),
        initial_attempts + usize::from(reject_amended_part_i)
    );
    let mut assessments = Vec::new();
    for (attempt, checked) in initial_part_i.iter().enumerate() {
        let (_, checked_result) = checked.result()?;
        let check_id = &checked_result["data"]["check_id"];
        let passed_facts = attempt >= fact_failures as usize;
        assert_eq!(checked_result["success"], passed_facts);
        let (result_at, verdict) = if passed_facts {
            one(journal.calls.values().filter(|call| {
                call.tool().ok() == Some("review_sections")
                    && call.result().is_ok_and(|(_, result)| {
                        result["data"]["fact_receipts"]["CHAPTERS_1_2"] == *check_id
                    })
            }))?
            .result()?
        } else {
            checked.result()?
        };
        let amended = minor_check.is_some() && attempt + 1 == initial_attempts;
        let approved = attempt + 1 == initial_part_i.len() && !amended;
        assert_eq!(verdict["data"]["status"] == "minor_revision", amended);
        assert_eq!(verdict["success"] == true, approved);
        let mut deliveries = Vec::new();
        let mut author_received_at = None;
        for (source, target, actor, label) in [
            (
                if approved || amended { document } else { fact },
                file,
                writer,
                "AuthorOutcome",
            ),
            (
                if approved { document } else { fact },
                role("FACT_FILE")?,
                role("FACT_VM")?,
                "FactInput",
            ),
            (
                fact,
                role("REVIEW_FILE")?,
                role("REVIEW_VM")?,
                "ReviewInput",
            ),
        ] {
            let mut matches: Vec<_> = journal
                .frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(source, target)
                        && compiled["labels"][label] == u64::from(frame.logical_label)
                        && frame.value().is_ok_and(|payload| payload == *verdict)
                })
                .collect();
            // The reviewer receives the failed check and the resulting disposition.
            // Both carry the same check ID and must have separate receive commits.
            let count = if label == "ReviewInput" && !passed_facts {
                2
            } else {
                1
            };
            assert_eq!(matches.len(), count);
            matches.sort_by_key(|(_, frame)| frame.received);
            let mut previous = result_at;
            for (&id, frame) in matches {
                let received_at = frame.received()?;
                let commit = &receipts[&id]["receive_commit"];
                assert_eq!(commit["actor"], actor);
                assert!(previous < received_at);
                previous = received_at;
                if actor == writer {
                    author_received_at = Some(received_at);
                }
                deliveries.push(json!({"frame":id,"commit":commit}));
            }
        }
        if let Some(next) = initial_part_i.get(attempt + 1).filter(|_| !amended) {
            let received_at = author_received_at.ok_or("author revision verdict")?;
            let revised = one(journal.calls.values().filter(|call| {
                call.tool().ok() == Some("revise_section")
                    && received_at < call.emitted
                    && call.ended.is_some_and(|(at, _)| at < next.emitted)
            }))?;
            let section = revised.request["arguments"]["section_name"]
                .as_str()
                .ok_or("revised chapter")?;
            let current = &next.result()?.1["data"]["input_snapshot"]["sections"][section];
            assert_eq!(*current, revised.result()?.1["data"]["artifact"]);
            assert_ne!(
                *current,
                verdict["data"]["input_snapshot"]["sections"][section]
            );
        }
        assessments.push(
            json!({"deliveries":deliveries,"check_id":check_id,"check_success":passed_facts,
            "review_id":verdict["data"]["review_id"], "status":verdict["data"]["status"]}),
        );
    }
    let mut minor_amendments = Vec::new();
    if let Some(checked) = minor_check {
        let review = one(journal.calls.values().filter(|call| {
            call.tool().ok() == Some("review_sections")
                && call.request["arguments"]["section_name"] == "CHAPTERS_1_2"
                && call
                    .result()
                    .is_ok_and(|(_, result)| result["data"]["status"] == "minor_revision")
        }))?;
        let receipt = &review.result()?.1["data"];
        let (checked_at, check) = checked.result()?;
        assert_eq!(check["success"], !reject_amended_part_i);
        assert_ne!(
            check["data"]["check_id"],
            receipt["fact_receipts"]["CHAPTERS_1_2"]
        );
        let mut effects: Vec<_> = journal
            .calls
            .values()
            .filter(|call| {
                call.tool().ok() == Some("revise_section")
                    && call.request["arguments"]["review_id"] == receipt["review_id"]
            })
            .collect();
        effects.sort_by_key(|call| call.emitted);
        assert_eq!(effects.len(), minor_sections.len());
        for (effect, chapter) in effects.into_iter().zip(minor_sections) {
            assert_eq!(effect.request["arguments"]["section_name"], chapter);
            assert!(review.ended()?.0 < effect.emitted && effect.ended()?.0 < checked.emitted);
            let revised = &effect.result()?.1["data"]["artifact"];
            assert_eq!(revised["content"], receipt["minor_replacements"][chapter]);
            assert_eq!(
                revised["previous_id"],
                receipt["input_snapshot"]["sections"][chapter]["id"]
            );
            assert_eq!(revised["minor_review_id"], receipt["review_id"]);
            assert_eq!(
                *revised,
                check["data"]["input_snapshot"]["sections"][chapter]
            );
            minor_amendments.push(json!({"section":chapter,"review_id":receipt["review_id"],
                "fact_id":check["data"]["check_id"],"checked_at":checked_at,"artifact":revised}));
        }
        let source = if reject_amended_part_i {
            fact
        } else {
            document
        };
        let mut rejection_received = None;
        for (sender, recipient, receiver, message) in [
            (source, file, writer, "AuthorOutcome"),
            (source, role("FACT_FILE")?, role("FACT_VM")?, "FactInput"),
            (
                fact,
                role("REVIEW_FILE")?,
                role("REVIEW_VM")?,
                "ReviewInput",
            ),
        ] {
            let (&id, frame) = one(journal.frames.iter().filter(|(_, frame)| {
                frame.direction(sender, recipient)
                    && compiled["labels"][message] == u64::from(frame.logical_label)
                    && frame.value().is_ok_and(|payload| payload == *check)
            }))?;
            let at = frame.received()?;
            assert!(checked_at < at);
            assert_eq!(receipts[&id]["receive_commit"]["actor"], receiver);
            if recipient == file {
                rejection_received = Some(at);
            }
        }
        if reject_amended_part_i {
            let next = initial_part_i
                .last()
                .ok_or("reassessment after rejected amendments")?;
            let received = rejection_received.ok_or("author's rejected amendment receipt")?;
            let corrected = one(journal.calls.values().filter(|call| {
                call.tool().ok() == Some("revise_section")
                    && received < call.emitted
                    && call.ended.is_some_and(|(at, _)| at < next.emitted)
            }))?;
            let chapter = corrected.request["arguments"]["section_name"]
                .as_str()
                .ok_or("corrected chapter")?;
            assert_eq!(
                next.result()?.1["data"]["input_snapshot"]["sections"][chapter],
                corrected.result()?.1["data"]["artifact"]
            );
            assert_ne!(
                corrected.result()?.1["data"]["artifact"],
                check["data"]["input_snapshot"]["sections"][chapter]
            );
        }
    }
    let editor_loans: Vec<_> = journal
        .frames
        .iter()
        .filter(|(_, frame)| {
            (frame.direction(owner, editor) || frame.direction(editor, owner))
                && u64::from(frame.logical_label) == loan_label
        })
        .collect();
    assert_eq!(editor_loans.len(), 2 * (1 + retries));
    let reassessor = role("FACT_EDITORIAL")?;
    let mut reassessment_loans: Vec<_> = journal
        .frames
        .iter()
        .filter(|(_, frame)| {
            (frame.direction(owner, reassessor) || frame.direction(reassessor, owner))
                && u64::from(frame.logical_label) == loan_label
        })
        .collect();
    reassessment_loans.sort_by_key(|(_, frame)| frame.received);
    assert_eq!(reassessment_loans.len(), 2 * retries);

    let mut editorial_evidence = Vec::new();
    let mut outcomes = Vec::new();
    {
        let mut decisions: Vec<_> = journal
            .calls
            .values()
            .filter(|call| {
                call.tool()
                    .is_ok_and(|tool| tool == "make_editorial_decision")
            })
            .collect();
        decisions.sort_by_key(|call| call.emitted);
        assert_eq!(decisions.len(), materials.len());
        let mut transfers = editor_loans;
        transfers.sort_by_key(|(_, frame)| frame.received);
        let combined = one(journal
            .calls
            .values()
            .filter(|call| call.tool().is_ok_and(|tool| tool == "combine_sections")))?;
        for (attempt, (decided, (_, material_frame))) in decisions
            .into_iter()
            .zip(materials.iter().copied())
            .enumerate()
        {
            let ((&grant, granted), (&returned, reclaimed)) =
                (transfers[2 * attempt], transfers[2 * attempt + 1]);
            assert!(granted.direction(owner, editor) && reclaimed.direction(editor, owner));
            let grant = &receipts[&grant]["receive_commit"];
            let returned = &receipts[&returned]["receive_commit"];
            assert_eq!(grant["actor"], editor);
            assert_eq!(returned["actor"], owner);
            assert!(granted.received()? < material_frame.received()?);
            assert!(material_frame.received()? < decided.started()?.0);
            let returned_at = reclaimed.received()?;
            assert!(decided.ended()?.0 < returned_at);
            let material = material_frame.value()?;
            let decision = decided.result()?.1;
            for field in ["input_snapshot", "review_receipts"] {
                assert_eq!(decision["data"][field], material[field]);
            }
            let final_decision = attempt == retries;
            assert_eq!(decision["success"], final_decision);
            let mut evidence = json!({"grant":grant,"return":returned,
                "decision_id":decision["data"]["decision_id"],"decision_success":decision["success"]});
            if final_decision {
                assert!(decided.ended()?.0 < combined.started()?.0);
                assert!(combined.ended()?.0 < returned_at);
                let published = &combined.result()?.1["data"];
                for field in ["input_snapshot", "review_receipts", "fact_receipts"] {
                    assert_eq!(published[field], material[field]);
                }
                assert_eq!(
                    published["editorial_receipts"]["final"],
                    decision["data"]["decision_id"]
                );
                evidence["published_artifact"] = published["artifact_id"].clone();
                let (result_at, result) = combined.result()?;
                outcomes.push((editor, result.clone(), result_at, [None; 3]));
            } else {
                let next_grant = transfers[2 * attempt + 2].1.received()?;
                assert!(returned_at < next_grant);
                let ((&fact_grant_id, fact_grant), (&fact_return_id, fact_return)) = (
                    reassessment_loans[2 * attempt],
                    reassessment_loans[2 * attempt + 1],
                );
                assert!(
                    fact_grant.direction(owner, reassessor)
                        && fact_return.direction(reassessor, owner)
                );
                let grant_at = fact_grant.received()?;
                let return_at = fact_return.received()?;
                assert!(returned_at < grant_at && grant_at < return_at && return_at < next_grant);
                let grant_commit = &receipts[&fact_grant_id]["receive_commit"];
                let return_commit = &receipts[&fact_return_id]["receive_commit"];
                assert_eq!(grant_commit["actor"], reassessor);
                assert_eq!(return_commit["actor"], owner);
                let revised = one(journal.calls.values().filter(|call| {
                    call.tool().is_ok_and(|tool| tool == "revise_section")
                        && call.emitted > returned_at
                        && call.ended.is_some_and(|(end, _)| end < grant_at)
                }))?;
                let section = revised.request["arguments"]["section_name"]
                    .as_str()
                    .ok_or("revised section")?;
                let mut checked_in_custody: Vec<_> = journal
                    .calls
                    .values()
                    .filter(|call| {
                        call.tool().ok() == Some("fact_check")
                            && call.request["arguments"]["section_name"] == "CHAPTERS_1_2"
                            && grant_at < call.emitted
                            && call.emitted < return_at
                    })
                    .collect();
                checked_in_custody.sort_by_key(|call| call.emitted);
                let first_check = checked_in_custody.first().ok_or("first reassessment")?;
                assert_eq!(
                    first_check.result()?.1["data"]["input_snapshot"]["sections"][section],
                    revised.result()?.1["data"]["artifact"]
                );
                let errors = if attempt == 0 {
                    post_edit_fact + post_edit_review
                } else {
                    0
                };
                assert_eq!(checked_in_custody.len(), 1 + errors as usize);
                let updated = materials[attempt + 1].1.value()?;
                assert_ne!(
                    updated["review_receipts"]["CHAPTERS_1_2"],
                    material["review_receipts"]["CHAPTERS_1_2"]
                );
                assert_ne!(
                    updated["fact_receipts"]["CHAPTERS_1_2"],
                    material["fact_receipts"]["CHAPTERS_1_2"]
                );
                let refreshed_fact = one(journal.calls.values().filter(|call| {
                    call.tool().ok() == Some("fact_check")
                        && call.result().is_ok_and(|(_, result)| {
                            result["data"]["check_id"] == updated["fact_receipts"]["CHAPTERS_1_2"]
                        })
                }))?;
                let refreshed_review = one(journal.calls.values().filter(|call| {
                    call.tool().ok() == Some("review_sections")
                        && call.result().is_ok_and(|(_, result)| {
                            result["data"]["review_id"]
                                == updated["review_receipts"]["CHAPTERS_1_2"]
                        })
                }))?;
                assert!(grant_at < refreshed_fact.emitted && refreshed_fact.ended()?.0 < return_at);
                assert!(
                    grant_at < refreshed_review.emitted && refreshed_review.ended()?.0 < return_at
                );
                let checks = checked_in_custody
                    .iter()
                    .map(|call| {
                        let (_, check) = call.result()?;
                        Ok(json!({"check_id":check["data"]["check_id"],"success":check["success"]}))
                    })
                    .collect::<Result<Vec<_>>>()?;
                evidence["reassessment"] =
                    json!({"loan":grant_commit,"return":return_commit,"checks":checks});
                outcomes.push((
                    document,
                    decision.clone(),
                    returned_at,
                    [
                        Some(revised.emitted),
                        Some(refreshed_fact.emitted),
                        Some(refreshed_review.emitted),
                    ],
                ));
            }
            editorial_evidence.push(evidence);
        }
    }
    let mut outcome_receives = Vec::new();
    for (target, (message, file, actor)) in [
        ("AuthorOutcome", "WRITER_A_FILE", "WRITER_A_VM"),
        ("FactInput", "FACT_FILE", "FACT_VM"),
        ("ReviewInput", "REVIEW_FILE", "REVIEW_VM"),
    ]
    .into_iter()
    .enumerate()
    {
        let label = compiled["labels"][message]
            .as_u64()
            .ok_or("editorial outcome label")?;
        let (file, actor) = (role(file)?, role(actor)?);
        let mut deliveries: Vec<_> = journal
            .frames
            .iter()
            .filter(|(_, frame)| {
                frame.identity["target"] == file
                    && u64::from(frame.logical_label) == label
                    && frame.value().is_ok_and(|payload| {
                        !(message == "FactInput" && payload["sections"].is_object())
                            && !matches!(
                                payload["tool_name"].as_str(),
                                Some("fact_check" | "review_sections")
                            )
                    })
            })
            .collect();
        deliveries.sort_by_key(|(_, frame)| frame.received);
        assert_eq!(deliveries.len(), outcomes.len());
        for ((&id, frame), (source, result, available_at, next_effects)) in
            deliveries.into_iter().zip(&outcomes)
        {
            let expected_source = if target != 0 && result["tool_name"] == "make_editorial_decision"
            {
                reassessor
            } else {
                *source
            };
            assert!(frame.direction(expected_source, file));
            assert_eq!(&frame.value()?, result);
            let received = frame.received()?;
            let commit = &receipts[&id]["receive_commit"];
            assert_eq!(commit["actor"], actor);
            assert!(*available_at < received);
            assert!(next_effects[target].is_none_or(|at| received < at));
            outcome_receives.push(json!({"frame":id,"commit":commit,"tool_name":result["tool_name"],
                "decision_id":result["data"]["decision_id"],"artifact_id":result["data"]["artifact_id"]}));
        }
    }
    let audit = json!({"task":"7H","status":"passed",
        "runtime_binary_sha256":digest(&runtime)?,"events_sha256":digest(&events)?,
        "effects":journal.calls.len(),"captured_records":dynamic["captured_records"],"correspondence":correspondence,
        "evaluation_flow":{
            "chapter_three_evaluation_receive_row":evaluation_received,
            "chapter_four_figure_creation_start_row":evaluation_creation.started()?.0,
            "chapter_four_insertion_end_row":evaluation_figure.ended()?.0,
            "chapter_three_references_start_row":contribution_references.started()?.0
        },
        "part_i_submissions":submissions,
        "fact_checks":fact_checks,
        "part_i_assessments":assessments,"minor_amendments":minor_amendments,
        "review_verdicts":review_verdicts,
        "publication":{"material_commits":material_commits,"loan_transfers":2 * editorial_evidence.len(),
            "editorial":editorial_evidence,"outcome_receives":outcome_receives},
        "violations":0,"simulator_complete":journal.score["complete"],
        "model_calls":0,"scored_benchmark":false});
    save(&output.join("preparation-audit.json"), &audit)?;
    Ok(audit)
}
