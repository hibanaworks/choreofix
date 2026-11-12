//! Real report requests, owned resource Endpoints and result-driven revision.
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
pub const SELECT_REVIEW: u16 = 810;
pub const DECIDE_A: u16 = 811;
pub const DECIDE_B: u16 = 812;
pub const SECTION_A: &str = "section_a";
pub const SECTION_B: &str = "section_b";
pub const SESSION: u32 = 3;

pub type DocumentLoan<'r> = g::Msg<10, Endpoint<'r, DOCUMENT>>;
pub type DatabaseLoan<'r> = g::Msg<11, Endpoint<'r, DATABASE>>;
pub type Research = g::Msg<1, ReportArguments<1>>;
pub type ResearchResult<'a> = g::Msg<2, &'a [u8]>;
pub type WriteA = g::Msg<69, ReportArguments<2>>;
pub type ReviseA = g::Msg<76, ReportArguments<3>>;
pub type WriteB = g::Msg<169, ReportArguments<2>>;
pub type ReviseB = g::Msg<176, ReportArguments<3>>;
pub type ReferencesA = g::Msg<60, ReportArguments<4>>;
pub type ReferencesB = g::Msg<160, ReportArguments<4>>;
pub type WriteOnDocument = g::Msg<190, ReportArguments<2>>;
pub type ReviseOnDocument = g::Msg<191, ReportArguments<3>>;
pub type CombineOnDocument = g::Msg<192, ReportArguments<6>>;
pub type DocumentResult<'a> = g::Msg<193, &'a [u8]>;
pub type ReferencesOnDatabase = g::Msg<194, ReportArguments<4>>;
pub type DatabaseResult<'a> = g::Msg<195, &'a [u8]>;
pub type Combine = g::Msg<70, ReportArguments<6>>;
pub type ReadyA<'a> = g::Msg<90, &'a [u8]>;
pub type ReadyB<'a> = g::Msg<91, &'a [u8]>;
pub type ReviewA = g::Msg<71, ReportArguments<5>>;
pub type ReReviewA = g::Msg<77, ReportArguments<5>>;
pub type ReviewB = g::Msg<171, ReportArguments<5>>;
pub type ReReviewB = g::Msg<177, ReportArguments<5>>;
pub type ApprovedA<'a> = g::Msg<30, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<31, ReviewReply<'a, false>>;
pub type ApprovedB<'a> = g::Msg<40, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<41, ReviewReply<'a, false>>;

// These messages carry the review's actual arguments. Distinct labels identify
// its projected occurrence; filesystem operations do not choose a task phase.
pub type FirstA = g::Msg<80, ReportArguments<5>>;
pub type FirstB = g::Msg<81, ReportArguments<5>>;
pub type AgainABeforeB = g::Msg<82, ReportArguments<5>>;
pub type BDuringARevision = g::Msg<83, ReportArguments<5>>;
pub type BOnly = g::Msg<84, ReportArguments<5>>;
pub type AgainAAfterB = g::Msg<85, ReportArguments<5>>;
pub type AgainBBeforeA = g::Msg<86, ReportArguments<5>>;
pub type ADuringBRevision = g::Msg<87, ReportArguments<5>>;
pub type AOnly = g::Msg<88, ReportArguments<5>>;
pub type AgainBAfterA = g::Msg<89, ReportArguments<5>>;
pub type AgainADuringBRevision = g::Msg<92, ReportArguments<5>>;
pub type AgainBDuringARevision = g::Msg<93, ReportArguments<5>>;
pub type AgainBBeforeAReview = g::Msg<94, ReportArguments<5>>;
pub type AgainABeforeBReview = g::Msg<95, ReportArguments<5>>;
pub type DocumentExit = g::Msg<234, i32>;
pub type DatabaseExit = g::Msg<235, i32>;
pub type ResearchExitA = g::Msg<236, i32>;
pub type ResearchExitB = g::Msg<237, i32>;
pub type ConfirmApprovedA<'a> = g::Msg<224, ReviewReply<'a, true>>;
pub type ConfirmApprovedB<'a> = g::Msg<225, ReviewReply<'a, true>>;

fn intent<const REVIEWER: u8, const AUTHOR: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, g::Msg<LABEL, ReportArguments<5>>>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEWER, g::Msg<LABEL, ReportArguments<5>>>(),
            g::send::<REVIEW_WASI, AUTHOR, g::Msg<LABEL, ReportArguments<5>>>(),
        ),
    )
}

