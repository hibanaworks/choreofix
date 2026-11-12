use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    time::{Duration, Instant},
};

// Controlled native effect fixture, not AI or the original simulator. This
// append-only evidence is never read by a participant to authorize a step.
struct ActualEffects {
    started: Instant,
    journal: Cell<Vec<Value>>,
}
impl Domain for ActualEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let start = self.started.elapsed().as_nanos() as u64;
        let mut journal = self.journal.take();
        let id = journal.len();
        journal
            .push(json!({"id":id,"agent":agent,"tool":tool,"arguments":arguments,"start":start}));
        self.journal.set(journal);
        // Native scheduling fixture only: each held effect waits for the
        // independent effect's actual start. Serialization reaches the deadline.
        // These controlled witnesses are not live-model timing measurements.
        let waiting_for = {
            let journal = self.journal.take();
            let prior = |name: &str| journal[..id].iter().filter(|e| e["tool"] == name).count();
            let peer = match tool {
                "code_review" if prior("code_review") == 0 => Some(("run_tests", 3)),
                "code_review" => Some(("merge_code", 1)),
                "merge_code" if prior("merge_code") == 0 => Some(("code_review", 2)),
                "run_tests" if prior("run_tests") == 2 => Some(("code_review", 1)),
                _ => None,
            };
            self.journal.set(journal);
            peer
        };
        if let Some((peer, required)) = waiting_for {
            std::future::poll_fn(|cx| {
                let journal = self.journal.take();
                let started = journal.iter().filter(|e| e["tool"] == peer).count() >= required;
                self.journal.set(journal);
                if started {
                    core::task::Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    core::task::Poll::Pending
                }
            })
            .await;
        }
        hibana_runtime::host::sleep(Duration::from_millis(2))
            .await
            .map_err(|e| e.to_string())?;
        let result =
            json!({"success":true,"agent":agent,"tool":tool,"arguments":arguments,"effect":id});
        let mut journal = self.journal.take();
        journal[id]["end"] = json!(self.started.elapsed().as_nanos() as u64);
        journal[id]["result"] = result.clone();
        self.journal.set(journal);
        Ok(result)
    }
}

