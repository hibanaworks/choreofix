//! Whole-source fixtures. Native scripted participants are not AI or Wasm guests.
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, report_arguments::ReportArguments, service::Domain},
};
use hibana::{
    Endpoint, ParallelEndpoint,
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, program::project, tap},
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

use hibana_runtime::host::deadline;

fn record(row: tap::TapEvent) -> [u32; 5] {
    [
        row.ts(),
        u32::from(row.id()),
        u32::from(row.causal_key()),
        row.arg0(),
        row.arg1(),
    ]
}

#[test]
fn the_three_author_source_projects_all_actual_roles_without_duplicating_a_resource() {
    let source = super::global::source();
    assert!(tap::projected(&project::<AUTHOR_A>(&source)).count() > 0);
    assert!(tap::projected(&project::<AUTHOR_B>(&source)).count() > 0);
    assert!(tap::projected(&project::<AUTHOR_C>(&source)).count() > 0);
    assert!(tap::projected(&project::<FACT_CHECKER>(&source)).count() > 0);
    assert!(tap::projected(&project::<EDITOR>(&source)).count() > 0);
    assert!(tap::projected(&project::<RESEARCH_SERVICE>(&source)).count() > 0);
    assert!(tap::projected(&project::<DOCUMENT>(&source)).count() > 0);
    assert!(tap::projected(&project::<DATABASE>(&source)).count() > 0);
    assert!(tap::projected(&project::<SECTION_A>(&source)).count() > 0);
    assert!(tap::projected(&project::<SECTION_B>(&source)).count() > 0);
    assert!(tap::projected(&project::<SECTION_C>(&source)).count() > 0);
    assert!(tap::projected(&project::<REVIEW_SERVICE>(&source)).count() > 0);
    assert!(tap::projected(&project::<ARCHIVE>(&source)).count() > 0);
}

// Actual observed effect records and immutable failure injection only.
struct Effects {
    events: Cell<Vec<Value>>,
    failures: [[usize; 2]; 3],
}

