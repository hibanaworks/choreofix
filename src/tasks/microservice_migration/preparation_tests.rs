use super::*;
use crate::imp::transport::InMemoryTransport;
use core::{future::poll_fn, pin::pin, task::Poll};
use hibana::runtime::{
    SessionKitStorage, ids::SessionId, program::project, transport::EndpointBinding,
};
use serde_json::json;
use std::{cell::Cell, time::Duration};

#[test]
fn all_authored_dependency_graphs_are_checked_without_selecting_an_order() {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let possible_edges = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
    let mut accepted = 0;
    for graph in 0..1 << possible_edges.len() {
        let active = possible_edges
            .iter()
            .enumerate()
            .filter_map(|(bit, &edge)| (graph & (1 << bit) != 0).then_some(edge));
        let mut dependencies = [(); 3].map(|_| Vec::<&str>::new());
        for (service, requires) in active.clone() {
            dependencies[service].push(SERVICES[requires]);
        }
        let expected = orders.iter().any(|order| {
            active.clone().all(|(service, requires)| {
                order.iter().position(|&index| index == requires)
                    < order.iter().position(|&index| index == service)
            })
        });
        let value = json!({"title":"migration_plan","dependencies":{
            "microservice_a":dependencies[0],
            "microservice_b":dependencies[1],
            "microservice_c":dependencies[2],
        }});
        let actual = Arguments::<PLAN>::validate_payload(Payload::new(&encode(&value).unwrap()));
        assert_eq!(actual.is_ok(), expected, "{value}");
        accepted += usize::from(expected);
    }
    assert_eq!(accepted, 25);
    for bad in [
        json!({"title":"wrong"}),
        json!({"title":"migration_plan","dependencies":null}),
        json!({"title":"migration_plan","dependencies":{"microservice_a":["microservice_b","microservice_b"]}}),
        json!({"title":"migration_plan","dependencies":{"microservice_a":["microservice_a"]}}),
        json!({"title":"migration_plan","dependencies":{"microservice_a":["other"]}}),
        json!({"title":"migration_plan","dependencies":{"other":[]}}),
    ] {
        assert!(Arguments::<PLAN>::validate_payload(Payload::new(&encode(&bad).unwrap())).is_err());
    }
    assert!(
        Arguments::<PLAN>::validate_payload(Payload::new(br#"{"title":"migration_plan"}"#)).is_ok()
    );
    assert!(
        Arguments::<DEVELOPER_A>::validate_payload(Payload::new(
            br#"{"feature_name":"microservice_b"}"#
        ))
        .is_err()
    );
    assert!(
        Arguments::<SERVICE_A>::validate_payload(Payload::new(br#"{"pr_id":"microservice_b"}"#))
            .is_err()
    );
}

#[derive(Default)]
struct Journal(Cell<Vec<(&'static str, &'static str, Value)>>);

impl ReceivedEffects for Journal {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let arguments = decode(received.as_bytes());
        async move {
            let arguments = arguments?;
            let (agent, tool) = P::RPC;
            let mut entries = self.0.take();
            if tool == "run_tests" {
                let suite = &arguments["suite"];
                let candidate = entries
                    .iter()
                    .rposition(|(_, name, args)| {
                        *name == "implement_feature" && args["feature_name"] == *suite
                    })
                    .unwrap()
                    + 1;
                let review = entries
                    .iter()
                    .rposition(|(_, name, args)| *name == "code_review" && args["pr_id"] == *suite)
                    .unwrap()
                    + 1;
                entries.push((agent, tool, arguments.clone()));
                let receipt = entries.len();
                self.0.set(entries);
                let success = *suite != SERVICES[1];
                return Ok(json!({"tool_name":tool,"success":success,"data":{
                    "agent":agent, "suite":suite, "service":suite, "candidate":candidate,
                    "review":review, "plan":1, "receipt":receipt,
                    "status":if success {"tests passed"} else {"tests_failed"}
                }}));
            }
            let candidate = entries.iter().rposition(|(_, tool, args)| {
                *tool == "implement_feature" && args["feature_name"] == arguments["pr_id"]
            });
            let prior_reviews = entries
                .iter()
                .filter(|(_, tool, args)| {
                    *tool == "code_review" && args["pr_id"] == arguments["pr_id"]
                })
                .count();
            entries.push((agent, tool, arguments.clone()));
            let receipt = entries.len();
            self.0.set(entries);
            if tool == "design_proposal" {
                let expected = dependencies(arguments.get("dependencies"))?;
                return Ok(json!({"tool_name":tool,"success":true,"data":{
                    "agent":agent,"title":arguments["title"],"status":"proposed",
                    "receipt":receipt,"dependencies":expected,
                }}));
            }
            if tool == "migrate_schema" {
                let name = arguments["migration_name"].as_str().unwrap();
                return Ok(json!({"tool_name":tool,"success":true,"data":{
                    "agent":agent,"migration_name":name,"service":name.strip_suffix("_schema").unwrap(),
                    "status":"migrated","receipt":receipt,"plan":1,
                }}));
            }
            if tool == "code_review" {
                let rejected = arguments["pr_id"] == SERVICES[0] && prior_reviews < 2;
                let status = if rejected {
                    "changes_requested"
                } else {
                    "reviewed"
                };
                return Ok(json!({"tool_name":tool,"success":!rejected,"data":{
                    "agent":agent,"pr_id":arguments["pr_id"],"service":arguments["pr_id"],
                    "status":status,"receipt":receipt,"plan":1,
                    "candidate":candidate.expect("review after actual build") + 1,
                }}));
            }
            assert_eq!(tool, "implement_feature");
            // Simulated work completes only once all three independent effects start.
            poll_fn(|cx| {
                let entries = self.0.take();
                let started = entries
                    .iter()
                    .filter(|entry| entry.1 == "implement_feature")
                    .count();
                self.0.set(entries);
                if started >= 3 {
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await;
            Ok(json!({"tool_name":tool,"success":true,"data":{
                "agent":agent,"feature_name":arguments["feature_name"],"service":arguments["feature_name"],
                "status":"implemented","receipt":receipt,"plan":1,
            }}))
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn approved_services_share_one_test_environment_in_each_chosen_order() -> ShellResult<()> {
    let source = preparation();
    for (index, order) in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
    .into_iter()
    .enumerate()
    {
        let session = SessionId::new(404 + index as u32);
        let bindings = [SERVICE_A, SERVICE_B, SERVICE_C, TEST_ENV, STAGING_ENV].map(|role| {
            // SAFETY: each capability is freshly issued once for one distinct
            // service belonging only to this test execution.
            #[allow(unsafe_code)]
            let binding = unsafe { EndpointBinding::new() };
            (session, role, binding)
        });
        let transport = InMemoryTransport::new(bindings);
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport)?;
        let mut author = rv.enter(session, &project::<ARCHITECT_FILE>(&source))?;
        let mut architect_side = rv.enter(session, &project::<ARCHITECT>(&source))?;
        let staging = rv.enter(session, &project::<STAGING_ENV>(&source))?;
        let mut a = rv.enter(session, &project::<DEVELOPER_A>(&source))?;
        let mut b = rv.enter(session, &project::<DEVELOPER_B>(&source))?;
        let mut c = rv.enter(session, &project::<DEVELOPER_C>(&source))?;
        let mut a_file = rv.enter(session, &project::<DEVELOPER_A_FILE>(&source))?;
        let mut b_file = rv.enter(session, &project::<DEVELOPER_B_FILE>(&source))?;
        let mut c_file = rv.enter(session, &project::<DEVELOPER_C_FILE>(&source))?;
        let mut review_a = rv.enter(session, &project::<REVIEWER_A>(&source))?;
        let mut review_b = rv.enter(session, &project::<REVIEWER_B>(&source))?;
        let mut review_c = rv.enter(session, &project::<REVIEWER_C>(&source))?;
        let mut review_file = rv.enter(session, &project::<REVIEW_FILE>(&source))?;
        let mut testing_slab = vec![0; 128 * 1024];
        let mut testing_storage = SessionKitStorage::uninit();
        let testing = testing_storage
            .init()
            .rendezvous(&mut testing_slab, &transport)?;
        let mut schema_side = testing.enter(session, &project::<SCHEMAS>(&source))?;
        let mut devops_file = testing.enter(session, &project::<DEVOPS_FILE>(&source))?;
        let mut test_side_a = testing.enter(session, &project::<TESTER_A>(&source))?;
        let mut test_side_b = testing.enter(session, &project::<TESTER_B>(&source))?;
        let mut test_side_c = testing.enter(session, &project::<TESTER_C>(&source))?;
        let mut env_owner = testing.enter(session, &project::<ENV_OWNER>(&source))?;
        let environment = testing.enter(session, &project::<TEST_ENV>(&source))?;
        let mut test_command = rv.enter(session, &project::<TEST_COMMAND>(&source))?;
        let mut test_file = rv.enter(session, &project::<TEST_FILE>(&source))?;
        let service_a = rv.enter(session, &project::<SERVICE_A>(&source))?;
        let service_b = rv.enter(session, &project::<SERVICE_B>(&source))?;
        let service_c = rv.enter(session, &project::<SERVICE_C>(&source))?;
        let journal = Journal::default();
        let a_args = Arguments(br#"{"feature_name":"microservice_a"}"#.to_vec());
        {
            let mut early = pin!(a_file.send::<Implement<DEVELOPER_A>>(&a_args));
            let result = poll_fn(|cx| Poll::Ready(early.as_mut().poll(cx))).await;
            assert!(
                matches!(result, Poll::Ready(Err(error)) if error.is_unavailable()),
                "{result:?}"
            );
        }
        {
            let arguments = Arguments(br#"{"pr_id":"microservice_a"}"#.to_vec());
            let mut early = pin!(review_file.send::<Review<SERVICE_A>>(&arguments));
            let result = poll_fn(|cx| Poll::Ready(early.as_mut().poll(cx))).await;
            assert!(
                matches!(result, Poll::Ready(Err(error)) if error.is_unavailable()),
                "{result:?}"
            );
        }
        assert_eq!(transport.take_observations().len(), 0);
        let authored = json!({"title":"migration_plan","dependencies":{
            "microservice_a":["microservice_c"], "microservice_b":["microservice_a"],
        }});
        let expected = json!({"microservice_a":["microservice_c"],"microservice_b":["microservice_a"],"microservice_c":[]});
        let [mut test_a, mut test_b, mut test_c] = test_file.par()?;
        let check_review = |receipt: Value, service: &str| -> ShellResult<()> {
            assert_eq!(receipt["data"]["service"], service);
            assert_eq!(receipt["data"]["status"], "reviewed");
            let review = usize::try_from(receipt["data"]["receipt"].as_u64().unwrap()).unwrap() - 1;
            let implementation =
                usize::try_from(receipt["data"]["candidate"].as_u64().unwrap()).unwrap() - 1;
            let entries = journal.0.take();
            assert_eq!(entries[review].1, "code_review");
            assert_eq!(entries[review].2["pr_id"], service);
            assert_eq!(
                entries[..review]
                    .iter()
                    .rposition(|(_, tool, args)| *tool == "implement_feature"
                        && args["feature_name"] == service),
                Some(implementation)
            );
            journal.0.set(entries);
            Ok(())
        };
        hibana_runtime::race(
        async {
            async { architect(&mut architect_side, staging, &journal).await };
            async { schemas(&mut schema_side, &journal).await };
            async {
                let plan = decode(devops_file.recv::<Proposed<DEVOPS_FILE>>().await?)?;
                for index in order {
                    let args=encode(&json!({"migration_name":format!("{}_schema", SERVICES[index])}))?;
                    let reply = match index {
                        0 => {devops_file.send::<Migrate<SCHEMA_A>>(&Arguments(args)).await?;
                              decode(devops_file.recv::<Migrated<SCHEMA_A>>().await?)?}
                        1 => {devops_file.send::<Migrate<SCHEMA_B>>(&Arguments(args)).await?;
                              decode(devops_file.recv::<Migrated<SCHEMA_B>>().await?)?}
                        2 => {devops_file.send::<Migrate<SCHEMA_C>>(&Arguments(args)).await?;
                              decode(devops_file.recv::<Migrated<SCHEMA_C>>().await?)?}
                        _ => unreachable!(),
                    };
                    assert_eq!(reply["data"]["service"],SERVICES[index]);
                    assert_eq!(reply["data"]["plan"],plan["data"]["receipt"]);
                }
                Ok(())
            };
            async { developer(&mut a, service_a, &journal).await };
            async { developer(&mut b, service_b, &journal).await };
            async { developer(&mut c, service_c, &journal).await };
            async { reviewer::<REVIEWER_A, DEVELOPER_A, SERVICE_A>(&mut review_a, &journal).await };
            async { reviewer::<REVIEWER_B, DEVELOPER_B, SERVICE_B>(&mut review_b, &journal).await };
            async { reviewer::<REVIEWER_C, DEVELOPER_C, SERVICE_C>(&mut review_c, &journal).await };
            async {
                let report =
                    tester::<TESTER_A, SERVICE_A>(&mut test_side_a, &journal).await?;
                assert_eq!(report["success"], true);
            };
            async {
                let report =
                    tester::<TESTER_B, SERVICE_B>(&mut test_side_b, &journal).await?;
                assert_eq!(report["success"], false);
            };
            async {
                let report =
                    tester::<TESTER_C, SERVICE_C>(&mut test_side_c, &journal).await?;
                assert_eq!(report["success"], true);
            };
            async {
                let _environment = test_environment(&mut env_owner, environment).await?;
            };
            async {
                check_review(
                    decode(test_a.recv::<TestCandidate<SERVICE_A>>().await?)?,
                    SERVICES[0],
                )
            };
            async {
                check_review(decode(test_b.recv::<TestCandidate<SERVICE_B>>().await?)?, SERVICES[1])?;
            };
            async {
                author
                    .send::<Propose>(&Arguments(encode(&authored)?))
                    .await?;
                let result = decode(author.recv::<Proposed<ARCHITECT_FILE>>().await?)?;
                same(&result["data"]["dependencies"], &expected)
            };
            async {
                let plan = decode(a_file.recv::<Proposed<DEVELOPER_A>>().await?)?;
                same(&plan["data"]["dependencies"], &expected)?;
                for (attempt, accepted) in [false, false, true].into_iter().enumerate() {
                    a_file.send::<Implement<DEVELOPER_A>>(&a_args).await?;
                    let built = decode(a_file.recv::<Implemented>().await?)?;
                    assert_eq!(built["data"]["service"], SERVICES[0]);
                    same(&built["data"]["plan"], &plan["data"]["receipt"])?;
                    let reviewed = decode(a_file.recv::<Reviewed<DEVELOPER_A>>().await?)?;
                    assert_eq!(reviewed["success"], accepted);
                    same(&reviewed["data"]["candidate"], &built["data"]["receipt"])?;
                    // Leave A rejected until C's real Endpoint has reached the tester
                    // and delivered C's approval. A join across services would stall.
                    if attempt == 0 {
                        check_review(
                            decode(test_c.recv::<TestCandidate<SERVICE_C>>().await?)?,
                            SERVICES[2],
                        )?;
                        if order[0] != 2 { continue; }
                        // C finishes testing while A's next implementation is still withheld.
                        test_command
                            .send::<RunTests<TESTER_C>>(&Arguments(encode(
                                &json!({"suite":SERVICES[2]}),
                            )?))
                            .await?;
                        transport.take_observations().for_each(drop);
                        {
                            let other = Arguments(encode(&json!({"suite":SERVICES[1]}))?);
                            let mut overlapping = pin!(test_command.send::<RunTests<TESTER_B>>(&other));
                            let second = poll_fn(|cx| Poll::Ready(overlapping.as_mut().poll(cx))).await;
                            assert!(matches!(second, Poll::Ready(Err(error)) if error.is_unavailable()), "{second:?}");
                            assert_eq!(transport.take_observations().len(), 0);
                        }
                        let report = decode(test_command.recv::<Tested<TESTER_C>>().await?)?;
                        assert_eq!(report["data"]["suite"], SERVICES[2]);
                        assert_eq!(report["success"], true);
                    }
                }
                for &service in &order[usize::from(order[0] == 2)..] {
                    let bytes = encode(&json!({"suite": SERVICES[service]}))?;
                    let report = match service {
                        0 => {
                            test_command.send::<RunTests<TESTER_A>>(&Arguments(bytes)).await?;
                            decode(test_command.recv::<Tested<TESTER_A>>().await?)?
                        }
                        1 => {
                            test_command.send::<RunTests<TESTER_B>>(&Arguments(bytes)).await?;
                            decode(test_command.recv::<Tested<TESTER_B>>().await?)?
                        }
                        2 => {
                            test_command.send::<RunTests<TESTER_C>>(&Arguments(bytes)).await?;
                            decode(test_command.recv::<Tested<TESTER_C>>().await?)?
                        }
                        _ => unreachable!(),
                    };
                    assert_eq!(report["data"]["suite"], SERVICES[service]);
                    assert_eq!(report["success"], service != 1);
                }
                let tested = decode(a_file.recv::<TestReport>().await?)?;
                assert_eq!(tested["data"]["service"], SERVICES[0]);
                assert_eq!(tested["success"], true);
                Ok(())
            };
            async {
                let plan = decode(b_file.recv::<Proposed<DEVELOPER_B>>().await?)?;
                same(&plan["data"]["dependencies"], &expected)?;
                b_file
                    .send::<Implement<DEVELOPER_B>>(&Arguments(
                        br#"{"feature_name":"microservice_b"}"#.to_vec(),
                    ))
                    .await?;
                let built = decode(b_file.recv::<Implemented>().await?)?;
                assert_eq!(built["data"]["service"], SERVICES[1]);
                same(&built["data"]["plan"], &plan["data"]["receipt"])?;
                let reviewed = decode(b_file.recv::<Reviewed<DEVELOPER_B>>().await?)?;
                assert_eq!(reviewed["success"], true);
                same(&reviewed["data"]["candidate"], &built["data"]["receipt"])?;
                let failed = decode(b_file.recv::<TestReport>().await?)?;
                assert_eq!(failed["success"], false);
                same(&failed["data"]["review"], &reviewed["data"]["receipt"])?;
                same(&failed["data"]["candidate"], &built["data"]["receipt"])?;
                b_file.send::<Implement<DEVELOPER_B>>(&Arguments(encode(&json!({"feature_name": SERVICES[1]}))?)).await?;
                let repaired = decode(b_file.recv::<Implemented>().await?)?;
                assert_eq!(repaired["data"]["service"], SERVICES[1]);
                assert_ne!(repaired["data"]["receipt"], failed["data"]["candidate"]);
                same(&repaired["data"]["plan"], &failed["data"]["plan"])?;
                let reviewed = decode(b_file.recv::<Reviewed<DEVELOPER_B>>().await?)?;
                check_review(reviewed, SERVICES[1])
            };
            async {
                let plan = decode(c_file.recv::<Proposed<DEVELOPER_C>>().await?)?;
                same(&plan["data"]["dependencies"], &expected)?;
                c_file
                    .send::<Implement<DEVELOPER_C>>(&Arguments(
                        br#"{"feature_name":"microservice_c"}"#.to_vec(),
                    ))
                    .await?;
                let built = decode(c_file.recv::<Implemented>().await?)?;
                assert_eq!(built["data"]["service"], SERVICES[2]);
                same(&built["data"]["plan"], &plan["data"]["receipt"])?;
                let reviewed = decode(c_file.recv::<Reviewed<DEVELOPER_C>>().await?)?;
                assert_eq!(reviewed["success"], true);
                same(&reviewed["data"]["candidate"], &built["data"]["receipt"])?;
                let tested = decode(c_file.recv::<TestReport>().await?)?;
                assert_eq!(tested["success"], true);
                same(&tested["data"]["candidate"], &built["data"]["receipt"])
            };
            async {
                async {
                    let [mut a, mut b, mut c] = review_file.par()?;
                    async {
                        for accepted in [false, false, true] {
                            let candidate = decode(a.recv::<Candidate<SERVICE_A>>().await?)?;
                            a.send::<Review<SERVICE_A>>(&Arguments(
                                br#"{"pr_id":"microservice_a"}"#.to_vec(),
                            ))
                            .await?;
                            let review = decode(a.recv::<Reviewed<DEVELOPER_A>>().await?)?;
                            assert_eq!(review["success"], accepted);
                            same(&review["data"]["candidate"], &candidate["data"]["receipt"])?;
                        }
                        Ok(())
                    };
                    async {
                        let candidate = decode(b.recv::<Candidate<SERVICE_B>>().await?)?;
                        b.send::<Review<SERVICE_B>>(&Arguments(
                            br#"{"pr_id":"microservice_b"}"#.to_vec(),
                        ))
                        .await?;
                        let review = decode(b.recv::<Reviewed<DEVELOPER_B>>().await?)?;
                        assert_eq!(review["success"], true);
                        same(&review["data"]["candidate"], &candidate["data"]["receipt"])
                    };
                    async {
                        let candidate = decode(c.recv::<Candidate<SERVICE_C>>().await?)?;
                        c.send::<Review<SERVICE_C>>(&Arguments(
                            br#"{"pr_id":"microservice_c"}"#.to_vec(),
                        ))
                        .await?;
                        let review = decode(c.recv::<Reviewed<DEVELOPER_C>>().await?)?;
                        assert_eq!(review["success"], true);
                        same(&review["data"]["candidate"], &candidate["data"]["receipt"])
                    };
                    Ok::<(), ShellError>(())
                }.await?;
                let repaired = decode(review_file.recv::<Candidate<SERVICE_B>>().await?)?;
                assert_eq!(repaired["data"]["service"], SERVICES[1]);
                assert_eq!(repaired["data"]["agent"], "DEVELOPER_B");
                review_file.send::<Review<SERVICE_B>>(&Arguments(encode(&json!({"pr_id":SERVICES[1]}))?)).await?;
                let reviewed = decode(review_file.recv::<Reviewed<DEVELOPER_B>>().await?)?;
                assert_eq!(reviewed["data"]["candidate"], repaired["data"]["receipt"]);
                check_review(reviewed, SERVICES[1])?;
                let a = decode(review_file.recv::<Candidate<SERVICE_A>>().await?)?;
                let c = decode(review_file.recv::<Candidate<SERVICE_C>>().await?)?;
                assert_eq!(a["data"]["status"], "tests passed");
                assert_eq!(c["data"]["status"], "tests passed");
                Ok(())
            };
            Ok::<(), ShellError>(())
        },
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
    .await?;
        transport.take_observations().for_each(drop);
        let mut replay = pin!(a_file.send::<Implement<DEVELOPER_A>>(&a_args));
        let result = poll_fn(|cx| Poll::Ready(replay.as_mut().poll(cx))).await;
        assert!(
            matches!(result, Poll::Ready(Err(error)) if error.is_unavailable()),
            "an approved service must not acquire a new build: {result:?}"
        );
        assert_eq!(transport.take_observations().len(), 0);
        let b_args = Arguments(br#"{"feature_name":"microservice_b"}"#.to_vec());
        let mut replay = pin!(b_file.send::<Implement<DEVELOPER_B>>(&b_args));
        let result = poll_fn(|cx| Poll::Ready(replay.as_mut().poll(cx))).await;
        assert!(
            matches!(result, Poll::Ready(Err(error)) if error.is_unavailable()),
            "the developer cannot edit the newly approved repair: {result:?}"
        );
        assert_eq!(transport.take_observations().len(), 0);
        let mut calls = journal
            .0
            .into_inner()
            .into_iter()
            .map(|(agent, tool, _)| (agent, tool))
            .collect::<Vec<_>>();
        assert_eq!(calls.remove(0), ("ARCHITECT", "design_proposal"));

        calls.sort_unstable();
        assert_eq!(
            calls,
            [
                ("DEVELOPER_A", "implement_feature"),
                ("DEVELOPER_A", "implement_feature"),
                ("DEVELOPER_A", "implement_feature"),
                ("DEVELOPER_B", "implement_feature"),
                ("DEVELOPER_B", "implement_feature"),
                ("DEVELOPER_C", "implement_feature"),
                ("DEVOPS", "migrate_schema"),
                ("DEVOPS", "migrate_schema"),
                ("DEVOPS", "migrate_schema"),
                ("REVIEWER", "code_review"),
                ("REVIEWER", "code_review"),
                ("REVIEWER", "code_review"),
                ("REVIEWER", "code_review"),
                ("REVIEWER", "code_review"),
                ("REVIEWER", "code_review"),
                ("TESTER", "run_tests"),
                ("TESTER", "run_tests"),
                ("TESTER", "run_tests"),
            ]
        );
    }
    Ok(())
}
