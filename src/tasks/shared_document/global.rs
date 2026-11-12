//! One document authority, independent references/reviews and a frozen publisher.
use crate::{shell::imp::report_arguments::ReportArguments, shell::imp::review_reply::ReviewReply};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const WRITER_A: u8 = 0;
pub const WRITER_B: u8 = 1;
pub const EDITOR: u8 = 2;
pub const DOCUMENT: u8 = 3;
pub const DOCUMENT_SERVICE: u8 = 4;
pub const A_IO: u8 = 5;
pub const B_IO: u8 = 6;
pub const EDITOR_IO: u8 = 7;
pub const FIRST_WRITER: u16 = 890;
pub const SESSION: u32 = 73;
pub const CHAPTER_A: &str = "CHAPTER_1";
pub const CHAPTER_B: &str = "CHAPTER_2";

pub type InitialDocument<'r> = g::Msg<1, Endpoint<'r, DOCUMENT>>;
pub type RevisionDocument<'r> = g::Msg<3, Endpoint<'r, DOCUMENT>>;
pub type WriteA = g::Msg<4, ReportArguments<2>>;
pub type WriteB = g::Msg<5, ReportArguments<2>>;
pub type WriteOnDocument = g::Msg<6, ReportArguments<2>>;
pub type Written<'a> = g::Msg<7, &'a [u8]>;
pub type ReferencesA = g::Msg<8, ReportArguments<4>>;
pub type ReferencesB = g::Msg<9, ReportArguments<4>>;
pub type ReferencedA<'a> = g::Msg<10, &'a [u8]>;
pub type ReferencedB<'a> = g::Msg<11, &'a [u8]>;
pub type MaterialA<'a> = g::Msg<12, &'a [u8]>;
pub type MaterialB<'a> = g::Msg<13, &'a [u8]>;
pub type ReferenceMaterialA<'a> = g::Msg<14, &'a [u8]>;
pub type ReferenceMaterialB<'a> = g::Msg<15, &'a [u8]>;
pub type ReviewA = g::Msg<16, ReportArguments<5>>;
pub type ReviewB = g::Msg<17, ReportArguments<5>>;
pub type ReviewedA<'a> = g::Msg<18, &'a [u8]>;
pub type ReviewedB<'a> = g::Msg<19, &'a [u8]>;
pub type RejectedA<'a> = g::Msg<21, ReviewReply<'a, false>>;
pub type RejectedB<'a> = g::Msg<23, ReviewReply<'a, false>>;
pub type ReviseA = g::Msg<24, ReportArguments<3>>;
pub type ReviseB = g::Msg<25, ReportArguments<3>>;
pub type ReviseOnDocument = g::Msg<26, ReportArguments<3>>;
pub type Revised<'a> = g::Msg<27, &'a [u8]>;
pub type Combine = g::Msg<28, ReportArguments<6>>;
pub type CombineOnDocument = g::Msg<29, ReportArguments<6>>;
pub type Combined<'a> = g::Msg<30, &'a [u8]>;
pub type FinishedB<'a> = g::Msg<32, &'a [u8]>;
pub type DocumentReturn<'r> = g::Msg<33, Endpoint<'r, DOCUMENT>>;
pub type FinalApprovedA<'a> = g::Msg<34, ReviewReply<'a, true>>;
pub type FinalApprovedB<'a> = g::Msg<35, ReviewReply<'a, true>>;
pub type UpdatedDocument<'a> = g::Msg<36, &'a [u8]>;
pub type FrozenDocument<'r> = g::Msg<37, Endpoint<'r, DOCUMENT>>;
pub type CombineForward = g::Msg<38, ReportArguments<6>>;
pub type CombinedForEditor<'a> = g::Msg<39, &'a [u8]>;
pub type ReconsiderA<'a> = g::Msg<40, ReviewReply<'a, false>>;
pub type ReconsiderB<'a> = g::Msg<41, ReviewReply<'a, false>>;
pub type RejectedAForB<'a> = g::Msg<42, ReviewReply<'a, false>>;
pub type RejectedBForA<'a> = g::Msg<43, ReviewReply<'a, false>>;

