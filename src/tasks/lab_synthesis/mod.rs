//! Three independent Wasm scientists and two physical instrument owners.
pub mod global;
mod imp;
pub mod localside;
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
        let mut chemist = rv.enter(session, &p.chemist)?;
        let mut chemist_io = rv.enter(session, &p.chemist_io)?;
        let mut biologist = rv.enter(session, &p.biologist)?;
        let mut biologist_io = rv.enter(session, &p.biologist_io)?;
        let mut analyst = rv.enter(session, &p.analyst)?;
        let mut analyst_io = rv.enter(session, &p.analyst_io)?;
        let mut spec_service = rv.enter(session, &p.spec_service)?;
        let mut cent_service = rv.enter(session, &p.cent_service)?;
        let spectrometer = rv.enter(session, &p.spectrometer)?;
        let centrifuge = rv.enter(session, &p.centrifuge)?;
        let [chemist_terminal, biologist_terminal, analyst_terminal] = terminals;
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
        let analyst_fs = PrivateFs::new(analyst_terminal, bus);
        let mut analyst_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut analyst_store = HibanaWasiGuestStorage::uninit();
        let analyst_guest =
            analyst_store.init(module, GuestMemory::new(&mut analyst_memory), bindings())?;
        async {
            localside::instruments::spectrometer(&mut spec_service, spectrometer, bus).await?;
        };
        async {
            localside::instruments::centrifuge(&mut cent_service, centrifuge, bus).await?;
        };
        async {
            localside::chemist::run(chemist_guest, &mut chemist).await?;
        };
        async {
            localside::biologist::run(biologist_guest, &mut biologist).await?;
        };
        async {
            localside::analyst::run(analyst_guest, &mut analyst).await?;
        };
        async {
            private_io::run(&mut chemist_io, chemist_fs).await?;
        };
        async {
            private_io::run(&mut biologist_io, biologist_fs).await?;
        };
        async {
            private_io::run(&mut analyst_io, analyst_fs).await?;
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
        ["CHEMIST", "BIOLOGIST", "ANALYST"],
        [
            (SessionId::new(SESSION), SPECTROMETER),
            (SessionId::new(SESSION), CENTRIFUGE),
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
#[cfg(test)]
mod tests;
