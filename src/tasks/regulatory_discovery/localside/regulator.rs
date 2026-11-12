//! The scientist writes Endpoint operations directly, using actual guest input.
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
    endpoint: &mut Endpoint<REGULATOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    'reviews: loop {
        let submission = imp::decode(task.recv::<Decision>().await?)?;
        imp::decision_receipt(&submission["decision"], &submission["receipt"])?;
        let mut study = task.recv::<Specimen<STUDY>>().await?;
        loop {
            match guest
                .next_file::<ClosedFile, REGULATOR>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "regulator exited before approving the current study",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/verify_regulatory_compliance", "decision")
                        .and_then(|v| {
                            imp::same(&v, &json!({"decision":"go_no_go"}))?;
                            Ok(v)
                        });
                    match requested {
                        Err(error) => {
                            let result =
                                json!({"status":"invalid_arguments","error":error.to_string()});
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
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                        Ok(arguments) => {
                            let mut admitted = submission.clone();
                            admitted["arguments"] = arguments;
                            let bytes = imp::encode(&admitted)?;
                            study.send::<Review>(&bytes.as_slice()).await?;
                            task.send::<Specimen<STUDY>>(study).await?;
                            let branch = task.offer().await?;
                            match branch.label() {
                                label if label == Approved::LOGICAL_LABEL => {
                                    let result = imp::decode(branch.recv::<Approved>().await?)?;
                                    imp::regulatory_receipt(&admitted, &result)?;
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
                                    break 'reviews;
                                }
                                label if label == Rejected::LOGICAL_LABEL => {
                                    let result = imp::decode(branch.recv::<Rejected>().await?)?;
                                    imp::regulatory_receipt(&admitted, &result)?;
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
                                    continue 'reviews;
                                }
                                _ => {
                                    return Err(ShellError::message(
                                        "unexpected regulatory result",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, REGULATOR>(&mut private, SERVICE_FD)
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
