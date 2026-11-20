//! Full-stack preparation, reviews and integrated testing.
use super::arguments::*;
use crate::shell::{
    global::{Exit, FileExit, Loan, file_metadata, private_io},
    imp::review_reply::ReviewReply,
};
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};

pub const SESSION: u32 = 82;
pub const DB_OWNER: u8 = 0;
pub const API_OWNER: u8 = 1;
pub const DATABASE: u8 = 2;
pub const API_GATEWAY: u8 = 3;
pub const DB_SERVICE: u8 = 4;
pub const API_SERVICE: u8 = 5;
pub const BACKEND_SERVICE: u8 = 6;
pub const FRONTEND_SERVICE: u8 = 7;
pub const REVIEW_SERVICE: u8 = 8;
pub const TEST_OWNER: u8 = 9;
pub const TEST_ENV: u8 = 10;
pub const TEST_SERVICE: u8 = 11;
pub const DB_FILE: u8 = 32;
pub const BACKEND_FILE: u8 = 33;
pub const REVIEW_FILE: u8 = 34;
pub const FRONTEND_FILE: u8 = 35;
pub const TEST_FILE: u8 = 36;
pub const DB_VM: u8 = 64;
pub const BACKEND_VM: u8 = 65;
pub const REVIEW_VM: u8 = 66;
pub const FRONTEND_VM: u8 = 67;
pub const TEST_VM: u8 = 68;
pub const DB_IO: u8 = 80;
pub const BACKEND_IO: u8 = 81;
pub const REVIEW_IO: u8 = 82;
pub const FRONTEND_IO: u8 = 83;
pub const TEST_IO: u8 = 84;
pub const DB_DATA: u8 = 96;
pub const BACKEND_DATA: u8 = 97;
pub const REVIEW_DATA: u8 = 98;
pub const FRONTEND_DATA: u8 = 99;
pub const TEST_DATA: u8 = 100;

pub type MigrateSchema = g::Msg<1, Arguments<MIGRATION>>;
pub type SchemaMigrated<'a> = g::Msg<2, &'a [u8]>;
pub type SchemaReady<'a> = g::Msg<3, &'a [u8]>;
pub type DefineApi = g::Msg<4, Arguments<DEFINITION>>;
pub type ApiDefined<'a> = g::Msg<5, &'a [u8]>;
pub type ImplementBackend = g::Msg<6, Arguments<IMPLEMENTATION>>;
pub type BackendImplemented<'a> = g::Msg<7, &'a [u8]>;

const OPERATION_IO: FdBinding = FdBinding {
    read: Some(FdReadRow::Base),
    write: Some(FdWriteRow::Object),
    readdir: None,
};

