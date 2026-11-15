//! The worker uses the real document; shell requests arrive as typed arguments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::report_arguments::ReportArguments};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(endpoint: &mut Endpoint<ROLE>) -> ShellResult<()> {
    if ROLE != WRITER_A && ROLE != WRITER_B {
        return Err(ShellError::message("this document has two writers"));
    }
    let branch = endpoint.offer().await?;
    let mut document = match branch.label() {
        InitialDocument::LOGICAL_LABEL => branch.recv::<InitialDocument>().await?,
        InitialDocumentB::LOGICAL_LABEL if ROLE == WRITER_B => {
            branch.recv::<InitialDocumentB>().await?
        }
        _ => return Err(ShellError::message("expected the actual initial document")),
    };
    let value = if ROLE == WRITER_A {
        endpoint.recv::<WriteArgumentsA>().await?.into_value()
    } else {
        endpoint.recv::<WriteArgumentsB>().await?.into_value()
    };
    let arguments = ReportArguments::<2>::from_bytes(&imp::encode(&value)?)?;
    document.send::<WriteOnDocument>(&arguments).await?;
    let result = document.recv::<Written>().await?.to_vec();
    async {
        let [mut response, mut handoff] = endpoint.par()?;
        async {
            if ROLE == WRITER_A {
                response.send::<WriteResultA>(&result.as_slice()).await?;
            } else {
                response.send::<WriteResultB>(&result.as_slice()).await?;
            }
            Ok::<(), ShellError>(())
        };
        async {
            handoff.send::<InitialDocument>(document).await?;
        };
        Ok(())
    }
    .await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            RevisionReviewA::LOGICAL_LABEL if ROLE == WRITER_A => {
                let review = branch.recv::<RevisionReviewA>().await?;
                endpoint.send::<FeedbackA>(&review.as_bytes()).await?;
                let mut document = endpoint.recv::<RevisionDocument>().await?;
                let value = endpoint.recv::<RevisionArgumentsA>().await?.into_value();
                let arguments = ReportArguments::<3>::from_bytes(&imp::encode(&value)?)?;
                document.send::<ReviseOnDocument>(&arguments).await?;
                let result = document.recv::<Revised>().await?.to_vec();
                async {
                    let [mut response, mut handoff] = endpoint.par()?;
                    async {
                        response.send::<RevisionResultA>(&result.as_slice()).await?;
                    };
                    async {
                        handoff.send::<RevisionDocument>(document).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            RevisionReviewB::LOGICAL_LABEL if ROLE == WRITER_B => {
                let review = branch.recv::<RevisionReviewB>().await?;
                endpoint.send::<FeedbackB>(&review.as_bytes()).await?;
                let mut document = endpoint.recv::<RevisionDocument>().await?;
                let value = endpoint.recv::<RevisionArgumentsB>().await?.into_value();
                let arguments = ReportArguments::<3>::from_bytes(&imp::encode(&value)?)?;
                document.send::<ReviseOnDocument>(&arguments).await?;
                let result = document.recv::<Revised>().await?.to_vec();
                async {
                    let [mut response, mut handoff] = endpoint.par()?;
                    async {
                        response.send::<RevisionResultB>(&result.as_slice()).await?;
                    };
                    async {
                        handoff.send::<RevisionDocument>(document).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            RevisionReviewAForB::LOGICAL_LABEL if ROLE == WRITER_B => {
                let review = branch.recv::<RevisionReviewAForB>().await?;
                endpoint.send::<FeedbackB>(&review.as_bytes()).await?;
            }
            RevisionReviewBForA::LOGICAL_LABEL if ROLE == WRITER_A => {
                let review = branch.recv::<RevisionReviewBForA>().await?;
                endpoint.send::<FeedbackA>(&review.as_bytes()).await?;
            }
            FrozenDocument::LOGICAL_LABEL if ROLE == WRITER_A => {
                let mut document = branch.recv::<FrozenDocument>().await?;
                let arguments = endpoint.recv::<CombineForward>().await?;
                document.send::<CombineOnDocument>(&arguments).await?;
                let result = document.recv::<Combined>().await?.to_vec();
                async {
                    let [mut editor, mut result_file, mut returned] = endpoint.par()?;
                    async {
                        editor.send::<CombinedForEditor>(&result.as_slice()).await?;
                    };
                    async {
                        result_file.send::<FeedbackA>(&result.as_slice()).await?;
                    };
                    async {
                        returned.send::<DocumentReturn>(document).await?;
                    };
                    Ok(())
                }
                .await?;
                return Ok(());
            }
            FinishedB::LOGICAL_LABEL if ROLE == WRITER_B => {
                let result = branch.recv::<FinishedB>().await?.to_vec();
                endpoint.send::<FeedbackB>(&result.as_slice()).await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected writer communication {label}"
                )));
            }
        }
    }
}
