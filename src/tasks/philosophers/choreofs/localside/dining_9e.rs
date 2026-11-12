//! Endpoint operations remain visible in each scoped participant body.
use super::super::{global, identity, imp::PhilosopherArguments};
use super::{fork, wasi};
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            file_data::FileData,
            private_fs::PrivateFs,
            service::Domain,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::protocol;

#[hibana_runtime::scope]
pub(crate) async fn run(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::*;
    use identity::roles::*;
    let programs = project_9e();
    bus.observe(programs.observation());
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
    };
    let a_input = crate::shell::imp::service::Validation::new();
    let think_a_input = crate::shell::imp::service::Validation::new();
    let b_input = crate::shell::imp::service::Validation::new();
    let think_b_input = crate::shell::imp::service::Validation::new();
    let c_input = crate::shell::imp::service::Validation::new();
    let think_c_input = crate::shell::imp::service::Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(identity::SESSION);
    let result = async {
        rv.set_resolver(
            &programs.a,
            hibana::runtime::resolver::ResolverRef::<EAT_A_ARGUMENTS>::decision_state(
                &a_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_a,
            hibana::runtime::resolver::ResolverRef::<THINK_A_ARGUMENTS>::decision_state(
                &think_a_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut a = rv.enter(session, &programs.a)?;
        let mut think_a = rv.enter(session, &programs.think_a)?;
        let mut fork_a = rv.enter(session, &programs.fork_a)?;
        let resource_a = rv.enter(session, &programs.resource_a)?;
        let think_file_a = rv.enter(session, &programs.a_think_file)?;
        let eat_file_a = rv.enter(session, &programs.a_eat_file)?;
        let a_wasi = rv.enter(session, &programs.a_wasi)?;
        let mut a_io = rv.enter(session, &programs.a_io)?;

        rv.set_resolver(
            &programs.b,
            hibana::runtime::resolver::ResolverRef::<EAT_B_ARGUMENTS>::decision_state(
                &b_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_b,
            hibana::runtime::resolver::ResolverRef::<THINK_B_ARGUMENTS>::decision_state(
                &think_b_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut b = rv.enter(session, &programs.b)?;
        let mut think_b = rv.enter(session, &programs.think_b)?;
        let mut fork_b = rv.enter(session, &programs.fork_b)?;
        let resource_b = rv.enter(session, &programs.resource_b)?;
        let think_file_b = rv.enter(session, &programs.b_think_file)?;
        let eat_file_b = rv.enter(session, &programs.b_eat_file)?;
        let b_wasi = rv.enter(session, &programs.b_wasi)?;
        let mut b_io = rv.enter(session, &programs.b_io)?;

        rv.set_resolver(
            &programs.c,
            hibana::runtime::resolver::ResolverRef::<EAT_C_ARGUMENTS>::decision_state(
                &c_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_c,
            hibana::runtime::resolver::ResolverRef::<THINK_C_ARGUMENTS>::decision_state(
                &think_c_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut c = rv.enter(session, &programs.c)?;
        let mut think_c = rv.enter(session, &programs.think_c)?;
        let mut fork_c = rv.enter(session, &programs.fork_c)?;
        let resource_c = rv.enter(session, &programs.resource_c)?;
        let think_file_c = rv.enter(session, &programs.c_think_file)?;
        let eat_file_c = rv.enter(session, &programs.c_eat_file)?;
        let c_wasi = rv.enter(session, &programs.c_wasi)?;
        let mut c_io = rv.enter(session, &programs.c_io)?;
        let [a_terminal, b_terminal, c_terminal] = terminals;

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_a.send::<Loan<A_THINK_FILE>>(think_file_a).await?;
            let status = {
                let [mut io, mut operation] = think_a.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            think_a_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_a_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_a_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_a_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_A % 7)];
                                    let result = bus
                                        .execute(agent, "think", arguments.into_value())
                                        .await
                                        .map_err(ShellError::message)?;
                                    data.replace(result)?;
                                    operation
                                        .send::<protocol::FdCloseObjectRetMsg>(&close)
                                        .await?;
                                }
                            }
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
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
                let [mut io, mut operation] = a.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            a_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded a_input; the preview publishes no effect.
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
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Eat<FORK_A_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_C_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_A_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_A_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_C_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_C_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(A % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
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
                                        .send::<Meal<FORK_C_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_A_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_C_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected eat-file request {label}"
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
            fork::run(&mut fork_a, resource_a).await?;
        };
        async {
            wasi::run::<A_WASI, A_THINK_FILE, A_EAT_FILE>(module, a_wasi).await?;
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
                let [mut io, mut operation] = think_b.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            think_b_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_b_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_b_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_b_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_B % 7)];
                                    let result = bus
                                        .execute(agent, "think", arguments.into_value())
                                        .await
                                        .map_err(ShellError::message)?;
                                    data.replace(result)?;
                                    operation
                                        .send::<protocol::FdCloseObjectRetMsg>(&close)
                                        .await?;
                                }
                            }
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
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
                let [mut io, mut operation] = b.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            b_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded b_input; the preview publishes no effect.
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
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Eat<FORK_B_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_A_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_B_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_B_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_A_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_A_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(B % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
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
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_B_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_A_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected eat-file request {label}"
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
            fork::run(&mut fork_b, resource_b).await?;
        };
        async {
            wasi::run::<B_WASI, B_THINK_FILE, B_EAT_FILE>(module, b_wasi).await?;
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
                let [mut io, mut operation] = think_c.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            think_c_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_c_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_c_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_c_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_C % 7)];
                                    let result = bus
                                        .execute(agent, "think", arguments.into_value())
                                        .await
                                        .map_err(ShellError::message)?;
                                    data.replace(result)?;
                                    operation
                                        .send::<protocol::FdCloseObjectRetMsg>(&close)
                                        .await?;
                                }
                            }
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
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
                let [mut io, mut operation] = c.par()?;
                loop {
                    let branch = hibana_runtime::race(io.offer(), operation.offer()).await?;
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
                        protocol::LABEL_WASI_FD_CLOSE_OBJECT => {
                            let request = branch.recv::<protocol::FdCloseObjectReqMsg>().await?;
                            let close = data.objects().choreofs().fd_close(request);
                            c_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded c_input; the preview publishes no effect.
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
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Eat<FORK_C_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_B_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_C_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_C_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_B_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_B_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(C % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
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
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_C_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_B_RESOURCE>>(right_resource).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            operation
                                .send::<protocol::FdCloseObjectRetMsg>(&close)
                                .await?;
                        }
                        241 => {
                            let status = branch.recv::<FileExit>().await?;
                            break status;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected eat-file request {label}"
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
            fork::run(&mut fork_c, resource_c).await?;
        };
        async {
            wasi::run::<C_WASI, C_THINK_FILE, C_EAT_FILE>(module, c_wasi).await?;
        };
        async {
            private_io::run(
                &mut c_io,
                PrivateFs::new(c_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };
        Ok(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),"checkpoint":observations.checkpoint()}));
    result
}