fn latest<'a>(events: &'a [Value], tools: &[&str], name: &str) -> &'a Value {
    &events
        .iter()
        .rev()
        .find(|row| {
            row["event"] == "end"
                && row["tool"]
                    .as_str()
                    .is_some_and(|tool| tools.contains(&tool))
                && (row["result"]["data"]["section"] == name
                    || row["result"]["data"]["scope"] == name
                    || row["result"]["data"]["topic"] == name)
        })
        .expect("fixture requires the actual completed input")["result"]
}
impl Domain for Effects {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut events = self.events.take();
        let sequence = events.iter().filter(|r| r["event"] == "start").count() + 1;
        events.push(json!({"event":"start","actor":actor,"tool":tool,"arguments":arguments,"sequence":sequence}));
        self.events.set(events);
        for _ in 0..match tool {
            "research_topic" => 1,
            "fact_check" | "review_sections" => 4,
            "combine_sections" => 2,
            _ => 8,
        } {
            hibana_runtime::yield_now().await;
        }
        let mut events = self.events.take();
        let result = if tool == "combine_sections" {
            let mut sections = json!({});
            let mut references = json!({});
            let mut reviews = json!({"facts":{},"reviews":{}});
            for name in imp::SECTIONS {
                sections[name] =
                    latest(&events, &["write_section", "revise_section"], name)["data"]["artifact"]
                        .clone();
                references[name] =
                    latest(&events, &["update_references"], name)["data"]["artifact"].clone();
                reviews["facts"][name] = latest(&events, &["fact_check"], name).clone();
                reviews["reviews"][name] = latest(&events, &["review_sections"], name).clone();
            }
            super::publication_tests::reply(&arguments, (&sections, &references, &reviews))
        } else {
            let author = if tool == "research_topic" {
                imp::AUTHORS
                    .iter()
                    .position(|name| *name == actor)
                    .expect("researcher")
            } else {
                imp::SECTIONS
                    .iter()
                    .position(|name| Some(*name) == arguments["section_name"].as_str())
                    .expect("section")
            };
            let name = imp::SECTIONS[author];
            match tool {
                "research_topic" => {
                    json!({"tool_name":tool,"success":true,"data":{"agent":actor,"topic":arguments["topic"],"artifact":{"id":format!("research:{sequence}"),"content":format!("real fixture source {sequence}")}}})
                }
                "write_section" | "revise_section" | "update_references" => {
                    let research = latest(&events, &["research_topic"], imp::TOPICS[author]);
                    let mut artifact = json!({"id":format!("{tool}:{sequence}"),"content":format!("material for {name}"),
                        "research_inputs":{imp::TOPICS[author]:research["data"]["artifact"]}});
                    if tool == "write_section" {
                        artifact["content"] = arguments["content_notes"].clone();
                    }
                    if tool == "revise_section" {
                        artifact["content"] = json!(format!(
                                "{}\n{}",
                                latest(&events, &["write_section", "revise_section"], name)["data"]
                                    ["artifact"]["content"]
                                    .as_str()
                                    .unwrap(),
                                arguments["feedback"].as_str().unwrap()
                            ));
                        artifact["feedback"] = arguments["feedback"].clone();
                    }
                    json!({"tool_name":tool,"success":true,"data":{"agent":actor,"section":name,"artifact":artifact}})
                }
                "fact_check" | "review_sections" => {
                    let snapshot = imp::snapshot(
                        author as u8,
                        latest(&events, &["write_section", "revise_section"], name),
                        latest(&events, &["update_references"], name),
                    )
                    .map_err(|e| e.to_string())?;
                    let occurrence = events
                        .iter()
                        .filter(|row| {
                            row["event"] == "end"
                                && row["tool"] == tool
                                && (row["result"]["data"]["section"] == name
                                    || row["result"]["data"]["scope"] == name)
                        })
                        .count();
                    let success =
                        occurrence >= self.failures[author][usize::from(tool == "review_sections")];
                    if tool == "fact_check" {
                        json!({"tool_name":tool,"success":success,"data":{"agent":actor,"section":name,"check_id":format!("facts:{sequence}"),"input_snapshot":snapshot}})
                    } else {
                        let facts = latest(&events, &["fact_check"], name);
                        json!({"tool_name":tool,"success":success,"data":{"agent":actor,"scope":name,"review_id":format!("review:{sequence}"),"input_snapshot":snapshot,
                            "fact_receipts":{name:facts["data"]["check_id"]}}})
                    }
                }
                _ => return Err("unexpected effect".into()),
            }
        };
        events.push(json!({"event":"end","actor":actor,"tool":tool,"arguments":arguments,"sequence":sequence,"result":result}));
        self.events.set(events);
        Ok(result)
    }
}