fn document<const WHO: u8, const LABEL: u8, M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<WHO, DOCUMENT_SERVICE, g::Msg<LABEL, M::Payload>>(),
        g::seq(
            g::send::<DOCUMENT_SERVICE, WHO, DocumentLoan>(),
            g::seq(
                g::send::<DOCUMENT, DOCUMENT_SERVICE, M>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, DOCUMENT, DocumentResult>(),
                    g::send::<WHO, DOCUMENT_SERVICE, DocumentLoan>(),
                ),
            ),
        ),
    )
}
fn revise_a() -> impl Projectable {
    g::seq(
        document::<A_WASI, 76, ReviseOnDocument>(),
        g::send::<A_WASI, REVIEW_A, ReadyA>(),
    )
}
fn revise_b() -> impl Projectable {
    g::seq(
        document::<B_WASI, 176, ReviseOnDocument>(),
        g::send::<B_WASI, REVIEW_B, ReadyB>(),
    )
}
fn references_a() -> impl Projectable {
    g::seq(
        g::send::<A_WASI, DATABASE_SERVICE, ReferencesA>(),
        g::seq(
            g::send::<DATABASE_SERVICE, A_WASI, DatabaseLoan>(),
            g::seq(
                g::send::<DATABASE, DATABASE_SERVICE, ReferencesOnDatabase>(),
                g::seq(
                    g::send::<DATABASE_SERVICE, DATABASE, DatabaseResult>(),
                    g::seq(
                        g::send::<A_WASI, DATABASE_SERVICE, DatabaseLoan>(),
                        g::seq(
                            g::send::<A_WASI, REVIEW_A, ReadyA>(),
                            g::send::<A_WASI, REVIEW_B, ReadyA>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn references_b() -> impl Projectable {
    g::seq(
        g::send::<B_WASI, DATABASE_SERVICE, ReferencesB>(),
        g::seq(
            g::send::<DATABASE_SERVICE, B_WASI, DatabaseLoan>(),
            g::seq(
                g::send::<DATABASE, DATABASE_SERVICE, ReferencesOnDatabase>(),
                g::seq(
                    g::send::<DATABASE_SERVICE, DATABASE, DatabaseResult>(),
                    g::seq(
                        g::send::<B_WASI, DATABASE_SERVICE, DatabaseLoan>(),
                        g::seq(
                            g::send::<B_WASI, REVIEW_A, ReadyB>(),
                            g::send::<B_WASI, REVIEW_B, ReadyB>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn combine() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, Combine>(),
        g::seq(
            g::send::<DOCUMENT_SERVICE, REVIEW_WASI, DocumentLoan>(),
            g::seq(
                g::send::<DOCUMENT, DOCUMENT_SERVICE, CombineOnDocument>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, DOCUMENT, DocumentResult>(),
                    g::send::<REVIEW_WASI, DOCUMENT_SERVICE, DocumentLoan>(),
                ),
            ),
        ),
    )
}
fn outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, REVIEW_B, M>(),
        g::seq(
            g::send::<REVIEW_A, DOCUMENT_SERVICE, M>(),
            g::seq(
                g::send::<REVIEW_A, A_WASI, M>(),
                g::seq(
                    g::send::<REVIEW_A, B_WASI, M>(),
                    g::send::<REVIEW_A, REVIEW_WASI, M>(),
                ),
            ),
        ),
    )
}
fn final_outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, DOCUMENT_SERVICE, M>(),
        g::seq(
            g::send::<REVIEW_A, A_WASI, M>(),
            g::send::<REVIEW_A, REVIEW_WASI, M>(),
        ),
    )
}
fn outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REVIEW_A, M>(),
        g::seq(
            g::send::<REVIEW_B, DOCUMENT_SERVICE, M>(),
            g::seq(
                g::send::<REVIEW_B, A_WASI, M>(),
                g::seq(
                    g::send::<REVIEW_B, B_WASI, M>(),
                    g::send::<REVIEW_B, REVIEW_WASI, M>(),
                ),
            ),
        ),
    )
}
fn final_outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, DOCUMENT_SERVICE, M>(),
        g::seq(
            g::send::<REVIEW_B, B_WASI, M>(),
            g::send::<REVIEW_B, REVIEW_WASI, M>(),
        ),
    )
}
fn exits() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, DocumentExit>(),
        g::seq(
            g::send::<REVIEW_WASI, DATABASE_SERVICE, DatabaseExit>(),
            g::par(
                g::send::<REVIEW_WASI, RESEARCH_A, ResearchExitA>(),
                g::send::<REVIEW_WASI, RESEARCH_B, ResearchExitB>(),
            ),
        ),
    )
}

// Forward both actual positive replies before leaving the review workflow.
// Combining itself occurs once, outside every repeatable fragment.
fn finish() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, ConfirmApprovedA>(),
        g::send::<REVIEW_WASI, DOCUMENT_SERVICE, ConfirmApprovedB>(),
    )
}

