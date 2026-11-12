//! Actual request continuations are returned before a machine operation starts.
use super::imp::JobArguments;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const TECHNICIAN_A: u8 = 0;
pub const TECHNICIAN_B: u8 = 1;
pub const SUPERVISOR: u8 = 2;
pub const LATHE: u8 = 3;
pub const MILL: u8 = 4;
pub const LATHE_SERVICE: u8 = 5;
pub const MILL_SERVICE: u8 = 6;
pub const REPORT_SERVICE: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const SUPERVISOR_IO: u8 = 10;
pub const LATHE_REQUEST_A: u8 = 11;
pub const LATHE_REQUEST_B: u8 = 12;
pub const MILL_REQUEST_A: u8 = 13;
pub const MILL_REQUEST_B: u8 = 14;
pub const FIRST_LATHE: u16 = 920;
pub const FIRST_MILL: u16 = 921;
pub const LATHE_ARRIVAL: u16 = 922;
pub const MILL_ARRIVAL: u16 = 923;
pub const SESSION: u32 = 111;

pub type LatheIntentA = g::Msg<1, JobArguments<0>>;
pub type LatheIntentB = g::Msg<2, JobArguments<1>>;
pub type MillIntentA = g::Msg<3, JobArguments<2>>;
pub type MillIntentB = g::Msg<4, JobArguments<3>>;
pub type LatheLoan<'r> = g::Msg<5, Endpoint<'r, LATHE>>;
pub type MillLoan<'r> = g::Msg<6, Endpoint<'r, MILL>>;
pub type LatheJobA = g::Msg<7, JobArguments<0>>;
pub type MillJobA = g::Msg<8, JobArguments<2>>;
pub type LatheResult<'a> = g::Msg<9, &'a [u8]>;
pub type MillResult<'a> = g::Msg<10, &'a [u8]>;
pub type ReturnLathe<'r> = g::Msg<11, Endpoint<'r, LATHE>>;
pub type ReturnMill<'r> = g::Msg<12, Endpoint<'r, MILL>>;
pub type LatheDoneA<'a> = g::Msg<13, &'a [u8]>;
pub type LatheDoneB<'a> = g::Msg<14, &'a [u8]>;
pub type MillDoneA<'a> = g::Msg<15, &'a [u8]>;
pub type MillDoneB<'a> = g::Msg<16, &'a [u8]>;
pub type CollectReports<'a> = g::Msg<17, &'a [u8]>;
pub type Reports<'a> = g::Msg<18, &'a [u8]>;
pub type LatheJobB = g::Msg<19, JobArguments<1>>;
pub type MillJobB = g::Msg<20, JobArguments<3>>;
pub type CompletedJobs<'a> = g::Msg<21, &'a [u8]>;
pub type LatheReady = g::Msg<22, ()>;
pub type LatheBusy = g::Msg<23, ()>;
pub type MillReady = g::Msg<24, ()>;
pub type MillBusy = g::Msg<25, ()>;
pub type LatheRequestLoanA<'r> = g::Msg<26, Endpoint<'r, LATHE_REQUEST_A>>;
pub type LatheRequestLoanB<'r> = g::Msg<27, Endpoint<'r, LATHE_REQUEST_B>>;
pub type MillRequestLoanA<'r> = g::Msg<28, Endpoint<'r, MILL_REQUEST_A>>;
pub type MillRequestLoanB<'r> = g::Msg<29, Endpoint<'r, MILL_REQUEST_B>>;
pub type ReturnLatheRequestA<'r> = g::Msg<30, Endpoint<'r, LATHE_REQUEST_A>>;
pub type ReturnLatheRequestB<'r> = g::Msg<31, Endpoint<'r, LATHE_REQUEST_B>>;
pub type ReturnMillRequestA<'r> = g::Msg<32, Endpoint<'r, MILL_REQUEST_A>>;
pub type ReturnMillRequestB<'r> = g::Msg<33, Endpoint<'r, MILL_REQUEST_B>>;
pub type LatheRetryA = g::Msg<34, JobArguments<0>>;
pub type LatheRetryB = g::Msg<35, JobArguments<1>>;
pub type MillRetryA = g::Msg<36, JobArguments<2>>;
pub type MillRetryB = g::Msg<37, JobArguments<3>>;

