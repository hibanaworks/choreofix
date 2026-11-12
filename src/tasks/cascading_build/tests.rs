//! Scripted native Endpoint participants, not Wasm or model qualifications.
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, review_reply::ReviewReply, service::Domain},
};
use hibana::{
    g::Message,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        resolver::{DecisionArm, ResolverRef},
    },
};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    time::{Duration, Instant},
};
#[cfg(miri)]
#[path = "../handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// Pure actual-data fixtures; they do not determine Endpoint permissions.
fn checked_objects() -> ([Value; 3], [Value; 3]) {
    let objects = core::array::from_fn(|index| {
        let inputs = match index {
            0 => json!({"CORE_LIB":"current core","CONFIG":"current configuration"}),
            1 => json!({"CORE_LIB":"current core","DATA_MODELS":"current models"}),
            _ => json!({"CONFIG":"current configuration","DATA_MODELS":"current models"}),
        };
        json!({"id":format!("object:{}",index+1),"builder":imp::AGENTS[index],
            "module":imp::MODULES[index],"inputs":inputs,
            "bytes":json!({"module":imp::MODULES[index],"inputs":inputs}).to_string(),
            "content_sha256":"fixture-content"})
    });
    let tests = core::array::from_fn(|index| {
        json!({"tool_name":"run_build_tests","success":true,
            "data":{"agent":"TEST_RUNNER","module":imp::MODULES[index],
                "receipt":format!("test:{index}"),"input_object":objects[index]}})
    });
    (objects, tests)
}

#[test]
fn candidate_requires_each_modules_actual_two_input_objects() {
    let (objects, tests) = checked_objects();
    assert!(imp::finish(&objects, &tests).is_ok());
    for (index, object) in objects.iter().enumerate() {
        for inputs in [
            json!({}),
            json!({"CORE_LIB":"current core","CONFIG":"current configuration",
            "DATA_MODELS":"current models"}),
            json!({"UNBOUND":"data","OTHER":"data"}),
        ] {
            let mut malformed = object.clone();
            malformed["inputs"] = inputs.clone();
            malformed["bytes"] =
                json!(json!({"module":imp::MODULES[index],"inputs":inputs}).to_string());
            assert!(imp::validate_object(&malformed, index).is_err());
        }
    }
}

#[test]
fn a_same_byte_rebuild_cannot_reuse_an_older_positive_test() {
    let (mut objects, mut tests) = checked_objects();
    objects[0]["id"] = json!("object:4");
    assert!(!imp::current(&objects, &tests, 0).unwrap());
    assert!(imp::finish(&objects, &tests).is_err());
    tests[0]["data"]["input_object"] = objects[0].clone();
    tests[0]["data"]["receipt"] = json!("test:new-object-occurrence");
    assert!(imp::finish(&objects, &tests).is_ok());
}

#[test]
fn a_positive_test_cannot_hide_later_shared_input_changes() {
    let (mut objects, mut tests) = checked_objects();
    objects[1]["inputs"]["CORE_LIB"] = json!("new actual core");
    objects[1]["bytes"] =
        json!(json!({"module":imp::MODULES[1],"inputs":objects[1]["inputs"]}).to_string());
    tests[1]["data"]["input_object"] = objects[1].clone();
    assert!(imp::validate_object(&objects[1], 1).is_ok());
    assert!(imp::current(&objects, &tests, 1).unwrap());
    assert!(!imp::current(&objects, &tests, 0).unwrap());
    assert_eq!(imp::needed(&objects, &tests).unwrap(), Some(0));
    assert!(imp::finish(&objects, &tests).is_err());
}

#[test]
fn wrong_modules_extra_arguments_and_malformed_candidates_are_rejected() {
    assert!(imp::arguments(json!({"module_name":"module_b"}), 0).is_err());
    assert!(imp::arguments(json!({"module_name":"module_a","unchecked":true}), 0).is_err());
    assert!(imp::arguments(json!({"module_name":" Module A "}), 0).is_ok());
    assert!(imp::link_arguments(json!({"modules":"module_a,module_a,module_b"})).is_err());
    assert!(imp::candidate_parts(&json!({"objects":[],"tests":[]})).is_err());
}

