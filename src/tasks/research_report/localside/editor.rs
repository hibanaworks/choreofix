//! The reviewer's actual request selects a projected branch; each independent
//! projected operation is driven concurrently where the global uses par.
use super::super::global::*;
use crate::shell::imp::report_arguments::ReportArguments;
use crate::shell::imp::review_reply::ReviewReply;
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

#[hibana_runtime::scope]
pub async fn run(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<REVIEW_WASI>,
    selection: &super::super::imp::ReviewTarget,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<crate::shell::imp::file_wire::ClosedFile, REVIEW_WASI>(
                &mut private,
                SERVICE_FD,
            )
            .await?
        {
            hibana_wasip1_runtime::FileBoundary::Closed {
                pending,
                file: closed,
            } => {
                let completion = {
                    let (reply, errno) = if closed.path == b"service/combine_sections" {
                        match ReportArguments::<6>::from_close(&closed) {
                            Err(error) => (
                                json!({"status":"invalid_arguments","error":error.to_string()}),
                                28,
                            ),
                            Ok(arguments) => match selection.approvals() {
                                Err(error) => (
                                    json!({"status":"out_of_order","error":error.to_string()}),
                                    8,
                                ),
                                Ok((a, b)) => {
                                    let a = ReviewReply::<true>::from_bytes(&a)?;
                                    let b = ReviewReply::<true>::from_bytes(&b)?;
                                    task.send::<ConfirmApprovedA>(&a).await?;
                                    task.send::<ConfirmApprovedB>(&b).await?;
                                    task.send::<Combine>(&arguments).await?;
                                    let mut document = task.recv::<DocumentLoan>().await?;
                                    document.send::<CombineOnDocument>(&arguments).await?;
                                    let bytes = document.recv::<DocumentResult>().await?.to_vec();
                                    task.send::<DocumentLoan>(document).await?;
                                    (
                                        serde_json::from_slice::<Value>(&bytes).map_err(
                                            |error| ShellError::message(error.to_string()),
                                        )?,
                                        0,
                                    )
                                }
                            },
                        }
                    } else {
                        let arguments = ReportArguments::<5>::from_close(&closed)
                            .map(ReportArguments::into_value)
                            .and_then(|value| match value["section_name"].as_str() {
                                Some(SECTION_A) => Ok((SECTION_A, value)),
                                Some(SECTION_B) => Ok((SECTION_B, value)),
                                _ => Err(ShellError::message("unassigned review section")),
                            });
                        match arguments {
                            Err(error) => (
                                json!({"status":"invalid_arguments","error":error.to_string()}),
                                28,
                            ),
                            Ok((feature, value)) => {
                                let bytes = serde_json::to_vec(&value)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                let arguments = ReportArguments::<5>::from_bytes(&bytes)?;
                                selection.record(feature);
                                'review: {
                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        80 if feature == SECTION_A => {
                                            branch.send::<FirstA>(&arguments).await?;

                                            task.send::<FirstA>(&arguments).await?;
                                            task.send::<FirstA>(&arguments).await?;
                                            task.send::<ReviewA>(&arguments).await?;
                                        }
                                        81 if feature == SECTION_B => {
                                            branch.send::<FirstB>(&arguments).await?;

                                            task.send::<FirstB>(&arguments).await?;
                                            task.send::<FirstB>(&arguments).await?;
                                            task.send::<ReviewB>(&arguments).await?;
                                        }
                                        82 if feature == SECTION_A => {
                                            branch.send::<AgainABeforeB>(&arguments).await?;

                                            task.send::<AgainABeforeB>(&arguments).await?;
                                            task.send::<AgainABeforeB>(&arguments).await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        83 if feature == SECTION_B => {
                                            branch.send::<BDuringARevision>(&arguments).await?;

                                            task.send::<BDuringARevision>(&arguments).await?;
                                            task.send::<BDuringARevision>(&arguments).await?;
                                            task.send::<BDuringARevision>(&arguments).await?;
                                            task.send::<ReviewB>(&arguments).await?;
                                        }
                                        84 if feature == SECTION_B => {
                                            branch.send::<BOnly>(&arguments).await?;

                                            task.send::<BOnly>(&arguments).await?;
                                            task.send::<BOnly>(&arguments).await?;
                                            task.send::<ReviewB>(&arguments).await?;
                                        }
                                        85 if feature == SECTION_A => {
                                            branch.send::<AgainAAfterB>(&arguments).await?;

                                            task.send::<AgainAAfterB>(&arguments).await?;
                                            task.send::<AgainAAfterB>(&arguments).await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        86 if feature == SECTION_B => {
                                            branch.send::<AgainBBeforeA>(&arguments).await?;

                                            task.send::<AgainBBeforeA>(&arguments).await?;
                                            task.send::<AgainBBeforeA>(&arguments).await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        87 if feature == SECTION_A => {
                                            branch.send::<ADuringBRevision>(&arguments).await?;

                                            task.send::<ADuringBRevision>(&arguments).await?;
                                            task.send::<ADuringBRevision>(&arguments).await?;
                                            task.send::<ADuringBRevision>(&arguments).await?;
                                            task.send::<ReviewA>(&arguments).await?;
                                        }
                                        88 if feature == SECTION_A => {
                                            branch.send::<AOnly>(&arguments).await?;

                                            task.send::<AOnly>(&arguments).await?;
                                            task.send::<AOnly>(&arguments).await?;
                                            task.send::<ReviewA>(&arguments).await?;
                                        }
                                        89 if feature == SECTION_B => {
                                            branch.send::<AgainBAfterA>(&arguments).await?;

                                            task.send::<AgainBAfterA>(&arguments).await?;
                                            task.send::<AgainBAfterA>(&arguments).await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        92 if feature == SECTION_A => {
                                            branch
                                                .send::<AgainADuringBRevision>(&arguments)
                                                .await?;

                                            task.send::<AgainADuringBRevision>(&arguments).await?;
                                            task.send::<AgainADuringBRevision>(&arguments).await?;
                                            task.send::<AgainADuringBRevision>(&arguments).await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        93 if feature == SECTION_B => {
                                            branch
                                                .send::<AgainBDuringARevision>(&arguments)
                                                .await?;

                                            task.send::<AgainBDuringARevision>(&arguments).await?;
                                            task.send::<AgainBDuringARevision>(&arguments).await?;
                                            task.send::<AgainBDuringARevision>(&arguments).await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        94 if feature == SECTION_B => {
                                            branch.send::<AgainBBeforeAReview>(&arguments).await?;

                                            task.send::<AgainBBeforeAReview>(&arguments).await?;
                                            task.send::<AgainBBeforeAReview>(&arguments).await?;
                                            task.send::<ReReviewB>(&arguments).await?;
                                        }
                                        95 if feature == SECTION_A => {
                                            branch.send::<AgainABeforeBReview>(&arguments).await?;

                                            task.send::<AgainABeforeBReview>(&arguments).await?;
                                            task.send::<AgainABeforeBReview>(&arguments).await?;
                                            task.send::<ReReviewA>(&arguments).await?;
                                        }
                                        _ => {
                                            break 'review (
                                                json!({"status":"out_of_order","error":"review is unavailable in the projected protocol"}),
                                                8,
                                            );
                                        }
                                    }
                                    let feedback = task.offer().await?;
                                    let reviewed = match feedback.label() {
                                        30 | 31 => SECTION_A,
                                        40 | 41 => SECTION_B,
                                        _ => {
                                            return Err(ShellError::message(
                                                "unknown reviewed section",
                                            ));
                                        }
                                    };
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
                                    selection.received(reviewed, &bytes)?;
                                    (
                                        serde_json::from_slice::<Value>(&bytes)
                                            .map_err(|e| ShellError::message(e.to_string()))?,
                                        0,
                                    )
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
                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, errno))
                };
                pending.complete(WasiImportCompletion::FdClose(completion))?;
            }
            hibana_wasip1_runtime::FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                task.send::<DocumentExit>(&status).await?;
                task.send::<DatabaseExit>(&status).await?;
                {
                    let [mut a, mut b] = task.par()?;
                    async {
                        async {
                            a.send::<ResearchExitA>(&status).await?;
                        };
                        async {
                            b.send::<ResearchExitB>(&status).await?;
                        };
                        Ok(())
                    }
                    .await?;
                }
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
