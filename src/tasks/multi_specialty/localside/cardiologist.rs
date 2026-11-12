//! Direct cardiologist referral, investigation, diagnosis and conference result.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::g::Message;
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<CARDIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (assessment, mut call) = async {
        async { json::decode(task.recv::<CardReferral>().await?) };
        async {
            Ok(guest
                .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
                .await?)
        };
    }
    .await?;
    patient::actual_result(&assessment)?;
    let bytes = json::encode(&assessment)?;
    async {
        let [mut owner, mut radiologist, mut imaging] = task.par()?;
        async {
            owner.send::<ReferralForOwner>(&bytes.as_slice()).await?;
        };
        async {
            radiologist.send::<CtOrder>(&bytes.as_slice()).await?;
        };
        async {
            imaging.send::<CtOrderForOwner>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("specialist exited before actual tests"));
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
                        let mut panel = task.recv::<PanelLoan<CARDIAC_PANEL>>().await?;
                        panel.send::<TestRequest>(&bytes.as_slice()).await?;
                        let attempt = json::decode(panel.recv::<TestResult>().await?)?;
                        patient::actual_result(&attempt)?;
                        task.send::<PanelReturn<CARDIAC_PANEL>>(panel).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            PanelPartial::LOGICAL_LABEL => {
                                let receipts = json::decode(branch.recv::<PanelPartial>().await?)?;
                                if patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])?
                                {
                                    return Err(ShellError::message(
                                        "partial reply already has the complete panel",
                                    ));
                                }
                                (attempt, 0)
                            }
                            PanelComplete::LOGICAL_LABEL => {
                                let receipts = json::decode(branch.recv::<PanelComplete>().await?)?;
                                if !patient::panel_complete(&receipts, &["ecg", "cardiac_enzymes"])?
                                {
                                    return Err(ShellError::message(
                                        "completed reply lacks actual tests",
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
                            _ => return Err(ShellError::message("unexpected actual panel result")),
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
    let ((general, cross, ct, mri), mut call) = async {
        async {
            let [mut g, mut x, mut c, mut m] = task.par()?;
            async { json::decode(g.recv::<GeneralForCard>().await?) };
            async { json::decode(x.recv::<NeuroPanelForCard>().await?) };
            async { json::decode(c.recv::<CtForCard>().await?) };
            async { json::decode(m.recv::<MriForCard>().await?) };
        };
        async {
            Ok(guest
                .next_file::<ClosedFile, CARDIOLOGIST>(&mut private, SERVICE_FD)
                .await?)
        };
    }
    .await?;
    json::same(&general["assessment"], &assessment)?;
    if !patient::panel_complete(&general["tests"], &["vitals", "basic_labs"])?
        || !patient::panel_complete(&cross, &["neuro_exam", "nerve_conduction"])?
    {
        return Err(ShellError::message(
            "shared investigation lacks actual panel findings",
        ));
    }
    for image in [&ct, &mri] {
        patient::actual_result(image)?;
    }
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("specialist exited before diagnosis"));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments =
                    patient::file_arguments(&file, b"service/diagnose").and_then(|arguments| {
                        if arguments.get("symptoms").is_some_and(|v| !v.is_string()) {
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
                        task.send::<CardDiagnosisRequest>(&bytes.as_slice()).await?;
                        let diagnosis = json::decode(task.recv::<CardDiagnosisResult>().await?)?;
                        patient::actual_result(&diagnosis)?;
                        let final_case = json::decode(task.recv::<FinalForCard>().await?)?;
                        json::same(&final_case["case"]["cardiac"]["result"], &diagnosis)?;
                        let mut reply = diagnosis;
                        reply["consultation"] = final_case;
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
                let reply = json!({"status":"out_of_order","error":"the consultation Endpoint has completed"});
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
