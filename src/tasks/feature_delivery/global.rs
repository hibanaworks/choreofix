//! Actual operation files, independent preparation and serial integrated publications.
use super::arguments::Arguments;
use crate::shell::global::{Exit, FileExit, Loan};
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const SESSION: u32 = 62;
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "payment_processing";
pub const SUITE_A: &str = "full_test_suite_after_auth";
pub const SUITE_B: &str = "full_test_suite_after_payment";
pub const REPO_OWNER: u8 = 0;
pub const REVIEW_A: u8 = 6;
pub const REVIEW_B: u8 = 7;
pub const TEST_OWNER: u8 = 8;
pub const CI_OWNER: u8 = 10;
pub const REPO: u8 = 11;
pub const TEST_ENV: u8 = 12;
pub const BUILD_SERVER: u8 = 13;
pub const RESOURCE_INIT: u8 = 15;
pub const A_FILE: u8 = 32;
pub const B_FILE: u8 = 33;
pub const A_REVIEW_FILE: u8 = 34;
pub const B_REVIEW_FILE: u8 = 35;
pub const TEST_FILE: u8 = 36;
pub const CI_FILE: u8 = 37;
pub const A_VM: u8 = 64;
pub const B_VM: u8 = 65;
pub const REVIEW_VM: u8 = 66;
pub const TEST_VM: u8 = 67;
pub const CI_VM: u8 = 68;
pub const A_DATA: u8 = 96;
pub const B_DATA: u8 = 97;
pub const A_REVIEW_DATA: u8 = 98;
pub const B_REVIEW_DATA: u8 = 99;
pub const TEST_DATA: u8 = 100;
pub const CI_DATA: u8 = 101;
pub const A_IO: u8 = 16;
pub const B_IO: u8 = 17;
pub const REVIEW_IO: u8 = 18;
pub const TEST_IO: u8 = 19;
pub const CI_IO: u8 = 20;
pub const MERGE_ORDER: u16 = 862;
pub const REVIEW_DECISION_A: u16 = 865;
pub const REVIEW_DECISION_B: u16 = 866;
pub const CI_DECISION_A: u16 = 863;
pub const CI_DECISION_B: u16 = 864;

