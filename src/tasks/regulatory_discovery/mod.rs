//! Five wash guests and the physical effect owners for the 14M global.
pub mod global;
pub mod imp;
pub mod localside;
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
        bus.observe(json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let actual_hplc = Cell::new(Value::Null);
    let actual_cell = Cell::new(Value::Null);
    let actual_review = Cell::new(Value::Null);
    let selected = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &p.hplc_service,
        ResolverRef::<HPLC_RESULT>::decision_state(&actual_hplc, imp::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.cell_service,
        ResolverRef::<CELL_RESULT>::decision_state(&actual_cell, imp::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.review_service,
        ResolverRef::<REGULATORY_RESULT>::decision_state(&actual_review, imp::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.lead,
        ResolverRef::<SELECT_TOX>::decision_state(&selected, imp::choose_toxicology),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.lead,
        ResolverRef::<SELECT_BIO>::decision_state(&selected, imp::choose_biology),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let result = async {
        let mut chemist = rv.enter(session, &p.chemist)?;
        let mut chemist_io = rv.enter(session, &p.chemist_io)?;
        let mut biologist = rv.enter(session, &p.biologist)?;
        let mut biologist_io = rv.enter(session, &p.biologist_io)?;
        let mut toxicologist = rv.enter(session, &p.toxicologist)?;
        let mut toxicologist_io = rv.enter(session, &p.toxicologist_io)?;
        let mut lead = rv.enter(session, &p.lead)?;
        let mut lead_io = rv.enter(session, &p.lead_io)?;
        let mut regulator = rv.enter(session, &p.regulator)?;
        let mut regulator_io = rv.enter(session, &p.regulator_io)?;
        let mut hplc_service = rv.enter(session, &p.hplc_service)?;
        let mut mass_service = rv.enter(session, &p.mass_service)?;
        let mut cell_service = rv.enter(session, &p.cell_service)?;
        let mut review_service = rv.enter(session, &p.review_service)?;
        let hplc = rv.enter(session, &p.hplc)?;
        let mass = rv.enter(session, &p.mass)?;
        let cell = rv.enter(session, &p.cell)?;
        let qc = rv.enter(session, &p.qc)?;
        let culture = rv.enter(session, &p.culture)?;
        let bioassay = rv.enter(session, &p.bioassay)?;
        let stability = rv.enter(session, &p.stability)?;
        let purity = rv.enter(session, &p.purity)?;
        let toxicity = rv.enter(session, &p.toxicity)?;
        let structural = rv.enter(session, &p.structural)?;
        let study = rv.enter(session, &p.study)?;
        let [
            chemist_terminal,
            biologist_terminal,
            toxicologist_terminal,
            lead_terminal,
            regulator_terminal,
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
        let regulator_fs = PrivateFs::new(regulator_terminal, bus);
        let mut regulator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut regulator_store = HibanaWasiGuestStorage::uninit();
        let regulator_guest =
            regulator_store.init(module, GuestMemory::new(&mut regulator_memory), bindings())?;
        async {
            localside::hplc::run(&mut hplc_service, (hplc, &actual_hplc), bus).await?;
        };
        async {
            localside::cells::run(&mut cell_service, (cell, &actual_cell), bus).await?;
        };
        async {
            localside::mass::run(&mut mass_service, mass, bus).await?;
        };
        async {
            localside::review::run(&mut review_service, &actual_review, bus).await?;
        };
        async {
            localside::chemist::run(chemist_guest, (&mut chemist, qc)).await?;
        };
        async {
            localside::biologist::run(
                biologist_guest,
                (&mut biologist, culture, bioassay, stability),
            )
            .await?;
        };
        async {
            localside::toxicologist::run(toxicologist_guest, (&mut toxicologist, purity, toxicity))
                .await?;
        };
        async {
            localside::lead::run(lead_guest, (&mut lead, structural, study), &selected).await?;
        };
        async {
            localside::regulator::run(regulator_guest, &mut regulator).await?;
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
        async {
            private_io::run(&mut regulator_io, regulator_fs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),"checkpoint":observations.checkpoint()}));
    result
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        [
            "CHEMIST",
            "BIOLOGIST",
            "TOXICOLOGIST",
            "LEAD_SCIENTIST",
            "REGULATORY_SPECIALIST",
        ],
        [
            HPLC, MASS, CELL, QC, CULTURE, BIOASSAY, STABILITY, PURITY, TOXICITY, STRUCTURAL, STUDY,
        ]
        .map(|role| (SessionId::new(SESSION), role)),
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
