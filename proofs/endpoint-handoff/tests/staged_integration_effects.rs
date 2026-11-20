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
    global::Loan,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use staged_integration::{
    arguments::{Arguments, EffectRequest},
    global::*,
    imp as data,
};
use std::{collections::VecDeque, time::Duration};

struct StagingEffects {
    results: [Value; 3],
    calls: Cell<Vec<Value>>,
    validations: Cell<VecDeque<bool>>,
}
impl ReceivedEffects for StagingEffects {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let arguments = data::decode(received.as_bytes());
        async move {
            let arguments = arguments?;
            let (agent, tool) = P::RPC;
            let mut result = match tool {
                "deploy_service" => &self.results[0],
                "validate_staging" => &self.results[1],
                "rollback_service" => &self.results[2],
                _ => return Err(ShellError::message("unknown fixture effect")),
            }
            .clone();
            if tool == "validate_staging" {
                let mut validations = self.validations.take();
                result["success"] = json!(validations.pop_front().expect("unexpected validation"));
                self.validations.set(validations);
            }
            let mut calls = self.calls.take();
            calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
            self.calls.set(calls);
            Ok(result)
        }
    }
}

#[hibana_runtime::scope]
async fn component(inputs: &Value, effects: &StagingEffects) -> ShellResult<()> {
    let ready = g::seq(
        g::send::<STAGING_ENV, REPO, StagePassedA>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
            g::seq(
                g::send::<REPO, BUILD_SERVER, StageUpdateA>(),
                g::seq(
                    g::send::<STAGE_OWNER, STAGE_OWNER_B, StageHistory>(),
                    g::send::<STAGE_OWNER, STAGE_OWNER_B, StageLoan>(),
                ),
            ),
        ),
    );
    let rejected = g::seq(
        g::send::<STAGING_ENV, REPO, StageFailedA>(),
        g::seq(
            g::send::<STAGING_ENV, RELEASE_FILE, FileStageResultA>(),
            g::seq(
                g::send::<RELEASE_FILE, STAGING_ENV, RollbackRequestA>(),
                g::seq(
                    g::send::<STAGING_ENV, RELEASE_FILE, RolledBackA>(),
                    g::seq(
                        g::send::<STAGING_ENV, REPO, RollbackReceiptA>(),
                        g::send::<REPO, BUILD_SERVER, StageUpdateA>(),
                    ),
                ),
            ),
        ),
    );
    let staging = || {
        g::seq(
            g::send::<BUILD_SERVER, STAGE_OWNER, StagingReadyA>(),
            staged_integration::global::staging_a(),
        )
    };
    let stages = g::seq(
        g::send::<RESOURCE_INIT, STAGE_OWNER, StageLoan>(),
        g::seq(
            g::seq(
                staging(),
                g::route(ready, g::seq(g::seq(rejected, staging()).roll(), ready))
                    .resolve::<STAGE_DECISION_A>(),
            ),
            g::send::<STAGING_ENV, REPO, StagePassedB>(),
        ),
    );
    let source = g::seq(
        g::send::<RESOURCE_INIT, REPO, Loan<MERGE_REQUESTS>>(),
        stages,
    );
    let init_program = project::<RESOURCE_INIT>(&source);
    let owner_program = project::<STAGE_OWNER>(&source);
    let next_program = project::<STAGE_OWNER_B>(&source);
    let environment_program = project::<STAGING_ENV>(&source);
    let requests_program = project::<MERGE_REQUESTS>(&source);
    let repo_program = project::<REPO>(&source);
    let build_program = project::<BUILD_SERVER>(&source);
    let file_program = project::<RELEASE_FILE>(&source);
    let session = SessionId::new(SESSION);
    let (bus, _, _) = Bus::open(
        ["RELEASE_MANAGER"],
        [(session, STAGING_ENV), (session, MERGE_REQUESTS)],
    );
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
    let mut next = rv.enter(session, &next_program)?;
    let resource = rv.enter(session, &environment_program)?;
    let requests = rv.enter(session, &requests_program)?;
    let mut repo = rv.enter(session, &repo_program)?;
    let mut build = rv.enter(session, &build_program)?;
    let mut file = rv.enter(session, &file_program)?;
    init.send::<Loan<MERGE_REQUESTS>>(requests).await?;
    let mut requests = repo.recv::<Loan<MERGE_REQUESTS>>().await?;
    init.send::<StageLoan>(resource).await?;
    hibana_runtime::race(
        async {
            let (_, mut environment, _, _) = async {
                async {
                    staged_integration::localside::staging::run_a(&mut owner, (effects, &actual))
                        .await?;
                };
                async {
                    let history = data::decode(next.recv::<StageHistory>().await?)?;
                    assert_eq!(history[0]["deployment"], effects.results[0]);
                    assert_eq!(history[0]["validation"], effects.results[1]);
                    let mut environment = next.recv::<StageLoan>().await?;
                    let bytes = data::encode(&effects.results[1])?;
                    let reply = shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
                    environment.send::<StagePassedB>(&reply).await?;
                    Ok::<_, ShellError>(environment)
                };
                async {
                    let bytes = data::encode(inputs)?;
                    loop {
                        build.send::<StagingReadyA>(&bytes.as_slice()).await?;
                        let arguments = requests.recv::<CheckStageA>().await?.into_value();
                        assert_eq!(arguments, json!({"service": "user_authentication"}));
                        repo.send::<StageInputsA>(&bytes.as_slice()).await?;
                        assert_eq!(
                            data::decode(repo.recv::<InstalledA>().await?)?,
                            effects.results[0]
                        );
                        let outcome = repo.offer().await?;
                        use hibana::g::Message;
                        match outcome.label() {
                            StagePassedA::LOGICAL_LABEL => {
                                let validation =
                                    data::decode(outcome.recv::<StagePassedA>().await?)?;
                                assert_eq!(validation, effects.results[1]);
                                let bytes = data::encode(&validation)?;
                                repo.send::<StageUpdateA>(&bytes.as_slice()).await?;
                                assert_eq!(
                                    data::decode(build.recv::<StageUpdateA>().await?)?,
                                    validation
                                );
                                break;
                            }
                            StageFailedA::LOGICAL_LABEL => {
                                let validation =
                                    data::decode(outcome.recv::<StageFailedA>().await?)?;
                                assert_eq!(validation["success"], false);
                                let rollback =
                                    data::decode(repo.recv::<RollbackReceiptA>().await?)?;
                                assert_eq!(rollback, effects.results[2]);
                                let bytes = data::encode(&validation)?;
                                repo.send::<StageUpdateA>(&bytes.as_slice()).await?;
                                assert_eq!(
                                    data::decode(build.recv::<StageUpdateA>().await?)?,
                                    validation
                                );
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected fixture verdict {label}"
                                )));
                            }
                        }
                    }
                    repo.recv::<StagePassedB>().await?;
                };
                async {
                    loop {
                        let received = data::decode(file.recv::<FileStagingReadyA>().await?)?;
                        assert_eq!(&received, inputs);
                        let request = br#"{"service":"user_authentication"}"#.as_slice();
                        file.send::<DeployRequestA>(&EffectRequest(Arguments::<5>::try_from(
                            request,
                        )?))
                        .await?;
                        assert_eq!(
                            data::decode(file.recv::<DeployedA>().await?)?,
                            effects.results[0]
                        );
                        file.send::<ValidateRequestA>(&EffectRequest(Arguments::<6>::try_from(
                            request,
                        )?))
                        .await?;
                        let validation = data::decode(file.recv::<FileStageResultA>().await?)?;
                        if validation["success"] == true {
                            assert_eq!(validation, effects.results[1]);
                            break;
                        }
                        file.send::<RollbackRequestA>(&EffectRequest(Arguments::<7>::try_from(
                            request,
                        )?))
                        .await?;
                        assert_eq!(
                            data::decode(file.recv::<RolledBackA>().await?)?,
                            effects.results[2]
                        );
                    }
                };
            }
            .await?;
            let mut rejected = effects.results[1].clone();
            rejected["success"] = json!(false);
            let bytes = data::encode(&rejected)?;
            let rejected = shell::imp::review_reply::ReviewReply::<false>::from_bytes(&bytes)?;
            let _ = bus.transport.take_observations().count();
            let error = hibana_runtime::race(
                async {
                    Ok::<_, std::io::Error>(environment.send::<StageFailedA>(&rejected).await)
                },
                hibana_runtime::host::deadline(Duration::from_secs(1)),
            )
            .await
            .expect("retired A operation must reject, not wait")
            .expect_err("B must not revisit A after resource handoff");
            assert!(
                !format!("{error:?}").contains("SessionFault"),
                "the attempted old operation must itself be refused: {error:?}"
            );
            assert_eq!(
                bus.transport.take_observations().count(),
                0,
                "retired A operation emitted a frame"
            );
            Ok(())
        },
        hibana_runtime::host::deadline(Duration::from_secs(5)),
    )
    .await
}

