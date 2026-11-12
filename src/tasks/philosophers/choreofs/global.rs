//! Each shared request runs through the endpoint of its required resource.
use super::{
    identity::{roles9e, roles9h, roles9m},
    imp::PhilosopherArguments,
};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub const SERVICE_FD: u8 = 4;
pub type Think = g::Msg<1, PhilosopherArguments>;
pub type Thought<'a> = g::Msg<2, &'a [u8]>;
pub type Eat<const RESOURCE: u8> = g::Msg<RESOURCE, PhilosopherArguments>;
pub type Meal<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type ServiceExit = g::Msg<241, i32>;

fn think<const VM: u8, const SERVICE: u8>() -> impl Projectable {
    g::seq(
        g::send::<VM, SERVICE, Think>(),
        g::send::<SERVICE, VM, Thought>(),
    )
}

fn eat<
    const VM: u8,
    const SERVICE: u8,
    const LEFT_OWNER: u8,
    const RIGHT_OWNER: u8,
    const LEFT: u8,
    const RIGHT: u8,
>() -> impl Projectable {
    g::seq(
        g::par(
            g::seq(
                g::send::<LEFT_OWNER, VM, Loan<LEFT>>(),
                g::send::<LEFT, SERVICE, Eat<LEFT>>(),
            ),
            g::seq(
                g::send::<RIGHT_OWNER, VM, Loan<RIGHT>>(),
                g::send::<RIGHT, SERVICE, Eat<RIGHT>>(),
            ),
        ),
        g::seq(
            g::par(
                g::send::<SERVICE, LEFT, Meal<LEFT>>(),
                g::send::<SERVICE, RIGHT, Meal<RIGHT>>(),
            ),
            g::par(
                g::send::<VM, LEFT_OWNER, Loan<LEFT>>(),
                g::send::<VM, RIGHT_OWNER, Loan<RIGHT>>(),
            ),
        ),
    )
}

pub struct Programs9E {
    pub a: RoleProgram<{ roles9e::A }>,
    pub fork_a: RoleProgram<{ roles9e::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9e::A_WASI }>,
    pub a_io: RoleProgram<{ roles9e::A_IO }>,
    pub resource_a: RoleProgram<{ roles9e::FORK_A_RESOURCE }>,
    pub b: RoleProgram<{ roles9e::B }>,
    pub fork_b: RoleProgram<{ roles9e::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9e::B_WASI }>,
    pub b_io: RoleProgram<{ roles9e::B_IO }>,
    pub resource_b: RoleProgram<{ roles9e::FORK_B_RESOURCE }>,
    pub c: RoleProgram<{ roles9e::C }>,
    pub fork_c: RoleProgram<{ roles9e::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9e::C_WASI }>,
    pub c_io: RoleProgram<{ roles9e::C_IO }>,
    pub resource_c: RoleProgram<{ roles9e::FORK_C_RESOURCE }>,
}
pub fn project_9e() -> Programs9E {
    use roles9e::*;
    let task = g::seq(
        g::par(
            think::<A_WASI, A>(),
            g::par(think::<B_WASI, B>(), think::<C_WASI, C>()),
        ),
        g::seq(
            g::seq(
                eat::<A_WASI, A, FORK_A, FORK_C, FORK_A_RESOURCE, FORK_C_RESOURCE>(),
                g::seq(
                    eat::<B_WASI, B, FORK_B, FORK_A, FORK_B_RESOURCE, FORK_A_RESOURCE>(),
                    eat::<C_WASI, C, FORK_C, FORK_B, FORK_C_RESOURCE, FORK_B_RESOURCE>(),
                ),
            ),
            g::par(
                g::send::<A_WASI, A, ServiceExit>(),
                g::par(
                    g::send::<B_WASI, B, ServiceExit>(),
                    g::send::<C_WASI, C, ServiceExit>(),
                ),
            ),
        ),
    );
    let protocol = g::par(
        task,
        g::par(
            crate::shell::global::private_io::<A_WASI, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B_WASI, B_IO>(),
                crate::shell::global::private_io::<C_WASI, C_IO>(),
            ),
        ),
    );
    Programs9E {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        resource_a: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        resource_b: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        resource_c: project(&protocol),
    }
}

