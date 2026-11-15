//! Direct operation-file choreography for three development and staged-release pipelines.
#[cfg(test)]
mod tests;

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

pub const SESSION: u32 = 63;
pub const REPO_OWNER: u8 = 0;
pub const TEST_OWNER: u8 = 1;
pub const CI_OWNER: u8 = 2;
pub const STAGE_OWNER: u8 = 3;
pub const REVIEW_A: u8 = 4;
pub const REVIEW_B: u8 = 5;
pub const REVIEW_C: u8 = 6;
pub const RESOURCE_INIT: u8 = 7;
pub const REPO: u8 = 8;
pub const TEST_ENV: u8 = 9;
pub const BUILD_SERVER: u8 = 10;
pub const STAGING_ENV: u8 = 11;
pub const STAGE_OWNER_B: u8 = 12;
pub const STAGE_OWNER_C: u8 = 13;
pub const A_FILE: u8 = 32;
pub const B_FILE: u8 = 33;
pub const C_FILE: u8 = 34;
pub const A_REVIEW_FILE: u8 = 35;
pub const B_REVIEW_FILE: u8 = 36;
pub const C_REVIEW_FILE: u8 = 37;
pub const TEST_FILE: u8 = 38;
pub const CI_FILE: u8 = 39;
pub const RELEASE_FILE: u8 = 40;
pub const A_VM: u8 = 64;
pub const B_VM: u8 = 65;
pub const C_VM: u8 = 66;
pub const REVIEW_VM: u8 = 67;
pub const TEST_VM: u8 = 68;
pub const CI_VM: u8 = 69;
pub const RELEASE_VM: u8 = 70;
pub const A_IO: u8 = 80;
pub const B_IO: u8 = 81;
pub const C_IO: u8 = 82;
pub const REVIEW_IO: u8 = 83;
pub const TEST_IO: u8 = 84;
pub const CI_IO: u8 = 85;
pub const RELEASE_IO: u8 = 86;
pub const A_DATA: u8 = 96;
pub const B_DATA: u8 = 97;
pub const C_DATA: u8 = 98;
pub const A_REVIEW_DATA: u8 = 99;
pub const B_REVIEW_DATA: u8 = 100;
pub const C_REVIEW_DATA: u8 = 101;
pub const TEST_DATA: u8 = 102;
pub const CI_DATA: u8 = 103;
pub const RELEASE_DATA: u8 = 104;
pub type RepoLoan<'r> = g::Msg<50, Endpoint<'r, REPO>>;
pub type TestLoan<'r> = g::Msg<51, Endpoint<'r, TEST_ENV>>;
pub type BuildLoan<'r> = g::Msg<52, Endpoint<'r, BUILD_SERVER>>;
pub type StageLoan<'r> = g::Msg<53, Endpoint<'r, STAGING_ENV>>;
pub type TaskExit = g::Msg<54, i32>;
pub type StageHistory<'a> = g::Msg<222, &'a [u8]>;
pub type StageCiPassedA<'a> = g::Msg<55, ReviewReply<'a, true>>;
pub type StageCiFailedA<'a> = g::Msg<56, ReviewReply<'a, false>>;
pub type StageCiApprovedA<'a> = g::Msg<57, ReviewReply<'a, true>>;
pub type StageCiRejectedA<'a> = g::Msg<58, ReviewReply<'a, false>>;
pub type StageCiPassedB<'a> = g::Msg<59, ReviewReply<'a, true>>;
pub type StageCiFailedB<'a> = g::Msg<60, ReviewReply<'a, false>>;
pub type StageCiApprovedB<'a> = g::Msg<61, ReviewReply<'a, true>>;
pub type StageCiRejectedB<'a> = g::Msg<62, ReviewReply<'a, false>>;
pub type StageCiPassedC<'a> = g::Msg<63, ReviewReply<'a, true>>;
pub type StageCiFailedC<'a> = g::Msg<64, ReviewReply<'a, false>>;
pub type StageCiApprovedC<'a> = g::Msg<65, ReviewReply<'a, true>>;
pub type StageCiRejectedC<'a> = g::Msg<66, ReviewReply<'a, false>>;
pub type InterfaceAToB<'a> = g::Msg<223, &'a [u8]>;
pub type InterfaceAToC<'a> = g::Msg<224, &'a [u8]>;
pub type InterfaceBToA<'a> = g::Msg<225, &'a [u8]>;
pub type InterfaceBToC<'a> = g::Msg<226, &'a [u8]>;
pub type InterfaceCToA<'a> = g::Msg<227, &'a [u8]>;
pub type InterfaceCToB<'a> = g::Msg<228, &'a [u8]>;

pub const REVIEW_DECISION_A: u16 = 900;
pub const CI_DECISION_A: u16 = 901;
pub const STAGE_DECISION_A: u16 = 902;
pub type ImplementA = g::Msg<100, Arguments<0>>;
pub type ImplementationA<'a> = g::Msg<101, &'a [u8]>;
pub type TestCandidateA<'a> = g::Msg<102, &'a [u8]>;
pub type FileTestCandidateA<'a> = g::Msg<103, &'a [u8]>;
pub type TestRequestA = g::Msg<104, Arguments<1>>;
pub type TestedA<'a> = g::Msg<105, &'a [u8]>;
pub type FileTestedA<'a> = g::Msg<106, &'a [u8]>;
pub type ReviewCandidateA<'a> = g::Msg<107, &'a [u8]>;
pub type FileReviewCandidateA<'a> = g::Msg<108, &'a [u8]>;
pub type ReviewRequestA = g::Msg<109, Arguments<2>>;
pub type ApprovedA<'a> = g::Msg<110, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<111, ReviewReply<'a, false>>;
pub type RepairApprovedA<'a> = g::Msg<112, ReviewReply<'a, true>>;
pub type RepairRejectedA<'a> = g::Msg<113, ReviewReply<'a, false>>;
pub type StageRepairApprovedA<'a> = g::Msg<114, ReviewReply<'a, true>>;
pub type StageRepairRejectedA<'a> = g::Msg<115, ReviewReply<'a, false>>;
pub type ReviewFeedbackA<'a> = g::Msg<116, &'a [u8]>;
pub type FileReviewResultA<'a> = g::Msg<117, &'a [u8]>;
pub type MergeRequestA = g::Msg<119, Arguments<3>>;
pub type MergedA<'a> = g::Msg<120, &'a [u8]>;
pub type PublicationA<'a> = g::Msg<121, &'a [u8]>;
pub type FilePublicationA<'a> = g::Msg<122, &'a [u8]>;
pub type CiRequestA = g::Msg<123, Arguments<4>>;
pub type CiPassedA<'a> = g::Msg<124, ReviewReply<'a, true>>;
pub type CiFailedA<'a> = g::Msg<125, ReviewReply<'a, false>>;
pub type CiUpdateA<'a> = g::Msg<126, &'a [u8]>;
pub type FileCiResultA<'a> = g::Msg<127, &'a [u8]>;
pub type StagingReadyA<'a> = g::Msg<128, &'a [u8]>;
pub type FileStagingReadyA<'a> = g::Msg<129, &'a [u8]>;
pub type DeployRequestA = g::Msg<130, Arguments<5>>;
pub type DeployedA<'a> = g::Msg<131, &'a [u8]>;
pub type ValidateRequestA = g::Msg<132, Arguments<6>>;
pub type StagePassedA<'a> = g::Msg<133, ReviewReply<'a, true>>;
pub type StageFailedA<'a> = g::Msg<134, ReviewReply<'a, false>>;
pub type FileStageResultA<'a> = g::Msg<135, &'a [u8]>;
pub type RollbackRequestA = g::Msg<136, Arguments<7>>;
pub type RolledBackA<'a> = g::Msg<137, &'a [u8]>;
pub type RollbackReceiptA<'a> = g::Msg<138, &'a [u8]>;
pub type StageUpdateA<'a> = g::Msg<139, &'a [u8]>;

