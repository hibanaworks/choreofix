//! The reviewer's actual request selects a projected branch; each independent
//! projected operation is driven concurrently where the global uses par.
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
pub async fn run(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<REVIEW_WASI>,
    selection: &super::super::imp::ReviewTarget,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut control = Some(task.recv::<ReviewControl>().await?);
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
                        let arguments = match closed.path.as_slice() {
                            b"service/code_review/user_authentication" => closed
                                .arguments(b"service/code_review/user_authentication", "pr_id")
                                .and_then(|value| {
                                    if value["pr_id"] == FEATURE_A {
                                        Ok((FEATURE_A, value))
                                    } else {
                                        Err(ShellError::message("review target differs from path"))
                                    }
                                }),
                            b"service/code_review/notification_feature" => closed
                                .arguments(b"service/code_review/notification_feature", "pr_id")
                                .and_then(|value| {
                                    if value["pr_id"] == FEATURE_B {
                                        Ok((FEATURE_B, value))
                                    } else {
                                        Err(ShellError::message("review target differs from path"))
                                    }
                                }),
                            _ => Err(ShellError::message("unknown review service")),
                        };
                        let (reply, errno) = match arguments {
                            Err(error) => (
                                json!({"status":"invalid_arguments","error":error.to_string()}),
                                28,
                            ),
                            Ok(_) if control.is_none() => (
                                json!({"status":"out_of_order","error":"review control has been transferred"}),
                                8,
                            ),
                            Ok((feature, value)) => {
                                let bytes = serde_json::to_vec(&value)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                let arguments = RepositoryArguments::<3>::from_bytes(&bytes)?;
                                selection.record(feature);
                                'review_request: {
                                    let review_control =
                                        control.as_mut().expect("owned review control");
                                    let branch = review_control.offer().await?;
                                    match branch.label() {
                                        80 if feature == FEATURE_A => {
                                            branch.send::<FirstA>(&arguments).await?;
                                            review_control.send::<FirstA>(&arguments).await?;
                                            review_control.send::<FirstA>(&arguments).await?;
                                            review_control.send::<FirstA>(&arguments).await?;
                                            task.recv::<ReadyA>().await?;
                                            let [mut review, mut other] = task.par()?;
                                            async {
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReviewA>(&arguments).await?;
                                                };
                                                async {
                                                    other.recv::<ReadyB>().await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        81 if feature == FEATURE_B => {
                                            branch.send::<FirstB>(&arguments).await?;
                                            review_control.send::<FirstB>(&arguments).await?;
                                            review_control.send::<FirstB>(&arguments).await?;
                                            review_control.send::<FirstB>(&arguments).await?;
                                            task.recv::<ReadyB>().await?;
                                            let [mut review, mut other] = task.par()?;
                                            async {
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReviewB>(&arguments).await?;
                                                };
                                                async {
                                                    other.recv::<ReadyA>().await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        82 if feature == FEATURE_A => {
                                            branch.send::<AgainABeforeB>(&arguments).await?;
                                            review_control
                                                .send::<AgainABeforeB>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainABeforeB>(&arguments)
                                                .await?;
                                            task.recv::<ReadyA>().await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        83 if feature == FEATURE_B => {
                                            branch.send::<BDuringARevision>(&arguments).await?;
                                            review_control
                                                .send::<BDuringARevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<BDuringARevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<BDuringARevision>(&arguments)
                                                .await?;
                                            let [mut other, mut review] = task.par()?;
                                            async {
                                                async {
                                                    other.recv::<ReadyA>().await?;
                                                };
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReviewB>(&arguments).await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        84 if feature == FEATURE_B => {
                                            branch.send::<BAfterAMerge>(&arguments).await?;
                                            review_control.send::<BAfterAMerge>(&arguments).await?;
                                            review_control.send::<BAfterAMerge>(&arguments).await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReviewB>(&arguments).await?;
                                        }
                                        85 if feature == FEATURE_A => {
                                            branch.send::<AgainAAfterB>(&arguments).await?;
                                            review_control.send::<AgainAAfterB>(&arguments).await?;
                                            review_control.send::<AgainAAfterB>(&arguments).await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        86 if feature == FEATURE_B => {
                                            branch.send::<AgainBBeforeA>(&arguments).await?;
                                            review_control
                                                .send::<AgainBBeforeA>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainBBeforeA>(&arguments)
                                                .await?;
                                            task.recv::<ReadyB>().await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        87 if feature == FEATURE_A => {
                                            branch.send::<ADuringBRevision>(&arguments).await?;
                                            review_control
                                                .send::<ADuringBRevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<ADuringBRevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<ADuringBRevision>(&arguments)
                                                .await?;
                                            let [mut other, mut review] = task.par()?;
                                            async {
                                                async {
                                                    other.recv::<ReadyB>().await?;
                                                };
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReviewA>(&arguments).await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        88 if feature == FEATURE_A => {
                                            branch.send::<AAfterBMerge>(&arguments).await?;
                                            review_control.send::<AAfterBMerge>(&arguments).await?;
                                            review_control.send::<AAfterBMerge>(&arguments).await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReviewA>(&arguments).await?;
                                        }
                                        89 if feature == FEATURE_B => {
                                            branch.send::<AgainBAfterA>(&arguments).await?;
                                            review_control.send::<AgainBAfterA>(&arguments).await?;
                                            review_control.send::<AgainBAfterA>(&arguments).await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        92 if feature == FEATURE_A => {
                                            branch
                                                .send::<AgainADuringBRevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainADuringBRevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainADuringBRevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainADuringBRevision>(&arguments)
                                                .await?;
                                            let [mut other, mut review] = task.par()?;
                                            async {
                                                async {
                                                    other.recv::<ReadyB>().await?;
                                                };
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReReviewA>(&arguments).await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        93 if feature == FEATURE_B => {
                                            branch
                                                .send::<AgainBDuringARevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainBDuringARevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainBDuringARevision>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainBDuringARevision>(&arguments)
                                                .await?;
                                            let [mut other, mut review] = task.par()?;
                                            async {
                                                async {
                                                    other.recv::<ReadyA>().await?;
                                                };
                                                async {
                                                    review
                                                        .send::<ReviewControl>(
                                                            control
                                                                .take()
                                                                .expect("owned review control"),
                                                        )
                                                        .await?;
                                                    review.send::<ReReviewB>(&arguments).await?;
                                                };
                                                Ok(())
                                            }
                                            .await?;
                                        }
                                        94 if feature == FEATURE_B => {
                                            branch.send::<AgainBBeforeAReview>(&arguments).await?;
                                            review_control
                                                .send::<AgainBBeforeAReview>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainBBeforeAReview>(&arguments)
                                                .await?;
                                            task.recv::<ReadyB>().await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        95 if feature == FEATURE_A => {
                                            branch.send::<AgainABeforeBReview>(&arguments).await?;
                                            review_control
                                                .send::<AgainABeforeBReview>(&arguments)
                                                .await?;
                                            review_control
                                                .send::<AgainABeforeBReview>(&arguments)
                                                .await?;
                                            task.recv::<ReadyA>().await?;
                                            task.send::<ReviewControl>(
                                                control.take().expect("owned review control"),
                                            )
                                            .await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        _ => {
                                            break 'review_request (
                                                json!({"status":"out_of_order","error":"review is unavailable in the projected protocol"}),
                                                8,
                                            );
                                        }
                                    }
                                    let feedback = task.offer().await?;
                                    let bytes = match feedback.label() {
                                        30 => feedback.recv::<ApprovedA>().await?.to_vec(),
                                        31 => feedback.recv::<RejectedA>().await?.to_vec(),
                                        40 => feedback.recv::<ApprovedB>().await?.to_vec(),
                                        41 => feedback.recv::<RejectedB>().await?.to_vec(),
                                        label => {
                                            return Err(ShellError::message(format!(
                                                "unexpected review reply {label}"
                                            )));
                                        }
                                    };
                                    let returned = task.offer().await?;
                                    match returned.label() {
                                        11 => {
                                            control = Some(returned.recv::<ReviewControl>().await?);
                                        }
                                        235 => {
                                            let finished =
                                                returned.recv::<ReviewsComplete>().await?;
                                            task.send::<ReviewControl>(finished).await?;
                                        }
                                        label => {
                                            return Err(ShellError::message(format!(
                                                "unexpected review control return {label}"
                                            )));
                                        }
                                    }
                                    break 'review_request (
                                        serde_json::from_slice::<Value>(&bytes)
                                            .map_err(|e| ShellError::message(e.to_string()))?,
                                        0,
                                    );
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
                if control.is_some() {
                    return Err(ShellError::message(
                        "guest exited while it still owns review control",
                    ));
                }
                task.send::<RepoExit>(&status).await?;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