pub struct Programs9M {
    pub a: RoleProgram<{ roles9m::A }>,
    pub fork_a: RoleProgram<{ roles9m::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9m::A_WASI }>,
    pub a_io: RoleProgram<{ roles9m::A_IO }>,
    pub resource_a: RoleProgram<{ roles9m::FORK_A_RESOURCE }>,
    pub b: RoleProgram<{ roles9m::B }>,
    pub fork_b: RoleProgram<{ roles9m::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9m::B_WASI }>,
    pub b_io: RoleProgram<{ roles9m::B_IO }>,
    pub resource_b: RoleProgram<{ roles9m::FORK_B_RESOURCE }>,
    pub c: RoleProgram<{ roles9m::C }>,
    pub fork_c: RoleProgram<{ roles9m::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9m::C_WASI }>,
    pub c_io: RoleProgram<{ roles9m::C_IO }>,
    pub resource_c: RoleProgram<{ roles9m::FORK_C_RESOURCE }>,
    pub d: RoleProgram<{ roles9m::D }>,
    pub fork_d: RoleProgram<{ roles9m::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles9m::D_WASI }>,
    pub d_io: RoleProgram<{ roles9m::D_IO }>,
    pub resource_d: RoleProgram<{ roles9m::FORK_D_RESOURCE }>,
    pub e: RoleProgram<{ roles9m::E }>,
    pub fork_e: RoleProgram<{ roles9m::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles9m::E_WASI }>,
    pub e_io: RoleProgram<{ roles9m::E_IO }>,
    pub resource_e: RoleProgram<{ roles9m::FORK_E_RESOURCE }>,
}
pub fn project_9m() -> Programs9M {
    use roles9m::*;
    let task = g::seq(
        g::par(
            g::par(think::<A_WASI, A>(), think::<B_WASI, B>()),
            g::par(
                think::<C_WASI, C>(),
                g::par(think::<D_WASI, D>(), think::<E_WASI, E>()),
            ),
        ),
        g::seq(
            g::seq(
                g::par(
                    eat::<A_WASI, A, FORK_A, FORK_E, FORK_A_RESOURCE, FORK_E_RESOURCE>(),
                    eat::<C_WASI, C, FORK_C, FORK_B, FORK_C_RESOURCE, FORK_B_RESOURCE>(),
                ),
                g::seq(
                    g::par(
                        eat::<B_WASI, B, FORK_B, FORK_A, FORK_B_RESOURCE, FORK_A_RESOURCE>(),
                        eat::<D_WASI, D, FORK_D, FORK_C, FORK_D_RESOURCE, FORK_C_RESOURCE>(),
                    ),
                    eat::<E_WASI, E, FORK_E, FORK_D, FORK_E_RESOURCE, FORK_D_RESOURCE>(),
                ),
            ),
            g::par(
                g::par(
                    g::send::<A_WASI, A, ServiceExit>(),
                    g::send::<B_WASI, B, ServiceExit>(),
                ),
                g::par(
                    g::send::<C_WASI, C, ServiceExit>(),
                    g::par(
                        g::send::<D_WASI, D, ServiceExit>(),
                        g::send::<E_WASI, E, ServiceExit>(),
                    ),
                ),
            ),
        ),
    );
    let protocol = g::par(
        task,
        g::par(
            g::par(
                crate::shell::global::private_io::<A_WASI, A_IO>(),
                crate::shell::global::private_io::<B_WASI, B_IO>(),
            ),
            g::par(
                crate::shell::global::private_io::<C_WASI, C_IO>(),
                g::par(
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                    crate::shell::global::private_io::<E_WASI, E_IO>(),
                ),
            ),
        ),
    );
    Programs9M {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        resource_a: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        resource_b: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        resource_c: project(&protocol),
        d: project(&protocol),
        fork_d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        resource_d: project(&protocol),
        e: project(&protocol),
        fork_e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        resource_e: project(&protocol),
    }
}

