use super::{Arguments, global::*, localside};
use crate::{
    imp::json::decode,
    shell::{
        ShellResult,
        global::{Exit, Loan},
        imp::{bus::Bus, private_fs::PrivateFs, service::Domain},
        localside::file_metadata,
    },
};
use hibana::{
    Endpoint,
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, program::project, tap},
};
use hibana_epf::{Acceptance, AuditStorage, Capture, ExecutionStorage, Protocol, RecordId, Unfold};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::Cell, fs, process::Command, time::Duration};

#[derive(Default)]
struct World(Cell<Vec<Value>>);

impl Domain for World {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        args: Value,
    ) -> Result<Value, String> {
        let mut events = self.0.take();
        let data = match tool {
            "write_section" => json!({"section":args["section_name"], "artifact":{
                "id":"document:1", "content":args["content_notes"]
            }}),
            "create_figure" => json!({"figure_id":"figure:original", "artifact":{
                "id":"figure:original", "content":args["description"]
            }}),
            "insert_figure" => {
                assert!(
                    events
                        .iter()
                        .any(|event| event["tool_name"] == "write_section"
                            && event["data"]["section"] == args["section_name"])
                );
                let created = events
                    .iter()
                    .find(|event| {
                        event["tool_name"] == "create_figure"
                            && event["data"]["figure_id"] == args["figure_id"]
                    })
                    .expect("only an actual figure can be inserted");
                json!({"section":args["section_name"], "figure_id":args["figure_id"],
                    "artifact":created["data"]["artifact"]})
            }
            _ => panic!("unexpected effect {tool}"),
        };
        let reply = json!({"success":true, "tool_name":tool, "data":data});
        events.push(json!({"agent":agent, "tool_name":tool, "arguments":args, "data":data}));
        self.0.set(events);
        Ok(reply)
    }
}

