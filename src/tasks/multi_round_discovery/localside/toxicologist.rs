//! Purity precedes toxicity; the scientist decides when to submit its last assay.
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
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<TOXICOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut previous;

    loop {
        match guest
            .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "scientist exited before completing mass spectrometry",
                ));
            }
            FileBoundary::Closed { pending, file } => match imp::analysis_file::<2>(&file) {
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
                    let input = imp::AnalysisArguments::<2>::from_bytes(&bytes)?;
                    task.send::<Intent<2>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<MASS>>().await?;
                    instrument.send::<Request<2>>(&input).await?;
                    let result = imp::decode(instrument.recv::<Passed<2>>().await?)?;
                    imp::reply(2, &result)?;
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

                    previous = result;
                    break;
                }
            },
        }
    }
    loop {
        let branch = task.offer().await?;
        if branch.label() == Final::<TOXICOLOGIST>::LOGICAL_LABEL {
            previous["coordination"] = imp::decode(branch.recv::<Final<TOXICOLOGIST>>().await?)?;
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
        let (mut purity, mut toxicity) = async {
            let [mut pure, mut toxic] = task.par()?;
            async { pure.recv::<Specimen<PURITY>>().await };
            async { toxic.recv::<Specimen<TOXICITY>>().await };
        }
        .await?;
        previous["coordination"] = json!({"received_specimens":["PURITY","TOXICITY"]});
        let bytes = imp::encode(&previous)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        let actual_purity = loop {
            let FileBoundary::Closed { pending, file } = guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                .await?
            else {
                return Err(ShellError::message(
                    "toxicologist exited before measuring purity",
                ));
            };
            match imp::analysis_file::<7>(&file) {
                Err(error) => {
                    let bytes = imp::encode(
                        &json!({"status":"invalid_arguments","error":error.to_string()}),
                    )?;
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
                    let input = imp::AnalysisArguments::<7>::from_bytes(&bytes)?;
                    purity.send::<Intent<7>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                    instrument.send::<Request<7>>(&input).await?;
                    task.send::<Instrument<HPLC>>(instrument).await?;
                    task.send::<Specimen<PURITY>>(purity).await?;
                    let branch = task.offer().await?;
                    match branch.label() {
                        label if label == Passed::<7>::LOGICAL_LABEL => {
                            let actual = imp::decode(branch.recv::<Passed<7>>().await?)?;
                            let bytes = imp::encode(&actual)?;
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
                            break actual;
                        }
                        label if label == Failed::<7>::LOGICAL_LABEL => {
                            let actual = imp::decode(branch.recv::<Failed<7>>().await?)?;
                            purity = task.recv::<Specimen<PURITY>>().await?;
                            let bytes = imp::encode(&actual)?;
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
                        _ => return Err(ShellError::message("unexpected purity result")),
                    }
                }
            }
        };
        let bytes = imp::encode(&actual_purity)?;
        task.send::<PurityEvidence<CELL_SERVICE>>(&imp::JsonTo::<CELL_SERVICE>::from_bytes(
            &bytes,
        )?)
        .await?;
        let mut last_toxicity = None;
        loop {
            let FileBoundary::Closed { pending, file } = guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                .await?
            else {
                return Err(ShellError::message("toxicologist exited with its specimen"));
            };
            if imp::evidence_file(&file, "cytotoxicity").is_ok() && last_toxicity.is_some() {
                task.send::<Specimen<TOXICITY>>(toxicity).await?;
                previous = json!({"evidence":"cytotoxicity","result":last_toxicity});
                let bytes = imp::encode(&previous)?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                )))?;
                break;
            }
            let reply = match imp::analysis_file::<8>(&file) {
                Err(error) => Err(error),
                Ok(arguments) => {
                    let bytes = imp::encode(&arguments)?;
                    let input = imp::AnalysisArguments::<8>::from_bytes(&bytes)?;
                    toxicity.send::<Intent<8>>(&input).await?;
                    let (mut cell, mut samples) = async {
                        let [mut laboratory, mut stock] = task.par()?;
                        async { laboratory.recv::<Instrument<CELL>>().await };
                        async { stock.recv::<Instrument<SAMPLES>>().await };
                    }
                    .await?;
                    async {
                        async { cell.send::<Request<8>>(&input).await };
                        async { samples.send::<SampleRequest<8>>(&input).await };
                    }
                    .await?;
                    async {
                        let [mut stock, mut laboratory] = task.par()?;
                        async { stock.send::<Instrument<SAMPLES>>(samples).await };
                        async { laboratory.send::<Instrument<CELL>>(cell).await };
                    }
                    .await?;
                    task.send::<Specimen<TOXICITY>>(toxicity).await?;
                    let branch = task.offer().await?;
                    let actual = if branch.label() == Passed::<8>::LOGICAL_LABEL {
                        imp::decode(branch.recv::<Passed<8>>().await?)?
                    } else {
                        imp::decode(branch.recv::<Failed<8>>().await?)?
                    };
                    toxicity = task.recv::<Specimen<TOXICITY>>().await?;
                    last_toxicity = Some(actual.clone());
                    Ok(actual)
                }
            };
            let (reply, errno) = match reply {
                Ok(value) => (value, 0),
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
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
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
