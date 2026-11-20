//! Direct file projections for the seven participants in the multi-round study.
pub mod global;
pub mod imp;
mod localside;

use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    imp::{bus::Bus, private_fs::PrivateFs},
    localside::{file_metadata, private_io},
};
use global::*;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            chem_terminal,
            bio_terminal,
            tox_terminal,
            form_terminal,
            clin_terminal,
            reg_terminal,
            dir_terminal,
        ],
    ) = Bus::open(
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
            CHEM_FILE,
            CHEM_VM,
            BIO_FILE,
            BIO_VM,
            TOX_FILE,
            TOX_VM,
            FORM_FILE,
            FORM_VM,
            CLIN_FILE,
            CLIN_VM,
            REG_FILE,
            REG_VM,
            DIR_FILE,
            DIR_VM,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let programs = programs();
        bus.projected(programs.observation());
        let capture = |event| {
            bus.capture(event);
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rendezvous = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rendezvous.observe(&capture);
        let observations = rendezvous.tap();
        let result = async {
            let mut bio_data = rendezvous.enter(session, &programs.bio_data)?;
            let mut bio_io = rendezvous.enter(session, &programs.bio_io)?;
            let mut biologist = rendezvous.enter(session, &programs.biologist)?;
            let mut cell_service = rendezvous.enter(session, &programs.cell_service)?;
            let mut chem_data = rendezvous.enter(session, &programs.chem_data)?;
            let mut chem_io = rendezvous.enter(session, &programs.chem_io)?;
            let mut chemist = rendezvous.enter(session, &programs.chemist)?;
            let mut clin_data = rendezvous.enter(session, &programs.clin_data)?;
            let mut clin_io = rendezvous.enter(session, &programs.clin_io)?;
            let mut clinician = rendezvous.enter(session, &programs.clinician)?;
            let mut dir_data = rendezvous.enter(session, &programs.dir_data)?;
            let mut dir_io = rendezvous.enter(session, &programs.dir_io)?;
            let mut director = rendezvous.enter(session, &programs.director)?;
            let mut form_data = rendezvous.enter(session, &programs.form_data)?;
            let mut form_io = rendezvous.enter(session, &programs.form_io)?;
            let mut form_service = rendezvous.enter(session, &programs.form_service)?;
            let mut formulator = rendezvous.enter(session, &programs.formulator)?;
            let mut hplc_service = rendezvous.enter(session, &programs.hplc_service)?;
            let mut init = rendezvous.enter(session, &programs.init)?;
            let mut mass_service = rendezvous.enter(session, &programs.mass_service)?;
            let mut reg_data = rendezvous.enter(session, &programs.reg_data)?;
            let mut reg_io = rendezvous.enter(session, &programs.reg_io)?;
            let mut regulator = rendezvous.enter(session, &programs.regulator)?;
            let mut study_service = rendezvous.enter(session, &programs.study_service)?;
            let mut tox_data = rendezvous.enter(session, &programs.tox_data)?;
            let mut tox_io = rendezvous.enter(session, &programs.tox_io)?;
            let mut toxicologist = rendezvous.enter(session, &programs.toxicologist)?;
            let bio_file = rendezvous.enter(session, &programs.bio_file)?;
            let bio_vm = rendezvous.enter(session, &programs.bio_vm)?;
            let bioassay = rendezvous.enter(session, &programs.bioassay)?;
            let cell = rendezvous.enter(session, &programs.cell)?;
            let chem_file = rendezvous.enter(session, &programs.chem_file)?;
            let chem_vm = rendezvous.enter(session, &programs.chem_vm)?;
            let clin_file = rendezvous.enter(session, &programs.clin_file)?;
            let clin_vm = rendezvous.enter(session, &programs.clin_vm)?;
            let culture = rendezvous.enter(session, &programs.culture)?;
            let dir_file = rendezvous.enter(session, &programs.dir_file)?;
            let dir_vm = rendezvous.enter(session, &programs.dir_vm)?;
            let form = rendezvous.enter(session, &programs.form)?;
            let form_file = rendezvous.enter(session, &programs.form_file)?;
            let form_vm = rendezvous.enter(session, &programs.form_vm)?;
            let formulation = rendezvous.enter(session, &programs.formulation)?;
            let hplc = rendezvous.enter(session, &programs.hplc)?;
            let mass = rendezvous.enter(session, &programs.mass)?;
            let purity = rendezvous.enter(session, &programs.purity)?;
            let qc = rendezvous.enter(session, &programs.qc)?;
            let reg_file = rendezvous.enter(session, &programs.reg_file)?;
            let reg_vm = rendezvous.enter(session, &programs.reg_vm)?;
            let samples = rendezvous.enter(session, &programs.samples)?;
            let stability = rendezvous.enter(session, &programs.stability)?;
            let study = rendezvous.enter(session, &programs.study)?;
            let tox_file = rendezvous.enter(session, &programs.tox_file)?;
            let tox_vm = rendezvous.enter(session, &programs.tox_vm)?;
            let toxicity = rendezvous.enter(session, &programs.toxicity)?;

            async {
                let [
                    mut chem,
                    mut bio,
                    mut tox,
                    mut form,
                    mut clin,
                    mut reg,
                    mut dir,
                ] = init.par()?;
                async { chem.send::<Loan<CHEM_VM>>(chem_vm).await };
                async { bio.send::<Loan<BIO_VM>>(bio_vm).await };
                async { tox.send::<Loan<TOX_VM>>(tox_vm).await };
                async { form.send::<Loan<FORM_VM>>(form_vm).await };
                async { clin.send::<Loan<CLIN_VM>>(clin_vm).await };
                async { reg.send::<Loan<REG_VM>>(reg_vm).await };
                async { dir.send::<Loan<DIR_VM>>(dir_vm).await };
                Ok(())
            };

            async {
                localside::hplc::run(&mut hplc_service, hplc, &simulator).await?;
            };
            async {
                localside::mass::run(&mut mass_service, mass, &simulator).await?;
            };
            async {
                localside::cells::run(&mut cell_service, (cell, samples), &simulator).await?;
            };
            async {
                localside::formulation::run(&mut form_service, form, &simulator).await?;
            };
            async {
                localside::study::run(&mut study_service, &simulator).await?;
            };

            async {
                file_metadata::run(&mut chem_data, chem_file, CHEM_FILES).await?;
            };
            async {
                private_io::run(
                    &mut chem_io,
                    PrivateFs::new(chem_terminal, bus, &CHEM_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = chemist.par()?;
                    async { file.recv::<Loan<CHEM_FILE>>().await };
                    async { vm.recv::<Loan<CHEM_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::chemist::run(&mut chemist, qc).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut request_1 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let mut reply_2 = vec![0; incoming_capacity];
                        let mut request_3 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_3 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(CHEM_FILES[0], file)
                            .write_as::<FileRequest<0>>(&mut request_0)
                            .read_as::<FileResult<0>>(&mut reply_0)
                            .file(CHEM_FILES[1])?
                            .write_as::<FileRequest<3>>(&mut request_1)
                            .read_as::<FileResult<3>>(&mut reply_1)
                            .file(CHEM_FILES[2])?
                            .read_as::<StudyFile<CHEM_FILE>>(&mut reply_2)
                            .file(CHEM_FILES[3])?
                            .write_as::<FileRequest<15>>(&mut request_3)
                            .read_as::<FileResult<15>>(&mut reply_3);
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
            };

            async {
                file_metadata::run(&mut bio_data, bio_file, BIO_FILES).await?;
            };
            async {
                private_io::run(&mut bio_io, PrivateFs::new(bio_terminal, bus, &BIO_FILES)).await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = biologist.par()?;
                    async { file.recv::<Loan<BIO_FILE>>().await };
                    async { vm.recv::<Loan<BIO_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::biologist::run(&mut biologist, culture).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut request_1 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let mut request_2 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_2 = vec![0; incoming_capacity];
                        let mut request_3 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_4 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(BIO_FILES[0], file)
                            .write_as::<FileRequest<1>>(&mut request_0)
                            .read_as::<FileResult<1>>(&mut reply_0)
                            .file(BIO_FILES[1])?
                            .write_as::<FileRequest<5>>(&mut request_1)
                            .read_as::<FileResult<5>>(&mut reply_1)
                            .file(BIO_FILES[2])?
                            .write_as::<FileRequest<6>>(&mut request_2)
                            .read_as::<FileResult<6>>(&mut reply_2)
                            .file(BIO_FILES[3])?
                            .write_as::<Publish<5>>(&mut request_3)
                            .file(BIO_FILES[4])?
                            .read_as::<StudyFile<BIO_FILE>>(&mut reply_4);
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
            };

            async {
                file_metadata::run(&mut tox_data, tox_file, TOX_FILES).await?;
            };
            async {
                private_io::run(&mut tox_io, PrivateFs::new(tox_terminal, bus, &TOX_FILES)).await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = toxicologist.par()?;
                    async { file.recv::<Loan<TOX_FILE>>().await };
                    async { vm.recv::<Loan<TOX_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::toxicologist::run(&mut toxicologist).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut request_1 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let mut request_2 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_2 = vec![0; incoming_capacity];
                        let mut request_3 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_4 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(TOX_FILES[0], file)
                            .write_as::<FileRequest<2>>(&mut request_0)
                            .read_as::<FileResult<2>>(&mut reply_0)
                            .file(TOX_FILES[1])?
                            .write_as::<FileRequest<7>>(&mut request_1)
                            .read_as::<FileResult<7>>(&mut reply_1)
                            .file(TOX_FILES[2])?
                            .write_as::<FileRequest<8>>(&mut request_2)
                            .read_as::<FileResult<8>>(&mut reply_2)
                            .file(TOX_FILES[3])?
                            .write_as::<Publish<8>>(&mut request_3)
                            .file(TOX_FILES[4])?
                            .read_as::<StudyFile<TOX_FILE>>(&mut reply_4);
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
            };

            async {
                file_metadata::run(&mut form_data, form_file, FORM_FILES).await?;
            };
            async {
                private_io::run(
                    &mut form_io,
                    PrivateFs::new(form_terminal, bus, &FORM_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = formulator.par()?;
                    async { file.recv::<Loan<FORM_FILE>>().await };
                    async { vm.recv::<Loan<FORM_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::formulator::run(&mut formulator).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(FORM_FILES[0], file)
                            .write_as::<FileRequest<4>>(&mut request_0)
                            .read_as::<FileResult<4>>(&mut reply_0)
                            .file(FORM_FILES[1])?
                            .read_as::<StudyFile<FORM_FILE>>(&mut reply_1);
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
            };

            async {
                file_metadata::run(&mut clin_data, clin_file, CLIN_FILES).await?;
            };
            async {
                private_io::run(
                    &mut clin_io,
                    PrivateFs::new(clin_terminal, bus, &CLIN_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = clinician.par()?;
                    async { file.recv::<Loan<CLIN_FILE>>().await };
                    async { vm.recv::<Loan<CLIN_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::clinician::run(&mut clinician).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(CLIN_FILES[0], file)
                            .write_as::<FileRequest<9>>(&mut request_0)
                            .read_as::<FileResult<9>>(&mut reply_0)
                            .file(CLIN_FILES[1])?
                            .read_as::<StudyFile<CLIN_FILE>>(&mut reply_1);
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
            };

            async {
                file_metadata::run(&mut reg_data, reg_file, REG_FILES).await?;
            };
            async {
                private_io::run(&mut reg_io, PrivateFs::new(reg_terminal, bus, &REG_FILES)).await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = regulator.par()?;
                    async { file.recv::<Loan<REG_FILE>>().await };
                    async { vm.recv::<Loan<REG_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::regulator::run(&mut regulator).await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut request_1 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let mut request_2 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_3 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(REG_FILES[0], file)
                            .write_as::<FileRequest<10>>(&mut request_0)
                            .read_as::<FileResult<10>>(&mut reply_0)
                            .file(REG_FILES[1])?
                            .write_as::<FileRequest<11>>(&mut request_1)
                            .read_as::<FileResult<11>>(&mut reply_1)
                            .file(REG_FILES[2])?
                            .write_as::<Publish<10>>(&mut request_2)
                            .file(REG_FILES[3])?
                            .read_as::<StudyFile<REG_FILE>>(&mut reply_3);
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
            };

            async {
                file_metadata::run(&mut dir_data, dir_file, DIR_FILES).await?;
            };
            async {
                private_io::run(&mut dir_io, PrivateFs::new(dir_terminal, bus, &DIR_FILES)).await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = director.par()?;
                    async { file.recv::<Loan<DIR_FILE>>().await };
                    async { vm.recv::<Loan<DIR_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::director::run((
                            &mut director,
                            study,
                            formulation,
                            bioassay,
                            stability,
                            purity,
                            toxicity,
                        ))
                        .await?;
                    };
                    async {
                        let mut request_0 = vec![0; GUEST_MEMORY_BYTES];
                        let mut reply_0 = vec![0; incoming_capacity];
                        let mut reply_1 = vec![0; incoming_capacity];
                        let file = ChoreoFsFile::new(DIR_FILES[0], file)
                            .write_as::<DecisionRequest>(&mut request_0)
                            .read_as::<DecisionResult>(&mut reply_0)
                            .file(DIR_FILES[1])?
                            .read_as::<StudyFile<DIR_FILE>>(&mut reply_1);
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
            };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    io.finish(bus, result, None).await
}
