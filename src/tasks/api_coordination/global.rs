//! Agreement, actual repository writes, amendments and a fenced test snapshot.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const FRONTEND: u8 = 0;
pub const BACKEND: u8 = 1;
pub const TESTER: u8 = 2;
pub const STORE: u8 = 7;
pub const FRONTEND_IO: u8 = 8;
pub const BACKEND_IO: u8 = 9;
pub const TESTER_IO: u8 = 10;
pub const REPO: u8 = 13;
pub const TEST_ENV: u8 = 14;
pub const SESSION: u32 = 4;

pub type DefineApi<'a> = g::Msg<1, &'a [u8]>;
pub type ApiDefined<'a> = g::Msg<2, &'a [u8]>;
pub type Contract<'a> = g::Msg<3, &'a [u8]>;
pub type Agreement<'a> = g::Msg<4, &'a [u8]>;
pub type ImplementBackend<'a> = g::Msg<5, &'a [u8]>;
pub type BackendImplemented<'a> = g::Msg<6, &'a [u8]>;
pub type ImplementFrontend<'a> = g::Msg<7, &'a [u8]>;
pub type FrontendImplemented<'a> = g::Msg<8, &'a [u8]>;
pub type FrontendReady<'a> = g::Msg<9, &'a [u8]>;
pub type RepoLoan<'r> = g::Msg<10, Endpoint<'r, REPO>>;
pub type TestLoan<'r> = g::Msg<11, Endpoint<'r, TEST_ENV>>;
pub type Finalize<'a> = g::Msg<12, &'a [u8]>;
pub type Ready<'a> = g::Msg<13, &'a [u8]>;
pub type ReadForTests<'a> = g::Msg<14, &'a [u8]>;
pub type RunTests<'a> = g::Msg<15, &'a [u8]>;
pub type RepositoryTested<'a> = g::Msg<16, &'a [u8]>;
pub type EnvironmentTested<'a> = g::Msg<17, &'a [u8]>;
pub type BackendResult<'a> = g::Msg<18, &'a [u8]>;
pub type FrontendResult<'a> = g::Msg<19, &'a [u8]>;
pub type BackendIntent<'a> = g::Msg<20, &'a [u8]>;

fn backend() -> impl Projectable {
    g::seq(
        g::send::<BACKEND, STORE, BackendIntent>(),
        g::seq(
            g::send::<REPO, STORE, ImplementBackend>(),
            g::send::<STORE, REPO, BackendImplemented>(),
        ),
    )
}

fn frontend() -> impl Projectable {
    g::seq(
        g::send::<BACKEND, FRONTEND, RepoLoan>(),
        g::seq(
            g::send::<REPO, STORE, ImplementFrontend>(),
            g::seq(
                g::send::<STORE, REPO, FrontendImplemented>(),
                g::seq(
                    g::send::<FRONTEND, BACKEND, FrontendReady>(),
                    g::send::<FRONTEND, BACKEND, RepoLoan>(),
                ),
            ),
        ),
    )
}

fn implementation() -> impl Projectable {
    g::seq(
        g::send::<REPO, STORE, DefineApi>(),
        g::seq(
            g::send::<STORE, REPO, ApiDefined>(),
            g::seq(
                g::send::<BACKEND, FRONTEND, Contract>(),
                g::seq(
                    g::send::<FRONTEND, BACKEND, Agreement>(),
                    g::route(g::seq(backend(), frontend()), g::seq(frontend(), backend())),
                ),
            ),
        ),
    )
}

fn testing() -> impl Projectable {
    g::seq(
        g::send::<REPO, STORE, Finalize>(),
        g::seq(
            // Returning the actual editing Endpoint closes the old continuation.
            g::send::<BACKEND, STORE, RepoLoan>(),
            g::seq(
                g::send::<STORE, TESTER, Ready>(),
                g::seq(
                    g::par(
                        g::send::<STORE, TESTER, RepoLoan>(),
                        g::send::<STORE, TESTER, TestLoan>(),
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
                                    g::send::<TESTER, STORE, RepoLoan>(),
                                    g::send::<TESTER, STORE, TestLoan>(),
                                ),
                                g::par(
                                    g::send::<STORE, BACKEND, BackendResult>(),
                                    g::send::<STORE, FRONTEND, FrontendResult>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub struct Programs {
    pub frontend: RoleProgram<FRONTEND>,
    pub backend: RoleProgram<BACKEND>,
    pub tester: RoleProgram<TESTER>,
    pub store: RoleProgram<STORE>,
    pub repo: RoleProgram<REPO>,
    pub test_env: RoleProgram<TEST_ENV>,
    pub frontend_io: RoleProgram<FRONTEND_IO>,
    pub backend_io: RoleProgram<BACKEND_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
}

impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"4E",
            "session":SESSION,"roles":{"FRONTEND_DEV":FRONTEND,"BACKEND_DEV":BACKEND,
            "TESTER":TESTER,"STORE":STORE,"REPO":REPO,"TEST_ENV":TEST_ENV,
            "FRONTEND_IO":FRONTEND_IO,"BACKEND_IO":BACKEND_IO,"TESTER_IO":TESTER_IO},
            "labels":{"DefineApi":DefineApi::LOGICAL_LABEL,"ApiDefined":ApiDefined::LOGICAL_LABEL,
            "ImplementBackend":ImplementBackend::LOGICAL_LABEL,"BackendImplemented":BackendImplemented::LOGICAL_LABEL,
            "ImplementFrontend":ImplementFrontend::LOGICAL_LABEL,"FrontendImplemented":FrontendImplemented::LOGICAL_LABEL,
            "ReadForTests":ReadForTests::LOGICAL_LABEL,"RunTests":RunTests::LOGICAL_LABEL,
            "RepositoryTested":RepositoryTested::LOGICAL_LABEL,"EnvironmentTested":EnvironmentTested::LOGICAL_LABEL,
            "BackendResult":BackendResult::LOGICAL_LABEL,"FrontendResult":FrontendResult::LOGICAL_LABEL,
            "BackendIntent":BackendIntent::LOGICAL_LABEL,"Finalize":Finalize::LOGICAL_LABEL,
            "Contract":Contract::LOGICAL_LABEL,"Agreement":Agreement::LOGICAL_LABEL},
            "programs":[rows(&self.frontend),rows(&self.backend),rows(&self.tester),rows(&self.store),
            rows(&self.repo),rows(&self.test_env),rows(&self.frontend_io),rows(&self.backend_io),rows(&self.tester_io)]})
    }
}

pub fn programs() -> Programs {
    let business = g::seq(
        g::send::<STORE, BACKEND, RepoLoan>(),
        g::seq(
            implementation(),
            g::route(g::seq(implementation().roll(), testing()), testing()),
        ),
    );
    let protocol = g::par(
        business,
        g::par(
            crate::shell::global::private_io::<FRONTEND, FRONTEND_IO>(),
            g::par(
                crate::shell::global::private_io::<BACKEND, BACKEND_IO>(),
                crate::shell::global::private_io::<TESTER, TESTER_IO>(),
            ),
        ),
    );
    Programs {
        frontend: project(&protocol),
        backend: project(&protocol),
        tester: project(&protocol),
        store: project(&protocol),
        repo: project(&protocol),
        test_env: project(&protocol),
        frontend_io: project(&protocol),
        backend_io: project(&protocol),
        tester_io: project(&protocol),
    }
}
