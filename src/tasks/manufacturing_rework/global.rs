//! Each workstation serves either assigned worker first. Three of the six jobs also require a cabinet tool.
use super::imp::{DO_JOB, INSPECT_QUALITY};
use crate::shell::global::{FileExit, Loan};
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const A_FILES: u8 = 15;
pub const ALPHA_C_FILES: u8 = 40;
pub const B_FILES: u8 = 16;
pub const BETA_A_FILES: u8 = 41;
pub const C_FILES: u8 = 17;
pub const GAMMA_B_FILES: u8 = 42;
pub const WORKER_A: u8 = 0;
pub const WORKER_B: u8 = 1;
pub const WORKER_C: u8 = 2;
pub const DEPOT: u8 = 3;
pub const TOOL_0: u8 = 4;
pub const TOOL_1: u8 = 5;
pub const ALPHA: u8 = 6;
pub const BETA: u8 = 7;
pub const GAMMA: u8 = 8;
pub const FIRST_A: u16 = 940;
pub const FIRST_B: u16 = 941;
pub const NEXT: u16 = 942;
pub const ARRIVAL_0: u16 = 943;
pub const ARRIVAL_1: u16 = 944;
pub const FINISHED_FIRST: u16 = 945;
pub const QUALITY_FIRST: u16 = 946;
pub const SESSION: u32 = 112;
pub const ALPHA_C: u8 = 28;
pub const BETA_A: u8 = 29;
pub const GAMMA_B: u8 = 30;
pub const ALPHA_MACHINE: u8 = 31;
pub const BETA_MACHINE: u8 = 32;
pub const GAMMA_MACHINE: u8 = 33;
pub const ALPHA_OWNER: u8 = 34;
pub const BETA_OWNER: u8 = 35;
pub const GAMMA_OWNER: u8 = 36;
pub const ALPHA_SERVICE: u8 = 37;
pub const BETA_SERVICE: u8 = 38;
pub const GAMMA_SERVICE: u8 = 39;
pub const ALPHA_JOB_C: u8 = 5;
pub const BETA_JOB_A: u8 = 3;
pub const GAMMA_JOB_B: u8 = 4;
pub const INSPECTOR: u8 = 26;
pub const DISPATCHER: u8 = 27;
pub const BENCH: u8 = 43;
pub const GAUGE: u8 = 44;
pub const READOUT: u8 = 45;

pub type StationJob<const WORKER: u8> = g::Msg<35, super::imp::JobArguments<WORKER>>;
pub type Intent<const WORKER: u8> = g::Msg<10, super::imp::JobArguments<WORKER>>;
pub type Loan0<'r> = g::Msg<1, Endpoint<'r, TOOL_0>>;
pub type Loan1<'r> = g::Msg<2, Endpoint<'r, TOOL_1>>;
pub type Request0<const WORKER: u8> = g::Msg<3, super::imp::JobArguments<WORKER, DO_JOB>>;
pub type Request1<const WORKER: u8> = g::Msg<4, super::imp::JobArguments<WORKER, DO_JOB>>;
pub type Reply0<'a> = g::Msg<5, &'a [u8]>;
pub type Reply1<'a> = g::Msg<6, &'a [u8]>;
pub type Return0<'r> = g::Msg<7, Endpoint<'r, TOOL_0>>;
pub type Return1<'r> = g::Msg<8, Endpoint<'r, TOOL_1>>;
pub type JobIntent<const JOB: u8> = g::Msg<20, super::imp::JobArguments<JOB, DO_JOB>>;
pub type JobResult<'a> = g::Msg<21, &'a [u8]>;
pub type StationResult<'a, const MACHINE: u8> = g::Msg<MACHINE, &'a [u8]>;
pub type FinishedPart<'a, const WORKER: u8> = g::Msg<WORKER, &'a [u8]>;
pub type CheckPart<const JOB: u8> = g::Msg<JOB, super::imp::JobArguments<JOB, INSPECT_QUALITY>>;
pub type PartReady<'a> = g::Msg<22, &'a [u8]>;
pub type JobLoan<'r, const CAP: u8> = g::Msg<23, Endpoint<'r, CAP>>;
pub type Approved<'a, const JOB: u8> = g::Msg<25, super::imp::InspectionReply<'a, JOB, true>>;
pub type Failed<'a, const JOB: u8> = g::Msg<26, super::imp::InspectionReply<'a, JOB, false>>;
pub type FailureStream<'a, const JOB: u8> = g::Msg<26, super::imp::FailureItem<'a, JOB>>;
pub type DispatchRequest<const JOB: u8> = g::Msg<27, super::imp::DispatchArguments<JOB>>;
pub type Dispatched<'a> = g::Msg<28, &'a [u8]>;
pub type WorkOrder<'a, const JOB: u8> = g::Msg<JOB, &'a [u8]>;
pub type JobUpdate<'a> = g::Msg<36, &'a [u8]>;
pub type OrderEvidence<'a> = g::Msg<29, &'a [u8]>;
pub type ApprovalForWorker<'a, const JOB: u8> =
    g::Msg<31, super::imp::InspectionReply<'a, JOB, true>>;