fn lathe_a_ready(after: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, LATHE_REQUEST_A, LatheReady>(),
        g::seq(
            g::send::<TECHNICIAN_A, LATHE_SERVICE, ReturnLatheRequestA>(),
            g::seq(
                g::send::<LATHE_SERVICE, TECHNICIAN_A, LatheLoan>(),
                g::seq(g::send::<LATHE, LATHE_SERVICE, LatheJobA>(), after),
            ),
        ),
    )
}
fn lathe_b_ready(after: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, LATHE_REQUEST_B, LatheReady>(),
        g::seq(
            g::send::<TECHNICIAN_B, LATHE_SERVICE, ReturnLatheRequestB>(),
            g::seq(
                g::send::<LATHE_SERVICE, TECHNICIAN_B, LatheLoan>(),
                g::seq(g::send::<LATHE, LATHE_SERVICE, LatheJobB>(), after),
            ),
        ),
    )
}
fn mill_a_ready(after: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, MILL_REQUEST_A, MillReady>(),
        g::seq(
            g::send::<TECHNICIAN_A, MILL_SERVICE, ReturnMillRequestA>(),
            g::seq(
                g::send::<MILL_SERVICE, TECHNICIAN_A, MillLoan>(),
                g::seq(g::send::<MILL, MILL_SERVICE, MillJobA>(), after),
            ),
        ),
    )
}
fn mill_b_ready(after: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, MILL_REQUEST_B, MillReady>(),
        g::seq(
            g::send::<TECHNICIAN_B, MILL_SERVICE, ReturnMillRequestB>(),
            g::seq(
                g::send::<MILL_SERVICE, TECHNICIAN_B, MillLoan>(),
                g::seq(g::send::<MILL, MILL_SERVICE, MillJobB>(), after),
            ),
        ),
    )
}
fn lathe_a_finish() -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, LATHE, LatheResult>(),
        g::par(
            g::send::<TECHNICIAN_A, LATHE_SERVICE, ReturnLathe>(),
            g::send::<TECHNICIAN_A, SUPERVISOR, LatheDoneA>(),
        ),
    )
}
fn lathe_b_finish() -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, LATHE, LatheResult>(),
        g::par(
            g::send::<TECHNICIAN_B, LATHE_SERVICE, ReturnLathe>(),
            g::send::<TECHNICIAN_B, SUPERVISOR, LatheDoneB>(),
        ),
    )
}
fn mill_a_finish() -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, MILL, MillResult>(),
        g::par(
            g::send::<TECHNICIAN_A, MILL_SERVICE, ReturnMill>(),
            g::send::<TECHNICIAN_A, SUPERVISOR, MillDoneA>(),
        ),
    )
}
fn mill_b_finish() -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, MILL, MillResult>(),
        g::par(
            g::send::<TECHNICIAN_B, MILL_SERVICE, ReturnMill>(),
            g::send::<TECHNICIAN_B, SUPERVISOR, MillDoneB>(),
        ),
    )
}
fn lathe_a() -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, TECHNICIAN_A, LatheRequestLoanA>(),
        lathe_a_ready(lathe_a_finish()),
    )
}
fn lathe_b() -> impl Projectable {
    g::seq(
        g::send::<LATHE_SERVICE, TECHNICIAN_B, LatheRequestLoanB>(),
        lathe_b_ready(lathe_b_finish()),
    )
}
fn mill_a() -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, TECHNICIAN_A, MillRequestLoanA>(),
        mill_a_ready(mill_a_finish()),
    )
}
fn mill_b() -> impl Projectable {
    g::seq(
        g::send::<MILL_SERVICE, TECHNICIAN_B, MillRequestLoanB>(),
        mill_b_ready(mill_b_finish()),
    )
}
fn lathe_a_then_b() -> impl Projectable {
    let completed_first = g::seq(
        g::send::<LATHE_SERVICE, LATHE, LatheResult>(),
        g::par(
            g::seq(
                g::send::<TECHNICIAN_A, LATHE_SERVICE, ReturnLathe>(),
                lathe_b(),
            ),
            g::send::<TECHNICIAN_A, SUPERVISOR, LatheDoneA>(),
        ),
    );
    let request_first = g::seq(
        g::par(
            lathe_a_finish(),
            g::seq(
                g::send::<LATHE_SERVICE, TECHNICIAN_B, LatheRequestLoanB>(),
                g::seq(
                    g::send::<LATHE_SERVICE, LATHE_REQUEST_B, LatheBusy>(),
                    g::seq(
                        g::send::<LATHE_REQUEST_B, LATHE_SERVICE, LatheRetryB>(),
                        g::send::<LATHE_SERVICE, LATHE_REQUEST_B, LatheBusy>(),
                    )
                    .roll(),
                ),
            ),
        ),
        lathe_b_ready(lathe_b_finish()),
    );
    g::seq(
        g::send::<LATHE_SERVICE, TECHNICIAN_A, LatheRequestLoanA>(),
        lathe_a_ready(g::route(completed_first, request_first).resolve::<LATHE_ARRIVAL>()),
    )
}
fn lathe_b_then_a() -> impl Projectable {
    let completed_first = g::seq(
        g::send::<LATHE_SERVICE, LATHE, LatheResult>(),
        g::par(
            g::seq(
                g::send::<TECHNICIAN_B, LATHE_SERVICE, ReturnLathe>(),
                lathe_a(),
            ),
            g::send::<TECHNICIAN_B, SUPERVISOR, LatheDoneB>(),
        ),
    );
    let request_first = g::seq(
        g::par(
            lathe_b_finish(),
            g::seq(
                g::send::<LATHE_SERVICE, TECHNICIAN_A, LatheRequestLoanA>(),
                g::seq(
                    g::send::<LATHE_SERVICE, LATHE_REQUEST_A, LatheBusy>(),
                    g::seq(
                        g::send::<LATHE_REQUEST_A, LATHE_SERVICE, LatheRetryA>(),
                        g::send::<LATHE_SERVICE, LATHE_REQUEST_A, LatheBusy>(),
                    )
                    .roll(),
                ),
            ),
        ),
        lathe_a_ready(lathe_a_finish()),
    );
    g::seq(
        g::send::<LATHE_SERVICE, TECHNICIAN_B, LatheRequestLoanB>(),
        lathe_b_ready(g::route(completed_first, request_first).resolve::<LATHE_ARRIVAL>()),
    )
}
fn mill_a_then_b() -> impl Projectable {
    let completed_first = g::seq(
        g::send::<MILL_SERVICE, MILL, MillResult>(),
        g::par(
            g::seq(
                g::send::<TECHNICIAN_A, MILL_SERVICE, ReturnMill>(),
                mill_b(),
            ),
            g::send::<TECHNICIAN_A, SUPERVISOR, MillDoneA>(),
        ),
    );
    let request_first = g::seq(
        g::par(
            mill_a_finish(),
            g::seq(
                g::send::<MILL_SERVICE, TECHNICIAN_B, MillRequestLoanB>(),
                g::seq(
                    g::send::<MILL_SERVICE, MILL_REQUEST_B, MillBusy>(),
                    g::seq(
                        g::send::<MILL_REQUEST_B, MILL_SERVICE, MillRetryB>(),
                        g::send::<MILL_SERVICE, MILL_REQUEST_B, MillBusy>(),
                    )
                    .roll(),
                ),
            ),
        ),
        mill_b_ready(mill_b_finish()),
    );
    g::seq(
        g::send::<MILL_SERVICE, TECHNICIAN_A, MillRequestLoanA>(),
        mill_a_ready(g::route(completed_first, request_first).resolve::<MILL_ARRIVAL>()),
    )
}
fn mill_b_then_a() -> impl Projectable {
    let completed_first = g::seq(
        g::send::<MILL_SERVICE, MILL, MillResult>(),
        g::par(
            g::seq(
                g::send::<TECHNICIAN_B, MILL_SERVICE, ReturnMill>(),
                mill_a(),
            ),
            g::send::<TECHNICIAN_B, SUPERVISOR, MillDoneB>(),
        ),
    );
    let request_first = g::seq(
        g::par(
            mill_b_finish(),
            g::seq(
                g::send::<MILL_SERVICE, TECHNICIAN_A, MillRequestLoanA>(),
                g::seq(
                    g::send::<MILL_SERVICE, MILL_REQUEST_A, MillBusy>(),
                    g::seq(
                        g::send::<MILL_REQUEST_A, MILL_SERVICE, MillRetryA>(),
                        g::send::<MILL_SERVICE, MILL_REQUEST_A, MillBusy>(),
                    )
                    .roll(),
                ),
            ),
        ),
        mill_a_ready(mill_a_finish()),
    );
    g::seq(
        g::send::<MILL_SERVICE, TECHNICIAN_B, MillRequestLoanB>(),
        mill_b_ready(g::route(completed_first, request_first).resolve::<MILL_ARRIVAL>()),
    )
}
pub fn business() -> impl Projectable {
    let lathe = g::par(
        g::par(
            g::send::<TECHNICIAN_A, LATHE_SERVICE, LatheIntentA>(),
            g::send::<TECHNICIAN_B, LATHE_SERVICE, LatheIntentB>(),
        ),
        g::route(lathe_a_then_b(), lathe_b_then_a()).resolve::<FIRST_LATHE>(),
    );
    let mill = g::par(
        g::par(
            g::send::<TECHNICIAN_A, MILL_SERVICE, MillIntentA>(),
            g::send::<TECHNICIAN_B, MILL_SERVICE, MillIntentB>(),
        ),
        g::route(mill_a_then_b(), mill_b_then_a()).resolve::<FIRST_MILL>(),
    );
    g::seq(
        g::par(lathe, mill),
        g::seq(
            g::send::<SUPERVISOR, REPORT_SERVICE, CompletedJobs>(),
            g::seq(
                g::send::<SUPERVISOR, REPORT_SERVICE, CollectReports>(),
                g::send::<REPORT_SERVICE, SUPERVISOR, Reports>(),
            ),
        ),
    )
}
pub fn source() -> impl Projectable {
    g::par(
        business(),
        g::par(
            crate::shell::global::private_io::<TECHNICIAN_A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<TECHNICIAN_B, B_IO>(),
                crate::shell::global::private_io::<SUPERVISOR, SUPERVISOR_IO>(),
            ),
        ),
    )
}

