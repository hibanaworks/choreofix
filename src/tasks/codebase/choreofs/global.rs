//! Direct file requests, actual module Endpoints and original effect results.
use super::wire::{Arguments, CommitSubmission};
use crate::shell::global::FileExit;
pub use crate::shell::global::Loan;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const SESSION: u32 = 20;
pub type DesignRequest<const ROLE: u8> = g::Msg<1, Arguments<ROLE, 0>>;
pub type ImplementRequest<const ROLE: u8> = g::Msg<2, Arguments<ROLE, 1>>;
pub type CommitRequest<const ROLE: u8> = g::Msg<3, CommitSubmission<ROLE>>;
pub type TestRequest<const ROLE: u8> = g::Msg<4, Arguments<ROLE, 3>>;
pub type DesignCompleted<'a> = g::Msg<5, &'a [u8]>;
pub type ImplementCompleted<'a> = g::Msg<6, &'a [u8]>;
pub type CommitCompleted<'a> = g::Msg<7, &'a [u8]>;
pub type TestCompleted<'a> = g::Msg<8, &'a [u8]>;
pub type ModuleRequest<const RESOURCE: u8, const DEV: u8> = g::Msg<RESOURCE, Arguments<DEV, 2>>;
pub type ModuleCompleted<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;

const fn object(
    path: &'static [u8],
    fd: u8,
) -> hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static> {
    use hibana_wasip1_runtime::{
        choreofs::{ChoreoFsObject, FdSpec, ObjectId},
        protocol,
    };
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}
pub const FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 4] = [
    object(b"service/design_feature", 4),
    object(b"service/implement_code", 5),
    object(b"service/commit_changes", 17),
    object(b"service/run_local_tests", 18),
];

fn preparation<const OWNER: u8, const FILE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, DesignRequest<OWNER>>(),
        g::seq(
            g::send::<OWNER, FILE, DesignCompleted>(),
            g::seq(
                g::send::<FILE, OWNER, ImplementRequest<OWNER>>(),
                g::send::<OWNER, FILE, ImplementCompleted>(),
            ),
        ),
    )
}

// Reused by all fifteen researchers. The argument edges remain explicit at
// each call site; this is the same two-resource choreography, not a dispatcher.
fn commit<const OWNER: u8, const FIRST: u8, const SECOND: u8>(
    (request, reply): (impl Projectable + Copy, impl Projectable + Copy),
    (first_loan, second_loan): (impl Projectable + Copy, impl Projectable + Copy),
    (first_return, second_return): (impl Projectable + Copy, impl Projectable + Copy),
) -> impl Projectable + Copy {
    g::seq(
        request,
        g::seq(
            g::par(first_loan, second_loan),
            g::seq(
                g::par(
                    g::send::<FIRST, OWNER, ModuleRequest<FIRST, OWNER>>(),
                    g::send::<SECOND, OWNER, ModuleRequest<SECOND, OWNER>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<OWNER, FIRST, ModuleCompleted<FIRST>>(),
                        g::send::<OWNER, SECOND, ModuleCompleted<SECOND>>(),
                    ),
                    g::seq(g::par(first_return, second_return), reply),
                ),
            ),
        ),
    )
}
fn test<const OWNER: u8, const FILE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, TestRequest<OWNER>>(),
        g::send::<OWNER, FILE, TestCompleted>(),
    )
}

