//! The operation file carries the real request; there is no host-selected phase.
use super::super::{global::*, imp::Arguments};
use crate::shell::{ShellError, ShellResult, imp::file_data::FileData};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
#[hibana_runtime::scope]
pub async fn run<'run>(
    io: &mut Endpoint<'run, DEPLOYER>,
    file: Endpoint<'run, DEPLOY_FILE>,
) -> ShellResult<()> {
    let mut data = FileData::new(DEPLOY_OBJECT);
    let mut operation = io.recv::<Loan<DEPLOY_CLOSE>>().await?;
    io.send::<Loan<DEPLOY_FILE>>(file).await?;
    loop {
        let incoming = hibana_runtime::race(
            async {
                Ok::<_, ShellError>(core::ops::ControlFlow::Continue(io.offer().await?.label()))
            },
            async {
                let request = operation.recv::<protocol::FdCloseObjectReqMsg>().await?;
                Ok::<_, ShellError>(core::ops::ControlFlow::Break(
                    data.objects().choreofs().fd_close(request),
                ))
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
                        "guest exited before the actual operation: {status}"
                    )));
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected file request {label}"
                    )));
                }
            },
            core::ops::ControlFlow::Break(close) => {
                let arguments = match if close.0.errno() == 0 {
                    Arguments::<3>::from_bytes(&data.bytes)
                } else {
                    Err(ShellError::message("invalid file descriptor"))
                } {
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
                operation.send::<DeployIntent>(&arguments).await?;
                let mut resource = operation.recv::<Loan<DEPLOY_ENV>>().await?;
                resource.send::<DeployRequest>(&arguments).await?;
                let response = resource.recv::<DeployResult>().await?.to_vec();
                operation.send::<Loan<DEPLOY_ENV>>(resource).await?;
                operation
                    .send::<protocol::FdCloseObjectRetMsg>(&close)
                    .await?;
                let actual: serde_json::Value = serde_json::from_slice(&response)
                    .map_err(|e| ShellError::message(e.to_string()))?;
                let notice = serde_json::to_vec(
                    &serde_json::json!({"kind":"deploy_result","result":actual}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                operation.send::<Notice>(&notice.as_slice()).await?;
                let notification = serde_json::to_vec(
                    &serde_json::json!({"kind":"deployment_complete","result":actual}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                async {
                    async {
                        operation.send::<Notice>(&notification.as_slice()).await?;
                    };
                    async {
                        io.send::<Notice>(&notification.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                break;
            }
        }
    }
    // The operation is complete. Metadata remains available while the other
    // participants work; no operation-ready flag or holder table is needed.
    let status =
        loop {
            let label = io.offer().await?.label();
            match label {
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

                241 => break io.recv::<FileExit>().await?,
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected file request {label}"
                    )));
                }
            }
        };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    Ok(())
}
