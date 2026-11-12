//! Actual candidate bytes flow from repository work to tests and review.
use crate::shell::imp::repository_arguments::RepositoryArguments;
use hibana::{Endpoint, g, runtime::program::Projectable};

pub mod delivery;
pub mod wasi;

pub const DEVELOPER_A: u8 = 0;
pub const DEVELOPER_B: u8 = 1;
pub const TEST_WRITER: u8 = 2;
pub const REVIEWER: u8 = 3;
pub const CI_RUNNER: u8 = 4;
pub const REPOSITORY: u8 = 5;
pub const REVIEW_A: u8 = 6;
pub const REVIEW_B: u8 = 7;
pub const TEST_DATA_A: u8 = 8;
pub const TEST_DATA_B: u8 = 9;
pub const CI_SERVICE: u8 = 10;
pub const REPO: u8 = 11;
pub const TEST_ENV: u8 = 12;
pub const BUILD_SERVER: u8 = 13;
pub const SESSION: u32 = 62;
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "payment_processing";
pub const CI_DECISION_A: u16 = 863;
pub const CI_DECISION_B: u16 = 864;
pub const SUITE_A: &str = "full_test_suite_after_auth";
pub const SUITE_B: &str = "full_test_suite_after_payment";

pub type ImplementA = g::Msg<1, RepositoryArguments<1>>;
pub type ImplementB = g::Msg<2, RepositoryArguments<1>>;
pub type RepoLoan<'r> = g::Msg<3, Endpoint<'r, REPO>>;
pub type ImplementOnRepo = g::Msg<4, RepositoryArguments<1>>;
pub type RepoResult<'a> = g::Msg<5, &'a [u8]>;
pub type RepoReturn<'r> = g::Msg<6, Endpoint<'r, REPO>>;
pub type CandidateForReviewA<'a> = g::Msg<7, &'a [u8]>;
pub type CandidateForReviewB<'a> = g::Msg<8, &'a [u8]>;
pub type CandidateForTestsA<'a> = g::Msg<9, &'a [u8]>;
pub type CandidateForTestsB<'a> = g::Msg<10, &'a [u8]>;
pub type TestIntentA = g::Msg<11, RepositoryArguments<4>>;
pub type TestIntentB = g::Msg<12, RepositoryArguments<4>>;
pub type TestLoan<'r> = g::Msg<15, Endpoint<'r, TEST_ENV>>;
pub type TestRequestA = g::Msg<16, RepositoryArguments<4>>;
pub type TestRequestB = g::Msg<17, RepositoryArguments<4>>;
pub type TestResultA<'a> = g::Msg<18, &'a [u8]>;
pub type TestResultB<'a> = g::Msg<19, &'a [u8]>;
pub type TestReturn<'r> = g::Msg<20, Endpoint<'r, TEST_ENV>>;
pub type TestedA<'a> = g::Msg<21, &'a [u8]>;
pub type TestedB<'a> = g::Msg<22, &'a [u8]>;
pub type TestsForCiA<'a> = g::Msg<23, &'a [u8]>;
pub type TestsForCiB<'a> = g::Msg<24, &'a [u8]>;
pub type ReviewFirstA = g::Msg<25, RepositoryArguments<3>>;
pub type ReviewFirstB = g::Msg<26, RepositoryArguments<3>>;
pub type ReviewRequestA = g::Msg<27, RepositoryArguments<3>>;
pub type ReviewRequestB = g::Msg<28, RepositoryArguments<3>>;
pub type ReviewResultA<'a> = g::Msg<29, &'a [u8]>;
pub type ReviewResultB<'a> = g::Msg<30, &'a [u8]>;
pub type BuildLoan<'r> = g::Msg<31, Endpoint<'r, BUILD_SERVER>>;
pub type BuildReturn<'r> = g::Msg<32, Endpoint<'r, BUILD_SERVER>>;
pub type CiIntentA = g::Msg<35, RepositoryArguments<5>>;
pub type CiIntentB = g::Msg<36, RepositoryArguments<5>>;
pub type RunCiA = g::Msg<39, RepositoryArguments<5>>;
pub type RunCiB = g::Msg<40, RepositoryArguments<5>>;
pub type CiPassedA<'a> = g::Msg<41, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type CiFailedA<'a> = g::Msg<42, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type CiPassedB<'a> = g::Msg<43, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type CiFailedB<'a> = g::Msg<44, crate::shell::imp::review_reply::ReviewReply<'a, false>>;

pub fn tests_a() -> impl Projectable {
    g::seq(
        g::send::<TEST_WRITER, TEST_DATA_A, TestIntentA>(),
        g::seq(
            g::send::<TEST_DATA_A, TEST_WRITER, CandidateForTestsA>(),
            g::seq(
                g::send::<TEST_ENV, TEST_DATA_A, TestRequestA>(),
                g::seq(
                    g::send::<TEST_DATA_A, TEST_ENV, TestResultA>(),
                    g::par(
                        g::send::<TEST_DATA_A, DEVELOPER_A, TestedA>(),
                        g::send::<TEST_WRITER, CI_SERVICE, TestsForCiA>(),
                    ),
                ),
            ),
        ),
    )
}
pub fn tests_b() -> impl Projectable {
    g::seq(
        g::send::<TEST_WRITER, TEST_DATA_B, TestIntentB>(),
        g::seq(
            g::send::<TEST_DATA_B, TEST_WRITER, CandidateForTestsB>(),
            g::seq(
                g::send::<TEST_ENV, TEST_DATA_B, TestRequestB>(),
                g::seq(
                    g::send::<TEST_DATA_B, TEST_ENV, TestResultB>(),
                    g::par(
                        g::send::<TEST_DATA_B, DEVELOPER_B, TestedB>(),
                        g::send::<TEST_WRITER, CI_SERVICE, TestsForCiB>(),
                    ),
                ),
            ),
        ),
    )
}
fn select_a() -> impl Projectable {
    g::seq(
        g::send::<REVIEWER, REPOSITORY, ReviewFirstA>(),
        g::seq(
            g::send::<REVIEWER, DEVELOPER_A, ReviewFirstA>(),
            g::seq(
                g::send::<REVIEWER, DEVELOPER_B, ReviewFirstA>(),
                g::send::<REVIEWER, TEST_WRITER, ReviewFirstA>(),
            ),
        ),
    )
}
fn select_b() -> impl Projectable {
    g::seq(
        g::send::<REVIEWER, REPOSITORY, ReviewFirstB>(),
        g::seq(
            g::send::<REVIEWER, DEVELOPER_A, ReviewFirstB>(),
            g::seq(
                g::send::<REVIEWER, DEVELOPER_B, ReviewFirstB>(),
                g::send::<REVIEWER, TEST_WRITER, ReviewFirstB>(),
            ),
        ),
    )
}
