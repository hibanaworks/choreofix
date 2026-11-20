//! Endpoint-bound operation files, actual resources and result-file delivery.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const ADMIN: u8 = 0;
pub const BACKEND: u8 = 1;
pub const TESTER: u8 = 2;
pub const DATABASE: u8 = 3;
pub const TEST_ENV: u8 = 4;
pub const ADMIN_VM: u8 = 5;
pub const BACKEND_VM: u8 = 6;
pub const TESTER_VM: u8 = 7;
pub const ADMIN_IO: u8 = 8;
pub const BACKEND_IO: u8 = 9;
pub const TESTER_IO: u8 = 10;
pub const ADMIN_OUTCOME: u8 = 11;
pub const BACKEND_OUTCOME: u8 = 12;
pub const ADMIN_OUTCOME_FILE: u8 = 13;
pub const BACKEND_OUTCOME_FILE: u8 = 14;
pub const DATABASE_RESOURCE: u8 = 15;
pub const TEST_RESOURCE: u8 = 16;
pub const ADMIN_FILE: u8 = 17;
pub const BACKEND_FILE: u8 = 18;
pub const TESTER_FILE: u8 = 19;
pub const ADMIN_CLOSE: u8 = 20;
pub const BACKEND_CLOSE: u8 = 21;
pub const TESTER_CLOSE: u8 = 22;
pub const ADMIN_OUTCOME_READ: u8 = 23;
pub const BACKEND_OUTCOME_READ: u8 = 24;
pub const TEST_RESULT: u16 = 683;
pub const SERVICE_FD: u8 = 4;
pub const OUTCOME_FD: u8 = 5;
pub const OUTCOME_PATH: &[u8] = b"service/outcome";

const OPERATION_BINDING: protocol::FdBinding = protocol::FdBinding {
    read: Some(protocol::FdReadRow::Base),
    write: Some(protocol::FdWriteRow::Object),
    readdir: None,
};
pub const ADMIN_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/migrate_schema",
    ObjectId(SERVICE_FD as u32),
    FdSpec::new(SERVICE_FD, 66),
    b"",
    OPERATION_BINDING,
);
pub const BACKEND_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/implement_backend",
    ObjectId(SERVICE_FD as u32),
    FdSpec::new(SERVICE_FD, 66),
    b"",
    OPERATION_BINDING,
);
pub const TESTER_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/run_integration_tests",
    ObjectId(SERVICE_FD as u32),
    FdSpec::new(SERVICE_FD, 66),
    b"",
    OPERATION_BINDING,
);
pub const OUTCOME_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    OUTCOME_PATH,
    ObjectId(5),
    FdSpec::new(5, 2),
    b"",
    protocol::FdBinding::read(protocol::FdReadRow::Base),
);

pub type Loan<'run, const RESOURCE: u8> = g::Msg<10, Endpoint<'run, RESOURCE>>;
pub type FileExit = g::Msg<241, i32>;
pub type FileApplied = g::Msg<165, ()>;
pub type AppliedResult<'a> = g::Msg<42, &'a [u8]>;
pub type MigrationIntent = g::Msg<18, super::imp::ApiArguments<{ super::imp::MIGRATE }, false>>;
pub type TestIntent = g::Msg<19, super::imp::ApiArguments<{ super::imp::TEST }, false>>;
pub type MigrationRequest = g::Msg<20, super::imp::ApiArguments<{ super::imp::MIGRATE }>>;
pub type MigrationCompleted<'a> = g::Msg<21, &'a [u8]>;
pub type TestRequest = g::Msg<22, super::imp::ApiArguments<{ super::imp::TEST }>>;
pub type TestCompleted<'a> = g::Msg<23, &'a [u8]>;
pub type BackendRequest = g::Msg<24, super::imp::ApiArguments<{ super::imp::IMPLEMENT }>>;
pub type MigrationDone = g::Msg<40, ()>;
pub type BackendReady = g::Msg<41, ()>;
pub type PassedAdmin<'a> = g::Msg<50, &'a [u8]>;
pub type PassedBackend<'a> = g::Msg<51, &'a [u8]>;
pub type FailedAdmin<'a> = g::Msg<60, &'a [u8]>;
pub type FailedBackend<'a> = g::Msg<61, &'a [u8]>;

