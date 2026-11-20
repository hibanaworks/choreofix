//! 10M: overlapping affine build inputs, actual tests and data-bound rebuilding.
pub mod global;
pub(crate) mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
mod wire;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let (
        mut connection,
        mut io,
        [
            a_terminal,
            b_terminal,
            c_terminal,
            integrator_terminal,
            tester_terminal,
        ],
    ) = Bus::open(
        [
            "BUILDER_A",
            "BUILDER_B",
            "BUILDER_C",
            "INTEGRATOR",
            "TEST_RUNNER",
        ],
        [
            (SessionId::new(SESSION), CORE),
            (SessionId::new(SESSION), CONFIG),
            (SessionId::new(SESSION), MODELS),
            (SessionId::new(SESSION), A_FILE),
            (SessionId::new(SESSION), B_FILE),
            (SessionId::new(SESSION), C_FILE),
            (SessionId::new(SESSION), INTEGRATOR_FILE),
            (SessionId::new(SESSION), TESTER_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.projected(p.observation());
            let capture = |row| bus.capture(row);
            let input = core::cell::Cell::new(serde_json::Value::Null);
            let mut slab = vec![0; 2 * 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let session = SessionId::new(SESSION);
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut c = rv.enter(session, &p.c)?;
                let mut integrator = rv.enter(session, &p.integrator)?;
                let mut tester = rv.enter(session, &p.tester)?;
                let mut compiler = rv.enter(session, &p.compiler)?;
                let mut testing = rv.enter(session, &p.testing)?;
                let core = rv.enter(session, &p.core)?;
                let config = rv.enter(session, &p.config)?;
                let models = rv.enter(session, &p.models)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut c_io = rv.enter(session, &p.c_io)?;
                let mut integrator_io = rv.enter(session, &p.integrator_io)?;
                let mut tester_io = rv.enter(session, &p.tester_io)?;
                let mut a_data = rv.enter(session, &p.a_data)?;
                let a_file = rv.enter(session, &p.a_file)?;
                let a_vm = rv.enter(session, &p.a_vm)?;
                let mut b_data = rv.enter(session, &p.b_data)?;
                let b_file = rv.enter(session, &p.b_file)?;
                let b_vm = rv.enter(session, &p.b_vm)?;
                let mut c_data = rv.enter(session, &p.c_data)?;
                let c_file = rv.enter(session, &p.c_file)?;
                let c_vm = rv.enter(session, &p.c_vm)?;
                let mut integrator_data = rv.enter(session, &p.integrator_data)?;
                let integrator_file = rv.enter(session, &p.integrator_file)?;
                let integrator_vm = rv.enter(session, &p.integrator_vm)?;
                let mut tester_data = rv.enter(session, &p.tester_data)?;
                let tester_file = rv.enter(session, &p.tester_file)?;
                let tester_vm = rv.enter(session, &p.tester_vm)?;
                rv.set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_A>::decision_state(&input, imp::selected::<0>),
                )
                .map_err(|_| ShellError::message("could not bind actual A candidate input"))?;
                rv.set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_B>::decision_state(&input, imp::selected::<1>),
                )
                .map_err(|_| ShellError::message("could not bind actual B candidate input"))?;
                async {
                    file_metadata::run(&mut a_data, a_file, BUILDER_FILES).await?;
                };
                async {
                    file_metadata::run(&mut b_data, b_file, BUILDER_FILES).await?;
                };
                async {
                    file_metadata::run(&mut c_data, c_file, BUILDER_FILES).await?;
                };
                async {
                    file_metadata::run(&mut integrator_data, integrator_file, INTEGRATOR_FILES)
                        .await?;
                };
                async {
                    file_metadata::run(&mut tester_data, tester_file, TESTER_FILES).await?;
                };
                let (a_file, b_file, c_file, integrator_file, tester_file) = async {
                    let [mut a, mut b, mut c, mut integrator, mut tester] = compiler.par()?;
                    async { a.recv::<Loan<A_FILE>>().await };
                    async { b.recv::<Loan<B_FILE>>().await };
                    async { c.recv::<Loan<C_FILE>>().await };
                    async { integrator.recv::<Loan<INTEGRATOR_FILE>>().await };
                    async { tester.recv::<Loan<TESTER_FILE>>().await };
                }
                .await?;
                async {
                    localside::builder::run(&mut a).await?;
                };
                async {
                    localside::builder::run(&mut b).await?;
                };
                async {
                    localside::builder::run(&mut c).await?;
                };
                async {
                    localside::compiler::run(&mut compiler, (core, config, models), &simulator)
                        .await?;
                };
                async {
                    localside::testing::run(&mut testing, &simulator).await?;
                };
                async {
                    localside::integrator::run(&mut integrator, &input).await?;
                };
                async {
                    localside::tester::run(&mut tester).await?;
                };
                async {
                    private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &BUILDER_FILES))
                        .await?;
                };
                async {
                    let file = a_file;
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let (mut updates, mut feedback, mut reconsideration, mut outcome) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(a_vm).file(
                        ChoreoFsFile::new(BUILDER_FILES[0], file)
                            .write_as::<CompileFileA>(&mut request)
                            .read_as::<FileCompiled>(&mut result)
                            .file(BUILDER_FILES[1])?
                            .read_as::<FileUpdated>(&mut updates)
                            .file(BUILDER_FILES[2])?
                            .read_as::<FileFeedback>(&mut feedback)
                            .file(BUILDER_FILES[3])?
                            .read_as::<FileReconsidered>(&mut reconsideration)
                            .file(BUILDER_FILES[4])?
                            .read_as::<FileFinished>(&mut outcome),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut guest_storage = HibanaWasiGuestStorage::uninit();
                    let guest = guest_storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &BUILDER_FILES))
                        .await?;
                };
                async {
                    let file = b_file;
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let (mut updates, mut feedback, mut reconsideration, mut outcome) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(b_vm).file(
                        ChoreoFsFile::new(BUILDER_FILES[0], file)
                            .write_as::<CompileFileB>(&mut request)
                            .read_as::<FileCompiled>(&mut result)
                            .file(BUILDER_FILES[1])?
                            .read_as::<FileUpdated>(&mut updates)
                            .file(BUILDER_FILES[2])?
                            .read_as::<FileFeedback>(&mut feedback)
                            .file(BUILDER_FILES[3])?
                            .read_as::<FileReconsidered>(&mut reconsideration)
                            .file(BUILDER_FILES[4])?
                            .read_as::<FileFinished>(&mut outcome),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut guest_storage = HibanaWasiGuestStorage::uninit();
                    let guest = guest_storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &BUILDER_FILES))
                        .await?;
                };
                async {
                    let file = c_file;
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let (mut updates, mut feedback, mut reconsideration, mut outcome) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(c_vm).file(
                        ChoreoFsFile::new(BUILDER_FILES[0], file)
                            .write_as::<CompileFileC>(&mut request)
                            .read_as::<FileCompiled>(&mut result)
                            .file(BUILDER_FILES[1])?
                            .read_as::<FileUpdated>(&mut updates)
                            .file(BUILDER_FILES[2])?
                            .read_as::<FileFeedback>(&mut feedback)
                            .file(BUILDER_FILES[3])?
                            .read_as::<FileReconsidered>(&mut reconsideration)
                            .file(BUILDER_FILES[4])?
                            .read_as::<FileFinished>(&mut outcome),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut guest_storage = HibanaWasiGuestStorage::uninit();
                    let guest = guest_storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    private_io::run(
                        &mut integrator_io,
                        PrivateFs::new(integrator_terminal, bus, &INTEGRATOR_FILES),
                    )
                    .await?;
                };
                async {
                    let file = integrator_file;
                    let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut result = vec![0; incoming_capacity];
                    let mut candidate = vec![0; incoming_capacity];
                    let files = ChoreoFsFiles::new(integrator_vm).file(
                        ChoreoFsFile::new(INTEGRATOR_FILES[0], file)
                            .write_as::<LinkFile>(&mut request)
                            .read_as::<FileLinked>(&mut result)
                            .file(INTEGRATOR_FILES[1])?
                            .read_as::<FileReady>(&mut candidate),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut guest_storage = HibanaWasiGuestStorage::uninit();
                    let guest = guest_storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    private_io::run(
                        &mut tester_io,
                        PrivateFs::new(tester_terminal, bus, &TESTER_FILES),
                    )
                    .await?;
                };
                async {
                    let file = tester_file;
                    let (mut a, mut b, mut c) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut ar, mut br, mut cr, mut object) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(tester_vm).file(
                        ChoreoFsFile::new(TESTER_FILES[0], file)
                            .write_as::<TestFileA>(&mut a)
                            .read_as::<FileTested>(&mut ar)
                            .file(TESTER_FILES[1])?
                            .write_as::<TestFileB>(&mut b)
                            .read_as::<FileTested>(&mut br)
                            .file(TESTER_FILES[2])?
                            .write_as::<TestFileC>(&mut c)
                            .read_as::<FileTested>(&mut cr)
                            .file(TESTER_FILES[3])?
                            .read_as::<ObjectForTesting>(&mut object),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut guest_storage = HibanaWasiGuestStorage::uninit();
                    let guest = guest_storage.init(module, GuestMemory::new(&mut memory), files)?;
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
        Ok(()) => io.emit(bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
