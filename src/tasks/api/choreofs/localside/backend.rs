use super::super::global::*;
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
    io: &mut Endpoint<'run, BACKEND>,
    (file, mut control): (Endpoint<'run, BACKEND_FILE>, Endpoint<'run, BACKEND_CLOSE>),
    (domain, input): (&impl Domain, &Validation),
) -> ShellResult<()> {
    let mut data = FileData::new(BACKEND_OBJECT);
    io.send::<Loan<BACKEND_FILE>>(file).await?;
    loop {
        let incoming = hibana_runtime::race(
            async {
                let branch = io.offer().await?;
                Ok(core::ops::ControlFlow::Continue(branch.label()))
            },
            async {
                control
                    .recv::<MigrationDone>()
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
        let mut authority = Some(control);
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
                    let operation = authority
                        .as_mut()
                        .ok_or_else(|| ShellError::message("missing owned close Endpoint"))?;
                    input.record(if close.0.errno() == 0 {
                        serde_json::from_slice(&data.bytes)
                            .map_err(|error| ShellError::message(error.to_string()))
                            .and_then(|value| {
                                if value == serde_json::json!({"component":"new_api_endpoints"}) {
                                    Ok(value)
                                } else {
                                    Err(ShellError::message("arguments name a different component"))
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
                    let result = domain
                        .execute("BACKEND_DEV", "implement_backend", admitted)
                        .await
                        .map_err(ShellError::message)?;
                    operation.send::<BackendReady>(&()).await?;
                    data.replace(result)?;
                    operation
                        .send::<protocol::FdCloseObjectRetMsg>(&close)
                        .await?;
                    io.send::<CloseAuthority<BACKEND_CLOSE>>(
                        authority
                            .take()
                            .ok_or_else(|| ShellError::message("missing owned close Endpoint"))?,
                    )
                    .await?;
                }
            }
        }
    };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    let _file = io.recv::<Loan<BACKEND_FILE>>().await?;
    Ok(())
}
