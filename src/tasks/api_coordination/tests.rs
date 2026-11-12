use super::{global::*, imp, localside};
use crate::shell::{
    ShellError,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::cell::Cell;

// Actual fixture effects, retained for independent inspection after execution.
// Neither a participant nor a resolver reads this append-only journal.
struct ActualArguments(Cell<Vec<Value>>);
impl Domain for ActualArguments {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        // A deterministic native effect fixture, not the upstream simulator or AI.
        let mut effects = self.0.take();
        let result = json!({"success":true,"agent":agent,"tool":tool,
            "arguments":arguments,"effect":effects.len()});
        effects.push(result.clone());
        self.0.set(effects);
        Ok(result)
    }
}

#[hibana_runtime::main]
#[test]
async fn actual_resource_handoff_repeats_agreement_and_both_implementations_before_tests()
-> Result<(), ShellError> {
    for (changes, order_mask) in [0u32, 1, 3]
        .into_iter()
        .flat_map(|changes| (0..(1u32 << (changes + 1))).map(move |mask| (changes, mask)))
    {
        let p = programs();
        let effects = ActualArguments(Cell::new(Vec::new()));
        let session = SessionId::new(4000 + changes * 100 + order_mask);
        let (bus, _reader, _terminals) = Bus::open(
            ["FRONTEND_DEV", "BACKEND_DEV", "TESTER"],
            [(session, REPO), (session, TEST_ENV)],
        );
        let mut owner_slab = vec![0; 1024 * 1024];
        // Append-only evidence. Participants never inspect these records.
        let owner_records = Cell::new(Vec::new());
        let owner_capture = |event| {
            let mut records = owner_records.take();
            records.push(event);
            owner_records.set(records);
        };
        let caller_records = Cell::new(Vec::new());
        let caller_capture = |event| {
            let mut records = caller_records.take();
            records.push(event);
            caller_records.set(records);
        };
        let mut owner_storage = SessionKitStorage::uninit();
        let owner = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut caller_storage = SessionKitStorage::uninit();
        let caller = caller_storage
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        owner.observe(&owner_capture);
        caller.observe(&caller_capture);
        let mut store = owner.enter(session, &p.store)?;
        let repo = owner.enter(session, &p.repo)?;
        let test_env = owner.enter(session, &p.test_env)?;
        let mut frontend = caller.enter(session, &p.frontend)?;
        let mut backend = caller.enter(session, &p.backend)?;
        let mut tester = caller.enter(session, &p.tester)?;
        let [mut frontend, mut frontend_private] = frontend.par()?;
        let [mut backend, mut backend_private] = backend.par()?;
        let [mut tester, mut tester_private] = tester.par()?;
        let mut frontend_io = owner.enter(session, &p.frontend_io)?;
        let mut backend_io = owner.enter(session, &p.backend_io)?;
        let mut tester_io = owner.enter(session, &p.tester_io)?;

        let result = hibana_runtime::race(
            async {
                async {
                    localside::resources::repository(&mut store, (repo, test_env), &effects)
                        .await
                        .map_err(|error| ShellError::message(format!("repository: {error}")))?;
                };
                async {
                    let mut repo = backend.recv::<RepoLoan>().await?;
                    let candidate = {
                        let mut candidate = Value::Null;
                        for visit in 0..=changes {
                            let frontend_first = order_mask & (1 << visit) != 0;
                            let define = br#"{"api_name":"feature_api_spec"}"#.as_slice();
                            repo.send::<DefineApi>(&define).await?;
                            let api: Value =
                                serde_json::from_slice(repo.recv::<ApiDefined>().await?)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                            let contract = serde_json::to_vec(&api).unwrap();
                            backend.send::<Contract>(&contract.as_slice()).await?;
                            assert_eq!(backend.recv::<Agreement>().await?, contract);
                            let request = br#"{"feature_name":"backend_endpoint"}"#.as_slice();
                            let (implemented, front) = if frontend_first {
                                backend.send::<RepoLoan>(repo).await?;
                                let front: Value =
                                    serde_json::from_slice(backend.recv::<FrontendReady>().await?)
                                        .map_err(|e| ShellError::message(e.to_string()))?;
                                repo = backend.recv::<RepoLoan>().await?;
                                backend.send::<BackendIntent>(&request).await?;
                                repo.send::<ImplementBackend>(&request).await?;
                                let implemented: Value = serde_json::from_slice(
                                    repo.recv::<BackendImplemented>().await?,
                                )
                                .map_err(|e| ShellError::message(e.to_string()))?;
                                (implemented, front)
                            } else {
                                backend.send::<BackendIntent>(&request).await?;
                                repo.send::<ImplementBackend>(&request).await?;
                                let implemented: Value = serde_json::from_slice(
                                    repo.recv::<BackendImplemented>().await?,
                                )
                                .map_err(|e| ShellError::message(e.to_string()))?;
                                backend.send::<RepoLoan>(repo).await?;
                                let front: Value =
                                    serde_json::from_slice(backend.recv::<FrontendReady>().await?)
                                        .map_err(|e| ShellError::message(e.to_string()))?;
                                repo = backend.recv::<RepoLoan>().await?;
                                (implemented, front)
                            };
                            candidate = imp::candidate(&api, &implemented, &front);
                        }
                        candidate
                    };
                    let bytes = serde_json::to_vec(&candidate).unwrap();
                    repo.send::<Finalize>(&bytes.as_slice()).await?;
                    backend.send::<RepoLoan>(repo).await?;
                    let result: Value =
                        serde_json::from_slice(backend.recv::<BackendResult>().await?)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                    assert_eq!(result["tool"], "run_tests");
                    assert_eq!(result["success"], true);
                };
                async {
                    let mut contract = frontend.recv::<Contract>().await?.to_vec();
                    loop {
                        frontend.send::<Agreement>(&contract.as_slice()).await?;
                        let mut repo = frontend.recv::<RepoLoan>().await?;
                        let request = br#"{"feature_name":"frontend_component"}"#.as_slice();
                        repo.send::<ImplementFrontend>(&request).await?;
                        let bytes = repo.recv::<FrontendImplemented>().await?.to_vec();
                        frontend.send::<FrontendReady>(&bytes.as_slice()).await?;
                        frontend.send::<RepoLoan>(repo).await?;
                        let branch = frontend.offer().await?;
                        match branch.label() {
                            3 => contract = branch.recv::<Contract>().await?.to_vec(),
                            19 => {
                                let result: Value =
                                    serde_json::from_slice(branch.recv::<FrontendResult>().await?)
                                        .map_err(|e| ShellError::message(e.to_string()))?;
                                assert_eq!(result["success"], true);
                                break;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected frontend label {label}"
                                )));
                            }
                        }
                    }
                };
                async {
                    let ready: Value = serde_json::from_slice(tester.recv::<Ready>().await?)
                        .map_err(|e| ShellError::message(e.to_string()))?;
                    assert_eq!(ready["frontend"]["agent"], "FRONTEND_DEV");
                    assert_eq!(ready["backend"]["agent"], "BACKEND_DEV");
                    let (mut repo, mut test_env) = async {
                        let [mut repository, mut environment] = tester.par()?;
                        async { Ok::<_, ShellError>(repository.recv::<RepoLoan>().await?) };
                        async { Ok::<_, ShellError>(environment.recv::<TestLoan>().await?) };
                    }
                    .await?;
                    let (result, tested) = async {
                        async {
                            assert!(caller.enter(session, &p.repo).is_err());
                            assert!(owner.enter(session, &p.repo).is_err());
                            let request = br#"{"suite":"integration_tests"}"#.as_slice();
                            repo.send::<ReadForTests>(&request).await?;
                            let result = repo.recv::<RepositoryTested>().await?.to_vec();
                            Ok::<_, ShellError>(result)
                        };
                        async {
                            let request = br#"{"suite":"integration_tests"}"#.as_slice();
                            test_env.send::<RunTests>(&request).await?;
                            let result = test_env.recv::<EnvironmentTested>().await?.to_vec();
                            Ok::<_, ShellError>(result)
                        };
                    }
                    .await?;
                    assert_eq!(result, tested);
                    let [mut repository, mut environment] = tester.par()?;
                    async {
                        repository.send::<RepoLoan>(repo).await?;
                    };
                    async {
                        environment.send::<TestLoan>(test_env).await?;
                    };
                };
                Ok::<_, ShellError>(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        if result.is_err() {
            for observation in bus
                .transport
                .take_observations()
                .iter()
                .rev()
                .take(36)
                .rev()
            {
                eprintln!("{observation}");
            }
        }
        result?;
        async {
            async {
                frontend_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                backend_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                tester_private
                    .send::<crate::shell::global::Exit>(&0)
                    .await?;
            };
            async {
                assert_eq!(frontend_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(backend_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            async {
                assert_eq!(tester_io.recv::<crate::shell::global::Exit>().await?, 0);
            };
            Ok(())
        }
        .await?;
        let actual_effects = effects.0.take();
        assert_eq!(actual_effects.len(), 3 * (changes as usize + 1) + 1);
        for (agent, tool, expected) in [
            ("BACKEND_DEV", "define_api", changes as usize + 1),
            ("BACKEND_DEV", "implement_feature", changes as usize + 1),
            ("FRONTEND_DEV", "implement_feature", changes as usize + 1),
            ("TESTER", "run_tests", 1),
        ] {
            assert_eq!(
                actual_effects
                    .iter()
                    .filter(|e| e["agent"] == agent && e["tool"] == tool)
                    .count(),
                expected
            );
        }
        let primary = owner_records.take();
        let secondary = caller_records.take();
        use hibana::runtime::tap;
        use hibana_epf::{
            Acceptance, AuditStorage, Capture, ExecutionStorage, Protocol, RecordId, RouteChoice,
            Unfold,
        };
        let captures = [
            Capture::new(&primary, owner.tap().checkpoint()).unwrap(),
            Capture::new(&secondary, caller.tap().checkpoint()).unwrap(),
        ];
        let snapshot: Vec<_> = tap::projected(&p.store).collect();
        let mut source_storage = vec![0; Protocol::workspace_len(&snapshot).unwrap()];
        let protocol = Protocol::new(&snapshot, &mut source_storage).unwrap();
        let count: usize = captures.iter().map(|c| c.records().len()).sum();
        let capacity = count.checked_mul(2).and_then(|n| n.checked_add(1)).unwrap();
        let mut unfolding = vec![Unfold::Source; capacity];
        let mut choices = vec![
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
                    choices: &mut choices,
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
            .expect("the actual API amendments need a checked source execution");
        let mut received = std::collections::BTreeMap::<u8, usize>::new();
        for (id, occurrence) in audit
            .execution()
            .order
            .iter()
            .zip(audit.execution().occurrences)
        {
            let record = captures[id.producer].records()[id.ordinal];
            if record.id() == tap::ENDPOINT_RECV && record.causal_key() >> 8 == u16::from(STORE) {
                let source = audit
                    .path()
                    .unfolding()
                    .origin(occurrence.unwrap())
                    .unwrap();
                *received
                    .entry(snapshot[2 + usize::from(source)].arg0() as u8)
                    .or_default() += 1;
            }
        }
        for label in [1, 5, 7, 20] {
            assert_eq!(received[&label], changes as usize + 1);
        }
        assert_eq!(received[&14], 1);
        assert_eq!(received[&15], 1);
        let commit_count = audit
            .execution()
            .occurrences
            .iter()
            .filter(|o| o.is_some())
            .count();
        assert!(commit_count > 0);
        eprintln!(
            "4E changes={changes} order_mask={order_mask} records={count} committed={commit_count}"
        );
        if let Some(directory) = std::env::var_os("CHOREOFIX_COORDINATION_EVIDENCE") {
            let directory = std::path::PathBuf::from(directory)
                .join(format!("changes{changes}-order{order_mask}"));
            std::fs::create_dir(&directory).unwrap();
            for (name, records) in [
                ("protocol.bin", snapshot.as_slice()),
                ("owner.bin", primary.as_slice()),
                ("caller.bin", secondary.as_slice()),
            ] {
                let bytes: Vec<_> = records
                    .iter()
                    .flat_map(|record| <[u8; 16]>::from(*record))
                    .collect();
                std::fs::write(directory.join(name), bytes).unwrap();
            }
            std::fs::write(directory.join("inputs.json"), serde_json::to_vec_pretty(&json!({
                    "session":session.raw(),"api_changes":changes,"frontend_first_by_visit":
                        (0..=changes).map(|visit| order_mask & (1 << visit) != 0).collect::<Vec<_>>(),
                "derivation_capacity":capacity,"owner_checkpoint":owner.tap().checkpoint(),
                "caller_checkpoint":caller.tap().checkpoint(),"frames":bus.transport.take_observations(),
                "actual_effects":actual_effects,"effect_requests_by_label":received,
                "epf_dynamic_audit":"passed","records":count,"committed":commit_count,
                "wasm":false,"model_calls":0,"scored_runs":0
            })).unwrap()).unwrap();
        }
    }
    Ok(())
}

struct NoEffects;
impl Domain for NoEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        panic!("invalid API arguments reached an effect: {agent} {tool} {arguments}")
    }
}

#[hibana_runtime::main]
#[test]
async fn invalid_api_definition_is_rejected_by_the_real_owner_before_an_effect()
-> Result<(), ShellError> {
    for bytes in [
        br#"{"api_name":"other_api"}"#.as_slice(),
        br#"{"api_name":7}"#,
        br#"{"api_name":"feature_api_spec","feature_name":"backend_endpoint"}"#,
    ] {
        let p = programs();
        let session = SessionId::new(4010);
        let (bus, _, _) = Bus::open(
            ["FRONTEND_DEV", "BACKEND_DEV", "TESTER"],
            [(session, REPO), (session, TEST_ENV)],
        );
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
        let mut store = owner.enter(session, &p.store)?;
        let repo = owner.enter(session, &p.repo)?;
        let test_env = owner.enter(session, &p.test_env)?;
        let mut backend = caller.enter(session, &p.backend)?;
        let [mut backend, _private] = backend.par()?;
        let (rejected, held) = async {
            async {
                Ok::<_, ShellError>(
                    localside::resources::repository(&mut store, (repo, test_env), &NoEffects)
                        .await,
                )
            };
            async {
                let mut repo = backend.recv::<RepoLoan>().await?;
                repo.send::<DefineApi>(&bytes).await?;
                Ok::<_, ShellError>(repo)
            };
        }
        .await?;
        assert_eq!(
            rejected.unwrap_err().to_string(),
            "API coordination arguments differ from the assignment"
        );
        drop(held);
    }
    Ok(())
}