fn repeat_b_last() -> impl Projectable {
    review_b_final_until_approved(finish())
}
fn repeat_a_last() -> impl Projectable {
    review_a_final_until_approved(finish())
}

fn review_b_result(review: impl Projectable) -> impl Projectable {
    g::seq(
        review,
        g::route(
            g::seq(final_outcome_b::<ApprovedB>(), finish()),
            g::seq(final_outcome_b::<RejectedB>(), repeat_b_last()),
        )
        .resolve::<DECIDE_B>(),
    )
}
fn review_a_result(review: impl Projectable) -> impl Projectable {
    g::seq(
        review,
        g::route(
            g::seq(final_outcome_a::<ApprovedA>(), finish()),
            g::seq(final_outcome_a::<RejectedA>(), repeat_a_last()),
        )
        .resolve::<DECIDE_A>(),
    )
}

fn review_a_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        intent::<REVIEW_A, A_WASI, 82>(),
        g::seq(revise_a(), g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
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
                            intent::<REVIEW_A, A_WASI, 82>(),
                            g::seq(revise_a(), g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    )
                    .roll(),
                    outcome_a::<ApprovedA>(),
                ),
            )
            .resolve::<DECIDE_A>(),
            approved,
        ),
    )
}
fn review_a_final_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        intent::<REVIEW_A, A_WASI, 82>(),
        g::seq(revise_a(), g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                final_outcome_a::<ApprovedA>(),
                g::seq(
                    g::seq(
                        final_outcome_a::<RejectedA>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 82>(),
                            g::seq(revise_a(), g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    )
                    .roll(),
                    final_outcome_a::<ApprovedA>(),
                ),
            )
            .resolve::<DECIDE_A>(),
            approved,
        ),
    )
}
fn review_b_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        intent::<REVIEW_B, B_WASI, 86>(),
        g::seq(revise_b(), g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
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
                            intent::<REVIEW_B, B_WASI, 86>(),
                            g::seq(revise_b(), g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    )
                    .roll(),
                    outcome_b::<ApprovedB>(),
                ),
            )
            .resolve::<DECIDE_B>(),
            approved,
        ),
    )
}

fn review_b_final_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        intent::<REVIEW_B, B_WASI, 86>(),
        g::seq(revise_b(), g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                final_outcome_b::<ApprovedB>(),
                g::seq(
                    g::seq(
                        final_outcome_b::<RejectedB>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 86>(),
                            g::seq(revise_b(), g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    )
                    .roll(),
                    final_outcome_b::<ApprovedB>(),
                ),
            )
            .resolve::<DECIDE_B>(),
            approved,
        ),
    )
}

fn both_rejected_a_ready() -> impl Projectable {
    g::route(
        g::seq(
            g::seq(
                intent::<REVIEW_A, A_WASI, 92>(),
                g::send::<REVIEW_WASI, B_WASI, AgainADuringBRevision>(),
            ),
            g::seq(
                g::par(revise_b(), g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    ),
                    g::seq(
                        outcome_a::<RejectedA>(),
                        review_a_until_approved(g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        )),
                    ),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
        g::seq(
            intent::<REVIEW_B, B_WASI, 94>(),
            g::seq(
                revise_b(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                intent::<REVIEW_A, A_WASI, 85>(),
                                review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                            ),
                        ),
                        g::seq(
                            outcome_b::<RejectedB>(),
                            review_b_until_approved(g::seq(
                                intent::<REVIEW_A, A_WASI, 85>(),
                                review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                            )),
                        ),
                    )
                    .resolve::<DECIDE_B>(),
                ),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
}
fn both_rejected_b_ready() -> impl Projectable {
    g::route(
        g::seq(
            intent::<REVIEW_A, A_WASI, 95>(),
            g::seq(
                revise_a(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                intent::<REVIEW_B, B_WASI, 89>(),
                                review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                            ),
                        ),
                        g::seq(
                            outcome_a::<RejectedA>(),
                            review_a_until_approved(g::seq(
                                intent::<REVIEW_B, B_WASI, 89>(),
                                review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                            )),
                        ),
                    )
                    .resolve::<DECIDE_A>(),
                ),
            ),
        ),
        g::seq(
            g::seq(
                intent::<REVIEW_B, B_WASI, 93>(),
                g::send::<REVIEW_WASI, A_WASI, AgainBDuringARevision>(),
            ),
            g::seq(
                g::par(revise_a(), g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    ),
                    g::seq(
                        outcome_b::<RejectedB>(),
                        review_b_until_approved(g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        )),
                    ),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
}

