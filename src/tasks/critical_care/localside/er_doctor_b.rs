//! Actual patient B requests, parallel review and final care data.
use super::super::{global::*, imp};
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
    endpoint: &mut hibana::Endpoint<ER_B>,
) -> ShellResult<i32> {
    let [mut clinical, mut review, mut private] = endpoint.par()?;
    let ((pending, mut reply, _panel), prescription_review) = async {
        async {
            loop {
                match guest.next_file::<ClosedFile, ER_B>(&mut private, SERVICE_FD).await? {
                    FileBoundary::Exit(_) => return Err(ShellError::message("patient B doctor exited before assessment")),
                    FileBoundary::Closed { pending, file } => {
                        let arguments = imp::file_arguments(&file, (b"service/assess_patient", "patient_b"));
                        let (reply, errno) = match &arguments {
                            Err(error) => (json!({"status":"invalid_arguments","error":error.to_string()}), 28),
                            Ok(arguments) => {
                                let bytes = imp::encode(arguments)?;
                                clinical.send::<AssessmentRequestB>(&bytes.as_slice()).await?;
                                let result = imp::decode(clinical.recv::<AssessmentResultB>().await?)?;
                                imp::actual_result(&result, "patient_b")?;
                                (result, 0)
                            }
                        };
                        let bytes = imp::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;

                        if arguments.is_ok() { break; }
                    }
                }
            }
            let panel = loop {
                match guest.next_file::<ClosedFile, ER_B>(&mut private, SERVICE_FD).await? {
                    FileBoundary::Exit(_) => return Err(ShellError::message("patient B doctor exited before the test panel")),
                    FileBoundary::Closed { pending, file } => {
                        let arguments = imp::file_arguments(&file, (b"service/order_tests", "patient_b"))
                            .and_then(|arguments| { imp::tests(&arguments, ("patient_b", &["stroke_panel", "ct_angio"]))?; Ok(arguments) });
                        let (reply, errno) = match arguments {
                            Err(error) => (json!({"status":"invalid_arguments","error":error.to_string()}), 28),
                            Ok(arguments) => {
                                let mut panel = clinical.recv::<PanelLoan<PANEL_B>>().await?;
                                let bytes = imp::encode(&arguments)?;
                                panel.send::<TestRequest>(&bytes.as_slice()).await?;
                                let attempt = imp::decode(panel.recv::<TestResult>().await?)?;
                                imp::actual_result(&attempt, "patient_b")?;
                                clinical.send::<PanelReturn<PANEL_B>>(panel).await?;
                                let branch = clinical.offer().await?;
                                match branch.label() {
                                    PanelPartial::LOGICAL_LABEL => {
                                        let receipts = imp::decode(branch.recv::<PanelPartial>().await?)?;
                                        if imp::panel_complete(&receipts, ("patient_b", &["stroke_panel", "ct_angio"]))? {
                                            return Err(ShellError::message("partial reply contains a complete actual panel"));
                                        }
                                    }
                                    PanelCompleted::<PANEL_B>::LOGICAL_LABEL => {
                                        let panel = branch.recv::<PanelCompleted<PANEL_B>>().await?;
                                        let receipts = imp::decode(clinical.recv::<PanelComplete>().await?)?;
                                        if !imp::panel_complete(&receipts, ("patient_b", &["stroke_panel", "ct_angio"]))? {
                                            return Err(ShellError::message("completed reply lacks actual patient results"));
                                        }
                                        let reply = attempt;
                                        let errno = 0;
                                        let bytes = imp::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;

                                        break panel;
                                    }
                                    _ => return Err(ShellError::message("unexpected patient panel reply")),
                                }
                                (attempt, 0)
                            }
                        };
                        let bytes = imp::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;

                    }
                }
            };
            loop {
                match guest.next_file::<ClosedFile, ER_B>(&mut private, SERVICE_FD).await? {
                    FileBoundary::Exit(_) => return Err(ShellError::message("patient B doctor exited before diagnosis")),
                    FileBoundary::Closed { pending, file } => {
                        let arguments = imp::file_arguments(&file, (b"service/diagnose", "patient_b"))
                            .and_then(|arguments| { imp::optional_text(&arguments, "symptoms")?; Ok(arguments) });
                        let (reply, errno) = match arguments {
                            Err(error) => (json!({"status":"invalid_arguments","error":error.to_string()}), 28),
                            Ok(arguments) => {
                                let findings = imp::decode(clinical.recv::<FindingsB>().await?)?;
                                imp::actual_result(&findings["assessment"], "patient_b")?;
                                imp::actual_result(&findings["ct"], "patient_b")?;
                                imp::actual_result(&findings["mri"], "patient_b")?;
                                let bytes = imp::encode(&arguments)?;
                                clinical.send::<DiagnosisRequestB>(&bytes.as_slice()).await?;
                                let diagnosis = imp::decode(clinical.recv::<DiagnosisResultB>().await?)?;
                                imp::actual_result(&diagnosis, "patient_b")?;
                                let care = imp::decode(clinical.recv::<IcuForER>().await?)?;
                                imp::actual_result(&care, "patient_b")?;
                                break Ok::<_, ShellError>((pending,
                                    json!({"success":true,"diagnosis":diagnosis,"care":care,"findings":findings}), panel));
                            }
                        };
                        let bytes = imp::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;

                    }
                }
            }
        };
        async {
            let result = imp::decode(review.recv::<ReviewForB>().await?)?;
            imp::actual_result(&result, "patient_A_and_B")?;
            Ok(result)
        };
    }.await?;
    reply["prescription_review"] = prescription_review;
    let errno = 0;
    let bytes = imp::encode(&reply)?;
    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
        private
            .send::<ResultChunkMsg>(&ResultChunk::new(
                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                chunk,
            )?)
            .await?;
    }
    let mut guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
        protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
    )))?;

    drop(clinical);
    drop(review);
    loop {
        match guest
            .next_file::<ClosedFile, ER_B>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply = json!({"status":"out_of_order","error":"the assigned patient requests have completed"});
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
