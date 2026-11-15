//! Actual resource receives and effects in the production staging localside.
//! This native component fixture does not qualify the seven-agent Wasm task.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "staged_integration/material.rs"]
mod material;
#[path = "../../../src/tasks/staged_integration/mod.rs"]
pub mod staged_integration;

use core::cell::Cell;
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project, resolver::ResolverRef},
};
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use staged_integration::{arguments::Arguments, global::*, imp as data};
use std::time::Duration;

struct StagingEffects {
    results: [Value; 3],
    calls: Cell<Vec<Value>>,
}
impl Domain for StagingEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let result = match tool {
            "deploy_service" => &self.results[0],
            "validate_staging" => &self.results[1],
            "rollback_service" => &self.results[2],
            _ => return Err("unknown fixture effect".into()),
        };
        let mut calls = self.calls.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.calls.set(calls);
        Ok(result.clone())
    }
}

#[hibana_runtime::scope]
async fn component(inputs: &Value, effects: &StagingEffects) -> ShellResult<()> {
    let selected = g::route(
        g::seq(
            g::send::<STAGING_ENV, REPO, StagePassedA>(),
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
        ),
        g::seq(
            g::send::<STAGING_ENV, REPO, StageFailedA>(),
            g::seq(
                g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
                g::seq(
                    g::send::<RELEASE_FILE, STAGING_ENV, RollbackRequestA>(),
                    g::seq(
                        g::send::<STAGING_ENV, RELEASE_FILE, RolledBackA>(),
                        g::send::<STAGING_ENV, REPO, RollbackReceiptA>(),
                    ),
                ),
            ),
        ),
    )
    .resolve::<STAGE_DECISION_A>();
    let source = g::seq(
        g::send::<RESOURCE_INIT, STAGE_OWNER, StageLoan>(),
        g::seq(
            staged_integration::global::staging_a(),
            g::seq(selected, g::send::<REPO, STAGING_ENV, TaskExit>()),
        ),
    );
    let init_program = project::<RESOURCE_INIT>(&source);
    let owner_program = project::<STAGE_OWNER>(&source);
    let environment_program = project::<STAGING_ENV>(&source);
    let repo_program = project::<REPO>(&source);
    let file_program = project::<RELEASE_FILE>(&source);
    let session = SessionId::new(SESSION);
    let (bus, _, _) = Bus::open(["RELEASE_MANAGER"], [(session, STAGING_ENV)]);
    let actual = Cell::new(None);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.set_resolver(
        &environment_program,
        ResolverRef::<STAGE_DECISION_A>::decision_state(
            &actual,
            staged_integration::actual_outcome,
        ),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let mut init = rv.enter(session, &init_program)?;
    let mut owner = rv.enter(session, &owner_program)?;
    let resource = rv.enter(session, &environment_program)?;
    let mut repo = rv.enter(session, &repo_program)?;
    let mut file = rv.enter(session, &file_program)?;
    init.send::<StageLoan>(resource).await?;
    hibana_runtime::race(
        async {
            async {
                staged_integration::localside::staging::run(&mut owner, (effects, &actual)).await?;
            };
            async {
                let bytes = data::encode(inputs)?;
                repo.send::<StagingReadyA>(&bytes.as_slice()).await?;
                let outcome = repo.offer().await?;
                use hibana::g::Message;
                match outcome.label() {
                    StagePassedA::LOGICAL_LABEL => {
                        outcome.recv::<StagePassedA>().await?;
                    }
                    StageFailedA::LOGICAL_LABEL => {
                        outcome.recv::<StageFailedA>().await?;
                        let rollback = data::decode(repo.recv::<RollbackReceiptA>().await?)?;
                        assert_eq!(rollback, effects.results[2]);
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected fixture verdict {label}"
                        )));
                    }
                }
                repo.send::<TaskExit>(&0).await?;
            };
            async {
                let received = data::decode(file.recv::<FileStagingReadyA>().await?)?;
                assert_eq!(&received, inputs);
                let request = br#"{"service":"user_authentication"}"#;
                file.send::<DeployRequestA>(&Arguments::<5>::from_bytes(request)?)
                    .await?;
                let deployment = data::decode(file.recv::<DeployedA>().await?)?;
                assert_eq!(deployment, effects.results[0]);
                file.send::<ValidateRequestA>(&Arguments::<6>::from_bytes(request)?)
                    .await?;
                let validation = data::decode(file.recv::<FileStageResultA>().await?)?;
                assert_eq!(validation, effects.results[1]);
                if validation["success"] == false {
                    file.send::<RollbackRequestA>(&Arguments::<7>::from_bytes(request)?)
                        .await?;
                    assert_eq!(
                        data::decode(file.recv::<RolledBackA>().await?)?,
                        effects.results[2]
                    );
                }
            };
            Ok(())
        },
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await
}

#[hibana_runtime::main]
#[test]
async fn actual_staging_endpoint_returns_positive_validation_and_rolls_back_a_negative_one()
-> ShellResult<()> {
    for success in [true, false] {
        let (inputs, deployment, mut validation, rollback) = material::material(0);
        validation["success"] = json!(success);
        let effects = StagingEffects {
            results: [deployment, validation, rollback],
            calls: Cell::new(Vec::new()),
        };
        component(&inputs, &effects).await?;
        let calls = effects.calls.take();
        assert_eq!(calls.len(), if success { 2 } else { 3 });
        assert_eq!(calls[0]["tool"], "deploy_service");
        assert_eq!(calls[1]["tool"], "validate_staging");
        if !success {
            assert_eq!(calls[2]["tool"], "rollback_service");
        }
        assert!(calls.iter().all(|call| call["agent"] == "RELEASE_MANAGER"
            && call["arguments"] == json!({"service":"user_authentication"})));
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn an_endpoint_received_stale_or_negative_ci_input_cannot_start_a_staging_effect()
-> ShellResult<()> {
    for kind in 0..4 {
        let (mut inputs, deployment, validation, rollback) = material::material(0);
        match kind {
            0 => inputs["ci"]["success"] = json!(false),
            1 => inputs["ci"]["data"]["repository_sha256"] = json!("old repository"),
            2 => {
                inputs["ci"]["data"]["published_candidates"]["user_authentication"] = json!("other")
            }
            3 => inputs["ci"]["data"]["publication_event"] = json!(0),
            _ => unreachable!(),
        }
        let effects = StagingEffects {
            results: [deployment, validation, rollback],
            calls: Cell::new(Vec::new()),
        };
        let error = component(&inputs, &effects).await.unwrap_err();
        assert!(!error.to_string().contains("deadline"), "{error}");
        assert!(effects.calls.take().is_empty());
    }
    Ok(())
}