pub type OrderForWorker<'a> = g::Msg<32, &'a [u8]>;
pub type InspectionResult<'a> = g::Msg<34, &'a [u8]>;
pub type Measurement<'a, const JOB: u8> = g::Msg<JOB, &'a [u8]>;

/// The inspector may have several samples ready. The gauge can start its
/// next measurement only after its electronics has returned the actual reading.
pub fn gauge() -> impl Projectable {
    let sample = g::route(
        g::route(
            g::send::<GAUGE, READOUT, CheckPart<WORKER_A>>(),
            g::route(
                g::send::<GAUGE, READOUT, CheckPart<WORKER_B>>(),
                g::send::<GAUGE, READOUT, CheckPart<WORKER_C>>(),
            ),
        ),
        g::route(
            g::send::<GAUGE, READOUT, CheckPart<BETA_JOB_A>>(),
            g::route(
                g::send::<GAUGE, READOUT, CheckPart<GAMMA_JOB_B>>(),
                g::send::<GAUGE, READOUT, CheckPart<ALPHA_JOB_C>>(),
            ),
        ),
    );
    g::seq(
        g::send::<READOUT, BENCH, Loan<GAUGE>>(),
        g::seq(
            g::seq(
                sample,
                g::seq(
                    g::send::<READOUT, GAUGE, InspectionResult>(),
                    g::send::<GAUGE, INSPECTOR, InspectionResult>(),
                ),
            )
            .roll(),
            g::seq(
                g::send::<INSPECTOR, BENCH, FileExit>(),
                g::send::<BENCH, READOUT, Loan<GAUGE>>(),
            ),
        ),
    )
}

