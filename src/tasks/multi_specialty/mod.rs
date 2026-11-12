//! 5M: shared patient findings, independent imaging and an actual case conference.
pub mod global;
pub mod localside;
// Both tasks use the same patient/panel data contract. This module contains
// only codecs and actual-result validation, never Endpoint operations.
use crate::imp::patient;
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
        bus.observe(json!({"type":"protocol_capture","producer":0,
        "records":[crate::imp::projection::record(event)]}))
    };
    let general_input = Cell::new(Value::Null);
    let cardiac_input = Cell::new(Value::Null);
    let neuro_input = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &p.general,
        ResolverRef::<GENERAL_COMPLETE>::decision_state(&general_input, |value: &Cell<Value>| {
            patient::panel_arm(value, &["vitals", "basic_labs"])
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.cardiac,
        ResolverRef::<CARDIAC_COMPLETE>::decision_state(&cardiac_input, |value: &Cell<Value>| {
            patient::panel_arm(value, &["ecg", "cardiac_enzymes"])
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.neurology,
        ResolverRef::<NEURO_COMPLETE>::decision_state(&neuro_input, |value: &Cell<Value>| {
            patient::panel_arm(value, &["neuro_exam", "nerve_conduction"])
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let result = async {
        let [
            er_terminal,
            cardio_terminal,
            neuro_terminal,
            radiology_terminal,
            pharmacy_terminal,
        ] = terminals;

        let mut er = rv.enter(session, &p.er)?;
        let mut er_io = rv.enter(session, &p.er_io)?;
        let er_fs = PrivateFs::new(er_terminal, bus);
        let mut er_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut er_store = HibanaWasiGuestStorage::uninit();
        let er_guest = er_store.init(module, GuestMemory::new(&mut er_memory), bindings())?;

        let mut cardio = rv.enter(session, &p.cardio)?;
        let mut cardio_io = rv.enter(session, &p.cardio_io)?;
        let cardio_fs = PrivateFs::new(cardio_terminal, bus);
        let mut cardio_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut cardio_store = HibanaWasiGuestStorage::uninit();
        let cardio_guest =
            cardio_store.init(module, GuestMemory::new(&mut cardio_memory), bindings())?;

        let mut neuro = rv.enter(session, &p.neuro)?;
        let mut neuro_io = rv.enter(session, &p.neuro_io)?;
        let neuro_fs = PrivateFs::new(neuro_terminal, bus);
        let mut neuro_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut neuro_store = HibanaWasiGuestStorage::uninit();
        let neuro_guest =
            neuro_store.init(module, GuestMemory::new(&mut neuro_memory), bindings())?;

        let mut radiologist = rv.enter(session, &p.radiologist)?;
        let mut radiology_io = rv.enter(session, &p.radiology_io)?;
        let radiology_fs = PrivateFs::new(radiology_terminal, bus);
        let mut radiology_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut radiology_store = HibanaWasiGuestStorage::uninit();
        let radiology_guest =
            radiology_store.init(module, GuestMemory::new(&mut radiology_memory), bindings())?;

        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let pharmacy_fs = PrivateFs::new(pharmacy_terminal, bus);
        let mut pharmacy_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut pharmacy_store = HibanaWasiGuestStorage::uninit();
        let pharmacy_guest =
            pharmacy_store.init(module, GuestMemory::new(&mut pharmacy_memory), bindings())?;

        let mut general = rv.enter(session, &p.general)?;
        let general_panel = rv.enter(session, &p.general_panel)?;
        let mut cardiac = rv.enter(session, &p.cardiac)?;
        let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
        let mut neurology = rv.enter(session, &p.neurology)?;
        let neuro_panel = rv.enter(session, &p.neuro_panel)?;
        let mut ct_service = rv.enter(session, &p.ct_service)?;
        let ct = rv.enter(session, &p.ct)?;
        let mut mri_service = rv.enter(session, &p.mri_service)?;
        let mri = rv.enter(session, &p.mri)?;
        let mut pharmacy = rv.enter(session, &p.pharmacy)?;
        let mut conference = rv.enter(session, &p.conference)?;
        let room = rv.enter(session, &p.room)?;
        async {
            localside::services::general(&mut general, (general_panel, &general_input), bus)
                .await?;
        };
        async {
            localside::services::cardiac(&mut cardiac, (cardiac_panel, &cardiac_input), bus)
                .await?;
        };
        async {
            localside::services::neurology(&mut neurology, (neuro_panel, &neuro_input), bus)
                .await?;
        };
        async {
            localside::services::ct(&mut ct_service, ct, bus).await?;
        };
        async {
            localside::services::mri(&mut mri_service, mri, bus).await?;
        };
        async {
            localside::services::pharmacy(&mut pharmacy, bus).await?;
        };
        async {
            localside::services::conference(&mut conference, room, bus).await?;
        };
        async {
            let code = localside::er_doctor::run(er_guest, &mut er).await?;
            if code != 0 {
                return Err(ShellError::message(format!("ER_DOCTOR exited with {code}")));
            }
        };
        async {
            private_io::run(&mut er_io, er_fs).await?;
        };
        async {
            let code = localside::cardiologist::run(cardio_guest, &mut cardio).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "CARDIOLOGIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut cardio_io, cardio_fs).await?;
        };
        async {
            let code = localside::neurologist::run(neuro_guest, &mut neuro).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "NEUROLOGIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut neuro_io, neuro_fs).await?;
        };
        async {
            let code = localside::radiologist::run(radiology_guest, &mut radiologist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "RADIOLOGIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut radiology_io, radiology_fs).await?;
        };
        async {
            let code = localside::pharmacist::run(pharmacy_guest, &mut pharmacist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "PHARMACIST exited with {code}"
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
        [
            "ER_DOCTOR",
            "CARDIOLOGIST",
            "NEUROLOGIST",
            "RADIOLOGIST",
            "PHARMACIST",
        ],
        [
            (session, GENERAL_PANEL),
            (session, CARDIAC_PANEL),
            (session, NEURO_PANEL),
            (session, CT_SCANNER),
            (session, MRI),
            (session, CONSULTATION_ROOM),
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