pub struct Programs {
    pub a: RoleProgram<TECHNICIAN_A>,
    pub b: RoleProgram<TECHNICIAN_B>,
    pub supervisor: RoleProgram<SUPERVISOR>,
    pub lathe: RoleProgram<LATHE>,
    pub mill: RoleProgram<MILL>,
    pub lathe_service: RoleProgram<LATHE_SERVICE>,
    pub mill_service: RoleProgram<MILL_SERVICE>,
    pub report_service: RoleProgram<REPORT_SERVICE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub supervisor_io: RoleProgram<SUPERVISOR_IO>,
    pub lathe_request_a: RoleProgram<LATHE_REQUEST_A>,
    pub lathe_request_b: RoleProgram<LATHE_REQUEST_B>,
    pub mill_request_a: RoleProgram<MILL_REQUEST_A>,
    pub mill_request_b: RoleProgram<MILL_REQUEST_B>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("LatheIntentA", LatheIntentA::LOGICAL_LABEL),
            ("LatheIntentB", LatheIntentB::LOGICAL_LABEL),
            ("MillIntentA", MillIntentA::LOGICAL_LABEL),
            ("MillIntentB", MillIntentB::LOGICAL_LABEL),
            ("LatheLoan", LatheLoan::LOGICAL_LABEL),
            ("MillLoan", MillLoan::LOGICAL_LABEL),
            ("LatheJobA", LatheJobA::LOGICAL_LABEL),
            ("LatheJobB", LatheJobB::LOGICAL_LABEL),
            ("MillJobA", MillJobA::LOGICAL_LABEL),
            ("MillJobB", MillJobB::LOGICAL_LABEL),
            ("LatheResult", LatheResult::LOGICAL_LABEL),
            ("MillResult", MillResult::LOGICAL_LABEL),
            ("ReturnLathe", ReturnLathe::LOGICAL_LABEL),
            ("ReturnMill", ReturnMill::LOGICAL_LABEL),
            ("LatheDoneA", LatheDoneA::LOGICAL_LABEL),
            ("LatheDoneB", LatheDoneB::LOGICAL_LABEL),
            ("MillDoneA", MillDoneA::LOGICAL_LABEL),
            ("MillDoneB", MillDoneB::LOGICAL_LABEL),
            ("CompletedJobs", CompletedJobs::LOGICAL_LABEL),
            ("CollectReports", CollectReports::LOGICAL_LABEL),
            ("Reports", Reports::LOGICAL_LABEL),
            ("LatheReady", LatheReady::LOGICAL_LABEL),
            ("LatheBusy", LatheBusy::LOGICAL_LABEL),
            ("MillReady", MillReady::LOGICAL_LABEL),
            ("MillBusy", MillBusy::LOGICAL_LABEL),
            ("LatheRequestLoanA", LatheRequestLoanA::LOGICAL_LABEL),
            ("LatheRequestLoanB", LatheRequestLoanB::LOGICAL_LABEL),
            ("MillRequestLoanA", MillRequestLoanA::LOGICAL_LABEL),
            ("MillRequestLoanB", MillRequestLoanB::LOGICAL_LABEL),
            ("ReturnLatheRequestA", ReturnLatheRequestA::LOGICAL_LABEL),
            ("ReturnLatheRequestB", ReturnLatheRequestB::LOGICAL_LABEL),
            ("ReturnMillRequestA", ReturnMillRequestA::LOGICAL_LABEL),
            ("ReturnMillRequestB", ReturnMillRequestB::LOGICAL_LABEL),
            ("LatheRetryA", LatheRetryA::LOGICAL_LABEL),
            ("LatheRetryB", LatheRetryB::LOGICAL_LABEL),
            ("MillRetryA", MillRetryA::LOGICAL_LABEL),
            ("MillRetryB", MillRetryB::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"11E","session":SESSION,"labels":labels,
            "roles":{"TECHNICIAN_A":TECHNICIAN_A,"TECHNICIAN_B":TECHNICIAN_B,"SUPERVISOR":SUPERVISOR,
                "LATHE":LATHE,"MILL":MILL,"LATHE_SERVICE":LATHE_SERVICE,"MILL_SERVICE":MILL_SERVICE,
                "REPORT_SERVICE":REPORT_SERVICE,"A_IO":A_IO,"B_IO":B_IO,"SUPERVISOR_IO":SUPERVISOR_IO,
                "LATHE_REQUEST_A":LATHE_REQUEST_A,"LATHE_REQUEST_B":LATHE_REQUEST_B,
                "MILL_REQUEST_A":MILL_REQUEST_A,"MILL_REQUEST_B":MILL_REQUEST_B},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.supervisor),rows(&self.lathe),rows(&self.mill),
                rows(&self.lathe_service),rows(&self.mill_service),rows(&self.report_service),rows(&self.a_io),
                rows(&self.b_io),rows(&self.supervisor_io),rows(&self.lathe_request_a),rows(&self.lathe_request_b),
                rows(&self.mill_request_a),rows(&self.mill_request_b)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project::<TECHNICIAN_A>(&source),
        b: project::<TECHNICIAN_B>(&source),
        supervisor: project::<SUPERVISOR>(&source),
        lathe: project::<LATHE>(&source),
        mill: project::<MILL>(&source),
        lathe_service: project::<LATHE_SERVICE>(&source),
        mill_service: project::<MILL_SERVICE>(&source),
        report_service: project::<REPORT_SERVICE>(&source),
        a_io: project::<A_IO>(&source),
        b_io: project::<B_IO>(&source),
        supervisor_io: project::<SUPERVISOR_IO>(&source),
        lathe_request_a: project::<LATHE_REQUEST_A>(&source),
        lathe_request_b: project::<LATHE_REQUEST_B>(&source),
        mill_request_a: project::<MILL_REQUEST_A>(&source),
        mill_request_b: project::<MILL_REQUEST_B>(&source),
    }
}
