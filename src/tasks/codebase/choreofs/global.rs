//! Direct file requests, actual module Endpoints and original effect results.
use super::wire::{Arguments, ModuleArguments};
use crate::shell::global::FileExit;
pub use crate::shell::global::Loan;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const SESSION: u32 = 20;
pub type DesignRequest<const ROLE: u8> = g::Msg<1, Arguments<ROLE, 0>>;
pub type ImplementRequest<const ROLE: u8> = g::Msg<2, Arguments<ROLE, 1>>;
pub type CommitRequest<const ROLE: u8> = g::Msg<3, Arguments<ROLE, 2>>;
pub type TestRequest<const ROLE: u8> = g::Msg<4, Arguments<ROLE, 3>>;
pub type DesignCompleted<'a> = g::Msg<5, &'a [u8]>;
pub type ImplementCompleted<'a> = g::Msg<6, &'a [u8]>;
pub type CommitCompleted<'a> = g::Msg<7, &'a [u8]>;
pub type TestCompleted<'a> = g::Msg<8, &'a [u8]>;
pub type ModuleRequest<const RESOURCE: u8> = g::Msg<RESOURCE, ModuleArguments>;
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

pub type ModuleTestResult<'a, const LABEL: u8> = g::Msg<LABEL, &'a [u8]>;
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
    (request, reply): (impl Projectable, impl Projectable),
    (first_loan, second_loan): (impl Projectable, impl Projectable),
    (first_return, second_return): (impl Projectable, impl Projectable),
) -> impl Projectable {
    g::seq(
        request,
        g::seq(
            g::par(first_loan, second_loan),
            g::seq(
                g::par(
                    g::send::<FIRST, OWNER, ModuleRequest<FIRST>>(),
                    g::send::<SECOND, OWNER, ModuleRequest<SECOND>>(),
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
    pub const AUTH: u8 = 32;
    pub const AUTH_RESOURCE: u8 = 64;
    pub const DATABASE: u8 = 33;
    pub const DATABASE_RESOURCE: u8 = 65;
    pub const API: u8 = 34;
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
    pub auth: RoleProgram<{ roles1e::AUTH }>,
    pub auth_resource: RoleProgram<{ roles1e::AUTH_RESOURCE }>,
    pub database: RoleProgram<{ roles1e::DATABASE }>,
    pub database_resource: RoleProgram<{ roles1e::DATABASE_RESOURCE }>,
    pub api: RoleProgram<{ roles1e::API }>,
    pub api_resource: RoleProgram<{ roles1e::API_RESOURCE }>,
}
pub fn project_1e() -> Programs1E {
    use roles1e::*;
    let commit_a = commit::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<A_FILE, A, CommitRequest<A>>(),
            g::send::<A, A_FILE, CommitCompleted>(),
        ),
        (
            g::send::<AUTH, A, Loan<AUTH_RESOURCE>>(),
            g::send::<DATABASE, A, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<A, AUTH, Loan<AUTH_RESOURCE>>(),
            g::send::<A, DATABASE, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_b = commit::<B, API_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<B_FILE, B, CommitRequest<B>>(),
            g::send::<B, B_FILE, CommitCompleted>(),
        ),
        (
            g::send::<API, B, Loan<API_RESOURCE>>(),
            g::send::<DATABASE, B, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<B, API, Loan<API_RESOURCE>>(),
            g::send::<B, DATABASE, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_c = commit::<C, API_RESOURCE, AUTH_RESOURCE>(
        (
            g::send::<C_FILE, C, CommitRequest<C>>(),
            g::send::<C, C_FILE, CommitCompleted>(),
        ),
        (
            g::send::<API, C, Loan<API_RESOURCE>>(),
            g::send::<AUTH, C, Loan<AUTH_RESOURCE>>(),
        ),
        (
            g::send::<C, API, Loan<API_RESOURCE>>(),
            g::send::<C, AUTH, Loan<AUTH_RESOURCE>>(),
        ),
    );
    let task = g::seq(
        g::par(
            g::par(preparation::<A, A_FILE>(), preparation::<B, B_FILE>()),
            preparation::<C, C_FILE>(),
        ),
        g::seq(
            commit_a,
            g::par(
                test::<A, A_FILE>(),
                g::seq(
                    commit_b,
                    g::par(test::<B, B_FILE>(), g::seq(commit_c, test::<C, C_FILE>())),
                ),
            ),
        ),
    );
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
        auth: project(&protocol),
        auth_resource: project(&protocol),
        database: project(&protocol),
        database_resource: project(&protocol),
        api: project(&protocol),
        api_resource: project(&protocol),
    }
}
impl Programs1E {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles1e::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1E","session":SESSION,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[API_RESOURCE,AUTH_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":AUTH},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":DATABASE},"API_MODULE":{"role":API_RESOURCE,"service":API}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.auth),rows(&self.auth_resource),rows(&self.database),rows(&self.database_resource),rows(&self.api),rows(&self.api_resource)]})
    }
}
pub mod roles1m {
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
    let commit_a = commit::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<A_FILE, A, CommitRequest<A>>(),
            g::send::<A, A_FILE, CommitCompleted>(),
        ),
        (
            g::send::<AUTH, A, Loan<AUTH_RESOURCE>>(),
            g::send::<DATABASE, A, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<A, AUTH, Loan<AUTH_RESOURCE>>(),
            g::send::<A, DATABASE, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_b = commit::<B, API_RESOURCE, DATABASE_RESOURCE>(
        (
            g::send::<B_FILE, B, CommitRequest<B>>(),
            g::send::<B, B_FILE, CommitCompleted>(),
        ),
        (
            g::send::<API, B, Loan<API_RESOURCE>>(),
            g::send::<DATABASE, B, Loan<DATABASE_RESOURCE>>(),
        ),
        (
            g::send::<B, API, Loan<API_RESOURCE>>(),
            g::send::<B, DATABASE, Loan<DATABASE_RESOURCE>>(),
        ),
    );
    let commit_c = commit::<C, API_RESOURCE, PAYMENT_RESOURCE>(
        (
            g::send::<C_FILE, C, CommitRequest<C>>(),
            g::send::<C, C_FILE, CommitCompleted>(),
        ),
        (
            g::send::<API, C, Loan<API_RESOURCE>>(),
            g::send::<PAYMENT, C, Loan<PAYMENT_RESOURCE>>(),
        ),
        (
            g::send::<C, API, Loan<API_RESOURCE>>(),
            g::send::<C, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
        ),
    );
    let commit_d = commit::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
        (
            g::send::<D_FILE, D, CommitRequest<D>>(),
            g::send::<D, D_FILE, CommitCompleted>(),
        ),
        (
            g::send::<NOTIFY, D, Loan<NOTIFY_RESOURCE>>(),
            g::send::<PAYMENT, D, Loan<PAYMENT_RESOURCE>>(),
        ),
        (
            g::send::<D, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
            g::send::<D, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
        ),
    );
    let commit_e = commit::<E, AUTH_RESOURCE, NOTIFY_RESOURCE>(
        (
            g::send::<E_FILE, E, CommitRequest<E>>(),
            g::send::<E, E_FILE, CommitCompleted>(),
        ),
        (
            g::send::<AUTH, E, Loan<AUTH_RESOURCE>>(),
            g::send::<NOTIFY, E, Loan<NOTIFY_RESOURCE>>(),
        ),
        (
            g::send::<E, AUTH, Loan<AUTH_RESOURCE>>(),
            g::send::<E, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
        ),
    );
    let task = g::seq(
        g::par(
            g::par(
                g::par(
                    g::par(preparation::<A, A_FILE>(), preparation::<B, B_FILE>()),
                    preparation::<C, C_FILE>(),
                ),
                preparation::<D, D_FILE>(),
            ),
            preparation::<E, E_FILE>(),
        ),
        g::seq(
            g::par(commit_a, commit_c),
            g::par(
                g::par(test::<A, A_FILE>(), test::<C, C_FILE>()),
                g::seq(
                    g::par(commit_b, commit_d),
                    g::par(
                        g::par(test::<B, B_FILE>(), test::<D, D_FILE>()),
                        g::seq(commit_e, test::<E, E_FILE>()),
                    ),
                ),
            ),
        ),
    );
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
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1M","session":SESSION,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[API_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resources":[NOTIFY_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resources":[AUTH_RESOURCE,NOTIFY_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":AUTH},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":DATABASE},"API_MODULE":{"role":API_RESOURCE,"service":API},"PAYMENT_MODULE":{"role":PAYMENT_RESOURCE,"service":PAYMENT},"NOTIFY_MODULE":{"role":NOTIFY_RESOURCE,"service":NOTIFY}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.auth),rows(&self.auth_resource),rows(&self.database),rows(&self.database_resource),rows(&self.api),rows(&self.api_resource),rows(&self.payment),rows(&self.payment_resource),rows(&self.notify),rows(&self.notify_resource)]})
    }
}
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
    pub const ANALYTICS: u8 = 37;
    pub const ANALYTICS_RESOURCE: u8 = 69;
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
    let initial_a = g::seq(
        commit::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
            (
                g::send::<A_FILE, A, CommitRequest<A>>(),
                g::send::<A, A_FILE, CommitCompleted>(),
            ),
            (
                g::send::<AUTH, A, Loan<AUTH_RESOURCE>>(),
                g::send::<DATABASE, A, Loan<DATABASE_RESOURCE>>(),
            ),
            (
                g::send::<A, AUTH, Loan<AUTH_RESOURCE>>(),
                g::send::<A, DATABASE, Loan<DATABASE_RESOURCE>>(),
            ),
        ),
        g::send::<A_FILE, A, TestRequest<A>>(),
    );
    let result_a = g::route(
        g::par(
            g::par(
                g::send::<A, A_FILE, TestCompleted>(),
                g::send::<A, AUTH, ModuleTestResult<DATABASE_RESOURCE>>(),
            ),
            g::send::<A, DATABASE, ModuleTestResult<AUTH_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<A, A_FILE, TestCompleted>(),
                    g::send::<A, AUTH, ModuleTestResult<DATABASE_RESOURCE>>(),
                ),
                g::send::<A, DATABASE, ModuleTestResult<AUTH_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<A_FILE, A, ImplementRequest<A>>(),
                        g::send::<A, A_FILE, ImplementCompleted>(),
                    ),
                    commit::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
                        (
                            g::send::<A_FILE, A, CommitRequest<A>>(),
                            g::send::<A, A_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<AUTH, A, Loan<AUTH_RESOURCE>>(),
                            g::send::<DATABASE, A, Loan<DATABASE_RESOURCE>>(),
                        ),
                        (
                            g::send::<A, AUTH, Loan<AUTH_RESOURCE>>(),
                            g::send::<A, DATABASE, Loan<DATABASE_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<A_FILE, A, TestRequest<A>>(),
            ),
        ),
    )
    .resolve::<CHECK_A>()
    .roll();
    let initial_b = g::seq(
        commit::<B, API_RESOURCE, DATABASE_RESOURCE>(
            (
                g::send::<B_FILE, B, CommitRequest<B>>(),
                g::send::<B, B_FILE, CommitCompleted>(),
            ),
            (
                g::send::<API, B, Loan<API_RESOURCE>>(),
                g::send::<DATABASE, B, Loan<DATABASE_RESOURCE>>(),
            ),
            (
                g::send::<B, API, Loan<API_RESOURCE>>(),
                g::send::<B, DATABASE, Loan<DATABASE_RESOURCE>>(),
            ),
        ),
        g::send::<B_FILE, B, TestRequest<B>>(),
    );
    let result_b = g::route(
        g::par(
            g::par(
                g::send::<B, B_FILE, TestCompleted>(),
                g::send::<B, API, ModuleTestResult<DATABASE_RESOURCE>>(),
            ),
            g::send::<B, DATABASE, ModuleTestResult<API_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<B, B_FILE, TestCompleted>(),
                    g::send::<B, API, ModuleTestResult<DATABASE_RESOURCE>>(),
                ),
                g::send::<B, DATABASE, ModuleTestResult<API_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<B_FILE, B, ImplementRequest<B>>(),
                        g::send::<B, B_FILE, ImplementCompleted>(),
                    ),
                    commit::<B, API_RESOURCE, DATABASE_RESOURCE>(
                        (
                            g::send::<B_FILE, B, CommitRequest<B>>(),
                            g::send::<B, B_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<API, B, Loan<API_RESOURCE>>(),
                            g::send::<DATABASE, B, Loan<DATABASE_RESOURCE>>(),
                        ),
                        (
                            g::send::<B, API, Loan<API_RESOURCE>>(),
                            g::send::<B, DATABASE, Loan<DATABASE_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<B_FILE, B, TestRequest<B>>(),
            ),
        ),
    )
    .resolve::<CHECK_B>()
    .roll();
    let initial_c = g::seq(
        commit::<C, API_RESOURCE, PAYMENT_RESOURCE>(
            (
                g::send::<C_FILE, C, CommitRequest<C>>(),
                g::send::<C, C_FILE, CommitCompleted>(),
            ),
            (
                g::send::<API, C, Loan<API_RESOURCE>>(),
                g::send::<PAYMENT, C, Loan<PAYMENT_RESOURCE>>(),
            ),
            (
                g::send::<C, API, Loan<API_RESOURCE>>(),
                g::send::<C, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
            ),
        ),
        g::send::<C_FILE, C, TestRequest<C>>(),
    );
    let result_c = g::route(
        g::par(
            g::par(
                g::send::<C, C_FILE, TestCompleted>(),
                g::send::<C, API, ModuleTestResult<PAYMENT_RESOURCE>>(),
            ),
            g::send::<C, PAYMENT, ModuleTestResult<API_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<C, C_FILE, TestCompleted>(),
                    g::send::<C, API, ModuleTestResult<PAYMENT_RESOURCE>>(),
                ),
                g::send::<C, PAYMENT, ModuleTestResult<API_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<C_FILE, C, ImplementRequest<C>>(),
                        g::send::<C, C_FILE, ImplementCompleted>(),
                    ),
                    commit::<C, API_RESOURCE, PAYMENT_RESOURCE>(
                        (
                            g::send::<C_FILE, C, CommitRequest<C>>(),
                            g::send::<C, C_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<API, C, Loan<API_RESOURCE>>(),
                            g::send::<PAYMENT, C, Loan<PAYMENT_RESOURCE>>(),
                        ),
                        (
                            g::send::<C, API, Loan<API_RESOURCE>>(),
                            g::send::<C, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<C_FILE, C, TestRequest<C>>(),
            ),
        ),
    )
    .resolve::<CHECK_C>()
    .roll();
    let initial_d = g::seq(
        commit::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
            (
                g::send::<D_FILE, D, CommitRequest<D>>(),
                g::send::<D, D_FILE, CommitCompleted>(),
            ),
            (
                g::send::<NOTIFY, D, Loan<NOTIFY_RESOURCE>>(),
                g::send::<PAYMENT, D, Loan<PAYMENT_RESOURCE>>(),
            ),
            (
                g::send::<D, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
                g::send::<D, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
            ),
        ),
        g::send::<D_FILE, D, TestRequest<D>>(),
    );
    let result_d = g::route(
        g::par(
            g::par(
                g::send::<D, D_FILE, TestCompleted>(),
                g::send::<D, NOTIFY, ModuleTestResult<PAYMENT_RESOURCE>>(),
            ),
            g::send::<D, PAYMENT, ModuleTestResult<NOTIFY_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<D, D_FILE, TestCompleted>(),
                    g::send::<D, NOTIFY, ModuleTestResult<PAYMENT_RESOURCE>>(),
                ),
                g::send::<D, PAYMENT, ModuleTestResult<NOTIFY_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<D_FILE, D, ImplementRequest<D>>(),
                        g::send::<D, D_FILE, ImplementCompleted>(),
                    ),
                    commit::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
                        (
                            g::send::<D_FILE, D, CommitRequest<D>>(),
                            g::send::<D, D_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<NOTIFY, D, Loan<NOTIFY_RESOURCE>>(),
                            g::send::<PAYMENT, D, Loan<PAYMENT_RESOURCE>>(),
                        ),
                        (
                            g::send::<D, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
                            g::send::<D, PAYMENT, Loan<PAYMENT_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<D_FILE, D, TestRequest<D>>(),
            ),
        ),
    )
    .resolve::<CHECK_D>()
    .roll();
    let initial_e = g::seq(
        commit::<E, ANALYTICS_RESOURCE, NOTIFY_RESOURCE>(
            (
                g::send::<E_FILE, E, CommitRequest<E>>(),
                g::send::<E, E_FILE, CommitCompleted>(),
            ),
            (
                g::send::<ANALYTICS, E, Loan<ANALYTICS_RESOURCE>>(),
                g::send::<NOTIFY, E, Loan<NOTIFY_RESOURCE>>(),
            ),
            (
                g::send::<E, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                g::send::<E, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
            ),
        ),
        g::send::<E_FILE, E, TestRequest<E>>(),
    );
    let result_e = g::route(
        g::par(
            g::par(
                g::send::<E, E_FILE, TestCompleted>(),
                g::send::<E, ANALYTICS, ModuleTestResult<NOTIFY_RESOURCE>>(),
            ),
            g::send::<E, NOTIFY, ModuleTestResult<ANALYTICS_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<E, E_FILE, TestCompleted>(),
                    g::send::<E, ANALYTICS, ModuleTestResult<NOTIFY_RESOURCE>>(),
                ),
                g::send::<E, NOTIFY, ModuleTestResult<ANALYTICS_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<E_FILE, E, ImplementRequest<E>>(),
                        g::send::<E, E_FILE, ImplementCompleted>(),
                    ),
                    commit::<E, ANALYTICS_RESOURCE, NOTIFY_RESOURCE>(
                        (
                            g::send::<E_FILE, E, CommitRequest<E>>(),
                            g::send::<E, E_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<ANALYTICS, E, Loan<ANALYTICS_RESOURCE>>(),
                            g::send::<NOTIFY, E, Loan<NOTIFY_RESOURCE>>(),
                        ),
                        (
                            g::send::<E, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                            g::send::<E, NOTIFY, Loan<NOTIFY_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<E_FILE, E, TestRequest<E>>(),
            ),
        ),
    )
    .resolve::<CHECK_E>()
    .roll();
    let initial_f = g::seq(
        commit::<F, ANALYTICS_RESOURCE, SEARCH_RESOURCE>(
            (
                g::send::<F_FILE, F, CommitRequest<F>>(),
                g::send::<F, F_FILE, CommitCompleted>(),
            ),
            (
                g::send::<ANALYTICS, F, Loan<ANALYTICS_RESOURCE>>(),
                g::send::<SEARCH, F, Loan<SEARCH_RESOURCE>>(),
            ),
            (
                g::send::<F, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                g::send::<F, SEARCH, Loan<SEARCH_RESOURCE>>(),
            ),
        ),
        g::send::<F_FILE, F, TestRequest<F>>(),
    );
    let result_f = g::route(
        g::par(
            g::par(
                g::send::<F, F_FILE, TestCompleted>(),
                g::send::<F, ANALYTICS, ModuleTestResult<SEARCH_RESOURCE>>(),
            ),
            g::send::<F, SEARCH, ModuleTestResult<ANALYTICS_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<F, F_FILE, TestCompleted>(),
                    g::send::<F, ANALYTICS, ModuleTestResult<SEARCH_RESOURCE>>(),
                ),
                g::send::<F, SEARCH, ModuleTestResult<ANALYTICS_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<F_FILE, F, ImplementRequest<F>>(),
                        g::send::<F, F_FILE, ImplementCompleted>(),
                    ),
                    commit::<F, ANALYTICS_RESOURCE, SEARCH_RESOURCE>(
                        (
                            g::send::<F_FILE, F, CommitRequest<F>>(),
                            g::send::<F, F_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<ANALYTICS, F, Loan<ANALYTICS_RESOURCE>>(),
                            g::send::<SEARCH, F, Loan<SEARCH_RESOURCE>>(),
                        ),
                        (
                            g::send::<F, ANALYTICS, Loan<ANALYTICS_RESOURCE>>(),
                            g::send::<F, SEARCH, Loan<SEARCH_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<F_FILE, F, TestRequest<F>>(),
            ),
        ),
    )
    .resolve::<CHECK_F>()
    .roll();
    let initial_g = g::seq(
        commit::<G, AUTH_RESOURCE, SEARCH_RESOURCE>(
            (
                g::send::<G_FILE, G, CommitRequest<G>>(),
                g::send::<G, G_FILE, CommitCompleted>(),
            ),
            (
                g::send::<AUTH, G, Loan<AUTH_RESOURCE>>(),
                g::send::<SEARCH, G, Loan<SEARCH_RESOURCE>>(),
            ),
            (
                g::send::<G, AUTH, Loan<AUTH_RESOURCE>>(),
                g::send::<G, SEARCH, Loan<SEARCH_RESOURCE>>(),
            ),
        ),
        g::send::<G_FILE, G, TestRequest<G>>(),
    );
    let result_g = g::route(
        g::par(
            g::par(
                g::send::<G, G_FILE, TestCompleted>(),
                g::send::<G, AUTH, ModuleTestResult<SEARCH_RESOURCE>>(),
            ),
            g::send::<G, SEARCH, ModuleTestResult<AUTH_RESOURCE>>(),
        ),
        g::seq(
            g::par(
                g::par(
                    g::send::<G, G_FILE, TestCompleted>(),
                    g::send::<G, AUTH, ModuleTestResult<SEARCH_RESOURCE>>(),
                ),
                g::send::<G, SEARCH, ModuleTestResult<AUTH_RESOURCE>>(),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<G_FILE, G, ImplementRequest<G>>(),
                        g::send::<G, G_FILE, ImplementCompleted>(),
                    ),
                    commit::<G, AUTH_RESOURCE, SEARCH_RESOURCE>(
                        (
                            g::send::<G_FILE, G, CommitRequest<G>>(),
                            g::send::<G, G_FILE, CommitCompleted>(),
                        ),
                        (
                            g::send::<AUTH, G, Loan<AUTH_RESOURCE>>(),
                            g::send::<SEARCH, G, Loan<SEARCH_RESOURCE>>(),
                        ),
                        (
                            g::send::<G, AUTH, Loan<AUTH_RESOURCE>>(),
                            g::send::<G, SEARCH, Loan<SEARCH_RESOURCE>>(),
                        ),
                    ),
                ),
                g::send::<G_FILE, G, TestRequest<G>>(),
            ),
        ),
    )
    .resolve::<CHECK_G>()
    .roll();
    let task = g::seq(
        g::seq(
            g::par(
                g::par(
                    g::par(
                        g::par(
                            g::par(
                                g::par(preparation::<A, A_FILE>(), preparation::<B, B_FILE>()),
                                preparation::<C, C_FILE>(),
                            ),
                            preparation::<D, D_FILE>(),
                        ),
                        preparation::<E, E_FILE>(),
                    ),
                    preparation::<F, F_FILE>(),
                ),
                preparation::<G, G_FILE>(),
            ),
            g::seq(
                g::seq(
                    g::par(g::par(initial_a, initial_c), initial_e),
                    g::par(g::par(initial_b, initial_d), initial_f),
                ),
                initial_g,
            ),
        ),
        g::seq(
            g::seq(
                g::par(g::par(result_a, result_c), result_e),
                g::par(g::par(result_b, result_d), result_f),
            ),
            result_g,
        ),
    );
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
                                            g::send::<A_FILE, A_DATA, FileExit>(),
                                        ),
                                        g::seq(
                                            crate::shell::global::file_metadata::<B_FILE, B_DATA>()
                                                .roll(),
                                            g::send::<B_FILE, B_DATA, FileExit>(),
                                        ),
                                    ),
                                    g::seq(
                                        crate::shell::global::file_metadata::<C_FILE, C_DATA>()
                                            .roll(),
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
                        g::seq(
                            crate::shell::global::file_metadata::<F_FILE, F_DATA>().roll(),
                            g::send::<F_FILE, F_DATA, FileExit>(),
                        ),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<G_FILE, G_DATA>().roll(),
                        g::send::<G_FILE, G_DATA, FileExit>(),
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
impl Programs1H {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles1h::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"1H","session":SESSION,"participants":{"DEVELOPER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resolver":CHECK_A,"resources":[AUTH_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resolver":CHECK_B,"resources":[API_RESOURCE,DATABASE_RESOURCE]},"DEVELOPER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resolver":CHECK_C,"resources":[API_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resolver":CHECK_D,"resources":[NOTIFY_RESOURCE,PAYMENT_RESOURCE]},"DEVELOPER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resolver":CHECK_E,"resources":[ANALYTICS_RESOURCE,NOTIFY_RESOURCE]},"DEVELOPER_F":{"role":F,"file":F_FILE,"vm":F_VM,"io":F_IO,"data":F_DATA,"resolver":CHECK_F,"resources":[ANALYTICS_RESOURCE,SEARCH_RESOURCE]},"DEVELOPER_G":{"role":G,"file":G_FILE,"vm":G_VM,"io":G_IO,"data":G_DATA,"resolver":CHECK_G,"resources":[AUTH_RESOURCE,SEARCH_RESOURCE]}},"resources":{"AUTH_MODULE":{"role":AUTH_RESOURCE,"service":AUTH},"DATABASE_MODULE":{"role":DATABASE_RESOURCE,"service":DATABASE},"API_MODULE":{"role":API_RESOURCE,"service":API},"PAYMENT_MODULE":{"role":PAYMENT_RESOURCE,"service":PAYMENT},"NOTIFY_MODULE":{"role":NOTIFY_RESOURCE,"service":NOTIFY},"ANALYTICS_MODULE":{"role":ANALYTICS_RESOURCE,"service":ANALYTICS},"SEARCH_MODULE":{"role":SEARCH_RESOURCE,"service":SEARCH}},"labels":{"design":1,"implement":2,"commit":3,"test":4,"designed":5,"implemented":6,"committed":7,"tested":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.f),rows(&self.f_file),rows(&self.f_vm),rows(&self.f_io),rows(&self.f_data),rows(&self.g),rows(&self.g_file),rows(&self.g_vm),rows(&self.g_io),rows(&self.g_data),rows(&self.auth),rows(&self.auth_resource),rows(&self.database),rows(&self.database_resource),rows(&self.api),rows(&self.api_resource),rows(&self.payment),rows(&self.payment_resource),rows(&self.notify),rows(&self.notify_resource),rows(&self.analytics),rows(&self.analytics_resource),rows(&self.search),rows(&self.search_resource)]})
    }
}
