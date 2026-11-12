use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
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
use std::cell::Cell;
#[cfg(miri)]
#[path = "../handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// Native data fixture, not the common simulator or an AI performance trial.
// Its journal is the actual material/effect data; no task or resolver reads it.
struct BuildData(Cell<Vec<Value>>);
impl Domain for BuildData {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut journal = self.0.take();
        let data = if tool == "compile_module" {
            let module = arguments["module_name"]
                .as_str()
                .unwrap()
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            let changed = agent == "BUILDER_A"
                || journal
                    .iter()
                    .any(|entry| entry["data"]["agent"] == "BUILDER_A");
            let inputs = json!({"CORE_LIB":{"base":"core","modules":{}},
                "SHARED_TYPES":{"base":"provided types","modules":if changed {
                    json!({"module_a":"declarations from module_a"}) } else { json!({}) }}});
            let bytes = imp::encode(&json!({"module":module,"inputs":inputs})).unwrap();
            json!({"agent":agent,"module":module,"status":"compiled",
                "object":{"id":format!("object:{}",journal.len()),"bytes":String::from_utf8(bytes).unwrap(),
                    "content_sha256":"fixture-only-not-a-cryptographic-receipt", "module":module,
                    "builder":agent,"inputs":inputs}})
        } else {
            assert_eq!(tool, "link_modules");
            let a = &journal
                .iter()
                .rev()
                .find(|entry| entry["data"]["agent"] == "BUILDER_A")
                .unwrap()["data"]["object"];
            let b = &journal
                .iter()
                .rev()
                .find(|entry| entry["data"]["agent"] == "BUILDER_B")
                .unwrap()["data"]["object"];
            let inputs = json!({"module_a":a,"module_b":b});
            json!({"agent":agent,"status":"linked","inputs":inputs,"issues":[],
                "binary":{"id":"fixture-link","bytes":String::from_utf8(imp::encode(&inputs).unwrap()).unwrap()}})
        };
        let result = json!({"tool_name":tool,"success":true,"data":data});
        journal.push(result.clone());
        self.0.set(journal);
        Ok(result)
    }
}

