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
    endpoint: &mut hibana::Endpoint<ARCHITECT>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    for _ in 0..2 {
        loop {
            match guest
                .next_file::<ClosedFile, ARCHITECT>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "architect exited before both proposals",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/design_proposal", "title")
                        .and_then(|v| {
                            if matches!(
                                v["title"].as_str(),
                                Some("module_a_refactor" | "module_b_refactor")
                            ) {
                                Ok(v)
                            } else {
                                Err(ShellError::message("unknown module proposal"))
                            }
                        });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let bytes = imp::encode(arguments)?;
                            if arguments["title"] == "module_a_refactor" {
                                task.send::<DesignA>(&bytes.as_slice()).await?;
                            } else {
                                task.send::<DesignB>(&bytes.as_slice()).await?;
                            }
                            (imp::decode(task.recv::<DesignResult>().await?)?, 0)
                        }
                    };
                    let bytes = imp::encode(&reply)?;
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
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, ARCHITECT>(&mut private, SERVICE_FD)
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
