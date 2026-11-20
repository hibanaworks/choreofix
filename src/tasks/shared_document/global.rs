//! One document authority; shell files are projections of this same source.
use crate::shell::global::{FileExit, Loan};
use crate::shell::imp::{report_arguments::ReportArguments, review_reply::ReviewReply};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const WRITER_A: u8 = 0;
pub const WRITER_B: u8 = 1;
pub const EDITOR: u8 = 2;
pub const DOCUMENT: u8 = 3;
pub const DOCUMENT_SERVICE: u8 = 4;
pub const A_IO: u8 = 5;
pub const B_IO: u8 = 6;
pub const EDITOR_IO: u8 = 7;
pub const ALLOCATION_FILE: u8 = 8;
pub const ALLOCATION_DATA: u8 = 9;
pub const A_FILE: u8 = 10;
pub const A_DATA: u8 = 11;
pub const B_FILE: u8 = 12;
pub const B_DATA: u8 = 13;
pub const A_VM: u8 = 14;
pub const B_VM: u8 = 15;
pub const EDITOR_VM: u8 = 16;
pub const SESSION: u32 = 73;
pub const CHAPTER_A: &str = "CHAPTER_1";
pub const CHAPTER_B: &str = "CHAPTER_2";

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
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
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )
}
pub const WRITER_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/write_section", 4),
    operation(b"service/update_references", 5),
    operation(b"service/revise_section", 17),
    incoming(b"service/feedback", 18),
];
pub const ALLOCATION_OBJECT: ChoreoFsObject<'static> = operation(b"service/allocate_document", 4);
pub const EDITOR_FILES: [ChoreoFsObject<'static>; 6] = [
    ALLOCATION_OBJECT,
    operation(b"service/review_chapter_1", 5),
    operation(b"service/review_chapter_2", 17),
    operation(b"service/reconsider_a", 18),
    operation(b"service/reconsider_b", 19),
    operation(b"service/combine_sections", 20),
];

