//! Actual operation files, API agreement, both implementation orders and revision.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const BACKEND_VM: u8 = 0;
pub const FRONTEND_VM: u8 = 1;
pub const TESTER_VM: u8 = 2;
pub const BACKEND_IO: u8 = 3;
pub const FRONTEND_IO: u8 = 4;
pub const TESTER_IO: u8 = 5;
pub const DEFINE_OWNER: u8 = 6;
pub const DEFINE_CLOSE: u8 = 7;
pub const DEFINE_FILE: u8 = 8;
pub const BACKEND_OWNER: u8 = 9;
pub const BACKEND_CLOSE: u8 = 10;
pub const BACKEND_FILE: u8 = 11;
pub const FRONTEND_OWNER: u8 = 12;
pub const FRONTEND_CLOSE: u8 = 13;
pub const FRONTEND_FILE: u8 = 14;
pub const AGREEMENT_OWNER: u8 = 15;
pub const AGREEMENT_CLOSE: u8 = 16;
pub const AGREEMENT_FILE: u8 = 17;
pub const TESTER_OWNER: u8 = 21;
pub const TESTER_CLOSE: u8 = 22;
pub const TESTER_FILE: u8 = 23;
pub const REPO: u8 = 24;
pub const TEST_ENV: u8 = 25;
pub const STORE: u8 = 26;
pub const BACKEND_OUTCOME: u8 = 27;
pub const BACKEND_OUTCOME_FILE: u8 = 29;
pub const FRONTEND_OUTCOME: u8 = 30;
pub const FRONTEND_OUTCOME_FILE: u8 = 32;
pub const SESSION: u32 = 4;

const OPERATION_BINDING: protocol::FdBinding = protocol::FdBinding {
    read: Some(protocol::FdReadRow::Base),
    write: Some(protocol::FdWriteRow::Object),
    readdir: None,
};
pub const DEFINE_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/define_api",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    OPERATION_BINDING,
);
pub const BACKEND_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/implement_feature",
    ObjectId(5),
    FdSpec::new(5, 66, 1),
    b"",
    OPERATION_BINDING,
);
pub const FRONTEND_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/implement_feature",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    OPERATION_BINDING,
);
pub const AGREEMENT_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/agree_api",
    ObjectId(5),
    FdSpec::new(5, 66, 1),
    b"",
    OPERATION_BINDING,
);
pub const TESTER_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/run_tests",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    OPERATION_BINDING,
);
pub const OUTCOME_PATH: &[u8] = b"service/outcome";
pub const BACKEND_OUTCOME_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    OUTCOME_PATH,
    ObjectId(7),
    FdSpec::new(7, 2, 1),
    b"",
    protocol::FdBinding::read(protocol::FdReadRow::Base),
);
pub const FRONTEND_OUTCOME_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    OUTCOME_PATH,
    ObjectId(6),
    FdSpec::new(6, 2, 1),
    b"",
    protocol::FdBinding::read(protocol::FdReadRow::Base),
);
pub type Loan<'run, const RESOURCE: u8> = g::Msg<10, Endpoint<'run, RESOURCE>>;
pub type FileExit = g::Msg<241, i32>;
pub type DefineIntent = g::Msg<20, super::imp::Arguments<{ super::imp::DEFINE }>>;
pub type DefineApi = g::Msg<1, super::imp::Arguments<{ super::imp::DEFINE }>>;
pub type ApiDefined<'a> = g::Msg<2, &'a [u8]>;
pub type Contract<'a> = g::Msg<3, &'a [u8]>;
pub type BackendFirst<'a> = g::Msg<21, &'a [u8]>;
pub type FrontendFirst<'a> = g::Msg<22, &'a [u8]>;
pub type AgreementAccepted<'a> = g::Msg<27, &'a [u8]>;
pub type BackendIntent = g::Msg<23, super::imp::Arguments<{ super::imp::BACKEND }>>;
pub type ImplementBackend = g::Msg<5, super::imp::Arguments<{ super::imp::BACKEND }>>;
pub type BackendImplemented<'a> = g::Msg<6, &'a [u8]>;
pub type FrontendIntent = g::Msg<24, super::imp::Arguments<{ super::imp::FRONTEND }>>;
pub type ImplementFrontend = g::Msg<7, super::imp::Arguments<{ super::imp::FRONTEND }>>;
pub type FrontendImplemented<'a> = g::Msg<8, &'a [u8]>;
pub type Ready<'a> = g::Msg<13, &'a [u8]>;
pub type TestIntent = g::Msg<26, super::imp::Arguments<{ super::imp::TEST }>>;
pub type ReadForTests = g::Msg<14, super::imp::Arguments<{ super::imp::TEST }>>;
pub type RunTests = g::Msg<15, super::imp::Arguments<{ super::imp::TEST }>>;
pub type RepositoryTested<'a> = g::Msg<16, &'a [u8]>;
pub type EnvironmentTested<'a> = g::Msg<17, &'a [u8]>;
pub type BackendResult<'a> = g::Msg<18, &'a [u8]>;
pub type FrontendResult<'a> = g::Msg<19, &'a [u8]>;

