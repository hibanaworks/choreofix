//! Each mounted file is the shell's projection of actual CI/CD operations.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const DEV_WASI: u8 = 0;
pub const TEST_WASI: u8 = 1;
pub const DEPLOY_WASI: u8 = 2;
pub const DEVELOPER: u8 = 3;
pub const TESTER: u8 = 4;
pub const DEPLOYER: u8 = 5;
pub const BUILD_OWNER: u8 = 6;
pub const DEPLOY_OWNER: u8 = 7;
pub const DEV_IO: u8 = 8;
pub const TEST_IO: u8 = 9;
pub const DEPLOY_IO: u8 = 10;
pub const BUILD_SERVER: u8 = 11;
pub const DEPLOY_ENV: u8 = 12;
pub const DEV_FILE: u8 = 13;
pub const TEST_FILE: u8 = 14;
pub const DEPLOY_FILE: u8 = 15;
pub const DEV_CLOSE: u8 = 16;
pub const TEST_CLOSE: u8 = 17;
pub const DEPLOY_CLOSE: u8 = 18;
pub const SESSION: u32 = 16;
pub type Loan<'run, const RESOURCE: u8> = g::Msg<10, Endpoint<'run, RESOURCE>>;
pub type CompileIntent = g::Msg<18, super::imp::Arguments<1>>;
pub type TestIntent = g::Msg<19, super::imp::Arguments<2>>;
pub type DeployIntent = g::Msg<24, super::imp::Arguments<3>>;
pub type CompileRequest = g::Msg<30, super::imp::Arguments<1>>;
pub type CompileResult<'a> = g::Msg<31, &'a [u8]>;
pub type TestRequest = g::Msg<32, super::imp::Arguments<2>>;
pub type TestResult<'a> = g::Msg<33, &'a [u8]>;
pub type DeployRequest = g::Msg<34, super::imp::Arguments<3>>;
pub type DeployResult<'a> = g::Msg<35, &'a [u8]>;
// Each notice carries its actual result. Its source occurrence is in the global;
// the JSON kind is display/audit data, never a second progress cursor.
pub type Notice<'a> = g::Msg<36, &'a [u8]>;
pub type FileExit = g::Msg<241, i32>;
pub const COMPILE_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/compile_and_test",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);
pub const TEST_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/run_integration_tests",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);
pub const DEPLOY_OBJECT: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/deploy_application",
    ObjectId(4),
    FdSpec::new(4, 66, 1),
    b"",
    protocol::FdBinding {
        read: Some(protocol::FdReadRow::Base),
        write: Some(protocol::FdWriteRow::Object),
        readdir: None,
    },
);