pub type RepoLoan<'r> = g::Msg<10, Endpoint<'r, REPO>>;
pub type TestLoan<'r> = g::Msg<11, Endpoint<'r, TEST_ENV>>;
pub type BuildLoan<'r> = g::Msg<12, Endpoint<'r, BUILD_SERVER>>;
pub type ImplementA = g::Msg<200, Arguments<0>>;
pub type ImplementB = g::Msg<201, Arguments<1>>;
pub type MergeA = g::Msg<202, Arguments<2>>;
pub type MergeB = g::Msg<203, Arguments<3>>;
pub type ReviewRequestA = g::Msg<204, Arguments<4>>;
pub type ReviewRequestB = g::Msg<205, Arguments<5>>;
pub type TestRequestA = g::Msg<206, Arguments<6>>;
pub type TestRequestB = g::Msg<207, Arguments<7>>;
pub type CiRequestA = g::Msg<208, Arguments<8>>;
pub type CiRequestB = g::Msg<209, Arguments<9>>;
pub type ImplementationA<'a> = g::Msg<210, &'a [u8]>;
pub type ImplementationB<'a> = g::Msg<211, &'a [u8]>;
pub type ReadyReviewA<'a> = g::Msg<212, &'a [u8]>;
pub type ReadyReviewB<'a> = g::Msg<213, &'a [u8]>;
pub type ReadyTestA<'a> = g::Msg<214, &'a [u8]>;
pub type ReadyTestB<'a> = g::Msg<215, &'a [u8]>;
pub type TestedA<'a> = g::Msg<216, &'a [u8]>;
pub type TestedB<'a> = g::Msg<217, &'a [u8]>;
pub type FileTestedA<'a> = g::Msg<218, &'a [u8]>;
pub type FileTestedB<'a> = g::Msg<219, &'a [u8]>;
pub type ApprovedA<'a> = g::Msg<220, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<221, ReviewReply<'a, false>>;
pub type ApprovedB<'a> = g::Msg<222, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<223, ReviewReply<'a, false>>;
pub type ReviewFeedbackA<'a> = g::Msg<224, &'a [u8]>;
pub type ReviewFeedbackB<'a> = g::Msg<225, &'a [u8]>;
pub type FileReviewResultA<'a> = g::Msg<226, &'a [u8]>;
pub type FileReviewResultB<'a> = g::Msg<227, &'a [u8]>;
pub type MergeReadyA<'a> = g::Msg<228, &'a [u8]>;
pub type MergeReadyB<'a> = g::Msg<229, &'a [u8]>;
pub type MergedA<'a> = g::Msg<230, &'a [u8]>;
pub type MergedB<'a> = g::Msg<231, &'a [u8]>;
pub type PublicationA<'a> = g::Msg<232, &'a [u8]>;
pub type PublicationB<'a> = g::Msg<233, &'a [u8]>;
pub type FilePublicationA<'a> = g::Msg<234, &'a [u8]>;
pub type FilePublicationB<'a> = g::Msg<235, &'a [u8]>;
pub type CiPassedA<'a> = g::Msg<236, ReviewReply<'a, true>>;
pub type CiFailedA<'a> = g::Msg<237, ReviewReply<'a, false>>;
pub type CiPassedB<'a> = g::Msg<238, ReviewReply<'a, true>>;
pub type CiFailedB<'a> = g::Msg<239, ReviewReply<'a, false>>;
pub type CiUpdate<'a> = g::Msg<240, &'a [u8]>;
pub type FileCiResultA<'a> = g::Msg<242, &'a [u8]>;
pub type FileCiResultB<'a> = g::Msg<243, &'a [u8]>;
pub type InterfaceA<'a> = g::Msg<244, &'a [u8]>;
pub type InterfaceB<'a> = g::Msg<245, &'a [u8]>;
pub type TaskExit = g::Msg<246, i32>;
pub type LastCiPassedA<'a> = g::Msg<247, ReviewReply<'a, true>>;
pub type LastCiPassedB<'a> = g::Msg<252, ReviewReply<'a, true>>;
pub type FileReviewCandidateA<'a> = g::Msg<248, &'a [u8]>;
pub type FileReviewCandidateB<'a> = g::Msg<249, &'a [u8]>;
pub type FileTestCandidateA<'a> = g::Msg<250, &'a [u8]>;
pub type FileTestCandidateB<'a> = g::Msg<251, &'a [u8]>;
pub type RepairApprovedA<'a> = g::Msg<180, ReviewReply<'a, true>>;
pub type RepairRejectedA<'a> = g::Msg<181, ReviewReply<'a, false>>;
pub type RepairApprovedB<'a> = g::Msg<182, ReviewReply<'a, true>>;
pub type RepairRejectedB<'a> = g::Msg<183, ReviewReply<'a, false>>;

const fn writable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66, 1),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}
const fn readable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2, 1),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: None,
            readdir: None,
        },
    )
}
pub const DEVELOPER_FILES: [ChoreoFsObject<'static>; 7] = [
    writable(b"service/implement_feature", 4),
    writable(b"service/merge_code", 5),
    readable(b"service/review", 6),
    readable(b"service/merge_ready", 7),
    readable(b"service/integration", 8),
    readable(b"service/interface_change", 9),
    readable(b"service/peer_review", 10),
];
pub const REVIEW_FILES: [ChoreoFsObject<'static>; 6] = [
    writable(b"service/code_review/user_authentication", 4),
    writable(b"service/code_review/payment_processing", 5),
    readable(b"service/integration/user_authentication", 6),
    readable(b"service/integration/payment_processing", 7),
    readable(b"service/code_ready/user_authentication", 8),
    readable(b"service/code_ready/payment_processing", 9),
];
pub const TEST_FILES: [ChoreoFsObject<'static>; 7] = [
    writable(b"service/write_tests/user_authentication", 4),
    writable(b"service/write_tests/payment_processing", 5),
    readable(b"service/integration", 6),
    readable(b"service/code_ready/user_authentication", 8),
    readable(b"service/code_ready/payment_processing", 9),
    readable(b"service/review/user_authentication", 10),
    readable(b"service/review/payment_processing", 11),
];
pub const CI_FILES: [ChoreoFsObject<'static>; 4] = [
    writable(b"service/run_tests/user_authentication", 4),
    writable(b"service/run_tests/payment_processing", 5),
    readable(b"service/publication/user_authentication", 6),
    readable(b"service/publication/payment_processing", 7),
];

