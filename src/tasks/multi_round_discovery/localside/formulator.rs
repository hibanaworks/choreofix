//! Formulation requests use the received compound and the actual material loan.
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
    endpoint: &mut Endpoint<FORMULATOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let _qc = task.recv::<Specimen<QC>>().await?;
    let identity = imp::decode(task.recv::<Identity<FORMULATOR>>().await?)?;
    imp::reply(3, &identity)?;
    let mut previous = identity.clone();
    let bytes = imp::encode(&identity)?;
    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
        private
            .send::<ResultChunkMsg>(&ResultChunk::new(
                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                chunk,
            )?)
            .await?;
    }
    loop {
        let branch = task.offer().await?;
        if branch.label() == Final::<FORMULATOR>::LOGICAL_LABEL {
            let actual = imp::decode(branch.recv::<Final<FORMULATOR>>().await?)?;
            imp::same(&actual["compound"], &identity)?;
            previous["coordination"] = actual;
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
        let mut material = branch.recv::<Specimen<FORMULATION>>().await?;
        previous["coordination"] = json!({"received_specimens":["FORMULATION"]});
        let bytes = imp::encode(&previous)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        loop {
            let FileBoundary::Closed { pending, file } = guest
                .next_file::<ClosedFile, FORMULATOR>(&mut private, SERVICE_FD)
                .await?
            else {
                return Err(ShellError::message(
                    "formulator exited before preparing the material",
                ));
            };
            let requested = imp::analysis_file::<4>(&file).and_then(|value| {
                imp::formulation_input(&identity, &value)?;
                Ok(value)
            });
            match requested {
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
                    let input = imp::AnalysisArguments::<4>::from_bytes(&bytes)?;
                    material.send::<Intent<4>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<FORM>>().await?;
                    instrument.send::<Request<4>>(&input).await?;
                    task.send::<Instrument<FORM>>(instrument).await?;
                    task.send::<Specimen<FORMULATION>>(material).await?;
                    let branch = task.offer().await?;
                    match branch.label() {
                        label if label == Passed::<4>::LOGICAL_LABEL => {
                            let actual = imp::decode(branch.recv::<Passed<4>>().await?)?;
                            imp::formulation_receipt(&identity, &actual)?;
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
                            previous = actual;
                            break;
                        }
                        label if label == Failed::<4>::LOGICAL_LABEL => {
                            let actual = imp::decode(branch.recv::<Failed<4>>().await?)?;
                            imp::formulation_receipt(&identity, &actual)?;
                            material = task.recv::<Specimen<FORMULATION>>().await?;
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
                        _ => return Err(ShellError::message("unexpected formulation result")),
                    }
                }
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, FORMULATOR>(&mut private, SERVICE_FD)
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
