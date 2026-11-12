//! The same feature authority follows implementation, review, merge and CI.
//! These functions compose source terms only; Endpoint execution stays inline
//! in the corresponding localsides.
use super::*;
use crate::shell::imp::review_reply::ReviewReply;

pub const AUTH_A: u8 = 14;
pub const AUTH_B: u8 = 15;
pub const REVIEW_DECISION_A: u16 = 865;
pub const REVIEW_DECISION_B: u16 = 866;
pub const RECOVERY_REVIEW_A: u16 = 868;
pub const RECOVERY_REVIEW_B: u16 = 869;

pub type Feature<'r, const ROLE: u8> = g::Msg<45, Endpoint<'r, ROLE>>;
pub type Implement = g::Msg<46, RepositoryArguments<1>>;
pub type Review = g::Msg<48, RepositoryArguments<3>>;
pub type Approved<'a> = g::Msg<49, ReviewReply<'a, true>>;
pub type Rejected<'a> = g::Msg<50, ReviewReply<'a, false>>;
pub type Merge = g::Msg<53, RepositoryArguments<2>>;
pub type Merged<'a> = g::Msg<54, &'a [u8]>;
pub type ReviewApproved<'a> = g::Msg<55, ReviewReply<'a, true>>;
pub type ReviewRejected<'a> = g::Msg<56, ReviewReply<'a, false>>;
pub type IntegratedA<'a> = g::Msg<57, ReviewReply<'a, true>>;
pub type IntegrationFailedA<'a> = g::Msg<58, ReviewReply<'a, false>>;
pub type IntegratedB<'a> = g::Msg<59, ReviewReply<'a, true>>;
pub type IntegrationFailedB<'a> = g::Msg<60, ReviewReply<'a, false>>;
pub type RecoveryApproved<'a> = g::Msg<61, ReviewReply<'a, true>>;
pub type RecoveryRejected<'a> = g::Msg<62, ReviewReply<'a, false>>;
pub type CiResult<'a> = g::Msg<63, &'a [u8]>;
pub type MergeLoanA<'r> = g::Msg<64, Endpoint<'r, AUTH_A>>;
pub type MergeLoanB<'r> = g::Msg<65, Endpoint<'r, AUTH_B>>;
pub type RecoveryReviewApproved<'a> = g::Msg<66, ReviewReply<'a, true>>;
pub type RecoveryReviewRejected<'a> = g::Msg<67, ReviewReply<'a, false>>;
pub type MergeOnRepo = g::Msg<70, RepositoryArguments<2>>;
pub type RunCiOnFeatureA = g::Msg<71, RepositoryArguments<5>>;
pub type RunCiOnFeatureB = g::Msg<72, RepositoryArguments<5>>;
pub type MergeAdmissionA = g::Msg<73, RepositoryArguments<2>>;
pub type MergeAdmissionB = g::Msg<74, RepositoryArguments<2>>;
pub type InterfaceChangedA<'a> = g::Msg<75, &'a [u8]>;
pub type InterfaceChangedB<'a> = g::Msg<76, &'a [u8]>;
pub type LastCiPassedA<'a> = g::Msg<78, ReviewReply<'a, true>>;
pub type LastCiPassedB<'a> = g::Msg<79, ReviewReply<'a, true>>;

fn implement<const AUTHOR: u8, const AUTH: u8>(
    admission: impl Projectable,
    candidate_delivery: impl Projectable,
) -> impl Projectable {
    g::seq(
        admission,
        g::seq(
            g::send::<REPOSITORY, AUTHOR, RepoLoan>(),
            g::seq(
                g::par(
                    g::send::<AUTH, REPOSITORY, Implement>(),
                    g::send::<REPO, REPOSITORY, ImplementOnRepo>(),
                ),
                g::seq(
                    g::send::<REPOSITORY, REPO, RepoResult>(),
                    g::seq(
                        g::send::<AUTHOR, REPOSITORY, RepoReturn>(),
                        candidate_delivery,
                    ),
                ),
            ),
        ),
    )
}

fn review_feedback<
    const AUTHOR: u8,
    const AUTH: u8,
    const REV: u8,
    const TEST: u8,
    M: g::Message,
    N: g::Message,
