//! The guest chooses commands; its task Endpoint admits business effects.
use super::super::global::*;
use crate::shell::imp::repository_arguments::RepositoryArguments;
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFileMsg, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<ROLE>,
) -> ShellResult<i32> {
    let [task, mut private] = endpoint.par()?;
    let mut task_endpoint = Some(task);
    let mut fuel_id = 0u16;
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;
                    let completion = if request.0.fd() == SERVICE_FD {
                        let feature = if ROLE == A_WASI { FEATURE_A } else { FEATURE_B };
                        let assigned_branch = if ROLE == A_WASI {
                            MERGE_BRANCH_A
                        } else {
                            MERGE_BRANCH_B
                        };
                        let arguments = match closed.path.as_slice() {
                            b"service/implement_feature" => {
                                if closed.completion.0.errno() != 0 {
                                    Err(ShellError::message("implementation file close failed"))
                                } else {
                                    RepositoryArguments::<1>::from_bytes(&closed.bytes)
                                        .map(RepositoryArguments::into_value)
                                }
                            }
                            .and_then(|value| {
                                if value["feature_name"] == feature {
                                    Ok(value)
                                } else {
                                    Err(ShellError::message(
                                        "feature differs from this participant",
                                    ))
                                }
                            }),
                            b"service/merge_code" => closed
                                .arguments(b"service/merge_code", "branch")
                                .and_then(|value| {
                                    if value["branch"] == assigned_branch {
                                        Ok(value)
                                    } else {
                                        Err(ShellError::message(
                                            "branch differs from this participant",
                                        ))
                                    }
                                }),
                            _ => Err(ShellError::message("unknown developer service")),
                        };
                        let (reply, errno) = match arguments {
                            Err(error) => (
                                json!({"status":"invalid_arguments","error":error.to_string()}),
                                28,
                            ),
                            Ok(_) if task_endpoint.is_none() => (
                                json!({"status":"out_of_order","error":"business fragment has completed"}),
                                8,
                            ),
                            Ok(arguments) => {
                                let arguments = serde_json::to_vec(&arguments)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                let task =
                                    task_endpoint.as_mut().expect("active business fragment");
                                loop {
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
                                            branch.recv::<BAfterAMerge>().await?;
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
                                            branch.recv::<AAfterBMerge>().await?;
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
                                        30 if ROLE == B_WASI => {
                                            branch.recv::<ApprovedA>().await?;
                                        }
                                        31 if ROLE == B_WASI => {
                                            branch.recv::<RejectedA>().await?;
                                        }
                                        40 if ROLE == A_WASI => {
                                            branch.recv::<ApprovedB>().await?;
                                        }
                                        41 if ROLE == A_WASI => {
                                            branch.recv::<RejectedB>().await?;
                                        }
                                        69 | 76 | 169 | 176
                                            if closed.path == b"service/implement_feature" =>
                                        {
                                            let arguments =
                                                RepositoryArguments::<1>::from_bytes(&arguments)?;
                                            match branch.label() {
                                                69 => branch.send::<ImplementA>(&arguments).await?,
                                                76 => branch.send::<ReviseA>(&arguments).await?,
                                                169 => {
                                                    branch.send::<ImplementB>(&arguments).await?
                                                }
                                                176 => branch.send::<ReviseB>(&arguments).await?,
                                                _ => unreachable!(),
                                            };
                                            let mut repository = task.recv::<RepoLoan>().await?;
                                            repository.send::<ImplementOnRepo>(&arguments).await?;
                                            let bytes =
                                                repository.recv::<RepoResult>().await?.to_vec();
                                            task.send::<RepoLoan>(repository).await?;
                                            if ROLE == A_WASI {
                                                task.send::<ReadyA>(&bytes.as_slice()).await?;
                                            } else {
                                                task.send::<ReadyB>(&bytes.as_slice()).await?;
                                            }
                                            let mut result: Value = serde_json::from_slice(&bytes)
                                                .map_err(|e| ShellError::message(e.to_string()))?;
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
                                                        branch.recv::<BAfterAMerge>().await?;
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
                                                        branch.recv::<AAfterBMerge>().await?;
                                                    }
                                                    89 => {
                                                        branch.recv::<AgainBAfterA>().await?;
                                                    }
                                                    92 => {
                                                        branch
                                                            .recv::<AgainADuringBRevision>()
                                                            .await?;
                                                    }
                                                    93 => {
                                                        branch
                                                            .recv::<AgainBDuringARevision>()
                                                            .await?;
                                                    }
                                                    94 => {
                                                        branch
                                                            .recv::<AgainBBeforeAReview>()
                                                            .await?;
                                                    }
                                                    95 => {
                                                        branch
                                                            .recv::<AgainABeforeBReview>()
                                                            .await?;
                                                    }
                                                    30 if ROLE == A_WASI => {
                                                        break branch
                                                            .recv::<ApprovedA>()
                                                            .await?
                                                            .to_vec();
                                                    }
                                                    31 if ROLE == A_WASI => {
                                                        break branch
                                                            .recv::<RejectedA>()
                                                            .await?
                                                            .to_vec();
                                                    }
                                                    40 if ROLE == B_WASI => {
                                                        break branch
                                                            .recv::<ApprovedB>()
                                                            .await?
                                                            .to_vec();
                                                    }
                                                    41 if ROLE == B_WASI => {
                                                        break branch
                                                            .recv::<RejectedB>()
                                                            .await?
                                                            .to_vec();
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
                                            result["coordination"] = json!({"code_review":serde_json::from_slice::<Value>(&feedback).map_err(|e| ShellError::message(e.to_string()))?});
                                            break (result, 0);
                                        }
                                        70 | 170 if closed.path == b"service/merge_code" => {
                                            let arguments =
                                                RepositoryArguments::<2>::from_bytes(&arguments)?;
                                            match branch.label() {
                                                70 => branch.send::<MergeA>(&arguments).await?,
                                                170 => branch.send::<MergeB>(&arguments).await?,
                                                _ => unreachable!(),
                                            };
                                            let mut repository = task.recv::<RepoLoan>().await?;
                                            repository.send::<MergeOnRepo>(&arguments).await?;
                                            let bytes =
                                                repository.recv::<RepoResult>().await?.to_vec();
                                            task.send::<RepoLoan>(repository).await?;
                                            let control = task.recv::<ReviewControl>().await?;
                                            task.send::<ReviewControl>(control).await?;
                                            drop(task_endpoint.take());
                                            break (
                                                serde_json::from_slice(&bytes).map_err(|e| {
                                                    ShellError::message(e.to_string())
                                                })?,
                                                0,
                                            );
                                        }
                                        _ => {
                                            break (
                                                json!({"status":"out_of_order","error":"operation is unavailable in the projected protocol"}),
                                                8,
                                            );
                                        }
                                    }
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                            request.0.fd(),
                            errno,
                        ))
                    } else {
                        closed.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
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
