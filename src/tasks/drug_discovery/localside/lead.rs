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
    endpoint: &mut hibana::Endpoint<LEAD>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut call = guest
        .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
        .await?;
    let reference = loop {
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
                        imp::assigned(&value, "sample", "reference_methods")?;
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
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?;
    };
    let (bio, tox, mut call) = async {
        let [mut b, mut t] = task.par()?;
        async { imp::decode(b.recv::<BioReport>().await?) };
        async { imp::decode(t.recv::<ToxReport>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
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
    imp::same(&bio["compound"], &tox["compound"])?;

    let structural = loop {
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
                        imp::assigned(&value, "sample", "structural_confirmation")?;
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
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?;
    };
    let mut call = guest
        .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
        .await?;
    let characterization = loop {
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
                        imp::assigned(&value, "sample", "characterization")?;
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
                let mut visible = reply.clone();
                if requested.is_ok() {
                    visible["context"] = json!({"compound":bio["compound"],"bio":bio,"tox":tox,"reference":reference,"structural":structural});
                }
                let bytes = imp::encode(&visible)?;
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
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?;
    };
    let mut call = guest
        .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
        .await?;
    let _decision = loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing the task",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = imp::decision(&file);
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let reply = {
                            let mut decision = json!({"go":arguments["go"],"reference":reference,"compound":bio["compound"],"bio":bio,"tox":tox,"analyses":{"structural":structural,"characterization":characterization}});
                            let bytes = imp::encode(&decision)?;
                            task.send::<DecisionForOwner>(&bytes.as_slice()).await?;
                            let receipt = imp::decode(task.recv::<DecisionSubmitted>().await?)?;
                            imp::decision_receipt(&decision, &receipt)?;
                            decision["receipt"] = receipt.clone();
                            let bytes = imp::encode(&decision)?;
                            task.send::<Decision>(&bytes.as_slice()).await?;
                            receipt
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
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?;
    };
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
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
