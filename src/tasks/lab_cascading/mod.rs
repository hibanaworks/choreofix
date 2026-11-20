//! 13H's scientific choreography and direct resource localsides.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod analyst;
    pub mod biologist;
    pub mod centrifuge;
    pub mod chemist;
    pub mod chromatograph;
    pub mod fume;
    pub mod inspection;
    pub mod quality;
    pub mod safety;
    pub mod spectrometer;
}

use crate::shell::{
    GUEST_MEMORY_BYTES, ShellError, ShellResult,
    imp::{bus::Bus, private_fs::PrivateFs},
    localside::{file_metadata, private_io},
};
use global::*;
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            chem_terminal,
            bio_terminal,
            analyst_terminal,
            qc_terminal,
            safety_terminal,
        ],
    ) = Bus::open(
        [
            "CHEMIST",
            "BIOLOGIST",
            "ANALYST",
            "QC_CONTROLLER",
            "SAFETY_OFFICER",
        ],
        [
            (session, CHEMIST),
            (session, BIOLOGIST),
            (session, SPECTROMETER),
            (session, CENTRIFUGE),
            (session, FUME_HOOD),
            (session, CHROMATOGRAPH),
            (session, REAGENT_SUPPLY),
            (session, CHEM_FILE),
            (session, BIO_FILE),
            (session, ANALYST_FILE),
            (session, QC_FILE),
            (session, SAFETY_FILE),
            (session, CHEM_VM),
            (session, BIO_VM),
            (session, ANALYST_VM),
            (session, QC_VM),
            (session, SAFETY_VM),
        ],
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let synthesis = Cell::new(Value::Null);
        let quality = Cell::new(Value::Null);
        let capture = |event| {
            bus.capture(event);
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.fume_owner,
            ResolverRef::<SYNTHESIS_OUTCOME>::decision_state(&synthesis, imp::decide),
        )
        .map_err(|e| ShellError::message(format!("synthesis resolver: {e:?}")))?;
        rv.set_resolver(
            &p.fume_owner,
            ResolverRef::<SYNTHESIS_FAILURE>::decision_state(
                &synthesis,
                imp::decide_synthesis_failure,
            ),
        )
        .map_err(|e| ShellError::message(format!("failure resolver: {e:?}")))?;
        rv.set_resolver(
            &p.fume_owner,
            ResolverRef::<QUALITY_OUTCOME>::decision_state(&quality, imp::decide),
        )
        .map_err(|e| ShellError::message(format!("quality resolver: {e:?}")))?;
        let result = async {
            let mut init = rv.enter(session, &p.init)?;
            let synthesis_endpoint = rv.enter(session, &p.chemist)?;
            let assay_endpoint = rv.enter(session, &p.biologist)?;
            let spec = rv.enter(session, &p.spectrometer)?;
            let cent = rv.enter(session, &p.centrifuge)?;
            let hood = rv.enter(session, &p.fume_hood)?;
            let chroma = rv.enter(session, &p.chromatograph)?;
            let stock = rv.enter(session, &p.reagent_supply)?;
            let mut chem = rv.enter(session, &p.chem_control)?;
            let mut bio = rv.enter(session, &p.bio_control)?;
            let mut analyst = rv.enter(session, &p.analyst)?;
            let mut qc = rv.enter(session, &p.qc)?;
            let mut safety = rv.enter(session, &p.safety_officer)?;
            let mut spec_owner = rv.enter(session, &p.spec_owner)?;
            let mut cent_owner = rv.enter(session, &p.cent_owner)?;
            let mut fume_owner = rv.enter(session, &p.fume_owner)?;
            let mut chroma_owner = rv.enter(session, &p.chroma_owner)?;
            let mut inspection = rv.enter(session, &p.inspection_service)?;
            let mut chem_io = rv.enter(session, &p.chem_io)?;
            let mut chem_data = rv.enter(session, &p.chem_data)?;
            let chem_file = rv.enter(session, &p.chem_file)?;
            let mut bio_io = rv.enter(session, &p.bio_io)?;
            let mut bio_data = rv.enter(session, &p.bio_data)?;
            let bio_file = rv.enter(session, &p.bio_file)?;
            let mut analyst_io = rv.enter(session, &p.analyst_io)?;
            let mut analyst_data = rv.enter(session, &p.analyst_data)?;
            let analyst_file = rv.enter(session, &p.analyst_file)?;
            let mut qc_io = rv.enter(session, &p.qc_io)?;
            let mut qc_data = rv.enter(session, &p.qc_data)?;
            let qc_file = rv.enter(session, &p.qc_file)?;
            let mut safety_io = rv.enter(session, &p.safety_io)?;
            let mut safety_data = rv.enter(session, &p.safety_data)?;
            let safety_file = rv.enter(session, &p.safety_file)?;
            let chem_vm = rv.enter(session, &p.chem_vm)?;
            let bio_vm = rv.enter(session, &p.bio_vm)?;
            let analyst_vm = rv.enter(session, &p.analyst_vm)?;
            let qc_vm = rv.enter(session, &p.qc_vm)?;
            let safety_vm = rv.enter(session, &p.safety_vm)?;
            async {
                async {
                    let [
                        mut a,
                        mut b,
                        mut c,
                        mut d,
                        mut e,
                        mut f,
                        mut g,
                        mut chem_root,
                        mut bio_root,
                        mut analyst_root,
                        mut qc_root,
                        mut safety_root,
                    ] = init.par()?;
                    async {
                        a.send::<Loan<SPECTROMETER>>(spec).await?;
                    };
                    async {
                        b.send::<Loan<CENTRIFUGE>>(cent).await?;
                    };
                    async {
                        c.send::<Loan<FUME_HOOD>>(hood).await?;
                    };
                    async {
                        d.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                    };
                    async {
                        e.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                    };
                    async {
                        f.send::<Loan<CHEMIST>>(synthesis_endpoint).await?;
                    };
                    async {
                        g.send::<Loan<BIOLOGIST>>(assay_endpoint).await?;
                    };
                    async {
                        chem_root.send::<Loan<CHEM_VM>>(chem_vm).await?;
                    };
                    async {
                        bio_root.send::<Loan<BIO_VM>>(bio_vm).await?;
                    };
                    async {
                        analyst_root.send::<Loan<ANALYST_VM>>(analyst_vm).await?;
                    };
                    async {
                        qc_root.send::<Loan<QC_VM>>(qc_vm).await?;
                    };
                    async {
                        safety_root.send::<Loan<SAFETY_VM>>(safety_vm).await?;
                    };
                    Ok(())
                }
                .await?;
                drop(init.offer().await?);
                async {
                    let [mut a, mut b, mut c] = init.par()?;
                    async {
                        drop(a.recv::<Loan<CHEMIST>>().await?);
                    };
                    async {
                        drop(b.recv::<Loan<BIOLOGIST>>().await?);
                    };
                    async {
                        drop(c.recv::<Loan<REAGENT_SUPPLY>>().await?);
                    };
                    Ok(())
                }
                .await?;
            };
            async {
                localside::spectrometer::run(&mut spec_owner, &simulator).await?;
            };
            async {
                localside::centrifuge::run(&mut cent_owner, &simulator).await?;
            };
            async {
                localside::fume::run(&mut fume_owner, &simulator, (&synthesis, &quality)).await?;
            };
            async {
                localside::chromatograph::run(&mut chroma_owner, &simulator).await?;
            };
            async {
                localside::inspection::run(&mut inspection, &simulator).await?;
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
                file_metadata::run(&mut safety_data, safety_file, SAFETY_FILES).await?;
            };
            async {
                private_io::run(
                    &mut chem_io,
                    PrivateFs::new(chem_terminal, bus, &CHEM_FILES),
                )
                .await?;
            };
            async {
                private_io::run(&mut bio_io, PrivateFs::new(bio_terminal, bus, &BIO_FILES)).await?;
            };
            async {
                private_io::run(
                    &mut analyst_io,
                    PrivateFs::new(analyst_terminal, bus, &ANALYST_FILES),
                )
                .await?;
            };
            async {
                private_io::run(&mut qc_io, PrivateFs::new(qc_terminal, bus, &QC_FILES)).await?;
            };
            async {
                private_io::run(
                    &mut safety_io,
                    PrivateFs::new(safety_terminal, bus, &SAFETY_FILES),
                )
                .await?;
            };
            async {
                let (file, synthesis, root) = async {
                    let [mut file, mut source, mut wasm] = chem.par()?;
                    async { file.recv::<Loan<CHEM_FILE>>().await };
                    async { source.recv::<Loan<CHEMIST>>().await };
                    async { wasm.recv::<Loan<CHEM_VM>>().await };
                }
                .await?;
                async {
                    localside::chemist::run(&mut chem, synthesis).await?;
                };
                async {
                    let [mut request_0, mut request_1, mut request_2, mut request_5] =
                        [(); 4].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                    let [
                        mut reply_0,
                        mut reply_1,
                        mut reply_2,
                        mut reply_3,
                        mut reply_4,
                    ] = [(); 5].map(|_| vec![0; incoming_capacity]);
                    let file = ChoreoFsFile::new(CHEM_FILES[0], file)
                        .write_as::<FileSynthesis>(&mut request_0)
                        .read_as::<FileSynthesisResult>(&mut reply_0)
                        .file(CHEM_FILES[1])?
                        .write_as::<FileAnalysis>(&mut request_1)
                        .read_as::<FileAnalysisResult>(&mut reply_1)
                        .file(CHEM_FILES[2])?
                        .write_as::<FilePurification>(&mut request_2)
                        .read_as::<FilePurificationResult>(&mut reply_2)
                        .file(CHEM_FILES[3])?
                        .read_as::<FileChemQuality>(&mut reply_3)
                        .file(CHEM_FILES[4])?
                        .write_as::<FileFinal>(&mut request_5)
                        .read_as::<FileFinalResult>(&mut reply_4);
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
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
                let (file, assay, root) = async {
                    let [mut file, mut source, mut wasm] = bio.par()?;
                    async { file.recv::<Loan<BIO_FILE>>().await };
                    async { source.recv::<Loan<BIOLOGIST>>().await };
                    async { wasm.recv::<Loan<BIO_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::biologist::run(&mut bio, assay).await?;
                        Ok(())
                    };
                    async {
                        let [
                            mut request_0,
                            mut request_1,
                            mut request_3,
                            mut request_4,
                            mut request_5,
                        ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                        ] = [(); 6].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(BIO_FILES[0], file)
                            .write_as::<FileCells>(&mut request_0)
                            .read_as::<FileCellsResult>(&mut reply_0)
                            .file(BIO_FILES[1])?
                            .write_as::<FileCalibration>(&mut request_1)
                            .read_as::<FileCalibrationResult>(&mut reply_1)
                            .file(BIO_FILES[2])?
                            .read_as::<FileBioBatch>(&mut reply_2)
                            .file(BIO_FILES[3])?
                            .write_as::<FileBioassay>(&mut request_3)
                            .read_as::<FileBioassayResult>(&mut reply_3)
                            .file(BIO_FILES[4])?
                            .write_as::<FilePublishAnalyst>(&mut request_4)
                            .read_as::<FilePublishedAnalyst>(&mut reply_4)
                            .file(BIO_FILES[5])?
                            .write_as::<FilePublishQc>(&mut request_5)
                            .read_as::<FilePublishedQc>(&mut reply_5);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        Ok((file.into_endpoint(), status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
                Ok(())
            };
            async {
                let (file, root) = async {
                    let [mut file, mut wasm] = analyst.par()?;
                    async { file.recv::<Loan<ANALYST_FILE>>().await };
                    async { wasm.recv::<Loan<ANALYST_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::analyst::run(&mut analyst).await?;
                        Ok(())
                    };
                    async {
                        let [mut request_1, mut request_2, mut request_3] =
                            [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [mut reply_0, mut reply_1, mut reply_2, mut reply_3] =
                            [(); 4].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(ANALYST_FILES[0], file)
                            .read_as::<FileAnalystBatch>(&mut reply_0)
                            .file(ANALYST_FILES[1])?
                            .write_as::<FileChromatography>(&mut request_1)
                            .read_as::<FileChromatographyResult>(&mut reply_1)
                            .file(ANALYST_FILES[2])?
                            .write_as::<FileMolecular>(&mut request_2)
                            .read_as::<FileMolecularResult>(&mut reply_2)
                            .file(ANALYST_FILES[3])?
                            .write_as::<FileVerification>(&mut request_3)
                            .read_as::<FileVerificationResult>(&mut reply_3);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        Ok((file.into_endpoint(), status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
                Ok(())
            };
            async {
                let (file, root) = async {
                    let [mut file, mut wasm] = qc.par()?;
                    async { file.recv::<Loan<QC_FILE>>().await };
                    async { wasm.recv::<Loan<QC_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::quality::run(&mut qc).await?;
                        Ok(())
                    };
                    async {
                        let [mut request_1] = [(); 1].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [mut reply_0, mut reply_1, mut reply_2] =
                            [(); 3].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(QC_FILES[0], file)
                            .read_as::<FileQcInput>(&mut reply_0)
                            .file(QC_FILES[1])?
                            .write_as::<FileQuality>(&mut request_1)
                            .read_as::<FileQualityResult>(&mut reply_1)
                            .file(QC_FILES[2])?
                            .read_as::<FileQcQuality>(&mut reply_2);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        Ok((file.into_endpoint(), status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
                Ok(())
            };
            async {
                let (file, root) = async {
                    let [mut file, mut wasm] = safety.par()?;
                    async { file.recv::<Loan<SAFETY_FILE>>().await };
                    async { wasm.recv::<Loan<SAFETY_VM>>().await };
                }
                .await?;
                async {
                    localside::safety::run(&mut safety).await?;
                };
                async {
                    let [mut request_0] = [(); 1].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                    let [mut reply_0] = [(); 1].map(|_| vec![0; incoming_capacity]);
                    let file = ChoreoFsFile::new(SAFETY_FILES[0], file)
                        .write_as::<FileInspection>(&mut request_0)
                        .read_as::<FileInspectionResult>(&mut reply_0);
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
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
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    let message = match &result {
        Ok(()) => json!({"type":"run_result","status":"complete"}),
        Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
    };
    io.emit(bus, message).await?;
    result
}
