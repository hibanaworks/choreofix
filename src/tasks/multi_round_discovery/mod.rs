//! Seven actual Wasm scientists and five physical owners share one 14H source.
pub mod global;
pub mod imp;
pub mod localside;
use crate::{imp::transport::InMemoryTransport, shell::ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};

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
        bus.observe(json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
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
        let mut toxicologist = rv.enter(session, &p.toxicologist)?;
        let mut toxicologist_io = rv.enter(session, &p.toxicologist_io)?;
        let mut formulator = rv.enter(session, &p.formulator)?;
        let mut formulator_io = rv.enter(session, &p.formulator_io)?;
        let mut clinician = rv.enter(session, &p.clinician)?;
        let mut clinician_io = rv.enter(session, &p.clinician_io)?;
        let mut regulator = rv.enter(session, &p.regulator)?;
        let mut regulator_io = rv.enter(session, &p.regulator_io)?;
        let mut director = rv.enter(session, &p.director)?;
        let mut director_io = rv.enter(session, &p.director_io)?;
        let mut hplc_service = rv.enter(session, &p.hplc_service)?;
        let mut mass_service = rv.enter(session, &p.mass_service)?;
        let mut cell_service = rv.enter(session, &p.cell_service)?;
        let mut form_service = rv.enter(session, &p.form_service)?;
        let mut study_service = rv.enter(session, &p.study_service)?;
        let hplc = rv.enter(session, &p.hplc)?;
        let mass = rv.enter(session, &p.mass)?;
        let cell = rv.enter(session, &p.cell)?;
        let form = rv.enter(session, &p.form)?;
        let samples = rv.enter(session, &p.samples)?;
        let qc = rv.enter(session, &p.qc)?;
        let culture = rv.enter(session, &p.culture)?;
        let formulation = rv.enter(session, &p.formulation)?;
        let bioassay = rv.enter(session, &p.bioassay)?;
        let stability = rv.enter(session, &p.stability)?;
        let purity = rv.enter(session, &p.purity)?;
        let toxicity = rv.enter(session, &p.toxicity)?;
        let study = rv.enter(session, &p.study)?;
        let [
            chemist_terminal,
            biologist_terminal,
            toxicologist_terminal,
            formulator_terminal,
            clinician_terminal,
            regulator_terminal,
            director_terminal,
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
        let formulator_fs = PrivateFs::new(formulator_terminal, bus);
        let mut formulator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut formulator_store = HibanaWasiGuestStorage::uninit();
        let formulator_guest =
            formulator_store.init(module, GuestMemory::new(&mut formulator_memory), bindings())?;
        let clinician_fs = PrivateFs::new(clinician_terminal, bus);
        let mut clinician_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut clinician_store = HibanaWasiGuestStorage::uninit();
        let clinician_guest =
            clinician_store.init(module, GuestMemory::new(&mut clinician_memory), bindings())?;
        let regulator_fs = PrivateFs::new(regulator_terminal, bus);
        let mut regulator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut regulator_store = HibanaWasiGuestStorage::uninit();
        let regulator_guest =
            regulator_store.init(module, GuestMemory::new(&mut regulator_memory), bindings())?;
        let director_fs = PrivateFs::new(director_terminal, bus);
        let mut director_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut director_store = HibanaWasiGuestStorage::uninit();
        let director_guest =
            director_store.init(module, GuestMemory::new(&mut director_memory), bindings())?;
        async {
            localside::hplc::run(&mut hplc_service, hplc, bus).await?;
        };
        async {
            localside::mass::run(&mut mass_service, mass, bus).await?;
        };
        async {
            localside::cells::run(&mut cell_service, (cell, samples), bus).await?;
        };
        async {
            localside::formulation::run(&mut form_service, form, bus).await?;
        };
        async {
            localside::study::run(&mut study_service, bus).await?;
        };
        async {
            localside::chemist::run(chemist_guest, (&mut chemist, qc)).await?;
        };
        async {
            localside::biologist::run(biologist_guest, (&mut biologist, culture)).await?;
        };
        async {
            localside::toxicologist::run(toxicologist_guest, &mut toxicologist).await?;
        };
        async {
            localside::formulator::run(formulator_guest, &mut formulator).await?;
        };
        async {
            localside::clinician::run(clinician_guest, &mut clinician).await?;
        };
        async {
            localside::regulator::run(regulator_guest, &mut regulator).await?;
        };
        async {
            localside::director::run(
                director_guest,
                (
                    &mut director,
                    study,
                    formulation,
                    bioassay,
                    stability,
                    purity,
                    toxicity,
                ),
            )
            .await?;
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
            private_io::run(&mut formulator_io, formulator_fs).await?;
        };
        async {
            private_io::run(&mut clinician_io, clinician_fs).await?;
        };
        async {
            private_io::run(&mut regulator_io, regulator_fs).await?;
        };
        async {
            private_io::run(&mut director_io, director_fs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),"checkpoint":observations.checkpoint()}));
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let (bus, mut owner, terminals) = crate::shell::imp::bus::Bus::open(
        [
            "CHEMIST",
            "BIOLOGIST",
            "TOXICOLOGIST",
            "FORMULATION_SCIENTIST",
            "CLINICAL_LEAD",
            "REGULATORY_SPECIALIST",
            "PROJECT_DIRECTOR",
        ],
        [
            HPLC,
            MASS,
            CELL,
            FORM,
            SAMPLES,
            QC,
            CULTURE,
            FORMULATION,
            BIOASSAY,
            STABILITY,
            PURITY,
            TOXICITY,
            STUDY,
        ]
        .map(|role| (SessionId::new(SESSION), role)),
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), owner.run(&bus)).await;
    match &result {
        Ok(()) => {
            owner
                .emit(&bus, json!({"type":"run_result","status":"complete"}))
                .await?
        }
        Err(error) => {
            owner
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