#[hibana_runtime::main]
#[test]
async fn all_design_refactor_review_and_suite_orders_use_real_affine_endpoints() -> ShellResult<()>
{
    let programs = programs();
    let mut evidence = Vec::new();
    for choices in 0..64u32 {
        let session = SessionId::new(40000 + choices);
        let (bus, _reader, _terminals) = Bus::open(
            [
                "ARCHITECT",
                "DEVELOPER_A",
                "DEVELOPER_B",
                "REVIEWER",
                "TESTER",
            ],
            [(session, REPO), (session, TEST_ENV)],
        );
        let effects = ActualEffects {
            started: Instant::now(),
            journal: Cell::new(Vec::new()),
        };
        let primary = Cell::new(Vec::new());
        let secondary = Cell::new(Vec::new());
        let owner_capture = |event| {
            let mut records = primary.take();
            records.push(event);
            primary.set(records);
        };
        let caller_capture = |event| {
            let mut records = secondary.take();
            records.push(event);
            secondary.set(records);
        };
        let mut owner_slab = vec![0; 1024 * 1024];
        let mut owner_store = SessionKitStorage::uninit();
        let owner = owner_store
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut caller_store = SessionKitStorage::uninit();
        let caller = caller_store
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        owner.observe(&owner_capture);
        caller.observe(&caller_capture);
        let mut store = owner.enter(session, &programs.store)?;
        let mut review_owner = owner.enter(session, &programs.review_service)?;
        let repo = owner.enter(session, &programs.repo)?;
        let env = owner.enter(session, &programs.test_env)?;
        let mut architect = caller.enter(session, &programs.architect)?;
        let mut a = caller.enter(session, &programs.a)?;
        let mut b = caller.enter(session, &programs.b)?;
        let mut reviewer = caller.enter(session, &programs.reviewer)?;
        let mut tester = caller.enter(session, &programs.tester)?;
        let [mut architect, mut architect_private] = architect.par()?;
        let [mut a, mut a_private] = a.par()?;
        let [mut b, mut b_private] = b.par()?;
        let [mut reviewer, mut reviewer_private] = reviewer.par()?;
        let [mut tester, mut tester_private] = tester.par()?;
        let mut architect_io = owner.enter(session, &programs.architect_io)?;
        let mut a_io = owner.enter(session, &programs.a_io)?;
        let mut b_io = owner.enter(session, &programs.b_io)?;
        let mut reviewer_io = owner.enter(session, &programs.reviewer_io)?;
        let mut tester_io = owner.enter(session, &programs.tester_io)?;
        let run = hibana_runtime::race(
            async {
                async {
                    localside::resources::repository(&mut store, (repo, env), &effects).await?;
                };
                async {
                    localside::reviewer::run(&mut review_owner, &effects).await?;
                };
                async {
                    if choices & 1 == 0 {
                        architect
                            .send::<DesignA>(&br#"{"title":"module_a_refactor"}"#.as_slice())
                            .await?;
                        imp::successful(&imp::decode(architect.recv::<DesignResult>().await?)?)?;
                        architect
                            .send::<DesignB>(&br#"{"title":"module_b_refactor"}"#.as_slice())
                            .await?;
                    } else {
                        architect
                            .send::<DesignB>(&br#"{"title":"module_b_refactor"}"#.as_slice())
                            .await?;
                        imp::successful(&imp::decode(architect.recv::<DesignResult>().await?)?)?;
                        architect
                            .send::<DesignA>(&br#"{"title":"module_a_refactor"}"#.as_slice())
                            .await?;
                    }
                    imp::successful(&imp::decode(architect.recv::<DesignResult>().await?)?)?;
                };
                async {
                    imp::successful(&imp::decode(a.recv::<DesignResult>().await?)?)?;
                    let mut repo = a.recv::<RepoLoan>().await?;
                    assert!(caller.enter(session, &programs.repo).is_err());
                    for target in if choices & 2 == 0 {
                        ["module_a", "SHARED_LIBRARY"]
                    } else {
                        ["SHARED_LIBRARY", "module_a"]
                    } {
                        let arguments = imp::encode(&json!({"target":target}))?;
                        if target == "SHARED_LIBRARY" {
                            a.send::<SharedIntent>(&arguments.as_slice()).await?;
                        } else {
                            a.send::<OwnIntent>(&arguments.as_slice()).await?;
                        }
                        repo.send::<Refactor>(&arguments.as_slice()).await?;
                        imp::successful(&imp::decode(repo.recv::<Refactored>().await?)?)?;
                    }
                    a.send::<RepoLoan>(repo).await?;
                    let tests = imp::decode(a.recv::<TestSummary>().await?)?;
                    let review = imp::decode(a.recv::<Approved>().await?)?;
                    imp::successful(&review["result"])?;
                    let ready = imp::decode(a.recv::<MergeReady>().await?)?;
                    imp::same(&ready["tests"], &tests)?;
                    imp::same(&ready["review"], &review)?;
                    let mut repo = a.recv::<RepoLoan>().await?;
                    let arguments = json!({"branch":"module_a"});
                    let bytes = imp::encode(&arguments)?;
                    a.send::<MergeIntent>(&bytes.as_slice()).await?;
                    let request = imp::encode(&json!({"arguments":arguments,"checked":ready}))?;
                    repo.send::<MergeRequest>(&request.as_slice()).await?;
                    imp::successful(&imp::decode(repo.recv::<Merged>().await?)?)?;
                    a.send::<RepoLoan>(repo).await?;
                };
                async {
                    imp::successful(&imp::decode(b.recv::<DesignResult>().await?)?)?;
                    let mut repo = b.recv::<RepoLoan>().await?;
                    assert!(caller.enter(session, &programs.repo).is_err());
                    for target in if choices & 4 == 0 {
                        ["module_b", "SHARED_LIBRARY"]
                    } else {
                        ["SHARED_LIBRARY", "module_b"]
                    } {
                        let arguments = imp::encode(&json!({"target":target}))?;
                        if target == "SHARED_LIBRARY" {
                            b.send::<SharedIntent>(&arguments.as_slice()).await?;
                        } else {
                            b.send::<OwnIntent>(&arguments.as_slice()).await?;
                        }
                        repo.send::<Refactor>(&arguments.as_slice()).await?;
                        imp::successful(&imp::decode(repo.recv::<Refactored>().await?)?)?;
                    }
                    b.send::<RepoLoan>(repo).await?;
                    let tests = imp::decode(b.recv::<TestSummaryB>().await?)?;
                    let review = imp::decode(b.recv::<Approved>().await?)?;
                    imp::successful(&review["result"])?;
                    let ready = imp::decode(b.recv::<MergeReady>().await?)?;
                    imp::same(&ready["tests"], &tests)?;
                    imp::same(&ready["review"], &review)?;
                    let mut repo = b.recv::<RepoLoan>().await?;
                    let arguments = json!({"branch":"module_b"});
                    let bytes = imp::encode(&arguments)?;
                    b.send::<MergeIntent>(&bytes.as_slice()).await?;
                    let request = imp::encode(&json!({"arguments":arguments,"checked":ready}))?;
                    repo.send::<MergeRequest>(&request.as_slice()).await?;
                    imp::successful(&imp::decode(repo.recv::<Merged>().await?)?)?;
                    b.send::<RepoLoan>(repo).await?;
                };
                async {
                    let candidate = imp::decode(reviewer.recv::<Candidate>().await?)?;
                    for target in if choices & 8 == 0 {
                        ["module_a", "module_b"]
                    } else {
                        ["module_b", "module_a"]
                    } {
                        let bytes = imp::encode(&json!({"pr_id":target}))?;
                        if target == "module_a" {
                            reviewer.send::<ReviewA>(&bytes.as_slice()).await?;
                        } else {
                            reviewer.send::<ReviewB>(&bytes.as_slice()).await?;
                        }
                        let reply = imp::decode(reviewer.recv::<Approved>().await?)?;
                        imp::same(&reply["candidate"], &candidate)?;
                        imp::same(&reply["arguments"], &json!({"pr_id":target}))?;
                    }
                };
                async {
                    for order in [choices & 16 != 0, choices & 32 != 0] {
                        let candidate = imp::decode(tester.recv::<TestPlan>().await?)?;
                        let (mut repo, mut environment) = async {
                            let [mut r, mut e] = tester.par()?;
                            async { Ok::<_, ShellError>(r.recv::<RepoLoan>().await?) };
                            async { Ok::<_, ShellError>(e.recv::<TestLoan>().await?) };
                        }
                        .await?;
                        assert!(caller.enter(session, &programs.repo).is_err());
                        assert!(owner.enter(session, &programs.test_env).is_err());
                        for suite in if order {
                            ["module_b_tests", "module_a_tests"]
                        } else {
                            ["module_a_tests", "module_b_tests"]
                        } {
                            let arguments = json!({"suite":suite});
                            let bytes = imp::encode(&arguments)?;
                            if suite == "module_a_tests" {
                                tester.send::<TestA>(&bytes.as_slice()).await?;
                            } else {
                                tester.send::<TestB>(&bytes.as_slice()).await?;
                            }
                            let request =
                                imp::encode(&json!({"candidate":candidate,"arguments":arguments}))?;
                            let (read, tested) = async {
                                async {
                                    repo.send::<ReadSnapshot>(&request.as_slice()).await?;
                                    imp::decode(repo.recv::<RepositoryTested>().await?)
                                };
                                async {
                                    environment.send::<RunTests>(&request.as_slice()).await?;
                                    imp::decode(environment.recv::<EnvironmentTested>().await?)
                                };
                            }
                            .await?;
                            imp::same(&read, &tested)?;
                            imp::same(&read["candidate"], &candidate)?;
                            imp::successful(&read["result"])?;
                        }
                        async {
                            let [mut r, mut e] = tester.par()?;
                            async {
                                r.send::<RepoLoan>(repo).await?;
                            };
                            async {
                                e.send::<TestLoan>(environment).await?;
                            };
                            Ok(())
                        }
                        .await?;
                    }
                };
                Ok::<_, ShellError>(())
            },
            hibana_runtime::host::deadline(Duration::from_secs(10)),
        )
        .await;
        if let Err(error) = run {
            for observation in bus
                .transport
                .take_observations()
                .iter()
                .rev()
                .take(30)
                .rev()
            {
                eprintln!("{observation}");
            }
            return Err(ShellError::message(format!(
                "4M choices {choices}: {error}"
            )));
        }
        async {
            async {
                architect_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                a_private.send::<crate::shell::global::Exit>(&0).await?;
            };
            async {
                b_private.send::<crate::shell::global::Exit>(&0).await?;
            };
            async {
                reviewer_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                tester_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                assert_eq!(architect_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(a_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(b_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(reviewer_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(tester_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            Ok(())
        }
        .await?;
        let journal = effects.journal.take();
        if let Some(directory) = std::env::var_os("HIBANA_4M_EVIDENCE") {
            std::fs::create_dir_all(&directory).map_err(|e| ShellError::message(e.to_string()))?;
            let primary_records = primary.take();
            let secondary_records = secondary.take();
            let raw = json!({"task":"4M","choices":choices,"native_fixture":true,"controlled_effect_start_barriers":true,"session":session.raw(),"effects":journal,
              "capture":[primary_records.iter().copied().map(crate::imp::projection::record).collect::<Vec<_>>(),secondary_records.iter().copied().map(crate::imp::projection::record).collect::<Vec<_>>()],
              "checkpoints":[owner.tap().checkpoint(),caller.tap().checkpoint()]});
            std::fs::write(
                std::path::Path::new(&directory).join(format!("condition-{choices}.json")),
                serde_json::to_vec(&raw).unwrap(),
            )
            .map_err(|e| ShellError::message(e.to_string()))?;
            primary.set(primary_records);
            secondary.set(secondary_records);
        }

        assert_eq!(journal.len(), 14);
        let position = |agent: &str, tool: &str| {
            journal
                .iter()
                .position(|e| e["agent"] == agent && e["tool"] == tool)
                .unwrap()
        };
        for (agent, proposal) in [
            ("DEVELOPER_A", "module_a_refactor"),
            ("DEVELOPER_B", "module_b_refactor"),
        ] {
            let designed = journal
                .iter()
                .find(|e| e["tool"] == "design_proposal" && e["arguments"]["title"] == proposal)
                .unwrap();
            let writes: Vec<_> = journal
                .iter()
                .filter(|e| e["agent"] == agent && e["tool"] == "refactor_code")
                .collect();
            assert_eq!(writes.len(), 2);
            assert!(
                writes
                    .iter()
                    .all(|e| designed["end"].as_u64() < e["start"].as_u64())
            );
            let merge = &journal[position(agent, "merge_code")];
            let target = if agent == "DEVELOPER_A" {
                "module_a"
            } else {
                "module_b"
            };
            let review = journal
                .iter()
                .find(|e| e["tool"] == "code_review" && e["arguments"]["pr_id"] == target)
                .unwrap();
            assert!(review["end"].as_u64() < merge["start"].as_u64());
        }
        for shared in journal.iter().filter(|e| {
            e["tool"] == "refactor_code" && e["arguments"]["target"] == "SHARED_LIBRARY"
        }) {
            let next_shared = journal
                .iter()
                .filter(|e| {
                    e["tool"] == "refactor_code"
                        && e["arguments"]["target"] == "SHARED_LIBRARY"
                        && e["start"].as_u64() > shared["end"].as_u64()
                })
                .map(|e| e["start"].as_u64().unwrap())
                .min()
                .unwrap_or(u64::MAX);
            for suite in ["module_a_tests", "module_b_tests"] {
                assert!(journal.iter().any(|e| e["tool"] == "run_tests"
                    && e["arguments"]["suite"] == suite
                    && e["start"].as_u64() > shared["end"].as_u64()
                    && e["end"].as_u64().unwrap() < next_shared));
            }
        }
        let overlap = |left: &Value, right: &Value| {
            left["start"].as_u64() < right["end"].as_u64()
                && right["start"].as_u64() < left["end"].as_u64()
        };
        assert!(
            journal
                .iter()
                .filter(|e| e["tool"] == "code_review")
                .any(|r| journal
                    .iter()
                    .filter(|e| e["tool"] == "run_tests")
                    .any(|t| overlap(r, t))),
            "actual native review/test intervals must overlap"
        );
        assert!(
            journal
                .iter()
                .filter(|e| e["tool"] == "code_review")
                .any(|r| journal
                    .iter()
                    .filter(|e| e["tool"] == "merge_code")
                    .any(|t| overlap(r, t))),
            "actual native merge/review intervals must overlap"
        );
        let records = [primary.take(), secondary.take()];
        let checkpoints = [owner.tap().checkpoint(), caller.tap().checkpoint()];
        use hibana_epf::{
            Acceptance, AuditStorage, Capture, ExecutionStorage, Protocol, RecordId, RouteChoice,
            Unfold,
        };
        let captures = [
            Capture::new(&records[0], checkpoints[0]).unwrap(),
            Capture::new(&records[1], checkpoints[1]).unwrap(),
        ];
        let snapshot: Vec<_> = hibana::runtime::tap::projected(&programs.store).collect();
        let mut source = vec![0; Protocol::workspace_len(&snapshot).unwrap()];
        let protocol = Protocol::new(&snapshot, &mut source).unwrap();
        let count: usize = captures.iter().map(|c| c.records().len()).sum();
        let capacity = count * 2 + 1;
        let mut unfolding = vec![Unfold::Source; capacity];
        let mut routes = vec![
            RouteChoice {
                span: [0, 0],
                right: false
            };
            protocol.audit_choice_capacity(capacity).unwrap()
        ];
        let mut order = vec![
            RecordId {
                producer: 0,
                ordinal: 0
            };
            count
        ];
        let mut occurrences = vec![None; count];
        let mut acceptances = vec![
            Acceptance {
                first_poll: RecordId {
                    producer: 0,
                    ordinal: 0
                },
                before: 0
            };
            count
        ];
        let mut scratch = vec![0; protocol.audit_workspace_len(&captures, capacity).unwrap()];
        let observed_producers: Vec<_> = captures
            .iter()
            .enumerate()
            .flat_map(|(producer, capture)| std::iter::repeat_n(producer, capture.records().len()))
            .collect();
        let audit = protocol
            .audit(
                &captures,
                AuditStorage {
                    unfolding: &mut unfolding,
                    choices: &mut routes,
                    execution: ExecutionStorage {
                        session: session.raw(),
                        observed_producers: &observed_producers,
                        order: &mut order,
                        occurrences: &mut occurrences,
                        acceptances: &mut acceptances,
                    },
                },
                &mut scratch,
            )
            .expect("4M needs a checked source execution, not only simulator completion");
        assert_eq!(audit.execution().order.len(), count);

        evidence.push(json!({"choices":choices,"effects":journal.len(),"epf_dynamic_audit":"passed","capture_records":count,"raw":format!("condition-{choices}.json")}));
    }
    if let Some(directory) = std::env::var_os("HIBANA_4M_EVIDENCE") {
        std::fs::create_dir_all(&directory).map_err(|e| ShellError::message(e.to_string()))?;
        std::fs::write(
            std::path::Path::new(&directory).join("native-orders.json"),
            serde_json::to_vec_pretty(
                &json!({"task":"4M","native_fixture":true,"model_calls":0,"controlled_effect_start_barriers":true,"conditions":evidence}),
            )
            .unwrap(),
        )
        .map_err(|e| ShellError::message(e.to_string()))?;
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn actual_receiver_rejects_wrong_assignee_and_substituted_resource_arguments_before_effect()
-> ShellResult<()> {
    let programs = programs();
    for (admitted, actual) in [("module_b", "module_b"), ("module_a", "SHARED_LIBRARY")] {
        let session = SessionId::new(40900);
        let (bus, _reader, _terminals) = Bus::open(
            [
                "ARCHITECT",
                "DEVELOPER_A",
                "DEVELOPER_B",
                "REVIEWER",
                "TESTER",
            ],
            [(session, REPO), (session, TEST_ENV)],
        );
        let effects = ActualEffects {
            started: Instant::now(),
            journal: Cell::new(Vec::new()),
        };
        let mut owner_slab = vec![0; 1024 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let owner = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut caller_storage = SessionKitStorage::uninit();
        let caller = caller_storage
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        let mut store = owner.enter(session, &programs.store)?;
        let repo = owner.enter(session, &programs.repo)?;
        let env = owner.enter(session, &programs.test_env)?;
        let mut architect = caller.enter(session, &programs.architect)?;
        let mut developer = caller.enter(session, &programs.a)?;
        let [mut architect, _private] = architect.par()?;
        let [mut developer, _private_developer] = developer.par()?;
        let result = hibana_runtime::race(
            async {
                async {
                    Ok::<_, ShellError>(
                        localside::resources::repository(&mut store, (repo, env), &effects).await,
                    )
                };
                async {
                    architect
                        .send::<DesignA>(&br#"{"title":"module_a_refactor"}"#.as_slice())
                        .await?;
                    architect.recv::<DesignResult>().await?;
                    developer.recv::<DesignResult>().await?;
                    let mut repo = developer.recv::<RepoLoan>().await?;
                    let bytes = imp::encode(&json!({"target":admitted}))?;
                    developer.send::<OwnIntent>(&bytes.as_slice()).await?;
                    let bytes = imp::encode(&json!({"target":actual}))?;
                    repo.send::<Refactor>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>(())
                };
            },
            hibana_runtime::host::deadline(Duration::from_secs(5)),
        )
        .await?;
        assert!(result.0.is_err());
        let effects = effects.journal.take();
        assert!(effects.iter().all(|e| e["tool"] == "design_proposal"));
    }
    Ok(())
}
