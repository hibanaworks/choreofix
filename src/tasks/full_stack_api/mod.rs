mod arguments;
mod global;
mod localside;

use crate::{
    imp::projection,
    shell::{
        GUEST_MEMORY_BYTES, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    },
};
use global::*;
use hibana::{
    g::Message,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{RoleProgram, project},
    },
};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::json;

pub(crate) struct Programs {
    db_owner: RoleProgram<DB_OWNER>,
    api_owner: RoleProgram<API_OWNER>,
    database: RoleProgram<DATABASE>,
    gateway: RoleProgram<API_GATEWAY>,
    db_service: RoleProgram<DB_SERVICE>,
    api_service: RoleProgram<API_SERVICE>,
    backend_service: RoleProgram<BACKEND_SERVICE>,
    db_file: RoleProgram<DB_FILE>,
    backend_file: RoleProgram<BACKEND_FILE>,
    db_vm: RoleProgram<DB_VM>,
    backend_vm: RoleProgram<BACKEND_VM>,
    db_io: RoleProgram<DB_IO>,
    backend_io: RoleProgram<BACKEND_IO>,
    db_data: RoleProgram<DB_DATA>,
    backend_data: RoleProgram<BACKEND_DATA>,
    frontend_service: RoleProgram<FRONTEND_SERVICE>,
    review_service: RoleProgram<REVIEW_SERVICE>,
    test_owner: RoleProgram<TEST_OWNER>,
    test_env: RoleProgram<TEST_ENV>,
    test_service: RoleProgram<TEST_SERVICE>,
    review_file: RoleProgram<REVIEW_FILE>,
    review_vm: RoleProgram<REVIEW_VM>,
    review_io: RoleProgram<REVIEW_IO>,
    review_data: RoleProgram<REVIEW_DATA>,
    frontend_file: RoleProgram<FRONTEND_FILE>,
    frontend_vm: RoleProgram<FRONTEND_VM>,
    frontend_io: RoleProgram<FRONTEND_IO>,
    frontend_data: RoleProgram<FRONTEND_DATA>,
    test_file: RoleProgram<TEST_FILE>,
    test_vm: RoleProgram<TEST_VM>,
    test_io: RoleProgram<TEST_IO>,
    test_data: RoleProgram<TEST_DATA>,
}

pub(crate) fn programs() -> Programs {
    let source = protocol();
    Programs {
        db_owner: project(&source),
        api_owner: project(&source),
        database: project(&source),
        gateway: project(&source),
        db_service: project(&source),
        api_service: project(&source),
        backend_service: project(&source),
        db_file: project(&source),
        backend_file: project(&source),
        db_vm: project(&source),
        backend_vm: project(&source),
        db_io: project(&source),
        backend_io: project(&source),
        db_data: project(&source),
        backend_data: project(&source),
        frontend_service: project(&source),
        review_service: project(&source),
        test_owner: project(&source),
        test_env: project(&source),
        test_service: project(&source),
        review_file: project(&source),
        review_vm: project(&source),
        review_io: project(&source),
        review_data: project(&source),
        frontend_file: project(&source),
        frontend_vm: project(&source),
        frontend_io: project(&source),
        frontend_data: project(&source),
        test_file: project(&source),
        test_vm: project(&source),
        test_io: project(&source),
        test_data: project(&source),
    }
}

impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use projection::rows;

        let roles = [
            ("DB_OWNER", DB_OWNER),
            ("API_OWNER", API_OWNER),
            ("DATABASE", DATABASE),
            ("API_GATEWAY", API_GATEWAY),
            ("DB_SERVICE", DB_SERVICE),
            ("API_SERVICE", API_SERVICE),
            ("BACKEND_SERVICE", BACKEND_SERVICE),
            ("FRONTEND_SERVICE", FRONTEND_SERVICE),
            ("REVIEW_SERVICE", REVIEW_SERVICE),
            ("TEST_OWNER", TEST_OWNER),
            ("TEST_ENV", TEST_ENV),
            ("TEST_SERVICE", TEST_SERVICE),
            ("DB_FILE", DB_FILE),
            ("BACKEND_FILE", BACKEND_FILE),
            ("REVIEW_FILE", REVIEW_FILE),
            ("FRONTEND_FILE", FRONTEND_FILE),
            ("TEST_FILE", TEST_FILE),
            ("DB_VM", DB_VM),
            ("BACKEND_VM", BACKEND_VM),
            ("REVIEW_VM", REVIEW_VM),
            ("FRONTEND_VM", FRONTEND_VM),
            ("TEST_VM", TEST_VM),
            ("DB_IO", DB_IO),
            ("BACKEND_IO", BACKEND_IO),
            ("REVIEW_IO", REVIEW_IO),
            ("FRONTEND_IO", FRONTEND_IO),
            ("TEST_IO", TEST_IO),
            ("DB_DATA", DB_DATA),
            ("BACKEND_DATA", BACKEND_DATA),
            ("REVIEW_DATA", REVIEW_DATA),
            ("FRONTEND_DATA", FRONTEND_DATA),
            ("TEST_DATA", TEST_DATA),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        let labels = [
            ("Loan", Loan::<DATABASE>::LOGICAL_LABEL),
            ("MigrateSchema", MigrateSchema::LOGICAL_LABEL),
            ("SchemaMigrated", SchemaMigrated::LOGICAL_LABEL),
            ("SchemaReady", SchemaReady::LOGICAL_LABEL),
            ("DefineApi", DefineApi::LOGICAL_LABEL),
            ("ApiDefined", ApiDefined::LOGICAL_LABEL),
            ("ImplementBackend", ImplementBackend::LOGICAL_LABEL),
            ("BackendImplemented", BackendImplemented::LOGICAL_LABEL),
            ("BackendCode", BackendCode::LOGICAL_LABEL),
            ("ApiContract", ApiContract::LOGICAL_LABEL),
            ("BackendCandidate", BackendCandidate::LOGICAL_LABEL),
            ("ReviewBackend", ReviewBackend::LOGICAL_LABEL),
            ("BackendReviewed", BackendReviewed::LOGICAL_LABEL),
            ("BackendApproval", BackendApproval::LOGICAL_LABEL),
            ("UiInputs", UiInputs::LOGICAL_LABEL),
            ("ImplementFrontend", ImplementFrontend::LOGICAL_LABEL),
            ("FrontendImplemented", FrontendImplemented::LOGICAL_LABEL),
            ("UiCode", UiCode::LOGICAL_LABEL),
            ("UiCandidate", UiCandidate::LOGICAL_LABEL),
            ("ReviewFrontend", ReviewFrontend::LOGICAL_LABEL),
            ("FrontendReviewed", FrontendReviewed::LOGICAL_LABEL),
            ("TestInputs", TestInputs::LOGICAL_LABEL),
            ("RunIntegration", RunIntegration::LOGICAL_LABEL),
            ("IntegrationResult", IntegrationResult::LOGICAL_LABEL),
            ("IntegrationPassed", IntegrationPassed::LOGICAL_LABEL),
            ("IntegrationFailed", IntegrationFailed::LOGICAL_LABEL),
            ("BackendAccepted", BackendAccepted::LOGICAL_LABEL),
            ("BackendChanges", BackendChanges::LOGICAL_LABEL),
            ("UiChanges", UiChanges::LOGICAL_LABEL),
            ("UiAccepted", UiAccepted::LOGICAL_LABEL),
            ("UiChangesAfterTest", UiChangesAfterTest::LOGICAL_LABEL),
            ("RepairDecision", RepairDecision::LOGICAL_LABEL),
            ("IntegrationReport", IntegrationReport::LOGICAL_LABEL),
            ("UiRetained", UiRetained::LOGICAL_LABEL),
            ("BackendRetained", BackendRetained::LOGICAL_LABEL),
            ("BackendRepair", BackendRepair::LOGICAL_LABEL),
            (
                "BackendChangesAfterTest",
                BackendChangesAfterTest::LOGICAL_LABEL,
            ),
            (
                "BackendIntegrationPassed",
                BackendIntegrationPassed::LOGICAL_LABEL,
            ),
            ("ChangeApi", ChangeApi::LOGICAL_LABEL),
            ("ChangeSchema", ChangeSchema::LOGICAL_LABEL),
            ("ApiRetained", ApiRetained::LOGICAL_LABEL),
            ("SchemaRetained", SchemaRetained::LOGICAL_LABEL),
            ("UiInvalidated", UiInvalidated::LOGICAL_LABEL),
            (
                "BackendSchemaInvalidated",
                BackendSchemaInvalidated::LOGICAL_LABEL,
            ),
            ("ReleaseDatabase", ReleaseDatabase::LOGICAL_LABEL),
            ("ReleaseGateway", ReleaseGateway::LOGICAL_LABEL),
            ("NeedSchema", NeedSchema::LOGICAL_LABEL),
            ("RequestMigration", RequestMigration::LOGICAL_LABEL),
            ("SchemaInUse", SchemaInUse::LOGICAL_LABEL),
            ("SchemaStatus", SchemaStatus::LOGICAL_LABEL),
            ("DbOutcome", Outcome::<DB_FILE>::LOGICAL_LABEL),
            ("BackendOutcome", Outcome::<BACKEND_FILE>::LOGICAL_LABEL),
            ("ReviewOutcome", Outcome::<REVIEW_FILE>::LOGICAL_LABEL),
            ("UiOutcome", Outcome::<FRONTEND_FILE>::LOGICAL_LABEL),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();

        json!({
            "type": "compiled_projection",
            "task": "8M",
            "format_version": 1,
            "session": SESSION,
            "roles": roles,
            "labels": labels,
            "programs": [
                rows(&self.db_owner),
                rows(&self.api_owner),
                rows(&self.database),
                rows(&self.gateway),
                rows(&self.db_service),
                rows(&self.api_service),
                rows(&self.backend_service),
                rows(&self.db_file),
                rows(&self.backend_file),
                rows(&self.db_vm),
                rows(&self.backend_vm),
                rows(&self.db_io),
                rows(&self.backend_io),
                rows(&self.db_data),
                rows(&self.backend_data),
                rows(&self.frontend_service),
                rows(&self.review_service),
                rows(&self.test_owner),
                rows(&self.test_env),
                rows(&self.test_service),
                rows(&self.review_file),
                rows(&self.review_vm),
                rows(&self.review_io),
                rows(&self.review_data),
                rows(&self.frontend_file),
                rows(&self.frontend_vm),
                rows(&self.frontend_io),
                rows(&self.frontend_data),
                rows(&self.test_file),
                rows(&self.test_vm),
                rows(&self.test_io),
                rows(&self.test_data),
            ]
        })
    }
}

