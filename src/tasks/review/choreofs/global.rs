//! Repository operations require the same delegated Endpoint throughout each
//! actual request, result and return. Review results choose the next work.
use crate::shell::imp::repository_arguments::RepositoryArguments;
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
pub const REPO: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const REVIEW_IO: u8 = 10;
pub const REVIEW: u8 = 11;
pub const REPO_ENDPOINT: u8 = 13;
pub const SELECT_REVIEW: u16 = 260;
pub const DECIDE_A: u16 = 264;
pub const DECIDE_B: u16 = 265;
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "notification_feature";
pub const MERGE_BRANCH_A: &str = "dev_a_feature";
pub const MERGE_BRANCH_B: &str = "dev_b_feature";

pub type RepoLoan<'r> = g::Msg<10, Endpoint<'r, REPO_ENDPOINT>>;
pub type ReviewControl<'r> = g::Msg<11, Endpoint<'r, REVIEW>>;
pub type ReviewsComplete<'r> = g::Msg<235, Endpoint<'r, REVIEW>>;
pub type ImplementA = g::Msg<69, RepositoryArguments<1>>;
pub type ReviseA = g::Msg<76, RepositoryArguments<1>>;
pub type ImplementB = g::Msg<169, RepositoryArguments<1>>;
pub type ReviseB = g::Msg<176, RepositoryArguments<1>>;
pub type MergeA = g::Msg<70, RepositoryArguments<2>>;
pub type MergeB = g::Msg<170, RepositoryArguments<2>>;
pub type ImplementOnRepo = g::Msg<190, RepositoryArguments<1>>;
pub type MergeOnRepo = g::Msg<191, RepositoryArguments<2>>;
pub type RepoResult<'a> = g::Msg<192, &'a [u8]>;
pub type ReadyA<'a> = g::Msg<90, &'a [u8]>;
pub type ReadyB<'a> = g::Msg<91, &'a [u8]>;
pub type ReviewA = g::Msg<71, RepositoryArguments<3>>;
pub type ReReviewA = g::Msg<77, RepositoryArguments<3>>;
pub type ReviewB = g::Msg<171, RepositoryArguments<3>>;
pub type ReReviewB = g::Msg<177, RepositoryArguments<3>>;
pub type ApprovedA<'a> = g::Msg<30, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<31, ReviewReply<'a, false>>;
pub type ApprovedB<'a> = g::Msg<40, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<41, ReviewReply<'a, false>>;

// These messages carry the review's actual arguments. Distinct labels identify
// its projected occurrence; filesystem operations do not choose a task phase.
pub type FirstA = g::Msg<80, RepositoryArguments<3>>;
pub type FirstB = g::Msg<81, RepositoryArguments<3>>;
pub type AgainABeforeB = g::Msg<82, RepositoryArguments<3>>;
pub type BDuringARevision = g::Msg<83, RepositoryArguments<3>>;
pub type BAfterAMerge = g::Msg<84, RepositoryArguments<3>>;
pub type AgainAAfterB = g::Msg<85, RepositoryArguments<3>>;
pub type AgainBBeforeA = g::Msg<86, RepositoryArguments<3>>;
pub type ADuringBRevision = g::Msg<87, RepositoryArguments<3>>;
pub type AAfterBMerge = g::Msg<88, RepositoryArguments<3>>;
pub type AgainBAfterA = g::Msg<89, RepositoryArguments<3>>;
pub type AgainADuringBRevision = g::Msg<92, RepositoryArguments<3>>;
pub type AgainBDuringARevision = g::Msg<93, RepositoryArguments<3>>;
pub type AgainBBeforeAReview = g::Msg<94, RepositoryArguments<3>>;
pub type AgainABeforeBReview = g::Msg<95, RepositoryArguments<3>>;
pub type RepoExit = g::Msg<234, i32>;

fn intent<const REVIEWER: u8, const AUTHOR: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW, REPO, g::Msg<LABEL, RepositoryArguments<3>>>(),
        g::seq(
            g::send::<REVIEW, REVIEWER, g::Msg<LABEL, RepositoryArguments<3>>>(),
            g::send::<REVIEW, AUTHOR, g::Msg<LABEL, RepositoryArguments<3>>>(),
        ),
    )
}

