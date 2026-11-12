//! 4M: five independent wash guests, actual resource owners and direct localsides.
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
    terminals: [crate::shell::imp::bus::Terminal; 5],
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
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
  "records":[crate::imp::projection::record(event)]}))
    };
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let result = async {
        let mut architect = rv.enter(session, &p.architect)?;
        let mut a = rv.enter(session, &p.a)?;
        let mut b = rv.enter(session, &p.b)?;
        let mut reviewer = rv.enter(session, &p.reviewer)?;
        let mut tester = rv.enter(session, &p.tester)?;
        let mut store = rv.enter(session, &p.store)?;
        let mut review_service = rv.enter(session, &p.review_service)?;
        let repo = rv.enter(session, &p.repo)?;
        let environment = rv.enter(session, &p.test_env)?;
        let mut architect_io = rv.enter(session, &p.architect_io)?;
        let mut a_io = rv.enter(session, &p.a_io)?;
        let mut b_io = rv.enter(session, &p.b_io)?;
        let mut reviewer_io = rv.enter(session, &p.reviewer_io)?;
        let mut tester_io = rv.enter(session, &p.tester_io)?;
        let [
            architect_terminal,
            a_terminal,
            b_terminal,
            reviewer_terminal,
            tester_terminal,
        ] = terminals;
        let architect_fs = PrivateFs::new(architect_terminal, bus);
        let a_fs = PrivateFs::new(a_terminal, bus);
        let b_fs = PrivateFs::new(b_terminal, bus);
        let reviewer_fs = PrivateFs::new(reviewer_terminal, bus);
        let tester_fs = PrivateFs::new(tester_terminal, bus);
        let mut architect_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut architect_store = HibanaWasiGuestStorage::uninit();
        let architect_guest =
            architect_store.init(module, GuestMemory::new(&mut architect_memory), bindings())?;
        let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut a_store = HibanaWasiGuestStorage::uninit();
        let a_guest = a_store.init(module, GuestMemory::new(&mut a_memory), bindings())?;
        let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut b_store = HibanaWasiGuestStorage::uninit();
        let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
        let mut reviewer_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut reviewer_store = HibanaWasiGuestStorage::uninit();
        let reviewer_guest =
            reviewer_store.init(module, GuestMemory::new(&mut reviewer_memory), bindings())?;
        let mut tester_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut tester_store = HibanaWasiGuestStorage::uninit();
        let tester_guest =
            tester_store.init(module, GuestMemory::new(&mut tester_memory), bindings())?;
        async {
            localside::resources::repository(&mut store, (repo, environment), bus).await?;
        };
        async {
            localside::reviewer::run(&mut review_service, bus).await?;
        };
        async {
            localside::architect::run(architect_guest, &mut architect).await?;
        };
        async {
            localside::developer::run(a_guest, &mut a).await?;
        };
        async {
            localside::developer::run(b_guest, &mut b).await?;
        };
        async {
            localside::reviewer_wasi::run(reviewer_guest, &mut reviewer).await?;
        };
        async {
            localside::tester::run(tester_guest, &mut tester).await?;
        };
        async {
            private_io::run(&mut architect_io, architect_fs).await?;
        };
        async {
            private_io::run(&mut a_io, a_fs).await?;
        };
        async {
            private_io::run(&mut b_io, b_fs).await?;
        };
        async {
            private_io::run(&mut reviewer_io, reviewer_fs).await?;
        };
        async {
            private_io::run(&mut tester_io, tester_fs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(
        serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),
  "checkpoint":observations.checkpoint()}),
    );
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        [
            "ARCHITECT",
            "DEVELOPER_A",
            "DEVELOPER_B",
            "REVIEWER",
            "TESTER",
        ],
        [
            (SessionId::new(SESSION), REPO),
            (SessionId::new(SESSION), TEST_ENV),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result{
  Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
  Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
 }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
