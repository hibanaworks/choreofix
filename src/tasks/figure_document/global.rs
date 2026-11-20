//! The two independent preparations meet when the creator borrows both resources.
use super::Arguments;
use crate::shell::global::{Exit, FileExit, Loan, file_metadata, private_io};
use hibana::runtime::program::{RoleProgram, project};
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};

pub const DOCUMENT: u8 = 1;
pub const FIGURE_STORE: u8 = 2;
pub const DOCUMENT_OWNER: u8 = 3;
pub const FIGURE_OWNER: u8 = 4;
pub const FIGURE_CREATOR: u8 = 5;
pub const WRITER_FILE: u8 = 6;
pub const FIGURE_FILE: u8 = 7;
pub const FIGURE_DATA: u8 = 8;
pub const FIGURE_VM: u8 = 9;
pub const FIGURE_IO: u8 = 10;
pub const WRITER_A_SNAPSHOTS: u8 = 11;
pub const WRITER_B_SNAPSHOTS: u8 = 12;
pub const WRITER_B_FILE: u8 = 13;
pub const FACTS_12: u8 = 17;
pub const FACTS_3: u8 = 18;
pub const FACT_FILE: u8 = 19;
pub const EDITOR: u8 = 20;
pub const EDITOR_FILE: u8 = 21;
pub const EDITOR_DATA: u8 = 22;
pub const EDITOR_VM: u8 = 23;
pub const EDITOR_IO: u8 = 24;
pub const ARCHIVE: u8 = 33;
pub const FACT_DATA: u8 = 34;
pub const FACT_VM: u8 = 35;
pub const FACT_IO: u8 = 36;
pub const DRAFTING: u8 = 25;
pub const WRITER_A_DATA: u8 = 26;
pub const WRITER_B_DATA: u8 = 27;
pub const REFERENCES: u8 = 28;
pub const WRITER_A_VM: u8 = 29;
pub const WRITER_B_VM: u8 = 30;
pub const WRITER_A_IO: u8 = 31;
pub const WRITER_B_IO: u8 = 32;

pub const WRITE: u8 = 1;
pub const CREATE_1: u8 = 41;
pub const CREATE_3: u8 = 42;
pub const INSERT_1: u8 = 39;
pub const INSERT_3: u8 = 40;
pub const READ: u8 = 4;
pub const REVISE_A: u8 = 5;
pub const REVISE_B: u8 = 38;
pub const CHECK_12: u8 = 15;
pub const CHECK_3: u8 = 16;
pub const REVIEW_1: u8 = 17;
pub const REVIEW_2: u8 = 18;
pub const REVIEW_3: u8 = 19;
pub const COMBINE: u8 = 20;
pub const DRAFT_1: u8 = 26;
pub const DRAFT_2: u8 = 27;
pub const DRAFT_3: u8 = 28;
pub const REFERENCES_1: u8 = 29;
pub const REFERENCES_2: u8 = 30;
pub const REFERENCES_3: u8 = 31;

#[cfg(test)]
pub type Write = g::Msg<1, Arguments<WRITE>>;
#[cfg(test)]
pub type Written<'a> = g::Msg<2, &'a [u8]>;
pub type Create<const OP: u8> = g::Msg<OP, Arguments<OP>>;
pub type Created<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
pub type Insert<const CHAPTER: u8> = g::Msg<CHAPTER, Arguments<CHAPTER>>;
pub type Inserted<'a, const CHAPTER: u8> = g::Msg<CHAPTER, &'a [u8]>;
pub type ReadSection = g::Msg<10, Arguments<READ>>;
pub type SectionSnapshot<'a> = g::Msg<11, &'a [u8]>;
pub type ReadingFinished = g::Msg<12, i32>;
pub type Revise<const OP: u8> = g::Msg<13, Arguments<OP>>;
pub type Revised<'a> = g::Msg<14, &'a [u8]>;
pub type QualityRequest<const OP: u8> = g::Msg<OP, Arguments<OP>>;
pub type QualityResult<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
pub type ReviewMaterial<'a> = g::Msg<21, &'a [u8]>;
pub type Combine = g::Msg<COMBINE, Arguments<COMBINE>>;
pub type AdmittedCombine = g::Msg<22, Arguments<COMBINE>>;
pub type CombineOnDocument = g::Msg<23, Arguments<COMBINE>>;
pub type Combined<'a> = g::Msg<24, &'a [u8]>;
pub type Reconsider<'a> = g::Msg<25, &'a [u8]>;
pub type FactOutcome<'a> = g::Msg<FACT_FILE, &'a [u8]>;
pub type WriterOutcome<'a, const FILE: u8> = g::Msg<FILE, &'a [u8]>;
pub type FactsFailed = g::Msg<35, ()>;
pub type Published<'a, const RECIPIENT: u8> = g::Msg<RECIPIENT, &'a [u8]>;
pub type DraftRequest<const CHAPTER: u8> = g::Msg<CHAPTER, Arguments<CHAPTER>>;
pub type DraftWritten<'a, const CHAPTER: u8> = g::Msg<CHAPTER, &'a [u8]>;
pub type ReferenceRequest<const OP: u8> = g::Msg<OP, Arguments<OP>>;
pub type ReferenceResult<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        FdBinding {
            read: Some(FdReadRow::Base),
            write: Some(FdWriteRow::Object),
            readdir: None,
        },
    )
}