pub struct Programs9H {
    pub a: RoleProgram<{ roles9h::A }>,
    pub fork_a: RoleProgram<{ roles9h::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9h::A_WASI }>,
    pub a_io: RoleProgram<{ roles9h::A_IO }>,
    pub resource_a: RoleProgram<{ roles9h::FORK_A_RESOURCE }>,
    pub b: RoleProgram<{ roles9h::B }>,
    pub fork_b: RoleProgram<{ roles9h::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9h::B_WASI }>,
    pub b_io: RoleProgram<{ roles9h::B_IO }>,
    pub resource_b: RoleProgram<{ roles9h::FORK_B_RESOURCE }>,
    pub c: RoleProgram<{ roles9h::C }>,
    pub fork_c: RoleProgram<{ roles9h::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9h::C_WASI }>,
    pub c_io: RoleProgram<{ roles9h::C_IO }>,
    pub resource_c: RoleProgram<{ roles9h::FORK_C_RESOURCE }>,
    pub d: RoleProgram<{ roles9h::D }>,
    pub fork_d: RoleProgram<{ roles9h::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles9h::D_WASI }>,
    pub d_io: RoleProgram<{ roles9h::D_IO }>,
    pub resource_d: RoleProgram<{ roles9h::FORK_D_RESOURCE }>,
    pub e: RoleProgram<{ roles9h::E }>,
    pub fork_e: RoleProgram<{ roles9h::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles9h::E_WASI }>,
    pub e_io: RoleProgram<{ roles9h::E_IO }>,
    pub resource_e: RoleProgram<{ roles9h::FORK_E_RESOURCE }>,
    pub f: RoleProgram<{ roles9h::F }>,
    pub fork_f: RoleProgram<{ roles9h::FORK_F }>,
    pub f_wasi: RoleProgram<{ roles9h::F_WASI }>,
    pub f_io: RoleProgram<{ roles9h::F_IO }>,
    pub resource_f: RoleProgram<{ roles9h::FORK_F_RESOURCE }>,
    pub g: RoleProgram<{ roles9h::G }>,
    pub fork_g: RoleProgram<{ roles9h::FORK_G }>,
    pub g_wasi: RoleProgram<{ roles9h::G_WASI }>,
    pub g_io: RoleProgram<{ roles9h::G_IO }>,
    pub resource_g: RoleProgram<{ roles9h::FORK_G_RESOURCE }>,
}
pub fn project_9h() -> Programs9H {
    use roles9h::*;
    let task = g::seq(
        g::par(
            g::par(
                think::<A_WASI, A>(),
                g::par(think::<B_WASI, B>(), think::<C_WASI, C>()),
            ),
            g::par(
                g::par(think::<D_WASI, D>(), think::<E_WASI, E>()),
                g::par(think::<F_WASI, F>(), think::<G_WASI, G>()),
            ),
        ),
        g::seq(
            g::seq(
                g::par(
                    eat::<A_WASI, A, FORK_A, FORK_G, FORK_A_RESOURCE, FORK_G_RESOURCE>(),
                    g::par(
                        eat::<C_WASI, C, FORK_C, FORK_B, FORK_C_RESOURCE, FORK_B_RESOURCE>(),
                        eat::<E_WASI, E, FORK_E, FORK_D, FORK_E_RESOURCE, FORK_D_RESOURCE>(),
                    ),
                ),
                g::seq(
                    g::par(
                        eat::<B_WASI, B, FORK_B, FORK_A, FORK_B_RESOURCE, FORK_A_RESOURCE>(),
                        g::par(
                            eat::<D_WASI, D, FORK_D, FORK_C, FORK_D_RESOURCE, FORK_C_RESOURCE>(),
                            eat::<F_WASI, F, FORK_F, FORK_E, FORK_F_RESOURCE, FORK_E_RESOURCE>(),
                        ),
                    ),
                    eat::<G_WASI, G, FORK_G, FORK_F, FORK_G_RESOURCE, FORK_F_RESOURCE>(),
                ),
            ),
            g::par(
                g::par(
                    g::send::<A_WASI, A, ServiceExit>(),
                    g::par(
                        g::send::<B_WASI, B, ServiceExit>(),
                        g::send::<C_WASI, C, ServiceExit>(),
                    ),
                ),
                g::par(
                    g::par(
                        g::send::<D_WASI, D, ServiceExit>(),
                        g::send::<E_WASI, E, ServiceExit>(),
                    ),
                    g::par(
                        g::send::<F_WASI, F, ServiceExit>(),
                        g::send::<G_WASI, G, ServiceExit>(),
                    ),
                ),
            ),
        ),
    );
    let protocol = g::par(
        task,
        g::par(
            g::par(
                crate::shell::global::private_io::<A_WASI, A_IO>(),
                g::par(
                    crate::shell::global::private_io::<B_WASI, B_IO>(),
                    crate::shell::global::private_io::<C_WASI, C_IO>(),
                ),
            ),
            g::par(
                g::par(
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                    crate::shell::global::private_io::<E_WASI, E_IO>(),
                ),
                g::par(
                    crate::shell::global::private_io::<F_WASI, F_IO>(),
                    crate::shell::global::private_io::<G_WASI, G_IO>(),
                ),
            ),
        ),
    );
    Programs9H {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        resource_a: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        resource_b: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        resource_c: project(&protocol),
        d: project(&protocol),
        fork_d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        resource_d: project(&protocol),
        e: project(&protocol),
        fork_e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        resource_e: project(&protocol),
        f: project(&protocol),
        fork_f: project(&protocol),
        f_wasi: project(&protocol),
        f_io: project(&protocol),
        resource_f: project(&protocol),
        g: project(&protocol),
        fork_g: project(&protocol),
        g_wasi: project(&protocol),
        g_io: project(&protocol),
        resource_g: project(&protocol),
    }
}

