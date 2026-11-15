//! One physical staging owner. Each request is received on its actual resource Endpoint.
use super::super::{actual_outcome, arguments::FEATURES, global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message, runtime::resolver::DecisionArm};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<STAGE_OWNER>,
    (domain, actual): (&D, &Cell<Option<Value>>),
) -> ShellResult<()> {
    let mut environment = owner.recv::<StageLoan>().await?;
    let mut validations = [Value::Null, Value::Null, Value::Null];
    loop {
        let branch = environment.offer().await?;
        match branch.label() {
            StagingReadyA::LOGICAL_LABEL => {
                let inputs = imp::decode(branch.recv::<StagingReadyA>().await?)?;
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
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        environment
                            .send::<StagePassedA>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        environment
                            .send::<FileStageResultA>(&bytes.as_slice())
                            .await?;
                        validations[0] = json!({"deployment":deployment,"validation":validation});
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
            StagingReadyB::LOGICAL_LABEL => {
                let inputs = imp::decode(branch.recv::<StagingReadyB>().await?)?;
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
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        environment
                            .send::<StagePassedB>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        environment
                            .send::<FileStageResultB>(&bytes.as_slice())
                            .await?;
                        validations[1] = json!({"deployment":deployment,"validation":validation});
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
            StagingReadyC::LOGICAL_LABEL => {
                let inputs = imp::decode(branch.recv::<StagingReadyC>().await?)?;
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
                match actual_outcome(actual).map_err(|e| ShellError::message(format!("{e:?}")))? {
                    DecisionArm::Left => {
                        environment
                            .send::<StagePassedC>(&ReviewReply::<true>::from_bytes(&bytes)?)
                            .await?;
                        environment
                            .send::<FileStageResultC>(&bytes.as_slice())
                            .await?;
                        validations[2] = json!({"deployment":deployment,"validation":validation});
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
            TaskExit::LOGICAL_LABEL => {
                if branch.recv::<TaskExit>().await? != 0 {
                    return Err(ShellError::message("staged integration failed"));
                }
                break;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected staging operation {label}"
                )));
            }
        }
    }
    Ok(())
}
