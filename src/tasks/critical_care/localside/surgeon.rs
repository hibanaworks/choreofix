//! The SURGEON owns its guest and uses the actual received operation rights.
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
    endpoint: &mut hibana::Endpoint<SURGEON>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, SURGEON>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "SURGEON exited before its actual request",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments =
                    imp::file_arguments(&file, (b"service/perform_surgery", "patient_a")).and_then(
                        |arguments| {
                            if arguments["procedure"] != "trauma_surgery" {
                                return Err(ShellError::message("unknown surgical procedure"));
                            }
                            Ok(arguments)
                        },
                    );
                let (reply, errno) = match &arguments {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        task.send::<SurgeryIntent>(&bytes.as_slice()).await?;
                        let mut resource = task.recv::<SurgeryLoan>().await?;
                        resource.send::<SurgeryRequest>(&bytes.as_slice()).await?;
                        let result = imp::decode(resource.recv::<SurgeryResult>().await?)?;
                        task.send::<SurgeryLoan>(resource).await?;

                        imp::actual_result(&result, "patient_a")?;
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

                if arguments.is_ok() {
                    break;
                }
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, SURGEON>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the assigned request has completed"});
                let errno = 8;
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
            }
        }
    }
}
