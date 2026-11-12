//! Three actual image/report pairs, with the request order chosen by the guest.
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
    endpoint: &mut hibana::Endpoint<RADIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (pending, reply) = async {
        let [mut ct, mut mri] = task.par()?;
        // Bounded actual receipts also supply the returned findings and image IDs.
        let mut data = json!({"images":{}, "reports":{}});
        loop {
            match guest.next_file::<ClosedFile, RADIOLOGIST>(&mut private, SERVICE_FD).await? {
                FileBoundary::Exit(_) => return Err(ShellError::message("radiologist exited before all actual reports")),
                FileBoundary::Closed { pending, file } => {
                    let arguments = (|| {
                        if file.completion.0.errno() != 0 ||
                            ![b"service/perform_imaging".as_slice(), b"service/interpret_imaging".as_slice()].contains(&file.path.as_slice())
                        {
                            return Err(ShellError::message("expected an imaging service file"));
                        }
                        let arguments = imp::decode(&file.bytes)?;
                        match (arguments["patient_id"].as_str(), arguments["modality"].as_str()) {
                            (Some("patient_a"), Some(m)) if m.eq_ignore_ascii_case("CT") => {},
                            (Some("patient_b"), Some(m)) if m.eq_ignore_ascii_case("CT") || m.eq_ignore_ascii_case("MRI") => {},
                            _ => return Err(ShellError::message("unknown imaging patient or modality")),
                        }
                        Ok(arguments)
                    })();
                    let (mut reply, errno) = match arguments {
                        Err(error) => (json!({"status":"invalid_arguments","error":error.to_string()}), 28),
                        Ok(arguments) => {
                            let patient = arguments["patient_id"].as_str().expect("validated patient");
                            let modality = arguments["modality"].as_str().expect("validated modality").to_ascii_uppercase();
                            let key = format!("{patient}:{modality}");
                            match file.path.as_slice() {
                                b"service/interpret_imaging" if data["images"][&key].is_null() =>
                                    (json!({"status":"invalid_arguments","error":"the matching actual image is absent"}), 28),
                                _ => {
                                    let bytes = imp::encode(&arguments)?;
                                    let result = match (file.path.as_slice(), patient, modality.as_str()) {
                                        (b"service/perform_imaging", "patient_a", "CT") => {
                                            ct.send::<CtPerformIntentA>(&bytes.as_slice()).await?;
                                            let mut resource = ct.recv::<CtLoan>().await?;
                                            resource.send::<CtPerformRequestA>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<CtImageA>().await?)?;
                                            ct.send::<CtLoan>(resource).await?;
                                            result
                                        }
                                        (b"service/interpret_imaging", "patient_a", "CT") => {
                                            ct.send::<CtInterpretIntentA>(&bytes.as_slice()).await?;
                                            let mut resource = ct.recv::<CtLoan>().await?;
                                            resource.send::<CtInterpretRequestA>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<CtReportA>().await?)?;
                                            ct.send::<CtLoan>(resource).await?;
                                            result
                                        }
                                        (b"service/perform_imaging", "patient_b", "CT") => {
                                            ct.send::<CtPerformIntentB>(&bytes.as_slice()).await?;
                                            let mut resource = ct.recv::<CtLoan>().await?;
                                            resource.send::<CtPerformRequestB>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<CtImageB>().await?)?;
                                            ct.send::<CtLoan>(resource).await?;
                                            result
                                        }
                                        (b"service/interpret_imaging", "patient_b", "CT") => {
                                            ct.send::<CtInterpretIntentB>(&bytes.as_slice()).await?;
                                            let mut resource = ct.recv::<CtLoan>().await?;
                                            resource.send::<CtInterpretRequestB>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<CtReportB>().await?)?;
                                            ct.send::<CtLoan>(resource).await?;
                                            result
                                        }
                                        (b"service/perform_imaging", "patient_b", "MRI") => {
                                            mri.send::<MriPerformIntent>(&bytes.as_slice()).await?;
                                            let mut resource = mri.recv::<MriLoan>().await?;
                                            resource.send::<MriPerformRequest>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<MriImage>().await?)?;
                                            mri.send::<MriLoan>(resource).await?;
                                            result
                                        }
                                        (b"service/interpret_imaging", "patient_b", "MRI") => {
                                            mri.send::<MriInterpretIntent>(&bytes.as_slice()).await?;
                                            let mut resource = mri.recv::<MriLoan>().await?;
                                            resource.send::<MriInterpretRequest>(&bytes.as_slice()).await?;
                                            let result = imp::decode(resource.recv::<MriReport>().await?)?;
                                            mri.send::<MriLoan>(resource).await?;
                                            result
                                        }
                                        _ => return Err(ShellError::message("validated imaging call has another target")),
                                    };
                                    imp::actual_result(&result, patient)?;
                                    if file.path == b"service/perform_imaging" {
                                        data["images"][&key] = result.clone();
                                    } else {
                                        imp::same(&result["data"]["inputs"]["image"], &data["images"][&key]["data"]["artifact_id"])?;
                                        data["reports"][&key] = result.clone();
                                    }
                                    (result, 0)
                                }
                            }
                        }
                    };
                    if data["reports"].as_object().expect("actual report object").len() == 3 {
                        reply["findings"] = data;
                        break Ok::<_, ShellError>((pending, reply));
                    }
                    let bytes = imp::encode(&reply)?;
                    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                        private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
                    }
                    guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;
                }
            }
        }
    }.await?;
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
        protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
    )))?;
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, RADIOLOGIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the actual imaging case is complete"});
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
