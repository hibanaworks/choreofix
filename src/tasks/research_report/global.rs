//! Real report requests, owned resource Endpoints and result-driven revision.
use crate::shell::global::{FileExit, Loan};
use crate::shell::imp::report_arguments::ReportArguments;
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const A_WASI: u8 = 0;
pub const B_WASI: u8 = 1;
pub const REVIEW_WASI: u8 = 2;
pub const REVIEW_A: u8 = 5;
pub const REVIEW_B: u8 = 6;
pub const DOCUMENT_SERVICE: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const REVIEW_IO: u8 = 10;
pub const DOCUMENT: u8 = 13;
pub const DATABASE: u8 = 14;
pub const DATABASE_SERVICE: u8 = 15;
pub const RESEARCH_A: u8 = 16;
pub const RESEARCH_B: u8 = 17;
pub const EDITOR_FILE: u8 = 18;
pub const A_FILE: u8 = 19;
pub const B_FILE: u8 = 20;
pub const A_DATA: u8 = 21;
pub const B_DATA: u8 = 22;
pub const EDITOR_DATA: u8 = 23;
pub const A_VM: u8 = 24;
pub const B_VM: u8 = 25;
pub const EDITOR_VM: u8 = 26;
pub const WRITE_REQUESTS: u8 = 27;
pub const REFERENCE_REQUESTS: u8 = 28;
pub const INITIAL_DOCUMENT: u8 = 29;
pub const SECTION_A: &str = "section_a";
pub const SECTION_B: &str = "section_b";
pub const SESSION: u32 = 3;

const fn operation(
    path: &'static [u8],
    fd: u8,
) -> hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static> {
    use hibana_wasip1_runtime::{
        choreofs::{ChoreoFsObject, FdSpec, ObjectId},
        protocol,
    };
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
pub const AUTHOR_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 5] = [
    operation(b"service/research_topic", 4),
    operation(b"service/write_section", 5),
    operation(b"service/update_references", 17),
    operation(b"service/revise_section", 18),
    hibana_wasip1_runtime::choreofs::ChoreoFsObject::readable(
        b"service/feedback",
        hibana_wasip1_runtime::choreofs::ObjectId(19),
        hibana_wasip1_runtime::choreofs::FdSpec::new(19, 2),
        b"",
        hibana_wasip1_runtime::protocol::FdBinding::read(
            hibana_wasip1_runtime::protocol::FdReadRow::Base,
        ),
    ),
];
pub const EDITOR_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 3] = [
    operation(b"service/review_section_a", 4),
    operation(b"service/review_section_b", 5),
    operation(b"service/combine_sections", 17),
];

