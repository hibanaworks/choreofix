mod facts;
mod figures;
mod preparation;
#[cfg(test)]
mod preparation_tests;
mod publication;
mod review;
mod writers;

use crate::{
    imp::projection,
    shell::{
        ShellResult,
        global::Loan,
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    },
};
use facts::*;
use figures::*;
use hibana::{
    g::Message,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{RoleProgram, project},
    },
};
use preparation::*;
use publication::*;
use review::*;
use serde_json::{Value, json};
use writers::*;

const SESSION: u32 = 86;
const SESSION_STORAGE_BYTES: usize = 1024 * 1024;

// One list for the sealed role images used for both attachment and observation.
macro_rules! roles {
    ($($field:ident: $role:ident),+ $(,)?) => {
        pub(crate) struct Programs {
            $($field: RoleProgram<$role>,)+
        }

        impl Programs {
            pub(crate) fn new() -> Self {
                let source = with_preparation_guests(preparation());
                Self { $($field: project::<$role>(&source),)+ }
            }

            pub(crate) fn observation(&self) -> Value {
                let roles: serde_json::Map<String, Value> = [
                    $((stringify!($role).to_owned(), Value::from($role)),)+
                ].into_iter().collect();
                let programs = [$(projection::rows(&self.$field)),+];
                let labels: serde_json::Map<String, Value> = [
                    ("Write1", Request::<WRITE_1>::LOGICAL_LABEL),
                    ("Write2", Request::<WRITE_2>::LOGICAL_LABEL),
                    ("Write3", Request::<WRITE_3>::LOGICAL_LABEL),
                    ("Write4", Request::<WRITE_4>::LOGICAL_LABEL),
                    ("References1", Request::<REFERENCES_1>::LOGICAL_LABEL),
                    ("References2", Request::<REFERENCES_2>::LOGICAL_LABEL),
                    ("References3", Request::<REFERENCES_3>::LOGICAL_LABEL),
                    ("References4", Request::<REFERENCES_4>::LOGICAL_LABEL),
                    ("ContributionToPartI", ContributionToPartI::LOGICAL_LABEL),
                    ("EvaluationToPartI", EvaluationToPartI::LOGICAL_LABEL),
                    ("EvaluationToChapter3", EvaluationToChapter3::LOGICAL_LABEL),
                    ("Finalize1", Request::<FINALIZE_1>::LOGICAL_LABEL),
                    ("Finalize2", Request::<FINALIZE_2>::LOGICAL_LABEL),
                    ("Check12", Request::<CHECK_PART_I>::LOGICAL_LABEL),
                    ("Check3", Request::<CHECK_3>::LOGICAL_LABEL),
                    ("Check4", Request::<CHECK_4>::LOGICAL_LABEL),
                    ("FactInput", FactInput::LOGICAL_LABEL),
                    ("ReviewInput", ReviewInput::LOGICAL_LABEL),
                    ("PublicationMaterial", PublicationMaterial::LOGICAL_LABEL),
                    ("Editorial", Request::<EDITORIAL>::LOGICAL_LABEL),
                    ("Combine", Request::<COMBINE>::LOGICAL_LABEL),
                    ("Reconsider1", Request::<RECONSIDER_1>::LOGICAL_LABEL),
                    ("Reconsider2", Request::<RECONSIDER_2>::LOGICAL_LABEL),
                    ("Reconsider3", Request::<REVISE_3>::LOGICAL_LABEL),
                    ("Reconsider4", Request::<REVISE_4>::LOGICAL_LABEL),
                    ("Minor1", Request::<MINOR_1>::LOGICAL_LABEL),
                    ("Minor2", Request::<MINOR_2>::LOGICAL_LABEL),
                    ("Minor3", Request::<MINOR_3>::LOGICAL_LABEL),
                    ("Minor4", Request::<MINOR_4>::LOGICAL_LABEL),
                    ("AuthorOutcome", AuthorOutcome::LOGICAL_LABEL),
                    ("Create1", Request::<CREATE_1>::LOGICAL_LABEL),
                    ("Create3", Request::<CREATE_3>::LOGICAL_LABEL),
                    ("Create4", Request::<CREATE_4>::LOGICAL_LABEL),
                    ("Insert1", Request::<INSERT_1>::LOGICAL_LABEL),
                    ("Insert3", Request::<INSERT_3>::LOGICAL_LABEL),
                    ("Insert4", Request::<INSERT_4>::LOGICAL_LABEL),
                    ("Loan", Loan::<DOCUMENT>::LOGICAL_LABEL),
                ].into_iter().map(|(name, label)| (name.to_owned(), Value::from(label))).collect();
                json!({
                    "type":"compiled_projection", "format_version":1, "task":"7H", "session":SESSION,
                    "roles":roles,
                    "labels":labels,
                    "programs":programs.as_slice(),
                })
            }
        }
    }
}

