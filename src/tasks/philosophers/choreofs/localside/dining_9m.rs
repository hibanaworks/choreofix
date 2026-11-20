//! Endpoint operations remain visible in each scoped participant body.
use super::super::{global, identity, imp::PhilosopherArguments};
use super::fork;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            file_data::FileData,
            private_fs::PrivateFs,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage, protocol,
};

#[hibana_runtime::scope]
pub(crate) async fn run(
    module: &[u8],
    (bus, simulator): (&Bus, &impl crate::shell::imp::service::PairedEffects),
    terminals: [Terminal; 5],
) -> ShellResult<()> {
    use global::*;
    use identity::roles::*;
    let programs = project_9m();
    bus.projected(programs.observation());
    let capture = |event| bus.capture(event);
    let a_input = crate::shell::imp::service::Validation::new();
    let b_input = crate::shell::imp::service::Validation::new();
    let c_input = crate::shell::imp::service::Validation::new();
    let d_input = crate::shell::imp::service::Validation::new();
    let e_input = crate::shell::imp::service::Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(identity::SESSION);
    let result = async {
        rv.set_resolver(
            &programs.eat_close_a,
            hibana::runtime::resolver::ResolverRef::<EAT_A_ARGUMENTS>::decision_state(
                &a_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut a = rv.enter(session, &programs.a)?;
        let mut think_a = rv.enter(session, &programs.think_a)?;
        let eat_close_a = rv.enter(session, &programs.eat_close_a)?;
        let mut fork_a = rv.enter(session, &programs.fork_a)?;
        let resource_a = rv.enter(session, &programs.resource_a)?;
        let think_file_a = rv.enter(session, &programs.a_think_file)?;
        let eat_file_a = rv.enter(session, &programs.a_eat_file)?;
        let a_wasi = rv.enter(session, &programs.a_wasi)?;
        let mut a_io = rv.enter(session, &programs.a_io)?;

        rv.set_resolver(
            &programs.eat_close_b,
            hibana::runtime::resolver::ResolverRef::<EAT_B_ARGUMENTS>::decision_state(
                &b_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut b = rv.enter(session, &programs.b)?;
        let mut think_b = rv.enter(session, &programs.think_b)?;
        let eat_close_b = rv.enter(session, &programs.eat_close_b)?;
        let mut fork_b = rv.enter(session, &programs.fork_b)?;
        let resource_b = rv.enter(session, &programs.resource_b)?;
        let think_file_b = rv.enter(session, &programs.b_think_file)?;
        let eat_file_b = rv.enter(session, &programs.b_eat_file)?;
        let b_wasi = rv.enter(session, &programs.b_wasi)?;
        let mut b_io = rv.enter(session, &programs.b_io)?;

        rv.set_resolver(
            &programs.eat_close_c,
            hibana::runtime::resolver::ResolverRef::<EAT_C_ARGUMENTS>::decision_state(
                &c_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut c = rv.enter(session, &programs.c)?;
        let mut think_c = rv.enter(session, &programs.think_c)?;
        let eat_close_c = rv.enter(session, &programs.eat_close_c)?;
        let mut fork_c = rv.enter(session, &programs.fork_c)?;
        let resource_c = rv.enter(session, &programs.resource_c)?;
        let think_file_c = rv.enter(session, &programs.c_think_file)?;
        let eat_file_c = rv.enter(session, &programs.c_eat_file)?;
        let c_wasi = rv.enter(session, &programs.c_wasi)?;
        let mut c_io = rv.enter(session, &programs.c_io)?;

        rv.set_resolver(
            &programs.eat_close_d,
            hibana::runtime::resolver::ResolverRef::<EAT_D_ARGUMENTS>::decision_state(
                &d_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut d = rv.enter(session, &programs.d)?;
        let mut think_d = rv.enter(session, &programs.think_d)?;
        let eat_close_d = rv.enter(session, &programs.eat_close_d)?;
        let mut fork_d = rv.enter(session, &programs.fork_d)?;
        let resource_d = rv.enter(session, &programs.resource_d)?;
        let think_file_d = rv.enter(session, &programs.d_think_file)?;
        let eat_file_d = rv.enter(session, &programs.d_eat_file)?;
        let d_wasi = rv.enter(session, &programs.d_wasi)?;
        let mut d_io = rv.enter(session, &programs.d_io)?;

        rv.set_resolver(
            &programs.eat_close_e,
            hibana::runtime::resolver::ResolverRef::<EAT_E_ARGUMENTS>::decision_state(
                &e_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut e = rv.enter(session, &programs.e)?;
        let mut think_e = rv.enter(session, &programs.think_e)?;
        let eat_close_e = rv.enter(session, &programs.eat_close_e)?;
        let mut fork_e = rv.enter(session, &programs.fork_e)?;
        let resource_e = rv.enter(session, &programs.resource_e)?;
        let think_file_e = rv.enter(session, &programs.e_think_file)?;
        let eat_file_e = rv.enter(session, &programs.e_eat_file)?;
        let e_wasi = rv.enter(session, &programs.e_wasi)?;
        let mut e_io = rv.enter(session, &programs.e_io)?;
        let mut requests = rv.enter(session, &programs.requests)?;
        let mut table = rv.enter(session, &programs.table)?;
        let [a_terminal, b_terminal, c_terminal, d_terminal, e_terminal] = terminals;
        async {
            for _ in 0..5 {
                let request = requests.offer().await?;
                match request.label() {
                    A_EAT_CLOSE => {
                        let args = request.recv::<TableMeal<A_EAT_CLOSE>>().await?;
                        table.send::<TableMeal<FORK_A>>(&args).await?;
                        table.send::<TableMeal<FORK_E_RESOURCE>>(&args).await?;
                    }
                    B_EAT_CLOSE => {
                        let args = request.recv::<TableMeal<B_EAT_CLOSE>>().await?;
                        table.send::<TableMeal<FORK_B>>(&args).await?;
                        table.send::<TableMeal<FORK_A_RESOURCE>>(&args).await?;
                    }
                    C_EAT_CLOSE => {
                        let args = request.recv::<TableMeal<C_EAT_CLOSE>>().await?;
                        table.send::<TableMeal<FORK_C>>(&args).await?;
                        table.send::<TableMeal<FORK_B_RESOURCE>>(&args).await?;
                    }
                    D_EAT_CLOSE => {
                        let args = request.recv::<TableMeal<D_EAT_CLOSE>>().await?;
                        table.send::<TableMeal<FORK_D>>(&args).await?;
                        table.send::<TableMeal<FORK_C_RESOURCE>>(&args).await?;
                    }
                    E_EAT_CLOSE => {
                        let args = request.recv::<TableMeal<E_EAT_CLOSE>>().await?;
                        table.send::<TableMeal<FORK_E>>(&args).await?;
                        table.send::<TableMeal<FORK_D_RESOURCE>>(&args).await?;
                    }
                    _ => return Err(ShellError::message("unknown diner request")),
                }
            }
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_a.send::<Loan<A_THINK_FILE>>(think_file_a).await?;
            let status = {
                let io = &mut think_a;
                loop {
                    let branch = io.offer().await?;
                    match branch.label() {
                        protocol::LABEL_WASI_PATH_OPEN => {
                            let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                            let response = data.open(received);
                            io.send::<protocol::PathOpenRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                            let received = branch.recv::<protocol::FdWriteObjectReqMsg>().await?;
                            let response = data.write(received)?;
                            io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_GET => {
                            let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                            let response = protocol::FdPrestatRet(
                                protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                            );
                            io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                            let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                            let response = protocol::FdPrestatDirNameRet(
                                protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                            );
                            io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_READDIR => {
                            let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                            let response =
                                data.objects().choreofs().fd_readdir(request).read_dir()?;
                            io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_READ => {
                            let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                            let response = data.read(received)?;
                            io.send::<protocol::FdReadRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FDSTAT_GET => {
                            let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_fdstat_get(request);
                            io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FILESTAT_GET => {
                            let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_filestat_get(request);
                            io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                            let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().path_filestat_get(request);
                            io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_CLOSE => {
                            let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                            let response = data.objects().choreofs().fd_close(request);
                            io.send::<protocol::FdCloseRetMsg>(&response).await?;
                        }

                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        identity::THINK_REQUEST => {
                            let received = branch.recv::<Think<THINK_A>>().await?;
                            let result = simulator.apply(received).await?;
                            data.replace(result)?;
                            io.send::<ThinkCompleted>(&()).await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected think-file request {label}"
                            )));
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            let mut data = FileData::new(EAT_OBJECT);
            a.send::<Loan<A_EAT_FILE>>(eat_file_a).await?;
            let status = {
                let mut authority = Some(eat_close_a);
                let io = &mut a;
                loop {
                    let incoming = if let Some(operation) = authority.as_mut() {
                        hibana_runtime::race(
                            async {
                                let branch = io.offer().await?;
                                Ok(core::ops::ControlFlow::Continue(branch.label()))
                            },
                            async {
                                operation
                                    .recv::<protocol::FdCloseObjectReqMsg>()
                                    .await
                                    .map(core::ops::ControlFlow::Break)
                            },
                        )
                        .await?
                    } else {
                        core::ops::ControlFlow::Continue(io.offer().await?.label())
                    };
                    match incoming {
                        core::ops::ControlFlow::Continue(label) => match label {
                            protocol::LABEL_WASI_PATH_OPEN => {
                                let received = io.recv::<protocol::PathOpenReqMsg>().await?;
                                let response = data.open(received);
                                io.send::<protocol::PathOpenRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                                let received = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                                let response = data.write(received)?;
                                io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                                let response = protocol::FdPrestatRet(
                                    protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                                );
                                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                                let response = protocol::FdPrestatDirNameRet(
                                    protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                                );
                                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_READDIR => {
                                let request = io.recv::<protocol::FdReaddirReqMsg>().await?;
                                let response =
                                    data.objects().choreofs().fd_readdir(request).read_dir()?;
                                io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_READ => {
                                let received = io.recv::<protocol::FdReadReqMsg>().await?;
                                let response = data.read(received)?;
                                io.send::<protocol::FdReadRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_fdstat_get(request);
                                io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_filestat_get(request);
                                io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().path_filestat_get(request);
                                io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_CLOSE => {
                                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                                let response = data.objects().choreofs().fd_close(request);
                                io.send::<protocol::FdCloseRetMsg>(&response).await?;
                            }

                            241 => {
                                let status = io.recv::<FileExit>().await?;
                                break status;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected eat-file request {label}"
                                )));
                            }
                        },
                        core::ops::ControlFlow::Break(request) => {
                            let close = data.objects().choreofs().fd_close(request);
                            let operation = authority.as_mut().ok_or_else(|| {
                                ShellError::message("missing owned close Endpoint")
                            })?;
                            a_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // Resolve from the actual decoded request before replying.
                            drop(operation.offer().await?);
                            let arguments =
                                a_input.take().and_then(PhilosopherArguments::from_value);
                            let admitted = match arguments {
                                Err(_) => {
                                    // A rejected close changes no file data.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                    continue;
                                }
                                Ok(arguments) => arguments,
                            };
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                            operation.send::<TableMeal<A_EAT_CLOSE>>(&admitted).await?;
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            operation
                                                .recv::<ForkRequest<FORK_A_RESOURCE, A>>()
                                                .await?,
                                        );
                                    let resource =
                                        operation.recv::<Loan<FORK_A_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            io.recv::<ForkRequest<FORK_E_RESOURCE, A>>().await?,
                                        );
                                    let resource = io.recv::<Loan<FORK_E_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left.as_bytes() != admitted.as_bytes()
                                || right.as_bytes() != admitted.as_bytes()
                            {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let result = simulator.apply_pair(left, right).await?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_A_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_E_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                async {
                                    async {
                                        operation
                                            .send::<Loan<FORK_A_RESOURCE>>(left_resource)
                                            .await?;
                                    };
                                    async {
                                        io.send::<Loan<FORK_E_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            io.send::<CloseAuthority<A_EAT_CLOSE>>(authority.take().ok_or_else(
                                || ShellError::message("missing owned close Endpoint"),
                            )?)
                            .await?;
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            fork::run::<B, FORK_A, FORK_A_RESOURCE>(&mut fork_a, resource_a).await?;
        };
        async {
            let mut endpoint = a_wasi;
            let (think, eat) = async {
                let [mut think, mut eat] = endpoint.par()?;
                async { think.recv::<Loan<A_THINK_FILE>>().await };
                async { eat.recv::<Loan<A_EAT_FILE>>().await };
            }
            .await?;
            let mut thought = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(THINK_OBJECT, think)
                        .write_as::<Think<THINK_A>>(&mut thought)
                        .close_with::<ThinkCompleted>(),
                )?
                .file(
                    ChoreoFsFile::new(EAT_OBJECT, eat).close_with::<CloseAuthority<A_EAT_CLOSE>>(),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (eat, (think, ()))) = files.into_parts();
            let mut think = think.into_endpoint();
            let mut eat = eat.into_endpoint();
            think.send::<FileExit>(&status).await?;
            eat.send::<FileExit>(&status).await?;
            drop(think);
            drop(eat);
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut a_io,
                PrivateFs::new(a_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_b.send::<Loan<B_THINK_FILE>>(think_file_b).await?;
            let status = {
                let io = &mut think_b;
                loop {
                    let branch = io.offer().await?;
                    match branch.label() {
                        protocol::LABEL_WASI_PATH_OPEN => {
                            let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                            let response = data.open(received);
                            io.send::<protocol::PathOpenRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                            let received = branch.recv::<protocol::FdWriteObjectReqMsg>().await?;
                            let response = data.write(received)?;
                            io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_GET => {
                            let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                            let response = protocol::FdPrestatRet(
                                protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                            );
                            io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                            let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                            let response = protocol::FdPrestatDirNameRet(
                                protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                            );
                            io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_READDIR => {
                            let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                            let response =
                                data.objects().choreofs().fd_readdir(request).read_dir()?;
                            io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_READ => {
                            let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                            let response = data.read(received)?;
                            io.send::<protocol::FdReadRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FDSTAT_GET => {
                            let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_fdstat_get(request);
                            io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FILESTAT_GET => {
                            let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_filestat_get(request);
                            io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                            let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().path_filestat_get(request);
                            io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_CLOSE => {
                            let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                            let response = data.objects().choreofs().fd_close(request);
                            io.send::<protocol::FdCloseRetMsg>(&response).await?;
                        }

                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        identity::THINK_REQUEST => {
                            let received = branch.recv::<Think<THINK_B>>().await?;
                            let result = simulator.apply(received).await?;
                            data.replace(result)?;
                            io.send::<ThinkCompleted>(&()).await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected think-file request {label}"
                            )));
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            let mut data = FileData::new(EAT_OBJECT);
            b.send::<Loan<B_EAT_FILE>>(eat_file_b).await?;
            let status = {
                let mut authority = Some(eat_close_b);
                let io = &mut b;
                loop {
                    let incoming = if let Some(operation) = authority.as_mut() {
                        hibana_runtime::race(
                            async {
                                let branch = io.offer().await?;
                                Ok(core::ops::ControlFlow::Continue(branch.label()))
                            },
                            async {
                                operation
                                    .recv::<protocol::FdCloseObjectReqMsg>()
                                    .await
                                    .map(core::ops::ControlFlow::Break)
                            },
                        )
                        .await?
                    } else {
                        core::ops::ControlFlow::Continue(io.offer().await?.label())
                    };
                    match incoming {
                        core::ops::ControlFlow::Continue(label) => match label {
                            protocol::LABEL_WASI_PATH_OPEN => {
                                let received = io.recv::<protocol::PathOpenReqMsg>().await?;
                                let response = data.open(received);
                                io.send::<protocol::PathOpenRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                                let received = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                                let response = data.write(received)?;
                                io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                                let response = protocol::FdPrestatRet(
                                    protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                                );
                                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                                let response = protocol::FdPrestatDirNameRet(
                                    protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                                );
                                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_READDIR => {
                                let request = io.recv::<protocol::FdReaddirReqMsg>().await?;
                                let response =
                                    data.objects().choreofs().fd_readdir(request).read_dir()?;
                                io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_READ => {
                                let received = io.recv::<protocol::FdReadReqMsg>().await?;
                                let response = data.read(received)?;
                                io.send::<protocol::FdReadRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_fdstat_get(request);
                                io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_filestat_get(request);
                                io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().path_filestat_get(request);
                                io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_CLOSE => {
                                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                                let response = data.objects().choreofs().fd_close(request);
                                io.send::<protocol::FdCloseRetMsg>(&response).await?;
                            }

                            241 => {
                                let status = io.recv::<FileExit>().await?;
                                break status;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected eat-file request {label}"
                                )));
                            }
                        },
                        core::ops::ControlFlow::Break(request) => {
                            let close = data.objects().choreofs().fd_close(request);
                            let operation = authority.as_mut().ok_or_else(|| {
                                ShellError::message("missing owned close Endpoint")
                            })?;
                            b_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // Resolve from the actual decoded request before replying.
                            drop(operation.offer().await?);
                            let arguments =
                                b_input.take().and_then(PhilosopherArguments::from_value);
                            let admitted = match arguments {
                                Err(_) => {
                                    // A rejected close changes no file data.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                    continue;
                                }
                                Ok(arguments) => arguments,
                            };
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                            operation.send::<TableMeal<B_EAT_CLOSE>>(&admitted).await?;
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            operation
                                                .recv::<ForkRequest<FORK_B_RESOURCE, B>>()
                                                .await?,
                                        );
                                    let resource =
                                        operation.recv::<Loan<FORK_B_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            io.recv::<ForkRequest<FORK_A_RESOURCE, B>>().await?,
                                        );
                                    let resource = io.recv::<Loan<FORK_A_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left.as_bytes() != admitted.as_bytes()
                                || right.as_bytes() != admitted.as_bytes()
                            {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let result = simulator.apply_pair(left, right).await?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_B_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_A_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                async {
                                    async {
                                        operation
                                            .send::<Loan<FORK_B_RESOURCE>>(left_resource)
                                            .await?;
                                    };
                                    async {
                                        io.send::<Loan<FORK_A_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            io.send::<CloseAuthority<B_EAT_CLOSE>>(authority.take().ok_or_else(
                                || ShellError::message("missing owned close Endpoint"),
                            )?)
                            .await?;
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            fork::run::<C, FORK_B, FORK_B_RESOURCE>(&mut fork_b, resource_b).await?;
        };
        async {
            let mut endpoint = b_wasi;
            let (think, eat) = async {
                let [mut think, mut eat] = endpoint.par()?;
                async { think.recv::<Loan<B_THINK_FILE>>().await };
                async { eat.recv::<Loan<B_EAT_FILE>>().await };
            }
            .await?;
            let mut thought = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(THINK_OBJECT, think)
                        .write_as::<Think<THINK_B>>(&mut thought)
                        .close_with::<ThinkCompleted>(),
                )?
                .file(
                    ChoreoFsFile::new(EAT_OBJECT, eat).close_with::<CloseAuthority<B_EAT_CLOSE>>(),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (eat, (think, ()))) = files.into_parts();
            let mut think = think.into_endpoint();
            let mut eat = eat.into_endpoint();
            think.send::<FileExit>(&status).await?;
            eat.send::<FileExit>(&status).await?;
            drop(think);
            drop(eat);
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut b_io,
                PrivateFs::new(b_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_c.send::<Loan<C_THINK_FILE>>(think_file_c).await?;
            let status = {
                let io = &mut think_c;
                loop {
                    let branch = io.offer().await?;
                    match branch.label() {
                        protocol::LABEL_WASI_PATH_OPEN => {
                            let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                            let response = data.open(received);
                            io.send::<protocol::PathOpenRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                            let received = branch.recv::<protocol::FdWriteObjectReqMsg>().await?;
                            let response = data.write(received)?;
                            io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_GET => {
                            let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                            let response = protocol::FdPrestatRet(
                                protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                            );
                            io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                            let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                            let response = protocol::FdPrestatDirNameRet(
                                protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                            );
                            io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_READDIR => {
                            let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                            let response =
                                data.objects().choreofs().fd_readdir(request).read_dir()?;
                            io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_READ => {
                            let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                            let response = data.read(received)?;
                            io.send::<protocol::FdReadRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FDSTAT_GET => {
                            let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_fdstat_get(request);
                            io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FILESTAT_GET => {
                            let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_filestat_get(request);
                            io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                            let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().path_filestat_get(request);
                            io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_CLOSE => {
                            let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                            let response = data.objects().choreofs().fd_close(request);
                            io.send::<protocol::FdCloseRetMsg>(&response).await?;
                        }

                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        identity::THINK_REQUEST => {
                            let received = branch.recv::<Think<THINK_C>>().await?;
                            let result = simulator.apply(received).await?;
                            data.replace(result)?;
                            io.send::<ThinkCompleted>(&()).await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected think-file request {label}"
                            )));
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            let mut data = FileData::new(EAT_OBJECT);
            c.send::<Loan<C_EAT_FILE>>(eat_file_c).await?;
            let status = {
                let mut authority = Some(eat_close_c);
                let io = &mut c;
                loop {
                    let incoming = if let Some(operation) = authority.as_mut() {
                        hibana_runtime::race(
                            async {
                                let branch = io.offer().await?;
                                Ok(core::ops::ControlFlow::Continue(branch.label()))
                            },
                            async {
                                operation
                                    .recv::<protocol::FdCloseObjectReqMsg>()
                                    .await
                                    .map(core::ops::ControlFlow::Break)
                            },
                        )
                        .await?
                    } else {
                        core::ops::ControlFlow::Continue(io.offer().await?.label())
                    };
                    match incoming {
                        core::ops::ControlFlow::Continue(label) => match label {
                            protocol::LABEL_WASI_PATH_OPEN => {
                                let received = io.recv::<protocol::PathOpenReqMsg>().await?;
                                let response = data.open(received);
                                io.send::<protocol::PathOpenRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                                let received = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                                let response = data.write(received)?;
                                io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                                let response = protocol::FdPrestatRet(
                                    protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                                );
                                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                                let response = protocol::FdPrestatDirNameRet(
                                    protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                                );
                                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_READDIR => {
                                let request = io.recv::<protocol::FdReaddirReqMsg>().await?;
                                let response =
                                    data.objects().choreofs().fd_readdir(request).read_dir()?;
                                io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_READ => {
                                let received = io.recv::<protocol::FdReadReqMsg>().await?;
                                let response = data.read(received)?;
                                io.send::<protocol::FdReadRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_fdstat_get(request);
                                io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_filestat_get(request);
                                io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().path_filestat_get(request);
                                io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_CLOSE => {
                                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                                let response = data.objects().choreofs().fd_close(request);
                                io.send::<protocol::FdCloseRetMsg>(&response).await?;
                            }

                            241 => {
                                let status = io.recv::<FileExit>().await?;
                                break status;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected eat-file request {label}"
                                )));
                            }
                        },
                        core::ops::ControlFlow::Break(request) => {
                            let close = data.objects().choreofs().fd_close(request);
                            let operation = authority.as_mut().ok_or_else(|| {
                                ShellError::message("missing owned close Endpoint")
                            })?;
                            c_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // Resolve from the actual decoded request before replying.
                            drop(operation.offer().await?);
                            let arguments =
                                c_input.take().and_then(PhilosopherArguments::from_value);
                            let admitted = match arguments {
                                Err(_) => {
                                    // A rejected close changes no file data.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                    continue;
                                }
                                Ok(arguments) => arguments,
                            };
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                            operation.send::<TableMeal<C_EAT_CLOSE>>(&admitted).await?;
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            operation
                                                .recv::<ForkRequest<FORK_C_RESOURCE, C>>()
                                                .await?,
                                        );
                                    let resource =
                                        operation.recv::<Loan<FORK_C_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            io.recv::<ForkRequest<FORK_B_RESOURCE, C>>().await?,
                                        );
                                    let resource = io.recv::<Loan<FORK_B_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left.as_bytes() != admitted.as_bytes()
                                || right.as_bytes() != admitted.as_bytes()
                            {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let result = simulator.apply_pair(left, right).await?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_C_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_B_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                async {
                                    async {
                                        operation
                                            .send::<Loan<FORK_C_RESOURCE>>(left_resource)
                                            .await?;
                                    };
                                    async {
                                        io.send::<Loan<FORK_B_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            io.send::<CloseAuthority<C_EAT_CLOSE>>(authority.take().ok_or_else(
                                || ShellError::message("missing owned close Endpoint"),
                            )?)
                            .await?;
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            fork::run::<D, FORK_C, FORK_C_RESOURCE>(&mut fork_c, resource_c).await?;
        };
        async {
            let mut endpoint = c_wasi;
            let (think, eat) = async {
                let [mut think, mut eat] = endpoint.par()?;
                async { think.recv::<Loan<C_THINK_FILE>>().await };
                async { eat.recv::<Loan<C_EAT_FILE>>().await };
            }
            .await?;
            let mut thought = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(THINK_OBJECT, think)
                        .write_as::<Think<THINK_C>>(&mut thought)
                        .close_with::<ThinkCompleted>(),
                )?
                .file(
                    ChoreoFsFile::new(EAT_OBJECT, eat).close_with::<CloseAuthority<C_EAT_CLOSE>>(),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (eat, (think, ()))) = files.into_parts();
            let mut think = think.into_endpoint();
            let mut eat = eat.into_endpoint();
            think.send::<FileExit>(&status).await?;
            eat.send::<FileExit>(&status).await?;
            drop(think);
            drop(eat);
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut c_io,
                PrivateFs::new(c_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_d.send::<Loan<D_THINK_FILE>>(think_file_d).await?;
            let status = {
                let io = &mut think_d;
                loop {
                    let branch = io.offer().await?;
                    match branch.label() {
                        protocol::LABEL_WASI_PATH_OPEN => {
                            let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                            let response = data.open(received);
                            io.send::<protocol::PathOpenRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                            let received = branch.recv::<protocol::FdWriteObjectReqMsg>().await?;
                            let response = data.write(received)?;
                            io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_GET => {
                            let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                            let response = protocol::FdPrestatRet(
                                protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                            );
                            io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                            let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                            let response = protocol::FdPrestatDirNameRet(
                                protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                            );
                            io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_READDIR => {
                            let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                            let response =
                                data.objects().choreofs().fd_readdir(request).read_dir()?;
                            io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_READ => {
                            let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                            let response = data.read(received)?;
                            io.send::<protocol::FdReadRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FDSTAT_GET => {
                            let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_fdstat_get(request);
                            io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FILESTAT_GET => {
                            let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_filestat_get(request);
                            io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                            let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().path_filestat_get(request);
                            io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_CLOSE => {
                            let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                            let response = data.objects().choreofs().fd_close(request);
                            io.send::<protocol::FdCloseRetMsg>(&response).await?;
                        }

                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        identity::THINK_REQUEST => {
                            let received = branch.recv::<Think<THINK_D>>().await?;
                            let result = simulator.apply(received).await?;
                            data.replace(result)?;
                            io.send::<ThinkCompleted>(&()).await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected think-file request {label}"
                            )));
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            let mut data = FileData::new(EAT_OBJECT);
            d.send::<Loan<D_EAT_FILE>>(eat_file_d).await?;
            let status = {
                let mut authority = Some(eat_close_d);
                let io = &mut d;
                loop {
                    let incoming = if let Some(operation) = authority.as_mut() {
                        hibana_runtime::race(
                            async {
                                let branch = io.offer().await?;
                                Ok(core::ops::ControlFlow::Continue(branch.label()))
                            },
                            async {
                                operation
                                    .recv::<protocol::FdCloseObjectReqMsg>()
                                    .await
                                    .map(core::ops::ControlFlow::Break)
                            },
                        )
                        .await?
                    } else {
                        core::ops::ControlFlow::Continue(io.offer().await?.label())
                    };
                    match incoming {
                        core::ops::ControlFlow::Continue(label) => match label {
                            protocol::LABEL_WASI_PATH_OPEN => {
                                let received = io.recv::<protocol::PathOpenReqMsg>().await?;
                                let response = data.open(received);
                                io.send::<protocol::PathOpenRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                                let received = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                                let response = data.write(received)?;
                                io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                                let response = protocol::FdPrestatRet(
                                    protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                                );
                                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                                let response = protocol::FdPrestatDirNameRet(
                                    protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                                );
                                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_READDIR => {
                                let request = io.recv::<protocol::FdReaddirReqMsg>().await?;
                                let response =
                                    data.objects().choreofs().fd_readdir(request).read_dir()?;
                                io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_READ => {
                                let received = io.recv::<protocol::FdReadReqMsg>().await?;
                                let response = data.read(received)?;
                                io.send::<protocol::FdReadRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_fdstat_get(request);
                                io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_filestat_get(request);
                                io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().path_filestat_get(request);
                                io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_CLOSE => {
                                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                                let response = data.objects().choreofs().fd_close(request);
                                io.send::<protocol::FdCloseRetMsg>(&response).await?;
                            }

                            241 => {
                                let status = io.recv::<FileExit>().await?;
                                break status;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected eat-file request {label}"
                                )));
                            }
                        },
                        core::ops::ControlFlow::Break(request) => {
                            let close = data.objects().choreofs().fd_close(request);
                            let operation = authority.as_mut().ok_or_else(|| {
                                ShellError::message("missing owned close Endpoint")
                            })?;
                            d_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // Resolve from the actual decoded request before replying.
                            drop(operation.offer().await?);
                            let arguments =
                                d_input.take().and_then(PhilosopherArguments::from_value);
                            let admitted = match arguments {
                                Err(_) => {
                                    // A rejected close changes no file data.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                    continue;
                                }
                                Ok(arguments) => arguments,
                            };
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                            operation.send::<TableMeal<D_EAT_CLOSE>>(&admitted).await?;
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            operation
                                                .recv::<ForkRequest<FORK_D_RESOURCE, D>>()
                                                .await?,
                                        );
                                    let resource =
                                        operation.recv::<Loan<FORK_D_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            io.recv::<ForkRequest<FORK_C_RESOURCE, D>>().await?,
                                        );
                                    let resource = io.recv::<Loan<FORK_C_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left.as_bytes() != admitted.as_bytes()
                                || right.as_bytes() != admitted.as_bytes()
                            {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let result = simulator.apply_pair(left, right).await?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_D_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_C_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                async {
                                    async {
                                        operation
                                            .send::<Loan<FORK_D_RESOURCE>>(left_resource)
                                            .await?;
                                    };
                                    async {
                                        io.send::<Loan<FORK_C_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            io.send::<CloseAuthority<D_EAT_CLOSE>>(authority.take().ok_or_else(
                                || ShellError::message("missing owned close Endpoint"),
                            )?)
                            .await?;
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            fork::run::<E, FORK_D, FORK_D_RESOURCE>(&mut fork_d, resource_d).await?;
        };
        async {
            let mut endpoint = d_wasi;
            let (think, eat) = async {
                let [mut think, mut eat] = endpoint.par()?;
                async { think.recv::<Loan<D_THINK_FILE>>().await };
                async { eat.recv::<Loan<D_EAT_FILE>>().await };
            }
            .await?;
            let mut thought = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(THINK_OBJECT, think)
                        .write_as::<Think<THINK_D>>(&mut thought)
                        .close_with::<ThinkCompleted>(),
                )?
                .file(
                    ChoreoFsFile::new(EAT_OBJECT, eat).close_with::<CloseAuthority<D_EAT_CLOSE>>(),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (eat, (think, ()))) = files.into_parts();
            let mut think = think.into_endpoint();
            let mut eat = eat.into_endpoint();
            think.send::<FileExit>(&status).await?;
            eat.send::<FileExit>(&status).await?;
            drop(think);
            drop(eat);
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut d_io,
                PrivateFs::new(d_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_e.send::<Loan<E_THINK_FILE>>(think_file_e).await?;
            let status = {
                let io = &mut think_e;
                loop {
                    let branch = io.offer().await?;
                    match branch.label() {
                        protocol::LABEL_WASI_PATH_OPEN => {
                            let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                            let response = data.open(received);
                            io.send::<protocol::PathOpenRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                            let received = branch.recv::<protocol::FdWriteObjectReqMsg>().await?;
                            let response = data.write(received)?;
                            io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_GET => {
                            let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                            let response = protocol::FdPrestatRet(
                                protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                            );
                            io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                            let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                            let response = protocol::FdPrestatDirNameRet(
                                protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                            );
                            io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_READDIR => {
                            let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                            let response =
                                data.objects().choreofs().fd_readdir(request).read_dir()?;
                            io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_READ => {
                            let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                            let response = data.read(received)?;
                            io.send::<protocol::FdReadRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FDSTAT_GET => {
                            let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_fdstat_get(request);
                            io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_FD_FILESTAT_GET => {
                            let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().fd_filestat_get(request);
                            io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                        }
                        protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                            let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                            let response = data.objects().choreofs().path_filestat_get(request);
                            io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                .await?;
                        }
                        protocol::LABEL_WASI_FD_CLOSE => {
                            let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                            let response = data.objects().choreofs().fd_close(request);
                            io.send::<protocol::FdCloseRetMsg>(&response).await?;
                        }

                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        identity::THINK_REQUEST => {
                            let received = branch.recv::<Think<THINK_E>>().await?;
                            let result = simulator.apply(received).await?;
                            data.replace(result)?;
                            io.send::<ThinkCompleted>(&()).await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected think-file request {label}"
                            )));
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            let mut data = FileData::new(EAT_OBJECT);
            e.send::<Loan<E_EAT_FILE>>(eat_file_e).await?;
            let status = {
                let mut authority = Some(eat_close_e);
                let io = &mut e;
                loop {
                    let incoming = if let Some(operation) = authority.as_mut() {
                        hibana_runtime::race(
                            async {
                                let branch = io.offer().await?;
                                Ok(core::ops::ControlFlow::Continue(branch.label()))
                            },
                            async {
                                operation
                                    .recv::<protocol::FdCloseObjectReqMsg>()
                                    .await
                                    .map(core::ops::ControlFlow::Break)
                            },
                        )
                        .await?
                    } else {
                        core::ops::ControlFlow::Continue(io.offer().await?.label())
                    };
                    match incoming {
                        core::ops::ControlFlow::Continue(label) => match label {
                            protocol::LABEL_WASI_PATH_OPEN => {
                                let received = io.recv::<protocol::PathOpenReqMsg>().await?;
                                let response = data.open(received);
                                io.send::<protocol::PathOpenRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                                let received = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                                let response = data.write(received)?;
                                io.send::<protocol::FdWriteObjectRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                                let response = protocol::FdPrestatRet(
                                    protocol::FdPrestat::new_with_errno(request.0.fd(), 0, 8),
                                );
                                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                                let response = protocol::FdPrestatDirNameRet(
                                    protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                                );
                                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_READDIR => {
                                let request = io.recv::<protocol::FdReaddirReqMsg>().await?;
                                let response =
                                    data.objects().choreofs().fd_readdir(request).read_dir()?;
                                io.send::<protocol::FdReaddirRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_READ => {
                                let received = io.recv::<protocol::FdReadReqMsg>().await?;
                                let response = data.read(received)?;
                                io.send::<protocol::FdReadRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_fdstat_get(request);
                                io.send::<protocol::FdFdstatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().fd_filestat_get(request);
                                io.send::<protocol::FdFilestatGetRetMsg>(&response).await?;
                            }
                            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                                let response = data.objects().choreofs().path_filestat_get(request);
                                io.send::<protocol::PathFilestatGetRetMsg>(&response)
                                    .await?;
                            }
                            protocol::LABEL_WASI_FD_CLOSE => {
                                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                                let response = data.objects().choreofs().fd_close(request);
                                io.send::<protocol::FdCloseRetMsg>(&response).await?;
                            }

                            241 => {
                                let status = io.recv::<FileExit>().await?;
                                break status;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected eat-file request {label}"
                                )));
                            }
                        },
                        core::ops::ControlFlow::Break(request) => {
                            let close = data.objects().choreofs().fd_close(request);
                            let operation = authority.as_mut().ok_or_else(|| {
                                ShellError::message("missing owned close Endpoint")
                            })?;
                            e_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // Resolve from the actual decoded request before replying.
                            drop(operation.offer().await?);
                            let arguments =
                                e_input.take().and_then(PhilosopherArguments::from_value);
                            let admitted = match arguments {
                                Err(_) => {
                                    // A rejected close changes no file data.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                    continue;
                                }
                                Ok(arguments) => arguments,
                            };
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                            operation.send::<TableMeal<E_EAT_CLOSE>>(&admitted).await?;
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            operation
                                                .recv::<ForkRequest<FORK_E_RESOURCE, E>>()
                                                .await?,
                                        );
                                    let resource =
                                        operation.recv::<Loan<FORK_E_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments =
                                        crate::shell::imp::service::ResourceReceipt::retain(
                                            io.recv::<ForkRequest<FORK_D_RESOURCE, E>>().await?,
                                        );
                                    let resource = io.recv::<Loan<FORK_D_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left.as_bytes() != admitted.as_bytes()
                                || right.as_bytes() != admitted.as_bytes()
                            {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let result = simulator.apply_pair(left, right).await?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_E_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_D_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                async {
                                    async {
                                        operation
                                            .send::<Loan<FORK_E_RESOURCE>>(left_resource)
                                            .await?;
                                    };
                                    async {
                                        io.send::<Loan<FORK_D_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            io.send::<CloseAuthority<E_EAT_CLOSE>>(authority.take().ok_or_else(
                                || ShellError::message("missing owned close Endpoint"),
                            )?)
                            .await?;
                        }
                    }
                }
            };
            if status != 0 {
                return Err(ShellError::message(format!("guest exited with {status}")));
            }
        };
        async {
            fork::run::<A, FORK_E, FORK_E_RESOURCE>(&mut fork_e, resource_e).await?;
        };
        async {
            let mut endpoint = e_wasi;
            let (think, eat) = async {
                let [mut think, mut eat] = endpoint.par()?;
                async { think.recv::<Loan<E_THINK_FILE>>().await };
                async { eat.recv::<Loan<E_EAT_FILE>>().await };
            }
            .await?;
            let mut thought = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let files = ChoreoFsFiles::new(endpoint)
                .file(
                    ChoreoFsFile::new(THINK_OBJECT, think)
                        .write_as::<Think<THINK_E>>(&mut thought)
                        .close_with::<ThinkCompleted>(),
                )?
                .file(
                    ChoreoFsFile::new(EAT_OBJECT, eat).close_with::<CloseAuthority<E_EAT_CLOSE>>(),
                )?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (eat, (think, ()))) = files.into_parts();
            let mut think = think.into_endpoint();
            let mut eat = eat.into_endpoint();
            think.send::<FileExit>(&status).await?;
            eat.send::<FileExit>(&status).await?;
            drop(think);
            drop(eat);
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut e_io,
                PrivateFs::new(e_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };
        Ok(())
    }
    .await;
    bus.checkpoint(session.raw(), observations.checkpoint());
    result
}
