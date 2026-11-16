//! Independent feature reviews share one affine repository owner.
use super::imp::Arguments;
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
pub const REPO_OWNER: u8 = 0;
pub const REVIEW_A: u8 = 2;
pub const REVIEW_B: u8 = 3;
pub const REPO: u8 = 7;
pub const REPO_INIT: u8 = 13;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const REVIEW_IO: u8 = 10;
pub const TARGET_A: u8 = 5;
pub const TARGET_B: u8 = 6;
pub const DECIDE_A: u16 = 264;
pub const DECIDE_B: u16 = 265;
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "notification_feature";
pub const MERGE_BRANCH_A: &str = "dev_a_feature";
pub const MERGE_BRANCH_B: &str = "dev_b_feature";
pub const A_FILE: u8 = 32;
pub const B_FILE: u8 = 33;
pub const A_REVIEW_FILE: u8 = 34;
pub const B_REVIEW_FILE: u8 = 35;
pub const A_VM: u8 = 64;
pub const B_VM: u8 = 65;
pub const REVIEW_VM: u8 = 66;
pub const A_DATA: u8 = 96;
pub const B_DATA: u8 = 97;
pub const A_REVIEW_DATA: u8 = 98;
pub const B_REVIEW_DATA: u8 = 99;
pub type RepoLoan<'run> = g::Msg<10, Endpoint<'run, REPO>>;
pub type FileImplementA = g::Msg<200, Arguments<0>>;
pub type FileImplementB = g::Msg<201, Arguments<2>>;
pub type FileMergeA = g::Msg<202, Arguments<1>>;
pub type FileMergeB = g::Msg<203, Arguments<3>>;
pub type FileReview<const TARGET: u8> = g::Msg<TARGET, Arguments<TARGET>>;
pub type FileReviewResult<'a, const TARGET: u8> = g::Msg<TARGET, &'a [u8]>;
pub type Ready<'a, const TARGET: u8> = g::Msg<TARGET, &'a [u8]>;
pub type FileUpdate<'a, const FILE: u8> = g::Msg<FILE, &'a [u8]>;
pub type FileImplementationA<'a> = g::Msg<208, &'a [u8]>;
pub type FileImplementationB<'a> = g::Msg<209, &'a [u8]>;
pub type FileMergedA<'a> = g::Msg<210, &'a [u8]>;
pub type FileMergedB<'a> = g::Msg<211, &'a [u8]>;
pub type ApprovedA<'a> = g::Msg<30, ReviewReply<'a, true>>;
pub type RejectedA<'a> = g::Msg<31, ReviewReply<'a, false>>;
pub type ApprovedB<'a> = g::Msg<40, ReviewReply<'a, true>>;
pub type RejectedB<'a> = g::Msg<41, ReviewReply<'a, false>>;
pub type RepoExit = g::Msg<234, i32>;
const fn writable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: None,
            readdir: None,
        },
    )
}
pub const DEVELOPER_FILES: [ChoreoFsObject<'static>; 3] = [
    writable(b"service/implement_feature", 4),
    writable(b"service/merge_code", 5),
    readable(b"service/review", 6),
];
pub const REVIEW_FILES: [ChoreoFsObject<'static>; 2] = [
    writable(b"service/code_review/user_authentication", 4),
    writable(b"service/code_review/notification_feature", 5),
];

