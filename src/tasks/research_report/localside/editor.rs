//! The editor receives actual file requests and returns actual review results.
use super::super::global::*;
use crate::shell::global::Loan;
use crate::shell::imp::report_arguments::ReportArguments;
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, REVIEW_WASI>,
) -> ShellResult<(
    Endpoint<'run, DOCUMENT>,
    Endpoint<'run, REVIEW_A>,
    Endpoint<'run, REVIEW_B>,
)> {
    // These bytes are the received effect results used in final confirmation.
    // They do not select the next request or authorize a file operation.
    let mut reviewed_a = Vec::new();
    let mut reviewed_b = Vec::new();
    let arguments = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewA::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReviewA>().await?;
                endpoint.send::<ReviewA>(&arguments).await?;
                endpoint.send::<ReviewA>(&arguments).await?;
                let result = endpoint.offer().await?;
                reviewed_a = match result.label() {
                    ApprovedA::LOGICAL_LABEL => result.recv::<ApprovedA>().await?.to_vec(),
                    RejectedA::LOGICAL_LABEL => result.recv::<RejectedA>().await?.to_vec(),
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected section A result {label}"
                        )));
                    }
                };
                endpoint
                    .send::<ReviewResultA>(&reviewed_a.as_slice())
                    .await?;
            }
            ReviewB::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReviewB>().await?;
                endpoint.send::<ReviewB>(&arguments).await?;
                endpoint.send::<ReviewB>(&arguments).await?;
                let result = endpoint.offer().await?;
                reviewed_b = match result.label() {
                    ApprovedB::LOGICAL_LABEL => result.recv::<ApprovedB>().await?.to_vec(),
                    RejectedB::LOGICAL_LABEL => result.recv::<RejectedB>().await?.to_vec(),
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected section B result {label}"
                        )));
                    }
                };
                endpoint
                    .send::<ReviewResultB>(&reviewed_b.as_slice())
                    .await?;
            }
            Combine::LOGICAL_LABEL => {
                break branch.recv::<Combine>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected editor operation {label}"
                )));
            }
        }
    };
    let a = ReviewReply::<true>::from_bytes(&reviewed_a)?;
    let b = ReviewReply::<true>::from_bytes(&reviewed_b)?;
    endpoint.send::<Combine>(&arguments).await?;
    endpoint.send::<ConfirmApprovedA>(&a).await?;
    endpoint.send::<ConfirmApprovedB>(&b).await?;
    let (review_a, review_b) = async {
        let [mut a, mut b] = endpoint.par()?;
        async { a.recv::<Loan<REVIEW_A>>().await };
        async { b.recv::<Loan<REVIEW_B>>().await };
    }
    .await?;
    let mut document = endpoint.recv::<DocumentLoan>().await?;
    let request = ReportArguments::<6>::from_bytes(arguments.as_bytes())?;
    document.send::<CombineOnDocument>(&request).await?;
    let result = document.recv::<DocumentResult>().await?.to_vec();
    async {
        let [mut reply, mut a, mut b] = endpoint.par()?;
        async {
            reply.send::<CombinedReply>(&result.as_slice()).await?;
        };
        async {
            a.send::<CombinedA>(&result.as_slice()).await?;
        };
        async {
            b.send::<CombinedB>(&result.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok((document, review_a, review_b))
}
