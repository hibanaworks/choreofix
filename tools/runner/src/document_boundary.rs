//! Real wash guests and the shared document world; no model calls or permissions.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct Case {
    depth: u8,
    first: &'static str,
    review: &'static str,
    references_first: bool,
    cross_revision: bool,
    failures: &'static [u32],
    adversarial: bool,
    repeat_write: bool,
    repeat_references_after_revision: bool,
    large_reply: bool,
    voluntary_revision: bool,
}

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("document terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing document reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty document reply")?
        .map_err(Into::into)
}

async fn writer(guest: &process::Guest, agent: &str, chapter: &str, case: &Case) -> Result<()> {
    if case.adversarial {
        let reply = guest
            .command(
                agent,
                "echo '{\"section_name\":\"OUTSIDE\"}' > /service/write_section".into(),
            )
            .await?;
        assert!(
            reply["stdout"]
                .as_str()
                .ok_or("denial output")?
                .contains("Invalid argument"),
            "{reply}"
        );
    }
    let tools = if case.references_first {
        ["update_references", "write_section"]
    } else {
        ["write_section", "update_references"]
    };
    for tool in tools {
        let args = if tool == "write_section" {
            json!({"section_name":chapter,"content_notes":format!("independent material by {agent}")})
        } else {
            json!({"section_name":chapter})
        };
        let reply = guest
            .command(
                agent,
                format!("printf '%s' '{args}' > /service/{tool}; cat /service/{tool}"),
            )
            .await?;
        assert!(
            !reply["stdout"]
                .as_str()
                .ok_or("terminal output")?
                .contains("wash:"),
            "{reply}"
        );
        assert_eq!(response(&reply)?["success"], true, "{reply}");
        if tool == "write_section" && agent == "WRITER_A" && case.repeat_write {
            let repeated = json!({"section_name":chapter,"content_notes":"second initial draft"});
            let reply = guest
                .command(
                    agent,
                    format!("printf '%s' '{repeated}' > /service/write_section"),
                )
                .await?;
            assert!(
                reply["stdout"]
                    .as_str()
                    .ok_or("denial output")?
                    .contains("Permission denied"),
                "{reply}"
            );
        }
    }
    loop {
        let reply = guest.command(agent, "cat /service/feedback".into()).await?;
        let actual = response(&reply)?;
        if actual["tool_name"] == "combine_sections" {
            assert_eq!(actual["success"], true);
            break;
        }
        assert_eq!(actual["tool_name"], "review_sections");
        if actual["data"]["scope"] != chapter {
            continue;
        }
        let target = if case.cross_revision {
            if chapter == "CHAPTER_1" {
                "CHAPTER_2"
            } else {
                "CHAPTER_1"
            }
        } else {
            chapter
        };
        let feedback = actual["message"]
            .as_str()
            .unwrap_or("reconsider the current review");
        let feedback = if case.large_reply {
            format!("{feedback}\n{}", "追記資料".repeat(500))
        } else {
            feedback.to_owned()
        };
        let args = json!({"section_name":target,"feedback":feedback});
        let revised = if case.large_reply {
            guest
                .command(agent, "printf '%s' '' > /work/revision.json".into())
                .await?;
            let encoded = args.to_string();
            let mut remaining = encoded.as_str();
            while !remaining.is_empty() {
                let mut end = remaining.len().min(2048);
                while !remaining.is_char_boundary(end) {
                    end -= 1;
                }
                let chunk = &remaining[..end];
                let reply = guest
                    .command(
                        agent,
                        format!("printf '%s' '{chunk}' >> /work/revision.json"),
                    )
                    .await?;
                assert!(
                    !reply["stdout"]
                        .as_str()
                        .ok_or("terminal output")?
                        .contains("wash:"),
                    "{reply}"
                );
                remaining = &remaining[end..];
            }
            guest.command(agent, "cat /work/revision.json > /service/revise_section; cat /service/revise_section".into()).await?
        } else {
            guest.command(agent, format!("printf '%s' '{args}' > /service/revise_section; cat /service/revise_section")).await?
        };
        assert!(
            !revised["stdout"]
                .as_str()
                .ok_or("terminal output")?
                .contains("wash:"),
            "{revised}"
        );
        let revised = response(&revised)?;
        assert_eq!(revised["success"], true, "{revised}");
        assert_eq!(revised["data"]["section"], target);
        if case.large_reply {
            assert_eq!(revised["data"]["artifact"]["feedback"], feedback);
            assert!(revised.to_string().len() > 8192);
        }
        if case.repeat_references_after_revision && agent == "WRITER_A" {
            let arguments = json!({"section_name":chapter});
            let reply = guest
                .command(
                    agent,
                    format!("printf '%s' '{arguments}' > /service/update_references"),
                )
                .await?;
            assert!(
                reply["stdout"]
                    .as_str()
                    .ok_or("denial output")?
                    .contains("Permission denied"),
                "{reply}"
            );
        }
    }
    if !case.adversarial {
        assert_eq!(
            guest.command(agent, "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(())
}

async fn editor(guest: &process::Guest, case: &Case) -> Result<()> {
    let input = json!({"first_writer":case.first});
    let selected = guest
        .command(
            "EDITOR",
            format!(
                "printf '%s' '{input}' > /service/allocate_document; cat /service/allocate_document"
            ),
        )
        .await?;
    assert!(
        !selected["stdout"]
            .as_str()
            .ok_or("terminal output")?
            .contains("wash:"),
        "{selected}"
    );
    if case.adversarial {
        let early = guest
            .command(
                "EDITOR",
                "printf '%s' '{\"sections\":\"CHAPTER_1,CHAPTER_2\"}' > /service/combine_sections"
                    .into(),
            )
            .await?;
        assert!(
            early["stdout"]
                .as_str()
                .ok_or("early publication output")?
                .contains("Permission denied"),
            "{early}"
        );
        let reply = guest
            .command(
                "EDITOR",
                "echo '{\"section_name\":\"OUTSIDE\"}' > /service/review_chapter_1".into(),
            )
            .await?;
        assert!(
            reply["stdout"]
                .as_str()
                .ok_or("denial output")?
                .contains("Invalid argument"),
            "{reply}"
        );
    }
    let order = if case.review == "CHAPTER_1" {
        [1, 2]
    } else {
        [2, 1]
    };
    let mut voluntary = case.voluntary_revision;
    loop {
        // The fixture chooses one of two legal request orders. Both requests
        // are admitted before either result is read from its operation file.
        for number in order {
            let args = json!({"section_name":format!("CHAPTER_{number}")});
            let reply = guest
                .command(
                    "EDITOR",
                    format!("printf '%s' '{args}' > /service/review_chapter_{number}"),
                )
                .await?;
            assert!(
                !reply["stdout"]
                    .as_str()
                    .ok_or("terminal output")?
                    .contains("wash:"),
                "{reply}"
            );
        }
        let mut reviews = Vec::new();
        for number in [1, 2] {
            let reply = guest.command("EDITOR", format!("cat /service/review_chapter_{number} > /work/review_{number}.json; cat /work/review_{number}.json")).await?;
            assert!(
                !reply["stdout"]
                    .as_str()
                    .ok_or("terminal output")?
                    .contains("wash:"),
                "{reply}"
            );
            reviews.push(response(&reply)?);
        }
        let rejected = reviews.iter().position(|review| review["success"] == false);
        if rejected.is_none() && !voluntary {
            break;
        }
        let index = rejected.unwrap_or(0);
        if rejected.is_none() {
            voluntary = false;
        }
        let writer = if index == 0 { "a" } else { "b" };
        let input = json!({"review_id":reviews[index]["data"]["review_id"]});
        let reply = guest.command("EDITOR", format!("printf '%s' '{input}' > /service/reconsider_{writer}; cat /service/reconsider_{writer}")).await?;
        assert!(
            !reply["stdout"]
                .as_str()
                .ok_or("terminal output")?
                .contains("wash:"),
            "{reply}"
        );
        assert_eq!(response(&reply)?["success"], true);
    }
    let args = json!({"sections":if case.review=="CHAPTER_1" {"CHAPTER_1,CHAPTER_2"} else {"CHAPTER_2,CHAPTER_1"}});
    let reply = guest
        .command(
            "EDITOR",
            format!(
                "printf '%s' '{args}' > /service/combine_sections; cat /service/combine_sections"
            ),
        )
        .await?;
    assert!(
        !reply["stdout"]
            .as_str()
            .ok_or("terminal output")?
            .contains("wash:"),
        "{reply}"
    );
    assert_eq!(response(&reply)?["success"], true, "{reply}");
    if !case.adversarial {
        assert_eq!(
            guest.command("EDITOR", "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn both_initial_writers_and_review_orders_complete_the_real_document() -> Result<()> {
    run(&[
        Case {
            depth: 0,
            first: "WRITER_A",
            review: "CHAPTER_1",
            references_first: false,
            cross_revision: false,
            failures: &[],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 0,
            first: "WRITER_B",
            review: "CHAPTER_2",
            references_first: true,
            cross_revision: false,
            failures: &[],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
    ])
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn repeated_rejection_cross_chapter_edits_and_post_publication_commands_use_real_wasm()
-> Result<()> {
    run(&[
        Case {
            depth: 2,
            first: "WRITER_A",
            review: "CHAPTER_2",
            references_first: true,
            cross_revision: false,
            failures: &[],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 2,
            first: "WRITER_B",
            review: "CHAPTER_1",
            references_first: false,
            cross_revision: true,
            failures: &[],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 0,
            first: "WRITER_A",
            review: "CHAPTER_1",
            references_first: true,
            cross_revision: true,
            failures: &[1, 3, 5],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 0,
            first: "WRITER_B",
            review: "CHAPTER_2",
            references_first: false,
            cross_revision: false,
            failures: &[1, 3, 5, 7],
            adversarial: false,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 0,
            first: "WRITER_A",
            review: "CHAPTER_2",
            references_first: true,
            cross_revision: false,
            failures: &[],
            adversarial: true,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
        Case {
            depth: 2,
            first: "WRITER_B",
            review: "CHAPTER_1",
            references_first: false,
            cross_revision: true,
            failures: &[],
            adversarial: true,
            repeat_write: false,
            repeat_references_after_revision: false,
            large_reply: false,

            voluntary_revision: false,
        },
    ])
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn duplicate_initial_write_returns_an_error_then_preparation_and_review_continue()
-> Result<()> {
    run(&[Case {
        depth: 0,
        first: "WRITER_A",
        review: "CHAPTER_1",
        references_first: false,
        cross_revision: false,
        failures: &[],
        adversarial: false,
        repeat_write: true,
        repeat_references_after_revision: false,
        large_reply: false,

        voluntary_revision: false,
    }])
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn duplicate_references_after_revision_return_an_error_then_publication_continues()
-> Result<()> {
    run(&[Case {
        depth: 1,
        first: "WRITER_A",
        review: "CHAPTER_1",
        references_first: false,
        cross_revision: false,
        failures: &[],
        adversarial: false,
        repeat_write: false,
        repeat_references_after_revision: true,
        large_reply: false,

        voluntary_revision: false,
    }])
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn a_large_unicode_revision_reply_survives_chunks_and_publication() -> Result<()> {
    run(&[Case {
        depth: 1,
        first: "WRITER_A",
        review: "CHAPTER_1",
        references_first: false,
        cross_revision: false,
        failures: &[],
        adversarial: false,
        repeat_write: false,
        repeat_references_after_revision: false,
        large_reply: true,

        voluntary_revision: false,
    }])
    .await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified document runtime, actual wash and shared world"]
async fn an_editor_can_reconsider_a_positive_current_review_before_freezing() -> Result<()> {
    run(&[Case {
        depth: 0,
        first: "WRITER_B",
        review: "CHAPTER_2",
        references_first: true,
        cross_revision: false,
        failures: &[],
        adversarial: false,
        repeat_write: false,
        repeat_references_after_revision: false,
        large_reply: false,
        voluntary_revision: true,
    }])
    .await
}

#[hibana_runtime::scope]
async fn run(cases: &[Case]) -> Result<()> {
    if cases.is_empty() {
        return Err("empty document qualification".into());
    }
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("document evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "document-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_DOCUMENT_WASI_BINARY").ok_or("document binary")?)
            .canonicalize()?;
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_DOCUMENT_BUILD_RECEIPT").ok_or("document receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &input, &directory)?;
    let auditor = PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("document auditor")?)
        .canonicalize()?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "7E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"7E","depth":case.depth,"initial_writer":case.first,
            "review_first":case.review,"references_first":case.references_first,"cross_chapter_revision":case.cross_revision,
            "failure_overrides":case.failures,"adversarial_inputs":case.adversarial,
            "duplicate_initial_write":case.repeat_write,"duplicate_references_after_revision":case.repeat_references_after_revision,
            "large_unicode_reply":case.large_reply,"voluntary_positive_review_revision":case.voluntary_revision,
            "model_calls":0,"scored_benchmark":false,
            "build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,"harness_source_sha256":digest(Path::new("tools/runner/src/document_boundary.rs"))?,
            "test_barriers":if case.adversarial {"keep guests alive after their real publication replies to test late shell requests; not a direct alternate-Endpoint attack"} else {"none"}}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let failures = case
            .failures
            .iter()
            .map(|&at| ("review_sections", at))
            .collect::<Vec<_>>();
        let (simulator, _) =
            oracle::Simulator::new("7E", case.depth, 0.01, trace.clone(), &failures)?;
        let (guest, runtime) = process::guest(&binary, module, &["7E"], &output, trace, simulator)?;
        let ((),result)=hibana_runtime::race(async {
            async {
                let guest=&guest;
                async {
                    for (agent,chapter) in [("WRITER_A","CHAPTER_1"),("WRITER_B","CHAPTER_2")] {
                        async move {writer(guest,agent,chapter,case).await};
                    }
                    async {editor(guest,case).await};
                    Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
                }.await?;
                if case.adversarial {
                    for (agent,chapter) in [("WRITER_A","CHAPTER_1"),("WRITER_B","CHAPTER_2")] {
                        let denied=guest.command(agent,format!("echo '{}' > /service/revise_section",json!({"section_name":chapter,"feedback":"after publication"}))).await?;
                        assert!(denied["stdout"].as_str().ok_or("denial output")?.contains("Permission denied"),"{denied}");
                    }
                    for agent in ["WRITER_A","WRITER_B","EDITOR"] {
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                    }
                }
                Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
            };
            async {runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let events = result["score"]["events"]
            .as_array()
            .ok_or("document effects")?;
        assert_eq!(
            events
                .iter()
                .filter(|e| e["tool"] == "review_sections" && e["success"] == false)
                .count(),
            usize::from(case.depth) + case.failures.len()
        );
        let revisions = events
            .iter()
            .filter(|e| e["tool"] == "revise_section")
            .count();
        assert_eq!(events.len(), 7 + 3 * revisions);
        let status = Command::new(&auditor)
            .arg("--audit-document")
            .arg(output.join("events.jsonl"))
            .stdout(Stdio::from(fs::File::create(output.join("audit.json"))?))
            .stderr(Stdio::from(fs::File::create(
                output.join("audit-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!("document audit failed ({status}): {}", output.display()).into());
        }
        let audit: Value = serde_json::from_slice(&fs::read(output.join("audit.json"))?)?;
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["dynamic_protocol"]["status"], "passed");
        assert_eq!(audit["actual_resource_endpoint_requests"], 3 + revisions);
        let status = Command::new(&auditor)
            .arg("--check-document-mutants")
            .arg(output.join("events.jsonl"))
            .arg(output.join("mutants"))
            .stdout(Stdio::null())
            .stderr(Stdio::from(fs::File::create(
                output.join("mutants-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!(
                "document record mutations failed ({status}): {}",
                output.display()
            )
            .into());
        }
        let mutants: Value =
            serde_json::from_slice(&fs::read(output.join("mutants/result.json"))?)?;
        outcomes.push(json!({"directory":output,"domain_calls":events.len(),"revisions":revisions,
            "record_mutants_rejected":mutants["rejected"],"model_calls":0,"scored_benchmark":false}));
        save(
            &directory.join("progress.json"),
            &json!({"completed_cases":outcomes,"expected_cases":cases.len()}),
        )?;
    }
    save(
        &directory.join("result.json"),
        &json!({"status":"passed","cases":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
