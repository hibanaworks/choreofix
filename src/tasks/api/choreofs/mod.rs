//! The three actual guests, services, resources and result objects share one global.
pub mod global;
mod imp;
mod localside;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::PrivateFs,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    (bus, simulator): (&Bus, &impl crate::shell::imp::service::ReceivedEffects),
    terminals: [Terminal; 3],
) -> ShellResult<()> {
    use global::*;
    let p = programs();
    bus.projected(p.observation());
    let result = super::imp::TestInput::default();
    let capture = |event| {
        bus.capture(event);
    };
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(28);
    let result = async {
        rv.set_resolver(
            &p.tester_close,
            ResolverRef::<TEST_RESULT>::decision_state(&result, super::imp::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut admin = rv.enter(session, &p.admin)?;
        let mut backend = rv.enter(session, &p.backend)?;
        let mut tester = rv.enter(session, &p.tester)?;
        let mut database = rv.enter(session, &p.database)?;
        let mut test_env = rv.enter(session, &p.test_env)?;
        let database_resource = rv.enter(session, &p.database_resource)?;
        let test_resource = rv.enter(session, &p.test_resource)?;
        let admin_vm = rv.enter(session, &p.admin_vm)?;
        let backend_vm = rv.enter(session, &p.backend_vm)?;
        let tester_vm = rv.enter(session, &p.tester_vm)?;
        let mut admin_io = rv.enter(session, &p.admin_io)?;
        let mut backend_io = rv.enter(session, &p.backend_io)?;
        let mut tester_io = rv.enter(session, &p.tester_io)?;
        let mut admin_outcome = rv.enter(session, &p.admin_outcome)?;
        let mut backend_outcome = rv.enter(session, &p.backend_outcome)?;
        let admin_outcome_file = rv.enter(session, &p.admin_outcome_file)?;
        let backend_outcome_file = rv.enter(session, &p.backend_outcome_file)?;
        let admin_file = rv.enter(session, &p.admin_file)?;
        let backend_file = rv.enter(session, &p.backend_file)?;
        let tester_file = rv.enter(session, &p.tester_file)?;
        let mut admin_close = rv.enter(session, &p.admin_close)?;
        let mut backend_close = rv.enter(session, &p.backend_close)?;
        let mut tester_close = rv.enter(session, &p.tester_close)?;
        let admin_read = rv.enter(session, &p.admin_outcome_read)?;
        let backend_read = rv.enter(session, &p.backend_outcome_read)?;
        let [bterm, aterm, tterm] = terminals;
        async {
            localside::file::run(&mut admin, admin_file, ADMIN_OBJECT).await?;
        };
        async {
            localside::file::run(&mut backend, backend_file, BACKEND_OBJECT).await?;
        };
        async {
            localside::file::run(&mut tester, tester_file, TESTER_OBJECT).await?;
        };
        async {
            localside::admin::run(&mut admin_close, simulator).await?;
        };
        async {
            localside::backend::run(&mut backend_close, simulator).await?;
        };
        async {
            localside::tester::run(
                &mut tester_close,
                (admin_read, backend_read),
                (simulator, &result),
            )
            .await?;
        };
        async {
            localside::resources::database(&mut database, database_resource).await?;
        };
        async {
            localside::resources::test_environment(&mut test_env, test_resource).await?;
        };
        async {
            localside::outcome::admin(&mut admin_outcome, admin_outcome_file).await?;
        };
        async {
            localside::outcome::backend(&mut backend_outcome, backend_outcome_file).await?;
        };
        async {
            let mut endpoint = admin_vm;
            let (service, outcome) = async {
                let [mut service, mut outcome] = endpoint.par()?;
                async { service.recv::<Loan<ADMIN_FILE>>().await };
                async { outcome.recv::<Loan<ADMIN_OUTCOME_FILE>>().await };
            }
            .await?;
            let mut submitted = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(ADMIN_OBJECT, service)
                        .write_as::<MigrationIntent>(&mut submitted)
                        .close_with::<FileApplied>(),
                )?
                .file(ChoreoFsFile::new(OUTCOME_OBJECT, outcome))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (outcome, (service, ()))) = files.into_parts();
            let mut service = service.into_endpoint();
            let mut outcome = outcome.into_endpoint();
            outcome.send::<FileExit>(&status).await?;
            service.send::<FileExit>(&status).await?;
            async {
                let [mut service_return, mut outcome_return] = endpoint.par()?;
                async { service_return.send::<Loan<ADMIN_FILE>>(service).await };
                async {
                    outcome_return
                        .send::<Loan<ADMIN_OUTCOME_FILE>>(outcome)
                        .await
                };
            }
            .await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut admin_io,
                PrivateFs::new(aterm, bus, &[ADMIN_OBJECT, OUTCOME_OBJECT]),
            )
            .await?;
        };
        async {
            let mut endpoint = backend_vm;
            let (service, outcome) = async {
                let [mut service, mut outcome] = endpoint.par()?;
                async { service.recv::<Loan<BACKEND_FILE>>().await };
                async { outcome.recv::<Loan<BACKEND_OUTCOME_FILE>>().await };
            }
            .await?;
            let mut submitted = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(BACKEND_OBJECT, service)
                        .write_as::<BackendRequest>(&mut submitted)
                        .close_with::<FileApplied>(),
                )?
                .file(ChoreoFsFile::new(OUTCOME_OBJECT, outcome))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (outcome, (service, ()))) = files.into_parts();
            let mut service = service.into_endpoint();
            let mut outcome = outcome.into_endpoint();
            outcome.send::<FileExit>(&status).await?;
            service.send::<FileExit>(&status).await?;
            async {
                let [mut service_return, mut outcome_return] = endpoint.par()?;
                async { service_return.send::<Loan<BACKEND_FILE>>(service).await };
                async {
                    outcome_return
                        .send::<Loan<BACKEND_OUTCOME_FILE>>(outcome)
                        .await
                };
            }
            .await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut backend_io,
                PrivateFs::new(bterm, bus, &[BACKEND_OBJECT, OUTCOME_OBJECT]),
            )
            .await?;
        };
        async {
            let mut endpoint = tester_vm;
            let service = endpoint.recv::<Loan<TESTER_FILE>>().await?;
            let mut submitted = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint).file(
                ChoreoFsFile::new(TESTER_OBJECT, service)
                    .write_as::<TestIntent>(&mut submitted)
                    .close_with::<FileApplied>(),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (service, ())) = files.into_parts();
            let mut service = service.into_endpoint();
            service.send::<FileExit>(&status).await?;
            endpoint.send::<Loan<TESTER_FILE>>(service).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut tester_io, PrivateFs::new(tterm, bus, &[TESTER_OBJECT])).await?;
        };
        Ok(())
    }
    .await;
    bus.checkpoint(session.raw(), observations.checkpoint());
    result
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(28);
    let (mut connection, mut reader, terminals) = Bus::open(
        ["BACKEND_DEV", "DB_ADMIN", "TESTER"],
        [
            (session, global::DATABASE_RESOURCE),
            (session, global::TEST_RESOURCE),
            (session, global::ADMIN_FILE),
            (session, global::BACKEND_FILE),
            (session, global::TESTER_FILE),
            (session, global::ADMIN_OUTCOME_FILE),
            (session, global::BACKEND_OUTCOME_FILE),
            (session, global::ADMIN_OUTCOME_READ),
            (session, global::BACKEND_OUTCOME_READ),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        run_guests(module, (bus, &simulator), terminals),
        reader.run(bus),
    )
    .await;
    reader.finish(bus, result, None).await
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
