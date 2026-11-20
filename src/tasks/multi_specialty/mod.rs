//! Each guest owns its operation view and directly drives the clinical global.
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::{
    imp::patient,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::Value;
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
        mut connection,
        mut io,
        [
            er_terminal,
            cardio_terminal,
            neuro_terminal,
            radiology_terminal,
            pharmacy_terminal,
        ],
    ) = Bus::open(
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
            (session, ER_FILE),
            (session, CARDIO_FILE),
            (session, NEURO_FILE),
            (session, RADIOLOGY_FILE),
            (session, PHARMACY_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.projected(p.observation());
            let capture = |event| bus.capture(event);
            let general_input = Cell::new(Value::Null);
            let cardiac_input = Cell::new(Value::Null);
            let neuro_input = Cell::new(Value::Null);
            let mut slab = vec![0; 2 * 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            rv.set_resolver(
                &p.general,
                ResolverRef::<GENERAL_COMPLETE>::decision_state(
                    &general_input,
                    |data: &Cell<Value>| patient::panel_arm(data, &["vitals", "basic_labs"]),
                ),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
            rv.set_resolver(
                &p.cardiac,
                ResolverRef::<CARDIAC_COMPLETE>::decision_state(
                    &cardiac_input,
                    |data: &Cell<Value>| patient::panel_arm(data, &["ecg", "cardiac_enzymes"]),
                ),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
            rv.set_resolver(
                &p.neurology,
                ResolverRef::<NEURO_COMPLETE>::decision_state(
                    &neuro_input,
                    |data: &Cell<Value>| {
                        patient::panel_arm(data, &["neuro_exam", "nerve_conduction"])
                    },
                ),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
            let result = async {
                let mut er = rv.enter(session, &p.er)?;
                let mut cardio = rv.enter(session, &p.cardio)?;
                let mut neuro = rv.enter(session, &p.neuro)?;
                let mut radiologist = rv.enter(session, &p.radiologist)?;
                let mut pharmacist = rv.enter(session, &p.pharmacist)?;
                let mut er_io = rv.enter(session, &p.er_io)?;
                let mut cardio_io = rv.enter(session, &p.cardio_io)?;
                let mut neuro_io = rv.enter(session, &p.neuro_io)?;
                let mut radiology_io = rv.enter(session, &p.radiology_io)?;
                let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
                let mut general = rv.enter(session, &p.general)?;
                let mut cardiac = rv.enter(session, &p.cardiac)?;
                let mut neurology = rv.enter(session, &p.neurology)?;
                let mut ct_service = rv.enter(session, &p.ct_service)?;
                let mut mri_service = rv.enter(session, &p.mri_service)?;
                let mut pharmacy = rv.enter(session, &p.pharmacy)?;
                let mut conference = rv.enter(session, &p.conference)?;
                let general_panel = rv.enter(session, &p.general_panel)?;
                let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
                let neuro_panel = rv.enter(session, &p.neuro_panel)?;
                let ct = rv.enter(session, &p.ct)?;
                let mri = rv.enter(session, &p.mri)?;
                let room = rv.enter(session, &p.room)?;
                let mut er_vm = rv.enter(session, &p.er_vm)?;
                let mut cardio_vm = rv.enter(session, &p.cardio_vm)?;
                let mut neuro_vm = rv.enter(session, &p.neuro_vm)?;
                let mut radiology_vm = rv.enter(session, &p.radiology_vm)?;
                let mut pharmacy_vm = rv.enter(session, &p.pharmacy_vm)?;
                let mut er_data = rv.enter(session, &p.er_data)?;
                let mut cardio_data = rv.enter(session, &p.cardio_data)?;
                let mut neuro_data = rv.enter(session, &p.neuro_data)?;
                let mut radiology_data = rv.enter(session, &p.radiology_data)?;
                let mut pharmacy_data = rv.enter(session, &p.pharmacy_data)?;
                let er_file = rv.enter(session, &p.er_file)?;
                let cardio_file = rv.enter(session, &p.cardio_file)?;
                let neuro_file = rv.enter(session, &p.neuro_file)?;
                let radiology_file = rv.enter(session, &p.radiology_file)?;
                let pharmacy_file = rv.enter(session, &p.pharmacy_file)?;
                async {
                    file_metadata::run(&mut er_data, er_file, ER_FILES).await?;
                };
                async {
                    file_metadata::run(&mut cardio_data, cardio_file, CARDIO_FILES).await?;
                };
                async {
                    file_metadata::run(&mut neuro_data, neuro_file, NEURO_FILES).await?;
                };
                async {
                    file_metadata::run(&mut radiology_data, radiology_file, RADIOLOGY_FILES)
                        .await?;
                };
                async {
                    file_metadata::run(&mut pharmacy_data, pharmacy_file, PHARMACY_FILES).await?;
                };
                let (er_file, cardio_file, neuro_file, radiology_file, pharmacy_file) = async {
                    async { er_vm.recv::<Loan<ER_FILE>>().await };
                    async { cardio_vm.recv::<Loan<CARDIO_FILE>>().await };
                    async { neuro_vm.recv::<Loan<NEURO_FILE>>().await };
                    async { radiology_vm.recv::<Loan<RADIOLOGY_FILE>>().await };
                    async { pharmacy_vm.recv::<Loan<PHARMACY_FILE>>().await };
                }
                .await?;
                async {
                    localside::services::general(
                        &mut general,
                        (general_panel, &general_input),
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::services::cardiac(
                        &mut cardiac,
                        (cardiac_panel, &cardiac_input),
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::services::neurology(
                        &mut neurology,
                        (neuro_panel, &neuro_input),
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::services::ct(&mut ct_service, ct, &simulator).await?;
                };
                async {
                    localside::services::mri(&mut mri_service, mri, &simulator).await?;
                };
                async {
                    localside::services::conference(&mut conference, room, &simulator).await?;
                };
                async {
                    localside::services::pharmacy(&mut pharmacy, &simulator).await?;
                };
                async {
                    localside::er_doctor::run(&mut er).await?;
                };
                async {
                    localside::cardiologist::run(&mut cardio).await?;
                };
                async {
                    localside::neurologist::run(&mut neuro).await?;
                };
                async {
                    localside::radiologist::run(&mut radiologist).await?;
                };
                async {
                    localside::pharmacist::run(&mut pharmacist).await?;
                };
                async {
                    private_io::run(&mut er_io, PrivateFs::new(er_terminal, bus, &ER_FILES))
                        .await?;
                };
                async {
                    private_io::run(
                        &mut cardio_io,
                        PrivateFs::new(cardio_terminal, bus, &CARDIO_FILES),
                    )
                    .await?;
                };
                async {
                    private_io::run(
                        &mut neuro_io,
                        PrivateFs::new(neuro_terminal, bus, &NEURO_FILES),
                    )
                    .await?;
                };
                async {
                    private_io::run(
                        &mut radiology_io,
                        PrivateFs::new(radiology_terminal, bus, &RADIOLOGY_FILES),
                    )
                    .await?;
                };
                async {
                    private_io::run(
                        &mut pharmacy_io,
                        PrivateFs::new(pharmacy_terminal, bus, &PHARMACY_FILES),
                    )
                    .await?;
                };
                async {
                    let mut assess_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut assess_result = vec![0; incoming_capacity];
                    let mut general_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut general_result = vec![0; incoming_capacity];
                    let mut candidate_result = vec![0; incoming_capacity];
                    let mut conference_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut conference_result = vec![0; incoming_capacity];
                    let mut summary_result = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(ER_FILES[0], er_file)
                        .write_as::<AssessmentRequest>(&mut assess_request)
                        .read_as::<FileAssessmentResult>(&mut assess_result)
                        .file(ER_FILES[1])?
                        .write_as::<GeneralFileRequest>(&mut general_request)
                        .read_as::<FileGeneralResult>(&mut general_result)
                        .file(ER_FILES[2])?
                        .read_as::<Candidate>(&mut candidate_result)
                        .file(ER_FILES[3])?
                        .write_as::<ConferenceIntent>(&mut conference_request)
                        .read_as::<FileConferenceResult>(&mut conference_result)
                        .file(ER_FILES[4])?
                        .read_as::<FinalForER>(&mut summary_result);
                    let files = ChoreoFsFiles::new(er_vm).file(file)?;
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
                    let mut referral_result = vec![0; incoming_capacity];
                    let mut tests_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut tests_result = vec![0; incoming_capacity];
                    let mut findings_result = vec![0; incoming_capacity];
                    let mut diagnosis_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut diagnosis_result = vec![0; incoming_capacity];
                    let mut summary_result = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(CARDIO_FILES[0], cardio_file)
                        .read_as::<FileCardReferral>(&mut referral_result)
                        .file(CARDIO_FILES[1])?
                        .write_as::<CardiacFileRequest>(&mut tests_request)
                        .read_as::<FileCardiacResult>(&mut tests_result)
                        .file(CARDIO_FILES[2])?
                        .read_as::<FileCardFindings>(&mut findings_result)
                        .file(CARDIO_FILES[3])?
                        .write_as::<CardDiagnosisRequest>(&mut diagnosis_request)
                        .read_as::<FileCardDiagnosisResult>(&mut diagnosis_result)
                        .file(CARDIO_FILES[4])?
                        .read_as::<FinalForCard>(&mut summary_result);
                    let files = ChoreoFsFiles::new(cardio_vm).file(file)?;
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
                    let mut referral_result = vec![0; incoming_capacity];
                    let mut tests_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut tests_result = vec![0; incoming_capacity];
                    let mut findings_result = vec![0; incoming_capacity];
                    let mut diagnosis_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut diagnosis_result = vec![0; incoming_capacity];
                    let mut summary_result = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(NEURO_FILES[0], neuro_file)
                        .read_as::<FileNeuroReferral>(&mut referral_result)
                        .file(NEURO_FILES[1])?
                        .write_as::<NeuroFileRequest>(&mut tests_request)
                        .read_as::<FileNeuroResult>(&mut tests_result)
                        .file(NEURO_FILES[2])?
                        .read_as::<FileNeuroFindings>(&mut findings_result)
                        .file(NEURO_FILES[3])?
                        .write_as::<NeuroDiagnosisRequest>(&mut diagnosis_request)
                        .read_as::<FileNeuroDiagnosisResult>(&mut diagnosis_result)
                        .file(NEURO_FILES[4])?
                        .read_as::<FinalForNeuro>(&mut summary_result);
                    let files = ChoreoFsFiles::new(neuro_vm).file(file)?;
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
                    let mut ct_order_result = vec![0; incoming_capacity];
                    let mut ct_perform_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut ct_perform_result = vec![0; incoming_capacity];
                    let mut ct_interpret_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut ct_interpret_result = vec![0; incoming_capacity];
                    let mut mri_order_result = vec![0; incoming_capacity];
                    let mut mri_perform_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut mri_perform_result = vec![0; incoming_capacity];
                    let mut mri_interpret_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut mri_interpret_result = vec![0; incoming_capacity];
                    let mut summary_result = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(RADIOLOGY_FILES[0], radiology_file)
                        .read_as::<FileCtOrder>(&mut ct_order_result)
                        .file(RADIOLOGY_FILES[1])?
                        .write_as::<CtPerformIntent>(&mut ct_perform_request)
                        .read_as::<FileCtImageResult>(&mut ct_perform_result)
                        .file(RADIOLOGY_FILES[2])?
                        .write_as::<CtInterpretIntent>(&mut ct_interpret_request)
                        .read_as::<FileCtReportResult>(&mut ct_interpret_result)
                        .file(RADIOLOGY_FILES[3])?
                        .read_as::<FileMriOrder>(&mut mri_order_result)
                        .file(RADIOLOGY_FILES[4])?
                        .write_as::<MriPerformIntent>(&mut mri_perform_request)
                        .read_as::<FileMriImageResult>(&mut mri_perform_result)
                        .file(RADIOLOGY_FILES[5])?
                        .write_as::<MriInterpretIntent>(&mut mri_interpret_request)
                        .read_as::<FileMriReportResult>(&mut mri_interpret_result)
                        .file(RADIOLOGY_FILES[6])?
                        .read_as::<FinalForRad>(&mut summary_result);
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
                    let mut review_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut review_result = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(PHARMACY_FILES[0], pharmacy_file)
                        .write_as::<ReviewRequest>(&mut review_request)
                        .read_as::<FileReviewResult>(&mut review_result);
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
            bus.checkpoint(SESSION, observations.checkpoint());
            result
        },
        io.run(bus),
    )
    .await;
    io.finish(bus, result, None).await
}
pub fn projection() -> Value {
    global::programs().observation()
}
