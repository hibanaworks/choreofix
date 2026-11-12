//! Actual WASI file requests, resource transfers and effects share one global.
use super::{identity::roles, imp::PhilosopherArguments};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const THINK_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/think",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);
pub const EAT_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/eat",
    ObjectId(5),
    FdSpec::new(5, 66, 1),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);
pub type Eat<const RESOURCE: u8> = g::Msg<RESOURCE, PhilosopherArguments>;
pub type Meal<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type FileExit = g::Msg<241, i32>;
pub const EAT_A_ARGUMENTS: u16 = 700;
pub const EAT_B_ARGUMENTS: u16 = 701;
pub const EAT_C_ARGUMENTS: u16 = 702;
pub const EAT_D_ARGUMENTS: u16 = 703;
pub const EAT_E_ARGUMENTS: u16 = 704;
pub const EAT_F_ARGUMENTS: u16 = 705;
pub const EAT_G_ARGUMENTS: u16 = 706;
pub const THINK_A_ARGUMENTS: u16 = 742;
pub const THINK_B_ARGUMENTS: u16 = 743;
pub const THINK_C_ARGUMENTS: u16 = 744;
pub const THINK_D_ARGUMENTS: u16 = 745;
pub const THINK_E_ARGUMENTS: u16 = 746;
pub const THINK_F_ARGUMENTS: u16 = 747;
pub const THINK_G_ARGUMENTS: u16 = 748;