const fn incoming(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}

pub const FACT_OBJECTS: [ChoreoFsObject<'static>; 3] = [
    incoming(b"service/outcome", 4),
    operation(b"service/fact_check/CHAPTERS_1_2", 5),
    operation(b"service/fact_check/CHAPTER_3", 6),
];

pub const FIGURE_OBJECTS: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/create_figure/CHAPTER_1", 4),
    operation(b"service/create_figure/CHAPTER_3", 5),
    operation(b"service/insert_figure/CHAPTER_1", 6),
    operation(b"service/insert_figure/CHAPTER_3", 7),
];

pub const READ_SECTION: ChoreoFsObject<'static> = operation(b"service/read_section", 4);
#[cfg(test)]
pub const WRITE_SECTION: ChoreoFsObject<'static> = operation(b"service/write_section", 5);

pub const WRITER_A_OBJECTS: [ChoreoFsObject<'static>; 7] = [
    READ_SECTION,
    operation(b"service/write_section/CHAPTER_1", 5),
    operation(b"service/write_section/CHAPTER_2", 6),
    operation(b"service/update_references/CHAPTER_1", 7),
    operation(b"service/update_references/CHAPTER_2", 8),
    incoming(b"service/outcome", 9),
    operation(b"service/revise_section", 10),
];
pub const WRITER_B_OBJECTS: [ChoreoFsObject<'static>; 5] = [
    READ_SECTION,
    operation(b"service/write_section/CHAPTER_3", 5),
    operation(b"service/update_references/CHAPTER_3", 6),
    incoming(b"service/outcome", 7),
    operation(b"service/revise_section", 8),
];

pub const EDITOR_OBJECTS: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/review_sections/CHAPTER_1", 4),
    operation(b"service/review_sections/CHAPTER_2", 5),
    operation(b"service/review_sections/CHAPTER_3", 6),
    operation(b"service/combine_sections", 7),
    incoming(b"service/review_material", 8),
];

pub fn draft<const FILE: u8, const OP: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<FILE, DOCUMENT, DraftRequest<OP>>(),
        g::send::<DOCUMENT, FILE, DraftWritten<OP>>(),
    )
}

#[cfg(test)]
fn draft_order() -> impl Projectable {
    let first = draft::<WRITER_FILE, DRAFT_1>();
    let second = draft::<WRITER_FILE, DRAFT_2>();
    let third = draft::<WRITER_B_FILE, DRAFT_3>();
    g::route(
        g::seq(
            first,
            g::route(g::seq(second, third), g::seq(third, second)),
        ),
        g::route(
            g::seq(second, g::route(g::seq(first, third), g::seq(third, first))),
            g::seq(
                third,
                g::route(g::seq(first, second), g::seq(second, first)),
            ),
        ),
    )
}

pub fn lend_writer_files() -> impl Projectable {
    g::par(
        g::send::<WRITER_A_DATA, DRAFTING, Loan<WRITER_FILE>>(),
        g::send::<WRITER_B_DATA, DRAFTING, Loan<WRITER_B_FILE>>(),
    )
}

/// The two file views share an actor. The document completes the three
/// required drafts in the order requested by the writers.
#[cfg(test)]
pub fn initial_drafts() -> impl Projectable {
    g::seq(lend_writer_files(), draft_order())
}

