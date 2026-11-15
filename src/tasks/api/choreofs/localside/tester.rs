use super::super::{
    global::*,
    imp::{ApiArguments, TEST},
};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        file_data::FileData,
        service::{Domain, Validation},
    },
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;

#[hibana_runtime::scope]
pub(crate) async fn run<'run>(
    io: &mut Endpoint<'run, TESTER>,
    (file, mut control, admin_read, backend_read): (
        Endpoint<'run, TESTER_FILE>,
        Endpoint<'run, TESTER_CLOSE>,
        Endpoint<'run, ADMIN_OUTCOME_READ>,
        Endpoint<'run, BACKEND_OUTCOME_READ>,
    ),
    (domain, input, result_input): (
        &impl Domain,
        &Validation,
        &super::super::super::imp::TestInput,
    ),
) -> ShellResult<()> {
    let mut data = FileData::new(TESTER_OBJECT);
    io.send::<Loan<TESTER_FILE>>(file).await?;
    loop {
        let incoming = hibana_runtime::race(
            async {
                let branch = io.offer().await?;
                Ok(core::ops::ControlFlow::Continue(branch.label()))
            },
            async {
                control
                    .recv::<BackendReady>()
                    .await
                    .map(core::ops::ControlFlow::Break)
            },
        )
        .await?;
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
                    let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                        request.0.fd(),
                        0,
                        8,
                    ));
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
                    let response = data.objects().choreofs().fd_readdir(request).read_dir()?;
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
                    return Err(ShellError::message(format!(
                        "guest exited before dependency: {status}"
                    )));
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected operation-file request {label}"
                    )));
                }
            },
            core::ops::ControlFlow::Break(()) => break,
        }
    }
    let status = {
        let mut authority = Some((control, admin_read, backend_read));
        loop {
            let incoming = if let Some((operation, _, _)) = authority.as_mut() {
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
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
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
                        let response = data.objects().choreofs().fd_readdir(request).read_dir()?;
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
                            "unexpected operation-file request {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(request) => {
                    let close = data.objects().choreofs().fd_close(request);
                    let operation = &mut authority
                        .as_mut()
                        .ok_or_else(|| ShellError::message("missing owned close Endpoint"))?
                        .0;
                    input.record(if close.0.errno() == 0 {
                        serde_json::from_slice(&data.bytes)
                            .map_err(|error| ShellError::message(error.to_string()))
                            .and_then(|value| {
                                if value
                                    == serde_json::json!({"suite":"backend_schema_integration"})
                                {
                                    Ok(value)
                                } else {
                                    Err(ShellError::message(
                                        "arguments name a different test suite",
                                    ))
                                }
                            })
                    } else {
                        Err(ShellError::message("invalid file descriptor"))
                    });
                    drop(operation.offer().await?);
                    let admitted = match input.take() {
                        Ok(value) => value,
                        Err(_) => {
                            operation
                                .send::<protocol::FdCloseRejectedRetMsg>(
                                    &protocol::FdCloseRejectedRet::new(close.0.fd(), 28)?,
                                )
                                .await?;
                            continue;
                        }
                    };
                    let (mut operation, admin_read, backend_read) = authority
                        .take()
                        .ok_or_else(|| ShellError::message("missing owned result Endpoints"))?;
                    let encoded = serde_json::to_vec(&admitted)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    let arguments = ApiArguments::<TEST>::from_bytes(&encoded)?;
                    operation.send::<TestIntent>(&arguments).await?;
                    let request = operation.recv::<TestRequest>().await?.into_value();
                    let mut test_environment = operation.recv::<Loan<TEST_RESOURCE>>().await?;
                    if request != admitted {
                        return Err(ShellError::message(
                            "resource request differs from admitted test",
                        ));
                    }
                    let result = domain
                        .execute("TESTER", "run_integration_tests", request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = serde_json::to_vec(&result)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    test_environment
                        .send::<TestCompleted>(&bytes.as_slice())
                        .await?;
                    operation
                        .send::<Loan<TEST_RESOURCE>>(test_environment)
                        .await?;
                    result_input.record(result.clone());
                    let decision = operation.offer().await?;
                    let label = decision.label();
                    drop(decision);
                    {
                        let [mut admin, mut backend] = operation.par()?;
                        match label {
                            50 | 51 => {
                                async {
                                    async {
                                        admin.send::<PassedAdmin>(&bytes.as_slice()).await?;
                                        admin.send::<Loan<ADMIN_OUTCOME_READ>>(admin_read).await?;
                                    };
                                    async {
                                        backend.send::<PassedBackend>(&bytes.as_slice()).await?;
                                        backend
                                            .send::<Loan<BACKEND_OUTCOME_READ>>(backend_read)
                                            .await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            60 | 61 => {
                                async {
                                    async {
                                        admin.send::<FailedAdmin>(&bytes.as_slice()).await?;
                                        admin.send::<Loan<ADMIN_OUTCOME_READ>>(admin_read).await?;
                                    };
                                    async {
                                        backend.send::<FailedBackend>(&bytes.as_slice()).await?;
                                        backend
                                            .send::<Loan<BACKEND_OUTCOME_READ>>(backend_read)
                                            .await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                            }
                            _ => return Err(ShellError::message("unexpected test-result branch")),
                        }
                    }
                    data.replace(result)?;
                    operation
                        .send::<protocol::FdCloseObjectRetMsg>(&close)
                        .await?;
                    io.send::<CloseAuthority<TESTER_CLOSE>>(operation).await?;
                }
            }
        }
    };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    let _file = io.recv::<Loan<TESTER_FILE>>().await?;
    Ok(())
}