// Each function below is one entire controlled fixture participant, never a
// product phase helper. Production effect receivers are called unchanged.
#[hibana_runtime::scope]
async fn author<'r, const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut Endpoint<'r, AUTHOR>,
    mut section: Endpoint<'r, SECTION>,
    start: usize,
) -> ShellResult<()> {
    for _ in 0..start {
        hibana_runtime::yield_now().await;
    }
    let research = imp::Arguments::<AUTHOR, 1>::from_bytes(&imp::encode(
        &json!({"topic":imp::TOPICS[usize::from(AUTHOR)]}),
    )?)?;
    section.send::<C::Research>(&research).await?;
    let researched = endpoint.recv::<C::Researched>().await?.into_value();
    imp::research(AUTHOR, &researched)?;
    async {
        let [mut writing,mut references]=section.par()?;
        async {
            let args=json!({"section_name":imp::SECTIONS[usize::from(AUTHOR)],"content_notes":format!("independent findings {AUTHOR}")});
            writing.send::<C::Write>(&imp::Arguments::<AUTHOR,2>::from_bytes(&imp::encode(&args)?)?).await?;
            let result=writing.recv::<C::Written>().await?.into_value();
            imp::material(AUTHOR,2,&result,(&args,&researched))?;
        };
        async {
            let args=json!({"section_name":imp::SECTIONS[usize::from(AUTHOR)]});
            references.send::<C::References>(&imp::Arguments::<AUTHOR,4>::from_bytes(&imp::encode(&args)?)?).await?;
            let result=references.recv::<C::Referenced>().await?.into_value();
            imp::material(AUTHOR,4,&result,(&args,&researched))?;
        };
        Ok::<(),ShellError>(())
    }.await?;
    endpoint.send::<Section<SECTION>>(section).await?;
    loop {
        let branch = endpoint.offer().await?;
        let rejected = match branch.label() {
            label if label == C::FactRejectionForAuthor::LOGICAL_LABEL => branch
                .recv::<C::FactRejectionForAuthor>()
                .await?
                .into_value(),
            label if label == C::ReviewRejectionForAuthor::LOGICAL_LABEL => branch
                .recv::<C::ReviewRejectionForAuthor>()
                .await?
                .into_value(),
            label if label == C::FinalForAuthor::LOGICAL_LABEL => {
                assert_eq!(
                    branch.recv::<C::FinalForAuthor>().await?.into_value()["success"],
                    true
                );
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected author input")),
        };
        assert_eq!(rejected["success"], false);
        let mut section = endpoint.recv::<Section<SECTION>>().await?;
        let args = json!({"section_name":imp::SECTIONS[usize::from(AUTHOR)],"feedback":format!("address actual {}",rejected["tool_name"])});
        section
            .send::<C::Revise>(&imp::Arguments::<AUTHOR, 3>::from_bytes(&imp::encode(
                &args,
            )?)?)
            .await?;
        let revised = section.recv::<C::Revised>().await?.into_value();
        imp::material(AUTHOR, 3, &revised, (&args, &researched))?;
        endpoint.send::<Section<SECTION>>(section).await?;
    }
}

#[hibana_runtime::scope]
async fn fact<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut ParallelEndpoint<FACT_CHECKER>,
    early_archive: bool,
) -> ShellResult<()> {
    let (mut written, references) = async {
        let [mut written, mut references] = endpoint.par()?;
        async { Ok::<_, ShellError>(written.recv::<C::MaterialForFact>().await?.into_value()) };
        async {
            Ok::<_, ShellError>(
                references
                    .recv::<C::ReferencesForFact>()
                    .await?
                    .into_value(),
            )
        };
    }
    .await?;
    let mut section = endpoint.recv::<Section<SECTION>>().await?;
    loop {
        let snapshot = imp::snapshot(AUTHOR, &written, &references)?;
        endpoint
            .send::<C::FactMaterial>(&imp::Material::new(snapshot.clone())?)
            .await?;
        let args = json!({"section_name":imp::SECTIONS[usize::from(AUTHOR)]});
        section
            .send::<C::FactRequest>(&imp::Arguments::<AUTHOR, 5>::from_bytes(&imp::encode(
                &args,
            )?)?)
            .await?;
        endpoint.send::<Section<SECTION>>(section).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::FactRejected::LOGICAL_LABEL => {
                let actual = branch.recv::<C::FactRejected>().await?.into_value();
                imp::checked(AUTHOR, &actual, &snapshot)?;
            }
            label if label == C::FactPassed::LOGICAL_LABEL => {
                let facts = branch.recv::<C::FactPassed>().await?.into_value();
                imp::checked(AUTHOR, &facts, &snapshot)?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    label if label == Section::<SECTION>::LOGICAL_LABEL => {
                        let section = branch.recv::<Section<SECTION>>().await?;
                        let reviewed = endpoint.recv::<C::FinalForFact>().await?.into_value();
                        imp::reviewed(AUTHOR, &reviewed, (&snapshot, &facts))?;
                        endpoint.send::<Section<SECTION>>(section).await?;
                        return Ok(());
                    }
                    label if label == C::RevisedForFact::LOGICAL_LABEL => {
                        written = branch.recv::<C::RevisedForFact>().await?.into_value();
                        section = endpoint.recv::<Section<SECTION>>().await?;
                        if early_archive {
                            {
                                endpoint
                                    .send::<Section<SECTION>>(section)
                                    .await
                                    .expect_err("revised chapter still needs fresh approval");
                                return Err(ShellError::message("early archive rejected"));
                            };
                        }
                        continue;
                    }
                    _ => return Err(ShellError::message("unexpected factual continuation")),
                }
            }
            _ => return Err(ShellError::message("unexpected factual verdict")),
        }
        written = endpoint.recv::<C::RevisedForFact>().await?.into_value();
        section = endpoint.recv::<Section<SECTION>>().await?;
        if early_archive {
            endpoint
                .send::<Section<SECTION>>(section)
                .await
                .expect_err("revised chapter still needs fresh approval");
            return Err(ShellError::message("early archive rejected"));
        }
    }
}

