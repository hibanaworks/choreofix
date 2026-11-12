//! Actual review requests, backend results and projected result branches.
use super::super::super::imp::ReviewInput;
use super::super::global::*;
use crate::shell::imp::review_reply::ReviewReply;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use hibana::runtime::resolver::DecisionArm;

pub async fn run<'r, D: Domain>(
    mut endpoint: Endpoint<'r, REVIEW_B>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            11 => {
                let control = branch.recv::<ReviewControl>().await?;
                let branch = endpoint.offer().await?;
                let arguments = match branch.label() {
                    171 => branch.recv::<ReviewB>().await?,
                    177 => branch.recv::<ReReviewB>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["pr_id"] != FEATURE_B {
                    return Err(ShellError::message("review target differs"));
                }
                let result = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
                match super::super::super::imp::decide(review)
                    .map_err(|_| ShellError::message("review result has no success field"))?
                {
                    DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;

                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ReviewControl>(control).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<ReviewControl>(control).await?;
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
                branch.recv::<BAfterAMerge>().await?;
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
            11 => {
                let control = branch.recv::<ReviewControl>().await?;
                let branch = endpoint.offer().await?;
                let arguments = match branch.label() {
                    171 => branch.recv::<ReviewB>().await?,
                    177 => branch.recv::<ReReviewB>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["pr_id"] != FEATURE_B {
                    return Err(ShellError::message("review target differs"));
                }
                let result = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
                match super::super::super::imp::decide(review)
                    .map_err(|_| ShellError::message("review result has no success field"))?
                {
                    DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;

                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ApprovedB>(&reply).await?;
                        endpoint.send::<ReviewsComplete>(control).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;

                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<RejectedB>(&reply).await?;
                        endpoint.send::<ReviewControl>(control).await?;
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
                branch.recv::<BAfterAMerge>().await?;
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
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-B operation {label}"
                )));
            }
        }
    }
}
