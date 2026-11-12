//! WASI private I/O shared by the task-specific global protocols.
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::protocol;
pub type Exit = g::Msg<240, i32>;

pub fn private_io<const VM: u8, const ANSWER: u8>() -> impl Projectable {
    let directory = g::seq(
        g::send::<VM, ANSWER, protocol::FdReaddirReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdReaddirRetMsg>(),
    );
    let write_terminal = g::seq(
        g::send::<VM, ANSWER, protocol::FdWriteReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdWriteRetMsg>(),
    );
    let read = g::seq(
        g::send::<VM, ANSWER, protocol::FdReadReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdReadRetMsg>(),
    );
    let stat = g::seq(
        g::send::<VM, ANSWER, protocol::FdFdstatGetReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdFdstatGetRetMsg>(),
    );
    let prestat = g::seq(
        g::send::<VM, ANSWER, protocol::FdPrestatGetReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdPrestatGetRetMsg>(),
    );
    let prestat_name = g::seq(
        g::send::<VM, ANSWER, protocol::FdPrestatDirNameReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdPrestatDirNameRetMsg>(),
    );
    let file_stat = g::seq(
        g::send::<VM, ANSWER, protocol::FdFilestatGetReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdFilestatGetRetMsg>(),
    );
    let close = g::seq(
        g::send::<VM, ANSWER, protocol::FdCloseReqMsg>(),
        g::send::<ANSWER, VM, protocol::FdCloseRetMsg>(),
    );
    let open = g::seq(
        g::send::<VM, ANSWER, protocol::PathOpenReqMsg>(),
        g::send::<ANSWER, VM, protocol::PathOpenRetMsg>(),
    );
    let path_stat = g::seq(
        g::send::<VM, ANSWER, protocol::PathFilestatGetReqMsg>(),
        g::send::<ANSWER, VM, protocol::PathFilestatGetRetMsg>(),
    );
    let memory_grow = g::seq(
        g::send::<VM, ANSWER, protocol::MemoryGrowReqMsg>(),
        g::send::<ANSWER, VM, protocol::MemoryGrowRetMsg>(),
    );
    let io = g::route(
        directory,
        g::route(
            write_terminal,
            g::route(
                read,
                g::route(
                    stat,
                    g::route(
                        prestat,
                        g::route(
                            prestat_name,
                            g::route(
                                file_stat,
                                g::route(close, g::route(open, g::route(path_stat, memory_grow))),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    io.roll()
}
