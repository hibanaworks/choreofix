//! Exercise the document runtime through the ordinary experiment runner's guest connection.
use crate::{
    evidence::{Result, Trace, digest, save},
    oracle::Simulator,
    process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const CHAPTERS: [&str; 3] = ["CHAPTER_1", "CHAPTER_2", "CHAPTER_3"];
const FACT_SCOPES: [&str; 2] = ["CHAPTERS_1_2", "CHAPTER_3"];
const AGENTS: [&str; 5] = [
    "WRITER_A",
    "WRITER_B",
    "FIGURE_CREATOR",
    "FACT_CHECKER",
    "EDITOR",
];

#[hibana_runtime::main]
#[test]
#[ignore = "requires a built document-wasi and wash along with an evidence directory"]
async fn five_wash_guests_complete_the_document_and_reconsider() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let runtime =
        PathBuf::from(std::env::var_os("CHOREOFIX_DOCUMENT_WASI_BINARY").ok_or("document binary")?)
            .canonicalize()?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm")
        .canonicalize()?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence directory")?);
    fs::create_dir_all(&evidence)?;
    // Each schedule is driven through the same compiled global. This covers
    // all six writer orders with early, late and reversed figure insertions.
    for (case, depth, preparation) in [
        (
            "depth-0",
            0u8,
            [
                ("create", 0),
                ("write", 0),
                ("insert", 0),
                ("write", 2),
                ("read", 2),
                ("create", 2),
                ("insert", 2),
                ("write", 1),
            ],
        ),
        (
            "depth-1",
            1,
            [
                ("write", 1),
                ("create", 2),
                ("write", 2),
                ("insert", 2),
                ("read", 2),
                ("write", 0),
                ("create", 0),
                ("insert", 0),
            ],
        ),
        (
            "depth-1-recheck-chapters-1-2",
            1,
            [
                ("create", 0),
                ("write", 2),
                ("read", 2),
                ("write", 0),
                ("insert", 0),
                ("create", 2),
                ("insert", 2),
                ("write", 1),
            ],
        ),
        (
            "depth-2",
            2,
            [
                ("write", 2),
                ("create", 2),
                ("insert", 2),
                ("read", 2),
                ("write", 0),
                ("create", 0),
                ("write", 1),
                ("insert", 0),
            ],
        ),
        (
            "write-1-2-3",
            0,
            [
                ("write", 0),
                ("write", 1),
                ("write", 2),
                ("read", 2),
                ("create", 2),
                ("create", 0),
                ("insert", 2),
                ("insert", 0),
            ],
        ),
        (
            "write-2-1-3",
            0,
            [
                ("create", 2),
                ("create", 0),
                ("write", 1),
                ("write", 0),
                ("insert", 0),
                ("write", 2),
                ("read", 2),
                ("insert", 2),
            ],
        ),
        (
            "write-3-2-1",
            0,
            [
                ("write", 2),
                ("read", 2),
                ("write", 1),
                ("write", 0),
                ("create", 0),
                ("create", 2),
                ("insert", 0),
                ("insert", 2),
            ],
        ),
    ] {
        let output = evidence.join(case);
        fs::create_dir(&output)?;
        projection::archive(&runtime, &output, "7M")?;
        save(
            &output.join("case.json"),
            &json!({
                "task":"7M", "depth":depth, "preparation":preparation, "wash_wasm_sha256":digest(&module)?,
                "runtime_binary_sha256":digest(&runtime)?,
                "compiled_projection_sha256":digest(&output.join("projection.json"))?,
                "model_calls":0, "scored_benchmark":false,
            }),
        )?;
        let overlap = case == "depth-0";
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) = Simulator::new(
            "7M",
            depth,
            if overlap { 0.1 } else { 0.0 },
            trace.clone(),
            &[],
        )?;
        for actor in fixture["fixtures"].as_array().ok_or("7M actors")? {
            let agent = actor["agent"].as_str().ok_or("actor name")?;
            fs::write(
                output.join(format!("{agent}-prompt.txt")),
                crate::prompt(Path::new("."), "7M", actor)?,
            )?;
        }
        let (guest, runtime_loop) =
            process::guest(&runtime, &module, &["7M"], &output, trace, simulator)?;
        let exercise = async {
            let agents = AGENTS;
            let output_json = |reply: &Value| -> Value {
                let stdout = reply["stdout"].as_str().unwrap();
                serde_json::from_str(stdout.strip_suffix("wash> ").unwrap().trim())
                    .unwrap_or_else(|error| panic!("invalid JSON from wash: {error}: {stdout:?}"))
            };
            for (agent, chapter) in [
                ("WRITER_A", CHAPTERS[1]),
                ("WRITER_B", CHAPTERS[2]),
                ("WRITER_A", CHAPTERS[0]),
            ] {
                let path = format!("/service/update_references/{chapter}");
                let args = json!({"section_name":chapter});
                let receipt = output_json(
                    &guest
                        .command(agent, format!("echo '{args}' > {path}; cat {path}"))
                        .await?,
                );
                assert_eq!(receipt["success"], true);
                assert_eq!(receipt["data"]["section"], chapter);
            }
            let early = json!({"section_name":CHAPTERS[2],"figure_id":"not-yet-created"});
            let refused = guest
                .command(
                    "FIGURE_CREATOR",
                    format!(
                        "echo '{early}' | dd of=/service/insert_figure/CHAPTER_3 oflag=nonblock"
                    ),
                )
                .await?;
            let error = refused["stdout"]
                .as_str()
                .ok_or("early insertion shell reply")?;
            assert!(
                error.contains("Permission denied")
                    || error.contains("Resource temporarily unavailable"),
                "{refused}"
            );
            let mismatched = guest
                .command(
                    "FIGURE_CREATOR",
                    format!("echo '{early}' > /service/insert_figure/CHAPTER_1"),
                )
                .await?;
            assert!(
                mismatched["stdout"]
                    .as_str()
                    .unwrap()
                    .contains("Invalid argument (os error 28)"),
                "{mismatched}"
            );
            let mut figures = [Value::Null, Value::Null, Value::Null];
            let steps = if overlap {
                let (created, written) = async {
                    async {
                        guest.command("FIGURE_CREATOR", r#"echo '{"description":"Diagram for CHAPTER_1"}' > /service/create_figure/CHAPTER_1; cat /service/create_figure/CHAPTER_1"#.into()).await
                    };
                    async {
                        guest.command("WRITER_A", r#"echo '{"section_name":"CHAPTER_1","content_notes":"Draft for CHAPTER_1"}' > /service/write_section/CHAPTER_1; cat /service/write_section/CHAPTER_1"#.into()).await
                    };
                }.await?;
                let created = output_json(&created);
                let written = output_json(&written);
                assert_eq!(created["success"], true);
                assert_eq!(written["success"], true);
                assert_eq!(written["data"]["section"], CHAPTERS[0]);
                assert!(created["data"]["figure_id"].is_string());
                figures[0] = created["data"]["figure_id"].clone();
                &preparation[2..]
            } else {
                &preparation[..]
            };
            let mut read_chapter = Value::Null;
            for &(tool, index) in steps {
                let chapter = CHAPTERS[index];
                let writer = if index == 2 { "WRITER_B" } else { "WRITER_A" };
                let (actor, path, args) = match tool {
                    "create" => (
                        "FIGURE_CREATOR",
                        format!("/service/create_figure/{chapter}"),
                        json!({"description":format!("Diagram for {chapter}")}),
                    ),
                    "write" => {
                        let notes = if index == 1 && !read_chapter.is_null() {
                            format!("Method using {}", read_chapter.as_str().unwrap())
                        } else {
                            format!("Draft for {chapter}")
                        };
                        (
                            writer,
                            format!("/service/write_section/{chapter}"),
                            json!({"section_name":chapter,"content_notes":notes}),
                        )
                    }
                    "read" => (
                        "WRITER_A",
                        "/service/read_section".to_owned(),
                        json!({"section_name":chapter}),
                    ),
                    "insert" => (
                        "FIGURE_CREATOR",
                        format!("/service/insert_figure/{chapter}"),
                        json!({"section_name":chapter,"figure_id":figures[index]}),
                    ),
                    _ => unreachable!(),
                };
                let reply = guest
                    .command(actor, format!("echo '{args}' > {path}; cat {path}"))
                    .await?;
                let actual = output_json(&reply);
                assert_eq!(
                    actual["success"], true,
                    "{case}: {tool} {chapter}: {actual}"
                );
                match tool {
                    "create" => {
                        assert!(actual["data"]["figure_id"].is_string());
                        figures[index] = actual["data"]["figure_id"].clone();
                    }
                    "read" => {
                        assert!(actual["data"]["artifact"]["id"].is_string());
                        read_chapter = actual["data"]["artifact"]["id"].clone();
                    }
                    "insert" => assert_eq!(actual["data"]["figure_id"], figures[index]),
                    _ => (),
                }
                if tool != "create" {
                    assert_eq!(actual["data"]["section"], chapter);
                }
            }
            let scopes = if depth == 0 || case == "depth-1-recheck-chapters-1-2" {
                FACT_SCOPES
            } else {
                [FACT_SCOPES[1], FACT_SCOPES[0]]
            };
            let mut checks = serde_json::Map::<String, Value>::new();
            let mut recheck = None;
            let mut reviewed_snapshots = Vec::new();
            let mut combined = None;
            let mut pending_outcome = None;
            for round in 0..=2 * depth {
                let material = match pending_outcome.take() {
                    Some(material) => material,
                    None => output_json(
                        &guest
                            .command("FACT_CHECKER", "cat /service/outcome".into())
                            .await?,
                    ),
                };
                if let Some(changed) = recheck {
                    let unchanged = FACT_SCOPES
                        .into_iter()
                        .find(|scope| *scope != changed)
                        .unwrap();
                    assert_eq!(material.as_object().unwrap().len(), 1);
                    assert!(material[unchanged].is_null());
                    let args = json!({"section_name":unchanged});
                    let refused = guest.command(
                        "FACT_CHECKER",
                        format!("echo '{args}' | dd of=/service/fact_check/{unchanged} oflag=nonblock"),
                    ).await?;
                    let text = refused["stdout"]
                        .as_str()
                        .ok_or("unchanged scope request")?;
                    assert!(
                        text.contains("Permission denied")
                            || text.contains("Resource temporarily unavailable"),
                        "{refused}"
                    );
                } else {
                    assert_eq!(material.as_object().unwrap().len(), 2);
                }
                if depth == 1 && round == 1 {
                    let reread = guest
                        .command(
                            "FACT_CHECKER",
                            "dd if=/service/outcome iflag=nonblock".into(),
                        )
                        .await?;
                    let text = reread["stdout"]
                        .as_str()
                        .ok_or("duplicate material reply")?;
                    assert!(
                        text.contains("Permission denied")
                            || text.contains("Resource temporarily unavailable"),
                        "{reread}"
                    );
                    if case == "depth-1-recheck-chapters-1-2" {
                        // The live checker read the delivered revision and then tried
                        // to read the notice and old result again before resubmitting.
                        for path in ["/service/outcome", "/service/fact_check/CHAPTERS_1_2"] {
                            let reply =
                                guest.command("FACT_CHECKER", format!("cat {path}")).await?;
                            let text = reply["stdout"].as_str().ok_or("duplicate read output")?;
                            assert!(text.contains("Permission denied"), "{reply}");
                        }
                    }
                }
                for scope in scopes
                    .into_iter()
                    .filter(|scope| recheck.is_none_or(|changed| *scope == changed))
                {
                    let reply = &material[scope];
                    assert!(reply.is_object());
                    let path = format!("/service/fact_check/{scope}");
                    if round == 0 && scope == FACT_SCOPES[1] {
                        let wrong = json!({"section_name": FACT_SCOPES[0]});
                        let rejected = guest
                            .command("FACT_CHECKER", format!("echo '{wrong}' > {path}"))
                            .await?;
                        assert!(
                            rejected["stdout"]
                                .as_str()
                                .unwrap()
                                .contains("Invalid argument (os error 28)"),
                            "{rejected}",
                        );
                    }
                    let args = json!({"section_name":scope});
                    let sent = guest
                        .command("FACT_CHECKER", format!("echo '{args}' > {path}"))
                        .await?;
                    assert_eq!(sent["stdout"], "wash> ");
                    let actual =
                        output_json(&guest.command("FACT_CHECKER", format!("cat {path}")).await?);
                    assert_eq!(&actual["data"]["input_snapshot"], reply);
                    if let Some(previous) = checks.insert(scope.into(), actual) {
                        assert_ne!(
                            &previous["data"]["input_snapshot"], reply,
                            "{scope} was checked again without a material change",
                        );
                    }
                }
                if let Some(rejection) = FACT_SCOPES
                    .into_iter()
                    .map(|scope| &checks[scope])
                    .find(|receipt| receipt["success"] == false)
                {
                    let (writer, chapter) = if rejection["data"]["section"] == FACT_SCOPES[0] {
                        (
                            "WRITER_A",
                            if depth == 1 { CHAPTERS[0] } else { CHAPTERS[1] },
                        )
                    } else {
                        ("WRITER_B", CHAPTERS[2])
                    };
                    let notice =
                        output_json(&guest.command(writer, "cat /service/outcome".into()).await?);
                    assert_eq!(&notice, rejection);
                    if depth == 1 {
                        let unassigned = if writer == "WRITER_A" {
                            CHAPTERS[2]
                        } else {
                            CHAPTERS[0]
                        };
                        let wrong = json!({"section_name":unassigned,"feedback":"outside the assigned chapters"});
                        let refused = guest
                            .command(writer, format!("echo '{wrong}' > /service/revise_section"))
                            .await?;
                        assert!(
                            refused["stdout"]
                                .as_str()
                                .unwrap()
                                .contains("Invalid argument (os error 28)"),
                            "{refused}",
                        );
                    }
                    let args = json!({"section_name":chapter,"feedback":"Support the section with evidence"});
                    let revised = output_json(&guest.command(
                    writer,
                    format!("echo '{args}' > /service/revise_section; cat /service/revise_section"),
                ).await?);
                    assert_eq!(revised["success"], true);
                    assert_eq!(revised["data"]["section"], chapter);
                    recheck = Some(if chapter == CHAPTERS[2] {
                        FACT_SCOPES[1]
                    } else {
                        FACT_SCOPES[0]
                    });
                    continue;
                }
                let (next, publication) = async {
                    async {
                        let reply = guest.command("FACT_CHECKER", "cat /service/outcome".into()).await?;
                        Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(output_json(&reply))
                    };
                    async {
                let mut material = None;
                for _ in 0..64 {
                    let reply = guest
                        .command("EDITOR", "cat /service/review_material".into())
                        .await?;
                    let text = reply["stdout"]
                        .as_str()
                        .ok_or("review material shell output")?;
                    if text.contains("Permission denied")
                        || text.contains("Resource temporarily unavailable")
                    {
                        hibana_runtime::host::sleep(Duration::from_millis(2)).await?;
                    } else {
                        material = Some(output_json(&reply));
                        break;
                    }
                }
                let reply = material.ok_or("editor did not receive current material")?;
                assert_eq!(
                    reply["fact_receipts"],
                    json!({
                        FACT_SCOPES[0]:checks[FACT_SCOPES[0]]["data"]["check_id"],
                        FACT_SCOPES[1]:checks[FACT_SCOPES[1]]["data"]["check_id"],
                    }),
                );
                let mut review_receipts = serde_json::Map::new();
                let mut review_rejections = Vec::new();
                let order = if depth == 0 {
                    [CHAPTERS[2], CHAPTERS[0], CHAPTERS[1]]
                } else {
                    [CHAPTERS[0], CHAPTERS[2], CHAPTERS[1]]
                };
                for chapter in order {
                    let path = format!("/service/review_sections/{chapter}");
                    let args = json!({"section_name":chapter});
                    let sent = guest
                        .command("EDITOR", format!("echo '{args}' > {path}"))
                        .await?;
                    assert_eq!(sent["stdout"], "wash> ");
                    let actual =
                        output_json(&guest.command("EDITOR", format!("cat {path}")).await?);
                    assert_eq!(actual["data"]["input_snapshot"], reply["input_snapshot"]);
                    assert_eq!(actual["data"]["fact_receipts"], reply["fact_receipts"]);
                    if actual["success"] == true {
                        review_receipts
                            .insert(chapter.to_owned(), actual["data"]["review_id"].clone());
                    } else {
                        review_rejections.push(actual);
                    }
                }
                reviewed_snapshots.push(reply["input_snapshot"].clone());
                let path = "/service/combine_sections";
                let args = json!({"sections":"CHAPTER_3,CHAPTER_1,CHAPTER_2"});
                let sent = guest
                    .command("EDITOR", format!("echo '{args}' > {path}"))
                    .await?;
                if review_receipts.len() == CHAPTERS.len() {
                    assert_eq!(sent["stdout"], "wash> ");
                    let result =
                        output_json(&guest.command("EDITOR", format!("cat {path}")).await?);
                    assert_eq!(result["success"], true);
                    assert_eq!(
                        result["data"]["sections"],
                        json!([CHAPTERS[2], CHAPTERS[0], CHAPTERS[1]])
                    );
                    assert_eq!(result["data"]["input_snapshot"], reply["input_snapshot"]);
                    assert_eq!(result["data"]["fact_receipts"], reply["fact_receipts"]);
                    assert_eq!(
                        result["data"]["review_receipts"],
                        Value::Object(review_receipts)
                    );
                    Ok::<Option<Value>, Box<dyn std::error::Error + Send + Sync>>(Some(result))
                } else {
                    assert!(
                        sent["stdout"]
                            .as_str()
                            .unwrap()
                            .contains("Permission denied (os error 2)"),
                        "{sent}",
                    );
                    let rejection = CHAPTERS
                        .into_iter()
                        .find_map(|chapter| {
                            review_rejections
                                .iter()
                                .find(|receipt| receipt["data"]["scope"] == chapter)
                        })
                        .expect("actual review rejection");
                    let writer = if rejection["data"]["scope"] == CHAPTERS[2] {
                        "WRITER_B"
                    } else {
                        "WRITER_A"
                    };
                    let notice =
                        output_json(&guest.command(writer, "cat /service/outcome".into()).await?);
                    assert_eq!(&notice, rejection);
                    let args = json!({"section_name":notice["data"]["scope"],"feedback":"Revise the section and support the review claims"});
                    let revised = output_json(&guest.command(
                    writer,
                    format!("echo '{args}' > /service/revise_section; cat /service/revise_section"),
                ).await?);
                    assert_eq!(revised["success"], true);
                    assert_eq!(revised["data"]["section"], notice["data"]["scope"]);
                    recheck = Some(if revised["data"]["section"] == CHAPTERS[2] {
                        FACT_SCOPES[1]
                    } else {
                        FACT_SCOPES[0]
                    });
                    Ok(None)
                }
                    };
                }.await?;
                if let Some(publication) = publication {
                    assert_eq!(next, publication);
                    combined = Some(publication);
                    break;
                }
                pending_outcome = Some(next);
            }
            if depth == 1 {
                assert_eq!(reviewed_snapshots.len(), 2);
                assert_ne!(
                    reviewed_snapshots[0]["sections"][CHAPTERS[0]],
                    reviewed_snapshots[1]["sections"][CHAPTERS[0]]
                );
            }
            let combined = combined.expect("editor completed assembly");
            for agent in ["WRITER_A", "WRITER_B"] {
                let result = guest.command(agent, "cat /service/outcome".into()).await?;
                assert_eq!(output_json(&result), combined);
            }
            for agent in agents {
                assert_eq!(
                    guest.command(agent, "exit 0".into()).await?["exit_status"],
                    0
                );
            }

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        };
        let ((), result) = hibana_runtime::race(
            async {
                async { exercise.await };
                async { runtime_loop.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(60)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(result["protocol"]["status"], "complete");
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let effects = result["score"]["events"]
            .as_array()
            .ok_or("7M score events")?;
        assert_eq!(effects.len(), [17, 24, 26][usize::from(depth)]);
        for (action, tool) in [("write", "write_section"), ("insert", "insert_figure")] {
            let actual: Vec<_> = effects
                .iter()
                .filter(|effect| effect["tool"] == tool)
                .map(|effect| effect["args"]["section_name"].as_str().unwrap())
                .collect();
            let expected: Vec<_> = preparation
                .iter()
                .filter(|(step, _)| *step == action)
                .map(|(_, chapter)| CHAPTERS[*chapter])
                .collect();
            assert_eq!(actual, expected, "{case}: actual {tool} order");
        }
        if overlap {
            let records = crate::journal::records(&output.join("events.jsonl"))?;
            let interval = |tool: &str, key: &str, value: &str| {
                let call = records
                    .iter()
                    .find(|row| {
                        row["direction"] == "domain_start"
                            && row["message"]["tool"] == tool
                            && row["message"]["arguments"][key] == value
                    })
                    .expect("actual simulator call");
                let result = &records
                    .iter()
                    .find(|row| {
                        row["direction"] == "simulator"
                            && row["message"]["id"] == call["message"]["id"]
                    })
                    .expect("actual simulator result")["message"];
                (
                    result["started_ns"].as_u64().unwrap(),
                    result["finished_ns"].as_u64().unwrap(),
                )
            };
            let creation = interval("create_figure", "description", "Diagram for CHAPTER_1");
            let writing = interval("write_section", "section_name", CHAPTERS[0]);
            let overlap_ns = creation
                .1
                .min(writing.1)
                .saturating_sub(creation.0.max(writing.0));
            assert!(
                overlap_ns > 0,
                "independent effects failed to overlap: {creation:?} {writing:?}"
            );
            save(
                &output.join("overlap.json"),
                &json!({
                    "clock": "runner monotonic elapsed", "create_figure_ns": creation,
                    "write_section_ns": writing, "overlap_ns": overlap_ns,
                }),
            )?;
        }
        let checks: Vec<_> = effects
            .iter()
            .filter(|effect| effect["tool"] == "fact_check")
            .map(|effect| effect["args"]["section_name"].as_str().unwrap())
            .collect();
        let [pair, third] = FACT_SCOPES;
        let expected_checks: &[&str] = match (case, depth) {
            (_, 0) => &[pair, third],
            ("depth-1-recheck-chapters-1-2", 1) => &[pair, third, pair, pair],
            (_, 1) => &[third, pair, third, pair],
            (_, 2) => &[third, pair, pair, third, pair],
            _ => unreachable!(),
        };
        assert_eq!(checks, expected_checks, "{case}: checked scope sequence");
        let revisions = effects
            .iter()
            .filter(|event| event["tool"] == "revise_section")
            .map(|event| event["args"]["section_name"].as_str().unwrap())
            .collect::<Vec<_>>();
        let expected_revisions: &[&str] = match (case, depth) {
            (_, 0) => &[],
            ("depth-1-recheck-chapters-1-2", 1) => &[CHAPTERS[0], CHAPTERS[0]],
            (_, 1) => &[CHAPTERS[2], CHAPTERS[0]],
            (_, 2) => &[CHAPTERS[1], CHAPTERS[2], CHAPTERS[0]],
            _ => unreachable!(),
        };
        assert_eq!(revisions, expected_revisions, "{case}: revised chapters");
        let checked = crate::journal::Evidence::open(&output.join("events.jsonl"))?
            .audit(crate::figure_document_audit::check)?;
        assert_eq!(checked["status"], "passed");
        save(&output.join("audit.json"), &checked)?;
        if depth == 2 {
            let path = output.join("events.jsonl");
            let mut altered = crate::journal::records(&path)?;
            let insertion = checked["effects"]
                .as_array()
                .unwrap()
                .iter()
                .find(|effect| effect["tool"] == "insert_figure")
                .unwrap();
            let request = &insertion["authorities"][1]["frame"];
            let frame = altered
                .iter_mut()
                .find(|row| {
                    row["direction"] == "runtime"
                        && row["message"]["type"] == "protocol_frame"
                        && row["message"]["action"] == "send"
                        && row["message"]["frame_id"] == *request
                })
                .expect("actual figure-store request");
            let arguments = json!({"section_name":"CHAPTER_3","figure_id":"another-figure"});
            frame["message"]["payload"] = json!(serde_json::to_vec(&arguments)?);
            let (artifact, _) = crate::projection::load(&path)?;
            let execution = crate::projection::check_dynamic(&artifact, &altered)?;
            let error = crate::figure_document_audit::check(&altered, &artifact, &execution)
                .expect_err("a substituted resource request cannot account for the actual effect");
            assert!(
                error
                    .to_string()
                    .contains("7M frame data differs at FIGURE_STORE/DOCUMENT/insert_3"),
                "{error}"
            );
            save(
                &output.join("substituted-resource-result.json"),
                &json!({"rejected":true,"reason":error.to_string()}),
            )?;
        }
    }
    Ok(())
}