pub type Submitted<const ROLE: u8> = g::Msg<ROLE, CommitSubmission<ROLE>>;
pub mod roles1e {
    pub const A: u8 = 0;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const B: u8 = 1;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const C: u8 = 2;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const REQUESTS: u8 = 31;
    pub const REPOSITORY: u8 = 32;
    pub const AUTH_RESOURCE: u8 = 64;
    pub const DATABASE_RESOURCE: u8 = 65;
    pub const API_RESOURCE: u8 = 66;
}
pub struct Programs1E {
    pub a: RoleProgram<{ roles1e::A }>,
    pub a_file: RoleProgram<{ roles1e::A_FILE }>,
    pub a_vm: RoleProgram<{ roles1e::A_VM }>,
    pub a_io: RoleProgram<{ roles1e::A_IO }>,
    pub a_data: RoleProgram<{ roles1e::A_DATA }>,
    pub b: RoleProgram<{ roles1e::B }>,
    pub b_file: RoleProgram<{ roles1e::B_FILE }>,
    pub b_vm: RoleProgram<{ roles1e::B_VM }>,
    pub b_io: RoleProgram<{ roles1e::B_IO }>,
    pub b_data: RoleProgram<{ roles1e::B_DATA }>,
    pub c: RoleProgram<{ roles1e::C }>,
    pub c_file: RoleProgram<{ roles1e::C_FILE }>,
    pub c_vm: RoleProgram<{ roles1e::C_VM }>,
    pub c_io: RoleProgram<{ roles1e::C_IO }>,
    pub c_data: RoleProgram<{ roles1e::C_DATA }>,
    pub requests: RoleProgram<{ roles1e::REQUESTS }>,
    pub repository: RoleProgram<{ roles1e::REPOSITORY }>,
    pub auth_resource: RoleProgram<{ roles1e::AUTH_RESOURCE }>,
    pub database_resource: RoleProgram<{ roles1e::DATABASE_RESOURCE }>,
    pub api_resource: RoleProgram<{ roles1e::API_RESOURCE }>,
}
pub fn project_1e() -> Programs1E {
    use roles1e::*;
    let commit_a = commit::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<REPOSITORY, A, CommitRequest<A>>(),
            g::send::<A, A_FILE, CommitCompleted>(),
        ),
        (
            g::send::<REPOSITORY, A, Loan<AUTH_RESOURCE>>(),
            g::send::<REPOSITORY, A, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<A, REPOSITORY, Loan<AUTH_RESOURCE>>(),
            g::send::<A, REPOSITORY, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_b = commit::<B, API_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<REPOSITORY, B, CommitRequest<B>>(),
            g::send::<B, B_FILE, CommitCompleted>(),
        ),
        (
            g::send::<REPOSITORY, B, Loan<API_RESOURCE>>(),
            g::send::<REPOSITORY, B, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<B, REPOSITORY, Loan<API_RESOURCE>>(),
            g::send::<B, REPOSITORY, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_c = commit::<C, API_RESOURCE, AUTH_RESOURCE>(
        (
            g::send::<REPOSITORY, C, CommitRequest<C>>(),
            g::send::<C, C_FILE, CommitCompleted>(),
        ),
        (
            g::send::<REPOSITORY, C, Loan<API_RESOURCE>>(),
            g::send::<REPOSITORY, C, Loan<AUTH_RESOURCE>>(),
        ),
        (
            g::send::<C, REPOSITORY, Loan<API_RESOURCE>>(),
            g::send::<C, REPOSITORY, Loan<AUTH_RESOURCE>>(),
        ),
    );
    let ready = g::par(
        g::seq(
            preparation::<A, A_FILE>(),
            g::send::<A_FILE, REQUESTS, Submitted<A>>(),
        ),
        g::par(
            g::seq(
                preparation::<B, B_FILE>(),
                g::send::<B_FILE, REQUESTS, Submitted<B>>(),
            ),
            g::seq(
                preparation::<C, C_FILE>(),
                g::send::<C_FILE, REQUESTS, Submitted<C>>(),
            ),
        ),
    );
    let commits = g::route(
        g::seq(
            commit_a,
            g::route(g::seq(commit_b, commit_c), g::seq(commit_c, commit_b)),
        ),
        g::route(
            g::seq(
                commit_b,
                g::route(g::seq(commit_a, commit_c), g::seq(commit_c, commit_a)),
            ),
            g::seq(
                commit_c,
                g::route(g::seq(commit_a, commit_b), g::seq(commit_b, commit_a)),
            ),
        ),
    );
    let tests = g::par(
        test::<A, A_FILE>(),
        g::par(test::<B, B_FILE>(), test::<C, C_FILE>()),
    );
    let task = g::seq(ready, g::seq(commits, tests));
    let protocol = g::seq(
        g::par(
            g::par(
                g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
                g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            ),
            g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
        ),
        g::par(
            task,
            g::par(
                g::par(
                    g::par(
                        g::seq(
                            crate::shell::global::private_io::<A_VM, A_IO>(),
                            g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                        ),
                        g::seq(
                            crate::shell::global::private_io::<B_VM, B_IO>(),
                            g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<C_VM, C_IO>(),
                        g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
                    ),
                ),
                g::par(
                    g::par(
                        g::seq(
                            crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
                            g::send::<A_FILE, A_DATA, FileExit>(),
                        ),
                        g::seq(
                            crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                            g::send::<B_FILE, B_DATA, FileExit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<C_FILE, C_DATA>().roll(),
                        g::send::<C_FILE, C_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    Programs1E {
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        requests: project(&protocol),
        repository: project(&protocol),
        auth_resource: project(&protocol),
        database_resource: project(&protocol),
        api_resource: project(&protocol),
    }
}
impl Programs1E {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles1e::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1E","session":SESSION,"submissions":REQUESTS,"repository":REPOSITORY,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[API_RESOURCE,AUTH_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":REPOSITORY},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":REPOSITORY},"API_MODULE":{"role":API_RESOURCE,"service":REPOSITORY}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.requests),rows(&self.repository),rows(&self.auth_resource),rows(&self.database_resource),rows(&self.api_resource)]})
    }
}
pub type ModuleClaim<const LABEL: u8, const DEV: u8> = g::Msg<LABEL, CommitSubmission<DEV>>;

// One physical module can serve either of its two developers first.
fn module_use<const DEV: u8, const OWNER: u8, const RESOURCE: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<OWNER, DEV, Loan<RESOURCE>>(),
        g::seq(
            g::send::<RESOURCE, DEV, ModuleRequest<RESOURCE, DEV>>(),
            g::seq(
                g::send::<DEV, RESOURCE, ModuleCompleted<RESOURCE>>(),
                g::send::<DEV, OWNER, Loan<RESOURCE>>(),
            ),
        ),
    )
}
pub mod roles1m {
    pub const REQUESTS: u8 = 31;
    pub const REPOSITORY: u8 = 30;
    pub const A: u8 = 3;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const B: u8 = 4;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const C: u8 = 5;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const D: u8 = 6;
    pub const D_FILE: u8 = 83;
    pub const D_VM: u8 = 99;
    pub const D_IO: u8 = 115;
    pub const D_DATA: u8 = 131;
    pub const E: u8 = 7;
    pub const E_FILE: u8 = 84;
    pub const E_VM: u8 = 100;
    pub const E_IO: u8 = 116;
    pub const E_DATA: u8 = 132;
    pub const AUTH: u8 = 32;
    pub const AUTH_RESOURCE: u8 = 64;
    pub const DATABASE: u8 = 33;
    pub const DATABASE_RESOURCE: u8 = 65;
    pub const API: u8 = 34;
    pub const API_RESOURCE: u8 = 66;
    pub const PAYMENT: u8 = 35;
    pub const PAYMENT_RESOURCE: u8 = 67;
    pub const NOTIFY: u8 = 36;
    pub const NOTIFY_RESOURCE: u8 = 68;
}
pub struct Programs1M {
    pub requests: RoleProgram<{ roles1m::REQUESTS }>,
    pub repository: RoleProgram<{ roles1m::REPOSITORY }>,
    pub a: RoleProgram<{ roles1m::A }>,
    pub a_file: RoleProgram<{ roles1m::A_FILE }>,
    pub a_vm: RoleProgram<{ roles1m::A_VM }>,
    pub a_io: RoleProgram<{ roles1m::A_IO }>,
    pub a_data: RoleProgram<{ roles1m::A_DATA }>,
    pub b: RoleProgram<{ roles1m::B }>,
    pub b_file: RoleProgram<{ roles1m::B_FILE }>,
    pub b_vm: RoleProgram<{ roles1m::B_VM }>,
    pub b_io: RoleProgram<{ roles1m::B_IO }>,
    pub b_data: RoleProgram<{ roles1m::B_DATA }>,
    pub c: RoleProgram<{ roles1m::C }>,
    pub c_file: RoleProgram<{ roles1m::C_FILE }>,
    pub c_vm: RoleProgram<{ roles1m::C_VM }>,
    pub c_io: RoleProgram<{ roles1m::C_IO }>,
    pub c_data: RoleProgram<{ roles1m::C_DATA }>,
    pub d: RoleProgram<{ roles1m::D }>,
    pub d_file: RoleProgram<{ roles1m::D_FILE }>,
    pub d_vm: RoleProgram<{ roles1m::D_VM }>,
    pub d_io: RoleProgram<{ roles1m::D_IO }>,
    pub d_data: RoleProgram<{ roles1m::D_DATA }>,
    pub e: RoleProgram<{ roles1m::E }>,
    pub e_file: RoleProgram<{ roles1m::E_FILE }>,
    pub e_vm: RoleProgram<{ roles1m::E_VM }>,
    pub e_io: RoleProgram<{ roles1m::E_IO }>,
    pub e_data: RoleProgram<{ roles1m::E_DATA }>,
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
    let preparation = g::par(
        g::par(
            g::par(preparation::<A, A_FILE>(), preparation::<B, B_FILE>()),
            preparation::<C, C_FILE>(),
        ),
        g::par(preparation::<D, D_FILE>(), preparation::<E, E_FILE>()),
    );
    let submissions = g::par(
        g::par(
            g::par(
                g::send::<A_FILE, REQUESTS, Submitted<A>>(),
                g::send::<B_FILE, REQUESTS, Submitted<B>>(),
            ),
            g::send::<C_FILE, REQUESTS, Submitted<C>>(),
        ),
        g::par(
            g::send::<D_FILE, REQUESTS, Submitted<D>>(),
            g::send::<E_FILE, REQUESTS, Submitted<E>>(),
        ),
    );
    // A module owner requests the next physical module only when it holds the first.
    // The acquisition order is database, payment, auth, API, notify; no developer has precedence.
    let api_b = g::seq(
        g::send::<REPOSITORY, API, ModuleClaim<B_FILE, B>>(),
        module_use::<B, API, API_RESOURCE>(),
    );
    let api_c = g::seq(
        g::send::<REPOSITORY, API, ModuleClaim<C_FILE, C>>(),
        module_use::<C, API, API_RESOURCE>(),
    );
    let api = g::route(g::seq(api_b, api_c), g::seq(api_c, api_b));
    let auth_a = g::seq(
        g::send::<REPOSITORY, AUTH, ModuleClaim<A_FILE, A>>(),
        module_use::<A, AUTH, AUTH_RESOURCE>(),
    );
    let auth_e = g::seq(
        g::send::<REPOSITORY, AUTH, Submitted<E>>(),
        g::seq(
            g::send::<AUTH, REQUESTS, ModuleClaim<E_FILE, E>>(),
            g::seq(
                g::send::<AUTH, E, CommitRequest<E>>(),
                module_use::<E, AUTH, AUTH_RESOURCE>(),
            ),
        ),
    );
    let auth = g::route(g::seq(auth_a, auth_e), g::seq(auth_e, auth_a));
    let database_a = g::seq(
        g::send::<REPOSITORY, DATABASE, Submitted<A>>(),
        g::seq(
            g::send::<DATABASE, REQUESTS, ModuleClaim<A_FILE, A>>(),
            g::seq(
                g::send::<DATABASE, A, CommitRequest<A>>(),
                module_use::<A, DATABASE, DATABASE_RESOURCE>(),
            ),
        ),
    );
    let database_b = g::seq(
        g::send::<REPOSITORY, DATABASE, Submitted<B>>(),
        g::seq(
            g::send::<DATABASE, REQUESTS, ModuleClaim<B_FILE, B>>(),
            g::seq(
                g::send::<DATABASE, B, CommitRequest<B>>(),
                module_use::<B, DATABASE, DATABASE_RESOURCE>(),
            ),
        ),
    );
    let database = g::route(
        g::seq(database_a, database_b),
        g::seq(database_b, database_a),
    );
    let notify_d = g::seq(
        g::send::<REPOSITORY, NOTIFY, ModuleClaim<D_FILE, D>>(),
        module_use::<D, NOTIFY, NOTIFY_RESOURCE>(),
    );
    let notify_e = g::seq(
        g::send::<REPOSITORY, NOTIFY, ModuleClaim<E_FILE, E>>(),
        module_use::<E, NOTIFY, NOTIFY_RESOURCE>(),
    );
    let notify = g::route(g::seq(notify_d, notify_e), g::seq(notify_e, notify_d));
    let payment_c = g::seq(
        g::send::<REPOSITORY, PAYMENT, Submitted<C>>(),
        g::seq(
            g::send::<PAYMENT, REQUESTS, ModuleClaim<C_FILE, C>>(),
            g::seq(
                g::send::<PAYMENT, C, CommitRequest<C>>(),
                module_use::<C, PAYMENT, PAYMENT_RESOURCE>(),
            ),
        ),
    );
    let payment_d = g::seq(
        g::send::<REPOSITORY, PAYMENT, Submitted<D>>(),
        g::seq(
            g::send::<PAYMENT, REQUESTS, ModuleClaim<D_FILE, D>>(),
            g::seq(
                g::send::<PAYMENT, D, CommitRequest<D>>(),
                module_use::<D, PAYMENT, PAYMENT_RESOURCE>(),
            ),
        ),
    );
    let payment = g::route(g::seq(payment_c, payment_d), g::seq(payment_d, payment_c));
    let modules = g::par(g::par(g::par(api, auth), database), g::par(notify, payment));
    let results = g::par(
        g::par(
            g::par(
                g::seq(g::send::<A, A_FILE, CommitCompleted>(), test::<A, A_FILE>()),
                g::seq(g::send::<B, B_FILE, CommitCompleted>(), test::<B, B_FILE>()),
            ),
            g::seq(g::send::<C, C_FILE, CommitCompleted>(), test::<C, C_FILE>()),
        ),
        g::par(
            g::seq(g::send::<D, D_FILE, CommitCompleted>(), test::<D, D_FILE>()),
            g::seq(g::send::<E, E_FILE, CommitCompleted>(), test::<E, E_FILE>()),
        ),
    );
    let task = g::seq(preparation, g::par(submissions, g::seq(modules, results)));
    let protocol = g::seq(
        g::par(
            g::par(
                g::par(
                    g::par(
                        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
                        g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
                    ),
                    g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
                ),
                g::send::<D_DATA, D_VM, Loan<D_FILE>>(),
            ),
            g::send::<E_DATA, E_VM, Loan<E_FILE>>(),
        ),
        g::par(
            task,
            g::par(
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::seq(
                                    crate::shell::global::private_io::<A_VM, A_IO>(),
                                    g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                                ),
                                g::seq(
                                    crate::shell::global::private_io::<B_VM, B_IO>(),
                                    g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                                ),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<C_VM, C_IO>(),
                                g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
                            ),
                        ),
                        g::seq(
                            crate::shell::global::private_io::<D_VM, D_IO>(),
                            g::send::<D_VM, D_IO, crate::shell::global::Exit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<E_VM, E_IO>(),
                        g::send::<E_VM, E_IO, crate::shell::global::Exit>(),
                    ),
                ),
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::seq(
                                    crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
                                    g::send::<A_FILE, A_DATA, FileExit>(),
                                ),
                                g::seq(
                                    crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                                    g::send::<B_FILE, B_DATA, FileExit>(),
                                ),
                            ),
                            g::seq(
                                crate::shell::global::file_metadata::<C_FILE, C_DATA>().roll(),
                                g::send::<C_FILE, C_DATA, FileExit>(),
                            ),
                        ),
                        g::seq(
                            crate::shell::global::file_metadata::<D_FILE, D_DATA>().roll(),
                            g::send::<D_FILE, D_DATA, FileExit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<E_FILE, E_DATA>().roll(),
                        g::send::<E_FILE, E_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    Programs1M {
        requests: project(&protocol),
        repository: project(&protocol),
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        d: project(&protocol),
        d_file: project(&protocol),
        d_vm: project(&protocol),
        d_io: project(&protocol),
        d_data: project(&protocol),
        e: project(&protocol),
        e_file: project(&protocol),
        e_vm: project(&protocol),
        e_io: project(&protocol),
        e_data: project(&protocol),
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
impl Programs1M {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles1m::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1M","session":SESSION,"submissions":REQUESTS,"repository":REPOSITORY,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[API_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resources":[NOTIFY_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resources":[AUTH_RESOURCE,NOTIFY_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":AUTH},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":DATABASE},"API_MODULE":{"role":API_RESOURCE,"service":API},"PAYMENT_MODULE":{"role":PAYMENT_RESOURCE,"service":PAYMENT},"NOTIFY_MODULE":{"role":NOTIFY_RESOURCE,"service":NOTIFY}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.requests),rows(&self.repository),rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.auth),rows(&self.auth_resource),rows(&self.database),rows(&self.database_resource),rows(&self.api),rows(&self.api_resource),rows(&self.payment),rows(&self.payment_resource),rows(&self.notify),rows(&self.notify_resource)]})
    }
}
pub type ModuleFileClosed<const MODULE: u8> = g::Msg<MODULE, i32>;
pub type ModuleLoan<'run, const DEV: u8, const RESOURCE: u8> =
    g::Msg<DEV, hibana::Endpoint<'run, RESOURCE>>;

pub mod roles1h {
    pub const A: u8 = 8;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const CHECK_A: u16 = 300;
    pub const B: u8 = 9;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const CHECK_B: u16 = 301;
    pub const C: u8 = 10;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const CHECK_C: u16 = 302;
    pub const D: u8 = 11;
    pub const D_FILE: u8 = 83;
    pub const D_VM: u8 = 99;
    pub const D_IO: u8 = 115;
    pub const D_DATA: u8 = 131;
    pub const CHECK_D: u16 = 303;
    pub const E: u8 = 12;
    pub const E_FILE: u8 = 84;
    pub const E_VM: u8 = 100;
    pub const E_IO: u8 = 116;
    pub const E_DATA: u8 = 132;
    pub const CHECK_E: u16 = 304;
    pub const F: u8 = 13;
    pub const F_FILE: u8 = 85;
    pub const F_VM: u8 = 101;
    pub const F_IO: u8 = 117;
    pub const F_DATA: u8 = 133;
    pub const CHECK_F: u16 = 305;
    pub const G: u8 = 14;
    pub const G_FILE: u8 = 86;
    pub const G_VM: u8 = 102;
    pub const G_IO: u8 = 118;
    pub const G_DATA: u8 = 134;
    pub const CHECK_G: u16 = 306;
    pub const AUTH_REQUESTS: u8 = 150;
    pub const AUTH: u8 = 32;
    pub const AUTH_RESOURCE: u8 = 64;
    pub const DATABASE_REQUESTS: u8 = 151;
    pub const DATABASE: u8 = 33;
    pub const DATABASE_RESOURCE: u8 = 65;
    pub const API_REQUESTS: u8 = 152;
    pub const API: u8 = 34;
    pub const API_RESOURCE: u8 = 66;
    pub const PAYMENT_REQUESTS: u8 = 153;
    pub const PAYMENT: u8 = 35;
    pub const PAYMENT_RESOURCE: u8 = 67;
    pub const NOTIFY_REQUESTS: u8 = 154;
    pub const NOTIFY: u8 = 36;
    pub const NOTIFY_RESOURCE: u8 = 68;
    pub const ANALYTICS_REQUESTS: u8 = 155;
    pub const ANALYTICS: u8 = 37;
    pub const ANALYTICS_RESOURCE: u8 = 69;
    pub const SEARCH_REQUESTS: u8 = 156;
    pub const SEARCH: u8 = 38;
    pub const SEARCH_RESOURCE: u8 = 70;
}
pub struct Programs1H {
    pub a: RoleProgram<{ roles1h::A }>,
    pub a_file: RoleProgram<{ roles1h::A_FILE }>,
    pub a_vm: RoleProgram<{ roles1h::A_VM }>,
    pub a_io: RoleProgram<{ roles1h::A_IO }>,
    pub a_data: RoleProgram<{ roles1h::A_DATA }>,
    pub b: RoleProgram<{ roles1h::B }>,
    pub b_file: RoleProgram<{ roles1h::B_FILE }>,
    pub b_vm: RoleProgram<{ roles1h::B_VM }>,
    pub b_io: RoleProgram<{ roles1h::B_IO }>,
    pub b_data: RoleProgram<{ roles1h::B_DATA }>,
    pub c: RoleProgram<{ roles1h::C }>,
    pub c_file: RoleProgram<{ roles1h::C_FILE }>,
    pub c_vm: RoleProgram<{ roles1h::C_VM }>,
    pub c_io: RoleProgram<{ roles1h::C_IO }>,
    pub c_data: RoleProgram<{ roles1h::C_DATA }>,
    pub d: RoleProgram<{ roles1h::D }>,
    pub d_file: RoleProgram<{ roles1h::D_FILE }>,
    pub d_vm: RoleProgram<{ roles1h::D_VM }>,
    pub d_io: RoleProgram<{ roles1h::D_IO }>,
    pub d_data: RoleProgram<{ roles1h::D_DATA }>,
    pub e: RoleProgram<{ roles1h::E }>,
    pub e_file: RoleProgram<{ roles1h::E_FILE }>,
    pub e_vm: RoleProgram<{ roles1h::E_VM }>,
    pub e_io: RoleProgram<{ roles1h::E_IO }>,
    pub e_data: RoleProgram<{ roles1h::E_DATA }>,
    pub f: RoleProgram<{ roles1h::F }>,
    pub f_file: RoleProgram<{ roles1h::F_FILE }>,
    pub f_vm: RoleProgram<{ roles1h::F_VM }>,
    pub f_io: RoleProgram<{ roles1h::F_IO }>,
    pub f_data: RoleProgram<{ roles1h::F_DATA }>,
    pub g: RoleProgram<{ roles1h::G }>,
    pub g_file: RoleProgram<{ roles1h::G_FILE }>,
    pub g_vm: RoleProgram<{ roles1h::G_VM }>,
    pub g_io: RoleProgram<{ roles1h::G_IO }>,
    pub g_data: RoleProgram<{ roles1h::G_DATA }>,
    pub auth_requests: RoleProgram<{ roles1h::AUTH_REQUESTS }>,
    pub auth: RoleProgram<{ roles1h::AUTH }>,
    pub auth_resource: RoleProgram<{ roles1h::AUTH_RESOURCE }>,
    pub database_requests: RoleProgram<{ roles1h::DATABASE_REQUESTS }>,
    pub database: RoleProgram<{ roles1h::DATABASE }>,
    pub database_resource: RoleProgram<{ roles1h::DATABASE_RESOURCE }>,
    pub api_requests: RoleProgram<{ roles1h::API_REQUESTS }>,
    pub api: RoleProgram<{ roles1h::API }>,
    pub api_resource: RoleProgram<{ roles1h::API_RESOURCE }>,
    pub payment_requests: RoleProgram<{ roles1h::PAYMENT_REQUESTS }>,
    pub payment: RoleProgram<{ roles1h::PAYMENT }>,
    pub payment_resource: RoleProgram<{ roles1h::PAYMENT_RESOURCE }>,
    pub notify_requests: RoleProgram<{ roles1h::NOTIFY_REQUESTS }>,
    pub notify: RoleProgram<{ roles1h::NOTIFY }>,
    pub notify_resource: RoleProgram<{ roles1h::NOTIFY_RESOURCE }>,
    pub analytics_requests: RoleProgram<{ roles1h::ANALYTICS_REQUESTS }>,
    pub analytics: RoleProgram<{ roles1h::ANALYTICS }>,
    pub analytics_resource: RoleProgram<{ roles1h::ANALYTICS_RESOURCE }>,
    pub search_requests: RoleProgram<{ roles1h::SEARCH_REQUESTS }>,
    pub search: RoleProgram<{ roles1h::SEARCH }>,
    pub search_resource: RoleProgram<{ roles1h::SEARCH_RESOURCE }>,
}
pub fn project_1h() -> Programs1H {
    use roles1h::*;
    let a = {
        let implemented = g::seq(
            g::send::<A_FILE, A, ImplementRequest<A>>(),
            g::send::<A, A_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<A_FILE, DATABASE_REQUESTS, Submitted<A>>();
        let tested = g::seq(
            g::send::<A, A_FILE, CommitCompleted>(),
            g::send::<A_FILE, A, TestRequest<A>>(),
        );
        let verdict = g::send::<A, A_FILE, TestCompleted>();
        g::seq(
            preparation::<A, A_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<A_FILE, DATABASE_REQUESTS, ModuleFileClosed<DATABASE>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_A>()
                    .roll(),
                ),
            ),
        )
    };
    let b = {
        let implemented = g::seq(
            g::send::<B_FILE, B, ImplementRequest<B>>(),
            g::send::<B, B_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<B_FILE, DATABASE_REQUESTS, Submitted<B>>();
        let tested = g::seq(
            g::send::<B, B_FILE, CommitCompleted>(),
            g::send::<B_FILE, B, TestRequest<B>>(),
        );
        let verdict = g::send::<B, B_FILE, TestCompleted>();
        g::seq(
            preparation::<B, B_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<B_FILE, DATABASE_REQUESTS, ModuleFileClosed<DATABASE>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_B>()
                    .roll(),
                ),
            ),
        )
    };
    let c = {
        let implemented = g::seq(
            g::send::<C_FILE, C, ImplementRequest<C>>(),
            g::send::<C, C_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<C_FILE, PAYMENT_REQUESTS, Submitted<C>>();
        let tested = g::seq(
            g::send::<C, C_FILE, CommitCompleted>(),
            g::send::<C_FILE, C, TestRequest<C>>(),
        );
        let verdict = g::send::<C, C_FILE, TestCompleted>();
        g::seq(
            preparation::<C, C_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<C_FILE, PAYMENT_REQUESTS, ModuleFileClosed<PAYMENT>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_C>()
                    .roll(),
                ),
            ),
        )
    };
    let d = {
        let implemented = g::seq(
            g::send::<D_FILE, D, ImplementRequest<D>>(),
            g::send::<D, D_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<D_FILE, PAYMENT_REQUESTS, Submitted<D>>();
        let tested = g::seq(
            g::send::<D, D_FILE, CommitCompleted>(),
            g::send::<D_FILE, D, TestRequest<D>>(),
        );
        let verdict = g::send::<D, D_FILE, TestCompleted>();
        g::seq(
            preparation::<D, D_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<D_FILE, PAYMENT_REQUESTS, ModuleFileClosed<PAYMENT>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_D>()
                    .roll(),
                ),
            ),
        )
    };
    let e = {
        let implemented = g::seq(
            g::send::<E_FILE, E, ImplementRequest<E>>(),
            g::send::<E, E_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<E_FILE, ANALYTICS_REQUESTS, Submitted<E>>();
        let tested = g::seq(
            g::send::<E, E_FILE, CommitCompleted>(),
            g::send::<E_FILE, E, TestRequest<E>>(),
        );
        let verdict = g::send::<E, E_FILE, TestCompleted>();
        g::seq(
            preparation::<E, E_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<E_FILE, ANALYTICS_REQUESTS, ModuleFileClosed<ANALYTICS>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_E>()
                    .roll(),
                ),
            ),
        )
    };
    let f = {
        let implemented = g::seq(
            g::send::<F_FILE, F, ImplementRequest<F>>(),
            g::send::<F, F_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<F_FILE, ANALYTICS_REQUESTS, Submitted<F>>();
        let tested = g::seq(
            g::send::<F, F_FILE, CommitCompleted>(),
            g::send::<F_FILE, F, TestRequest<F>>(),
        );
        let verdict = g::send::<F, F_FILE, TestCompleted>();
        g::seq(
            preparation::<F, F_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<F_FILE, ANALYTICS_REQUESTS, ModuleFileClosed<ANALYTICS>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_F>()
                    .roll(),
                ),
            ),
        )
    };
    let g = {
        let implemented = g::seq(
            g::send::<G_FILE, G, ImplementRequest<G>>(),
            g::send::<G, G_FILE, ImplementCompleted>(),
        );
        let submission = g::send::<G_FILE, AUTH_REQUESTS, Submitted<G>>();
        let tested = g::seq(
            g::send::<G, G_FILE, CommitCompleted>(),
            g::send::<G_FILE, G, TestRequest<G>>(),
        );
        let verdict = g::send::<G, G_FILE, TestCompleted>();
        g::seq(
            preparation::<G, G_FILE>(),
            g::seq(
                submission,
                g::seq(
                    tested,
                    g::route(
                        g::seq(
                            verdict,
                            g::send::<G_FILE, AUTH_REQUESTS, ModuleFileClosed<AUTH>>(),
                        ),
                        g::seq(verdict, g::seq(implemented, g::seq(submission, tested))),
                    )
                    .resolve::<CHECK_G>()
                    .roll(),
                ),
            ),
        )
    };
    let analytics_e = g::seq(
        g::send::<ANALYTICS, NOTIFY_REQUESTS, ModuleClaim<E_FILE, E>>(),
        g::seq(
            g::send::<ANALYTICS, E, CommitRequest<E>>(),
            g::seq(
                g::send::<ANALYTICS, E, ModuleLoan<E, ANALYTICS_RESOURCE>>(),
                g::seq(
                    g::send::<ANALYTICS_RESOURCE, E, ModuleRequest<ANALYTICS_RESOURCE, E>>(),
                    g::seq(
                        g::send::<E, ANALYTICS_RESOURCE, ModuleCompleted<ANALYTICS_RESOURCE>>(),
                        g::send::<E, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let analytics_f = g::seq(
        g::send::<ANALYTICS, SEARCH_REQUESTS, ModuleClaim<F_FILE, F>>(),
        g::seq(
            g::send::<ANALYTICS, F, CommitRequest<F>>(),
            g::seq(
                g::send::<ANALYTICS, F, ModuleLoan<F, ANALYTICS_RESOURCE>>(),
                g::seq(
                    g::send::<ANALYTICS_RESOURCE, F, ModuleRequest<ANALYTICS_RESOURCE, F>>(),
                    g::seq(
                        g::send::<F, ANALYTICS_RESOURCE, ModuleCompleted<ANALYTICS_RESOURCE>>(),
                        g::send::<F, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let analytics = g::route(
        g::seq(analytics_e.roll(), analytics_f.roll()),
        g::seq(analytics_f.roll(), analytics_e.roll()),
    );
    let api_b = g::seq(
        g::send::<API, B, ModuleLoan<B, API_RESOURCE>>(),
        g::seq(
            g::send::<API_RESOURCE, B, ModuleRequest<API_RESOURCE, B>>(),
            g::seq(
                g::send::<B, API_RESOURCE, ModuleCompleted<API_RESOURCE>>(),
                g::send::<B, API, Loan<API_RESOURCE>>(),
            ),
        ),
    );
    let api_c = g::seq(
        g::send::<API, C, ModuleLoan<C, API_RESOURCE>>(),
        g::seq(
            g::send::<API_RESOURCE, C, ModuleRequest<API_RESOURCE, C>>(),
            g::seq(
                g::send::<C, API_RESOURCE, ModuleCompleted<API_RESOURCE>>(),
                g::send::<C, API, Loan<API_RESOURCE>>(),
            ),
        ),
    );
    let api = g::route(
        g::seq(api_b.roll(), api_c.roll()),
        g::seq(api_c.roll(), api_b.roll()),
    );
    let auth_g = g::seq(
        g::send::<AUTH, SEARCH_REQUESTS, ModuleClaim<G_FILE, G>>(),
        g::seq(
            g::send::<AUTH, G, CommitRequest<G>>(),
            g::seq(
                g::send::<AUTH, G, ModuleLoan<G, AUTH_RESOURCE>>(),
                g::seq(
                    g::send::<AUTH_RESOURCE, G, ModuleRequest<AUTH_RESOURCE, G>>(),
                    g::seq(
                        g::send::<G, AUTH_RESOURCE, ModuleCompleted<AUTH_RESOURCE>>(),
                        g::send::<G, AUTH, Loan<AUTH_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let auth_a = g::seq(
        g::send::<AUTH, A, ModuleLoan<A, AUTH_RESOURCE>>(),
        g::seq(
            g::send::<AUTH_RESOURCE, A, ModuleRequest<AUTH_RESOURCE, A>>(),
            g::seq(
                g::send::<A, AUTH_RESOURCE, ModuleCompleted<AUTH_RESOURCE>>(),
                g::send::<A, AUTH, Loan<AUTH_RESOURCE>>(),
            ),
        ),
    );
    let auth = g::route(
        g::seq(auth_g.roll(), auth_a.roll()),
        g::seq(auth_a.roll(), auth_g.roll()),
    );
    let database_a = g::seq(
        g::send::<DATABASE, AUTH_REQUESTS, ModuleClaim<A_FILE, A>>(),
        g::seq(
            g::send::<DATABASE, A, CommitRequest<A>>(),
            g::seq(
                g::send::<DATABASE, A, ModuleLoan<A, DATABASE_RESOURCE>>(),
                g::seq(
                    g::send::<DATABASE_RESOURCE, A, ModuleRequest<DATABASE_RESOURCE, A>>(),
                    g::seq(
                        g::send::<A, DATABASE_RESOURCE, ModuleCompleted<DATABASE_RESOURCE>>(),
                        g::send::<A, DATABASE, Loan<DATABASE_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let database_b = g::seq(
        g::send::<DATABASE, API_REQUESTS, ModuleClaim<B_FILE, B>>(),
        g::seq(
            g::send::<DATABASE, B, CommitRequest<B>>(),
            g::seq(
                g::send::<DATABASE, B, ModuleLoan<B, DATABASE_RESOURCE>>(),
                g::seq(
                    g::send::<DATABASE_RESOURCE, B, ModuleRequest<DATABASE_RESOURCE, B>>(),
                    g::seq(
                        g::send::<B, DATABASE_RESOURCE, ModuleCompleted<DATABASE_RESOURCE>>(),
                        g::send::<B, DATABASE, Loan<DATABASE_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let database = g::route(
        g::seq(database_a.roll(), database_b.roll()),
        g::seq(database_b.roll(), database_a.roll()),
    );
    let notify_d = g::seq(
        g::send::<NOTIFY, D, ModuleLoan<D, NOTIFY_RESOURCE>>(),
        g::seq(
            g::send::<NOTIFY_RESOURCE, D, ModuleRequest<NOTIFY_RESOURCE, D>>(),
            g::seq(
                g::send::<D, NOTIFY_RESOURCE, ModuleCompleted<NOTIFY_RESOURCE>>(),
                g::send::<D, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
            ),
        ),
    );
    let notify_e = g::seq(
        g::send::<NOTIFY, E, ModuleLoan<E, NOTIFY_RESOURCE>>(),
        g::seq(
            g::send::<NOTIFY_RESOURCE, E, ModuleRequest<NOTIFY_RESOURCE, E>>(),
            g::seq(
                g::send::<E, NOTIFY_RESOURCE, ModuleCompleted<NOTIFY_RESOURCE>>(),
                g::send::<E, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
            ),
        ),
    );
    let notify = g::route(
        g::seq(notify_d.roll(), notify_e.roll()),
        g::seq(notify_e.roll(), notify_d.roll()),
    );
    let payment_c = g::seq(
        g::send::<PAYMENT, API_REQUESTS, ModuleClaim<C_FILE, C>>(),
        g::seq(
            g::send::<PAYMENT, C, CommitRequest<C>>(),
            g::seq(
                g::send::<PAYMENT, C, ModuleLoan<C, PAYMENT_RESOURCE>>(),
                g::seq(
                    g::send::<PAYMENT_RESOURCE, C, ModuleRequest<PAYMENT_RESOURCE, C>>(),
                    g::seq(
                        g::send::<C, PAYMENT_RESOURCE, ModuleCompleted<PAYMENT_RESOURCE>>(),
                        g::send::<C, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let payment_d = g::seq(
        g::send::<PAYMENT, NOTIFY_REQUESTS, ModuleClaim<D_FILE, D>>(),
        g::seq(
            g::send::<PAYMENT, D, CommitRequest<D>>(),
            g::seq(
                g::send::<PAYMENT, D, ModuleLoan<D, PAYMENT_RESOURCE>>(),
                g::seq(
                    g::send::<PAYMENT_RESOURCE, D, ModuleRequest<PAYMENT_RESOURCE, D>>(),
                    g::seq(
                        g::send::<D, PAYMENT_RESOURCE, ModuleCompleted<PAYMENT_RESOURCE>>(),
                        g::send::<D, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let payment = g::route(
        g::seq(payment_c.roll(), payment_d.roll()),
        g::seq(payment_d.roll(), payment_c.roll()),
    );
    let search_f = g::seq(
        g::send::<SEARCH, F, ModuleLoan<F, SEARCH_RESOURCE>>(),
        g::seq(
            g::send::<SEARCH_RESOURCE, F, ModuleRequest<SEARCH_RESOURCE, F>>(),
            g::seq(
                g::send::<F, SEARCH_RESOURCE, ModuleCompleted<SEARCH_RESOURCE>>(),
                g::send::<F, SEARCH, Loan<SEARCH_RESOURCE>>(),
            ),
        ),
    );
    let search_g = g::seq(
        g::send::<SEARCH, G, ModuleLoan<G, SEARCH_RESOURCE>>(),
        g::seq(
            g::send::<SEARCH_RESOURCE, G, ModuleRequest<SEARCH_RESOURCE, G>>(),
            g::seq(
                g::send::<G, SEARCH_RESOURCE, ModuleCompleted<SEARCH_RESOURCE>>(),
                g::send::<G, SEARCH, Loan<SEARCH_RESOURCE>>(),
            ),
        ),
    );
    let search = g::route(
        g::seq(search_f.roll(), search_g.roll()),
        g::seq(search_g.roll(), search_f.roll()),
    );
    let modules = g::par(
        g::par(g::par(database, payment), analytics),
        g::par(g::par(auth, api), g::par(notify, search)),
    );
    let developers = g::par(g::par(g::par(a, b), g::par(c, d)), g::par(g::par(e, f), g));
    let task = g::par(modules, developers);
    let protocol = g::seq(
        g::par(
            g::par(
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
                                g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
                            ),
                            g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
                        ),
                        g::send::<D_DATA, D_VM, Loan<D_FILE>>(),
                    ),
                    g::send::<E_DATA, E_VM, Loan<E_FILE>>(),
                ),
                g::send::<F_DATA, F_VM, Loan<F_FILE>>(),
            ),
            g::send::<G_DATA, G_VM, Loan<G_FILE>>(),
        ),
        g::par(
            task,
            g::par(
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::par(
                                    g::par(
                                        g::seq(
                                            crate::shell::global::private_io::<A_VM, A_IO>(),
                                            g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                                        ),
                                        g::seq(
                                            crate::shell::global::private_io::<B_VM, B_IO>(),
                                            g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                                        ),
                                    ),
                                    g::seq(
                                        crate::shell::global::private_io::<C_VM, C_IO>(),
                                        g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
                                    ),
                                ),
                                g::seq(
                                    crate::shell::global::private_io::<D_VM, D_IO>(),
                                    g::send::<D_VM, D_IO, crate::shell::global::Exit>(),
                                ),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<E_VM, E_IO>(),
                                g::send::<E_VM, E_IO, crate::shell::global::Exit>(),
                            ),
                        ),
                        g::seq(
                            crate::shell::global::private_io::<F_VM, F_IO>(),
                            g::send::<F_VM, F_IO, crate::shell::global::Exit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<G_VM, G_IO>(),
                        g::send::<G_VM, G_IO, crate::shell::global::Exit>(),
                    ),
                ),
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::par(
                                    g::par(
                                        g::seq(
                                            crate::shell::global::file_metadata::<A_FILE, A_DATA>()
                                                .roll(),
                                            g::seq(
                                                g::send::<A_FILE, A_DATA, FileExit>(),
                                                g::send::<
                                                    A_FILE,
                                                    AUTH_REQUESTS,
                                                    ModuleFileClosed<AUTH>,
                                                >(),
                                            ),
                                        ),
                                        g::seq(
                                            crate::shell::global::file_metadata::<B_FILE, B_DATA>()
                                                .roll(),
                                            g::seq(
                                                g::send::<B_FILE, B_DATA, FileExit>(),
                                                g::send::<
                                                    B_FILE,
                                                    API_REQUESTS,
                                                    ModuleFileClosed<API>,
                                                >(),
                                            ),
                                        ),
                                    ),
                                    g::seq(
                                        crate::shell::global::file_metadata::<C_FILE, C_DATA>()
                                            .roll(),
                                        g::seq(
                                            g::send::<C_FILE, C_DATA, FileExit>(),
                                            g::send::<C_FILE, API_REQUESTS, ModuleFileClosed<API>>(
                                            ),
                                        ),
                                    ),
                                ),
                                g::seq(
                                    crate::shell::global::file_metadata::<D_FILE, D_DATA>().roll(),
                                    g::seq(
                                        g::send::<D_FILE, D_DATA, FileExit>(),
                                        g::send::<D_FILE, NOTIFY_REQUESTS, ModuleFileClosed<NOTIFY>>(
                                        ),
                                    ),
                                ),
                            ),
                            g::seq(
                                crate::shell::global::file_metadata::<E_FILE, E_DATA>().roll(),
                                g::seq(
                                    g::send::<E_FILE, E_DATA, FileExit>(),
                                    g::send::<E_FILE, NOTIFY_REQUESTS, ModuleFileClosed<NOTIFY>>(),
                                ),
                            ),
                        ),
                        g::seq(
                            crate::shell::global::file_metadata::<F_FILE, F_DATA>().roll(),
                            g::seq(
                                g::send::<F_FILE, F_DATA, FileExit>(),
                                g::send::<F_FILE, SEARCH_REQUESTS, ModuleFileClosed<SEARCH>>(),
                            ),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<G_FILE, G_DATA>().roll(),
                        g::seq(
                            g::send::<G_FILE, G_DATA, FileExit>(),
                            g::send::<G_FILE, SEARCH_REQUESTS, ModuleFileClosed<SEARCH>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    Programs1H {
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        d: project(&protocol),
        d_file: project(&protocol),
        d_vm: project(&protocol),
        d_io: project(&protocol),
        d_data: project(&protocol),
        e: project(&protocol),
        e_file: project(&protocol),
        e_vm: project(&protocol),
        e_io: project(&protocol),
        e_data: project(&protocol),
        f: project(&protocol),
        f_file: project(&protocol),
        f_vm: project(&protocol),
        f_io: project(&protocol),
        f_data: project(&protocol),
        g: project(&protocol),
        g_file: project(&protocol),
        g_vm: project(&protocol),
        g_io: project(&protocol),
        g_data: project(&protocol),
        auth_requests: project(&protocol),
        auth: project(&protocol),
        auth_resource: project(&protocol),
        database_requests: project(&protocol),
        database: project(&protocol),
        database_resource: project(&protocol),
        api_requests: project(&protocol),
        api: project(&protocol),
        api_resource: project(&protocol),
        payment_requests: project(&protocol),
        payment: project(&protocol),
        payment_resource: project(&protocol),
        notify_requests: project(&protocol),
        notify: project(&protocol),
        notify_resource: project(&protocol),
        analytics_requests: project(&protocol),
        analytics: project(&protocol),
        analytics_resource: project(&protocol),
        search_requests: project(&protocol),
        search: project(&protocol),
        search_resource: project(&protocol),
    }
}
impl Programs1H {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles1h::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1H","session":SESSION,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resolver":CHECK_A,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resolver":CHECK_B,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resolver":CHECK_C,"resources":[API_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resolver":CHECK_D,"resources":[NOTIFY_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resolver":CHECK_E,"resources":[ANALYTICS_RESOURCE,NOTIFY_RESOURCE]},"DEVELOPER_F":{"role":F,"file":F_FILE,"vm":F_VM,"io":F_IO,"data":F_DATA,"resolver":CHECK_F,"resources":[ANALYTICS_RESOURCE,SEARCH_RESOURCE]},"DEVELOPER_G":{"role":G,"file":G_FILE,"vm":G_VM,"io":G_IO,"data":G_DATA,"resolver":CHECK_G,"resources":[AUTH_RESOURCE,SEARCH_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":AUTH,"requests":AUTH_REQUESTS},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":DATABASE,"requests":DATABASE_REQUESTS},"API_MODULE":{"role":API_RESOURCE,"service":API,"requests":API_REQUESTS},"PAYMENT_MODULE":{"role":PAYMENT_RESOURCE,"service":PAYMENT,"requests":PAYMENT_REQUESTS},"NOTIFY_MODULE":{"role":NOTIFY_RESOURCE,"service":NOTIFY,"requests":NOTIFY_REQUESTS},"ANALYTICS_MODULE":{"role":ANALYTICS_RESOURCE,"service":ANALYTICS,"requests":ANALYTICS_REQUESTS},"SEARCH_MODULE":{"role":SEARCH_RESOURCE,"service":SEARCH,"requests":SEARCH_REQUESTS}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.f),rows(&self.f_file),rows(&self.f_vm),rows(&self.f_io),rows(&self.f_data),rows(&self.g),rows(&self.g_file),rows(&self.g_vm),rows(&self.g_io),rows(&self.g_data),rows(&self.auth_requests),rows(&self.auth),rows(&self.auth_resource),rows(&self.database_requests),rows(&self.database),rows(&self.database_resource),rows(&self.api_requests),rows(&self.api),rows(&self.api_resource),rows(&self.payment_requests),rows(&self.payment),rows(&self.payment_resource),rows(&self.notify_requests),rows(&self.notify),rows(&self.notify_resource),rows(&self.analytics_requests),rows(&self.analytics),rows(&self.analytics_resource),rows(&self.search_requests),rows(&self.search),rows(&self.search_resource)]})
    }
}
