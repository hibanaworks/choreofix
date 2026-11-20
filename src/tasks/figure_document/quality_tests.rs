use super::{
    Arguments, CHAPTERS, FACT_SCOPES,
    global::*,
    localside::{editor_guest, quality},
    sections,
    tests::{audit_fixture, fixture_io, wasm_fixture},
};
use crate::{
    imp::json::decode,
    shell::{
        ShellError, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{bus::Bus, private_fs::PrivateFs, service::Domain},
        localside::file_metadata,
    },
};
use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
        tap,
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Evidence {
    Current,
    StaleFacts,
    StaleReview,
    RejectedReview,
}

struct World {
    evidence: Evidence,
    snapshot: Value,
    effects: Cell<Vec<Value>>,
}

impl Domain for World {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        args: Value,
    ) -> Result<Value, String> {
        let mut effects = self.effects.take();
        let mut success = true;
        let scope = args["section_name"].as_str();
        let facts = json!({FACT_SCOPES[0]:format!("fact:{}",FACT_SCOPES[0]), FACT_SCOPES[1]:format!("fact:{}",FACT_SCOPES[1])});
        let data = match tool {
            "fact_check" => {
                assert_eq!(agent, "FACT_CHECKER");
                let scope = scope.unwrap();
                let keep = if scope == FACT_SCOPES[0] {
                    &CHAPTERS[..2]
                } else {
                    &CHAPTERS[2..]
                };
                let mut snapshot = self.snapshot.clone();
                for category in ["sections", "references", "figures"] {
                    snapshot[category]
                        .as_object_mut()
                        .unwrap()
                        .retain(|name, _| keep.contains(&name.as_str()));
                }
                if self.evidence == Evidence::StaleFacts && scope == FACT_SCOPES[1] {
                    snapshot["sections"][CHAPTERS[2]]["id"] = json!("draft:3:old");
                }
                json!({"agent":agent,"section":scope,"check_id":facts[scope], "input_snapshot":snapshot})
            }
            "review_sections" => {
                assert_eq!(agent, "EDITOR");
                assert_eq!(
                    effects
                        .iter()
                        .filter(|event| event["tool_name"] == "fact_check")
                        .count(),
                    2
                );
                let scope = scope.unwrap();
                let mut snapshot = self.snapshot.clone();
                if self.evidence == Evidence::StaleReview && scope == CHAPTERS[1] {
                    snapshot["sections"][CHAPTERS[2]]["id"] = json!("draft:3:old");
                }
                success = self.evidence != Evidence::RejectedReview || scope != CHAPTERS[1];
                json!({"agent":agent,"scope":scope,"review_id":format!("review:{scope}"), "input_snapshot":snapshot,"fact_receipts":facts})
            }
            "combine_sections" => {
                assert_eq!(agent, "EDITOR");
                let parts: Vec<_> = sections(args["sections"].as_str().unwrap()).collect();
                let reviews: serde_json::Map<_, _> = effects
                    .iter()
                    .filter(|event| event["tool_name"] == "review_sections")
                    .map(|event| {
                        assert_eq!(event["success"], true);
                        assert_eq!(event["data"]["input_snapshot"], self.snapshot);
                        (
                            event["data"]["scope"].as_str().unwrap().to_owned(),
                            event["data"]["review_id"].clone(),
                        )
                    })
                    .collect();
                assert_eq!(reviews.len(), 3);
                json!({"agent":agent,"artifact_id":"publication:1", "sections":parts,"input_snapshot":self.snapshot,
                    "fact_receipts":facts,"review_receipts":reviews})
            }
            _ => panic!("unexpected publication effect: {tool}"),
        };
        let actual = json!({"success":success,"tool_name":tool,"data":data});
        effects.push(actual.clone());
        self.effects.set(effects);
        Ok(actual)
    }
}

