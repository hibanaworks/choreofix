//! Each feature's staging participant owns the same resource until its positive result.
use super::super::{actual_outcome, arguments::FEATURES, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run_a<D: Domain>(
    owner: &mut Endpoint<STAGE_OWNER>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut environment = owner.recv::<StageLoan>().await?;
    let mut validations = [Value::Null, Value::Null, Value::Null];
    loop {
        let inputs = imp::decode(environment.recv::<StagingReadyA>().await?)?;
        imp::staging_inputs(&inputs, 0)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyA>(&bytes.as_slice())
            .await?;
        let arguments = environment.recv::<DeployRequestA>().await?.into_value();
        imp::deploy_input(&arguments, &inputs, (0, &validations))?;
        let deployment = domain
            .execute("RELEASE_MANAGER", "deploy_service", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::deployed(&deployment, &inputs, (0, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<DeployedA>(&bytes.as_slice()).await?;
        let arguments = environment.recv::<ValidateRequestA>().await?.into_value();
        imp::assigned(&arguments, "service", FEATURES[0])?;
        let validation = domain
            .execute("RELEASE_MANAGER", "validate_staging", arguments)
            .await
            .map_err(ShellError::message)?;
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
                let arguments = environment.recv::<RollbackRequestA>().await?.into_value();
                imp::rollback_input(&arguments, &deployment, &validation, 0)?;
                let rollback = domain
                    .execute("RELEASE_MANAGER", "rollback_service", arguments)
                    .await
                    .map_err(ShellError::message)?;
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
pub async fn run_b<D: Domain>(
    owner: &mut Endpoint<STAGE_OWNER_B>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let history = imp::decode(owner.recv::<StageHistory>().await?)?;
    let mut validations: [Value; 3] =
        serde_json::from_value(history).map_err(|error| ShellError::message(error.to_string()))?;
    let mut environment = owner.recv::<StageLoan>().await?;
    loop {
        let inputs = imp::decode(environment.recv::<StagingReadyB>().await?)?;
        imp::staging_inputs(&inputs, 1)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyB>(&bytes.as_slice())
            .await?;
        let arguments = environment.recv::<DeployRequestB>().await?.into_value();
        imp::deploy_input(&arguments, &inputs, (1, &validations))?;
        let deployment = domain
            .execute("RELEASE_MANAGER", "deploy_service", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::deployed(&deployment, &inputs, (1, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<DeployedB>(&bytes.as_slice()).await?;
        let arguments = environment.recv::<ValidateRequestB>().await?.into_value();
        imp::assigned(&arguments, "service", FEATURES[1])?;
        let validation = domain
            .execute("RELEASE_MANAGER", "validate_staging", arguments)
            .await
            .map_err(ShellError::message)?;
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
                let arguments = environment.recv::<RollbackRequestB>().await?.into_value();
                imp::rollback_input(&arguments, &deployment, &validation, 1)?;
                let rollback = domain
                    .execute("RELEASE_MANAGER", "rollback_service", arguments)
                    .await
                    .map_err(ShellError::message)?;
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
pub async fn run_c<D: Domain>(
    owner: &mut Endpoint<STAGE_OWNER_C>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let history = imp::decode(owner.recv::<StageHistory>().await?)?;
    let validations: [Value; 3] =
        serde_json::from_value(history).map_err(|error| ShellError::message(error.to_string()))?;
    let mut environment = owner.recv::<StageLoan>().await?;
    loop {
        let inputs = imp::decode(environment.recv::<StagingReadyC>().await?)?;
        imp::staging_inputs(&inputs, 2)?;
        let bytes = imp::encode(&inputs)?;
        environment
            .send::<FileStagingReadyC>(&bytes.as_slice())
            .await?;
        let arguments = environment.recv::<DeployRequestC>().await?.into_value();
        imp::deploy_input(&arguments, &inputs, (2, &validations))?;
        let deployment = domain
            .execute("RELEASE_MANAGER", "deploy_service", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::deployed(&deployment, &inputs, (2, &validations))?;
        let bytes = imp::encode(&deployment)?;
        environment.send::<DeployedC>(&bytes.as_slice()).await?;
        let arguments = environment.recv::<ValidateRequestC>().await?.into_value();
        imp::assigned(&arguments, "service", FEATURES[2])?;
        let validation = domain
            .execute("RELEASE_MANAGER", "validate_staging", arguments)
            .await
            .map_err(ShellError::message)?;
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
                return Ok(());
            }
            DecisionArm::Right => {
                environment
                    .send::<StageFailedC>(&ReviewReply::<false>::from_bytes(&bytes)?)
                    .await?;
                environment
                    .send::<FileStageResultC>(&bytes.as_slice())
                    .await?;
                let arguments = environment.recv::<RollbackRequestC>().await?.into_value();
                imp::rollback_input(&arguments, &deployment, &validation, 2)?;
                let rollback = domain
                    .execute("RELEASE_MANAGER", "rollback_service", arguments)
                    .await
                    .map_err(ShellError::message)?;
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
