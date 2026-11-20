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
    FdSpec::new(4, 66),
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
    FdSpec::new(5, 66),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);
pub type Eat<const RESOURCE: u8> = g::Msg<RESOURCE, PhilosopherArguments>;
pub type ForkRequest<'a, const RESOURCE: u8, const DINER: u8> =
    g::Msg<RESOURCE, super::imp::ForkRequest<'a, DINER>>;
pub type Meal<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type CloseAuthority<'run, const RESOURCE: u8> = g::Msg<165, Endpoint<'run, RESOURCE>>;
pub type FileExit = g::Msg<241, i32>;
pub const EAT_A_ARGUMENTS: u16 = 700;
pub const EAT_B_ARGUMENTS: u16 = 701;
pub const EAT_C_ARGUMENTS: u16 = 702;
pub const EAT_D_ARGUMENTS: u16 = 703;
pub const EAT_E_ARGUMENTS: u16 = 704;
pub const EAT_F_ARGUMENTS: u16 = 705;
pub const EAT_G_ARGUMENTS: u16 = 706;

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
pub type Think<'a, const VIEW: u8> =
    g::Msg<{ super::identity::THINK_REQUEST }, super::imp::Thought<'a, VIEW>>;
pub type ThinkCompleted = g::Msg<{ super::identity::THINK_COMPLETED }, ()>;

fn thinking<const FILE: u8, const OWNER: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, Think<OWNER>>(),
        g::send::<OWNER, FILE, ThinkCompleted>(),
    )
    .roll()
}

// Each diner can request its place after its own successful think and object close.
pub type TableMeal<const DINER: u8> = g::Msg<DINER, PhilosopherArguments>;

fn ready_to_eat<const FILE: u8, const CLOSE: u8, const RESOLVER: u16>() -> impl Projectable {
    let request = g::send::<FILE, CLOSE, protocol::FdCloseObjectReqMsg>();
    let ready = g::seq(
        g::send::<CLOSE, FILE, protocol::FdCloseObjectRetMsg>(),
        g::send::<CLOSE, { roles::MEAL_REQUESTS }, TableMeal<CLOSE>>(),
    );
    let retry = g::seq(
        g::send::<CLOSE, FILE, protocol::FdCloseRejectedRetMsg>(),
        request,
    );
    g::seq(
        request,
        g::route(ready, g::seq(retry.roll(), ready)).resolve::<RESOLVER>(),
    )
}

fn meal<
    const CLOSE: u8,
    const LEFT: u8,
    const RIGHT: u8,
    const LEFT_FORK: u8,
    const RIGHT_FORK: u8,
>() -> impl Projectable + Copy {
    let requests = g::par(
        g::send::<CLOSE, LEFT, Eat<LEFT>>(),
        g::send::<CLOSE, RIGHT, Eat<RIGHT_FORK>>(),
    );
    let loans = g::par(
        g::seq(
            g::send::<LEFT_FORK, CLOSE, ForkRequest<LEFT_FORK, CLOSE>>(),
            g::send::<LEFT, CLOSE, Loan<LEFT_FORK>>(),
        ),
        g::seq(
            g::send::<RIGHT_FORK, CLOSE, ForkRequest<RIGHT_FORK, CLOSE>>(),
            g::send::<RIGHT, CLOSE, Loan<RIGHT_FORK>>(),
        ),
    );
    let results = g::par(
        g::send::<LEFT_FORK, LEFT, Meal<LEFT_FORK>>(),
        g::send::<RIGHT_FORK, RIGHT, Meal<RIGHT_FORK>>(),
    );
    let returns = g::par(
        g::send::<CLOSE, LEFT, Loan<LEFT_FORK>>(),
        g::send::<CLOSE, RIGHT, Loan<RIGHT_FORK>>(),
    );
    g::seq(
        g::send::<{ roles::TABLE }, CLOSE, TableMeal<CLOSE>>(),
        g::seq(requests, g::seq(loans, g::seq(results, returns))),
    )
}

