use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<const WHO: u8>(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<WHO>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (_design, mut call) = async {
        async { imp::decode(task.recv::<DesignResult>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    Err(ShellError::message("developer exited before its design"))
                }
                call => Ok::<_, ShellError>(call),
            }
        };
    }
    .await?;
    let mut repo = task.recv::<RepoLoan>().await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "developer exited before both refactors",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/refactor_code", "target")
                    .and_then(|v| {
                        let own = if WHO == A { "module_a" } else { "module_b" };
                        if v["target"] == own || v["target"] == "SHARED_LIBRARY" {
                            Ok(v)
                        } else {
                            Err(ShellError::message(
                                "module differs from the developer assignment",
                            ))
                        }
                    });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        if arguments["target"] == "SHARED_LIBRARY" {
                            task.send::<SharedIntent>(&bytes.as_slice()).await?;
                        } else {
                            task.send::<OwnIntent>(&bytes.as_slice()).await?;
                        }
                        repo.send::<Refactor>(&bytes.as_slice()).await?;
                        (imp::decode(repo.recv::<Refactored>().await?)?, 0)
                    }
                };
                let bytes = imp::encode(&reply)?;
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
                if requested.is_ok() {
                    break;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
            .await?;
    }

    let mut call = guest
        .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
        .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "developer exited before both refactors",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/refactor_code", "target")
                    .and_then(|v| {
                        let own = if WHO == A { "module_a" } else { "module_b" };
                        if v["target"] == own || v["target"] == "SHARED_LIBRARY" {
                            Ok(v)
                        } else {
                            Err(ShellError::message(
                                "module differs from the developer assignment",
                            ))
                        }
                    });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        if arguments["target"] == "SHARED_LIBRARY" {
                            task.send::<SharedIntent>(&bytes.as_slice()).await?;
                        } else {
                            task.send::<OwnIntent>(&bytes.as_slice()).await?;
                        }
                        repo.send::<Refactor>(&bytes.as_slice()).await?;
                        (imp::decode(repo.recv::<Refactored>().await?)?, 0)
                    }
                };
                let bytes = imp::encode(&reply)?;
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
                if requested.is_ok() {
                    break;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
            .await?;
    }

    task.send::<RepoLoan>(repo).await?;
    let (ready, mut call) = async {
        async {
            let tests = if WHO == A {
                imp::decode(task.recv::<TestSummary>().await?)?
            } else {
                imp::decode(task.recv::<TestSummaryB>().await?)?
            };
            let review = imp::decode(task.recv::<Approved>().await?)?;
            let ready = imp::decode(task.recv::<MergeReady>().await?)?;
            imp::same(&ready["tests"], &tests)?;
            imp::same(&ready["review"], &review)?;
            Ok::<_, ShellError>(ready)
        };
        async {
            match guest
                .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message("developer exited before merge")),
                call => Ok::<_, ShellError>(call),
            }
        };
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("developer exited before merge"));
            }
            FileBoundary::Closed { pending, file } => {
                let own = if WHO == A { "module_a" } else { "module_b" };
                let requested = file
                    .arguments(b"service/merge_code", "branch")
                    .and_then(|v| imp::arguments(&imp::encode(&v)?, "branch", own));
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let mut repo = task.recv::<RepoLoan>().await?;
                        let bytes = imp::encode(arguments)?;
                        task.send::<MergeIntent>(&bytes.as_slice()).await?;
                        let bytes = imp::encode(&json!({"arguments":arguments,"checked":ready}))?;
                        repo.send::<MergeRequest>(&bytes.as_slice()).await?;
                        let result = imp::decode(repo.recv::<Merged>().await?)?;
                        task.send::<RepoLoan>(repo).await?;
                        (result, 0)
                    }
                };
                let bytes = imp::encode(&reply)?;
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
                if requested.is_ok() {
                    break;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
            .await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, WHO>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(code) => {
                let code = code.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the task Endpoint has completed"});
                let bytes = imp::encode(&reply)?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
