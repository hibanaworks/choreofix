//! Typed file requests and actual resource Endpoints, with no shell dispatcher.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{report_arguments::ReportArguments, review_reply::ReviewReply},
};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    endpoint: &mut Endpoint<ROLE>,
    domain: &impl crate::shell::imp::service::Domain,
) -> ShellResult<()> {
    if ROLE != A_WASI && ROLE != B_WASI {
        return Err(ShellError::message("this report has two authors"));
    }
    let (agent, arguments) = if ROLE == A_WASI {
        (
            "RESEARCHER_A",
            endpoint.recv::<ResearchA>().await?.into_value(),
        )
    } else {
        (
            "RESEARCHER_B",
            endpoint.recv::<ResearchB>().await?.into_value(),
        )
    };
    let result = domain
        .execute(agent, "research_topic", arguments)
        .await
        .map_err(ShellError::message)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<ResearchResult>(&bytes.as_slice()).await?;
    async {
        let [mut writing, mut references] = endpoint.par()?;
        async {
            let admitted = if ROLE == A_WASI {
                writing.recv::<WriteArgumentsA>().await?.into_value()
            } else {
                writing.recv::<WriteArgumentsB>().await?.into_value()
            };
            let mut document = writing.recv::<DocumentLoan>().await?;
            let arguments = ReportArguments::<2>::from_bytes(&imp::encode(&admitted)?)?;
            document.send::<WriteOnDocument>(&arguments).await?;
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
            let admitted = if ROLE == A_WASI {
                references.recv::<ReferenceArgumentsA>().await?.into_value()
            } else {
                references.recv::<ReferenceArgumentsB>().await?.into_value()
            };
            let mut database = references.recv::<DatabaseLoan>().await?;
            let arguments = ReportArguments::<4>::from_bytes(&imp::encode(&admitted)?)?;
            database.send::<ReferencesOnDatabase>(&arguments).await?;
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
                endpoint.send::<ReviewA>(&arguments).await?;
            }
            ReviewB::LOGICAL_LABEL if ROLE == B_WASI => {
                let arguments = branch.recv::<ReviewB>().await?;
                endpoint.send::<ReviewB>(&arguments).await?;
            }
            DocumentLoan::LOGICAL_LABEL => {
                let mut document = branch.recv::<DocumentLoan>().await?;
                let admitted = if ROLE == A_WASI {
                    endpoint.recv::<RevisionArgumentsA>().await?.into_value()
                } else {
                    endpoint.recv::<RevisionArgumentsB>().await?.into_value()
                };
                let arguments = ReportArguments::<3>::from_bytes(&imp::encode(&admitted)?)?;
                document.send::<ReviseOnDocument>(&arguments).await?;
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
