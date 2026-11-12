//! Actual review requests, backend results and projected result branches.
use super::super::global::*;
use super::super::imp::ReviewInput;
use crate::shell::imp::review_reply::ReviewReply;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use hibana::runtime::resolver::DecisionArm;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REVIEW_A>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
    endpoint.recv::<ReadyA>().await?;
    endpoint.recv::<ReadyB>().await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            71 | 77 => {
                let arguments = match branch.label() {
                    71 => branch.recv::<ReviewA>().await?,
                    77 => branch.recv::<ReReviewA>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["section_name"] != SECTION_A {
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

                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                    }
                }
            }
            80 => {
                branch.recv::<FirstA>().await?;
            }
            82 => {
                branch.recv::<AgainABeforeB>().await?;
            }
            85 => {
                branch.recv::<AgainAAfterB>().await?;
            }
            87 => {
                branch.recv::<ADuringBRevision>().await?;
            }
            88 => {
                branch.recv::<AOnly>().await?;
            }
            92 => {
                branch.recv::<AgainADuringBRevision>().await?;
            }
            95 => {
                branch.recv::<AgainABeforeBReview>().await?;
            }
            90 => {
                branch.recv::<ReadyA>().await?;
            }
            40 => {
                branch.recv::<ApprovedB>().await?;
                break;
            }
            41 => {
                branch.recv::<RejectedB>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-A operation {label}"
                )));
            }
        }
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            71 | 77 => {
                let arguments = match branch.label() {
                    71 => branch.recv::<ReviewA>().await?,
                    77 => branch.recv::<ReReviewA>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["section_name"] != SECTION_A {
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

                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                    }
                }
            }
            80 => {
                branch.recv::<FirstA>().await?;
            }
            82 => {
                branch.recv::<AgainABeforeB>().await?;
            }
            85 => {
                branch.recv::<AgainAAfterB>().await?;
            }
            87 => {
                branch.recv::<ADuringBRevision>().await?;
            }
            88 => {
                branch.recv::<AOnly>().await?;
            }
            92 => {
                branch.recv::<AgainADuringBRevision>().await?;
            }
            95 => {
                branch.recv::<AgainABeforeBReview>().await?;
            }
            90 => {
                branch.recv::<ReadyA>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-A operation {label}"
                )));
            }
        }
    }
}
