//! The specialist's real referral, panel requests, findings and diagnosis.
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
    endpoint: &mut hibana::Endpoint<CARDIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (referral, mut call) = async {
        async { json::decode(task.recv::<Referral>().await?) };
        async {
            Ok(guest
                .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
                .await?)
        };
    }
    .await?;
    patient::actual_result(&referral)?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "cardiologist exited before the actual panel",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments =
                    patient::file_arguments(&file, b"service/order_tests").and_then(|arguments| {
                        patient::tests(&arguments, &["ecg", "cardiac_enzymes"])?;
                        Ok(arguments)
                    });
                let (reply, errno) = match &arguments {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let bytes = json::encode(arguments)?;
                        let mut panel = task.recv::<CardiacPanelLoan>().await?;
                        panel.send::<CardiacTests>(&bytes.as_slice()).await?;
                        let attempt = json::decode(panel.recv::<CardiacTestResults>().await?)?;
                        patient::actual_result(&attempt)?;
                        task.send::<CardiacPanelReturn>(panel).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            30 => {
                                let receipts =
                                    json::decode(branch.recv::<CardiacTestsPartial>().await?)?;
                                if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])?
                                {
                                    return Err(ShellError::message(
                                        "partial reply contains a complete panel",
                                    ));
                                }
                                (attempt, 0)
                            }
                            32 => {
                                let receipts =
                                    json::decode(branch.recv::<CardiacTestsComplete>().await?)?;
                                if !patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])?
                                {
                                    return Err(ShellError::message(
                                        "completed reply lacks actual panel results",
                                    ));
                                }
                                let bytes = json::encode(&attempt)?;
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
                                guest = pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                        SERVICE_FD, 0,
                                    )),
                                ))?;
                                break;
                            }
                            _ => return Err(ShellError::message("unexpected cardiac panel reply")),
                        }
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
            }
        }
        call = guest
            .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
    }
    let (findings, mut call) = async {
        async { json::decode(task.recv::<GeneralFindings>().await?) };
        async {
            Ok(guest
                .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
                .await?)
        };
    }
    .await?;
    json::same(&findings["assessment"], &referral)?;
    if !patient::panel_complete(&findings["tests"], &["vitals", "basic_labs"])? {
        return Err(ShellError::message(
            "general findings lack actual panel results",
        ));
    }
    patient::actual_result(&findings["imaging"])?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "cardiologist exited before the actual diagnosis",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments =
                    patient::file_arguments(&file, b"service/diagnose").and_then(|arguments| {
                        if arguments
                            .get("symptoms")
                            .is_some_and(|value| !value.is_string())
                        {
                            return Err(ShellError::message("symptoms must be a string"));
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
                        task.send::<DiagnosisRequest>(&bytes.as_slice()).await?;
                        let diagnosis = json::decode(task.recv::<DiagnosisResult>().await?)?;
                        patient::actual_result(&diagnosis)?;
                        let bytes = json::encode(&diagnosis)?;
                        task.send::<CardiacFindings>(&bytes.as_slice()).await?;
                        let mut reply = diagnosis;
                        reply["consultation"] = findings.clone();
                        (reply, 0)
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
        call = guest
            .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the diagnosis Endpoint has completed"});
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
