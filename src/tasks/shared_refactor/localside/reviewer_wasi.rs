use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<REVIEWER>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (candidate, mut call) = async {
        async { imp::decode(task.recv::<Candidate>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "reviewer exited before candidate delivery",
                )),
                call => Ok::<_, ShellError>(call),
            }
        };
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("reviewer exited before both reviews"));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/code_review", "pr_id")
                    .and_then(|v| {
                        if matches!(v["pr_id"].as_str(), Some("module_a" | "module_b")) {
                            Ok(v)
                        } else {
                            Err(ShellError::message("unknown review object"))
                        }
                    });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        if arguments["pr_id"] == "module_a" {
                            task.send::<ReviewA>(&bytes.as_slice()).await?;
                        } else {
                            task.send::<ReviewB>(&bytes.as_slice()).await?;
                        }
                        let result = imp::decode(task.recv::<Approved>().await?)?;
                        imp::same(&result["candidate"], &candidate)?;
                        imp::same(&result["arguments"], arguments)?;
                        (result["result"].clone(), 0)
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
            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
            .await?;
    }

    let mut call = guest
        .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
        .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("reviewer exited before both reviews"));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/code_review", "pr_id")
                    .and_then(|v| {
                        if matches!(v["pr_id"].as_str(), Some("module_a" | "module_b")) {
                            Ok(v)
                        } else {
                            Err(ShellError::message("unknown review object"))
                        }
                    });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        if arguments["pr_id"] == "module_a" {
                            task.send::<ReviewA>(&bytes.as_slice()).await?;
                        } else {
                            task.send::<ReviewB>(&bytes.as_slice()).await?;
                        }
                        let result = imp::decode(task.recv::<Approved>().await?)?;
                        imp::same(&result["candidate"], &candidate)?;
                        imp::same(&result["arguments"], arguments)?;
                        (result["result"].clone(), 0)
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
            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
            .await?;
    }

    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
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