fn inspection<const JOB: u8, const WORKER: u8, const CAP: u8>() -> impl Projectable {
    g::seq(
        g::send::<CAP, INSPECTOR, PartReady>(),
        g::seq(
            g::send::<WORKER, INSPECTOR, JobLoan<CAP>>(),
            g::seq(
                g::send::<CAP, INSPECTOR, CheckPart<JOB>>(),
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

pub(crate) fn begin0<const WORKER: u8>() -> impl Projectable + Copy {
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
fn finish0<const WORKER: u8>() -> impl Projectable + Copy {
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
fn whole0<const WORKER: u8>() -> impl Projectable + Copy {
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

fn workstation<
    const MACHINE: u8,
    const OWNER: u8,
    const SERVICE: u8,
    const PRIMARY_FILE: u8,
    const WORKER: u8,
    const OTHER_FILE: u8,
    const OTHER_JOB: u8,
>() -> impl Projectable {
    let primary = g::seq(
        g::send::<PRIMARY_FILE, WORKER, StationJob<WORKER>>(),
        g::seq(
            g::send::<OWNER, WORKER, JobLoan<MACHINE>>(),
            g::seq(
                g::send::<MACHINE, DEPOT, Intent<WORKER>>(),
                g::seq(
                    g::send::<DEPOT, MACHINE, StationResult<MACHINE>>(),
                    g::par(
                        g::send::<WORKER, OWNER, JobLoan<MACHINE>>(),
                        g::send::<WORKER, PRIMARY_FILE, JobResult>(),
                    ),
                ),
            ),
        ),
    );
    let other = g::seq(
        g::send::<OTHER_FILE, SERVICE, StationJob<OTHER_JOB>>(),
        g::seq(
            g::send::<OWNER, SERVICE, JobLoan<MACHINE>>(),
            g::seq(
                g::send::<MACHINE, SERVICE, JobIntent<OTHER_JOB>>(),
                g::seq(
                    g::send::<SERVICE, MACHINE, JobResult>(),
                    g::par(
                        g::send::<SERVICE, OWNER, JobLoan<MACHINE>>(),
                        g::par(
                            g::send::<SERVICE, OTHER_FILE, JobResult>(),
                            g::send::<SERVICE, INSPECTOR, FinishedPart<OTHER_JOB>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let inspect = g::seq(
        check_part::<SERVICE, OTHER_JOB>(),
        disposition::<SERVICE, OTHER_FILE, OTHER_JOB>(),
    );
    g::route(
        g::seq(other, g::par(primary, inspect)),
        g::seq(primary, g::seq(other, inspect)),
    )
}

pub fn workstation_files() -> impl Projectable {
    g::par(
        g::par(
            g::send::<A_FILES, ALPHA_OWNER, Loan<ALPHA>>(),
            g::send::<ALPHA_C_FILES, ALPHA_OWNER, Loan<ALPHA_C>>(),
        ),
        g::par(
            g::par(
                g::send::<B_FILES, BETA_OWNER, Loan<BETA>>(),
                g::send::<BETA_A_FILES, BETA_OWNER, Loan<BETA_A>>(),
            ),
            g::par(
                g::send::<C_FILES, GAMMA_OWNER, Loan<GAMMA>>(),
                g::send::<GAMMA_B_FILES, GAMMA_OWNER, Loan<GAMMA_B>>(),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    let stations = g::par(
        workstation::<
            ALPHA_MACHINE,
            ALPHA_OWNER,
            ALPHA_SERVICE,
            ALPHA,
            WORKER_A,
            ALPHA_C,
            ALPHA_JOB_C,
        >(),
        g::par(
            workstation::<BETA_MACHINE, BETA_OWNER, BETA_SERVICE, BETA, WORKER_B, BETA_A, BETA_JOB_A>(
            ),
            workstation::<
                GAMMA_MACHINE,
                GAMMA_OWNER,
                GAMMA_SERVICE,
                GAMMA,
                WORKER_C,
                GAMMA_B,
                GAMMA_JOB_B,
            >(),
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
    let tool_checks = g::par(
        tool_quality::<WORKER_A, ALPHA>(),
        g::par(
            tool_quality::<WORKER_B, BETA>(),
            tool_quality::<WORKER_C, GAMMA>(),
        ),
    );
    g::par(gauge(), g::par(stations, g::seq(serving, tool_checks)))
}

fn tool_quality<const WORKER: u8, const FILE: u8>() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<WORKER, INSPECTOR, FinishedPart<WORKER>>(),
            check_part::<WORKER, WORKER>(),
        ),
        disposition::<WORKER, FILE, WORKER>(),
    )
}

fn disposition<const WORKER: u8, const STATION: u8, const JOB: u8>() -> impl Projectable {
    g::route(
        g::seq(
            g::send::<WORKER, STATION, JobUpdate>(),
            g::send::<WORKER, DISPATCHER, FailureStream<JOB>>(),
        ),
        g::seq(
            g::send::<WORKER, DISPATCHER, FailureStream<JOB>>(),
            g::seq(
                g::send::<DISPATCHER, WORKER, DispatchRequest<JOB>>(),
                g::par(
                    g::send::<WORKER, DISPATCHER, WorkOrder<JOB>>(),
                    g::send::<WORKER, STATION, JobUpdate>(),
                ),
            ),
        ),
    )
}

fn check_part<const SERVICE: u8, const JOB: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<INSPECTOR, SERVICE, CheckPart<JOB>>(),
        g::seq(
            g::send::<SERVICE, BENCH, CheckPart<JOB>>(),
            g::send::<BENCH, SERVICE, Measurement<JOB>>(),
        ),
    )
}
