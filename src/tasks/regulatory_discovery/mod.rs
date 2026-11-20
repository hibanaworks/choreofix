//! Direct ChoreoFS projections for the five participants in the regulatory study.
#[path = "../drug_discovery/arguments.rs"]
mod arguments;
#[path = "../drug_discovery/imp.rs"]
mod receipts;

pub mod global;
mod imp;
mod reanalysis;
mod localside {
    pub mod biologist;
    pub mod chemist;
    pub mod instruments;
    pub mod lead;
    pub mod regulator;
    pub mod review_owner;
    pub mod toxicologist;
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
            tox_terminal,
            lead_terminal,
            reg_terminal,
        ],
    ) = Bus::open(
        [
            "CHEMIST",
            "BIOLOGIST",
            "TOXICOLOGIST",
            "LEAD_SCIENTIST",
            "REGULATORY_SPECIALIST",
        ],
        [
            HPLC,
            MASS,
            CELL,
            SYNTHESIS_VIEW,
            CULTURE_VIEW,
            ASSAY_VIEW,
            STABILITY_VIEW,
            PURITY_VIEW,
            TOXICITY_VIEW,
            STRUCTURE_VIEW,
            STUDY,
            CHEM_FILE,
            CHEM_VM,
            BIO_FILE,
            BIO_VM,
            TOX_FILE,
            TOX_VM,
            LEAD_FILE,
            LEAD_VM,
            REG_FILE,
            REG_VM,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let programs = programs();
        bus.projected(programs.observation());
        let selection = Cell::new(Value::Null);
        let capture = |event| {
            bus.capture(event);
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rendezvous = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rendezvous.observe(&capture);
        let observations = rendezvous.tap();
        rendezvous
            .set_resolver(
                &programs.lead,
                ResolverRef::<SELECT_BIOLOGY>::decision_state(&selection, imp::select_biology),
            )
            .map_err(|e| ShellError::message(format!("biology resolver: {e:?}")))?;
        rendezvous
            .set_resolver(
                &programs.lead,
                ResolverRef::<SELECT_TOXICOLOGY>::decision_state(
                    &selection,
                    imp::select_toxicology,
                ),
            )
            .map_err(|e| ShellError::message(format!("toxicology resolver: {e:?}")))?;

        let result = async {
            let mut init = rendezvous.enter(session, &programs.init)?;
            let mut chem_data = rendezvous.enter(session, &programs.chem_data)?;
            let mut bio_data = rendezvous.enter(session, &programs.bio_data)?;
            let mut tox_data = rendezvous.enter(session, &programs.tox_data)?;
            let mut lead_data = rendezvous.enter(session, &programs.lead_data)?;
            let mut reg_data = rendezvous.enter(session, &programs.reg_data)?;
            let mut chemist = rendezvous.enter(session, &programs.chemist)?;
            let mut biologist = rendezvous.enter(session, &programs.biologist)?;
            let mut toxicologist = rendezvous.enter(session, &programs.toxicologist)?;
            let mut lead = rendezvous.enter(session, &programs.lead)?;
            let mut regulator = rendezvous.enter(session, &programs.regulator)?;
            let mut hplc_owner = rendezvous.enter(session, &programs.hplc_owner)?;
            let mut mass_owner = rendezvous.enter(session, &programs.mass_owner)?;
            let mut cell_owner = rendezvous.enter(session, &programs.cell_owner)?;
            let mut review_owner = rendezvous.enter(session, &programs.review_owner)?;
            let mut chem_io = rendezvous.enter(session, &programs.chem_io)?;
            let mut bio_io = rendezvous.enter(session, &programs.bio_io)?;
            let mut tox_io = rendezvous.enter(session, &programs.tox_io)?;
            let mut lead_io = rendezvous.enter(session, &programs.lead_io)?;
            let mut reg_io = rendezvous.enter(session, &programs.reg_io)?;
            let hplc = rendezvous.enter(session, &programs.hplc)?;
            let mass = rendezvous.enter(session, &programs.mass)?;
            let cell = rendezvous.enter(session, &programs.cell)?;
            let synthesis = rendezvous.enter(session, &programs.synthesis)?;
            let culture = rendezvous.enter(session, &programs.culture)?;
            let assay = rendezvous.enter(session, &programs.assay)?;
            let stability = rendezvous.enter(session, &programs.stability)?;
            let purity = rendezvous.enter(session, &programs.purity)?;
            let toxicity = rendezvous.enter(session, &programs.toxicity)?;
            let structure = rendezvous.enter(session, &programs.structure)?;
            let study = rendezvous.enter(session, &programs.study)?;
            let chem_file = rendezvous.enter(session, &programs.chem_file)?;
            let chem_vm = rendezvous.enter(session, &programs.chem_vm)?;
            let bio_file = rendezvous.enter(session, &programs.bio_file)?;
            let bio_vm = rendezvous.enter(session, &programs.bio_vm)?;
            let tox_file = rendezvous.enter(session, &programs.tox_file)?;
            let tox_vm = rendezvous.enter(session, &programs.tox_vm)?;
            let lead_file = rendezvous.enter(session, &programs.lead_file)?;
            let lead_vm = rendezvous.enter(session, &programs.lead_vm)?;
            let reg_file = rendezvous.enter(session, &programs.reg_file)?;
            let reg_vm = rendezvous.enter(session, &programs.reg_vm)?;
            async {
                let [
                    mut hplc_lane,
                    mut mass_lane,
                    mut cell_lane,
                    mut chem_vm_lane,
                    mut bio_vm_lane,
                    mut tox_vm_lane,
                    mut lead_vm_lane,
                    mut reg_vm_lane,
                ] = init.par()?;
                async {
                    hplc_lane.send::<Loan<HPLC>>(hplc).await?;
                };
                async {
                    mass_lane.send::<Loan<MASS>>(mass).await?;
                };
                async {
                    cell_lane.send::<Loan<CELL>>(cell).await?;
                };
                async {
                    chem_vm_lane.send::<Loan<CHEM_VM>>(chem_vm).await?;
                };
                async {
                    bio_vm_lane.send::<Loan<BIO_VM>>(bio_vm).await?;
                };
                async {
                    tox_vm_lane.send::<Loan<TOX_VM>>(tox_vm).await?;
                };
                async {
                    lead_vm_lane.send::<Loan<LEAD_VM>>(lead_vm).await?;
                };
                async {
                    reg_vm_lane.send::<Loan<REG_VM>>(reg_vm).await?;
                };
                Ok(())
            };
            async {
                localside::instruments::hplc(&mut hplc_owner, &simulator).await?;
            };
            async {
                localside::instruments::mass(&mut mass_owner, &simulator).await?;
            };
            async {
                localside::instruments::cell(&mut cell_owner, &simulator).await?;
            };
            async {
                localside::review_owner::run(&mut review_owner, &simulator).await?;
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
                    let [mut file, mut wasm] = chemist.par()?;
                    async { file.recv::<Loan<CHEM_FILE>>().await };
                    async { wasm.recv::<Loan<CHEM_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::chemist::run(&mut chemist, synthesis).await?;
                    };
                    async {
                        let [mut request_0, mut request_1, mut request_3] =
                            [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [mut reply_0, mut reply_1, mut reply_2, mut reply_3] =
                            [(); 4].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(CHEM_FILES[0], file)
                            .write_as::<Request<SYNTHESIS_QC>>(&mut request_0)
                            .read_as::<Reply<SYNTHESIS_QC>>(&mut reply_0)
                            .file(CHEM_FILES[1])?
                            .write_as::<Request<MOLECULAR_ID>>(&mut request_1)
                            .read_as::<Reply<MOLECULAR_ID>>(&mut reply_1)
                            .file(CHEM_FILES[2])?
                            .read_as::<ApprovalNotice>(&mut reply_2)
                            .file(CHEM_FILES[3])?
                            .write_as::<Request<DOCUMENTATION>>(&mut request_3)
                            .read_as::<Reply<DOCUMENTATION>>(&mut reply_3);
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
                    let [mut file, mut wasm] = biologist.par()?;
                    async { file.recv::<Loan<BIO_FILE>>().await };
                    async { wasm.recv::<Loan<BIO_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::biologist::run(&mut biologist, culture, assay, stability)
                            .await?;
                    };
                    async {
                        let [mut request_0, mut request_2, mut request_3] =
                            [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                        ] = [(); 5].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(BIO_FILES[0], file)
                            .write_as::<Request<CULTURE_PREP>>(&mut request_0)
                            .read_as::<Reply<CULTURE_PREP>>(&mut reply_0)
                            .file(BIO_FILES[1])?
                            .read_as::<BiologicalSample>(&mut reply_1)
                            .file(BIO_FILES[2])?
                            .write_as::<Request<BIOASSAY>>(&mut request_2)
                            .read_as::<Reply<BIOASSAY>>(&mut reply_2)
                            .file(BIO_FILES[3])?
                            .write_as::<Request<STABILITY>>(&mut request_3)
                            .read_as::<Reply<STABILITY>>(&mut reply_3)
                            .file(BIO_FILES[4])?
                            .read_as::<StudyNotice>(&mut reply_4);
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
                    let [mut file, mut wasm] = toxicologist.par()?;
                    async { file.recv::<Loan<TOX_FILE>>().await };
                    async { wasm.recv::<Loan<TOX_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::toxicologist::run(&mut toxicologist, purity, toxicity).await?;
                    };
                    async {
                        let [mut request_0, mut request_2, mut request_3] =
                            [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                        ] = [(); 5].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(TOX_FILES[0], file)
                            .write_as::<Request<BASELINE>>(&mut request_0)
                            .read_as::<Reply<BASELINE>>(&mut reply_0)
                            .file(TOX_FILES[1])?
                            .read_as::<ToxicologySample>(&mut reply_1)
                            .file(TOX_FILES[2])?
                            .write_as::<Request<PURITY>>(&mut request_2)
                            .read_as::<Reply<PURITY>>(&mut reply_2)
                            .file(TOX_FILES[3])?
                            .write_as::<Request<CYTOTOXICITY>>(&mut request_3)
                            .read_as::<Reply<CYTOTOXICITY>>(&mut reply_3)
                            .file(TOX_FILES[4])?
                            .read_as::<StudyNotice>(&mut reply_4);
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
                file_metadata::run(&mut lead_data, lead_file, LEAD_FILES).await?;
            };
            async {
                private_io::run(
                    &mut lead_io,
                    PrivateFs::new(lead_terminal, bus, &LEAD_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut wasm] = lead.par()?;
                    async { file.recv::<Loan<LEAD_FILE>>().await };
                    async { wasm.recv::<Loan<LEAD_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::lead::run(&mut lead, study, (structure, &selection, &simulator))
                            .await?;
                    };
                    async {
                        let [
                            mut request_0,
                            mut request_1,
                            mut request_2,
                            mut request_3,
                            mut request_7,
                        ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                            mut reply_6,
                        ] = [(); 7].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(LEAD_FILES[0], file)
                            .write_as::<Request<REFERENCE>>(&mut request_0)
                            .read_as::<Reply<REFERENCE>>(&mut reply_0)
                            .file(LEAD_FILES[1])?
                            .write_as::<Request<STRUCTURE>>(&mut request_1)
                            .read_as::<Reply<STRUCTURE>>(&mut reply_1)
                            .file(LEAD_FILES[2])?
                            .write_as::<Request<CHARACTERIZATION>>(&mut request_2)
                            .read_as::<Reply<CHARACTERIZATION>>(&mut reply_2)
                            .file(LEAD_FILES[3])?
                            .write_as::<Request<DECISION>>(&mut request_3)
                            .read_as::<Reply<DECISION>>(&mut reply_3)
                            .file(LEAD_FILES[4])?
                            .read_as::<BioReport>(&mut reply_4)
                            .file(LEAD_FILES[5])?
                            .read_as::<ToxReport>(&mut reply_5)
                            .file(LEAD_FILES[6])?
                            .read_as::<ReviewResult>(&mut reply_6)
                            .file(LEAD_FILES[7])?
                            .write_as::<Request<REANALYSIS>>(&mut request_7);
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
                    let [mut file, mut wasm] = regulator.par()?;
                    async { file.recv::<Loan<REG_FILE>>().await };
                    async { wasm.recv::<Loan<REG_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::regulator::run(&mut regulator).await?;
                    };
                    async {
                        let mut request_1 = vec![0; GUEST_MEMORY_BYTES];
                        let [mut reply_0, mut reply_1] =
                            [(); 2].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(REG_FILES[0], file)
                            .read_as::<Decision>(&mut reply_0)
                            .file(REG_FILES[1])?
                            .write_as::<Request<COMPLIANCE>>(&mut request_1)
                            .read_as::<Reply<COMPLIANCE>>(&mut reply_1);
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
    let message = match &result {
        Ok(()) => json!({"type":"run_result", "status":"complete"}),
        Err(error) => json!({"type":"run_result", "status":"failed", "error":error.to_string()}),
    };
    io.emit(bus, message).await?;
    result
}
