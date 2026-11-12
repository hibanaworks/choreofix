//! The ICU_COORDINATOR owns its guest and uses the actual received operation rights.
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
    endpoint: &mut hibana::Endpoint<ICU_COORDINATOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, ICU_COORDINATOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "ICU_COORDINATOR exited before its actual request",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments = imp::file_arguments(&file, (b"service/manage_icu", "patient_b"));
                let (reply, errno) = match &arguments {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = imp::encode(arguments)?;
                        task.send::<IcuIntent>(&bytes.as_slice()).await?;
                        let (mut bed, mut ventilator) = async {
                            let [mut bed, mut vent] = task.par()?;
                            async { bed.recv::<ResourceLoan<ICU_BEDS>>().await };
                            async { vent.recv::<ResourceLoan<VENTILATOR>>().await };
                        }
                        .await?;
                        async {
                            async {
                                bed.send::<BedRequest>(&bytes.as_slice()).await?;
                            };
                            async {
                                ventilator
                                    .send::<VentilatorRequest>(&bytes.as_slice())
                                    .await?;
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;
                        let (result, other) = async {
                            async { imp::decode(bed.recv::<BedResult>().await?) };
                            async { imp::decode(ventilator.recv::<VentilatorResult>().await?) };
                        }
                        .await?;
                        imp::same(&result, &other)?;
                        async {
                            let [mut bed_branch, mut vent_branch] = task.par()?;
                            async {
                                bed_branch.send::<ResourceReturn<ICU_BEDS>>(bed).await?;
                            };
                            async {
                                vent_branch
                                    .send::<ResourceReturn<VENTILATOR>>(ventilator)
                                    .await?;
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;

                        imp::actual_result(&result, "patient_b")?;
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
            .next_file::<ClosedFile, ICU_COORDINATOR>(&mut private, SERVICE_FD)
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
