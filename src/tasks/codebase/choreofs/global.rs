//! Typed service requests and physical module handoffs share one global.
pub use super::input::{EASY, HARD, MEDIUM};
use super::wire::{COMMIT, Close, DESIGN, IMPLEMENT, TEST};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const SERVICE_FD: u8 = 4;
pub type DesignRequest<const TASK: u8, const ROLE: u8> = g::Msg<104, Close<TASK, ROLE, DESIGN>>;
pub type DesignCompleted<'a> = g::Msg<107, &'a [u8]>;
pub type ImplementRequest<const TASK: u8, const ROLE: u8> =
    g::Msg<129, Close<TASK, ROLE, IMPLEMENT>>;
pub type ImplementCompleted<'a> = g::Msg<132, &'a [u8]>;
pub type CommitRequest<const TASK: u8, const ROLE: u8, const RESOURCE: u8> =
    g::Msg<RESOURCE, Close<TASK, ROLE, COMMIT>>;
pub type CommitCompleted<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type TestRequest<const TASK: u8, const ROLE: u8> = g::Msg<179, Close<TASK, ROLE, TEST>>;
pub type TestCompleted<'a> = g::Msg<182, &'a [u8]>;
pub type ServiceExit = g::Msg<241, i32>;

fn developer<
    const TASK: u8,
    const VM: u8,
    const OWNER: u8,
    const FIRST: u8,
    const SECOND: u8,
    const FIRST_RESOURCE: u8,
    const SECOND_RESOURCE: u8,
>() -> impl Projectable {
    let design = g::seq(
        g::send::<VM, OWNER, DesignRequest<TASK, OWNER>>(),
        g::send::<OWNER, VM, DesignCompleted>(),
    );
    let implementation = g::seq(
        g::send::<VM, OWNER, ImplementRequest<TASK, OWNER>>(),
        g::send::<OWNER, VM, ImplementCompleted>(),
    );
    let acquire = g::par(
        g::seq(
            g::send::<FIRST, VM, Loan<FIRST_RESOURCE>>(),
            g::send::<FIRST_RESOURCE, OWNER, CommitRequest<TASK, OWNER, FIRST_RESOURCE>>(),
        ),
        g::seq(
            g::send::<SECOND, VM, Loan<SECOND_RESOURCE>>(),
            g::send::<SECOND_RESOURCE, OWNER, CommitRequest<TASK, OWNER, SECOND_RESOURCE>>(),
        ),
    );
    let release = g::par(
        g::send::<VM, FIRST, Loan<FIRST_RESOURCE>>(),
        g::send::<VM, SECOND, Loan<SECOND_RESOURCE>>(),
    );
    let commit = g::seq(
        g::seq(design, implementation),
        g::seq(
            acquire,
            g::seq(
                g::par(
                    g::send::<OWNER, FIRST_RESOURCE, CommitCompleted<FIRST_RESOURCE>>(),
                    g::send::<OWNER, SECOND_RESOURCE, CommitCompleted<SECOND_RESOURCE>>(),
                ),
                release,
            ),
        ),
    );
    let test = g::seq(
        g::send::<VM, OWNER, TestRequest<TASK, OWNER>>(),
        g::send::<OWNER, VM, TestCompleted>(),
    );
    g::seq(commit, g::seq(test, g::send::<VM, OWNER, ServiceExit>()))
}

