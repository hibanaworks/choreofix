//! Direct operation-file Endpoints, two affine machines and actual report data.
pub mod global;
pub(crate) mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::shell::{ShellError, ShellResult};
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
    let (mut connection, mut io, [a_terminal, b_terminal, supervisor_terminal]) = Bus::open(
        ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
        [
            (session, LATHE),
            (session, MILL),
            (session, LATHE_FILE_A),
            (session, LATHE_FILE_B),
            (session, MILL_FILE_A),
            (session, MILL_FILE_B),
            (session, SUPERVISOR_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.projected(p.observation());
            let capture = |row| bus.capture(row);
            let mut slab = vec![0; 2 * 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut supervisor = rv.enter(session, &p.supervisor)?;
                let mut lathe_service = rv.enter(session, &p.lathe_service)?;
                let mut mill_service = rv.enter(session, &p.mill_service)?;
                let lathe = rv.enter(session, &p.lathe)?;
                let mill = rv.enter(session, &p.mill)?;
                let mut lathe_data_a = rv.enter(session, &p.lathe_data_a)?;
                let lathe_file_a = rv.enter(session, &p.lathe_file_a)?;
                let mut lathe_data_b = rv.enter(session, &p.lathe_data_b)?;
                let lathe_file_b = rv.enter(session, &p.lathe_file_b)?;
                let mut mill_data_a = rv.enter(session, &p.mill_data_a)?;
                let mill_file_a = rv.enter(session, &p.mill_file_a)?;
                let mut mill_data_b = rv.enter(session, &p.mill_data_b)?;
                let mill_file_b = rv.enter(session, &p.mill_file_b)?;
                let mut supervisor_data = rv.enter(session, &p.supervisor_data)?;
                let supervisor_file = rv.enter(session, &p.supervisor_file)?;
                let a_vm = rv.enter(session, &p.a_vm)?;
                let b_vm = rv.enter(session, &p.b_vm)?;
                let supervisor_vm = rv.enter(session, &p.supervisor_vm)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut supervisor_io = rv.enter(session, &p.supervisor_io)?;
                async {
                    file_metadata::run(&mut lathe_data_a, lathe_file_a, [JOB_FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut lathe_data_b, lathe_file_b, [JOB_FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut mill_data_a, mill_file_a, [JOB_FILES[1]]).await?;
                };
                async {
                    file_metadata::run(&mut mill_data_b, mill_file_b, [JOB_FILES[1]]).await?;
                };
                async {
                    file_metadata::run(&mut supervisor_data, supervisor_file, [REPORT_FILE])
                        .await?;
                };
                let ((lathe_file_a, lathe_file_b), (mill_file_a, mill_file_b), supervisor_file) =
                    async {
                        async {
                            let [mut a, mut b] = lathe_service.par()?;
                            async { a.recv::<Loan<LATHE_FILE_A>>().await };
                            async { b.recv::<Loan<LATHE_FILE_B>>().await };
                        };
                        async {
                            let [mut a, mut b] = mill_service.par()?;
                            async { a.recv::<Loan<MILL_FILE_A>>().await };
                            async { b.recv::<Loan<MILL_FILE_B>>().await };
                        };
                        async { supervisor.recv::<Loan<SUPERVISOR_FILE>>().await };
                    }
                    .await?;
                async {
                    localside::lathe::run(&mut lathe_service, lathe).await?;
                };
                async {
                    localside::mill::run(&mut mill_service, mill).await?;
                };
                async {
                    localside::technician_a::run(&mut a, &simulator).await?;
                };
                async {
                    localside::technician_b::run(&mut b, &simulator).await?;
                };
                async {
                    localside::supervisor::run(&mut supervisor, &simulator).await?;
                };
                async {
                    private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &JOB_FILES)).await?;
                };
                async {
                    private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &JOB_FILES)).await?;
                };
                async {
                    private_io::run(
                        &mut supervisor_io,
                        PrivateFs::new(supervisor_terminal, bus, &[REPORT_FILE]),
                    )
                    .await?;
                };
                async {
                    let (mut lathe_request, mut mill_request) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut lathe_result, mut mill_result) =
                        (vec![0; incoming_capacity], vec![0; incoming_capacity]);
                    let files = ChoreoFsFiles::new(a_vm)
                        .file(
                            ChoreoFsFile::new(JOB_FILES[0], lathe_file_a)
                                .write_as::<RequestLatheA>(&mut lathe_request)
                                .read_as::<FileLatheResultA>(&mut lathe_result),
                        )?
                        .file(
                            ChoreoFsFile::new(JOB_FILES[1], mill_file_a)
                                .write_as::<RequestMillA>(&mut mill_request)
                                .read_as::<FileMillResultA>(&mut mill_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (lathe, (mill, ()))) = files.into_parts();
                    async {
                        async {
                            lathe.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            mill.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let (mut lathe_request, mut mill_request) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut lathe_result, mut mill_result) =
                        (vec![0; incoming_capacity], vec![0; incoming_capacity]);
                    let files = ChoreoFsFiles::new(b_vm)
                        .file(
                            ChoreoFsFile::new(JOB_FILES[0], lathe_file_b)
                                .write_as::<RequestLatheB>(&mut lathe_request)
                                .read_as::<FileLatheResultB>(&mut lathe_result),
                        )?
                        .file(
                            ChoreoFsFile::new(JOB_FILES[1], mill_file_b)
                                .write_as::<RequestMillB>(&mut mill_request)
                                .read_as::<FileMillResultB>(&mut mill_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (lathe, (mill, ()))) = files.into_parts();
                    async {
                        async {
                            lathe.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            mill.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut response = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(supervisor_vm).file(
                        ChoreoFsFile::new(REPORT_FILE, supervisor_file)
                            .write_as::<CollectReports>(&mut request)
                            .read_as::<Reports>(&mut response),
                    )?;
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
    match &result {
        Ok(()) => io.emit(bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
