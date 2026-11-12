//! Four independent wash guests and three actual instrument owners for 14E.
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
    terminals: [crate::shell::imp::bus::Terminal; 4],
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
        let mut chemist = rv.enter(session, &p.chemist)?;
        let mut biologist = rv.enter(session, &p.biologist)?;
        let mut toxicologist = rv.enter(session, &p.toxicologist)?;
        let mut lead = rv.enter(session, &p.lead)?;
        let mut chemist_io = rv.enter(session, &p.chemist_io)?;
        let mut biologist_io = rv.enter(session, &p.biologist_io)?;
        let mut toxicologist_io = rv.enter(session, &p.toxicologist_io)?;
        let mut lead_io = rv.enter(session, &p.lead_io)?;
        let mut hplc_service = rv.enter(session, &p.hplc_service)?;
        let mut mass_service = rv.enter(session, &p.mass_service)?;
        let mut cell_service = rv.enter(session, &p.cell_service)?;
        let hplc = rv.enter(session, &p.hplc)?;
        let mass = rv.enter(session, &p.mass)?;
        let cell = rv.enter(session, &p.cell)?;
        let [
            chemist_terminal,
            biologist_terminal,
            toxicologist_terminal,
            lead_terminal,
        ] = terminals;
        let chemist_fs = PrivateFs::new(chemist_terminal, bus);
        let mut chemist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut chemist_store = HibanaWasiGuestStorage::uninit();
        let chemist_guest =
            chemist_store.init(module, GuestMemory::new(&mut chemist_memory), bindings())?;
        let biologist_fs = PrivateFs::new(biologist_terminal, bus);
        let mut biologist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut biologist_store = HibanaWasiGuestStorage::uninit();
        let biologist_guest =
            biologist_store.init(module, GuestMemory::new(&mut biologist_memory), bindings())?;
        let toxicologist_fs = PrivateFs::new(toxicologist_terminal, bus);
        let mut toxicologist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut toxicologist_store = HibanaWasiGuestStorage::uninit();
        let toxicologist_guest = toxicologist_store.init(
            module,
            GuestMemory::new(&mut toxicologist_memory),
            bindings(),
        )?;
        let lead_fs = PrivateFs::new(lead_terminal, bus);
        let mut lead_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut lead_store = HibanaWasiGuestStorage::uninit();
        let lead_guest = lead_store.init(module, GuestMemory::new(&mut lead_memory), bindings())?;
        async {
            localside::instruments::hplc(&mut hplc_service, hplc, bus).await?;
        };
        async {
            localside::instruments::mass(&mut mass_service, mass, bus).await?;
        };
        async {
            localside::instruments::cells(&mut cell_service, cell, bus).await?;
        };
        async {
            localside::chemist::run(chemist_guest, &mut chemist).await?;
        };
        async {
            localside::biologist::run(biologist_guest, &mut biologist).await?;
        };
        async {
            localside::toxicologist::run(toxicologist_guest, &mut toxicologist).await?;
        };
        async {
            localside::lead::run(lead_guest, &mut lead).await?;
        };
        async {
            private_io::run(&mut chemist_io, chemist_fs).await?;
        };
        async {
            private_io::run(&mut biologist_io, biologist_fs).await?;
        };
        async {
            private_io::run(&mut toxicologist_io, toxicologist_fs).await?;
        };
        async {
            private_io::run(&mut lead_io, lead_fs).await?;
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
        ["CHEMIST", "BIOLOGIST", "TOXICOLOGIST", "LEAD_SCIENTIST"],
        [
            (SessionId::new(SESSION), HPLC),
            (SessionId::new(SESSION), MASS_SPEC),
            (SessionId::new(SESSION), CELL_LAB),
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