fn write_a() -> impl Projectable {
    g::seq(
        g::send::<WRITER_A, DOCUMENT_SERVICE, WriteA>(),
        g::seq(
            g::send::<DOCUMENT, DOCUMENT_SERVICE, WriteOnDocument>(),
            g::send::<DOCUMENT_SERVICE, DOCUMENT, Written>(),
        ),
    )
}
fn write_b() -> impl Projectable {
    g::seq(
        g::send::<WRITER_B, DOCUMENT_SERVICE, WriteB>(),
        g::seq(
            g::send::<DOCUMENT, DOCUMENT_SERVICE, WriteOnDocument>(),
            g::send::<DOCUMENT_SERVICE, DOCUMENT, Written>(),
        ),
    )
}
pub fn preparation() -> impl Projectable {
    let a_first = g::seq(
        g::send::<DOCUMENT_SERVICE, WRITER_A, InitialDocument>(),
        g::seq(
            write_a(),
            g::par(
                g::send::<DOCUMENT_SERVICE, EDITOR, MaterialA>(),
                g::seq(
                    g::send::<WRITER_A, WRITER_B, InitialDocument>(),
                    g::seq(
                        write_b(),
                        g::par(
                            g::send::<DOCUMENT_SERVICE, EDITOR, MaterialB>(),
                            g::send::<WRITER_B, EDITOR, InitialDocument>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let b_first = g::seq(
        g::send::<DOCUMENT_SERVICE, WRITER_B, InitialDocument>(),
        g::seq(
            write_b(),
            g::par(
                g::send::<DOCUMENT_SERVICE, EDITOR, MaterialB>(),
                g::seq(
                    g::send::<WRITER_B, WRITER_A, InitialDocument>(),
                    g::seq(
                        write_a(),
                        g::par(
                            g::send::<DOCUMENT_SERVICE, EDITOR, MaterialA>(),
                            g::send::<WRITER_A, EDITOR, InitialDocument>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::par(
        g::route(a_first, b_first).resolve::<FIRST_WRITER>(),
        g::par(
            g::seq(
                g::send::<WRITER_A, DOCUMENT_SERVICE, ReferencesA>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, WRITER_A, ReferencedA>(),
                    g::send::<DOCUMENT_SERVICE, EDITOR, ReferenceMaterialA>(),
                ),
            ),
            g::seq(
                g::send::<WRITER_B, DOCUMENT_SERVICE, ReferencesB>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, WRITER_B, ReferencedB>(),
                    g::send::<DOCUMENT_SERVICE, EDITOR, ReferenceMaterialB>(),
                ),
            ),
        ),
    )
}
pub fn reviews() -> impl Projectable {
    g::par(
        g::seq(
            g::send::<EDITOR, DOCUMENT_SERVICE, ReviewA>(),
            g::send::<DOCUMENT_SERVICE, EDITOR, ReviewedA>(),
        ),
        g::seq(
            g::send::<EDITOR, DOCUMENT_SERVICE, ReviewB>(),
            g::send::<DOCUMENT_SERVICE, EDITOR, ReviewedB>(),
        ),
    )
}
fn revision_a() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<EDITOR, DOCUMENT_SERVICE, ReconsiderA>(),
            g::par(
                g::send::<EDITOR, WRITER_A, RejectedA>(),
                g::send::<EDITOR, WRITER_B, RejectedAForB>(),
            ),
        ),
        g::seq(
            g::send::<EDITOR, WRITER_A, RevisionDocument>(),
            g::seq(
                g::send::<WRITER_A, DOCUMENT_SERVICE, ReviseA>(),
                g::seq(
                    g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocument>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, DOCUMENT, Revised>(),
                        g::par(
                            g::send::<DOCUMENT_SERVICE, EDITOR, UpdatedDocument>(),
                            g::send::<WRITER_A, EDITOR, RevisionDocument>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn revision_b() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<EDITOR, DOCUMENT_SERVICE, ReconsiderB>(),
            g::par(
                g::send::<EDITOR, WRITER_B, RejectedB>(),
                g::send::<EDITOR, WRITER_A, RejectedBForA>(),
            ),
        ),
        g::seq(
            g::send::<EDITOR, WRITER_B, RevisionDocument>(),
            g::seq(
                g::send::<WRITER_B, DOCUMENT_SERVICE, ReviseB>(),
                g::seq(
                    g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocument>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, DOCUMENT, Revised>(),
                        g::par(
                            g::send::<DOCUMENT_SERVICE, EDITOR, UpdatedDocument>(),
                            g::send::<WRITER_B, EDITOR, RevisionDocument>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn approved() -> impl Projectable {
    // Returning the actual authority closes the old editing roll before the
    // final approval is announced. The editor cannot revive an old retry.
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
                                g::seq(
                                    g::send::<WRITER_A, EDITOR, CombinedForEditor>(),
                                    g::send::<EDITOR, WRITER_B, FinishedB>(),
                                ),
                                g::send::<WRITER_A, DOCUMENT_SERVICE, DocumentReturn>(),
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
    g::par(
        business(),
        g::par(
            crate::shell::global::private_io::<WRITER_A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<WRITER_B, B_IO>(),
                crate::shell::global::private_io::<EDITOR, EDITOR_IO>(),
            ),
        ),
    )
}

/// Immutable images of the sole business/WASI source, not task progress.
pub struct Programs {
    pub a: RoleProgram<WRITER_A>,
    pub b: RoleProgram<WRITER_B>,
    pub editor: RoleProgram<EDITOR>,
    pub document: RoleProgram<DOCUMENT>,
    pub service: RoleProgram<DOCUMENT_SERVICE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub editor_io: RoleProgram<EDITOR_IO>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("InitialDocument", InitialDocument::LOGICAL_LABEL),
            ("RevisionDocument", RevisionDocument::LOGICAL_LABEL),
            ("WriteA", WriteA::LOGICAL_LABEL),
            ("WriteB", WriteB::LOGICAL_LABEL),
            ("WriteOnDocument", WriteOnDocument::LOGICAL_LABEL),
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
            ("RejectedA", RejectedA::LOGICAL_LABEL),
            ("RejectedB", RejectedB::LOGICAL_LABEL),
            ("ReconsiderA", ReconsiderA::LOGICAL_LABEL),
            ("ReconsiderB", ReconsiderB::LOGICAL_LABEL),
            ("RejectedAForB", RejectedAForB::LOGICAL_LABEL),
            ("RejectedBForA", RejectedBForA::LOGICAL_LABEL),
            ("ReviseA", ReviseA::LOGICAL_LABEL),
            ("ReviseB", ReviseB::LOGICAL_LABEL),
            ("ReviseOnDocument", ReviseOnDocument::LOGICAL_LABEL),
            ("Revised", Revised::LOGICAL_LABEL),
            ("Combine", Combine::LOGICAL_LABEL),
            ("CombineOnDocument", CombineOnDocument::LOGICAL_LABEL),
            ("Combined", Combined::LOGICAL_LABEL),
            ("FinishedB", FinishedB::LOGICAL_LABEL),
            ("DocumentReturn", DocumentReturn::LOGICAL_LABEL),
            ("FinalApprovedA", FinalApprovedA::LOGICAL_LABEL),
            ("FinalApprovedB", FinalApprovedB::LOGICAL_LABEL),
            ("UpdatedDocument", UpdatedDocument::LOGICAL_LABEL),
            ("FrozenDocument", FrozenDocument::LOGICAL_LABEL),
            ("CombineForward", CombineForward::LOGICAL_LABEL),
            ("CombinedForEditor", CombinedForEditor::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"7E","session":SESSION,
            "labels":labels,
            "roles":{"WRITER_A":WRITER_A,"WRITER_B":WRITER_B,"EDITOR":EDITOR,
                "DOCUMENT":DOCUMENT,"DOCUMENT_SERVICE":DOCUMENT_SERVICE,
                "A_IO":A_IO,"B_IO":B_IO,"EDITOR_IO":EDITOR_IO},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.editor),rows(&self.document),
                rows(&self.service),rows(&self.a_io),rows(&self.b_io),rows(&self.editor_io)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project::<WRITER_A>(&source),
        b: project::<WRITER_B>(&source),
        editor: project::<EDITOR>(&source),
        document: project::<DOCUMENT>(&source),
        service: project::<DOCUMENT_SERVICE>(&source),
        a_io: project::<A_IO>(&source),
        b_io: project::<B_IO>(&source),
        editor_io: project::<EDITOR_IO>(&source),
    }
}