#[test]
fn resolver_reads_actual_failed_receipt_and_restores_its_input() {
    let (objects, mut tests) = checked_objects();
    tests[2]["success"] = json!(false);
    let actual = imp::candidate(&objects, &tests).unwrap();
    let input = Cell::new(actual.clone());
    assert_eq!(imp::selected::<0>(&input).unwrap(), DecisionArm::Right);
    assert_eq!(imp::selected::<2>(&input).unwrap(), DecisionArm::Left);
    assert_eq!(input.take(), actual);
    let malformed = json!({"objects":["not an actual candidate"]});
    input.set(malformed.clone());
    assert!(imp::selected::<0>(&input).is_err());
    assert_eq!(input.take(), malformed);
}

// Actual fixture material, results and timing observations. These records do
// not authorize an Endpoint operation. The separate common Python world is not
// used by this fixture; actual Wasm/model qualifications remain outstanding.
struct BuildWorld {
    journal: Cell<Vec<Value>>,
    failures: &'static [usize],
    start: Instant,
}
impl Domain for BuildWorld {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        args: Value,
    ) -> Result<Value, String> {
        let mut rows = self.journal.take();
        let id = rows.len();
        let mut material = json!({"CONFIG":{"base":"provided CONFIG","modules":{}},
            "CORE_LIB":{"base":"provided CORE_LIB","modules":{}},
            "DATA_MODELS":{"base":"provided DATA_MODELS","modules":{}}});
        for row in &rows {
            if row["tool"] == "compile_module" {
                for (name, value) in row["result"]["data"]["shared_outputs"].as_object().unwrap() {
                    material[name] = value.clone();
                }
            }
        }
        let result = if tool == "compile_module" {
            let module = args["module_name"].as_str().unwrap();
            let index = imp::module(&json!(module)).unwrap();
            let names = match index {
                0 => ["CORE_LIB", "CONFIG"],
                1 => ["CORE_LIB", "DATA_MODELS"],
                _ => ["CONFIG", "DATA_MODELS"],
            };
            let mut updates = json!({});
            for name in names {
                let mut data = material[name].clone();
                data["modules"][module] = json!(format!("declarations from {module}"));
                if data != material[name] {
                    updates[name] = data.clone();
                }
                material[name] = data;
            }
            let inputs: serde_json::Map<String, Value> = names
                .into_iter()
                .map(|name| (name.into(), material[name].clone()))
                .collect();
            let encoded = json!({"module":module,"inputs":inputs}).to_string();
            json!({"tool_name":tool,"success":true,"data":{"agent":agent,"module":module,"status":"compiled",
                "object":{"id":format!("object:{}",id+1),"builder":agent,"module":module,
                    "inputs":inputs,"bytes":encoded,"content_sha256":"native-fixture-not-a-cryptographic-receipt"},
                "shared_outputs":updates,"artifacts_modified":!updates.as_object().unwrap().is_empty()}})
        } else if tool == "run_build_tests" {
            let module = args["module_name"].as_str().unwrap();
            let obj = rows
                .iter()
                .rev()
                .find(|row| {
                    row["tool"] == "compile_module" && row["result"]["data"]["module"] == module
                })
                .unwrap()["result"]["data"]["object"]
                .clone();
            let stale = obj["inputs"]
                .as_object()
                .unwrap()
                .iter()
                .any(|(name, data)| material[name] != *data);
            let trial = rows
                .iter()
                .filter(|row| row["tool"] == "run_build_tests")
                .count();
            let success = !stale && !self.failures.contains(&trial);
            json!({"tool_name":tool,"success":success,"data":{"agent":agent,"module":module,
                "status":if success {"passed"}else{"needs_rebuild"},"input_object":obj,
                "receipt":format!("test:{}",id+1)}})
        } else {
            assert_eq!(tool, "link_modules");
            let mut objects = json!({});
            let mut checks = json!({});
            let mut issues = Vec::new();
            for module in imp::MODULES {
                let obj = &rows
                    .iter()
                    .rev()
                    .find(|row| {
                        row["tool"] == "compile_module" && row["result"]["data"]["module"] == module
                    })
                    .unwrap()["result"]["data"]["object"];
                let test = &rows
                    .iter()
                    .rev()
                    .find(|row| {
                        row["tool"] == "run_build_tests"
                            && row["result"]["data"]["module"] == module
                    })
                    .unwrap()["result"];
                if test["success"] != true
                    || test["data"]["input_object"] != *obj
                    || obj["inputs"]
                        .as_object()
                        .unwrap()
                        .iter()
                        .any(|(name, data)| material[name] != *data)
                {
                    issues.push(module);
                }
                objects[module] = obj.clone();
                checks[module] = json!({"run_build_tests":test["data"]["receipt"]});
            }
            json!({"tool_name":tool,"success":true,"data":{"agent":agent,
                "status":if issues.is_empty(){"linked"}else{"linked_with_issues"},"inputs":objects,"checks":checks,
                "issues":issues,"binary":{"id":"native-link","bytes":objects.to_string()}}})
        };
        rows.push(
            json!({"agent":agent,"tool":tool,"arguments":args,"result":result,
            "started":self.start.elapsed().as_nanos() as u64,"completed":null}),
        );
        self.journal.set(rows);
        // Controlled pending polls expose the source's build/test overlap.
        // They are labeled native-fixture scheduling, never model timing.
        for _ in 0..4 {
            hibana_runtime::yield_now().await;
        }
        let mut rows = self.journal.take();
        rows[id]["completed"] = json!(self.start.elapsed().as_nanos() as u64);
        self.journal.set(rows);
        Ok(result)
    }
}