roles!(
    owner: DOCUMENT_OWNER, document: DOCUMENT, drafting: DRAFTING, references: REFERENCES,
    a_file: WRITER_A_FILE, b_file: WRITER_B_FILE, c_file: WRITER_C_FILE,
    a_data: WRITER_A_DATA, b_data: WRITER_B_DATA, c_data: WRITER_C_DATA,
    a_vm: WRITER_A_VM, b_vm: WRITER_B_VM, c_vm: WRITER_C_VM,
    a_io: WRITER_A_IO, b_io: WRITER_B_IO, c_io: WRITER_C_IO,
    figure_owner: FIGURE_OWNER, figure_store: FIGURE_STORE, figure_creator: FIGURE_CREATOR,
    figure_file: FIGURE_FILE, figure_data: FIGURE_DATA, figure_vm: FIGURE_VM, figure_io: FIGURE_IO,
    fact_part_i: FACT_PART_I,
    fact_editorial: FACT_EDITORIAL, fact_3: FACT_3, fact_4: FACT_4, fact_file: FACT_FILE,
    fact_data: FACT_DATA, fact_vm: FACT_VM, fact_io: FACT_IO,
    review_file: REVIEW_FILE, review_data: REVIEW_DATA, review_vm: REVIEW_VM, review_io: REVIEW_IO,
    editor: EDITOR, editor_file: EDITOR_FILE, editor_data: EDITOR_DATA, editor_vm: EDITOR_VM,
    editor_io: EDITOR_IO,
);

