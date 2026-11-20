use super::{
    Arguments,
    global::*,
    localside,
    tests::{audit_fixture, fixture_io, wasm_fixture},
};
use crate::{
    imp::json::{decode, encode},
    shell::{
        GUEST_MEMORY_BYTES, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{
            bus::Bus,
            private_fs::PrivateFs,
            service::{ReceivedEffects, SimulatorOperation},
        },
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
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

const WRITER_DATA: u8 = 14;
const WRITER_VM: u8 = 15;
const WRITER_IO: u8 = 16;

#[derive(Default)]
struct DraftWorld(Cell<Vec<Value>>);

impl ReceivedEffects for DraftWorld {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let args = crate::imp::json::decode(received.as_bytes());
        async move {
            let args = args?;
            let (agent, tool) = P::RPC;

            if tool == "write_section" && agent == "WRITER_B" {
                // The writer's first effect remains in flight while a reader enters.
                let mut events = self.0.take();
                events.push(json!({"start":tool}));
                self.0.set(events);
                loop {
                    let events = self.0.take();
                    let observed = events.iter().any(|event| {
                        event["tool_name"] == "read_section"
                            && event["data"]["section"] == "CHAPTER_3"
                            && event["success"] == false
                    });
                    self.0.set(events);
                    if observed {
                        break;
                    }
                    hibana_runtime::yield_now().await;
                }
            }

            let mut events = self.0.take();
            let section = &args["section_name"];
            let (status, artifact) = match tool {
                "write_section" => (
                    "written",
                    json!({"id":if agent == "WRITER_B" { "draft:1" } else { "draft:3" }, "content":args["content_notes"]}),
                ),
                "revise_section" => (
                    "revised",
                    json!({"id":"draft:2","content":args["feedback"]}),
                ),
                "read_section" => {
                    assert!(matches!(agent, "WRITER_A" | "WRITER_B"));
                    match events.iter().rev().find(|event| {
                        matches!(
                            event["tool_name"].as_str(),
                            Some("write_section" | "revise_section")
                        ) && &event["data"]["section"] == section
                    }) {
                        Some(completed) => ("read", completed["data"]["artifact"].clone()),
                        None => ("missing_material", Value::Null),
                    }
                }
                _ => panic!("unexpected fixture effect {tool}"),
            };
            let reply = json!({"tool_name":tool,"success": !artifact.is_null(),
            "data":{"section":section,"status":status,"artifact":artifact}});
            events.push(reply.clone());
            self.0.set(events);
            Ok(reply)
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn wasm_reads_do_not_lock_the_document_or_return_unfinished_content() -> ShellResult<()> {
    let objects = [READ_SECTION, WRITE_SECTION];
    let source = g::seq(
        g::send::<WRITER_DATA, WRITER_VM, Loan<WRITER_FILE>>(),
        g::par(
            g::seq(
                write_chapter::<WRITER_B_FILE, DRAFT_3>(),
                // This scenario exercises a draft which uses another writer's revision.
                g::seq(
                    revise_chapter::<WRITER_B_FILE, REVISE_B>(),
                    write_chapter::<WRITER_FILE, DRAFT_1>(),
                ),
            ),
            g::par(
                snapshot_reads::<WRITER_B_FILE, WRITER_B_SNAPSHOTS>(),
                g::par(
                    g::seq(
                        g::par(
                            snapshot_reads::<WRITER_FILE, WRITER_A_SNAPSHOTS>(),
                            file_metadata::<WRITER_FILE, WRITER_DATA>().roll(),
                        ),
                        g::send::<WRITER_FILE, WRITER_DATA, FileExit>(),
                    ),
                    g::seq(
                        private_io::<WRITER_VM, WRITER_IO>(),
                        g::send::<WRITER_VM, WRITER_IO, Exit>(),
                    ),
                ),
            ),
        ),
    );
    let module = wasm_fixture("snapshot_fixture.rs")?;
    let document_program = project::<DOCUMENT>(&source);
    let writer_b_program = project::<WRITER_B_FILE>(&source);
    let read_a_program = project::<WRITER_A_SNAPSHOTS>(&source);
    let read_b_program = project::<WRITER_B_SNAPSHOTS>(&source);
    let metadata_program = project::<WRITER_DATA>(&source);
    let file_program = project::<WRITER_FILE>(&source);
    let vm_program = project::<WRITER_VM>(&source);
    let io_program = project::<WRITER_IO>(&source);
    let session = SessionId::new(75);
    let (bus, _, [terminal]) = Bus::open(["WRITER_A"], [(session, WRITER_FILE)]);
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
    let mut doc = rv.enter(session, &document_program)?;
    let mut writer_b = rv.enter(session, &writer_b_program)?;
    let mut a_snapshots = rv.enter(session, &read_a_program)?;
    let mut b_snapshots = rv.enter(session, &read_b_program)?;
    let mut metadata = rv.enter(session, &metadata_program)?;
    let file = rv.enter(session, &file_program)?;
    let mut root = rv.enter(session, &vm_program)?;
    let mut io = rv.enter(session, &io_program)?;
    let world = DraftWorld::default();
    let trial = async {
        async {
            let actual = world
                .apply(doc.recv::<DraftRequest<DRAFT_3>>().await?)
                .await?;
            doc.send::<Written>(&encode(&actual)?.as_slice()).await?;
            let actual = world.apply(doc.recv::<Revise<REVISE_B>>().await?).await?;
            doc.send::<Revised>(&encode(&actual)?.as_slice()).await?;
            let actual = world
                .apply(doc.recv::<DraftRequest<DRAFT_1>>().await?)
                .await?;
            doc.send::<Written>(&encode(&actual)?.as_slice()).await?;
        };
        async {
            let draft = Arguments::new(
                &json!({"section_name":"CHAPTER_3", "content_notes":"original text"}),
            )?;
            writer_b.send::<DraftRequest<DRAFT_3>>(&draft).await?;
            assert_eq!(
                decode(writer_b.recv::<Written>().await?)?["data"]["artifact"]["id"],
                "draft:1"
            );
            loop {
                let events = world.0.take();
                let read = events.iter().any(|event| {
                    event["tool_name"] == "read_section"
                        && event["data"]["artifact"]["id"] == "draft:1"
                });
                world.0.set(events);
                if read {
                    break;
                }
                hibana_runtime::yield_now().await;
            }
            let revision =
                Arguments::new(&json!({"section_name":"CHAPTER_3", "feedback":"revised wording"}))?;
            writer_b.send::<Revise<REVISE_B>>(&revision).await?;
            assert_eq!(
                decode(writer_b.recv::<Revised>().await?)?["data"]["artifact"]["id"],
                "draft:2"
            );
            writer_b.send::<ReadingFinished>(&0).await?;
        };
        async { localside::snapshots(&mut a_snapshots, &world).await };
        async { localside::snapshots(&mut b_snapshots, &world).await };
        async { file_metadata::run(&mut metadata, file, objects).await };
        async { fixture_io(&mut io, PrivateFs::new(terminal, &bus, &objects)).await };
        async {
            let file = root.recv::<Loan<WRITER_FILE>>().await?;
            let [mut request, mut reply, mut draft, mut written] =
                [(); 4].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(READ_SECTION, file)
                .write_as::<ReadSection<WRITER_A_SNAPSHOTS>>(&mut request)
                .read_as::<SectionSnapshot>(&mut reply)
                .file(WRITE_SECTION)?
                .write_as::<DraftRequest<DRAFT_1>>(&mut draft)
                .read_as::<Written>(&mut written);
            let files = ChoreoFsFiles::new(root).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            root.send::<Exit>(&status).await?;
            let mut file = file.into_endpoint();
            file.send::<ReadingFinished>(&status).await?;
            file.send::<FileExit>(&status).await?;
        };
        Ok(())
    };
    hibana_runtime::race(
        trial,
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await?;
    let events = world.0.take();
    assert_eq!(events[0]["start"], "write_section");
    assert_eq!(events[1]["tool_name"], "read_section");
    assert_eq!(events[1]["success"], false);
    let write = events
        .iter()
        .position(|result| result["tool_name"] == "write_section")
        .unwrap();
    let revise = events
        .iter()
        .position(|result| result["tool_name"] == "revise_section")
        .unwrap();
    assert!(
        events[write + 1..revise]
            .iter()
            .any(|result| result["tool_name"] == "read_section"
                && result["data"]["artifact"] == events[write]["data"]["artifact"])
    );
    let quoted = events[revise + 1..]
        .iter()
        .rfind(|result| result["tool_name"] == "read_section")
        .unwrap();
    assert_eq!(
        quoted["data"]["artifact"],
        events[revise]["data"]["artifact"]
    );
    let draft = &events.last().unwrap()["data"];
    assert_eq!(draft["section"], "CHAPTER_1");
    let quoted = &quoted["data"]["artifact"];
    assert_eq!(
        draft["artifact"]["content"],
        format!(
            "Chapter 3 ({}): {}",
            quoted["id"].as_str().unwrap(),
            quoted["content"].as_str().unwrap()
        )
    );
    let records = records.take();
    let snapshot: Vec<_> = tap::projected(&read_a_program).collect();
    let accepted = audit_fixture(&records, observation.checkpoint(), &snapshot, session);
    println!(
        "{} fixture effects, {} complete tap records, {accepted} checked wire acceptances",
        events.len() - 1,
        records.len()
    );
    Ok(())
}
