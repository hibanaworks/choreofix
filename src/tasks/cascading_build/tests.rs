//! Scripted native Endpoint participants, not Wasm or model qualifications.
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{PairedEffects, ReceivedEffects, ResourceReceipt, SimulatorOperation},
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::cell::Cell;

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
    assert_eq!(
        imp::rebuild_modules(&objects, &tests).unwrap(),
        ["module_a"]
    );
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
fn guest_can_choose_each_of_multiple_stale_modules() {
    let (objects, mut tests) = checked_objects();
    tests[0]["success"] = json!(false);
    tests[2]["success"] = json!(false);
    assert_eq!(
        imp::rebuild_modules(&objects, &tests).unwrap(),
        ["module_a", "module_c"]
    );
    assert_eq!(imp::requested_module(b"module_c\n"), Some(2));
    assert_eq!(imp::requested_module(b"module_a"), Some(0));
    assert!(imp::requested_module(b"unknown").is_none());
    assert!(imp::reconsider(&objects, &tests, 1).is_err());
    for index in [2, 0] {
        assert_eq!(
            imp::reconsider(&objects, &tests, index).unwrap()["module"],
            imp::MODULES[index]
        );
    }
}

#[test]
fn typed_files_reject_wrong_modules_and_unbound_fields() {
    use super::wire::Arguments;
    use hibana::runtime::wire::{Payload, WirePayload};
    assert!(
        Arguments::<A>::validate_payload(Payload::new(br#"{"module_name":"module_a"}"#)).is_ok()
    );
    for bytes in [
        br#"{"module_name":"module_b"}"#.as_slice(),
        br#"{"module_name":"module_a","next":true}"#,
        b"invalid json",
    ] {
        assert!(Arguments::<A>::validate_payload(Payload::new(bytes)).is_err());
    }
    assert!(
        Arguments::<INTEGRATOR>::validate_payload(Payload::new(
            br#"{"modules":"module_c,module_b,module_a"}"#
        ))
        .is_ok()
    );
    assert!(
        Arguments::<INTEGRATOR>::validate_payload(Payload::new(
            br#"{"modules":"module_a,module_a,module_b"}"#
        ))
        .is_err()
    );
}

#[hibana_runtime::main]
#[test]
async fn actual_compiler_rejects_changed_resource_arguments_before_any_effect() -> ShellResult<()> {
    struct Effects(Cell<usize>);
    impl ReceivedEffects for Effects {
        fn apply<'io, P: SimulatorOperation>(
            &'io self,
            _: hibana::runtime::wire::ReceivedPayload<'_, P>,
        ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
            async move {
                self.0.set(self.0.get() + 1);
                Err(ShellError::message("unexpected effect"))
            }
        }
    }
    impl PairedEffects for Effects {
        async fn apply_pair<'a, 'b, P: SimulatorOperation<2>>(
            &self,
            _: impl Into<ResourceReceipt<'a, P>>,
            _: impl Into<ResourceReceipt<'b, P>>,
        ) -> ShellResult<Value> {
            self.0.set(self.0.get() + 1);
            Err(ShellError::message("unexpected compile effect"))
        }
    }
    // The wrong builder is already rejected by the resource payload codec. The
    // real receiver below also rejects different bytes valid for the same builder.
    assert!(
        <super::wire::BuildRequest<A> as hibana::runtime::wire::WirePayload>::validate_payload(
            hibana::runtime::wire::Payload::new(br#"{"module_name":"module_b"}"#),
        )
        .is_err()
    );
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
        [
            (session, CORE),
            (session, CONFIG),
            (session, MODELS),
            (session, A_FILE),
            (session, B_FILE),
            (session, C_FILE),
            (session, INTEGRATOR_FILE),
            (session, TESTER_FILE),
        ],
    );
    let mut owner_slab = vec![0; 2 * 1024 * 1024];
    let mut caller_slab = vec![0; 2 * 1024 * 1024];
    let mut owner_storage = SessionKitStorage::uninit();
    let mut caller_storage = SessionKitStorage::uninit();
    let owner = owner_storage
        .init()
        .rendezvous(&mut owner_slab, &bus.transport)?;
    let caller = caller_storage
        .init()
        .rendezvous(&mut caller_slab, &bus.transport)?;
    let mut compiler = owner.enter(session, &p.compiler)?;
    let core = owner.enter(session, &p.core)?;
    let config = owner.enter(session, &p.config)?;
    let models = owner.enter(session, &p.models)?;
    let mut a = caller.enter(session, &p.a)?;
    let mut a_data = caller.enter(session, &p.a_data)?;
    let a_file = caller.enter(session, &p.a_file)?;
    let mut b_data = caller.enter(session, &p.b_data)?;
    let b_file = caller.enter(session, &p.b_file)?;
    let mut c_data = caller.enter(session, &p.c_data)?;
    let c_file = caller.enter(session, &p.c_file)?;
    let mut integrator_data = caller.enter(session, &p.integrator_data)?;
    let integrator_file = caller.enter(session, &p.integrator_file)?;
    let mut tester_data = caller.enter(session, &p.tester_data)?;
    let tester_file = caller.enter(session, &p.tester_file)?;
    let world = Effects(Cell::new(0));
    async {
        async {
            a_data.send::<Loan<A_FILE>>(a_file).await?;
        };
        async {
            b_data.send::<Loan<B_FILE>>(b_file).await?;
        };
        async {
            c_data.send::<Loan<C_FILE>>(c_file).await?;
        };
        async {
            integrator_data
                .send::<Loan<INTEGRATOR_FILE>>(integrator_file)
                .await?;
        };
        async {
            tester_data.send::<Loan<TESTER_FILE>>(tester_file).await?;
        };
        Ok::<(), ShellError>(())
    };
    let (mut file, b_file, c_file, integrator_file, tester_file) = async {
        let [mut a, mut b, mut c, mut integrator, mut tester] = compiler.par()?;
        async { a.recv::<Loan<A_FILE>>().await };
        async { b.recv::<Loan<B_FILE>>().await };
        async { c.recv::<Loan<C_FILE>>().await };
        async { integrator.recv::<Loan<INTEGRATOR_FILE>>().await };
        async { tester.recv::<Loan<TESTER_FILE>>().await };
    }
    .await?;
    let (error, (_core, _config, _files)) = async {
        async {
            let error = localside::compiler::run(&mut compiler, (core, config, models), &world)
                .await
                .expect_err("changed resource input must be rejected");
            Ok::<_, ShellError>(error.to_string())
        };
        async {
            let ((), (core, config)) = async {
                async {
                    file.send::<CompileFileA>(&super::wire::Arguments::from_bytes(
                        br#"{"module_name":"module_a"}"#,
                    )?)
                    .await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    let admitted = a.recv::<CompileFileA>().await?.into_value();
                    let (mut core, mut config) = async {
                        let [mut library, mut configuration] = a.par()?;
                        async { library.recv::<LoanCoreA>().await };
                        async { configuration.recv::<LoanConfigA>().await };
                    }
                    .await?;
                    let bytes = imp::encode(&admitted)?;
                    a.send::<CompileA>(&bytes.as_slice()).await?;
                    async {
                        async {
                            core.send::<CompileCore<A>>(&super::wire::BuildRequest(&bytes))
                                .await?;
                        };
                        async {
                            config
                                .send::<CompileConfig<A>>(&super::wire::BuildRequest(
                                    br#"{"module_name":"MODULE A"}"#,
                                ))
                                .await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    Ok::<_, ShellError>((core, config))
                };
            }
            .await?;
            Ok::<_, ShellError>((
                core,
                config,
                (file, b_file, c_file, integrator_file, tester_file),
            ))
        };
    }
    .await?;
    assert!(
        error.contains("compile resources differ from admitted module A input"),
        "{error}"
    );
    assert_eq!(world.0.get(), 0);
    Ok(())
}
