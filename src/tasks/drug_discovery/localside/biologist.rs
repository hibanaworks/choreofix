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
    endpoint: &mut hibana::Endpoint<BIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (mut cell, mut call) = async {
        async { task.recv::<CellLoan>().await };
        async {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "scientist exited before completing the task",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;

    let prep = loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing the task",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/run_cell_assay", "assay_type")
                    .and_then(|value| {
                        imp::assigned(&value, "assay_type", "culture_prep")?;
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
                            task.send::<CellIntent>(&bytes.as_slice()).await?;
                            cell.send::<CellRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(cell.recv::<CellResult>().await?)?;
                            imp::successful(&result)?;
                            result
                        };
                        (reply, 0)
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
                    break reply;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
    };
    let (compound, mut call) = async {
        async { imp::decode(task.recv::<CompoundBio>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "scientist exited before completing the task",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;

    let assay = loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing the task",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/run_cell_assay", "assay_type")
                    .and_then(|value| {
                        imp::assigned(&value, "assay_type", "bioassay")?;
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
                            task.send::<CellIntent>(&bytes.as_slice()).await?;
                            cell.send::<CellRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(cell.recv::<CellResult>().await?)?;
                            imp::successful(&result)?;
                            result
                        };
                        (reply, 0)
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
                    break reply;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
    };
    let mut call = guest
        .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
        .await?;
    let stability = loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing the task",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/run_hplc_analysis", "sample")
                    .and_then(|value| {
                        imp::assigned(&value, "sample", "stability")?;
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
                            task.send::<HplcIntent>(&bytes.as_slice()).await?;
                            let mut instrument = task.recv::<HplcLoan>().await?;
                            instrument.send::<HplcRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(instrument.recv::<HplcResult>().await?)?;
                            imp::successful(&result)?;
                            task.send::<HplcLoan>(instrument).await?;
                            result
                        };
                        (reply, 0)
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
                    break reply;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
    };
    task.send::<CellLoan>(cell).await?;
    let report = json!({"compound":compound,"prep":prep,"assay":assay,"stability":stability});
    let bytes = imp::encode(&report)?;
    task.send::<BioReport>(&bytes.as_slice()).await?;
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
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
