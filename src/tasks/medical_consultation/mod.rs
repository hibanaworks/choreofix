//! 5E clinical findings and actual CT Endpoint delegation, under qualification.
pub mod global;
use crate::imp::patient;
pub mod localside;
#[cfg(test)]
mod tests;

use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};
use std::cell::Cell;

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
        bus.observe(json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(event)]}))
    };
    let general_input = Cell::new(Value::Null);
    let cardiac_input = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &p.general_service,
        ResolverRef::<GENERAL_COMPLETE>::decision_state(&general_input, |input: &Cell<Value>| {
            patient::panel_arm(input, &["vitals", "basic_labs"])
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.cardiac_service,
        ResolverRef::<CARDIAC_COMPLETE>::decision_state(&cardiac_input, |input: &Cell<Value>| {
            patient::panel_arm(input, &["ecg", "cardiac_enzymes"])
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let result = async {
        let [er_terminal, cardio_terminal, pharmacy_terminal] = terminals;

        let mut er = rv.enter(session, &p.er)?;
        let mut er_io = rv.enter(session, &p.er_io)?;
        let er_fs = PrivateFs::new(er_terminal, bus);
        let mut er_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut er_store = HibanaWasiGuestStorage::uninit();
        let er_guest = er_store.init(module, GuestMemory::new(&mut er_memory), bindings())?;

        let mut cardiologist = rv.enter(session, &p.cardiologist)?;
        let mut cardio_io = rv.enter(session, &p.cardio_io)?;
        let cardio_fs = PrivateFs::new(cardio_terminal, bus);
        let mut cardio_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut cardio_store = HibanaWasiGuestStorage::uninit();
        let cardio_guest =
            cardio_store.init(module, GuestMemory::new(&mut cardio_memory), bindings())?;

        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let pharmacy_fs = PrivateFs::new(pharmacy_terminal, bus);
        let mut pharmacy_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut pharmacy_store = HibanaWasiGuestStorage::uninit();
        let pharmacy_guest =
            pharmacy_store.init(module, GuestMemory::new(&mut pharmacy_memory), bindings())?;

        let mut general_service = rv.enter(session, &p.general_service)?;
        let general_panel = rv.enter(session, &p.general_panel)?;
        let mut cardiac_service = rv.enter(session, &p.cardiac_service)?;
        let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
        let mut pharmacy_service = rv.enter(session, &p.pharmacy_service)?;
        let mut imaging_service = rv.enter(session, &p.imaging_service)?;
        let ct = rv.enter(session, &p.ct)?;

        async {
            localside::services::general(
                &mut general_service,
                (general_panel, &general_input),
                bus,
            )
            .await?;
        };
        async {
            localside::services::cardiac(
                &mut cardiac_service,
                (cardiac_panel, &cardiac_input),
                bus,
            )
            .await?;
        };
        async {
            localside::services::pharmacy(&mut pharmacy_service, bus).await?;
        };
        async {
            localside::services::imaging(&mut imaging_service, ct, bus).await?;
        };
        async {
            let code = localside::er_doctor::run(er_guest, &mut er).await?;
            if code != 0 {
                return Err(ShellError::message(format!("ER doctor exited with {code}")));
            }
        };
        async {
            private_io::run(&mut er_io, er_fs).await?;
        };
        async {
            let code = localside::cardiologist::run(cardio_guest, &mut cardiologist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "cardiologist exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut cardio_io, cardio_fs).await?;
        };
        async {
            let code = localside::pharmacist::run(pharmacy_guest, &mut pharmacist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "pharmacist exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut pharmacy_io, pharmacy_fs).await?;
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
        ["ER_DOCTOR", "CARDIOLOGIST", "PHARMACIST"],
        [
            (session, CT_SCANNER),
            (session, GENERAL_PANEL),
            (session, CARDIAC_PANEL),
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