#[hibana_runtime::scope]
async fn exercise(evidence: Evidence, module: Option<&[u8]>) -> ShellResult<()> {
    let closed = g::send::<EDITOR_FILE, EDITOR_DATA, FileExit>();
    let exited = g::send::<EDITOR_VM, EDITOR_IO, Exit>();
    let source = g::seq(
        g::par(
            g::par(lend_writer_files(), lend_fact_scopes()),
            g::send::<EDITOR_DATA, EDITOR, Loan<EDITOR_FILE>>(),
        ),
        g::par(
            g::par(
                reconsidering_publication(),
                g::route(
                    closed,
                    g::seq(file_metadata::<EDITOR_FILE, EDITOR_DATA>().roll(), closed),
                ),
            ),
            g::route(exited, g::seq(private_io::<EDITOR_VM, EDITOR_IO>(), exited)),
        ),
    );
    let doc_program = project::<DOCUMENT>(&source);
    let owner_program = project::<DOCUMENT_OWNER>(&source);
    let facts12_program = project::<FACTS_12>(&source);
    let facts3_program = project::<FACTS_3>(&source);
    let fact_data_program = project::<FACT_DATA>(&source);
    let fact_vm_program = project::<FACT_VM>(&source);
    let fact_file_program = project::<FACT_FILE>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let writer_a_data_program = project::<WRITER_A_DATA>(&source);
    let writer_b_data_program = project::<WRITER_B_DATA>(&source);
    let writer_a_program = project::<WRITER_FILE>(&source);
    let writer_b_program = project::<WRITER_B_FILE>(&source);
    let archive_program = project::<ARCHIVE>(&source);
    let editor_program = project::<EDITOR>(&source);
    let editor_file_program = project::<EDITOR_FILE>(&source);
    let metadata_program = project::<EDITOR_DATA>(&source);
    let vm_program = project::<EDITOR_VM>(&source);
    let io_program = project::<EDITOR_IO>(&source);
    let session = SessionId::new(76);
    let (bus, _, [terminal]) = Bus::open(
        ["EDITOR"],
        [
            DOCUMENT,
            EDITOR_FILE,
            WRITER_FILE,
            WRITER_B_FILE,
            FACTS_12,
            FACTS_3,
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
    let mut owner = rv.enter(session, &owner_program)?;
    let doc = rv.enter(session, &doc_program)?;
    let mut fact_data = rv.enter(session, &fact_data_program)?;
    let fact12 = rv.enter(session, &facts12_program)?;
    let fact3 = rv.enter(session, &facts3_program)?;
    let mut fact_vm = rv.enter(session, &fact_vm_program)?;
    let mut fact_file = rv.enter(session, &fact_file_program)?;
    let mut drafting = rv.enter(session, &drafting_program)?;
    let mut writer_a_data = rv.enter(session, &writer_a_data_program)?;
    let mut writer_b_data = rv.enter(session, &writer_b_data_program)?;
    let writer_a = rv.enter(session, &writer_a_program)?;
    let writer_b = rv.enter(session, &writer_b_program)?;
    let mut archive = rv.enter(session, &archive_program)?;
    let mut editor = rv.enter(session, &editor_program)?;
    let mut metadata = rv.enter(session, &metadata_program)?;
    let file = rv.enter(session, &editor_file_program)?;
    let root = rv.enter(session, &vm_program)?;
    let mut io = rv.enter(session, &io_program)?;
    let artifact = |id: &str, content: &str| json!({"id":id,"content":content});
    let world = World {
        evidence,
        effects: Cell::default(),
        snapshot: json!({
            "sections": {
                CHAPTERS[0]:artifact("draft:1:current","Introduction"),
                CHAPTERS[1]:artifact("draft:2:current","Methodology"),
                CHAPTERS[2]:artifact("draft:3:current","Revised results")
            },
            "references": {
                CHAPTERS[0]:artifact("refs:1","References A"),
                CHAPTERS[1]:artifact("refs:2","References B"),
                CHAPTERS[2]:artifact("refs:3","References C")
            },
            "figures": {
                CHAPTERS[0]:artifact("figure:1","Overview"),
                CHAPTERS[2]:artifact("figure:3","Results plot")
            }
        }),
    };
    let trial = async {
        async {
            let result =
                quality::reconsider(&mut owner, doc, (world.snapshot.clone(), &world)).await?;
            assert_eq!(result["data"]["artifact_id"], "publication:1");
            Ok(())
        };
        async {
            let [mut a, mut b] = fact_data.par()?;
            async { a.send::<Loan<FACTS_12>>(fact12).await };
            async { b.send::<Loan<FACTS_3>>(fact3).await };
            Ok(())
        };
        async {
            let (mut fact12, mut fact3) = async {
                let [mut a, mut b] = fact_vm.par()?;
                async { a.recv::<Loan<FACTS_12>>().await };
                async { b.recv::<Loan<FACTS_3>>().await };
            }
            .await?;
            async {
                quality::facts::<FACTS_12, CHECK_12>(&mut fact12, &world).await?;
                if evidence == Evidence::Current {
                    assert!(
                        quality::facts::<FACTS_12, CHECK_12>(&mut fact12, &world)
                            .await?
                            .is_none()
                    );
                } else {
                    // Retain this view until the owner can reject the bad evidence.
                    core::future::pending::<()>().await;
                }
                Ok(())
            };
            async {
                quality::facts::<FACTS_3, CHECK_3>(&mut fact3, &world).await?;
                if evidence == Evidence::Current {
                    assert!(
                        quality::facts::<FACTS_3, CHECK_3>(&mut fact3, &world)
                            .await?
                            .is_none()
                    );
                } else {
                    // Retain this view until the owner can reject the bad evidence.
                    core::future::pending::<()>().await;
                }
                Ok(())
            };
            Ok(())
        };
        async { writer_a_data.send::<Loan<WRITER_FILE>>(writer_a).await };
        async { writer_b_data.send::<Loan<WRITER_B_FILE>>(writer_b).await };
        async {
            let (mut a, mut b) = async {
                let [mut first, mut second] = drafting.par()?;
                async { first.recv::<Loan<WRITER_FILE>>().await };
                async { second.recv::<Loan<WRITER_B_FILE>>().await };
            }
            .await?;
            if evidence == Evidence::Current {
                let (first, second) = async {
                    async { a.recv::<WriterOutcome<WRITER_FILE>>().await };
                    async { b.recv::<WriterOutcome<WRITER_B_FILE>>().await };
                }
                .await?;
                let first = decode(first)?;
                assert_eq!(first["data"]["artifact_id"], "publication:1");
                assert_eq!(decode(second)?, first);
            } else {
                // Leave both loaned views live until the owner rejects the invalid evidence.
                let _retained = (a, b);
                core::future::pending::<()>().await;
            }
            Ok(())
        };
        async {
            if evidence == Evidence::Current {
                drop(archive.recv::<Loan<DOCUMENT>>().await?);
            }
            Ok(())
        };
        async {
            let initial = decode(fact_file.recv::<FactOutcome>().await?)?;
            assert!(initial[FACT_SCOPES[0]].is_object());
            assert!(initial[FACT_SCOPES[1]].is_object());
            fact_file
                .send::<QualityRequest<CHECK_3>>(&Arguments::new(
                    &json!({"section_name":FACT_SCOPES[1]}),
                )?)
                .await?;
            fact_file.recv::<QualityResult<CHECK_3>>().await?;
            fact_file
                .send::<QualityRequest<CHECK_12>>(&Arguments::new(
                    &json!({"section_name":FACT_SCOPES[0]}),
                )?)
                .await?;
            fact_file.recv::<QualityResult<CHECK_12>>().await?;
            if evidence == Evidence::Current {
                assert_eq!(
                    decode(fact_file.recv::<FactOutcome>().await?)?["data"]["artifact_id"],
                    "publication:1"
                );
            }
        };
        async { file_metadata::run(&mut metadata, file, EDITOR_OBJECTS).await };
        async { fixture_io(&mut io, PrivateFs::new(terminal, &bus, &EDITOR_OBJECTS)).await };
        async {
            let mut file = editor.recv::<Loan<EDITOR_FILE>>().await?;
            async { quality::editor(&mut editor, &world).await };
            if let Some(module) = module {
                editor_guest(module, (root, file)).await?;
            } else {
                if evidence != Evidence::StaleFacts {
                    file.recv::<ReviewMaterial>().await?;
                    file.send::<QualityRequest<REVIEW_2>>(&Arguments::new(
                        &json!({"section_name":CHAPTERS[1]}),
                    )?)
                    .await?;
                    let _ = decode(file.recv::<QualityResult<REVIEW_2>>().await?)?;
                    file.send::<QualityRequest<REVIEW_1>>(&Arguments::new(
                        &json!({"section_name":CHAPTERS[0]}),
                    )?)
                    .await?;
                    file.recv::<QualityResult<REVIEW_1>>().await?;
                    file.send::<QualityRequest<REVIEW_3>>(&Arguments::new(
                        &json!({"section_name":CHAPTERS[2]}),
                    )?)
                    .await?;
                    file.recv::<QualityResult<REVIEW_3>>().await?;
                }
                // Keep the guest's view live until the owner rejects the invalid evidence.
                let _retained = (root, file);
                core::future::pending::<()>().await;
            }
            Ok(())
        };
        Ok(())
    };
    let outcome = hibana_runtime::race(
        trial,
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await;
    match (evidence, outcome) {
        (Evidence::Current, outcome) => outcome?,
        (_, Err(ShellError::Message(message))) => {
            assert_eq!(message, "reconsideration lacks an actual revision request");
        }
        (_, other) => panic!("unexpected result after {evidence:?}: {other:?}"),
    }
    let effects = world.effects.take();
    let wanted = match evidence {
        Evidence::Current => 6,
        Evidence::StaleFacts => 2,
        Evidence::StaleReview | Evidence::RejectedReview => 5,
    };
    assert_eq!(effects.len(), wanted);
    assert_eq!(effects[0]["data"]["section"], FACT_SCOPES[1]);
    assert_eq!(
        effects
            .iter()
            .filter(|event| event["tool_name"] == "combine_sections")
            .count(),
        usize::from(evidence == Evidence::Current)
    );
    let records = records.take();
    let snapshot: Vec<_> = tap::projected(&owner_program).collect();
    let checked = audit_fixture(&records, observation.checkpoint(), &snapshot, session);
    println!(
        "{} fixture effects, {} complete tap records, {checked} checked wire acceptances",
        effects.len(),
        records.len()
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn wasm_editor_assembles_only_after_current_fact_and_review_receipts() -> ShellResult<()> {
    let module = wasm_fixture("editor_fixture.rs")?;
    exercise(Evidence::Current, Some(&module)).await
}

#[hibana_runtime::main]
#[test]
async fn old_facts_or_old_or_rejected_reviews_never_authorize_publication() -> ShellResult<()> {
    for evidence in [
        Evidence::StaleFacts,
        Evidence::StaleReview,
        Evidence::RejectedReview,
    ] {
        exercise(evidence, None).await?;
    }
    Ok(())
}