pub struct Programs9E {
    pub requests: RoleProgram<{ roles::MEAL_REQUESTS }>,
    pub table: RoleProgram<{ roles::TABLE }>,
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub eat_close_a: RoleProgram<{ roles::A_EAT_CLOSE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub eat_close_b: RoleProgram<{ roles::B_EAT_CLOSE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
    pub eat_close_c: RoleProgram<{ roles::C_EAT_CLOSE }>,
}
impl Programs9E {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9E","programs":[rows(&self.requests),rows(&self.table),rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.eat_close_a),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.eat_close_b),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file),rows(&self.eat_close_c)]})
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
    let ready_a = g::seq(
        thinking::<A_THINK_FILE, THINK_A>(),
        ready_to_eat::<A_EAT_FILE, A_EAT_CLOSE, EAT_A_ARGUMENTS>(),
    );
    let ready_b = g::seq(
        thinking::<B_THINK_FILE, THINK_B>(),
        ready_to_eat::<B_EAT_FILE, B_EAT_CLOSE, EAT_B_ARGUMENTS>(),
    );
    let ready_c = g::seq(
        thinking::<C_THINK_FILE, THINK_C>(),
        ready_to_eat::<C_EAT_FILE, C_EAT_CLOSE, EAT_C_ARGUMENTS>(),
    );
    let ready = g::par(ready_a, g::par(ready_b, ready_c));
    let a = meal::<A_EAT_CLOSE, FORK_A, FORK_C, FORK_A_RESOURCE, FORK_C_RESOURCE>();
    let b = meal::<B_EAT_CLOSE, FORK_B, FORK_A, FORK_B_RESOURCE, FORK_A_RESOURCE>();
    let c = meal::<C_EAT_CLOSE, FORK_C, FORK_B, FORK_C_RESOURCE, FORK_B_RESOURCE>();
    let meals = g::route(
        g::seq(a, g::route(g::seq(b, c), g::seq(c, b))),
        g::route(
            g::seq(b, g::route(g::seq(a, c), g::seq(c, a))),
            g::seq(c, g::route(g::seq(a, b), g::seq(b, a))),
        ),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                g::seq(
                    g::send::<A, A_EAT_FILE, CloseAuthority<A_EAT_CLOSE>>(),
                    g::send::<A_EAT_FILE, A, FileExit>(),
                ),
            ),
            g::send::<B_THINK_FILE, THINK_B, FileExit>(),
        ),
        g::par(
            g::par(
                g::seq(
                    g::send::<B, B_EAT_FILE, CloseAuthority<B_EAT_CLOSE>>(),
                    g::send::<B_EAT_FILE, B, FileExit>(),
                ),
                g::send::<C_THINK_FILE, THINK_C, FileExit>(),
            ),
            g::seq(
                g::send::<C, C_EAT_FILE, CloseAuthority<C_EAT_CLOSE>>(),
                g::send::<C_EAT_FILE, C, FileExit>(),
            ),
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
        g::seq(g::par(io, g::seq(ready, g::seq(meals, exits))), returns),
    );
    Programs9E {
        requests: project(&global),
        table: project(&global),
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        eat_close_a: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        eat_close_b: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
        eat_close_c: project(&global),
    }
}

// Each fork lends its single Endpoint to both of its neighbors once.
fn fork_visit<const CLOSE: u8, const FORK: u8, const RESOURCE: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<RESOURCE, CLOSE, ForkRequest<RESOURCE, CLOSE>>(),
        g::seq(
            g::send::<FORK, CLOSE, Loan<RESOURCE>>(),
            g::seq(
                g::send::<RESOURCE, FORK, Meal<RESOURCE>>(),
                g::send::<CLOSE, FORK, Loan<RESOURCE>>(),
            ),
        ),
    )
}