#[hibana_runtime::scope]
async fn exercise<const DRAFT: u8, const CREATE: u8, const INSERT: u8>(
    figures_first: bool,
    substitutes_store_request: bool,
) -> ShellResult<()> {
    let source = prepare_chapters(figure_chapter::<CREATE, INSERT>(
        draft::<WRITER_FILE, DRAFT>(),
    ));
    let document_program = project::<DOCUMENT>(&source);
    let store_program = project::<FIGURE_STORE>(&source);
    let document_owner_program = project::<DOCUMENT_OWNER>(&source);
    let figure_owner_program = project::<FIGURE_OWNER>(&source);
    let creator_program = project::<FIGURE_CREATOR>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let writer_file_program = project::<WRITER_FILE>(&source);
    let figure_file_program = project::<FIGURE_FILE>(&source);
    let session = SessionId::new(73);
    let (bus, _, []) = Bus::open([], [DOCUMENT, FIGURE_STORE].map(|role| (session, role)));
    let mut slab = vec![0; 128 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut document_owner = rv.enter(session, &document_owner_program)?;
    let mut figure_owner = rv.enter(session, &figure_owner_program)?;
    let document = rv.enter(session, &document_program)?;
    let store = rv.enter(session, &store_program)?;
    let mut creator = rv.enter(session, &creator_program)?;
    let mut drafting = rv.enter(session, &drafting_program)?;
    let mut writer_file = rv.enter(session, &writer_file_program)?;
    let mut figure_file = rv.enter(session, &figure_file_program)?;
    let world = World::default();
    let chapter = if DRAFT == DRAFT_1 {
        "CHAPTER_1"
    } else {
        "CHAPTER_3"
    };
    let write =
        Arguments::<DRAFT>::new(&json!({"section_name":chapter,"content_notes":"chapter draft"}))?;
    let create = Arguments::new(&json!({"description":"The figure selected by the agent"}))?;
    let trial = async {
        async {
            document_owner.send::<Loan<DOCUMENT>>(document).await?;
            drop(document_owner.recv::<Loan<DOCUMENT>>().await?);
        };
        async {
            figure_owner.send::<Loan<FIGURE_STORE>>(store).await?;
            drop(figure_owner.recv::<Loan<FIGURE_STORE>>().await?);
        };
        async {
            localside::document::<2>(&mut drafting, &world).await?;
        };
        async {
            if substitutes_store_request {
                let mut store = creator.recv::<Loan<FIGURE_STORE>>().await?;
                let create = store.recv::<Create<CREATE>>().await?.value()?;
                let figure = world
                    .execute("FIGURE_CREATOR", "create_figure", create)
                    .await
                    .map_err(crate::shell::ShellError::message)?;
                let bytes = crate::imp::json::encode(&figure)?;
                store.send::<Created<CREATE>>(&bytes.as_slice()).await?;
                store.recv::<Insert<INSERT>>().await?;
                let substituted =
                    Arguments::new(&json!({"section_name":chapter, "figure_id":"replacement"}))?;
                store.send::<Insert<INSERT>>(&substituted).await?;
                store.recv::<Inserted<INSERT>>().await?;
            } else {
                localside::creator::<2>(&mut creator, &world).await?;
            }
            Ok(())
        };
        async {
            let figure = if figures_first {
                figure_file.send::<Create<CREATE>>(&create).await?;
                let figure = decode(figure_file.recv::<Created<CREATE>>().await?)?;
                writer_file.send::<DraftRequest<DRAFT>>(&write).await?;
                assert_eq!(
                    decode(writer_file.recv::<DraftWritten<DRAFT>>().await?)?["data"]["section"],
                    chapter
                );
                figure
            } else {
                writer_file.send::<DraftRequest<DRAFT>>(&write).await?;
                assert_eq!(
                    decode(writer_file.recv::<DraftWritten<DRAFT>>().await?)?["data"]["section"],
                    chapter
                );
                figure_file.send::<Create<CREATE>>(&create).await?;
                decode(figure_file.recv::<Created<CREATE>>().await?)?
            };
            let insert = Arguments::new(
                &json!({"section_name":chapter, "figure_id":figure["data"]["figure_id"]}),
            )?;
            figure_file.send::<Insert<INSERT>>(&insert).await?;
            let inserted = decode(figure_file.recv::<Inserted<INSERT>>().await?)?;
            assert_eq!(inserted["data"]["artifact"], figure["data"]["artifact"]);
        };
        Ok(())
    };
    let result = hibana_runtime::race(trial, async {
        hibana_runtime::host::sleep(Duration::from_secs(5)).await?;
        panic!("figure document protocol stalled");
    })
    .await;
    let events = world.0.take();
    let first = if figures_first {
        "create_figure"
    } else {
        "write_section"
    };
    let second = if figures_first {
        "write_section"
    } else {
        "create_figure"
    };
    assert_eq!(events[0]["tool_name"], first);
    assert_eq!(events[1]["tool_name"], second);
    if substitutes_store_request {
        let error = result.expect_err("substituted resource arguments must fail");
        assert!(
            error
                .to_string()
                .contains("differs from its actual admitted data"),
            "{error:?}"
        );
        assert_eq!(events.len(), 2);
    } else {
        result?;
        assert_eq!(events.len(), 3);
        assert_eq!(events[2]["tool_name"], "insert_figure");
        assert_eq!(
            events[2]["arguments"],
            json!({"section_name":chapter,"figure_id":"figure:original"})
        );
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn creation_and_writing_may_start_in_either_order() -> ShellResult<()> {
    exercise::<DRAFT_1, CREATE_1, INSERT_1>(true, false).await?;
    exercise::<DRAFT_3, CREATE_3, INSERT_3>(false, false).await
}

#[hibana_runtime::main]
#[test]
async fn a_substituted_store_request_cannot_replace_the_file_request() -> ShellResult<()> {
    exercise::<DRAFT_1, CREATE_1, INSERT_1>(true, true).await
}

#[hibana_runtime::main]
#[test]
async fn wasm_figure_operations_use_the_delegated_document_and_store() -> ShellResult<()> {
    let module = wasm_fixture("creator_fixture.rs")?;
    let source = chapter_with_wasm_figure_creator();
    let document_program = project::<DOCUMENT>(&source);
    let store_program = project::<FIGURE_STORE>(&source);
    let document_owner_program = project::<DOCUMENT_OWNER>(&source);
    let figure_owner_program = project::<FIGURE_OWNER>(&source);
    let creator_program = project::<FIGURE_CREATOR>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let writer_file_program = project::<WRITER_FILE>(&source);
    let figure_file_program = project::<FIGURE_FILE>(&source);
    let metadata_program = project::<FIGURE_DATA>(&source);
    let vm_program = project::<FIGURE_VM>(&source);
    let io_program = project::<FIGURE_IO>(&source);
    let session = SessionId::new(74);
    let (bus, _, [terminal]) = Bus::open(
        ["FIGURE_CREATOR"],
        [DOCUMENT, FIGURE_STORE, FIGURE_FILE].map(|role| (session, role)),
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
    let mut document_owner = rv.enter(session, &document_owner_program)?;
    let mut figure_owner = rv.enter(session, &figure_owner_program)?;
    let document = rv.enter(session, &document_program)?;
    let store = rv.enter(session, &store_program)?;
    let mut creator = rv.enter(session, &creator_program)?;
    let mut drafting = rv.enter(session, &drafting_program)?;
    let mut writer = rv.enter(session, &writer_file_program)?;
    let mut metadata = rv.enter(session, &metadata_program)?;
    let file = rv.enter(session, &figure_file_program)?;
    let root = rv.enter(session, &vm_program)?;
    let mut io = rv.enter(session, &io_program)?;
    let world = World::default();
    let trial = async {
        async {
            document_owner.send::<Loan<DOCUMENT>>(document).await?;
            drop(document_owner.recv::<Loan<DOCUMENT>>().await?);
        };
        async {
            figure_owner.send::<Loan<FIGURE_STORE>>(store).await?;
            drop(figure_owner.recv::<Loan<FIGURE_STORE>>().await?);
        };
        async {
            localside::document::<2>(&mut drafting, &world).await?;
        };
        async { localside::creator::<2>(&mut creator, &world).await };
        async { localside::creator_guest(&module, root).await };
        async { file_metadata::run(&mut metadata, file, FIGURE_OBJECTS).await };
        async {
            let write = Arguments::new(
                &json!({"section_name":"CHAPTER_1", "content_notes":"First draft"}),
            )?;
            writer.send::<DraftRequest<DRAFT_1>>(&write).await?;
            assert_eq!(
                decode(writer.recv::<DraftWritten<DRAFT_1>>().await?)?["data"]["section"],
                "CHAPTER_1"
            );
        };
        async { fixture_io(&mut io, PrivateFs::new(terminal, &bus, &FIGURE_OBJECTS)).await };
        Ok(())
    };
    hibana_runtime::race(
        trial,
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await?;
    let events = world.0.take();
    assert_eq!(events.len(), 3);
    assert_eq!(events[2]["tool_name"], "insert_figure");
    assert_eq!(
        events[2]["arguments"],
        json!({"section_name":"CHAPTER_1", "figure_id":"figure:original"})
    );
    assert_eq!(
        events[2]["data"]["artifact"]["content"],
        "Figure from the guest"
    );
    let records = records.take();
    let snapshot: Vec<_> = tap::projected(&drafting_program).collect();
    let accepted = audit_fixture(&records, observation.checkpoint(), &snapshot, session);
    println!(
        "3 fixture effects, {} complete tap records, {accepted} checked wire acceptances",
        records.len()
    );
    Ok(())
}

pub(super) fn wasm_fixture(name: &str) -> ShellResult<Vec<u8>> {
    let artifact =
        std::env::temp_dir().join(format!("choreofix-7m-{name}-{}.wasm", std::process::id()));
    let compiled = Command::new("rustc")
        .args([
            "+stable",
            "--edition=2024",
            "--target=wasm32-wasip1",
            "-Copt-level=s",
            "-Cpanic=abort",
            "-Cstrip=debuginfo",
        ])
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../src/tasks/figure_document")
                .join(name),
        )
        .arg("-o")
        .arg(&artifact)
        .status()?;
    assert!(compiled.success());
    let module = fs::read(&artifact);
    fs::remove_file(&artifact)?;
    Ok(module?)
}

pub(super) async fn fixture_io<const ROLE: u8>(
    io: &mut Endpoint<'_, ROLE>,
    data: PrivateFs<'_>,
) -> ShellResult<()> {
    loop {
        let branch = io.offer().await?;
        match branch.label() {
            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                let received = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                let reply = data.objects().choreofs().fd_fdstat_get(received);
                io.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                let fd = branch.recv::<protocol::FdPrestatGetReqMsg>().await?.0.fd();
                let reply = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    fd,
                    1,
                    if fd == 3 { 0 } else { 8 },
                ));
                io.send::<protocol::FdPrestatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let fd = branch
                    .recv::<protocol::FdPrestatDirNameReqMsg>()
                    .await?
                    .0
                    .fd();
                let reply = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    fd,
                    b"/",
                    if fd == 3 { 0 } else { 8 },
                )?);
                io.send::<protocol::FdPrestatDirNameRetMsg>(&reply).await?;
            }
            Exit::LOGICAL_LABEL => {
                assert_eq!(branch.recv::<Exit>().await?, 0);
                break Ok(());
            }
            label => panic!("unexpected fixture I/O {label}"),
        }
    }
}