#[hibana_runtime::scope]
async fn builder<const WHO: u8>(endpoint: &mut hibana::ParallelEndpoint<WHO>) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            LoanCoreA::LOGICAL_LABEL | LoanConfigA::LOGICAL_LABEL => {
                drop(branch);
                let (mut core, mut config) = async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<LoanCoreA>().await?) };
                    async { Ok::<_, ShellError>(configuration.recv::<LoanConfigA>().await?) };
                }
                .await?;
                let args = br#"{"module_name":"module_a"}"#.as_slice();
                endpoint.send::<CompileA>(&args).await?;
                let (first, second) = async {
                    async {
                        core.send::<CompileCore>(&args).await?;
                        imp::decode(core.recv::<CoreBuilt>().await?)
                    };
                    async {
                        config.send::<CompileConfig>(&args).await?;
                        imp::decode(config.recv::<ConfigBuilt>().await?)
                    };
                }
                .await?;
                assert_eq!(first, second);
                async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async {
                        library.send::<ReturnCore>(core).await?;
                    };
                    async {
                        configuration.send::<ReturnConfig>(config).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
            }
            LoanCoreB::LOGICAL_LABEL | LoanModelsB::LOGICAL_LABEL => {
                drop(branch);
                let (mut core, mut models) = async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<LoanCoreB>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<LoanModelsB>().await?) };
                }
                .await?;
                let args = br#"{"module_name":"module_b"}"#.as_slice();
                endpoint.send::<CompileB>(&args).await?;
                let (first, second) = async {
                    async {
                        core.send::<CompileCore>(&args).await?;
                        imp::decode(core.recv::<CoreBuilt>().await?)
                    };
                    async {
                        models.send::<CompileModels>(&args).await?;
                        imp::decode(models.recv::<ModelsBuilt>().await?)
                    };
                }
                .await?;
                assert_eq!(first, second);
                async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async {
                        library.send::<ReturnCore>(core).await?;
                    };
                    async {
                        definitions.send::<ReturnModels>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
            }
            LoanConfigC::LOGICAL_LABEL | LoanModelsC::LOGICAL_LABEL => {
                drop(branch);
                let (mut config, mut models) = async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(configuration.recv::<LoanConfigC>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<LoanModelsC>().await?) };
                }
                .await?;
                let args = br#"{"module_name":"module_c"}"#.as_slice();
                endpoint.send::<CompileC>(&args).await?;
                let (first, second) = async {
                    async {
                        config.send::<CompileConfig>(&args).await?;
                        imp::decode(config.recv::<ConfigBuilt>().await?)
                    };
                    async {
                        models.send::<CompileModels>(&args).await?;
                        imp::decode(models.recv::<ModelsBuilt>().await?)
                    };
                }
                .await?;
                assert_eq!(first, second);
                async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async {
                        configuration.send::<ReturnConfig>(config).await?;
                    };
                    async {
                        definitions.send::<ReturnModels>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
            }
            UpdateA::LOGICAL_LABEL => {
                imp::decode(branch.recv::<UpdateA>().await?)?;
            }
            UpdateB::LOGICAL_LABEL => {
                imp::decode(branch.recv::<UpdateB>().await?)?;
            }
            UpdateC::LOGICAL_LABEL => {
                imp::decode(branch.recv::<UpdateC>().await?)?;
            }
            Feedback::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Feedback>().await?)?;
            }
            RetryA::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryA>().await?)?;
            }
            RetryB::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryB>().await?)?;
            }
            RetryC::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryC>().await?)?;
            }
            RetryAToB::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryAToB>().await?)?;
            }
            RetryAToC::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryAToC>().await?)?;
            }
            RetryBToA::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryBToA>().await?)?;
            }
            RetryBToC::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryBToC>().await?)?;
            }
            RetryCToA::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryCToA>().await?)?;
            }
            RetryCToB::LOGICAL_LABEL => {
                imp::decode(branch.recv::<RetryCToB>().await?)?;
            }
            FinishedA::LOGICAL_LABEL => {
                assert_eq!(
                    imp::decode(branch.recv::<FinishedA>().await?)?["success"],
                    true
                );
                return Ok(());
            }
            FinishedB::LOGICAL_LABEL => {
                assert_eq!(
                    imp::decode(branch.recv::<FinishedB>().await?)?["success"],
                    true
                );
                return Ok(());
            }
            FinishedC::LOGICAL_LABEL => {
                assert_eq!(
                    imp::decode(branch.recv::<FinishedC>().await?)?["success"],
                    true
                );
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected native builder operation")),
        }
    }
}

