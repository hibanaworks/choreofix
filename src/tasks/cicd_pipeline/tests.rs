use super::{arguments::Arguments, global::*, localside};
use crate::{
    imp::json::decode,
    shell::{
        ShellError, ShellResult,
        imp::{bus::Bus, service::Domain},
    },
};
use hibana::{
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::project,
        tap,
        wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

#[derive(Default)]
struct Calls(Cell<Vec<Value>>, Option<(&'static str, &'static str)>);

impl Domain for Calls {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.0.take();
        let success = !self.1.is_some_and(|(operation, target)| {
            tool == operation
                && arguments
                    .get("target")
                    .or_else(|| arguments.get("component"))
                    .and_then(Value::as_str)
                    == Some(target)
                && !calls
                    .iter()
                    .any(|call| call["tool"] == tool && call["arguments"] == arguments)
        });
        let result = json!({"success":success,"agent":agent,"tool":tool,"arguments":arguments,
                            "data":{"component":arguments["component"],"developer":agent}});
        calls.push(result.clone());
        self.0.set(calls);
        Ok(result)
    }
}

// The adversarial sender claims the wire type of the authorized request.
struct Substituted;

impl WireEncode for Substituted {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        let bytes = br#"{"artifact":"backend_image"}"#;
        output
            .get_mut(..bytes.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(bytes);
        Ok(bytes.len())
    }
}

impl WirePayload for Substituted {
    const SCHEMA_ID: u32 = Arguments::<FRONTEND_DEV>::SCHEMA_ID;
    type Decoded<'a> = &'a [u8];

    fn validate_payload(_: Payload<'_>) -> Result<(), CodecError> {
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input.as_bytes()
    }
}

