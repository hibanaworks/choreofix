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
    let mut call = guest
        .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
        .await?;
    let qc = loop {
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
                        imp::assigned(&value, "sample", "synthesis_qc")?;
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
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
    };
    let mut call = guest
        .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
        .await?;
    let identity = loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing the task",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/run_mass_spec_analysis", "sample")
                    .and_then(|value| {
                        imp::assigned(&value, "sample", "molecular_id")?;
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
                            task.send::<MassIntent>(&bytes.as_slice()).await?;
                            let mut instrument = task.recv::<MassLoan>().await?;
                            instrument.send::<MassRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(instrument.recv::<MassResult>().await?)?;
                            imp::successful(&result)?;
                            task.send::<MassLoan>(instrument).await?;
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
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
    };
    let compound = json!({"qc":qc,"identity":identity});
    let bytes = imp::encode(&compound)?;
    async {
        let [mut bio, mut tox] = task.par()?;
        async {
            bio.send::<CompoundBio>(&bytes.as_slice()).await?;
        };
        async {
            tox.send::<CompoundTox>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    let (decision, mut call) = async {
        async { imp::decode(task.recv::<Decision>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
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

    let _documentation = loop {
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
                        imp::assigned(&value, "sample", "documentation")?;
                        Ok(value)
                    });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let reply = {
                            let envelope = json!({"arguments":arguments,"decision":decision});
                            let bytes = imp::encode(&envelope)?;
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
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
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