pub type BackendCode<'a> = g::Msg<8, &'a [u8]>;
pub type ApiContract<'a> = g::Msg<9, &'a [u8]>;
pub type BackendCandidate<'a> = g::Msg<10, &'a [u8]>;
pub type ReviewBackend = g::Msg<11, Arguments<BACKEND_REVIEW>>;
pub type BackendReviewed<'a> = g::Msg<12, &'a [u8]>;
pub type BackendApproval<'a> = g::Msg<13, &'a [u8]>;
pub type UiInputs<'a> = g::Msg<14, &'a [u8]>;
pub type ImplementFrontend = g::Msg<15, Arguments<FRONTEND>>;
pub type FrontendImplemented<'a> = g::Msg<16, &'a [u8]>;
pub type UiCode<'a> = g::Msg<17, &'a [u8]>;
pub type UiCandidate<'a> = g::Msg<18, &'a [u8]>;
pub type ReviewFrontend = g::Msg<19, Arguments<FRONTEND_REVIEW>>;
pub type FrontendReviewed<'a> = g::Msg<20, &'a [u8]>;
pub type TestInputs<'a> = g::Msg<21, &'a [u8]>;
pub type RunIntegration = g::Msg<22, Arguments<INTEGRATION>>;
pub type IntegrationResult<'a> = g::Msg<23, &'a [u8]>;
pub type IntegrationPassed<'a> = g::Msg<24, ReviewReply<'a, true>>;
pub type IntegrationFailed<'a> = g::Msg<25, ReviewReply<'a, false>>;
pub type BackendAccepted<'a> = g::Msg<26, ReviewReply<'a, true>>;
pub type BackendChanges<'a> = g::Msg<27, ReviewReply<'a, false>>;
pub type UiChanges<'a> = g::Msg<28, ReviewReply<'a, false>>;
pub type UiAccepted<'a> = g::Msg<38, ReviewReply<'a, true>>;
pub type UiChangesAfterTest<'a> = g::Msg<39, ReviewReply<'a, false>>;
pub type RepairDecision = g::Msg<40, Arguments<REPAIR>>;
pub type IntegrationReport<'a> = g::Msg<41, &'a [u8]>;
pub type UiRetained<'a> = g::Msg<42, ReviewReply<'a, false>>;
pub type BackendRetained<'a> = g::Msg<43, ReviewReply<'a, false>>;
pub type BackendRepair<'a> = g::Msg<44, ReviewReply<'a, false>>;
pub type BackendChangesAfterTest<'a> = g::Msg<45, ReviewReply<'a, false>>;
pub type BackendIntegrationPassed<'a> = g::Msg<46, ReviewReply<'a, true>>;
pub type ChangeApi<'a> = g::Msg<48, ReviewReply<'a, false>>;
pub type ChangeSchema<'a> = g::Msg<49, ReviewReply<'a, false>>;
pub type ApiRetained<'a> = g::Msg<50, ReviewReply<'a, false>>;
pub type SchemaRetained<'a> = g::Msg<51, ReviewReply<'a, false>>;
pub type UiInvalidated<'a> = g::Msg<52, ReviewReply<'a, false>>;
pub type BackendSchemaInvalidated<'a> = g::Msg<53, ReviewReply<'a, false>>;
pub type ReleaseDatabase<'a> = g::Msg<54, ReviewReply<'a, true>>;
pub type ReleaseGateway<'a> = g::Msg<55, ReviewReply<'a, true>>;
pub type NeedSchema = g::Msg<56, Arguments<SCHEMA_REQUEST>>;
pub type RequestMigration<'a> = g::Msg<57, &'a [u8]>;
pub type SchemaInUse<'a> = g::Msg<58, &'a [u8]>;
pub type SchemaStatus<'a> = g::Msg<59, &'a [u8]>;
pub type Outcome<'a, const FILE: u8> = g::Msg<FILE, &'a [u8]>;

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        OPERATION_IO,
    )
}

const fn incoming(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}

pub const DB_FILES: [ChoreoFsObject<'static>; 3] = [
    operation(b"service/migrate_schema", 4),
    incoming(b"service/outcome", 5),
    incoming(b"service/schema_status", 6),
];
pub const BACKEND_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/define_api", 4),
    incoming(b"service/schema", 5),
    operation(b"service/implement_backend", 6),
    incoming(b"service/outcome", 7),
    ChoreoFsObject::writable(
        b"service/request_migration",
        ObjectId(8),
        FdSpec::new(8, 64),
        FdBinding::write(FdWriteRow::Object),
    ),
];
pub const REVIEW_FILES: [ChoreoFsObject<'static>; 6] = [
    incoming(b"service/backend_candidate", 4),
    operation(b"service/code_review/backend_api", 5),
    incoming(b"service/frontend_candidate", 6),
    operation(b"service/code_review/frontend_ui", 7),
    incoming(b"service/outcome", 8),
    ChoreoFsObject::writable(
        b"service/repair",
        ObjectId(9),
        FdSpec::new(9, 64),
        FdBinding::write(FdWriteRow::Object),
    ),
];
pub const FRONTEND_FILES: [ChoreoFsObject<'static>; 3] = [
    incoming(b"service/backend", 4),
    operation(b"service/implement_frontend", 5),
    incoming(b"service/outcome", 6),
];
pub const TEST_FILES: [ChoreoFsObject<'static>; 2] = [
    incoming(b"service/test_candidate", 4),
    operation(b"service/run_integration_tests", 5),
];

