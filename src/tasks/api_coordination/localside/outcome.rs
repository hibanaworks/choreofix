//! File metadata only; incoming values are received by ChoreoFS in the guest.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use hibana_wasip1_runtime::choreofs::ChoreoFsObjectSet;
use hibana_wasip1_runtime::protocol;
#[hibana_runtime::scope]
pub(crate) async fn backend<'run>(
    endpoint: &mut Endpoint<'run, BACKEND_OUTCOME>,
    view: Endpoint<'run, BACKEND_OUTCOME_FILE>,
) -> ShellResult<()> {
    let objects = ChoreoFsObjectSet::new([BACKEND_OUTCOME_OBJECT]);
    let fs = objects.choreofs();
    endpoint.send::<Loan<BACKEND_OUTCOME_FILE>>(view).await?;
    let status =
        loop {
            let label = endpoint.offer().await?.label();
            match label {
                241 => break endpoint.recv::<FileExit>().await?,

                protocol::LABEL_WASI_PATH_OPEN => {
                    let request = endpoint.recv::<protocol::PathOpenReqMsg>().await?;
                    let response = fs.path_open(request).opened_ret();
                    endpoint.send::<protocol::PathOpenRetMsg>(&response).await?;
                }
                protocol::LABEL_WASI_FD_FDSTAT_GET => {
                    let request = endpoint.recv::<protocol::FdFdstatGetReqMsg>().await?;
                    let response = fs.fd_fdstat_get(request);
                    endpoint
                        .send::<protocol::FdFdstatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_FILESTAT_GET => {
                    let request = endpoint.recv::<protocol::FdFilestatGetReqMsg>().await?;
                    let response = fs.fd_filestat_get(request);
                    endpoint
                        .send::<protocol::FdFilestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                    let request = endpoint.recv::<protocol::PathFilestatGetReqMsg>().await?;
                    let response = fs.path_filestat_get(request);
                    endpoint
                        .send::<protocol::PathFilestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_CLOSE => {
                    let request = endpoint.recv::<protocol::FdCloseReqMsg>().await?;
                    let response = fs.fd_close(request);
                    endpoint.send::<protocol::FdCloseRetMsg>(&response).await?;
                }
                protocol::LABEL_WASI_FD_PRESTAT_GET => {
                    let request = endpoint.recv::<protocol::FdPrestatGetReqMsg>().await?;
                    let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                        request.0.fd(),
                        0,
                        8,
                    ));
                    endpoint
                        .send::<protocol::FdPrestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                    let request = endpoint.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                    let response = protocol::FdPrestatDirNameRet(
                        protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                    );
                    endpoint
                        .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_READDIR => {
                    let request = endpoint.recv::<protocol::FdReaddirReqMsg>().await?;
                    let response = fs.fd_readdir(request).read_dir()?;
                    endpoint
                        .send::<protocol::FdReaddirRetMsg>(&response)
                        .await?;
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected outcome endpoint {label}"
                    )));
                }
            }
        };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    Ok(())
}
#[hibana_runtime::scope]
pub(crate) async fn frontend<'run>(
    endpoint: &mut Endpoint<'run, FRONTEND_OUTCOME>,
    view: Endpoint<'run, FRONTEND_OUTCOME_FILE>,
) -> ShellResult<()> {
    let objects = ChoreoFsObjectSet::new([FRONTEND_OUTCOME_OBJECT]);
    let fs = objects.choreofs();
    endpoint.send::<Loan<FRONTEND_OUTCOME_FILE>>(view).await?;
    let status =
        loop {
            let label = endpoint.offer().await?.label();
            match label {
                241 => break endpoint.recv::<FileExit>().await?,

                protocol::LABEL_WASI_PATH_OPEN => {
                    let request = endpoint.recv::<protocol::PathOpenReqMsg>().await?;
                    let response = fs.path_open(request).opened_ret();
                    endpoint.send::<protocol::PathOpenRetMsg>(&response).await?;
                }
                protocol::LABEL_WASI_FD_FDSTAT_GET => {
                    let request = endpoint.recv::<protocol::FdFdstatGetReqMsg>().await?;
                    let response = fs.fd_fdstat_get(request);
                    endpoint
                        .send::<protocol::FdFdstatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_FILESTAT_GET => {
                    let request = endpoint.recv::<protocol::FdFilestatGetReqMsg>().await?;
                    let response = fs.fd_filestat_get(request);
                    endpoint
                        .send::<protocol::FdFilestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                    let request = endpoint.recv::<protocol::PathFilestatGetReqMsg>().await?;
                    let response = fs.path_filestat_get(request);
                    endpoint
                        .send::<protocol::PathFilestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_CLOSE => {
                    let request = endpoint.recv::<protocol::FdCloseReqMsg>().await?;
                    let response = fs.fd_close(request);
                    endpoint.send::<protocol::FdCloseRetMsg>(&response).await?;
                }
                protocol::LABEL_WASI_FD_PRESTAT_GET => {
                    let request = endpoint.recv::<protocol::FdPrestatGetReqMsg>().await?;
                    let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                        request.0.fd(),
                        0,
                        8,
                    ));
                    endpoint
                        .send::<protocol::FdPrestatGetRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                    let request = endpoint.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                    let response = protocol::FdPrestatDirNameRet(
                        protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                    );
                    endpoint
                        .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                        .await?;
                }
                protocol::LABEL_WASI_FD_READDIR => {
                    let request = endpoint.recv::<protocol::FdReaddirReqMsg>().await?;
                    let response = fs.fd_readdir(request).read_dir()?;
                    endpoint
                        .send::<protocol::FdReaddirRetMsg>(&response)
                        .await?;
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected outcome endpoint {label}"
                    )));
                }
            }
        };
    if status != 0 {
        return Err(ShellError::message(format!("guest exited with {status}")));
    }
    Ok(())
}