pub mod roles1e {
    pub const A: u8 = 0;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const AUTH: u8 = 7;
    pub const AUTH_RESOURCE: u8 = 28;
    pub const DATABASE: u8 = 8;
    pub const DATABASE_RESOURCE: u8 = 29;
    pub const API: u8 = 9;
    pub const API_RESOURCE: u8 = 30;
}
pub struct Programs1E {
    pub a: RoleProgram<{ roles1e::A }>,
    pub a_wasi: RoleProgram<{ roles1e::A_WASI }>,
    pub a_io: RoleProgram<{ roles1e::A_IO }>,
    pub b: RoleProgram<{ roles1e::B }>,
    pub b_wasi: RoleProgram<{ roles1e::B_WASI }>,
    pub b_io: RoleProgram<{ roles1e::B_IO }>,
    pub c: RoleProgram<{ roles1e::C }>,
    pub c_wasi: RoleProgram<{ roles1e::C_WASI }>,
    pub c_io: RoleProgram<{ roles1e::C_IO }>,
    pub auth: RoleProgram<{ roles1e::AUTH }>,
    pub auth_resource: RoleProgram<{ roles1e::AUTH_RESOURCE }>,
    pub database: RoleProgram<{ roles1e::DATABASE }>,
    pub database_resource: RoleProgram<{ roles1e::DATABASE_RESOURCE }>,
    pub api: RoleProgram<{ roles1e::API }>,
    pub api_resource: RoleProgram<{ roles1e::API_RESOURCE }>,
}
pub fn project_1e() -> Programs1E {
    use roles1e::*;
    let dev_0 = developer::<EASY, A_WASI, A, AUTH, DATABASE, AUTH_RESOURCE, DATABASE_RESOURCE>();
    let dev_1 = developer::<EASY, B_WASI, B, API, DATABASE, API_RESOURCE, DATABASE_RESOURCE>();
    let dev_2 = developer::<EASY, C_WASI, C, API, AUTH, API_RESOURCE, AUTH_RESOURCE>();
    let task = g::seq(dev_0, g::seq(dev_1, dev_2));
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
    Programs1E {
        a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        auth: project(&protocol),
        auth_resource: project(&protocol),
        database: project(&protocol),
        database_resource: project(&protocol),
        api: project(&protocol),
        api_resource: project(&protocol),
    }
}
pub mod roles1m {
    pub const A: u8 = 0;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D: u8 = 3;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E: u8 = 4;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const AUTH: u8 = 7;
    pub const AUTH_RESOURCE: u8 = 28;
    pub const DATABASE: u8 = 8;
    pub const DATABASE_RESOURCE: u8 = 29;
    pub const API: u8 = 9;
    pub const API_RESOURCE: u8 = 30;
    pub const PAYMENT: u8 = 10;
    pub const PAYMENT_RESOURCE: u8 = 31;
    pub const NOTIFY: u8 = 11;
    pub const NOTIFY_RESOURCE: u8 = 32;
}
pub struct Programs1M {
    pub a: RoleProgram<{ roles1m::A }>,
    pub a_wasi: RoleProgram<{ roles1m::A_WASI }>,
    pub a_io: RoleProgram<{ roles1m::A_IO }>,
    pub b: RoleProgram<{ roles1m::B }>,
    pub b_wasi: RoleProgram<{ roles1m::B_WASI }>,
    pub b_io: RoleProgram<{ roles1m::B_IO }>,
    pub c: RoleProgram<{ roles1m::C }>,
    pub c_wasi: RoleProgram<{ roles1m::C_WASI }>,
    pub c_io: RoleProgram<{ roles1m::C_IO }>,
    pub d: RoleProgram<{ roles1m::D }>,
    pub d_wasi: RoleProgram<{ roles1m::D_WASI }>,
    pub d_io: RoleProgram<{ roles1m::D_IO }>,
    pub e: RoleProgram<{ roles1m::E }>,
    pub e_wasi: RoleProgram<{ roles1m::E_WASI }>,
    pub e_io: RoleProgram<{ roles1m::E_IO }>,
    pub auth: RoleProgram<{ roles1m::AUTH }>,
    pub auth_resource: RoleProgram<{ roles1m::AUTH_RESOURCE }>,
    pub database: RoleProgram<{ roles1m::DATABASE }>,
    pub database_resource: RoleProgram<{ roles1m::DATABASE_RESOURCE }>,
    pub api: RoleProgram<{ roles1m::API }>,
    pub api_resource: RoleProgram<{ roles1m::API_RESOURCE }>,
    pub payment: RoleProgram<{ roles1m::PAYMENT }>,
    pub payment_resource: RoleProgram<{ roles1m::PAYMENT_RESOURCE }>,
    pub notify: RoleProgram<{ roles1m::NOTIFY }>,
    pub notify_resource: RoleProgram<{ roles1m::NOTIFY_RESOURCE }>,
}
pub fn project_1m() -> Programs1M {
    use roles1m::*;
    let dev_0 = developer::<MEDIUM, A_WASI, A, AUTH, DATABASE, AUTH_RESOURCE, DATABASE_RESOURCE>();
    let dev_1 = developer::<MEDIUM, B_WASI, B, API, DATABASE, API_RESOURCE, DATABASE_RESOURCE>();
    let dev_2 = developer::<MEDIUM, C_WASI, C, API, PAYMENT, API_RESOURCE, PAYMENT_RESOURCE>();
    let dev_3 =
        developer::<MEDIUM, D_WASI, D, NOTIFY, PAYMENT, NOTIFY_RESOURCE, PAYMENT_RESOURCE>();
    let dev_4 = developer::<MEDIUM, E_WASI, E, AUTH, NOTIFY, AUTH_RESOURCE, NOTIFY_RESOURCE>();
    let task = g::seq(g::par(dev_0, dev_2), g::seq(g::par(dev_1, dev_3), dev_4));
    let protocol = g::par(
        task,
        g::par(
            crate::shell::global::private_io::<A_WASI, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B_WASI, B_IO>(),
                g::par(
                    crate::shell::global::private_io::<C_WASI, C_IO>(),
                    g::par(
                        crate::shell::global::private_io::<D_WASI, D_IO>(),
                        crate::shell::global::private_io::<E_WASI, E_IO>(),
                    ),
                ),
            ),
        ),
    );
    Programs1M {
        a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        auth: project(&protocol),
        auth_resource: project(&protocol),
        database: project(&protocol),
        database_resource: project(&protocol),
        api: project(&protocol),
        api_resource: project(&protocol),
        payment: project(&protocol),
        payment_resource: project(&protocol),
        notify: project(&protocol),
        notify_resource: project(&protocol),
    }
}

