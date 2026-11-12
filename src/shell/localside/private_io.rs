use crate::shell::{
    ShellError, ShellResult,
    global::Exit,
    imp::private_fs::{PrivateFs, WRITE},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsRequest, ObjectId},
    protocol,
};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    endpoint: &mut hibana::Endpoint<ROLE>,
    mut io: PrivateFs,
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            240 => {
                let status = branch.recv::<Exit>().await?;
                io.terminal.finish(io.bus, status, io.output).await?;
                return Ok(());
            }
            protocol::LABEL_WASI_FD_READDIR => {
                let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                let response = io.objects().choreofs().fd_readdir(request).read_dir()?;
                endpoint
                    .send::<protocol::FdReaddirRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_READ => {
                let received = branch.recv::<protocol::FdReadReqMsg>().await?;
                let response = {
                    let read = protocol::FdReadReq::request(&received).0;
                    if read.fd() == 0 && io.offsets[0] == io.input.len() {
                        let command = io
                            .terminal
                            .read_command(io.bus, std::mem::take(&mut io.output))
                            .await?;
                        if command.len() > 4096 || command.as_bytes().contains(&0) {
                            return Err(ShellError::message(
                                "shell command exceeds guest input bound",
                            ));
                        }
                        io.input = (command.len() as u32).to_le_bytes().to_vec();
                        io.input.extend_from_slice(command.as_bytes());
                        io.offsets[0] = 0;
                    }
                    let index = read.fd() as usize;
                    let offset = io.offsets.get(index).copied().unwrap_or(0);
                    let (response, next) = io
                        .objects()
                        .choreofs()
                        .fd_read(received)
                        .read_from(offset)?;
                    if let Some(offset) = io.offsets.get_mut(index) {
                        *offset = next;
                    }
                    response
                };
                endpoint.send::<protocol::FdReadRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_FD_WRITE => {
                let request = branch.recv::<protocol::FdWriteReqMsg>().await?;
                let operation = io.objects().choreofs().fd_write(request);
                let response = io.write(operation)?;
                endpoint.send::<protocol::FdWriteRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_PATH_OPEN => {
                let received = branch.recv::<protocol::PathOpenReqMsg>().await?;
                let response = 'path_open: {
                    let request = protocol::PathOpenReq::request(&received).0;
                    if request.preopen_fd() != 3 {
                        return Err(ShellError::message("non-root path_open"));
                    }
                    let path = request.path().strip_prefix(b"/").unwrap_or(request.path());
                    let created = if !io.paths.iter().any(|existing| existing == path)
                        && request.open_flags() & 1 != 0
                        && request.rights_base() & WRITE != 0
                        && let Some(name) = path.strip_prefix(b"work/")
                    {
                        if name.is_empty()
                            || name.len() > 64
                            || name.contains(&b'/')
                            || name.contains(&0)
                        {
                            break 'path_open protocol::PathOpenedRet(protocol::PathOpened::new(
                                0, 28,
                            ));
                        }
                        let Some(index) = io.paths.iter().position(Vec::is_empty) else {
                            break 'path_open protocol::PathOpenedRet(protocol::PathOpened::new(
                                0, 22,
                            ));
                        };
                        io.paths[index].extend_from_slice(path);
                        Some(index)
                    } else {
                        None
                    };
                    let open = io.objects().choreofs().path_open(received);
                    if let Some(fd) = open.fd() {
                        let fd = fd as usize;
                        if request.open_flags() & 8 != 0 {
                            match open.object() {
                                Some(ObjectId(6..=13)) => io.work[fd - 6].clear(),
                                _ => {
                                    return Err(ShellError::message(
                                        "truncate is only supported on work files",
                                    ));
                                }
                            }
                        }
                        io.offsets[fd] = 0;
                        io.fd_flags[fd] = request.fd_flags();
                    }
                    if let Some(index) = created {
                        if !open.is_open() {
                            io.paths[index].clear();
                        }
                        io.refresh_listing();
                    }
                    open.opened_ret()
                };
                endpoint.send::<protocol::PathOpenRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                let response = io.objects().choreofs().fd_fdstat_get(request);
                endpoint
                    .send::<protocol::FdFdstatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                let response = io.objects().choreofs().fd_filestat_get(request);
                endpoint
                    .send::<protocol::FdFilestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                let fd = request.0.fd();
                let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    fd,
                    1,
                    if fd == 3 { 0 } else { 8 },
                ));
                endpoint
                    .send::<protocol::FdPrestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                let fd = request.0.fd();
                let response = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    fd,
                    b"/",
                    if fd == 3 { 0 } else { 8 },
                )?);
                endpoint
                    .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_CLOSE => {
                let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                let response = io.objects().choreofs().fd_close(request);
                endpoint.send::<protocol::FdCloseRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                if protocol::PathFilestatGetReq::request(&request)
                    .0
                    .preopen_fd()
                    != 3
                {
                    return Err(ShellError::message("non-root path_filestat_get"));
                }
                let response = io.objects().choreofs().path_filestat_get(request);
                endpoint
                    .send::<protocol::PathFilestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_ENGINE_MEMORY_GROW => {
                let request = branch.recv::<protocol::MemoryGrowReqMsg>().await?;
                let decision = if request.0.would_fit() {
                    protocol::MemoryGrowDecision::grant()
                } else {
                    protocol::MemoryGrowDecision::reject()
                };
                let response = protocol::MemoryGrowRet(decision);
                endpoint
                    .send::<protocol::MemoryGrowRetMsg>(&response)
                    .await?;
            }
            label => return Err(ShellError::message(format!("unexpected WASI row {label}"))),
        }
    }
}