fn references<const FILE: u8, const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, REFERENCES, ReferenceRequest<OP>>(),
        g::send::<REFERENCES, FILE, ReferenceResult<OP>>(),
    )
}

pub fn chapter_references() -> impl Projectable {
    g::par(
        references::<WRITER_FILE, REFERENCES_1>(),
        g::par(
            references::<WRITER_FILE, REFERENCES_2>(),
            references::<WRITER_B_FILE, REFERENCES_3>(),
        ),
    )
}

/// Drafts use the document's exclusive endpoint. References and read requests
/// can progress while a draft is pending, including in the other writer's VM.
pub fn writer_activities(chapter_work: impl Projectable) -> impl Projectable {
    let close_a = g::send::<WRITER_FILE, WRITER_A_DATA, FileExit>();
    let close_b = g::send::<WRITER_B_FILE, WRITER_B_DATA, FileExit>();
    let exit_a = g::send::<WRITER_A_VM, WRITER_A_IO, Exit>();
    let exit_b = g::send::<WRITER_B_VM, WRITER_B_IO, Exit>();
    g::par(
        chapter_work,
        g::par(
            g::par(
                snapshot_reads::<WRITER_FILE, WRITER_A_SNAPSHOTS>(),
                snapshot_reads::<WRITER_B_FILE, WRITER_B_SNAPSHOTS>(),
            ),
            g::par(
                g::par(
                    g::route(
                        close_a,
                        g::seq(
                            file_metadata::<WRITER_FILE, WRITER_A_DATA>().roll(),
                            close_a,
                        ),
                    ),
                    g::route(
                        close_b,
                        g::seq(
                            file_metadata::<WRITER_B_FILE, WRITER_B_DATA>().roll(),
                            close_b,
                        ),
                    ),
                ),
                g::par(
                    g::route(
                        exit_a,
                        g::seq(private_io::<WRITER_A_VM, WRITER_A_IO>(), exit_a),
                    ),
                    g::route(
                        exit_b,
                        g::seq(private_io::<WRITER_B_VM, WRITER_B_IO>(), exit_b),
                    ),
                ),
            ),
        ),
    )
}

#[cfg(test)]
pub fn document_preparation() -> impl Projectable {
    g::seq(
        lend_writer_files(),
        writer_activities(g::par(draft_order(), chapter_references())),
    )
}

pub fn lend_fact_scopes() -> impl Projectable {
    g::par(
        g::send::<FACT_DATA, FACT_VM, Loan<FACTS_12>>(),
        g::send::<FACT_DATA, FACT_VM, Loan<FACTS_3>>(),
    )
}

fn fact_check<const FACT: u8, const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FACT_FILE, FACT, QualityRequest<OP>>(),
        g::seq(
            g::send::<FACT, FACT_FILE, QualityResult<OP>>(),
            g::send::<FACT, DOCUMENT_OWNER, QualityResult<OP>>(),
        ),
    )
}

fn review<const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<EDITOR_FILE, EDITOR, QualityRequest<OP>>(),
        g::seq(
            g::send::<EDITOR, EDITOR_FILE, QualityResult<OP>>(),
            g::send::<EDITOR, DOCUMENT_OWNER, QualityResult<OP>>(),
        ),
    )
}

fn assembly() -> impl Projectable {
    g::seq(
        g::send::<DOCUMENT_OWNER, EDITOR, Loan<DOCUMENT>>(),
        g::seq(
            g::send::<EDITOR_FILE, DOCUMENT_OWNER, Combine>(),
            g::seq(
                g::send::<DOCUMENT_OWNER, EDITOR, AdmittedCombine>(),
                g::seq(
                    g::send::<DOCUMENT, DOCUMENT_OWNER, CombineOnDocument>(),
                    g::par(
                        g::send::<EDITOR, ARCHIVE, Loan<DOCUMENT>>(),
                        g::send::<DOCUMENT_OWNER, EDITOR_FILE, Combined>(),
                    ),
                ),
            ),
        ),
    )
}

