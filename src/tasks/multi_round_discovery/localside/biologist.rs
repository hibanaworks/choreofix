//! Biological work keeps actual specimen ownership until the scientist submits it.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<'r>(
    mut guest: &mut HibanaWasiGuest,
    (endpoint, mut culture): (&mut Endpoint<'r, BIOLOGIST>, Endpoint<'r, CULTURE>),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut previous;

    loop {
        match guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing an analysis",
                ));
            }
            FileBoundary::Closed { pending, file } => match imp::analysis_file::<1>(&file) {
                Err(error) => {
                    let result = json!({"status":"invalid_arguments","error":error.to_string()});
                    let bytes = imp::encode(&result)?;
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 28)),
                    ))?;
                }
                Ok(arguments) => {
                    let bytes = imp::encode(&arguments)?;
                    let input = imp::AnalysisArguments::<1>::from_bytes(&bytes)?;
                    culture.send::<Intent<1>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<CELL>>().await?;
                    instrument.send::<Request<1>>(&input).await?;
                    task.send::<Instrument<CELL>>(instrument).await?;
                    task.send::<Specimen<CULTURE>>(culture).await?;

                    let branch = task.offer().await?;
                    match branch.label() {
                        label if label == Passed::<1>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Passed<1>>().await?)?;
                            let bytes = imp::encode(&result)?;
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
                            previous = result;
                            break;
                        }
                        label if label == Failed::<1>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Failed<1>>().await?)?;
                            culture = task.recv::<Specimen<CULTURE>>().await?;
                            let bytes = imp::encode(&result)?;
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
                        }
                        _ => {
                            return Err(ShellError::message("unexpected actual analysis outcome"));
                        }
                    }
                }
            },
        }
    }
    loop {
        let branch = task.offer().await?;
        if branch.label() == Final::<BIOLOGIST>::LOGICAL_LABEL {
            previous["coordination"] = imp::decode(branch.recv::<Final<BIOLOGIST>>().await?)?;
            let bytes = imp::encode(&previous)?;
            for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                private
                    .send::<ResultChunkMsg>(&ResultChunk::new(
                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                        chunk,
                    )?)
                    .await?;
            }
            break;
        }
        drop(branch);
        let (bioassay, stability) = async {
            let [mut bio, mut stable] = task.par()?;
            async { bio.recv::<Specimen<BIOASSAY>>().await };
            async { stable.recv::<Specimen<STABILITY>>().await };
        }
        .await?;
        previous["coordination"] = json!({"received_specimens":["BIOASSAY","STABILITY"]});
        let bytes = imp::encode(&previous)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        let (mut bioassay, mut stability) = (Some(bioassay), Some(stability));
        let mut last_bioassay = None;
        let control = &mut task;
        (guest, private, previous) = async move {
            let [mut biological, mut chromatograph] = control.par()?;
            let last_reply = loop {
                let FileBoundary::Closed { pending, file } = guest
                    .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                    .await?
                else {
                    return Err(ShellError::message(
                        "biologist exited with unreturned material",
                    ));
                };
                let reply = if imp::evidence_file(&file, "bioassay").is_ok() {
                    match (last_bioassay.as_ref(), bioassay.take()) {
                        (Some(actual), Some(material)) => {
                            biological.send::<Specimen<BIOASSAY>>(material).await?;
                            Ok(json!({"evidence":"bioassay","result":actual}))
                        }
                        (_, material) => {
                            bioassay = material;
                            Err(ShellError::message("no measured bioassay Endpoint is held"))
                        }
                    }
                } else if let Ok(arguments) = imp::analysis_file::<5>(&file) {
                    if let Some(mut material) = bioassay.take() {
                        let bytes = imp::encode(&arguments)?;
                        let input = imp::AnalysisArguments::<5>::from_bytes(&bytes)?;
                        material.send::<Intent<5>>(&input).await?;
                        let (mut cell, mut samples) = async {
                            let [mut laboratory, mut stock] = biological.par()?;
                            async { laboratory.recv::<Instrument<CELL>>().await };
                            async { stock.recv::<Instrument<SAMPLES>>().await };
                        }
                        .await?;
                        async {
                            async { cell.send::<Request<5>>(&input).await };
                            async { samples.send::<SampleRequest<5>>(&input).await };
                        }
                        .await?;
                        async {
                            let [mut stock, mut laboratory] = biological.par()?;
                            async { stock.send::<Instrument<SAMPLES>>(samples).await };
                            async { laboratory.send::<Instrument<CELL>>(cell).await };
                        }
                        .await?;
                        biological.send::<Specimen<BIOASSAY>>(material).await?;
                        let branch = biological.offer().await?;
                        let actual = if branch.label() == Passed::<5>::LOGICAL_LABEL {
                            imp::decode(branch.recv::<Passed<5>>().await?)?
                        } else {
                            imp::decode(branch.recv::<Failed<5>>().await?)?
                        };
                        bioassay = Some(biological.recv::<Specimen<BIOASSAY>>().await?);
                        last_bioassay = Some(actual.clone());
                        Ok(actual)
                    } else {
                        Err(ShellError::message("bioassay Endpoint has been submitted"))
                    }
                } else if let Ok(arguments) = imp::analysis_file::<6>(&file) {
                    if let Some(mut material) = stability.take() {
                        let bytes = imp::encode(&arguments)?;
                        let input = imp::AnalysisArguments::<6>::from_bytes(&bytes)?;
                        material.send::<Intent<6>>(&input).await?;
                        let mut instrument = chromatograph.recv::<Instrument<HPLC>>().await?;
                        instrument.send::<Request<6>>(&input).await?;
                        chromatograph.send::<Instrument<HPLC>>(instrument).await?;
                        chromatograph.send::<Specimen<STABILITY>>(material).await?;
                        let branch = chromatograph.offer().await?;
                        if branch.label() == Passed::<6>::LOGICAL_LABEL {
                            Ok(imp::decode(branch.recv::<Passed<6>>().await?)?)
                        } else {
                            let actual = imp::decode(branch.recv::<Failed<6>>().await?)?;
                            stability = Some(chromatograph.recv::<Specimen<STABILITY>>().await?);
                            Ok(actual)
                        }
                    } else {
                        Err(ShellError::message("stability Endpoint has been returned"))
                    }
                } else {
                    Err(ShellError::message(
                        "invalid biological analysis or evidence file",
                    ))
                };
                let (reply, errno) = match reply {
                    Ok(actual) => (actual, 0),
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
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
                if bioassay.is_none() && stability.is_none() {
                    break reply;
                }
            };
            Ok((guest, private, last_reply))
        }
        .await?;
    }
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
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the project has a final outcome"}),
                )?;
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