fn implement_a<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<A_WASI, REPO, g::Msg<LABEL, RepositoryArguments<1>>>(),
        g::seq(
            g::send::<REPO, A_WASI, RepoLoan>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, ImplementOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult>(),
                    g::seq(
                        g::send::<A_WASI, REPO, RepoLoan>(),
                        g::send::<A_WASI, REVIEW_WASI, ReadyA>(),
                    ),
                ),
            ),
        ),
    )
}
fn implement_b<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<B_WASI, REPO, g::Msg<LABEL, RepositoryArguments<1>>>(),
        g::seq(
            g::send::<REPO, B_WASI, RepoLoan>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, ImplementOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult>(),
                    g::seq(
                        g::send::<B_WASI, REPO, RepoLoan>(),
                        g::send::<B_WASI, REVIEW_WASI, ReadyB>(),
                    ),
                ),
            ),
        ),
    )
}
fn merge_a() -> impl Projectable {
    g::seq(
        g::send::<A_WASI, REPO, MergeA>(),
        g::seq(
            g::send::<REPO, A_WASI, RepoLoan>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, MergeOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult>(),
                    g::send::<A_WASI, REPO, RepoLoan>(),
                ),
            ),
        ),
    )
}
fn merge_b() -> impl Projectable {
    g::seq(
        g::send::<B_WASI, REPO, MergeB>(),
        g::seq(
            g::send::<REPO, B_WASI, RepoLoan>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, MergeOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult>(),
                    g::send::<B_WASI, REPO, RepoLoan>(),
                ),
            ),
        ),
    )
}
fn request_a<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, REVIEW_A, ReviewControl>(),
        g::send::<REVIEW_WASI, REVIEW_A, g::Msg<LABEL, RepositoryArguments<3>>>(),
    )
}
fn request_b<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, REVIEW_B, ReviewControl>(),
        g::send::<REVIEW_WASI, REVIEW_B, g::Msg<LABEL, RepositoryArguments<3>>>(),
    )
}
fn outcome_a<M: g::Message>() -> impl Projectable {
    let result = g::seq(
        g::send::<REVIEW_A, A_WASI, M>(),
        g::seq(
            g::send::<REVIEW_A, B_WASI, M>(),
            g::seq(
                g::send::<REVIEW_A, REVIEW_WASI, M>(),
                g::seq(
                    g::send::<REVIEW_A, REVIEW_B, M>(),
                    g::send::<REVIEW_A, REPO, M>(),
                ),
            ),
        ),
    );
    g::seq(result, g::send::<REVIEW_A, REVIEW_WASI, ReviewControl>())
}
fn outcome_b<M: g::Message>() -> impl Projectable {
    let result = g::seq(
        g::send::<REVIEW_B, A_WASI, M>(),
        g::seq(
            g::send::<REVIEW_B, B_WASI, M>(),
            g::seq(
                g::send::<REVIEW_B, REVIEW_WASI, M>(),
                g::seq(
                    g::send::<REVIEW_B, REVIEW_A, M>(),
                    g::send::<REVIEW_B, REPO, M>(),
                ),
            ),
        ),
    );
    g::seq(result, g::send::<REVIEW_B, REVIEW_WASI, ReviewControl>())
}
fn finish() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, A_WASI, ReviewControl>(),
        g::seq(
            g::send::<A_WASI, B_WASI, ReviewControl>(),
            g::seq(
                g::send::<B_WASI, REPO, ReviewControl>(),
                g::send::<REVIEW_WASI, REPO, RepoExit>(),
            ),
        ),
    )
}

fn final_outcome_a<M: g::Message, Loan: g::Message>() -> impl Projectable {
    let result = g::seq(
        g::send::<REVIEW_A, A_WASI, M>(),
        g::seq(
            g::send::<REVIEW_A, REVIEW_WASI, M>(),
            g::send::<REVIEW_A, REPO, M>(),
        ),
    );
    g::seq(result, g::send::<REVIEW_A, REVIEW_WASI, Loan>())
}
fn final_outcome_b<M: g::Message, Loan: g::Message>() -> impl Projectable {
    let result = g::seq(
        g::send::<REVIEW_B, B_WASI, M>(),
        g::seq(
            g::send::<REVIEW_B, REVIEW_WASI, M>(),
            g::send::<REVIEW_B, REPO, M>(),
        ),
    );
    g::seq(result, g::send::<REVIEW_B, REVIEW_WASI, Loan>())
}

