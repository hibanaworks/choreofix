//! Direct file requests, stable shared-code snapshots and independent validation/publication.
use super::arguments::Arguments;
pub use crate::shell::global::{Exit, FileExit, Loan};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const SESSION: u32 = 44;
pub const REPO_OWNER: u8 = 0;
pub const TEST_OWNER: u8 = 1;
pub const ARCHITECT: u8 = 2;
pub const REVIEWER: u8 = 3;
pub const RESOURCE_INIT: u8 = 4;
pub const REPO: u8 = 5;
pub const TEST_ENV: u8 = 6;
pub const ARCH_FILE: u8 = 16;
pub const ARCH_VM: u8 = 32;
pub const ARCH_DATA: u8 = 48;
pub const ARCH_IO: u8 = 64;
pub const A_FILE: u8 = 17;
pub const A_VM: u8 = 33;
pub const A_DATA: u8 = 49;
pub const A_IO: u8 = 65;
pub const B_FILE: u8 = 18;
pub const B_VM: u8 = 34;
pub const B_DATA: u8 = 50;
pub const B_IO: u8 = 66;
pub const REVIEW_FILE: u8 = 19;
pub const REVIEW_VM: u8 = 35;
pub const REVIEW_DATA: u8 = 51;
pub const REVIEW_IO: u8 = 67;
pub const TEST_FILE: u8 = 20;
pub const TEST_VM: u8 = 36;
pub const TEST_DATA: u8 = 52;
pub const TEST_IO: u8 = 68;
pub type RepoLoan<'run> = g::Msg<70, Endpoint<'run, REPO>>;
pub type TestLoan<'run> = g::Msg<71, Endpoint<'run, TEST_ENV>>;
pub type DesignA = g::Msg<100, Arguments<0>>;
pub type DesignedA<'a> = g::Msg<101, &'a [u8]>;
pub type DesignB = g::Msg<102, Arguments<1>>;
pub type DesignedB<'a> = g::Msg<103, &'a [u8]>;
pub type RefactorA = g::Msg<104, Arguments<2>>;
pub type RefactoredA<'a> = g::Msg<105, &'a [u8]>;
pub type RefactorSharedA = g::Msg<106, Arguments<3>>;
pub type SharedChangedA<'a> = g::Msg<107, &'a [u8]>;
pub type RefactorB = g::Msg<108, Arguments<4>>;
pub type RefactoredB<'a> = g::Msg<109, &'a [u8]>;
pub type RefactorSharedB = g::Msg<110, Arguments<5>>;
pub type SharedChangedB<'a> = g::Msg<111, &'a [u8]>;
pub type ReadyReviewA<'a> = g::Msg<112, &'a [u8]>;
pub type ReadyReviewB<'a> = g::Msg<113, &'a [u8]>;
pub type ReviewRequestA = g::Msg<114, Arguments<6>>;
pub type ReviewedA<'a> = g::Msg<115, &'a [u8]>;
pub type FileReviewedA<'a> = g::Msg<116, &'a [u8]>;
pub type ReviewRequestB = g::Msg<117, Arguments<7>>;
pub type ReviewedB<'a> = g::Msg<118, &'a [u8]>;
pub type FileReviewedB<'a> = g::Msg<119, &'a [u8]>;
pub type ReadyTestA<'a> = g::Msg<120, &'a [u8]>;
pub type ReadyTestB<'a> = g::Msg<121, &'a [u8]>;
pub type TestRequestA = g::Msg<122, Arguments<8>>;
pub type TestedA<'a> = g::Msg<123, &'a [u8]>;
pub type FileTestedA<'a> = g::Msg<124, &'a [u8]>;
pub type TestRequestB = g::Msg<125, Arguments<9>>;
pub type TestedB<'a> = g::Msg<126, &'a [u8]>;
pub type FileTestedB<'a> = g::Msg<127, &'a [u8]>;
pub type MergeA = g::Msg<128, Arguments<10>>;
pub type MergedA<'a> = g::Msg<129, &'a [u8]>;
pub type MergeB = g::Msg<130, Arguments<11>>;
pub type MergedB<'a> = g::Msg<131, &'a [u8]>;
pub type ReviewExit = g::Msg<132, i32>;
pub type TestExit = g::Msg<133, i32>;
pub type ReviewFeedbackA<'a> = g::Msg<134, &'a [u8]>;
pub type TestFeedbackA<'a> = g::Msg<135, &'a [u8]>;
pub type ReviewFeedbackB<'a> = g::Msg<136, &'a [u8]>;
pub type TestFeedbackB<'a> = g::Msg<137, &'a [u8]>;
pub type PublicationA<'a> = g::Msg<138, &'a [u8]>;
pub type PublishedA<'a> = g::Msg<139, &'a [u8]>;
pub type PublicationB<'a> = g::Msg<140, &'a [u8]>;
pub type PublishedB<'a> = g::Msg<141, &'a [u8]>;
pub type CandidateA<'a> = g::Msg<142, &'a [u8]>;
pub type CandidateB<'a> = g::Msg<143, &'a [u8]>;
pub type RepoExit = g::Msg<144, ()>;
pub type ReadCandidateA = g::Msg<145, ()>;
pub type ReadCandidateB = g::Msg<146, ()>;
const fn object(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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
const fn readable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )
}
pub const ARCH_FILES: [ChoreoFsObject<'static>; 2] = [
    object(b"service/design_proposal/module_a", 4),
    object(b"service/design_proposal/module_b", 5),
];
pub const A_FILES: [ChoreoFsObject<'static>; 5] = [
    object(b"service/refactor_code/module_a", 4),
    object(b"service/refactor_code/SHARED_LIBRARY", 5),
    object(b"service/merge_code", 6),
    readable(b"service/review", 7),
    readable(b"service/test_result", 8),
];
pub const B_FILES: [ChoreoFsObject<'static>; 5] = [
    object(b"service/refactor_code/module_b", 4),
    object(b"service/refactor_code/SHARED_LIBRARY", 5),
    object(b"service/merge_code", 6),
    readable(b"service/review", 7),
    readable(b"service/test_result", 8),
];
pub const REVIEW_FILES: [ChoreoFsObject<'static>; 2] = [
    object(b"service/code_review/module_a", 4),
    object(b"service/code_review/module_b", 5),
];
pub const TEST_FILES: [ChoreoFsObject<'static>; 2] = [
    object(b"service/run_tests/module_a_tests", 4),
    object(b"service/run_tests/module_b_tests", 5),
];
pub fn source() -> impl Projectable {
    let resources = g::par(
        g::send::<RESOURCE_INIT, REPO_OWNER, RepoLoan>(),
        g::send::<RESOURCE_INIT, TEST_OWNER, TestLoan>(),
    );
    let files = g::par(
        g::send::<ARCH_DATA, ARCH_VM, Loan<ARCH_FILE>>(),
        g::par(
            g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
            g::par(
                g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
                g::par(
                    g::send::<REVIEW_DATA, REVIEW_VM, Loan<REVIEW_FILE>>(),
                    g::send::<TEST_DATA, TEST_VM, Loan<TEST_FILE>>(),
                ),
            ),
        ),
    );
    let designs = g::par(
        g::seq(
            g::send::<ARCH_FILE, ARCHITECT, DesignA>(),
            g::send::<ARCHITECT, ARCH_FILE, DesignedA>(),
        ),
        g::seq(
            g::send::<ARCH_FILE, ARCHITECT, DesignB>(),
            g::send::<ARCHITECT, ARCH_FILE, DesignedB>(),
        ),
    );
    let edits_a = g::par(
        g::seq(
            g::send::<A_FILE, REPO, RefactorA>(),
            g::send::<REPO, A_FILE, RefactoredA>(),
        ),
        g::seq(
            g::send::<A_FILE, REPO, RefactorSharedA>(),
            g::send::<REPO, A_FILE, SharedChangedA>(),
        ),
    );
    let edits_b = g::par(
        g::seq(
            g::send::<B_FILE, REPO, RefactorB>(),
            g::send::<REPO, B_FILE, RefactoredB>(),
        ),
        g::seq(
            g::send::<B_FILE, REPO, RefactorSharedB>(),
            g::send::<REPO, B_FILE, SharedChangedB>(),
        ),
    );
    let module_a = g::seq(
        g::par(
            g::seq(
                g::send::<REPO_OWNER, REVIEWER, ReadyReviewA>(),
                g::seq(
                    g::send::<REVIEW_FILE, REVIEWER, ReviewRequestA>(),
                    g::par(
                        g::send::<REVIEWER, REPO_OWNER, ReviewedA>(),
                        g::send::<REVIEWER, REVIEW_FILE, FileReviewedA>(),
                    ),
                ),
            ),
            g::seq(
                g::send::<REPO_OWNER, TEST_ENV, ReadyTestA>(),
                g::seq(
                    g::send::<TEST_FILE, TEST_ENV, TestRequestA>(),
                    g::par(
                        g::send::<TEST_ENV, REPO_OWNER, TestedA>(),
                        g::send::<TEST_ENV, TEST_FILE, FileTestedA>(),
                    ),
                ),
            ),
        ),
        g::seq(
            g::par(
                g::send::<REPO_OWNER, A_FILE, ReviewFeedbackA>(),
                g::send::<REPO_OWNER, A_FILE, TestFeedbackA>(),
            ),
            g::seq(
                g::send::<A_FILE, REPO_OWNER, MergeA>(),
                g::seq(
                    g::send::<REPO_OWNER, REPO, PublicationA>(),
                    g::seq(
                        g::send::<REPO, REPO_OWNER, PublishedA>(),
                        g::send::<REPO_OWNER, A_FILE, MergedA>(),
                    ),
                ),
            ),
        ),
    );
    let module_b = g::seq(
        g::par(
            g::seq(
                g::send::<REPO_OWNER, REVIEWER, ReadyReviewB>(),
                g::seq(
                    g::send::<REVIEW_FILE, REVIEWER, ReviewRequestB>(),
                    g::par(
                        g::send::<REVIEWER, REPO_OWNER, ReviewedB>(),
                        g::send::<REVIEWER, REVIEW_FILE, FileReviewedB>(),
                    ),
                ),
            ),
            g::seq(
                g::send::<REPO_OWNER, TEST_ENV, ReadyTestB>(),
                g::seq(
                    g::send::<TEST_FILE, TEST_ENV, TestRequestB>(),
                    g::par(
                        g::send::<TEST_ENV, REPO_OWNER, TestedB>(),
                        g::send::<TEST_ENV, TEST_FILE, FileTestedB>(),
                    ),
                ),
            ),
        ),
        g::seq(
            g::par(
                g::send::<REPO_OWNER, B_FILE, ReviewFeedbackB>(),
                g::send::<REPO_OWNER, B_FILE, TestFeedbackB>(),
            ),
            g::seq(
                g::send::<B_FILE, REPO_OWNER, MergeB>(),
                g::seq(
                    g::send::<REPO_OWNER, REPO, PublicationB>(),
                    g::seq(
                        g::send::<REPO, REPO_OWNER, PublishedB>(),
                        g::send::<REPO_OWNER, B_FILE, MergedB>(),
                    ),
                ),
            ),
        ),
    );
    let task = g::seq(
        g::par(
            designs,
            g::par(
                g::seq(
                    edits_a,
                    g::seq(
                        g::seq(
                            g::send::<REPO_OWNER, REPO, ReadCandidateA>(),
                            g::send::<REPO, REPO_OWNER, CandidateA>(),
                        ),
                        module_a,
                    ),
                ),
                g::seq(
                    edits_b,
                    g::seq(
                        g::seq(
                            g::send::<REPO_OWNER, REPO, ReadCandidateB>(),
                            g::send::<REPO, REPO_OWNER, CandidateB>(),
                        ),
                        module_b,
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<REPO_OWNER, REVIEWER, ReviewExit>(),
            g::par(
                g::send::<REPO_OWNER, REPO, RepoExit>(),
                g::send::<REPO_OWNER, TEST_ENV, TestExit>(),
            ),
        ),
    );
    let metadata = g::par(
        g::seq(
            crate::shell::global::file_metadata::<ARCH_FILE, ARCH_DATA>().roll(),
            g::send::<ARCH_FILE, ARCH_DATA, FileExit>(),
        ),
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
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<REVIEW_FILE, REVIEW_DATA>().roll(),
                        g::send::<REVIEW_FILE, REVIEW_DATA, FileExit>(),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<TEST_FILE, TEST_DATA>().roll(),
                        g::send::<TEST_FILE, TEST_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            crate::shell::global::private_io::<ARCH_VM, ARCH_IO>(),
            g::send::<ARCH_VM, ARCH_IO, Exit>(),
        ),
        g::par(
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
                    g::seq(
                        crate::shell::global::private_io::<TEST_VM, TEST_IO>(),
                        g::send::<TEST_VM, TEST_IO, Exit>(),
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
    pub architect: RoleProgram<ARCHITECT>,
    pub reviewer: RoleProgram<REVIEWER>,
    pub resource_init: RoleProgram<RESOURCE_INIT>,
    pub repo: RoleProgram<REPO>,
    pub environment: RoleProgram<TEST_ENV>,
    pub arch_file: RoleProgram<ARCH_FILE>,
    pub arch_vm: RoleProgram<ARCH_VM>,
    pub arch_data: RoleProgram<ARCH_DATA>,
    pub arch_io: RoleProgram<ARCH_IO>,
    pub a_file: RoleProgram<A_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub a_io: RoleProgram<A_IO>,
    pub b_file: RoleProgram<B_FILE>,
    pub b_vm: RoleProgram<B_VM>,
    pub b_data: RoleProgram<B_DATA>,
    pub b_io: RoleProgram<B_IO>,
    pub review_file: RoleProgram<REVIEW_FILE>,
    pub review_vm: RoleProgram<REVIEW_VM>,
    pub review_data: RoleProgram<REVIEW_DATA>,
    pub review_io: RoleProgram<REVIEW_IO>,
    pub test_file: RoleProgram<TEST_FILE>,
    pub test_vm: RoleProgram<TEST_VM>,
    pub test_data: RoleProgram<TEST_DATA>,
    pub test_io: RoleProgram<TEST_IO>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("RepoLoan", RepoLoan::LOGICAL_LABEL),
            ("TestLoan", TestLoan::LOGICAL_LABEL),
            ("DesignA", DesignA::LOGICAL_LABEL),
            ("DesignedA", DesignedA::LOGICAL_LABEL),
            ("DesignB", DesignB::LOGICAL_LABEL),
            ("DesignedB", DesignedB::LOGICAL_LABEL),
            ("RefactorA", RefactorA::LOGICAL_LABEL),
            ("RefactoredA", RefactoredA::LOGICAL_LABEL),
            ("RefactorSharedA", RefactorSharedA::LOGICAL_LABEL),
            ("SharedChangedA", SharedChangedA::LOGICAL_LABEL),
            ("RefactorB", RefactorB::LOGICAL_LABEL),
            ("RefactoredB", RefactoredB::LOGICAL_LABEL),
            ("RefactorSharedB", RefactorSharedB::LOGICAL_LABEL),
            ("SharedChangedB", SharedChangedB::LOGICAL_LABEL),
            ("ReadyReviewA", ReadyReviewA::LOGICAL_LABEL),
            ("ReadyReviewB", ReadyReviewB::LOGICAL_LABEL),
            ("ReviewRequestA", ReviewRequestA::LOGICAL_LABEL),
            ("ReviewedA", ReviewedA::LOGICAL_LABEL),
            ("FileReviewedA", FileReviewedA::LOGICAL_LABEL),
            ("ReviewRequestB", ReviewRequestB::LOGICAL_LABEL),
            ("ReviewedB", ReviewedB::LOGICAL_LABEL),
            ("FileReviewedB", FileReviewedB::LOGICAL_LABEL),
            ("ReadyTestA", ReadyTestA::LOGICAL_LABEL),
            ("ReadyTestB", ReadyTestB::LOGICAL_LABEL),
            ("TestRequestA", TestRequestA::LOGICAL_LABEL),
            ("TestedA", TestedA::LOGICAL_LABEL),
            ("FileTestedA", FileTestedA::LOGICAL_LABEL),
            ("TestRequestB", TestRequestB::LOGICAL_LABEL),
            ("TestedB", TestedB::LOGICAL_LABEL),
            ("FileTestedB", FileTestedB::LOGICAL_LABEL),
            ("MergeA", MergeA::LOGICAL_LABEL),
            ("MergedA", MergedA::LOGICAL_LABEL),
            ("MergeB", MergeB::LOGICAL_LABEL),
            ("MergedB", MergedB::LOGICAL_LABEL),
            ("ReviewFeedbackA", ReviewFeedbackA::LOGICAL_LABEL),
            ("TestFeedbackA", TestFeedbackA::LOGICAL_LABEL),
            ("ReviewFeedbackB", ReviewFeedbackB::LOGICAL_LABEL),
            ("TestFeedbackB", TestFeedbackB::LOGICAL_LABEL),
            ("PublicationA", PublicationA::LOGICAL_LABEL),
            ("PublishedA", PublishedA::LOGICAL_LABEL),
            ("PublicationB", PublicationB::LOGICAL_LABEL),
            ("PublishedB", PublishedB::LOGICAL_LABEL),
            ("CandidateA", CandidateA::LOGICAL_LABEL),
            ("ReadCandidateA", ReadCandidateA::LOGICAL_LABEL),
            ("CandidateB", CandidateB::LOGICAL_LABEL),
            ("ReadCandidateB", ReadCandidateB::LOGICAL_LABEL),
            ("RepoExit", RepoExit::LOGICAL_LABEL),
            ("ReviewExit", ReviewExit::LOGICAL_LABEL),
            ("TestExit", TestExit::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(n, l)| (n.into(), l.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"4M","session":SESSION,"labels":labels,"roles":{"REPO_OWNER":REPO_OWNER,"TEST_OWNER":TEST_OWNER,"ARCHITECT":ARCHITECT,"REVIEWER":REVIEWER,"RESOURCE_INIT":RESOURCE_INIT,"REPO":REPO,"TEST_ENV":TEST_ENV,"ARCH_FILE":ARCH_FILE,"ARCH_VM":ARCH_VM,"ARCH_DATA":ARCH_DATA,"ARCH_IO":ARCH_IO,"A_FILE":A_FILE,"A_VM":A_VM,"A_DATA":A_DATA,"A_IO":A_IO,"B_FILE":B_FILE,"B_VM":B_VM,"B_DATA":B_DATA,"B_IO":B_IO,"REVIEW_FILE":REVIEW_FILE,"REVIEW_VM":REVIEW_VM,"REVIEW_DATA":REVIEW_DATA,"REVIEW_IO":REVIEW_IO,"TEST_FILE":TEST_FILE,"TEST_VM":TEST_VM,"TEST_DATA":TEST_DATA,"TEST_IO":TEST_IO},"programs":[rows(&self.repo_owner),rows(&self.test_owner),rows(&self.architect),rows(&self.reviewer),rows(&self.resource_init),rows(&self.repo),rows(&self.environment),rows(&self.arch_file),rows(&self.arch_vm),rows(&self.arch_data),rows(&self.arch_io),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_data),rows(&self.a_io),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_data),rows(&self.b_io),rows(&self.review_file),rows(&self.review_vm),rows(&self.review_data),rows(&self.review_io),rows(&self.test_file),rows(&self.test_vm),rows(&self.test_data),rows(&self.test_io)]})
    }
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        repo_owner: project(&p),
        test_owner: project(&p),
        architect: project(&p),
        reviewer: project(&p),
        resource_init: project(&p),
        repo: project(&p),
        environment: project(&p),
        arch_file: project(&p),
        arch_vm: project(&p),
        arch_data: project(&p),
        arch_io: project(&p),
        a_file: project(&p),
        a_vm: project(&p),
        a_data: project(&p),
        a_io: project(&p),
        b_file: project(&p),
        b_vm: project(&p),
        b_data: project(&p),
        b_io: project(&p),
        review_file: project(&p),
        review_vm: project(&p),
        review_data: project(&p),
        review_io: project(&p),
        test_file: project(&p),
        test_vm: project(&p),
        test_data: project(&p),
        test_io: project(&p),
    }
}
