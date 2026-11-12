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
pub(crate) async fn run(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use global::*;
    use identity::roles::*;
    let programs = project_9h();
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
    let d_input = crate::shell::imp::service::Validation::new();
    let think_d_input = crate::shell::imp::service::Validation::new();
    let e_input = crate::shell::imp::service::Validation::new();
    let think_e_input = crate::shell::imp::service::Validation::new();
    let f_input = crate::shell::imp::service::Validation::new();
    let think_f_input = crate::shell::imp::service::Validation::new();
    let g_input = crate::shell::imp::service::Validation::new();
    let think_g_input = crate::shell::imp::service::Validation::new();
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

        rv.set_resolver(
            &programs.d,
            hibana::runtime::resolver::ResolverRef::<EAT_D_ARGUMENTS>::decision_state(
                &d_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_d,
            hibana::runtime::resolver::ResolverRef::<THINK_D_ARGUMENTS>::decision_state(
                &think_d_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut d = rv.enter(session, &programs.d)?;
        let mut think_d = rv.enter(session, &programs.think_d)?;
        let mut fork_d = rv.enter(session, &programs.fork_d)?;
        let resource_d = rv.enter(session, &programs.resource_d)?;
        let think_file_d = rv.enter(session, &programs.d_think_file)?;
        let eat_file_d = rv.enter(session, &programs.d_eat_file)?;
        let d_wasi = rv.enter(session, &programs.d_wasi)?;
        let mut d_io = rv.enter(session, &programs.d_io)?;

        rv.set_resolver(
            &programs.e,
            hibana::runtime::resolver::ResolverRef::<EAT_E_ARGUMENTS>::decision_state(
                &e_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_e,
            hibana::runtime::resolver::ResolverRef::<THINK_E_ARGUMENTS>::decision_state(
                &think_e_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut e = rv.enter(session, &programs.e)?;
        let mut think_e = rv.enter(session, &programs.think_e)?;
        let mut fork_e = rv.enter(session, &programs.fork_e)?;
        let resource_e = rv.enter(session, &programs.resource_e)?;
        let think_file_e = rv.enter(session, &programs.e_think_file)?;
        let eat_file_e = rv.enter(session, &programs.e_eat_file)?;
        let e_wasi = rv.enter(session, &programs.e_wasi)?;
        let mut e_io = rv.enter(session, &programs.e_io)?;

        rv.set_resolver(
            &programs.f,
            hibana::runtime::resolver::ResolverRef::<EAT_F_ARGUMENTS>::decision_state(
                &f_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_f,
            hibana::runtime::resolver::ResolverRef::<THINK_F_ARGUMENTS>::decision_state(
                &think_f_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut f = rv.enter(session, &programs.f)?;
        let mut think_f = rv.enter(session, &programs.think_f)?;
        let mut fork_f = rv.enter(session, &programs.fork_f)?;
        let resource_f = rv.enter(session, &programs.resource_f)?;
        let think_file_f = rv.enter(session, &programs.f_think_file)?;
        let eat_file_f = rv.enter(session, &programs.f_eat_file)?;
        let f_wasi = rv.enter(session, &programs.f_wasi)?;
        let mut f_io = rv.enter(session, &programs.f_io)?;

        rv.set_resolver(
            &programs.g,
            hibana::runtime::resolver::ResolverRef::<EAT_G_ARGUMENTS>::decision_state(
                &g_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.think_g,
            hibana::runtime::resolver::ResolverRef::<THINK_G_ARGUMENTS>::decision_state(
                &think_g_input,
                crate::shell::imp::service::Validation::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut g = rv.enter(session, &programs.g)?;
        let mut think_g = rv.enter(session, &programs.think_g)?;
        let mut fork_g = rv.enter(session, &programs.fork_g)?;
        let resource_g = rv.enter(session, &programs.resource_g)?;
        let think_file_g = rv.enter(session, &programs.g_think_file)?;
        let eat_file_g = rv.enter(session, &programs.g_eat_file)?;
        let g_wasi = rv.enter(session, &programs.g_wasi)?;
        let mut g_io = rv.enter(session, &programs.g_io)?;
        let [
            a_terminal,
            b_terminal,
            c_terminal,
            d_terminal,
            e_terminal,
            f_terminal,
            g_terminal,
        ] = terminals;

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
                                        right.send::<Eat<FORK_G_RESOURCE>>(&admitted).await?;
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
                                    let arguments = right.recv::<Eat<FORK_G_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_G_RESOURCE>>().await?;
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
                                        .send::<Meal<FORK_G_RESOURCE>>(&response)
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
                                        right.send::<Loan<FORK_G_RESOURCE>>(right_resource).await?;
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

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_d.send::<Loan<D_THINK_FILE>>(think_file_d).await?;
            let status = {
                let [mut io, mut operation] = think_d.par()?;
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
                            think_d_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_d_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_d_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_d_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_D % 7)];
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
            d.send::<Loan<D_EAT_FILE>>(eat_file_d).await?;
            let status = {
                let [mut io, mut operation] = d.par()?;
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
                            d_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded d_input; the preview publishes no effect.
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
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Eat<FORK_D_RESOURCE>>(&admitted).await?;
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
                                    let arguments = left.recv::<Eat<FORK_D_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_D_RESOURCE>>().await?;
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
                            let agent = identity::AGENTS[usize::from(D % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
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
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_D_RESOURCE>>(left_resource).await?;
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
            fork::run(&mut fork_d, resource_d).await?;
        };
        async {
            wasi::run::<D_WASI, D_THINK_FILE, D_EAT_FILE>(module, d_wasi).await?;
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
                let [mut io, mut operation] = think_e.par()?;
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
                            think_e_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_e_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_e_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_e_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_E % 7)];
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
            e.send::<Loan<E_EAT_FILE>>(eat_file_e).await?;
            let status = {
                let [mut io, mut operation] = e.par()?;
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
                            e_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded e_input; the preview publishes no effect.
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
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Eat<FORK_E_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_D_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_E_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_E_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_D_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_D_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(E % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
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
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_E_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_D_RESOURCE>>(right_resource).await?;
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
            fork::run(&mut fork_e, resource_e).await?;
        };
        async {
            wasi::run::<E_WASI, E_THINK_FILE, E_EAT_FILE>(module, e_wasi).await?;
        };
        async {
            private_io::run(
                &mut e_io,
                PrivateFs::new(e_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_f.send::<Loan<F_THINK_FILE>>(think_file_f).await?;
            let status = {
                let [mut io, mut operation] = think_f.par()?;
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
                            think_f_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_f_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_f_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_f_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_F % 7)];
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
            f.send::<Loan<F_EAT_FILE>>(eat_file_f).await?;
            let status = {
                let [mut io, mut operation] = f.par()?;
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
                            f_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded f_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments =
                                f_input.take().and_then(PhilosopherArguments::from_value);
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
                                        left.send::<Eat<FORK_F_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_E_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_F_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_F_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_E_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_E_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(F % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_F_RESOURCE>>(&response)
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
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_F_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_E_RESOURCE>>(right_resource).await?;
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
            fork::run(&mut fork_f, resource_f).await?;
        };
        async {
            wasi::run::<F_WASI, F_THINK_FILE, F_EAT_FILE>(module, f_wasi).await?;
        };
        async {
            private_io::run(
                &mut f_io,
                PrivateFs::new(f_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };

        async {
            let mut data = FileData::new(THINK_OBJECT);
            think_g.send::<Loan<G_THINK_FILE>>(think_file_g).await?;
            let status = {
                let [mut io, mut operation] = think_g.par()?;
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
                            think_g_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded think_g_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments = think_g_input
                                .take()
                                .and_then(PhilosopherArguments::from_value);
                            match arguments {
                                Err(_) => {
                                    // EINVAL preserves the exact think_g_input bytes for
                                    // inspection and a corrected retry.
                                    operation
                                        .send::<protocol::FdCloseRejectedRetMsg>(
                                            &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                        )
                                        .await?;
                                }
                                Ok(arguments) => {
                                    let agent = identity::AGENTS[usize::from(THINK_G % 7)];
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
            g.send::<Loan<G_EAT_FILE>>(eat_file_g).await?;
            let status = {
                let [mut io, mut operation] = g.par()?;
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
                            g_input.record(if close.0.errno() == 0 {
                                PhilosopherArguments::from_bytes(&data.bytes)
                                    .map(PhilosopherArguments::into_value)
                            } else {
                                Err(ShellError::message("invalid file descriptor"))
                            });
                            // par borrows a selected branch. The resolver reads the
                            // actual decoded g_input; the preview publishes no effect.
                            drop(operation.offer().await?);
                            let arguments =
                                g_input.take().and_then(PhilosopherArguments::from_value);
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
                                        left.send::<Eat<FORK_G_RESOURCE>>(&admitted).await?;
                                    };
                                    async {
                                        right.send::<Eat<FORK_F_RESOURCE>>(&admitted).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            let ((left, mut left_resource), (right, mut right_resource)) = async {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    let arguments = left.recv::<Eat<FORK_G_RESOURCE>>().await?;
                                    let resource = left.recv::<Loan<FORK_G_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                                async {
                                    let arguments = right.recv::<Eat<FORK_F_RESOURCE>>().await?;
                                    let resource = right.recv::<Loan<FORK_F_RESOURCE>>().await?;
                                    Ok((arguments, resource))
                                };
                            }
                            .await?;
                            if left != admitted || right != admitted {
                                return Err(ShellError::message(
                                    "resource requests differ from the received file arguments",
                                ));
                            }
                            let agent = identity::AGENTS[usize::from(G % 7)];
                            let result = bus
                                .execute(agent, "eat", admitted.into_value())
                                .await
                                .map_err(ShellError::message)?;
                            data.replace(result)?;
                            let response = data.bytes.as_slice();
                            async {
                                async {
                                    left_resource
                                        .send::<Meal<FORK_G_RESOURCE>>(&response)
                                        .await?;
                                };
                                async {
                                    right_resource
                                        .send::<Meal<FORK_F_RESOURCE>>(&response)
                                        .await?;
                                };
                                Ok(())
                            }
                            .await?;
                            {
                                let [mut left, mut right] = operation.par()?;
                                async {
                                    async {
                                        left.send::<Loan<FORK_G_RESOURCE>>(left_resource).await?;
                                    };
                                    async {
                                        right.send::<Loan<FORK_F_RESOURCE>>(right_resource).await?;
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
            fork::run(&mut fork_g, resource_g).await?;
        };
        async {
            wasi::run::<G_WASI, G_THINK_FILE, G_EAT_FILE>(module, g_wasi).await?;
        };
        async {
            private_io::run(
                &mut g_io,
                PrivateFs::new(g_terminal, bus, &[THINK_OBJECT, EAT_OBJECT]),
            )
            .await?;
        };
        Ok(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":session.raw(),"checkpoint":observations.checkpoint()}));
    result
}
