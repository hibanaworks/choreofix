//! 5H: actual patient data and affine equipment custody.
pub mod global;
mod imp;
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
    terminals: [crate::shell::imp::bus::Terminal; 7],
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
    let actual_a = Cell::new(Value::Null);
    let actual_b = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &p.patient_a,
        ResolverRef::<PANEL_A_COMPLETE>::decision_state(&actual_a, |value: &Cell<Value>| {
            imp::panel_arm(
                value,
                ("patient_a", &["trauma_panel", "blood_type", "coag"]),
            )
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rv.set_resolver(
        &p.patient_b,
        ResolverRef::<PANEL_B_COMPLETE>::decision_state(&actual_b, |value: &Cell<Value>| {
            imp::panel_arm(value, ("patient_b", &["stroke_panel", "ct_angio"]))
        }),
    )
    .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let result = async {
        let [
            er_a_terminal,
            er_b_terminal,
            radiologist_terminal,
            surgeon_terminal,
            anesthesiologist_terminal,
            icu_coordinator_terminal,
            pharmacist_terminal,
        ] = terminals;
        let mut er_a = rv.enter(session, &p.er_a)?;
        let mut er_a_io = rv.enter(session, &p.er_a_io)?;
        let er_a_fs = PrivateFs::new(er_a_terminal, bus);
        let mut er_a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut er_a_store = HibanaWasiGuestStorage::uninit();
        let er_a_guest = er_a_store.init(module, GuestMemory::new(&mut er_a_memory), bindings())?;
        let mut er_b = rv.enter(session, &p.er_b)?;
        let mut er_b_io = rv.enter(session, &p.er_b_io)?;
        let er_b_fs = PrivateFs::new(er_b_terminal, bus);
        let mut er_b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut er_b_store = HibanaWasiGuestStorage::uninit();
        let er_b_guest = er_b_store.init(module, GuestMemory::new(&mut er_b_memory), bindings())?;
        let mut radiologist = rv.enter(session, &p.radiologist)?;
        let mut radiology_io = rv.enter(session, &p.radiology_io)?;
        let radiologist_fs = PrivateFs::new(radiologist_terminal, bus);
        let mut radiologist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut radiologist_store = HibanaWasiGuestStorage::uninit();
        let radiologist_guest = radiologist_store.init(
            module,
            GuestMemory::new(&mut radiologist_memory),
            bindings(),
        )?;
        let mut surgeon = rv.enter(session, &p.surgeon)?;
        let mut surgery_io = rv.enter(session, &p.surgery_io)?;
        let surgeon_fs = PrivateFs::new(surgeon_terminal, bus);
        let mut surgeon_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut surgeon_store = HibanaWasiGuestStorage::uninit();
        let surgeon_guest =
            surgeon_store.init(module, GuestMemory::new(&mut surgeon_memory), bindings())?;
        let mut anesthesiologist = rv.enter(session, &p.anesthesiologist)?;
        let mut anesthesia_io = rv.enter(session, &p.anesthesia_io)?;
        let anesthesiologist_fs = PrivateFs::new(anesthesiologist_terminal, bus);
        let mut anesthesiologist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut anesthesiologist_store = HibanaWasiGuestStorage::uninit();
        let anesthesiologist_guest = anesthesiologist_store.init(
            module,
            GuestMemory::new(&mut anesthesiologist_memory),
            bindings(),
        )?;
        let mut icu_coordinator = rv.enter(session, &p.icu_coordinator)?;
        let mut icu_io = rv.enter(session, &p.icu_io)?;
        let icu_coordinator_fs = PrivateFs::new(icu_coordinator_terminal, bus);
        let mut icu_coordinator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut icu_coordinator_store = HibanaWasiGuestStorage::uninit();
        let icu_coordinator_guest = icu_coordinator_store.init(
            module,
            GuestMemory::new(&mut icu_coordinator_memory),
            bindings(),
        )?;
        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let pharmacist_fs = PrivateFs::new(pharmacist_terminal, bus);
        let mut pharmacist_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut pharmacist_store = HibanaWasiGuestStorage::uninit();
        let pharmacist_guest =
            pharmacist_store.init(module, GuestMemory::new(&mut pharmacist_memory), bindings())?;
        let mut patient_a = rv.enter(session, &p.patient_a)?;
        let mut patient_b = rv.enter(session, &p.patient_b)?;
        let panel_a = rv.enter(session, &p.panel_a)?;
        let panel_b = rv.enter(session, &p.panel_b)?;
        let mut ct_service = rv.enter(session, &p.ct_service)?;
        let ct = rv.enter(session, &p.ct)?;
        let mut mri_service = rv.enter(session, &p.mri_service)?;
        let mri = rv.enter(session, &p.mri)?;
        let mut anesthesia = rv.enter(session, &p.anesthesia)?;
        let room = rv.enter(session, &p.room)?;
        let mut surgery = rv.enter(session, &p.surgery)?;
        let mut icu = rv.enter(session, &p.icu)?;
        let bed = rv.enter(session, &p.bed)?;
        let ventilator = rv.enter(session, &p.ventilator)?;
        let mut pharmacy = rv.enter(session, &p.pharmacy)?;
        async {
            localside::patients::patient_a(&mut patient_a, (panel_a, &actual_a), bus).await?;
        };
        async {
            localside::patients::patient_b(&mut patient_b, (panel_b, &actual_b), bus).await?;
        };
        async {
            localside::imaging::ct(&mut ct_service, ct, bus).await?;
        };
        async {
            localside::imaging::mri(&mut mri_service, mri, bus).await?;
        };
        async {
            localside::care::anesthesia(&mut anesthesia, room, bus).await?;
        };
        async {
            localside::care::surgery(&mut surgery, bus).await?;
        };
        async {
            localside::care::icu(&mut icu, (bed, ventilator), bus).await?;
        };
        async {
            localside::care::pharmacy(&mut pharmacy, bus).await?;
        };
        async {
            let code = localside::er_doctor_a::run(er_a_guest, &mut er_a).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "ER_DOCTOR_A exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut er_a_io, er_a_fs).await?;
        };
        async {
            let code = localside::er_doctor_b::run(er_b_guest, &mut er_b).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "ER_DOCTOR_B exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut er_b_io, er_b_fs).await?;
        };
        async {
            let code = localside::radiologist::run(radiologist_guest, &mut radiologist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "RADIOLOGIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut radiology_io, radiologist_fs).await?;
        };
        async {
            let code = localside::surgeon::run(surgeon_guest, &mut surgeon).await?;
            if code != 0 {
                return Err(ShellError::message(format!("SURGEON exited with {code}")));
            }
        };
        async {
            private_io::run(&mut surgery_io, surgeon_fs).await?;
        };
        async {
            let code =
                localside::anesthesiologist::run(anesthesiologist_guest, &mut anesthesiologist)
                    .await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "ANESTHESIOLOGIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut anesthesia_io, anesthesiologist_fs).await?;
        };
        async {
            let code = localside::icu_coordinator::run(icu_coordinator_guest, &mut icu_coordinator)
                .await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "ICU_COORDINATOR exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut icu_io, icu_coordinator_fs).await?;
        };
        async {
            let code = localside::pharmacist::run(pharmacist_guest, &mut pharmacist).await?;
            if code != 0 {
                return Err(ShellError::message(format!(
                    "PHARMACIST exited with {code}"
                )));
            }
        };
        async {
            private_io::run(&mut pharmacy_io, pharmacist_fs).await?;
        };
        Ok::<(), ShellError>(())
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
            "ER_DOCTOR_A",
            "ER_DOCTOR_B",
            "RADIOLOGIST",
            "SURGEON",
            "ANESTHESIOLOGIST",
            "ICU_COORDINATOR",
            "PHARMACIST",
        ],
        [
            (session, PANEL_A),
            (session, PANEL_B),
            (session, CT_SCANNER),
            (session, MRI),
            (session, OPERATING_ROOM),
            (session, ICU_BEDS),
            (session, VENTILATOR),
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
