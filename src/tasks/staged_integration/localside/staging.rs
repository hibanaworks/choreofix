//! Each feature's staging participant owns the same resource until its positive result.
use super::super::{
    actual_outcome,
    arguments::{Arguments, FEATURES},
    global::*,
    imp,
};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::ReceivedEffects},
};
use core::cell::Cell;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run_a<D: ReceivedEffects>(
    owner: &mut Endpoint<STAGE_OWNER>,
    (effects, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut environment = owner.recv::<StageLoan>().await?;
    let mut validations = [Value::Null, Value::Null, Value::Null];
    loop {
        let inputs = imp::decode(owner.recv::<StagingReadyA>().await?)?;
        imp::staging_inputs(&inputs, 0)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyA>(&bytes.as_slice())
            .await?;
        let request = environment.recv::<DeployRequestA>().await?;
        let checked = Arguments::<5>::try_from(request.as_bytes())?;
        let deployment = effects.apply(request);
        environment.send::<CheckStageA>(&checked).await?;
        let arguments = checked.into_value();
        let inputs = imp::decode(environment.recv::<StageInputsA>().await?)?;
        imp::deploy_input(&arguments, &inputs, (0, &validations))?;
        let deployment = deployment.await?;
        imp::deployed(&deployment, &inputs, (0, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<InstalledA>(&bytes.as_slice()).await?;
        environment.send::<DeployedA>(&bytes.as_slice()).await?;
        let request = environment.recv::<ValidateRequestA>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::assigned(&arguments, "service", FEATURES[0])?;
        let validation = effects.apply(request).await?;
        imp::validated(&validation, &deployment, 0)?;
        let bytes = imp::encode(&validation)?;
        actual.set(Some(validation.clone()));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
            DecisionArm::Left => {
                environment
                    .send::<StagePassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultA>(&bytes.as_slice())
                    .await?;
                validations[0] = json!({"deployment":deployment,"validation":validation});
                let history = imp::encode(
                    &serde_json::to_value(&validations)
                        .map_err(|error| ShellError::message(error.to_string()))?,
                )?;
                owner.send::<StageHistory>(&history.as_slice()).await?;
                owner.send::<StageLoan>(environment).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                environment
                    .send::<StageFailedA>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultA>(&bytes.as_slice())
                    .await?;
                let request = environment.recv::<RollbackRequestA>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::rollback_input(&arguments, &deployment, &validation, 0)?;
                let rollback = effects.apply(request).await?;
                imp::rolled_back(&rollback, &deployment, &validation, 0)?;
                let bytes = imp::encode(&rollback)?;
                environment.send::<RolledBackA>(&bytes.as_slice()).await?;
                environment
                    .send::<RollbackReceiptA>(&bytes.as_slice())
                    .await?;
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn run_b<D: ReceivedEffects>(
    owner: &mut Endpoint<STAGE_OWNER_B>,
    (effects, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let history = imp::decode(owner.recv::<StageHistory>().await?)?;
    let mut validations: [Value; 3] =
        serde_json::from_value(history).map_err(|error| ShellError::message(error.to_string()))?;
    let mut environment = owner.recv::<StageLoan>().await?;
    loop {
        let inputs = imp::decode(owner.recv::<StagingReadyB>().await?)?;
        imp::staging_inputs(&inputs, 1)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyB>(&bytes.as_slice())
            .await?;
        let request = environment.recv::<DeployRequestB>().await?;
        let checked = Arguments::<13>::try_from(request.as_bytes())?;
        let deployment = effects.apply(request);
        environment.send::<CheckStageB>(&checked).await?;
        let arguments = checked.into_value();
        let inputs = imp::decode(environment.recv::<StageInputsB>().await?)?;
        imp::deploy_input(&arguments, &inputs, (1, &validations))?;
        let deployment = deployment.await?;
        imp::deployed(&deployment, &inputs, (1, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<InstalledB>(&bytes.as_slice()).await?;
        environment.send::<DeployedB>(&bytes.as_slice()).await?;
        let request = environment.recv::<ValidateRequestB>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::assigned(&arguments, "service", FEATURES[1])?;
        let validation = effects.apply(request).await?;
        imp::validated(&validation, &deployment, 1)?;
        let bytes = imp::encode(&validation)?;
        actual.set(Some(validation.clone()));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
            DecisionArm::Left => {
                environment
                    .send::<StagePassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultB>(&bytes.as_slice())
                    .await?;
                validations[1] = json!({"deployment":deployment,"validation":validation});
                let history = imp::encode(
                    &serde_json::to_value(&validations)
                        .map_err(|error| ShellError::message(error.to_string()))?,
                )?;
                owner.send::<StageHistory>(&history.as_slice()).await?;
                owner.send::<StageLoan>(environment).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                environment
                    .send::<StageFailedB>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultB>(&bytes.as_slice())
                    .await?;
                let request = environment.recv::<RollbackRequestB>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::rollback_input(&arguments, &deployment, &validation, 1)?;
                let rollback = effects.apply(request).await?;
                imp::rolled_back(&rollback, &deployment, &validation, 1)?;
                let bytes = imp::encode(&rollback)?;
                environment.send::<RolledBackB>(&bytes.as_slice()).await?;
                environment
                    .send::<RollbackReceiptB>(&bytes.as_slice())
                    .await?;
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn run_c<D: ReceivedEffects>(
    owner: &mut Endpoint<STAGE_OWNER_C>,
    (effects, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let history = imp::decode(owner.recv::<StageHistory>().await?)?;
    let validations: [Value; 3] =
        serde_json::from_value(history).map_err(|error| ShellError::message(error.to_string()))?;
    let mut environment = owner.recv::<StageLoan>().await?;
    loop {
        let inputs = imp::decode(owner.recv::<StagingReadyC>().await?)?;
        imp::staging_inputs(&inputs, 2)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyC>(&bytes.as_slice())
            .await?;
        let request = environment.recv::<DeployRequestC>().await?;
        let checked = Arguments::<21>::try_from(request.as_bytes())?;
        let deployment = effects.apply(request);
        environment.send::<CheckStageC>(&checked).await?;
        let arguments = checked.into_value();
        let inputs = imp::decode(environment.recv::<StageInputsC>().await?)?;
        imp::deploy_input(&arguments, &inputs, (2, &validations))?;
        let deployment = deployment.await?;
        imp::deployed(&deployment, &inputs, (2, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<InstalledC>(&bytes.as_slice()).await?;
        environment.send::<DeployedC>(&bytes.as_slice()).await?;
        let request = environment.recv::<ValidateRequestC>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::assigned(&arguments, "service", FEATURES[2])?;
        let validation = effects.apply(request).await?;
        imp::validated(&validation, &deployment, 2)?;
        let bytes = imp::encode(&validation)?;
        actual.set(Some(validation.clone()));
        match actual_outcome(actual).map_err(|error| ShellError::message(format!("{error:?}")))? {
            DecisionArm::Left => {
                environment
                    .send::<StagePassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultC>(&bytes.as_slice())
                    .await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                environment.send::<TaskExit>(&0).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                environment
                    .send::<StageFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultC>(&bytes.as_slice())
                    .await?;
                let request = environment.recv::<RollbackRequestC>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::rollback_input(&arguments, &deployment, &validation, 2)?;
                let rollback = effects.apply(request).await?;
                imp::rolled_back(&rollback, &deployment, &validation, 2)?;
                let bytes = imp::encode(&rollback)?;
                environment.send::<RolledBackC>(&bytes.as_slice()).await?;
                environment
                    .send::<RollbackReceiptC>(&bytes.as_slice())
                    .await?;
            }
        }
    }
}
