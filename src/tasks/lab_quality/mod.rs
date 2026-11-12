//! Four independent Wasm scientists and the actual synthesis/QC effect owners.
pub mod global;
pub(crate) mod imp;
pub mod localside;

#[cfg(test)]
mod tests;

use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::cell::Cell;

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
    use hibana::runtime::resolver::{DecisionArm, ResolverError, ResolverRef};
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let p = programs();
    bus.observe(p.observation());
    let capture = |event| {
        bus.observe(json!({"type":"protocol_capture","producer":0,
   "records":[crate::imp::projection::record(event)]}))
    };
    let selection = Cell::new(Value::Null);
    let actual_qc = Cell::new(Value::Null);
    let actual_bio = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &p.qc,
        ResolverRef::<SELECT_QC>::decision_state(&selection, |input: &Cell<Value>| {
            let value = input.take();
            let decision = match value["scientist"].as_str() {
                Some("CHEMIST") => Ok(DecisionArm::Left),
                Some("BIOLOGIST") => Ok(DecisionArm::Right),
                _ => Err(ResolverError::reject()),
            };
            input.set(value);
            decision
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.fume_service,
        ResolverRef::<QC_SUCCESS>::decision_state(&actual_qc, imp::decide),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.fume_service,
        ResolverRef::<QC_SUBJECT>::decision_state(&actual_qc, |input: &Cell<Value>| {
            let value = input.take();
            let decision = match value["data"]["scientist"].as_str() {
                Some("CHEMIST") => Ok(DecisionArm::Left),
                Some("BIOLOGIST") => Ok(DecisionArm::Right),
                _ => Err(ResolverError::reject()),
            };
            input.set(value);
            decision
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.cent_service,
        ResolverRef::<BIO_SUCCESS>::decision_state(&actual_bio, imp::decide),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let result = async {
        let [
            chemist_terminal,
            biologist_terminal,
            analyst_terminal,
            qc_terminal,
        ] = terminals;

        let mut chemist = rv.enter(session, &p.chemist)?;
        let mut chemist_io = rv.enter(session, &p.chemist_io)?;
        let chemist_fs = PrivateFs::new(chemist_terminal, bus);
        let mut chemist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut chemist_store = HibanaWasiGuestStorage::uninit();
        let chemist_guest =
            chemist_store.init(module, GuestMemory::new(&mut chemist_memory), bindings())?;

        let mut biologist = rv.enter(session, &p.biologist)?;
        let mut biologist_io = rv.enter(session, &p.biologist_io)?;
        let biologist_fs = PrivateFs::new(biologist_terminal, bus);
        let mut biologist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut biologist_store = HibanaWasiGuestStorage::uninit();
        let biologist_guest =
            biologist_store.init(module, GuestMemory::new(&mut biologist_memory), bindings())?;

        let mut analyst = rv.enter(session, &p.analyst)?;
        let mut analyst_io = rv.enter(session, &p.analyst_io)?;
        let analyst_fs = PrivateFs::new(analyst_terminal, bus);
        let mut analyst_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut analyst_store = HibanaWasiGuestStorage::uninit();
        let analyst_guest =
            analyst_store.init(module, GuestMemory::new(&mut analyst_memory), bindings())?;

        let mut qc = rv.enter(session, &p.qc)?;
        let mut qc_io = rv.enter(session, &p.qc_io)?;
        let qc_fs = PrivateFs::new(qc_terminal, bus);
        let mut qc_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut qc_store = HibanaWasiGuestStorage::uninit();
        let qc_guest = qc_store.init(module, GuestMemory::new(&mut qc_memory), bindings())?;

        let mut spec_service = rv.enter(session, &p.spec_service)?;
        let spectrometer = rv.enter(session, &p.spectrometer)?;
        let mut cent_service = rv.enter(session, &p.cent_service)?;
        let centrifuge = rv.enter(session, &p.centrifuge)?;
        let biology = rv.enter(session, &p.biology)?;
        let mut fume_service = rv.enter(session, &p.fume_service)?;
        let fume_hood = rv.enter(session, &p.fume_hood)?;
        let chemistry = rv.enter(session, &p.chemistry)?;
        async {
            localside::spectrometer::run(&mut spec_service, spectrometer, bus).await?;
        };
        async {
            localside::centrifuge::run(&mut cent_service, (centrifuge, biology, &actual_bio), bus)
                .await?;
        };
        async {
            localside::fume::run(&mut fume_service, (fume_hood, chemistry), (bus, &actual_qc))
                .await?;
        };

        async {
            let code = localside::chemist::run(chemist_guest, &mut chemist).await?;
            if code != 0 {
                return Err(ShellError::message(format!("chemist exited with {code}")));
            }
        };
        async {
            private_io::run(&mut chemist_io, chemist_fs).await?;
        };

        async {
            let code = localside::biologist::run(biologist_guest, &mut biologist).await?;
            if code != 0 {
                return Err(ShellError::message(format!("biologist exited with {code}")));
            }
        };
        async {
            private_io::run(&mut biologist_io, biologist_fs).await?;
        };

        async {
            let code = localside::analyst::run(analyst_guest, &mut analyst).await?;
            if code != 0 {
                return Err(ShellError::message(format!("analyst exited with {code}")));
            }
        };
        async {
            private_io::run(&mut analyst_io, analyst_fs).await?;
        };

        async {
            let code = localside::qc::run(qc_guest, &mut qc, &selection).await?;
            if code != 0 {
                return Err(ShellError::message(format!("qc exited with {code}")));
            }
        };
        async {
            private_io::run(&mut qc_io, qc_fs).await?;
        };

        Ok(())
    }
    .await;
    bus.observe(
        json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),
  "checkpoint":observations.checkpoint()}),
    );
    result
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let session = SessionId::new(SESSION);
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        ["CHEMIST", "BIOLOGIST", "ANALYST", "QC_CONTROLLER"],
        [
            (session, SPECTROMETER),
            (session, CENTRIFUGE),
            (session, FUME_HOOD),
            (session, CHEMISTRY),
            (session, BIOLOGY),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result {
        Ok(()) => {
            reader
                .emit(&bus, json!({"type":"run_result","status":"complete"}))
                .await?
        }
        Err(error) => {
            reader
                .emit(
                    &bus,
                    json!({"type":"run_result","status":"failed","error":error.to_string()}),
                )
                .await?
        }
    }
    result
}
pub fn projection() -> Value {
    global::programs().observation()
}