#[hibana_runtime::main]
#[test]
async fn actual_staging_retries_negative_results_then_hands_off_without_old_authority()
-> ShellResult<()> {
    for outcomes in [&[true][..], &[false, false, true][..]] {
        let (inputs, deployment, validation, rollback) = material::material(0);
        let effects = StagingEffects {
            results: [deployment, validation, rollback],
            calls: Cell::new(Vec::new()),
            validations: Cell::new(outcomes.iter().copied().collect()),
        };
        component(&inputs, &effects).await?;
        assert!(effects.validations.take().is_empty());
        let calls = effects.calls.take();
        assert_eq!(calls.len(), (outcomes.len() - 1) * 3 + 2);
        for (attempt, &success) in outcomes.iter().enumerate() {
            assert_eq!(calls[attempt * 3]["tool"], "deploy_service");
            assert_eq!(calls[attempt * 3 + 1]["tool"], "validate_staging");
            if !success {
                assert_eq!(calls[attempt * 3 + 2]["tool"], "rollback_service");
            }
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
            validations: Cell::new([true].into()),
        };
        let error = component(&inputs, &effects).await.unwrap_err();
        assert!(!error.to_string().contains("deadline"), "{error}");
        assert!(effects.calls.take().is_empty());
    }
    Ok(())
}
