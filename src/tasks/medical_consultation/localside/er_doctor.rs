//! ER findings, actual scan custody and panel recovery in the guest's request order.
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
    endpoint: &mut hibana::Endpoint<ER_DOCTOR>,
) -> ShellResult<i32> {
    let [mut clinical, mut review, mut private] = endpoint.par()?;
    let ((pending, mut reply, findings, diagnosis), medication_review) = async {
        async {
            let assessment = loop {
                match guest.next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD).await? {
                    FileBoundary::Exit(_) => return Err(ShellError::message("ER doctor exited before assessment")),
                    FileBoundary::Closed { pending, file } => {
                        match patient::file_arguments(&file, b"service/assess_patient") {
                            Err(error) => { let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?; }
                            Ok(arguments) => {
                                let bytes = json::encode(&arguments)?;
                                clinical.send::<AssessmentRequest>(&bytes.as_slice()).await?;
                                let assessment = json::decode(clinical.recv::<AssessmentResult>().await?)?;
                                patient::actual_result(&assessment)?;
                                let bytes = json::encode(&assessment)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
)))?;
                                break assessment;
                            }
                        }
                    }
                }
            };
            let ((pending, reply, general_results, imaging, _general_panel), _cardiac_panel) = async {
                let [mut tests, mut scan, mut referral] = clinical.par()?;
                async {
                    // One std guest has one outstanding file close. Either
                    // independent request may arrive first, including CT
                    // between partial panel submissions.
                    'both: loop {
                        match guest.next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD).await? {
                            FileBoundary::Exit(_) => return Err(ShellError::message("ER doctor exited before tests and imaging")),
                            FileBoundary::Closed { pending, file } => {
                                match file.path.as_slice() {
                                    b"service/order_tests" => {
                                        match patient::file_arguments(&file, b"service/order_tests").and_then(|arguments| {
    patient::tests(&arguments, &["vitals","basic_labs"])?;
    Ok(arguments)
}) {
                                            Err(error) => { let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?; }
                                            Ok(arguments) => {
                                                let bytes = json::encode(&arguments)?;
let mut panel = tests.recv::<GeneralPanelLoan>().await?;
panel.send::<GeneralTests>(&bytes.as_slice()).await?;
let attempt = json::decode(panel.recv::<GeneralTestResults>().await?)?;
patient::actual_result(&attempt)?;
tests.send::<GeneralPanelReturn>(panel).await?;
let branch = tests.offer().await?;
                                                match branch.label() {
                                                    22 => {
                                                        let receipts = json::decode(branch.recv::<GeneralTestsPartial>().await?)?;
                                                        if patient::panel_complete(&receipts, &["vitals","basic_labs"])? {
                                                            return Err(ShellError::message("partial reply contains a complete panel"));
                                                        }
                                                        let bytes = json::encode(&attempt)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
)))?;
                                                    }
                                                    25 => {
                                                        let panel = branch.recv::<GeneralPanelCompleted>().await?;
                                                        let receipts = json::decode(tests.recv::<GeneralTestsComplete>().await?)?;
                                                        if !patient::panel_complete(&receipts, &["vitals","basic_labs"])? {
                                                            return Err(ShellError::message("completed reply lacks actual general results"));
                                                        }
                                                        let bytes = json::encode(&attempt)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
)))?;
                                                        loop {
                                                            match guest.next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD).await? {
                                                                FileBoundary::Exit(_) => return Err(ShellError::message("ER doctor exited before CT")),
                                                                FileBoundary::Closed { pending, file } => {
                                                                    match patient::file_arguments(&file, b"service/perform_imaging").and_then(|arguments| {
    if arguments["modality"].as_str().is_none_or(|value| !value.eq_ignore_ascii_case("CT")) {
        return Err(ShellError::message("expected CT imaging"));
    }
    Ok(arguments)
}) {
                                                                        Err(error) => { let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?; }
                                                                        Ok(arguments) => {
                                                                            let bytes = json::encode(&arguments)?;
scan.send::<ImagingIntent>(&bytes.as_slice()).await?;
let mut ct = scan.recv::<CtLoan>().await?;
ct.send::<ImagingRequest>(&bytes.as_slice()).await?;
let image = json::decode(ct.recv::<ImagingResult>().await?)?;
patient::actual_result(&image)?;
scan.send::<CtLoan>(ct).await?;
                                                                            break 'both Ok::<_,ShellError>((pending, image.clone(), receipts, image, panel));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => return Err(ShellError::message("unexpected general panel reply")),
                                                }
                                            }
                                        }
                                    }
                                    b"service/perform_imaging" => {
                                        match patient::file_arguments(&file, b"service/perform_imaging").and_then(|arguments| {
    if arguments["modality"].as_str().is_none_or(|value| !value.eq_ignore_ascii_case("CT")) {
        return Err(ShellError::message("expected CT imaging"));
    }
    Ok(arguments)
}) {
                                            Err(error) => { let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?; }
                                            Ok(arguments) => {
                                                let bytes = json::encode(&arguments)?;
scan.send::<ImagingIntent>(&bytes.as_slice()).await?;
let mut ct = scan.recv::<CtLoan>().await?;
ct.send::<ImagingRequest>(&bytes.as_slice()).await?;
let image = json::decode(ct.recv::<ImagingResult>().await?)?;
patient::actual_result(&image)?;
scan.send::<CtLoan>(ct).await?;
                                                let bytes = json::encode(&image)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
)))?;
                                                loop {
                                                    match guest.next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD).await? {
                                                        FileBoundary::Exit(_) => return Err(ShellError::message("ER doctor exited before completing the general panel")),
                                                        FileBoundary::Closed { pending, file } => {
                                                            match patient::file_arguments(&file, b"service/order_tests").and_then(|arguments| {
    patient::tests(&arguments, &["vitals","basic_labs"])?;
    Ok(arguments)
}) {
                                                                Err(error) => { let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?; }
                                                                Ok(arguments) => {
                                                                    let bytes = json::encode(&arguments)?;
let mut panel = tests.recv::<GeneralPanelLoan>().await?;
panel.send::<GeneralTests>(&bytes.as_slice()).await?;
let attempt = json::decode(panel.recv::<GeneralTestResults>().await?)?;
patient::actual_result(&attempt)?;
tests.send::<GeneralPanelReturn>(panel).await?;
let branch = tests.offer().await?;
                                                                    match branch.label() {
                                                                        22 => {
                                                                            let receipts = json::decode(branch.recv::<GeneralTestsPartial>().await?)?;
                                                                            if patient::panel_complete(&receipts, &["vitals","basic_labs"])? {
                                                                                return Err(ShellError::message("partial reply contains a complete panel"));
                                                                            }
                                                                            let bytes = json::encode(&attempt)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
)))?;
                                                                        }
                                                                        25 => {
                                                                            let panel = branch.recv::<GeneralPanelCompleted>().await?;
                                                                            let receipts = json::decode(tests.recv::<GeneralTestsComplete>().await?)?;
                                                                            if !patient::panel_complete(&receipts, &["vitals","basic_labs"])? {
                                                                                return Err(ShellError::message("completed reply lacks actual general results"));
                                                                            }
                                                                            break 'both Ok::<_,ShellError>((pending, attempt, receipts, image, panel));
                                                                        }
                                                                        _ => return Err(ShellError::message("unexpected general panel reply")),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        let error = ShellError::message("expected general tests or CT imaging");
                                        let reply = json!({"status":"invalid_arguments","error":error.to_string()});
let bytes = json::encode(&reply)?;
for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
    private.send::<ResultChunkMsg>(&ResultChunk::new(
        index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk,
    )?).await?;
}
guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
)))?;
                                    }
                                }
                            }
                        }
                    }
                };
                async {
                    let bytes = json::encode(&assessment)?;
                    async {
                        let [mut doctor, mut owner] = referral.par()?;
                        async { doctor.send::<Referral>(&bytes.as_slice()).await?; };
                        async { owner.send::<ReferralForOwner>(&bytes.as_slice()).await?; };
                        Ok(())
                    }.await?;
                    Ok(referral.recv::<CardiacPanelCompleted>().await?)
                };
            }.await?;
            let findings = json!({"assessment":assessment,"tests":general_results,"imaging":imaging});
            let bytes = json::encode(&findings)?;
            async {
                let [mut doctor, mut owner] = clinical.par()?;
                async { doctor.send::<GeneralFindings>(&bytes.as_slice()).await?; };
                async { owner.send::<FindingsForOwner>(&bytes.as_slice()).await?; };
                Ok(())
            }.await?;
            let diagnosis = json::decode(clinical.recv::<CardiacFindings>().await?)?;
            patient::actual_result(&diagnosis)?;
            Ok((pending, reply, findings, diagnosis))
        };
        async {
            let result = json::decode(review.recv::<MedicationReview>().await?)?;
            patient::actual_result(&result)?;
            Ok(result)
        };
    }.await?;
    reply["consultation"] = json!({
        "findings":findings,
        "cardiac_diagnosis":diagnosis,
        "medication_review":medication_review,
    });
    let bytes = json::encode(&reply)?;
    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
        private
            .send::<ResultChunkMsg>(&ResultChunk::new(
                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                chunk,
            )?)
            .await?;
    }
    let mut guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
        protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
    )))?;
    drop(clinical);
    drop(review);
    loop {
        match guest
            .next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD)
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