pub type Allocate = g::Msg<44, ReportArguments<7>>;
pub type AllocationAccepted<'a> = g::Msg<45, &'a [u8]>;
pub type InitialDocument<'r> = g::Msg<1, Endpoint<'r, DOCUMENT>>;
pub type InitialDocumentB<'r> = g::Msg<2, Endpoint<'r, DOCUMENT>>;
pub type RevisionDocument<'r> = g::Msg<3, Endpoint<'r, DOCUMENT>>;
pub type WriteA = g::Msg<4, ReportArguments<8>>;
pub type WriteB = g::Msg<5, ReportArguments<9>>;
const WRITE_ON_DOCUMENT: u8 = 6;
pub type WriteOnDocumentA = g::Msg<WRITE_ON_DOCUMENT, ReportArguments<8, true>>;
pub type WriteOnDocumentB = g::Msg<WRITE_ON_DOCUMENT, ReportArguments<9, true>>;
pub type Written<'a> = g::Msg<7, &'a [u8]>;
pub type ReferencesA = g::Msg<8, ReportArguments<10, true>>;
pub type ReferencesB = g::Msg<9, ReportArguments<11, true>>;
pub type ReferencedA<'a> = g::Msg<10, &'a [u8]>;
pub type ReferencedB<'a> = g::Msg<11, &'a [u8]>;
pub type MaterialA<'a> = g::Msg<12, &'a [u8]>;
pub type MaterialB<'a> = g::Msg<13, &'a [u8]>;
pub type ReferenceMaterialA<'a> = g::Msg<14, &'a [u8]>;
pub type ReferenceMaterialB<'a> = g::Msg<15, &'a [u8]>;
pub type ReviewA = g::Msg<16, ReportArguments<12, true>>;
pub type ReviewB = g::Msg<17, ReportArguments<13, true>>;
pub type ReviewedA<'a> = g::Msg<18, &'a [u8]>;
pub type ReviewedB<'a> = g::Msg<19, &'a [u8]>;
pub type RevisionReviewA = g::Msg<21, ReportArguments<17>>;
pub type RevisionReviewB = g::Msg<23, ReportArguments<17>>;
pub type ReviseA = g::Msg<24, ReportArguments<30>>;
pub type ReviseB = g::Msg<25, ReportArguments<31>>;
const REVISE_ON_DOCUMENT: u8 = 26;
pub type ReviseOnDocumentA = g::Msg<REVISE_ON_DOCUMENT, ReportArguments<30, true>>;
pub type ReviseOnDocumentB = g::Msg<REVISE_ON_DOCUMENT, ReportArguments<31, true>>;
pub type Revised<'a> = g::Msg<27, &'a [u8]>;
pub type Combine = g::Msg<28, ReportArguments<6>>;
pub type CombineOnDocument = g::Msg<29, ReportArguments<6, true>>;
pub type Combined<'a> = g::Msg<30, &'a [u8]>;
pub type FinishedB<'a> = g::Msg<32, &'a [u8]>;
pub type DocumentReturn<'r> = g::Msg<33, Endpoint<'r, DOCUMENT>>;
pub type FinalApprovedA<'a> = g::Msg<34, ReviewReply<'a, true>>;
pub type FinalApprovedB<'a> = g::Msg<35, ReviewReply<'a, true>>;
pub type UpdatedDocument<'a> = g::Msg<36, &'a [u8]>;
pub type FrozenDocument<'r> = g::Msg<37, Endpoint<'r, DOCUMENT>>;
pub type CombineForward = g::Msg<38, ReportArguments<6>>;
pub type CombinedForEditor<'a> = g::Msg<39, &'a [u8]>;
pub type ReconsiderA = g::Msg<40, ReportArguments<17>>;
pub type ReconsiderB = g::Msg<41, ReportArguments<17>>;
pub type RevisionReviewAForB = g::Msg<42, ReportArguments<17>>;
pub type RevisionReviewBForA = g::Msg<43, ReportArguments<17>>;
pub type WriteArgumentsA = g::Msg<48, ReportArguments<8>>;
pub type WriteArgumentsB = g::Msg<49, ReportArguments<9>>;
pub type WriteResultA<'a> = g::Msg<50, &'a [u8]>;
pub type WriteResultB<'a> = g::Msg<51, &'a [u8]>;
pub type RevisionArgumentsA = g::Msg<52, ReportArguments<30>>;
pub type RevisionArgumentsB = g::Msg<53, ReportArguments<31>>;
pub type RevisionResultA<'a> = g::Msg<54, &'a [u8]>;
pub type RevisionResultB<'a> = g::Msg<55, &'a [u8]>;
pub type FeedbackA<'a> = g::Msg<56, &'a [u8]>;
pub type FeedbackB<'a> = g::Msg<57, &'a [u8]>;
pub type ReviewEvidenceA<'a> = g::Msg<60, &'a [u8]>;
pub type ReviewEvidenceB<'a> = g::Msg<61, &'a [u8]>;
pub type RequestReconsiderA = g::Msg<62, ReportArguments<18>>;
pub type RequestReconsiderB = g::Msg<63, ReportArguments<18>>;
pub type CombineInput = g::Msg<66, ReportArguments<16>>;
pub type CombinedReply<'a> = g::Msg<67, &'a [u8]>;
pub type RevisionAckA<'a> = g::Msg<68, &'a [u8]>;
pub type RevisionAckB<'a> = g::Msg<69, &'a [u8]>;