pub type TestPassed<'a> = g::Msg<183, &'a [u8]>;
pub type TestFailed<'a> = g::Msg<184, &'a [u8]>;
// Each result names the developer's other module: its resource ID on success,
// its owner ID on failure. The ring's distinct pairs identify both developers
// at a module, and each developer's two parallel notifications stay distinct.
pub type ModuleTestResult<'a, const LABEL: u8> = g::Msg<LABEL, &'a [u8]>;

fn revision_attempt<
    const VM: u8,
    const OWNER: u8,
    const FIRST: u8,
    const SECOND: u8,
    const FIRST_RESOURCE: u8,
    const SECOND_RESOURCE: u8,
>() -> impl Projectable {
    g::seq(
        g::seq(
            g::send::<VM, OWNER, ImplementRequest<HARD, OWNER>>(),
            g::send::<OWNER, VM, ImplementCompleted>(),
        ),
        g::seq(
            g::par(
                g::seq(
                    g::send::<FIRST, VM, Loan<FIRST_RESOURCE>>(),
                    g::send::<FIRST_RESOURCE, OWNER, CommitRequest<HARD, OWNER, FIRST_RESOURCE>>(),
                ),
                g::seq(
                    g::send::<SECOND, VM, Loan<SECOND_RESOURCE>>(),
                    g::send::<SECOND_RESOURCE, OWNER, CommitRequest<HARD, OWNER, SECOND_RESOURCE>>(
                    ),
                ),
            ),
            g::seq(
                g::par(
                    g::send::<OWNER, FIRST_RESOURCE, CommitCompleted<FIRST_RESOURCE>>(),
                    g::send::<OWNER, SECOND_RESOURCE, CommitCompleted<SECOND_RESOURCE>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<VM, FIRST, Loan<FIRST_RESOURCE>>(),
                        g::send::<VM, SECOND, Loan<SECOND_RESOURCE>>(),
                    ),
                    g::send::<VM, OWNER, TestRequest<HARD, OWNER>>(),
                ),
            ),
        ),
    )
}

fn revising_developer<
    const VM: u8,
    const OWNER: u8,
    const FIRST: u8,
    const SECOND: u8,
    const FIRST_RESOURCE: u8,
    const SECOND_RESOURCE: u8,
    const CHECK: u16,
>() -> (impl Projectable, impl Projectable) {
    let complete = g::seq(
        g::par(
            g::send::<OWNER, VM, TestPassed>(),
            g::par(
                g::send::<OWNER, FIRST, ModuleTestResult<SECOND_RESOURCE>>(),
                g::send::<OWNER, SECOND, ModuleTestResult<FIRST_RESOURCE>>(),
            ),
        ),
        g::send::<VM, OWNER, ServiceExit>(),
    );
    let revise = g::seq(
        g::par(
            g::send::<OWNER, VM, TestFailed>(),
            g::par(
                g::send::<OWNER, FIRST, ModuleTestResult<SECOND>>(),
                g::send::<OWNER, SECOND, ModuleTestResult<FIRST>>(),
            ),
        ),
        revision_attempt::<VM, OWNER, FIRST, SECOND, FIRST_RESOURCE, SECOND_RESOURCE>(),
    );
    let initial = g::seq(
        g::seq(
            g::send::<VM, OWNER, DesignRequest<HARD, OWNER>>(),
            g::send::<OWNER, VM, DesignCompleted>(),
        ),
        revision_attempt::<VM, OWNER, FIRST, SECOND, FIRST_RESOURCE, SECOND_RESOURCE>(),
    );
    (
        initial,
        g::route(complete, revise).resolve::<CHECK>().roll(),
    )
}