>(
    peer: impl Projectable,
) -> impl Projectable {
    g::seq(
        g::send::<AUTH, AUTHOR, M>(),
        g::seq(
            peer,
            g::seq(
                g::send::<AUTH, REPOSITORY, N>(),
                g::seq(
                    g::send::<AUTH, TEST_WRITER, N>(),
                    g::seq(
                        g::send::<AUTH, CI_SERVICE, N>(),
                        g::seq(g::send::<AUTH, REV, N>(), g::send::<AUTH, TEST, N>()),
                    ),
                ),
            ),
        ),
    )
}

fn approved_a() -> impl Projectable {
    g::seq(
        review_feedback::<DEVELOPER_A, AUTH_A, REVIEW_A, TEST_DATA_A, Approved, ReviewApproved>(
            g::send::<AUTH_A, DEVELOPER_B, ReviewApproved>(),
        ),
        g::send::<REVIEWER, REPOSITORY, Feature<AUTH_A>>(),
    )
}
fn rejected_a() -> impl Projectable {
    g::seq(
        review_feedback::<DEVELOPER_A, AUTH_A, REVIEW_A, TEST_DATA_A, Rejected, ReviewRejected>(
            g::send::<AUTH_A, DEVELOPER_B, ReviewRejected>(),
        ),
        g::send::<REVIEWER, DEVELOPER_A, Feature<AUTH_A>>(),
    )
}
fn approved_b() -> impl Projectable {
    g::seq(
        review_feedback::<DEVELOPER_B, AUTH_B, REVIEW_B, TEST_DATA_B, Approved, ReviewApproved>(
            g::send::<AUTH_B, DEVELOPER_A, ReviewApproved>(),
        ),
        g::send::<REVIEWER, REPOSITORY, Feature<AUTH_B>>(),
    )
}
fn rejected_b() -> impl Projectable {
    g::seq(
        review_feedback::<DEVELOPER_B, AUTH_B, REVIEW_B, TEST_DATA_B, Rejected, ReviewRejected>(
            g::send::<AUTH_B, DEVELOPER_A, ReviewRejected>(),
        ),
        g::send::<REVIEWER, DEVELOPER_B, Feature<AUTH_B>>(),
    )
}
fn candidate_a() -> impl Projectable {
    implement::<DEVELOPER_A, AUTH_A>(
        g::send::<DEVELOPER_A, REPOSITORY, ImplementA>(),
        g::seq(
            g::send::<REPOSITORY, REVIEW_A, CandidateForReviewA>(),
            g::seq(
                g::send::<REPOSITORY, TEST_DATA_A, CandidateForTestsA>(),
                g::send::<AUTH_A, DEVELOPER_B, InterfaceChangedA>(),
            ),
        ),
    )
}
fn candidate_b() -> impl Projectable {
    implement::<DEVELOPER_B, AUTH_B>(
        g::send::<DEVELOPER_B, REPOSITORY, ImplementB>(),
        g::seq(
            g::send::<REPOSITORY, REVIEW_B, CandidateForReviewB>(),
            g::seq(
                g::send::<REPOSITORY, TEST_DATA_B, CandidateForTestsB>(),
                g::send::<AUTH_B, DEVELOPER_A, InterfaceChangedB>(),
            ),
        ),
    )
}
fn implement_a() -> impl Projectable {
    g::seq(
        candidate_a(),
        g::send::<DEVELOPER_A, REVIEWER, Feature<AUTH_A>>(),
    )
}
fn implement_b() -> impl Projectable {
    g::seq(
        candidate_b(),
        g::send::<DEVELOPER_B, REVIEWER, Feature<AUTH_B>>(),
    )
}
fn tests_a() -> impl Projectable {
    super::tests_a()
}
fn tests_b() -> impl Projectable {
    super::tests_b()
}
fn request_a() -> impl Projectable {
    g::seq(
        g::send::<AUTH_A, REVIEW_A, Review>(),
        g::send::<REVIEW_A, AUTH_A, ReviewResultA>(),
    )
}
fn request_b() -> impl Projectable {
    g::seq(
        g::send::<AUTH_B, REVIEW_B, Review>(),
        g::send::<REVIEW_B, AUTH_B, ReviewResultB>(),
    )
}
fn repair_a() -> impl Projectable {
    g::seq(
        implement_a(),
        g::seq(
            request_a(),
            g::seq(tests_a(), g::send::<REVIEW_A, REPOSITORY, ReviewResultA>()),
        ),
    )
}
fn repair_b() -> impl Projectable {
    g::seq(
        implement_b(),
        g::seq(
            request_b(),
            g::seq(tests_b(), g::send::<REVIEW_B, REPOSITORY, ReviewResultB>()),
        ),
    )
}
fn ci_repair_a() -> impl Projectable {
    g::seq(
        candidate_a(),
        g::seq(
            tests_a(),
            g::seq(
                g::send::<REVIEWER, REVIEW_A, ReviewRequestA>(),
                g::seq(
                    request_a(),
                    g::send::<REVIEW_A, REPOSITORY, ReviewResultA>(),
                ),
            ),
        ),
    )
}
fn ci_repair_b() -> impl Projectable {
    g::seq(
        candidate_b(),
        g::seq(
            tests_b(),
            g::seq(
                g::send::<REVIEWER, REVIEW_B, ReviewRequestB>(),
                g::seq(
                    request_b(),
                    g::send::<REVIEW_B, REPOSITORY, ReviewResultB>(),
                ),
            ),
        ),
    )
}
fn reviewed_a() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, REPOSITORY, ReviewResultA>(),
        g::route(
            approved_a(),
            g::seq(g::seq(rejected_a(), repair_a()).roll(), approved_a()),
        )
        .resolve::<REVIEW_DECISION_A>(),
    )
}
fn reviewed_b() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REPOSITORY, ReviewResultB>(),
        g::route(
            approved_b(),
            g::seq(g::seq(rejected_b(), repair_b()).roll(), approved_b()),
        )
        .resolve::<REVIEW_DECISION_B>(),
    )
}
fn recovery_reviewed_a() -> impl Projectable {
    let approved = || {
        review_feedback::<
            REVIEWER,
            AUTH_A,
            REVIEW_A,
            TEST_DATA_A,
            RecoveryApproved,
            RecoveryReviewApproved,
        >(g::send::<AUTH_A, DEVELOPER_B, RecoveryReviewApproved>())
    };
    let rejected = || {
        review_feedback::<
            REVIEWER,
            AUTH_A,
            REVIEW_A,
            TEST_DATA_A,
            RecoveryRejected,
            RecoveryReviewRejected,
        >(g::send::<AUTH_A, DEVELOPER_B, RecoveryReviewRejected>())
    };
    g::route(
        approved(),
        g::seq(g::seq(rejected(), ci_repair_a()).roll(), approved()),
    )
    .resolve::<RECOVERY_REVIEW_A>()
}
fn recovery_reviewed_b() -> impl Projectable {
    let approved = || {
        review_feedback::<
            REVIEWER,
            AUTH_B,
            REVIEW_B,
            TEST_DATA_B,
            RecoveryApproved,
            RecoveryReviewApproved,
        >(g::send::<AUTH_B, DEVELOPER_A, RecoveryReviewApproved>())
    };
    let rejected = || {
        review_feedback::<
            REVIEWER,
            AUTH_B,
            REVIEW_B,
            TEST_DATA_B,
            RecoveryRejected,
            RecoveryReviewRejected,
        >(g::send::<AUTH_B, DEVELOPER_A, RecoveryReviewRejected>())
    };
    g::route(
        approved(),
        g::seq(g::seq(rejected(), ci_repair_b()).roll(), approved()),
    )
    .resolve::<RECOVERY_REVIEW_B>()
}

