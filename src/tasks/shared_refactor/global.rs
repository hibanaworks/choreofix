//! Shared repository changes are fenced by real Endpoint returns and test loans.
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub const ARCHITECT: u8 = 0;
pub const A: u8 = 1;
pub const B: u8 = 2;
pub const REVIEWER: u8 = 3;
pub const TESTER: u8 = 4;
pub const REVIEW_SERVICE: u8 = 5;
pub const STORE: u8 = 7;
pub const ARCHITECT_IO: u8 = 8;
pub const A_IO: u8 = 9;
pub const B_IO: u8 = 10;
pub const REVIEWER_IO: u8 = 11;
pub const TESTER_IO: u8 = 12;
pub const REPO: u8 = 13;
pub const TEST_ENV: u8 = 14;
pub const SESSION: u32 = 40;
pub type DesignA<'a> = g::Msg<1, &'a [u8]>;
pub type DesignB<'a> = g::Msg<2, &'a [u8]>;
pub type DesignResult<'a> = g::Msg<3, &'a [u8]>;
pub type OwnIntent<'a> = g::Msg<4, &'a [u8]>;
pub type SharedIntent<'a> = g::Msg<5, &'a [u8]>;
pub type Refactor<'a> = g::Msg<6, &'a [u8]>;
pub type Refactored<'a> = g::Msg<7, &'a [u8]>;
pub type RepoLoan<'r> = g::Msg<10, Endpoint<'r, REPO>>;
pub type TestLoan<'r> = g::Msg<11, Endpoint<'r, TEST_ENV>>;
pub type TestPlan<'a> = g::Msg<12, &'a [u8]>;
pub type TestA<'a> = g::Msg<13, &'a [u8]>;
pub type TestB<'a> = g::Msg<14, &'a [u8]>;
pub type ReadSnapshot<'a> = g::Msg<15, &'a [u8]>;
pub type RunTests<'a> = g::Msg<16, &'a [u8]>;
pub type RepositoryTested<'a> = g::Msg<17, &'a [u8]>;
pub type EnvironmentTested<'a> = g::Msg<18, &'a [u8]>;
pub type TestSummary<'a> = g::Msg<19, &'a [u8]>;
pub type CandidateForOwner<'a> = g::Msg<30, &'a [u8]>;
pub type TestSummaryB<'a> = g::Msg<31, &'a [u8]>;
pub type Candidate<'a> = g::Msg<20, &'a [u8]>;
pub type ReviewA<'a> = g::Msg<21, &'a [u8]>;
pub type ReviewB<'a> = g::Msg<22, &'a [u8]>;
pub type AcceptedA<'a> = g::Msg<23, &'a [u8]>;
pub type AcceptedB<'a> = g::Msg<24, &'a [u8]>;
pub type Approved<'a> = g::Msg<25, ReviewReply<'a, true>>;
pub type MergeReady<'a> = g::Msg<26, &'a [u8]>;
pub type MergeIntent<'a> = g::Msg<27, &'a [u8]>;
pub type MergeRequest<'a> = g::Msg<28, &'a [u8]>;
pub type Merged<'a> = g::Msg<29, &'a [u8]>;