fn editorial_checks() -> impl Projectable {
    g::route(
        g::seq(
            g::send::<DOCUMENT_OWNER, EDITOR, ReviewMaterial>(),
            g::seq(
                g::send::<EDITOR, EDITOR_FILE, ReviewMaterial>(),
                g::par(
                    review::<REVIEW_1>(),
                    g::par(review::<REVIEW_2>(), review::<REVIEW_3>()),
                ),
            ),
        ),
        g::send::<DOCUMENT_OWNER, EDITOR, FactsFailed>(),
    )
}

pub fn reconsidering_publication() -> impl Projectable {
    let revise = g::route(
        g::seq(
            g::send::<DOCUMENT_OWNER, WRITER_FILE, WriterOutcome<WRITER_FILE>>(),
            g::seq(
                revise_chapter::<WRITER_FILE, REVISE_A>(),
                g::seq(
                    g::send::<DOCUMENT_OWNER, FACT_FILE, FactOutcome>(),
                    fact_check::<FACTS_12, CHECK_12>(),
                ),
            ),
        ),
        g::seq(
            g::send::<DOCUMENT_OWNER, WRITER_B_FILE, WriterOutcome<WRITER_B_FILE>>(),
            g::seq(
                revise_chapter::<WRITER_B_FILE, REVISE_B>(),
                g::seq(
                    g::send::<DOCUMENT_OWNER, FACT_FILE, FactOutcome>(),
                    fact_check::<FACTS_3, CHECK_3>(),
                ),
            ),
        ),
    );
    let finish = g::seq(
        assembly(),
        g::par(
            g::send::<DOCUMENT_OWNER, FACT_FILE, FactOutcome>(),
            g::par(
                g::send::<DOCUMENT_OWNER, FACTS_12, Published<FACTS_12>>(),
                g::par(
                    g::send::<DOCUMENT_OWNER, FACTS_3, Published<FACTS_3>>(),
                    g::par(
                        g::send::<DOCUMENT_OWNER, WRITER_FILE, WriterOutcome<WRITER_FILE>>(),
                        g::send::<DOCUMENT_OWNER, WRITER_B_FILE, WriterOutcome<WRITER_B_FILE>>(),
                    ),
                ),
            ),
        ),
    );
    let repair = g::seq(
        g::send::<DOCUMENT, EDITOR, Reconsider>(),
        g::seq(revise, editorial_checks()),
    );
    g::seq(
        g::seq(
            g::send::<DOCUMENT_OWNER, FACT_FILE, FactOutcome>(),
            g::par(
                fact_check::<FACTS_12, CHECK_12>(),
                fact_check::<FACTS_3, CHECK_3>(),
            ),
        ),
        g::seq(
            editorial_checks(),
            g::route(finish, g::seq(repair.roll(), finish)),
        ),
    )
}

pub fn snapshot_reads<const FILE: u8, const SNAPSHOTS: u8>() -> impl Projectable {
    let query = g::seq(
        g::send::<FILE, SNAPSHOTS, ReadSection>(),
        g::send::<SNAPSHOTS, FILE, SectionSnapshot>(),
    );
    let finished = g::send::<FILE, SNAPSHOTS, ReadingFinished>();
    g::route(finished, g::seq(query.roll(), finished))
}

#[cfg(test)]
pub fn write_chapter<const FILE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, DOCUMENT, Write>(),
        g::send::<DOCUMENT, FILE, Written>(),
    )
}

pub fn revise_chapter<const FILE: u8, const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, DOCUMENT, Revise<OP>>(),
        g::send::<DOCUMENT, FILE, Revised>(),
    )
}

/// Creation and writing use different resources. Insertion needs both results.
pub fn figure_chapter<const CREATE: u8, const INSERT: u8>(
    write: impl Projectable,
) -> impl Projectable {
    let create = g::seq(
        g::send::<FIGURE_FILE, FIGURE_STORE, Create<CREATE>>(),
        g::send::<FIGURE_STORE, FIGURE_FILE, Created<CREATE>>(),
    );
    let insert = g::seq(
        g::send::<FIGURE_FILE, DOCUMENT, Insert<INSERT>>(),
        g::seq(
            g::send::<DOCUMENT, FIGURE_STORE, Insert<INSERT>>(),
            g::seq(
                g::send::<FIGURE_STORE, DOCUMENT, Insert<INSERT>>(),
                g::seq(
                    g::send::<DOCUMENT, FIGURE_STORE, Inserted<INSERT>>(),
                    g::send::<FIGURE_STORE, FIGURE_FILE, Inserted<INSERT>>(),
                ),
            ),
        ),
    );
    g::seq(g::par(write, create), insert)
}

