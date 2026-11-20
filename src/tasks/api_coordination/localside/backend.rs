use super::super::{
    global::*,
    imp::{self, Arguments},
};
use crate::shell::{ShellError, ShellResult, imp::file_data::FileData};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;

#[hibana_runtime::scope]
pub async fn run<'run>(
    io: &mut Endpoint<'run, BACKEND_OWNER>,
    file: Endpoint<'run, BACKEND_FILE>,
) -> ShellResult<()> {
    let mut data = FileData::new(BACKEND_OBJECT);
    let mut operation = io.recv::<Loan<BACKEND_CLOSE>>().await?;
    io.send::<Loan<BACKEND_FILE>>(file).await?;

    let status = loop {
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
                241 => break io.recv::<FileExit>().await?,
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
                protocol::LABEL_WASI_FD_READ => {
                    let received = io.recv::<protocol::FdReadReqMsg>().await?;
                    let response = data.read(received)?;
                    io.send::<protocol::FdReadRetMsg>(&response).await?;
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
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected file metadata request {label}"
                    )));
                }
            },
            core::ops::ControlFlow::Break(close) => {
                let admitted = match if close.0.errno() == 0 {
                    Arguments::<{ imp::BACKEND }>::from_bytes(&data.bytes)
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
                let arguments = admitted;
                operation.send::<BackendIntent>(&arguments.intent()).await?;
                let mut repository = operation.recv::<Loan<REPO>>().await?;
                repository.send::<ImplementBackend>(&arguments).await?;
                let response = repository.recv::<BackendImplemented>().await?.to_vec();
                operation.send::<Loan<REPO>>(repository).await?;
                data.replace(
                    serde_json::from_slice(&response)
                        .map_err(|error| ShellError::message(error.to_string()))?,
                )?;
                operation
                    .send::<protocol::FdCloseObjectRetMsg>(&close)
                    .await?;
            }
        }
    };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    Ok(())
}
