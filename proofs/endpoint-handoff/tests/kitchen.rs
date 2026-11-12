//! Compile the canonical whole kitchen global; Endpoint execution follows here.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/kitchen/mod.rs"]
pub mod kitchen;

#[test]
fn kitchen_whole_source_projects_all_roles() {
    let _ = kitchen::global::programs();
}

use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use kitchen::{global::*, imp as data, localside};
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use std::{cell::Cell, time::Duration};
#[cfg(miri)]
#[path = "../../../src/tasks/handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

#[test]
fn kitchen_data_arguments_match_the_original_nine_operations() {
    let requests = [
        json!({"dish":"appetizer"}),
        json!({"dish":"main_course"}),
        json!({"dish":"dessert"}),
        json!({"ingredient":"sauce","for_chef":"CHEF_C"}),
        json!({"ingredient":"garnish","for_chef":"CHEF_A"}),
        json!({"ingredient":"glaze","for_chef":"CHEF_B"}),
        json!({"dish":"appetizer","ingredient":"garnish"}),
        json!({"dish":"main_course","ingredient":"glaze"}),
        json!({"dish":"dessert","ingredient":"sauce"}),
    ];
    macro_rules! target {
        ($index:literal) => {
            for (index, value) in requests.iter().enumerate() {
                assert_eq!(
                    data::Arguments::<$index>::from_bytes(&data::encode(value).unwrap()).is_ok(),
                    index == $index
                );
                let mut changed = value.clone();
                changed["extra"] = json!(true);
                assert!(
                    data::Arguments::<$index>::from_bytes(&data::encode(&changed).unwrap())
                        .is_err()
                );
            }
        };
    }
    target!(0);
    target!(1);
    target!(2);
    target!(3);
    target!(4);
    target!(5);
    target!(6);
    target!(7);
    target!(8);
    assert!(data::Arguments::<9>::from_bytes(br#"{"dish":"appetizer"}"#).is_err());
    assert!(data::Arguments::<255>::from_bytes(b"{}").is_err());
}

#[test]
fn kitchen_data_results_bind_the_actual_recipe() {
    let arguments = json!({"dish":"appetizer","ingredient":"garnish"});
    let result = json!({"tool_name":"combine_dish","success":true,"data":{
        "dish":"appetizer","ingredient":"garnish","chef":"CHEF_A","status":"dish completed",
        "base_event":1,"ingredient_event":2}});
    data::completed("combine_dish", "CHEF_A", &arguments, &result).unwrap();
    for (pointer, value) in [
        ("/tool_name", json!("prepare_base_dish")),
        ("/success", json!(false)),
        ("/data/chef", json!("CHEF_B")),
        ("/data/dish", json!("dessert")),
        ("/data/ingredient", json!("sauce")),
        ("/data/status", json!("cooking")),
        ("/data/base_event", Value::Null),
        ("/data/ingredient_event", Value::Null),
        ("/data/ingredient_event", json!(1)),
    ] {
        let mut changed = result.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(data::completed("combine_dish", "CHEF_A", &arguments, &changed).is_err());
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn kitchen_receivers_reject_changed_requests_before_effect() -> ShellResult<()> {
    use hibana::{
        g,
        runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    };
    struct Untrusted(&'static [u8]);
    impl WireEncode for Untrusted {
        fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
            output
                .get_mut(..self.0.len())
                .ok_or(CodecError::Truncated)?
                .copy_from_slice(self.0);
            Ok(self.0.len())
        }
    }
    impl WirePayload for Untrusted {
        const SCHEMA_ID: u32 = <data::Arguments<0> as WirePayload>::SCHEMA_ID;
        type Decoded<'a> = &'a [u8];
        fn validate_payload(_: Payload<'_>) -> Result<(), CodecError> {
            Ok(())
        }
        fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
            input.as_bytes()
        }
    }
    type WrongOven = g::Msg<6, Untrusted>;
    type WrongStove = g::Msg<9, Untrusted>;
    let p = programs();
    for wrong_oven in [true, false] {
        for bytes in [
            br#"{"dish":"main_course"}"#.as_slice(),
            br#"{"dish":"appetizer","extra":true}"#.as_slice(),
            b"{".as_slice(),
            b"null".as_slice(),
        ] {
            let session = SessionId::new(SESSION);
            let (bus, _, _) = Bus::open(data::CHEFS, [(session, OVEN), (session, STOVE)]);
            let inputs = [const { Cell::new(Value::Null) }; 2];
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
                    &p.cooking,
                    ResolverRef::<FIRST_A>::decision_state(&inputs[0], data::first_a),
                )
                .map_err(|e| ShellError::message(format!("{e:?}")))?;
            let mut cooking = owner.enter(session, &p.cooking)?;
            let oven = owner.enter(session, &p.oven)?;
            let stove = owner.enter(session, &p.stove)?;
            let mut chef = caller.enter(session, &p.a)?;
            let effects = FoodEffects {
                events: Cell::new(Vec::new()),
                ingredient_delay: Duration::ZERO,
            };
            let error = hibana_runtime::race(
                async {
                    async {
                        localside::cooking::run(&mut cooking, (oven, stove), (&bus, &inputs))
                            .await?;
                    };
                    async {
                        bus.fixture_peer::<1>(&effects, &["prepare_base_dish"])
                            .await?;
                    };
                    async {
                        let [mut business, _private] = chef.par()?;
                        let [mut intent, mut work, _own, _received] = business.par()?;
                        let arguments =
                            data::Arguments::<0>::from_bytes(br#"{"dish":"appetizer"}"#)?;
                        intent.send::<BaseIntentA>(&arguments).await?;
                        drop(work.offer().await?);
                        let (mut oven, mut stove) = async {
                            let [mut o, mut s] = work.par()?;
                            async { o.recv::<OvenLoan>().await };
                            async { s.recv::<StoveLoan>().await };
                        }
                        .await?;
                        async {
                            async {
                                if wrong_oven {
                                    oven.send::<WrongOven>(&Untrusted(bytes)).await?;
                                } else {
                                    oven.send::<OvenRequestA>(&arguments).await?;
                                }
                                Ok::<(), ShellError>(())
                            };
                            async {
                                if wrong_oven {
                                    stove.send::<StoveRequestA>(&arguments).await?;
                                } else {
                                    stove.send::<WrongStove>(&Untrusted(bytes)).await?;
                                }
                                Ok::<(), ShellError>(())
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;
                        core::future::pending::<ShellResult<()>>().await?;
                    };
                    Ok::<(), ShellError>(())
                },
                deadline(Duration::from_secs(5)),
            )
            .await
            .expect_err("changed request must be rejected");
            assert!(
                matches!(error, ShellError::Endpoint(_))
                    && error.to_string().contains("Codec(Malformed)"),
                "must reject at the payload boundary rather than hang: {error}"
            );
            assert!(
                effects.events.take().is_empty(),
                "mismatched actual resource request reached Domain::execute"
            );
        }
    }
    Ok(())
}

// Controlled I/O measurements and actual result data; this fixture schedules no
// application effect and grants no operation. The native callers order inputs.
struct FoodEffects {
    events: Cell<Vec<Value>>,
    ingredient_delay: Duration,
}
impl Domain for FoodEffects {
    async fn execute(
        &self,
        chef: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut events = self.events.take();
        let id = events.len();
        let inputs = if tool == "combine_dish" {
            let base = events
                .iter()
                .rev()
                .find(|event| {
                    event["boundary"] == "finish"
                        && event["tool"] == "prepare_base_dish"
                        && event["chef"] == chef
                })
                .map(|event| event["id"].clone());
            let ingredient = events
                .iter()
                .rev()
                .find(|event| {
                    event["boundary"] == "finish"
                        && event["tool"] == "prepare_ingredient"
                        && event["arguments"]["for_chef"] == chef
                        && event["arguments"]["ingredient"] == arguments["ingredient"]
                })
                .map(|event| event["id"].clone());
            match (base, ingredient) {
                (Some(base), Some(ingredient)) => Some((base, ingredient)),
                _ => {
                    self.events.set(events);
                    return Err("combination has no actual completed material inputs".into());
                }
            }
        } else {
            None
        };
        events.push(
            json!({"id":id,"boundary":"start","chef":chef,"tool":tool,"arguments":arguments}),
        );
        self.events.set(events);
        let delay = if tool == "prepare_ingredient" {
            self.ingredient_delay
        } else if tool == "prepare_base_dish" {
            Duration::from_millis(20)
        } else {
            Duration::from_millis(1)
        };
        #[cfg(not(miri))]
        hibana_runtime::host::sleep(delay)
            .await
            .map_err(|e| e.to_string())?;
        #[cfg(miri)]
        let _ = delay;
        let mut result_data = arguments.clone();
        result_data["chef"] = json!(chef);
        result_data["status"] = json!(match tool {
            "prepare_base_dish" => "base dish prepared",
            "prepare_ingredient" => "ingredient prepared",
            "combine_dish" => "dish completed",
            _ => return Err("unknown fixture tool".into()),
        });
        if let Some((base, ingredient)) = inputs {
            result_data["base_event"] = base;
            result_data["ingredient_event"] = ingredient;
        }
        let result = json!({"tool_name":tool,"success":true,"data":result_data});
        let mut events = self.events.take();
        events.push(json!({"id":id,"boundary":"finish","chef":chef,"tool":tool,"arguments":arguments,"result":result}));
        self.events.set(events);
        Ok(result)
    }
}

#[cfg(not(miri))]
#[hibana_runtime::main]
#[test]
async fn kitchen_all_base_orders_keep_independent_dishes_moving() -> ShellResult<()> {
    let p = programs();
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        for wait_for_first_dish in [false, true] {
            for ingredient_millis in [3, 35] {
                let session = SessionId::new(SESSION);
                let (bus, _, _) = Bus::open(data::CHEFS, [(session, OVEN), (session, STOVE)]);
                let effects = FoodEffects {
                    events: Cell::new(Vec::new()),
                    ingredient_delay: Duration::from_millis(ingredient_millis),
                };
                let inputs = [const { Cell::new(Value::Null) }; 2];
                let next = (&inputs[0], &inputs[1]);
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
                        &p.cooking,
                        ResolverRef::<FIRST_A>::decision_state(&inputs[0], data::first_a),
                    )
                    .map_err(|e| ShellError::message(format!("{e:?}")))?;
                owner
                    .set_resolver(
                        &p.cooking,
                        ResolverRef::<FIRST_B>::decision_state(&inputs[0], data::first_b),
                    )
                    .map_err(|e| ShellError::message(format!("{e:?}")))?;
                owner
                    .set_resolver(
                        &p.cooking,
                        ResolverRef::<NEXT>::decision_state(&next, data::next),
                    )
                    .map_err(|e| ShellError::message(format!("{e:?}")))?;
                let mut cooking = owner.enter(session, &p.cooking)?;
                let mut ingredients = owner.enter(session, &p.ingredients)?;
                let mut finish = owner.enter(session, &p.finish)?;
                let oven = owner.enter(session, &p.oven)?;
                let stove = owner.enter(session, &p.stove)?;
                let mut a_io = owner.enter(session, &p.a_io)?;
                let mut b_io = owner.enter(session, &p.b_io)?;
                let mut c_io = owner.enter(session, &p.c_io)?;
                let mut a = caller.enter(session, &p.a)?;
                let mut b = caller.enter(session, &p.b)?;
                let mut c = caller.enter(session, &p.c)?;
                let execution = hibana_runtime::race(
                    async {
                        async {
                            localside::cooking::run(&mut cooking, (oven, stove), (&bus, &inputs))
                                .await?;
                        };
                        async {
                            localside::ingredients::run(&mut ingredients, &bus).await?;
                        };
                        async {
                            localside::finish::run(&mut finish, &bus).await?;
                        };
                        async {
                            bus.fixture_peer::<9>(
                                &effects,
                                &["prepare_base_dish", "prepare_ingredient", "combine_dish"],
                            )
                            .await?;
                        };
                        async {
                            assert_eq!(a_io.recv::<shell::global::Exit>().await?, 0);
                        };
                        async {
                            assert_eq!(b_io.recv::<shell::global::Exit>().await?, 0);
                        };
                        async {
                            assert_eq!(c_io.recv::<shell::global::Exit>().await?, 0);
                        };
                        async {
                            let [mut business, mut private] = a.par()?;
                            async {
                                let [mut intent, mut work, mut own, mut received] =
                                    business.par()?;
                                async {
                                    let rank = order.iter().position(|chef| *chef == 0).unwrap();
                                    if rank > 0 {
                                        core::future::poll_fn(|cx| {
                                            let history = effects.events.take();
                                            let ready = if wait_for_first_dish && rank == 2 {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "finish"
                                                        && e["tool"] == "combine_dish"
                                                        && e["chef"] == data::CHEFS[order[0]]
                                                })
                                            } else {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "start"
                                                        && e["tool"] == "prepare_base_dish"
                                                        && e["chef"] == data::CHEFS[order[rank - 1]]
                                                })
                                            };
                                            effects.events.set(history);
                                            if ready {
                                                core::task::Poll::Ready(())
                                            } else {
                                                cx.waker().wake_by_ref();
                                                core::task::Poll::Pending
                                            }
                                        })
                                        .await;
                                    }
                                    let arguments = data::Arguments::<0>::from_bytes(
                                        &data::encode(&data::expected(0).unwrap())?,
                                    )?;

                                    intent.send::<BaseIntentA>(&arguments).await?;
                                    drop(work.offer().await?);
                                    let (mut oven, mut stove) = async {
                                        let [mut o, mut s] = work.par()?;
                                        async { o.recv::<OvenLoan>().await };
                                        async { s.recv::<StoveLoan>().await };
                                    }
                                    .await?;
                                    async {
                                        async {
                                            oven.send::<OvenRequestA>(&arguments).await?;
                                        };
                                        async {
                                            stove.send::<StoveRequestA>(&arguments).await?;
                                        };
                                        Ok::<(), ShellError>(())
                                    }
                                    .await?;
                                    let (left, right) = async {
                                        async { data::decode(oven.recv::<OvenResult>().await?) };
                                        async { data::decode(stove.recv::<StoveResult>().await?) };
                                    }
                                    .await?;
                                    assert_eq!(left, right);
                                    data::completed(
                                        "prepare_base_dish",
                                        "CHEF_A",
                                        &arguments.value(),
                                        &left,
                                    )?;
                                    async {
                                        let [mut o, mut s] = work.par()?;
                                        async {
                                            o.send::<ReturnOven>(oven).await?;
                                        };
                                        async {
                                            s.send::<ReturnStove>(stove).await?;
                                        };
                                        Ok::<(), ShellError>(())
                                    }
                                    .await?;
                                };
                                async {
                                    async {
                                        let arguments = data::Arguments::<3>::from_bytes(
                                            &data::encode(&data::expected(3).unwrap())?,
                                        )?;
                                        own.send::<PrepareSauce>(&arguments).await?;
                                        let result =
                                            data::decode(own.recv::<SaucePrepared>().await?)?;
                                        data::completed(
                                            "prepare_ingredient",
                                            "CHEF_A",
                                            &arguments.value(),
                                            &result,
                                        )?;
                                    };
                                    async {
                                        let result =
                                            data::decode(received.recv::<Garnish>().await?)?;
                                        assert_eq!(result["success"], true);
                                        assert_eq!(result["data"]["for_chef"], "CHEF_A");
                                        assert_eq!(result["data"]["ingredient"], data::RECEIVED[0]);
                                    };
                                    Ok::<(), ShellError>(())
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?;
                            let arguments = data::Arguments::<6>::from_bytes(&data::encode(
                                &data::expected(6).unwrap(),
                            )?)?;
                            business.send::<CombineA>(&arguments).await?;
                            let result = data::decode(business.recv::<DishA>().await?)?;
                            data::completed("combine_dish", "CHEF_A", &arguments.value(), &result)?;
                            private.send::<shell::global::Exit>(&0_i32).await?;
                        };
                        async {
                            let [mut business, mut private] = b.par()?;
                            async {
                                let [mut intent, mut work, mut own, mut received] =
                                    business.par()?;
                                async {
                                    let rank = order.iter().position(|chef| *chef == 1).unwrap();
                                    if rank > 0 {
                                        core::future::poll_fn(|cx| {
                                            let history = effects.events.take();
                                            let ready = if wait_for_first_dish && rank == 2 {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "finish"
                                                        && e["tool"] == "combine_dish"
                                                        && e["chef"] == data::CHEFS[order[0]]
                                                })
                                            } else {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "start"
                                                        && e["tool"] == "prepare_base_dish"
                                                        && e["chef"] == data::CHEFS[order[rank - 1]]
                                                })
                                            };
                                            effects.events.set(history);
                                            if ready {
                                                core::task::Poll::Ready(())
                                            } else {
                                                cx.waker().wake_by_ref();
                                                core::task::Poll::Pending
                                            }
                                        })
                                        .await;
                                    }
                                    let arguments = data::Arguments::<1>::from_bytes(
                                        &data::encode(&data::expected(1).unwrap())?,
                                    )?;

                                    intent.send::<BaseIntentB>(&arguments).await?;
                                    drop(work.offer().await?);
                                    let (mut oven, mut stove) = async {
                                        let [mut o, mut s] = work.par()?;
                                        async { o.recv::<OvenLoan>().await };
                                        async { s.recv::<StoveLoan>().await };
                                    }
                                    .await?;
                                    async {
                                        async {
                                            oven.send::<OvenRequestB>(&arguments).await?;
                                        };
                                        async {
                                            stove.send::<StoveRequestB>(&arguments).await?;
                                        };
                                        Ok::<(), ShellError>(())
                                    }
                                    .await?;
                                    let (left, right) = async {
                                        async { data::decode(oven.recv::<OvenResult>().await?) };
                                        async { data::decode(stove.recv::<StoveResult>().await?) };
                                    }
                                    .await?;
                                    assert_eq!(left, right);
                                    data::completed(
                                        "prepare_base_dish",
                                        "CHEF_B",
                                        &arguments.value(),
                                        &left,
                                    )?;
                                    async {
                                        let [mut o, mut s] = work.par()?;
                                        async {
                                            o.send::<ReturnOven>(oven).await?;
                                        };
                                        async {
                                            s.send::<ReturnStove>(stove).await?;
                                        };
                                        Ok::<(), ShellError>(())
                                    }
                                    .await?;
                                };
                                async {
                                    async {
                                        let arguments = data::Arguments::<4>::from_bytes(
                                            &data::encode(&data::expected(4).unwrap())?,
                                        )?;
                                        own.send::<PrepareGarnish>(&arguments).await?;
                                        let result =
                                            data::decode(own.recv::<GarnishPrepared>().await?)?;
                                        data::completed(
                                            "prepare_ingredient",
                                            "CHEF_B",
                                            &arguments.value(),
                                            &result,
                                        )?;
                                    };
                                    async {
                                        let result = data::decode(received.recv::<Glaze>().await?)?;
                                        assert_eq!(result["success"], true);
                                        assert_eq!(result["data"]["for_chef"], "CHEF_B");
                                        assert_eq!(result["data"]["ingredient"], data::RECEIVED[1]);
                                    };
                                    Ok::<(), ShellError>(())
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?;
                            let arguments = data::Arguments::<7>::from_bytes(&data::encode(
                                &data::expected(7).unwrap(),
                            )?)?;
                            business.send::<CombineB>(&arguments).await?;
                            let result = data::decode(business.recv::<DishB>().await?)?;
                            data::completed("combine_dish", "CHEF_B", &arguments.value(), &result)?;
                            private.send::<shell::global::Exit>(&0_i32).await?;
                        };
                        async {
                            let [mut business, mut private] = c.par()?;
                            async {
                                let [mut intent, mut work, mut received, mut own] =
                                    business.par()?;
                                async {
                                    let rank = order.iter().position(|chef| *chef == 2).unwrap();
                                    if rank > 0 {
                                        core::future::poll_fn(|cx| {
                                            let history = effects.events.take();
                                            let ready = if wait_for_first_dish && rank == 2 {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "finish"
                                                        && e["tool"] == "combine_dish"
                                                        && e["chef"] == data::CHEFS[order[0]]
                                                })
                                            } else {
                                                history.iter().any(|e| {
                                                    e["boundary"] == "start"
                                                        && e["tool"] == "prepare_base_dish"
                                                        && e["chef"] == data::CHEFS[order[rank - 1]]
                                                })
                                            };
                                            effects.events.set(history);
                                            if ready {
                                                core::task::Poll::Ready(())
                                            } else {
                                                cx.waker().wake_by_ref();
                                                core::task::Poll::Pending
                                            }
                                        })
                                        .await;
                                    }
                                    let arguments = data::Arguments::<2>::from_bytes(
                                        &data::encode(&data::expected(2).unwrap())?,
                                    )?;

                                    intent.send::<BaseIntentC>(&arguments).await?;
                                    let mut oven = work.recv::<OvenLoan>().await?;
                                    oven.send::<OvenRequestC>(&arguments).await?;
                                    let result = data::decode(oven.recv::<OvenResult>().await?)?;
                                    data::completed(
                                        "prepare_base_dish",
                                        "CHEF_C",
                                        &arguments.value(),
                                        &result,
                                    )?;
                                    work.send::<ReturnOven>(oven).await?;
                                };
                                async {
                                    async {
                                        let arguments = data::Arguments::<5>::from_bytes(
                                            &data::encode(&data::expected(5).unwrap())?,
                                        )?;
                                        own.send::<PrepareGlaze>(&arguments).await?;
                                        let result =
                                            data::decode(own.recv::<GlazePrepared>().await?)?;
                                        data::completed(
                                            "prepare_ingredient",
                                            "CHEF_C",
                                            &arguments.value(),
                                            &result,
                                        )?;
                                    };
                                    async {
                                        let result = data::decode(received.recv::<Sauce>().await?)?;
                                        assert_eq!(result["success"], true);
                                        assert_eq!(result["data"]["for_chef"], "CHEF_C");
                                        assert_eq!(result["data"]["ingredient"], data::RECEIVED[2]);
                                    };
                                    Ok::<(), ShellError>(())
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?;
                            let arguments = data::Arguments::<8>::from_bytes(&data::encode(
                                &data::expected(8).unwrap(),
                            )?)?;
                            business.send::<CombineC>(&arguments).await?;
                            let result = data::decode(business.recv::<DishC>().await?)?;
                            data::completed("combine_dish", "CHEF_C", &arguments.value(), &result)?;
                            private.send::<shell::global::Exit>(&0_i32).await?;
                        };
                        Ok::<(), ShellError>(())
                    },
                    hibana_runtime::host::deadline(Duration::from_secs(5)),
                )
                .await;
                let events = effects.events.take();
                if let Some(directory) = std::env::var_os("CHOREOFIX_KITCHEN_CASE_OUTPUT") {
                    let path = std::path::PathBuf::from(directory);
                    std::fs::create_dir_all(&path)?;
                    std::fs::write(
                        path.join(format!(
                            "{}{}{}-{}-{ingredient_millis}.json",
                            order[0],
                            order[1],
                            order[2],
                            u8::from(wait_for_first_dish)
                        )),
                        data::encode(
                            &json!({"controlled_native_fixture":true,"model_calls":0,"order":order,
                            "wait_for_first_dish":wait_for_first_dish,"ingredient_millis":ingredient_millis,
                            "events":events,"frames":bus.transport.take_observations(),"error":execution.as_ref().err().map(ToString::to_string)}),
                        )?,
                    )?;
                }
                execution.map_err(|e|ShellError::message(format!("order={order:?} wait={wait_for_first_dish} ingredients={ingredient_millis}: {e}")))?;
                assert_eq!(events.len(), 18);
                let mut active = 0;
                let mut maximum = 0;
                let mut actual_order = Vec::new();
                for event in &events {
                    if event["tool"] == "prepare_base_dish" {
                        if event["boundary"] == "start" {
                            active += 1;
                            maximum = maximum.max(active);
                            actual_order.push(event["chef"].as_str().unwrap());
                        } else {
                            active -= 1;
                        }
                    }
                }
                assert_eq!(active, 0);
                assert_eq!(maximum, 1);
                assert_eq!(actual_order, order.map(|index| data::CHEFS[index]));
                if wait_for_first_dish {
                    let first = events
                        .iter()
                        .position(|e| {
                            e["boundary"] == "finish"
                                && e["tool"] == "combine_dish"
                                && e["chef"] == data::CHEFS[order[0]]
                        })
                        .unwrap();
                    let last = events
                        .iter()
                        .position(|e| {
                            e["boundary"] == "start"
                                && e["tool"] == "prepare_base_dish"
                                && e["chef"] == data::CHEFS[order[2]]
                        })
                        .unwrap();
                    assert!(first < last);
                }
            }
        }
    }
    Ok(())
}