fn attempt_a() -> impl Projectable + Copy {
    g::par(
        g::seq(
            g::send::<A_FILE, REPO, FileImplementA>(),
            g::seq(
                g::send::<REPO, A_FILE, FileImplementationA>(),
                g::send::<REPO, REVIEW_A, Ready<TARGET_A>>(),
            ),
        ),
        g::send::<A_REVIEW_FILE, REVIEW_A, FileReview<TARGET_A>>(),
    )
}
fn outcome_a<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, REPO, M>(),
        g::seq(
            g::send::<REPO, A_FILE, FileUpdate<A_FILE>>(),
            g::send::<REVIEW_A, A_REVIEW_FILE, FileReviewResult<TARGET_A>>(),
        ),
    )
}
fn feature_a() -> impl Projectable {
    let rejected = g::seq(outcome_a::<RejectedA>(), attempt_a());
    let approved = g::route(
        outcome_a::<ApprovedA>(),
        g::seq(rejected.roll(), outcome_a::<ApprovedA>()),
    )
    .resolve::<DECIDE_A>();
    g::seq(
        g::seq(attempt_a(), approved),
        g::seq(
            g::send::<A_FILE, REPO, FileMergeA>(),
            g::send::<REPO, A_FILE, FileMergedA>(),
        ),
    )
}
fn attempt_b() -> impl Projectable + Copy {
    g::par(
        g::seq(
            g::send::<B_FILE, REPO, FileImplementB>(),
            g::seq(
                g::send::<REPO, B_FILE, FileImplementationB>(),
                g::send::<REPO, REVIEW_B, Ready<TARGET_B>>(),
            ),
        ),
        g::send::<B_REVIEW_FILE, REVIEW_B, FileReview<TARGET_B>>(),
    )
}
fn outcome_b<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REPO, M>(),
        g::seq(
            g::send::<REPO, B_FILE, FileUpdate<B_FILE>>(),
            g::send::<REVIEW_B, B_REVIEW_FILE, FileReviewResult<TARGET_B>>(),
        ),
    )
}
fn feature_b() -> impl Projectable {
    let rejected = g::seq(outcome_b::<RejectedB>(), attempt_b());
    let approved = g::route(
        outcome_b::<ApprovedB>(),
        g::seq(rejected.roll(), outcome_b::<ApprovedB>()),
    )
    .resolve::<DECIDE_B>();
    g::seq(
        g::seq(attempt_b(), approved),
        g::seq(
            g::send::<B_FILE, REPO, FileMergeB>(),
            g::send::<REPO, B_FILE, FileMergedB>(),
        ),
    )
}
pub fn source() -> impl Projectable {
    let files = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::par(
                g::send::<A_REVIEW_DATA, REVIEW_VM, Loan<A_REVIEW_FILE>>(),
                g::send::<B_REVIEW_DATA, REVIEW_VM, Loan<B_REVIEW_FILE>>(),
            ),
        ),
    );
    let task = g::seq(
        g::par(feature_a(), feature_b()),
        g::send::<REVIEW_A, REPO, RepoExit>(),
    );
    let metadata = g::par(
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<A_REVIEW_FILE, A_REVIEW_DATA>().roll(),
                g::send::<A_REVIEW_FILE, A_REVIEW_DATA, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<B_REVIEW_FILE, B_REVIEW_DATA>().roll(),
                g::send::<B_REVIEW_FILE, B_REVIEW_DATA, FileExit>(),
            ),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
                g::send::<A_FILE, A_DATA, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                g::send::<B_FILE, B_DATA, FileExit>(),
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
            g::seq(
                crate::shell::global::private_io::<REVIEW_VM, REVIEW_IO>(),
                g::send::<REVIEW_VM, REVIEW_IO, Exit>(),
            ),
        ),
    );
    g::seq(
        g::send::<REPO_INIT, REPO_OWNER, RepoLoan>(),
        g::seq(files, g::par(task, g::par(metadata, io))),
    )
}
pub struct Programs {
    pub repo_owner: RoleProgram<REPO_OWNER>,
    pub repo: RoleProgram<REPO>,
    pub repo_init: RoleProgram<REPO_INIT>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
    pub a_file: RoleProgram<A_FILE>,
    pub b_file: RoleProgram<B_FILE>,
    pub a_review_file: RoleProgram<A_REVIEW_FILE>,
    pub b_review_file: RoleProgram<B_REVIEW_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub review_vm: RoleProgram<REVIEW_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_data: RoleProgram<B_DATA>,
    pub a_review_data: RoleProgram<A_REVIEW_DATA>,
    pub b_review_data: RoleProgram<B_REVIEW_DATA>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        let roles = serde_json::json!({"REPO_OWNER":REPO_OWNER,"REPO":REPO,"REPO_INIT":REPO_INIT,"REVIEW_A":REVIEW_A,"REVIEW_B":REVIEW_B,"A_IO":A_IO,"B_IO":B_IO,"REVIEW_IO":REVIEW_IO,"A_FILE":A_FILE,"B_FILE":B_FILE,"A_REVIEW_FILE":A_REVIEW_FILE,"B_REVIEW_FILE":B_REVIEW_FILE,"A_VM":A_VM,"B_VM":B_VM,"REVIEW_VM":REVIEW_VM,"A_DATA":A_DATA,"B_DATA":B_DATA,"A_REVIEW_DATA":A_REVIEW_DATA,"B_REVIEW_DATA":B_REVIEW_DATA});
        let programs = [
            rows(&self.repo_owner),
            rows(&self.repo),
            rows(&self.repo_init),
            rows(&self.review_a),
            rows(&self.review_b),
            rows(&self.a_io),
            rows(&self.b_io),
            rows(&self.review_io),
            rows(&self.a_file),
            rows(&self.b_file),
            rows(&self.a_review_file),
            rows(&self.b_review_file),
            rows(&self.a_vm),
            rows(&self.b_vm),
            rows(&self.review_vm),
            rows(&self.a_data),
            rows(&self.b_data),
            rows(&self.a_review_data),
            rows(&self.b_review_data),
        ];
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"6E","roles":roles,"programs":programs.as_slice()})
    }
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        repo_owner: project(&p),
        repo: project(&p),
        repo_init: project(&p),
        review_a: project(&p),
        review_b: project(&p),
        a_io: project(&p),
        b_io: project(&p),
        review_io: project(&p),
        a_file: project(&p),
        b_file: project(&p),
        a_review_file: project(&p),
        b_review_file: project(&p),
        a_vm: project(&p),
        b_vm: project(&p),
        review_vm: project(&p),
        a_data: project(&p),
        b_data: project(&p),
        a_review_data: project(&p),
        b_review_data: project(&p),
    }
}