fn repeat_a_before_b() -> impl Projectable {
    g::route(
        g::seq(
            intent::<REVIEW_A, A_WASI, 82>(),
            g::seq(
                revise_a(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                intent::<REVIEW_B, B_WASI, 84>(),
                                review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReviewB>()),
                            ),
                        ),
                        outcome_a::<RejectedA>(),
                    )
                    .resolve::<DECIDE_A>(),
                ),
            ),
        ),
        g::seq(
            g::seq(
                intent::<REVIEW_B, B_WASI, 83>(),
                g::send::<REVIEW_WASI, A_WASI, BDuringARevision>(),
            ),
            g::seq(
                g::par(revise_a(), g::send::<REVIEW_WASI, REVIEW_B, ReviewB>()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    ),
                    g::seq(outcome_b::<RejectedB>(), both_rejected_a_ready()),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
    .roll()
}
fn repeat_b_before_a() -> impl Projectable {
    g::route(
        g::seq(
            g::seq(
                intent::<REVIEW_A, A_WASI, 87>(),
                g::send::<REVIEW_WASI, B_WASI, ADuringBRevision>(),
            ),
            g::seq(
                g::par(revise_b(), g::send::<REVIEW_WASI, REVIEW_A, ReviewA>()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    ),
                    g::seq(outcome_a::<RejectedA>(), both_rejected_b_ready()),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
        g::seq(
            intent::<REVIEW_B, B_WASI, 86>(),
            g::seq(
                revise_b(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                intent::<REVIEW_A, A_WASI, 88>(),
                                review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReviewA>()),
                            ),
                        ),
                        outcome_b::<RejectedB>(),
                    )
                    .resolve::<DECIDE_B>(),
                ),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
    .roll()
}
fn first_a() -> impl Projectable {
    g::seq(
        intent::<REVIEW_A, A_WASI, 80>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEW_A, ReviewA>(),
            g::route(
                g::seq(
                    outcome_a::<ApprovedA>(),
                    g::seq(
                        intent::<REVIEW_B, B_WASI, 84>(),
                        review_b_result(g::send::<REVIEW_WASI, REVIEW_B, ReviewB>()),
                    ),
                ),
                g::seq(outcome_a::<RejectedA>(), repeat_a_before_b()),
            )
            .resolve::<DECIDE_A>(),
        ),
    )
}
fn first_b() -> impl Projectable {
    g::seq(
        intent::<REVIEW_B, B_WASI, 81>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEW_B, ReviewB>(),
            g::route(
                g::seq(
                    outcome_b::<ApprovedB>(),
                    g::seq(
                        intent::<REVIEW_A, A_WASI, 88>(),
                        review_a_result(g::send::<REVIEW_WASI, REVIEW_A, ReviewA>()),
                    ),
                ),
                g::seq(outcome_b::<RejectedB>(), repeat_b_before_a()),
            )
            .resolve::<DECIDE_B>(),
        ),
    )
}