#[hibana_runtime::scope]
pub async fn run_stdio(wash: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        bus,
        mut io,
        [
            db_terminal,
            backend_terminal,
            review_terminal,
            frontend_terminal,
            test_terminal,
        ],
    ) = Bus::open(
        [
            "DB_ADMIN",
            "BACKEND_DEV",
            "REVIEWER",
            "FRONTEND_DEV",
            "TESTER",
        ],
        [
            DATABASE,
            API_GATEWAY,
            TEST_ENV,
            DB_FILE,
            BACKEND_FILE,
            REVIEW_FILE,
            FRONTEND_FILE,
            TEST_FILE,
        ]
        .map(|role| (session, role)),
    );
    let trial = async {
        let programs = programs();
        bus.observe(programs.observation());
        let Programs {
            db_owner,
            api_owner,
            database,
            gateway,
            db_service,
            api_service,
            backend_service,
            db_file,
            backend_file,
            db_vm,
            backend_vm,
            db_io,
            backend_io,
            db_data,
            backend_data,
            frontend_service,
            review_service,
            test_owner,
            test_env,
            test_service,
            review_file,
            review_vm,
            review_io,
            review_data,
            frontend_file,
            frontend_vm,
            frontend_io,
            frontend_data,
            test_file,
            test_vm,
            test_io,
            test_data,
        } = programs;
        let record = |event| {
            bus.observe(json!({
                "type":"protocol_capture", "producer":0, "records":[projection::record(event)]
            }));
        };
        let mut slab = vec![0; 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&record);
        let observation = rv.tap();
        let result = async {
            let mut db_owner = rv.enter(session, &db_owner)?;
            let mut api_owner = rv.enter(session, &api_owner)?;
            let database = rv.enter(session, &database)?;
            let gateway = rv.enter(session, &gateway)?;
            let mut db_service = rv.enter(session, &db_service)?;
            let mut api_service = rv.enter(session, &api_service)?;
            let mut backend_service = rv.enter(session, &backend_service)?;
            let mut db_data = rv.enter(session, &db_data)?;
            let mut backend_data = rv.enter(session, &backend_data)?;
            let db_file = rv.enter(session, &db_file)?;
            let backend_file = rv.enter(session, &backend_file)?;
            let mut db_vm = rv.enter(session, &db_vm)?;
            let mut backend_vm = rv.enter(session, &backend_vm)?;
            let mut db_io = rv.enter(session, &db_io)?;
            let mut backend_io = rv.enter(session, &backend_io)?;
            let mut frontend_service = rv.enter(session, &frontend_service)?;
            let mut review_service = rv.enter(session, &review_service)?;
            let mut test_owner = rv.enter(session, &test_owner)?;
            let test_env = rv.enter(session, &test_env)?;
            let mut test_service = rv.enter(session, &test_service)?;
            let mut review_data = rv.enter(session, &review_data)?;
            let review_file = rv.enter(session, &review_file)?;
            let mut review_vm = rv.enter(session, &review_vm)?;
            let mut review_io = rv.enter(session, &review_io)?;
            let mut frontend_data = rv.enter(session, &frontend_data)?;
            let frontend_file = rv.enter(session, &frontend_file)?;
            let mut frontend_vm = rv.enter(session, &frontend_vm)?;
            let mut frontend_io = rv.enter(session, &frontend_io)?;
            let mut test_data = rv.enter(session, &test_data)?;
            let test_file = rv.enter(session, &test_file)?;
            let mut test_vm = rv.enter(session, &test_vm)?;
            let mut test_io = rv.enter(session, &test_io)?;
            async {
                db_owner.send::<Loan<DATABASE>>(database).await?;
                drop(db_owner.recv::<Loan<DATABASE>>().await?);
            };
            async {
                api_owner.send::<Loan<API_GATEWAY>>(gateway).await?;
                drop(api_owner.recv::<Loan<API_GATEWAY>>().await?);
            };
            async {
                localside::migrate(&mut db_service, &bus).await?;
            };
            async {
                localside::define(&mut api_service, &bus).await?;
            };
            async {
                localside::implement(&mut backend_service, &bus).await?;
            };
            async {
                test_owner.send::<Loan<TEST_ENV>>(test_env).await?;
                loop {
                    let branch = test_owner.offer().await?;
                    match branch.label() {
                        IntegrationPassed::LOGICAL_LABEL => {
                            let report = branch.recv::<IntegrationPassed>().await?.to_vec();
                            test_owner
                                .send::<IntegrationReport>(&report.as_slice())
                                .await?;
                            drop(test_owner.recv::<Loan<TEST_ENV>>().await?);
                            let reply =
                                crate::shell::imp::review_reply::ReviewReply::from_bytes(&report)?;
                            async {
                                let [mut db, mut api] = test_owner.par()?;
                                async { db.send::<ReleaseDatabase>(&reply).await };
                                async { api.send::<ReleaseGateway>(&reply).await };
                            }
                            .await?;
                            break;
                        }
                        IntegrationFailed::LOGICAL_LABEL => {
                            let report = branch.recv::<IntegrationFailed>().await?.to_vec();
                            test_owner
                                .send::<IntegrationReport>(&report.as_slice())
                                .await?;
                        }
                        label => {
                            return Err(crate::shell::ShellError::message(format!(
                                "unexpected test environment report {label}"
                            )));
                        }
                    }
                }
            };
            async {
                localside::review(&mut review_service, &bus).await?;
            };
            async {
                localside::frontend(&mut frontend_service, &bus).await?;
            };
            async {
                localside::test(&mut test_service, &bus).await?;
            };
            async {
                file_metadata::run(&mut db_data, db_file, DB_FILES).await?;
            };
            async {
                file_metadata::run(&mut backend_data, backend_file, BACKEND_FILES).await?;
            };
            async {
                private_io::run(&mut db_io, PrivateFs::new(db_terminal, &bus, &DB_FILES)).await?;
            };
            async {
                private_io::run(
                    &mut backend_io,
                    PrivateFs::new(backend_terminal, &bus, &BACKEND_FILES),
                )
                .await?;
            };
            async {
                file_metadata::run(&mut review_data, review_file, REVIEW_FILES).await?;
            };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(review_terminal, &bus, &REVIEW_FILES),
                )
                .await?;
            };
            async {
                file_metadata::run(&mut frontend_data, frontend_file, FRONTEND_FILES).await?;
            };
            async {
                private_io::run(
                    &mut frontend_io,
                    PrivateFs::new(frontend_terminal, &bus, &FRONTEND_FILES),
                )
                .await?;
            };
            async {
                file_metadata::run(&mut test_data, test_file, TEST_FILES).await?;
            };
            async {
                private_io::run(
                    &mut test_io,
                    PrivateFs::new(test_terminal, &bus, &TEST_FILES),
                )
                .await?;
            };
            async {
                let file = db_vm.recv::<Loan<DB_FILE>>().await?;
                let mut write = vec![0; GUEST_MEMORY_BYTES];
                let mut result = vec![0; incoming_capacity];
                let mut outcome = vec![0; incoming_capacity];
                let mut schema_status = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DB_FILES[0], file)
                    .write_as::<MigrateSchema>(&mut write)
                    .read_as::<SchemaMigrated>(&mut result)
                    .file(DB_FILES[1])?
                    .read_as::<Outcome<DB_FILE>>(&mut outcome)
                    .file(DB_FILES[2])?
                    .read_as::<SchemaStatus>(&mut schema_status);
                let files = ChoreoFsFiles::new(db_vm).file(file)?;
                let mut memory = vec![0; GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(wash, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut vm, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                vm.send::<Exit>(&status).await?;
            };
            async {
                let file = backend_vm.recv::<Loan<BACKEND_FILE>>().await?;
                let mut write = vec![0; GUEST_MEMORY_BYTES];
                let mut result = vec![0; incoming_capacity];
                let mut schema = vec![0; incoming_capacity];
                let mut implementation_args = vec![0; GUEST_MEMORY_BYTES];
                let mut implementation_result = vec![0; incoming_capacity];
                let mut outcome = vec![0; incoming_capacity];
                let mut schema_request = [0; 512];
                let file = ChoreoFsFile::new(BACKEND_FILES[0], file)
                    .write_as::<DefineApi>(&mut write)
                    .read_as::<ApiDefined>(&mut result)
                    .file(BACKEND_FILES[1])?
                    .read_as::<SchemaReady>(&mut schema)
                    .file(BACKEND_FILES[2])?
                    .write_as::<ImplementBackend>(&mut implementation_args)
                    .read_as::<BackendImplemented>(&mut implementation_result)
                    .file(BACKEND_FILES[3])?
                    .read_as::<Outcome<BACKEND_FILE>>(&mut outcome)
                    .file(BACKEND_FILES[4])?
                    .write_as::<NeedSchema>(&mut schema_request);
                let files = ChoreoFsFiles::new(backend_vm).file(file)?;
                let mut memory = vec![0; GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(wash, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut vm, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                vm.send::<Exit>(&status).await?;
            };
            async {
                let file = review_vm.recv::<Loan<REVIEW_FILE>>().await?;
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let mut read_2 = vec![0; incoming_capacity];
                let mut write_3 = vec![0; GUEST_MEMORY_BYTES];
                let mut read_3 = vec![0; incoming_capacity];
                let mut read_4 = vec![0; incoming_capacity];
                let mut decision = [0; 64];
                let file = ChoreoFsFile::new(REVIEW_FILES[0], file)
                    .read_as::<BackendCandidate>(&mut read_0)
                    .file(REVIEW_FILES[1])?
                    .write_as::<ReviewBackend>(&mut write_1)
                    .read_as::<BackendReviewed>(&mut read_1)
                    .file(REVIEW_FILES[2])?
                    .read_as::<UiCandidate>(&mut read_2)
                    .file(REVIEW_FILES[3])?
                    .write_as::<ReviewFrontend>(&mut write_3)
                    .read_as::<FrontendReviewed>(&mut read_3)
                    .file(REVIEW_FILES[4])?
                    .read_as::<Outcome<REVIEW_FILE>>(&mut read_4)
                    .file(REVIEW_FILES[5])?
                    .write_as::<RepairDecision>(&mut decision);
                let files = ChoreoFsFiles::new(review_vm).file(file)?;
                let mut memory = vec![0; GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(wash, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut vm, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                vm.send::<Exit>(&status).await?;
            };
            async {
                let file = frontend_vm.recv::<Loan<FRONTEND_FILE>>().await?;
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let mut read_2 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(FRONTEND_FILES[0], file)
                    .read_as::<UiInputs>(&mut read_0)
                    .file(FRONTEND_FILES[1])?
                    .write_as::<ImplementFrontend>(&mut write_1)
                    .read_as::<FrontendImplemented>(&mut read_1)
                    .file(FRONTEND_FILES[2])?
                    .read_as::<Outcome<FRONTEND_FILE>>(&mut read_2);
                let files = ChoreoFsFiles::new(frontend_vm).file(file)?;
                let mut memory = vec![0; GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(wash, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut vm, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                vm.send::<Exit>(&status).await?;
            };
            async {
                let file = test_vm.recv::<Loan<TEST_FILE>>().await?;
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(TEST_FILES[0], file)
                    .read_as::<TestInputs>(&mut read_0)
                    .file(TEST_FILES[1])?
                    .write_as::<RunIntegration>(&mut write_1)
                    .read_as::<IntegrationResult>(&mut read_1);
                let files = ChoreoFsFiles::new(test_vm).file(file)?;
                let mut memory = vec![0; GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(wash, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut vm, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                vm.send::<Exit>(&status).await?;
            };
            Ok(())
        }
        .await;
        bus.observe(json!({
            "type":"protocol_checkpoint", "producer":0, "session":SESSION,
            "checkpoint":observation.checkpoint()
        }));
        result
    };
    let result = hibana_runtime::race(trial, io.run(&bus)).await;
    let report = match &result {
        Ok(()) => json!({"type":"run_result","status":"complete"}),
        Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
    };
    io.emit(&bus, report).await?;
    result
}
