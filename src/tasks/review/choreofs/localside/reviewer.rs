//! Each actual review owns its readiness and result; only rejected attempts repeat.
use super::super::{
    super::imp::{ReviewInput, decide},
    global::*,
};
use crate::shell::imp::service::ReceivedEffects;
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply};
use hibana::{Endpoint, runtime::resolver::DecisionArm};

#[hibana_runtime::scope]
pub async fn run_a<'run>(
    owner: &mut Endpoint<'run, REVIEW_OWNER_A>,
    mut task: Endpoint<'run, REVIEW_A>,
    (effects, actual): (&impl ReceivedEffects, &ReviewInput),
) -> ShellResult<()> {
    loop {
        let result = async {
            let [mut implementation, mut request] = task.par()?;
            let (ready, arguments) = async {
                async { Ok(implementation.recv::<Ready<TARGET_A>>().await?.to_vec()) };
                async { Ok(request.recv::<FileReview<TARGET_A>>().await?) };
            }
            .await?;
            let implemented: serde_json::Value =
                serde_json::from_slice(&ready).map_err(|e| ShellError::message(e.to_string()))?;
            if implemented["success"] != true {
                return Err(ShellError::message(
                    "review lacks a successful implementation",
                ));
            }
            effects.apply(arguments).await
        }
        .await?;
        let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
        actual.record(result);
        match decide(actual).map_err(|_| ShellError::message("review outcome lacks success"))? {
            DecisionArm::Left => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                owner.send::<ApprovedLoanA>(task).await?;
                owner.send::<ApprovedA>(&reply).await?;
                owner.send::<RepoExit>(&0).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                owner.send::<RejectedLoanA>(task).await?;
                owner.send::<RejectedA>(&reply).await?;
                task = owner.recv::<RejectedLoanA>().await?;
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn run_b<'run>(
    owner: &mut Endpoint<'run, REVIEW_OWNER_B>,
    mut task: Endpoint<'run, REVIEW_B>,
    (effects, actual): (&impl ReceivedEffects, &ReviewInput),
) -> ShellResult<()> {
    loop {
        let result = async {
            let [mut implementation, mut request] = task.par()?;
            let (ready, arguments) = async {
                async { Ok(implementation.recv::<Ready<TARGET_B>>().await?.to_vec()) };
                async { Ok(request.recv::<FileReview<TARGET_B>>().await?) };
            }
            .await?;
            let implemented: serde_json::Value =
                serde_json::from_slice(&ready).map_err(|e| ShellError::message(e.to_string()))?;
            if implemented["success"] != true {
                return Err(ShellError::message(
                    "review lacks a successful implementation",
                ));
            }
            effects.apply(arguments).await
        }
        .await?;
        let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
        actual.record(result);
        match decide(actual).map_err(|_| ShellError::message("review outcome lacks success"))? {
            DecisionArm::Left => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                owner.send::<ApprovedLoanB>(task).await?;
                owner.send::<ApprovedB>(&reply).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                owner.send::<RejectedLoanB>(task).await?;
                owner.send::<RejectedB>(&reply).await?;
                task = owner.recv::<RejectedLoanB>().await?;
            }
        }
    }
}
