//! 11M's first three tool-using jobs, preserving arrival and completion order.
//! Workstation/QC/rework composition is still unfinished.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const WORKER_A: u8 = 0;
pub const WORKER_B: u8 = 1;
pub const WORKER_C: u8 = 2;
pub const DEPOT: u8 = 3;
pub const TOOL_0: u8 = 4;
pub const TOOL_1: u8 = 5;
pub const FIRST_A: u16 = 940;
pub const FIRST_B: u16 = 941;
pub const NEXT: u16 = 942;
pub const ARRIVAL_0: u16 = 943;
pub const ARRIVAL_1: u16 = 944;
pub const FINISHED_FIRST: u16 = 945;
pub const QUALITY_FIRST: u16 = 946;
pub const SESSION: u32 = 112;
pub const INSPECTOR: u8 = 26;
pub const DISPATCHER: u8 = 27;

pub type Intent<const WORKER: u8> = g::Msg<10, super::imp::JobArguments<WORKER>>;
pub type Loan0<'r> = g::Msg<1, Endpoint<'r, TOOL_0>>;
pub type Loan1<'r> = g::Msg<2, Endpoint<'r, TOOL_1>>;
pub type Request0<const WORKER: u8> = g::Msg<3, super::imp::JobArguments<WORKER>>;
pub type Request1<const WORKER: u8> = g::Msg<4, super::imp::JobArguments<WORKER>>;
pub type Reply0<'a> = g::Msg<5, &'a [u8]>;
pub type Reply1<'a> = g::Msg<6, &'a [u8]>;
pub type Return0<'r> = g::Msg<7, Endpoint<'r, TOOL_0>>;
pub type Return1<'r> = g::Msg<8, Endpoint<'r, TOOL_1>>;
pub type JobIntent<const JOB: u8> = g::Msg<20, super::imp::JobArguments<JOB>>;
pub type JobResult<'a> = g::Msg<21, &'a [u8]>;
pub type PartReady<'a> = g::Msg<22, &'a [u8]>;
pub type JobLoan<'r, const CAP: u8> = g::Msg<23, Endpoint<'r, CAP>>;
pub type InspectRequest<const JOB: u8> = g::Msg<24, super::imp::JobArguments<JOB>>;
pub type Approved<'a, const JOB: u8> = g::Msg<25, super::imp::InspectionReply<'a, JOB, true>>;
pub type Failed<'a, const JOB: u8> = g::Msg<26, super::imp::InspectionReply<'a, JOB, false>>;
pub type DispatchRequest<const JOB: u8> = g::Msg<27, super::imp::DispatchArguments<JOB>>;
pub type Dispatched<'a> = g::Msg<28, &'a [u8]>;
pub type OrderEvidence<'a> = g::Msg<29, &'a [u8]>;
pub type ApprovalForWorker<'a, const JOB: u8> =
    g::Msg<31, super::imp::InspectionReply<'a, JOB, true>>;
pub type OrderForWorker<'a> = g::Msg<32, &'a [u8]>;
pub type InspectionResult<'a> = g::Msg<34, &'a [u8]>;

fn inspection<const JOB: u8, const WORKER: u8, const CAP: u8>() -> impl Projectable {
    g::seq(
        g::send::<CAP, INSPECTOR, PartReady>(),
        g::seq(
            g::send::<WORKER, INSPECTOR, JobLoan<CAP>>(),
            g::seq(
                g::send::<CAP, INSPECTOR, InspectRequest<JOB>>(),
                g::send::<INSPECTOR, CAP, InspectionResult>(),
            ),
        ),
    )
}

fn approved<const JOB: u8, const WORKER: u8, const CAP: u8>() -> impl Projectable {
    g::seq(
        g::send::<CAP, DISPATCHER, Approved<JOB>>(),
        g::seq(
            g::send::<INSPECTOR, DISPATCHER, JobLoan<CAP>>(),
            g::seq(
                g::par(
                    g::send::<CAP, WORKER, ApprovalForWorker<JOB>>(),
                    g::send::<CAP, DEPOT, Approved<JOB>>(),
                ),
                g::send::<DISPATCHER, DEPOT, JobLoan<CAP>>(),
            ),
        ),
    )
}

