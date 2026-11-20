//! The build workspace and registry are independent affine resources.
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const REGISTRY_OWNER: u8 = 1;
pub const BUILD_SERVER: u8 = 2;
pub const ARTIFACT_STORE: u8 = 3;
pub const FRONTEND_DEV: u8 = 4;
pub const BACKEND_DEV: u8 = 5;
pub const FRONTEND_FILE: u8 = 6;
pub const BACKEND_FILE: u8 = 7;

pub type Publish<const DEVELOPER: u8> = g::Msg<80, Arguments<DEVELOPER>>;
pub type ReadBuild<const DEVELOPER: u8> = g::Msg<81, Arguments<DEVELOPER>>;
pub type RegisterArtifact<const DEVELOPER: u8> = g::Msg<82, Arguments<DEVELOPER>>;
pub type Published<'a> = g::Msg<83, &'a [u8]>;
pub type RegistryIntent<const DEVELOPER: u8> = g::Msg<DEVELOPER, Arguments<DEVELOPER>>;
pub type WorkspaceLoan<'run> = g::Msg<86, Endpoint<'run, BUILD_SERVER>>;
pub type RegistryLoan<'run> = g::Msg<87, Endpoint<'run, ARTIFACT_STORE>>;

/// Publication consumes evidence from both delegated resources.
pub(super) fn publication<const DEVELOPER: u8, const FILE: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<FILE, DEVELOPER, Publish<DEVELOPER>>(),
        g::seq(
            g::send::<DEVELOPER, REGISTRY_OWNER, RegistryIntent<DEVELOPER>>(),
            g::seq(
                g::send::<REGISTRY_OWNER, DEVELOPER, RegistryLoan>(),
                g::seq(
                    g::par(
                        g::send::<BUILD_SERVER, REGISTRY_OWNER, ReadBuild<DEVELOPER>>(),
                        g::send::<ARTIFACT_STORE, REGISTRY_OWNER, RegisterArtifact<DEVELOPER>>(),
                    ),
                    g::seq(
                        g::send::<REGISTRY_OWNER, DEVELOPER, Published>(),
                        g::seq(
                            g::send::<DEVELOPER, REGISTRY_OWNER, RegistryLoan>(),
                            g::send::<DEVELOPER, FILE, Published>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

use super::arguments::{
    Arguments, BACKEND_COMPONENT, E2E_SUITE, FRONTEND_COMPONENT, FULL_STACK, INTEGRATION_SUITE,
    PRODUCTION, QA_WORKSPACE, RELEASE_WORKSPACE, STAGING,
};
use crate::shell::{global::Loan, imp::review_reply::ReviewReply};

pub const BUILD_MASTER: u8 = 8;
pub const QA: u8 = 9;
pub const RELEASE: u8 = 10;
pub const TEST_ENV: u8 = 11;
pub const PROD_ENV: u8 = 12;
pub const QA_FILE: u8 = 13;
pub const RELEASE_FILE: u8 = 14;
pub const PROD_OWNER: u8 = 16;
pub const MASTER_FILE: u8 = 17;
pub const FRONTEND_VM: u8 = 18;
pub const BACKEND_VM: u8 = 19;
pub const QA_VM: u8 = 20;
pub const RELEASE_VM: u8 = 21;
pub const MASTER_VM: u8 = 22;
pub const FRONTEND_IO: u8 = 23;
pub const BACKEND_IO: u8 = 24;
pub const QA_IO: u8 = 25;
pub const RELEASE_IO: u8 = 26;
pub const MASTER_IO: u8 = 27;
pub const FRONTEND_DATA: u8 = 28;
pub const BACKEND_DATA: u8 = 29;
pub const QA_DATA: u8 = 30;
pub const RELEASE_DATA: u8 = 31;
pub const MASTER_DATA: u8 = 32;
pub const SESSION: u32 = 162;

pub type BeginFrontend = g::Msg<88, ()>;
pub type BeginBackend = g::Msg<89, ()>;
pub type BuildReport<'a, const DEVELOPER: u8> = g::Msg<DEVELOPER, &'a [u8]>;
pub type Compile<const COMPONENT: u8> = g::Msg<91, Arguments<COMPONENT>>;
pub type Compiled<'a> = g::Msg<92, &'a [u8]>;
pub type UnitTest<const COMPONENT: u8> = g::Msg<93, Arguments<COMPONENT>>;
pub type UnitTested<'a> = g::Msg<94, &'a [u8]>;
pub type PullForQa = g::Msg<95, Arguments<QA_WORKSPACE>>;
pub type PullForRelease = g::Msg<96, Arguments<RELEASE_WORKSPACE>>;
pub type QaPulled<'a> = g::Msg<97, ReviewReply<'a, true>>;
pub type ReleasePulled<'a> = g::Msg<98, ReviewReply<'a, true>>;
pub type Integration = g::Msg<99, Arguments<INTEGRATION_SUITE>>;
pub type E2e = g::Msg<100, Arguments<E2E_SUITE>>;
pub type Integrated<'a> = g::Msg<101, &'a [u8]>;
pub type E2eTested<'a> = g::Msg<102, &'a [u8]>;
pub type Stage = g::Msg<103, Arguments<FULL_STACK>>;
pub type Staged<'a> = g::Msg<104, &'a [u8]>;
pub type SmokeStaging = g::Msg<105, Arguments<STAGING>>;
pub type StagingSmoke<'a> = g::Msg<106, ReviewReply<'a, true>>;
pub type Deploy = g::Msg<107, Arguments<FULL_STACK>>;
pub type Deployed<'a> = g::Msg<108, ReviewReply<'a, true>>;
pub type SmokeProduction = g::Msg<109, Arguments<PRODUCTION>>;
pub type ProductionSmoke<'a> = g::Msg<110, &'a [u8]>;
pub type ReadyForQa = g::Msg<111, ()>;
pub type ReadyForRelease<'a> = g::Msg<112, &'a [u8]>;
pub type Promote<'a> = g::Msg<113, ReviewReply<'a, true>>;
pub type PipelineResult<'a> = g::Msg<114, &'a [u8]>;
pub type StagingResult<'a> = g::Msg<115, &'a [u8]>;
pub type StagingAborted<'a> = g::Msg<116, ReviewReply<'a, false>>;
pub type ChooseFrontend = g::Msg<117, Arguments<{ super::arguments::FRONTEND_FIRST }>>;
pub type ChooseBackend = g::Msg<118, Arguments<{ super::arguments::BACKEND_FIRST }>>;
pub type FinalResult<'a> = g::Msg<119, &'a [u8]>;
pub type StagingReady<'a> = g::Msg<120, ReviewReply<'a, true>>;
pub type FrontendFinished<'a> = g::Msg<121, &'a [u8]>;
pub type BackendFinished<'a> = g::Msg<122, &'a [u8]>;
pub type BuildVerified<'a> = g::Msg<139, ReviewReply<'a, true>>;
pub type FrontendVerified<'a> = g::Msg<125, ReviewReply<'a, true>>;
pub type BackendVerified<'a> = g::Msg<126, ReviewReply<'a, true>>;
pub type QaFinished<'a> = g::Msg<123, &'a [u8]>;
pub type ReleaseFinished<'a> = g::Msg<124, &'a [u8]>;

pub type Revise<'a, const DEVELOPER: u8> = g::Msg<DEVELOPER, ReviewReply<'a, false>>;
pub type RegistryAccepted<'a, const DEVELOPER: u8> = g::Msg<DEVELOPER, ReviewReply<'a, true>>;
pub type RegistryRejected<'a> = g::Msg<128, ReviewReply<'a, false>>;
pub type WorkspaceAvailable = g::Msg<127, ()>;

pub type QaPassed<'a> = g::Msg<129, ReviewReply<'a, true>>;
pub type QaFailed<'a> = g::Msg<130, ReviewReply<'a, false>>;
pub type QaReport<'a> = g::Msg<131, &'a [u8]>;
pub type RepairDecision = g::Msg<132, Arguments<{ super::arguments::REPAIR_SELECTION }>>;
pub type QaSnapshot = g::Msg<133, ()>;
pub type BuildEnded<'a, const DEVELOPER: u8> = g::Msg<DEVELOPER, &'a [u8]>;
pub type ReleaseApproved<'a> = g::Msg<134, ReviewReply<'a, true>>;
pub type ReleaseDenied<'a> = g::Msg<135, ReviewReply<'a, false>>;
pub type QaReviseFrontend<'a> = g::Msg<140, ReviewReply<'a, false>>;
pub type QaReviseBackend<'a> = g::Msg<141, ReviewReply<'a, false>>;
pub type ReconsiderQa = g::Msg<137, ()>;

fn build<const DEVELOPER: u8, const FILE: u8, const COMPONENT: u8>() -> impl Projectable + Copy {
    let report = || g::send::<BUILD_SERVER, BUILD_MASTER, BuildReport<DEVELOPER>>();
    let failed = || {
        g::seq(
            g::send::<BUILD_SERVER, REGISTRY_OWNER, RegistryRejected>(),
            report(),
        )
    };
    g::seq(
        g::send::<BUILD_MASTER, DEVELOPER, WorkspaceLoan>(),
        g::seq(
            g::send::<BUILD_SERVER, FILE, WorkspaceAvailable>(),
            g::seq(
                g::send::<FILE, BUILD_SERVER, Compile<COMPONENT>>(),
                g::seq(
                    g::route(
                        g::seq(
                            g::send::<BUILD_SERVER, FILE, Compiled>(),
                            g::seq(
                                g::send::<FILE, BUILD_SERVER, UnitTest<COMPONENT>>(),
                                g::route(
                                    g::seq(
                                        g::send::<BUILD_SERVER, FILE, UnitTested>(),
                                        g::seq(
                                            g::send::<
                                                BUILD_SERVER,
                                                REGISTRY_OWNER,
                                                RegistryAccepted<DEVELOPER>,
                                            >(),
                                            g::seq(publication::<DEVELOPER, FILE>(), report()),
                                        ),
                                    ),
                                    g::seq(failed(), g::send::<BUILD_SERVER, FILE, UnitTested>()),
                                ),
                            ),
                        ),
                        g::seq(failed(), g::send::<BUILD_SERVER, FILE, Compiled>()),
                    ),
                    g::send::<DEVELOPER, BUILD_MASTER, WorkspaceLoan>(),
                ),
            ),
        ),
    )
}

pub fn pipeline() -> impl Projectable {
    let frontend = || build::<FRONTEND_DEV, FRONTEND_FILE, FRONTEND_COMPONENT>();
    let backend = || build::<BACKEND_DEV, BACKEND_FILE, BACKEND_COMPONENT>();
    let builds = g::seq(
        g::par(
            g::send::<BUILD_MASTER, FRONTEND_FILE, BeginFrontend>(),
            g::send::<BUILD_MASTER, BACKEND_FILE, BeginBackend>(),
        ),
        g::route(
            g::seq(
                g::send::<MASTER_FILE, BUILD_MASTER, ChooseFrontend>(),
                g::seq(frontend(), backend()),
            ),
            g::seq(
                g::send::<MASTER_FILE, BUILD_MASTER, ChooseBackend>(),
                g::seq(backend(), frontend()),
            ),
        ),
    );
    let verified_frontend = || {
        g::seq(
            g::send::<BUILD_MASTER, REGISTRY_OWNER, FrontendVerified>(),
            g::seq(
                g::send::<REGISTRY_OWNER, FRONTEND_DEV, BuildVerified>(),
                g::send::<FRONTEND_DEV, FRONTEND_FILE, BuildVerified>(),
            ),
        )
    };
    let verified_backend = || {
        g::seq(
            g::send::<BUILD_MASTER, REGISTRY_OWNER, BackendVerified>(),
            g::seq(
                g::send::<REGISTRY_OWNER, BACKEND_DEV, BuildVerified>(),
                g::send::<BACKEND_DEV, BACKEND_FILE, BuildVerified>(),
            ),
        )
    };
    let build_recovery = g::seq(
        g::route(
            verified_frontend(),
            g::seq(
                g::seq(
                    g::send::<BUILD_MASTER, FRONTEND_FILE, Revise<FRONTEND_DEV>>(),
                    frontend(),
                )
                .roll(),
                verified_frontend(),
            ),
        ),
        g::route(
            verified_backend(),
            g::seq(
                g::seq(
                    g::send::<BUILD_MASTER, BACKEND_FILE, Revise<BACKEND_DEV>>(),
                    backend(),
                )
                .roll(),
                verified_backend(),
            ),
        ),
    );
    let qa = || {
        g::seq(
            g::send::<BUILD_MASTER, QA, WorkspaceLoan>(),
            g::seq(
                g::send::<BUILD_MASTER, REGISTRY_OWNER, QaSnapshot>(),
                g::seq(
                    g::send::<BUILD_MASTER, QA_FILE, ReadyForQa>(),
                    g::seq(
                        g::send::<QA_FILE, ARTIFACT_STORE, PullForQa>(),
                        g::seq(
                            g::send::<ARTIFACT_STORE, QA_FILE, QaPulled>(),
                            g::route(
                                g::seq(
                                    g::send::<QA_FILE, TEST_ENV, Integration>(),
                                    g::seq(
                                        g::send::<TEST_ENV, QA_FILE, Integrated>(),
                                        g::seq(
                                            g::send::<QA_FILE, TEST_ENV, E2e>(),
                                            g::send::<TEST_ENV, QA_FILE, E2eTested>(),
                                        ),
                                    ),
                                ),
                                g::seq(
                                    g::send::<QA_FILE, TEST_ENV, E2e>(),
                                    g::seq(
                                        g::send::<TEST_ENV, QA_FILE, E2eTested>(),
                                        g::seq(
                                            g::send::<QA_FILE, TEST_ENV, Integration>(),
                                            g::send::<TEST_ENV, QA_FILE, Integrated>(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let qa_passed = || {
        g::seq(
            g::send::<QA, BUILD_MASTER, QaPassed>(),
            g::send::<BUILD_MASTER, MASTER_FILE, QaReport>(),
        )
    };
    let qa_failed = || {
        g::seq(
            g::send::<QA, BUILD_MASTER, QaFailed>(),
            g::send::<BUILD_MASTER, MASTER_FILE, QaReport>(),
        )
    };
    let build_ended = || {
        g::par(
            g::seq(
                g::send::<BUILD_MASTER, FRONTEND_DEV, BuildEnded<FRONTEND_DEV>>(),
                g::send::<FRONTEND_DEV, FRONTEND_FILE, BuildEnded<FRONTEND_DEV>>(),
            ),
            g::seq(
                g::send::<BUILD_MASTER, BACKEND_DEV, BuildEnded<BACKEND_DEV>>(),
                g::send::<BACKEND_DEV, BACKEND_FILE, BuildEnded<BACKEND_DEV>>(),
            ),
        )
    };
    let repair = g::seq(
        g::seq(
            g::send::<MASTER_FILE, BUILD_MASTER, RepairDecision>(),
            g::seq(
                g::send::<BUILD_MASTER, QA, ReconsiderQa>(),
                g::send::<QA, BUILD_MASTER, WorkspaceLoan>(),
            ),
        ),
        g::seq(
            g::route(
                verified_frontend(),
                g::seq(
                    g::seq(
                        g::send::<BUILD_MASTER, FRONTEND_FILE, QaReviseFrontend>(),
                        frontend(),
                    )
                    .roll(),
                    verified_frontend(),
                ),
            ),
            g::route(
                verified_backend(),
                g::seq(
                    g::seq(
                        g::send::<BUILD_MASTER, BACKEND_FILE, QaReviseBackend>(),
                        backend(),
                    )
                    .roll(),
                    verified_backend(),
                ),
            ),
        ),
    );
    let staging = g::seq(
        g::send::<BUILD_MASTER, RELEASE_FILE, ReadyForRelease>(),
        g::seq(
            g::send::<RELEASE_FILE, ARTIFACT_STORE, PullForRelease>(),
            g::seq(
                g::send::<ARTIFACT_STORE, RELEASE_FILE, ReleasePulled>(),
                g::seq(
                    g::send::<QA, RELEASE, Loan<TEST_ENV>>(),
                    g::seq(
                        g::send::<RELEASE_FILE, TEST_ENV, Stage>(),
                        g::send::<TEST_ENV, RELEASE_FILE, Staged>(),
                    ),
                ),
            ),
        ),
    );
    let deployment = {
        g::seq(
            g::send::<RELEASE, PROD_OWNER, Promote>(),
            g::seq(
                g::send::<PROD_OWNER, RELEASE, Loan<PROD_ENV>>(),
                g::seq(
                    g::send::<RELEASE_FILE, PROD_ENV, Deploy>(),
                    g::seq(
                        g::send::<PROD_ENV, RELEASE_FILE, Deployed>(),
                        g::seq(
                            g::send::<RELEASE_FILE, PROD_ENV, SmokeProduction>(),
                            g::seq(
                                g::send::<PROD_ENV, RELEASE_FILE, ProductionSmoke>(),
                                g::seq(
                                    g::send::<RELEASE, PROD_OWNER, Loan<PROD_ENV>>(),
                                    g::send::<RELEASE, BUILD_MASTER, PipelineResult>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let abort = || {
        g::seq(
            g::send::<RELEASE, BUILD_MASTER, StagingAborted>(),
            g::seq(
                g::send::<BUILD_MASTER, RELEASE_FILE, StagingAborted>(),
                g::send::<RELEASE, PROD_OWNER, StagingAborted>(),
            ),
        )
    };
    let production = {
        g::route(
            g::seq(
                g::send::<RELEASE, BUILD_MASTER, StagingSmoke>(),
                g::seq(
                    g::send::<BUILD_MASTER, RELEASE_FILE, Promote>(),
                    g::seq(g::send::<BUILD_MASTER, RELEASE, Promote>(), deployment),
                ),
            ),
            abort(),
        )
    };
    let verify_staging = {
        g::route(
            g::seq(
                g::send::<RELEASE, RELEASE_FILE, StagingReady>(),
                g::seq(
                    g::send::<RELEASE_FILE, TEST_ENV, SmokeStaging>(),
                    g::seq(
                        g::send::<TEST_ENV, RELEASE_FILE, StagingResult>(),
                        g::seq(g::send::<RELEASE, QA, Loan<TEST_ENV>>(), production),
                    ),
                ),
            ),
            g::seq(g::send::<RELEASE, QA, Loan<TEST_ENV>>(), abort()),
        )
    };
    let approved = || {
        g::seq(
            build_ended(),
            g::seq(
                g::send::<BUILD_MASTER, QA, ReleaseApproved>(),
                g::seq(
                    g::send::<BUILD_MASTER, REGISTRY_OWNER, ReleaseApproved>(),
                    g::seq(
                        g::send::<BUILD_MASTER, RELEASE, ReleaseApproved>(),
                        g::seq(staging, verify_staging),
                    ),
                ),
            ),
        )
    };
    let denied = g::seq(
        build_ended(),
        g::seq(
            g::send::<BUILD_MASTER, QA, ReleaseDenied>(),
            g::seq(
                g::send::<BUILD_MASTER, REGISTRY_OWNER, ReleaseDenied>(),
                g::seq(
                    g::send::<BUILD_MASTER, RELEASE, ReleaseDenied>(),
                    g::seq(
                        g::send::<BUILD_MASTER, RELEASE_FILE, ReadyForRelease>(),
                        g::send::<RELEASE, PROD_OWNER, StagingAborted>(),
                    ),
                ),
            ),
        ),
    );
    let approved = || g::seq(approved(), g::send::<BUILD_MASTER, QA_FILE, QaFinished>());
    let denied = g::seq(denied, g::send::<BUILD_MASTER, QA_FILE, QaFinished>());
    let qa_and_recovery = g::seq(
        qa(),
        g::route(
            g::seq(qa_passed(), approved()),
            g::seq(
                qa_failed(),
                g::seq(
                    repair,
                    g::seq(
                        qa(),
                        g::route(g::seq(qa_passed(), approved()), g::seq(qa_failed(), denied)),
                    ),
                ),
            ),
        ),
    );
    let finished = g::par(
        g::send::<BUILD_MASTER, FRONTEND_FILE, FrontendFinished>(),
        g::par(
            g::send::<BUILD_MASTER, BACKEND_FILE, BackendFinished>(),
            g::par(
                g::send::<BUILD_MASTER, RELEASE_FILE, ReleaseFinished>(),
                g::send::<BUILD_MASTER, MASTER_FILE, FinalResult>(),
            ),
        ),
    );
    g::seq(
        builds,
        g::seq(build_recovery, g::seq(qa_and_recovery, finished)),
    )
}

pub fn with_shells() -> impl Projectable {
    use crate::shell::global::{Exit, FileExit, file_metadata, private_io};

    let files = g::par(
        g::send::<FRONTEND_DATA, FRONTEND_VM, Loan<FRONTEND_FILE>>(),
        g::par(
            g::send::<BACKEND_DATA, BACKEND_VM, Loan<BACKEND_FILE>>(),
            g::par(
                g::send::<QA_DATA, QA_VM, Loan<QA_FILE>>(),
                g::par(
                    g::send::<RELEASE_DATA, RELEASE_VM, Loan<RELEASE_FILE>>(),
                    g::send::<MASTER_DATA, MASTER_VM, Loan<MASTER_FILE>>(),
                ),
            ),
        ),
    );
    let frontend = g::par(
        g::seq(
            file_metadata::<FRONTEND_FILE, FRONTEND_DATA>().roll(),
            g::send::<FRONTEND_FILE, FRONTEND_DATA, FileExit>(),
        ),
        g::seq(
            private_io::<FRONTEND_VM, FRONTEND_IO>(),
            g::send::<FRONTEND_VM, FRONTEND_IO, Exit>(),
        ),
    );
    let backend = g::par(
        g::seq(
            file_metadata::<BACKEND_FILE, BACKEND_DATA>().roll(),
            g::send::<BACKEND_FILE, BACKEND_DATA, FileExit>(),
        ),
        g::seq(
            private_io::<BACKEND_VM, BACKEND_IO>(),
            g::send::<BACKEND_VM, BACKEND_IO, Exit>(),
        ),
    );
    let qa = g::par(
        g::seq(
            file_metadata::<QA_FILE, QA_DATA>().roll(),
            g::send::<QA_FILE, QA_DATA, FileExit>(),
        ),
        g::seq(
            private_io::<QA_VM, QA_IO>(),
            g::send::<QA_VM, QA_IO, Exit>(),
        ),
    );
    let release = g::par(
        g::seq(
            file_metadata::<RELEASE_FILE, RELEASE_DATA>().roll(),
            g::send::<RELEASE_FILE, RELEASE_DATA, FileExit>(),
        ),
        g::seq(
            private_io::<RELEASE_VM, RELEASE_IO>(),
            g::send::<RELEASE_VM, RELEASE_IO, Exit>(),
        ),
    );
    let master = g::par(
        g::seq(
            file_metadata::<MASTER_FILE, MASTER_DATA>().roll(),
            g::send::<MASTER_FILE, MASTER_DATA, FileExit>(),
        ),
        g::seq(
            private_io::<MASTER_VM, MASTER_IO>(),
            g::send::<MASTER_VM, MASTER_IO, Exit>(),
        ),
    );
    g::seq(
        files,
        g::par(
            pipeline(),
            g::par(
                frontend,
                g::par(backend, g::par(qa, g::par(release, master))),
            ),
        ),
    )
}
