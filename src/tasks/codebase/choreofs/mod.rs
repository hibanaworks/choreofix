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
        let mut requests = rv.enter(session, &programs.requests)?;
        let mut repository = rv.enter(session, &programs.repository)?;
        let mut auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api_resource = rv.enter(session, &programs.api_resource)?;
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
            for _ in [A, B, C] {
                let request = requests.offer().await?;
                match request.label() {
                    A => {
                        let args = request.recv::<Submitted<A>>().await?;
                        repository.send::<CommitRequest<A>>(&args).await?;
                        async {
                            let [mut first, mut second] = repository.par()?;
                            async {
                                first.send::<Loan<AUTH_RESOURCE>>(auth_resource).await?;
                            };
                            async {
                                second
                                    .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                                    .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = repository.par()?;
                        (auth_resource, database_resource) = async {
                            async { first.recv::<Loan<AUTH_RESOURCE>>().await };
                            async { second.recv::<Loan<DATABASE_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    B => {
                        let args = request.recv::<Submitted<B>>().await?;
                        repository.send::<CommitRequest<B>>(&args).await?;
                        async {
                            let [mut first, mut second] = repository.par()?;
                            async {
                                first.send::<Loan<API_RESOURCE>>(api_resource).await?;
                            };
                            async {
                                second
                                    .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                                    .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = repository.par()?;
                        (api_resource, database_resource) = async {
                            async { first.recv::<Loan<API_RESOURCE>>().await };
                            async { second.recv::<Loan<DATABASE_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    C => {
                        let args = request.recv::<Submitted<C>>().await?;
                        repository.send::<CommitRequest<C>>(&args).await?;
                        async {
                            let [mut first, mut second] = repository.par()?;
                            async {
                                first.send::<Loan<API_RESOURCE>>(api_resource).await?;
                            };
                            async {
                                second.send::<Loan<AUTH_RESOURCE>>(auth_resource).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = repository.par()?;
                        (api_resource, auth_resource) = async {
                            async { first.recv::<Loan<API_RESOURCE>>().await };
                            async { second.recv::<Loan<AUTH_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    _ => return Err(ShellError::message("unknown repository submission")),
                }
            }
            Ok(())
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
                    .write_as::<Submitted<A>>(&mut commit)
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
                    .write_as::<Submitted<B>>(&mut commit)
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
                    .write_as::<Submitted<C>>(&mut commit)
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
        let mut requests = rv.enter(session, &programs.requests)?;
        let mut repository = rv.enter(session, &programs.repository)?;
        let mut auth = rv.enter(session, &programs.auth)?;
        let mut auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database = rv.enter(session, &programs.database)?;
        let mut database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api = rv.enter(session, &programs.api)?;
        let mut api_resource = rv.enter(session, &programs.api_resource)?;
        let mut payment = rv.enter(session, &programs.payment)?;
        let mut payment_resource = rv.enter(session, &programs.payment_resource)?;
        let mut notify = rv.enter(session, &programs.notify)?;
        let mut notify_resource = rv.enter(session, &programs.notify_resource)?;
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
            localside::developer_1m::run::<A, AUTH_RESOURCE, DATABASE_RESOURCE>(&mut a, bus)
                .await?;
        };
        async {
            localside::developer_1m::run::<B, API_RESOURCE, DATABASE_RESOURCE>(&mut b, bus).await?;
        };
        async {
            localside::developer_1m::run::<C, API_RESOURCE, PAYMENT_RESOURCE>(&mut c, bus).await?;
        };
        async {
            localside::developer_1m::run::<D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(&mut d, bus)
                .await?;
        };
        async {
            localside::developer_1m::run::<E, AUTH_RESOURCE, NOTIFY_RESOURCE>(&mut e, bus).await?;
        };
        async {
            for _ in 0..10 {
                let request = requests.offer().await?;
                match request.label() {
                    A => {
                        let args = request.recv::<Submitted<A>>().await?;
                        repository.send::<Submitted<A>>(&args).await?;
                    }
                    A_FILE => {
                        let args = request.recv::<ModuleClaim<A_FILE, A>>().await?;
                        repository.send::<ModuleClaim<A_FILE, A>>(&args).await?;
                    }
                    B => {
                        let args = request.recv::<Submitted<B>>().await?;
                        repository.send::<Submitted<B>>(&args).await?;
                    }
                    B_FILE => {
                        let args = request.recv::<ModuleClaim<B_FILE, B>>().await?;
                        repository.send::<ModuleClaim<B_FILE, B>>(&args).await?;
                    }
                    C => {
                        let args = request.recv::<Submitted<C>>().await?;
                        repository.send::<Submitted<C>>(&args).await?;
                    }
                    C_FILE => {
                        let args = request.recv::<ModuleClaim<C_FILE, C>>().await?;
                        repository.send::<ModuleClaim<C_FILE, C>>(&args).await?;
                    }
                    D => {
                        let args = request.recv::<Submitted<D>>().await?;
                        repository.send::<Submitted<D>>(&args).await?;
                    }
                    D_FILE => {
                        let args = request.recv::<ModuleClaim<D_FILE, D>>().await?;
                        repository.send::<ModuleClaim<D_FILE, D>>(&args).await?;
                    }
                    E => {
                        let args = request.recv::<Submitted<E>>().await?;
                        repository.send::<Submitted<E>>(&args).await?;
                    }
                    E_FILE => {
                        let args = request.recv::<ModuleClaim<E_FILE, E>>().await?;
                        repository.send::<ModuleClaim<E_FILE, E>>(&args).await?;
                    }
                    _ => return Err(ShellError::message("unknown module request")),
                }
            }
        };
        async {
            for _ in 0..2 {
                let claim = api.offer().await?;
                match claim.label() {
                    B_FILE => {
                        claim.recv::<ModuleClaim<B_FILE, B>>().await?;
                    }
                    C_FILE => {
                        claim.recv::<ModuleClaim<C_FILE, C>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown api claim")),
                }
                api.send::<Loan<API_RESOURCE>>(api_resource).await?;
                api_resource = api.recv::<Loan<API_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = auth.offer().await?;
                match claim.label() {
                    A_FILE => {
                        claim.recv::<ModuleClaim<A_FILE, A>>().await?;
                    }
                    E => {
                        let args = claim.recv::<Submitted<E>>().await?;
                        auth.send::<ModuleClaim<E_FILE, E>>(&args).await?;
                        auth.send::<CommitRequest<E>>(&args).await?;
                    }
                    _ => return Err(ShellError::message("unknown auth claim")),
                }
                auth.send::<Loan<AUTH_RESOURCE>>(auth_resource).await?;
                auth_resource = auth.recv::<Loan<AUTH_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = database.offer().await?;
                match claim.label() {
                    A => {
                        let args = claim.recv::<Submitted<A>>().await?;
                        database.send::<ModuleClaim<A_FILE, A>>(&args).await?;
                        database.send::<CommitRequest<A>>(&args).await?;
                    }
                    B => {
                        let args = claim.recv::<Submitted<B>>().await?;
                        database.send::<ModuleClaim<B_FILE, B>>(&args).await?;
                        database.send::<CommitRequest<B>>(&args).await?;
                    }
                    _ => return Err(ShellError::message("unknown database claim")),
                }
                database
                    .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                    .await?;
                database_resource = database.recv::<Loan<DATABASE_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = notify.offer().await?;
                match claim.label() {
                    D_FILE => {
                        claim.recv::<ModuleClaim<D_FILE, D>>().await?;
                    }
                    E_FILE => {
                        claim.recv::<ModuleClaim<E_FILE, E>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown notify claim")),
                }
                notify
                    .send::<Loan<NOTIFY_RESOURCE>>(notify_resource)
                    .await?;
                notify_resource = notify.recv::<Loan<NOTIFY_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = payment.offer().await?;
                match claim.label() {
                    C => {
                        let args = claim.recv::<Submitted<C>>().await?;
                        payment.send::<ModuleClaim<C_FILE, C>>(&args).await?;
                        payment.send::<CommitRequest<C>>(&args).await?;
                    }
                    D => {
                        let args = claim.recv::<Submitted<D>>().await?;
                        payment.send::<ModuleClaim<D_FILE, D>>(&args).await?;
                        payment.send::<CommitRequest<D>>(&args).await?;
                    }
                    _ => return Err(ShellError::message("unknown payment claim")),
                }
                payment
                    .send::<Loan<PAYMENT_RESOURCE>>(payment_resource)
                    .await?;
                payment_resource = payment.recv::<Loan<PAYMENT_RESOURCE>>().await?;
            }
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
                    .write_as::<Submitted<A>>(&mut commit)
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
                    .write_as::<Submitted<B>>(&mut commit)
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
                    .write_as::<Submitted<C>>(&mut commit)
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
                    .write_as::<Submitted<D>>(&mut commit)
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
                    .write_as::<Submitted<E>>(&mut commit)
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
        let mut auth_requests = rv.enter(session, &programs.auth_requests)?;
        let mut auth = rv.enter(session, &programs.auth)?;
        let mut auth_resource = rv.enter(session, &programs.auth_resource)?;
        let mut database_requests = rv.enter(session, &programs.database_requests)?;
        let mut database = rv.enter(session, &programs.database)?;
        let mut database_resource = rv.enter(session, &programs.database_resource)?;
        let mut api_requests = rv.enter(session, &programs.api_requests)?;
        let mut api = rv.enter(session, &programs.api)?;
        let mut api_resource = rv.enter(session, &programs.api_resource)?;
        let mut payment_requests = rv.enter(session, &programs.payment_requests)?;
        let mut payment = rv.enter(session, &programs.payment)?;
        let mut payment_resource = rv.enter(session, &programs.payment_resource)?;
        let mut notify_requests = rv.enter(session, &programs.notify_requests)?;
        let mut notify = rv.enter(session, &programs.notify)?;
        let mut notify_resource = rv.enter(session, &programs.notify_resource)?;
        let mut analytics_requests = rv.enter(session, &programs.analytics_requests)?;
        let mut analytics = rv.enter(session, &programs.analytics)?;
        let mut analytics_resource = rv.enter(session, &programs.analytics_resource)?;
        let mut search_requests = rv.enter(session, &programs.search_requests)?;
        let mut search = rv.enter(session, &programs.search)?;
        let mut search_resource = rv.enter(session, &programs.search_resource)?;
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
            localside::developer_reconsider::run::<A, DATABASE_RESOURCE, AUTH_RESOURCE>(
                &mut a, bus, &report_a,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<B, DATABASE_RESOURCE, API_RESOURCE>(
                &mut b, bus, &report_b,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<C, PAYMENT_RESOURCE, API_RESOURCE>(
                &mut c, bus, &report_c,
            )
            .await?;
        };
        async {
            localside::developer_reconsider::run::<D, PAYMENT_RESOURCE, NOTIFY_RESOURCE>(
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
            for _ in 0..2 {
                loop {
                    let request = analytics_requests.offer().await?;
                    match request.label() {
                        E => {
                            let arguments = request.recv::<Submitted<E>>().await?;
                            analytics.send::<ModuleClaim<E_FILE, E>>(&arguments).await?;
                            analytics.send::<CommitRequest<E>>(&arguments).await?;
                            analytics
                                .send::<ModuleLoan<E, ANALYTICS_RESOURCE>>(analytics_resource)
                                .await?;
                            analytics_resource =
                                analytics.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                        }
                        ANALYTICS => {
                            let status = request.recv::<ModuleFileClosed<ANALYTICS>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        F => {
                            let arguments = request.recv::<Submitted<F>>().await?;
                            analytics.send::<ModuleClaim<F_FILE, F>>(&arguments).await?;
                            analytics.send::<CommitRequest<F>>(&arguments).await?;
                            analytics
                                .send::<ModuleLoan<F, ANALYTICS_RESOURCE>>(analytics_resource)
                                .await?;
                            analytics_resource =
                                analytics.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = api_requests.offer().await?;
                    match request.label() {
                        B_FILE => {
                            request.recv::<ModuleClaim<B_FILE, B>>().await?;
                            api.send::<ModuleLoan<B, API_RESOURCE>>(api_resource)
                                .await?;
                            api_resource = api.recv::<Loan<API_RESOURCE>>().await?;
                        }
                        API => {
                            let status = request.recv::<ModuleFileClosed<API>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        C_FILE => {
                            request.recv::<ModuleClaim<C_FILE, C>>().await?;
                            api.send::<ModuleLoan<C, API_RESOURCE>>(api_resource)
                                .await?;
                            api_resource = api.recv::<Loan<API_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = auth_requests.offer().await?;
                    match request.label() {
                        G => {
                            let arguments = request.recv::<Submitted<G>>().await?;
                            auth.send::<ModuleClaim<G_FILE, G>>(&arguments).await?;
                            auth.send::<CommitRequest<G>>(&arguments).await?;
                            auth.send::<ModuleLoan<G, AUTH_RESOURCE>>(auth_resource)
                                .await?;
                            auth_resource = auth.recv::<Loan<AUTH_RESOURCE>>().await?;
                        }
                        AUTH => {
                            let status = request.recv::<ModuleFileClosed<AUTH>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        A_FILE => {
                            request.recv::<ModuleClaim<A_FILE, A>>().await?;
                            auth.send::<ModuleLoan<A, AUTH_RESOURCE>>(auth_resource)
                                .await?;
                            auth_resource = auth.recv::<Loan<AUTH_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = database_requests.offer().await?;
                    match request.label() {
                        A => {
                            let arguments = request.recv::<Submitted<A>>().await?;
                            database.send::<ModuleClaim<A_FILE, A>>(&arguments).await?;
                            database.send::<CommitRequest<A>>(&arguments).await?;
                            database
                                .send::<ModuleLoan<A, DATABASE_RESOURCE>>(database_resource)
                                .await?;
                            database_resource = database.recv::<Loan<DATABASE_RESOURCE>>().await?;
                        }
                        DATABASE => {
                            let status = request.recv::<ModuleFileClosed<DATABASE>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        B => {
                            let arguments = request.recv::<Submitted<B>>().await?;
                            database.send::<ModuleClaim<B_FILE, B>>(&arguments).await?;
                            database.send::<CommitRequest<B>>(&arguments).await?;
                            database
                                .send::<ModuleLoan<B, DATABASE_RESOURCE>>(database_resource)
                                .await?;
                            database_resource = database.recv::<Loan<DATABASE_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = notify_requests.offer().await?;
                    match request.label() {
                        D_FILE => {
                            request.recv::<ModuleClaim<D_FILE, D>>().await?;
                            notify
                                .send::<ModuleLoan<D, NOTIFY_RESOURCE>>(notify_resource)
                                .await?;
                            notify_resource = notify.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                        }
                        NOTIFY => {
                            let status = request.recv::<ModuleFileClosed<NOTIFY>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        E_FILE => {
                            request.recv::<ModuleClaim<E_FILE, E>>().await?;
                            notify
                                .send::<ModuleLoan<E, NOTIFY_RESOURCE>>(notify_resource)
                                .await?;
                            notify_resource = notify.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = payment_requests.offer().await?;
                    match request.label() {
                        C => {
                            let arguments = request.recv::<Submitted<C>>().await?;
                            payment.send::<ModuleClaim<C_FILE, C>>(&arguments).await?;
                            payment.send::<CommitRequest<C>>(&arguments).await?;
                            payment
                                .send::<ModuleLoan<C, PAYMENT_RESOURCE>>(payment_resource)
                                .await?;
                            payment_resource = payment.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                        }
                        PAYMENT => {
                            let status = request.recv::<ModuleFileClosed<PAYMENT>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        D => {
                            let arguments = request.recv::<Submitted<D>>().await?;
                            payment.send::<ModuleClaim<D_FILE, D>>(&arguments).await?;
                            payment.send::<CommitRequest<D>>(&arguments).await?;
                            payment
                                .send::<ModuleLoan<D, PAYMENT_RESOURCE>>(payment_resource)
                                .await?;
                            payment_resource = payment.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
        };
        async {
            for _ in 0..2 {
                loop {
                    let request = search_requests.offer().await?;
                    match request.label() {
                        F_FILE => {
                            request.recv::<ModuleClaim<F_FILE, F>>().await?;
                            search
                                .send::<ModuleLoan<F, SEARCH_RESOURCE>>(search_resource)
                                .await?;
                            search_resource = search.recv::<Loan<SEARCH_RESOURCE>>().await?;
                        }
                        SEARCH => {
                            let status = request.recv::<ModuleFileClosed<SEARCH>>().await?;
                            if status != 0 {
                                return Err(ShellError::message(format!(
                                    "module requester exited with {status}"
                                )));
                            }
                            break;
                        }
                        G_FILE => {
                            request.recv::<ModuleClaim<G_FILE, G>>().await?;
                            search
                                .send::<ModuleLoan<G, SEARCH_RESOURCE>>(search_resource)
                                .await?;
                            search_resource = search.recv::<Loan<SEARCH_RESOURCE>>().await?;
                        }

                        _ => return Err(ShellError::message("unknown module request")),
                    }
                }
            }
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
                    .write_as::<Submitted<A>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<DATABASE>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<AUTH>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<B>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<DATABASE>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<API>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<C>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<PAYMENT>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<API>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<D>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<PAYMENT>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<NOTIFY>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<E>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<ANALYTICS>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<NOTIFY>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<F>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<ANALYTICS>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<SEARCH>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
                    .write_as::<Submitted<G>>(&mut commit)
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
            async {
                let [mut commit, mut metadata] = file.par()?;
                async {
                    commit.send::<ModuleFileClosed<AUTH>>(&status).await?;
                };
                async {
                    metadata.send::<ModuleFileClosed<SEARCH>>(&status).await?;
                };
                Ok(())
            }
            .await?;
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