#[test]
fn the_actual_build_and_private_io_source_is_well_formed() {
    assert_eq!(hibana::g::diagnose(&source()), None);
    let observation = programs().observation();
    assert_eq!(observation["programs"].as_array().unwrap().len(), 9);
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn both_first_builders_transfer_the_same_resources_and_only_current_objects_reach_link()
-> ShellResult<()> {
    let p = programs();
    for first in [DecisionArm::Left, DecisionArm::Right] {
        let session = SessionId::new(SESSION);
        let (bus, _io, _terminals) = Bus::open(
            ["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
            [(session, CORE_LIB), (session, SHARED_TYPES)],
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
                &p.service,
                ResolverRef::<FIRST_BUILDER>::decision_state(&first, |arm| Ok(*arm)),
            )
            .map_err(|_| ShellError::message("first builder resolver"))?;
        let mut service = owner.enter(session, &p.service)?;
        let core = owner.enter(session, &p.core)?;
        let types = owner.enter(session, &p.types)?;
        let mut a = caller.enter(session, &p.a)?;
        let mut b = caller.enter(session, &p.b)?;
        let mut integrator = caller.enter(session, &p.integrator)?;
        let [mut a, _a_private] = a.par()?;
        let [mut b, _b_private] = b.par()?;
        let [mut integrator, _i_private] = integrator.par()?;
        let effects = BuildData(Cell::new(Vec::new()));
        hibana_runtime::race(
            async {
                async {
                    localside::resources::run(&mut service, (core, types), &effects).await?;
                };
                async {
                    drop(a.offer().await?);
                    let (mut core, mut types) = async {
                        let [mut library, mut definitions] = a.par()?;
                        async { Ok::<_, ShellError>(library.recv::<CoreLoan>().await?) };
                        async { Ok::<_, ShellError>(definitions.recv::<TypesLoan>().await?) };
                    }
                    .await?;
                    let arguments = br#"{"module_name":"module_a"}"#.as_slice();
                    a.send::<CompileA>(&arguments).await?;
                    let (library, definitions) = async {
                        async {
                            core.send::<CompileCore>(&arguments).await?;
                            Ok::<_, ShellError>(core.recv::<CoreCompiled>().await?.to_vec())
                        };
                        async {
                            types.send::<CompileTypes>(&arguments).await?;
                            Ok::<_, ShellError>(types.recv::<TypesCompiled>().await?.to_vec())
                        };
                    }
                    .await?;
                    assert_eq!(library, definitions);
                    async {
                        let [mut notification, mut handoff] = a.par()?;
                        async {
                            notification.send::<BuiltA>(&library.as_slice()).await?;
                        };
                        async {
                            handoff.send::<TypesUpdated>(&library.as_slice()).await?;
                            let [mut library, mut definitions] = handoff.par()?;
                            async {
                                library.send::<CoreLoan>(core).await?;
                            };
                            async {
                                definitions.send::<TypesLoan>(types).await?;
                            };
                            Ok::<(), ShellError>(())
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                };
                async {
                    loop {
                        let incoming = b.offer().await?;
                        if incoming.label() == TypesUpdated::LOGICAL_LABEL {
                            let actual = imp::decode(incoming.recv::<TypesUpdated>().await?)?;
                            imp::compiled(&actual, "BUILDER_A", "module_a")?;
                        } else {
                            drop(incoming);
                        }
                        let (mut core, mut types) = async {
                            let [mut library, mut definitions] = b.par()?;
                            async { Ok::<_, ShellError>(library.recv::<CoreLoan>().await?) };
                            async { Ok::<_, ShellError>(definitions.recv::<TypesLoan>().await?) };
                        }
                        .await?;
                        let arguments = br#"{"module_name":"module_b"}"#.as_slice();
                        b.send::<CompileB>(&arguments).await?;
                        let (library, definitions) = async {
                            async {
                                core.send::<CompileCore>(&arguments).await?;
                                Ok::<_, ShellError>(core.recv::<CoreCompiled>().await?.to_vec())
                            };
                            async {
                                types.send::<CompileTypes>(&arguments).await?;
                                Ok::<_, ShellError>(types.recv::<TypesCompiled>().await?.to_vec())
                            };
                        }
                        .await?;
                        assert_eq!(library, definitions);
                        let next = b.offer().await?;
                        match next.label() {
                            CoreLoan::LOGICAL_LABEL | TypesLoan::LOGICAL_LABEL => {
                                drop(next);
                                async {
                                    let [mut library, mut definitions] = b.par()?;
                                    async {
                                        library.send::<CoreLoan>(core).await?;
                                    };
                                    async {
                                        definitions.send::<TypesLoan>(types).await?;
                                    };
                                    Ok::<(), ShellError>(())
                                }
                                .await?;
                            }
                            BuiltB::LOGICAL_LABEL
                            | ReturnCore::LOGICAL_LABEL
                            | ReturnTypes::LOGICAL_LABEL => {
                                drop(next);
                                async {
                                    let [mut notification, mut core_return, mut types_return] =
                                        b.par()?;
                                    async {
                                        notification.send::<BuiltB>(&library.as_slice()).await?;
                                    };
                                    async {
                                        core_return.send::<ReturnCore>(core).await?;
                                    };
                                    async {
                                        types_return.send::<ReturnTypes>(types).await?;
                                    };
                                    Ok::<(), ShellError>(())
                                }
                                .await?;
                                break;
                            }
                            _ => {
                                return Err(ShellError::message("unexpected builder continuation"));
                            }
                        }
                    }
                };
                async {
                    drop(integrator.offer().await?);
                    let (a, b) = async {
                        let [mut a, mut b] = integrator.par()?;
                        async { imp::decode(a.recv::<BuiltA>().await?) };
                        async { imp::decode(b.recv::<BuiltB>().await?) };
                    }
                    .await?;
                    let objects = [
                        imp::compiled(&a, "BUILDER_A", "module_a")?,
                        imp::compiled(&b, "BUILDER_B", "module_b")?,
                    ];
                    integrator
                        .send::<Link>(&br#"{"modules":"module_a,module_b"}"#.as_slice())
                        .await?;
                    let linked = imp::decode(integrator.recv::<Linked>().await?)?;
                    imp::linked(&linked, &objects)?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                120
            } else {
                10
            })),
        )
        .await?;
        let journal = effects.0.take();
        let agents: Vec<_> = journal
            .iter()
            .map(|value| value["data"]["agent"].as_str().unwrap())
            .collect();
        assert_eq!(
            agents,
            match first {
                DecisionArm::Left => vec!["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
                DecisionArm::Right => vec!["BUILDER_B", "BUILDER_A", "BUILDER_B", "INTEGRATOR"],
            }
        );
        if first == DecisionArm::Right {
            assert_ne!(
                journal[0]["data"]["object"]["inputs"]["SHARED_TYPES"],
                journal[2]["data"]["object"]["inputs"]["SHARED_TYPES"]
            );
        }
    }
    Ok(())
}

#[test]
fn argument_validation_preserves_normalized_names_and_rejects_other_modules_or_extra_fields() {
    assert!(imp::compile_arguments(json!({"module_name":"Module A"}), "module_a").is_ok());
    assert!(imp::compile_arguments(json!({"module_name":"module_b"}), "module_a").is_err());
    assert!(
        imp::compile_arguments(json!({"module_name":"module_a","ignore":true}), "module_a")
            .is_err()
    );
    assert!(imp::link_arguments(json!({"modules":"Module B, Module A"})).is_ok());
    assert!(imp::link_arguments(json!({"modules":"module_a,module_a,module_b"})).is_err());
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn the_production_receiver_rejects_wrong_module_or_either_substituted_resource_before_effect()
-> ShellResult<()> {
    let p = programs();
    let first = DecisionArm::Left;
    for substituted in [0, 1, 2] {
        let session = SessionId::new(SESSION);
        let (bus, _io, _terminals) = Bus::open(
            ["BUILDER_A", "BUILDER_B", "INTEGRATOR"],
            [(session, CORE_LIB), (session, SHARED_TYPES)],
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
                &p.service,
                ResolverRef::<FIRST_BUILDER>::decision_state(&first, |arm| Ok(*arm)),
            )
            .map_err(|_| ShellError::message("initial builder"))?;
        let mut service = owner.enter(session, &p.service)?;
        let core = owner.enter(session, &p.core)?;
        let types = owner.enter(session, &p.types)?;
        let mut a = caller.enter(session, &p.a)?;
        let [mut a, _private] = a.par()?;
        let effects = BuildData(Cell::new(Vec::new()));
        let result = hibana_runtime::race(
            async {
                async {
                    localside::resources::run(&mut service, (core, types), &effects).await?;
                };
                async {
                    drop(a.offer().await?);
                    let (mut core, mut types) = async {
                        let [mut library, mut definitions] = a.par()?;
                        async { Ok::<_, ShellError>(library.recv::<CoreLoan>().await?) };
                        async { Ok::<_, ShellError>(definitions.recv::<TypesLoan>().await?) };
                    }
                    .await?;
                    let correct = br#"{"module_name":"module_a"}"#.as_slice();
                    let wrong = br#"{"module_name":"module_b"}"#.as_slice();
                    a.send::<CompileA>(&if substituted == 0 { wrong } else { correct })
                        .await?;
                    async {
                        async {
                            core.send::<CompileCore>(&if substituted == 1 {
                                wrong
                            } else {
                                correct
                            })
                            .await?;
                        };
                        async {
                            types
                                .send::<CompileTypes>(&if substituted == 2 {
                                    wrong
                                } else {
                                    correct
                                })
                                .await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    core.recv::<CoreCompiled>().await?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                120
            } else {
                5
            })),
        )
        .await;
        let expected = if substituted == 0 {
            "build arguments differ from the assigned module"
        } else {
            "compile resources differ from admitted arguments"
        };
        assert!(
            result
                .as_ref()
                .is_err_and(|error| error.to_string() == expected),
            "{result:?}"
        );
        assert!(
            effects.0.take().is_empty(),
            "malformed/substituted input must not reach Domain::execute"
        );
    }
    Ok(())
}