pub struct Programs {
    pub a_wasi: RoleProgram<A_WASI>,
    pub b_wasi: RoleProgram<B_WASI>,
    pub review_wasi: RoleProgram<REVIEW_WASI>,
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
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"3E",
            "session":SESSION,
            "roles":{
                "RESEARCHER_A":A_WASI,"RESEARCHER_B":B_WASI,"EDITOR":REVIEW_WASI,
                "DOCUMENT":DOCUMENT,"DOCUMENT_SERVICE":DOCUMENT_SERVICE,
                "DATABASE":DATABASE,"DATABASE_SERVICE":DATABASE_SERVICE,
                "RESEARCH_A":RESEARCH_A,"RESEARCH_B":RESEARCH_B,
                "REVIEW_A":REVIEW_A,"REVIEW_B":REVIEW_B,
                "A_IO":A_IO,"B_IO":B_IO,"EDITOR_IO":REVIEW_IO
            },
            "labels":{
                "Research":Research::LOGICAL_LABEL,"ResearchResult":ResearchResult::LOGICAL_LABEL,
                "ReadyA":ReadyA::LOGICAL_LABEL,"ReadyB":ReadyB::LOGICAL_LABEL,
                "WriteA":WriteA::LOGICAL_LABEL,"WriteB":WriteB::LOGICAL_LABEL,
                "ReviseA":ReviseA::LOGICAL_LABEL,"ReviseB":ReviseB::LOGICAL_LABEL,
                "ReferencesA":ReferencesA::LOGICAL_LABEL,"ReferencesB":ReferencesB::LOGICAL_LABEL,
                "WriteOnDocument":WriteOnDocument::LOGICAL_LABEL,
                "ReviseOnDocument":ReviseOnDocument::LOGICAL_LABEL,
                "CombineOnDocument":CombineOnDocument::LOGICAL_LABEL,
                "ReferencesOnDatabase":ReferencesOnDatabase::LOGICAL_LABEL,
                "DocumentResult":DocumentResult::LOGICAL_LABEL,"DatabaseResult":DatabaseResult::LOGICAL_LABEL,
                "Combine":Combine::LOGICAL_LABEL,
                "ReviewA":ReviewA::LOGICAL_LABEL,"ReReviewA":ReReviewA::LOGICAL_LABEL,
                "ReviewB":ReviewB::LOGICAL_LABEL,"ReReviewB":ReReviewB::LOGICAL_LABEL,
                "ApprovedA":ApprovedA::LOGICAL_LABEL,"RejectedA":RejectedA::LOGICAL_LABEL,
                "ApprovedB":ApprovedB::LOGICAL_LABEL,"RejectedB":RejectedB::LOGICAL_LABEL,
                "ConfirmApprovedA":ConfirmApprovedA::LOGICAL_LABEL,"ConfirmApprovedB":ConfirmApprovedB::LOGICAL_LABEL
            },
            "review_intents":[FirstA::LOGICAL_LABEL,FirstB::LOGICAL_LABEL,AgainABeforeB::LOGICAL_LABEL,
                BDuringARevision::LOGICAL_LABEL,BOnly::LOGICAL_LABEL,AgainAAfterB::LOGICAL_LABEL,
                AgainBBeforeA::LOGICAL_LABEL,ADuringBRevision::LOGICAL_LABEL,AOnly::LOGICAL_LABEL,
                AgainBAfterA::LOGICAL_LABEL,AgainADuringBRevision::LOGICAL_LABEL,
                AgainBDuringARevision::LOGICAL_LABEL,AgainBBeforeAReview::LOGICAL_LABEL,AgainABeforeBReview::LOGICAL_LABEL],
            "programs":[rows(&self.a_wasi), rows(&self.b_wasi), rows(&self.review_wasi),
            rows(&self.review_a), rows(&self.review_b), rows(&self.document_service), rows(&self.document), rows(&self.database),
            rows(&self.database_service), rows(&self.research_a), rows(&self.research_b),
            rows(&self.a_io), rows(&self.b_io), rows(&self.review_io)]})
    }
}

pub fn programs() -> Programs {
    let research = g::par(
        g::seq(
            g::send::<A_WASI, RESEARCH_A, Research>(),
            g::send::<RESEARCH_A, A_WASI, ResearchResult>(),
        ),
        g::seq(
            g::send::<B_WASI, RESEARCH_B, Research>(),
            g::send::<RESEARCH_B, B_WASI, ResearchResult>(),
        ),
    );
    let initial = g::seq(
        research,
        g::seq(
            document::<A_WASI, 69, WriteOnDocument>(),
            g::seq(
                g::par(references_a(), document::<B_WASI, 169, WriteOnDocument>()),
                references_b(),
            ),
        ),
    );
    let reviews = g::route(first_a(), first_b()).resolve::<SELECT_REVIEW>();
    let finish = g::seq(combine(), exits());
    let task = g::seq(initial, g::seq(reviews, finish));
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            crate::shell::global::private_io::<REVIEW_WASI, REVIEW_IO>(),
        ),
    );
    let protocol = g::par(task, private);
    Programs {
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        review_wasi: project(&protocol),
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