/// Run the writers, the figure creator and their shared document preparation.
#[hibana_runtime::scope]
pub async fn run_stdio(wash: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        bus,
        mut io,
        [
            a_terminal,
            b_terminal,
            c_terminal,
            figure_terminal,
            fact_terminal,
            review_terminal,
            editor_terminal,
        ],
    ) = Bus::open(
        [
            "WRITER_A",
            "WRITER_B",
            "WRITER_C",
            "FIGURE_CREATOR",
            "FACT_CHECKER",
            "REVIEWER",
            "EDITOR_IN_CHIEF",
        ],
        [
            DOCUMENT,
            FIGURE_STORE,
            FIGURE_FILE,
            WRITER_A_FILE,
            WRITER_B_FILE,
            WRITER_C_FILE,
            FACT_FILE,
            REVIEW_FILE,
            EDITOR_FILE,
        ]
        .map(|role| (session, role)),
    );
    let trial = async {
        let programs = Programs::new();
        bus.observe(programs.observation());
        let capture = |event| {
            bus.observe(json!({"type":"protocol_capture","producer":0,
                "records":[projection::record(event)]}))
        };
        let mut slab = vec![0; SESSION_STORAGE_BYTES];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let tap = rv.tap();
        let result = async {
            let mut owner = rv.enter(session, &programs.owner)?;
            let doc = rv.enter(session, &programs.document)?;
            let mut drafting = rv.enter(session, &programs.drafting)?;
            let mut references = rv.enter(session, &programs.references)?;
            let mut a_data = rv.enter(session, &programs.a_data)?;
            let mut b_data = rv.enter(session, &programs.b_data)?;
            let mut c_data = rv.enter(session, &programs.c_data)?;
            let a_file = rv.enter(session, &programs.a_file)?;
            let b_file = rv.enter(session, &programs.b_file)?;
            let c_file = rv.enter(session, &programs.c_file)?;
            let a_vm = rv.enter(session, &programs.a_vm)?;
            let b_vm = rv.enter(session, &programs.b_vm)?;
            let c_vm = rv.enter(session, &programs.c_vm)?;
            let mut a_io = rv.enter(session, &programs.a_io)?;
            let mut b_io = rv.enter(session, &programs.b_io)?;
            let mut c_io = rv.enter(session, &programs.c_io)?;
            let mut figure_owner = rv.enter(session, &programs.figure_owner)?;
            let store = rv.enter(session, &programs.figure_store)?;
            let mut creator = rv.enter(session, &programs.figure_creator)?;
            let mut figure_data = rv.enter(session, &programs.figure_data)?;
            let file = rv.enter(session, &programs.figure_file)?;
            let vm = rv.enter(session, &programs.figure_vm)?;
            let mut figure_io = rv.enter(session, &programs.figure_io)?;
            let mut fact_part_i = rv.enter(session, &programs.fact_part_i)?;
            let mut fact_editorial = rv.enter(session, &programs.fact_editorial)?;
            let mut fact_3 = rv.enter(session, &programs.fact_3)?;
            let mut fact_4 = rv.enter(session, &programs.fact_4)?;
            let mut fact_data = rv.enter(session, &programs.fact_data)?;
            let fact_file = rv.enter(session, &programs.fact_file)?;
            let fact_vm = rv.enter(session, &programs.fact_vm)?;
            let mut fact_io = rv.enter(session, &programs.fact_io)?;
            let mut review_data = rv.enter(session, &programs.review_data)?;
            let review_file = rv.enter(session, &programs.review_file)?;
            let review_vm = rv.enter(session, &programs.review_vm)?;
            let mut review_io = rv.enter(session, &programs.review_io)?;
            let mut editor = rv.enter(session, &programs.editor)?;
            let mut editor_data = rv.enter(session, &programs.editor_data)?;
            let editor_file = rv.enter(session, &programs.editor_file)?;
            let editor_vm = rv.enter(session, &programs.editor_vm)?;
            let mut editor_io = rv.enter(session, &programs.editor_io)?;
            async { document_owner(&mut owner, doc, &bus).await };
            async {
                write_drafts(&mut drafting, &bus).await?;
                Ok(())
            };
            async {
                update_references(&mut references, &bus).await?;
                Ok(())
            };
            async { file_metadata::run(&mut a_data, a_file, A_FILES).await };
            async { file_metadata::run(&mut b_data, b_file, B_FILES).await };
            async { file_metadata::run(&mut c_data, c_file, C_FILES).await };
            async { private_io::run(&mut a_io, PrivateFs::new(a_terminal, &bus, &A_FILES)).await };
            async { private_io::run(&mut b_io, PrivateFs::new(b_terminal, &bus, &B_FILES)).await };
            async { private_io::run(&mut c_io, PrivateFs::new(c_terminal, &bus, &C_FILES)).await };
            async { writer_a(wash, a_vm).await };
            async { writer_b(wash, b_vm).await };
            async { writer_c(wash, c_vm).await };
            async {
                figure_owner.send::<Loan<FIGURE_STORE>>(store).await?;
                drop(figure_owner.recv::<Loan<FIGURE_STORE>>().await?);
                Ok(())
            };
            async { figure_store(&mut creator, &bus).await };
            async { file_metadata::run(&mut figure_data, file, FIGURE_FILES).await };
            async {
                private_io::run(
                    &mut figure_io,
                    PrivateFs::new(figure_terminal, &bus, &FIGURE_FILES),
                )
                .await
            };
            async { figure_guest(wash, vm).await };
            async { check_and_review(&mut fact_part_i, &bus).await };
            async { check_and_review(&mut fact_editorial, &bus).await };
            async {
                check_chapter::<FACT_3, CHECK_3, MATERIAL_3, REVISE_3, MINOR_3>(&mut fact_3, &bus)
                    .await
            };
            async {
                check_chapter::<FACT_4, CHECK_4, MATERIAL_4, REVISE_4, MINOR_4>(&mut fact_4, &bus)
                    .await
            };
            async { file_metadata::run(&mut fact_data, fact_file, FACT_FILES).await };
            async {
                private_io::run(
                    &mut fact_io,
                    PrivateFs::new(fact_terminal, &bus, &FACT_FILES),
                )
                .await
            };
            async { fact_guest(wash, fact_vm).await };
            async { file_metadata::run(&mut review_data, review_file, REVIEW_FILES).await };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(review_terminal, &bus, &REVIEW_FILES),
                )
                .await
            };
            async { review_guest(wash, review_vm).await };
            async { edit_document(&mut editor, &bus).await };
            async { file_metadata::run(&mut editor_data, editor_file, EDITOR_FILES).await };
            async {
                private_io::run(
                    &mut editor_io,
                    PrivateFs::new(editor_terminal, &bus, &EDITOR_FILES),
                )
                .await
            };
            async { editor_guest(wash, editor_vm).await };
            Ok(())
        }
        .await;
        bus.observe(
            json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,
            "checkpoint":tap.checkpoint()}),
        );
        result
    };
    let result = hibana_runtime::race(trial, io.run(&bus)).await;
    let report = match &result {
        Ok(()) => json!({"type":"run_result","status":"complete"}),
        Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
    };
    io.emit(&bus, report).await?;
    result
}