pub(super) fn audit_fixture(
    records: &[tap::TapEvent],
    checkpoint: Option<u32>,
    snapshot: &[tap::TapEvent],
    session: SessionId,
) -> usize {
    let captures = [Capture::new(records, checkpoint).expect("complete Wasm fixture capture")];
    let mut source_work = vec![0; Protocol::workspace_len(snapshot).unwrap()];
    let protocol = Protocol::new(snapshot, &mut source_work).unwrap();
    let count = records.len();
    let mut unfolding = vec![Unfold::Source; count * 2 + 1];
    let record_id = RecordId {
        producer: 0,
        ordinal: 0,
    };
    let mut order = vec![record_id; count];
    let mut occurrences = vec![None; count];
    let mut acceptances = vec![
        Acceptance {
            first_poll: record_id,
            before: 0
        };
        count
    ];
    let mut workspace = vec![
        0;
        protocol
            .audit_workspace_len(&captures, unfolding.len())
            .unwrap()
    ];
    let producer_order = vec![0; count];
    let audit = protocol
        .audit(
            &captures,
            AuditStorage {
                unfolding: &mut unfolding,
                execution: ExecutionStorage {
                    session: session.raw(),
                    observed_producers: &producer_order,
                    order: &mut order,
                    occurrences: &mut occurrences,
                    acceptances: &mut acceptances,
                },
            },
            &mut workspace,
        )
        .expect("EPF checks the Wasm fixture against its compiled source");
    audit.execution().acceptances.len()
}
