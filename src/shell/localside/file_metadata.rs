//! File bytes and metadata only. Business requests are sent by ChoreoFS itself.
use super::super::global::{FileExit, Loan};
use crate::shell::{ShellError, ShellResult, imp::file_data::FileData};
use hibana::Endpoint;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsRequest},
    protocol,
};

#[hibana_runtime::scope]
pub async fn run<'run, const FILE: u8, const DATA: u8, const N: usize>(
    io: &mut Endpoint<'run, DATA>,
    file: Endpoint<'run, FILE>,
    objects: [ChoreoFsObject<'static>; N],
) -> ShellResult<()> {
    io.send::<Loan<FILE>>(file).await?;
    let mut data = objects.map(FileData::new);
    loop {
        let label = io.offer().await?.label();
        match label {
            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                let request = io.recv::<protocol::FdPrestatGetReqMsg>().await?;
                let reply = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    request.0.fd(),
                    0,
                    8,
                ));
                io.send::<protocol::FdPrestatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let request = io.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                let reply = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    request.0.fd(),
                    b"",
                    8,
                )?);
                io.send::<protocol::FdPrestatDirNameRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_PATH_OPEN => {
                let request = io.recv::<protocol::PathOpenReqMsg>().await?;
                let open = protocol::PathOpenReq::request(&request);
                let mut path = open.0.path();
                while let Some(rest) = path.strip_prefix(b"/").or_else(|| path.strip_prefix(b"./"))
                {
                    path = rest;
                }
                let reply =
                    if let Some(index) = objects.iter().position(|object| object.path() == path) {
                        data[index].open(request)
                    } else {
                        protocol::PathOpenedRet(protocol::PathOpened::new(0, 44))
                    };
                io.send::<protocol::PathOpenRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_WRITE_OBJECT => {
                let request = io.recv::<protocol::FdWriteObjectReqMsg>().await?;
                let fd = protocol::FdWriteObjectReq::request(&request).0.fd();
                let reply =
                    if let Some(index) = objects.iter().position(|object| object.fd().fd() == fd) {
                        data[index].write(request)?
                    } else {
                        protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(fd, 0, 8))
                    };
                io.send::<protocol::FdWriteObjectRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                let request = io.recv::<protocol::FdFdstatGetReqMsg>().await?;
                let reply = ChoreoFsObjectSet::new(objects)
                    .choreofs()
                    .fd_fdstat_get(request);
                io.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                let request = io.recv::<protocol::FdFilestatGetReqMsg>().await?;
                let material = core::array::from_fn(|i| {
                    ChoreoFsObject::read_write(
                        objects[i].path(),
                        objects[i].object(),
                        objects[i].fd(),
                        &data[i].bytes,
                        protocol::FdBinding {
                            read: Some(protocol::FdReadRow::Base),
                            write: Some(protocol::FdWriteRow::Object),
                            readdir: None,
                        },
                    )
                });
                let reply = ChoreoFsObjectSet::<N>::new(material)
                    .choreofs()
                    .fd_filestat_get(request);
                io.send::<protocol::FdFilestatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                let request = io.recv::<protocol::PathFilestatGetReqMsg>().await?;
                let material = core::array::from_fn(|i| {
                    ChoreoFsObject::read_write(
                        objects[i].path(),
                        objects[i].object(),
                        objects[i].fd(),
                        &data[i].bytes,
                        protocol::FdBinding {
                            read: Some(protocol::FdReadRow::Base),
                            write: Some(protocol::FdWriteRow::Object),
                            readdir: None,
                        },
                    )
                });
                let reply = ChoreoFsObjectSet::<N>::new(material)
                    .choreofs()
                    .path_filestat_get(request);
                io.send::<protocol::PathFilestatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_CLOSE => {
                let request = io.recv::<protocol::FdCloseReqMsg>().await?;
                let reply = ChoreoFsObjectSet::new(objects).choreofs().fd_close(request);
                io.send::<protocol::FdCloseRetMsg>(&reply).await?;
            }
            241 => {
                let status = io.recv::<FileExit>().await?;
                if status != 0 {
                    return Err(ShellError::message(format!("shell exited with {status}")));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected file metadata request {label}"
                )));
            }
        }
    }
}