pub const REVIEW_DECISION_B: u16 = 903;
pub const CI_DECISION_B: u16 = 904;
pub const STAGE_DECISION_B: u16 = 905;
pub type ImplementB = g::Msg<141, Arguments<8>>;
pub type ImplementationB<'a> = g::Msg<142, &'a [u8]>;
pub type TestCandidateB<'a> = g::Msg<143, &'a [u8]>;
pub type FileTestCandidateB<'a> = g::Msg<144, &'a [u8]>;
pub type TestRequestB = g::Msg<145, Arguments<9>>;
pub type TestedB<'a> = g::Msg<146, &'a [u8]>;
pub type FileTestedB<'a> = g::Msg<147, &'a [u8]>;
pub type ReviewCandidateB<'a> = g::Msg<148, &'a [u8]>;
pub type FileReviewCandidateB<'a> = g::Msg<149, &'a [u8]>;
pub type ReviewRequestB = g::Msg<150, Arguments<10>>;
pub type ApprovedB<'a> = g::Msg<151, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<152, ReviewReply<'a, false>>;
pub type RepairApprovedB<'a> = g::Msg<153, ReviewReply<'a, true>>;
pub type RepairRejectedB<'a> = g::Msg<154, ReviewReply<'a, false>>;
pub type StageRepairApprovedB<'a> = g::Msg<155, ReviewReply<'a, true>>;
pub type StageRepairRejectedB<'a> = g::Msg<156, ReviewReply<'a, false>>;
pub type ReviewFeedbackB<'a> = g::Msg<157, &'a [u8]>;
pub type FileReviewResultB<'a> = g::Msg<158, &'a [u8]>;
pub type MergeRequestB = g::Msg<160, Arguments<11>>;
pub type MergedB<'a> = g::Msg<161, &'a [u8]>;
pub type PublicationB<'a> = g::Msg<162, &'a [u8]>;
pub type FilePublicationB<'a> = g::Msg<163, &'a [u8]>;
pub type CiRequestB = g::Msg<164, Arguments<12>>;
pub type CiPassedB<'a> = g::Msg<165, ReviewReply<'a, true>>;
pub type CiFailedB<'a> = g::Msg<166, ReviewReply<'a, false>>;
pub type CiUpdateB<'a> = g::Msg<167, &'a [u8]>;
pub type FileCiResultB<'a> = g::Msg<168, &'a [u8]>;
pub type StagingReadyB<'a> = g::Msg<169, &'a [u8]>;
pub type FileStagingReadyB<'a> = g::Msg<170, &'a [u8]>;
pub type DeployRequestB = g::Msg<171, Arguments<13>>;
pub type DeployedB<'a> = g::Msg<172, &'a [u8]>;
pub type ValidateRequestB = g::Msg<173, Arguments<14>>;
pub type StagePassedB<'a> = g::Msg<174, ReviewReply<'a, true>>;
pub type StageFailedB<'a> = g::Msg<175, ReviewReply<'a, false>>;
pub type FileStageResultB<'a> = g::Msg<176, &'a [u8]>;
pub type RollbackRequestB = g::Msg<177, Arguments<15>>;
pub type RolledBackB<'a> = g::Msg<178, &'a [u8]>;
pub type RollbackReceiptB<'a> = g::Msg<179, &'a [u8]>;
pub type StageUpdateB<'a> = g::Msg<180, &'a [u8]>;

pub const REVIEW_DECISION_C: u16 = 906;
pub const CI_DECISION_C: u16 = 907;
pub const STAGE_DECISION_C: u16 = 908;
pub type ImplementC = g::Msg<182, Arguments<16>>;
pub type ImplementationC<'a> = g::Msg<183, &'a [u8]>;
pub type TestCandidateC<'a> = g::Msg<184, &'a [u8]>;
pub type FileTestCandidateC<'a> = g::Msg<185, &'a [u8]>;
pub type TestRequestC = g::Msg<186, Arguments<17>>;
pub type TestedC<'a> = g::Msg<187, &'a [u8]>;
pub type FileTestedC<'a> = g::Msg<188, &'a [u8]>;
pub type ReviewCandidateC<'a> = g::Msg<189, &'a [u8]>;
pub type FileReviewCandidateC<'a> = g::Msg<190, &'a [u8]>;
pub type ReviewRequestC = g::Msg<191, Arguments<18>>;
pub type ApprovedC<'a> = g::Msg<192, ReviewReply<'a, true>>;
pub type RejectedC<'a> = g::Msg<193, ReviewReply<'a, false>>;
pub type RepairApprovedC<'a> = g::Msg<194, ReviewReply<'a, true>>;
pub type RepairRejectedC<'a> = g::Msg<195, ReviewReply<'a, false>>;
pub type StageRepairApprovedC<'a> = g::Msg<196, ReviewReply<'a, true>>;
pub type StageRepairRejectedC<'a> = g::Msg<197, ReviewReply<'a, false>>;
pub type ReviewFeedbackC<'a> = g::Msg<198, &'a [u8]>;
pub type FileReviewResultC<'a> = g::Msg<199, &'a [u8]>;
pub type MergeRequestC = g::Msg<201, Arguments<19>>;
pub type MergedC<'a> = g::Msg<202, &'a [u8]>;
pub type PublicationC<'a> = g::Msg<203, &'a [u8]>;
pub type FilePublicationC<'a> = g::Msg<204, &'a [u8]>;
pub type CiRequestC = g::Msg<205, Arguments<20>>;
pub type CiPassedC<'a> = g::Msg<206, ReviewReply<'a, true>>;
pub type CiFailedC<'a> = g::Msg<207, ReviewReply<'a, false>>;
pub type CiUpdateC<'a> = g::Msg<208, &'a [u8]>;
pub type FileCiResultC<'a> = g::Msg<209, &'a [u8]>;
pub type StagingReadyC<'a> = g::Msg<210, &'a [u8]>;
pub type FileStagingReadyC<'a> = g::Msg<211, &'a [u8]>;
pub type DeployRequestC = g::Msg<212, Arguments<21>>;
pub type DeployedC<'a> = g::Msg<213, &'a [u8]>;
pub type ValidateRequestC = g::Msg<214, Arguments<22>>;
pub type StagePassedC<'a> = g::Msg<215, ReviewReply<'a, true>>;
pub type StageFailedC<'a> = g::Msg<216, ReviewReply<'a, false>>;
pub type FileStageResultC<'a> = g::Msg<217, &'a [u8]>;
pub type RollbackRequestC = g::Msg<218, Arguments<23>>;
pub type RolledBackC<'a> = g::Msg<219, &'a [u8]>;
pub type RollbackReceiptC<'a> = g::Msg<220, &'a [u8]>;
pub type StageUpdateC<'a> = g::Msg<221, &'a [u8]>;

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
pub const DEVELOPER_A_FILES: [ChoreoFsObject<'static>; 13] = [
    writable(b"service/implement_feature", 4),
    writable(b"service/merge_code", 5),
    readable(b"service/review/user_authentication", 6),
    readable(b"service/review/payment_processing", 7),
    readable(b"service/review/notification_system", 8),
    readable(b"service/integration/user_authentication", 9),
    readable(b"service/integration/payment_processing", 10),
    readable(b"service/integration/notification_system", 11),
    readable(b"service/release/user_authentication", 12),
    readable(b"service/release/payment_processing", 13),
    readable(b"service/release/notification_system", 14),
    readable(b"service/interface/payment_processing", 15),
    readable(b"service/interface/notification_system", 16),
];
pub const DEVELOPER_B_FILES: [ChoreoFsObject<'static>; 13] = [
    writable(b"service/implement_feature", 4),
    writable(b"service/merge_code", 5),
    readable(b"service/review/user_authentication", 6),
    readable(b"service/review/payment_processing", 7),
    readable(b"service/review/notification_system", 8),
    readable(b"service/integration/user_authentication", 9),
    readable(b"service/integration/payment_processing", 10),
    readable(b"service/integration/notification_system", 11),
    readable(b"service/release/user_authentication", 12),
    readable(b"service/release/payment_processing", 13),
    readable(b"service/release/notification_system", 14),
    readable(b"service/interface/user_authentication", 15),
    readable(b"service/interface/notification_system", 16),
];
pub const DEVELOPER_C_FILES: [ChoreoFsObject<'static>; 13] = [
    writable(b"service/implement_feature", 4),
    writable(b"service/merge_code", 5),
    readable(b"service/review/user_authentication", 6),
    readable(b"service/review/payment_processing", 7),
    readable(b"service/review/notification_system", 8),
    readable(b"service/integration/user_authentication", 9),
    readable(b"service/integration/payment_processing", 10),
    readable(b"service/integration/notification_system", 11),
    readable(b"service/release/user_authentication", 12),
    readable(b"service/release/payment_processing", 13),
    readable(b"service/release/notification_system", 14),
    readable(b"service/interface/user_authentication", 15),
    readable(b"service/interface/payment_processing", 16),
];
pub const REVIEW_FILES: [ChoreoFsObject<'static>; 12] = [
    writable(b"service/code_review/user_authentication", 4),
    writable(b"service/code_review/payment_processing", 5),
    writable(b"service/code_review/notification_system", 6),
    readable(b"service/candidate/user_authentication", 7),
    readable(b"service/candidate/payment_processing", 8),
    readable(b"service/candidate/notification_system", 9),
    readable(b"service/integration/user_authentication", 10),
    readable(b"service/integration/payment_processing", 11),
    readable(b"service/integration/notification_system", 12),
    readable(b"service/release/user_authentication", 13),
    readable(b"service/release/payment_processing", 14),
    readable(b"service/release/notification_system", 15),
];
pub const TEST_FILES: [ChoreoFsObject<'static>; 15] = [
    writable(b"service/write_tests/user_authentication", 4),
    writable(b"service/write_tests/payment_processing", 5),
    writable(b"service/write_tests/notification_system", 6),
    readable(b"service/candidate/user_authentication", 7),
    readable(b"service/candidate/payment_processing", 8),
    readable(b"service/candidate/notification_system", 9),
    readable(b"service/review/user_authentication", 10),
    readable(b"service/review/payment_processing", 11),
    readable(b"service/review/notification_system", 12),
    readable(b"service/integration/user_authentication", 13),
    readable(b"service/integration/payment_processing", 14),
    readable(b"service/integration/notification_system", 15),
    readable(b"service/release/user_authentication", 16),
    readable(b"service/release/payment_processing", 17),
    readable(b"service/release/notification_system", 18),
];
pub const CI_FILES: [ChoreoFsObject<'static>; 9] = [
    writable(b"service/run_tests/user_authentication", 4),
    writable(b"service/run_tests/payment_processing", 5),
    writable(b"service/run_tests/notification_system", 6),
    readable(b"service/publication/user_authentication", 7),
    readable(b"service/publication/payment_processing", 8),
    readable(b"service/publication/notification_system", 9),
    readable(b"service/release/user_authentication", 10),
    readable(b"service/release/payment_processing", 11),
    readable(b"service/release/notification_system", 12),
];
pub const RELEASE_FILES: [ChoreoFsObject<'static>; 12] = [
    writable(b"service/deploy/user_authentication", 4),
    writable(b"service/deploy/payment_processing", 5),
    writable(b"service/deploy/notification_system", 6),
    writable(b"service/validate/user_authentication", 7),
    writable(b"service/validate/payment_processing", 8),
    writable(b"service/validate/notification_system", 9),
    writable(b"service/rollback/user_authentication", 10),
    writable(b"service/rollback/payment_processing", 11),
    writable(b"service/rollback/notification_system", 12),
    readable(b"service/release/user_authentication", 13),
    readable(b"service/release/payment_processing", 14),
    readable(b"service/release/notification_system", 15),
];