pub fn preparation() -> impl Projectable {
    let migration = g::seq(g::send::<DB_OWNER, DB_SERVICE, Loan<DATABASE>>(), migrate());
    let definition = g::seq(
        g::send::<API_OWNER, API_SERVICE, Loan<API_GATEWAY>>(),
        define(),
    );
    let implementation = || {
        g::seq(
            backend_code(),
            g::seq(
                g::send::<BACKEND_SERVICE, DB_SERVICE, SchemaInUse>(),
                g::send::<DB_SERVICE, DB_FILE, SchemaStatus>(),
            ),
        )
    };
    g::par(
        g::seq(
            migration,
            g::seq(
                g::send::<BACKEND_SERVICE, BACKEND_FILE, SchemaReady>(),
                g::route(
                    implementation(),
                    g::seq(
                        g::send::<BACKEND_FILE, BACKEND_SERVICE, NeedSchema>(),
                        g::seq(
                            g::send::<BACKEND_SERVICE, DB_SERVICE, RequestMigration>(),
                            g::seq(
                                g::send::<DB_SERVICE, DB_FILE, SchemaStatus>(),
                                g::seq(
                                    migrate(),
                                    g::seq(
                                        g::send::<BACKEND_SERVICE, BACKEND_FILE, SchemaReady>(),
                                        implementation(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        definition,
    )
}

fn migrate() -> impl Projectable {
    g::seq(
        g::send::<DB_FILE, DATABASE, MigrateSchema>(),
        g::par(
            g::send::<DATABASE, DB_FILE, SchemaMigrated>(),
            g::send::<DATABASE, BACKEND_SERVICE, SchemaReady>(),
        ),
    )
}

fn define() -> impl Projectable {
    g::seq(
        g::send::<BACKEND_FILE, API_GATEWAY, DefineApi>(),
        g::par(
            g::send::<API_GATEWAY, BACKEND_FILE, ApiDefined>(),
            g::send::<API_GATEWAY, REVIEW_SERVICE, ApiContract>(),
        ),
    )
}

fn implement_against_schema() -> impl Projectable {
    g::par(
        g::send::<BACKEND_SERVICE, BACKEND_FILE, SchemaReady>(),
        backend_code(),
    )
}

fn backend_code() -> impl Projectable {
    g::seq(
        g::send::<BACKEND_FILE, BACKEND_SERVICE, ImplementBackend>(),
        g::par(
            g::send::<BACKEND_SERVICE, BACKEND_FILE, BackendImplemented>(),
            g::send::<BACKEND_SERVICE, REVIEW_SERVICE, BackendCode>(),
        ),
    )
}

fn backend_review_request() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_SERVICE, REVIEW_FILE, BackendCandidate>(),
        g::send::<REVIEW_FILE, REVIEW_SERVICE, ReviewBackend>(),
    )
}

fn backend_approved() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendAccepted>(),
        g::par(
            g::send::<REVIEW_SERVICE, REVIEW_FILE, BackendReviewed>(),
            g::par(
                g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, BackendApproval>(),
            ),
        ),
    )
}

fn complete_backend_review(rejected: impl Projectable) -> impl Projectable {
    g::seq(
        backend_review_request(),
        g::route(
            backend_approved(),
            g::seq(
                g::seq(
                    g::seq(
                        rejected,
                        g::par(
                            g::send::<REVIEW_SERVICE, REVIEW_FILE, BackendReviewed>(),
                            g::seq(
                                g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                                backend_code(),
                            ),
                        ),
                    ),
                    backend_review_request(),
                )
                .roll(),
                backend_approved(),
            ),
        ),
    )
}

fn approval_and_testing() -> impl Projectable {
    let backend_review =
        complete_backend_review(g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendChanges>());
    let tested = || {
        let reports = g::seq(
            g::seq(
                g::send::<TEST_ENV, TEST_OWNER, IntegrationPassed>(),
                g::par(
                    g::send::<TEST_OWNER, REVIEW_SERVICE, IntegrationReport>(),
                    g::send::<TEST_ENV, TEST_FILE, IntegrationResult>(),
                ),
            ),
            g::par(
                g::send::<REVIEW_SERVICE, DB_FILE, Outcome<DB_FILE>>(),
                g::par(
                    g::send::<REVIEW_SERVICE, REVIEW_FILE, Outcome<REVIEW_FILE>>(),
                    g::par(
                        g::seq(
                            g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, IntegrationPassed>(),
                            g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
                        ),
                        g::seq(
                            g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendIntegrationPassed>(),
                            g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                        ),
                    ),
                ),
            ),
        );
        g::seq(
            reports,
            g::seq(
                g::send::<TEST_SERVICE, TEST_OWNER, Loan<TEST_ENV>>(),
                g::par(
                    g::seq(
                        g::send::<TEST_OWNER, DB_SERVICE, ReleaseDatabase>(),
                        g::send::<DB_SERVICE, DB_OWNER, Loan<DATABASE>>(),
                    ),
                    g::seq(
                        g::send::<TEST_OWNER, API_SERVICE, ReleaseGateway>(),
                        g::send::<API_SERVICE, API_OWNER, Loan<API_GATEWAY>>(),
                    ),
                ),
            ),
        )
    };
    let rework = g::seq(
        g::seq(
            g::send::<TEST_ENV, TEST_OWNER, IntegrationFailed>(),
            g::par(
                g::send::<TEST_OWNER, REVIEW_SERVICE, IntegrationReport>(),
                g::send::<TEST_ENV, TEST_FILE, IntegrationResult>(),
            ),
        ),
        g::seq(
            g::send::<REVIEW_SERVICE, REVIEW_FILE, Outcome<REVIEW_FILE>>(),
            g::seq(
                g::send::<REVIEW_FILE, REVIEW_SERVICE, RepairDecision>(),
                g::seq(repair(), test_approved_pair()),
            ),
        ),
    );
    let attempts = g::seq(
        g::seq(review_ui::<UiChanges>(), test_approved_pair()),
        g::route(tested(), g::seq(rework.roll(), tested())),
    );
    g::seq(
        backend_review,
        g::seq(
            g::send::<TEST_OWNER, TEST_SERVICE, Loan<TEST_ENV>>(),
            g::seq(
                g::send::<FRONTEND_SERVICE, FRONTEND_FILE, UiInputs>(),
                attempts,
            ),
        ),
    )
}

fn ui_review_request() -> impl Projectable {
    g::seq(
        g::send::<FRONTEND_FILE, FRONTEND_SERVICE, ImplementFrontend>(),
        g::seq(
            g::par(
                g::send::<FRONTEND_SERVICE, FRONTEND_FILE, FrontendImplemented>(),
                g::send::<FRONTEND_SERVICE, REVIEW_SERVICE, UiCode>(),
            ),
            g::seq(
                g::send::<REVIEW_SERVICE, REVIEW_FILE, UiCandidate>(),
                g::send::<REVIEW_FILE, REVIEW_SERVICE, ReviewFrontend>(),
            ),
        ),
    )
}

fn review_ui<Changes: g::Message>() -> impl Projectable {
    let approved = g::seq(
        g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, UiAccepted>(),
        g::par(
            g::send::<REVIEW_SERVICE, REVIEW_FILE, FrontendReviewed>(),
            g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
        ),
    );
    let revise = g::seq(
        g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, Changes>(),
        g::par(
            g::send::<REVIEW_SERVICE, REVIEW_FILE, FrontendReviewed>(),
            g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
        ),
    );
    g::seq(
        ui_review_request(),
        g::route(
            approved,
            g::seq(g::seq(revise, ui_review_request()).roll(), approved),
        ),
    )
}

fn test_approved_pair() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_SERVICE, TEST_ENV, TestInputs>(),
        g::seq(
            g::send::<TEST_ENV, TEST_FILE, TestInputs>(),
            g::send::<TEST_FILE, TEST_ENV, RunIntegration>(),
        ),
    )
}

fn repair() -> impl Projectable {
    let ui = g::seq(
        g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, IntegrationFailed>(),
        g::seq(
            g::par(
                g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
                g::seq(
                    g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendRetained>(),
                    g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                ),
            ),
            review_ui::<UiChangesAfterTest>(),
        ),
    );
    let backend = g::seq(
        g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendRepair>(),
        g::seq(
            g::par(
                g::seq(
                    g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, UiRetained>(),
                    g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
                ),
                g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
            ),
            g::seq(
                backend_code(),
                complete_backend_review(g::send::<
                    REVIEW_SERVICE,
                    BACKEND_SERVICE,
                    BackendChangesAfterTest,
                >()),
            ),
        ),
    );
    let api = g::seq(
        g::send::<REVIEW_SERVICE, API_SERVICE, ChangeApi>(),
        g::seq(
            g::par(
                g::seq(
                    g::send::<REVIEW_SERVICE, DB_SERVICE, SchemaRetained>(),
                    g::send::<REVIEW_SERVICE, DB_FILE, Outcome<DB_FILE>>(),
                ),
                g::par(
                    g::seq(
                        g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, UiInvalidated>(),
                        g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
                    ),
                    g::seq(
                        g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendRetained>(),
                        g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                    ),
                ),
            ),
            define(),
        ),
    );
    let schema = g::seq(
        g::send::<REVIEW_SERVICE, DB_SERVICE, ChangeSchema>(),
        g::seq(
            g::par(
                g::par(
                    g::send::<REVIEW_SERVICE, API_SERVICE, ApiRetained>(),
                    g::send::<REVIEW_SERVICE, DB_FILE, Outcome<DB_FILE>>(),
                ),
                g::par(
                    g::seq(
                        g::send::<REVIEW_SERVICE, FRONTEND_SERVICE, UiInvalidated>(),
                        g::send::<REVIEW_SERVICE, FRONTEND_FILE, Outcome<FRONTEND_FILE>>(),
                    ),
                    g::seq(
                        g::send::<REVIEW_SERVICE, BACKEND_SERVICE, BackendSchemaInvalidated>(),
                        g::send::<REVIEW_SERVICE, BACKEND_FILE, Outcome<BACKEND_FILE>>(),
                    ),
                ),
            ),
            g::seq(migrate(), implement_against_schema()),
        ),
    );
    g::route(
        g::seq(
            g::send::<REVIEW_SERVICE, DB_SERVICE, SchemaRetained>(),
            g::seq(
                g::par(
                    g::send::<REVIEW_SERVICE, API_SERVICE, ApiRetained>(),
                    g::send::<REVIEW_SERVICE, DB_FILE, Outcome<DB_FILE>>(),
                ),
                g::route(ui, backend),
            ),
        ),
        g::seq(
            g::route(api, schema),
            g::seq(
                complete_backend_review(g::send::<
                    REVIEW_SERVICE,
                    BACKEND_SERVICE,
                    BackendChangesAfterTest,
                >()),
                g::seq(
                    g::send::<FRONTEND_SERVICE, FRONTEND_FILE, UiInputs>(),
                    review_ui::<UiChangesAfterTest>(),
                ),
            ),
        ),
    )
}

fn shell_io<const VM: u8, const IO: u8, const FILE: u8, const DATA: u8>() -> impl Projectable {
    g::par(
        g::seq(private_io::<VM, IO>(), g::send::<VM, IO, Exit>()),
        g::seq(
            file_metadata::<FILE, DATA>().roll(),
            g::send::<FILE, DATA, FileExit>(),
        ),
    )
}

pub fn protocol() -> impl Projectable {
    let files = g::par(
        g::par(
            g::send::<DB_DATA, DB_VM, Loan<DB_FILE>>(),
            g::send::<BACKEND_DATA, BACKEND_VM, Loan<BACKEND_FILE>>(),
        ),
        g::par(
            g::send::<REVIEW_DATA, REVIEW_VM, Loan<REVIEW_FILE>>(),
            g::par(
                g::send::<FRONTEND_DATA, FRONTEND_VM, Loan<FRONTEND_FILE>>(),
                g::send::<TEST_DATA, TEST_VM, Loan<TEST_FILE>>(),
            ),
        ),
    );
    let io = g::par(
        g::par(
            shell_io::<DB_VM, DB_IO, DB_FILE, DB_DATA>(),
            shell_io::<BACKEND_VM, BACKEND_IO, BACKEND_FILE, BACKEND_DATA>(),
        ),
        g::par(
            shell_io::<REVIEW_VM, REVIEW_IO, REVIEW_FILE, REVIEW_DATA>(),
            g::par(
                shell_io::<FRONTEND_VM, FRONTEND_IO, FRONTEND_FILE, FRONTEND_DATA>(),
                shell_io::<TEST_VM, TEST_IO, TEST_FILE, TEST_DATA>(),
            ),
        ),
    );
    g::seq(
        files,
        g::par(g::seq(preparation(), approval_and_testing()), io),
    )
}
