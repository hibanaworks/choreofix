//! The guest chooses commands; its task Endpoint admits business effects.
use super::super::global::*;
use crate::shell::imp::report_arguments::ReportArguments;
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiImportCompletion},
    protocol,
};
use serde_json::{Value, json};

pub async fn run<'guest, 'r, const ROLE: u8>(
    guest: &mut HibanaWasiGuest<'guest>,
    endpoint: &mut hibana::Endpoint<'r, ROLE>,
) -> ShellResult<i32> {
    let [task, mut private] = endpoint.par()?;
    let mut task_endpoint = Some(task);
    loop {
        match guest
            .next_file::<crate::shell::imp::file_wire::ClosedFile, ROLE>(&mut private, SERVICE_FD)
            .await?
        {
            hibana_wasip1_runtime::FileBoundary::Closed {
                pending,
                file: closed,
            } => {
                let completion = {
                    let feature = if ROLE == A_WASI { SECTION_A } else { SECTION_B };
                    let arguments = match closed.path.as_slice() {
                        b"service/research_topic" => ReportArguments::<1>::from_close(&closed)
                            .map(ReportArguments::into_value)
                            .and_then(|value| {
                                let topic = if ROLE == A_WASI {
                                    "subtopic_a"
                                } else {
                                    "subtopic_b"
                                };
                                if value["topic"] == topic {
                                    Ok(value)
                                } else {
                                    Err(ShellError::message(
                                        "research topic differs from this participant",
                                    ))
                                }
                            }),
                        b"service/write_section" => ReportArguments::<2>::from_close(&closed)
                            .map(ReportArguments::into_value),
                        b"service/revise_section" => ReportArguments::<3>::from_close(&closed)
                            .map(ReportArguments::into_value),
                        b"service/update_references" => ReportArguments::<4>::from_close(&closed)
                            .map(ReportArguments::into_value),
                        _ => Err(ShellError::message("unknown researcher service")),
                    }
                    .and_then(|value| {
                        if closed.path == b"service/research_topic"
                            || value["section_name"] == feature
                        {
                            Ok(value)
                        } else {
                            Err(ShellError::message("section differs from this participant"))
                        }
                    });
                    let (reply, errno) = match arguments {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let arguments = serde_json::to_vec(&arguments)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            'request: {
                                let Some(task) = task_endpoint.as_mut() else {
                                    break 'request (
                                        json!({"status":"out_of_order","error":"this participant's business Endpoint has completed"}),
                                        8,
                                    );
                                };
                                let mut result = match closed.path.as_slice() {
                                    b"service/research_topic" => {
                                        let arguments =
                                            ReportArguments::<1>::from_bytes(&arguments)?;
                                        task.send::<Research>(&arguments).await?;
                                        let bytes = task.recv::<ResearchResult>().await?.to_vec();
                                        let result: Value = serde_json::from_slice(&bytes)
                                            .map_err(|error| {
                                                ShellError::message(error.to_string())
                                            })?;
                                        if result["success"] != true {
                                            return Err(ShellError::message(
                                                "research did not complete",
                                            ));
                                        }
                                        break 'request (result, 0);
                                    }
                                    b"service/write_section" => {
                                        let arguments =
                                            ReportArguments::<2>::from_bytes(&arguments)?;
                                        if ROLE == A_WASI {
                                            task.send::<WriteA>(&arguments).await?;
                                        } else {
                                            task.send::<WriteB>(&arguments).await?;
                                        }
                                        let mut document = task.recv::<DocumentLoan>().await?;
                                        document.send::<WriteOnDocument>(&arguments).await?;
                                        let bytes =
                                            document.recv::<DocumentResult>().await?.to_vec();
                                        task.send::<DocumentLoan>(document).await?;
                                        let result: Value = serde_json::from_slice(&bytes)
                                            .map_err(|error| {
                                                ShellError::message(error.to_string())
                                            })?;
                                        if result["success"] != true {
                                            return Err(ShellError::message(
                                                "section write did not complete",
                                            ));
                                        }
                                        break 'request (result, 0);
                                    }
                                    b"service/update_references" => {
                                        let arguments =
                                            ReportArguments::<4>::from_bytes(&arguments)?;
                                        if ROLE == A_WASI {
                                            task.send::<ReferencesA>(&arguments).await?;
                                        } else {
                                            task.send::<ReferencesB>(&arguments).await?;
                                        }
                                        let mut database = task.recv::<DatabaseLoan>().await?;
                                        database.send::<ReferencesOnDatabase>(&arguments).await?;
                                        let bytes =
                                            database.recv::<DatabaseResult>().await?.to_vec();
                                        task.send::<DatabaseLoan>(database).await?;
                                        let result: Value = serde_json::from_slice(&bytes)
                                            .map_err(|error| {
                                                ShellError::message(error.to_string())
                                            })?;
                                        if result["success"] != true {
                                            return Err(ShellError::message(
                                                "references did not complete",
                                            ));
                                        }
                                        if ROLE == A_WASI {
                                            task.send::<ReadyA>(&bytes.as_slice()).await?;
                                            task.send::<ReadyA>(&bytes.as_slice()).await?;
                                        } else {
                                            task.send::<ReadyB>(&bytes.as_slice()).await?;
                                            task.send::<ReadyB>(&bytes.as_slice()).await?;
                                        }
                                        result
                                    }
                                    _ => loop {
                                        let branch = task.offer().await?;
                                        match branch.label() {
                                            80 => {
                                                branch.recv::<FirstA>().await?;
                                                continue;
                                            }
                                            81 => {
                                                branch.recv::<FirstB>().await?;
                                                continue;
                                            }
                                            82 => {
                                                branch.recv::<AgainABeforeB>().await?;
                                                continue;
                                            }
                                            83 => {
                                                branch.recv::<BDuringARevision>().await?;
                                                continue;
                                            }
                                            84 => {
                                                branch.recv::<BOnly>().await?;
                                                continue;
                                            }
                                            85 => {
                                                branch.recv::<AgainAAfterB>().await?;
                                                continue;
                                            }
                                            86 => {
                                                branch.recv::<AgainBBeforeA>().await?;
                                                continue;
                                            }
                                            87 => {
                                                branch.recv::<ADuringBRevision>().await?;
                                                continue;
                                            }
                                            88 => {
                                                branch.recv::<AOnly>().await?;
                                                continue;
                                            }
                                            89 => {
                                                branch.recv::<AgainBAfterA>().await?;
                                                continue;
                                            }
                                            92 => {
                                                branch.recv::<AgainADuringBRevision>().await?;
                                                continue;
                                            }
                                            93 => {
                                                branch.recv::<AgainBDuringARevision>().await?;
                                                continue;
                                            }
                                            94 => {
                                                branch.recv::<AgainBBeforeAReview>().await?;
                                                continue;
                                            }
                                            95 => {
                                                branch.recv::<AgainABeforeBReview>().await?;
                                                continue;
                                            }
                                            30 if ROLE == B_WASI => {
                                                branch.recv::<ApprovedA>().await?;
                                                continue;
                                            }
                                            31 if ROLE == B_WASI => {
                                                branch.recv::<RejectedA>().await?;
                                                continue;
                                            }
                                            40 if ROLE == A_WASI => {
                                                branch.recv::<ApprovedB>().await?;
                                                continue;
                                            }
                                            41 if ROLE == A_WASI => {
                                                branch.recv::<RejectedB>().await?;
                                                continue;
                                            }
                                            76 | 176
                                                if closed.path == b"service/revise_section" =>
                                            {
                                                let arguments =
                                                    ReportArguments::<3>::from_bytes(&arguments)?;
                                                match branch.label() {
                                                    76 => {
                                                        branch.send::<ReviseA>(&arguments).await?
                                                    }
                                                    176 => {
                                                        branch.send::<ReviseB>(&arguments).await?
                                                    }
                                                    _ => unreachable!(),
                                                };
                                                let mut document =
                                                    task.recv::<DocumentLoan>().await?;
                                                document
                                                    .send::<ReviseOnDocument>(&arguments)
                                                    .await?;
                                                let bytes = document
                                                    .recv::<DocumentResult>()
                                                    .await?
                                                    .to_vec();
                                                task.send::<DocumentLoan>(document).await?;
                                                let result: Value = serde_json::from_slice(&bytes)
                                                    .map_err(|error| {
                                                        ShellError::message(error.to_string())
                                                    })?;
                                                if result["success"] != true {
                                                    return Err(ShellError::message(
                                                        "revision did not complete",
                                                    ));
                                                }
                                                if ROLE == A_WASI {
                                                    task.send::<ReadyA>(&bytes.as_slice()).await?;
                                                } else {
                                                    task.send::<ReadyB>(&bytes.as_slice()).await?;
                                                }
                                                break result;
                                            }
                                            _ => {
                                                break 'request (
                                                    json!({"status":"out_of_order","error":"operation is unavailable in the projected protocol"}),
                                                    8,
                                                );
                                            }
                                        }
                                    },
                                };
                                let feedback = loop {
                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        80 => {
                                            branch.recv::<FirstA>().await?;
                                        }
                                        81 => {
                                            branch.recv::<FirstB>().await?;
                                        }
                                        82 => {
                                            branch.recv::<AgainABeforeB>().await?;
                                        }
                                        83 => {
                                            branch.recv::<BDuringARevision>().await?;
                                        }
                                        84 => {
                                            branch.recv::<BOnly>().await?;
                                        }
                                        85 => {
                                            branch.recv::<AgainAAfterB>().await?;
                                        }
                                        86 => {
                                            branch.recv::<AgainBBeforeA>().await?;
                                        }
                                        87 => {
                                            branch.recv::<ADuringBRevision>().await?;
                                        }
                                        88 => {
                                            branch.recv::<AOnly>().await?;
                                        }
                                        89 => {
                                            branch.recv::<AgainBAfterA>().await?;
                                        }
                                        92 => {
                                            branch.recv::<AgainADuringBRevision>().await?;
                                        }
                                        93 => {
                                            branch.recv::<AgainBDuringARevision>().await?;
                                        }
                                        94 => {
                                            branch.recv::<AgainBBeforeAReview>().await?;
                                        }
                                        95 => {
                                            branch.recv::<AgainABeforeBReview>().await?;
                                        }
                                        30 if ROLE == A_WASI => {
                                            break branch.recv::<ApprovedA>().await?.to_vec();
                                        }
                                        31 if ROLE == A_WASI => {
                                            break branch.recv::<RejectedA>().await?.to_vec();
                                        }
                                        40 if ROLE == B_WASI => {
                                            break branch.recv::<ApprovedB>().await?.to_vec();
                                        }
                                        41 if ROLE == B_WASI => {
                                            break branch.recv::<RejectedB>().await?.to_vec();
                                        }
                                        30 => {
                                            branch.recv::<ApprovedA>().await?;
                                        }
                                        31 => {
                                            branch.recv::<RejectedA>().await?;
                                        }
                                        40 => {
                                            branch.recv::<ApprovedB>().await?;
                                        }
                                        41 => {
                                            branch.recv::<RejectedB>().await?;
                                        }
                                        label => {
                                            return Err(ShellError::message(format!(
                                                "unexpected review result {label}"
                                            )));
                                        }
                                    }
                                };
                                let feedback: Value = serde_json::from_slice(&feedback)
                                    .map_err(|error| ShellError::message(error.to_string()))?;
                                if feedback["success"] == true {
                                    // The author's last selected operation has committed.
                                    // Retire that actual fragment; private WASI I/O continues.
                                    drop(task_endpoint.take());
                                }
                                result["coordination"] = json!({"review_sections":feedback});
                                (result, 0)
                            }
                        }
                    };
                    let bytes = serde_json::to_vec(&reply)
                        .map_err(|e| ShellError::message(e.to_string()))?;
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
                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, errno))
                };
                pending.complete(WasiImportCompletion::FdClose(completion))?;
            }
            hibana_wasip1_runtime::FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                if task_endpoint.is_some() {
                    return Err(ShellError::message(
                        "guest exited with its business fragment still active",
                    ));
                }
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