fn merge<const AUTHOR: u8, const AUTH: u8, M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<AUTHOR, REPOSITORY, M>(),
        g::seq(
            g::send::<REPOSITORY, AUTHOR, RepoLoan>(),
            g::seq(
                g::par(
                    g::send::<AUTH, REPOSITORY, Merge>(),
                    g::send::<REPO, REPOSITORY, MergeOnRepo>(),
                ),
                g::seq(
                    g::send::<REPOSITORY, AUTH, Merged>(),
                    g::seq(
                        g::send::<AUTHOR, REPOSITORY, RepoReturn>(),
                        g::seq(
                            g::send::<REPOSITORY, CI_SERVICE, Merged>(),
                            g::seq(
                                g::send::<AUTHOR, CI_RUNNER, Merged>(),
                                g::send::<AUTHOR, CI_RUNNER, Feature<AUTH>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn ci_request_a() -> impl Projectable {
    g::seq(merge::<DEVELOPER_A, AUTH_A, MergeAdmissionA>(), ci_a())
}
pub(crate) fn ci_a() -> impl Projectable {
    g::seq(
        g::send::<CI_RUNNER, CI_SERVICE, CiIntentA>(),
        g::seq(
            g::par(
                g::send::<AUTH_A, CI_SERVICE, RunCiOnFeatureA>(),
                g::send::<BUILD_SERVER, CI_SERVICE, RunCiA>(),
            ),
            g::seq(
                g::send::<CI_SERVICE, AUTH_A, CiResult>(),
                g::seq(
                    g::send::<CI_SERVICE, BUILD_SERVER, CiResult>(),
                    g::send::<CI_SERVICE, REPOSITORY, CiResult>(),
                ),
            ),
        ),
    )
}

fn ci_request_b() -> impl Projectable {
    g::seq(merge::<DEVELOPER_B, AUTH_B, MergeAdmissionB>(), ci_b())
}
pub(crate) fn ci_b() -> impl Projectable {
    g::seq(
        g::send::<CI_RUNNER, CI_SERVICE, CiIntentB>(),
        g::seq(
            g::par(
                g::send::<AUTH_B, CI_SERVICE, RunCiOnFeatureB>(),
                g::send::<BUILD_SERVER, CI_SERVICE, RunCiB>(),
            ),
            g::seq(
                g::send::<CI_SERVICE, AUTH_B, CiResult>(),
                g::seq(
                    g::send::<CI_SERVICE, BUILD_SERVER, CiResult>(),
                    g::send::<CI_SERVICE, REPOSITORY, CiResult>(),
                ),
            ),
        ),
    )
}
fn ci_feedback<const AUTH: u8, const REV: u8, const TEST: u8, N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<AUTH, DEVELOPER_A, N>(),
        g::seq(
            g::send::<AUTH, DEVELOPER_B, N>(),
            g::seq(
                g::send::<AUTH, REVIEWER, N>(),
                g::seq(
                    g::send::<AUTH, TEST_WRITER, N>(),
                    g::seq(
                        g::send::<AUTH, CI_SERVICE, N>(),
                        g::seq(
                            g::send::<AUTH, REV, N>(),
                            g::seq(g::send::<AUTH, TEST, N>(), g::send::<AUTH, CI_RUNNER, N>()),
                        ),
                    ),
                ),
            ),
        ),
    )
}
pub(crate) fn passed_a<N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<AUTH_A, REPOSITORY, IntegratedA>(),
        g::seq(
            g::send::<CI_RUNNER, REPOSITORY, Feature<AUTH_A>>(),
            ci_feedback::<AUTH_A, REVIEW_A, TEST_DATA_A, N>(),
        ),
    )
}
pub(crate) fn failed_a() -> impl Projectable {
    g::seq(
        g::send::<AUTH_A, REPOSITORY, IntegrationFailedA>(),
        g::seq(
            ci_feedback::<AUTH_A, REVIEW_A, TEST_DATA_A, CiFailedA>(),
            g::send::<CI_RUNNER, DEVELOPER_A, Feature<AUTH_A>>(),
        ),
    )
}
pub(crate) fn passed_b<N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<AUTH_B, REPOSITORY, IntegratedB>(),
        g::seq(
            g::send::<CI_RUNNER, REPOSITORY, Feature<AUTH_B>>(),
            ci_feedback::<AUTH_B, REVIEW_B, TEST_DATA_B, N>(),
        ),
    )
}
pub(crate) fn failed_b() -> impl Projectable {
    g::seq(
        g::send::<AUTH_B, REPOSITORY, IntegrationFailedB>(),
        g::seq(
            ci_feedback::<AUTH_B, REVIEW_B, TEST_DATA_B, CiFailedB>(),
            g::send::<CI_RUNNER, DEVELOPER_B, Feature<AUTH_B>>(),
        ),
    )
}
fn integrated_a<N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REPOSITORY, DEVELOPER_A, MergeLoanA>(),
        g::seq(
            ci_request_a(),
            g::route(
                passed_a::<N>(),
                g::seq(
                    g::seq(
                        failed_a(),
                        g::seq(ci_repair_a(), g::seq(recovery_reviewed_a(), ci_request_a())),
                    )
                    .roll(),
                    passed_a::<N>(),
                ),
            )
            .resolve::<CI_DECISION_A>(),
        ),
    )
}

