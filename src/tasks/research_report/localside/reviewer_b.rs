//! Actual review requests, backend results and projected result branches.
use super::super::global::*;
use super::super::imp::ReviewInput;
use crate::shell::imp::review_reply::ReviewReply;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use hibana::runtime::resolver::DecisionArm;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REVIEW_B>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
    endpoint.recv::<ReadyA>().await?;
    endpoint.recv::<ReadyB>().await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            171 | 177 => {
                let arguments = match branch.label() {
                    171 => branch.recv::<ReviewB>().await?,
                    177 => branch.recv::<ReReviewB>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["section_name"] != SECTION_B {
                    return Err(ShellError::message("review target differs"));
                }
                let result = domain
                    .execute("EDITOR", "review_sections", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
                match super::super::imp::decide(review)
                    .map_err(|_| ShellError::message("review result has no success field"))?
                {
                    DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;

                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                    }
                }
            }
            81 => {
                branch.recv::<FirstB>().await?;
            }
            83 => {
                branch.recv::<BDuringARevision>().await?;
            }
            84 => {
                branch.recv::<BOnly>().await?;
            }
            86 => {
                branch.recv::<AgainBBeforeA>().await?;
            }
            89 => {
                branch.recv::<AgainBAfterA>().await?;
            }
            93 => {
                branch.recv::<AgainBDuringARevision>().await?;
            }
            94 => {
                branch.recv::<AgainBBeforeAReview>().await?;
            }
            91 => {
                branch.recv::<ReadyB>().await?;
            }
            30 => {
                branch.recv::<ApprovedA>().await?;
                break;
            }
            31 => {
                branch.recv::<RejectedA>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-B operation {label}"
                )));
            }
        }
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            171 | 177 => {
                let arguments = match branch.label() {
                    171 => branch.recv::<ReviewB>().await?,
                    177 => branch.recv::<ReReviewB>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["section_name"] != SECTION_B {
                    return Err(ShellError::message("review target differs"));
                }
                let result = domain
                    .execute("EDITOR", "review_sections", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
                match super::super::imp::decide(review)
                    .map_err(|_| ShellError::message("review result has no success field"))?
                {
                    DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;

                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                    }
                }
            }
            81 => {
                branch.recv::<FirstB>().await?;
            }
            83 => {
                branch.recv::<BDuringARevision>().await?;
            }
            84 => {
                branch.recv::<BOnly>().await?;
            }
            86 => {
                branch.recv::<AgainBBeforeA>().await?;
            }
            89 => {
                branch.recv::<AgainBAfterA>().await?;
            }
            93 => {
                branch.recv::<AgainBDuringARevision>().await?;
            }
            94 => {
                branch.recv::<AgainBBeforeAReview>().await?;
            }
            91 => {
                branch.recv::<ReadyB>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-B operation {label}"
                )));
            }
        }
    }
}
