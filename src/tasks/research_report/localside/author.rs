//! Typed file requests and actual resource Endpoints, with no shell dispatcher.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::report_arguments::ReportArguments};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(endpoint: &mut Endpoint<ROLE>) -> ShellResult<()> {
    if ROLE != A_WASI && ROLE != B_WASI {
        return Err(ShellError::message("this report has two authors"));
    }
    let mut document = endpoint.recv::<DocumentLoan>().await?;
    let admitted = if ROLE == A_WASI {
        endpoint.recv::<WriteArgumentsA>().await?.into_value()
    } else {
        endpoint.recv::<WriteArgumentsB>().await?.into_value()
    };
    let arguments = ReportArguments::<2>::from_bytes(&imp::encode(&admitted)?)?;
    document.send::<WriteOnDocument>(&arguments).await?;
    let result = document.recv::<DocumentResult>().await?.to_vec();
    async {
        let [mut response, mut returned] = endpoint.par()?;
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
    let mut database = endpoint.recv::<DatabaseLoan>().await?;
    let admitted = if ROLE == A_WASI {
        endpoint.recv::<ReferenceArgumentsA>().await?.into_value()
    } else {
        endpoint.recv::<ReferenceArgumentsB>().await?.into_value()
    };
    let arguments = ReportArguments::<4>::from_bytes(&imp::encode(&admitted)?)?;
    database.send::<ReferencesOnDatabase>(&arguments).await?;
    let result = database.recv::<DatabaseResult>().await?.to_vec();
    async {
        let [mut response, mut returned] = endpoint.par()?;
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
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
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
                if ROLE == A_WASI {
                    endpoint.send::<FeedbackA>(&bytes.as_slice()).await?;
                }
            }
            RejectedA::LOGICAL_LABEL => {
                let bytes = branch.recv::<RejectedA>().await?.to_vec();
                if ROLE == A_WASI {
                    endpoint.send::<FeedbackA>(&bytes.as_slice()).await?;
                }
            }
            ApprovedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<ApprovedB>().await?.to_vec();
                if ROLE == B_WASI {
                    endpoint.send::<FeedbackB>(&bytes.as_slice()).await?;
                }
            }
            RejectedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<RejectedB>().await?.to_vec();
                if ROLE == B_WASI {
                    endpoint.send::<FeedbackB>(&bytes.as_slice()).await?;
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
