//! Seven guest-owned operation views directly drive the emergency-care global.
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};
use std::cell::Cell;
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (
        bus,
        mut io,
        [
            er_a_terminal,
            er_b_terminal,
            radiology_terminal,
            surgery_terminal,
            anesthesia_terminal,
            icu_terminal,
            pharmacy_terminal,
        ],
    ) = Bus::open(
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
            (session, ER_A_FILE),
            (session, ER_B_FILE),
            (session, RADIOLOGY_FILE),
            (session, SURGERY_FILE),
            (session, ANESTHESIA_FILE),
            (session, ICU_FILE),
            (session, PHARMACY_FILE),
        ],
    );
    let participants = async {
        let p = programs();
        bus.observe(p.observation());
        let capture = |event| {
            bus.observe(json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
        };
        let actual_a = Cell::new(Value::Null);
        let actual_b = Cell::new(Value::Null);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.patient_a,
            ResolverRef::<PANEL_A_COMPLETE>::decision_state(&actual_a, |value: &Cell<Value>| {
                imp::panel_arm(
                    value,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                )
            }),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.patient_b,
            ResolverRef::<PANEL_B_COMPLETE>::decision_state(&actual_b, |value: &Cell<Value>| {
                imp::panel_arm(value, ("patient_b", &["stroke_panel", "ct_angio"]))
            }),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let result = async {
            let mut er_a = rv.enter(session, &p.er_a)?;
            let mut er_b = rv.enter(session, &p.er_b)?;
            let mut radiologist = rv.enter(session, &p.radiologist)?;
            let mut surgeon = rv.enter(session, &p.surgeon)?;
            let mut anesthesiologist = rv.enter(session, &p.anesthesiologist)?;
            let mut icu_coordinator = rv.enter(session, &p.icu_coordinator)?;
            let mut pharmacist = rv.enter(session, &p.pharmacist)?;
            let mut er_a_io = rv.enter(session, &p.er_a_io)?;
            let mut er_b_io = rv.enter(session, &p.er_b_io)?;
            let mut radiology_io = rv.enter(session, &p.radiology_io)?;
            let mut surgery_io = rv.enter(session, &p.surgery_io)?;
            let mut anesthesia_io = rv.enter(session, &p.anesthesia_io)?;
            let mut icu_io = rv.enter(session, &p.icu_io)?;
            let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
            let mut patient_a = rv.enter(session, &p.patient_a)?;
            let mut patient_b = rv.enter(session, &p.patient_b)?;
            let mut ct_service = rv.enter(session, &p.ct_service)?;
            let mut mri_service = rv.enter(session, &p.mri_service)?;
            let mut anesthesia = rv.enter(session, &p.anesthesia)?;
            let mut surgery = rv.enter(session, &p.surgery)?;
            let mut icu = rv.enter(session, &p.icu)?;
            let mut pharmacy = rv.enter(session, &p.pharmacy)?;
            let panel_a = rv.enter(session, &p.panel_a)?;
            let panel_b = rv.enter(session, &p.panel_b)?;
            let ct = rv.enter(session, &p.ct)?;
            let mri = rv.enter(session, &p.mri)?;
            let room = rv.enter(session, &p.room)?;
            let bed = rv.enter(session, &p.bed)?;
            let ventilator = rv.enter(session, &p.ventilator)?;
            let mut er_a_vm = rv.enter(session, &p.er_a_vm)?;
            let mut er_a_data = rv.enter(session, &p.er_a_data)?;
            let mut er_b_vm = rv.enter(session, &p.er_b_vm)?;
            let mut er_b_data = rv.enter(session, &p.er_b_data)?;
            let mut radiology_vm = rv.enter(session, &p.radiology_vm)?;
            let mut radiology_data = rv.enter(session, &p.radiology_data)?;
            let mut surgery_vm = rv.enter(session, &p.surgery_vm)?;
            let mut surgery_data = rv.enter(session, &p.surgery_data)?;
            let mut anesthesia_vm = rv.enter(session, &p.anesthesia_vm)?;
            let mut anesthesia_data = rv.enter(session, &p.anesthesia_data)?;
            let mut icu_vm = rv.enter(session, &p.icu_vm)?;
            let mut icu_data = rv.enter(session, &p.icu_data)?;
            let mut pharmacy_vm = rv.enter(session, &p.pharmacy_vm)?;
            let mut pharmacy_data = rv.enter(session, &p.pharmacy_data)?;
            let er_a_file = rv.enter(session, &p.er_a_file)?;
            let er_b_file = rv.enter(session, &p.er_b_file)?;
            let radiology_file = rv.enter(session, &p.radiology_file)?;
            let surgery_file = rv.enter(session, &p.surgery_file)?;
            let anesthesia_file = rv.enter(session, &p.anesthesia_file)?;
            let icu_file = rv.enter(session, &p.icu_file)?;
            let pharmacy_file = rv.enter(session, &p.pharmacy_file)?;
            async {
                file_metadata::run(&mut er_a_data, er_a_file, ER_A_FILES).await?;
            };
            async {
                file_metadata::run(&mut er_b_data, er_b_file, ER_B_FILES).await?;
            };
            async {
                file_metadata::run(&mut radiology_data, radiology_file, RADIOLOGY_FILES).await?;
            };
            async {
                file_metadata::run(&mut surgery_data, surgery_file, SURGERY_FILES).await?;
            };
            async {
                file_metadata::run(&mut anesthesia_data, anesthesia_file, ANESTHESIA_FILES).await?;
            };
            async {
                file_metadata::run(&mut icu_data, icu_file, ICU_FILES).await?;
            };
            async {
                file_metadata::run(&mut pharmacy_data, pharmacy_file, PHARMACY_FILES).await?;
            };
            let (
                er_a_file,
                er_b_file,
                radiology_file,
                surgery_file,
                anesthesia_file,
                icu_file,
                pharmacy_file,
            ) = async {
                async { er_a_vm.recv::<Loan<ER_A_FILE>>().await };
                async { er_b_vm.recv::<Loan<ER_B_FILE>>().await };
                async { radiology_vm.recv::<Loan<RADIOLOGY_FILE>>().await };
                async { surgery_vm.recv::<Loan<SURGERY_FILE>>().await };
                async { anesthesia_vm.recv::<Loan<ANESTHESIA_FILE>>().await };
                async { icu_vm.recv::<Loan<ICU_FILE>>().await };
                async { pharmacy_vm.recv::<Loan<PHARMACY_FILE>>().await };
            }
            .await?;
            async {
                localside::patients::patient_a(&mut patient_a, (panel_a, &actual_a), &bus).await?;
            };
            async {
                localside::patients::patient_b(&mut patient_b, (panel_b, &actual_b), &bus).await?;
            };
            async {
                localside::imaging::ct(&mut ct_service, ct, &bus).await?;
            };
            async {
                localside::imaging::mri(&mut mri_service, mri, &bus).await?;
            };
            async {
                localside::care::anesthesia(&mut anesthesia, room, &bus).await?;
            };
            async {
                localside::care::surgery(&mut surgery, &bus).await?;
            };
            async {
                localside::care::icu(&mut icu, (bed, ventilator), &bus).await?;
            };
            async {
                localside::care::pharmacy(&mut pharmacy, &bus).await?;
            };
            async {
                localside::er_doctor_a::run(&mut er_a).await?;
            };
            async {
                localside::er_doctor_b::run(&mut er_b).await?;
            };
            async {
                localside::radiologist::run(&mut radiologist).await?;
            };
            async {
                localside::surgeon::run(&mut surgeon).await?;
            };
            async {
                localside::anesthesiologist::run(&mut anesthesiologist).await?;
            };
            async {
                localside::icu_coordinator::run(&mut icu_coordinator).await?;
            };
            async {
                localside::pharmacist::run(&mut pharmacist).await?;
            };
            async {
                private_io::run(
                    &mut er_a_io,
                    PrivateFs::new(er_a_terminal, &bus, &ER_A_FILES),
                )
                .await?;
            };
            async {
                private_io::run(
                    &mut er_b_io,
                    PrivateFs::new(er_b_terminal, &bus, &ER_B_FILES),
                )
                .await?;
            };
            async {
                private_io::run(
                    &mut radiology_io,
                    PrivateFs::new(radiology_terminal, &bus, &RADIOLOGY_FILES),
                )
                .await?;
            };
            async {
                private_io::run(
                    &mut surgery_io,
                    PrivateFs::new(surgery_terminal, &bus, &SURGERY_FILES),
                )
                .await?;
            };
            async {
                private_io::run(
                    &mut anesthesia_io,
                    PrivateFs::new(anesthesia_terminal, &bus, &ANESTHESIA_FILES),
                )
                .await?;
            };
            async {
                private_io::run(&mut icu_io, PrivateFs::new(icu_terminal, &bus, &ICU_FILES))
                    .await?;
            };
            async {
                private_io::run(
                    &mut pharmacy_io,
                    PrivateFs::new(pharmacy_terminal, &bus, &PHARMACY_FILES),
                )
                .await?;
            };
            async {
                let mut write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let mut read_2 = vec![0; incoming_capacity];
                let mut write_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_3 = vec![0; incoming_capacity];
                let mut read_4 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ER_A_FILES[0], er_a_file)
                    .write_as::<AssessmentRequestA>(&mut write_0)
                    .read_as::<FileAssessmentA>(&mut read_0)
                    .file(ER_A_FILES[1])?
                    .write_as::<PanelFileRequestA>(&mut write_1)
                    .read_as::<FilePanelResultA>(&mut read_1)
                    .file(ER_A_FILES[2])?
                    .read_as::<FileFindingsA>(&mut read_2)
                    .file(ER_A_FILES[3])?
                    .write_as::<DiagnosisRequestA>(&mut write_3)
                    .read_as::<FileDiagnosisResultA>(&mut read_3)
                    .file(ER_A_FILES[4])?
                    .read_as::<SummaryA>(&mut read_4);
                let files = ChoreoFsFiles::new(er_a_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let mut read_2 = vec![0; incoming_capacity];
                let mut write_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_3 = vec![0; incoming_capacity];
                let mut read_4 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ER_B_FILES[0], er_b_file)
                    .write_as::<AssessmentRequestB>(&mut write_0)
                    .read_as::<FileAssessmentB>(&mut read_0)
                    .file(ER_B_FILES[1])?
                    .write_as::<PanelFileRequestB>(&mut write_1)
                    .read_as::<FilePanelResultB>(&mut read_1)
                    .file(ER_B_FILES[2])?
                    .read_as::<FileFindingsB>(&mut read_2)
                    .file(ER_B_FILES[3])?
                    .write_as::<DiagnosisRequestB>(&mut write_3)
                    .read_as::<FileDiagnosisResultB>(&mut read_3)
                    .file(ER_B_FILES[4])?
                    .read_as::<SummaryB>(&mut read_4);
                let files = ChoreoFsFiles::new(er_b_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let mut write_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_2 = vec![0; incoming_capacity];
                let mut write_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_3 = vec![0; incoming_capacity];
                let mut write_4 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_4 = vec![0; incoming_capacity];
                let mut write_5 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_5 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(RADIOLOGY_FILES[0], radiology_file)
                    .write_as::<CtPerformIntentA>(&mut write_0)
                    .read_as::<FileCtImageA>(&mut read_0)
                    .file(RADIOLOGY_FILES[1])?
                    .write_as::<CtInterpretIntentA>(&mut write_1)
                    .read_as::<FileCtReportA>(&mut read_1)
                    .file(RADIOLOGY_FILES[2])?
                    .write_as::<CtPerformIntentB>(&mut write_2)
                    .read_as::<FileCtImageB>(&mut read_2)
                    .file(RADIOLOGY_FILES[3])?
                    .write_as::<CtInterpretIntentB>(&mut write_3)
                    .read_as::<FileCtReportB>(&mut read_3)
                    .file(RADIOLOGY_FILES[4])?
                    .write_as::<MriPerformIntent>(&mut write_4)
                    .read_as::<FileMriImage>(&mut read_4)
                    .file(RADIOLOGY_FILES[5])?
                    .write_as::<MriInterpretIntent>(&mut write_5)
                    .read_as::<FileMriReport>(&mut read_5);
                let files = ChoreoFsFiles::new(radiology_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(SURGERY_FILES[0], surgery_file)
                    .read_as::<FileDiagnosisSurgery>(&mut read_0)
                    .file(SURGERY_FILES[1])?
                    .write_as::<SurgeryIntent>(&mut write_1)
                    .read_as::<FileSurgeryResult>(&mut read_1);
                let files = ChoreoFsFiles::new(surgery_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ANESTHESIA_FILES[0], anesthesia_file)
                    .read_as::<FileDiagnosisAnesthesia>(&mut read_0)
                    .file(ANESTHESIA_FILES[1])?
                    .write_as::<AnesthesiaIntent>(&mut write_1)
                    .read_as::<FileAnesthesiaResult>(&mut read_1);
                let files = ChoreoFsFiles::new(anesthesia_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut read_0 = vec![0; incoming_capacity];
                let mut write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ICU_FILES[0], icu_file)
                    .read_as::<FileDiagnosisIcu>(&mut read_0)
                    .file(ICU_FILES[1])?
                    .write_as::<IcuIntent>(&mut write_1)
                    .read_as::<FileIcuResult>(&mut read_1);
                let files = ChoreoFsFiles::new(icu_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let mut write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut read_0 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(PHARMACY_FILES[0], pharmacy_file)
                    .write_as::<ReviewRequest>(&mut write_0)
                    .read_as::<FileReviewResult>(&mut read_0);
                let files = ChoreoFsFiles::new(pharmacy_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await;
        bus.observe(json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
        result
    };
    let result = hibana_runtime::race(participants, io.run(&bus)).await;
    match &result {
        Ok(()) => {
            io.emit(&bus, json!({"type":"run_result","status":"complete"}))
                .await?
        }
        Err(error) => {
            io.emit(
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