pub type WriteArgumentsA = g::Msg<48, ReportArguments<21>>;
pub type WriteArgumentsB = g::Msg<49, ReportArguments<22>>;
pub type RevisionArgumentsA = g::Msg<52, ReportArguments<23>>;
pub type RevisionArgumentsB = g::Msg<53, ReportArguments<24>>;
pub type ReferenceArgumentsA = g::Msg<54, ReportArguments<25>>;
pub type ReferenceArgumentsB = g::Msg<55, ReportArguments<26>>;
pub type WriteResultA<'a> = g::Msg<100, &'a [u8]>;
pub type WriteResultB<'a> = g::Msg<101, &'a [u8]>;
pub type ReferenceResultA<'a> = g::Msg<102, &'a [u8]>;
pub type ReferenceResultB<'a> = g::Msg<103, &'a [u8]>;
pub type RevisionResultA<'a> = g::Msg<104, &'a [u8]>;
pub type RevisionResultB<'a> = g::Msg<105, &'a [u8]>;
pub type ReferenceMaterialA<'a> = g::Msg<106, &'a [u8]>;
pub type ReferenceMaterialB<'a> = g::Msg<107, &'a [u8]>;
pub type ReportMaterial<'a> = g::Msg<226, &'a [u8]>;
pub type FeedbackA<'a> = g::Msg<108, &'a [u8]>;
pub type FeedbackB<'a> = g::Msg<109, &'a [u8]>;
pub type DocumentLoan<'r> = g::Msg<10, Endpoint<'r, DOCUMENT>>;
pub type DatabaseLoan<'r> = g::Msg<11, Endpoint<'r, DATABASE>>;
pub type ResearchA = g::Msg<1, ReportArguments<19, true>>;
pub type ResearchB = g::Msg<3, ReportArguments<20, true>>;
pub type ResearchResult<'a> = g::Msg<2, &'a [u8]>;
pub type WriteA = g::Msg<69, ReportArguments<21>>;
pub type ReviseA = g::Msg<76, ReportArguments<23>>;
pub type WriteB = g::Msg<169, ReportArguments<22>>;
pub type ReviseB = g::Msg<176, ReportArguments<24>>;
pub type ReferencesA = g::Msg<60, ReportArguments<25>>;
pub type ReferencesB = g::Msg<160, ReportArguments<26>>;
const WRITE_ON_DOCUMENT: u8 = 190;
pub type WriteOnDocumentA = g::Msg<WRITE_ON_DOCUMENT, ReportArguments<21, true>>;
pub type WriteOnDocumentB = g::Msg<WRITE_ON_DOCUMENT, ReportArguments<22, true>>;
const REVISE_ON_DOCUMENT: u8 = 191;
pub type ReviseOnDocumentA = g::Msg<REVISE_ON_DOCUMENT, ReportArguments<23, true>>;
pub type ReviseOnDocumentB = g::Msg<REVISE_ON_DOCUMENT, ReportArguments<24, true>>;
pub type CombineOnDocument = g::Msg<192, ReportArguments<29, true>>;
pub type DocumentResult<'a> = g::Msg<193, &'a [u8]>;
const REFERENCES_ON_DATABASE: u8 = 194;
pub type ReferencesOnDatabaseA = g::Msg<REFERENCES_ON_DATABASE, ReportArguments<25, true>>;
pub type ReferencesOnDatabaseB = g::Msg<REFERENCES_ON_DATABASE, ReportArguments<26, true>>;
pub type DatabaseResult<'a> = g::Msg<195, &'a [u8]>;
pub type Combine = g::Msg<70, ReportArguments<29>>;
const REVIEW_A_LABEL: u8 = 71;
pub type ReviewA = g::Msg<REVIEW_A_LABEL, ReportArguments<27>>;
pub type ReviewRequestA = g::Msg<REVIEW_A_LABEL, ReportArguments<27, true>>;
const REVIEW_B_LABEL: u8 = 171;
pub type ReviewB = g::Msg<REVIEW_B_LABEL, ReportArguments<28>>;
pub type ReviewRequestB = g::Msg<REVIEW_B_LABEL, ReportArguments<28, true>>;
pub type ReviewResultA<'a> = g::Msg<172, &'a [u8]>;
pub type ReviewResultB<'a> = g::Msg<173, &'a [u8]>;
pub type CombinedReply<'a> = g::Msg<174, &'a [u8]>;
pub type CombinedA<'a> = g::Msg<110, &'a [u8]>;
pub type CombinedB<'a> = g::Msg<111, &'a [u8]>;
pub type ApprovedA<'a> = g::Msg<30, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<31, ReviewReply<'a, false>>;
pub type ApprovedB<'a> = g::Msg<40, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<41, ReviewReply<'a, false>>;

pub type ConfirmApprovedA<'a> = g::Msg<224, ReviewReply<'a, true>>;
pub type ConfirmApprovedB<'a> = g::Msg<225, ReviewReply<'a, true>>;

fn review_input<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<EDITOR_FILE, REVIEW_WASI, M>(),
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, M>(),
    )
}

fn section_review_a() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, A_WASI, ReviewA>(),
        g::send::<A_WASI, RESEARCH_A, ReviewRequestA>(),
    )
}

fn section_review_b() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, B_WASI, ReviewB>(),
        g::send::<B_WASI, RESEARCH_B, ReviewRequestB>(),
    )
}

fn writing_a() -> impl Projectable + Copy {
    g::seq(
        g::send::<INITIAL_DOCUMENT, A_WASI, WriteArgumentsA>(),
        g::seq(
            g::send::<INITIAL_DOCUMENT, A_WASI, DocumentLoan>(),
            g::seq(
                g::send::<DOCUMENT, INITIAL_DOCUMENT, WriteOnDocumentA>(),
                g::seq(
                    g::send::<INITIAL_DOCUMENT, DOCUMENT, DocumentResult>(),
                    g::par(
                        g::send::<A_WASI, A_FILE, WriteResultA>(),
                        g::send::<A_WASI, INITIAL_DOCUMENT, DocumentLoan>(),
                    ),
                ),
            ),
        ),
    )
}