pub mod roles1h {
    pub const A: u8 = 0;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const CHECK_A: u16 = 300;
    pub const B: u8 = 1;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const CHECK_B: u16 = 301;
    pub const C: u8 = 2;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const CHECK_C: u16 = 302;
    pub const D: u8 = 3;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const CHECK_D: u16 = 303;
    pub const E: u8 = 4;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const CHECK_E: u16 = 304;
    pub const F: u8 = 5;
    pub const F_WASI: u8 = 19;
    pub const F_IO: u8 = 26;
    pub const CHECK_F: u16 = 305;
    pub const G: u8 = 6;
    pub const G_WASI: u8 = 20;
    pub const G_IO: u8 = 27;
    pub const CHECK_G: u16 = 306;
    pub const AUTH: u8 = 7;
    pub const AUTH_RESOURCE: u8 = 28;
    pub const DATABASE: u8 = 8;
    pub const DATABASE_RESOURCE: u8 = 29;
    pub const API: u8 = 9;
    pub const API_RESOURCE: u8 = 30;
    pub const PAYMENT: u8 = 10;
    pub const PAYMENT_RESOURCE: u8 = 31;
    pub const NOTIFY: u8 = 11;
    pub const NOTIFY_RESOURCE: u8 = 32;
    pub const ANALYTICS: u8 = 12;
    pub const ANALYTICS_RESOURCE: u8 = 33;
    pub const SEARCH: u8 = 13;
    pub const SEARCH_RESOURCE: u8 = 34;
}
pub struct Programs1H {
    pub a: RoleProgram<{ roles1h::A }>,
    pub a_wasi: RoleProgram<{ roles1h::A_WASI }>,
    pub a_io: RoleProgram<{ roles1h::A_IO }>,
    pub b: RoleProgram<{ roles1h::B }>,
    pub b_wasi: RoleProgram<{ roles1h::B_WASI }>,
    pub b_io: RoleProgram<{ roles1h::B_IO }>,
    pub c: RoleProgram<{ roles1h::C }>,
    pub c_wasi: RoleProgram<{ roles1h::C_WASI }>,
    pub c_io: RoleProgram<{ roles1h::C_IO }>,
    pub d: RoleProgram<{ roles1h::D }>,
    pub d_wasi: RoleProgram<{ roles1h::D_WASI }>,
    pub d_io: RoleProgram<{ roles1h::D_IO }>,
    pub e: RoleProgram<{ roles1h::E }>,
    pub e_wasi: RoleProgram<{ roles1h::E_WASI }>,
    pub e_io: RoleProgram<{ roles1h::E_IO }>,
    pub f: RoleProgram<{ roles1h::F }>,
    pub f_wasi: RoleProgram<{ roles1h::F_WASI }>,
    pub f_io: RoleProgram<{ roles1h::F_IO }>,
    pub g: RoleProgram<{ roles1h::G }>,
    pub g_wasi: RoleProgram<{ roles1h::G_WASI }>,
    pub g_io: RoleProgram<{ roles1h::G_IO }>,
    pub auth: RoleProgram<{ roles1h::AUTH }>,
    pub auth_resource: RoleProgram<{ roles1h::AUTH_RESOURCE }>,
    pub database: RoleProgram<{ roles1h::DATABASE }>,
    pub database_resource: RoleProgram<{ roles1h::DATABASE_RESOURCE }>,
    pub api: RoleProgram<{ roles1h::API }>,
    pub api_resource: RoleProgram<{ roles1h::API_RESOURCE }>,
    pub payment: RoleProgram<{ roles1h::PAYMENT }>,
    pub payment_resource: RoleProgram<{ roles1h::PAYMENT_RESOURCE }>,
    pub notify: RoleProgram<{ roles1h::NOTIFY }>,
    pub notify_resource: RoleProgram<{ roles1h::NOTIFY_RESOURCE }>,
    pub analytics: RoleProgram<{ roles1h::ANALYTICS }>,
    pub analytics_resource: RoleProgram<{ roles1h::ANALYTICS_RESOURCE }>,
    pub search: RoleProgram<{ roles1h::SEARCH }>,
    pub search_resource: RoleProgram<{ roles1h::SEARCH_RESOURCE }>,
}
pub fn project_1h() -> Programs1H {
    use roles1h::*;
    let (initial_0, result_0) =
        revising_developer::<A_WASI, A, AUTH, DATABASE, AUTH_RESOURCE, DATABASE_RESOURCE, CHECK_A>(
        );
    let (initial_1, result_1) =
        revising_developer::<B_WASI, B, API, DATABASE, API_RESOURCE, DATABASE_RESOURCE, CHECK_B>();
    let (initial_2, result_2) =
        revising_developer::<C_WASI, C, API, PAYMENT, API_RESOURCE, PAYMENT_RESOURCE, CHECK_C>();
    let (initial_3, result_3) = revising_developer::<
        D_WASI,
        D,
        NOTIFY,
        PAYMENT,
        NOTIFY_RESOURCE,
        PAYMENT_RESOURCE,
        CHECK_D,
    >();
    let (initial_4, result_4) = revising_developer::<
        E_WASI,
        E,
        ANALYTICS,
        NOTIFY,
        ANALYTICS_RESOURCE,
        NOTIFY_RESOURCE,
        CHECK_E,
    >();
    let (initial_5, result_5) = revising_developer::<
        F_WASI,
        F,
        ANALYTICS,
        SEARCH,
        ANALYTICS_RESOURCE,
        SEARCH_RESOURCE,
        CHECK_F,
    >();
    let (initial_6, result_6) =
        revising_developer::<G_WASI, G, AUTH, SEARCH, AUTH_RESOURCE, SEARCH_RESOURCE, CHECK_G>();
    let task = g::seq(
        g::seq(
            g::par(initial_0, g::par(initial_2, initial_4)),
            g::seq(g::par(initial_1, g::par(initial_3, initial_5)), initial_6),
        ),
        g::seq(
            g::par(result_0, g::par(result_2, result_4)),
            g::seq(g::par(result_1, g::par(result_3, result_5)), result_6),
        ),
    );
    let protocol = g::par(
        task,
        g::par(
            crate::shell::global::private_io::<A_WASI, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B_WASI, B_IO>(),
                g::par(
                    crate::shell::global::private_io::<C_WASI, C_IO>(),
                    g::par(
                        crate::shell::global::private_io::<D_WASI, D_IO>(),
                        g::par(
                            crate::shell::global::private_io::<E_WASI, E_IO>(),
                            g::par(
                                crate::shell::global::private_io::<F_WASI, F_IO>(),
                                crate::shell::global::private_io::<G_WASI, G_IO>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    Programs1H {
        a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        f: project(&protocol),
        f_wasi: project(&protocol),
        f_io: project(&protocol),
        g: project(&protocol),
        g_wasi: project(&protocol),
        g_io: project(&protocol),
        auth: project(&protocol),
        auth_resource: project(&protocol),
        database: project(&protocol),
        database_resource: project(&protocol),
        api: project(&protocol),
        api_resource: project(&protocol),
        payment: project(&protocol),
        payment_resource: project(&protocol),
        notify: project(&protocol),
        notify_resource: project(&protocol),
        analytics: project(&protocol),
        analytics_resource: project(&protocol),
        search: project(&protocol),
        search_resource: project(&protocol),
    }
}

impl Programs1E {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1E","programs":[rows(&self.a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.auth), rows(&self.auth_resource), rows(&self.database), rows(&self.database_resource), rows(&self.api), rows(&self.api_resource)]})
    }
}

impl Programs1M {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1M","programs":[rows(&self.a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.d), rows(&self.d_wasi), rows(&self.d_io), rows(&self.e), rows(&self.e_wasi), rows(&self.e_io), rows(&self.auth), rows(&self.auth_resource), rows(&self.database), rows(&self.database_resource), rows(&self.api), rows(&self.api_resource), rows(&self.payment), rows(&self.payment_resource), rows(&self.notify), rows(&self.notify_resource)]})
    }
}

impl Programs1H {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1H","programs":[rows(&self.a), rows(&self.a_wasi), rows(&self.a_io), rows(&self.b), rows(&self.b_wasi), rows(&self.b_io), rows(&self.c), rows(&self.c_wasi), rows(&self.c_io), rows(&self.d), rows(&self.d_wasi), rows(&self.d_io), rows(&self.e), rows(&self.e_wasi), rows(&self.e_io), rows(&self.f), rows(&self.f_wasi), rows(&self.f_io), rows(&self.g), rows(&self.g_wasi), rows(&self.g_io), rows(&self.auth), rows(&self.auth_resource), rows(&self.database), rows(&self.database_resource), rows(&self.api), rows(&self.api_resource), rows(&self.payment), rows(&self.payment_resource), rows(&self.notify), rows(&self.notify_resource), rows(&self.analytics), rows(&self.analytics_resource), rows(&self.search), rows(&self.search_resource)]})
    }
}
