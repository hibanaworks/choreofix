//! WASI private I/O shared by the task-specific global protocols.
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::protocol;
pub type Loan<'run, const ROLE: u8> = g::Msg<47, Endpoint<'run, ROLE>>;
pub type FileExit = g::Msg<241, i32>;
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

pub fn file_metadata<const FILE: u8, const DATA: u8>() -> impl Projectable + Copy {
    g::route(
        g::seq(
            g::send::<FILE, DATA, protocol::FdPrestatGetReqMsg>(),
            g::send::<DATA, FILE, protocol::FdPrestatGetRetMsg>(),
        ),
        g::route(
            g::seq(
                g::send::<FILE, DATA, protocol::FdPrestatDirNameReqMsg>(),
                g::send::<DATA, FILE, protocol::FdPrestatDirNameRetMsg>(),
            ),
            g::route(
                g::seq(
                    g::send::<FILE, DATA, protocol::PathOpenReqMsg>(),
                    g::send::<DATA, FILE, protocol::PathOpenRetMsg>(),
                ),
                g::route(
                    g::seq(
                        g::send::<FILE, DATA, protocol::FdWriteObjectReqMsg>(),
                        g::send::<DATA, FILE, protocol::FdWriteObjectRetMsg>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<FILE, DATA, protocol::FdFdstatGetReqMsg>(),
                            g::send::<DATA, FILE, protocol::FdFdstatGetRetMsg>(),
                        ),
                        g::route(
                            g::seq(
                                g::send::<FILE, DATA, protocol::FdFilestatGetReqMsg>(),
                                g::send::<DATA, FILE, protocol::FdFilestatGetRetMsg>(),
                            ),
                            g::route(
                                g::seq(
                                    g::send::<FILE, DATA, protocol::PathFilestatGetReqMsg>(),
                                    g::send::<DATA, FILE, protocol::PathFilestatGetRetMsg>(),
                                ),
                                g::seq(
                                    g::send::<FILE, DATA, protocol::FdCloseReqMsg>(),
                                    g::send::<DATA, FILE, protocol::FdCloseRetMsg>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
