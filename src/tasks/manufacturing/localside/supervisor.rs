//! The supervisor's actual close joins four completed job values, then requests the report.
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
    endpoint: &mut Endpoint<SUPERVISOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        let FileBoundary::Closed { pending, file } = guest
            .next_file::<ClosedFile, SUPERVISOR>(&mut private, SERVICE_FD)
            .await?
        else {
            return Err(ShellError::message(
                "supervisor exited before collecting completed reports",
            ));
        };
        let arguments = if file.completion.0.errno() != 0 || file.path != b"service/collect_reports"
        {
            Err(ShellError::message(
                "report file does not match collect_reports",
            ))
        } else {
            imp::decode(&file.bytes).and_then(imp::report_arguments)
        };
        let arguments = match arguments {
            Ok(arguments) => arguments,
            Err(error) => {
                let bytes =
                    imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
                )))?;
                continue;
            }
        };
        let (lathe, mill) = async {
            let [mut lathe, mut mill] = task.par()?;
            async {
                let branch = lathe.offer().await?;
                match branch.label() {
                    LatheDoneA::LOGICAL_LABEL => {
                        let a = imp::decode(branch.recv::<LatheDoneA>().await?)?;
                        let b = imp::decode(lathe.recv::<LatheDoneB>().await?)?;
                        Ok::<_, ShellError>([a, b])
                    }
                    LatheDoneB::LOGICAL_LABEL => {
                        let b = imp::decode(branch.recv::<LatheDoneB>().await?)?;
                        let a = imp::decode(lathe.recv::<LatheDoneA>().await?)?;
                        Ok([a, b])
                    }
                    _ => Err(ShellError::message("unexpected lathe completion")),
                }
            };
            async {
                let branch = mill.offer().await?;
                match branch.label() {
                    MillDoneA::LOGICAL_LABEL => {
                        let a = imp::decode(branch.recv::<MillDoneA>().await?)?;
                        let b = imp::decode(mill.recv::<MillDoneB>().await?)?;
                        Ok::<_, ShellError>([a, b])
                    }
                    MillDoneB::LOGICAL_LABEL => {
                        let b = imp::decode(branch.recv::<MillDoneB>().await?)?;
                        let a = imp::decode(mill.recv::<MillDoneA>().await?)?;
                        Ok([a, b])
                    }
                    _ => Err(ShellError::message("unexpected mill completion")),
                }
            };
        }
        .await?;
        let receipts = json!([lathe[0], lathe[1], mill[0], mill[1]]);
        let expected = imp::completed_jobs(&receipts)?;
        let bytes = imp::encode(&receipts)?;
        task.send::<CompletedJobs>(&bytes.as_slice()).await?;
        let bytes = imp::encode(&arguments)?;
        task.send::<CollectReports>(&bytes.as_slice()).await?;
        let result = imp::decode(task.recv::<Reports>().await?)?;
        imp::report_result(&expected, &result)?;
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
            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
        )))?;
        break;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, SUPERVISOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the four-job report has completed"}),
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
