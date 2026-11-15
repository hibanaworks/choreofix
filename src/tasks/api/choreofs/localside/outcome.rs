//! The result owner holds file bytes and its actual delegated read Endpoint.
use super::super::{
    global::*,
    imp::{OutcomeFile, outcome_objects},
};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;

#[hibana_runtime::scope]
pub(crate) async fn admin<'run>(
    endpoint: &mut Endpoint<'run, ADMIN_OUTCOME>,
    view: Endpoint<'run, ADMIN_OUTCOME_FILE>,
) -> ShellResult<()> {
    let mut file = OutcomeFile::new();
    endpoint.send::<Loan<ADMIN_OUTCOME_FILE>>(view).await?;
    let status = {
        let [mut publication, mut metadata] = endpoint.par()?;
        loop {
            let incoming = hibana_runtime::race(
                async {
                    metadata
                        .offer()
                        .await
                        .map(|branch| core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    publication
                        .offer()
                        .await
                        .map(|branch| core::ops::ControlFlow::Break(branch.label()))
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        return Err(ShellError::message(format!(
                            "guest exited before receiving test result: {status}"
                        )));
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(label) => {
                    let bytes = match label {
                        50 => {
                            let bytes = publication.recv::<PassedAdmin>().await?.to_vec();
                            let value: serde_json::Value = serde_json::from_slice(&bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["success"] != true {
                                return Err(ShellError::message(
                                    "pass label differs from actual test result",
                                ));
                            }
                            bytes
                        }
                        60 => {
                            let bytes = publication.recv::<FailedAdmin>().await?.to_vec();
                            let value: serde_json::Value = serde_json::from_slice(&bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["success"] != false {
                                return Err(ShellError::message(
                                    "failure label differs from actual test result",
                                ));
                            }
                            bytes
                        }
                        _ => return Err(ShellError::message("unexpected test-result publication")),
                    };
                    file.bytes = bytes;
                    break;
                }
            }
        }
        let mut read = loop {
            let incoming = hibana_runtime::race(
                async {
                    metadata
                        .offer()
                        .await
                        .map(|branch| core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    publication
                        .recv::<Loan<ADMIN_OUTCOME_READ>>()
                        .await
                        .map(core::ops::ControlFlow::Break)
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        return Err(ShellError::message(format!(
                            "guest exited before receiving test result: {status}"
                        )));
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(read) => break read,
            }
        };
        loop {
            let incoming = hibana_runtime::race(
                async {
                    let branch = metadata.offer().await?;
                    Ok(core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    read.recv::<protocol::FdReadReqMsg>()
                        .await
                        .map(core::ops::ControlFlow::Break)
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        break status;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(request) => {
                    let response = file.read(request)?;
                    read.send::<protocol::FdReadRetMsg>(&response).await?;
                }
            }
        }
    };
    if status != 0 {
        return Err(ShellError::message(format!(
            "result reader exited with {status}"
        )));
    }
    let _file = endpoint.recv::<Loan<ADMIN_OUTCOME_FILE>>().await?;
    Ok(())
}

#[hibana_runtime::scope]
pub(crate) async fn backend<'run>(
    endpoint: &mut Endpoint<'run, BACKEND_OUTCOME>,
    view: Endpoint<'run, BACKEND_OUTCOME_FILE>,
) -> ShellResult<()> {
    let mut file = OutcomeFile::new();
    endpoint.send::<Loan<BACKEND_OUTCOME_FILE>>(view).await?;
    let status = {
        let [mut publication, mut metadata] = endpoint.par()?;
        loop {
            let incoming = hibana_runtime::race(
                async {
                    let branch = metadata.offer().await?;
                    Ok(core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    publication
                        .offer()
                        .await
                        .map(|branch| core::ops::ControlFlow::Break(branch.label()))
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        return Err(ShellError::message(format!(
                            "guest exited before receiving test result: {status}"
                        )));
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(label) => {
                    let bytes = match label {
                        51 => {
                            let bytes = publication.recv::<PassedBackend>().await?.to_vec();
                            let value: serde_json::Value = serde_json::from_slice(&bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["success"] != true {
                                return Err(ShellError::message(
                                    "pass label differs from actual test result",
                                ));
                            }
                            bytes
                        }
                        61 => {
                            let bytes = publication.recv::<FailedBackend>().await?.to_vec();
                            let value: serde_json::Value = serde_json::from_slice(&bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["success"] != false {
                                return Err(ShellError::message(
                                    "failure label differs from actual test result",
                                ));
                            }
                            bytes
                        }
                        _ => return Err(ShellError::message("unexpected test-result publication")),
                    };
                    file.bytes = bytes;
                    break;
                }
            }
        }
        let mut read = loop {
            let incoming = hibana_runtime::race(
                async {
                    let branch = metadata.offer().await?;
                    Ok(core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    publication
                        .recv::<Loan<BACKEND_OUTCOME_READ>>()
                        .await
                        .map(core::ops::ControlFlow::Break)
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        return Err(ShellError::message(format!(
                            "guest exited before receiving test result: {status}"
                        )));
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(read) => break read,
            }
        };
        loop {
            let incoming = hibana_runtime::race(
                async {
                    let branch = metadata.offer().await?;
                    Ok(core::ops::ControlFlow::Continue(branch.label()))
                },
                async {
                    read.recv::<protocol::FdReadReqMsg>()
                        .await
                        .map(core::ops::ControlFlow::Break)
                },
            )
            .await?;
            match incoming {
                core::ops::ControlFlow::Continue(label) => match label {
                    protocol::LABEL_WASI_PATH_OPEN => {
                        let request = metadata.recv::<protocol::PathOpenReqMsg>().await?;
                        let response = file.open(request)?;
                        metadata.send::<protocol::PathOpenRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_FDSTAT_GET => {
                        let request = metadata.recv::<protocol::FdFdstatGetReqMsg>().await?;
                        let response = file.stat(request);
                        metadata
                            .send::<protocol::FdFdstatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::FdFilestatGetReqMsg>().await?;
                        let response = file.file_stat(request);
                        metadata
                            .send::<protocol::FdFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                        let request = metadata.recv::<protocol::PathFilestatGetReqMsg>().await?;
                        let response = file.path_stat(request);
                        metadata
                            .send::<protocol::PathFilestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_CLOSE => {
                        let request = metadata.recv::<protocol::FdCloseReqMsg>().await?;
                        let response = file.close(request);
                        metadata.send::<protocol::FdCloseRetMsg>(&response).await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_GET => {
                        let request = metadata.recv::<protocol::FdPrestatGetReqMsg>().await?;
                        let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ));
                        metadata
                            .send::<protocol::FdPrestatGetRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                        let request = metadata.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                        let response = protocol::FdPrestatDirNameRet(
                            protocol::FdPrestatDirNameDone::new(request.0.fd(), b"", 8)?,
                        );
                        metadata
                            .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                            .await?;
                    }
                    protocol::LABEL_WASI_FD_READDIR => {
                        let request = metadata.recv::<protocol::FdReaddirReqMsg>().await?;
                        let response = outcome_objects(&file.bytes)
                            .choreofs()
                            .fd_readdir(request)
                            .read_dir()?;
                        metadata
                            .send::<protocol::FdReaddirRetMsg>(&response)
                            .await?;
                    }
                    241 => {
                        let status = metadata.recv::<FileExit>().await?;
                        break status;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected outcome metadata {label}"
                        )));
                    }
                },
                core::ops::ControlFlow::Break(request) => {
                    let response = file.read(request)?;
                    read.send::<protocol::FdReadRetMsg>(&response).await?;
                }
            }
        }
    };
    if status != 0 {
        return Err(ShellError::message(format!(
            "result reader exited with {status}"
        )));
    }
    let _file = endpoint.recv::<Loan<BACKEND_OUTCOME_FILE>>().await?;
    Ok(())
}
