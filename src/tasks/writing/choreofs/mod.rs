//! Each shell file owns its Endpoint; localsides own actual requests and data.
pub mod global;
mod imp;
pub(crate) mod localside;
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::{Bus, Terminal},
        private_fs::PrivateFs,
    },
    localside::{file_metadata, private_io},
};
use global::*;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

#[hibana_runtime::scope]
async fn run_2e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::roles2e::*;
    let programs = project_2e();
    bus.observe(programs.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let mut slab = vec![0; 2 * 1024 * 1024];
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let [a_terminal, b_terminal, c_terminal] = terminals;
    let result = async {
        let mut a = rv.enter(session, &programs.a)?;
        let mut a_vm = rv.enter(session, &programs.a_vm)?;
        let mut a_data = rv.enter(session, &programs.a_data)?;
        let file_a = rv.enter(session, &programs.a_file)?;
        let mut a_io = rv.enter(session, &programs.a_io)?;
        let mut b = rv.enter(session, &programs.b)?;
        let mut b_vm = rv.enter(session, &programs.b_vm)?;
        let mut b_data = rv.enter(session, &programs.b_data)?;
        let file_b = rv.enter(session, &programs.b_file)?;
        let mut b_io = rv.enter(session, &programs.b_io)?;
        let mut c = rv.enter(session, &programs.c)?;
        let mut c_vm = rv.enter(session, &programs.c_vm)?;
        let mut c_data = rv.enter(session, &programs.c_data)?;
        let file_c = rv.enter(session, &programs.c_file)?;
        let mut c_io = rv.enter(session, &programs.c_io)?;
        let mut intro = rv.enter(session, &programs.intro)?;
        let intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut methods = rv.enter(session, &programs.methods)?;
        let methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut results = rv.enter(session, &programs.results)?;
        let results_resource = rv.enter(session, &programs.results_resource)?;
        async {
            file_metadata::run(&mut a_data, file_a, FILES).await?;
        };
        async {
            file_metadata::run(&mut b_data, file_b, FILES).await?;
        };
        async {
            file_metadata::run(&mut c_data, file_c, FILES).await?;
        };
        async {
            localside::researcher::run::<A, INTRO_RESOURCE, METHODS_RESOURCE>(&mut a, bus).await?;
        };
        async {
            localside::researcher::run::<B, METHODS_RESOURCE, RESULTS_RESOURCE>(&mut b, bus)
                .await?;
        };
        async {
            localside::researcher::run::<C, INTRO_RESOURCE, RESULTS_RESOURCE>(&mut c, bus).await?;
        };
        async {
            localside::section::run(&mut intro, intro_resource).await?;
        };
        async {
            localside::section::run(&mut methods, methods_resource).await?;
        };
        async {
            localside::section::run(&mut results, results_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<A>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<A>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<A>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<A>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &FILES)).await?;
        };
        async {
            let file = b_vm.recv::<Loan<B_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<B>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<B>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<B>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<B>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &FILES)).await?;
        };
        async {
            let file = c_vm.recv::<Loan<C_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<C>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<C>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<C>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<C>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &FILES)).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
    result
}
#[hibana_runtime::scope]
async fn run_2m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use global::roles2m::*;
    let programs = project_2m();
    bus.observe(programs.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let mut slab = vec![0; 2 * 1024 * 1024];
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let [a_terminal, b_terminal, c_terminal, d_terminal, e_terminal] = terminals;
    let result = async {
        let mut a = rv.enter(session, &programs.a)?;
        let mut a_vm = rv.enter(session, &programs.a_vm)?;
        let mut a_data = rv.enter(session, &programs.a_data)?;
        let file_a = rv.enter(session, &programs.a_file)?;
        let mut a_io = rv.enter(session, &programs.a_io)?;
        let mut b = rv.enter(session, &programs.b)?;
        let mut b_vm = rv.enter(session, &programs.b_vm)?;
        let mut b_data = rv.enter(session, &programs.b_data)?;
        let file_b = rv.enter(session, &programs.b_file)?;
        let mut b_io = rv.enter(session, &programs.b_io)?;
        let mut c = rv.enter(session, &programs.c)?;
        let mut c_vm = rv.enter(session, &programs.c_vm)?;
        let mut c_data = rv.enter(session, &programs.c_data)?;
        let file_c = rv.enter(session, &programs.c_file)?;
        let mut c_io = rv.enter(session, &programs.c_io)?;
        let mut d = rv.enter(session, &programs.d)?;
        let mut d_vm = rv.enter(session, &programs.d_vm)?;
        let mut d_data = rv.enter(session, &programs.d_data)?;
        let file_d = rv.enter(session, &programs.d_file)?;
        let mut d_io = rv.enter(session, &programs.d_io)?;
        let mut e = rv.enter(session, &programs.e)?;
        let mut e_vm = rv.enter(session, &programs.e_vm)?;
        let mut e_data = rv.enter(session, &programs.e_data)?;
        let file_e = rv.enter(session, &programs.e_file)?;
        let mut e_io = rv.enter(session, &programs.e_io)?;
        let mut background = rv.enter(session, &programs.background)?;
        let background_resource = rv.enter(session, &programs.background_resource)?;
        let mut intro = rv.enter(session, &programs.intro)?;
        let intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut methods = rv.enter(session, &programs.methods)?;
        let methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut experiments = rv.enter(session, &programs.experiments)?;
        let experiments_resource = rv.enter(session, &programs.experiments_resource)?;
        let mut results = rv.enter(session, &programs.results)?;
        let results_resource = rv.enter(session, &programs.results_resource)?;
        async {
            file_metadata::run(&mut a_data, file_a, FILES).await?;
        };
        async {
            file_metadata::run(&mut b_data, file_b, FILES).await?;
        };
        async {
            file_metadata::run(&mut c_data, file_c, FILES).await?;
        };
        async {
            file_metadata::run(&mut d_data, file_d, FILES).await?;
        };
        async {
            file_metadata::run(&mut e_data, file_e, FILES).await?;
        };
        async {
            localside::researcher::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(&mut a, bus)
                .await?;
        };
        async {
            localside::researcher::run::<B, BACKGROUND_RESOURCE, METHODS_RESOURCE>(&mut b, bus)
                .await?;
        };
        async {
            localside::researcher::run::<C, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(&mut c, bus)
                .await?;
        };
        async {
            localside::researcher::run::<D, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(&mut d, bus)
                .await?;
        };
        async {
            localside::researcher::run::<E, INTRO_RESOURCE, RESULTS_RESOURCE>(&mut e, bus).await?;
        };
        async {
            localside::section::run(&mut background, background_resource).await?;
        };
        async {
            localside::section::run(&mut intro, intro_resource).await?;
        };
        async {
            localside::section::run(&mut methods, methods_resource).await?;
        };
        async {
            localside::section::run(&mut experiments, experiments_resource).await?;
        };
        async {
            localside::section::run(&mut results, results_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<A>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<A>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<A>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<A>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &FILES)).await?;
        };
        async {
            let file = b_vm.recv::<Loan<B_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<B>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<B>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<B>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<B>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &FILES)).await?;
        };
        async {
            let file = c_vm.recv::<Loan<C_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<C>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<C>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<C>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<C>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &FILES)).await?;
        };
        async {
            let file = d_vm.recv::<Loan<D_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(d_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<D>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<D>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<D>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<D>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut d_io, PrivateFs::new(d_terminal, bus, &FILES)).await?;
        };
        async {
            let file = e_vm.recv::<Loan<E_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(e_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<E>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<E>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<E>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<E>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut e_io, PrivateFs::new(e_terminal, bus, &FILES)).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
    result
}
#[hibana_runtime::scope]
async fn run_2h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use global::roles2h::*;
    let programs = project_2h();
    bus.observe(programs.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let mut slab = vec![0; 2 * 1024 * 1024];
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let [
        a_terminal,
        b_terminal,
        c_terminal,
        d_terminal,
        e_terminal,
        f_terminal,
        g_terminal,
    ] = terminals;
    let result = async {
        let mut a = rv.enter(session, &programs.a)?;
        let mut a_vm = rv.enter(session, &programs.a_vm)?;
        let mut a_data = rv.enter(session, &programs.a_data)?;
        let file_a = rv.enter(session, &programs.a_file)?;
        let mut a_io = rv.enter(session, &programs.a_io)?;
        let mut b = rv.enter(session, &programs.b)?;
        let mut b_vm = rv.enter(session, &programs.b_vm)?;
        let mut b_data = rv.enter(session, &programs.b_data)?;
        let file_b = rv.enter(session, &programs.b_file)?;
        let mut b_io = rv.enter(session, &programs.b_io)?;
        let mut c = rv.enter(session, &programs.c)?;
        let mut c_vm = rv.enter(session, &programs.c_vm)?;
        let mut c_data = rv.enter(session, &programs.c_data)?;
        let file_c = rv.enter(session, &programs.c_file)?;
        let mut c_io = rv.enter(session, &programs.c_io)?;
        let mut d = rv.enter(session, &programs.d)?;
        let mut d_vm = rv.enter(session, &programs.d_vm)?;
        let mut d_data = rv.enter(session, &programs.d_data)?;
        let file_d = rv.enter(session, &programs.d_file)?;
        let mut d_io = rv.enter(session, &programs.d_io)?;
        let mut e = rv.enter(session, &programs.e)?;
        let mut e_vm = rv.enter(session, &programs.e_vm)?;
        let mut e_data = rv.enter(session, &programs.e_data)?;
        let file_e = rv.enter(session, &programs.e_file)?;
        let mut e_io = rv.enter(session, &programs.e_io)?;
        let mut f = rv.enter(session, &programs.f)?;
        let mut f_vm = rv.enter(session, &programs.f_vm)?;
        let mut f_data = rv.enter(session, &programs.f_data)?;
        let file_f = rv.enter(session, &programs.f_file)?;
        let mut f_io = rv.enter(session, &programs.f_io)?;
        let mut g = rv.enter(session, &programs.g)?;
        let mut g_vm = rv.enter(session, &programs.g_vm)?;
        let mut g_data = rv.enter(session, &programs.g_data)?;
        let file_g = rv.enter(session, &programs.g_file)?;
        let mut g_io = rv.enter(session, &programs.g_io)?;
        let mut background = rv.enter(session, &programs.background)?;
        let background_resource = rv.enter(session, &programs.background_resource)?;
        let mut intro = rv.enter(session, &programs.intro)?;
        let intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut related = rv.enter(session, &programs.related)?;
        let related_resource = rv.enter(session, &programs.related_resource)?;
        let mut methods = rv.enter(session, &programs.methods)?;
        let methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut experiments = rv.enter(session, &programs.experiments)?;
        let experiments_resource = rv.enter(session, &programs.experiments_resource)?;
        let mut results = rv.enter(session, &programs.results)?;
        let results_resource = rv.enter(session, &programs.results_resource)?;
        let mut discussion = rv.enter(session, &programs.discussion)?;
        let discussion_resource = rv.enter(session, &programs.discussion_resource)?;
        async {
            file_metadata::run(&mut a_data, file_a, FILES).await?;
        };
        async {
            file_metadata::run(&mut b_data, file_b, FILES).await?;
        };
        async {
            file_metadata::run(&mut c_data, file_c, FILES).await?;
        };
        async {
            file_metadata::run(&mut d_data, file_d, FILES).await?;
        };
        async {
            file_metadata::run(&mut e_data, file_e, FILES).await?;
        };
        async {
            file_metadata::run(&mut f_data, file_f, FILES).await?;
        };
        async {
            file_metadata::run(&mut g_data, file_g, FILES).await?;
        };
        async {
            localside::researcher::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(&mut a, bus)
                .await?;
        };
        async {
            localside::researcher::run::<B, BACKGROUND_RESOURCE, RELATED_RESOURCE>(&mut b, bus)
                .await?;
        };
        async {
            localside::researcher::run::<C, METHODS_RESOURCE, RELATED_RESOURCE>(&mut c, bus)
                .await?;
        };
        async {
            localside::researcher::run::<D, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(&mut d, bus)
                .await?;
        };
        async {
            localside::researcher::run::<E, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(&mut e, bus)
                .await?;
        };
        async {
            localside::researcher::run::<F, DISCUSSION_RESOURCE, RESULTS_RESOURCE>(&mut f, bus)
                .await?;
        };
        async {
            localside::researcher::run::<G, DISCUSSION_RESOURCE, INTRO_RESOURCE>(&mut g, bus)
                .await?;
        };
        async {
            localside::section::run(&mut background, background_resource).await?;
        };
        async {
            localside::section::run(&mut intro, intro_resource).await?;
        };
        async {
            localside::section::run(&mut related, related_resource).await?;
        };
        async {
            localside::section::run(&mut methods, methods_resource).await?;
        };
        async {
            localside::section::run(&mut experiments, experiments_resource).await?;
        };
        async {
            localside::section::run(&mut results, results_resource).await?;
        };
        async {
            localside::section::run(&mut discussion, discussion_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<A>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<A>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<A>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<A>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &FILES)).await?;
        };
        async {
            let file = b_vm.recv::<Loan<B_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<B>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<B>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<B>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<B>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &FILES)).await?;
        };
        async {
            let file = c_vm.recv::<Loan<C_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<C>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<C>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<C>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<C>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &FILES)).await?;
        };
        async {
            let file = d_vm.recv::<Loan<D_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(d_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<D>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<D>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<D>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<D>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut d_io, PrivateFs::new(d_terminal, bus, &FILES)).await?;
        };
        async {
            let file = e_vm.recv::<Loan<E_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(e_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<E>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<E>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<E>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<E>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut e_io, PrivateFs::new(e_terminal, bus, &FILES)).await?;
        };
        async {
            let file = f_vm.recv::<Loan<F_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(f_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<F>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<F>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<F>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<F>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut f_io, PrivateFs::new(f_terminal, bus, &FILES)).await?;
        };
        async {
            let file = g_vm.recv::<Loan<G_FILE>>().await?;
            let (mut read, mut draft, mut write, mut review) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut read_result, mut draft_result, mut write_result, mut review_result) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(g_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<ReadRequest<G>>(&mut read)
                    .read_as::<ReadResult>(&mut read_result)
                    .file(FILES[1])?
                    .write_as::<DraftRequest<G>>(&mut draft)
                    .read_as::<DraftResult>(&mut draft_result)
                    .file(FILES[2])?
                    .write_as::<WriteRequest<G>>(&mut write)
                    .read_as::<WriteResult>(&mut write_result)
                    .file(FILES[3])?
                    .write_as::<ReviewRequest<G>>(&mut review)
                    .read_as::<ReviewResult>(&mut review_result),
            )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut root, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<crate::shell::global::FileExit>(&status).await?;
            root.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(&mut g_io, PrivateFs::new(g_terminal, bus, &FILES)).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
    result
}
pub async fn run_stdio(module: &[u8], level: &str) -> ShellResult<()> {
    match level {
        "2E" => {
            use global::roles2e::*;
            let (bus, mut io, terminals) = Bus::open(
                ["RESEARCHER_A", "RESEARCHER_B", "RESEARCHER_C"],
                [
                    (SessionId::new(SESSION), INTRO_RESOURCE),
                    (SessionId::new(SESSION), METHODS_RESOURCE),
                    (SessionId::new(SESSION), RESULTS_RESOURCE),
                    (SessionId::new(SESSION), A_FILE),
                    (SessionId::new(SESSION), B_FILE),
                    (SessionId::new(SESSION), C_FILE),
                ],
            );
            let result =
                hibana_runtime::race(run_2e_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result { Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?, Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?, }
            result
        }
        "2M" => {
            use global::roles2m::*;
            let (bus, mut io, terminals) = Bus::open(
                [
                    "RESEARCHER_A",
                    "RESEARCHER_B",
                    "RESEARCHER_C",
                    "RESEARCHER_D",
                    "RESEARCHER_E",
                ],
                [
                    (SessionId::new(SESSION), BACKGROUND_RESOURCE),
                    (SessionId::new(SESSION), INTRO_RESOURCE),
                    (SessionId::new(SESSION), METHODS_RESOURCE),
                    (SessionId::new(SESSION), EXPERIMENTS_RESOURCE),
                    (SessionId::new(SESSION), RESULTS_RESOURCE),
                    (SessionId::new(SESSION), A_FILE),
                    (SessionId::new(SESSION), B_FILE),
                    (SessionId::new(SESSION), C_FILE),
                    (SessionId::new(SESSION), D_FILE),
                    (SessionId::new(SESSION), E_FILE),
                ],
            );
            let result =
                hibana_runtime::race(run_2m_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result { Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?, Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?, }
            result
        }
        "2H" => {
            use global::roles2h::*;
            let (bus, mut io, terminals) = Bus::open(
                [
                    "RESEARCHER_A",
                    "RESEARCHER_B",
                    "RESEARCHER_C",
                    "RESEARCHER_D",
                    "RESEARCHER_E",
                    "RESEARCHER_F",
                    "RESEARCHER_G",
                ],
                [
                    (SessionId::new(SESSION), BACKGROUND_RESOURCE),
                    (SessionId::new(SESSION), INTRO_RESOURCE),
                    (SessionId::new(SESSION), RELATED_RESOURCE),
                    (SessionId::new(SESSION), METHODS_RESOURCE),
                    (SessionId::new(SESSION), EXPERIMENTS_RESOURCE),
                    (SessionId::new(SESSION), RESULTS_RESOURCE),
                    (SessionId::new(SESSION), DISCUSSION_RESOURCE),
                    (SessionId::new(SESSION), A_FILE),
                    (SessionId::new(SESSION), B_FILE),
                    (SessionId::new(SESSION), C_FILE),
                    (SessionId::new(SESSION), D_FILE),
                    (SessionId::new(SESSION), E_FILE),
                    (SessionId::new(SESSION), F_FILE),
                    (SessionId::new(SESSION), G_FILE),
                ],
            );
            let result =
                hibana_runtime::race(run_2h_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result { Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?, Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?, }
            result
        }
        _ => Err(ShellError::message("expected writing task 2E, 2M or 2H")),
    }
}
pub fn projection(level: &str) -> ShellResult<serde_json::Value> {
    match level {
        "2E" => Ok(global::project_2e().observation()),
        "2M" => Ok(global::project_2m().observation()),
        "2H" => Ok(global::project_2h().observation()),
        _ => Err(ShellError::message("expected writing task 2E, 2M or 2H")),
    }
}