pub fn prepare_chapters(work: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(
            g::send::<DOCUMENT_OWNER, DRAFTING, Loan<DOCUMENT>>(),
            g::send::<FIGURE_OWNER, FIGURE_CREATOR, Loan<FIGURE_STORE>>(),
        ),
        g::seq(
            work,
            g::par(
                g::send::<DRAFTING, DOCUMENT_OWNER, Loan<DOCUMENT>>(),
                g::send::<FIGURE_CREATOR, FIGURE_OWNER, Loan<FIGURE_STORE>>(),
            ),
        ),
    )
}

pub fn figure_activities(chapter_work: impl Projectable) -> impl Projectable {
    let file_done = g::send::<FIGURE_FILE, FIGURE_DATA, FileExit>();
    let vm_done = g::send::<FIGURE_VM, FIGURE_IO, Exit>();
    g::par(
        chapter_work,
        g::par(
            g::route(
                file_done,
                g::seq(
                    file_metadata::<FIGURE_FILE, FIGURE_DATA>().roll(),
                    file_done,
                ),
            ),
            g::route(
                vm_done,
                g::seq(private_io::<FIGURE_VM, FIGURE_IO>(), vm_done),
            ),
        ),
    )
}

#[cfg(test)]
pub fn chapter_with_wasm_figure_creator() -> impl Projectable {
    g::seq(
        g::send::<FIGURE_DATA, FIGURE_VM, Loan<FIGURE_FILE>>(),
        figure_activities(prepare_chapters(figure_chapter::<CREATE_1, INSERT_1>(
            draft::<WRITER_FILE, DRAFT_1>(),
        ))),
    )
}

pub const SESSION: u32 = 85;

fn protocol() -> impl Projectable {
    let chapters = prepare_chapters(g::par(
        figure_chapter::<CREATE_1, INSERT_1>(draft::<WRITER_FILE, DRAFT_1>()),
        g::par(
            draft::<WRITER_FILE, DRAFT_2>(),
            figure_chapter::<CREATE_3, INSERT_3>(draft::<WRITER_B_FILE, DRAFT_3>()),
        ),
    ));
    let fact_file_exit = g::send::<FACT_FILE, FACT_DATA, FileExit>();
    let fact_exit = g::send::<FACT_VM, FACT_IO, Exit>();
    let editor_file_exit = g::send::<EDITOR_FILE, EDITOR_DATA, FileExit>();
    let editor_exit = g::send::<EDITOR_VM, EDITOR_IO, Exit>();
    g::seq(
        g::par(
            lend_writer_files(),
            g::par(
                g::seq(
                    lend_fact_scopes(),
                    g::send::<FACT_DATA, FACT_VM, Loan<FACT_FILE>>(),
                ),
                g::par(
                    g::send::<FIGURE_DATA, FIGURE_VM, Loan<FIGURE_FILE>>(),
                    g::send::<EDITOR_DATA, EDITOR, Loan<EDITOR_FILE>>(),
                ),
            ),
        ),
        writer_activities(g::par(
            g::seq(
                g::par(figure_activities(chapters), chapter_references()),
                reconsidering_publication(),
            ),
            g::par(
                g::route(
                    fact_file_exit,
                    g::seq(
                        file_metadata::<FACT_FILE, FACT_DATA>().roll(),
                        fact_file_exit,
                    ),
                ),
                g::par(
                    g::route(
                        fact_exit,
                        g::seq(private_io::<FACT_VM, FACT_IO>(), fact_exit),
                    ),
                    g::par(
                        g::route(
                            editor_file_exit,
                            g::seq(
                                file_metadata::<EDITOR_FILE, EDITOR_DATA>().roll(),
                                editor_file_exit,
                            ),
                        ),
                        g::route(
                            editor_exit,
                            g::seq(private_io::<EDITOR_VM, EDITOR_IO>(), editor_exit),
                        ),
                    ),
                ),
            ),
        )),
    )
}