pub struct Programs9M {
    pub requests: RoleProgram<{ roles::MEAL_REQUESTS }>,
    pub table: RoleProgram<{ roles::TABLE }>,
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub eat_close_a: RoleProgram<{ roles::A_EAT_CLOSE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub eat_close_b: RoleProgram<{ roles::B_EAT_CLOSE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
    pub eat_close_c: RoleProgram<{ roles::C_EAT_CLOSE }>,
    pub d: RoleProgram<{ roles::D }>,
    pub think_d: RoleProgram<{ roles::THINK_D }>,
    pub fork_d: RoleProgram<{ roles::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles::D_WASI }>,
    pub d_io: RoleProgram<{ roles::D_IO }>,
    pub resource_d: RoleProgram<{ roles::FORK_D_RESOURCE }>,
    pub d_think_file: RoleProgram<{ roles::D_THINK_FILE }>,
    pub d_eat_file: RoleProgram<{ roles::D_EAT_FILE }>,
    pub eat_close_d: RoleProgram<{ roles::D_EAT_CLOSE }>,
    pub e: RoleProgram<{ roles::E }>,
    pub think_e: RoleProgram<{ roles::THINK_E }>,
    pub fork_e: RoleProgram<{ roles::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles::E_WASI }>,
    pub e_io: RoleProgram<{ roles::E_IO }>,
    pub resource_e: RoleProgram<{ roles::FORK_E_RESOURCE }>,
    pub e_think_file: RoleProgram<{ roles::E_THINK_FILE }>,
    pub e_eat_file: RoleProgram<{ roles::E_EAT_FILE }>,
    pub eat_close_e: RoleProgram<{ roles::E_EAT_CLOSE }>,
}
impl Programs9M {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9M","programs":[rows(&self.requests),rows(&self.table),rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.eat_close_a),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.eat_close_b),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file),rows(&self.eat_close_c),rows(&self.d),rows(&self.think_d),rows(&self.fork_d),rows(&self.d_wasi),rows(&self.d_io),rows(&self.resource_d),rows(&self.d_think_file),rows(&self.d_eat_file),rows(&self.eat_close_d),rows(&self.e),rows(&self.think_e),rows(&self.fork_e),rows(&self.e_wasi),rows(&self.e_io),rows(&self.resource_e),rows(&self.e_think_file),rows(&self.e_eat_file),rows(&self.eat_close_e)]})
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
    let ready_a = g::seq(
        thinking::<A_THINK_FILE, THINK_A>(),
        ready_to_eat::<A_EAT_FILE, A_EAT_CLOSE, EAT_A_ARGUMENTS>(),
    );
    let ready_b = g::seq(
        thinking::<B_THINK_FILE, THINK_B>(),
        ready_to_eat::<B_EAT_FILE, B_EAT_CLOSE, EAT_B_ARGUMENTS>(),
    );
    let ready_c = g::seq(
        thinking::<C_THINK_FILE, THINK_C>(),
        ready_to_eat::<C_EAT_FILE, C_EAT_CLOSE, EAT_C_ARGUMENTS>(),
    );
    let ready_d = g::seq(
        thinking::<D_THINK_FILE, THINK_D>(),
        ready_to_eat::<D_EAT_FILE, D_EAT_CLOSE, EAT_D_ARGUMENTS>(),
    );
    let ready_e = g::seq(
        thinking::<E_THINK_FILE, THINK_E>(),
        ready_to_eat::<E_EAT_FILE, E_EAT_CLOSE, EAT_E_ARGUMENTS>(),
    );
    let ready = g::par(
        g::par(g::par(ready_a, ready_b), ready_c),
        g::par(ready_d, ready_e),
    );
    let claim_a_a = g::send::<TABLE, FORK_A, TableMeal<FORK_A>>();
    let claim_a_b = g::send::<TABLE, FORK_A, TableMeal<FORK_A_RESOURCE>>();
    let visit_a_a = fork_visit::<A_EAT_CLOSE, FORK_A, FORK_A_RESOURCE>();
    let visit_a_b = fork_visit::<B, FORK_A, FORK_A_RESOURCE>();
    let a_then_b = g::seq(g::seq(claim_a_a, visit_a_a), g::seq(claim_a_b, visit_a_b));
    let b_then_a = g::seq(g::seq(claim_a_b, visit_a_b), g::seq(claim_a_a, visit_a_a));
    let fork_a = g::route(a_then_b, b_then_a);
    let claim_b_b = g::send::<TABLE, FORK_B, TableMeal<FORK_B>>();
    let claim_b_c = g::send::<TABLE, FORK_B, TableMeal<FORK_B_RESOURCE>>();
    let visit_b_b = fork_visit::<B_EAT_CLOSE, FORK_B, FORK_B_RESOURCE>();
    let visit_b_c = fork_visit::<C, FORK_B, FORK_B_RESOURCE>();
    let b_then_c = g::seq(g::seq(claim_b_b, visit_b_b), g::seq(claim_b_c, visit_b_c));
    let c_then_b = g::seq(g::seq(claim_b_c, visit_b_c), g::seq(claim_b_b, visit_b_b));
    let fork_b = g::route(b_then_c, c_then_b);
    let claim_c_c = g::send::<TABLE, FORK_C, TableMeal<FORK_C>>();
    let claim_c_d = g::send::<TABLE, FORK_C, TableMeal<FORK_C_RESOURCE>>();
    let visit_c_c = fork_visit::<C_EAT_CLOSE, FORK_C, FORK_C_RESOURCE>();
    let visit_c_d = fork_visit::<D, FORK_C, FORK_C_RESOURCE>();
    let c_then_d = g::seq(g::seq(claim_c_c, visit_c_c), g::seq(claim_c_d, visit_c_d));
    let d_then_c = g::seq(g::seq(claim_c_d, visit_c_d), g::seq(claim_c_c, visit_c_c));
    let fork_c = g::route(c_then_d, d_then_c);
    let claim_d_d = g::send::<TABLE, FORK_D, TableMeal<FORK_D>>();
    let claim_d_e = g::send::<TABLE, FORK_D, TableMeal<FORK_D_RESOURCE>>();
    let visit_d_d = fork_visit::<D_EAT_CLOSE, FORK_D, FORK_D_RESOURCE>();
    let visit_d_e = fork_visit::<E, FORK_D, FORK_D_RESOURCE>();
    let d_then_e = g::seq(g::seq(claim_d_d, visit_d_d), g::seq(claim_d_e, visit_d_e));
    let e_then_d = g::seq(g::seq(claim_d_e, visit_d_e), g::seq(claim_d_d, visit_d_d));
    let fork_d = g::route(d_then_e, e_then_d);
    let claim_e_e = g::send::<TABLE, FORK_E, TableMeal<FORK_E>>();
    let claim_e_a = g::send::<TABLE, FORK_E, TableMeal<FORK_E_RESOURCE>>();
    let visit_e_e = fork_visit::<E_EAT_CLOSE, FORK_E, FORK_E_RESOURCE>();
    let visit_e_a = fork_visit::<A, FORK_E, FORK_E_RESOURCE>();
    let e_then_a = g::seq(g::seq(claim_e_e, visit_e_e), g::seq(claim_e_a, visit_e_a));
    let a_then_e = g::seq(g::seq(claim_e_a, visit_e_a), g::seq(claim_e_e, visit_e_e));
    let fork_e = g::route(e_then_a, a_then_e);
    // The first submitted meal has both incident forks first. Removing that seat
    // opens the ring into a path, so the other fork orders cannot form a wait cycle.
    let first_a = g::seq(
        claim_a_a,
        g::par(
            g::par(
                g::par(g::seq(visit_a_a, g::seq(claim_a_b, visit_a_b)), fork_b),
                fork_c,
            ),
            g::par(fork_d, a_then_e),
        ),
    );
    let first_b = g::seq(
        claim_b_b,
        g::par(
            g::par(
                g::par(b_then_a, g::seq(visit_b_b, g::seq(claim_b_c, visit_b_c))),
                fork_c,
            ),
            g::par(fork_d, fork_e),
        ),
    );
    let first_c = g::seq(
        claim_c_c,
        g::par(
            g::par(
                g::par(fork_a, c_then_b),
                g::seq(visit_c_c, g::seq(claim_c_d, visit_c_d)),
            ),
            g::par(fork_d, fork_e),
        ),
    );
    let first_d = g::seq(
        claim_d_d,
        g::par(
            g::par(g::par(fork_a, fork_b), d_then_c),
            g::par(g::seq(visit_d_d, g::seq(claim_d_e, visit_d_e)), fork_e),
        ),
    );
    let first_e = g::seq(
        claim_e_e,
        g::par(
            g::par(g::par(fork_a, fork_b), fork_c),
            g::par(e_then_d, g::seq(visit_e_e, g::seq(claim_e_a, visit_e_a))),
        ),
    );
    let meals = g::route(
        g::route(g::route(first_a, first_b), first_c),
        g::route(first_d, first_e),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                    g::seq(
                        g::send::<A, A_EAT_FILE, CloseAuthority<A_EAT_CLOSE>>(),
                        g::send::<A_EAT_FILE, A, FileExit>(),
                    ),
                ),
                g::send::<B_THINK_FILE, THINK_B, FileExit>(),
            ),
            g::par(
                g::seq(
                    g::send::<B, B_EAT_FILE, CloseAuthority<B_EAT_CLOSE>>(),
                    g::send::<B_EAT_FILE, B, FileExit>(),
                ),
                g::send::<C_THINK_FILE, THINK_C, FileExit>(),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::seq(
                        g::send::<C, C_EAT_FILE, CloseAuthority<C_EAT_CLOSE>>(),
                        g::send::<C_EAT_FILE, C, FileExit>(),
                    ),
                    g::send::<D_THINK_FILE, THINK_D, FileExit>(),
                ),
                g::seq(
                    g::send::<D, D_EAT_FILE, CloseAuthority<D_EAT_CLOSE>>(),
                    g::send::<D_EAT_FILE, D, FileExit>(),
                ),
            ),
            g::par(
                g::send::<E_THINK_FILE, THINK_E, FileExit>(),
                g::seq(
                    g::send::<E, E_EAT_FILE, CloseAuthority<E_EAT_CLOSE>>(),
                    g::send::<E_EAT_FILE, E, FileExit>(),
                ),
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
        g::seq(g::par(io, g::seq(ready, g::seq(meals, exits))), returns),
    );
    Programs9M {
        requests: project(&global),
        table: project(&global),
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        eat_close_a: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        eat_close_b: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
        eat_close_c: project(&global),
        d: project(&global),
        think_d: project(&global),
        fork_d: project(&global),
        d_wasi: project(&global),
        d_io: project(&global),
        resource_d: project(&global),
        d_think_file: project(&global),
        d_eat_file: project(&global),
        eat_close_d: project(&global),
        e: project(&global),
        think_e: project(&global),
        fork_e: project(&global),
        e_wasi: project(&global),
        e_io: project(&global),
        resource_e: project(&global),
        e_think_file: project(&global),
        e_eat_file: project(&global),
        eat_close_e: project(&global),
    }
}

