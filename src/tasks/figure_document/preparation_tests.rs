use super::{
    CHAPTERS,
    global::*,
    localside::{drafts, snapshots, writers},
    tests::{audit_fixture, fixture_io, wasm_fixture},
};
use crate::shell::{
    ShellResult,
    global::Loan,
    imp::{bus::Bus, private_fs::PrivateFs, service::Domain},
    localside::file_metadata,
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project, tap};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

#[derive(Default)]
struct World(Cell<Vec<Value>>);

impl Domain for World {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        args: Value,
    ) -> Result<Value, String> {
        let chapter = args["section_name"].as_str().unwrap();
        let mut journal = self.0.take();
        journal.push(json!({"started":tool,"section":chapter}));
        self.0.set(journal);
        let depends_on = match (tool, chapter) {
            ("write_section", "CHAPTER_2") => Some(("tool_name", "update_references", "CHAPTER_3")),
            ("update_references", "CHAPTER_3") => Some(("started", "write_section", "CHAPTER_2")),
            _ => None,
        };
        if let Some((field, operation, target)) = depends_on {
            loop {
                let journal = self.0.take();
                let reached = journal
                    .iter()
                    .any(|row| row[field] == operation && row["section"] == target);
                self.0.set(journal);
                if reached {
                    break;
                }
                hibana_runtime::yield_now().await;
            }
        }
        let mut journal = self.0.take();
        let (success, data) = match tool {
            "write_section" => (
                true,
                json!({"agent":agent,"section":chapter,"artifact":{
                    "id":format!("draft:{chapter}"),"content":args["content_notes"]
                }}),
            ),
            "update_references" => (
                true,
                json!({"agent":agent,"section":chapter,"scope":chapter,"artifact":{
                    "id":format!("refs:{chapter}"),"content":format!("References for {chapter} by {agent}")
                }}),
            ),
            "read_section" => {
                assert_eq!(agent, "WRITER_A");
                let written = journal
                    .iter()
                    .find(|row| row["tool_name"] == "write_section" && row["section"] == chapter);
                let (status, artifact) = match written {
                    Some(row) => ("read", row["actual"]["data"]["artifact"].clone()),
                    None => ("missing_material", Value::Null),
                };
                (
                    !artifact.is_null(),
                    json!({"section":chapter,"status":status,"artifact":artifact}),
                )
            }
            _ => panic!("unexpected writer fixture operation: {tool}"),
        };
        let actual = json!({"success":success,"tool_name":tool,"data":data});
        journal.push(json!({"tool_name":tool,"section":chapter,"actual":actual}));
        self.0.set(journal);
        Ok(actual)
    }
}