// The same role list constructs every image used by the runtime and its export.
macro_rules! programs {
    ($($field:ident : $role:ident),+ $(,)?) => {
        pub(super) struct Programs {
            $(pub(super) $field: RoleProgram<$role>,)+
        }

        impl Programs {
            pub(super) fn new() -> Self {
                let source = protocol();
                Self {
                    $($field: project::<$role>(&source),)+
                }
            }

            pub(super) fn observation(&self) -> serde_json::Value {
                use crate::imp::projection::rows;
                use hibana::g::Message;
                let roles: serde_json::Map<String, serde_json::Value> =
                    [$( (stringify!($role), $role), )+].into_iter()
                        .map(|(name, role)| (name.into(), role.into()))
                        .collect();
                let programs = [$(rows(&self.$field),)+];
                let labels = serde_json::json!({
                        "draft_1": DraftRequest::<DRAFT_1>::LOGICAL_LABEL,
                        "draft_2": DraftRequest::<DRAFT_2>::LOGICAL_LABEL,
                        "draft_3": DraftRequest::<DRAFT_3>::LOGICAL_LABEL,
                        "references_1": ReferenceRequest::<REFERENCES_1>::LOGICAL_LABEL,
                        "references_2": ReferenceRequest::<REFERENCES_2>::LOGICAL_LABEL,
                        "references_3": ReferenceRequest::<REFERENCES_3>::LOGICAL_LABEL,
                        "fact_outcome": FactOutcome::LOGICAL_LABEL,
                        "check_12": QualityRequest::<CHECK_12>::LOGICAL_LABEL,
                        "check_3": QualityRequest::<CHECK_3>::LOGICAL_LABEL,
                        "review_1": QualityRequest::<REVIEW_1>::LOGICAL_LABEL,
                        "review_2": QualityRequest::<REVIEW_2>::LOGICAL_LABEL,
                        "review_3": QualityRequest::<REVIEW_3>::LOGICAL_LABEL,
                        "read": ReadSection::LOGICAL_LABEL,
                        "snapshot": SectionSnapshot::LOGICAL_LABEL,
                        "writer_a_outcome": WriterOutcome::<WRITER_FILE>::LOGICAL_LABEL,
                        "writer_b_outcome": WriterOutcome::<WRITER_B_FILE>::LOGICAL_LABEL,
                        "revise": Revise::<REVISE_A>::LOGICAL_LABEL,
                        "revised": Revised::LOGICAL_LABEL,
                        "create_1": Create::<CREATE_1>::LOGICAL_LABEL,
                        "create_3": Create::<CREATE_3>::LOGICAL_LABEL,
                        "insert_1": Insert::<INSERT_1>::LOGICAL_LABEL,
                        "insert_3": Insert::<INSERT_3>::LOGICAL_LABEL,
                        "combine": Combine::LOGICAL_LABEL,
                        "combine_document": CombineOnDocument::LOGICAL_LABEL,
                        "combined": Combined::LOGICAL_LABEL,
                    });
                serde_json::json!({
                    "type": "compiled_projection",
                    "task": "7M",
                    "format_version": 1,
                    "session": SESSION,
                    "roles": roles,
                    "labels": labels,
                    "programs": programs.as_slice(),
                })
            }
        }
    };
}

programs! {
    document: DOCUMENT,
    owner: DOCUMENT_OWNER,
    store: FIGURE_STORE,
    store_owner: FIGURE_OWNER,
    creator: FIGURE_CREATOR,
    figure_file: FIGURE_FILE,
    figure_data: FIGURE_DATA,
    figure_vm: FIGURE_VM,
    figure_io: FIGURE_IO,
    drafting: DRAFTING,
    writer_a: WRITER_A_DATA,
    writer_b: WRITER_B_DATA,
    a_vm: WRITER_A_VM,
    b_vm: WRITER_B_VM,
    a_io: WRITER_A_IO,
    b_io: WRITER_B_IO,
    a_snapshots: WRITER_A_SNAPSHOTS,
    b_snapshots: WRITER_B_SNAPSHOTS,
    file_a: WRITER_FILE,
    file_b: WRITER_B_FILE,
    references: REFERENCES,
    fact12: FACTS_12,
    fact3: FACTS_3,
    fact_file: FACT_FILE,
    fact_data: FACT_DATA,
    fact_vm: FACT_VM,
    fact_io: FACT_IO,
    editor: EDITOR,
    editor_data: EDITOR_DATA,
    editor_file: EDITOR_FILE,
    editor_vm: EDITOR_VM,
    editor_io: EDITOR_IO,
    archive: ARCHIVE,
}
