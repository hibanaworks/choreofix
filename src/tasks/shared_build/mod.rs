//! Shared build inputs: real Endpoint handoff, object data and link receipts.
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    resolver::{DecisionArm, ResolverRef},
};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8], first: DecisionArm) -> ShellResult<()> {
    use crate::shell::{
        imp::{
            bus::Bus,
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let (bus, mut io, [a_terminal, b_terminal, integrator_terminal]) = Bus::open(
        ["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
        [
            (SessionId::new(SESSION), CORE_LIB),
            (SessionId::new(SESSION), SHARED_TYPES),
        ],
    );
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.observe(p.observation());
            let capture = |row| {
                bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(row)]}))
            };
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let session = SessionId::new(SESSION);
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut integrator = rv.enter(session, &p.integrator)?;
                let mut service = rv.enter(session, &p.service)?;
                let core = rv.enter(session, &p.core)?;
                let types = rv.enter(session, &p.types)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut integrator_io = rv.enter(session, &p.integrator_io)?;
                rv.set_resolver(
                    &p.service,
                    ResolverRef::<FIRST_BUILDER>::decision_state(&first, |arm| Ok(*arm)),
                )
                .map_err(|_| ShellError::message("could not bind initial builder allocation"))?;
                let a_fs = PrivateFs::new(a_terminal, &bus);
                let b_fs = PrivateFs::new(b_terminal, &bus);
                let integrator_fs = PrivateFs::new(integrator_terminal, &bus);
                let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut integrator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut a_storage = HibanaWasiGuestStorage::uninit();
                let mut b_storage = HibanaWasiGuestStorage::uninit();
                let mut integrator_storage = HibanaWasiGuestStorage::uninit();
                let a_guest =
                    a_storage.init(module, GuestMemory::new(&mut a_memory), bindings())?;
                let b_guest =
                    b_storage.init(module, GuestMemory::new(&mut b_memory), bindings())?;
                let integrator_guest = integrator_storage.init(
                    module,
                    GuestMemory::new(&mut integrator_memory),
                    bindings(),
                )?;
                async {
                    localside::resources::run(&mut service, (core, types), &bus).await?;
                };
                async {
                    localside::builder::run(a_guest, &mut a).await?;
                };
                async {
                    localside::builder::run(b_guest, &mut b).await?;
                };
                async {
                    localside::integrator::run(integrator_guest, &mut integrator).await?;
                };
                async {
                    private_io::run(&mut a_io, a_fs).await?;
                };
                async {
                    private_io::run(&mut b_io, b_fs).await?;
                };
                async {
                    private_io::run(&mut integrator_io, integrator_fs).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await;
            bus.observe(
                serde_json::json!({"type":"protocol_checkpoint","producer":0,
            "session":SESSION,"checkpoint":observations.checkpoint()}),
            );
            result
        },
        io.run(&bus),
    )
    .await;
    match &result {
        Ok(()) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
