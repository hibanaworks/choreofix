use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<FRONTEND>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (mut incoming, mut call) = async {
        async { Ok::<_, ShellError>((3u8, task.recv::<Contract>().await?.to_vec())) };
        async {
            match guest
                .next_file::<ClosedFile, FRONTEND>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    Err(ShellError::message("frontend exited before agreement"))
                }
                call => Ok(call),
            }
        };
    }
    .await?;
    'business: loop {
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message("frontend exited before its result"));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        file.arguments(b"service/agree_api", "api_name")
                            .and_then(|v| {
                                imp::arguments(
                                    &serde_json::to_vec(&v)
                                        .map_err(|e| ShellError::message(e.to_string()))?,
                                    "api_name",
                                    "feature_api_spec",
                                )
                            });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(_) => {
                            let actual: Value = serde_json::from_slice(&incoming.1)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            if incoming.0 == 3 {
                                task.send::<Agreement>(&incoming.1.as_slice()).await?;
                                (json!({"success":true,"api_contract":actual}), 0)
                            } else {
                                (actual, 0)
                            }
                        }
                    };

                    let bytes = serde_json::to_vec(&reply)
                        .map_err(|e| ShellError::message(e.to_string()))?;
                    for (index, chunk) in
                        bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                    {
                        private
                            .send::<ResultChunkMsg>(&ResultChunk::new(
                                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                chunk,
                            )?)
                            .await?;
                    }
                    guest =
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                        )))?;

                    if requested.is_ok() {
                        break;
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, FRONTEND>(&mut private, SERVICE_FD)
                .await?;
        }
        if incoming.0 == 19 {
            break 'business;
        }
        loop {
            match guest
                .next_file::<ClosedFile, FRONTEND>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message("frontend exited before implementation"));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/implement_feature", "feature_name")
                        .and_then(|v| {
                            imp::arguments(
                                &serde_json::to_vec(&v)
                                    .map_err(|e| ShellError::message(e.to_string()))?,
                                "feature_name",
                                "frontend_component",
                            )
                        });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let bytes = serde_json::to_vec(arguments)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            let mut repo = task.recv::<RepoLoan>().await?;
                            repo.send::<ImplementFrontend>(&bytes.as_slice()).await?;
                            let bytes = repo.recv::<FrontendImplemented>().await?.to_vec();
                            task.send::<FrontendReady>(&bytes.as_slice()).await?;
                            task.send::<RepoLoan>(repo).await?;
                            (
                                serde_json::from_slice(&bytes)
                                    .map_err(|e| ShellError::message(e.to_string()))?,
                                0,
                            )
                        }
                    };

                    let bytes = serde_json::to_vec(&reply)
                        .map_err(|e| ShellError::message(e.to_string()))?;
                    for (index, chunk) in
                        bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                    {
                        private
                            .send::<ResultChunkMsg>(&ResultChunk::new(
                                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                chunk,
                            )?)
                            .await?;
                    }
                    guest =
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                        )))?;

                    if requested.is_ok() {
                        break;
                    }
                }
            }
        }
        (incoming, call) = async {
            async {
                let branch = task.offer().await?;
                let label = branch.label();
                let bytes = match label {
                    3 => branch.recv::<Contract>().await?.to_vec(),
                    19 => branch.recv::<FrontendResult>().await?.to_vec(),
                    _ => return Err(ShellError::message("unexpected frontend result")),
                };
                Ok::<_, ShellError>((label, bytes))
            };
            async {
                match guest
                    .next_file::<ClosedFile, FRONTEND>(&mut private, SERVICE_FD)
                    .await?
                {
                    FileBoundary::Exit(_) => Err(ShellError::message(
                        "frontend exited without its final report",
                    )),
                    call => Ok(call),
                }
            };
        }
        .await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, FRONTEND>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the frontend Endpoint has completed"});
                let errno = 8;

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
            }
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
