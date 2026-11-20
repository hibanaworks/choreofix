//! Operation file data owner shared by the three participants.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::file_data::FileData};
use hibana::Endpoint;
use hibana_wasip1_runtime::{choreofs::ChoreoFsObject, protocol};

pub(crate) async fn run<'run, const OWNER: u8, const FILE: u8>(
    io: &mut Endpoint<'run, OWNER>,
    file: Endpoint<'run, FILE>,
    object: ChoreoFsObject<'static>,
) -> ShellResult<()> {
    let mut data = FileData::new(object);
    io.send::<Loan<FILE>>(file).await?;
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
                let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    request.0.fd(),
                    0,
                    8,
                ));
                io.send::<protocol::FdPrestatGetRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                let response = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    request.0.fd(),
                    b"",
                    8,
                )?);
                io.send::<protocol::FdPrestatDirNameRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_READDIR => {
                let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                let response = data.objects().choreofs().fd_readdir(request).read_dir()?;
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

            42 => {
                let bytes = branch.recv::<AppliedResult>().await?;
                data.replace(
                    serde_json::from_slice(bytes)
                        .map_err(|e| ShellError::message(e.to_string()))?,
                )?;
                io.send::<FileApplied>(&()).await?;
            }
            241 => {
                let status = branch.recv::<FileExit>().await?;
                if status != 0 {
                    return Err(ShellError::message(format!("guest exited with {status}")));
                }
                let _file = io.recv::<Loan<FILE>>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected operation-file message {label}"
                )));
            }
        }
    }
}
