//! A real wash writer: ChoreoFS admits requests; the task Endpoint admits effects.
use super::super::{global::*, imp};
use crate::{
    shell::imp::report_arguments::ReportArguments,
    shell::{
        ShellError, ShellResult,
        global::Exit as IoExit,
        imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
    },
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<ROLE>,
) -> ShellResult<i32> {
    if ROLE != WRITER_A && ROLE != WRITER_B {
        return Err(ShellError::message("the document has two actual writers"));
    }
    let assigned = if ROLE == WRITER_A {
        CHAPTER_A
    } else {
        CHAPTER_B
    };
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, ROLE>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "writer exited before the actual combined document",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                if file.path == b"service/feedback" {
                    let valid = file.completion.0.errno() == 0
                        && serde_json::from_slice::<Value>(&file.bytes)
                            .ok()
                            .is_some_and(|value| {
                                value.as_object().is_some_and(|object| object.is_empty())
                            });
                    if valid {
                        let branch = task.offer().await?;
                        let result = match branch.label() {
                            FrozenDocument::LOGICAL_LABEL if ROLE == WRITER_A => {
                                let mut document = branch.recv::<FrozenDocument>().await?;
                                let arguments = task.recv::<CombineForward>().await?;
                                document.send::<CombineOnDocument>(&arguments).await?;
                                let result = imp::decode(document.recv::<Combined>().await?)?;
                                let bytes = imp::encode(&result)?;
                                async {
                                    let [mut notification, mut returned] = task.par()?;
                                    async {
                                        notification
                                            .send::<CombinedForEditor>(&bytes.as_slice())
                                            .await?;
                                    };
                                    async {
                                        returned.send::<DocumentReturn>(document).await?;
                                    };
                                    Ok::<(), ShellError>(())
                                }
                                .await?;
                                result
                            }
                            FinishedB::LOGICAL_LABEL if ROLE == WRITER_B => {
                                imp::decode(branch.recv::<FinishedB>().await?)?
                            }
                            _ => {
                                let (reply, errno) = match branch.label() {
                                    RejectedA::LOGICAL_LABEL => {
                                        (imp::decode(branch.recv::<RejectedA>().await?)?, 0)
                                    }
                                    RejectedB::LOGICAL_LABEL => {
                                        (imp::decode(branch.recv::<RejectedB>().await?)?, 0)
                                    }
                                    RejectedAForB::LOGICAL_LABEL => {
                                        (imp::decode(branch.recv::<RejectedAForB>().await?)?, 0)
                                    }
                                    RejectedBForA::LOGICAL_LABEL => {
                                        (imp::decode(branch.recv::<RejectedBForA>().await?)?, 0)
                                    }
                                    _ => (
                                        json!({"status":"out_of_order","error":"complete the current document work before another feedback read"}),
                                        8,
                                    ),
                                };
                                let bytes = imp::encode(&reply)?;
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
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                        SERVICE_FD, errno,
                                    )),
                                ))?;
                                continue;
                            }
                        };
                        if result["tool_name"] != "combine_sections" || result["success"] != true {
                            return Err(ShellError::message(
                                "the document was not actually combined",
                            ));
                        }
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
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                        )))?;
                        break;
                    }
                }
                let (reply, errno) = 'request: {
                    match file.path.as_slice() {
                        b"service/write_section" => {
                            let arguments = match ReportArguments::<2>::from_close(&file) {
                                Ok(arguments) => arguments,
                                Err(error) => {
                                    break 'request (
                                        json!({"status":"invalid_arguments","error":error.to_string()}),
                                        28,
                                    );
                                }
                            };
                            let value: Value = serde_json::from_slice(&file.bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["section_name"] != assigned {
                                break 'request (
                                    json!({"status":"invalid_arguments","error":"write your initial chapter; either writer may revise either chapter"}),
                                    28,
                                );
                            }
                            let mut document = match task.recv::<InitialDocument>().await {
                                Ok(document) => document,
                                Err(error) if error.is_unavailable() => {
                                    break 'request (
                                        json!({"status":"out_of_order","error":"write_section is unavailable at this point in the choreography"}),
                                        8,
                                    );
                                }
                                Err(error) => return Err(error.into()),
                            };
                            if ROLE == WRITER_A {
                                task.send::<WriteA>(&arguments).await?;
                            } else {
                                task.send::<WriteB>(&arguments).await?;
                            }
                            document.send::<WriteOnDocument>(&arguments).await?;
                            let result = imp::decode(document.recv::<Written>().await?)?;
                            task.send::<InitialDocument>(document).await?;
                            (result, 0)
                        }
                        b"service/update_references" => {
                            let arguments = match ReportArguments::<4>::from_close(&file) {
                                Ok(arguments) => arguments,
                                Err(error) => {
                                    break 'request (
                                        json!({"status":"invalid_arguments","error":error.to_string()}),
                                        28,
                                    );
                                }
                            };
                            let value: Value = serde_json::from_slice(&file.bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if value["section_name"] != assigned {
                                break 'request (
                                    json!({"status":"invalid_arguments","error":"initial references belong to your assigned chapter"}),
                                    28,
                                );
                            }
                            let result = if ROLE == WRITER_A {
                                match task.send::<ReferencesA>(&arguments).await {
                                    Ok(()) => (),
                                    Err(error) if error.is_unavailable() => {
                                        break 'request (
                                            json!({"status":"out_of_order","error":"update_references is unavailable at this point in the choreography"}),
                                            8,
                                        );
                                    }
                                    Err(error) => return Err(error.into()),
                                }
                                imp::decode(task.recv::<ReferencedA>().await?)?
                            } else {
                                match task.send::<ReferencesB>(&arguments).await {
                                    Ok(()) => (),
                                    Err(error) if error.is_unavailable() => {
                                        break 'request (
                                            json!({"status":"out_of_order","error":"update_references is unavailable at this point in the choreography"}),
                                            8,
                                        );
                                    }
                                    Err(error) => return Err(error.into()),
                                }
                                imp::decode(task.recv::<ReferencedB>().await?)?
                            };
                            (result, 0)
                        }
                        b"service/revise_section" => {
                            let arguments = match ReportArguments::<3>::from_close(&file) {
                                Ok(arguments) => arguments,
                                Err(error) => {
                                    break 'request (
                                        json!({"status":"invalid_arguments","error":error.to_string()}),
                                        28,
                                    );
                                }
                            };
                            let value: Value = serde_json::from_slice(&file.bytes)
                                .map_err(|error| ShellError::message(error.to_string()))?;
                            if let Err(error) = imp::chapter(&value) {
                                break 'request (
                                    json!({"status":"invalid_arguments","error":error.to_string()}),
                                    28,
                                );
                            }
                            let mut document = match task.recv::<RevisionDocument>().await {
                                Ok(document) => document,
                                Err(error) if error.is_unavailable() => {
                                    break 'request (
                                        json!({"status":"out_of_order","error":"revise_section is unavailable at this point in the choreography"}),
                                        8,
                                    );
                                }
                                Err(error) => return Err(error.into()),
                            };
                            if ROLE == WRITER_A {
                                task.send::<ReviseA>(&arguments).await?;
                            } else {
                                task.send::<ReviseB>(&arguments).await?;
                            }
                            document.send::<ReviseOnDocument>(&arguments).await?;
                            let result = imp::decode(document.recv::<Revised>().await?)?;
                            task.send::<RevisionDocument>(document).await?;
                            (result, 0)
                        }
                        _ => (
                            json!({"status":"invalid_arguments","error":"unknown writer service or invalid feedback arguments"}),
                            28,
                        ),
                    }
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
                pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, ROLE>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the actual document has completed"}),
                )?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