fn integrated_b<N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REPOSITORY, DEVELOPER_B, MergeLoanB>(),
        g::seq(
            ci_request_b(),
            g::route(
                passed_b::<N>(),
                g::seq(
                    g::seq(
                        failed_b(),
                        g::seq(ci_repair_b(), g::seq(recovery_reviewed_b(), ci_request_b())),
                    )
                    .roll(),
                    passed_b::<N>(),
                ),
            )
            .resolve::<CI_DECISION_B>(),
        ),
    )
}

/// A whole delivery source. Initial reviews can execute while the repository
/// serves the other author. Tests retain both independent feature orders.
/// Each subsequent merge waits for the previous merge's actual successful CI.
pub fn protocol() -> impl Projectable {
    let tests = || g::route(g::seq(tests_a(), tests_b()), g::seq(tests_b(), tests_a()));
    let close = || {
        g::seq(
            g::send::<TEST_WRITER, REPOSITORY, TestReturn>(),
            g::send::<CI_RUNNER, CI_SERVICE, BuildReturn>(),
        )
    };
    let first_a = g::seq(
        super::select_a(),
        g::seq(
            implement_a(),
            g::seq(
                request_a(),
                g::seq(
                    implement_b(),
                    g::seq(
                        request_b(),
                        g::seq(tests(), g::seq(reviewed_a(), reviewed_b())),
                    ),
                ),
            ),
        ),
    );
    let first_b = g::seq(
        super::select_b(),
        g::seq(
            implement_b(),
            g::seq(
                request_b(),
                g::seq(
                    implement_a(),
                    g::seq(
                        request_a(),
                        g::seq(tests(), g::seq(reviewed_b(), reviewed_a())),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::send::<REPOSITORY, DEVELOPER_A, Feature<AUTH_A>>(),
        g::seq(
            g::send::<REPOSITORY, DEVELOPER_B, Feature<AUTH_B>>(),
            g::seq(
                g::send::<REPOSITORY, TEST_WRITER, TestLoan>(),
                g::seq(
                    g::send::<CI_SERVICE, CI_RUNNER, BuildLoan>(),
                    g::seq(
                        g::route(first_a, first_b),
                        g::route(
                            g::seq(
                                integrated_a::<CiPassedA>(),
                                g::seq(integrated_b::<LastCiPassedB>(), close()),
                            ),
                            g::seq(
                                integrated_b::<CiPassedB>(),
                                g::seq(integrated_a::<LastCiPassedA>(), close()),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_reentry_has_its_feature_owner() {
        let source = g::seq(
            g::send::<REPOSITORY, REVIEWER, Feature<AUTH_A>>(),
            g::seq(g::send::<REPOSITORY, TEST_WRITER, TestLoan>(), reviewed_a()),
        );
        assert_eq!(g::diagnose(&source), None);
    }

    #[test]
    fn ci_reentry_has_its_feature_owner() {
        let source = g::seq(
            g::send::<REPOSITORY, TEST_WRITER, TestLoan>(),
            g::seq(
                g::send::<CI_SERVICE, CI_RUNNER, BuildLoan>(),
                integrated_a::<CiPassedA>(),
            ),
        );
        assert_eq!(g::diagnose(&source), None);
    }

    #[test]
    fn review_and_ci_reentries_compose() {
        let source = g::seq(
            g::send::<REPOSITORY, REVIEWER, Feature<AUTH_A>>(),
            g::seq(
                g::send::<REPOSITORY, TEST_WRITER, TestLoan>(),
                g::seq(
                    g::send::<CI_SERVICE, CI_RUNNER, BuildLoan>(),
                    g::seq(
                        reviewed_a(),
                        g::seq(
                            integrated_a::<CiPassedA>(),
                            g::seq(
                                g::send::<TEST_WRITER, REPOSITORY, TestReturn>(),
                                g::send::<CI_RUNNER, CI_SERVICE, BuildReturn>(),
                            ),
                        ),
                    ),
                ),
            ),
        );
        assert_eq!(g::diagnose(&source), None);
    }
}
