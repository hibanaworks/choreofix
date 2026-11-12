//! The pharmacist's actual WASI request, review result and direct notification.
use super::super::global::*;
use crate::imp::{json, patient};
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
    endpoint: &mut hibana::Endpoint<PHARMACIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, PHARMACIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "pharmacist exited before the actual review",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments = patient::file_arguments(&file, b"service/review_prescriptions")
                    .and_then(|arguments| {
                        if arguments
                            .get("medications")
                            .is_some_and(|value| !value.is_string())
                        {
                            return Err(ShellError::message("medications must be a string"));
                        }
                        Ok(arguments)
                    });
                let (reply, errno) = match &arguments {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = json::encode(arguments)?;
                        task.send::<ReviewRequest>(&bytes.as_slice()).await?;
                        let result = json::decode(task.recv::<ReviewResult>().await?)?;
                        patient::actual_result(&result)?;
                        let bytes = json::encode(&result)?;
                        task.send::<MedicationReview>(&bytes.as_slice()).await?;
                        (result, 0)
                    }
                };
                let bytes = json::encode(&reply)?;
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
                if arguments.is_ok() {
                    break;
                }
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, PHARMACIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the review Endpoint has completed"});
                let bytes = json::encode(&reply)?;
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
