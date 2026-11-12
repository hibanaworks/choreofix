//! The chemist uses the actual QC, identity and final-documentation Endpoints.
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
    (endpoint, mut qc): (&mut Endpoint<'r, CHEMIST>, Endpoint<'r, QC>),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing an analysis",
                ));
            }
            FileBoundary::Closed { pending, file } => match imp::analysis_file::<0>(&file) {
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
                    let input = imp::AnalysisArguments::<0>::from_bytes(&bytes)?;
                    qc.send::<Intent<0>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                    instrument.send::<Request<0>>(&input).await?;
                    task.send::<Instrument<HPLC>>(instrument).await?;
                    task.send::<Specimen<QC>>(qc).await?;

                    let branch = task.offer().await?;
                    match branch.label() {
                        label if label == Passed::<0>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Passed<0>>().await?)?;
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
                            break;
                        }
                        label if label == Failed::<0>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Failed<0>>().await?)?;
                            qc = task.recv::<Specimen<QC>>().await?;
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
        match guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing mass spectrometry",
                ));
            }
            FileBoundary::Closed { pending, file } => match imp::analysis_file::<3>(&file) {
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
                    let input = imp::AnalysisArguments::<3>::from_bytes(&bytes)?;
                    task.send::<Intent<3>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<MASS>>().await?;
                    instrument.send::<Request<3>>(&input).await?;
                    let result = imp::decode(instrument.recv::<Passed<3>>().await?)?;
                    imp::reply(3, &result)?;
                    task.send::<Instrument<MASS>>(instrument).await?;
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;

                    break;
                }
            },
        }
    }

    let (mut call, mut study) = async {
        async {
            Ok(guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?)
        };
        async {
            let study = task.recv::<Specimen<STUDY>>().await?;
            let _outcome = imp::decode(task.recv::<Final<CHEMIST>>().await?)?;
            Ok(study)
        };
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing an analysis",
                ));
            }
            FileBoundary::Closed { pending, file } => match imp::analysis_file::<15>(&file) {
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
                    let input = imp::AnalysisArguments::<15>::from_bytes(&bytes)?;
                    study.send::<Intent<15>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                    instrument.send::<Request<15>>(&input).await?;
                    task.send::<Instrument<HPLC>>(instrument).await?;
                    task.send::<Specimen<STUDY>>(study).await?;

                    let branch = task.offer().await?;
                    match branch.label() {
                        label if label == Passed::<15>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Passed<15>>().await?)?;
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
                            break;
                        }
                        label if label == Failed::<15>::LOGICAL_LABEL => {
                            let result = imp::decode(branch.recv::<Failed<15>>().await?)?;
                            study = task.recv::<Specimen<STUDY>>().await?;
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
        call = guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
    }
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
                let result =
                    json!({"status":"out_of_order","error":"the scientific task has completed"});
                let bytes = imp::encode(&result)?;
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
