//! The three actual guests, services, resources and result objects share one global.
pub mod global;
mod imp;
mod localside;
#[cfg(test)]
mod tests;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::{PrivateFs, bindings},
            service::Validation,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
#[hibana_runtime::scope]
async fn run_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::*;
    let p = programs();
    bus.observe(p.observation());
    let ac = Validation::new();
    let bc = Validation::new();
    let tc = Validation::new();
    let result = super::imp::TestInput::default();
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(event)]}));
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
            &p.admin,
            ResolverRef::<CHECK_ADMIN>::decision_state(&ac, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.backend,
            ResolverRef::<CHECK_BACKEND>::decision_state(&bc, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.tester,
            ResolverRef::<CHECK_TESTER>::decision_state(&tc, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.tester,
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
        let mut admin_vm = rv.enter(session, &p.admin_vm)?;
        let mut backend_vm = rv.enter(session, &p.backend_vm)?;
        let mut tester_vm = rv.enter(session, &p.tester_vm)?;
        let mut admin_io = rv.enter(session, &p.admin_io)?;
        let mut backend_io = rv.enter(session, &p.backend_io)?;
        let mut tester_io = rv.enter(session, &p.tester_io)?;
        let mut admin_outcome = rv.enter(session, &p.admin_outcome)?;
        let mut backend_outcome = rv.enter(session, &p.backend_outcome)?;
        let mut admin_outcome_vm = rv.enter(session, &p.admin_outcome_vm)?;
        let mut backend_outcome_vm = rv.enter(session, &p.backend_outcome_vm)?;
        let [bterm, aterm, tterm] = terminals;
        let mut afile = imp::OutcomeFile::new();
        let mut bfile = imp::OutcomeFile::new();
        let afs = PrivateFs::new(aterm, bus);
        let bfs = PrivateFs::new(bterm, bus);
        let tfs = PrivateFs::new(tterm, bus);
        let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut tmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut astore = HibanaWasiGuestStorage::uninit();
        let mut bstore = HibanaWasiGuestStorage::uninit();
        let mut tstore = HibanaWasiGuestStorage::uninit();
        let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
        let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
        let tguest = tstore.init(module, GuestMemory::new(&mut tmemory), bindings())?;
        async {
            localside::admin::run(&mut admin, bus, &ac).await?;
        };
        async {
            localside::backend::run(&mut backend, bus, &bc).await?;
        };
        async {
            localside::tester::run(&mut tester, bus, &tc, &result).await?;
        };
        async {
            localside::resources::run(&mut database, database_resource).await?;
        };
        async {
            localside::resources::run(&mut test_env, test_resource).await?;
        };
        async {
            localside::outcome::admin::admin(&mut admin_outcome, &mut afile).await?;
        };
        async {
            localside::outcome::backend::backend(&mut backend_outcome, &mut bfile).await?;
        };
        async {
            localside::wasi::admin(aguest, &mut admin_vm, &mut admin_outcome_vm).await?;
        };
        async {
            localside::private_io::run(&mut admin_io, afs).await?;
        };
        async {
            localside::wasi::backend(bguest, &mut backend_vm, &mut backend_outcome_vm).await?;
        };
        async {
            localside::private_io::run(&mut backend_io, bfs).await?;
        };
        async {
            localside::wasi::tester(tguest, &mut tester_vm).await?;
        };
        async {
            private_io::run(&mut tester_io, tfs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(
        serde_json::json!({"type":"protocol_checkpoint","producer":0,
        "session":session.raw(),"checkpoint":observations.checkpoint()}),
    );
    result
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(28);
    let (bus, mut reader, terminals) = Bus::open(
        ["BACKEND_DEV", "DB_ADMIN", "TESTER"],
        [
            (session, global::DATABASE_RESOURCE),
            (session, global::TEST_RESOURCE),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result {
        Ok(()) => {
            reader
                .emit(
                    &bus,
                    serde_json::json!({"type":"run_result","status":"complete"}),
                )
                .await?
        }
        Err(e) => reader
            .emit(
                &bus,
                serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()}),
            )
            .await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
