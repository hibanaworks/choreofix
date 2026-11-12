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
    endpoint: &mut hibana::Endpoint<ANALYST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (biology, call) = async {
        async { imp::decode(task.recv::<ApprovedBiology>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, ANALYST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "scientist exited before receiving the actual result",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;
    let molecular = {
        let mut call = call;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing the laboratory work",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        file.arguments(b"service/run_analysis", "sample")
                            .and_then(|value| {
                                imp::assigned(&value, "sample", "molecular_detail")?;
                                Ok(value)
                            });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let reply = {
                                let admitted =
                                    json!({"arguments":arguments,"biology":biology["result"]});
                                let bytes = imp::encode(&admitted)?;
                                task.send::<Molecular>(&bytes.as_slice()).await?;
                                let mut instrument = task.recv::<SpecLoan>().await?;
                                instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                                let result =
                                    imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                                imp::successful(&result)?;
                                task.send::<SpecLoan>(instrument).await?;
                                result
                            };
                            (reply, 0)
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
                        break reply;
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, ANALYST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let separation = {
        let mut call = guest
            .next_file::<ClosedFile, ANALYST>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing the laboratory work",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/run_separation", "material")
                        .and_then(|value| {
                            imp::assigned(&value, "material", "verification")?;
                            Ok(value)
                        });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let reply = {
                                let admitted =
                                    json!({"arguments":arguments,"biology":biology["result"]});
                                let bytes = imp::encode(&admitted)?;
                                task.send::<Separation>(&bytes.as_slice()).await?;
                                let mut instrument = task.recv::<CentLoan>().await?;
                                instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                                let result =
                                    imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                                imp::successful(&result)?;
                                task.send::<CentLoan>(instrument).await?;
                                result
                            };
                            (reply, 0)
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
                        break reply;
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, ANALYST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let quality = json!({"biology":biology,"molecular":molecular,"separation":separation});
    let bytes = imp::encode(&quality)?;
    async {
        let [mut chemist, mut owner] = task.par()?;
        async {
            chemist.send::<Quality>(&bytes.as_slice()).await?;
        };
        async {
            owner.send::<QualityForOwner>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, ANALYST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
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
