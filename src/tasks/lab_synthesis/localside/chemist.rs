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
    endpoint: &mut hibana::Endpoint<CHEMIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let raw = {
        let mut call = guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
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
                                imp::assigned(&value, "sample", "raw_materials")?;
                                Ok(value)
                            });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let reply = {
                                let bytes = imp::encode(arguments)?;
                                task.send::<SpecIntent>(&bytes.as_slice()).await?;
                                let mut instrument = task.recv::<SpecLoan>().await?;
                                instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                                let result = imp::decode(instrument.recv::<SpecResult>().await?)?;
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
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let purification = {
        let mut call = guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
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
                            imp::assigned(&value, "material", "compound_purification")?;
                            Ok(value)
                        });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let reply = {
                                let bytes = imp::encode(arguments)?;
                                task.send::<CentIntent>(&bytes.as_slice()).await?;
                                let mut instrument = task.recv::<CentLoan>().await?;
                                instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                                let result = imp::decode(instrument.recv::<CentResult>().await?)?;
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
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let sample = json!({"raw":raw,"purification":purification});
    let bytes = imp::encode(&sample)?;
    task.send::<Sample>(&bytes.as_slice()).await?;
    let (quality, call) = async {
        async { imp::decode(task.recv::<Quality>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
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
    imp::same(&quality["biology"]["sample"], &sample)?;
    let _validation = {
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
                                imp::assigned(&value, "sample", "final_validation")?;
                                Ok(value)
                            });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let reply = {
                                let envelope = json!({"arguments":arguments,"quality":quality});
                                let bytes = imp::encode(&envelope)?;
                                task.send::<SpecIntent>(&bytes.as_slice()).await?;
                                let mut instrument = task.recv::<SpecLoan>().await?;
                                instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                                let result = imp::decode(instrument.recv::<SpecResult>().await?)?;
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
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
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