fn write_a(next: impl Projectable + Copy) -> impl Projectable + Copy {
    g::seq(
        g::send::<A_FILE, DOCUMENT_SERVICE, WriteA>(),
        g::seq(
            g::send::<DOCUMENT_SERVICE, WRITER_A, WriteArgumentsA>(),
            g::seq(
                g::send::<DOCUMENT, DOCUMENT_SERVICE, WriteOnDocumentA>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, DOCUMENT, Written>(),
                    g::par(g::send::<WRITER_A, A_FILE, WriteResultA>(), next),
                ),
            ),
        ),
    )
}
fn write_b(next: impl Projectable + Copy) -> impl Projectable + Copy {
    g::seq(
        g::send::<B_FILE, DOCUMENT_SERVICE, WriteB>(),
        g::seq(
            g::send::<DOCUMENT_SERVICE, WRITER_B, WriteArgumentsB>(),
            g::seq(
                g::send::<DOCUMENT, DOCUMENT_SERVICE, WriteOnDocumentB>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, DOCUMENT, Written>(),
                    g::par(g::send::<WRITER_B, B_FILE, WriteResultB>(), next),
                ),
            ),
        ),
    )
}
pub fn preparation() -> impl Projectable + Copy {
    let a_first = g::seq(
        g::send::<DOCUMENT_SERVICE, WRITER_A, InitialDocument>(),
        write_a(g::par(
            g::send::<DOCUMENT_SERVICE, EDITOR, MaterialA>(),
            g::seq(
                g::send::<WRITER_A, WRITER_B, InitialDocument>(),
                write_b(g::par(
                    g::send::<DOCUMENT_SERVICE, EDITOR, MaterialB>(),
                    g::send::<WRITER_B, EDITOR, InitialDocument>(),
                )),
            ),
        )),
    );
    let b_first = g::seq(
        g::send::<DOCUMENT_SERVICE, WRITER_B, InitialDocumentB>(),
        write_b(g::par(
            g::send::<DOCUMENT_SERVICE, EDITOR, MaterialB>(),
            g::seq(
                g::send::<WRITER_B, WRITER_A, InitialDocument>(),
                write_a(g::par(
                    g::send::<DOCUMENT_SERVICE, EDITOR, MaterialA>(),
                    g::send::<WRITER_A, EDITOR, InitialDocument>(),
                )),
            ),
        )),
    );
    g::par(
        g::seq(
            g::send::<ALLOCATION_FILE, DOCUMENT_SERVICE, Allocate>(),
            g::par(
                g::send::<DOCUMENT_SERVICE, ALLOCATION_FILE, AllocationAccepted>(),
                g::route(a_first, b_first),
            ),
        ),
        g::par(
            g::seq(
                g::send::<A_FILE, DOCUMENT_SERVICE, ReferencesA>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, A_FILE, ReferencedA>(),
                    g::send::<DOCUMENT_SERVICE, EDITOR, ReferenceMaterialA>(),
                ),
            ),
            g::seq(
                g::send::<B_FILE, DOCUMENT_SERVICE, ReferencesB>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, B_FILE, ReferencedB>(),
                    g::send::<DOCUMENT_SERVICE, EDITOR, ReferenceMaterialB>(),
                ),
            ),
        ),
    )
}
pub fn reviews() -> impl Projectable {
    g::par(
        g::seq(
            g::send::<ALLOCATION_FILE, DOCUMENT_SERVICE, ReviewA>(),
            g::par(
                g::send::<DOCUMENT_SERVICE, ALLOCATION_FILE, ReviewedA>(),
                g::send::<DOCUMENT_SERVICE, EDITOR, ReviewEvidenceA>(),
            ),
        ),
        g::seq(
            g::send::<ALLOCATION_FILE, DOCUMENT_SERVICE, ReviewB>(),
            g::par(
                g::send::<DOCUMENT_SERVICE, ALLOCATION_FILE, ReviewedB>(),
                g::send::<DOCUMENT_SERVICE, EDITOR, ReviewEvidenceB>(),
            ),
        ),
    )
}
fn revision_a() -> impl Projectable {
    g::seq(
        g::send::<ALLOCATION_FILE, EDITOR, RequestReconsiderA>(),
        g::seq(
            g::par(
                g::send::<EDITOR, DOCUMENT_SERVICE, ReconsiderA>(),
                g::par(
                    g::seq(
                        g::send::<EDITOR, WRITER_A, RevisionReviewA>(),
                        g::send::<WRITER_A, A_FILE, FeedbackA>(),
                    ),
                    g::seq(
                        g::send::<EDITOR, WRITER_B, RevisionReviewAForB>(),
                        g::send::<WRITER_B, B_FILE, FeedbackB>(),
                    ),
                ),
            ),
            g::seq(
                g::send::<EDITOR, WRITER_A, RevisionDocument>(),
                g::seq(
                    g::send::<A_FILE, DOCUMENT_SERVICE, ReviseA>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, WRITER_A, RevisionArgumentsA>(),
                        g::seq(
                            g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocumentA>(),
                            g::seq(
                                g::send::<DOCUMENT_SERVICE, DOCUMENT, Revised>(),
                                g::par(
                                    g::send::<WRITER_A, A_FILE, RevisionResultA>(),
                                    g::seq(
                                        g::par(
                                            g::send::<DOCUMENT_SERVICE, EDITOR, UpdatedDocument>(),
                                            g::send::<WRITER_A, EDITOR, RevisionDocument>(),
                                        ),
                                        g::send::<EDITOR, ALLOCATION_FILE, RevisionAckA>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn revision_b() -> impl Projectable {
    g::seq(
        g::send::<ALLOCATION_FILE, EDITOR, RequestReconsiderB>(),
        g::seq(
            g::par(
                g::send::<EDITOR, DOCUMENT_SERVICE, ReconsiderB>(),
                g::par(
                    g::seq(
                        g::send::<EDITOR, WRITER_B, RevisionReviewB>(),
                        g::send::<WRITER_B, B_FILE, FeedbackB>(),
                    ),
                    g::seq(
                        g::send::<EDITOR, WRITER_A, RevisionReviewBForA>(),
                        g::send::<WRITER_A, A_FILE, FeedbackA>(),
                    ),
                ),
            ),
            g::seq(
                g::send::<EDITOR, WRITER_B, RevisionDocument>(),
                g::seq(
                    g::send::<B_FILE, DOCUMENT_SERVICE, ReviseB>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, WRITER_B, RevisionArgumentsB>(),
                        g::seq(
                            g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocumentB>(),
                            g::seq(
                                g::send::<DOCUMENT_SERVICE, DOCUMENT, Revised>(),
                                g::par(
                                    g::send::<WRITER_B, B_FILE, RevisionResultB>(),
                                    g::seq(
                                        g::par(
                                            g::send::<DOCUMENT_SERVICE, EDITOR, UpdatedDocument>(),
                                            g::send::<WRITER_B, EDITOR, RevisionDocument>(),
                                        ),
                                        g::send::<EDITOR, ALLOCATION_FILE, RevisionAckB>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn approved() -> impl Projectable {
    g::seq(
        g::send::<ALLOCATION_FILE, EDITOR, CombineInput>(),
        g::seq(
            g::send::<EDITOR, WRITER_A, FrozenDocument>(),
            g::seq(
                g::send::<EDITOR, DOCUMENT_SERVICE, FinalApprovedA>(),
                g::seq(
                    g::send::<EDITOR, DOCUMENT_SERVICE, FinalApprovedB>(),
                    g::seq(
                        g::par(
                            g::send::<EDITOR, DOCUMENT_SERVICE, Combine>(),
                            g::send::<EDITOR, WRITER_A, CombineForward>(),
                        ),
                        g::seq(
                            g::send::<DOCUMENT, DOCUMENT_SERVICE, CombineOnDocument>(),
                            g::seq(
                                g::send::<DOCUMENT_SERVICE, DOCUMENT, Combined>(),
                                g::par(
                                    g::par(
                                        g::seq(
                                            g::send::<WRITER_A, EDITOR, CombinedForEditor>(),
                                            g::par(
                                                g::send::<EDITOR, ALLOCATION_FILE, CombinedReply>(),
                                                g::seq(
                                                    g::send::<EDITOR, WRITER_B, FinishedB>(),
                                                    g::send::<WRITER_B, B_FILE, FeedbackB>(),
                                                ),
                                            ),
                                        ),
                                        g::send::<WRITER_A, A_FILE, FeedbackA>(),
                                    ),
                                    g::send::<WRITER_A, DOCUMENT_SERVICE, DocumentReturn>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn business() -> impl Projectable {
    g::seq(
        preparation(),
        g::seq(
            reviews(),
            g::route(
                g::seq(
                    g::seq(g::route(revision_a(), revision_b()), reviews()).roll(),
                    approved(),
                ),
                approved(),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    let native = g::par(
        g::send::<DOCUMENT_SERVICE, A_VM, Loan<WRITER_A>>(),
        g::par(
            g::send::<DOCUMENT_SERVICE, B_VM, Loan<WRITER_B>>(),
            g::send::<DOCUMENT_SERVICE, EDITOR_VM, Loan<EDITOR>>(),
        ),
    );
    let grants = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::send::<ALLOCATION_DATA, EDITOR_VM, Loan<ALLOCATION_FILE>>(),
        ),
    );
    let files = g::par(
        g::seq(
            crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
            g::send::<A_FILE, A_DATA, FileExit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                g::send::<B_FILE, B_DATA, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<ALLOCATION_FILE, ALLOCATION_DATA>().roll(),
                g::send::<ALLOCATION_FILE, ALLOCATION_DATA, FileExit>(),
            ),
        ),
    );
    g::seq(
        native,
        g::seq(
            grants,
            g::par(
                business(),
                g::par(
                    files,
                    g::par(
                        g::seq(
                            crate::shell::global::private_io::<A_VM, A_IO>(),
                            g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::private_io::<B_VM, B_IO>(),
                                g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<EDITOR_VM, EDITOR_IO>(),
                                g::send::<EDITOR_VM, EDITOR_IO, crate::shell::global::Exit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub struct Programs {
    pub a: RoleProgram<WRITER_A>,
    pub b: RoleProgram<WRITER_B>,
    pub editor: RoleProgram<EDITOR>,
    pub document: RoleProgram<DOCUMENT>,
    pub service: RoleProgram<DOCUMENT_SERVICE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub editor_io: RoleProgram<EDITOR_IO>,
    pub allocation_file: RoleProgram<ALLOCATION_FILE>,
    pub allocation_data: RoleProgram<ALLOCATION_DATA>,
    pub a_file: RoleProgram<A_FILE>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_file: RoleProgram<B_FILE>,
    pub b_data: RoleProgram<B_DATA>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub editor_vm: RoleProgram<EDITOR_VM>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("InitialDocument", InitialDocument::LOGICAL_LABEL),
            ("InitialDocumentB", InitialDocumentB::LOGICAL_LABEL),
            ("RevisionDocument", RevisionDocument::LOGICAL_LABEL),
            ("WriteA", WriteA::LOGICAL_LABEL),
            ("WriteB", WriteB::LOGICAL_LABEL),
            ("WriteOnDocument", WRITE_ON_DOCUMENT),
            ("Written", Written::LOGICAL_LABEL),
            ("ReferencesA", ReferencesA::LOGICAL_LABEL),
            ("ReferencesB", ReferencesB::LOGICAL_LABEL),
            ("ReferencedA", ReferencedA::LOGICAL_LABEL),
            ("ReferencedB", ReferencedB::LOGICAL_LABEL),
            ("MaterialA", MaterialA::LOGICAL_LABEL),
            ("MaterialB", MaterialB::LOGICAL_LABEL),
            ("ReferenceMaterialA", ReferenceMaterialA::LOGICAL_LABEL),
            ("ReferenceMaterialB", ReferenceMaterialB::LOGICAL_LABEL),
            ("ReviewA", ReviewA::LOGICAL_LABEL),
            ("ReviewB", ReviewB::LOGICAL_LABEL),
            ("ReviewedA", ReviewedA::LOGICAL_LABEL),
            ("ReviewedB", ReviewedB::LOGICAL_LABEL),
            ("RevisionReviewA", RevisionReviewA::LOGICAL_LABEL),
            ("RevisionReviewB", RevisionReviewB::LOGICAL_LABEL),
            ("ReconsiderA", ReconsiderA::LOGICAL_LABEL),
            ("ReconsiderB", ReconsiderB::LOGICAL_LABEL),
            ("RevisionReviewAForB", RevisionReviewAForB::LOGICAL_LABEL),
            ("RevisionReviewBForA", RevisionReviewBForA::LOGICAL_LABEL),
            ("ReviseA", ReviseA::LOGICAL_LABEL),
            ("ReviseB", ReviseB::LOGICAL_LABEL),
            ("ReviseOnDocument", REVISE_ON_DOCUMENT),
            ("Revised", Revised::LOGICAL_LABEL),
            ("Combine", Combine::LOGICAL_LABEL),
            ("CombineOnDocument", CombineOnDocument::LOGICAL_LABEL),
            ("Combined", Combined::LOGICAL_LABEL),
            ("FinishedB", FinishedB::LOGICAL_LABEL),
            ("DocumentReturn", DocumentReturn::LOGICAL_LABEL),
            ("FinalApprovedA", FinalApprovedA::LOGICAL_LABEL),
            ("FinalApprovedB", FinalApprovedB::LOGICAL_LABEL),
            ("FrozenDocument", FrozenDocument::LOGICAL_LABEL),
            ("UpdatedDocument", UpdatedDocument::LOGICAL_LABEL),
            ("CombineForward", CombineForward::LOGICAL_LABEL),
            ("CombinedForEditor", CombinedForEditor::LOGICAL_LABEL),
            ("Allocate", Allocate::LOGICAL_LABEL),
            ("AllocationAccepted", AllocationAccepted::LOGICAL_LABEL),
            ("WriteArgumentsA", WriteArgumentsA::LOGICAL_LABEL),
            ("WriteArgumentsB", WriteArgumentsB::LOGICAL_LABEL),
            ("WriteResultA", WriteResultA::LOGICAL_LABEL),
            ("WriteResultB", WriteResultB::LOGICAL_LABEL),
            ("RevisionArgumentsA", RevisionArgumentsA::LOGICAL_LABEL),
            ("RevisionArgumentsB", RevisionArgumentsB::LOGICAL_LABEL),
            ("RevisionResultA", RevisionResultA::LOGICAL_LABEL),
            ("RevisionResultB", RevisionResultB::LOGICAL_LABEL),
            ("FeedbackA", FeedbackA::LOGICAL_LABEL),
            ("FeedbackB", FeedbackB::LOGICAL_LABEL),
            ("ReviewEvidenceA", ReviewEvidenceA::LOGICAL_LABEL),
            ("ReviewEvidenceB", ReviewEvidenceB::LOGICAL_LABEL),
            ("RequestReconsiderA", RequestReconsiderA::LOGICAL_LABEL),
            ("RequestReconsiderB", RequestReconsiderB::LOGICAL_LABEL),
            ("CombineInput", CombineInput::LOGICAL_LABEL),
            ("CombinedReply", CombinedReply::LOGICAL_LABEL),
            ("RevisionAckA", RevisionAckA::LOGICAL_LABEL),
            ("RevisionAckB", RevisionAckB::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"7E","session":SESSION,"labels":labels,
            "roles":{"WRITER_A":WRITER_A,"WRITER_B":WRITER_B,"EDITOR":EDITOR,"DOCUMENT":DOCUMENT,"DOCUMENT_SERVICE":DOCUMENT_SERVICE,
                "A_IO":A_IO,"B_IO":B_IO,"EDITOR_IO":EDITOR_IO,"ALLOCATION_FILE":ALLOCATION_FILE,"ALLOCATION_DATA":ALLOCATION_DATA,
                "A_FILE":A_FILE,"A_DATA":A_DATA,"B_FILE":B_FILE,"B_DATA":B_DATA,"A_VM":A_VM,"B_VM":B_VM,"EDITOR_VM":EDITOR_VM},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.editor),rows(&self.document),rows(&self.service),rows(&self.a_io),rows(&self.b_io),rows(&self.editor_io),
                rows(&self.allocation_file),rows(&self.allocation_data),rows(&self.a_file),rows(&self.a_data),rows(&self.b_file),rows(&self.b_data),rows(&self.a_vm),rows(&self.b_vm),rows(&self.editor_vm)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project(&source),
        b: project(&source),
        editor: project(&source),
        document: project(&source),
        service: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        editor_io: project(&source),
        allocation_file: project(&source),
        allocation_data: project(&source),
        a_file: project(&source),
        a_data: project(&source),
        b_file: project(&source),
        b_data: project(&source),
        a_vm: project(&source),
        b_vm: project(&source),
        editor_vm: project(&source),
    }
}