// Ordinary descriptor operations stay available while the resource is in use.
fn file_io<const FILE: u8, const OWNER: u8>() -> impl Projectable {
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
    let filestat = g::seq(
        g::send::<FILE, OWNER, protocol::FdFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdFilestatGetRetMsg>(),
    );
    let pathstat = g::seq(
        g::send::<FILE, OWNER, protocol::PathFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::PathFilestatGetRetMsg>(),
    );
    let close = g::seq(
        g::send::<FILE, OWNER, protocol::FdCloseReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdCloseRetMsg>(),
    );
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
    g::route(
        prestat,
        g::route(
            prestat_name,
            g::route(
                directory,
                g::route(
                    open,
                    g::route(
                        write,
                        g::route(
                            read,
                            g::route(stat, g::route(filestat, g::route(pathstat, close))),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}

fn outcome_metadata<const FILE: u8, const OWNER: u8>() -> impl Projectable {
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
    let stat = g::seq(
        g::send::<FILE, OWNER, protocol::FdFdstatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdFdstatGetRetMsg>(),
    );
    let filestat = g::seq(
        g::send::<FILE, OWNER, protocol::FdFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdFilestatGetRetMsg>(),
    );
    let pathstat = g::seq(
        g::send::<FILE, OWNER, protocol::PathFilestatGetReqMsg>(),
        g::send::<OWNER, FILE, protocol::PathFilestatGetRetMsg>(),
    );
    let close = g::seq(
        g::send::<FILE, OWNER, protocol::FdCloseReqMsg>(),
        g::send::<OWNER, FILE, protocol::FdCloseRetMsg>(),
    );
    g::route(
        prestat,
        g::route(
            prestat_name,
            g::route(
                directory,
                g::route(
                    open,
                    g::route(
                        stat,
                        g::route(
                            filestat,
                            g::route(
                                pathstat,
                                g::route(close, g::send::<FILE, OWNER, FileExit>()),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn outcome_reads<const FILE: u8, const READ: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, READ, protocol::FdReadReqMsg>(),
        g::send::<READ, FILE, protocol::FdReadRetMsg>(),
    )
    .roll()
}

pub struct Programs {
    pub admin: RoleProgram<ADMIN>,
    pub backend: RoleProgram<BACKEND>,
    pub tester: RoleProgram<TESTER>,
    pub database: RoleProgram<DATABASE>,
    pub test_env: RoleProgram<TEST_ENV>,
    pub database_resource: RoleProgram<DATABASE_RESOURCE>,
    pub test_resource: RoleProgram<TEST_RESOURCE>,
    pub admin_vm: RoleProgram<ADMIN_VM>,
    pub backend_vm: RoleProgram<BACKEND_VM>,
    pub tester_vm: RoleProgram<TESTER_VM>,
    pub admin_io: RoleProgram<ADMIN_IO>,
    pub backend_io: RoleProgram<BACKEND_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
    pub admin_outcome: RoleProgram<ADMIN_OUTCOME>,
    pub backend_outcome: RoleProgram<BACKEND_OUTCOME>,
    pub admin_outcome_file: RoleProgram<ADMIN_OUTCOME_FILE>,
    pub backend_outcome_file: RoleProgram<BACKEND_OUTCOME_FILE>,
    pub admin_file: RoleProgram<ADMIN_FILE>,
    pub backend_file: RoleProgram<BACKEND_FILE>,
    pub tester_file: RoleProgram<TESTER_FILE>,
    pub admin_close: RoleProgram<ADMIN_CLOSE>,
    pub backend_close: RoleProgram<BACKEND_CLOSE>,
    pub tester_close: RoleProgram<TESTER_CLOSE>,
    pub admin_outcome_read: RoleProgram<ADMIN_OUTCOME_READ>,
    pub backend_outcome_read: RoleProgram<BACKEND_OUTCOME_READ>,
}
pub fn programs() -> Programs {
    let bootstrap = g::par(
        g::par(
            g::send::<ADMIN, ADMIN_VM, Loan<ADMIN_FILE>>(),
            g::send::<ADMIN_OUTCOME, ADMIN_VM, Loan<ADMIN_OUTCOME_FILE>>(),
        ),
        g::par(
            g::par(
                g::send::<BACKEND, BACKEND_VM, Loan<BACKEND_FILE>>(),
                g::send::<BACKEND_OUTCOME, BACKEND_VM, Loan<BACKEND_OUTCOME_FILE>>(),
            ),
            g::send::<TESTER, TESTER_VM, Loan<TESTER_FILE>>(),
        ),
    );
    let migration = g::seq(
        g::send::<ADMIN_FILE, ADMIN_CLOSE, MigrationIntent>(),
        g::seq(
            g::send::<ADMIN_CLOSE, DATABASE, MigrationIntent>(),
            g::seq(
                g::send::<DATABASE_RESOURCE, ADMIN_CLOSE, MigrationRequest>(),
                g::seq(
                    g::send::<DATABASE, ADMIN_CLOSE, Loan<DATABASE_RESOURCE>>(),
                    g::seq(
                        g::send::<DATABASE_RESOURCE, DATABASE, MigrationCompleted>(),
                        g::seq(
                            g::send::<ADMIN_CLOSE, DATABASE, Loan<DATABASE_RESOURCE>>(),
                            g::par(
                                g::send::<ADMIN_CLOSE, BACKEND_CLOSE, MigrationDone>(),
                                g::seq(
                                    g::send::<ADMIN_CLOSE, ADMIN, AppliedResult>(),
                                    g::send::<ADMIN, ADMIN_FILE, FileApplied>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let backend = g::seq(
        g::send::<BACKEND_FILE, BACKEND_CLOSE, BackendRequest>(),
        g::par(
            g::send::<BACKEND_CLOSE, TESTER_CLOSE, BackendReady>(),
            g::seq(
                g::send::<BACKEND_CLOSE, BACKEND, AppliedResult>(),
                g::send::<BACKEND, BACKEND_FILE, FileApplied>(),
            ),
        ),
    );
    let test_resources = g::seq(
        g::send::<TESTER_FILE, TESTER_CLOSE, TestIntent>(),
        g::seq(
            g::send::<TESTER_CLOSE, TEST_ENV, TestIntent>(),
            g::seq(
                g::send::<TEST_RESOURCE, TESTER_CLOSE, TestRequest>(),
                g::seq(
                    g::send::<TEST_ENV, TESTER_CLOSE, Loan<TEST_RESOURCE>>(),
                    g::seq(
                        g::send::<TEST_RESOURCE, TEST_ENV, TestCompleted>(),
                        g::send::<TESTER_CLOSE, TEST_ENV, Loan<TEST_RESOURCE>>(),
                    ),
                ),
            ),
        ),
    );
    let outcome = g::route(
        g::par(
            g::seq(
                g::send::<TESTER_CLOSE, ADMIN_OUTCOME, PassedAdmin>(),
                g::send::<TESTER_CLOSE, ADMIN_OUTCOME, Loan<ADMIN_OUTCOME_READ>>(),
            ),
            g::seq(
                g::send::<TESTER_CLOSE, BACKEND_OUTCOME, PassedBackend>(),
                g::send::<TESTER_CLOSE, BACKEND_OUTCOME, Loan<BACKEND_OUTCOME_READ>>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<TESTER_CLOSE, ADMIN_OUTCOME, FailedAdmin>(),
                g::send::<TESTER_CLOSE, ADMIN_OUTCOME, Loan<ADMIN_OUTCOME_READ>>(),
            ),
            g::seq(
                g::send::<TESTER_CLOSE, BACKEND_OUTCOME, FailedBackend>(),
                g::send::<TESTER_CLOSE, BACKEND_OUTCOME, Loan<BACKEND_OUTCOME_READ>>(),
            ),
        ),
    )
    .resolve::<TEST_RESULT>();
    let test_admission = g::seq(
        test_resources,
        g::seq(
            outcome,
            g::seq(
                g::send::<TESTER_CLOSE, TESTER, AppliedResult>(),
                g::send::<TESTER, TESTER_FILE, FileApplied>(),
            ),
        ),
    );
    let test = g::seq(
        test_admission,
        g::par(
            outcome_reads::<ADMIN_OUTCOME_FILE, ADMIN_OUTCOME_READ>(),
            outcome_reads::<BACKEND_OUTCOME_FILE, BACKEND_OUTCOME_READ>(),
        ),
    );
    let ordinary_io = g::par(
        file_io::<ADMIN_FILE, ADMIN>(),
        g::par(
            file_io::<BACKEND_FILE, BACKEND>(),
            file_io::<TESTER_FILE, TESTER>(),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<ADMIN_VM, ADMIN_IO>(),
        g::par(
            crate::shell::global::private_io::<BACKEND_VM, BACKEND_IO>(),
            crate::shell::global::private_io::<TESTER_VM, TESTER_IO>(),
        ),
    );
    let results = g::par(
        outcome_metadata::<ADMIN_OUTCOME_FILE, ADMIN_OUTCOME>(),
        outcome_metadata::<BACKEND_OUTCOME_FILE, BACKEND_OUTCOME>(),
    );
    let task = g::seq(migration, g::seq(backend, test));
    let work = g::par(g::par(ordinary_io, private), g::par(task, results));
    let finish = g::par(
        g::seq(
            g::send::<ADMIN_FILE, ADMIN, FileExit>(),
            g::seq(
                g::par(
                    g::send::<ADMIN_VM, ADMIN, Loan<ADMIN_FILE>>(),
                    g::send::<ADMIN_VM, ADMIN_OUTCOME, Loan<ADMIN_OUTCOME_FILE>>(),
                ),
                g::send::<ADMIN_VM, ADMIN_IO, crate::shell::global::Exit>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<BACKEND_FILE, BACKEND, FileExit>(),
                g::seq(
                    g::par(
                        g::send::<BACKEND_VM, BACKEND, Loan<BACKEND_FILE>>(),
                        g::send::<BACKEND_VM, BACKEND_OUTCOME, Loan<BACKEND_OUTCOME_FILE>>(),
                    ),
                    g::send::<BACKEND_VM, BACKEND_IO, crate::shell::global::Exit>(),
                ),
            ),
            g::seq(
                g::send::<TESTER_FILE, TESTER, FileExit>(),
                g::seq(
                    g::send::<TESTER_VM, TESTER, Loan<TESTER_FILE>>(),
                    g::send::<TESTER_VM, TESTER_IO, crate::shell::global::Exit>(),
                ),
            ),
        ),
    );
    let global = g::seq(bootstrap, g::seq(work, finish));
    Programs {
        admin: project(&global),
        backend: project(&global),
        tester: project(&global),
        database: project(&global),
        test_env: project(&global),
        database_resource: project(&global),
        test_resource: project(&global),
        admin_vm: project(&global),
        backend_vm: project(&global),
        tester_vm: project(&global),
        admin_io: project(&global),
        backend_io: project(&global),
        tester_io: project(&global),
        admin_outcome: project(&global),
        backend_outcome: project(&global),
        admin_outcome_file: project(&global),
        backend_outcome_file: project(&global),
        admin_file: project(&global),
        backend_file: project(&global),
        tester_file: project(&global),
        admin_close: project(&global),
        backend_close: project(&global),
        tester_close: project(&global),
        admin_outcome_read: project(&global),
        backend_outcome_read: project(&global),
    }
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"8E",
        "roles":{
            "DB_ADMIN":ADMIN,"BACKEND_DEV":BACKEND,"TESTER":TESTER,
            "DATABASE":DATABASE,"TEST_ENV":TEST_ENV,
            "ADMIN_VM":ADMIN_VM,"BACKEND_VM":BACKEND_VM,"TESTER_VM":TESTER_VM,
            "ADMIN_IO":ADMIN_IO,"BACKEND_IO":BACKEND_IO,"TESTER_IO":TESTER_IO,
            "ADMIN_OUTCOME":ADMIN_OUTCOME,"BACKEND_OUTCOME":BACKEND_OUTCOME,
            "ADMIN_OUTCOME_FILE":ADMIN_OUTCOME_FILE,"BACKEND_OUTCOME_FILE":BACKEND_OUTCOME_FILE,
            "DATABASE_RESOURCE":DATABASE_RESOURCE,"TEST_RESOURCE":TEST_RESOURCE,
            "ADMIN_FILE":ADMIN_FILE,"BACKEND_FILE":BACKEND_FILE,"TESTER_FILE":TESTER_FILE,
            "ADMIN_CLOSE":ADMIN_CLOSE,"BACKEND_CLOSE":BACKEND_CLOSE,"TESTER_CLOSE":TESTER_CLOSE,
            "ADMIN_OUTCOME_READ":ADMIN_OUTCOME_READ,"BACKEND_OUTCOME_READ":BACKEND_OUTCOME_READ
        },"service_fd":SERVICE_FD,"outcome_fd":OUTCOME_FD,"programs":[
            rows(&self.admin),rows(&self.backend),rows(&self.tester),rows(&self.database),
            rows(&self.test_env),rows(&self.database_resource),rows(&self.test_resource),
            rows(&self.admin_vm),rows(&self.backend_vm),rows(&self.tester_vm),
            rows(&self.admin_io),rows(&self.backend_io),rows(&self.tester_io),
            rows(&self.admin_outcome),rows(&self.backend_outcome),
            rows(&self.admin_outcome_file),rows(&self.backend_outcome_file),
            rows(&self.admin_file),rows(&self.backend_file),rows(&self.tester_file),
            rows(&self.admin_close),rows(&self.backend_close),rows(&self.tester_close),
            rows(&self.admin_outcome_read),rows(&self.backend_outcome_read)
        ]})
    }
}
