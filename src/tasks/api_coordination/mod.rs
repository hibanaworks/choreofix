//! Real guests directly own their file projections and select their operations.
pub mod global;
mod imp;
pub mod localside;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::PrivateFs,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    (bus, simulator): (&Bus, &impl crate::shell::imp::service::PairedEffects),
    terminals: [Terminal; 3],
) -> ShellResult<()> {
    use global::*;
    let p = programs();
    bus.projected(p.observation());
    let capture = |event| bus.capture(event);
    let mut slab = vec![0; 2 * 1024 * 1024];
    // A decoded value cannot be larger than this run's receive workspace.
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let result = async {
        let mut store = rv.enter(session, &p.store)?;
        let backend_vm = rv.enter(session, &p.backend_vm)?;
        let frontend_vm = rv.enter(session, &p.frontend_vm)?;
        let tester_vm = rv.enter(session, &p.tester_vm)?;
        let mut backend_io = rv.enter(session, &p.backend_io)?;
        let mut frontend_io = rv.enter(session, &p.frontend_io)?;
        let mut tester_io = rv.enter(session, &p.tester_io)?;
        let mut define_owner = rv.enter(session, &p.define_owner)?;
        let define_close = rv.enter(session, &p.define_close)?;
        let define_file = rv.enter(session, &p.define_file)?;
        let mut backend_owner = rv.enter(session, &p.backend_owner)?;
        let backend_close = rv.enter(session, &p.backend_close)?;
        let backend_file = rv.enter(session, &p.backend_file)?;
        let mut frontend_owner = rv.enter(session, &p.frontend_owner)?;
        let frontend_close = rv.enter(session, &p.frontend_close)?;
        let frontend_file = rv.enter(session, &p.frontend_file)?;
        let mut agreement_owner = rv.enter(session, &p.agreement_owner)?;
        let agreement_close = rv.enter(session, &p.agreement_close)?;
        let agreement_file = rv.enter(session, &p.agreement_file)?;
        let mut tester_owner = rv.enter(session, &p.tester_owner)?;
        let tester_close = rv.enter(session, &p.tester_close)?;
        let tester_file = rv.enter(session, &p.tester_file)?;
        let repo = rv.enter(session, &p.repo)?;
        let environment = rv.enter(session, &p.test_env)?;
        let mut backend_outcome = rv.enter(session, &p.backend_outcome)?;
        let backend_outcome_file = rv.enter(session, &p.backend_outcome_file)?;
        let mut frontend_outcome = rv.enter(session, &p.frontend_outcome)?;
        let frontend_outcome_file = rv.enter(session, &p.frontend_outcome_file)?;
        let [backend_terminal, frontend_terminal, tester_terminal] = terminals;
        async {
            localside::definition::run(&mut define_owner, define_file).await?;
        };
        async {
            localside::backend::run(&mut backend_owner, backend_file).await?;
        };
        async {
            localside::frontend::run(&mut frontend_owner, frontend_file).await?;
        };
        async {
            localside::agreement::run(&mut agreement_owner, agreement_file).await?;
        };
        async {
            localside::tester::run(&mut tester_owner, tester_file).await?;
        };
        async {
            localside::resources::repository(
                &mut store,
                (
                    repo,
                    environment,
                    (
                        define_close,
                        backend_close,
                        frontend_close,
                        agreement_close,
                        tester_close,
                    ),
                ),
                simulator,
            )
            .await?;
        };
        async {
            localside::outcome::backend(&mut backend_outcome, backend_outcome_file).await?;
        };
        async {
            localside::outcome::frontend(&mut frontend_outcome, frontend_outcome_file).await?;
        };
        async {
            let mut endpoint = backend_vm;
            let (define, backend, outcome) = async {
                let [mut d, mut b, mut o] = endpoint.par()?;
                async { d.recv::<Loan<DEFINE_FILE>>().await };
                async { b.recv::<Loan<BACKEND_FILE>>().await };
                async { o.recv::<Loan<BACKEND_OUTCOME_FILE>>().await };
            }
            .await?;
            let mut notice = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(DEFINE_OBJECT, define))?
                .file(ChoreoFsFile::new(BACKEND_OBJECT, backend))?
                .file(
                    ChoreoFsFile::new(BACKEND_OUTCOME_OBJECT, outcome)
                        .read_as::<BackendResult>(&mut notice),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (outcome, (backend, (define, ())))) = files.into_parts();
            let mut define = define.into_endpoint();
            let mut backend = backend.into_endpoint();
            let mut outcome = outcome.into_endpoint();
            async {
                async {
                    define.send::<FileExit>(&status).await?;
                };
                async {
                    backend.send::<FileExit>(&status).await?;
                };
                async {
                    outcome.send::<FileExit>(&status).await?;
                };
                Ok(())
            }
            .await?;
            endpoint.send::<BackendExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut backend_io,
                PrivateFs::new(
                    backend_terminal,
                    bus,
                    &[DEFINE_OBJECT, BACKEND_OBJECT, BACKEND_OUTCOME_OBJECT],
                ),
            )
            .await?;
        };
        async {
            let mut endpoint = frontend_vm;
            let (frontend, agreement, outcome) = async {
                let [mut f, mut a, mut o] = endpoint.par()?;
                async { f.recv::<Loan<FRONTEND_FILE>>().await };
                async { a.recv::<Loan<AGREEMENT_FILE>>().await };
                async { o.recv::<Loan<FRONTEND_OUTCOME_FILE>>().await };
            }
            .await?;
            let mut notice = vec![0; incoming_capacity];
            let mut contract = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(FRONTEND_OBJECT, frontend))?
                .file(
                    ChoreoFsFile::new(AGREEMENT_OBJECT, agreement)
                        .read_as::<Contract>(&mut contract),
                )?
                .file(
                    ChoreoFsFile::new(FRONTEND_OUTCOME_OBJECT, outcome)
                        .read_as::<FrontendResult>(&mut notice),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (outcome, (agreement, (frontend, ())))) = files.into_parts();
            let mut frontend = frontend.into_endpoint();
            let mut agreement = agreement.into_endpoint();
            let mut outcome = outcome.into_endpoint();
            async {
                async {
                    frontend.send::<FileExit>(&status).await?;
                };
                async {
                    agreement.send::<FileExit>(&status).await?;
                };
                async {
                    outcome.send::<FileExit>(&status).await?;
                };
                Ok(())
            }
            .await?;
            endpoint.send::<FrontendExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut frontend_io,
                PrivateFs::new(
                    frontend_terminal,
                    bus,
                    &[FRONTEND_OBJECT, AGREEMENT_OBJECT, FRONTEND_OUTCOME_OBJECT],
                ),
            )
            .await?;
        };
        async {
            let mut endpoint = tester_vm;
            let tester = endpoint.recv::<Loan<TESTER_FILE>>().await?;
            let mut notice = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(TESTER_OBJECT, tester).read_as::<Ready>(&mut notice))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (tester, ())) = files.into_parts();
            let mut tester = tester.into_endpoint();
            tester.send::<FileExit>(&status).await?;
            endpoint.send::<TesterExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut tester_io,
                PrivateFs::new(tester_terminal, bus, &[TESTER_OBJECT]),
            )
            .await?;
        };
        Ok(())
    }
    .await;
    bus.checkpoint(session.raw(), observations.checkpoint());
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let session = SessionId::new(SESSION);
    let (mut connection, mut reader, terminals) = Bus::open(
        ["BACKEND_DEV", "FRONTEND_DEV", "TESTER"],
        [
            (session, DEFINE_CLOSE),
            (session, DEFINE_FILE),
            (session, BACKEND_CLOSE),
            (session, BACKEND_FILE),
            (session, FRONTEND_CLOSE),
            (session, FRONTEND_FILE),
            (session, AGREEMENT_CLOSE),
            (session, AGREEMENT_FILE),
            (session, TESTER_CLOSE),
            (session, TESTER_FILE),
            (session, REPO),
            (session, TEST_ENV),
            (session, BACKEND_OUTCOME_FILE),
            (session, FRONTEND_OUTCOME_FILE),
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
