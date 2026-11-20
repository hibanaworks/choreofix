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
        let mut submissions = rv.enter(session, &programs.submissions)?;
        let mut editor = rv.enter(session, &programs.editor)?;
        let mut intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut results_resource = rv.enter(session, &programs.results_resource)?;
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
            for _ in [A, B, C] {
                let request = submissions.offer().await?;
                match request.label() {
                    A => {
                        let args = request.recv::<Submitted<A>>().await?;
                        editor.send::<WriteRequest<A>>(&args).await?;
                        async {
                            let [mut first, mut second] = editor.par()?;
                            async {
                                first.send::<Loan<INTRO_RESOURCE>>(intro_resource).await?;
                            };
                            async {
                                second
                                    .send::<Loan<METHODS_RESOURCE>>(methods_resource)
                                    .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = editor.par()?;
                        (intro_resource, methods_resource) = async {
                            async { first.recv::<Loan<INTRO_RESOURCE>>().await };
                            async { second.recv::<Loan<METHODS_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    B => {
                        let args = request.recv::<Submitted<B>>().await?;
                        editor.send::<WriteRequest<B>>(&args).await?;
                        async {
                            let [mut first, mut second] = editor.par()?;
                            async {
                                first
                                    .send::<Loan<METHODS_RESOURCE>>(methods_resource)
                                    .await?;
                            };
                            async {
                                second
                                    .send::<Loan<RESULTS_RESOURCE>>(results_resource)
                                    .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = editor.par()?;
                        (methods_resource, results_resource) = async {
                            async { first.recv::<Loan<METHODS_RESOURCE>>().await };
                            async { second.recv::<Loan<RESULTS_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    C => {
                        let args = request.recv::<Submitted<C>>().await?;
                        editor.send::<WriteRequest<C>>(&args).await?;
                        async {
                            let [mut first, mut second] = editor.par()?;
                            async {
                                first.send::<Loan<INTRO_RESOURCE>>(intro_resource).await?;
                            };
                            async {
                                second
                                    .send::<Loan<RESULTS_RESOURCE>>(results_resource)
                                    .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let [mut first, mut second] = editor.par()?;
                        (intro_resource, results_resource) = async {
                            async { first.recv::<Loan<INTRO_RESOURCE>>().await };
                            async { second.recv::<Loan<RESULTS_RESOURCE>>().await };
                        }
                        .await?;
                    }
                    _ => return Err(ShellError::message("unknown writing submission")),
                }
            }
            Ok(())
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
                    .write_as::<Submitted<A>>(&mut write)
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
                    .write_as::<Submitted<B>>(&mut write)
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
                    .write_as::<Submitted<C>>(&mut write)
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
        let mut submissions = rv.enter(session, &programs.submissions)?;
        let mut editor = rv.enter(session, &programs.editor)?;
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
        let mut background_resource = rv.enter(session, &programs.background_resource)?;
        let mut intro = rv.enter(session, &programs.intro)?;
        let mut intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut methods = rv.enter(session, &programs.methods)?;
        let mut methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut experiments = rv.enter(session, &programs.experiments)?;
        let mut experiments_resource = rv.enter(session, &programs.experiments_resource)?;
        let mut results = rv.enter(session, &programs.results)?;
        let mut results_resource = rv.enter(session, &programs.results_resource)?;
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
            localside::section_writer::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(&mut a, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<B, BACKGROUND_RESOURCE, METHODS_RESOURCE>(&mut b, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<C, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
                &mut c, bus,
            )
            .await?;
        };
        async {
            localside::section_writer::run::<D, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
                &mut d, bus,
            )
            .await?;
        };
        async {
            localside::section_writer::run::<E, INTRO_RESOURCE, RESULTS_RESOURCE>(&mut e, bus)
                .await?;
        };
        async {
            for _ in 0..10 {
                let receipt = submissions.offer().await?;
                match receipt.label() {
                    A => {
                        let arguments = receipt.recv::<Submitted<A>>().await?;
                        editor.send::<Submitted<A>>(&arguments).await?;
                    }
                    A_FILE => {
                        let arguments = receipt.recv::<SectionClaim<A_FILE, A>>().await?;
                        editor.send::<SectionClaim<A_FILE, A>>(&arguments).await?;
                    }
                    B => {
                        let arguments = receipt.recv::<Submitted<B>>().await?;
                        editor.send::<Submitted<B>>(&arguments).await?;
                    }
                    B_FILE => {
                        let arguments = receipt.recv::<SectionClaim<B_FILE, B>>().await?;
                        editor.send::<SectionClaim<B_FILE, B>>(&arguments).await?;
                    }
                    C => {
                        let arguments = receipt.recv::<Submitted<C>>().await?;
                        editor.send::<Submitted<C>>(&arguments).await?;
                    }
                    C_FILE => {
                        let arguments = receipt.recv::<SectionClaim<C_FILE, C>>().await?;
                        editor.send::<SectionClaim<C_FILE, C>>(&arguments).await?;
                    }
                    D => {
                        let arguments = receipt.recv::<Submitted<D>>().await?;
                        editor.send::<Submitted<D>>(&arguments).await?;
                    }
                    D_FILE => {
                        let arguments = receipt.recv::<SectionClaim<D_FILE, D>>().await?;
                        editor.send::<SectionClaim<D_FILE, D>>(&arguments).await?;
                    }
                    E => {
                        let arguments = receipt.recv::<Submitted<E>>().await?;
                        editor.send::<Submitted<E>>(&arguments).await?;
                    }
                    E_FILE => {
                        let arguments = receipt.recv::<SectionClaim<E_FILE, E>>().await?;
                        editor.send::<SectionClaim<E_FILE, E>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown section request")),
                }
            }
        };
        async {
            for _ in 0..2 {
                let claim = background.offer().await?;
                match claim.label() {
                    A => {
                        let arguments = claim.recv::<Submitted<A>>().await?;
                        background
                            .send::<SectionClaim<A_FILE, A>>(&arguments)
                            .await?;
                        background.send::<WriteRequest<A>>(&arguments).await?;
                    }
                    B => {
                        let arguments = claim.recv::<Submitted<B>>().await?;
                        background
                            .send::<SectionClaim<B_FILE, B>>(&arguments)
                            .await?;
                        background.send::<WriteRequest<B>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown background section claim")),
                }
                background
                    .send::<Loan<BACKGROUND_RESOURCE>>(background_resource)
                    .await?;
                background_resource = background.recv::<Loan<BACKGROUND_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = experiments.offer().await?;
                match claim.label() {
                    C => {
                        let arguments = claim.recv::<Submitted<C>>().await?;
                        experiments
                            .send::<SectionClaim<C_FILE, C>>(&arguments)
                            .await?;
                        experiments.send::<WriteRequest<C>>(&arguments).await?;
                    }
                    D => {
                        let arguments = claim.recv::<Submitted<D>>().await?;
                        experiments
                            .send::<SectionClaim<D_FILE, D>>(&arguments)
                            .await?;
                        experiments.send::<WriteRequest<D>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown experiments section claim")),
                }
                experiments
                    .send::<Loan<EXPERIMENTS_RESOURCE>>(experiments_resource)
                    .await?;
                experiments_resource = experiments.recv::<Loan<EXPERIMENTS_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = intro.offer().await?;
                match claim.label() {
                    E => {
                        let arguments = claim.recv::<Submitted<E>>().await?;
                        intro.send::<SectionClaim<E_FILE, E>>(&arguments).await?;
                        intro.send::<WriteRequest<E>>(&arguments).await?;
                    }
                    A_FILE => {
                        claim.recv::<SectionClaim<A_FILE, A>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown intro section claim")),
                }
                intro.send::<Loan<INTRO_RESOURCE>>(intro_resource).await?;
                intro_resource = intro.recv::<Loan<INTRO_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = methods.offer().await?;
                match claim.label() {
                    B_FILE => {
                        claim.recv::<SectionClaim<B_FILE, B>>().await?;
                    }
                    C_FILE => {
                        claim.recv::<SectionClaim<C_FILE, C>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown methods section claim")),
                }
                methods
                    .send::<Loan<METHODS_RESOURCE>>(methods_resource)
                    .await?;
                methods_resource = methods.recv::<Loan<METHODS_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = results.offer().await?;
                match claim.label() {
                    D_FILE => {
                        claim.recv::<SectionClaim<D_FILE, D>>().await?;
                    }
                    E_FILE => {
                        claim.recv::<SectionClaim<E_FILE, E>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown results section claim")),
                }
                results
                    .send::<Loan<RESULTS_RESOURCE>>(results_resource)
                    .await?;
                results_resource = results.recv::<Loan<RESULTS_RESOURCE>>().await?;
            }
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
                    .write_as::<Submitted<A>>(&mut write)
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
                    .write_as::<Submitted<B>>(&mut write)
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
                    .write_as::<Submitted<C>>(&mut write)
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
                    .write_as::<Submitted<D>>(&mut write)
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
                    .write_as::<Submitted<E>>(&mut write)
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
        let mut submissions = rv.enter(session, &programs.submissions)?;
        let mut editor = rv.enter(session, &programs.editor)?;
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
        let mut background_resource = rv.enter(session, &programs.background_resource)?;
        let mut intro = rv.enter(session, &programs.intro)?;
        let mut intro_resource = rv.enter(session, &programs.intro_resource)?;
        let mut related = rv.enter(session, &programs.related)?;
        let mut related_resource = rv.enter(session, &programs.related_resource)?;
        let mut methods = rv.enter(session, &programs.methods)?;
        let mut methods_resource = rv.enter(session, &programs.methods_resource)?;
        let mut experiments = rv.enter(session, &programs.experiments)?;
        let mut experiments_resource = rv.enter(session, &programs.experiments_resource)?;
        let mut results = rv.enter(session, &programs.results)?;
        let mut results_resource = rv.enter(session, &programs.results_resource)?;
        let mut discussion = rv.enter(session, &programs.discussion)?;
        let mut discussion_resource = rv.enter(session, &programs.discussion_resource)?;
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
            localside::section_writer::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(&mut a, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<B, BACKGROUND_RESOURCE, RELATED_RESOURCE>(&mut b, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<C, METHODS_RESOURCE, RELATED_RESOURCE>(&mut c, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<D, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
                &mut d, bus,
            )
            .await?;
        };
        async {
            localside::section_writer::run::<E, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
                &mut e, bus,
            )
            .await?;
        };
        async {
            localside::section_writer::run::<F, DISCUSSION_RESOURCE, RESULTS_RESOURCE>(&mut f, bus)
                .await?;
        };
        async {
            localside::section_writer::run::<G, DISCUSSION_RESOURCE, INTRO_RESOURCE>(&mut g, bus)
                .await?;
        };
        async {
            for _ in 0..14 {
                let receipt = submissions.offer().await?;
                match receipt.label() {
                    A => {
                        let arguments = receipt.recv::<Submitted<A>>().await?;
                        editor.send::<Submitted<A>>(&arguments).await?;
                    }
                    A_FILE => {
                        let arguments = receipt.recv::<SectionClaim<A_FILE, A>>().await?;
                        editor.send::<SectionClaim<A_FILE, A>>(&arguments).await?;
                    }
                    B => {
                        let arguments = receipt.recv::<Submitted<B>>().await?;
                        editor.send::<Submitted<B>>(&arguments).await?;
                    }
                    B_FILE => {
                        let arguments = receipt.recv::<SectionClaim<B_FILE, B>>().await?;
                        editor.send::<SectionClaim<B_FILE, B>>(&arguments).await?;
                    }
                    C => {
                        let arguments = receipt.recv::<Submitted<C>>().await?;
                        editor.send::<Submitted<C>>(&arguments).await?;
                    }
                    C_FILE => {
                        let arguments = receipt.recv::<SectionClaim<C_FILE, C>>().await?;
                        editor.send::<SectionClaim<C_FILE, C>>(&arguments).await?;
                    }
                    D => {
                        let arguments = receipt.recv::<Submitted<D>>().await?;
                        editor.send::<Submitted<D>>(&arguments).await?;
                    }
                    D_FILE => {
                        let arguments = receipt.recv::<SectionClaim<D_FILE, D>>().await?;
                        editor.send::<SectionClaim<D_FILE, D>>(&arguments).await?;
                    }
                    E => {
                        let arguments = receipt.recv::<Submitted<E>>().await?;
                        editor.send::<Submitted<E>>(&arguments).await?;
                    }
                    E_FILE => {
                        let arguments = receipt.recv::<SectionClaim<E_FILE, E>>().await?;
                        editor.send::<SectionClaim<E_FILE, E>>(&arguments).await?;
                    }
                    F => {
                        let arguments = receipt.recv::<Submitted<F>>().await?;
                        editor.send::<Submitted<F>>(&arguments).await?;
                    }
                    F_FILE => {
                        let arguments = receipt.recv::<SectionClaim<F_FILE, F>>().await?;
                        editor.send::<SectionClaim<F_FILE, F>>(&arguments).await?;
                    }
                    G => {
                        let arguments = receipt.recv::<Submitted<G>>().await?;
                        editor.send::<Submitted<G>>(&arguments).await?;
                    }
                    G_FILE => {
                        let arguments = receipt.recv::<SectionClaim<G_FILE, G>>().await?;
                        editor.send::<SectionClaim<G_FILE, G>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown section request")),
                }
            }
        };
        async {
            for _ in 0..2 {
                let claim = background.offer().await?;
                match claim.label() {
                    A => {
                        let arguments = claim.recv::<Submitted<A>>().await?;
                        background
                            .send::<SectionClaim<A_FILE, A>>(&arguments)
                            .await?;
                        background.send::<WriteRequest<A>>(&arguments).await?;
                    }
                    B => {
                        let arguments = claim.recv::<Submitted<B>>().await?;
                        background
                            .send::<SectionClaim<B_FILE, B>>(&arguments)
                            .await?;
                        background.send::<WriteRequest<B>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown background section claim")),
                }
                background
                    .send::<Loan<BACKGROUND_RESOURCE>>(background_resource)
                    .await?;
                background_resource = background.recv::<Loan<BACKGROUND_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = discussion.offer().await?;
                match claim.label() {
                    F_FILE => {
                        claim.recv::<SectionClaim<F_FILE, F>>().await?;
                    }
                    G_FILE => {
                        claim.recv::<SectionClaim<G_FILE, G>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown discussion section claim")),
                }
                discussion
                    .send::<Loan<DISCUSSION_RESOURCE>>(discussion_resource)
                    .await?;
                discussion_resource = discussion.recv::<Loan<DISCUSSION_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = experiments.offer().await?;
                match claim.label() {
                    D_FILE => {
                        claim.recv::<SectionClaim<D_FILE, D>>().await?;
                    }
                    E_FILE => {
                        claim.recv::<SectionClaim<E_FILE, E>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown experiments section claim")),
                }
                experiments
                    .send::<Loan<EXPERIMENTS_RESOURCE>>(experiments_resource)
                    .await?;
                experiments_resource = experiments.recv::<Loan<EXPERIMENTS_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = intro.offer().await?;
                match claim.label() {
                    G => {
                        let arguments = claim.recv::<Submitted<G>>().await?;
                        intro.send::<SectionClaim<G_FILE, G>>(&arguments).await?;
                        intro.send::<WriteRequest<G>>(&arguments).await?;
                    }
                    A_FILE => {
                        claim.recv::<SectionClaim<A_FILE, A>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown intro section claim")),
                }
                intro.send::<Loan<INTRO_RESOURCE>>(intro_resource).await?;
                intro_resource = intro.recv::<Loan<INTRO_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = methods.offer().await?;
                match claim.label() {
                    C => {
                        let arguments = claim.recv::<Submitted<C>>().await?;
                        methods.send::<SectionClaim<C_FILE, C>>(&arguments).await?;
                        methods.send::<WriteRequest<C>>(&arguments).await?;
                    }
                    D => {
                        let arguments = claim.recv::<Submitted<D>>().await?;
                        methods.send::<SectionClaim<D_FILE, D>>(&arguments).await?;
                        methods.send::<WriteRequest<D>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown methods section claim")),
                }
                methods
                    .send::<Loan<METHODS_RESOURCE>>(methods_resource)
                    .await?;
                methods_resource = methods.recv::<Loan<METHODS_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = related.offer().await?;
                match claim.label() {
                    B_FILE => {
                        claim.recv::<SectionClaim<B_FILE, B>>().await?;
                    }
                    C_FILE => {
                        claim.recv::<SectionClaim<C_FILE, C>>().await?;
                    }
                    _ => return Err(ShellError::message("unknown related section claim")),
                }
                related
                    .send::<Loan<RELATED_RESOURCE>>(related_resource)
                    .await?;
                related_resource = related.recv::<Loan<RELATED_RESOURCE>>().await?;
            }
        };
        async {
            for _ in 0..2 {
                let claim = results.offer().await?;
                match claim.label() {
                    E => {
                        let arguments = claim.recv::<Submitted<E>>().await?;
                        results.send::<SectionClaim<E_FILE, E>>(&arguments).await?;
                        results.send::<WriteRequest<E>>(&arguments).await?;
                    }
                    F => {
                        let arguments = claim.recv::<Submitted<F>>().await?;
                        results.send::<SectionClaim<F_FILE, F>>(&arguments).await?;
                        results.send::<WriteRequest<F>>(&arguments).await?;
                    }
                    _ => return Err(ShellError::message("unknown results section claim")),
                }
                results
                    .send::<Loan<RESULTS_RESOURCE>>(results_resource)
                    .await?;
                results_resource = results.recv::<Loan<RESULTS_RESOURCE>>().await?;
            }
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
                    .write_as::<Submitted<A>>(&mut write)
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
                    .write_as::<Submitted<B>>(&mut write)
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
                    .write_as::<Submitted<C>>(&mut write)
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
                    .write_as::<Submitted<D>>(&mut write)
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
                    .write_as::<Submitted<E>>(&mut write)
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
                    .write_as::<Submitted<F>>(&mut write)
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
                    .write_as::<Submitted<G>>(&mut write)
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