// Each actual close is the operation's own projected request. The shell
// chooses it; no advance loan selects a command on the shell's behalf.
fn file_operation<const FILE: u8, const CLOSE: u8>(
    accepted: impl Projectable + Copy,
) -> impl Projectable + Copy {
    let request = g::send::<FILE, CLOSE, protocol::FdCloseObjectReqMsg>();
    let retry = g::seq(
        g::send::<CLOSE, FILE, protocol::FdCloseRejectedRetMsg>(),
        request,
    );
    g::seq(request, g::route(accepted, g::seq(retry.roll(), accepted)))
}

fn file_io<const FILE: u8, const OWNER: u8>() -> impl Projectable + Copy {
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

fn outcome_metadata<const FILE: u8, const OWNER: u8>() -> impl Projectable + Copy {
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
                    g::route(stat, g::route(filestat, g::route(pathstat, close))),
                ),
            ),
        ),
    )
    .roll()
}

pub type BackendExit = g::Msg<242, i32>;
pub type FrontendExit = g::Msg<243, i32>;
pub type TesterExit = g::Msg<244, i32>;

fn backend() -> impl Projectable + Copy {
    file_operation::<BACKEND_FILE, BACKEND_CLOSE>(g::seq(
        g::send::<BACKEND_CLOSE, STORE, BackendIntent>(),
        g::seq(
            g::send::<STORE, BACKEND_CLOSE, Loan<REPO>>(),
            g::seq(
                g::send::<REPO, STORE, ImplementBackend>(),
                g::seq(
                    g::send::<STORE, REPO, BackendImplemented>(),
                    g::seq(
                        g::send::<BACKEND_CLOSE, STORE, Loan<REPO>>(),
                        g::send::<BACKEND_CLOSE, BACKEND_FILE, protocol::FdCloseObjectRetMsg>(),
                    ),
                ),
            ),
        ),
    ))
}
fn frontend() -> impl Projectable + Copy {
    file_operation::<FRONTEND_FILE, FRONTEND_CLOSE>(g::seq(
        g::send::<FRONTEND_CLOSE, STORE, FrontendIntent>(),
        g::seq(
            g::send::<STORE, FRONTEND_CLOSE, Loan<REPO>>(),
            g::seq(
                g::send::<REPO, STORE, ImplementFrontend>(),
                g::seq(
                    g::send::<STORE, REPO, FrontendImplemented>(),
                    g::seq(
                        g::send::<FRONTEND_CLOSE, STORE, Loan<REPO>>(),
                        g::send::<FRONTEND_CLOSE, FRONTEND_FILE, protocol::FdCloseObjectRetMsg>(),
                    ),
                ),
            ),
        ),
    ))
}
pub(crate) fn choreography() -> impl Projectable + Copy {
    let definition = file_operation::<DEFINE_FILE, DEFINE_CLOSE>(g::seq(
        g::send::<DEFINE_CLOSE, STORE, DefineIntent>(),
        g::seq(
            g::send::<STORE, DEFINE_CLOSE, Loan<REPO>>(),
            g::seq(
                g::send::<REPO, STORE, DefineApi>(),
                g::seq(
                    g::send::<STORE, REPO, ApiDefined>(),
                    g::seq(
                        g::send::<DEFINE_CLOSE, STORE, Loan<REPO>>(),
                        g::seq(
                            g::send::<DEFINE_CLOSE, DEFINE_FILE, protocol::FdCloseObjectRetMsg>(),
                            g::send::<STORE, AGREEMENT_FILE, Contract>(),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let agreement = file_operation::<AGREEMENT_FILE, AGREEMENT_CLOSE>(g::route(
        g::seq(
            g::send::<AGREEMENT_CLOSE, STORE, BackendFirst>(),
            g::seq(
                g::send::<STORE, AGREEMENT_CLOSE, AgreementAccepted>(),
                g::seq(
                    g::send::<AGREEMENT_CLOSE, BACKEND_OUTCOME_FILE, BackendResult>(),
                    g::seq(
                        g::send::<AGREEMENT_CLOSE, AGREEMENT_FILE, protocol::FdCloseObjectRetMsg>(),
                        g::seq(
                            g::send::<AGREEMENT_CLOSE, AGREEMENT_FILE, Contract>(),
                            g::seq(backend(), frontend()),
                        ),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<AGREEMENT_CLOSE, STORE, FrontendFirst>(),
            g::seq(
                g::send::<STORE, AGREEMENT_CLOSE, AgreementAccepted>(),
                g::seq(
                    g::send::<AGREEMENT_CLOSE, BACKEND_OUTCOME_FILE, BackendResult>(),
                    g::seq(
                        g::send::<AGREEMENT_CLOSE, AGREEMENT_FILE, protocol::FdCloseObjectRetMsg>(),
                        g::seq(
                            g::send::<AGREEMENT_CLOSE, AGREEMENT_FILE, Contract>(),
                            g::seq(frontend(), backend()),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let revision = g::seq(
        definition,
        g::seq(agreement, g::send::<STORE, TESTER_FILE, Ready>()),
    );
    let testing = file_operation::<TESTER_FILE, TESTER_CLOSE>(g::seq(
        g::send::<TESTER_CLOSE, STORE, TestIntent>(),
        g::seq(
            g::par(
                g::send::<STORE, TESTER_CLOSE, Loan<REPO>>(),
                g::send::<STORE, TESTER_CLOSE, Loan<TEST_ENV>>(),
            ),
            g::seq(
                g::par(
                    g::send::<REPO, STORE, ReadForTests>(),
                    g::send::<TEST_ENV, STORE, RunTests>(),
                ),
                g::seq(
                    g::par(
                        g::send::<STORE, REPO, RepositoryTested>(),
                        g::send::<STORE, TEST_ENV, EnvironmentTested>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<TESTER_CLOSE, STORE, Loan<REPO>>(),
                            g::send::<TESTER_CLOSE, STORE, Loan<TEST_ENV>>(),
                        ),
                        g::seq(
                            g::par(
                                g::send::<STORE, BACKEND_OUTCOME_FILE, BackendResult>(),
                                g::send::<STORE, FRONTEND_OUTCOME_FILE, FrontendResult>(),
                            ),
                            g::seq(
                                g::send::<TESTER_CLOSE, TESTER_FILE, protocol::FdCloseObjectRetMsg>(
                                ),
                                g::send::<TESTER_CLOSE, TESTER_FILE, Ready>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let task = g::seq(revision.roll(), testing.roll());
    let controls = g::par(
        g::par(
            g::send::<STORE, DEFINE_OWNER, Loan<DEFINE_CLOSE>>(),
            g::send::<STORE, BACKEND_OWNER, Loan<BACKEND_CLOSE>>(),
        ),
        g::par(
            g::send::<STORE, FRONTEND_OWNER, Loan<FRONTEND_CLOSE>>(),
            g::par(
                g::send::<STORE, AGREEMENT_OWNER, Loan<AGREEMENT_CLOSE>>(),
                g::send::<STORE, TESTER_OWNER, Loan<TESTER_CLOSE>>(),
            ),
        ),
    );
    let files = g::par(
        g::par(
            g::send::<DEFINE_OWNER, BACKEND_VM, Loan<DEFINE_FILE>>(),
            g::send::<BACKEND_OWNER, BACKEND_VM, Loan<BACKEND_FILE>>(),
        ),
        g::par(
            g::par(
                g::send::<FRONTEND_OWNER, FRONTEND_VM, Loan<FRONTEND_FILE>>(),
                g::send::<AGREEMENT_OWNER, FRONTEND_VM, Loan<AGREEMENT_FILE>>(),
            ),
            g::par(
                g::send::<TESTER_OWNER, TESTER_VM, Loan<TESTER_FILE>>(),
                g::par(
                    g::send::<BACKEND_OUTCOME, BACKEND_VM, Loan<BACKEND_OUTCOME_FILE>>(),
                    g::send::<FRONTEND_OUTCOME, FRONTEND_VM, Loan<FRONTEND_OUTCOME_FILE>>(),
                ),
            ),
        ),
    );
    let metadata = g::par(
        g::par(
            file_io::<DEFINE_FILE, DEFINE_OWNER>(),
            file_io::<BACKEND_FILE, BACKEND_OWNER>(),
        ),
        g::par(
            g::par(
                file_io::<FRONTEND_FILE, FRONTEND_OWNER>(),
                g::par(
                    outcome_metadata::<AGREEMENT_FILE, AGREEMENT_OWNER>(),
                    g::seq(
                        g::send::<AGREEMENT_FILE, AGREEMENT_OWNER, protocol::FdWriteObjectReqMsg>(),
                        g::send::<AGREEMENT_OWNER, AGREEMENT_FILE, protocol::FdWriteObjectRetMsg>(),
                    )
                    .roll(),
                ),
            ),
            g::par(
                g::par(
                    outcome_metadata::<TESTER_FILE, TESTER_OWNER>(),
                    g::seq(
                        g::send::<TESTER_FILE, TESTER_OWNER, protocol::FdWriteObjectReqMsg>(),
                        g::send::<TESTER_OWNER, TESTER_FILE, protocol::FdWriteObjectRetMsg>(),
                    )
                    .roll(),
                ),
                g::par(
                    outcome_metadata::<BACKEND_OUTCOME_FILE, BACKEND_OUTCOME>(),
                    outcome_metadata::<FRONTEND_OUTCOME_FILE, FRONTEND_OUTCOME>(),
                ),
            ),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<BACKEND_VM, BACKEND_IO>(),
        g::par(
            crate::shell::global::private_io::<FRONTEND_VM, FRONTEND_IO>(),
            crate::shell::global::private_io::<TESTER_VM, TESTER_IO>(),
        ),
    );
    let finish = g::par(
        g::seq(
            g::par(
                g::send::<DEFINE_FILE, DEFINE_OWNER, FileExit>(),
                g::par(
                    g::send::<BACKEND_FILE, BACKEND_OWNER, FileExit>(),
                    g::send::<BACKEND_OUTCOME_FILE, BACKEND_OUTCOME, FileExit>(),
                ),
            ),
            g::seq(
                g::send::<BACKEND_VM, STORE, BackendExit>(),
                g::send::<BACKEND_VM, BACKEND_IO, crate::shell::global::Exit>(),
            ),
        ),
        g::par(
            g::seq(
                g::par(
                    g::send::<FRONTEND_FILE, FRONTEND_OWNER, FileExit>(),
                    g::par(
                        g::send::<AGREEMENT_FILE, AGREEMENT_OWNER, FileExit>(),
                        g::send::<FRONTEND_OUTCOME_FILE, FRONTEND_OUTCOME, FileExit>(),
                    ),
                ),
                g::seq(
                    g::send::<FRONTEND_VM, STORE, FrontendExit>(),
                    g::send::<FRONTEND_VM, FRONTEND_IO, crate::shell::global::Exit>(),
                ),
            ),
            g::seq(
                g::send::<TESTER_FILE, TESTER_OWNER, FileExit>(),
                g::seq(
                    g::send::<TESTER_VM, STORE, TesterExit>(),
                    g::send::<TESTER_VM, TESTER_IO, crate::shell::global::Exit>(),
                ),
            ),
        ),
    );
    g::seq(
        g::seq(controls, files),
        g::seq(g::par(task, g::par(metadata, private)), finish),
    )
}
pub struct Programs {
    pub backend_vm: RoleProgram<BACKEND_VM>,
    pub frontend_vm: RoleProgram<FRONTEND_VM>,
    pub tester_vm: RoleProgram<TESTER_VM>,
    pub backend_io: RoleProgram<BACKEND_IO>,
    pub frontend_io: RoleProgram<FRONTEND_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
    pub define_owner: RoleProgram<DEFINE_OWNER>,
    pub define_close: RoleProgram<DEFINE_CLOSE>,
    pub define_file: RoleProgram<DEFINE_FILE>,
    pub backend_owner: RoleProgram<BACKEND_OWNER>,
    pub backend_close: RoleProgram<BACKEND_CLOSE>,
    pub backend_file: RoleProgram<BACKEND_FILE>,
    pub frontend_owner: RoleProgram<FRONTEND_OWNER>,
    pub frontend_close: RoleProgram<FRONTEND_CLOSE>,
    pub frontend_file: RoleProgram<FRONTEND_FILE>,
    pub agreement_owner: RoleProgram<AGREEMENT_OWNER>,
    pub agreement_close: RoleProgram<AGREEMENT_CLOSE>,
    pub agreement_file: RoleProgram<AGREEMENT_FILE>,
    pub tester_owner: RoleProgram<TESTER_OWNER>,
    pub tester_close: RoleProgram<TESTER_CLOSE>,
    pub tester_file: RoleProgram<TESTER_FILE>,
    pub repo: RoleProgram<REPO>,
    pub test_env: RoleProgram<TEST_ENV>,
    pub store: RoleProgram<STORE>,
    pub backend_outcome: RoleProgram<BACKEND_OUTCOME>,
    pub backend_outcome_file: RoleProgram<BACKEND_OUTCOME_FILE>,
    pub frontend_outcome: RoleProgram<FRONTEND_OUTCOME>,
    pub frontend_outcome_file: RoleProgram<FRONTEND_OUTCOME_FILE>,
}
pub fn programs() -> Programs {
    let source = choreography();
    Programs {
        backend_vm: project(&source),
        frontend_vm: project(&source),
        tester_vm: project(&source),
        backend_io: project(&source),
        frontend_io: project(&source),
        tester_io: project(&source),
        define_owner: project(&source),
        define_close: project(&source),
        define_file: project(&source),
        backend_owner: project(&source),
        backend_close: project(&source),
        backend_file: project(&source),
        frontend_owner: project(&source),
        frontend_close: project(&source),
        frontend_file: project(&source),
        agreement_owner: project(&source),
        agreement_close: project(&source),
        agreement_file: project(&source),
        tester_owner: project(&source),
        tester_close: project(&source),
        tester_file: project(&source),
        repo: project(&source),
        test_env: project(&source),
        store: project(&source),
        backend_outcome: project(&source),
        backend_outcome_file: project(&source),
        frontend_outcome: project(&source),
        frontend_outcome_file: project(&source),
    }
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"4E",
                "session": SESSION, "operation_files": [
        {"name":"DEFINE","file":DEFINE_FILE,"metadata":DEFINE_OWNER,"control":DEFINE_CLOSE,
                        "guest":BACKEND_VM,"agent":"BACKEND_DEV","tool":"define_api","fd":DEFINE_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(DEFINE_OBJECT.path()).expect("static object path")},
        {"name":"BACKEND","file":BACKEND_FILE,"metadata":BACKEND_OWNER,"control":BACKEND_CLOSE,
                        "guest":BACKEND_VM,"agent":"BACKEND_DEV","tool":"implement_feature","fd":BACKEND_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(BACKEND_OBJECT.path()).expect("static object path")},
        {"name":"FRONTEND","file":FRONTEND_FILE,"metadata":FRONTEND_OWNER,"control":FRONTEND_CLOSE,
                        "guest":FRONTEND_VM,"agent":"FRONTEND_DEV","tool":"implement_feature","fd":FRONTEND_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(FRONTEND_OBJECT.path()).expect("static object path")},
        {"name":"AGREEMENT","file":AGREEMENT_FILE,"metadata":AGREEMENT_OWNER,"control":AGREEMENT_CLOSE,
                        "guest":FRONTEND_VM,"agent":"FRONTEND_DEV","tool":null,"fd":AGREEMENT_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(AGREEMENT_OBJECT.path()).expect("static object path")},
        {"name":"TESTER","file":TESTER_FILE,"metadata":TESTER_OWNER,"control":TESTER_CLOSE,
                        "guest":TESTER_VM,"agent":"TESTER","tool":"run_tests","fd":TESTER_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(TESTER_OBJECT.path()).expect("static object path")}
                    ], "outcome_files": [
        {"file":BACKEND_OUTCOME_FILE,"metadata":BACKEND_OUTCOME,
                        "guest":BACKEND_VM,"fd":BACKEND_OUTCOME_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(BACKEND_OUTCOME_OBJECT.path()).expect("static object path")},
        {"file":FRONTEND_OUTCOME_FILE,"metadata":FRONTEND_OUTCOME,
                        "guest":FRONTEND_VM,"fd":FRONTEND_OUTCOME_OBJECT.fd().fd(),
                        "path":core::str::from_utf8(FRONTEND_OUTCOME_OBJECT.path()).expect("static object path")}
                    ], "roles": {
                    "BACKEND_VM": BACKEND_VM,
                    "FRONTEND_VM": FRONTEND_VM,
                    "TESTER_VM": TESTER_VM,
                    "BACKEND_IO": BACKEND_IO,
                    "FRONTEND_IO": FRONTEND_IO,
                    "TESTER_IO": TESTER_IO,
                    "DEFINE_OWNER": DEFINE_OWNER,
                    "DEFINE_CLOSE": DEFINE_CLOSE,
                    "DEFINE_FILE": DEFINE_FILE,
                    "BACKEND_OWNER": BACKEND_OWNER,
                    "BACKEND_CLOSE": BACKEND_CLOSE,
                    "BACKEND_FILE": BACKEND_FILE,
                    "FRONTEND_OWNER": FRONTEND_OWNER,
                    "FRONTEND_CLOSE": FRONTEND_CLOSE,
                    "FRONTEND_FILE": FRONTEND_FILE,
                    "AGREEMENT_OWNER": AGREEMENT_OWNER,
                    "AGREEMENT_CLOSE": AGREEMENT_CLOSE,
                    "AGREEMENT_FILE": AGREEMENT_FILE,
                    "TESTER_OWNER": TESTER_OWNER,
                    "TESTER_CLOSE": TESTER_CLOSE,
                    "TESTER_FILE": TESTER_FILE,
                    "REPO": REPO,
                    "TEST_ENV": TEST_ENV,
                    "STORE": STORE,
                    "BACKEND_OUTCOME": BACKEND_OUTCOME,
                    "BACKEND_OUTCOME_FILE": BACKEND_OUTCOME_FILE,
                    "FRONTEND_OUTCOME": FRONTEND_OUTCOME,
                    "FRONTEND_OUTCOME_FILE": FRONTEND_OUTCOME_FILE
                }, "labels": {
                    "DefineIntent": DefineIntent::LOGICAL_LABEL,
                    "DefineApi": DefineApi::LOGICAL_LABEL,
                    "ApiDefined": ApiDefined::LOGICAL_LABEL,
                    "Contract": Contract::LOGICAL_LABEL,
                    "BackendFirst": BackendFirst::LOGICAL_LABEL,
                    "FrontendFirst": FrontendFirst::LOGICAL_LABEL,
                    "AgreementAccepted": AgreementAccepted::LOGICAL_LABEL,
                    "BackendIntent": BackendIntent::LOGICAL_LABEL,
                    "ImplementBackend": ImplementBackend::LOGICAL_LABEL,
                    "BackendImplemented": BackendImplemented::LOGICAL_LABEL,
                    "FrontendIntent": FrontendIntent::LOGICAL_LABEL,
                    "ImplementFrontend": ImplementFrontend::LOGICAL_LABEL,
                    "FrontendImplemented": FrontendImplemented::LOGICAL_LABEL,
                    "BackendExit": BackendExit::LOGICAL_LABEL,
                    "FrontendExit": FrontendExit::LOGICAL_LABEL,
                    "TesterExit": TesterExit::LOGICAL_LABEL,
                    "Ready": Ready::LOGICAL_LABEL,
                    "TestIntent": TestIntent::LOGICAL_LABEL,
                    "ReadForTests": ReadForTests::LOGICAL_LABEL,
                    "RunTests": RunTests::LOGICAL_LABEL,
                    "RepositoryTested": RepositoryTested::LOGICAL_LABEL,
                    "EnvironmentTested": EnvironmentTested::LOGICAL_LABEL,
                    "BackendResult": BackendResult::LOGICAL_LABEL,
                    "FrontendResult": FrontendResult::LOGICAL_LABEL
                }, "programs": [
                    rows(&self.backend_vm),
                    rows(&self.frontend_vm),
                    rows(&self.tester_vm),
                    rows(&self.backend_io),
                    rows(&self.frontend_io),
                    rows(&self.tester_io),
                    rows(&self.define_owner),
                    rows(&self.define_close),
                    rows(&self.define_file),
                    rows(&self.backend_owner),
                    rows(&self.backend_close),
                    rows(&self.backend_file),
                    rows(&self.frontend_owner),
                    rows(&self.frontend_close),
                    rows(&self.frontend_file),
                    rows(&self.agreement_owner),
                    rows(&self.agreement_close),
                    rows(&self.agreement_file),
                    rows(&self.tester_owner),
                    rows(&self.tester_close),
                    rows(&self.tester_file),
                    rows(&self.repo),
                    rows(&self.test_env),
                    rows(&self.store),
                    rows(&self.backend_outcome),
                    rows(&self.backend_outcome_file),
                    rows(&self.frontend_outcome),
                    rows(&self.frontend_outcome_file)
                ]})
    }
}