#[hibana_runtime::scope]
async fn editor_chapter<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut ParallelEndpoint<EDITOR>,
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        let facts = match branch.label() {
            label if label == C::FactsForEditor::LOGICAL_LABEL => {
                branch.recv::<C::FactsForEditor>().await?.into_value()
            }
            label if label == C::FailedFactsForEditor::LOGICAL_LABEL => {
                branch.recv::<C::FailedFactsForEditor>().await?;
                continue;
            }
            _ => return Err(ShellError::message("unexpected editorial factual input")),
        };
        let mut section = endpoint.recv::<Section<SECTION>>().await?;
        endpoint
            .send::<C::ReviewMaterial>(&imp::Material::new(
                json!({"snapshot":facts["data"]["input_snapshot"],"facts":facts}),
            )?)
            .await?;
        let args = json!({"section_name":imp::SECTIONS[usize::from(AUTHOR)]});
        section
            .send::<C::ReviewRequest>(&imp::Arguments::<AUTHOR, 6>::from_bytes(&imp::encode(
                &args,
            )?)?)
            .await?;
        endpoint.send::<Section<SECTION>>(section).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::ReviewPassed::LOGICAL_LABEL => {
                branch.recv::<C::ReviewPassed>().await?;
                return Ok(());
            }
            label if label == C::ReviewRejected::LOGICAL_LABEL => {
                branch.recv::<C::ReviewRejected>().await?;
            }
            _ => return Err(ShellError::message("unexpected editorial result")),
        }
    }
}