#[hibana_runtime::main]
#[test]
async fn two_wasm_writers_prepare_document_data_while_independent_references_overlap()
-> ShellResult<()> {
    let source = document_preparation();
    let document_program = project::<DOCUMENT>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let references_program = project::<REFERENCES>(&source);
    let a_snap_program = project::<WRITER_A_SNAPSHOTS>(&source);
    let b_snap_program = project::<WRITER_B_SNAPSHOTS>(&source);
    let a_data_program = project::<WRITER_A_DATA>(&source);
    let b_data_program = project::<WRITER_B_DATA>(&source);
    let a_file_program = project::<WRITER_FILE>(&source);
    let b_file_program = project::<WRITER_B_FILE>(&source);
    let a_vm_program = project::<WRITER_A_VM>(&source);
    let b_vm_program = project::<WRITER_B_VM>(&source);
    let a_io_program = project::<WRITER_A_IO>(&source);
    let b_io_program = project::<WRITER_B_IO>(&source);
    let a_module = wasm_fixture("writer_a_fixture.rs")?;
    let b_module = wasm_fixture("writer_b_fixture.rs")?;
    let session = SessionId::new(84);
    let (bus, _, [a_terminal, b_terminal]) = Bus::open(
        ["WRITER_A", "WRITER_B"],
        [WRITER_FILE, WRITER_B_FILE].map(|role| (session, role)),
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
    let mut document = rv.enter(session, &document_program)?;
    let mut drafting = rv.enter(session, &drafting_program)?;
    let mut refs = rv.enter(session, &references_program)?;
    let mut reads_a = rv.enter(session, &a_snap_program)?;
    let mut reads_b = rv.enter(session, &b_snap_program)?;
    let mut data_a = rv.enter(session, &a_data_program)?;
    let mut data_b = rv.enter(session, &b_data_program)?;
    let file_a = rv.enter(session, &a_file_program)?;
    let file_b = rv.enter(session, &b_file_program)?;
    let vm_a = rv.enter(session, &a_vm_program)?;
    let vm_b = rv.enter(session, &b_vm_program)?;
    let mut io_a = rv.enter(session, &a_io_program)?;
    let mut io_b = rv.enter(session, &b_io_program)?;
    let world = World::default();

    let trial = async {
        async {
            file_metadata::run(&mut data_a, file_a, WRITER_A_OBJECTS).await?;
        };
        async {
            file_metadata::run(&mut data_b, file_b, WRITER_B_OBJECTS).await?;
        };
        async {
            fixture_io(
                &mut io_a,
                PrivateFs::new(a_terminal, &bus, &WRITER_A_OBJECTS),
            )
            .await?;
        };
        async {
            fixture_io(
                &mut io_b,
                PrivateFs::new(b_terminal, &bus, &WRITER_B_OBJECTS),
            )
            .await?;
        };
        async {
            snapshots(&mut reads_a, ("WRITER_A", &world)).await?;
        };
        async {
            snapshots(&mut reads_b, ("WRITER_B", &world)).await?;
        };
        async {
            let (file_a, file_b) = async {
                let [mut a, mut b] = drafting.par()?;
                async { a.recv::<Loan<WRITER_FILE>>().await };
                async { b.recv::<Loan<WRITER_B_FILE>>().await };
            }
            .await?;
            async {
                writers::writer_a(&a_module, file_a, vm_a).await?;
            };
            async {
                writers::writer_b(&b_module, file_b, vm_b).await?;
            };
            Ok(())
        };
        async { drafts::initial(&mut document, &world).await };
        async { writers::references(&mut refs, &world).await };
    };
    let (_, _, _, _, _, _, _, sections, references) = hibana_runtime::race(
        trial,
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await?;
    let journal = world.0.take();
    let completed = journal
        .iter()
        .filter(|row| row.get("actual").is_some())
        .count();
    assert_eq!(completed, 7, "the wrong-chapter request reached an effect");
    let start = journal
        .iter()
        .position(|row| row["started"] == "write_section" && row["section"] == "CHAPTER_2")
        .unwrap();
    let done = journal
        .iter()
        .position(|row| row["tool_name"] == "write_section" && row["section"] == "CHAPTER_2")
        .unwrap();
    assert!(
        journal[start + 1..done]
            .iter()
            .any(|row| row["tool_name"] == "update_references" && row["section"] == "CHAPTER_3")
    );
    let material = json!({"sections":sections,"references":references});
    for chapter in CHAPTERS {
        for (tool, category) in [
            ("write_section", "sections"),
            ("update_references", "references"),
        ] {
            let effect = journal
                .iter()
                .find(|row| row["tool_name"] == tool && row["section"] == chapter)
                .unwrap();
            assert_eq!(
                material[category][chapter],
                effect["actual"]["data"]["artifact"]
            );
        }
    }
    let records = records.take();
    let snapshot: Vec<_> = tap::projected(&document_program).collect();
    let checked = audit_fixture(&records, observation.checkpoint(), &snapshot, session);
    println!(
        "{completed} fixture effects, {} complete tap records, {checked} checked wire acceptances",
        records.len()
    );
    Ok(())
}