/// One job's actual rejection/dispatch/replacement chain. Workstation/tool
/// allocation is supplied by the work fragments, not another authority table.
/// Retiring CAP on approval closes this job's repeatable request continuation.
pub fn job_quality<const JOB: u8, const WORKER: u8, const CAP: u8>(
    initial_work: impl Projectable,
    replacement_work: impl Projectable,
) -> impl Projectable {
    let rejected = g::seq(
        g::send::<CAP, DISPATCHER, Failed<JOB>>(),
        g::seq(
            g::send::<INSPECTOR, DISPATCHER, JobLoan<CAP>>(),
            g::seq(
                g::send::<CAP, DISPATCHER, DispatchRequest<JOB>>(),
                g::seq(
                    g::send::<DISPATCHER, CAP, Dispatched>(),
                    g::seq(
                        g::par(
                            g::send::<CAP, DEPOT, OrderEvidence>(),
                            g::send::<CAP, WORKER, OrderForWorker>(),
                        ),
                        g::seq(
                            g::send::<DISPATCHER, WORKER, JobLoan<CAP>>(),
                            g::seq(replacement_work, inspection::<JOB, WORKER, CAP>()),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        initial_work,
        g::seq(
            inspection::<JOB, WORKER, CAP>(),
            g::route(
                g::seq(rejected.roll(), approved::<JOB, WORKER, CAP>()),
                approved::<JOB, WORKER, CAP>(),
            )
            .resolve::<QUALITY_FIRST>(),
        ),
    )
}

fn begin0<const WORKER: u8>() -> impl Projectable {
    g::seq(
        g::send::<DEPOT, WORKER, Loan0>(),
        g::send::<TOOL_0, DEPOT, Request0<WORKER>>(),
    )
}
fn begin1<const WORKER: u8>() -> impl Projectable {
    g::seq(
        g::send::<DEPOT, WORKER, Loan1>(),
        g::send::<TOOL_1, DEPOT, Request1<WORKER>>(),
    )
}
fn finish0<const WORKER: u8>() -> impl Projectable {
    g::seq(
        g::send::<DEPOT, TOOL_0, Reply0>(),
        g::send::<WORKER, DEPOT, Return0>(),
    )
}
fn finish1<const WORKER: u8>() -> impl Projectable {
    g::seq(
        g::send::<DEPOT, TOOL_1, Reply1>(),
        g::send::<WORKER, DEPOT, Return1>(),
    )
}
fn whole0<const WORKER: u8>() -> impl Projectable {
    g::seq(begin0::<WORKER>(), finish0::<WORKER>())
}
fn whole1<const WORKER: u8>() -> impl Projectable {
    g::seq(begin1::<WORKER>(), finish1::<WORKER>())
}

fn last_two_order<const FIRST: u8, const LAST: u8>() -> impl Projectable {
    g::seq(
        begin0::<FIRST>(),
        g::route(
            g::seq(finish0::<FIRST>(), whole0::<LAST>()),
            g::par(finish0::<FIRST>(), whole1::<LAST>()),
        )
        .resolve::<ARRIVAL_1>(),
    )
}
fn last_two<const A: u8, const B: u8>() -> impl Projectable {
    g::route(last_two_order::<A, B>(), last_two_order::<B, A>()).resolve::<NEXT>()
}

fn two_running<const FIRST: u8, const SECOND: u8, const THIRD: u8>() -> impl Projectable {
    g::seq(
        begin1::<SECOND>(),
        g::route(
            g::par(
                g::seq(finish0::<FIRST>(), whole0::<THIRD>()),
                finish1::<SECOND>(),
            ),
            g::par(
                finish0::<FIRST>(),
                g::seq(finish1::<SECOND>(), whole1::<THIRD>()),
            ),
        )
        .resolve::<FINISHED_FIRST>(),
    )
}
fn first_job<const FIRST: u8, const A: u8, const B: u8>() -> impl Projectable {
    g::seq(
        begin0::<FIRST>(),
        g::route(
            g::seq(finish0::<FIRST>(), last_two::<A, B>()),
            g::route(two_running::<FIRST, A, B>(), two_running::<FIRST, B, A>()).resolve::<NEXT>(),
        )
        .resolve::<ARRIVAL_0>(),
    )
}

pub fn source() -> impl Projectable {
    let intents = g::par(
        g::send::<WORKER_A, DEPOT, Intent<WORKER_A>>(),
        g::par(
            g::send::<WORKER_B, DEPOT, Intent<WORKER_B>>(),
            g::send::<WORKER_C, DEPOT, Intent<WORKER_C>>(),
        ),
    );
    let serving = g::route(
        first_job::<WORKER_A, WORKER_B, WORKER_C>(),
        g::route(
            first_job::<WORKER_B, WORKER_A, WORKER_C>(),
            first_job::<WORKER_C, WORKER_A, WORKER_B>(),
        )
        .resolve::<FIRST_B>(),
    )
    .resolve::<FIRST_A>();
    g::par(intents, serving)
}

pub struct Programs {
    pub a: RoleProgram<WORKER_A>,
    pub b: RoleProgram<WORKER_B>,
    pub c: RoleProgram<WORKER_C>,
    pub depot: RoleProgram<DEPOT>,
    pub tool_0: RoleProgram<TOOL_0>,
    pub tool_1: RoleProgram<TOOL_1>,
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project::<WORKER_A>(&source),
        b: project::<WORKER_B>(&source),
        c: project::<WORKER_C>(&source),
        depot: project::<DEPOT>(&source),
        tool_0: project::<TOOL_0>(&source),
        tool_1: project::<TOOL_1>(&source),
    }
}