fn refactor<const WHO: u8, M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<WHO, STORE, M>(),
        g::seq(
            g::send::<REPO, STORE, Refactor>(),
            g::send::<STORE, REPO, Refactored>(),
        ),
    )
}
fn implementation<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<STORE, WHO, RepoLoan>(),
        g::seq(
            g::route(
                g::seq(
                    refactor::<WHO, OwnIntent>(),
                    refactor::<WHO, SharedIntent>(),
                ),
                g::seq(
                    refactor::<WHO, SharedIntent>(),
                    refactor::<WHO, OwnIntent>(),
                ),
            ),
            g::send::<WHO, STORE, RepoLoan>(),
        ),
    )
}
fn suite<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<TESTER, STORE, M>(),
        g::seq(
            g::par(
                g::send::<REPO, STORE, ReadSnapshot>(),
                g::send::<TEST_ENV, STORE, RunTests>(),
            ),
            g::par(
                g::send::<STORE, REPO, RepositoryTested>(),
                g::send::<STORE, TEST_ENV, EnvironmentTested>(),
            ),
        ),
    )
}
fn testing() -> impl Projectable {
    g::seq(
        g::send::<STORE, TESTER, TestPlan>(),
        g::seq(
            g::par(
                g::send::<STORE, TESTER, RepoLoan>(),
                g::send::<STORE, TESTER, TestLoan>(),
            ),
            g::seq(
                g::route(
                    g::seq(suite::<TestA>(), suite::<TestB>()),
                    g::seq(suite::<TestB>(), suite::<TestA>()),
                ),
                g::par(
                    g::send::<TESTER, STORE, RepoLoan>(),
                    g::send::<TESTER, STORE, TestLoan>(),
                ),
            ),
        ),
    )
}
fn review_request<M: g::Message, N: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<REVIEWER, REVIEW_SERVICE, M>(),
        g::send::<REVIEW_SERVICE, STORE, N>(),
    )
}
fn review_result<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_SERVICE, REVIEWER, Approved>(),
        g::seq(
            g::send::<REVIEW_SERVICE, STORE, Approved>(),
            g::send::<REVIEW_SERVICE, WHO, Approved>(),
        ),
    )
}
fn merge<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<STORE, WHO, MergeReady>(),
        g::seq(
            g::send::<STORE, WHO, RepoLoan>(),
            g::seq(
                g::send::<WHO, STORE, MergeIntent>(),
                g::seq(
                    g::send::<REPO, STORE, MergeRequest>(),
                    g::seq(
                        g::send::<STORE, REPO, Merged>(),
                        g::send::<WHO, STORE, RepoLoan>(),
                    ),
                ),
            ),
        ),
    )
}
fn final_tests() -> impl Projectable {
    g::seq(
        testing(),
        g::par(
            g::send::<STORE, A, TestSummary>(),
            g::send::<STORE, B, TestSummaryB>(),
        ),
    )
}
fn reviewed<const FIRST: u8, const SECOND: u8>(
    first: impl Projectable,
    second: impl Projectable,
) -> impl Projectable {
    g::seq(
        first,
        g::seq(
            g::par(review_result::<FIRST>(), final_tests()),
            g::seq(
                g::par(merge::<FIRST>(), g::seq(second, review_result::<SECOND>())),
                merge::<SECOND>(),
            ),
        ),
    )
}
fn publication() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<STORE, REVIEWER, Candidate>(),
            g::send::<STORE, REVIEW_SERVICE, CandidateForOwner>(),
        ),
        g::route(
            reviewed::<A, B>(
                review_request::<ReviewA, AcceptedA>(),
                review_request::<ReviewB, AcceptedB>(),
            ),
            reviewed::<B, A>(
                review_request::<ReviewB, AcceptedB>(),
                review_request::<ReviewA, AcceptedA>(),
            ),
        ),
    )
}
fn first_a() -> impl Projectable {
    g::seq(
        g::send::<ARCHITECT, STORE, DesignA>(),
        g::seq(
            g::send::<STORE, ARCHITECT, DesignResult>(),
            g::seq(
                g::send::<STORE, A, DesignResult>(),
                g::seq(
                    g::par(
                        g::seq(
                            g::send::<ARCHITECT, STORE, DesignB>(),
                            g::seq(
                                g::send::<STORE, ARCHITECT, DesignResult>(),
                                g::send::<STORE, B, DesignResult>(),
                            ),
                        ),
                        g::seq(implementation::<A>(), testing()),
                    ),
                    g::seq(implementation::<B>(), publication()),
                ),
            ),
        ),
    )
}
fn first_b() -> impl Projectable {
    g::seq(
        g::send::<ARCHITECT, STORE, DesignB>(),
        g::seq(
            g::send::<STORE, ARCHITECT, DesignResult>(),
            g::seq(
                g::send::<STORE, B, DesignResult>(),
                g::seq(
                    g::par(
                        g::seq(
                            g::send::<ARCHITECT, STORE, DesignA>(),
                            g::seq(
                                g::send::<STORE, ARCHITECT, DesignResult>(),
                                g::send::<STORE, A, DesignResult>(),
                            ),
                        ),
                        g::seq(implementation::<B>(), testing()),
                    ),
                    g::seq(implementation::<A>(), publication()),
                ),
            ),
        ),
    )
}
pub struct Programs {
    pub architect: RoleProgram<ARCHITECT>,
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub reviewer: RoleProgram<REVIEWER>,
    pub tester: RoleProgram<TESTER>,
    pub review_service: RoleProgram<REVIEW_SERVICE>,
    pub store: RoleProgram<STORE>,
    pub repo: RoleProgram<REPO>,
    pub test_env: RoleProgram<TEST_ENV>,
    pub architect_io: RoleProgram<ARCHITECT_IO>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub reviewer_io: RoleProgram<REVIEWER_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"4M",
         "session":SESSION,"roles":{"ARCHITECT":ARCHITECT,"DEVELOPER_A":A,"DEVELOPER_B":B,
         "REVIEWER":REVIEWER,"TESTER":TESTER,"REVIEW_SERVICE":REVIEW_SERVICE,"STORE":STORE,
         "REPO":REPO,"TEST_ENV":TEST_ENV,"ARCHITECT_IO":ARCHITECT_IO,"A_IO":A_IO,"B_IO":B_IO,
         "REVIEWER_IO":REVIEWER_IO,"TESTER_IO":TESTER_IO},
         "labels":{"DesignA":DesignA::LOGICAL_LABEL,"DesignB":DesignB::LOGICAL_LABEL,
         "DesignResult":DesignResult::LOGICAL_LABEL,"OwnIntent":OwnIntent::LOGICAL_LABEL,
         "SharedIntent":SharedIntent::LOGICAL_LABEL,"Refactor":Refactor::LOGICAL_LABEL,
         "Refactored":Refactored::LOGICAL_LABEL,"RepoLoan":RepoLoan::LOGICAL_LABEL,
         "TestLoan":TestLoan::LOGICAL_LABEL,"TestPlan":TestPlan::LOGICAL_LABEL,
         "TestA":TestA::LOGICAL_LABEL,"TestB":TestB::LOGICAL_LABEL,
         "ReadSnapshot":ReadSnapshot::LOGICAL_LABEL,"RunTests":RunTests::LOGICAL_LABEL,
         "RepositoryTested":RepositoryTested::LOGICAL_LABEL,"EnvironmentTested":EnvironmentTested::LOGICAL_LABEL,
         "TestSummary":TestSummary::LOGICAL_LABEL,"TestSummaryB":TestSummaryB::LOGICAL_LABEL,
         "Candidate":Candidate::LOGICAL_LABEL,"CandidateForOwner":CandidateForOwner::LOGICAL_LABEL,
         "ReviewA":ReviewA::LOGICAL_LABEL,"ReviewB":ReviewB::LOGICAL_LABEL,
         "AcceptedA":AcceptedA::LOGICAL_LABEL,"AcceptedB":AcceptedB::LOGICAL_LABEL,
         "Approved":Approved::LOGICAL_LABEL,"MergeReady":MergeReady::LOGICAL_LABEL,
         "MergeIntent":MergeIntent::LOGICAL_LABEL,"MergeRequest":MergeRequest::LOGICAL_LABEL,
         "Merged":Merged::LOGICAL_LABEL},"programs":[rows(&self.architect),rows(&self.a),rows(&self.b),
         rows(&self.reviewer),rows(&self.tester),rows(&self.review_service),rows(&self.store),
         rows(&self.repo),rows(&self.test_env),rows(&self.architect_io),rows(&self.a_io),
         rows(&self.b_io),rows(&self.reviewer_io),rows(&self.tester_io)]})
    }
}
pub fn programs() -> Programs {
    let business = g::route(first_a(), first_b());
    let source = g::par(
        business,
        g::par(
            crate::shell::global::private_io::<ARCHITECT, ARCHITECT_IO>(),
            g::par(
                crate::shell::global::private_io::<A, A_IO>(),
                g::par(
                    crate::shell::global::private_io::<B, B_IO>(),
                    g::par(
                        crate::shell::global::private_io::<REVIEWER, REVIEWER_IO>(),
                        crate::shell::global::private_io::<TESTER, TESTER_IO>(),
                    ),
                ),
            ),
        ),
    );
    Programs {
        architect: project(&source),
        a: project(&source),
        b: project(&source),
        reviewer: project(&source),
        tester: project(&source),
        review_service: project(&source),
        store: project(&source),
        repo: project(&source),
        test_env: project(&source),
        architect_io: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        reviewer_io: project(&source),
        tester_io: project(&source),
    }
}
