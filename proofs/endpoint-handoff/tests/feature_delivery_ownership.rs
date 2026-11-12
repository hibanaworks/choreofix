//! Miri checks ownership/cancellation on canonical CI source fragments.
//! The whole 768-event native matrix separately checks workflow conformance.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/feature_delivery/mod.rs"]
pub mod feature_delivery;
use core::cell::Cell;
use feature_delivery::{
    global::{
        delivery::{self, *},
        *,
    },
    imp as data, localside,
};
use hibana::{
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::project,
        resolver::{DecisionArm, ResolverError, ResolverRef},
    },
};
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{repository_arguments::RepositoryArguments, review_reply::ReviewReply, service::Domain},
};
#[cfg(miri)]
#[path = "../../../src/tasks/handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// The fixture's outer resource choice is made by Repository. These deliveries
// give each conditional peer that knowledge; they are not application messages.
type CaseA = g::Msg<200, ()>;
type CaseB = g::Msg<201, ()>;
fn source() -> impl hibana::runtime::program::Projectable {
    g::seq(
        g::send::<CI_SERVICE, CI_RUNNER, BuildLoan>(),
        g::seq(
            g::route(
                g::seq(
                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiA>(),
                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiB>(),
                ),
                g::seq(
                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiB>(),
                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiA>(),
                ),
            ),
            g::route(
                g::seq(
                    g::seq(
                        g::send::<REPOSITORY, REVIEW_A, CaseA>(),
                        g::seq(
                            g::send::<REPOSITORY, REVIEW_B, CaseA>(),
                            g::seq(
                                g::send::<REPOSITORY, TEST_DATA_A, CaseA>(),
                                g::seq(
                                    g::send::<REPOSITORY, TEST_DATA_B, CaseA>(),
                                    g::send::<REPOSITORY, CI_RUNNER, MergeLoanA>(),
                                ),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<REPOSITORY, CI_SERVICE, Merged>(),
                        g::seq(
                            delivery::ci_a(),
                            g::route(
                                g::seq(
                                    delivery::passed_a::<LastCiPassedA>(),
                                    g::send::<CI_RUNNER, CI_SERVICE, BuildReturn>(),
                                ),
                                g::seq(
                                    delivery::failed_a(),
                                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiA>(),
                                ),
                            )
                            .resolve::<CI_DECISION_A>(),
                        ),
                    ),
                ),
                g::seq(
                    g::seq(
                        g::send::<REPOSITORY, REVIEW_A, CaseB>(),
                        g::seq(
                            g::send::<REPOSITORY, REVIEW_B, CaseB>(),
                            g::seq(
                                g::send::<REPOSITORY, TEST_DATA_A, CaseB>(),
                                g::seq(
                                    g::send::<REPOSITORY, TEST_DATA_B, CaseB>(),
                                    g::send::<REPOSITORY, CI_RUNNER, MergeLoanB>(),
                                ),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<REPOSITORY, CI_SERVICE, Merged>(),
                        g::seq(
                            delivery::ci_b(),
                            g::route(
                                g::seq(
                                    delivery::passed_b::<LastCiPassedB>(),
                                    g::send::<CI_RUNNER, CI_SERVICE, BuildReturn>(),
                                ),
                                g::seq(
                                    delivery::failed_b(),
                                    g::send::<TEST_WRITER, CI_SERVICE, TestsForCiB>(),
                                ),
                            )
                            .resolve::<CI_DECISION_B>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
struct Effect {
    calls: Cell<usize>,
    result: Value,
}
impl Domain for Effect {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        _arguments: Value,
    ) -> Result<Value, String> {
        assert_eq!(agent, "CI_RUNNER");
        assert_eq!(tool, "run_tests");
        self.calls.set(self.calls.get() + 1);
        Ok(self.result.clone())
    }
}
fn decide(actual: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = actual.take();
    let decision = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    actual.set(value);
    decision
}
#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn ci_receiver_ownership_and_cancellation_paths() -> ShellResult<()> {
    let (
        author_a_program,
        author_b_program,
        writer_program,
        reviewer_program,
        runner_program,
        repo_program,
        review_a_program,
        review_b_program,
        data_a_program,
        data_b_program,
        ci_program,
        build_program,
        feature_a_program,
        feature_b_program,
    ) = {
        let source = source();
        (
            project::<DEVELOPER_A>(&source),
            project::<DEVELOPER_B>(&source),
            project::<TEST_WRITER>(&source),
            project::<REVIEWER>(&source),
            project::<CI_RUNNER>(&source),
            project::<REPOSITORY>(&source),
            project::<REVIEW_A>(&source),
            project::<REVIEW_B>(&source),
            project::<TEST_DATA_A>(&source),
            project::<TEST_DATA_B>(&source),
            project::<CI_SERVICE>(&source),
            project::<BUILD_SERVER>(&source),
            project::<AUTH_A>(&source),
            project::<AUTH_B>(&source),
        )
    };
    for selected in [FEATURE_A, FEATURE_B] {
        for trial in 0..4 {
            let tests_a = json!({"tool_name":"write_tests","success":true,"data":{"component":FEATURE_A,"agent":"TEST_WRITER","candidate_sha256":"auth","test_source":{"target":FEATURE_A},"test_sha256":"auth-tests","artifact_id":"development:101"}});
            let tests_b = json!({"tool_name":"write_tests","success":true,"data":{"component":FEATURE_B,"agent":"TEST_WRITER","candidate_sha256":"payment","test_source":{"target":FEATURE_B},"test_sha256":"payment-tests","artifact_id":"development:102"}});
            let identity = if selected == FEATURE_A {
                "auth"
            } else {
                "payment"
            };
            let publication = json!({"tool_name":"merge_code","success":true,"data":{"feature":selected,"branch":selected,"candidate_sha256":identity,"repository_sha256":"current","review_receipt":99,"publication_event":200,"published_candidates":{FEATURE_A:"auth",FEATURE_B:"payment"}}});
            let mut result = json!({"tool_name":"run_tests","success":trial!=1,"data":{"feature":selected,"suite":if selected==FEATURE_A{SUITE_A}else{SUITE_B},"agent":"CI_RUNNER","repository_sha256":"current","artifact_id":"development:201","publication_event":200,"publication_events":{FEATURE_A:if selected==FEATURE_A{200}else{199},FEATURE_B:if selected==FEATURE_B{200}else{199}},"published_candidates":{FEATURE_A:"auth",FEATURE_B:"payment"},"test_receipts":{FEATURE_A:101,FEATURE_B:102},"test_inputs":{FEATURE_A:tests_a["data"],FEATURE_B:tests_b["data"]}}});
            if trial == 3 {
                result["data"]["repository_sha256"] = json!("stale");
            }
            let effects = &Effect {
                calls: Cell::new(0),
                result,
            };
            let actual = Cell::new(Value::Null);
            let sid = SessionId::new(SESSION);
            // SAFETY: this isolated fixture is the sole issuer of three distinct resources.
            #[allow(unsafe_code)]
            let transport = transport::InMemoryTransport::new(unsafe {
                [
                    (
                        sid,
                        BUILD_SERVER,
                        hibana::runtime::transport::EndpointBinding::new(),
                    ),
                    (
                        sid,
                        AUTH_A,
                        hibana::runtime::transport::EndpointBinding::new(),
                    ),
                    (
                        sid,
                        AUTH_B,
                        hibana::runtime::transport::EndpointBinding::new(),
                    ),
                ]
            });
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &transport)?;
            let mut author_a = rv.enter(sid, &author_a_program)?;
            let mut author_b = rv.enter(sid, &author_b_program)?;
            let mut writer = rv.enter(sid, &writer_program)?;
            let mut reviewer = rv.enter(sid, &reviewer_program)?;
            let mut runner = rv.enter(sid, &runner_program)?;
            let mut repo = rv.enter(sid, &repo_program)?;
            let mut review_a = rv.enter(sid, &review_a_program)?;
            let mut review_b = rv.enter(sid, &review_b_program)?;
            let mut data_a = rv.enter(sid, &data_a_program)?;
            let mut data_b = rv.enter(sid, &data_b_program)?;
            let mut ci = rv.enter(sid, &ci_program)?;
            let build = rv.enter(sid, &build_program)?;
            let feature_a = rv.enter(sid, &feature_a_program)?;
            let feature_b = rv.enter(sid, &feature_b_program)?;
            rv.set_resolver(
                &runner_program,
                ResolverRef::<CI_DECISION_A>::decision_state(&actual, decide),
            )
            .unwrap();
            rv.set_resolver(
                &runner_program,
                ResolverRef::<CI_DECISION_B>::decision_state(&actual, decide),
            )
            .unwrap();
            let outcome = hibana_runtime::race(
                async {
                    async { localside::ci::run(&mut ci, build, effects).await };
                    async {
                        if selected == FEATURE_A {
                            repo.send::<CaseA>(&()).await?;
                            repo.send::<CaseA>(&()).await?;
                            repo.send::<CaseA>(&()).await?;
                            repo.send::<CaseA>(&()).await?;
                            repo.send::<MergeLoanA>(feature_a).await?;
                            let bytes = data::encode(&publication)?;
                            repo.send::<Merged>(&bytes.as_slice()).await?;
                            let result = data::decode(repo.recv::<CiResult>().await?)?;
                            let branch = repo.offer().await?;
                            match branch.label() {
                                IntegratedA::LOGICAL_LABEL => {
                                    branch.recv::<IntegratedA>().await?;
                                    let mut authority = repo.recv::<Feature<AUTH_A>>().await?;
                                    let bytes = data::encode(&result)?;
                                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                    authority.send::<LastCiPassedA>(&reply).await?;
                                }
                                IntegrationFailedA::LOGICAL_LABEL => {
                                    branch.recv::<IntegrationFailedA>().await?;
                                }
                                _ => {
                                    return Err(ShellError::message("unexpected ownership result"));
                                }
                            }
                        } else {
                            repo.send::<CaseB>(&()).await?;
                            repo.send::<CaseB>(&()).await?;
                            repo.send::<CaseB>(&()).await?;
                            repo.send::<CaseB>(&()).await?;
                            repo.send::<MergeLoanB>(feature_b).await?;
                            let bytes = data::encode(&publication)?;
                            repo.send::<Merged>(&bytes.as_slice()).await?;
                            let result = data::decode(repo.recv::<CiResult>().await?)?;
                            let branch = repo.offer().await?;
                            match branch.label() {
                                IntegratedB::LOGICAL_LABEL => {
                                    branch.recv::<IntegratedB>().await?;
                                    let mut authority = repo.recv::<Feature<AUTH_B>>().await?;
                                    let bytes = data::encode(&result)?;
                                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                    authority.send::<LastCiPassedB>(&reply).await?;
                                }
                                IntegrationFailedB::LOGICAL_LABEL => {
                                    branch.recv::<IntegrationFailedB>().await?;
                                }
                                _ => {
                                    return Err(ShellError::message("unexpected ownership result"));
                                }
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let mut build = runner.recv::<BuildLoan>().await?;
                        let branch = runner.offer().await?;
                        match branch.label() {
                            MergeLoanA::LOGICAL_LABEL => {
                                let mut authority = branch.recv::<MergeLoanA>().await?;
                                let admitted =
                                    RepositoryArguments::from_bytes(if selected == FEATURE_A {
                                        br#"{"suite":"full_test_suite_after_auth"}"#
                                    } else {
                                        br#"{"suite":"full_test_suite_after_payment"}"#
                                    })?;
                                let requested = RepositoryArguments::from_bytes(
                                    if (selected == FEATURE_A) != (trial == 2) {
                                        br#"{"suite":"full_test_suite_after_auth"}"#
                                    } else {
                                        br#"{"suite":"full_test_suite_after_payment"}"#
                                    },
                                )?;
                                runner.send::<CiIntentA>(&admitted).await?;
                                async {
                                    async {
                                        authority.send::<RunCiOnFeatureA>(&admitted).await?;
                                    };
                                    async {
                                        build.send::<RunCiA>(&requested).await?;
                                    };
                                    Ok::<_, ShellError>(())
                                }
                                .await?;
                                let received = data::decode(authority.recv::<CiResult>().await?)?;
                                data::same(
                                    &received,
                                    &data::decode(build.recv::<CiResult>().await?)?,
                                )?;
                                actual.set(received.clone());
                                let bytes = data::encode(&received)?;
                                if received["success"] == true {
                                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                                    authority.send::<IntegratedA>(&reply).await?;
                                    runner.send::<Feature<AUTH_A>>(authority).await?;
                                    runner.recv::<LastCiPassedA>().await?;
                                    runner.send::<BuildReturn>(build).await?;
                                } else {
                                    let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                                    authority.send::<IntegrationFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    authority.send::<CiFailedA>(&reply).await?;
                                    runner.recv::<CiFailedA>().await?;
                                    runner.send::<Feature<AUTH_A>>(authority).await?;
                                    let _owned_build = &build;
                                    core::future::pending::<ShellResult<()>>().await?;
                                }
                            }
                            MergeLoanB::LOGICAL_LABEL => {
                                let mut authority = branch.recv::<MergeLoanB>().await?;
                                let admitted =
                                    RepositoryArguments::from_bytes(if selected == FEATURE_A {
                                        br#"{"suite":"full_test_suite_after_auth"}"#
                                    } else {
                                        br#"{"suite":"full_test_suite_after_payment"}"#
                                    })?;
                                let requested = RepositoryArguments::from_bytes(
                                    if (selected == FEATURE_A) != (trial == 2) {
                                        br#"{"suite":"full_test_suite_after_auth"}"#
                                    } else {
                                        br#"{"suite":"full_test_suite_after_payment"}"#
                                    },
                                )?;
                                runner.send::<CiIntentB>(&admitted).await?;
                                async {
                                    async {
                                        authority.send::<RunCiOnFeatureB>(&admitted).await?;
                                    };
                                    async {
                                        build.send::<RunCiB>(&requested).await?;
                                    };
                                    Ok::<_, ShellError>(())
                                }
                                .await?;
                                let received = data::decode(authority.recv::<CiResult>().await?)?;
                                data::same(
                                    &received,
                                    &data::decode(build.recv::<CiResult>().await?)?,
                                )?;
                                actual.set(received.clone());
                                let bytes = data::encode(&received)?;
                                if received["success"] == true {
                                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                                    authority.send::<IntegratedB>(&reply).await?;
                                    runner.send::<Feature<AUTH_B>>(authority).await?;
                                    runner.recv::<LastCiPassedB>().await?;
                                    runner.send::<BuildReturn>(build).await?;
                                } else {
                                    let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                                    authority.send::<IntegrationFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    authority.send::<CiFailedB>(&reply).await?;
                                    runner.recv::<CiFailedB>().await?;
                                    runner.send::<Feature<AUTH_B>>(authority).await?;
                                    let _owned_build = &build;
                                    core::future::pending::<ShellResult<()>>().await?;
                                }
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "expected actual feature authority",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = author_a.offer().await?;
                        match branch.label() {
                            LastCiPassedA::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedA>().await?;
                            }
                            LastCiPassedB::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedB>().await?;
                            }
                            CiFailedA::LOGICAL_LABEL => {
                                branch.recv::<CiFailedA>().await?;
                                drop(author_a.recv::<Feature<AUTH_A>>().await?);
                                return Err(ShellError::message(
                                    "negative ownership prefix completed",
                                ));
                            }
                            CiFailedB::LOGICAL_LABEL => {
                                branch.recv::<CiFailedB>().await?;
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected receiver ownership feedback",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = author_b.offer().await?;
                        match branch.label() {
                            LastCiPassedA::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedA>().await?;
                            }
                            LastCiPassedB::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedB>().await?;
                            }
                            CiFailedA::LOGICAL_LABEL => {
                                branch.recv::<CiFailedA>().await?;
                            }
                            CiFailedB::LOGICAL_LABEL => {
                                branch.recv::<CiFailedB>().await?;
                                drop(author_b.recv::<Feature<AUTH_B>>().await?);
                                return Err(ShellError::message(
                                    "negative ownership prefix completed",
                                ));
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected receiver ownership feedback",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let bytes = data::encode(&tests_a)?;
                        writer.send::<TestsForCiA>(&bytes.as_slice()).await?;
                        let bytes = data::encode(&tests_b)?;
                        writer.send::<TestsForCiB>(&bytes.as_slice()).await?;
                        let branch = writer.offer().await?;
                        match branch.label() {
                            LastCiPassedA::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedA>().await?;
                            }
                            LastCiPassedB::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedB>().await?;
                            }
                            CiFailedA::LOGICAL_LABEL => {
                                branch.recv::<CiFailedA>().await?;
                            }
                            CiFailedB::LOGICAL_LABEL => {
                                branch.recv::<CiFailedB>().await?;
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected receiver ownership feedback",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = reviewer.offer().await?;
                        match branch.label() {
                            LastCiPassedA::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedA>().await?;
                            }
                            LastCiPassedB::LOGICAL_LABEL => {
                                branch.recv::<LastCiPassedB>().await?;
                            }
                            CiFailedA::LOGICAL_LABEL => {
                                branch.recv::<CiFailedA>().await?;
                            }
                            CiFailedB::LOGICAL_LABEL => {
                                branch.recv::<CiFailedB>().await?;
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected receiver ownership feedback",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = review_a.offer().await?;
                        match branch.label() {
                            CaseA::LOGICAL_LABEL => {
                                branch.recv::<CaseA>().await?;
                                let branch = review_a.offer().await?;
                                match branch.label() {
                                    LastCiPassedA::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedA>().await?;
                                    }
                                    LastCiPassedB::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedB>().await?;
                                    }
                                    CiFailedA::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedA>().await?;
                                    }
                                    CiFailedB::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedB>().await?;
                                    }
                                    _ => {
                                        return Err(ShellError::message(
                                            "unexpected receiver ownership feedback",
                                        ));
                                    }
                                }
                            }
                            CaseB::LOGICAL_LABEL => {
                                branch.recv::<CaseB>().await?;
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "expected the fixture's physical-resource choice",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = review_b.offer().await?;
                        match branch.label() {
                            CaseA::LOGICAL_LABEL => {
                                branch.recv::<CaseA>().await?;
                            }
                            CaseB::LOGICAL_LABEL => {
                                branch.recv::<CaseB>().await?;
                                let branch = review_b.offer().await?;
                                match branch.label() {
                                    LastCiPassedA::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedA>().await?;
                                    }
                                    LastCiPassedB::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedB>().await?;
                                    }
                                    CiFailedA::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedA>().await?;
                                    }
                                    CiFailedB::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedB>().await?;
                                    }
                                    _ => {
                                        return Err(ShellError::message(
                                            "unexpected receiver ownership feedback",
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "expected the fixture's physical-resource choice",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = data_a.offer().await?;
                        match branch.label() {
                            CaseA::LOGICAL_LABEL => {
                                branch.recv::<CaseA>().await?;
                                let branch = data_a.offer().await?;
                                match branch.label() {
                                    LastCiPassedA::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedA>().await?;
                                    }
                                    LastCiPassedB::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedB>().await?;
                                    }
                                    CiFailedA::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedA>().await?;
                                    }
                                    CiFailedB::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedB>().await?;
                                    }
                                    _ => {
                                        return Err(ShellError::message(
                                            "unexpected receiver ownership feedback",
                                        ));
                                    }
                                }
                            }
                            CaseB::LOGICAL_LABEL => {
                                branch.recv::<CaseB>().await?;
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "expected the fixture's physical-resource choice",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    async {
                        let branch = data_b.offer().await?;
                        match branch.label() {
                            CaseA::LOGICAL_LABEL => {
                                branch.recv::<CaseA>().await?;
                            }
                            CaseB::LOGICAL_LABEL => {
                                branch.recv::<CaseB>().await?;
                                let branch = data_b.offer().await?;
                                match branch.label() {
                                    LastCiPassedA::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedA>().await?;
                                    }
                                    LastCiPassedB::LOGICAL_LABEL => {
                                        branch.recv::<LastCiPassedB>().await?;
                                    }
                                    CiFailedA::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedA>().await?;
                                    }
                                    CiFailedB::LOGICAL_LABEL => {
                                        branch.recv::<CiFailedB>().await?;
                                    }
                                    _ => {
                                        return Err(ShellError::message(
                                            "unexpected receiver ownership feedback",
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "expected the fixture's physical-resource choice",
                                ));
                            }
                        }
                        Ok::<_, ShellError>(())
                    };
                    Ok::<_, ShellError>(())
                },
                deadline(core::time::Duration::from_secs(if cfg!(miri) {
                    120
                } else {
                    5
                })),
            )
            .await;
            match trial {
                0 => outcome?,
                1 => assert_eq!(
                    outcome.unwrap_err().to_string(),
                    "negative ownership prefix completed"
                ),
                2 => assert_eq!(
                    outcome.unwrap_err().to_string(),
                    "resource request differs from admitted arguments"
                ),
                3 => assert_eq!(
                    outcome.unwrap_err().to_string(),
                    "CI result differs from the admitted repository"
                ),
                _ => unreachable!(),
            }
            assert_eq!(effects.calls.get(), usize::from(trial != 2));
            eprintln!(
                "{}",
                json!({"feature":selected,"ownership_case":trial,"backend_calls":effects.calls.get(),"miri":cfg!(miri)})
            );
        }
    }
    Ok(())
}