fn repeat_b_last() -> impl Projectable {
    review_b_final_until_approved(merge_b())
}
fn repeat_a_last() -> impl Projectable {
    review_a_final_until_approved(merge_a())
}

fn merge_a_and_review_b(review: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(merge_a(), review),
        g::route(
            g::seq(final_outcome_b::<ApprovedB, ReviewsComplete>(), merge_b()),
            g::seq(
                final_outcome_b::<RejectedB, ReviewControl>(),
                repeat_b_last(),
            ),
        )
        .resolve::<DECIDE_B>(),
    )
}
fn merge_b_and_review_a(review: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(merge_b(), review),
        g::route(
            g::seq(final_outcome_a::<ApprovedA, ReviewsComplete>(), merge_a()),
            g::seq(
                final_outcome_a::<RejectedA, ReviewControl>(),
                repeat_a_last(),
            ),
        )
        .resolve::<DECIDE_A>(),
    )
}

fn review_a_until_approved(approved: impl Projectable) -> impl Projectable {
    let next = g::seq(
        intent::<REVIEW_A, A_WASI, 82>(),
        g::seq(implement_a::<76>(), request_a::<77>()),
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
                            g::seq(implement_a::<76>(), request_a::<77>()),
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
        g::seq(implement_a::<76>(), request_a::<77>()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                final_outcome_a::<ApprovedA, ReviewsComplete>(),
                g::seq(
                    g::seq(
                        final_outcome_a::<RejectedA, ReviewControl>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 82>(),
                            g::seq(implement_a::<76>(), request_a::<77>()),
                        ),
                    )
                    .roll(),
                    final_outcome_a::<ApprovedA, ReviewsComplete>(),
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
        g::seq(implement_b::<176>(), request_b::<177>()),
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
                            g::seq(implement_b::<176>(), request_b::<177>()),
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
        g::seq(implement_b::<176>(), request_b::<177>()),
    );
    g::seq(
        next,
        g::seq(
            g::route(
                final_outcome_b::<ApprovedB, ReviewsComplete>(),
                g::seq(
                    g::seq(
                        final_outcome_b::<RejectedB, ReviewControl>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 86>(),
                            g::seq(implement_b::<176>(), request_b::<177>()),
                        ),
                    )
                    .roll(),
                    final_outcome_b::<ApprovedB, ReviewsComplete>(),
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
                g::send::<REVIEW, B_WASI, AgainADuringBRevision>(),
            ),
            g::seq(
                g::par(implement_b::<176>(), request_a::<77>()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            merge_a_and_review_b(request_b::<177>()),
                        ),
                    ),
                    g::seq(
                        outcome_a::<RejectedA>(),
                        review_a_until_approved(g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            merge_a_and_review_b(request_b::<177>()),
                        )),
                    ),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
        g::seq(
            intent::<REVIEW_B, B_WASI, 94>(),
            g::seq(
                implement_b::<176>(),
                g::seq(
                    request_b::<177>(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                intent::<REVIEW_A, A_WASI, 85>(),
                                merge_b_and_review_a(request_a::<77>()),
                            ),
                        ),
                        g::seq(
                            outcome_b::<RejectedB>(),
                            review_b_until_approved(g::seq(
                                intent::<REVIEW_A, A_WASI, 85>(),
                                merge_b_and_review_a(request_a::<77>()),
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
                implement_a::<76>(),
                g::seq(
                    request_a::<77>(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                intent::<REVIEW_B, B_WASI, 89>(),
                                merge_a_and_review_b(request_b::<177>()),
                            ),
                        ),
                        g::seq(
                            outcome_a::<RejectedA>(),
                            review_a_until_approved(g::seq(
                                intent::<REVIEW_B, B_WASI, 89>(),
                                merge_a_and_review_b(request_b::<177>()),
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
                g::send::<REVIEW, A_WASI, AgainBDuringARevision>(),
            ),
            g::seq(
                g::par(implement_a::<76>(), request_b::<177>()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            merge_b_and_review_a(request_a::<77>()),
                        ),
                    ),
                    g::seq(
                        outcome_b::<RejectedB>(),
                        review_b_until_approved(g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            merge_b_and_review_a(request_a::<77>()),
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
                implement_a::<76>(),
                g::seq(
                    request_a::<77>(),
                    g::route(
                        g::seq(
                            outcome_a::<ApprovedA>(),
                            g::seq(
                                intent::<REVIEW_B, B_WASI, 84>(),
                                merge_a_and_review_b(request_b::<171>()),
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
                g::send::<REVIEW, A_WASI, BDuringARevision>(),
            ),
            g::seq(
                g::par(implement_a::<76>(), request_b::<171>()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 85>(),
                            merge_b_and_review_a(request_a::<77>()),
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
                g::send::<REVIEW, B_WASI, ADuringBRevision>(),
            ),
            g::seq(
                g::par(implement_b::<176>(), request_a::<71>()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 89>(),
                            merge_a_and_review_b(request_b::<177>()),
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
                implement_b::<176>(),
                g::seq(
                    request_b::<177>(),
                    g::route(
                        g::seq(
                            outcome_b::<ApprovedB>(),
                            g::seq(
                                intent::<REVIEW_A, A_WASI, 88>(),
                                merge_b_and_review_a(request_a::<71>()),
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
        g::seq(
            intent::<REVIEW_A, A_WASI, 80>(),
            g::send::<REVIEW, B_WASI, FirstA>(),
        ),
        g::seq(
            implement_a::<69>(),
            g::seq(
                g::par(request_a::<71>(), implement_b::<169>()),
                g::route(
                    g::seq(
                        outcome_a::<ApprovedA>(),
                        g::seq(
                            intent::<REVIEW_B, B_WASI, 84>(),
                            merge_a_and_review_b(request_b::<171>()),
                        ),
                    ),
                    g::seq(outcome_a::<RejectedA>(), repeat_a_before_b()),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
    )
}
fn first_b() -> impl Projectable {
    g::seq(
        g::seq(
            intent::<REVIEW_B, B_WASI, 81>(),
            g::send::<REVIEW, A_WASI, FirstB>(),
        ),
        g::seq(
            implement_b::<169>(),
            g::seq(
                g::par(request_b::<171>(), implement_a::<69>()),
                g::route(
                    g::seq(
                        outcome_b::<ApprovedB>(),
                        g::seq(
                            intent::<REVIEW_A, A_WASI, 88>(),
                            merge_b_and_review_a(request_a::<71>()),
                        ),
                    ),
                    g::seq(outcome_b::<RejectedB>(), repeat_b_before_a()),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
}

pub struct Programs {
    pub a_wasi: RoleProgram<A_WASI>,
    pub b_wasi: RoleProgram<B_WASI>,
    pub review: RoleProgram<REVIEW>,
    pub review_wasi: RoleProgram<REVIEW_WASI>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub repo: RoleProgram<REPO>,
    pub repo_endpoint: RoleProgram<REPO_ENDPOINT>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
}

impl Programs {
    /// Static observations of the same programs passed to `enter`.
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"6E",
            "roles":{"a_wasi":A_WASI,"b_wasi":B_WASI,"review_wasi":REVIEW_WASI,
                "review_a":REVIEW_A,"review_b":REVIEW_B,"repo":REPO,
                "review":REVIEW,"repo_endpoint":REPO_ENDPOINT,
                "a_io":A_IO,"b_io":B_IO,"review_io":REVIEW_IO},
            "programs":[rows(&self.a_wasi), rows(&self.b_wasi), rows(&self.review_wasi),
            rows(&self.review_a), rows(&self.review_b), rows(&self.repo), rows(&self.repo_endpoint),
            rows(&self.review),
            rows(&self.a_io), rows(&self.b_io), rows(&self.review_io)]})
    }
}

pub fn programs() -> Programs {
    let task = g::seq(
        g::send::<REPO, REVIEW_WASI, ReviewControl>(),
        g::route(g::seq(first_a(), finish()), g::seq(first_b(), finish()))
            .resolve::<SELECT_REVIEW>(),
    );
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
        review: project(&protocol),
        review_wasi: project(&protocol),
        review_a: project(&protocol),
        review_b: project(&protocol),
        repo: project(&protocol),
        repo_endpoint: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        review_io: project(&protocol),
    }
}