#[hibana_runtime::scope]
async fn tester(endpoint: &mut hibana::ParallelEndpoint<TESTER>) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        let (index, report) = match branch.label() {
            TestObjectA::LOGICAL_LABEL => (0, imp::decode(branch.recv::<TestObjectA>().await?)?),
            TestObjectB::LOGICAL_LABEL => (1, imp::decode(branch.recv::<TestObjectB>().await?)?),
            TestObjectC::LOGICAL_LABEL => (2, imp::decode(branch.recv::<TestObjectC>().await?)?),
            FinishedTester::LOGICAL_LABEL => {
                assert_eq!(
                    imp::decode(branch.recv::<FinishedTester>().await?)?["success"],
                    true
                );
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected native test input")),
        };
        let object = imp::object(&report, index)?;
        let request =
            imp::encode(&json!({"arguments":{"module_name":imp::MODULES[index]},"object":object}))?;
        match index {
            0 => endpoint.send::<TestA>(&request.as_slice()).await?,
            1 => endpoint.send::<TestB>(&request.as_slice()).await?,
            _ => endpoint.send::<TestC>(&request.as_slice()).await?,
        }
        imp::test_result(&imp::decode(endpoint.recv::<Tested>().await?)?, index)?;
    }
}

#[hibana_runtime::scope]
async fn integrator(
    endpoint: &mut hibana::ParallelEndpoint<INTEGRATOR>,
    input: &Cell<Value>,
    receipt_override: Option<&str>,
) -> ShellResult<()> {
    let (objects, tests) = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            RepairCandidate::LOGICAL_LABEL => {
                let actual = imp::decode(branch.recv::<RepairCandidate>().await?)?;
                let (objects, tests) = imp::candidate_parts(&actual)?;
                let index = imp::needed(&objects, &tests)?
                    .ok_or_else(|| ShellError::message("repair candidate is already current"))?;
                let mut request = imp::reconsider(&objects, &tests, index)?;
                if let Some(receipt) = receipt_override {
                    request["test"]["data"]["receipt"] = json!(receipt);
                }
                let bytes = imp::encode(&request)?;
                input.set(actual);
                // This offer resolves from the actual received candidate,
                // before borrowing the chosen source's four parallel sends.
                drop(endpoint.offer().await?);
                async {
                    let [mut selected, mut compiler, mut first_peer, mut second_peer] =
                        endpoint.par()?;
                    match index {
                        0 => {
                            async {
                                async {
                                    selected.send::<RetryA>(&bytes.as_slice()).await?;
                                };
                                async {
                                    compiler
                                        .send::<RetryAForCompiler>(&bytes.as_slice())
                                        .await?;
                                };
                                async {
                                    first_peer.send::<RetryAToB>(&bytes.as_slice()).await?;
                                };
                                async {
                                    second_peer.send::<RetryAToC>(&bytes.as_slice()).await?;
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?
                        }
                        1 => {
                            async {
                                async {
                                    selected.send::<RetryB>(&bytes.as_slice()).await?;
                                };
                                async {
                                    compiler
                                        .send::<RetryBForCompiler>(&bytes.as_slice())
                                        .await?;
                                };
                                async {
                                    first_peer.send::<RetryBToA>(&bytes.as_slice()).await?;
                                };
                                async {
                                    second_peer.send::<RetryBToC>(&bytes.as_slice()).await?;
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?
                        }
                        _ => {
                            async {
                                async {
                                    selected.send::<RetryC>(&bytes.as_slice()).await?;
                                };
                                async {
                                    compiler
                                        .send::<RetryCForCompiler>(&bytes.as_slice())
                                        .await?;
                                };
                                async {
                                    first_peer.send::<RetryCToA>(&bytes.as_slice()).await?;
                                };
                                async {
                                    second_peer.send::<RetryCToB>(&bytes.as_slice()).await?;
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?
                        }
                    }
                    Ok::<(), ShellError>(())
                }
                .await?;
            }
            ReadyCandidate::LOGICAL_LABEL => {
                let actual = imp::decode(branch.recv::<ReadyCandidate>().await?)?;
                let (objects, tests) = imp::candidate_parts(&actual)?;
                assert_eq!(imp::finish(&objects, &tests)?, actual);
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<FinishChecks>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
                break (objects, tests);
            }
            _ => return Err(ShellError::message("unexpected candidate")),
        }
    };
    let (_core, _config, _models) = async {
        let [mut library, mut configuration, mut definitions] = endpoint.par()?;
        async { Ok::<_, ShellError>(library.recv::<FrozenCore>().await?) };
        async { Ok::<_, ShellError>(configuration.recv::<FrozenConfig>().await?) };
        async { Ok::<_, ShellError>(definitions.recv::<FrozenModels>().await?) };
    }
    .await?;
    let request = imp::link_arguments(json!({"modules":"module_c,module_b,module_a"}))?;
    let bytes = imp::encode(&request)?;
    endpoint.send::<Link>(&bytes.as_slice()).await?;
    let actual = imp::decode(endpoint.recv::<Linked>().await?)?;
    imp::linked(&actual, &objects, &tests)?;
    let bytes = imp::encode(&actual)?;
    async {
        let [mut a, mut b, mut c, mut tester, mut service] = endpoint.par()?;
        async {
            a.send::<FinishedA>(&bytes.as_slice()).await?;
        };
        async {
            b.send::<FinishedB>(&bytes.as_slice()).await?;
        };
        async {
            c.send::<FinishedC>(&bytes.as_slice()).await?;
        };
        async {
            tester.send::<FinishedTester>(&bytes.as_slice()).await?;
        };
        async {
            service.send::<FinishedService>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn all_six_initial_orders_reconsider_stale_or_failed_builds_without_parallel_tests()
-> ShellResult<()> {
    let p = programs();
    for order in [
        [A, B, C],
        [A, C, B],
        [B, A, C],
        [B, C, A],
        [C, A, B],
        [C, B, A],
    ] {
        for (failures, receipt_override) in [
            (&[][..], None),
            (&[0, 1, 3, 4, 7][..], None),
            (&[2, 3, 4][..], None),
        ]
        .into_iter()
        .chain((order == [A, B, C]).then_some((&[][..], Some("not-the-actual-test-receipt"))))
        {
            let session = SessionId::new(SESSION);
            let (bus, _io, _terminals) = Bus::open(
                [
                    "BUILDER_A",
                    "BUILDER_B",
                    "BUILDER_C",
                    "INTEGRATOR",
                    "TEST_RUNNER",
                ],
                [(session, CORE), (session, CONFIG), (session, MODELS)],
            );
            let input = Cell::new(Value::Null);
            let mut owner_slab = vec![0; 1024 * 1024];
            let mut caller_slab = vec![0; 1024 * 1024];
            let owner_rows = Cell::new(Vec::new());
            let owner_capture = |event| {
                let mut rows = owner_rows.take();
                rows.push(crate::imp::projection::record(event));
                owner_rows.set(rows);
            };
            let caller_rows = Cell::new(Vec::new());
            let caller_capture = |event| {
                let mut rows = caller_rows.take();
                rows.push(crate::imp::projection::record(event));
                caller_rows.set(rows);
            };
            let mut owner_storage = SessionKitStorage::uninit();
            let mut caller_storage = SessionKitStorage::uninit();
            let owner = owner_storage
                .init()
                .rendezvous(&mut owner_slab, &bus.transport)?;
            let caller = caller_storage
                .init()
                .rendezvous(&mut caller_slab, &bus.transport)?;
            owner.observe(&owner_capture);
            caller.observe(&caller_capture);
            owner
                .set_resolver(
                    &p.compiler,
                    ResolverRef::<FIRST_A>::decision_state(&order, |order| {
                        Ok(if order[0] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .unwrap();
            owner
                .set_resolver(
                    &p.compiler,
                    ResolverRef::<FIRST_B>::decision_state(&order, |order| {
                        Ok(if order[0] == B {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .unwrap();
            owner
                .set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_A>::decision_state(&order, |order| {
                        Ok(if order[1] == B {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .unwrap();
            owner
                .set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_B>::decision_state(&order, |order| {
                        Ok(if order[1] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .unwrap();
            owner
                .set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_C>::decision_state(&order, |order| {
                        Ok(if order[1] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .unwrap();
            caller
                .set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_A>::decision_state(&input, imp::selected::<0>),
                )
                .unwrap();
            caller
                .set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_B>::decision_state(&input, imp::selected::<1>),
                )
                .unwrap();
            let mut compiler = owner.enter(session, &p.compiler)?;
            let mut testing = owner.enter(session, &p.testing)?;
            let core = owner.enter(session, &p.core)?;
            let config = owner.enter(session, &p.config)?;
            let models = owner.enter(session, &p.models)?;
            let mut a = caller.enter(session, &p.a)?;
            let mut b = caller.enter(session, &p.b)?;
            let mut c = caller.enter(session, &p.c)?;
            let mut test = caller.enter(session, &p.tester)?;
            let mut integration = caller.enter(session, &p.integrator)?;
            let [mut a, _pa] = a.par()?;
            let [mut b, _pb] = b.par()?;
            let [mut c, _pc] = c.par()?;
            let [mut test, _pt] = test.par()?;
            let [mut integration, _pi] = integration.par()?;
            let world = BuildWorld {
                journal: Cell::new(Vec::new()),
                failures,
                start: Instant::now(),
            };
            let completed = hibana_runtime::race(
                async {
                    async {
                        localside::compiler::run(&mut compiler, (core, config, models), &world)
                            .await?;
                    };
                    async {
                        localside::testing::run(&mut testing, &world).await?;
                    };
                    async {
                        builder(&mut a).await?;
                    };
                    async {
                        builder(&mut b).await?;
                    };
                    async {
                        builder(&mut c).await?;
                    };
                    async {
                        tester(&mut test).await?;
                    };
                    async {
                        integrator(&mut integration, &input, receipt_override).await?;
                    };
                    Ok::<(), ShellError>(())
                },
                deadline(Duration::from_secs(if cfg!(miri) { 300 } else { 15 })),
            )
            .await;
            if let Some(receipt) = receipt_override {
                let error = completed.expect_err("a changed repair receipt must be rejected");
                assert!(
                    error.to_string().contains(
                        "rebuild request does not match the current actual object and test"
                    )
                );
                let rows = world.journal.take();
                assert_eq!(rows.len(), 6, "no repair or link effect may execute");
                assert_eq!(
                    rows.iter()
                        .filter(|row| row["tool"] == "compile_module")
                        .count(),
                    3
                );
                assert!(
                    rows.iter()
                        .all(|row| row["result"]["data"]["receipt"] != receipt)
                );
                continue;
            }
            #[cfg(not(miri))]
            if let Err(error) = &completed {
                {
                    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../output/experiments/build-10M-development-v1/native-failures")
                        .join(format!(
                            "case-{}{}{}-{}-{}",
                            order[0],
                            order[1],
                            order[2],
                            failures.len(),
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        ));
                    std::fs::create_dir_all(&directory)?;
                    std::fs::write(directory.join("failure.json"),serde_json::to_vec_pretty(&json!({
                        "error":error.to_string(),"order":order,"faults":failures,
                        "journal":world.journal.take(),"transport":bus.transport.take_observations(),
                        "owner_records":owner_rows.take(),"caller_records":caller_rows.take(),
                        "compiled":p.observation()})).unwrap())?;
                }
            }
            completed?;
            let rows = world.journal.take();
            assert_eq!(rows.last().unwrap()["tool"], "link_modules");
            assert_eq!(rows.last().unwrap()["result"]["data"]["issues"], json!([]));
            let first: Vec<_> = rows
                .iter()
                .filter(|row| row["tool"] == "compile_module")
                .take(3)
                .map(|row| row["agent"].as_str().unwrap())
                .collect();
            assert_eq!(first, order.map(|role| imp::AGENTS[role as usize]));
            for tool in ["compile_module", "run_build_tests"] {
                let events: Vec<_> = rows.iter().filter(|row| row["tool"] == tool).collect();
                for pair in events.windows(2) {
                    assert!(
                        pair[0]["completed"].as_u64().unwrap()
                            <= pair[1]["started"].as_u64().unwrap()
                    );
                }
            }
            assert!(
                rows.iter().any(|test| test["tool"] == "run_build_tests"
                    && rows
                        .iter()
                        .any(|compile| compile["tool"] == "compile_module"
                            && compile["started"].as_u64().unwrap()
                                < test["completed"].as_u64().unwrap()
                            && test["started"].as_u64().unwrap()
                                < compile["completed"].as_u64().unwrap())),
                "{order:?}"
            );
            let repairs = rows
                .iter()
                .filter(|row| row["tool"] == "compile_module")
                .count()
                - 3;
            assert!(repairs >= 2, "the first two objects became stale");
            for module in imp::MODULES {
                let tests: Vec<_> = rows
                    .iter()
                    .filter(|row| {
                        row["tool"] == "run_build_tests"
                            && row["result"]["data"]["module"] == module
                    })
                    .collect();
                for pair in tests.windows(2) {
                    if pair[0]["result"]["success"] == false {
                        assert_ne!(
                            pair[0]["result"]["data"]["input_object"]["id"],
                            pair[1]["result"]["data"]["input_object"]["id"],
                            "a failed object must be rebuilt before another test"
                        );
                    }
                }
                if failures == [2, 3, 4] {
                    assert!(
                        rows.iter()
                            .filter(|row| {
                                row["tool"] == "compile_module"
                                    && row["result"]["data"]["module"] == module
                            })
                            .count()
                            >= 2,
                        "every selected repair branch must execute"
                    );
                }
            }
            if failures == [2, 3, 4] {
                let tests: Vec<_> = rows
                    .iter()
                    .filter(|row| row["tool"] == "run_build_tests")
                    .collect();
                for index in failures {
                    assert_eq!(tests[*index]["result"]["success"], false);
                }
            }
            // The old roll's sender has no resource entry authority after
            // publication. Reject another candidate before any frame is sent.
            bus.transport.take_observations();
            assert!(
                compiler
                    .send::<RepairCandidate>(&b"{}".as_slice())
                    .await
                    .is_err()
            );
            assert!(bus.transport.take_observations().is_empty());
        }
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_compiler_rejects_changed_resource_arguments_before_any_effect() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION);
    let (bus, _io, _terminals) = Bus::open(
        [
            "BUILDER_A",
            "BUILDER_B",
            "BUILDER_C",
            "INTEGRATOR",
            "TEST_RUNNER",
        ],
        [(session, CORE), (session, CONFIG), (session, MODELS)],
    );
    let mut owner_slab = vec![0; 1024 * 1024];
    let mut caller_slab = vec![0; 1024 * 1024];
    let mut owner_storage = SessionKitStorage::uninit();
    let mut caller_storage = SessionKitStorage::uninit();
    let owner = owner_storage
        .init()
        .rendezvous(&mut owner_slab, &bus.transport)?;
    let caller = caller_storage
        .init()
        .rendezvous(&mut caller_slab, &bus.transport)?;
    owner
        .set_resolver(
            &p.compiler,
            ResolverRef::<FIRST_A>::decision_state(&(), |_| Ok(DecisionArm::Left)),
        )
        .unwrap();
    owner
        .set_resolver(
            &p.compiler,
            ResolverRef::<SECOND_A>::decision_state(&(), |_| Ok(DecisionArm::Left)),
        )
        .unwrap();
    let mut compiler = owner.enter(session, &p.compiler)?;
    let core = owner.enter(session, &p.core)?;
    let config = owner.enter(session, &p.config)?;
    let models = owner.enter(session, &p.models)?;
    let mut a = caller.enter(session, &p.a)?;
    let [mut task, _private] = a.par()?;
    let world = BuildWorld {
        journal: Cell::new(Vec::new()),
        failures: &[],
        start: Instant::now(),
    };
    let (error, (_core, _config)) = async {
        async {
            let error = localside::compiler::run(&mut compiler, (core, config, models), &world)
                .await
                .expect_err("the actual receiver must reject changed resource arguments");
            Ok::<_, ShellError>(error.to_string())
        };
        async {
            drop(task.offer().await?);
            let (mut core, mut config) = async {
                let [mut library, mut configuration] = task.par()?;
                async { Ok::<_, ShellError>(library.recv::<LoanCoreA>().await?) };
                async { Ok::<_, ShellError>(configuration.recv::<LoanConfigA>().await?) };
            }
            .await?;
            let admitted = br#"{"module_name":"module_a"}"#.as_slice();
            let changed = br#"{"module_name":"module_b"}"#.as_slice();
            task.send::<CompileA>(&admitted).await?;
            async {
                async {
                    core.send::<CompileCore>(&admitted).await?;
                };
                async {
                    config.send::<CompileConfig>(&changed).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await?;
            // Keep both real views alive in the parent until receiver rejection.
            Ok::<_, ShellError>((core, config))
        };
    }
    .await?;
    assert!(
        error.contains("compile resources differ from admitted module A input"),
        "{error}"
    );
    assert!(world.journal.take().is_empty());
    Ok(())
}