#[hibana_runtime::scope]
async fn exercise(
    domain: &impl Domain,
    (starts, attack): ([usize; 3], Option<&str>),
    observed: &Cell<Vec<tap::TapEvent>>,
) -> ShellResult<(ShellResult<()>, Value)> {
    let source = super::global::source();
    let a_program = project::<AUTHOR_A>(&source);
    let b_program = project::<AUTHOR_B>(&source);
    let c_program = project::<AUTHOR_C>(&source);
    let fact_program = project::<FACT_CHECKER>(&source);
    let editor_program = project::<EDITOR>(&source);
    let research_program = project::<RESEARCH_SERVICE>(&source);
    let document_program = project::<DOCUMENT>(&source);
    let database_program = project::<DATABASE>(&source);
    let review_program = project::<REVIEW_SERVICE>(&source);
    let archive_program = project::<ARCHIVE>(&source);
    let a_view = project::<SECTION_A>(&source);
    let b_view = project::<SECTION_B>(&source);
    let c_view = project::<SECTION_C>(&source);

    let compiled = tap::projected(&document_program).collect::<Vec<_>>();
    let archive_sources = compiled
        .iter()
        .filter(|row| {
            row.id() == tap::PROJECTED_SOURCE
                && row.arg0() >> 24 == u32::from(ARCHIVE)
                && (row.arg0() >> 16) as u8 == DOCUMENT
        })
        .map(|row| row.ts() as u16)
        .collect::<Vec<_>>();
    let capture = |row| {
        let mut rows = observed.take();
        rows.push(row);
        observed.set(rows);
    };
    let session = SessionId::new(SESSION);
    let (bus, _, _) = Bus::open(
        [],
        [
            (session, DOCUMENT),
            (session, DATABASE),
            (session, SECTION_A),
            (session, SECTION_B),
            (session, SECTION_C),
        ],
    );
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let mut a = rv.enter(session, &a_program)?;
    let mut b = rv.enter(session, &b_program)?;
    let mut c = rv.enter(session, &c_program)?;
    let mut fact = rv.enter(session, &fact_program)?;
    let mut editor = rv.enter(session, &editor_program)?;
    let mut research = rv.enter(session, &research_program)?;
    let document = rv.enter(session, &document_program)?;
    let database = rv.enter(session, &database_program)?;
    let mut review = rv.enter(session, &review_program)?;
    let mut archive = rv.enter(session, &archive_program)?;
    let section_a = rv.enter(session, &a_view)?;
    let section_b = rv.enter(session, &b_view)?;
    let section_c = rv.enter(session, &c_view)?;

    let watchdog = 30;
    let run=hibana_runtime::race(async {
        async {localside::document::run(document,domain).await?;};
        async {localside::database::run(database,domain).await?;};
        async {
            let [mut a,mut b,mut c]=research.par()?;
            async {localside::research::run::<AUTHOR_A,a::Research>(&mut a,domain).await?;};
            async {localside::research::run::<AUTHOR_B,b::Research>(&mut b,domain).await?;};
            async {localside::research::run::<AUTHOR_C,c::Research>(&mut c,domain).await?;};
            Ok::<(),ShellError>(())
        };
        async {author::<AUTHOR_A,SECTION_A,a::Research>(&mut a,section_a,starts[0]).await?;};
        async {author::<AUTHOR_B,SECTION_B,b::Research>(&mut b,section_b,starts[1]).await?;};
        async {author::<AUTHOR_C,SECTION_C,c::Research>(&mut c,section_c,starts[2]).await?;};
        async {
            let [mut a,mut b,mut c]=fact.par()?;
            async {self::fact::<AUTHOR_A,SECTION_A,a::Research>(&mut a,attack==Some("early_archive")).await?;};
            async {self::fact::<AUTHOR_B,SECTION_B,b::Research>(&mut b,false).await?;};
            async {self::fact::<AUTHOR_C,SECTION_C,c::Research>(&mut c,false).await?;};
            Ok::<(),ShellError>(())
        };
        async {
            let arguments=ReportArguments::<6>::from_bytes(br#"{"sections":"section_a,section_b,section_c"}"#)?;
            async {
                let [mut a,mut b,mut c]=editor.par()?;
                async {editor_chapter::<AUTHOR_A,SECTION_A,a::Research>(&mut a).await?;};
                async {editor_chapter::<AUTHOR_B,SECTION_B,b::Research>(&mut b).await?;};
                async {editor_chapter::<AUTHOR_C,SECTION_C,c::Research>(&mut c).await?;};
                Ok::<(),ShellError>(())
            }.await?;
            editor.send::<Combine>(&arguments).await?;
            let result=imp::decode(editor.recv::<Published>().await?)?;
            assert_eq!(result["success"],true);
            assert!(editor.send::<Combine>(&arguments).await.expect_err("completed publication cannot repeat").is_unavailable());
        };
        async {
            let (a,b,c)=async {
                let [mut a,mut b,mut c]=review.par()?;
                async {localside::review::run::<AUTHOR_A,SECTION_A,a::Research>(&mut a,domain).await};
                async {localside::review::run::<AUTHOR_B,SECTION_B,b::Research>(&mut b,domain).await};
                async {localside::review::run::<AUTHOR_C,SECTION_C,c::Research>(&mut c,domain).await};
            }.await?;
            let mut results=json!({"facts":{"section_a":a.0,"section_b":b.0,"section_c":c.0},"reviews":{"section_a":a.1,"section_b":b.1,"section_c":c.1}});
            if attack==Some("stale_reviews") {results["facts"]["section_b"]["data"]["input_snapshot"]["sections"]["section_b"]["id"]=json!("stale chapter");}
            review.send::<FrozenReviews>(&imp::encode(&results)?.as_slice()).await?;
        };
        async {
            let (a,b,c)=async {
                let [mut a,mut b,mut c]=archive.par()?;
                async {Ok::<_,ShellError>(a.recv::<Section<SECTION_A>>().await?)};
                async {Ok::<_,ShellError>(b.recv::<Section<SECTION_B>>().await?)};
                async {Ok::<_,ShellError>(c.recv::<Section<SECTION_C>>().await?)};
            }.await?;
            let [mut first,mut second,mut third]=archive.par()?;
            async {first.send::<Section<SECTION_A>>(a).await?;};
            async {
                if attack==Some("missing_section_b") {
                    std::future::poll_fn(|cx| {
                        let rows=observed.take();
                        let received=rows.iter().any(|row|row.id()==tap::ENDPOINT_RECV && row.causal_key()>>8==u16::from(DOCUMENT) && archive_sources.contains(&(row.arg1() as u16)));
                        observed.set(rows);
                        if received {std::task::Poll::Ready(())} else {cx.waker().wake_by_ref();std::task::Poll::Pending}
                    }).await;
                    drop(b);
                    return Err(ShellError::message("fixture omitted one actual chapter authority"));
                }
                second.send::<Section<SECTION_B>>(b).await?;
            };
            async {third.send::<Section<SECTION_C>>(c).await?;};
            Ok::<(),ShellError>(())
        };
        Ok::<(),ShellError>(())
    },deadline(Duration::from_secs(watchdog))).await;
    let rows = observed.take();
    let captured = rows.iter().copied().map(record).collect::<Vec<_>>();
    let checkpoint = observations.checkpoint();
    hibana_epf::Capture::new(&rows, checkpoint)
        .map_err(|e| ShellError::message(format!("incomplete native capture: {e:?}")))?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let directory = root
        .join("output/experiments/research-paper-3M-development-v1/whole-native-runs")
        .join(format!(
            "run-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("global.rs"), include_str!("global.rs"))?;
    std::fs::write(
        directory.join("whole_tests.rs"),
        include_str!("whole_tests.rs"),
    )?;
    let report = json!({"compiled":compiled.into_iter().map(record).collect::<Vec<_>>(),"records":captured,"checkpoint":checkpoint,
        "frames":bus.transport.take_observations(),"model_calls":0,"wasm_conditions":0,"whole_task_application_qualified":false,
        "evidence_directory":directory,"error":run.as_ref().err().map(ToString::to_string)});
    std::fs::write(directory.join("protocol.json"), imp::encode(&report)?)?;
    Ok((run, report))
}

#[hibana_runtime::main]
#[test]
async fn whole_three_author_protocol_pipelines_actual_receivers_and_excludes_physical_conflicts()
-> ShellResult<()> {
    let effects = Effects {
        events: Cell::new(Vec::new()),
        failures: [[0, 0]; 3],
    };
    let observed = Cell::new(Vec::new());
    let (run, evidence) = exercise(&effects, ([0, 3, 6], None), &observed).await?;
    let events = effects.events.take();
    std::fs::write(
        std::path::Path::new(evidence["evidence_directory"].as_str().unwrap()).join("effects.json"),
        imp::encode(&json!(&events))?,
    )?;
    run?;
    assert_eq!(
        events
            .iter()
            .filter(|r| r["event"] == "end" && r["tool"] == "combine_sections")
            .count(),
        1
    );
    for resource in [
        ["write_section", "revise_section"].as_slice(),
        ["update_references"].as_slice(),
    ] {
        let mut active = 0;
        for row in &events {
            if row["tool"]
                .as_str()
                .is_some_and(|tool| resource.contains(&tool))
            {
                if row["event"] == "start" {
                    active += 1;
                    assert_eq!(
                        active, 1,
                        "one physical owner must exclude competing effects"
                    );
                } else {
                    active -= 1;
                }
            }
        }
        assert_eq!(active, 0);
    }
    let positions = |tool: &str| {
        events
            .iter()
            .enumerate()
            .filter(|(_, r)| r["tool"] == tool)
            .map(|(i, r)| {
                (
                    r["event"].as_str().unwrap(),
                    r["sequence"].as_u64().unwrap(),
                    i,
                )
            })
            .collect::<Vec<_>>()
    };
    let write = positions("write_section");
    let references = positions("update_references");
    assert!(write.iter().any(|(kind, seq, start)| *kind == "start"
        && references.iter().any(|(other, id, ref_start)| {
            *other == "start"
                && *ref_start
                    < write
                        .iter()
                        .find(|(k, s, _)| *k == "end" && s == seq)
                        .unwrap()
                        .2
                && *start
                    < references
                        .iter()
                        .find(|(k, s, _)| *k == "end" && s == id)
                        .unwrap()
                        .2
        })));
    let first_fact = events
        .iter()
        .position(|r| r["event"] == "start" && r["tool"] == "fact_check")
        .unwrap();
    let last_write = events
        .iter()
        .rposition(|r| r["event"] == "end" && r["tool"] == "write_section")
        .unwrap();
    assert!(
        first_fact < last_write,
        "fact checking must not wait for every chapter"
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn all_three_authors_revise_and_recheck_after_both_kinds_of_rejection() -> ShellResult<()> {
    let failures = [[2, 1], [1, 2], [0, 1]];
    let effects = Effects {
        events: Cell::new(Vec::new()),
        failures,
    };
    let observed = Cell::new(Vec::new());
    let (run, evidence) = exercise(&effects, ([4, 2, 0], None), &observed).await?;
    let events = effects.events.take();
    std::fs::write(
        std::path::Path::new(evidence["evidence_directory"].as_str().unwrap()).join("effects.json"),
        imp::encode(&json!(&events))?,
    )?;
    run?;
    for (author, failures) in failures.into_iter().enumerate() {
        let name = imp::SECTIONS[author];
        let count = |tool: &str| {
            events
                .iter()
                .filter(|r| {
                    r["event"] == "end"
                        && r["tool"] == tool
                        && (r["result"]["data"]["section"] == name
                            || r["result"]["data"]["scope"] == name)
                })
                .count()
        };
        assert_eq!(count("revise_section"), failures[0] + failures[1]);
        assert_eq!(count("fact_check"), failures[0] + failures[1] + 1);
        assert_eq!(count("review_sections"), failures[1] + 1);
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn whole_source_rejects_early_archival_stale_reviews_and_missing_custody() -> ShellResult<()>
{
    for (attack, failures) in [
        ("early_archive", [[1, 0], [0, 0], [0, 0]]),
        ("early_archive", [[0, 1], [0, 0], [0, 0]]),
        ("stale_reviews", [[0, 0]; 3]),
        ("missing_section_b", [[0, 0]; 3]),
    ] {
        let effects = Effects {
            events: Cell::new(Vec::new()),
            failures,
        };
        let observed = Cell::new(Vec::new());
        let (run, evidence) = exercise(&effects, ([0, 2, 4], Some(attack)), &observed).await?;
        let events = effects.events.take();
        std::fs::write(
            std::path::Path::new(evidence["evidence_directory"].as_str().unwrap())
                .join("effects.json"),
            imp::encode(&json!(&events))?,
        )?;
        assert!(
            matches!(run, Err(ShellError::Message(_))),
            "specific receiver/producer rejection, not a deadline or kernel failure: {run:?}"
        );
        assert!(!events.iter().any(|r| r["tool"] == "combine_sections"));
    }
    Ok(())
}

mod world;
