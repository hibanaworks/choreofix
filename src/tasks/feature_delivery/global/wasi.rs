//! One business choreography composed with five independent WASI I/O branches.
use super::{delivery::*, *};
use hibana::{
    g::Message,
    runtime::program::{RoleProgram, project},
};

pub const A_IO: u8 = 16;
pub const B_IO: u8 = 17;
pub const WRITER_IO: u8 = 18;
pub const REVIEWER_IO: u8 = 19;
pub const RUNNER_IO: u8 = 20;

/// Immutable images for the same source, not runtime permission or progress.
pub struct Programs {
    pub a: RoleProgram<DEVELOPER_A>,
    pub b: RoleProgram<DEVELOPER_B>,
    pub writer: RoleProgram<TEST_WRITER>,
    pub reviewer: RoleProgram<REVIEWER>,
    pub runner: RoleProgram<CI_RUNNER>,
    pub repository: RoleProgram<REPOSITORY>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub data_a: RoleProgram<TEST_DATA_A>,
    pub data_b: RoleProgram<TEST_DATA_B>,
    pub ci: RoleProgram<CI_SERVICE>,
    pub repo: RoleProgram<REPO>,
    pub environment: RoleProgram<TEST_ENV>,
    pub build: RoleProgram<BUILD_SERVER>,
    pub feature_a: RoleProgram<AUTH_A>,
    pub feature_b: RoleProgram<AUTH_B>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub writer_io: RoleProgram<WRITER_IO>,
    pub reviewer_io: RoleProgram<REVIEWER_IO>,
    pub runner_io: RoleProgram<RUNNER_IO>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("ImplementA", ImplementA::LOGICAL_LABEL),
            ("ImplementB", ImplementB::LOGICAL_LABEL),
            ("Implement", Implement::LOGICAL_LABEL),
            ("ImplementOnRepo", ImplementOnRepo::LOGICAL_LABEL),
            ("RepoResult", RepoResult::LOGICAL_LABEL),
            ("RepoLoan", RepoLoan::LOGICAL_LABEL),
            ("RepoReturn", RepoReturn::LOGICAL_LABEL),
            ("CandidateForReviewA", CandidateForReviewA::LOGICAL_LABEL),
            ("CandidateForReviewB", CandidateForReviewB::LOGICAL_LABEL),
            ("CandidateForTestsA", CandidateForTestsA::LOGICAL_LABEL),
            ("CandidateForTestsB", CandidateForTestsB::LOGICAL_LABEL),
            ("TestIntentA", TestIntentA::LOGICAL_LABEL),
            ("TestIntentB", TestIntentB::LOGICAL_LABEL),
            ("TestRequestA", TestRequestA::LOGICAL_LABEL),
            ("TestRequestB", TestRequestB::LOGICAL_LABEL),
            ("TestResultA", TestResultA::LOGICAL_LABEL),
            ("TestResultB", TestResultB::LOGICAL_LABEL),
            ("TestedA", TestedA::LOGICAL_LABEL),
            ("TestedB", TestedB::LOGICAL_LABEL),
            ("TestsForCiA", TestsForCiA::LOGICAL_LABEL),
            ("TestsForCiB", TestsForCiB::LOGICAL_LABEL),
            ("ReviewFirstA", ReviewFirstA::LOGICAL_LABEL),
            ("ReviewFirstB", ReviewFirstB::LOGICAL_LABEL),
            ("ReviewRequestA", ReviewRequestA::LOGICAL_LABEL),
            ("ReviewRequestB", ReviewRequestB::LOGICAL_LABEL),
            ("Review", Review::LOGICAL_LABEL),
            ("ReviewResultA", ReviewResultA::LOGICAL_LABEL),
            ("ReviewResultB", ReviewResultB::LOGICAL_LABEL),
            ("Approved", Approved::LOGICAL_LABEL),
            ("Rejected", Rejected::LOGICAL_LABEL),
            ("ReviewApproved", ReviewApproved::LOGICAL_LABEL),
            ("ReviewRejected", ReviewRejected::LOGICAL_LABEL),
            ("RecoveryApproved", RecoveryApproved::LOGICAL_LABEL),
            ("RecoveryRejected", RecoveryRejected::LOGICAL_LABEL),
            (
                "RecoveryReviewApproved",
                RecoveryReviewApproved::LOGICAL_LABEL,
            ),
            (
                "RecoveryReviewRejected",
                RecoveryReviewRejected::LOGICAL_LABEL,
            ),
            ("MergeAdmissionA", MergeAdmissionA::LOGICAL_LABEL),
            ("MergeAdmissionB", MergeAdmissionB::LOGICAL_LABEL),
            ("Merge", Merge::LOGICAL_LABEL),
            ("MergeOnRepo", MergeOnRepo::LOGICAL_LABEL),
            ("Merged", Merged::LOGICAL_LABEL),
            ("CiIntentA", CiIntentA::LOGICAL_LABEL),
            ("CiIntentB", CiIntentB::LOGICAL_LABEL),
            ("RunCiA", RunCiA::LOGICAL_LABEL),
            ("RunCiB", RunCiB::LOGICAL_LABEL),
            ("RunCiOnFeatureA", RunCiOnFeatureA::LOGICAL_LABEL),
            ("RunCiOnFeatureB", RunCiOnFeatureB::LOGICAL_LABEL),
            ("CiResult", CiResult::LOGICAL_LABEL),
            ("CiPassedA", CiPassedA::LOGICAL_LABEL),
            ("CiFailedA", CiFailedA::LOGICAL_LABEL),
            ("CiPassedB", CiPassedB::LOGICAL_LABEL),
            ("CiFailedB", CiFailedB::LOGICAL_LABEL),
            ("LastCiPassedA", LastCiPassedA::LOGICAL_LABEL),
            ("LastCiPassedB", LastCiPassedB::LOGICAL_LABEL),
            ("IntegratedA", IntegratedA::LOGICAL_LABEL),
            ("IntegrationFailedA", IntegrationFailedA::LOGICAL_LABEL),
            ("IntegratedB", IntegratedB::LOGICAL_LABEL),
            ("IntegrationFailedB", IntegrationFailedB::LOGICAL_LABEL),
            ("InterfaceChangedA", InterfaceChangedA::LOGICAL_LABEL),
            ("InterfaceChangedB", InterfaceChangedB::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"6M","session":SESSION,
 "labels":labels,
 "roles":{"DEVELOPER_A":DEVELOPER_A,"DEVELOPER_B":DEVELOPER_B,"TEST_WRITER":TEST_WRITER,"REVIEWER":REVIEWER,"CI_RUNNER":CI_RUNNER,"REPOSITORY":REPOSITORY,"REVIEW_A":REVIEW_A,"REVIEW_B":REVIEW_B,"TEST_DATA_A":TEST_DATA_A,"TEST_DATA_B":TEST_DATA_B,"CI_SERVICE":CI_SERVICE,"REPO":REPO,"TEST_ENV":TEST_ENV,"BUILD_SERVER":BUILD_SERVER,"AUTH_A":AUTH_A,"AUTH_B":AUTH_B,"A_IO":A_IO,"B_IO":B_IO,"WRITER_IO":WRITER_IO,"REVIEWER_IO":REVIEWER_IO,"RUNNER_IO":RUNNER_IO},
 "programs":[rows(&self.a),rows(&self.b),rows(&self.writer),rows(&self.reviewer),rows(&self.runner),rows(&self.repository),rows(&self.review_a),rows(&self.review_b),rows(&self.data_a),rows(&self.data_b),rows(&self.ci),rows(&self.repo),rows(&self.environment),rows(&self.build),rows(&self.feature_a),rows(&self.feature_b),rows(&self.a_io),rows(&self.b_io),rows(&self.writer_io),rows(&self.reviewer_io),rows(&self.runner_io)]})
    }
}
pub fn programs() -> Programs {
    let source = g::par(
        delivery::protocol(),
        g::par(
            crate::shell::global::private_io::<DEVELOPER_A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<DEVELOPER_B, B_IO>(),
                g::par(
                    crate::shell::global::private_io::<TEST_WRITER, WRITER_IO>(),
                    g::par(
                        crate::shell::global::private_io::<REVIEWER, REVIEWER_IO>(),
                        crate::shell::global::private_io::<CI_RUNNER, RUNNER_IO>(),
                    ),
                ),
            ),
        ),
    );
    Programs {
        a: project(&source),
        b: project(&source),
        writer: project(&source),
        reviewer: project(&source),
        runner: project(&source),
        repository: project(&source),
        review_a: project(&source),
        review_b: project(&source),
        data_a: project(&source),
        data_b: project(&source),
        ci: project(&source),
        repo: project(&source),
        environment: project(&source),
        build: project(&source),
        feature_a: project(&source),
        feature_b: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        writer_io: project(&source),
        reviewer_io: project(&source),
        runner_io: project(&source),
    }
}
