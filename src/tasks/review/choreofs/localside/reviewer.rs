//! Each actual review owns its readiness and result; only rejected attempts repeat.
use super::super::{
    super::imp::{ReviewInput, decide},
    global::*,
};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, runtime::resolver::DecisionArm};

#[hibana_runtime::scope]
pub async fn run_a<D: Domain>(
    task: &mut Endpoint<REVIEW_A>,
    (domain, actual): (&D, &ReviewInput),
) -> ShellResult<()> {
    loop {
        let (ready, arguments) = async {
            let [mut implementation, mut request] = task.par()?;
            async { Ok(implementation.recv::<Ready<TARGET_A>>().await?.to_vec()) };
            async { Ok(request.recv::<FileReview<TARGET_A>>().await?.into_value()) };
        }
        .await?;
        let implemented: serde_json::Value =
            serde_json::from_slice(&ready).map_err(|e| ShellError::message(e.to_string()))?;
        if implemented["success"] != true {
            return Err(ShellError::message(
                "review lacks a successful implementation",
            ));
        }
        let result = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
        actual.record(result);
        match decide(actual).map_err(|_| ShellError::message("review outcome lacks success"))? {
            DecisionArm::Left => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                task.send::<ApprovedA>(&reply).await?;
                task.send::<FileReviewResult<TARGET_A>>(&bytes.as_slice())
                    .await?;
                task.send::<RepoExit>(&0).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                task.send::<RejectedA>(&reply).await?;
                task.send::<FileReviewResult<TARGET_A>>(&bytes.as_slice())
                    .await?;
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn run_b<D: Domain>(
    task: &mut Endpoint<REVIEW_B>,
    (domain, actual): (&D, &ReviewInput),
) -> ShellResult<()> {
    loop {
        let (ready, arguments) = async {
            let [mut implementation, mut request] = task.par()?;
            async { Ok(implementation.recv::<Ready<TARGET_B>>().await?.to_vec()) };
            async { Ok(request.recv::<FileReview<TARGET_B>>().await?.into_value()) };
        }
        .await?;
        let implemented: serde_json::Value =
            serde_json::from_slice(&ready).map_err(|e| ShellError::message(e.to_string()))?;
        if implemented["success"] != true {
            return Err(ShellError::message(
                "review lacks a successful implementation",
            ));
        }
        let result = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
        actual.record(result);
        match decide(actual).map_err(|_| ShellError::message("review outcome lacks success"))? {
            DecisionArm::Left => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                task.send::<ApprovedB>(&reply).await?;
                task.send::<FileReviewResult<TARGET_B>>(&bytes.as_slice())
                    .await?;
                return Ok(());
            }
            DecisionArm::Right => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                task.send::<RejectedB>(&reply).await?;
                task.send::<FileReviewResult<TARGET_B>>(&bytes.as_slice())
                    .await?;
            }
        }
    }
}