// These are repeatedly used source fragments; no runtime protocol is hidden here.
fn file_io<const FILE: u8, const OWNER: u8>() -> impl Projectable {
    let prestat = g::seq(
        g::send::<FILE, OWNER, protocol::FdPrestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdPrestatGetRetMsg>(),
    );
    let prestat_name = g::seq(
        g::send::<FILE, OWNER, protocol::FdPrestatDirNameReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdPrestatDirNameRetMsg>(),
    );
    let directory = g::seq(
        g::send::<FILE, OWNER, protocol::FdReaddirReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdReaddirRetMsg>(),
    );
    let open = g::seq(
        g::send::<FILE, OWNER, protocol::PathOpenReqMsg>(),
        g::send::<OWNER, FILE, protocol::PathOpenRetMsg>(),
    );
    let write = g::seq(
        g::send::<FILE, OWNER, protocol::FdWriteObjectReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdWriteObjectRetMsg>(),
    );
    let read = g::seq(
        g::send::<FILE, OWNER, protocol::FdReadReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdReadRetMsg>(),
    );
    let stat = g::seq(
        g::send::<FILE, OWNER, protocol::FdFdstatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdFdstatGetRetMsg>(),
    );
    let file_stat = g::seq(
        g::send::<FILE, OWNER, protocol::FdFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdFilestatGetRetMsg>(),
    );
    let path_stat = g::seq(
        g::send::<FILE, OWNER, protocol::PathFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::PathFilestatGetRetMsg>(),
    );
    let close = g::seq(
        g::send::<FILE, OWNER, protocol::FdCloseReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdCloseRetMsg>(),
    );
    // Object close belongs to the business operation below, never this I/O roll.
    let io = g::route(
        open,
        g::route(
            write,
            g::route(
                read,
                g::route(stat, g::route(file_stat, g::route(path_stat, close))),
            ),
        ),
    );
    g::route(prestat, g::route(prestat_name, g::route(directory, io))).roll()
}
pub struct Programs9E {
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
}
impl Programs9E {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9E","programs":[rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file)]})
    }
}
pub fn project_9e() -> Programs9E {
    use roles::*;
    let bootstrap = g::par(
        g::par(
            g::par(
                g::send::<THINK_A, A_WASI, Loan<A_THINK_FILE>>(),
                g::send::<A, A_WASI, Loan<A_EAT_FILE>>(),
            ),
            g::par(
                g::send::<THINK_B, B_WASI, Loan<B_THINK_FILE>>(),
                g::send::<B, B_WASI, Loan<B_EAT_FILE>>(),
            ),
        ),
        g::par(
            g::send::<THINK_C, C_WASI, Loan<C_THINK_FILE>>(),
            g::send::<C, C_WASI, Loan<C_EAT_FILE>>(),
        ),
    );
    let io = g::par(
        g::par(
            g::par(
                g::par(
                    crate::shell::global::private_io::<A_WASI, A_IO>(),
                    file_io::<A_THINK_FILE, THINK_A>(),
                ),
                file_io::<A_EAT_FILE, A>(),
            ),
            g::par(
                crate::shell::global::private_io::<B_WASI, B_IO>(),
                file_io::<B_THINK_FILE, THINK_B>(),
            ),
        ),
        g::par(
            g::par(
                file_io::<B_EAT_FILE, B>(),
                crate::shell::global::private_io::<C_WASI, C_IO>(),
            ),
            g::par(
                file_io::<C_THINK_FILE, THINK_C>(),
                file_io::<C_EAT_FILE, C>(),
            ),
        ),
    );
    let thoughts = g::par(
        g::par(
            {
                let request = g::send::<A_THINK_FILE, THINK_A, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_A_ARGUMENTS>(),
                )
            },
            {
                let request = g::send::<B_THINK_FILE, THINK_B, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_B_ARGUMENTS>(),
                )
            },
        ),
        {
            let request = g::send::<C_THINK_FILE, THINK_C, protocol::FdCloseObjectReqMsg>();
            let approved = g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseObjectRetMsg>();
            let retry = g::seq(
                g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                request,
            );
            g::seq(
                request,
                g::route(approved, g::seq(retry.roll(), approved)).resolve::<THINK_C_ARGUMENTS>(),
            )
        },
    );
    let meals = g::seq(
        {
            let request = g::send::<A_EAT_FILE, A, protocol::FdCloseObjectReqMsg>();
            let left = g::send::<A, FORK_A, Eat<FORK_A_RESOURCE>>();
            let right = g::send::<A, FORK_C, Eat<FORK_C_RESOURCE>>();
            let approved = g::seq(
                g::par(left, right),
                g::seq(
                    g::par(
                        g::seq(
                            g::send::<FORK_A_RESOURCE, A, Eat<FORK_A_RESOURCE>>(),
                            g::send::<FORK_A, A, Loan<FORK_A_RESOURCE>>(),
                        ),
                        g::seq(
                            g::send::<FORK_C_RESOURCE, A, Eat<FORK_C_RESOURCE>>(),
                            g::send::<FORK_C, A, Loan<FORK_C_RESOURCE>>(),
                        ),
                    ),
                    g::seq(
                        g::par(
                            g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                            g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                        ),
                        g::seq(
                            g::par(
                                g::send::<A, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                g::send::<A, FORK_C, Loan<FORK_C_RESOURCE>>(),
                            ),
                            g::send::<A, A_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                        ),
                    ),
                ),
            );
            let retry = g::seq(
                g::send::<A, A_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                request,
            );
            g::seq(
                request,
                g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_A_ARGUMENTS>(),
            )
        },
        g::seq(
            {
                let request = g::send::<B_EAT_FILE, B, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<B, FORK_B, Eat<FORK_B_RESOURCE>>();
                let right = g::send::<B, FORK_A, Eat<FORK_A_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_B_RESOURCE, B, Eat<FORK_B_RESOURCE>>(),
                                g::send::<FORK_B, B, Loan<FORK_B_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_A_RESOURCE, B, Eat<FORK_A_RESOURCE>>(),
                                g::send::<FORK_A, B, Loan<FORK_A_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                                g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<B, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                    g::send::<B, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                ),
                                g::send::<B, B_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<B, B_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_B_ARGUMENTS>(),
                )
            },
            {
                let request = g::send::<C_EAT_FILE, C, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<C, FORK_C, Eat<FORK_C_RESOURCE>>();
                let right = g::send::<C, FORK_B, Eat<FORK_B_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_C_RESOURCE, C, Eat<FORK_C_RESOURCE>>(),
                                g::send::<FORK_C, C, Loan<FORK_C_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_B_RESOURCE, C, Eat<FORK_B_RESOURCE>>(),
                                g::send::<FORK_B, C, Loan<FORK_B_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                                g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<C, FORK_C, Loan<FORK_C_RESOURCE>>(),
                                    g::send::<C, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                ),
                                g::send::<C, C_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<C, C_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_C_ARGUMENTS>(),
                )
            },
        ),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                g::send::<A_EAT_FILE, A, FileExit>(),
            ),
            g::send::<B_THINK_FILE, THINK_B, FileExit>(),
        ),
        g::par(
            g::par(
                g::send::<B_EAT_FILE, B, FileExit>(),
                g::send::<C_THINK_FILE, THINK_C, FileExit>(),
            ),
            g::send::<C_EAT_FILE, C, FileExit>(),
        ),
    );
    let returns = g::par(
        g::par(
            g::send::<A_WASI, A_IO, crate::shell::global::Exit>(),
            g::send::<B_WASI, B_IO, crate::shell::global::Exit>(),
        ),
        g::send::<C_WASI, C_IO, crate::shell::global::Exit>(),
    );
    let global = g::seq(
        bootstrap,
        g::seq(g::par(io, g::seq(thoughts, g::seq(meals, exits))), returns),
    );
    Programs9E {
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
    }
}

