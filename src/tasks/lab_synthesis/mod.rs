//! Three autonomous wash guests execute the single laboratory choreography.
mod arguments;
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::shell::ShellResult;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [chem_terminal, bio_terminal, analyst_terminal]) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "ANALYST"],
        [
            (session, SPECTROMETER),
            (session, CENTRIFUGE),
            (session, CHEM_FILE),
            (session, BIO_FILE),
            (session, ANALYST_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        let result = async {
            let mut chemist = rv.enter(session, &p.chemist)?;
            let mut biologist = rv.enter(session, &p.biologist)?;
            let mut analyst = rv.enter(session, &p.analyst)?;
            let mut spec_owner = rv.enter(session, &p.spec_owner)?;
            let mut cent_owner = rv.enter(session, &p.cent_owner)?;
            let mut init = rv.enter(session, &p.init)?;
            let mut chem_data = rv.enter(session, &p.chem_data)?;
            let mut bio_data = rv.enter(session, &p.bio_data)?;
            let mut analyst_data = rv.enter(session, &p.analyst_data)?;
            let mut chem_vm = rv.enter(session, &p.chem_vm)?;
            let mut bio_vm = rv.enter(session, &p.bio_vm)?;
            let mut analyst_vm = rv.enter(session, &p.analyst_vm)?;
            let mut chem_io = rv.enter(session, &p.chem_io)?;
            let mut bio_io = rv.enter(session, &p.bio_io)?;
            let mut analyst_io = rv.enter(session, &p.analyst_io)?;
            let spec = rv.enter(session, &p.spec)?;
            let cent = rv.enter(session, &p.cent)?;
            let chem_file = rv.enter(session, &p.chem_file)?;
            let bio_file = rv.enter(session, &p.bio_file)?;
            let analyst_file = rv.enter(session, &p.analyst_file)?;
            async {
                async {
                    let [mut spectrum, mut centrifuge] = init.par()?;
                    async {
                        spectrum.send::<SpecLoan>(spec).await?;
                    };
                    async {
                        centrifuge.send::<CentLoan>(cent).await?;
                    };
                    Ok(())
                }
                .await?;
                let cent = init.recv::<CentLoan>().await?;
                let spec = init.recv::<SpecLoan>().await?;
                drop((cent, spec));
            };
            async {
                localside::chemist::run(&mut chemist).await?;
            };
            async {
                localside::biologist::run(&mut biologist, &simulator).await?;
            };
            async {
                localside::analyst::run(&mut analyst).await?;
            };
            async {
                localside::spectrometer::run(&mut spec_owner, &simulator).await?;
            };
            async {
                localside::centrifuge::run(&mut cent_owner, &simulator).await?;
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
            let (chem_file, bio_file, analyst_file) = async {
                async { chem_vm.recv::<Loan<CHEM_FILE>>().await };
                async { bio_vm.recv::<Loan<BIO_FILE>>().await };
                async { analyst_vm.recv::<Loan<ANALYST_FILE>>().await };
            }
            .await?;
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let mut result_2 = vec![0; incoming_capacity];
                let mut request_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_3 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(CHEM_FILES[0], chem_file)
                    .write_as::<Raw>(&mut request_0)
                    .read_as::<FileRawResult>(&mut result_0)
                    .file(CHEM_FILES[1])?
                    .write_as::<Purify>(&mut request_1)
                    .read_as::<FilePurifiedResult>(&mut result_1)
                    .file(CHEM_FILES[2])?
                    .read_as::<FileQuality>(&mut result_2)
                    .file(CHEM_FILES[3])?
                    .write_as::<Final>(&mut request_3)
                    .read_as::<FileFinalResult>(&mut result_3);
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let files = ChoreoFsFiles::new(chem_vm).file(file)?;
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut chem_io,
                    PrivateFs::new(chem_terminal, bus, &CHEM_FILES),
                )
                .await?;
            };
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let mut result_2 = vec![0; incoming_capacity];
                let mut request_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_3 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(BIO_FILES[0], bio_file)
                    .write_as::<Cells>(&mut request_0)
                    .read_as::<FileCellsResult>(&mut result_0)
                    .file(BIO_FILES[1])?
                    .write_as::<Calibrate>(&mut request_1)
                    .read_as::<FileCalibrationResult>(&mut result_1)
                    .file(BIO_FILES[2])?
                    .read_as::<Sample>(&mut result_2)
                    .file(BIO_FILES[3])?
                    .write_as::<Bioassay>(&mut request_3)
                    .read_as::<BioassayResult>(&mut result_3);
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let files = ChoreoFsFiles::new(bio_vm).file(file)?;
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(&mut bio_io, PrivateFs::new(bio_terminal, bus, &BIO_FILES)).await?;
            };
            async {
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let mut request_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_2 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ANALYST_FILES[0], analyst_file)
                    .read_as::<FileAssay>(&mut result_0)
                    .file(ANALYST_FILES[1])?
                    .write_as::<Molecular>(&mut request_1)
                    .read_as::<FileMolecularResult>(&mut result_1)
                    .file(ANALYST_FILES[2])?
                    .write_as::<Verify>(&mut request_2)
                    .read_as::<FileVerificationResult>(&mut result_2);
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let files = ChoreoFsFiles::new(analyst_vm).file(file)?;
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut analyst_io,
                    PrivateFs::new(analyst_terminal, bus, &ANALYST_FILES),
                )
                .await?;
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
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