impl Programs9E {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9E","programs":[rows(&self.a), rows(&self.fork_a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.resource_a), rows(&self.b), rows(&self.fork_b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.resource_b), rows(&self.c), rows(&self.fork_c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.resource_c)]})
    }
}

impl Programs9M {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9M","programs":[rows(&self.a), rows(&self.fork_a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.resource_a), rows(&self.b), rows(&self.fork_b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.resource_b), rows(&self.c), rows(&self.fork_c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.resource_c), rows(&self.d), rows(&self.fork_d), rows(&self.d_wasi), rows(&self.d_io), rows(&self.resource_d), rows(&self.e), rows(&self.fork_e), rows(&self.e_wasi), rows(&self.e_io), rows(&self.resource_e)]})
    }
}

impl Programs9H {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"9H","programs":[rows(&self.a), rows(&self.fork_a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.resource_a), rows(&self.b), rows(&self.fork_b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.resource_b), rows(&self.c), rows(&self.fork_c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.resource_c), rows(&self.d), rows(&self.fork_d), rows(&self.d_wasi), rows(&self.d_io), rows(&self.resource_d), rows(&self.e), rows(&self.fork_e), rows(&self.e_wasi), rows(&self.e_io), rows(&self.resource_e), rows(&self.f), rows(&self.fork_f), rows(&self.f_wasi), rows(&self.f_io), rows(&self.resource_f), rows(&self.g), rows(&self.fork_g), rows(&self.g_wasi), rows(&self.g_io), rows(&self.resource_g)]})
    }
}
