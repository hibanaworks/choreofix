//! Each operation file owns the actual projected request Endpoint.
pub mod global;
mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
mod wire;

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
    let (mut connection, mut io, [a_terminal, b_terminal, integrator_terminal]) = Bus::open(
        ["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
        [
            (session, CORE_LIB),
            (session, SHARED_TYPES),
            (session, A_FILE),
            (session, B_FILE),
            (session, INTEGRATOR_FILE),
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
                let mut integrator = rv.enter(session, &p.integrator)?;
                let mut service = rv.enter(session, &p.service)?;
                let core = rv.enter(session, &p.core)?;
                let types = rv.enter(session, &p.types)?;
                let mut a_data = rv.enter(session, &p.a_data)?;
                let a_file = rv.enter(session, &p.a_file)?;
                let a_vm = rv.enter(session, &p.a_vm)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_data = rv.enter(session, &p.b_data)?;
                let b_file = rv.enter(session, &p.b_file)?;
                let b_vm = rv.enter(session, &p.b_vm)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut integrator_data = rv.enter(session, &p.integrator_data)?;
                let integrator_file = rv.enter(session, &p.integrator_file)?;
                let integrator_vm = rv.enter(session, &p.integrator_vm)?;
                let mut integrator_io = rv.enter(session, &p.integrator_io)?;

                async {
                    file_metadata::run(&mut a_data, a_file, [FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut b_data, b_file, [FILES[0], FILES[2]]).await?;
                };
                async {
                    file_metadata::run(&mut integrator_data, integrator_file, [FILES[1]]).await?;
                };

                let (a_file, b_file, integrator_file) = async {
                    let [mut a, mut b, mut integrator] = service.par()?;
                    async { a.recv::<Loan<A_FILE>>().await };
                    async { b.recv::<Loan<B_FILE>>().await };
                    async { integrator.recv::<Loan<INTEGRATOR_FILE>>().await };
                }
                .await?;
                async {
                    localside::resources::run(&mut service, (core, types)).await?;
                };
                async {
                    localside::builder::run(&mut a, &simulator).await?;
                };
                async {
                    localside::builder::run(&mut b, &simulator).await?;
                };
                async {
                    localside::integrator::run(&mut integrator, &simulator).await?;
                };
                async {
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(a_vm).file(
                        ChoreoFsFile::new(FILES[0], a_file)
                            .write_as::<CompileA>(&mut request)
                            .read_as::<Compiled>(&mut result),
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
                async {
                    private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &FILES[..1]))
                        .await?;
                };
                async {
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let mut updates = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(b_vm).file(
                        ChoreoFsFile::new(FILES[0], b_file)
                            .write_as::<CompileB>(&mut request)
                            .read_as::<Compiled>(&mut result)
                            .file(FILES[2])?
                            .read_as::<TypesUpdated>(&mut updates),
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
                async {
                    private_io::run(
                        &mut b_io,
                        PrivateFs::new(b_terminal, bus, &[FILES[0], FILES[2]]),
                    )
                    .await?;
                };
                async {
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(integrator_vm).file(
                        ChoreoFsFile::new(FILES[1], integrator_file)
                            .write_as::<Link>(&mut request)
                            .read_as::<Linked>(&mut result),
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
                async {
                    private_io::run(
                        &mut integrator_io,
                        PrivateFs::new(integrator_terminal, bus, &FILES[1..2]),
                    )
                    .await?;
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
        Ok(()) => {
            io.emit(
                bus,
                serde_json::json!({"type":"run_result","status":"complete"}),
            )
            .await?
        }
        Err(error) => {
            io.emit(
                bus,
                serde_json::json!({
                    "type":"run_result","status":"failed","error":error.to_string()
                }),
            )
            .await?
        }
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