pub struct Programs9M {
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
    pub d: RoleProgram<{ roles::D }>,
    pub think_d: RoleProgram<{ roles::THINK_D }>,
    pub fork_d: RoleProgram<{ roles::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles::D_WASI }>,
    pub d_io: RoleProgram<{ roles::D_IO }>,
    pub resource_d: RoleProgram<{ roles::FORK_D_RESOURCE }>,
    pub d_think_file: RoleProgram<{ roles::D_THINK_FILE }>,
    pub d_eat_file: RoleProgram<{ roles::D_EAT_FILE }>,
    pub e: RoleProgram<{ roles::E }>,
    pub think_e: RoleProgram<{ roles::THINK_E }>,
    pub fork_e: RoleProgram<{ roles::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles::E_WASI }>,
    pub e_io: RoleProgram<{ roles::E_IO }>,
    pub resource_e: RoleProgram<{ roles::FORK_E_RESOURCE }>,
    pub e_think_file: RoleProgram<{ roles::E_THINK_FILE }>,
    pub e_eat_file: RoleProgram<{ roles::E_EAT_FILE }>,
}
impl Programs9M {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9M","programs":[rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file),rows(&self.d),rows(&self.think_d),rows(&self.fork_d),rows(&self.d_wasi),rows(&self.d_io),rows(&self.resource_d),rows(&self.d_think_file),rows(&self.d_eat_file),rows(&self.e),rows(&self.think_e),rows(&self.fork_e),rows(&self.e_wasi),rows(&self.e_io),rows(&self.resource_e),rows(&self.e_think_file),rows(&self.e_eat_file)]})
    }
}
pub fn project_9m() -> Programs9M {
    use roles::*;
    let bootstrap = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<THINK_A, A_WASI, Loan<A_THINK_FILE>>(),
                    g::send::<A, A_WASI, Loan<A_EAT_FILE>>(),
                ),
                g::par(
                    g::send::<THINK_B, B_WASI, Loan<B_THINK_FILE>>(),
                    g::send::<B, B_WASI, Loan<B_EAT_FILE>>(),
                ),
            ),
            g::par(
                g::send::<THINK_C, C_WASI, Loan<C_THINK_FILE>>(),
                g::send::<C, C_WASI, Loan<C_EAT_FILE>>(),
            ),
        ),
        g::par(
            g::par(
                g::send::<THINK_D, D_WASI, Loan<D_THINK_FILE>>(),
                g::send::<D, D_WASI, Loan<D_EAT_FILE>>(),
            ),
            g::par(
                g::send::<THINK_E, E_WASI, Loan<E_THINK_FILE>>(),
                g::send::<E, E_WASI, Loan<E_EAT_FILE>>(),
            ),
        ),
    );
    let io = g::par(
        g::par(
            g::par(
                g::par(
                    crate::shell::global::private_io::<A_WASI, A_IO>(),
                    file_io::<A_THINK_FILE, THINK_A>(),
                ),
                g::par(
                    file_io::<A_EAT_FILE, A>(),
                    crate::shell::global::private_io::<B_WASI, B_IO>(),
                ),
            ),
            g::par(
                g::par(
                    file_io::<B_THINK_FILE, THINK_B>(),
                    file_io::<B_EAT_FILE, B>(),
                ),
                g::par(
                    crate::shell::global::private_io::<C_WASI, C_IO>(),
                    file_io::<C_THINK_FILE, THINK_C>(),
                ),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    file_io::<C_EAT_FILE, C>(),
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                ),
                g::par(
                    file_io::<D_THINK_FILE, THINK_D>(),
                    file_io::<D_EAT_FILE, D>(),
                ),
            ),
            g::par(
                g::par(
                    crate::shell::global::private_io::<E_WASI, E_IO>(),
                    file_io::<E_THINK_FILE, THINK_E>(),
                ),
                file_io::<E_EAT_FILE, E>(),
            ),
        ),
    );
    let thoughts = g::par(
        g::par(
            g::par(
                {
                    let request = g::send::<A_THINK_FILE, THINK_A, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_A_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<B_THINK_FILE, THINK_B, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_B_ARGUMENTS>(),
                    )
                },
            ),
            {
                let request = g::send::<C_THINK_FILE, THINK_C, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_C_ARGUMENTS>(),
                )
            },
        ),
        g::par(
            {
                let request = g::send::<D_THINK_FILE, THINK_D, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_D, D_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_D, D_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_D_ARGUMENTS>(),
                )
            },
            {
                let request = g::send::<E_THINK_FILE, THINK_E, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_E, E_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_E, E_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_E_ARGUMENTS>(),
                )
            },
        ),
    );
    let meals = g::seq(
        g::par(
            {
                let request = g::send::<A_EAT_FILE, A, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<A, FORK_A, Eat<FORK_A_RESOURCE>>();
                let right = g::send::<A, FORK_E, Eat<FORK_E_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_A_RESOURCE, A, Eat<FORK_A_RESOURCE>>(),
                                g::send::<FORK_A, A, Loan<FORK_A_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_E_RESOURCE, A, Eat<FORK_E_RESOURCE>>(),
                                g::send::<FORK_E, A, Loan<FORK_E_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                                g::send::<FORK_E_RESOURCE, FORK_E, Meal<FORK_E_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<A, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                    g::send::<A, FORK_E, Loan<FORK_E_RESOURCE>>(),
                                ),
                                g::send::<A, A_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<A, A_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_A_ARGUMENTS>(),
                )
            },
            {
                let request = g::send::<C_EAT_FILE, C, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<C, FORK_C, Eat<FORK_C_RESOURCE>>();
                let right = g::send::<C, FORK_B, Eat<FORK_B_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_C_RESOURCE, C, Eat<FORK_C_RESOURCE>>(),
                                g::send::<FORK_C, C, Loan<FORK_C_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_B_RESOURCE, C, Eat<FORK_B_RESOURCE>>(),
                                g::send::<FORK_B, C, Loan<FORK_B_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                                g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<C, FORK_C, Loan<FORK_C_RESOURCE>>(),
                                    g::send::<C, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                ),
                                g::send::<C, C_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<C, C_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_C_ARGUMENTS>(),
                )
            },
        ),
        g::seq(
            g::par(
                {
                    let request = g::send::<B_EAT_FILE, B, protocol::FdCloseObjectReqMsg>();
                    let left = g::send::<B, FORK_B, Eat<FORK_B_RESOURCE>>();
                    let right = g::send::<B, FORK_A, Eat<FORK_A_RESOURCE>>();
                    let approved = g::seq(
                        g::par(left, right),
                        g::seq(
                            g::par(
                                g::seq(
                                    g::send::<FORK_B_RESOURCE, B, Eat<FORK_B_RESOURCE>>(),
                                    g::send::<FORK_B, B, Loan<FORK_B_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::send::<FORK_A_RESOURCE, B, Eat<FORK_A_RESOURCE>>(),
                                    g::send::<FORK_A, B, Loan<FORK_A_RESOURCE>>(),
                                ),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                                    g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<B, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                        g::send::<B, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                    ),
                                    g::send::<B, B_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                ),
                            ),
                        ),
                    );
                    let retry = g::seq(
                        g::send::<B, B_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<EAT_B_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<D_EAT_FILE, D, protocol::FdCloseObjectReqMsg>();
                    let left = g::send::<D, FORK_D, Eat<FORK_D_RESOURCE>>();
                    let right = g::send::<D, FORK_C, Eat<FORK_C_RESOURCE>>();
                    let approved = g::seq(
                        g::par(left, right),
                        g::seq(
                            g::par(
                                g::seq(
                                    g::send::<FORK_D_RESOURCE, D, Eat<FORK_D_RESOURCE>>(),
                                    g::send::<FORK_D, D, Loan<FORK_D_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::send::<FORK_C_RESOURCE, D, Eat<FORK_C_RESOURCE>>(),
                                    g::send::<FORK_C, D, Loan<FORK_C_RESOURCE>>(),
                                ),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<FORK_D_RESOURCE, FORK_D, Meal<FORK_D_RESOURCE>>(),
                                    g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<D, FORK_D, Loan<FORK_D_RESOURCE>>(),
                                        g::send::<D, FORK_C, Loan<FORK_C_RESOURCE>>(),
                                    ),
                                    g::send::<D, D_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                ),
                            ),
                        ),
                    );
                    let retry = g::seq(
                        g::send::<D, D_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<EAT_D_ARGUMENTS>(),
                    )
                },
            ),
            {
                let request = g::send::<E_EAT_FILE, E, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<E, FORK_E, Eat<FORK_E_RESOURCE>>();
                let right = g::send::<E, FORK_D, Eat<FORK_D_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_E_RESOURCE, E, Eat<FORK_E_RESOURCE>>(),
                                g::send::<FORK_E, E, Loan<FORK_E_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_D_RESOURCE, E, Eat<FORK_D_RESOURCE>>(),
                                g::send::<FORK_D, E, Loan<FORK_D_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_E_RESOURCE, FORK_E, Meal<FORK_E_RESOURCE>>(),
                                g::send::<FORK_D_RESOURCE, FORK_D, Meal<FORK_D_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<E, FORK_E, Loan<FORK_E_RESOURCE>>(),
                                    g::send::<E, FORK_D, Loan<FORK_D_RESOURCE>>(),
                                ),
                                g::send::<E, E_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<E, E_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_E_ARGUMENTS>(),
                )
            },
        ),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                    g::send::<A_EAT_FILE, A, FileExit>(),
                ),
                g::send::<B_THINK_FILE, THINK_B, FileExit>(),
            ),
            g::par(
                g::send::<B_EAT_FILE, B, FileExit>(),
                g::send::<C_THINK_FILE, THINK_C, FileExit>(),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::send::<C_EAT_FILE, C, FileExit>(),
                    g::send::<D_THINK_FILE, THINK_D, FileExit>(),
                ),
                g::send::<D_EAT_FILE, D, FileExit>(),
            ),
            g::par(
                g::send::<E_THINK_FILE, THINK_E, FileExit>(),
                g::send::<E_EAT_FILE, E, FileExit>(),
            ),
        ),
    );
    let returns = g::par(
        g::par(
            g::par(
                g::send::<A_WASI, A_IO, crate::shell::global::Exit>(),
                g::send::<B_WASI, B_IO, crate::shell::global::Exit>(),
            ),
            g::send::<C_WASI, C_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::send::<D_WASI, D_IO, crate::shell::global::Exit>(),
            g::send::<E_WASI, E_IO, crate::shell::global::Exit>(),
        ),
    );
    let global = g::seq(
        bootstrap,
        g::seq(g::par(io, g::seq(thoughts, g::seq(meals, exits))), returns),
    );
    Programs9M {
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
        d: project(&global),
        think_d: project(&global),
        fork_d: project(&global),
        d_wasi: project(&global),
        d_io: project(&global),
        resource_d: project(&global),
        d_think_file: project(&global),
        d_eat_file: project(&global),
        e: project(&global),
        think_e: project(&global),
        fork_e: project(&global),
        e_wasi: project(&global),
        e_io: project(&global),
        resource_e: project(&global),
        e_think_file: project(&global),
        e_eat_file: project(&global),
    }
}