#[hibana_runtime::main]
#[test]
async fn one_valid_request_cannot_publish_and_a_substituted_second_is_rejected() -> ShellResult<()>
{
    let source = g::seq(
        g::send::<BUILD_MASTER, FRONTEND_DEV, WorkspaceLoan>(),
        g::seq(
            g::send::<BUILD_SERVER, REGISTRY_OWNER, RegistryAccepted<FRONTEND_DEV>>(),
            publication::<FRONTEND_DEV, FRONTEND_FILE>(),
        ),
    );
    let session = SessionId::new(16_102);
    let (bus, _, _) = Bus::open([], [(session, BUILD_SERVER), (session, ARTIFACT_STORE)]);
    let mut slab = vec![0; 64 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut capture = rv.tap();
    let mut master = rv.enter(session, &project::<BUILD_MASTER>(&source))?;
    let registry_program = project::<REGISTRY_OWNER>(&source);
    let read_build_wire = tap::projected(&registry_program)
        .find(|row| {
            row.id() == tap::PROJECTED_RECV
                && row.arg0() as u8 == ReadBuild::<FRONTEND_DEV>::LOGICAL_LABEL
        })
        .expect("registry must receive the build workspace request")
        .arg0()
        >> 16;
    let mut registry_owner = rv.enter(session, &registry_program)?;
    let mut developer = rv.enter(session, &project::<FRONTEND_DEV>(&source))?;
    let mut file = rv.enter(session, &project::<FRONTEND_FILE>(&source))?;
    let build = rv.enter(session, &project::<BUILD_SERVER>(&source))?;
    let registry = rv.enter(session, &project::<ARTIFACT_STORE>(&source))?;
    let domain = Calls::default();
    let admitted = Arguments::from_bytes(br#"{"artifact":"frontend_image"}"#)?;

    let rejection = hibana_runtime::race(
        hibana_runtime::race(
            async {
                let error = localside::registry_owner(&mut registry_owner, registry, &domain)
                .await
                .err()
                .expect("malformed artifact request must be rejected");
                assert!(error.to_string().contains("Codec(Malformed)"), "{error}");
                Ok::<(), ShellError>(())
            },
            async {
                async {
                    master.send::<WorkspaceLoan>(build).await?;
                };
                async {
                    file.send::<Publish<FRONTEND_DEV>>(&admitted).await?;
                };
                async {
                    let mut build = developer.recv::<WorkspaceLoan>().await?;
                    build.send::<RegistryAccepted<FRONTEND_DEV>>(&crate::shell::imp::review_reply::ReviewReply::from_bytes(
                        br#"{"success":true,"data":{"component":"frontend","developer":"FRONTEND_DEV"}}"#,
                    )?).await?;
                    let admitted = developer.recv::<Publish<FRONTEND_DEV>>().await?;
                    developer.send::<RegistryIntent<FRONTEND_DEV>>(&admitted).await?;
                    let mut store = developer.recv::<RegistryLoan>().await?;
                    build.send::<ReadBuild<FRONTEND_DEV>>(&admitted).await?;
                    while !capture.any(|record| {
                        record.id() == tap::ENDPOINT_RECV
                            && record.causal_key() >> 8 == u16::from(REGISTRY_OWNER)
                            && u32::from(record.arg1() as u16) == read_build_wire
                    }) {
                        hibana_runtime::host::sleep(Duration::from_millis(1)).await?;
                    }
                    assert!(domain.0.take().is_empty());
                    store.send::<g::Msg<{ RegisterArtifact::<FRONTEND_DEV>::LOGICAL_LABEL }, Substituted>>(&Substituted).await?;
                    core::future::pending::<ShellResult<()>>().await
                };
                Ok::<(), ShellError>(())
            },
        ),
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
    .await;
    rejection?;
    assert!(domain.0.take().is_empty());
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn projected_pipeline_runs_every_effect_through_the_actual_resources() -> ShellResult<()> {
    use super::arguments::{BACKEND_COMPONENT, FRONTEND_COMPONENT};
    use crate::shell::global::Loan;

    for failure in [
        None,
        Some(("deploy_staging", "full_stack")),
        Some(("run_smoke_tests", "staging")),
        Some(("run_smoke_tests", "production")),
        Some(("compile_code", "frontend")),
        Some(("run_unit_tests", "backend")),
    ] {
        let source = pipeline();
        let session = SessionId::new(16_103);
        let (bus, _, _) = Bus::open(
            [],
            [BUILD_SERVER, ARTIFACT_STORE, TEST_ENV, PROD_ENV].map(|role| (session, role)),
        );
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut registry_owner = rv.enter(session, &project::<REGISTRY_OWNER>(&source))?;
        let mut master = rv.enter(session, &project::<BUILD_MASTER>(&source))?;
        let mut master_file = rv.enter(session, &project::<MASTER_FILE>(&source))?;
        let mut frontend = rv.enter(session, &project::<FRONTEND_DEV>(&source))?;
        let mut backend = rv.enter(session, &project::<BACKEND_DEV>(&source))?;
        let mut frontend_file = rv.enter(session, &project::<FRONTEND_FILE>(&source))?;
        let mut backend_file = rv.enter(session, &project::<BACKEND_FILE>(&source))?;
        let mut qa_file = rv.enter(session, &project::<QA_FILE>(&source))?;
        let mut release_file = rv.enter(session, &project::<RELEASE_FILE>(&source))?;
        let mut qa = rv.enter(session, &project::<QA>(&source))?;
        let mut release = rv.enter(session, &project::<RELEASE>(&source))?;
        let mut prod_owner = rv.enter(session, &project::<PROD_OWNER>(&source))?;
        let build = rv.enter(session, &project::<BUILD_SERVER>(&source))?;
        let registry = rv.enter(session, &project::<ARTIFACT_STORE>(&source))?;
        let test_env = rv.enter(session, &project::<TEST_ENV>(&source))?;
        let prod_env = rv.enter(session, &project::<PROD_ENV>(&source))?;
        let domain = Calls(Cell::default(), failure);
        let build_failed = matches!(failure, Some(("compile_code" | "run_unit_tests", _)));
        let complete = failure.is_none() || build_failed;
        hibana_runtime::race(
            async {
                async { localside::registry_owner(&mut registry_owner, registry, &domain).await };
                async {
                    localside::developer::<FRONTEND_DEV, FRONTEND_COMPONENT>(&mut frontend, &domain).await
                };
                async {
                    localside::developer::<BACKEND_DEV, BACKEND_COMPONENT>(&mut backend, &domain).await
                };
                async { localside::qa(&mut qa, test_env, &domain).await };
                async { localside::release(&mut release, &domain).await };
                async {
                    let branch = prod_owner.offer().await?;
                    match branch.label() {
                        Promote::LOGICAL_LABEL => {
                            branch.recv::<Promote>().await?;
                            prod_owner.send::<Loan<PROD_ENV>>(prod_env).await?;
                            drop(prod_owner.recv::<Loan<PROD_ENV>>().await?);
                        }
                        StagingAborted::LOGICAL_LABEL => {
                            branch.recv::<StagingAborted>().await?;
                        }
                        label => panic!("unexpected production continuation {label}"),
                    }
                    Ok::<(), ShellError>(())
                };
                async { localside::master(&mut master, build).await };
                async {
                    master_file
                        .send::<ChooseFrontend>(&Arguments::from_bytes(br#"{"first":"frontend"}"#)?)
                        .await?;
                    let report = decode(master_file.recv::<QaReport>().await?)?;
                    assert_eq!(report["success"], true);
                    let result = decode(master_file.recv::<FinalResult>().await?)?;
                    assert_eq!(
                        result["arguments"],
                        if failure == Some(("deploy_staging", "full_stack")) {
                            json!({"component":"full_stack"})
                        } else {
                            json!({"target":failure.filter(|_| !build_failed).map_or("production", |(_, target)| target)})
                        }
                    );
                    assert_eq!(result["success"], complete);
                };
                async {
                    frontend_file.recv::<BeginFrontend>().await?;
                    let component = Arguments::from_bytes(br#"{"component":"frontend"}"#)?;
                    loop {
                    frontend_file.recv::<WorkspaceAvailable>().await?;
                    frontend_file
                        .send::<Compile<FRONTEND_COMPONENT>>(&component)
                        .await?;
                    let compiled = frontend_file.recv::<Compiled>().await?.to_vec();
                    if decode(&compiled)?["success"] == false {
                        assert_eq!(frontend_file.recv::<Revise<FRONTEND_DEV>>().await?, compiled);
                        continue;
                    }
                    frontend_file
                        .send::<UnitTest<FRONTEND_COMPONENT>>(&component)
                        .await?;
                    let tested = frontend_file.recv::<UnitTested>().await?.to_vec();
                    if decode(&tested)?["success"] == false {
                        assert_eq!(frontend_file.recv::<Revise<FRONTEND_DEV>>().await?, tested);
                        continue;
                    }
                    let artifact = Arguments::from_bytes(br#"{"artifact":"frontend_image"}"#)?;
                    frontend_file
                        .send::<Publish<FRONTEND_DEV>>(&artifact)
                        .await?;
                    frontend_file.recv::<Published>().await?;
                        break;
                    }
                    frontend_file.recv::<BuildVerified>().await?;
                    frontend_file.recv::<BuildEnded<FRONTEND_DEV>>().await?;
                    let result = decode(frontend_file.recv::<FrontendFinished>().await?)?;
                    assert_eq!(result["success"], complete);
                };
                async {
                    backend_file.recv::<BeginBackend>().await?;
                    let component = Arguments::from_bytes(br#"{"component":"backend"}"#)?;
                    loop {
                    backend_file.recv::<WorkspaceAvailable>().await?;
                    backend_file
                        .send::<Compile<BACKEND_COMPONENT>>(&component)
                        .await?;
                    let compiled = backend_file.recv::<Compiled>().await?.to_vec();
                    if decode(&compiled)?["success"] == false {
                        assert_eq!(backend_file.recv::<Revise<BACKEND_DEV>>().await?, compiled);
                        continue;
                    }
                    backend_file
                        .send::<UnitTest<BACKEND_COMPONENT>>(&component)
                        .await?;
                    let tested = backend_file.recv::<UnitTested>().await?.to_vec();
                    if decode(&tested)?["success"] == false {
                        assert_eq!(backend_file.recv::<Revise<BACKEND_DEV>>().await?, tested);
                        continue;
                    }
                    let artifact = Arguments::from_bytes(br#"{"artifact":"backend_image"}"#)?;
                    backend_file.send::<Publish<BACKEND_DEV>>(&artifact).await?;
                    backend_file.recv::<Published>().await?;
                        break;
                    }
                    backend_file.recv::<BuildVerified>().await?;
                    backend_file.recv::<BuildEnded<BACKEND_DEV>>().await?;
                    let result = decode(backend_file.recv::<BackendFinished>().await?)?;
                    assert_eq!(result["success"], complete);
                };
                async {
                    qa_file.recv::<ReadyForQa>().await?;
                    qa_file
                        .send::<PullForQa>(&Arguments::from_bytes(br#"{"target":"test_harness"}"#)?)
                        .await?;
                    qa_file.recv::<QaPulled>().await?;
                    qa_file
                        .send::<E2e>(&Arguments::from_bytes(br#"{"suite":"user_flows"}"#)?)
                        .await?;
                    qa_file.recv::<E2eTested>().await?;
                    qa_file
                        .send::<Integration>(&Arguments::from_bytes(
                            br#"{"suite":"api_integration"}"#,
                        )?)
                        .await?;
                    qa_file.recv::<Integrated>().await?;
                    let result = decode(qa_file.recv::<QaFinished>().await?)?;
                    assert_eq!(result["success"], complete);
                };
                async {
                    release_file.recv::<ReadyForRelease>().await?;
                    release_file
                        .send::<PullForRelease>(&Arguments::from_bytes(
                            br#"{"target":"staging_deploy"}"#,
                        )?)
                        .await?;
                    release_file.recv::<ReleasePulled>().await?;
                    let stack = Arguments::from_bytes(br#"{"component":"full_stack"}"#)?;
                    release_file.send::<Stage>(&stack).await?;
                    let deployed = release_file.recv::<Staged>().await?.to_vec();
                    let staging = if decode(&deployed)?["success"] == true {
                        assert_eq!(release_file.recv::<StagingReady>().await?, deployed);
                        release_file
                            .send::<SmokeStaging>(&Arguments::from_bytes(
                                br#"{"target":"staging"}"#,
                            )?)
                            .await?;
                        release_file.recv::<StagingResult>().await?.to_vec()
                    } else {
                        deployed
                    };
                    let decision = release_file.offer().await?;
                    match decision.label() {
                        StagingAborted::LOGICAL_LABEL => {
                            assert_eq!(decision.recv::<StagingAborted>().await?, staging);
                        }
                        Promote::LOGICAL_LABEL => {
                            assert_eq!(decision.recv::<Promote>().await?, staging);
                            release_file.send::<Deploy>(&stack).await?;
                            release_file.recv::<Deployed>().await?;
                            release_file
                                .send::<SmokeProduction>(&Arguments::from_bytes(
                                    br#"{"target":"production"}"#,
                                )?)
                                .await?;
                            let result = decode(release_file.recv::<ProductionSmoke>().await?)?;
                            assert_eq!(result["success"], complete);
                        }
                        label => panic!("unexpected release decision {label}"),
                    }
                    let result = decode(release_file.recv::<ReleaseFinished>().await?)?;
                    assert_eq!(result["success"], complete);
                };
                Ok::<(), ShellError>(())
            },
            hibana_runtime::host::deadline(Duration::from_secs(5)),
        )
        .await?;
        let events = domain.0.take();
        let expected = [
            "compile_code",
            "run_unit_tests",
            "publish_artifact",
            "compile_code",
            "run_unit_tests",
            "publish_artifact",
            "pull_artifacts",
            "run_e2e_tests",
            "run_integration_tests",
            "pull_artifacts",
            "deploy_staging",
            "run_smoke_tests",
            "deploy_production",
            "run_smoke_tests",
        ];
        let effects = events
            .iter()
            .map(|e| e["tool"].as_str().unwrap())
            .collect::<Vec<_>>();
        let expected = match failure {
            Some(("deploy_staging", _)) => &expected[..11],
            Some(("run_smoke_tests", "staging")) => &expected[..12],
            _ => &expected[..],
        };
        if build_failed {
            let failed = events.iter().find(|call| call["success"] == false).unwrap();
            let first_publication = events
                .iter()
                .position(|call| call["tool"] == "publish_artifact")
                .unwrap();
            assert_ne!(events[first_publication]["agent"], failed["agent"]);
            let retry = events
                .iter()
                .rposition(|call| call["tool"] == "compile_code")
                .unwrap();
            assert_eq!(events[retry]["agent"], failed["agent"]);
            assert!(retry > first_publication);
            assert_eq!(
                events
                    .iter()
                    .filter(|call| call["tool"] == "compile_code")
                    .count(),
                3
            );
            assert_eq!(
                effects.len(),
                if failure == Some(("compile_code", "frontend")) {
                    15
                } else {
                    16
                }
            );
        } else {
            assert_eq!(effects, expected);
        }
    }
    Ok(())
}
