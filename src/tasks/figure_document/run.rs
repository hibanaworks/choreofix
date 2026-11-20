//! Runs the compiled 7M choreography on five independent wash instances.
use super::{
    global::*,
    localside::{self, quality, writers},
};
use crate::shell::{
    ShellResult,
    global::Loan,
    imp::{bus::Bus, private_fs::PrivateFs},
    localside::{file_metadata, private_io},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::json;

pub fn projection() -> serde_json::Value {
    Programs::new().observation()
}

#[hibana_runtime::scope]
pub async fn run_stdio(wash: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            a_terminal,
            b_terminal,
            figure_terminal,
            fact_terminal,
            editor_terminal,
        ],
    ) = Bus::open(
        [
            "WRITER_A",
            "WRITER_B",
            "FIGURE_CREATOR",
            "FACT_CHECKER",
            "EDITOR",
        ],
        [
            DOCUMENT,
            FIGURE_STORE,
            FIGURE_FILE,
            WRITER_FILE,
            WRITER_B_FILE,
            EDITOR_FILE,
            FACT_FILE,
            FACTS_12,
            FACTS_3,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let trial = async {
        let programs = Programs::new();
        bus.projected(programs.observation());
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observed = rv.tap();
        let result = async {
            let mut document_owner = rv.enter(session, &programs.owner)?;
            let doc = rv.enter(session, &programs.document)?;
            let mut fig_owner = rv.enter(session, &programs.store_owner)?;
            let store = rv.enter(session, &programs.store)?;
            let mut creator = rv.enter(session, &programs.creator)?;
            let mut fig_data = rv.enter(session, &programs.figure_data)?;
            let fig_file = rv.enter(session, &programs.figure_file)?;
            let fig_vm = rv.enter(session, &programs.figure_vm)?;
            let mut fig_io = rv.enter(session, &programs.figure_io)?;
            let mut drafting = rv.enter(session, &programs.drafting)?;
            let mut writer_a = rv.enter(session, &programs.writer_a)?;
            let mut writer_b = rv.enter(session, &programs.writer_b)?;
            let a_vm = rv.enter(session, &programs.a_vm)?;
            let b_vm = rv.enter(session, &programs.b_vm)?;
            let mut a_io = rv.enter(session, &programs.a_io)?;
            let mut b_io = rv.enter(session, &programs.b_io)?;
            let mut a_snapshots = rv.enter(session, &programs.a_snapshots)?;
            let mut b_snapshots = rv.enter(session, &programs.b_snapshots)?;
            let file_a = rv.enter(session, &programs.file_a)?;
            let file_b = rv.enter(session, &programs.file_b)?;
            let mut refs = rv.enter(session, &programs.references)?;
            let mut fact_data = rv.enter(session, &programs.fact_data)?;
            let fact12 = rv.enter(session, &programs.fact12)?;
            let fact3 = rv.enter(session, &programs.fact3)?;
            let fact_file = rv.enter(session, &programs.fact_file)?;
            let mut fact_vm = rv.enter(session, &programs.fact_vm)?;
            let mut fact_io = rv.enter(session, &programs.fact_io)?;
            let mut editor = rv.enter(session, &programs.editor)?;
            let mut editor_data = rv.enter(session, &programs.editor_data)?;
            let editor_file = rv.enter(session, &programs.editor_file)?;
            let editor_vm = rv.enter(session, &programs.editor_vm)?;
            let mut editor_io = rv.enter(session, &programs.editor_io)?;
            let mut archive = rv.enter(session, &programs.archive)?;
            async { localside::creator::<4>(&mut creator, &simulator).await };
            async {
                fig_owner.send::<Loan<FIGURE_STORE>>(store).await?;
                drop(fig_owner.recv::<Loan<FIGURE_STORE>>().await?);
            };
            async {
                file_metadata::run(&mut writer_a, file_a, WRITER_A_OBJECTS).await?;
            };
            async {
                file_metadata::run(&mut writer_b, file_b, WRITER_B_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut a_io,
                    PrivateFs::new(a_terminal, bus, &WRITER_A_OBJECTS),
                )
                .await?;
            };
            async {
                private_io::run(
                    &mut b_io,
                    PrivateFs::new(b_terminal, bus, &WRITER_B_OBJECTS),
                )
                .await?;
            };
            async { localside::snapshots(&mut a_snapshots, &simulator).await };
            async { localside::snapshots(&mut b_snapshots, &simulator).await };
            let (a, b) = async {
                let [mut a, mut b] = drafting.par()?;
                async { a.recv::<Loan<WRITER_FILE>>().await };
                async { b.recv::<Loan<WRITER_B_FILE>>().await };
            }
            .await?;
            async { writers::writer_a(wash, a, a_vm).await };
            async { writers::writer_b(wash, b, b_vm).await };
            async {
                let ((sections, figures), doc, references) = async {
                    async { localside::document::<5>(&mut drafting, &simulator).await };
                    async {
                        document_owner.send::<Loan<DOCUMENT>>(doc).await?;
                        document_owner.recv::<Loan<DOCUMENT>>().await
                    };
                    async { writers::references(&mut refs, &simulator).await };
                }
                .await?;
                let snapshot =
                    json!({"sections": sections, "references": references, "figures": figures});
                quality::reconsider(&mut document_owner, doc, (snapshot, &simulator)).await?;
            };
            async {
                file_metadata::run(&mut fig_data, fig_file, FIGURE_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut fig_io,
                    PrivateFs::new(figure_terminal, bus, &FIGURE_OBJECTS),
                )
                .await?;
            };
            async {
                localside::creator_guest(wash, fig_vm).await?;
            };
            async {
                async {
                    let [mut a, mut b] = fact_data.par()?;
                    async { a.send::<Loan<FACTS_12>>(fact12).await };
                    async { b.send::<Loan<FACTS_3>>(fact3).await };
                    Ok(())
                }
                .await?;
                file_metadata::run(&mut fact_data, fact_file, FACT_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut fact_io,
                    PrivateFs::new(fact_terminal, bus, &FACT_OBJECTS),
                )
                .await?;
            };
            async {
                let (mut fact12, mut fact3) = async {
                    let [mut a, mut b] = fact_vm.par()?;
                    async { a.recv::<Loan<FACTS_12>>().await };
                    async { b.recv::<Loan<FACTS_3>>().await };
                }
                .await?;
                async {
                    while quality::facts::<FACTS_12, CHECK_12>(&mut fact12, &simulator)
                        .await?
                        .is_some()
                    {}
                    Ok(())
                };
                async {
                    while quality::facts::<FACTS_3, CHECK_3>(&mut fact3, &simulator)
                        .await?
                        .is_some()
                    {}
                    Ok(())
                };
                localside::fact_checker_guest(wash, fact_vm).await?;
            };
            async {
                file_metadata::run(&mut editor_data, editor_file, EDITOR_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut editor_io,
                    PrivateFs::new(editor_terminal, bus, &EDITOR_OBJECTS),
                )
                .await?;
            };
            let file = editor.recv::<Loan<EDITOR_FILE>>().await?;
            async {
                while quality::editor(&mut editor, &simulator).await?.is_some() {}
                Ok(())
            };
            localside::editor_guest(wash, (editor_vm, file)).await?;

            drop(archive.recv::<Loan<DOCUMENT>>().await?);
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observed.checkpoint());
        result
    };
    let result = hibana_runtime::race(trial, io.run(bus)).await;
    io.finish(bus, result, None).await
}