pub struct Programs9H {
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
    pub d: RoleProgram<{ roles::D }>,
    pub think_d: RoleProgram<{ roles::THINK_D }>,
    pub fork_d: RoleProgram<{ roles::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles::D_WASI }>,
    pub d_io: RoleProgram<{ roles::D_IO }>,
    pub resource_d: RoleProgram<{ roles::FORK_D_RESOURCE }>,
    pub d_think_file: RoleProgram<{ roles::D_THINK_FILE }>,
    pub d_eat_file: RoleProgram<{ roles::D_EAT_FILE }>,
    pub e: RoleProgram<{ roles::E }>,
    pub think_e: RoleProgram<{ roles::THINK_E }>,
    pub fork_e: RoleProgram<{ roles::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles::E_WASI }>,
    pub e_io: RoleProgram<{ roles::E_IO }>,
    pub resource_e: RoleProgram<{ roles::FORK_E_RESOURCE }>,
    pub e_think_file: RoleProgram<{ roles::E_THINK_FILE }>,
    pub e_eat_file: RoleProgram<{ roles::E_EAT_FILE }>,
    pub f: RoleProgram<{ roles::F }>,
    pub think_f: RoleProgram<{ roles::THINK_F }>,
    pub fork_f: RoleProgram<{ roles::FORK_F }>,
    pub f_wasi: RoleProgram<{ roles::F_WASI }>,
    pub f_io: RoleProgram<{ roles::F_IO }>,
    pub resource_f: RoleProgram<{ roles::FORK_F_RESOURCE }>,
    pub f_think_file: RoleProgram<{ roles::F_THINK_FILE }>,
    pub f_eat_file: RoleProgram<{ roles::F_EAT_FILE }>,
    pub g: RoleProgram<{ roles::G }>,
    pub think_g: RoleProgram<{ roles::THINK_G }>,
    pub fork_g: RoleProgram<{ roles::FORK_G }>,
    pub g_wasi: RoleProgram<{ roles::G_WASI }>,
    pub g_io: RoleProgram<{ roles::G_IO }>,
    pub resource_g: RoleProgram<{ roles::FORK_G_RESOURCE }>,
    pub g_think_file: RoleProgram<{ roles::G_THINK_FILE }>,
    pub g_eat_file: RoleProgram<{ roles::G_EAT_FILE }>,
}
impl Programs9H {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9H","programs":[rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file),rows(&self.d),rows(&self.think_d),rows(&self.fork_d),rows(&self.d_wasi),rows(&self.d_io),rows(&self.resource_d),rows(&self.d_think_file),rows(&self.d_eat_file),rows(&self.e),rows(&self.think_e),rows(&self.fork_e),rows(&self.e_wasi),rows(&self.e_io),rows(&self.resource_e),rows(&self.e_think_file),rows(&self.e_eat_file),rows(&self.f),rows(&self.think_f),rows(&self.fork_f),rows(&self.f_wasi),rows(&self.f_io),rows(&self.resource_f),rows(&self.f_think_file),rows(&self.f_eat_file),rows(&self.g),rows(&self.think_g),rows(&self.fork_g),rows(&self.g_wasi),rows(&self.g_io),rows(&self.resource_g),rows(&self.g_think_file),rows(&self.g_eat_file)]})
    }
}
pub fn project_9h() -> Programs9H {
    use roles::*;
    let bootstrap = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<THINK_A, A_WASI, Loan<A_THINK_FILE>>(),
                    g::send::<A, A_WASI, Loan<A_EAT_FILE>>(),
                ),
                g::par(
                    g::send::<THINK_B, B_WASI, Loan<B_THINK_FILE>>(),
                    g::send::<B, B_WASI, Loan<B_EAT_FILE>>(),
                ),
            ),
            g::par(
                g::par(
                    g::send::<THINK_C, C_WASI, Loan<C_THINK_FILE>>(),
                    g::send::<C, C_WASI, Loan<C_EAT_FILE>>(),
                ),
                g::par(
                    g::send::<THINK_D, D_WASI, Loan<D_THINK_FILE>>(),
                    g::send::<D, D_WASI, Loan<D_EAT_FILE>>(),
                ),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::send::<THINK_E, E_WASI, Loan<E_THINK_FILE>>(),
                    g::send::<E, E_WASI, Loan<E_EAT_FILE>>(),
                ),
                g::par(
                    g::send::<THINK_F, F_WASI, Loan<F_THINK_FILE>>(),
                    g::send::<F, F_WASI, Loan<F_EAT_FILE>>(),
                ),
            ),
            g::par(
                g::send::<THINK_G, G_WASI, Loan<G_THINK_FILE>>(),
                g::send::<G, G_WASI, Loan<G_EAT_FILE>>(),
            ),
        ),
    );
    let io = g::par(
        g::par(
            g::par(
                g::par(
                    g::par(
                        crate::shell::global::private_io::<A_WASI, A_IO>(),
                        file_io::<A_THINK_FILE, THINK_A>(),
                    ),
                    file_io::<A_EAT_FILE, A>(),
                ),
                g::par(
                    g::par(
                        crate::shell::global::private_io::<B_WASI, B_IO>(),
                        file_io::<B_THINK_FILE, THINK_B>(),
                    ),
                    file_io::<B_EAT_FILE, B>(),
                ),
            ),
            g::par(
                g::par(
                    g::par(
                        crate::shell::global::private_io::<C_WASI, C_IO>(),
                        file_io::<C_THINK_FILE, THINK_C>(),
                    ),
                    file_io::<C_EAT_FILE, C>(),
                ),
                g::par(
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                    file_io::<D_THINK_FILE, THINK_D>(),
                ),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::par(
                        file_io::<D_EAT_FILE, D>(),
                        crate::shell::global::private_io::<E_WASI, E_IO>(),
                    ),
                    file_io::<E_THINK_FILE, THINK_E>(),
                ),
                g::par(
                    file_io::<E_EAT_FILE, E>(),
                    crate::shell::global::private_io::<F_WASI, F_IO>(),
                ),
            ),
            g::par(
                g::par(
                    g::par(
                        file_io::<F_THINK_FILE, THINK_F>(),
                        file_io::<F_EAT_FILE, F>(),
                    ),
                    crate::shell::global::private_io::<G_WASI, G_IO>(),
                ),
                g::par(
                    file_io::<G_THINK_FILE, THINK_G>(),
                    file_io::<G_EAT_FILE, G>(),
                ),
            ),
        ),
    );
    let thoughts = g::par(
        g::par(
            g::par(
                {
                    let request = g::send::<A_THINK_FILE, THINK_A, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_A, A_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_A_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<B_THINK_FILE, THINK_B, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_B, B_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_B_ARGUMENTS>(),
                    )
                },
            ),
            g::par(
                {
                    let request = g::send::<C_THINK_FILE, THINK_C, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_C, C_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_C_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<D_THINK_FILE, THINK_D, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_D, D_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_D, D_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_D_ARGUMENTS>(),
                    )
                },
            ),
        ),
        g::par(
            g::par(
                {
                    let request = g::send::<E_THINK_FILE, THINK_E, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_E, E_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_E, E_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_E_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<F_THINK_FILE, THINK_F, protocol::FdCloseObjectReqMsg>();
                    let approved =
                        g::send::<THINK_F, F_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                    let retry = g::seq(
                        g::send::<THINK_F, F_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<THINK_F_ARGUMENTS>(),
                    )
                },
            ),
            {
                let request = g::send::<G_THINK_FILE, THINK_G, protocol::FdCloseObjectReqMsg>();
                let approved = g::send::<THINK_G, G_THINK_FILE, protocol::FdCloseObjectRetMsg>();
                let retry = g::seq(
                    g::send::<THINK_G, G_THINK_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved))
                        .resolve::<THINK_G_ARGUMENTS>(),
                )
            },
        ),
    );
    let meals = g::seq(
        g::par(
            g::par(
                {
                    let request = g::send::<A_EAT_FILE, A, protocol::FdCloseObjectReqMsg>();
                    let left = g::send::<A, FORK_A, Eat<FORK_A_RESOURCE>>();
                    let right = g::send::<A, FORK_G, Eat<FORK_G_RESOURCE>>();
                    let approved = g::seq(
                        g::par(left, right),
                        g::seq(
                            g::par(
                                g::seq(
                                    g::send::<FORK_A_RESOURCE, A, Eat<FORK_A_RESOURCE>>(),
                                    g::send::<FORK_A, A, Loan<FORK_A_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::send::<FORK_G_RESOURCE, A, Eat<FORK_G_RESOURCE>>(),
                                    g::send::<FORK_G, A, Loan<FORK_G_RESOURCE>>(),
                                ),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                                    g::send::<FORK_G_RESOURCE, FORK_G, Meal<FORK_G_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<A, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                        g::send::<A, FORK_G, Loan<FORK_G_RESOURCE>>(),
                                    ),
                                    g::send::<A, A_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                ),
                            ),
                        ),
                    );
                    let retry = g::seq(
                        g::send::<A, A_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<EAT_A_ARGUMENTS>(),
                    )
                },
                {
                    let request = g::send::<C_EAT_FILE, C, protocol::FdCloseObjectReqMsg>();
                    let left = g::send::<C, FORK_C, Eat<FORK_C_RESOURCE>>();
                    let right = g::send::<C, FORK_B, Eat<FORK_B_RESOURCE>>();
                    let approved = g::seq(
                        g::par(left, right),
                        g::seq(
                            g::par(
                                g::seq(
                                    g::send::<FORK_C_RESOURCE, C, Eat<FORK_C_RESOURCE>>(),
                                    g::send::<FORK_C, C, Loan<FORK_C_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::send::<FORK_B_RESOURCE, C, Eat<FORK_B_RESOURCE>>(),
                                    g::send::<FORK_B, C, Loan<FORK_B_RESOURCE>>(),
                                ),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                                    g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<C, FORK_C, Loan<FORK_C_RESOURCE>>(),
                                        g::send::<C, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                    ),
                                    g::send::<C, C_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                ),
                            ),
                        ),
                    );
                    let retry = g::seq(
                        g::send::<C, C_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<EAT_C_ARGUMENTS>(),
                    )
                },
            ),
            {
                let request = g::send::<E_EAT_FILE, E, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<E, FORK_E, Eat<FORK_E_RESOURCE>>();
                let right = g::send::<E, FORK_D, Eat<FORK_D_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_E_RESOURCE, E, Eat<FORK_E_RESOURCE>>(),
                                g::send::<FORK_E, E, Loan<FORK_E_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_D_RESOURCE, E, Eat<FORK_D_RESOURCE>>(),
                                g::send::<FORK_D, E, Loan<FORK_D_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_E_RESOURCE, FORK_E, Meal<FORK_E_RESOURCE>>(),
                                g::send::<FORK_D_RESOURCE, FORK_D, Meal<FORK_D_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<E, FORK_E, Loan<FORK_E_RESOURCE>>(),
                                    g::send::<E, FORK_D, Loan<FORK_D_RESOURCE>>(),
                                ),
                                g::send::<E, E_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<E, E_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_E_ARGUMENTS>(),
                )
            },
        ),
        g::seq(
            g::par(
                g::par(
                    {
                        let request = g::send::<B_EAT_FILE, B, protocol::FdCloseObjectReqMsg>();
                        let left = g::send::<B, FORK_B, Eat<FORK_B_RESOURCE>>();
                        let right = g::send::<B, FORK_A, Eat<FORK_A_RESOURCE>>();
                        let approved = g::seq(
                            g::par(left, right),
                            g::seq(
                                g::par(
                                    g::seq(
                                        g::send::<FORK_B_RESOURCE, B, Eat<FORK_B_RESOURCE>>(),
                                        g::send::<FORK_B, B, Loan<FORK_B_RESOURCE>>(),
                                    ),
                                    g::seq(
                                        g::send::<FORK_A_RESOURCE, B, Eat<FORK_A_RESOURCE>>(),
                                        g::send::<FORK_A, B, Loan<FORK_A_RESOURCE>>(),
                                    ),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<FORK_B_RESOURCE, FORK_B, Meal<FORK_B_RESOURCE>>(),
                                        g::send::<FORK_A_RESOURCE, FORK_A, Meal<FORK_A_RESOURCE>>(),
                                    ),
                                    g::seq(
                                        g::par(
                                            g::send::<B, FORK_B, Loan<FORK_B_RESOURCE>>(),
                                            g::send::<B, FORK_A, Loan<FORK_A_RESOURCE>>(),
                                        ),
                                        g::send::<B, B_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                    ),
                                ),
                            ),
                        );
                        let retry = g::seq(
                            g::send::<B, B_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                            request,
                        );
                        g::seq(
                            request,
                            g::route(approved, g::seq(retry.roll(), approved))
                                .resolve::<EAT_B_ARGUMENTS>(),
                        )
                    },
                    {
                        let request = g::send::<D_EAT_FILE, D, protocol::FdCloseObjectReqMsg>();
                        let left = g::send::<D, FORK_D, Eat<FORK_D_RESOURCE>>();
                        let right = g::send::<D, FORK_C, Eat<FORK_C_RESOURCE>>();
                        let approved = g::seq(
                            g::par(left, right),
                            g::seq(
                                g::par(
                                    g::seq(
                                        g::send::<FORK_D_RESOURCE, D, Eat<FORK_D_RESOURCE>>(),
                                        g::send::<FORK_D, D, Loan<FORK_D_RESOURCE>>(),
                                    ),
                                    g::seq(
                                        g::send::<FORK_C_RESOURCE, D, Eat<FORK_C_RESOURCE>>(),
                                        g::send::<FORK_C, D, Loan<FORK_C_RESOURCE>>(),
                                    ),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<FORK_D_RESOURCE, FORK_D, Meal<FORK_D_RESOURCE>>(),
                                        g::send::<FORK_C_RESOURCE, FORK_C, Meal<FORK_C_RESOURCE>>(),
                                    ),
                                    g::seq(
                                        g::par(
                                            g::send::<D, FORK_D, Loan<FORK_D_RESOURCE>>(),
                                            g::send::<D, FORK_C, Loan<FORK_C_RESOURCE>>(),
                                        ),
                                        g::send::<D, D_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                    ),
                                ),
                            ),
                        );
                        let retry = g::seq(
                            g::send::<D, D_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                            request,
                        );
                        g::seq(
                            request,
                            g::route(approved, g::seq(retry.roll(), approved))
                                .resolve::<EAT_D_ARGUMENTS>(),
                        )
                    },
                ),
                {
                    let request = g::send::<F_EAT_FILE, F, protocol::FdCloseObjectReqMsg>();
                    let left = g::send::<F, FORK_F, Eat<FORK_F_RESOURCE>>();
                    let right = g::send::<F, FORK_E, Eat<FORK_E_RESOURCE>>();
                    let approved = g::seq(
                        g::par(left, right),
                        g::seq(
                            g::par(
                                g::seq(
                                    g::send::<FORK_F_RESOURCE, F, Eat<FORK_F_RESOURCE>>(),
                                    g::send::<FORK_F, F, Loan<FORK_F_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::send::<FORK_E_RESOURCE, F, Eat<FORK_E_RESOURCE>>(),
                                    g::send::<FORK_E, F, Loan<FORK_E_RESOURCE>>(),
                                ),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<FORK_F_RESOURCE, FORK_F, Meal<FORK_F_RESOURCE>>(),
                                    g::send::<FORK_E_RESOURCE, FORK_E, Meal<FORK_E_RESOURCE>>(),
                                ),
                                g::seq(
                                    g::par(
                                        g::send::<F, FORK_F, Loan<FORK_F_RESOURCE>>(),
                                        g::send::<F, FORK_E, Loan<FORK_E_RESOURCE>>(),
                                    ),
                                    g::send::<F, F_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                                ),
                            ),
                        ),
                    );
                    let retry = g::seq(
                        g::send::<F, F_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                        request,
                    );
                    g::seq(
                        request,
                        g::route(approved, g::seq(retry.roll(), approved))
                            .resolve::<EAT_F_ARGUMENTS>(),
                    )
                },
            ),
            {
                let request = g::send::<G_EAT_FILE, G, protocol::FdCloseObjectReqMsg>();
                let left = g::send::<G, FORK_G, Eat<FORK_G_RESOURCE>>();
                let right = g::send::<G, FORK_F, Eat<FORK_F_RESOURCE>>();
                let approved = g::seq(
                    g::par(left, right),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<FORK_G_RESOURCE, G, Eat<FORK_G_RESOURCE>>(),
                                g::send::<FORK_G, G, Loan<FORK_G_RESOURCE>>(),
                            ),
                            g::seq(
                                g::send::<FORK_F_RESOURCE, G, Eat<FORK_F_RESOURCE>>(),
                                g::send::<FORK_F, G, Loan<FORK_F_RESOURCE>>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<FORK_G_RESOURCE, FORK_G, Meal<FORK_G_RESOURCE>>(),
                                g::send::<FORK_F_RESOURCE, FORK_F, Meal<FORK_F_RESOURCE>>(),
                            ),
                            g::seq(
                                g::par(
                                    g::send::<G, FORK_G, Loan<FORK_G_RESOURCE>>(),
                                    g::send::<G, FORK_F, Loan<FORK_F_RESOURCE>>(),
                                ),
                                g::send::<G, G_EAT_FILE, protocol::FdCloseObjectRetMsg>(),
                            ),
                        ),
                    ),
                );
                let retry = g::seq(
                    g::send::<G, G_EAT_FILE, protocol::FdCloseRejectedRetMsg>(),
                    request,
                );
                g::seq(
                    request,
                    g::route(approved, g::seq(retry.roll(), approved)).resolve::<EAT_G_ARGUMENTS>(),
                )
            },
        ),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                    g::send::<A_EAT_FILE, A, FileExit>(),
                ),
                g::par(
                    g::send::<B_THINK_FILE, THINK_B, FileExit>(),
                    g::send::<B_EAT_FILE, B, FileExit>(),
                ),
            ),
            g::par(
                g::par(
                    g::send::<C_THINK_FILE, THINK_C, FileExit>(),
                    g::send::<C_EAT_FILE, C, FileExit>(),
                ),
                g::send::<D_THINK_FILE, THINK_D, FileExit>(),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::send::<D_EAT_FILE, D, FileExit>(),
                    g::send::<E_THINK_FILE, THINK_E, FileExit>(),
                ),
                g::par(
                    g::send::<E_EAT_FILE, E, FileExit>(),
                    g::send::<F_THINK_FILE, THINK_F, FileExit>(),
                ),
            ),
            g::par(
                g::par(
                    g::send::<F_EAT_FILE, F, FileExit>(),
                    g::send::<G_THINK_FILE, THINK_G, FileExit>(),
                ),
                g::send::<G_EAT_FILE, G, FileExit>(),
            ),
        ),
    );
    let returns = g::par(
        g::par(
            g::par(
                g::send::<A_WASI, A_IO, crate::shell::global::Exit>(),
                g::send::<B_WASI, B_IO, crate::shell::global::Exit>(),
            ),
            g::par(
                g::send::<C_WASI, C_IO, crate::shell::global::Exit>(),
                g::send::<D_WASI, D_IO, crate::shell::global::Exit>(),
            ),
        ),
        g::par(
            g::par(
                g::send::<E_WASI, E_IO, crate::shell::global::Exit>(),
                g::send::<F_WASI, F_IO, crate::shell::global::Exit>(),
            ),
            g::send::<G_WASI, G_IO, crate::shell::global::Exit>(),
        ),
    );
    let global = g::seq(
        bootstrap,
        g::seq(g::par(io, g::seq(thoughts, g::seq(meals, exits))), returns),
    );
    Programs9H {
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
        d: project(&global),
        think_d: project(&global),
        fork_d: project(&global),
        d_wasi: project(&global),
        d_io: project(&global),
        resource_d: project(&global),
        d_think_file: project(&global),
        d_eat_file: project(&global),
        e: project(&global),
        think_e: project(&global),
        fork_e: project(&global),
        e_wasi: project(&global),
        e_io: project(&global),
        resource_e: project(&global),
        e_think_file: project(&global),
        e_eat_file: project(&global),
        f: project(&global),
        think_f: project(&global),
        fork_f: project(&global),
        f_wasi: project(&global),
        f_io: project(&global),
        resource_f: project(&global),
        f_think_file: project(&global),
        f_eat_file: project(&global),
        g: project(&global),
        think_g: project(&global),
        fork_g: project(&global),
        g_wasi: project(&global),
        g_io: project(&global),
        resource_g: project(&global),
        g_think_file: project(&global),
        g_eat_file: project(&global),
    }
}