fn attempt_a() -> impl Projectable + Copy {
    g::seq(
        g::send::<A_FILE, REPO, ImplementA>(),
        g::seq(
            g::send::<REPO, A_FILE, ImplementationA>(),
            g::par(
                g::seq(
                    g::send::<REPO, REVIEW_A, ReadyReviewA>(),
                    g::seq(
                        g::send::<REPO, A_REVIEW_FILE, FileReviewCandidateA>(),
                        g::send::<A_REVIEW_FILE, REVIEW_A, ReviewRequestA>(),
                    ),
                ),
                g::par(
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReadyTestA>(),
                        g::seq(
                            g::send::<REPO, TEST_FILE, FileTestCandidateA>(),
                            g::seq(
                                g::send::<TEST_FILE, TEST_ENV, TestRequestA>(),
                                g::seq(
                                    g::send::<TEST_ENV, REPO, TestedA>(),
                                    g::send::<TEST_ENV, TEST_FILE, FileTestedA>(),
                                ),
                            ),
                        ),
                    ),
                    g::send::<REPO, B_FILE, InterfaceA>(),
                ),
            ),
        ),
    )
}
fn review_outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, ReviewFeedbackA>(),
            g::seq(
                g::send::<REPO, B_FILE, ReviewFeedbackA>(),
                g::seq(
                    g::send::<REPO, TEST_FILE, ReviewFeedbackA>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReviewFeedbackA>(),
                        g::send::<REVIEW_A, A_REVIEW_FILE, FileReviewResultA>(),
                    ),
                ),
            ),
        ),
    )
}
fn prepare_a<Approved: g::Message, Rejected: g::Message>() -> impl Projectable {
    g::seq(
        attempt_a(),
        g::route(
            review_outcome_a::<Approved>(),
            g::seq(
                g::seq(review_outcome_a::<Rejected>(), attempt_a()).roll(),
                review_outcome_a::<Approved>(),
            ),
        )
        .resolve::<REVIEW_DECISION_A>(),
    )
}
fn merge_ci_a() -> impl Projectable {
    g::seq(
        g::send::<REPO, A_FILE, MergeReadyA>(),
        g::seq(
            g::send::<A_FILE, REPO, MergeA>(),
            g::seq(
                g::send::<REPO, A_FILE, MergedA>(),
                g::seq(
                    g::send::<REPO, BUILD_SERVER, PublicationA>(),
                    g::seq(
                        g::send::<BUILD_SERVER, CI_FILE, FilePublicationA>(),
                        g::send::<CI_FILE, BUILD_SERVER, CiRequestA>(),
                    ),
                ),
            ),
        ),
    )
}
fn ci_outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, CiUpdate>(),
            g::seq(
                g::send::<REPO, B_FILE, CiUpdate>(),
                g::seq(
                    g::send::<REPO, A_REVIEW_FILE, CiUpdate>(),
                    g::seq(
                        g::send::<REPO, TEST_FILE, CiUpdate>(),
                        g::seq(
                            g::send::<REPO, REVIEW_A, CiUpdate>(),
                            g::seq(
                                g::send::<REPO, TEST_ENV, CiUpdate>(),
                                g::send::<BUILD_SERVER, CI_FILE, FileCiResultA>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn integrate_a<Passed: g::Message>() -> impl Projectable {
    g::seq(
        merge_ci_a(),
        g::route(
            ci_outcome_a::<Passed>(),
            g::seq(
                g::seq(
                    ci_outcome_a::<CiFailedA>(),
                    g::seq(
                        prepare_a::<RepairApprovedA, RepairRejectedA>(),
                        merge_ci_a(),
                    ),
                )
                .roll(),
                ci_outcome_a::<Passed>(),
            ),
        )
        .resolve::<CI_DECISION_A>(),
    )
}

fn attempt_b() -> impl Projectable + Copy {
    g::seq(
        g::send::<B_FILE, REPO, ImplementB>(),
        g::seq(
            g::send::<REPO, B_FILE, ImplementationB>(),
            g::par(
                g::seq(
                    g::send::<REPO, REVIEW_B, ReadyReviewB>(),
                    g::seq(
                        g::send::<REPO, B_REVIEW_FILE, FileReviewCandidateB>(),
                        g::send::<B_REVIEW_FILE, REVIEW_B, ReviewRequestB>(),
                    ),
                ),
                g::par(
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReadyTestB>(),
                        g::seq(
                            g::send::<REPO, TEST_FILE, FileTestCandidateB>(),
                            g::seq(
                                g::send::<TEST_FILE, TEST_ENV, TestRequestB>(),
                                g::seq(
                                    g::send::<TEST_ENV, REPO, TestedB>(),
                                    g::send::<TEST_ENV, TEST_FILE, FileTestedB>(),
                                ),
                            ),
                        ),
                    ),
                    g::send::<REPO, A_FILE, InterfaceB>(),
                ),
            ),
        ),
    )
}
fn review_outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, ReviewFeedbackB>(),
            g::seq(
                g::send::<REPO, B_FILE, ReviewFeedbackB>(),
                g::seq(
                    g::send::<REPO, TEST_FILE, ReviewFeedbackB>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReviewFeedbackB>(),
                        g::send::<REVIEW_B, B_REVIEW_FILE, FileReviewResultB>(),
                    ),
                ),
            ),
        ),
    )
}
fn prepare_b<Approved: g::Message, Rejected: g::Message>() -> impl Projectable {
    g::seq(
        attempt_b(),
        g::route(
            review_outcome_b::<Approved>(),
            g::seq(
                g::seq(review_outcome_b::<Rejected>(), attempt_b()).roll(),
                review_outcome_b::<Approved>(),
            ),
        )
        .resolve::<REVIEW_DECISION_B>(),
    )
}
fn merge_ci_b() -> impl Projectable {
    g::seq(
        g::send::<REPO, B_FILE, MergeReadyB>(),
        g::seq(
            g::send::<B_FILE, REPO, MergeB>(),
            g::seq(
                g::send::<REPO, B_FILE, MergedB>(),
                g::seq(
                    g::send::<REPO, BUILD_SERVER, PublicationB>(),
                    g::seq(
                        g::send::<BUILD_SERVER, CI_FILE, FilePublicationB>(),
                        g::send::<CI_FILE, BUILD_SERVER, CiRequestB>(),
                    ),
                ),
            ),
        ),
    )
}
fn ci_outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, CiUpdate>(),
            g::seq(
                g::send::<REPO, B_FILE, CiUpdate>(),
                g::seq(
                    g::send::<REPO, B_REVIEW_FILE, CiUpdate>(),
                    g::seq(
                        g::send::<REPO, TEST_FILE, CiUpdate>(),
                        g::seq(
                            g::send::<REPO, REVIEW_B, CiUpdate>(),
                            g::seq(
                                g::send::<REPO, TEST_ENV, CiUpdate>(),
                                g::send::<BUILD_SERVER, CI_FILE, FileCiResultB>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn integrate_b<Passed: g::Message>() -> impl Projectable {
    g::seq(
        merge_ci_b(),
        g::route(
            ci_outcome_b::<Passed>(),
            g::seq(
                g::seq(
                    ci_outcome_b::<CiFailedB>(),
                    g::seq(
                        prepare_b::<RepairApprovedB, RepairRejectedB>(),
                        merge_ci_b(),
                    ),
                )
                .roll(),
                ci_outcome_b::<Passed>(),
            ),
        )
        .resolve::<CI_DECISION_B>(),
    )
}

pub fn source() -> impl Projectable {
    let resources = g::par(
        g::send::<RESOURCE_INIT, REPO_OWNER, RepoLoan>(),
        g::par(
            g::send::<RESOURCE_INIT, TEST_OWNER, TestLoan>(),
            g::send::<RESOURCE_INIT, CI_OWNER, BuildLoan>(),
        ),
    );
    let files = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::par(
                g::send::<A_REVIEW_DATA, REVIEW_VM, Loan<A_REVIEW_FILE>>(),
                g::par(
                    g::send::<B_REVIEW_DATA, REVIEW_VM, Loan<B_REVIEW_FILE>>(),
                    g::par(
                        g::send::<TEST_DATA, TEST_VM, Loan<TEST_FILE>>(),
                        g::send::<CI_DATA, CI_VM, Loan<CI_FILE>>(),
                    ),
                ),
            ),
        ),
    );
    let task = g::seq(
        g::par(
            prepare_a::<ApprovedA, RejectedA>(),
            prepare_b::<ApprovedB, RejectedB>(),
        ),
        g::seq(
            g::route(
                g::seq(integrate_a::<CiPassedA>(), integrate_b::<LastCiPassedB>()),
                g::seq(integrate_b::<CiPassedB>(), integrate_a::<LastCiPassedA>()),
            )
            .resolve::<MERGE_ORDER>(),
            g::seq(
                g::send::<REPO, TEST_ENV, TaskExit>(),
                g::seq(
                    g::send::<REPO, REVIEW_A, TaskExit>(),
                    g::seq(
                        g::send::<REPO, REVIEW_B, TaskExit>(),
                        g::send::<REPO, BUILD_SERVER, TaskExit>(),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(
        g::seq(
            crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
            g::send::<A_FILE, A_DATA, FileExit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                g::send::<B_FILE, B_DATA, FileExit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<A_REVIEW_FILE, A_REVIEW_DATA>().roll(),
                    g::send::<A_REVIEW_FILE, A_REVIEW_DATA, FileExit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<B_REVIEW_FILE, B_REVIEW_DATA>()
                            .roll(),
                        g::send::<B_REVIEW_FILE, B_REVIEW_DATA, FileExit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::file_metadata::<TEST_FILE, TEST_DATA>().roll(),
                            g::send::<TEST_FILE, TEST_DATA, FileExit>(),
                        ),
                        g::seq(
                            crate::shell::global::file_metadata::<CI_FILE, CI_DATA>().roll(),
                            g::send::<CI_FILE, CI_DATA, FileExit>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            crate::shell::global::private_io::<A_VM, A_IO>(),
            g::send::<A_VM, A_IO, Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<B_VM, B_IO>(),
                g::send::<B_VM, B_IO, Exit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::private_io::<REVIEW_VM, REVIEW_IO>(),
                    g::send::<REVIEW_VM, REVIEW_IO, Exit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::private_io::<TEST_VM, TEST_IO>(),
                        g::send::<TEST_VM, TEST_IO, Exit>(),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<CI_VM, CI_IO>(),
                        g::send::<CI_VM, CI_IO, Exit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(resources, g::seq(files, g::par(task, g::par(metadata, io))))
}
pub struct Programs {
    pub repo_owner: RoleProgram<REPO_OWNER>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub test_owner: RoleProgram<TEST_OWNER>,
    pub ci_owner: RoleProgram<CI_OWNER>,
    pub repo: RoleProgram<REPO>,
    pub environment: RoleProgram<TEST_ENV>,
    pub build: RoleProgram<BUILD_SERVER>,
    pub resource_init: RoleProgram<RESOURCE_INIT>,
    pub a_file: RoleProgram<A_FILE>,
    pub b_file: RoleProgram<B_FILE>,
    pub a_review_file: RoleProgram<A_REVIEW_FILE>,
    pub b_review_file: RoleProgram<B_REVIEW_FILE>,
    pub test_file: RoleProgram<TEST_FILE>,
    pub ci_file: RoleProgram<CI_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub review_vm: RoleProgram<REVIEW_VM>,
    pub test_vm: RoleProgram<TEST_VM>,
    pub ci_vm: RoleProgram<CI_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_data: RoleProgram<B_DATA>,
    pub a_review_data: RoleProgram<A_REVIEW_DATA>,
    pub b_review_data: RoleProgram<B_REVIEW_DATA>,
    pub test_data: RoleProgram<TEST_DATA>,
    pub ci_data: RoleProgram<CI_DATA>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
    pub test_io: RoleProgram<TEST_IO>,
    pub ci_io: RoleProgram<CI_IO>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("RepoLoan", RepoLoan::LOGICAL_LABEL),
            ("TestLoan", TestLoan::LOGICAL_LABEL),
            ("BuildLoan", BuildLoan::LOGICAL_LABEL),
            ("ImplementA", ImplementA::LOGICAL_LABEL),
            ("ImplementB", ImplementB::LOGICAL_LABEL),
            ("MergeA", MergeA::LOGICAL_LABEL),
            ("MergeB", MergeB::LOGICAL_LABEL),
            ("ReviewRequestA", ReviewRequestA::LOGICAL_LABEL),
            ("ReviewRequestB", ReviewRequestB::LOGICAL_LABEL),
            ("TestRequestA", TestRequestA::LOGICAL_LABEL),
            ("TestRequestB", TestRequestB::LOGICAL_LABEL),
            ("CiRequestA", CiRequestA::LOGICAL_LABEL),
            ("CiRequestB", CiRequestB::LOGICAL_LABEL),
            ("ImplementationA", ImplementationA::LOGICAL_LABEL),
            ("ImplementationB", ImplementationB::LOGICAL_LABEL),
            ("ReadyReviewA", ReadyReviewA::LOGICAL_LABEL),
            ("ReadyReviewB", ReadyReviewB::LOGICAL_LABEL),
            ("ReadyTestA", ReadyTestA::LOGICAL_LABEL),
            ("ReadyTestB", ReadyTestB::LOGICAL_LABEL),
            ("TestedA", TestedA::LOGICAL_LABEL),
            ("TestedB", TestedB::LOGICAL_LABEL),
            ("FileTestedA", FileTestedA::LOGICAL_LABEL),
            ("FileTestedB", FileTestedB::LOGICAL_LABEL),
            ("ApprovedA", ApprovedA::LOGICAL_LABEL),
            ("RejectedA", RejectedA::LOGICAL_LABEL),
            ("ApprovedB", ApprovedB::LOGICAL_LABEL),
            ("RejectedB", RejectedB::LOGICAL_LABEL),
            ("ReviewFeedbackA", ReviewFeedbackA::LOGICAL_LABEL),
            ("ReviewFeedbackB", ReviewFeedbackB::LOGICAL_LABEL),
            ("FileReviewResultA", FileReviewResultA::LOGICAL_LABEL),
            ("FileReviewResultB", FileReviewResultB::LOGICAL_LABEL),
            ("MergeReadyA", MergeReadyA::LOGICAL_LABEL),
            ("MergeReadyB", MergeReadyB::LOGICAL_LABEL),
            ("MergedA", MergedA::LOGICAL_LABEL),
            ("MergedB", MergedB::LOGICAL_LABEL),
            ("PublicationA", PublicationA::LOGICAL_LABEL),
            ("PublicationB", PublicationB::LOGICAL_LABEL),
            ("FilePublicationA", FilePublicationA::LOGICAL_LABEL),
            ("FilePublicationB", FilePublicationB::LOGICAL_LABEL),
            ("CiPassedA", CiPassedA::LOGICAL_LABEL),
            ("CiFailedA", CiFailedA::LOGICAL_LABEL),
            ("CiPassedB", CiPassedB::LOGICAL_LABEL),
            ("CiFailedB", CiFailedB::LOGICAL_LABEL),
            ("CiUpdate", CiUpdate::LOGICAL_LABEL),
            ("FileCiResultA", FileCiResultA::LOGICAL_LABEL),
            ("FileCiResultB", FileCiResultB::LOGICAL_LABEL),
            ("InterfaceA", InterfaceA::LOGICAL_LABEL),
            ("InterfaceB", InterfaceB::LOGICAL_LABEL),
            ("TaskExit", TaskExit::LOGICAL_LABEL),
            ("LastCiPassedA", LastCiPassedA::LOGICAL_LABEL),
            ("LastCiPassedB", LastCiPassedB::LOGICAL_LABEL),
            ("FileReviewCandidateA", FileReviewCandidateA::LOGICAL_LABEL),
            ("FileReviewCandidateB", FileReviewCandidateB::LOGICAL_LABEL),
            ("FileTestCandidateA", FileTestCandidateA::LOGICAL_LABEL),
            ("FileTestCandidateB", FileTestCandidateB::LOGICAL_LABEL),
            ("RepairApprovedA", RepairApprovedA::LOGICAL_LABEL),
            ("RepairRejectedA", RepairRejectedA::LOGICAL_LABEL),
            ("RepairApprovedB", RepairApprovedB::LOGICAL_LABEL),
            ("RepairRejectedB", RepairRejectedB::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"6M","session":SESSION,"labels":labels,"roles":{"REPO_OWNER":REPO_OWNER,"REVIEW_A":REVIEW_A,"REVIEW_B":REVIEW_B,"TEST_OWNER":TEST_OWNER,"CI_OWNER":CI_OWNER,"REPO":REPO,"TEST_ENV":TEST_ENV,"BUILD_SERVER":BUILD_SERVER,"RESOURCE_INIT":RESOURCE_INIT,"A_FILE":A_FILE,"B_FILE":B_FILE,"A_REVIEW_FILE":A_REVIEW_FILE,"B_REVIEW_FILE":B_REVIEW_FILE,"TEST_FILE":TEST_FILE,"CI_FILE":CI_FILE,"A_VM":A_VM,"B_VM":B_VM,"REVIEW_VM":REVIEW_VM,"TEST_VM":TEST_VM,"CI_VM":CI_VM,"A_DATA":A_DATA,"B_DATA":B_DATA,"A_REVIEW_DATA":A_REVIEW_DATA,"B_REVIEW_DATA":B_REVIEW_DATA,"TEST_DATA":TEST_DATA,"CI_DATA":CI_DATA,"A_IO":A_IO,"B_IO":B_IO,"REVIEW_IO":REVIEW_IO,"TEST_IO":TEST_IO,"CI_IO":CI_IO},"programs":[rows(&self.repo_owner),rows(&self.review_a),rows(&self.review_b),rows(&self.test_owner),rows(&self.ci_owner),rows(&self.repo),rows(&self.environment),rows(&self.build),rows(&self.resource_init),rows(&self.a_file),rows(&self.b_file),rows(&self.a_review_file),rows(&self.b_review_file),rows(&self.test_file),rows(&self.ci_file),rows(&self.a_vm),rows(&self.b_vm),rows(&self.review_vm),rows(&self.test_vm),rows(&self.ci_vm),rows(&self.a_data),rows(&self.b_data),rows(&self.a_review_data),rows(&self.b_review_data),rows(&self.test_data),rows(&self.ci_data),rows(&self.a_io),rows(&self.b_io),rows(&self.review_io),rows(&self.test_io),rows(&self.ci_io)]})
    }
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        repo_owner: project(&p),
        review_a: project(&p),
        review_b: project(&p),
        test_owner: project(&p),
        ci_owner: project(&p),
        repo: project(&p),
        environment: project(&p),
        build: project(&p),
        resource_init: project(&p),
        a_file: project(&p),
        b_file: project(&p),
        a_review_file: project(&p),
        b_review_file: project(&p),
        test_file: project(&p),
        ci_file: project(&p),
        a_vm: project(&p),
        b_vm: project(&p),
        review_vm: project(&p),
        test_vm: project(&p),
        ci_vm: project(&p),
        a_data: project(&p),
        b_data: project(&p),
        a_review_data: project(&p),
        b_review_data: project(&p),
        test_data: project(&p),
        ci_data: project(&p),
        a_io: project(&p),
        b_io: project(&p),
        review_io: project(&p),
        test_io: project(&p),
        ci_io: project(&p),
    }
}
