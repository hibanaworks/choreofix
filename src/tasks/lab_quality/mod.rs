//! 13M's direct-file choreography, scientific localsides and QC effect boundary.
//! Direct device/QC localsides and four-Wasm execution; qualification is in progress.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod analyst;
    pub mod biologist;
    pub mod centrifuge;
    pub mod chemist;
    pub mod controller;
    pub mod qc;
    pub mod spectrometer;
}
#[cfg(test)]
mod tests;
use crate::shell::ShellResult;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        ShellError,
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana::g::Message;
    use hibana::runtime::resolver::ResolverRef;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [chem_terminal, bio_terminal, analyst_terminal, qc_terminal]) =
        Bus::open(
            ["CHEMIST", "BIOLOGIST", "ANALYST", "QC_CONTROLLER"],
            [
                (session, SPECTROMETER),
                (session, CENTRIFUGE),
                (session, FUME_HOOD),
                (session, CHEMISTRY),
                (session, BIOLOGY),
                (session, CHEM_FILE),
                (session, CHEM_VM),
                (session, BIO_FILE),
                (session, BIO_VM),
                (session, ANALYST_FILE),
                (session, ANALYST_VM),
                (session, QC_FILE),
                (session, QC_VM),
            ],
        );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let qc_actual = core::cell::Cell::new(serde_json::Value::Null);
        let bio_actual = core::cell::Cell::new(serde_json::Value::Null);
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.qc_service,
            ResolverRef::<QC_SUCCESS>::decision_state(&qc_actual, imp::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        rv.set_resolver(
            &p.qc_service,
            ResolverRef::<QC_SUBJECT>::decision_state(&qc_actual, imp::decide_subject),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        rv.set_resolver(
            &p.biology,
            ResolverRef::<BIO_SUCCESS>::decision_state(&bio_actual, imp::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let result = async {
            let mut chemist = rv.enter(session, &p.chemist)?;
            let mut biologist = rv.enter(session, &p.biologist)?;
            let mut analyst = rv.enter(session, &p.analyst)?;
            let mut qc = rv.enter(session, &p.qc)?;
            let mut spec_service = rv.enter(session, &p.spec_service)?;
            let mut cent_service = rv.enter(session, &p.cent_service)?;
            let mut qc_service = rv.enter(session, &p.qc_service)?;
            let mut init = rv.enter(session, &p.init)?;
            let mut chem_data = rv.enter(session, &p.chem_data)?;
            let mut bio_data = rv.enter(session, &p.bio_data)?;
            let mut analyst_data = rv.enter(session, &p.analyst_data)?;
            let mut qc_data = rv.enter(session, &p.qc_data)?;
            let mut chemist_io = rv.enter(session, &p.chemist_io)?;
            let mut biologist_io = rv.enter(session, &p.biologist_io)?;
            let mut analyst_io = rv.enter(session, &p.analyst_io)?;
            let mut qc_io = rv.enter(session, &p.qc_io)?;
            let spectrometer = rv.enter(session, &p.spectrometer)?;
            let centrifuge = rv.enter(session, &p.centrifuge)?;
            let fume_hood = rv.enter(session, &p.fume_hood)?;
            let chemistry = rv.enter(session, &p.chemistry)?;
            let biology = rv.enter(session, &p.biology)?;
            let chem_file = rv.enter(session, &p.chem_file)?;
            let bio_file = rv.enter(session, &p.bio_file)?;
            let analyst_file = rv.enter(session, &p.analyst_file)?;
            let qc_file = rv.enter(session, &p.qc_file)?;
            let chem_vm = rv.enter(session, &p.chem_vm)?;
            let bio_vm = rv.enter(session, &p.bio_vm)?;
            let analyst_vm = rv.enter(session, &p.analyst_vm)?;
            let qc_vm = rv.enter(session, &p.qc_vm)?;
            async {
                async {
                    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g] = init.par()?;
                    async {
                        a.send::<SpecLoan>(spectrometer).await?;
                    };
                    async {
                        b.send::<CentLoan>(centrifuge).await?;
                    };
                    async {
                        c.send::<FumeLoan>(fume_hood).await?;
                    };
                    async {
                        d.send::<Loan<CHEM_VM>>(chem_vm).await?;
                    };
                    async {
                        e.send::<Loan<BIO_VM>>(bio_vm).await?;
                    };
                    async {
                        f.send::<Loan<ANALYST_VM>>(analyst_vm).await?;
                    };
                    async {
                        g.send::<Loan<QC_VM>>(qc_vm).await?;
                    };
                    Ok(())
                }
                .await?;
                let branch = init.offer().await?;
                match branch.label() {
                    ChemAccess::LOGICAL_LABEL => {
                        let chemical = branch.recv::<ChemAccess>().await?;
                        let biological = init.recv::<BioAccess>().await?;
                        drop((chemical, biological));
                    }
                    BioAccess::LOGICAL_LABEL => {
                        let biological = branch.recv::<BioAccess>().await?;
                        let chemical = init.recv::<ChemAccess>().await?;
                        drop((chemical, biological));
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected retired scientific candidate {label}"
                        )));
                    }
                }
                async {
                    let [mut a, mut b, mut c] = init.par()?;
                    async {
                        let returned = a.recv::<SpecLoan>().await?;
                        drop(returned);
                        Ok(())
                    };
                    async {
                        let returned = b.recv::<CentLoan>().await?;
                        drop(returned);
                        Ok(())
                    };
                    async {
                        let returned = c.recv::<FumeLoan>().await?;
                        drop(returned);
                        Ok(())
                    };
                    Ok(())
                }
                .await?;
            };
            async {
                localside::spectrometer::run(&mut spec_service, &simulator).await?;
            };
            async {
                localside::centrifuge::run(&mut cent_service, (biology, &bio_actual), &simulator)
                    .await?;
            };
            async {
                localside::qc::run(&mut qc_service, (chemistry, &simulator, &qc_actual)).await?;
            };
            async {
                file_metadata::run(&mut chem_data, chem_file, CHEM_FILES).await?;
            };
            async {
                file_metadata::run(&mut bio_data, bio_file, BIO_FILES).await?;
            };
            async {
                file_metadata::run(&mut analyst_data, analyst_file, ANALYST_FILES).await?;
            };
            async {
                file_metadata::run(&mut qc_data, qc_file, QC_FILES).await?;
            };
            async {
                let (file, root) = async {
                    let [mut files, mut wasm] = chemist.par()?;
                    async { files.recv::<Loan<CHEM_FILE>>().await };
                    async { wasm.recv::<Loan<CHEM_VM>>().await };
                }
                .await?;
                async {
                    localside::chemist::run(&mut chemist).await?;
                };
                async {
                    let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_0 = vec![0; incoming_capacity];
                    let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; incoming_capacity];
                    let mut request_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; incoming_capacity];
                    let mut result_3 = vec![0; incoming_capacity];
                    let mut result_4 = vec![0; incoming_capacity];
                    let mut request_5 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_5 = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(CHEM_FILES[0], file)
                        .write_as::<FileSynthesis>(&mut request_0)
                        .read_as::<FileSynthesisResult>(&mut result_0)
                        .file(CHEM_FILES[1])?
                        .write_as::<FileAnalysis>(&mut request_1)
                        .read_as::<FileAnalysisResult>(&mut result_1)
                        .file(CHEM_FILES[2])?
                        .write_as::<FilePurification>(&mut request_2)
                        .read_as::<FilePurificationResult>(&mut result_2)
                        .file(CHEM_FILES[3])?
                        .read_as::<FileFeedback>(&mut result_3)
                        .file(CHEM_FILES[4])?
                        .read_as::<FileQuality>(&mut result_4)
                        .file(CHEM_FILES[5])?
                        .write_as::<FileFinal>(&mut request_5)
                        .read_as::<FileFinalResult>(&mut result_5);
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let files = ChoreoFsFiles::new(root).file(file)?;
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<Exit>(&status).await?;
                };
                Ok(())
            };
            async {
                private_io::run(
                    &mut chemist_io,
                    PrivateFs::new(chem_terminal, bus, &CHEM_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut files, mut wasm] = biologist.par()?;
                    async { files.recv::<Loan<BIO_FILE>>().await };
                    async { wasm.recv::<Loan<BIO_VM>>().await };
                }
                .await?;
                async {
                    localside::biologist::run(&mut biologist).await?;
                };
                async {
                    let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_0 = vec![0; incoming_capacity];
                    let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; incoming_capacity];
                    let mut result_2 = vec![0; incoming_capacity];
                    let mut request_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_3 = vec![0; incoming_capacity];
                    let mut result_4 = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(BIO_FILES[0], file)
                        .write_as::<FileCells>(&mut request_0)
                        .read_as::<FileCellsResult>(&mut result_0)
                        .file(BIO_FILES[1])?
                        .write_as::<FileCalibration>(&mut request_1)
                        .read_as::<FileCalibrationResult>(&mut result_1)
                        .file(BIO_FILES[2])?
                        .read_as::<FileSample>(&mut result_2)
                        .file(BIO_FILES[3])?
                        .write_as::<FileBioassay>(&mut request_3)
                        .read_as::<FileBioassayResult>(&mut result_3)
                        .file(BIO_FILES[4])?
                        .read_as::<FileFeedback>(&mut result_4);
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let files = ChoreoFsFiles::new(root).file(file)?;
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<Exit>(&status).await?;
                };
                Ok(())
            };
            async {
                private_io::run(
                    &mut biologist_io,
                    PrivateFs::new(bio_terminal, bus, &BIO_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut files, mut wasm] = analyst.par()?;
                    async { files.recv::<Loan<ANALYST_FILE>>().await };
                    async { wasm.recv::<Loan<ANALYST_VM>>().await };
                }
                .await?;
                async {
                    localside::analyst::run(&mut analyst).await?;
                };
                async {
                    let mut result_0 = vec![0; incoming_capacity];
                    let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; incoming_capacity];
                    let mut request_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(ANALYST_FILES[0], file)
                        .read_as::<FileAssay>(&mut result_0)
                        .file(ANALYST_FILES[1])?
                        .write_as::<FileMolecular>(&mut request_1)
                        .read_as::<FileMolecularResult>(&mut result_1)
                        .file(ANALYST_FILES[2])?
                        .write_as::<FileVerification>(&mut request_2)
                        .read_as::<FileVerificationResult>(&mut result_2);
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let files = ChoreoFsFiles::new(root).file(file)?;
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<Exit>(&status).await?;
                };
                Ok(())
            };
            async {
                private_io::run(
                    &mut analyst_io,
                    PrivateFs::new(analyst_terminal, bus, &ANALYST_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut files, mut wasm] = qc.par()?;
                    async { files.recv::<Loan<QC_FILE>>().await };
                    async { wasm.recv::<Loan<QC_VM>>().await };
                }
                .await?;
                async {
                    localside::controller::run(&mut qc).await?;
                };
                async {
                    let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_0 = vec![0; incoming_capacity];
                    let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; incoming_capacity];
                    let file = ChoreoFsFile::new(QC_FILES[0], file)
                        .write_as::<FileQcChemist>(&mut request_0)
                        .read_as::<FileQcChemistResult>(&mut result_0)
                        .file(QC_FILES[1])?
                        .write_as::<FileQcBiologist>(&mut request_1)
                        .read_as::<FileQcBiologistResult>(&mut result_1);
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let files = ChoreoFsFiles::new(root).file(file)?;
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<Exit>(&status).await?;
                };
                Ok(())
            };
            async {
                private_io::run(&mut qc_io, PrivateFs::new(qc_terminal, bus, &QC_FILES)).await?;
            };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    match &result{
 Ok(())=>io.emit(bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
 Err(error)=>io.emit(bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
    result
}