fn writing_b() -> impl Projectable + Copy {
    g::seq(
        g::send::<INITIAL_DOCUMENT, B_WASI, WriteArgumentsB>(),
        g::seq(
            g::send::<INITIAL_DOCUMENT, B_WASI, DocumentLoan>(),
            g::seq(
                g::send::<DOCUMENT, INITIAL_DOCUMENT, WriteOnDocumentB>(),
                g::seq(
                    g::send::<INITIAL_DOCUMENT, DOCUMENT, DocumentResult>(),
                    g::par(
                        g::send::<B_WASI, B_FILE, WriteResultB>(),
                        g::send::<B_WASI, INITIAL_DOCUMENT, DocumentLoan>(),
                    ),
                ),
            ),
        ),
    )
}
fn revise_a() -> impl Projectable {
    g::seq(
        g::send::<DOCUMENT_SERVICE, A_WASI, DocumentLoan>(),
        g::seq(
            g::send::<A_FILE, DOCUMENT_SERVICE, ReviseA>(),
            g::seq(
                g::send::<DOCUMENT_SERVICE, A_WASI, RevisionArgumentsA>(),
                g::seq(
                    g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocumentA>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, DOCUMENT, DocumentResult>(),
                        g::par(
                            g::send::<A_WASI, A_FILE, RevisionResultA>(),
                            g::send::<A_WASI, DOCUMENT_SERVICE, DocumentLoan>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn revise_b() -> impl Projectable {
    g::seq(
        g::send::<DOCUMENT_SERVICE, B_WASI, DocumentLoan>(),
        g::seq(
            g::send::<B_FILE, DOCUMENT_SERVICE, ReviseB>(),
            g::seq(
                g::send::<DOCUMENT_SERVICE, B_WASI, RevisionArgumentsB>(),
                g::seq(
                    g::send::<DOCUMENT, DOCUMENT_SERVICE, ReviseOnDocumentB>(),
                    g::seq(
                        g::send::<DOCUMENT_SERVICE, DOCUMENT, DocumentResult>(),
                        g::par(
                            g::send::<B_WASI, B_FILE, RevisionResultB>(),
                            g::send::<B_WASI, DOCUMENT_SERVICE, DocumentLoan>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn references_a() -> impl Projectable + Copy {
    g::seq(
        g::send::<DATABASE_SERVICE, A_WASI, ReferenceArgumentsA>(),
        g::seq(
            g::send::<DATABASE_SERVICE, A_WASI, DatabaseLoan>(),
            g::seq(
                g::send::<DATABASE, DATABASE_SERVICE, ReferencesOnDatabaseA>(),
                g::seq(
                    g::par(
                        g::send::<DATABASE_SERVICE, DATABASE, DatabaseResult>(),
                        g::send::<DATABASE_SERVICE, INITIAL_DOCUMENT, ReferenceMaterialA>(),
                    ),
                    g::par(
                        g::send::<A_WASI, A_FILE, ReferenceResultA>(),
                        g::send::<A_WASI, DATABASE_SERVICE, DatabaseLoan>(),
                    ),
                ),
            ),
        ),
    )
}
fn references_b() -> impl Projectable + Copy {
    g::seq(
        g::send::<DATABASE_SERVICE, B_WASI, ReferenceArgumentsB>(),
        g::seq(
            g::send::<DATABASE_SERVICE, B_WASI, DatabaseLoan>(),
            g::seq(
                g::send::<DATABASE, DATABASE_SERVICE, ReferencesOnDatabaseB>(),
                g::seq(
                    g::par(
                        g::send::<DATABASE_SERVICE, DATABASE, DatabaseResult>(),
                        g::send::<DATABASE_SERVICE, INITIAL_DOCUMENT, ReferenceMaterialB>(),
                    ),
                    g::par(
                        g::send::<B_WASI, B_FILE, ReferenceResultB>(),
                        g::send::<B_WASI, DATABASE_SERVICE, DatabaseLoan>(),
                    ),
                ),
            ),
        ),
    )
}
fn combine() -> impl Projectable {
    let approvals = g::seq(
        g::send::<REVIEW_WASI, RESEARCH_A, ConfirmApprovedA>(),
        g::seq(
            g::send::<RESEARCH_A, DOCUMENT_SERVICE, ConfirmApprovedA>(),
            g::seq(
                g::send::<REVIEW_WASI, RESEARCH_B, ConfirmApprovedB>(),
                g::send::<RESEARCH_B, DOCUMENT_SERVICE, ConfirmApprovedB>(),
            ),
        ),
    );
    let retired = g::par(
        g::send::<RESEARCH_A, REVIEW_WASI, Loan<REVIEW_A>>(),
        g::send::<RESEARCH_B, REVIEW_WASI, Loan<REVIEW_B>>(),
    );
    let publication = g::seq(
        g::send::<DOCUMENT_SERVICE, REVIEW_WASI, DocumentLoan>(),
        g::seq(
            g::send::<DOCUMENT, DOCUMENT_SERVICE, CombineOnDocument>(),
            g::seq(
                g::send::<DOCUMENT_SERVICE, DOCUMENT, DocumentResult>(),
                g::par(
                    g::send::<REVIEW_WASI, EDITOR_FILE, CombinedReply>(),
                    g::par(
                        g::send::<REVIEW_WASI, A_WASI, CombinedA>(),
                        g::send::<REVIEW_WASI, B_WASI, CombinedB>(),
                    ),
                ),
            ),
        ),
    );
    // Confirmation and retirement belong to this request's forward phase,
    // outside every elastic review roll.
    g::seq(
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Combine>(),
        g::seq(approvals, g::seq(retired, publication)),
    )
}
// Both observers consume the current result before the editor can select
// another review. Author feedback remains independent of the other observer.
fn outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, DOCUMENT_SERVICE, M>(),
        g::seq(
            g::send::<REVIEW_A, A_WASI, M>(),
            g::par(
                g::send::<A_WASI, A_FILE, FeedbackA>(),
                g::seq(
                    g::send::<A_WASI, REVIEW_A, M>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_WASI, M>(),
                        g::par(
                            g::send::<REVIEW_WASI, EDITOR_FILE, ReviewResultA>(),
                            g::seq(
                                g::send::<REVIEW_A, B_WASI, M>(),
                                g::send::<B_WASI, REVIEW_A, M>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, DOCUMENT_SERVICE, M>(),
        g::seq(
            g::send::<REVIEW_B, B_WASI, M>(),
            g::par(
                g::send::<B_WASI, B_FILE, FeedbackB>(),
                g::seq(
                    g::send::<B_WASI, REVIEW_B, M>(),
                    g::seq(
                        g::send::<REVIEW_B, REVIEW_WASI, M>(),
                        g::par(
                            g::send::<REVIEW_WASI, EDITOR_FILE, ReviewResultB>(),
                            g::seq(
                                g::send::<REVIEW_B, A_WASI, M>(),
                                g::send::<A_WASI, REVIEW_B, M>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

// The agent submits the actual publication request. Review services receive the
// actual positive receipts; their affine Endpoints move outside every roll.
fn finish() -> impl Projectable {
    g::send::<EDITOR_FILE, REVIEW_WASI, Combine>()
}

fn repeat_b_last() -> impl Projectable {
    review_b_until_approved(finish())
}
fn repeat_a_last() -> impl Projectable {
    review_a_until_approved(finish())
}

fn review_b_result(review: impl Projectable) -> impl Projectable {
    g::seq(
        review,
        g::route(
            g::seq(outcome_b::<ApprovedB>(), finish()),
            g::seq(outcome_b::<RejectedB>(), repeat_b_last()),
        ),
    )
}
fn review_a_result(review: impl Projectable) -> impl Projectable {
    g::seq(
        review,
        g::route(
            g::seq(outcome_a::<ApprovedA>(), finish()),
            g::seq(outcome_a::<RejectedA>(), repeat_a_last()),
        ),
    )
}

fn review_a_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        review_input::<ReviewA>(),
        g::seq(revise_a(), section_review_a()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                outcome_a::<ApprovedA>(),
                g::seq(
                    g::seq(
                        outcome_a::<RejectedA>(),
                        g::seq(
                            review_input::<ReviewA>(),
                            g::seq(revise_a(), section_review_a()),
                        ),
                    )
                    .roll(),
                    outcome_a::<ApprovedA>(),
                ),
            ),
            approved,
        ),
    )
}
fn review_b_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        review_input::<ReviewB>(),
        g::seq(revise_b(), section_review_b()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                outcome_b::<ApprovedB>(),
                g::seq(
                    g::seq(
                        outcome_b::<RejectedB>(),
                        g::seq(
                            review_input::<ReviewB>(),
                            g::seq(revise_b(), section_review_b()),
                        ),
                    )
                    .roll(),
                    outcome_b::<ApprovedB>(),
                ),
            ),
            approved,
        ),
    )
}

fn both_rejected_a_ready() -> impl Projectable {
    g::route(
        g::seq(
            review_input::<ReviewA>(),
            g::seq(
                g::par(revise_b(), section_review_a()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            review_input::<ReviewB>(),
                            review_b_result(section_review_b()),
                        ),
                    ),
                    g::seq(
                        outcome_a::<RejectedA>(),
                        review_a_until_approved(g::seq(
                            review_input::<ReviewB>(),
                            review_b_result(section_review_b()),
                        )),
                    ),
                ),
            ),
        ),
        g::seq(
            review_input::<ReviewB>(),
            g::seq(
                revise_b(),
                g::seq(
                    section_review_b(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                review_input::<ReviewA>(),
                                review_a_result(section_review_a()),
                            ),
                        ),
                        g::seq(
                            outcome_b::<RejectedB>(),
                            review_b_until_approved(g::seq(
                                review_input::<ReviewA>(),
                                review_a_result(section_review_a()),
                            )),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn both_rejected_b_ready() -> impl Projectable {
    g::route(
        g::seq(
            review_input::<ReviewA>(),
            g::seq(
                revise_a(),
                g::seq(
                    section_review_a(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                review_input::<ReviewB>(),
                                review_b_result(section_review_b()),
                            ),
                        ),
                        g::seq(
                            outcome_a::<RejectedA>(),
                            review_a_until_approved(g::seq(
                                review_input::<ReviewB>(),
                                review_b_result(section_review_b()),
                            )),
                        ),
                    ),
                ),
            ),
        ),
        g::seq(
            review_input::<ReviewB>(),
            g::seq(
                g::par(revise_a(), section_review_b()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            review_input::<ReviewA>(),
                            review_a_result(section_review_a()),
                        ),
                    ),
                    g::seq(
                        outcome_b::<RejectedB>(),
                        review_b_until_approved(g::seq(
                            review_input::<ReviewA>(),
                            review_a_result(section_review_a()),
                        )),
                    ),
                ),
            ),
        ),
    )
}

fn repeat_a_before_b() -> impl Projectable {
    g::route(
        g::seq(
            review_input::<ReviewA>(),
            g::seq(
                revise_a(),
                g::seq(
                    section_review_a(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                review_input::<ReviewB>(),
                                review_b_result(section_review_b()),
                            ),
                        ),
                        outcome_a::<RejectedA>(),
                    ),
                ),
            ),
        ),
        g::seq(
            review_input::<ReviewB>(),
            g::seq(
                g::par(revise_a(), section_review_b()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            review_input::<ReviewA>(),
                            review_a_result(section_review_a()),
                        ),
                    ),
                    g::seq(outcome_b::<RejectedB>(), both_rejected_a_ready()),
                ),
            ),
        ),
    )
    .roll()
}
fn repeat_b_before_a() -> impl Projectable {
    g::route(
        g::seq(
            review_input::<ReviewA>(),
            g::seq(
                g::par(revise_b(), section_review_a()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            review_input::<ReviewB>(),
                            review_b_result(section_review_b()),
                        ),
                    ),
                    g::seq(outcome_a::<RejectedA>(), both_rejected_b_ready()),
                ),
            ),
        ),
        g::seq(
            review_input::<ReviewB>(),
            g::seq(
                revise_b(),
                g::seq(
                    section_review_b(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                review_input::<ReviewA>(),
                                review_a_result(section_review_a()),
                            ),
                        ),
                        outcome_b::<RejectedB>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn transfer_initial_document<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<DOCUMENT_SERVICE, INITIAL_DOCUMENT, M>(),
        g::seq(
            g::send::<INITIAL_DOCUMENT, DOCUMENT_SERVICE, ReportMaterial>(),
            g::send::<INITIAL_DOCUMENT, DOCUMENT_SERVICE, DocumentLoan>(),
        ),
    )
}

fn first_a() -> impl Projectable {
    g::seq(
        review_input::<ReviewA>(),
        g::seq(
            section_review_a(),
            g::route(
                g::seq(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        transfer_initial_document::<ApprovedA>(),
                    ),
                    g::seq(
                        review_input::<ReviewB>(),
                        review_b_result(section_review_b()),
                    ),
                ),
                g::seq(
                    g::seq(
                        outcome_a::<RejectedA>(),
                        transfer_initial_document::<RejectedA>(),
                    ),
                    repeat_a_before_b(),
                ),
            ),
        ),
    )
}
fn first_b() -> impl Projectable {
    g::seq(
        review_input::<ReviewB>(),
        g::seq(
            section_review_b(),
            g::route(
                g::seq(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        transfer_initial_document::<ApprovedB>(),
                    ),
                    g::seq(
                        review_input::<ReviewA>(),
                        review_a_result(section_review_a()),
                    ),
                ),
                g::seq(
                    g::seq(
                        outcome_b::<RejectedB>(),
                        transfer_initial_document::<RejectedB>(),
                    ),
                    repeat_b_before_a(),
                ),
            ),
        ),
    )
}

pub struct Programs {
    pub initial_document: RoleProgram<INITIAL_DOCUMENT>,
    pub write_requests: RoleProgram<WRITE_REQUESTS>,
    pub reference_requests: RoleProgram<REFERENCE_REQUESTS>,
    pub a_wasi: RoleProgram<A_WASI>,
    pub b_wasi: RoleProgram<B_WASI>,
    pub review_wasi: RoleProgram<REVIEW_WASI>,
    pub a_file: RoleProgram<A_FILE>,
    pub b_file: RoleProgram<B_FILE>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_data: RoleProgram<B_DATA>,
    pub editor_data: RoleProgram<EDITOR_DATA>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub editor_vm: RoleProgram<EDITOR_VM>,
    pub editor_file: RoleProgram<EDITOR_FILE>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub document_service: RoleProgram<DOCUMENT_SERVICE>,
    pub document: RoleProgram<DOCUMENT>,
    pub database: RoleProgram<DATABASE>,
    pub database_service: RoleProgram<DATABASE_SERVICE>,
    pub research_a: RoleProgram<RESEARCH_A>,
    pub research_b: RoleProgram<RESEARCH_B>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
}

impl Programs {
    /// Static observations of the same programs passed to `enter`.
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels = serde_json::Map::from_iter(
            [
                ("ResearchA", ResearchA::LOGICAL_LABEL),
                ("ResearchB", ResearchB::LOGICAL_LABEL),
                ("ResearchResult", ResearchResult::LOGICAL_LABEL),
                ("WriteArgumentsA", WriteArgumentsA::LOGICAL_LABEL),
                ("WriteArgumentsB", WriteArgumentsB::LOGICAL_LABEL),
                ("RevisionArgumentsA", RevisionArgumentsA::LOGICAL_LABEL),
                ("RevisionArgumentsB", RevisionArgumentsB::LOGICAL_LABEL),
                ("ReferenceArgumentsA", ReferenceArgumentsA::LOGICAL_LABEL),
                ("ReferenceArgumentsB", ReferenceArgumentsB::LOGICAL_LABEL),
                ("WriteResultA", WriteResultA::LOGICAL_LABEL),
                ("WriteResultB", WriteResultB::LOGICAL_LABEL),
                ("ReferenceResultA", ReferenceResultA::LOGICAL_LABEL),
                ("ReferenceResultB", ReferenceResultB::LOGICAL_LABEL),
                ("RevisionResultA", RevisionResultA::LOGICAL_LABEL),
                ("RevisionResultB", RevisionResultB::LOGICAL_LABEL),
                ("ReferenceMaterialA", ReferenceMaterialA::LOGICAL_LABEL),
                ("ReferenceMaterialB", ReferenceMaterialB::LOGICAL_LABEL),
                ("ReportMaterial", ReportMaterial::LOGICAL_LABEL),
                ("FeedbackA", FeedbackA::LOGICAL_LABEL),
                ("FeedbackB", FeedbackB::LOGICAL_LABEL),
                ("CombinedA", CombinedA::LOGICAL_LABEL),
                ("CombinedB", CombinedB::LOGICAL_LABEL),
                ("DocumentLoan", DocumentLoan::LOGICAL_LABEL),
                ("DatabaseLoan", DatabaseLoan::LOGICAL_LABEL),
                ("WriteA", WriteA::LOGICAL_LABEL),
                ("WriteB", WriteB::LOGICAL_LABEL),
                ("ReviseA", ReviseA::LOGICAL_LABEL),
                ("ReviseB", ReviseB::LOGICAL_LABEL),
                ("ReferencesA", ReferencesA::LOGICAL_LABEL),
                ("ReferencesB", ReferencesB::LOGICAL_LABEL),
                ("WriteOnDocument", WRITE_ON_DOCUMENT),
                ("ReviseOnDocument", REVISE_ON_DOCUMENT),
                ("CombineOnDocument", CombineOnDocument::LOGICAL_LABEL),
                ("ReferencesOnDatabase", REFERENCES_ON_DATABASE),
                ("DocumentResult", DocumentResult::LOGICAL_LABEL),
                ("DatabaseResult", DatabaseResult::LOGICAL_LABEL),
                ("Combine", Combine::LOGICAL_LABEL),
                ("ReviewA", ReviewA::LOGICAL_LABEL),
                ("ReviewB", ReviewB::LOGICAL_LABEL),
                ("ReviewResultA", ReviewResultA::LOGICAL_LABEL),
                ("ReviewResultB", ReviewResultB::LOGICAL_LABEL),
                ("CombinedReply", CombinedReply::LOGICAL_LABEL),
                ("ApprovedA", ApprovedA::LOGICAL_LABEL),
                ("RejectedA", RejectedA::LOGICAL_LABEL),
                ("ApprovedB", ApprovedB::LOGICAL_LABEL),
                ("RejectedB", RejectedB::LOGICAL_LABEL),
                ("ConfirmApprovedA", ConfirmApprovedA::LOGICAL_LABEL),
                ("ConfirmApprovedB", ConfirmApprovedB::LOGICAL_LABEL),
            ]
            .into_iter()
            .map(|(name, label)| (name.to_owned(), serde_json::json!(label))),
        );
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"3E",
            "session":SESSION,
            "roles":{
                "RESEARCHER_A":A_WASI,"RESEARCHER_B":B_WASI,"EDITOR":REVIEW_WASI,
                "WRITE_REQUESTS":WRITE_REQUESTS,"REFERENCE_REQUESTS":REFERENCE_REQUESTS,
                "INITIAL_DOCUMENT":INITIAL_DOCUMENT,"DOCUMENT":DOCUMENT,"DOCUMENT_SERVICE":DOCUMENT_SERVICE,
                "DATABASE":DATABASE,"DATABASE_SERVICE":DATABASE_SERVICE,
                "RESEARCH_A":RESEARCH_A,"RESEARCH_B":RESEARCH_B,
                "REVIEW_A":REVIEW_A,"REVIEW_B":REVIEW_B,
                "A_IO":A_IO,"B_IO":B_IO,"EDITOR_IO":REVIEW_IO,"EDITOR_FILE":EDITOR_FILE,
                "A_FILE":A_FILE,"B_FILE":B_FILE,"A_DATA":A_DATA,"B_DATA":B_DATA,"EDITOR_DATA":EDITOR_DATA,"A_VM":A_VM,"B_VM":B_VM,"EDITOR_VM":EDITOR_VM
            },
            "labels":labels,
            "programs":[rows(&self.initial_document),rows(&self.write_requests),rows(&self.reference_requests),rows(&self.a_wasi), rows(&self.b_wasi), rows(&self.review_wasi),
            rows(&self.a_file), rows(&self.b_file), rows(&self.a_data), rows(&self.b_data), rows(&self.editor_data), rows(&self.a_vm), rows(&self.b_vm), rows(&self.editor_vm), rows(&self.editor_file), rows(&self.review_a), rows(&self.review_b), rows(&self.document_service), rows(&self.document), rows(&self.database),
            rows(&self.database_service), rows(&self.research_a), rows(&self.research_b),
            rows(&self.a_io), rows(&self.b_io), rows(&self.review_io)]})
    }
}

pub fn source() -> impl Projectable {
    let research = g::par(
        g::seq(
            g::send::<A_FILE, A_WASI, ResearchA>(),
            g::send::<A_WASI, A_FILE, ResearchResult>(),
        ),
        g::seq(
            g::send::<B_FILE, B_WASI, ResearchB>(),
            g::send::<B_WASI, B_FILE, ResearchResult>(),
        ),
    );
    let written_requests = g::par(
        g::send::<A_FILE, WRITE_REQUESTS, WriteA>(),
        g::send::<B_FILE, WRITE_REQUESTS, WriteB>(),
    );
    let reference_requests = g::par(
        g::send::<A_FILE, REFERENCE_REQUESTS, ReferencesA>(),
        g::send::<B_FILE, REFERENCE_REQUESTS, ReferencesB>(),
    );
    let writes = g::route(
        g::seq(writing_a(), writing_b()),
        g::seq(writing_b(), writing_a()),
    );
    let references = g::route(
        g::seq(references_a(), references_b()),
        g::seq(references_b(), references_a()),
    );
    let initial = g::seq(
        research,
        g::par(
            g::seq(written_requests, writes),
            g::seq(reference_requests, references),
        ),
    );
    let reviews = g::route(first_a(), first_b());
    let finish = combine();
    let task = g::seq(initial, g::seq(reviews, finish));
    let private = g::par(
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
                crate::shell::global::private_io::<EDITOR_VM, REVIEW_IO>(),
                g::send::<EDITOR_VM, REVIEW_IO, crate::shell::global::Exit>(),
            ),
        ),
    );
    let execution = g::seq(
        g::par(
            g::send::<DOCUMENT_SERVICE, A_VM, Loan<A_WASI>>(),
            g::par(
                g::send::<DOCUMENT_SERVICE, B_VM, Loan<B_WASI>>(),
                g::par(
                    g::send::<DOCUMENT_SERVICE, EDITOR_VM, Loan<REVIEW_WASI>>(),
                    g::par(
                        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
                        g::par(
                            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
                            g::send::<EDITOR_DATA, DOCUMENT_SERVICE, Loan<EDITOR_FILE>>(),
                        ),
                    ),
                ),
            ),
        ),
        g::par(
            task,
            g::par(
                g::par(
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
                            crate::shell::global::file_metadata::<EDITOR_FILE, EDITOR_DATA>()
                                .roll(),
                            g::send::<EDITOR_FILE, EDITOR_DATA, FileExit>(),
                        ),
                    ),
                ),
                private,
            ),
        ),
    );
    g::seq(
        g::seq(
            g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Loan<REVIEW_A>>(),
            g::seq(
                g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Loan<REVIEW_B>>(),
                g::seq(
                    g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Loan<RESEARCH_A>>(),
                    g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Loan<RESEARCH_B>>(),
                ),
            ),
        ),
        execution,
    )
}

pub fn programs() -> Programs {
    let protocol = source();
    Programs {
        initial_document: project(&protocol),
        write_requests: project(&protocol),
        reference_requests: project(&protocol),
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        review_wasi: project(&protocol),
        a_file: project(&protocol),
        b_file: project(&protocol),
        a_data: project(&protocol),
        b_data: project(&protocol),
        editor_data: project(&protocol),
        a_vm: project(&protocol),
        b_vm: project(&protocol),
        editor_vm: project(&protocol),
        editor_file: project(&protocol),
        review_a: project(&protocol),
        review_b: project(&protocol),
        document_service: project(&protocol),
        document: project(&protocol),
        database: project(&protocol),
        database_service: project(&protocol),
        research_a: project(&protocol),
        research_b: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        review_io: project(&protocol),
    }
}
