//! Execute 7H's production preparation with the projected Endpoints.
use super::{facts::*, figures::*, preparation::*, publication::*, review::*};
use crate::{imp, shell};
use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
use imp::json::decode;
use serde_json::{Value, json};
use shell::{
    ShellResult,
    global::Loan,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use std::{cell::Cell, time::Duration};

#[derive(Default)]
struct Journal(Cell<Vec<Value>>);

impl ReceivedEffects for Journal {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        request: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let (agent, tool) = P::RPC;
        let args = decode(request.as_bytes());
        async move {
            let args = args?;
            let mut entries = self.0.take();
            let section = &args["section_name"];
            if tool == "review_sections" {
                let checked = entries
                    .iter()
                    .find(|entry| {
                        entry["tool_name"] == "fact_check" && entry["data"]["section"] == *section
                    })
                    .ok_or_else(|| {
                        shell::ShellError::message("review requires an actual fact check")
                    })?;
                let approved = true;
                let result = json!({"success":approved,"tool_name":tool,"data":{
                    "scope":section,"agent":agent,"review_id":format!("review:{}", entries.len() + 1),
                    "status":if approved { "reviewed" } else { "needs_revision" },"input_snapshot":checked["data"]["input_snapshot"],
                    "fact_receipts":{(section.as_str().unwrap()):checked["data"]["check_id"]}
                }});
                entries.push(result.clone());
                self.0.set(entries);
                return Ok(result);
            }
            if tool == "fact_check" {
                let chapters: &[&str] = match section.as_str() {
                    Some("CHAPTERS_1_2") => &CHAPTERS[..2],
                    Some("CHAPTER_3") => &CHAPTERS[2..3],
                    Some("CHAPTER_4") => &CHAPTERS[3..],
                    _ => return Err(shell::ShellError::message("unknown fact-check scope")),
                };
                let mut snapshot = json!({"sections":{},"references":{},"figures":{}});
                for entry in &entries {
                    let Some(chapter) = entry["data"]["section"].as_str() else {
                        continue;
                    };
                    if !chapters.contains(&chapter) {
                        continue;
                    }
                    let category = match entry["tool_name"].as_str() {
                        Some("write_section" | "revise_section") => "sections",
                        Some("update_references") => "references",
                        Some("insert_figure") => "figures",
                        _ => continue,
                    };
                    snapshot[category][chapter] = entry["data"]["artifact"].clone();
                }
                let result = json!({"success":true,"tool_name":tool,"data":{
                    "section":section,"agent":agent,"check_id":format!("fact:{}", entries.len() + 1),
                    "input_snapshot":snapshot
                }});
                entries.push(result.clone());
                self.0.set(entries);
                return Ok(result);
            }
            if matches!(tool, "make_editorial_decision" | "combine_sections") {
                let mut snapshot = json!({"sections":{},"figures":{},"references":{}});
                let mut facts = serde_json::Map::new();
                let mut reviews = serde_json::Map::new();
                for scope in FACT_SCOPES {
                    let review = entries
                        .iter()
                        .rev()
                        .find(|entry| {
                            entry["tool_name"] == "review_sections"
                                && entry["data"]["scope"] == scope
                        })
                        .ok_or_else(|| shell::ShellError::message("missing publication review"))?;
                    let data = &review["data"];
                    for category in ["sections", "figures", "references"] {
                        for (chapter, artifact) in data["input_snapshot"][category]
                            .as_object()
                            .ok_or_else(|| shell::ShellError::message("fixture snapshot"))?
                        {
                            snapshot[category][chapter] = artifact.clone();
                        }
                    }
                    facts.insert(scope.to_owned(), data["fact_receipts"][scope].clone());
                    reviews.insert(scope.to_owned(), data["review_id"].clone());
                }
                let result = if tool == "make_editorial_decision" {
                    json!({"success":true,"tool_name":tool,"data":{"agent":agent,"decision_type":"final",
                    "decision_id":"fixture-decision","review_summary":args["review_summary"],
                    "review_receipts":reviews,"input_snapshot":snapshot}})
                } else {
                    json!({"success":true,"tool_name":tool,"data":{"agent":agent,"status":"combined",
                    "artifact_id":"fixture-publication","sections":CHAPTERS,"editorial_receipts":{"final":"fixture-decision"},
                    "review_receipts":reviews,"fact_receipts":facts,"input_snapshot":snapshot}})
                };
                entries.push(result.clone());
                self.0.set(entries);
                return Ok(result);
            }
            let mut artifact = json!({
                "id": format!("{tool}:{}", entries.len() + 1),
                "content": args["content_notes"].as_str().or(args["description"].as_str()).unwrap_or("Current references"),
            });
            if matches!(tool, "write_section" | "revise_section")
                && (section == "CHAPTER_1" || section == "CHAPTER_2")
            {
                let inputs: serde_json::Map<String, Value> = entries
                    .iter()
                    .filter_map(|entry| {
                        let chapter = entry["data"]["section"].as_str()?;
                        (entry["tool_name"] == "write_section"
                            && matches!(chapter, "CHAPTER_3" | "CHAPTER_4"))
                        .then(|| (chapter.to_owned(), entry["data"]["artifact"].clone()))
                    })
                    .collect();
                artifact["input_sections"] = Value::Object(inputs);
                if tool == "revise_section" {
                    let prior = entries
                        .iter()
                        .find(|entry| {
                            entry["tool_name"] == "write_section"
                                && entry["data"]["section"] == *section
                        })
                        .ok_or_else(|| shell::ShellError::message("missing draft to revise"))?;
                    artifact["content"] = json!(format!(
                        "{}\n{}",
                        prior["data"]["artifact"]["content"].as_str().unwrap(),
                        args["feedback"]
                            .as_str()
                            .unwrap_or("Cross-chapter consistency revision")
                    ));
                }
            }
            if tool == "update_references" && section == "CHAPTER_3" {
                let evaluation = entries
                    .iter()
                    .find(|entry| {
                        entry["tool_name"] == "write_section"
                            && entry["data"]["section"] == "CHAPTER_4"
                    })
                    .ok_or_else(|| {
                        shell::ShellError::message("Chapter 3 referenced an incomplete evaluation")
                    })?;
                artifact["input_sections"] = json!({"CHAPTER_4":evaluation["data"]["artifact"]});
            }
            if tool == "insert_figure" {
                let figure = entries
                    .iter()
                    .find(|entry| {
                        entry["tool_name"] == "create_figure"
                            && entry["data"]["figure_id"] == args["figure_id"]
                    })
                    .ok_or_else(|| shell::ShellError::message("insertion has no creation"))?;
                artifact = figure["data"]["artifact"].clone();
            }
            let mut result = json!({"success":true,"tool_name":tool,"data":{
                "agent":agent,"section":section,"artifact":artifact
            }});
            if matches!(tool, "insert_figure" | "create_figure") {
                result["data"]["figure_id"] = artifact["id"].clone();
            }
            entries.push(result.clone());
            self.0.set(entries);
            Ok(result)
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn each_draft_order_delivers_the_actual_cross_chapter_inputs() -> ShellResult<()> {
    let source = preparation();
    let document_program = project::<DOCUMENT>(&source);
    let owner_program = project::<DOCUMENT_OWNER>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let references_program = project::<REFERENCES>(&source);
    let a_program = project::<WRITER_A_FILE>(&source);
    let b_program = project::<WRITER_B_FILE>(&source);
    let c_program = project::<WRITER_C_FILE>(&source);
    let figure_file_program = project::<FIGURE_FILE>(&source);
    let figure_owner_program = project::<FIGURE_OWNER>(&source);
    let figure_store_program = project::<FIGURE_STORE>(&source);
    let figure_creator_program = project::<FIGURE_CREATOR>(&source);
    let part_i_facts_program = project::<FACT_PART_I>(&source);
    let editorial_facts_program = project::<FACT_EDITORIAL>(&source);
    let third_facts_program = project::<FACT_3>(&source);
    let fourth_facts_program = project::<FACT_4>(&source);
    let fact_file_program = project::<FACT_FILE>(&source);
    let review_file_program = project::<REVIEW_FILE>(&source);
    let editor_program = project::<EDITOR>(&source);
    let editor_file_program = project::<EDITOR_FILE>(&source);

    let chapters = [0, 1, 2, 3];
    let orders = chapters.into_iter().flat_map(|first| {
        chapters
            .into_iter()
            .filter(move |second| *second != first)
            .flat_map(move |second| {
                chapters
                    .into_iter()
                    .filter(move |third| *third != first && *third != second)
                    .map(move |third| [first, second, third, 6 - first - second - third])
            })
    });
    for (index, order) in orders.enumerate() {
        let session = SessionId::new(86 + index as u32);
        let (bus, _, []) = Bus::open([], [(session, DOCUMENT), (session, FIGURE_STORE)]);
        let trace = Cell::new(Vec::new());
        let record = |event| {
            let mut rows = trace.take();
            rows.push(imp::projection::record(event));
            trace.set(rows);
        };
        let mut slab = vec![0; super::SESSION_STORAGE_BYTES];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&record);
        let mut owner = rv.enter(session, &owner_program)?;
        let document = rv.enter(session, &document_program)?;
        let mut drafting = rv.enter(session, &drafting_program)?;
        let mut references = rv.enter(session, &references_program)?;
        let mut a = rv.enter(session, &a_program)?;
        let mut b = rv.enter(session, &b_program)?;
        let mut c = rv.enter(session, &c_program)?;
        let mut figure_owner = rv.enter(session, &figure_owner_program)?;
        let store = rv.enter(session, &figure_store_program)?;
        let mut creator = rv.enter(session, &figure_creator_program)?;
        let mut figure = rv.enter(session, &figure_file_program)?;
        let mut part_i = rv.enter(session, &part_i_facts_program)?;
        let mut editorial_facts = rv.enter(session, &editorial_facts_program)?;
        let mut third_facts = rv.enter(session, &third_facts_program)?;
        let mut fourth_facts = rv.enter(session, &fourth_facts_program)?;
        let mut fact_file = rv.enter(session, &fact_file_program)?;
        let mut review_file = rv.enter(session, &review_file_program)?;
        let mut editor = rv.enter(session, &editor_program)?;
        let mut editor_file = rv.enter(session, &editor_file_program)?;
        let world = Journal::default();
        let trial = async {
            async {
                document_owner(&mut owner, document, &world)
                    .await
                    .map_err(|error| shell::ShellError::message(format!("owner: {error:?}")))
            };
            async {
                edit_document(&mut editor, &world)
                    .await
                    .map_err(|error| shell::ShellError::message(format!("editor: {error:?}")))
            };
            async {
                let evidence = decode(editor_file.recv::<PublicationMaterial>().await.map_err(
                    |error| shell::ShellError::message(format!("editor file: {error:?}")),
                )?)?;
                assert!(
                    evidence["reviews"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|review| review["success"] == true)
                );
                editor_file
                    .send::<Request<EDITORIAL>>(&ChapterArguments::new(
                        json!({"review_summary":"Current reviewed chapters"}),
                    )?)
                    .await?;
                editor_file.recv::<ResultFor<EDITORIAL>>().await?;
                editor_file
                    .send::<Request<COMBINE>>(&ChapterArguments::new(
                        json!({"sections":CHAPTERS.join(",")}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(editor_file.recv::<ResultFor<COMBINE>>().await?)?["data"]["status"],
                    "combined"
                );
                Ok(())
            };
            async {
                figure_owner.send::<Loan<FIGURE_STORE>>(store).await?;
                drop(figure_owner.recv::<Loan<FIGURE_STORE>>().await?);
                Ok(())
            };
            async { figure_store(&mut creator, &world).await };
            async {
                check_and_review(&mut part_i, &world)
                    .await
                    .map_err(|error| shell::ShellError::message(format!("facts part I: {error:?}")))
            };
            async { check_and_review(&mut editorial_facts, &world).await };
            async {
                check_chapter::<FACT_3, CHECK_3, MATERIAL_3, REVISE_3, MINOR_3, CHAPTER_3_OUTCOME>(
                    &mut third_facts,
                    &world,
                )
                .await
                .map_err(|error| shell::ShellError::message(format!("facts 3: {error:?}")))
            };
            async {
                check_chapter::<FACT_4, CHECK_4, MATERIAL_4, REVISE_4, MINOR_4, CHAPTER_4_OUTCOME>(
                    &mut fourth_facts,
                    &world,
                )
                .await
                .map_err(|error| shell::ShellError::message(format!("facts 4: {error:?}")))
            };
            async {
                let fourth = decode(review_file.recv::<ReviewInput>().await.map_err(|error| {
                    shell::ShellError::message(format!("review file chapter 4: {error:?}"))
                })?)?;
                review_file
                    .send::<ReviewRequest<CHECK_4>>(&ReviewArguments::new(
                        json!({"section_name":"CHAPTER_4"}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(review_file.recv::<ReviewInput>().await?)?["data"]["input_snapshot"],
                    fourth["data"]["input_snapshot"]
                );
                let third = decode(review_file.recv::<ReviewInput>().await?)?;
                review_file
                    .send::<ReviewRequest<CHECK_3>>(&ReviewArguments::new(
                        json!({"section_name":"CHAPTER_3"}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(review_file.recv::<ReviewInput>().await?)?["data"]["input_snapshot"],
                    third["data"]["input_snapshot"]
                );
                let part = decode(review_file.recv::<ReviewInput>().await?)?;
                review_file
                    .send::<ReviewRequest<CHECK_PART_I>>(&ReviewArguments::new(
                        json!({"section_name":"CHAPTERS_1_2"}),
                    )?)
                    .await?;
                let scoped = decode(review_file.recv::<ReviewInput>().await?)?;
                assert_eq!(scoped["data"]["status"], "reviewed");
                assert_eq!(
                    scoped["data"]["input_snapshot"],
                    part["data"]["input_snapshot"]
                );
                let outcome = decode(review_file.recv::<ReviewInput>().await?)?;
                assert_eq!(outcome["data"]["status"], "combined");
                Ok(())
            };
            async {
                let fourth = decode(fact_file.recv::<FactInput>().await.map_err(|error| {
                    shell::ShellError::message(format!("fact file chapter 4: {error:?}"))
                })?)?;
                fact_file
                    .send::<Request<CHECK_4>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTER_4"}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(fact_file.recv::<ResultFor<CHECK_4>>().await?)?["data"]["input_snapshot"],
                    fourth
                );
                assert_eq!(
                    decode(fact_file.recv::<FactInput>().await?)?["data"]["scope"],
                    "CHAPTER_4"
                );
                let third = decode(fact_file.recv::<FactInput>().await?)?;
                fact_file
                    .send::<Request<CHECK_3>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTER_3"}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(fact_file.recv::<ResultFor<CHECK_3>>().await?)?["data"]["input_snapshot"],
                    third
                );
                assert_eq!(
                    decode(fact_file.recv::<FactInput>().await?)?["data"]["scope"],
                    "CHAPTER_3"
                );
                let part = decode(fact_file.recv::<FactInput>().await?)?;
                fact_file
                    .send::<Request<CHECK_PART_I>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTERS_1_2"}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(fact_file.recv::<ResultFor<CHECK_PART_I>>().await?)?["data"]["input_snapshot"],
                    part
                );
                let scoped = decode(fact_file.recv::<FactInput>().await?)?;
                assert_eq!(scoped["data"]["status"], "reviewed");
                let outcome = decode(fact_file.recv::<FactInput>().await?)?;
                assert_eq!(outcome["data"]["status"], "combined");
                Ok(())
            };
            async {
                let drafts = write_drafts(&mut drafting, &world).await?;
                assert_eq!(drafts.len(), 4);
                Ok(())
            };
            async {
                let references = update_references(&mut references, &world).await?;
                assert!(references[2]["input_sections"]["CHAPTER_4"]["id"].is_string());
                Ok(())
            };
            async {
                figure
                    .send::<Request<CREATE_3>>(&ChapterArguments::new(
                        json!({"description":"Contribution figure"}),
                    )?)
                    .await?;
                let third = decode(figure.recv::<ResultFor<CREATE_3>>().await?)?;
                figure
                    .send::<Request<CREATE_4>>(&ChapterArguments::new(
                        json!({"description":"Evaluation figure"}),
                    )?)
                    .await?;
                let fourth = decode(figure.recv::<ResultFor<CREATE_4>>().await?)?;
                figure
                    .send::<Request<CREATE_1>>(&ChapterArguments::new(
                        json!({"description":"Introduction figure"}),
                    )?)
                    .await?;
                let first = decode(figure.recv::<ResultFor<CREATE_1>>().await?)?;
                figure
                    .send::<FigureLookup<INSERT_3>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTER_3","figure_id":third["data"]["figure_id"]}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(figure.recv::<ResultFor<INSERT_3>>().await?)?["data"]["artifact"],
                    third["data"]["artifact"]
                );
                figure
                    .send::<FigureLookup<INSERT_1>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTER_1","figure_id":first["data"]["figure_id"]}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(figure.recv::<ResultFor<INSERT_1>>().await?)?["data"]["artifact"],
                    first["data"]["artifact"]
                );
                figure
                    .send::<FigureLookup<INSERT_4>>(&ChapterArguments::new(
                        json!({"section_name":"CHAPTER_4","figure_id":fourth["data"]["figure_id"]}),
                    )?)
                    .await?;
                assert_eq!(
                    decode(figure.recv::<ResultFor<INSERT_4>>().await?)?["data"]["artifact"],
                    fourth["data"]["artifact"]
                );
                Ok(())
            };
            async {
                a.send::<Request<REFERENCES_1>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[0]}),
                )?)
                .await?;
                a.recv::<ResultFor<REFERENCES_1>>().await?;
                {
                    use std::{
                        future::{Future, poll_fn},
                        pin::pin,
                        task::Poll,
                    };
                    let args = ChapterArguments::new(json!({"section_name":CHAPTERS[2]}))?;
                    let mut premature = pin!(b.send::<Request<REFERENCES_3>>(&args));
                    let accepted = poll_fn(|cx| {
                        Poll::Ready(matches!(premature.as_mut().poll(cx), Poll::Ready(Ok(_))))
                    })
                    .await;
                    assert!(!accepted);
                }
                let mut writes = [const { Value::Null }; 4];
                for chapter in order {
                    let args =
                        json!({"section_name":CHAPTERS[chapter],"content_notes":CHAPTERS[chapter]});
                    writes[chapter] = match chapter {
                        0 => {
                            a.send::<Request<WRITE_1>>(&ChapterArguments::new(args)?)
                                .await?;
                            decode(a.recv::<ResultFor<WRITE_1>>().await?)?
                        }
                        1 => {
                            a.send::<Request<WRITE_2>>(&ChapterArguments::new(args)?)
                                .await?;
                            decode(a.recv::<ResultFor<WRITE_2>>().await?)?
                        }
                        2 => {
                            b.send::<Request<WRITE_3>>(&ChapterArguments::new(args)?)
                                .await?;
                            decode(b.recv::<ResultFor<WRITE_3>>().await?)?
                        }
                        3 => {
                            c.send::<Request<WRITE_4>>(&ChapterArguments::new(args)?)
                                .await?;
                            decode(c.recv::<ResultFor<WRITE_4>>().await?)?
                        }
                        _ => unreachable!(),
                    };
                }
                c.send::<Request<REFERENCES_4>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[3]}),
                )?)
                .await?;
                c.recv::<ResultFor<REFERENCES_4>>().await?;
                assert_eq!(
                    decode(c.recv::<AuthorOutcome<CHAPTER_4_OUTCOME>>().await?)?["data"]["status"],
                    "reviewed"
                );
                assert_eq!(
                    decode(a.recv::<EvaluationToPartI>().await?)?,
                    writes[3]["data"]["artifact"]
                );
                assert_eq!(
                    decode(b.recv::<EvaluationToChapter3>().await?)?,
                    writes[3]["data"]["artifact"]
                );
                b.send::<Request<REFERENCES_3>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[2]}),
                )?)
                .await?;
                let actual = decode(b.recv::<ResultFor<REFERENCES_3>>().await?)?;
                assert_eq!(
                    actual["data"]["artifact"]["input_sections"]["CHAPTER_4"],
                    writes[3]["data"]["artifact"]
                );
                assert_eq!(
                    decode(b.recv::<AuthorOutcome<CHAPTER_3_OUTCOME>>().await?)?["data"]["status"],
                    "reviewed"
                );
                assert_eq!(
                    decode(a.recv::<ContributionToPartI>().await?)?,
                    writes[2]["data"]["artifact"]
                );
                a.send::<Request<REFERENCES_2>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[1]}),
                )?)
                .await?;
                a.recv::<ResultFor<REFERENCES_2>>().await?;
                a.send::<Request<FINALIZE_2>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[1],"feedback":"Frame the actual results"}),
                )?)
                .await?;
                let final_two = decode(a.recv::<ResultFor<FINALIZE_2>>().await?)?;
                a.send::<Request<FINALIZE_1>>(&ChapterArguments::new(
                    json!({"section_name":CHAPTERS[0]}),
                )?)
                .await?;
                let final_one = decode(a.recv::<ResultFor<FINALIZE_1>>().await?)?;
                let inputs = json!({(CHAPTERS[2]):writes[2]["data"]["artifact"], (CHAPTERS[3]):writes[3]["data"]["artifact"]});
                for final_chapter in [&final_one, &final_two] {
                    assert_eq!(final_chapter["input_sections"], inputs);
                }
                assert_ne!(final_two["id"], writes[1]["data"]["artifact"]["id"]);
                assert_eq!(
                    final_one["id"] == writes[0]["data"]["artifact"]["id"],
                    writes[0]["data"]["artifact"]["input_sections"] == inputs
                );
                let checked = decode(a.recv::<AuthorOutcome>().await?)?;
                assert_eq!(checked["tool_name"], "review_sections");
                assert_eq!(checked["success"], true);
                let outcome = decode(a.recv::<AuthorOutcome>().await?)?;
                assert_eq!(outcome["data"]["status"], "combined");
                Ok(())
            };
            Ok(())
        };
        if let Err(error) = hibana_runtime::race(
            trial,
            hibana_runtime::host::deadline(Duration::from_secs(5)),
        )
        .await
        {
            let frames: Vec<_> = bus
                .transport
                .take_observations()
                .map(|mut frame| {
                    frame
                        .as_object_mut()
                        .expect("transport frame object")
                        .remove("payload");
                    frame
                })
                .collect();
            eprintln!("order={order:?} frames={frames:?} trace={:?}", trace.take());
            return Err(error);
        }
        let entries = world.0.take();
        let initial_one = entries
            .iter()
            .find(|entry| {
                entry["tool_name"] == "write_section" && entry["data"]["section"] == CHAPTERS[0]
            })
            .unwrap();
        let needs_reframe = initial_one["data"]["artifact"]["input_sections"]
            .as_object()
            .unwrap()
            .len()
            != 2;
        assert_eq!(entries.len(), 23 + usize::from(needs_reframe));
        let actual: Vec<_> = entries
            .iter()
            .filter(|e| e["tool_name"] == "write_section")
            .map(|e| e["data"]["section"].as_str().unwrap())
            .collect();
        assert_eq!(actual, order.map(|chapter| CHAPTERS[chapter]));
    }
    Ok(())
}
