//! Typed operation files are direct views of the one clinical choreography.
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
    let (mut connection, mut io, [er_terminal, cardio_terminal, pharmacy_terminal]) = Bus::open(
        ["ER_DOCTOR", "CARDIOLOGIST", "PHARMACIST"],
        [
            (session, CT_SCANNER),
            (session, GENERAL_PANEL),
            (session, CARDIAC_PANEL),
            (session, ASSESS_FILE),
            (session, GENERAL_FILE),
            (session, IMAGING_FILE),
            (session, REFERRAL_FILE),
            (session, CARDIAC_FILE),
            (session, FINDINGS_FILE),
            (session, DIAGNOSIS_FILE),
            (session, REVIEW_FILE),
            (session, SUMMARY_FILE),
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
            let mut slab = vec![0; 2 * 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            rv.set_resolver(
                &p.general_service,
                ResolverRef::<GENERAL_COMPLETE>::decision_state(
                    &general_input,
                    |input: &Cell<Value>| patient::panel_arm(input, &["vitals", "basic_labs"]),
                ),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
            rv.set_resolver(
                &p.cardiac_service,
                ResolverRef::<CARDIAC_COMPLETE>::decision_state(
                    &cardiac_input,
                    |input: &Cell<Value>| patient::panel_arm(input, &["ecg", "cardiac_enzymes"]),
                ),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
            let result = async {
                let mut records = rv.enter(session, &p.records)?;
                let mut er = rv.enter(session, &p.er)?;
                let mut cardiologist = rv.enter(session, &p.cardiologist)?;
                let mut pharmacist = rv.enter(session, &p.pharmacist)?;
                let mut general_service = rv.enter(session, &p.general_service)?;
                let mut cardiac_service = rv.enter(session, &p.cardiac_service)?;
                let mut pharmacy_service = rv.enter(session, &p.pharmacy_service)?;
                let mut imaging_service = rv.enter(session, &p.imaging_service)?;
                let mut er_io = rv.enter(session, &p.er_io)?;
                let mut cardio_io = rv.enter(session, &p.cardio_io)?;
                let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
                let ct = rv.enter(session, &p.ct)?;
                let general_panel = rv.enter(session, &p.general_panel)?;
                let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
                let er_vm = rv.enter(session, &p.er_vm)?;
                let cardio_vm = rv.enter(session, &p.cardio_vm)?;
                let pharmacy_vm = rv.enter(session, &p.pharmacy_vm)?;
                let mut assess_data = rv.enter(session, &p.assess_data)?;
                let assess_file = rv.enter(session, &p.assess_file)?;
                let mut general_data = rv.enter(session, &p.general_data)?;
                let general_file = rv.enter(session, &p.general_file)?;
                let mut imaging_data = rv.enter(session, &p.imaging_data)?;
                let imaging_file = rv.enter(session, &p.imaging_file)?;
                let mut referral_data = rv.enter(session, &p.referral_data)?;
                let referral_file = rv.enter(session, &p.referral_file)?;
                let mut cardiac_data = rv.enter(session, &p.cardiac_data)?;
                let cardiac_file = rv.enter(session, &p.cardiac_file)?;
                let mut findings_data = rv.enter(session, &p.findings_data)?;
                let findings_file = rv.enter(session, &p.findings_file)?;
                let mut diagnosis_data = rv.enter(session, &p.diagnosis_data)?;
                let diagnosis_file = rv.enter(session, &p.diagnosis_file)?;
                let mut review_data = rv.enter(session, &p.review_data)?;
                let review_file = rv.enter(session, &p.review_file)?;
                let mut summary_data = rv.enter(session, &p.summary_data)?;
                let summary_file = rv.enter(session, &p.summary_file)?;
                async {
                    file_metadata::run(&mut assess_data, assess_file, [ER_FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut general_data, general_file, [ER_FILES[1]]).await?;
                };
                async {
                    file_metadata::run(&mut imaging_data, imaging_file, [ER_FILES[2]]).await?;
                };
                async {
                    file_metadata::run(&mut referral_data, referral_file, [CARDIO_FILES[0]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut cardiac_data, cardiac_file, [CARDIO_FILES[1]]).await?;
                };
                async {
                    file_metadata::run(&mut findings_data, findings_file, [CARDIO_FILES[2]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut diagnosis_data, diagnosis_file, [CARDIO_FILES[3]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut review_data, review_file, [PHARMACY_FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut summary_data, summary_file, [ER_FILES[3]]).await?;
                };
                let (
                    (assess_file, general_file, imaging_file, summary_file),
                    (referral_file, cardiac_file, findings_file, diagnosis_file),
                    review_file,
                ) = async {
                    async {
                        let [mut assess, mut panel, mut imaging, mut summary] = er.par()?;
                        async { assess.recv::<Loan<ASSESS_FILE>>().await };
                        async { panel.recv::<Loan<GENERAL_FILE>>().await };
                        async { imaging.recv::<Loan<IMAGING_FILE>>().await };
                        async { summary.recv::<Loan<SUMMARY_FILE>>().await };
                    };
                    async {
                        let [mut referral, mut panel, mut findings, mut diagnosis] =
                            cardiologist.par()?;
                        async { referral.recv::<Loan<REFERRAL_FILE>>().await };
                        async { panel.recv::<Loan<CARDIAC_FILE>>().await };
                        async { findings.recv::<Loan<FINDINGS_FILE>>().await };
                        async { diagnosis.recv::<Loan<DIAGNOSIS_FILE>>().await };
                    };
                    async { pharmacist.recv::<Loan<REVIEW_FILE>>().await };
                }
                .await?;
                async {
                    localside::records::run(&mut records).await?;
                };
                async {
                    localside::services::general(
                        &mut general_service,
                        (general_panel, &general_input),
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::services::cardiac(
                        &mut cardiac_service,
                        (cardiac_panel, &cardiac_input),
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::services::imaging(&mut imaging_service, ct, &simulator).await?;
                };
                async {
                    localside::services::pharmacy(&mut pharmacy_service, &simulator).await?;
                };
                async {
                    localside::er_doctor::run(&mut er).await?;
                };
                async {
                    localside::cardiologist::run(&mut cardiologist).await?;
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
                    let mut imaging_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut imaging_result = vec![0; incoming_capacity];
                    let mut summary_result = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(er_vm)
                        .file(
                            ChoreoFsFile::new(ER_FILES[0], assess_file)
                                .write_as::<AssessmentRequest>(&mut assess_request)
                                .read_as::<FileAssessmentResult>(&mut assess_result),
                        )?
                        .file(
                            ChoreoFsFile::new(ER_FILES[1], general_file)
                                .write_as::<GeneralFileRequest>(&mut general_request)
                                .read_as::<FileGeneralResult>(&mut general_result),
                        )?
                        .file(
                            ChoreoFsFile::new(ER_FILES[2], imaging_file)
                                .write_as::<ImagingIntent>(&mut imaging_request)
                                .read_as::<FileImagingResult>(&mut imaging_result),
                        )?
                        .file(
                            ChoreoFsFile::new(ER_FILES[3], summary_file)
                                .read_as::<Summary>(&mut summary_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (assess, (general, (imaging, (summary, ()))))) =
                        files.into_parts();
                    async {
                        async {
                            assess.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            general.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            imaging.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            summary.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let mut referral_result = vec![0; incoming_capacity];
                    let mut cardiac_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut cardiac_result = vec![0; incoming_capacity];
                    let mut findings_result = vec![0; incoming_capacity];
                    let mut diagnosis_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut diagnosis_result = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(cardio_vm)
                        .file(
                            ChoreoFsFile::new(CARDIO_FILES[0], referral_file)
                                .read_as::<FileReferral>(&mut referral_result),
                        )?
                        .file(
                            ChoreoFsFile::new(CARDIO_FILES[1], cardiac_file)
                                .write_as::<CardiacFileRequest>(&mut cardiac_request)
                                .read_as::<FileCardiacResult>(&mut cardiac_result),
                        )?
                        .file(
                            ChoreoFsFile::new(CARDIO_FILES[2], findings_file)
                                .read_as::<FileFindings>(&mut findings_result),
                        )?
                        .file(
                            ChoreoFsFile::new(CARDIO_FILES[3], diagnosis_file)
                                .write_as::<DiagnosisRequest>(&mut diagnosis_request)
                                .read_as::<FileDiagnosisResult>(&mut diagnosis_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (referral, (cardiac, (findings, (diagnosis, ()))))) =
                        files.into_parts();
                    async {
                        async {
                            referral.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            cardiac.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            findings.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            diagnosis.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let mut review_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut review_result = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(pharmacy_vm).file(
                        ChoreoFsFile::new(PHARMACY_FILES[0], review_file)
                            .write_as::<ReviewRequest>(&mut review_request)
                            .read_as::<FileReviewResult>(&mut review_result),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (review, ())) = files.into_parts();
                    review.into_endpoint().send::<FileExit>(&status).await?;
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