fn attempt_a() -> impl Projectable + Copy {
    g::seq(
        g::send::<A_FILE, REPO, ImplementA>(),
        g::seq(
            g::send::<REPO, A_FILE, ImplementationA>(),
            g::par(
                g::seq(
                    g::send::<REPO, TEST_ENV, TestCandidateA>(),
                    g::seq(
                        g::send::<REPO, TEST_FILE, FileTestCandidateA>(),
                        g::seq(
                            g::send::<TEST_FILE, TEST_ENV, TestRequestA>(),
                            g::seq(
                                g::send::<TEST_ENV, REPO, TestedA>(),
                                g::seq(
                                    g::send::<TEST_ENV, TEST_FILE, FileTestedA>(),
                                    g::seq(
                                        g::send::<REPO, REVIEW_A, ReviewCandidateA>(),
                                        g::seq(
                                            g::send::<REPO, A_REVIEW_FILE, FileReviewCandidateA>(),
                                            g::send::<A_REVIEW_FILE, REVIEW_A, ReviewRequestA>(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
                g::par(
                    g::send::<REPO, B_FILE, InterfaceAToB>(),
                    g::send::<REPO, C_FILE, InterfaceAToC>(),
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
                    g::send::<REPO, C_FILE, ReviewFeedbackA>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReviewFeedbackA>(),
                        g::seq(
                            g::send::<REPO, TEST_FILE, ReviewFeedbackA>(),
                            g::send::<REVIEW_A, A_REVIEW_FILE, FileReviewResultA>(),
                        ),
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
        g::send::<A_FILE, REPO, MergeRequestA>(),
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
    )
}
fn ci_outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, CiUpdateA>(),
            g::seq(
                g::send::<REPO, B_FILE, CiUpdateA>(),
                g::seq(
                    g::send::<REPO, C_FILE, CiUpdateA>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, CiUpdateA>(),
                        g::seq(
                            g::send::<REPO, REVIEW_A, CiUpdateA>(),
                            g::seq(
                                g::send::<REPO, A_REVIEW_FILE, CiUpdateA>(),
                                g::seq(
                                    g::send::<REPO, TEST_FILE, CiUpdateA>(),
                                    g::send::<BUILD_SERVER, CI_FILE, FileCiResultA>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn integrate_a() -> impl Projectable {
    g::seq(
        merge_ci_a(),
        g::route(
            ci_outcome_a::<CiPassedA>(),
            g::seq(
                g::seq(
                    ci_outcome_a::<CiFailedA>(),
                    g::seq(
                        prepare_a::<RepairApprovedA, RepairRejectedA>(),
                        merge_ci_a(),
                    ),
                )
                .roll(),
                ci_outcome_a::<CiPassedA>(),
            ),
        )
        .resolve::<CI_DECISION_A>(),
    )
}
pub fn staging_a() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, STAGING_ENV, StagingReadyA>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStagingReadyA>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, DeployRequestA>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, DeployedA>(),
                    g::send::<RELEASE_FILE, STAGING_ENV, ValidateRequestA>(),
                ),
            ),
        ),
    )
}
fn stage_passed_a() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StagePassedA>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
            g::seq(
                g::send::<REPO, A_FILE, StageUpdateA>(),
                g::seq(
                    g::send::<REPO, B_FILE, StageUpdateA>(),
                    g::seq(
                        g::send::<REPO, C_FILE, StageUpdateA>(),
                        g::seq(
                            g::send::<REPO, TEST_ENV, StageUpdateA>(),
                            g::seq(
                                g::send::<REPO, REVIEW_A, StageUpdateA>(),
                                g::seq(
                                    g::send::<REPO, A_REVIEW_FILE, StageUpdateA>(),
                                    g::seq(
                                        g::send::<REPO, TEST_FILE, StageUpdateA>(),
                                        g::seq(
                                            g::send::<REPO, CI_FILE, StageUpdateA>(),
                                            g::send::<REPO, BUILD_SERVER, StageUpdateA>(),
                                        ),
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
fn stage_failed_a() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StageFailedA>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, RollbackRequestA>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, RolledBackA>(),
                    g::seq(
                        g::send::<STAGING_ENV, REPO, RollbackReceiptA>(),
                        g::seq(
                            g::send::<REPO, A_FILE, StageUpdateA>(),
                            g::seq(
                                g::send::<REPO, B_FILE, StageUpdateA>(),
                                g::seq(
                                    g::send::<REPO, C_FILE, StageUpdateA>(),
                                    g::seq(
                                        g::send::<REPO, TEST_ENV, StageUpdateA>(),
                                        g::seq(
                                            g::send::<REPO, REVIEW_A, StageUpdateA>(),
                                            g::seq(
                                                g::send::<REPO, A_REVIEW_FILE, StageUpdateA>(),
                                                g::seq(
                                                    g::send::<REPO, TEST_FILE, StageUpdateA>(),
                                                    g::seq(
                                                        g::send::<REPO, CI_FILE, StageUpdateA>(),
                                                        g::send::<REPO, BUILD_SERVER, StageUpdateA>(
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
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
fn integrate_stage_a() -> impl Projectable {
    g::seq(
        merge_ci_a(),
        g::route(
            ci_outcome_a::<StageCiPassedA>(),
            g::seq(
                g::seq(
                    ci_outcome_a::<StageCiFailedA>(),
                    g::seq(
                        prepare_a::<StageCiApprovedA, StageCiRejectedA>(),
                        merge_ci_a(),
                    ),
                )
                .roll(),
                ci_outcome_a::<StageCiPassedA>(),
            ),
        )
        .resolve::<CI_DECISION_A>(),
    )
}
fn release_a() -> impl Projectable {
    g::seq(
        integrate_a(),
        g::seq(
            staging_a(),
            g::route(
                stage_passed_a(),
                g::seq(
                    g::seq(
                        stage_failed_a(),
                        g::seq(
                            prepare_a::<StageRepairApprovedA, StageRepairRejectedA>(),
                            g::seq(integrate_stage_a(), staging_a()),
                        ),
                    )
                    .roll(),
                    stage_passed_a(),
                ),
            )
            .resolve::<STAGE_DECISION_A>(),
        ),
    )
}

fn attempt_b() -> impl Projectable + Copy {
    g::seq(
        g::send::<B_FILE, REPO, ImplementB>(),
        g::seq(
            g::send::<REPO, B_FILE, ImplementationB>(),
            g::par(
                g::seq(
                    g::send::<REPO, TEST_ENV, TestCandidateB>(),
                    g::seq(
                        g::send::<REPO, TEST_FILE, FileTestCandidateB>(),
                        g::seq(
                            g::send::<TEST_FILE, TEST_ENV, TestRequestB>(),
                            g::seq(
                                g::send::<TEST_ENV, REPO, TestedB>(),
                                g::seq(
                                    g::send::<TEST_ENV, TEST_FILE, FileTestedB>(),
                                    g::seq(
                                        g::send::<REPO, REVIEW_B, ReviewCandidateB>(),
                                        g::seq(
                                            g::send::<REPO, B_REVIEW_FILE, FileReviewCandidateB>(),
                                            g::send::<B_REVIEW_FILE, REVIEW_B, ReviewRequestB>(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
                g::par(
                    g::send::<REPO, A_FILE, InterfaceBToA>(),
                    g::send::<REPO, C_FILE, InterfaceBToC>(),
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
                    g::send::<REPO, C_FILE, ReviewFeedbackB>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReviewFeedbackB>(),
                        g::seq(
                            g::send::<REPO, TEST_FILE, ReviewFeedbackB>(),
                            g::send::<REVIEW_B, B_REVIEW_FILE, FileReviewResultB>(),
                        ),
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
        g::send::<B_FILE, REPO, MergeRequestB>(),
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
    )
}
fn ci_outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, CiUpdateB>(),
            g::seq(
                g::send::<REPO, B_FILE, CiUpdateB>(),
                g::seq(
                    g::send::<REPO, C_FILE, CiUpdateB>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, CiUpdateB>(),
                        g::seq(
                            g::send::<REPO, REVIEW_B, CiUpdateB>(),
                            g::seq(
                                g::send::<REPO, B_REVIEW_FILE, CiUpdateB>(),
                                g::seq(
                                    g::send::<REPO, TEST_FILE, CiUpdateB>(),
                                    g::send::<BUILD_SERVER, CI_FILE, FileCiResultB>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn integrate_b() -> impl Projectable {
    g::seq(
        merge_ci_b(),
        g::route(
            ci_outcome_b::<CiPassedB>(),
            g::seq(
                g::seq(
                    ci_outcome_b::<CiFailedB>(),
                    g::seq(
                        prepare_b::<RepairApprovedB, RepairRejectedB>(),
                        merge_ci_b(),
                    ),
                )
                .roll(),
                ci_outcome_b::<CiPassedB>(),
            ),
        )
        .resolve::<CI_DECISION_B>(),
    )
}
pub fn staging_b() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, STAGING_ENV, StagingReadyB>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStagingReadyB>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, DeployRequestB>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, DeployedB>(),
                    g::send::<RELEASE_FILE, STAGING_ENV, ValidateRequestB>(),
                ),
            ),
        ),
    )
}
fn stage_passed_b() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StagePassedB>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultB>(),
            g::seq(
                g::send::<REPO, A_FILE, StageUpdateB>(),
                g::seq(
                    g::send::<REPO, B_FILE, StageUpdateB>(),
                    g::seq(
                        g::send::<REPO, C_FILE, StageUpdateB>(),
                        g::seq(
                            g::send::<REPO, TEST_ENV, StageUpdateB>(),
                            g::seq(
                                g::send::<REPO, REVIEW_B, StageUpdateB>(),
                                g::seq(
                                    g::send::<REPO, B_REVIEW_FILE, StageUpdateB>(),
                                    g::seq(
                                        g::send::<REPO, TEST_FILE, StageUpdateB>(),
                                        g::seq(
                                            g::send::<REPO, CI_FILE, StageUpdateB>(),
                                            g::send::<REPO, BUILD_SERVER, StageUpdateB>(),
                                        ),
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
fn stage_failed_b() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StageFailedB>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultB>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, RollbackRequestB>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, RolledBackB>(),
                    g::seq(
                        g::send::<STAGING_ENV, REPO, RollbackReceiptB>(),
                        g::seq(
                            g::send::<REPO, A_FILE, StageUpdateB>(),
                            g::seq(
                                g::send::<REPO, B_FILE, StageUpdateB>(),
                                g::seq(
                                    g::send::<REPO, C_FILE, StageUpdateB>(),
                                    g::seq(
                                        g::send::<REPO, TEST_ENV, StageUpdateB>(),
                                        g::seq(
                                            g::send::<REPO, REVIEW_B, StageUpdateB>(),
                                            g::seq(
                                                g::send::<REPO, B_REVIEW_FILE, StageUpdateB>(),
                                                g::seq(
                                                    g::send::<REPO, TEST_FILE, StageUpdateB>(),
                                                    g::seq(
                                                        g::send::<REPO, CI_FILE, StageUpdateB>(),
                                                        g::send::<REPO, BUILD_SERVER, StageUpdateB>(
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
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
fn integrate_stage_b() -> impl Projectable {
    g::seq(
        merge_ci_b(),
        g::route(
            ci_outcome_b::<StageCiPassedB>(),
            g::seq(
                g::seq(
                    ci_outcome_b::<StageCiFailedB>(),
                    g::seq(
                        prepare_b::<StageCiApprovedB, StageCiRejectedB>(),
                        merge_ci_b(),
                    ),
                )
                .roll(),
                ci_outcome_b::<StageCiPassedB>(),
            ),
        )
        .resolve::<CI_DECISION_B>(),
    )
}
fn release_b() -> impl Projectable {
    g::seq(
        integrate_b(),
        g::seq(
            staging_b(),
            g::route(
                stage_passed_b(),
                g::seq(
                    g::seq(
                        stage_failed_b(),
                        g::seq(
                            prepare_b::<StageRepairApprovedB, StageRepairRejectedB>(),
                            g::seq(integrate_stage_b(), staging_b()),
                        ),
                    )
                    .roll(),
                    stage_passed_b(),
                ),
            )
            .resolve::<STAGE_DECISION_B>(),
        ),
    )
}

fn attempt_c() -> impl Projectable + Copy {
    g::seq(
        g::send::<C_FILE, REPO, ImplementC>(),
        g::seq(
            g::send::<REPO, C_FILE, ImplementationC>(),
            g::par(
                g::seq(
                    g::send::<REPO, TEST_ENV, TestCandidateC>(),
                    g::seq(
                        g::send::<REPO, TEST_FILE, FileTestCandidateC>(),
                        g::seq(
                            g::send::<TEST_FILE, TEST_ENV, TestRequestC>(),
                            g::seq(
                                g::send::<TEST_ENV, REPO, TestedC>(),
                                g::seq(
                                    g::send::<TEST_ENV, TEST_FILE, FileTestedC>(),
                                    g::seq(
                                        g::send::<REPO, REVIEW_C, ReviewCandidateC>(),
                                        g::seq(
                                            g::send::<REPO, C_REVIEW_FILE, FileReviewCandidateC>(),
                                            g::send::<C_REVIEW_FILE, REVIEW_C, ReviewRequestC>(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
                g::par(
                    g::send::<REPO, A_FILE, InterfaceCToA>(),
                    g::send::<REPO, B_FILE, InterfaceCToB>(),
                ),
            ),
        ),
    )
}
fn review_outcome_c<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_C, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, ReviewFeedbackC>(),
            g::seq(
                g::send::<REPO, B_FILE, ReviewFeedbackC>(),
                g::seq(
                    g::send::<REPO, C_FILE, ReviewFeedbackC>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, ReviewFeedbackC>(),
                        g::seq(
                            g::send::<REPO, TEST_FILE, ReviewFeedbackC>(),
                            g::send::<REVIEW_C, C_REVIEW_FILE, FileReviewResultC>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn prepare_c<Approved: g::Message, Rejected: g::Message>() -> impl Projectable {
    g::seq(
        attempt_c(),
        g::route(
            review_outcome_c::<Approved>(),
            g::seq(
                g::seq(review_outcome_c::<Rejected>(), attempt_c()).roll(),
                review_outcome_c::<Approved>(),
            ),
        )
        .resolve::<REVIEW_DECISION_C>(),
    )
}
fn merge_ci_c() -> impl Projectable {
    g::seq(
        g::send::<C_FILE, REPO, MergeRequestC>(),
        g::seq(
            g::send::<REPO, C_FILE, MergedC>(),
            g::seq(
                g::send::<REPO, BUILD_SERVER, PublicationC>(),
                g::seq(
                    g::send::<BUILD_SERVER, CI_FILE, FilePublicationC>(),
                    g::send::<CI_FILE, BUILD_SERVER, CiRequestC>(),
                ),
            ),
        ),
    )
}
fn ci_outcome_c<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, CiUpdateC>(),
            g::seq(
                g::send::<REPO, B_FILE, CiUpdateC>(),
                g::seq(
                    g::send::<REPO, C_FILE, CiUpdateC>(),
                    g::seq(
                        g::send::<REPO, TEST_ENV, CiUpdateC>(),
                        g::seq(
                            g::send::<REPO, REVIEW_C, CiUpdateC>(),
                            g::seq(
                                g::send::<REPO, C_REVIEW_FILE, CiUpdateC>(),
                                g::seq(
                                    g::send::<REPO, TEST_FILE, CiUpdateC>(),
                                    g::send::<BUILD_SERVER, CI_FILE, FileCiResultC>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn integrate_c() -> impl Projectable {
    g::seq(
        merge_ci_c(),
        g::route(
            ci_outcome_c::<CiPassedC>(),
            g::seq(
                g::seq(
                    ci_outcome_c::<CiFailedC>(),
                    g::seq(
                        prepare_c::<RepairApprovedC, RepairRejectedC>(),
                        merge_ci_c(),
                    ),
                )
                .roll(),
                ci_outcome_c::<CiPassedC>(),
            ),
        )
        .resolve::<CI_DECISION_C>(),
    )
}
pub fn staging_c() -> impl Projectable {
    g::seq(
        g::send::<BUILD_SERVER, STAGING_ENV, StagingReadyC>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStagingReadyC>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, DeployRequestC>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, DeployedC>(),
                    g::send::<RELEASE_FILE, STAGING_ENV, ValidateRequestC>(),
                ),
            ),
        ),
    )
}
fn stage_passed_c() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StagePassedC>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultC>(),
            g::seq(
                g::send::<REPO, A_FILE, StageUpdateC>(),
                g::seq(
                    g::send::<REPO, B_FILE, StageUpdateC>(),
                    g::seq(
                        g::send::<REPO, C_FILE, StageUpdateC>(),
                        g::seq(
                            g::send::<REPO, TEST_ENV, StageUpdateC>(),
                            g::seq(
                                g::send::<REPO, REVIEW_C, StageUpdateC>(),
                                g::seq(
                                    g::send::<REPO, C_REVIEW_FILE, StageUpdateC>(),
                                    g::seq(
                                        g::send::<REPO, TEST_FILE, StageUpdateC>(),
                                        g::seq(
                                            g::send::<REPO, CI_FILE, StageUpdateC>(),
                                            g::send::<REPO, BUILD_SERVER, StageUpdateC>(),
                                        ),
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
fn stage_failed_c() -> impl Projectable {
    g::seq(
        g::send::<STAGING_ENV, REPO, StageFailedC>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultC>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, RollbackRequestC>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, RolledBackC>(),
                    g::seq(
                        g::send::<STAGING_ENV, REPO, RollbackReceiptC>(),
                        g::seq(
                            g::send::<REPO, A_FILE, StageUpdateC>(),
                            g::seq(
                                g::send::<REPO, B_FILE, StageUpdateC>(),
                                g::seq(
                                    g::send::<REPO, C_FILE, StageUpdateC>(),
                                    g::seq(
                                        g::send::<REPO, TEST_ENV, StageUpdateC>(),
                                        g::seq(
                                            g::send::<REPO, REVIEW_C, StageUpdateC>(),
                                            g::seq(
                                                g::send::<REPO, C_REVIEW_FILE, StageUpdateC>(),
                                                g::seq(
                                                    g::send::<REPO, TEST_FILE, StageUpdateC>(),
                                                    g::seq(
                                                        g::send::<REPO, CI_FILE, StageUpdateC>(),
                                                        g::send::<REPO, BUILD_SERVER, StageUpdateC>(
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
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
fn integrate_stage_c() -> impl Projectable {
    g::seq(
        merge_ci_c(),
        g::route(
            ci_outcome_c::<StageCiPassedC>(),
            g::seq(
                g::seq(
                    ci_outcome_c::<StageCiFailedC>(),
                    g::seq(
                        prepare_c::<StageCiApprovedC, StageCiRejectedC>(),
                        merge_ci_c(),
                    ),
                )
                .roll(),
                ci_outcome_c::<StageCiPassedC>(),
            ),
        )
        .resolve::<CI_DECISION_C>(),
    )
}
fn release_c() -> impl Projectable {
    g::seq(
        integrate_c(),
        g::seq(
            staging_c(),
            g::route(
                stage_passed_c(),
                g::seq(
                    g::seq(
                        stage_failed_c(),
                        g::seq(
                            prepare_c::<StageRepairApprovedC, StageRepairRejectedC>(),
                            g::seq(integrate_stage_c(), staging_c()),
                        ),
                    )
                    .roll(),
                    stage_passed_c(),
                ),
            )
            .resolve::<STAGE_DECISION_C>(),
        ),
    )
}

pub fn source() -> impl Projectable {
    let resources = g::par(
        g::send::<RESOURCE_INIT, REPO_OWNER, RepoLoan>(),
        g::par(
            g::send::<RESOURCE_INIT, TEST_OWNER, TestLoan>(),
            g::par(
                g::send::<RESOURCE_INIT, CI_OWNER, BuildLoan>(),
                g::send::<RESOURCE_INIT, STAGE_OWNER, StageLoan>(),
            ),
        ),
    );
    let files = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::par(
                g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
                g::par(
                    g::send::<A_REVIEW_DATA, REVIEW_VM, Loan<A_REVIEW_FILE>>(),
                    g::par(
                        g::send::<B_REVIEW_DATA, REVIEW_VM, Loan<B_REVIEW_FILE>>(),
                        g::par(
                            g::send::<C_REVIEW_DATA, REVIEW_VM, Loan<C_REVIEW_FILE>>(),
                            g::par(
                                g::send::<TEST_DATA, TEST_VM, Loan<TEST_FILE>>(),
                                g::par(
                                    g::send::<CI_DATA, CI_VM, Loan<CI_FILE>>(),
                                    g::send::<RELEASE_DATA, RELEASE_VM, Loan<RELEASE_FILE>>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let task = g::seq(
        g::par(
            g::seq(
                prepare_a::<ApprovedA, RejectedA>(),
                g::seq(
                    release_a(),
                    g::seq(
                        g::send::<STAGE_OWNER, STAGE_OWNER_B, StageHistory>(),
                        g::send::<STAGE_OWNER, STAGE_OWNER_B, StageLoan>(),
                    ),
                ),
            ),
            g::par(
                prepare_b::<ApprovedB, RejectedB>(),
                prepare_c::<ApprovedC, RejectedC>(),
            ),
        ),
        g::seq(
            g::seq(
                release_b(),
                g::seq(
                    g::send::<STAGE_OWNER_B, STAGE_OWNER_C, StageHistory>(),
                    g::seq(
                        g::send::<STAGE_OWNER_B, STAGE_OWNER_C, StageLoan>(),
                        release_c(),
                    ),
                ),
            ),
            g::seq(
                g::send::<STAGING_ENV, TEST_ENV, TaskExit>(),
                g::seq(
                    g::send::<STAGING_ENV, BUILD_SERVER, TaskExit>(),
                    g::seq(
                        g::send::<STAGING_ENV, REPO, TaskExit>(),
                        g::seq(
                            g::send::<STAGING_ENV, REVIEW_A, TaskExit>(),
                            g::seq(
                                g::send::<STAGING_ENV, REVIEW_B, TaskExit>(),
                                g::send::<STAGING_ENV, REVIEW_C, TaskExit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(g::seq(crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(), g::send::<A_FILE, A_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(), g::send::<B_FILE, B_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<C_FILE, C_DATA>().roll(), g::send::<C_FILE, C_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<A_REVIEW_FILE, A_REVIEW_DATA>().roll(), g::send::<A_REVIEW_FILE, A_REVIEW_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<B_REVIEW_FILE, B_REVIEW_DATA>().roll(), g::send::<B_REVIEW_FILE, B_REVIEW_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<C_REVIEW_FILE, C_REVIEW_DATA>().roll(), g::send::<C_REVIEW_FILE, C_REVIEW_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<TEST_FILE, TEST_DATA>().roll(), g::send::<TEST_FILE, TEST_DATA, FileExit>()), g::par(g::seq(crate::shell::global::file_metadata::<CI_FILE, CI_DATA>().roll(), g::send::<CI_FILE, CI_DATA, FileExit>()), g::seq(crate::shell::global::file_metadata::<RELEASE_FILE, RELEASE_DATA>().roll(), g::send::<RELEASE_FILE, RELEASE_DATA, FileExit>())))))))));
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
                    crate::shell::global::private_io::<C_VM, C_IO>(),
                    g::send::<C_VM, C_IO, Exit>(),
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
                        g::par(
                            g::seq(
                                crate::shell::global::private_io::<CI_VM, CI_IO>(),
                                g::send::<CI_VM, CI_IO, Exit>(),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<RELEASE_VM, RELEASE_IO>(),
                                g::send::<RELEASE_VM, RELEASE_IO, Exit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(resources, g::seq(files, g::par(task, g::par(metadata, io))))
}

pub struct Programs {
    pub repo_owner: RoleProgram<REPO_OWNER>,
    pub test_owner: RoleProgram<TEST_OWNER>,
    pub ci_owner: RoleProgram<CI_OWNER>,
    pub stage_owner: RoleProgram<STAGE_OWNER>,
    pub stage_owner_b: RoleProgram<STAGE_OWNER_B>,
    pub stage_owner_c: RoleProgram<STAGE_OWNER_C>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub review_c: RoleProgram<REVIEW_C>,
    pub resource_init: RoleProgram<RESOURCE_INIT>,
    pub repo: RoleProgram<REPO>,
    pub environment: RoleProgram<TEST_ENV>,
    pub build: RoleProgram<BUILD_SERVER>,
    pub staging: RoleProgram<STAGING_ENV>,
    pub a_file: RoleProgram<A_FILE>,
    pub b_file: RoleProgram<B_FILE>,
    pub c_file: RoleProgram<C_FILE>,
    pub a_review_file: RoleProgram<A_REVIEW_FILE>,
    pub b_review_file: RoleProgram<B_REVIEW_FILE>,
    pub c_review_file: RoleProgram<C_REVIEW_FILE>,
    pub test_file: RoleProgram<TEST_FILE>,
    pub ci_file: RoleProgram<CI_FILE>,
    pub release_file: RoleProgram<RELEASE_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub c_vm: RoleProgram<C_VM>,
    pub review_vm: RoleProgram<REVIEW_VM>,
    pub test_vm: RoleProgram<TEST_VM>,
    pub ci_vm: RoleProgram<CI_VM>,
    pub release_vm: RoleProgram<RELEASE_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_data: RoleProgram<B_DATA>,
    pub c_data: RoleProgram<C_DATA>,
    pub a_review_data: RoleProgram<A_REVIEW_DATA>,
    pub b_review_data: RoleProgram<B_REVIEW_DATA>,
    pub c_review_data: RoleProgram<C_REVIEW_DATA>,
    pub test_data: RoleProgram<TEST_DATA>,
    pub ci_data: RoleProgram<CI_DATA>,
    pub release_data: RoleProgram<RELEASE_DATA>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub c_io: RoleProgram<C_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
    pub test_io: RoleProgram<TEST_IO>,
    pub ci_io: RoleProgram<CI_IO>,
    pub release_io: RoleProgram<RELEASE_IO>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("RepoLoan", RepoLoan::LOGICAL_LABEL),
            ("TestLoan", TestLoan::LOGICAL_LABEL),
            ("BuildLoan", BuildLoan::LOGICAL_LABEL),
            ("StageLoan", StageLoan::LOGICAL_LABEL),
            ("TaskExit", TaskExit::LOGICAL_LABEL),
            ("StageHistory", StageHistory::LOGICAL_LABEL),
            ("StageCiPassedA", StageCiPassedA::LOGICAL_LABEL),
            ("StageCiFailedA", StageCiFailedA::LOGICAL_LABEL),
            ("StageCiApprovedA", StageCiApprovedA::LOGICAL_LABEL),
            ("StageCiRejectedA", StageCiRejectedA::LOGICAL_LABEL),
            ("StageCiPassedB", StageCiPassedB::LOGICAL_LABEL),
            ("StageCiFailedB", StageCiFailedB::LOGICAL_LABEL),
            ("StageCiApprovedB", StageCiApprovedB::LOGICAL_LABEL),
            ("StageCiRejectedB", StageCiRejectedB::LOGICAL_LABEL),
            ("StageCiPassedC", StageCiPassedC::LOGICAL_LABEL),
            ("StageCiFailedC", StageCiFailedC::LOGICAL_LABEL),
            ("StageCiApprovedC", StageCiApprovedC::LOGICAL_LABEL),
            ("StageCiRejectedC", StageCiRejectedC::LOGICAL_LABEL),
            ("InterfaceAToB", InterfaceAToB::LOGICAL_LABEL),
            ("InterfaceAToC", InterfaceAToC::LOGICAL_LABEL),
            ("InterfaceBToA", InterfaceBToA::LOGICAL_LABEL),
            ("InterfaceBToC", InterfaceBToC::LOGICAL_LABEL),
            ("InterfaceCToA", InterfaceCToA::LOGICAL_LABEL),
            ("InterfaceCToB", InterfaceCToB::LOGICAL_LABEL),
            ("ImplementA", ImplementA::LOGICAL_LABEL),
            ("ImplementationA", ImplementationA::LOGICAL_LABEL),
            ("TestCandidateA", TestCandidateA::LOGICAL_LABEL),
            ("FileTestCandidateA", FileTestCandidateA::LOGICAL_LABEL),
            ("TestRequestA", TestRequestA::LOGICAL_LABEL),
            ("TestedA", TestedA::LOGICAL_LABEL),
            ("FileTestedA", FileTestedA::LOGICAL_LABEL),
            ("ReviewCandidateA", ReviewCandidateA::LOGICAL_LABEL),
            ("FileReviewCandidateA", FileReviewCandidateA::LOGICAL_LABEL),
            ("ReviewRequestA", ReviewRequestA::LOGICAL_LABEL),
            ("ApprovedA", ApprovedA::LOGICAL_LABEL),
            ("RejectedA", RejectedA::LOGICAL_LABEL),
            ("RepairApprovedA", RepairApprovedA::LOGICAL_LABEL),
            ("RepairRejectedA", RepairRejectedA::LOGICAL_LABEL),
            ("StageRepairApprovedA", StageRepairApprovedA::LOGICAL_LABEL),
            ("StageRepairRejectedA", StageRepairRejectedA::LOGICAL_LABEL),
            ("ReviewFeedbackA", ReviewFeedbackA::LOGICAL_LABEL),
            ("FileReviewResultA", FileReviewResultA::LOGICAL_LABEL),
            ("MergeRequestA", MergeRequestA::LOGICAL_LABEL),
            ("MergedA", MergedA::LOGICAL_LABEL),
            ("PublicationA", PublicationA::LOGICAL_LABEL),
            ("FilePublicationA", FilePublicationA::LOGICAL_LABEL),
            ("CiRequestA", CiRequestA::LOGICAL_LABEL),
            ("CiPassedA", CiPassedA::LOGICAL_LABEL),
            ("CiFailedA", CiFailedA::LOGICAL_LABEL),
            ("CiUpdateA", CiUpdateA::LOGICAL_LABEL),
            ("FileCiResultA", FileCiResultA::LOGICAL_LABEL),
            ("StagingReadyA", StagingReadyA::LOGICAL_LABEL),
            ("FileStagingReadyA", FileStagingReadyA::LOGICAL_LABEL),
            ("DeployRequestA", DeployRequestA::LOGICAL_LABEL),
            ("DeployedA", DeployedA::LOGICAL_LABEL),
            ("ValidateRequestA", ValidateRequestA::LOGICAL_LABEL),
            ("StagePassedA", StagePassedA::LOGICAL_LABEL),
            ("StageFailedA", StageFailedA::LOGICAL_LABEL),
            ("FileStageResultA", FileStageResultA::LOGICAL_LABEL),
            ("RollbackRequestA", RollbackRequestA::LOGICAL_LABEL),
            ("RolledBackA", RolledBackA::LOGICAL_LABEL),
            ("RollbackReceiptA", RollbackReceiptA::LOGICAL_LABEL),
            ("StageUpdateA", StageUpdateA::LOGICAL_LABEL),
            ("ImplementB", ImplementB::LOGICAL_LABEL),
            ("ImplementationB", ImplementationB::LOGICAL_LABEL),
            ("TestCandidateB", TestCandidateB::LOGICAL_LABEL),
            ("FileTestCandidateB", FileTestCandidateB::LOGICAL_LABEL),
            ("TestRequestB", TestRequestB::LOGICAL_LABEL),
            ("TestedB", TestedB::LOGICAL_LABEL),
            ("FileTestedB", FileTestedB::LOGICAL_LABEL),
            ("ReviewCandidateB", ReviewCandidateB::LOGICAL_LABEL),
            ("FileReviewCandidateB", FileReviewCandidateB::LOGICAL_LABEL),
            ("ReviewRequestB", ReviewRequestB::LOGICAL_LABEL),
            ("ApprovedB", ApprovedB::LOGICAL_LABEL),
            ("RejectedB", RejectedB::LOGICAL_LABEL),
            ("RepairApprovedB", RepairApprovedB::LOGICAL_LABEL),
            ("RepairRejectedB", RepairRejectedB::LOGICAL_LABEL),
            ("StageRepairApprovedB", StageRepairApprovedB::LOGICAL_LABEL),
            ("StageRepairRejectedB", StageRepairRejectedB::LOGICAL_LABEL),
            ("ReviewFeedbackB", ReviewFeedbackB::LOGICAL_LABEL),
            ("FileReviewResultB", FileReviewResultB::LOGICAL_LABEL),
            ("MergeRequestB", MergeRequestB::LOGICAL_LABEL),
            ("MergedB", MergedB::LOGICAL_LABEL),
            ("PublicationB", PublicationB::LOGICAL_LABEL),
            ("FilePublicationB", FilePublicationB::LOGICAL_LABEL),
            ("CiRequestB", CiRequestB::LOGICAL_LABEL),
            ("CiPassedB", CiPassedB::LOGICAL_LABEL),
            ("CiFailedB", CiFailedB::LOGICAL_LABEL),
            ("CiUpdateB", CiUpdateB::LOGICAL_LABEL),
            ("FileCiResultB", FileCiResultB::LOGICAL_LABEL),
            ("StagingReadyB", StagingReadyB::LOGICAL_LABEL),
            ("FileStagingReadyB", FileStagingReadyB::LOGICAL_LABEL),
            ("DeployRequestB", DeployRequestB::LOGICAL_LABEL),
            ("DeployedB", DeployedB::LOGICAL_LABEL),
            ("ValidateRequestB", ValidateRequestB::LOGICAL_LABEL),
            ("StagePassedB", StagePassedB::LOGICAL_LABEL),
            ("StageFailedB", StageFailedB::LOGICAL_LABEL),
            ("FileStageResultB", FileStageResultB::LOGICAL_LABEL),
            ("RollbackRequestB", RollbackRequestB::LOGICAL_LABEL),
            ("RolledBackB", RolledBackB::LOGICAL_LABEL),
            ("RollbackReceiptB", RollbackReceiptB::LOGICAL_LABEL),
            ("StageUpdateB", StageUpdateB::LOGICAL_LABEL),
            ("ImplementC", ImplementC::LOGICAL_LABEL),
            ("ImplementationC", ImplementationC::LOGICAL_LABEL),
            ("TestCandidateC", TestCandidateC::LOGICAL_LABEL),
            ("FileTestCandidateC", FileTestCandidateC::LOGICAL_LABEL),
            ("TestRequestC", TestRequestC::LOGICAL_LABEL),
            ("TestedC", TestedC::LOGICAL_LABEL),
            ("FileTestedC", FileTestedC::LOGICAL_LABEL),
            ("ReviewCandidateC", ReviewCandidateC::LOGICAL_LABEL),
            ("FileReviewCandidateC", FileReviewCandidateC::LOGICAL_LABEL),
            ("ReviewRequestC", ReviewRequestC::LOGICAL_LABEL),
            ("ApprovedC", ApprovedC::LOGICAL_LABEL),
            ("RejectedC", RejectedC::LOGICAL_LABEL),
            ("RepairApprovedC", RepairApprovedC::LOGICAL_LABEL),
            ("RepairRejectedC", RepairRejectedC::LOGICAL_LABEL),
            ("StageRepairApprovedC", StageRepairApprovedC::LOGICAL_LABEL),
            ("StageRepairRejectedC", StageRepairRejectedC::LOGICAL_LABEL),
            ("ReviewFeedbackC", ReviewFeedbackC::LOGICAL_LABEL),
            ("FileReviewResultC", FileReviewResultC::LOGICAL_LABEL),
            ("MergeRequestC", MergeRequestC::LOGICAL_LABEL),
            ("MergedC", MergedC::LOGICAL_LABEL),
            ("PublicationC", PublicationC::LOGICAL_LABEL),
            ("FilePublicationC", FilePublicationC::LOGICAL_LABEL),
            ("CiRequestC", CiRequestC::LOGICAL_LABEL),
            ("CiPassedC", CiPassedC::LOGICAL_LABEL),
            ("CiFailedC", CiFailedC::LOGICAL_LABEL),
            ("CiUpdateC", CiUpdateC::LOGICAL_LABEL),
            ("FileCiResultC", FileCiResultC::LOGICAL_LABEL),
            ("StagingReadyC", StagingReadyC::LOGICAL_LABEL),
            ("FileStagingReadyC", FileStagingReadyC::LOGICAL_LABEL),
            ("DeployRequestC", DeployRequestC::LOGICAL_LABEL),
            ("DeployedC", DeployedC::LOGICAL_LABEL),
            ("ValidateRequestC", ValidateRequestC::LOGICAL_LABEL),
            ("StagePassedC", StagePassedC::LOGICAL_LABEL),
            ("StageFailedC", StageFailedC::LOGICAL_LABEL),
            ("FileStageResultC", FileStageResultC::LOGICAL_LABEL),
            ("RollbackRequestC", RollbackRequestC::LOGICAL_LABEL),
            ("RolledBackC", RolledBackC::LOGICAL_LABEL),
            ("RollbackReceiptC", RollbackReceiptC::LOGICAL_LABEL),
            ("StageUpdateC", StageUpdateC::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        let roles: serde_json::Map<String, serde_json::Value> = [
            ("REPO_OWNER", REPO_OWNER),
            ("TEST_OWNER", TEST_OWNER),
            ("CI_OWNER", CI_OWNER),
            ("STAGE_OWNER", STAGE_OWNER),
            ("STAGE_OWNER_B", STAGE_OWNER_B),
            ("STAGE_OWNER_C", STAGE_OWNER_C),
            ("REVIEW_A", REVIEW_A),
            ("REVIEW_B", REVIEW_B),
            ("REVIEW_C", REVIEW_C),
            ("RESOURCE_INIT", RESOURCE_INIT),
            ("REPO", REPO),
            ("TEST_ENV", TEST_ENV),
            ("BUILD_SERVER", BUILD_SERVER),
            ("STAGING_ENV", STAGING_ENV),
            ("A_FILE", A_FILE),
            ("B_FILE", B_FILE),
            ("C_FILE", C_FILE),
            ("A_REVIEW_FILE", A_REVIEW_FILE),
            ("B_REVIEW_FILE", B_REVIEW_FILE),
            ("C_REVIEW_FILE", C_REVIEW_FILE),
            ("TEST_FILE", TEST_FILE),
            ("CI_FILE", CI_FILE),
            ("RELEASE_FILE", RELEASE_FILE),
            ("A_VM", A_VM),
            ("B_VM", B_VM),
            ("C_VM", C_VM),
            ("REVIEW_VM", REVIEW_VM),
            ("TEST_VM", TEST_VM),
            ("CI_VM", CI_VM),
            ("RELEASE_VM", RELEASE_VM),
            ("A_DATA", A_DATA),
            ("B_DATA", B_DATA),
            ("C_DATA", C_DATA),
            ("A_REVIEW_DATA", A_REVIEW_DATA),
            ("B_REVIEW_DATA", B_REVIEW_DATA),
            ("C_REVIEW_DATA", C_REVIEW_DATA),
            ("TEST_DATA", TEST_DATA),
            ("CI_DATA", CI_DATA),
            ("RELEASE_DATA", RELEASE_DATA),
            ("A_IO", A_IO),
            ("B_IO", B_IO),
            ("C_IO", C_IO),
            ("REVIEW_IO", REVIEW_IO),
            ("TEST_IO", TEST_IO),
            ("CI_IO", CI_IO),
            ("RELEASE_IO", RELEASE_IO),
        ]
        .into_iter()
        .map(|(name, role)| (name.into(), role.into()))
        .collect();
        let programs = [
            rows(&self.repo_owner),
            rows(&self.test_owner),
            rows(&self.ci_owner),
            rows(&self.stage_owner),
            rows(&self.stage_owner_b),
            rows(&self.stage_owner_c),
            rows(&self.review_a),
            rows(&self.review_b),
            rows(&self.review_c),
            rows(&self.resource_init),
            rows(&self.repo),
            rows(&self.environment),
            rows(&self.build),
            rows(&self.staging),
            rows(&self.a_file),
            rows(&self.b_file),
            rows(&self.c_file),
            rows(&self.a_review_file),
            rows(&self.b_review_file),
            rows(&self.c_review_file),
            rows(&self.test_file),
            rows(&self.ci_file),
            rows(&self.release_file),
            rows(&self.a_vm),
            rows(&self.b_vm),
            rows(&self.c_vm),
            rows(&self.review_vm),
            rows(&self.test_vm),
            rows(&self.ci_vm),
            rows(&self.release_vm),
            rows(&self.a_data),
            rows(&self.b_data),
            rows(&self.c_data),
            rows(&self.a_review_data),
            rows(&self.b_review_data),
            rows(&self.c_review_data),
            rows(&self.test_data),
            rows(&self.ci_data),
            rows(&self.release_data),
            rows(&self.a_io),
            rows(&self.b_io),
            rows(&self.c_io),
            rows(&self.review_io),
            rows(&self.test_io),
            rows(&self.ci_io),
            rows(&self.release_io),
        ];
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"6H","session":SESSION,
            "labels":labels,"roles":roles,"programs":programs.as_slice()})
    }
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        repo_owner: project(&p),
        test_owner: project(&p),
        ci_owner: project(&p),
        stage_owner: project(&p),
        stage_owner_b: project(&p),
        stage_owner_c: project(&p),
        review_a: project(&p),
        review_b: project(&p),
        review_c: project(&p),
        resource_init: project(&p),
        repo: project(&p),
        environment: project(&p),
        build: project(&p),
        staging: project(&p),
        a_file: project(&p),
        b_file: project(&p),
        c_file: project(&p),
        a_review_file: project(&p),
        b_review_file: project(&p),
        c_review_file: project(&p),
        test_file: project(&p),
        ci_file: project(&p),
        release_file: project(&p),
        a_vm: project(&p),
        b_vm: project(&p),
        c_vm: project(&p),
        review_vm: project(&p),
        test_vm: project(&p),
        ci_vm: project(&p),
        release_vm: project(&p),
        a_data: project(&p),
        b_data: project(&p),
        c_data: project(&p),
        a_review_data: project(&p),
        b_review_data: project(&p),
        c_review_data: project(&p),
        test_data: project(&p),
        ci_data: project(&p),
        release_data: project(&p),
        a_io: project(&p),
        b_io: project(&p),
        c_io: project(&p),
        review_io: project(&p),
        test_io: project(&p),
        ci_io: project(&p),
        release_io: project(&p),
    }
}
