//! Actual operation files own projected Endpoints; localsides drive effects.
pub mod global;
pub(crate) mod input;
pub(crate) mod localside;
pub mod wire;
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
async fn run_1e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::roles1e::*;
    let programs = project_1e();
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
        let mut auth = rv.enter(session, &programs.auth)?;
        let auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database = rv.enter(session, &programs.database)?;
        let database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api = rv.enter(session, &programs.api)?;
        let api_resource = rv.enter(session, &programs.api_resource)?;
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
            localside::developer::run::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(&mut a, bus).await?;
        };
        async {
            localside::developer::run::<B, API_RESOURCE, DATABASE_RESOURCE>(&mut b, bus).await?;
        };
        async {
            localside::developer::run::<C, API_RESOURCE, AUTH_RESOURCE>(&mut c, bus).await?;
        };
        async {
            localside::module::run(&mut auth, auth_resource).await?;
        };
        async {
            localside::module::run(&mut database, database_resource).await?;
        };
        async {
            localside::module::run(&mut api, api_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<A>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<A>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<A>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<A>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<B>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<B>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<B>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<B>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<C>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<C>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<C>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<C>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
async fn run_1m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use global::roles1m::*;
    let programs = project_1m();
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
        let mut auth = rv.enter(session, &programs.auth)?;
        let auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database = rv.enter(session, &programs.database)?;
        let database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api = rv.enter(session, &programs.api)?;
        let api_resource = rv.enter(session, &programs.api_resource)?;
        let mut payment = rv.enter(session, &programs.payment)?;
        let payment_resource = rv.enter(session, &programs.payment_resource)?;
        let mut notify = rv.enter(session, &programs.notify)?;
        let notify_resource = rv.enter(session, &programs.notify_resource)?;
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
            localside::developer::run::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(&mut a, bus).await?;
        };
        async {
            localside::developer::run::<B, API_RESOURCE, DATABASE_RESOURCE>(&mut b, bus).await?;
        };
        async {
            localside::developer::run::<C, API_RESOURCE, PAYMENT_RESOURCE>(&mut c, bus).await?;
        };
        async {
            localside::developer::run::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(&mut d, bus).await?;
        };
        async {
            localside::developer::run::<E, AUTH_RESOURCE, NOTIFY_RESOURCE>(&mut e, bus).await?;
        };
        async {
            localside::module::run(&mut auth, auth_resource).await?;
        };
        async {
            localside::module::run(&mut database, database_resource).await?;
        };
        async {
            localside::module::run(&mut api, api_resource).await?;
        };
        async {
            localside::module::run(&mut payment, payment_resource).await?;
        };
        async {
            localside::module::run(&mut notify, notify_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<A>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<A>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<A>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<A>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<B>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<B>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<B>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<B>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<C>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<C>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<C>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<C>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(d_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<D>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<D>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<D>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<D>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(e_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<E>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<E>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<E>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<E>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
async fn run_1h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use global::roles1h::*;
    let programs = project_1h();
    bus.observe(programs.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let report_a = input::TestResult::default();
    let report_b = input::TestResult::default();
    let report_c = input::TestResult::default();
    let report_d = input::TestResult::default();
    let report_e = input::TestResult::default();
    let report_f = input::TestResult::default();
    let report_g = input::TestResult::default();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    rv.set_resolver(
        &programs.a,
        hibana::runtime::resolver::ResolverRef::<CHECK_A>::decision_state(
            &report_a,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        hibana::runtime::resolver::ResolverRef::<CHECK_B>::decision_state(
            &report_b,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        hibana::runtime::resolver::ResolverRef::<CHECK_C>::decision_state(
            &report_c,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        hibana::runtime::resolver::ResolverRef::<CHECK_D>::decision_state(
            &report_d,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        hibana::runtime::resolver::ResolverRef::<CHECK_E>::decision_state(
            &report_e,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.f,
        hibana::runtime::resolver::ResolverRef::<CHECK_F>::decision_state(
            &report_f,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.g,
        hibana::runtime::resolver::ResolverRef::<CHECK_G>::decision_state(
            &report_g,
            input::TestResult::decide,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
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
        let mut auth = rv.enter(session, &programs.auth)?;
        let auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database = rv.enter(session, &programs.database)?;
        let database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api = rv.enter(session, &programs.api)?;
        let api_resource = rv.enter(session, &programs.api_resource)?;
        let mut payment = rv.enter(session, &programs.payment)?;
        let payment_resource = rv.enter(session, &programs.payment_resource)?;
        let mut notify = rv.enter(session, &programs.notify)?;
        let notify_resource = rv.enter(session, &programs.notify_resource)?;
        let mut analytics = rv.enter(session, &programs.analytics)?;
        let analytics_resource = rv.enter(session, &programs.analytics_resource)?;
        let mut search = rv.enter(session, &programs.search)?;
        let search_resource = rv.enter(session, &programs.search_resource)?;
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
            localside::developer_reconsider::run::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(
                &mut a, bus, &report_a,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<B, API_RESOURCE, DATABASE_RESOURCE>(
                &mut b, bus, &report_b,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<C, API_RESOURCE, PAYMENT_RESOURCE>(
                &mut c, bus, &report_c,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
                &mut d, bus, &report_d,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<E, ANALYTICS_RESOURCE, NOTIFY_RESOURCE>(
                &mut e, bus, &report_e,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<F, ANALYTICS_RESOURCE, SEARCH_RESOURCE>(
                &mut f, bus, &report_f,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<G, AUTH_RESOURCE, SEARCH_RESOURCE>(
                &mut g, bus, &report_g,
            )
            .await?;
        };
        async {
            localside::module_reconsider::auth(&mut auth, auth_resource).await?;
        };
        async {
            localside::module_reconsider::database(&mut database, database_resource).await?;
        };
        async {
            localside::module_reconsider::api(&mut api, api_resource).await?;
        };
        async {
            localside::module_reconsider::payment(&mut payment, payment_resource).await?;
        };
        async {
            localside::module_reconsider::notify(&mut notify, notify_resource).await?;
        };
        async {
            localside::module_reconsider::analytics(&mut analytics, analytics_resource).await?;
        };
        async {
            localside::module_reconsider::search(&mut search, search_resource).await?;
        };
        async {
            let file = a_vm.recv::<Loan<A_FILE>>().await?;
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(a_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<A>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<A>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<A>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<A>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(b_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<B>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<B>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<B>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<B>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(c_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<C>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<C>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<C>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<C>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(d_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<D>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<D>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<D>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<D>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(e_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<E>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<E>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<E>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<E>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(f_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<F>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<F>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<F>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<F>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
            let (mut design, mut implementation, mut commit, mut test) = (
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
                vec![0; crate::shell::GUEST_MEMORY_BYTES],
            );
            let (mut designed, mut implemented, mut committed, mut tested) = (
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
                vec![0; incoming_capacity],
            );
            let files = ChoreoFsFiles::new(g_vm).file(
                ChoreoFsFile::new(FILES[0], file)
                    .write_as::<DesignRequest<G>>(&mut design)
                    .read_as::<DesignCompleted>(&mut designed)
                    .file(FILES[1])?
                    .write_as::<ImplementRequest<G>>(&mut implementation)
                    .read_as::<ImplementCompleted>(&mut implemented)
                    .file(FILES[2])?
                    .write_as::<CommitRequest<G>>(&mut commit)
                    .read_as::<CommitCompleted>(&mut committed)
                    .file(FILES[3])?
                    .write_as::<TestRequest<G>>(&mut test)
                    .read_as::<TestCompleted>(&mut tested),
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
        "1E" => {
            use global::roles1e::*;
            let (bus, mut io, terminals) = Bus::open(
                ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"],
                [
                    (SessionId::new(SESSION), AUTH_RESOURCE),
                    (SessionId::new(SESSION), DATABASE_RESOURCE),
                    (SessionId::new(SESSION), API_RESOURCE),
                    (SessionId::new(SESSION), A_FILE),
                    (SessionId::new(SESSION), B_FILE),
                    (SessionId::new(SESSION), C_FILE),
                ],
            );
            let result =
                hibana_runtime::race(run_1e_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result {Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,}
            result
        }
        "1M" => {
            use global::roles1m::*;
            let (bus, mut io, terminals) = Bus::open(
                [
                    "DEVELOPER_A",
                    "DEVELOPER_B",
                    "DEVELOPER_C",
                    "DEVELOPER_D",
                    "DEVELOPER_E",
                ],
                [
                    (SessionId::new(SESSION), AUTH_RESOURCE),
                    (SessionId::new(SESSION), DATABASE_RESOURCE),
                    (SessionId::new(SESSION), API_RESOURCE),
                    (SessionId::new(SESSION), PAYMENT_RESOURCE),
                    (SessionId::new(SESSION), NOTIFY_RESOURCE),
                    (SessionId::new(SESSION), A_FILE),
                    (SessionId::new(SESSION), B_FILE),
                    (SessionId::new(SESSION), C_FILE),
                    (SessionId::new(SESSION), D_FILE),
                    (SessionId::new(SESSION), E_FILE),
                ],
            );
            let result =
                hibana_runtime::race(run_1m_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result {Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,}
            result
        }
        "1H" => {
            use global::roles1h::*;
            let (bus, mut io, terminals) = Bus::open(
                [
                    "DEVELOPER_A",
                    "DEVELOPER_B",
                    "DEVELOPER_C",
                    "DEVELOPER_D",
                    "DEVELOPER_E",
                    "DEVELOPER_F",
                    "DEVELOPER_G",
                ],
                [
                    (SessionId::new(SESSION), AUTH_RESOURCE),
                    (SessionId::new(SESSION), DATABASE_RESOURCE),
                    (SessionId::new(SESSION), API_RESOURCE),
                    (SessionId::new(SESSION), PAYMENT_RESOURCE),
                    (SessionId::new(SESSION), NOTIFY_RESOURCE),
                    (SessionId::new(SESSION), ANALYTICS_RESOURCE),
                    (SessionId::new(SESSION), SEARCH_RESOURCE),
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
                hibana_runtime::race(run_1h_guests(module, &bus, terminals), io.run(&bus)).await;
            match &result {Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,}
            result
        }
        _ => Err(ShellError::message("expected codebase task 1E,1M or1H")),
    }
}
pub fn projection(level: &str) -> ShellResult<serde_json::Value> {
    match level {
        "1E" => Ok(global::project_1e().observation()),
        "1M" => Ok(global::project_1m().observation()),
        "1H" => Ok(global::project_1h().observation()),
        _ => Err(ShellError::message("unknown codebase projection")),
    }
}