pub struct Programs9H {
    pub requests: RoleProgram<{ roles::MEAL_REQUESTS }>,
    pub table: RoleProgram<{ roles::TABLE }>,
    pub a: RoleProgram<{ roles::A }>,
    pub think_a: RoleProgram<{ roles::THINK_A }>,
    pub fork_a: RoleProgram<{ roles::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles::A_WASI }>,
    pub a_io: RoleProgram<{ roles::A_IO }>,
    pub resource_a: RoleProgram<{ roles::FORK_A_RESOURCE }>,
    pub a_think_file: RoleProgram<{ roles::A_THINK_FILE }>,
    pub a_eat_file: RoleProgram<{ roles::A_EAT_FILE }>,
    pub eat_close_a: RoleProgram<{ roles::A_EAT_CLOSE }>,
    pub b: RoleProgram<{ roles::B }>,
    pub think_b: RoleProgram<{ roles::THINK_B }>,
    pub fork_b: RoleProgram<{ roles::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles::B_WASI }>,
    pub b_io: RoleProgram<{ roles::B_IO }>,
    pub resource_b: RoleProgram<{ roles::FORK_B_RESOURCE }>,
    pub b_think_file: RoleProgram<{ roles::B_THINK_FILE }>,
    pub b_eat_file: RoleProgram<{ roles::B_EAT_FILE }>,
    pub eat_close_b: RoleProgram<{ roles::B_EAT_CLOSE }>,
    pub c: RoleProgram<{ roles::C }>,
    pub think_c: RoleProgram<{ roles::THINK_C }>,
    pub fork_c: RoleProgram<{ roles::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles::C_WASI }>,
    pub c_io: RoleProgram<{ roles::C_IO }>,
    pub resource_c: RoleProgram<{ roles::FORK_C_RESOURCE }>,
    pub c_think_file: RoleProgram<{ roles::C_THINK_FILE }>,
    pub c_eat_file: RoleProgram<{ roles::C_EAT_FILE }>,
    pub eat_close_c: RoleProgram<{ roles::C_EAT_CLOSE }>,
    pub d: RoleProgram<{ roles::D }>,
    pub think_d: RoleProgram<{ roles::THINK_D }>,
    pub fork_d: RoleProgram<{ roles::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles::D_WASI }>,
    pub d_io: RoleProgram<{ roles::D_IO }>,
    pub resource_d: RoleProgram<{ roles::FORK_D_RESOURCE }>,
    pub d_think_file: RoleProgram<{ roles::D_THINK_FILE }>,
    pub d_eat_file: RoleProgram<{ roles::D_EAT_FILE }>,
    pub eat_close_d: RoleProgram<{ roles::D_EAT_CLOSE }>,
    pub e: RoleProgram<{ roles::E }>,
    pub think_e: RoleProgram<{ roles::THINK_E }>,
    pub fork_e: RoleProgram<{ roles::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles::E_WASI }>,
    pub e_io: RoleProgram<{ roles::E_IO }>,
    pub resource_e: RoleProgram<{ roles::FORK_E_RESOURCE }>,
    pub e_think_file: RoleProgram<{ roles::E_THINK_FILE }>,
    pub e_eat_file: RoleProgram<{ roles::E_EAT_FILE }>,
    pub eat_close_e: RoleProgram<{ roles::E_EAT_CLOSE }>,
    pub f: RoleProgram<{ roles::F }>,
    pub think_f: RoleProgram<{ roles::THINK_F }>,
    pub fork_f: RoleProgram<{ roles::FORK_F }>,
    pub f_wasi: RoleProgram<{ roles::F_WASI }>,
    pub f_io: RoleProgram<{ roles::F_IO }>,
    pub resource_f: RoleProgram<{ roles::FORK_F_RESOURCE }>,
    pub f_think_file: RoleProgram<{ roles::F_THINK_FILE }>,
    pub f_eat_file: RoleProgram<{ roles::F_EAT_FILE }>,
    pub eat_close_f: RoleProgram<{ roles::F_EAT_CLOSE }>,
    pub g: RoleProgram<{ roles::G }>,
    pub think_g: RoleProgram<{ roles::THINK_G }>,
    pub fork_g: RoleProgram<{ roles::FORK_G }>,
    pub g_wasi: RoleProgram<{ roles::G_WASI }>,
    pub g_io: RoleProgram<{ roles::G_IO }>,
    pub resource_g: RoleProgram<{ roles::FORK_G_RESOURCE }>,
    pub g_think_file: RoleProgram<{ roles::G_THINK_FILE }>,
    pub g_eat_file: RoleProgram<{ roles::G_EAT_FILE }>,
    pub eat_close_g: RoleProgram<{ roles::G_EAT_CLOSE }>,
}
impl Programs9H {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9H","programs":[rows(&self.requests),rows(&self.table),rows(&self.a),rows(&self.think_a),rows(&self.fork_a),rows(&self.a_wasi),rows(&self.a_io),rows(&self.resource_a),rows(&self.a_think_file),rows(&self.a_eat_file),rows(&self.eat_close_a),rows(&self.b),rows(&self.think_b),rows(&self.fork_b),rows(&self.b_wasi),rows(&self.b_io),rows(&self.resource_b),rows(&self.b_think_file),rows(&self.b_eat_file),rows(&self.eat_close_b),rows(&self.c),rows(&self.think_c),rows(&self.fork_c),rows(&self.c_wasi),rows(&self.c_io),rows(&self.resource_c),rows(&self.c_think_file),rows(&self.c_eat_file),rows(&self.eat_close_c),rows(&self.d),rows(&self.think_d),rows(&self.fork_d),rows(&self.d_wasi),rows(&self.d_io),rows(&self.resource_d),rows(&self.d_think_file),rows(&self.d_eat_file),rows(&self.eat_close_d),rows(&self.e),rows(&self.think_e),rows(&self.fork_e),rows(&self.e_wasi),rows(&self.e_io),rows(&self.resource_e),rows(&self.e_think_file),rows(&self.e_eat_file),rows(&self.eat_close_e),rows(&self.f),rows(&self.think_f),rows(&self.fork_f),rows(&self.f_wasi),rows(&self.f_io),rows(&self.resource_f),rows(&self.f_think_file),rows(&self.f_eat_file),rows(&self.eat_close_f),rows(&self.g),rows(&self.think_g),rows(&self.fork_g),rows(&self.g_wasi),rows(&self.g_io),rows(&self.resource_g),rows(&self.g_think_file),rows(&self.g_eat_file),rows(&self.eat_close_g)]})
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
    let ready_a = g::seq(
        thinking::<A_THINK_FILE, THINK_A>(),
        ready_to_eat::<A_EAT_FILE, A_EAT_CLOSE, EAT_A_ARGUMENTS>(),
    );
    let ready_b = g::seq(
        thinking::<B_THINK_FILE, THINK_B>(),
        ready_to_eat::<B_EAT_FILE, B_EAT_CLOSE, EAT_B_ARGUMENTS>(),
    );
    let ready_c = g::seq(
        thinking::<C_THINK_FILE, THINK_C>(),
        ready_to_eat::<C_EAT_FILE, C_EAT_CLOSE, EAT_C_ARGUMENTS>(),
    );
    let ready_d = g::seq(
        thinking::<D_THINK_FILE, THINK_D>(),
        ready_to_eat::<D_EAT_FILE, D_EAT_CLOSE, EAT_D_ARGUMENTS>(),
    );
    let ready_e = g::seq(
        thinking::<E_THINK_FILE, THINK_E>(),
        ready_to_eat::<E_EAT_FILE, E_EAT_CLOSE, EAT_E_ARGUMENTS>(),
    );
    let ready_f = g::seq(
        thinking::<F_THINK_FILE, THINK_F>(),
        ready_to_eat::<F_EAT_FILE, F_EAT_CLOSE, EAT_F_ARGUMENTS>(),
    );
    let ready_g = g::seq(
        thinking::<G_THINK_FILE, THINK_G>(),
        ready_to_eat::<G_EAT_FILE, G_EAT_CLOSE, EAT_G_ARGUMENTS>(),
    );
    let ready = g::par(
        g::par(g::par(ready_a, ready_b), g::par(ready_c, ready_d)),
        g::par(g::par(ready_e, ready_f), ready_g),
    );
    let claim_a_a = g::send::<TABLE, FORK_A, TableMeal<FORK_A>>();
    let claim_a_b = g::send::<TABLE, FORK_A, TableMeal<FORK_A_RESOURCE>>();
    let visit_a_a = fork_visit::<A_EAT_CLOSE, FORK_A, FORK_A_RESOURCE>();
    let visit_a_b = fork_visit::<B, FORK_A, FORK_A_RESOURCE>();
    let a_then_b = g::seq(g::seq(claim_a_a, visit_a_a), g::seq(claim_a_b, visit_a_b));
    let b_then_a = g::seq(g::seq(claim_a_b, visit_a_b), g::seq(claim_a_a, visit_a_a));
    let fork_a = g::route(a_then_b, b_then_a);
    let claim_b_b = g::send::<TABLE, FORK_B, TableMeal<FORK_B>>();
    let claim_b_c = g::send::<TABLE, FORK_B, TableMeal<FORK_B_RESOURCE>>();
    let visit_b_b = fork_visit::<B_EAT_CLOSE, FORK_B, FORK_B_RESOURCE>();
    let visit_b_c = fork_visit::<C, FORK_B, FORK_B_RESOURCE>();
    let b_then_c = g::seq(g::seq(claim_b_b, visit_b_b), g::seq(claim_b_c, visit_b_c));
    let c_then_b = g::seq(g::seq(claim_b_c, visit_b_c), g::seq(claim_b_b, visit_b_b));
    let fork_b = g::route(b_then_c, c_then_b);
    let claim_c_c = g::send::<TABLE, FORK_C, TableMeal<FORK_C>>();
    let claim_c_d = g::send::<TABLE, FORK_C, TableMeal<FORK_C_RESOURCE>>();
    let visit_c_c = fork_visit::<C_EAT_CLOSE, FORK_C, FORK_C_RESOURCE>();
    let visit_c_d = fork_visit::<D, FORK_C, FORK_C_RESOURCE>();
    let c_then_d = g::seq(g::seq(claim_c_c, visit_c_c), g::seq(claim_c_d, visit_c_d));
    let d_then_c = g::seq(g::seq(claim_c_d, visit_c_d), g::seq(claim_c_c, visit_c_c));
    let fork_c = g::route(c_then_d, d_then_c);
    let claim_d_d = g::send::<TABLE, FORK_D, TableMeal<FORK_D>>();
    let claim_d_e = g::send::<TABLE, FORK_D, TableMeal<FORK_D_RESOURCE>>();
    let visit_d_d = fork_visit::<D_EAT_CLOSE, FORK_D, FORK_D_RESOURCE>();
    let visit_d_e = fork_visit::<E, FORK_D, FORK_D_RESOURCE>();
    let d_then_e = g::seq(g::seq(claim_d_d, visit_d_d), g::seq(claim_d_e, visit_d_e));
    let e_then_d = g::seq(g::seq(claim_d_e, visit_d_e), g::seq(claim_d_d, visit_d_d));
    let fork_d = g::route(d_then_e, e_then_d);
    let claim_e_e = g::send::<TABLE, FORK_E, TableMeal<FORK_E>>();
    let claim_e_f = g::send::<TABLE, FORK_E, TableMeal<FORK_E_RESOURCE>>();
    let visit_e_e = fork_visit::<E_EAT_CLOSE, FORK_E, FORK_E_RESOURCE>();
    let visit_e_f = fork_visit::<F, FORK_E, FORK_E_RESOURCE>();
    let e_then_f = g::seq(g::seq(claim_e_e, visit_e_e), g::seq(claim_e_f, visit_e_f));
    let f_then_e = g::seq(g::seq(claim_e_f, visit_e_f), g::seq(claim_e_e, visit_e_e));
    let fork_e = g::route(e_then_f, f_then_e);
    let claim_f_f = g::send::<TABLE, FORK_F, TableMeal<FORK_F>>();
    let claim_f_g = g::send::<TABLE, FORK_F, TableMeal<FORK_F_RESOURCE>>();
    let visit_f_f = fork_visit::<F_EAT_CLOSE, FORK_F, FORK_F_RESOURCE>();
    let visit_f_g = fork_visit::<G, FORK_F, FORK_F_RESOURCE>();
    let f_then_g = g::seq(g::seq(claim_f_f, visit_f_f), g::seq(claim_f_g, visit_f_g));
    let g_then_f = g::seq(g::seq(claim_f_g, visit_f_g), g::seq(claim_f_f, visit_f_f));
    let fork_f = g::route(f_then_g, g_then_f);
    let claim_g_g = g::send::<TABLE, FORK_G, TableMeal<FORK_G>>();
    let claim_g_a = g::send::<TABLE, FORK_G, TableMeal<FORK_G_RESOURCE>>();
    let visit_g_g = fork_visit::<G_EAT_CLOSE, FORK_G, FORK_G_RESOURCE>();
    let visit_g_a = fork_visit::<A, FORK_G, FORK_G_RESOURCE>();
    let g_then_a = g::seq(g::seq(claim_g_g, visit_g_g), g::seq(claim_g_a, visit_g_a));
    let a_then_g = g::seq(g::seq(claim_g_a, visit_g_a), g::seq(claim_g_g, visit_g_g));
    let fork_g = g::route(g_then_a, a_then_g);
    // The first submitted meal has both incident forks first. Removing that seat
    // opens the ring into a path, so the other fork orders cannot form a wait cycle.
    let first_a = g::seq(
        claim_a_a,
        g::par(
            g::par(
                g::par(g::seq(visit_a_a, g::seq(claim_a_b, visit_a_b)), fork_b),
                g::par(fork_c, fork_d),
            ),
            g::par(g::par(fork_e, fork_f), a_then_g),
        ),
    );
    let first_b = g::seq(
        claim_b_b,
        g::par(
            g::par(
                g::par(b_then_a, g::seq(visit_b_b, g::seq(claim_b_c, visit_b_c))),
                g::par(fork_c, fork_d),
            ),
            g::par(g::par(fork_e, fork_f), fork_g),
        ),
    );
    let first_c = g::seq(
        claim_c_c,
        g::par(
            g::par(
                g::par(fork_a, c_then_b),
                g::par(g::seq(visit_c_c, g::seq(claim_c_d, visit_c_d)), fork_d),
            ),
            g::par(g::par(fork_e, fork_f), fork_g),
        ),
    );
    let first_d = g::seq(
        claim_d_d,
        g::par(
            g::par(
                g::par(fork_a, fork_b),
                g::par(d_then_c, g::seq(visit_d_d, g::seq(claim_d_e, visit_d_e))),
            ),
            g::par(g::par(fork_e, fork_f), fork_g),
        ),
    );
    let first_e = g::seq(
        claim_e_e,
        g::par(
            g::par(g::par(fork_a, fork_b), g::par(fork_c, e_then_d)),
            g::par(
                g::par(g::seq(visit_e_e, g::seq(claim_e_f, visit_e_f)), fork_f),
                fork_g,
            ),
        ),
    );
    let first_f = g::seq(
        claim_f_f,
        g::par(
            g::par(g::par(fork_a, fork_b), g::par(fork_c, fork_d)),
            g::par(
                g::par(f_then_e, g::seq(visit_f_f, g::seq(claim_f_g, visit_f_g))),
                fork_g,
            ),
        ),
    );
    let first_g = g::seq(
        claim_g_g,
        g::par(
            g::par(g::par(fork_a, fork_b), g::par(fork_c, fork_d)),
            g::par(
                g::par(fork_e, g_then_f),
                g::seq(visit_g_g, g::seq(claim_g_a, visit_g_a)),
            ),
        ),
    );
    let meals = g::route(
        g::route(g::route(first_a, first_b), g::route(first_c, first_d)),
        g::route(g::route(first_e, first_f), first_g),
    );
    let exits = g::par(
        g::par(
            g::par(
                g::par(
                    g::send::<A_THINK_FILE, THINK_A, FileExit>(),
                    g::seq(
                        g::send::<A, A_EAT_FILE, CloseAuthority<A_EAT_CLOSE>>(),
                        g::send::<A_EAT_FILE, A, FileExit>(),
                    ),
                ),
                g::par(
                    g::send::<B_THINK_FILE, THINK_B, FileExit>(),
                    g::seq(
                        g::send::<B, B_EAT_FILE, CloseAuthority<B_EAT_CLOSE>>(),
                        g::send::<B_EAT_FILE, B, FileExit>(),
                    ),
                ),
            ),
            g::par(
                g::par(
                    g::send::<C_THINK_FILE, THINK_C, FileExit>(),
                    g::seq(
                        g::send::<C, C_EAT_FILE, CloseAuthority<C_EAT_CLOSE>>(),
                        g::send::<C_EAT_FILE, C, FileExit>(),
                    ),
                ),
                g::send::<D_THINK_FILE, THINK_D, FileExit>(),
            ),
        ),
        g::par(
            g::par(
                g::par(
                    g::seq(
                        g::send::<D, D_EAT_FILE, CloseAuthority<D_EAT_CLOSE>>(),
                        g::send::<D_EAT_FILE, D, FileExit>(),
                    ),
                    g::send::<E_THINK_FILE, THINK_E, FileExit>(),
                ),
                g::par(
                    g::seq(
                        g::send::<E, E_EAT_FILE, CloseAuthority<E_EAT_CLOSE>>(),
                        g::send::<E_EAT_FILE, E, FileExit>(),
                    ),
                    g::send::<F_THINK_FILE, THINK_F, FileExit>(),
                ),
            ),
            g::par(
                g::par(
                    g::seq(
                        g::send::<F, F_EAT_FILE, CloseAuthority<F_EAT_CLOSE>>(),
                        g::send::<F_EAT_FILE, F, FileExit>(),
                    ),
                    g::send::<G_THINK_FILE, THINK_G, FileExit>(),
                ),
                g::seq(
                    g::send::<G, G_EAT_FILE, CloseAuthority<G_EAT_CLOSE>>(),
                    g::send::<G_EAT_FILE, G, FileExit>(),
                ),
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
        g::seq(g::par(io, g::seq(ready, g::seq(meals, exits))), returns),
    );
    Programs9H {
        requests: project(&global),
        table: project(&global),
        a: project(&global),
        think_a: project(&global),
        fork_a: project(&global),
        a_wasi: project(&global),
        a_io: project(&global),
        resource_a: project(&global),
        a_think_file: project(&global),
        a_eat_file: project(&global),
        eat_close_a: project(&global),
        b: project(&global),
        think_b: project(&global),
        fork_b: project(&global),
        b_wasi: project(&global),
        b_io: project(&global),
        resource_b: project(&global),
        b_think_file: project(&global),
        b_eat_file: project(&global),
        eat_close_b: project(&global),
        c: project(&global),
        think_c: project(&global),
        fork_c: project(&global),
        c_wasi: project(&global),
        c_io: project(&global),
        resource_c: project(&global),
        c_think_file: project(&global),
        c_eat_file: project(&global),
        eat_close_c: project(&global),
        d: project(&global),
        think_d: project(&global),
        fork_d: project(&global),
        d_wasi: project(&global),
        d_io: project(&global),
        resource_d: project(&global),
        d_think_file: project(&global),
        d_eat_file: project(&global),
        eat_close_d: project(&global),
        e: project(&global),
        think_e: project(&global),
        fork_e: project(&global),
        e_wasi: project(&global),
        e_io: project(&global),
        resource_e: project(&global),
        e_think_file: project(&global),
        e_eat_file: project(&global),
        eat_close_e: project(&global),
        f: project(&global),
        think_f: project(&global),
        fork_f: project(&global),
        f_wasi: project(&global),
        f_io: project(&global),
        resource_f: project(&global),
        f_think_file: project(&global),
        f_eat_file: project(&global),
        eat_close_f: project(&global),
        g: project(&global),
        think_g: project(&global),
        fork_g: project(&global),
        g_wasi: project(&global),
        g_io: project(&global),
        resource_g: project(&global),
        g_think_file: project(&global),
        g_eat_file: project(&global),
        eat_close_g: project(&global),
    }
}
