//! Typed file requests and actual resource Endpoints, with no shell dispatcher.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::ReceivedEffects},
};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    endpoint: &mut Endpoint<ROLE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    if ROLE != A_WASI && ROLE != B_WASI {
        return Err(ShellError::message("this report has two authors"));
    }
    let result = if ROLE == A_WASI {
        effects.apply(endpoint.recv::<ResearchA>().await?).await?
    } else {
        effects.apply(endpoint.recv::<ResearchB>().await?).await?
    };
    let bytes = imp::encode(&result)?;
    endpoint.send::<ResearchResult>(&bytes.as_slice()).await?;
    async {
        let [mut writing, mut references] = endpoint.par()?;
        async {
            let mut document = if ROLE == A_WASI {
                let admitted = writing.recv::<WriteArgumentsA>().await?;
                let mut document = writing.recv::<DocumentLoan>().await?;
                document.send::<WriteOnDocumentA>(&admitted.into()).await?;
                document
            } else {
                let admitted = writing.recv::<WriteArgumentsB>().await?;
                let mut document = writing.recv::<DocumentLoan>().await?;
                document.send::<WriteOnDocumentB>(&admitted.into()).await?;
                document
            };
            let result = document.recv::<DocumentResult>().await?.to_vec();
            async {
                let [mut response, mut returned] = writing.par()?;
                async {
                    if ROLE == A_WASI {
                        response.send::<WriteResultA>(&result.as_slice()).await?;
                    } else {
                        response.send::<WriteResultB>(&result.as_slice()).await?;
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    returned.send::<DocumentLoan>(document).await?;
                };
                Ok(())
            }
            .await?;
            Ok::<(), ShellError>(())
        };
        async {
            let mut database = if ROLE == A_WASI {
                let admitted = references.recv::<ReferenceArgumentsA>().await?;
                let mut database = references.recv::<DatabaseLoan>().await?;
                database
                    .send::<ReferencesOnDatabaseA>(&admitted.into())
                    .await?;
                database
            } else {
                let admitted = references.recv::<ReferenceArgumentsB>().await?;
                let mut database = references.recv::<DatabaseLoan>().await?;
                database
                    .send::<ReferencesOnDatabaseB>(&admitted.into())
                    .await?;
                database
            };
            let result = database.recv::<DatabaseResult>().await?.to_vec();
            async {
                let [mut response, mut returned] = references.par()?;
                async {
                    if ROLE == A_WASI {
                        response
                            .send::<ReferenceResultA>(&result.as_slice())
                            .await?;
                    } else {
                        response
                            .send::<ReferenceResultB>(&result.as_slice())
                            .await?;
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    returned.send::<DatabaseLoan>(database).await?;
                };
                Ok(())
            }
            .await?;
            Ok::<(), ShellError>(())
        };
        Ok(())
    }
    .await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewA::LOGICAL_LABEL if ROLE == A_WASI => {
                let arguments = branch.recv::<ReviewA>().await?;
                endpoint.send::<ReviewRequestA>(&arguments.into()).await?;
            }
            ReviewB::LOGICAL_LABEL if ROLE == B_WASI => {
                let arguments = branch.recv::<ReviewB>().await?;
                endpoint.send::<ReviewRequestB>(&arguments.into()).await?;
            }
            DocumentLoan::LOGICAL_LABEL => {
                let mut document = branch.recv::<DocumentLoan>().await?;
                if ROLE == A_WASI {
                    let admitted = endpoint.recv::<RevisionArgumentsA>().await?;
                    document.send::<ReviseOnDocumentA>(&admitted.into()).await?;
                } else {
                    let admitted = endpoint.recv::<RevisionArgumentsB>().await?;
                    document.send::<ReviseOnDocumentB>(&admitted.into()).await?;
                }
                let result = document.recv::<DocumentResult>().await?.to_vec();
                async {
                    let [mut response, mut returned] = endpoint.par()?;
                    async {
                        if ROLE == A_WASI {
                            response.send::<RevisionResultA>(&result.as_slice()).await?;
                        } else {
                            response.send::<RevisionResultB>(&result.as_slice()).await?;
                        }
                        Ok::<(), ShellError>(())
                    };
                    async {
                        returned.send::<DocumentLoan>(document).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            ApprovedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<ApprovedA>().await?.to_vec();
                let receipt = ReviewReply::<true>::from_bytes(&bytes)?;
                if ROLE == A_WASI {
                    async {
                        let [mut feedback, mut confirmed] = endpoint.par()?;
                        async { feedback.send::<FeedbackA>(&bytes.as_slice()).await };
                        async { confirmed.send::<ApprovedA>(&receipt).await };
                        Ok(())
                    }
                    .await?;
                } else {
                    endpoint.send::<ApprovedA>(&receipt).await?;
                }
            }
            RejectedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<RejectedA>().await?.to_vec();
                let receipt = ReviewReply::<false>::from_bytes(&bytes)?;
                if ROLE == A_WASI {
                    async {
                        let [mut feedback, mut confirmed] = endpoint.par()?;
                        async { feedback.send::<FeedbackA>(&bytes.as_slice()).await };
                        async { confirmed.send::<RejectedA>(&receipt).await };
                        Ok(())
                    }
                    .await?;
                } else {
                    endpoint.send::<RejectedA>(&receipt).await?;
                }
            }
            ApprovedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<ApprovedB>().await?.to_vec();
                let receipt = ReviewReply::<true>::from_bytes(&bytes)?;
                if ROLE == B_WASI {
                    async {
                        let [mut feedback, mut confirmed] = endpoint.par()?;
                        async { feedback.send::<FeedbackB>(&bytes.as_slice()).await };
                        async { confirmed.send::<ApprovedB>(&receipt).await };
                        Ok(())
                    }
                    .await?;
                } else {
                    endpoint.send::<ApprovedB>(&receipt).await?;
                }
            }
            RejectedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<RejectedB>().await?.to_vec();
                let receipt = ReviewReply::<false>::from_bytes(&bytes)?;
                if ROLE == B_WASI {
                    async {
                        let [mut feedback, mut confirmed] = endpoint.par()?;
                        async { feedback.send::<FeedbackB>(&bytes.as_slice()).await };
                        async { confirmed.send::<RejectedB>(&receipt).await };
                        Ok(())
                    }
                    .await?;
                } else {
                    endpoint.send::<RejectedB>(&receipt).await?;
                }
            }
            CombinedA::LOGICAL_LABEL if ROLE == A_WASI => {
                branch.recv::<CombinedA>().await?;
                return Ok(());
            }
            CombinedB::LOGICAL_LABEL if ROLE == B_WASI => {
                branch.recv::<CombinedB>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected author operation {label}"
                )));
            }
        }
    }
}