// Reused syscall fragments have no application phase or effect dispatch.
fn file_io<const FILE: u8, const OWNER: u8>() -> impl Projectable + Copy {
    g::route(
        g::seq(
            g::send::<FILE, OWNER, protocol::PathOpenReqMsg>(),
            g::send::<OWNER, FILE, protocol::PathOpenRetMsg>(),
        ),
        g::route(
            g::seq(
                g::send::<FILE, OWNER, protocol::FdWriteObjectReqMsg>(),
                g::send::<OWNER, FILE, protocol::FdWriteObjectRetMsg>(),
            ),
            g::route(
                g::seq(
                    g::send::<FILE, OWNER, protocol::FdPrestatGetReqMsg>(),
                    g::send::<OWNER, FILE, protocol::FdPrestatGetRetMsg>(),
                ),
                g::route(
                    g::seq(
                        g::send::<FILE, OWNER, protocol::FdPrestatDirNameReqMsg>(),
                        g::send::<OWNER, FILE, protocol::FdPrestatDirNameRetMsg>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<FILE, OWNER, protocol::FdReaddirReqMsg>(),
                            g::send::<OWNER, FILE, protocol::FdReaddirRetMsg>(),
                        ),
                        g::route(
                            g::seq(
                                g::send::<FILE, OWNER, protocol::FdFdstatGetReqMsg>(),
                                g::send::<OWNER, FILE, protocol::FdFdstatGetRetMsg>(),
                            ),
                            g::route(
                                g::seq(
                                    g::send::<FILE, OWNER, protocol::FdFilestatGetReqMsg>(),
                                    g::send::<OWNER, FILE, protocol::FdFilestatGetRetMsg>(),
                                ),
                                g::route(
                                    g::seq(
                                        g::send::<FILE, OWNER, protocol::PathFilestatGetReqMsg>(),
                                        g::send::<OWNER, FILE, protocol::PathFilestatGetRetMsg>(),
                                    ),
                                    g::seq(
                                        g::send::<FILE, OWNER, protocol::FdCloseReqMsg>(),
                                        g::send::<OWNER, FILE, protocol::FdCloseRetMsg>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn operation<const FILE: u8, const CLOSE: u8>(
    accepted: impl Projectable + Copy,
) -> impl Projectable + Copy {
    let request = g::send::<FILE, CLOSE, protocol::FdCloseObjectReqMsg>();
    let retry = g::seq(
        g::send::<CLOSE, FILE, protocol::FdCloseRejectedRetMsg>(),
        request,
    );
    g::seq(request, g::route(accepted, g::seq(retry.roll(), accepted)))
}
pub(crate) fn choreography() -> impl Projectable + Copy {
    let controls = g::par(
        g::send::<BUILD_OWNER, DEVELOPER, Loan<DEV_CLOSE>>(),
        g::par(
            g::send::<BUILD_OWNER, TESTER, Loan<TEST_CLOSE>>(),
            g::send::<DEPLOY_OWNER, DEPLOYER, Loan<DEPLOY_CLOSE>>(),
        ),
    );
    let files = g::par(
        g::send::<DEVELOPER, DEV_WASI, Loan<DEV_FILE>>(),
        g::par(
            g::send::<TESTER, TEST_WASI, Loan<TEST_FILE>>(),
            g::send::<DEPLOYER, DEPLOY_WASI, Loan<DEPLOY_FILE>>(),
        ),
    );
    let developer = operation::<DEV_FILE, DEV_CLOSE>(g::seq(
        g::send::<DEV_CLOSE, BUILD_OWNER, CompileIntent>(),
        g::seq(
            g::send::<BUILD_OWNER, DEV_CLOSE, Loan<BUILD_SERVER>>(),
            g::seq(
                g::send::<BUILD_SERVER, BUILD_OWNER, CompileRequest>(),
                g::seq(
                    g::send::<BUILD_OWNER, BUILD_SERVER, CompileResult>(),
                    g::seq(
                        g::send::<DEV_CLOSE, BUILD_OWNER, Loan<BUILD_SERVER>>(),
                        g::seq(
                            g::send::<DEV_CLOSE, DEV_FILE, protocol::FdCloseObjectRetMsg>(),
                            g::seq(
                                g::send::<DEV_CLOSE, DEV_FILE, Notice>(),
                                g::send::<DEV_CLOSE, TEST_FILE, Notice>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let tester = operation::<TEST_FILE, TEST_CLOSE>(g::seq(
        g::send::<TEST_CLOSE, BUILD_OWNER, TestIntent>(),
        g::seq(
            g::send::<BUILD_OWNER, TEST_CLOSE, Loan<BUILD_SERVER>>(),
            g::seq(
                g::send::<BUILD_SERVER, BUILD_OWNER, TestRequest>(),
                g::seq(
                    g::send::<BUILD_OWNER, BUILD_SERVER, TestResult>(),
                    g::seq(
                        g::send::<TEST_CLOSE, BUILD_OWNER, Loan<BUILD_SERVER>>(),
                        g::seq(
                            g::send::<TEST_CLOSE, TEST_FILE, protocol::FdCloseObjectRetMsg>(),
                            g::seq(
                                g::send::<TEST_CLOSE, TEST_FILE, Notice>(),
                                g::send::<TEST_CLOSE, DEPLOY_FILE, Notice>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let deployer = operation::<DEPLOY_FILE, DEPLOY_CLOSE>(g::seq(
        g::send::<DEPLOY_CLOSE, DEPLOY_OWNER, DeployIntent>(),
        g::seq(
            g::send::<DEPLOY_OWNER, DEPLOY_CLOSE, Loan<DEPLOY_ENV>>(),
            g::seq(
                g::send::<DEPLOY_ENV, DEPLOY_OWNER, DeployRequest>(),
                g::seq(
                    g::send::<DEPLOY_OWNER, DEPLOY_ENV, DeployResult>(),
                    g::seq(
                        g::send::<DEPLOY_CLOSE, DEPLOY_OWNER, Loan<DEPLOY_ENV>>(),
                        g::seq(
                            g::send::<DEPLOY_CLOSE, DEPLOY_FILE, protocol::FdCloseObjectRetMsg>(),
                            g::seq(
                                g::send::<DEPLOY_CLOSE, DEPLOY_FILE, Notice>(),
                                g::par(
                                    g::send::<DEPLOY_CLOSE, DEV_FILE, Notice>(),
                                    g::send::<DEPLOYER, TEST_FILE, Notice>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ));
    let task = g::seq(developer, g::seq(tester, deployer));
    let metadata = g::par(
        file_io::<DEV_FILE, DEVELOPER>(),
        g::par(
            file_io::<TEST_FILE, TESTER>(),
            file_io::<DEPLOY_FILE, DEPLOYER>(),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<DEV_WASI, DEV_IO>(),
        g::par(
            crate::shell::global::private_io::<TEST_WASI, TEST_IO>(),
            crate::shell::global::private_io::<DEPLOY_WASI, DEPLOY_IO>(),
        ),
    );
    let finish = g::par(
        g::seq(
            g::send::<DEV_FILE, DEVELOPER, FileExit>(),
            g::send::<DEV_WASI, DEV_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::seq(
                g::send::<TEST_FILE, TESTER, FileExit>(),
                g::send::<TEST_WASI, TEST_IO, crate::shell::global::Exit>(),
            ),
            g::seq(
                g::send::<DEPLOY_FILE, DEPLOYER, FileExit>(),
                g::send::<DEPLOY_WASI, DEPLOY_IO, crate::shell::global::Exit>(),
            ),
        ),
    );
    g::seq(
        g::seq(controls, files),
        g::seq(g::par(task, g::par(metadata, private)), finish),
    )
}
pub struct Programs {
    pub dev_wasi: RoleProgram<DEV_WASI>,
    pub test_wasi: RoleProgram<TEST_WASI>,
    pub deploy_wasi: RoleProgram<DEPLOY_WASI>,
    pub developer: RoleProgram<DEVELOPER>,
    pub tester: RoleProgram<TESTER>,
    pub deployer: RoleProgram<DEPLOYER>,
    pub build_owner: RoleProgram<BUILD_OWNER>,
    pub deploy_owner: RoleProgram<DEPLOY_OWNER>,
    pub dev_io: RoleProgram<DEV_IO>,
    pub test_io: RoleProgram<TEST_IO>,
    pub deploy_io: RoleProgram<DEPLOY_IO>,
    pub build_server: RoleProgram<BUILD_SERVER>,
    pub deploy_env: RoleProgram<DEPLOY_ENV>,
    pub dev_file: RoleProgram<DEV_FILE>,
    pub test_file: RoleProgram<TEST_FILE>,
    pub deploy_file: RoleProgram<DEPLOY_FILE>,
    pub dev_close: RoleProgram<DEV_CLOSE>,
    pub test_close: RoleProgram<TEST_CLOSE>,
    pub deploy_close: RoleProgram<DEPLOY_CLOSE>,
}
pub fn programs() -> Programs {
    let protocol = choreography();
    Programs {
        dev_wasi: project(&protocol),
        test_wasi: project(&protocol),
        deploy_wasi: project(&protocol),
        developer: project(&protocol),
        tester: project(&protocol),
        deployer: project(&protocol),
        build_owner: project(&protocol),
        deploy_owner: project(&protocol),
        dev_io: project(&protocol),
        test_io: project(&protocol),
        deploy_io: project(&protocol),
        build_server: project(&protocol),
        deploy_env: project(&protocol),
        dev_file: project(&protocol),
        test_file: project(&protocol),
        deploy_file: project(&protocol),
        dev_close: project(&protocol),
        test_close: project(&protocol),
        deploy_close: project(&protocol),
    }
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"16E","session":SESSION,
 "roles":{"DEV_WASI":DEV_WASI,"TEST_WASI":TEST_WASI,"DEPLOY_WASI":DEPLOY_WASI,"DEVELOPER":DEVELOPER,"TESTER":TESTER,"DEPLOYER":DEPLOYER,"BUILD_OWNER":BUILD_OWNER,"DEPLOY_OWNER":DEPLOY_OWNER,"DEV_IO":DEV_IO,"TEST_IO":TEST_IO,"DEPLOY_IO":DEPLOY_IO,"BUILD_SERVER":BUILD_SERVER,"DEPLOY_ENV":DEPLOY_ENV,"DEV_FILE":DEV_FILE,"TEST_FILE":TEST_FILE,"DEPLOY_FILE":DEPLOY_FILE,"DEV_CLOSE":DEV_CLOSE,"TEST_CLOSE":TEST_CLOSE,"DEPLOY_CLOSE":DEPLOY_CLOSE},
 "labels":{"Notice":36,"FileExit":241},
 "operation_files":[{"agent":"DEVELOPER","tool":"compile_and_test","guest":DEV_WASI,"file":DEV_FILE,"metadata":DEVELOPER,"close":DEV_CLOSE,"resource":BUILD_SERVER,"effect_owner":BUILD_OWNER,"path":"service/compile_and_test","fd":4,"intent_label":18,"request_label":30,"result_label":31,"result_kind":"compile_result"},{"agent":"TESTER","tool":"run_integration_tests","guest":TEST_WASI,"file":TEST_FILE,"metadata":TESTER,"close":TEST_CLOSE,"resource":BUILD_SERVER,"effect_owner":BUILD_OWNER,"path":"service/run_integration_tests","fd":4,"intent_label":19,"request_label":32,"result_label":33,"result_kind":"test_result"},{"agent":"DEPLOYER","tool":"deploy_application","guest":DEPLOY_WASI,"file":DEPLOY_FILE,"metadata":DEPLOYER,"close":DEPLOY_CLOSE,"resource":DEPLOY_ENV,"effect_owner":DEPLOY_OWNER,"path":"service/deploy_application","fd":4,"intent_label":24,"request_label":34,"result_label":35,"result_kind":"deploy_result"}],
 "programs":[rows(&self.dev_wasi),rows(&self.test_wasi),rows(&self.deploy_wasi),rows(&self.developer),rows(&self.tester),rows(&self.deployer),rows(&self.build_owner),rows(&self.deploy_owner),rows(&self.dev_io),rows(&self.test_io),rows(&self.deploy_io),rows(&self.build_server),rows(&self.deploy_env),rows(&self.dev_file),rows(&self.test_file),rows(&self.deploy_file),rows(&self.dev_close),rows(&self.test_close),rows(&self.deploy_close)]})
    }
}
