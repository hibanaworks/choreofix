use super::{
    global::*,
    preparation,
    publication::{self, SECTIONS},
    quality::{self, AUTHORS, ChapterArguments, TOPICS},
    tests::{editor_fixture, fixture_terminal},
};
use crate::{
    imp::json::decode,
    shell::{
        ShellError, ShellResult,
        global::Loan,
        imp::{
            bus::Bus,
            service::{ReceivedEffects, SimulatorOperation},
        },
        localside::file_metadata,
    },
};
use hibana::{
    Endpoint, EndpointBorrow,
    runtime::{SessionKitStorage, ids::SessionId, program::project, tap},
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

struct World {
    effects: Cell<Vec<Value>>,
    rejects: [usize; 4],
}

fn artifact<'a>(effects: &'a [Value], section: &str, tools: &[&str]) -> &'a Value {
    &effects
        .iter()
        .rev()
        .find(|event| {
            event["data"]["section"] == section
                && event["tool_name"]
                    .as_str()
                    .is_some_and(|tool| tools.contains(&tool))
        })
        .expect("completed fixture artifact")["data"]["artifact"]
}

impl ReceivedEffects for World {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let input = crate::imp::json::decode(received.as_bytes());
        async move {
            let args = input?;
            let (agent, tool) = P::RPC;
            hibana_runtime::yield_now().await;
            let mut effects = self.effects.take();
            let sequence = effects.len() + 1;
            let id = format!("{tool}:{sequence}");
            let sources = effects
                .iter()
                .filter(|event| event["tool_name"] == "research_topic")
                .map(|event| {
                    (
                        event["data"]["topic"].as_str().unwrap(),
                        &event["data"]["artifact"],
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            let section = args["section_name"]
                .as_str()
                .or_else(|| args["manuscript"].as_str());
            let mut success = true;
            let data = match tool {
                "research_topic" => {
                    let index = AUTHORS.iter().position(|&name| name == agent).unwrap();
                    assert_eq!(args["topic"], TOPICS[index]);
                    json!({"agent":agent,"topic":args["topic"],"artifact":{"id":id,"content":format!("Research by {agent}")}})
                }
                "write_section" | "update_references" | "revise_section" => {
                    let section = section.unwrap();
                    let index = SECTIONS.iter().position(|&name| name == section).unwrap();
                    assert_eq!(agent, AUTHORS[index]);
                    let content =
                        match tool {
                            "write_section" => args["content_notes"].as_str().unwrap().to_owned(),
                            "update_references" => format!("Bibliography for {section}"),
                            _ => {
                                format!(
                                    "{}\n{}",
                                    artifact(
                                        &effects,
                                        section,
                                        &["write_section", "revise_section"]
                                    )["content"]
                                        .as_str()
                                        .unwrap(),
                                    args["feedback"].as_str().unwrap()
                                )
                            }
                        };
                    let mut artifact = json!({"id":id,"content":content,"research_inputs":{TOPICS[index]:sources[TOPICS[index]]}});
                    if tool == "revise_section" {
                        artifact["feedback"] = args["feedback"].clone();
                    }
                    json!({"agent":agent,"section":section,"artifact":artifact})
                }
                "create_figure" => {
                    assert_eq!(agent, "DATA_ANALYST");
                    let inputs: serde_json::Map<_, _> = SECTIONS
                        .iter()
                        .zip(TOPICS)
                        .filter_map(|(&chapter, topic)| {
                            sources
                                .get(topic)
                                .map(|source| (chapter.to_owned(), json!({topic:source})))
                        })
                        .collect();
                    json!({"agent":agent,"figure_id":id,"artifact":{"id":id,"content":args["description"],"research_inputs":inputs}})
                }
                "insert_figure" => {
                    assert_eq!(agent, "DATA_ANALYST");
                    let created = effects
                        .iter()
                        .find(|event| {
                            event["tool_name"] == "create_figure"
                                && event["data"]["figure_id"] == args["figure_id"]
                        })
                        .unwrap();
                    json!({"agent":agent,"section":section.unwrap(),"figure_id":args["figure_id"],"artifact":created["data"]["artifact"]})
                }
                "peer_review" => {
                    assert_eq!(agent, "REVIEWER");
                    let section = section.unwrap();
                    let index = SECTIONS.iter().position(|&name| name == section).unwrap();
                    let previous = effects
                        .iter()
                        .filter(|event| {
                            event["tool_name"] == "peer_review" && event["data"]["scope"] == section
                        })
                        .count();
                    success = previous >= self.rejects[index];
                    json!({"agent":agent,"scope":section,"review_id":id,"input_snapshot":{
                        "sections":{section:artifact(&effects,section,&["write_section","revise_section"])},
                        "references":{section:artifact(&effects,section,&["update_references"])},
                        "figures":{section:artifact(&effects,section,&["insert_figure"])},
                    }})
                }
                "make_editorial_decision" | "combine_sections" => {
                    assert_eq!(agent, "EDITOR_IN_CHIEF");
                    let mut snapshot = json!({"sections":{},"references":{},"figures":{}});
                    let mut reviews = json!({});
                    for name in SECTIONS {
                        snapshot["sections"][name] =
                            artifact(&effects, name, &["write_section", "revise_section"]).clone();
                        snapshot["references"][name] =
                            artifact(&effects, name, &["update_references"]).clone();
                        snapshot["figures"][name] =
                            artifact(&effects, name, &["insert_figure"]).clone();
                        let review = effects
                            .iter()
                            .rev()
                            .find(|event| {
                                event["tool_name"] == "peer_review"
                                    && event["data"]["scope"] == name
                            })
                            .expect("actual review");
                        assert_eq!(review["success"], true);
                        reviews[name] = review["data"]["review_id"].clone();
                    }
                    if tool == "make_editorial_decision" {
                        json!({"agent":agent,"decision_type":"finalize","decision_id":"editorial:1",
                           "review_summary":args["review_summary"],"input_snapshot":snapshot,"review_receipts":reviews})
                    } else {
                        let decision = effects.last().expect("actual editorial effect");
                        assert_eq!(decision["tool_name"], "make_editorial_decision");
                        let sections: Vec<_> = args["sections"]
                            .as_str()
                            .unwrap()
                            .split(',')
                            .map(str::trim)
                            .collect();
                        json!({"agent":agent,"artifact_id":"combined:2","sections":sections,
                           "input_snapshot":snapshot,"review_receipts":reviews,
                           "editorial_receipts":{"finalize":decision["data"]["decision_id"]}})
                    }
                }
                _ => panic!("unexpected survey effect {tool}"),
            };
            let result = json!({"tool_name":tool,"success":success,"data":data});
            let mut journal = result.clone();
            journal["arguments"] = args;
            effects.push(journal);
            self.effects.set(effects);
            Ok(result)
        }
    }
}

async fn author_file<'r, const SECTION: u8, const FILE: u8>(
    file: &mut Endpoint<'r, FILE>,
    references_first: bool,
) -> ShellResult<()> {
    let index = usize::from(SECTION) - 1;
    let chapter = SECTIONS[index];
    let research = ChapterArguments::from_value(&json!({"topic":TOPICS[index]}))?;
    file.send::<Research<SECTION>>(&research).await?;
    let research = decode(file.recv::<Researched<SECTION>>().await?)?;
    assert_eq!(research["data"]["topic"], TOPICS[index]);

    let write = ChapterArguments::from_value(
        &json!({"section_name":chapter,"content_notes":format!("Original notes on {chapter}")}),
    )?;
    let references = ChapterArguments::from_value(&json!({"section_name":chapter}))?;
    if references_first {
        file.send::<References<SECTION>>(&references).await?;
        file.recv::<Referenced<SECTION>>().await?;
        file.send::<Write<SECTION>>(&write).await?;
        file.recv::<Written<SECTION>>().await?;
    } else {
        file.send::<Write<SECTION>>(&write).await?;
        file.recv::<Written<SECTION>>().await?;
        file.send::<References<SECTION>>(&references).await?;
        file.recv::<Referenced<SECTION>>().await?;
    }
    loop {
        let result = decode(file.recv::<AuthorNotice<SECTION>>().await?)?;
        assert_eq!(result["data"]["scope"], chapter);
        if result["success"] == true {
            return Ok(());
        }
        let revision = ChapterArguments::from_value(&json!({
            "section_name":chapter,"feedback":format!("Address {}",result["data"]["review_id"])
        }))?;
        file.send::<Revise<SECTION>>(&revision).await?;
        let actual = decode(file.recv::<Revised<SECTION>>().await?)?;
        assert_eq!(actual["data"]["section"], chapter);
    }
}

async fn analyst_file<const SECTION: u8>(
    file: &mut EndpointBorrow<'_, '_, ANALYST_FILE>,
) -> ShellResult<()> {
    let index = usize::from(SECTION) - 1;
    let chapter = SECTIONS[index];
    let research = decode(file.recv::<ResearchForAnalyst<SECTION>>().await?)?;
    assert_eq!(research["data"]["topic"], TOPICS[index]);
    let figure = ChapterArguments::from_value(
        &json!({"description":format!("Research sources for {chapter}")}),
    )?;
    file.send::<CreateFigure<SECTION>>(&figure).await?;
    let figure = decode(file.recv::<FigureCreated<SECTION>>().await?)?;
    let input = decode(file.recv::<InsertionInput<SECTION>>().await?)?;
    assert_eq!(input["figures"][chapter], figure["data"]["artifact"]);
    let args = ChapterArguments::from_value(
        &json!({"section_name":chapter,"figure_id":figure["data"]["figure_id"]}),
    )?;
    file.send::<InsertFigure<SECTION>>(&args).await?;
    let inserted = decode(file.recv::<Inserted<SECTION>>().await?)?;
    assert_eq!(inserted["data"]["artifact"], figure["data"]["artifact"]);
    Ok(())
}

async fn review_file<const SECTION: u8>(
    file: &mut EndpointBorrow<'_, '_, REVIEW_FILE>,
) -> ShellResult<()> {
    let chapter = SECTIONS[usize::from(SECTION) - 1];
    let args = ChapterArguments::from_value(&json!({"manuscript":chapter}))?;
    loop {
        let current = decode(file.recv::<ReviewInput<SECTION>>().await?)?;
        file.send::<Review<SECTION>>(&args).await?;
        let verdict = decode(file.recv::<ReviewResult<SECTION>>().await?)?;
        assert_eq!(verdict["data"]["input_snapshot"], current);
        if verdict["success"] == true {
            return Ok(());
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn four_chapters_publish_through_the_wasm_editor_after_zero_to_three_rejections()
-> ShellResult<()> {
    let source = wasi_publication(chapters());
    let module = editor_fixture()?;
    let a_program = project::<AUTHOR_A>(&source);
    let b_program = project::<AUTHOR_B>(&source);
    let c_program = project::<AUTHOR_C>(&source);
    let d_program = project::<AUTHOR_D>(&source);
    let section_a_program = project::<SECTION_A>(&source);
    let section_b_program = project::<SECTION_B>(&source);
    let section_c_program = project::<SECTION_C>(&source);
    let section_d_program = project::<SECTION_D>(&source);
    let a_file_program = project::<AUTHOR_FILE_A>(&source);
    let b_file_program = project::<AUTHOR_FILE_B>(&source);
    let c_file_program = project::<AUTHOR_FILE_C>(&source);
    let d_file_program = project::<AUTHOR_FILE_D>(&source);
    let database_program = project::<DATABASE>(&source);
    let figures_program = project::<FIGURES>(&source);
    let reviewer_program = project::<REVIEWER>(&source);
    let reviewer_file_program = project::<REVIEW_FILE>(&source);
    let analyst_program = project::<ANALYST>(&source);
    let analyst_file_program = project::<ANALYST_FILE>(&source);
    let editor_program = project::<EDITOR>(&source);
    let editor_data_program = project::<EDITOR_DATA>(&source);
    let editor_file_program = project::<EDITOR_FILE>(&source);
    let editor_io_program = project::<EDITOR_IO>(&source);
    let editor_vm_program = project::<EDITOR_VM>(&source);
    let world = World {
        effects: Cell::new(Vec::new()),
        rejects: [0, 3, 1, 2],
    };
    let session = SessionId::new(302);
    let (bus, _, [terminal]) = Bus::open(
        ["EDITOR_IN_CHIEF"],
        [
            SECTION_A,
            SECTION_B,
            SECTION_C,
            SECTION_D,
            DATABASE,
            FIGURES,
            EDITOR_FILE,
        ]
        .map(|role| (session, role)),
    );
    let records = Cell::new(Vec::new());
    let record = |event: tap::TapEvent| {
        let mut rows = records.take();
        rows.push(event);
        records.set(rows);
    };
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&record);
    let observation = rv.tap();
    let mut a = rv.enter(session, &a_program)?;
    let mut b = rv.enter(session, &b_program)?;
    let mut c = rv.enter(session, &c_program)?;
    let mut d = rv.enter(session, &d_program)?;
    let section_a = rv.enter(session, &section_a_program)?;
    let section_b = rv.enter(session, &section_b_program)?;
    let section_c = rv.enter(session, &section_c_program)?;
    let section_d = rv.enter(session, &section_d_program)?;
    let mut a_file = rv.enter(session, &a_file_program)?;
    let mut b_file = rv.enter(session, &b_file_program)?;
    let mut c_file = rv.enter(session, &c_file_program)?;
    let mut d_file = rv.enter(session, &d_file_program)?;
    let mut database = rv.enter(session, &database_program)?;
    let mut figures = rv.enter(session, &figures_program)?;
    let mut reviewer = rv.enter(session, &reviewer_program)?;
    let mut reviewer_input = rv.enter(session, &reviewer_file_program)?;
    let mut analyst = rv.enter(session, &analyst_program)?;
    let mut analyst_input = rv.enter(session, &analyst_file_program)?;
    let mut editor = rv.enter(session, &editor_program)?;
    let mut editor_data = rv.enter(session, &editor_data_program)?;
    let editor_file = rv.enter(session, &editor_file_program)?;
    let editor_vm = rv.enter(session, &editor_vm_program)?;
    let mut editor_io = rv.enter(session, &editor_io_program)?;

    let participants = async {
        async {
            preparation::references(&mut database, &world).await?;
        };
        async {
            preparation::figures(&mut figures, &world).await?;
        };
        async {
            quality::author::<SECTION_A, AUTHOR_A>(&mut a, section_a, &world).await?;
        };
        async {
            quality::author::<SECTION_B, AUTHOR_B>(&mut b, section_b, &world).await?;
        };
        async {
            quality::author::<SECTION_C, AUTHOR_C>(&mut c, section_c, &world).await?;
        };
        async {
            quality::author::<SECTION_D, AUTHOR_D>(&mut d, section_d, &world).await?;
        };
        async {
            author_file::<SECTION_A, AUTHOR_FILE_A>(&mut a_file, false).await?;
        };
        async {
            author_file::<SECTION_B, AUTHOR_FILE_B>(&mut b_file, true).await?;
        };
        async {
            author_file::<SECTION_C, AUTHOR_FILE_C>(&mut c_file, false).await?;
        };
        async {
            author_file::<SECTION_D, AUTHOR_FILE_D>(&mut d_file, true).await?;
        };
        async {
            let [mut a, mut b, mut c, mut d] = analyst.par()?;
            async {
                preparation::insert::<SECTION_A>(&mut a, &world).await?;
            };
            async {
                preparation::insert::<SECTION_B>(&mut b, &world).await?;
            };
            async {
                preparation::insert::<SECTION_C>(&mut c, &world).await?;
            };
            async {
                preparation::insert::<SECTION_D>(&mut d, &world).await?;
            };
            Ok(())
        };
        async {
            let [mut a, mut b, mut c, mut d] = analyst_input.par()?;
            async {
                analyst_file::<SECTION_A>(&mut a).await?;
            };
            async {
                analyst_file::<SECTION_B>(&mut b).await?;
            };
            async {
                analyst_file::<SECTION_C>(&mut c).await?;
            };
            async {
                analyst_file::<SECTION_D>(&mut d).await?;
            };
            Ok(())
        };
        async {
            let [mut a, mut b, mut c, mut d] = reviewer.par()?;
            async {
                quality::review::<SECTION_A>(&mut a, &world).await?;
            };
            async {
                quality::review::<SECTION_B>(&mut b, &world).await?;
            };
            async {
                quality::review::<SECTION_C>(&mut c, &world).await?;
            };
            async {
                quality::review::<SECTION_D>(&mut d, &world).await?;
            };
            Ok(())
        };
        async {
            let [mut a, mut b, mut c, mut d] = reviewer_input.par()?;
            async {
                review_file::<SECTION_A>(&mut a).await?;
            };
            async {
                review_file::<SECTION_B>(&mut b).await?;
            };
            async {
                review_file::<SECTION_C>(&mut c).await?;
            };
            async {
                review_file::<SECTION_D>(&mut d).await?;
            };
            Ok(())
        };
        async {
            let file = editor.recv::<Loan<EDITOR_FILE>>().await?;
            async {
                let actual = publication::run(&mut editor, &world).await?;
                assert_eq!(actual["data"]["artifact_id"], "combined:2");
            };
            async {
                super::editor_guest(&module, file, editor_vm).await?;
            };
            Ok(())
        };
        async {
            file_metadata::run(&mut editor_data, editor_file, EDITOR_OBJECTS).await?;
        };
        async {
            fixture_terminal(&mut editor_io, &bus, terminal).await?;
        };
        Ok::<(), ShellError>(())
    };
    hibana_runtime::race(
        participants,
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
    .await?;
    let effects = world.effects.take();
    assert_eq!(effects.len(), 38);
    assert_eq!(effects[37]["tool_name"], "combine_sections");
    let records = records.take();
    hibana_epf::Capture::new(&records, observation.checkpoint())
        .expect("complete capture of the combined native and WASI fixture");
    println!(
        "38 completed effects, {} complete tap records",
        records.len()
    );
    for (chapter, revisions) in SECTIONS.into_iter().zip(world.rejects) {
        let reviews: Vec<_> = effects
            .iter()
            .filter(|r| r["tool_name"] == "peer_review" && r["data"]["scope"] == chapter)
            .collect();
        assert_eq!(reviews.len(), revisions + 1);
        for pair in reviews.windows(2) {
            assert_eq!(pair[0]["success"], false);
            assert_ne!(
                pair[0]["data"]["input_snapshot"]["sections"][chapter]["id"],
                pair[1]["data"]["input_snapshot"]["sections"][chapter]["id"]
            );
        }
        assert_eq!(reviews.last().unwrap()["success"], true);
    }
    if let Some(directory) = std::env::var_os("CHOREFIX_SURVEY_FIXTURE_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir(&directory)?;
        let snapshot: Vec<_> = tap::projected(&editor_program).collect();
        for (name, rows) in [
            ("protocol.bin", snapshot.as_slice()),
            ("producer.bin", records.as_slice()),
        ] {
            let bytes: Vec<_> = rows.iter().flat_map(|row| <[u8; 16]>::from(*row)).collect();
            std::fs::write(directory.join(name), bytes)?;
        }
        let frames: Vec<_> = bus.transport.take_observations().collect();
        std::fs::write(
            directory.join("run.json"),
            serde_json::to_vec_pretty(&json!({
                "task":"3H","session":session.raw(),"checkpoint":observation.checkpoint(),
                "derivation_capacity":records.len()*2+1,"effects":effects,
                "frames":frames,"wasm_guests":1,"model_calls":0,"scored_runs":0,
            }))
            .map_err(|error| ShellError::message(error.to_string()))?,
        )?;
    }
    Ok(())
}
