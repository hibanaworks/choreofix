//! 4E's API agreement, delegated repository and test-environment operations.
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::{imp::transport::InMemoryTransport, shell::ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId};

#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    bus: &crate::shell::imp::bus::Bus,
    terminals: [crate::shell::imp::bus::Terminal; 3],
) -> ShellResult<()> {
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let p = programs();
    bus.observe(p.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let result = async {
        let mut frontend = rv.enter(session, &p.frontend)?;
        let mut backend = rv.enter(session, &p.backend)?;
        let mut tester = rv.enter(session, &p.tester)?;
        let mut owner = rv.enter(session, &p.store)?;
        let repo = rv.enter(session, &p.repo)?;
        let environment = rv.enter(session, &p.test_env)?;
        let mut frontend_io = rv.enter(session, &p.frontend_io)?;
        let mut backend_io = rv.enter(session, &p.backend_io)?;
        let mut tester_io = rv.enter(session, &p.tester_io)?;
        let [f_terminal, b_terminal, t_terminal] = terminals;
        let f_fs = PrivateFs::new(f_terminal, bus);
        let b_fs = PrivateFs::new(b_terminal, bus);
        let t_fs = PrivateFs::new(t_terminal, bus);
        let mut f_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut f_store = HibanaWasiGuestStorage::uninit();
        let f_guest = f_store.init(module, GuestMemory::new(&mut f_memory), bindings())?;
        let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut b_store = HibanaWasiGuestStorage::uninit();
        let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
        let mut t_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut t_store = HibanaWasiGuestStorage::uninit();
        let t_guest = t_store.init(module, GuestMemory::new(&mut t_memory), bindings())?;
        async {
            localside::resources::repository(&mut owner, (repo, environment), bus).await?;
        };
        async {
            localside::frontend::run(f_guest, &mut frontend).await?;
        };
        async {
            localside::backend::run(b_guest, &mut backend).await?;
        };
        async {
            localside::tester::run(t_guest, &mut tester).await?;
        };
        async {
            private_io::run(&mut frontend_io, f_fs).await?;
        };
        async {
            private_io::run(&mut backend_io, b_fs).await?;
        };
        async {
            private_io::run(&mut tester_io, t_fs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),"checkpoint":observations.checkpoint()}));
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        ["FRONTEND_DEV", "BACKEND_DEV", "TESTER"],
        [
            (SessionId::new(SESSION), REPO),
            (SessionId::new(SESSION), TEST_ENV),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result {
        Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
