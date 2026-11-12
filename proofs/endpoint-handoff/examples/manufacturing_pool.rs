//! Two real reusable tool values, observed through complete Endpoint capture.
//! Controlled native mechanism fixture, not the full 11M application.
use hibana::{Endpoint, g, runtime::program::Projectable};
const OWNER: u8 = 3;
const TOOL0: u8 = 4;
const TOOL1: u8 = 5;
const SERVICE: u8 = 6;
const FINISH: u8 = 7;
type Loan0<'r> = g::Msg<1, Endpoint<'r, TOOL0>>;
type Return0<'r> = g::Msg<2, Endpoint<'r, TOOL0>>;
type Loan1<'r> = g::Msg<3, Endpoint<'r, TOOL1>>;
type Return1<'r> = g::Msg<4, Endpoint<'r, TOOL1>>;
type Reply0 = g::Msg<5, u8>;
type Reply1 = g::Msg<6, u8>;
type Assigned<const LABEL: u8> = g::Msg<LABEL, u8>;
type Request0 = g::Msg<20, u8>;
type Request1 = g::Msg<21, u8>;
type Busy = g::Msg<31, u8>;
type Stop0 = g::Msg<60, ()>;
type Stop1 = g::Msg<61, ()>;
type Halt = g::Msg<62, ()>;
type Closed = g::Msg<63, ()>;
type Intent<const LABEL: u8> = g::Msg<LABEL, u8>;
type Done = g::Msg<74, ()>;
type Park0<'r> = g::Msg<75, Endpoint<'r, TOOL0>>;
type Park1<'r> = g::Msg<76, Endpoint<'r, TOOL1>>;
type Released0 = g::Msg<80, ()>;
type Released1 = g::Msg<81, ()>;

fn broadcast<M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, 0, M>(),
        g::seq(
            g::send::<OWNER, 1, M>(),
            g::seq(g::send::<OWNER, 2, M>(), g::send::<OWNER, SERVICE, M>()),
        ),
    )
}
fn lease0<const WORKER: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        broadcast::<Assigned<LABEL>>(),
        g::seq(
            g::send::<OWNER, WORKER, Loan0>(),
            g::seq(
                g::send::<TOOL0, SERVICE, Request0>(),
                g::seq(
                    g::send::<SERVICE, TOOL0, Reply0>(),
                    g::seq(
                        g::send::<WORKER, OWNER, Return0>(),
                        broadcast::<Released0>(),
                    ),
                ),
            ),
        ),
    )
}
fn lease1<const WORKER: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        broadcast::<Assigned<LABEL>>(),
        g::seq(
            g::send::<OWNER, WORKER, Loan1>(),
            g::seq(
                g::send::<TOOL1, SERVICE, Request1>(),
                g::seq(
                    g::send::<SERVICE, TOOL1, Reply1>(),
                    g::seq(
                        g::send::<WORKER, OWNER, Return1>(),
                        broadcast::<Released1>(),
                    ),
                ),
            ),
        ),
    )
}
fn source() -> impl Projectable {
    let input = g::par(
        g::send::<0, OWNER, Intent<70>>().roll(),
        g::par(
            g::send::<1, OWNER, Intent<71>>().roll(),
            g::send::<2, OWNER, Intent<72>>().roll(),
        ),
    );
    let leases0 = g::route(
        lease0::<0, 10>(),
        g::route(lease0::<1, 11>(), lease0::<2, 12>()),
    );
    let leases1 = g::route(
        lease1::<0, 13>(),
        g::route(lease1::<1, 14>(), lease1::<2, 15>()),
    );
    let close0 = g::seq(broadcast::<Stop0>(), g::send::<OWNER, FINISH, Park0>());
    let close1 = g::seq(broadcast::<Stop1>(), g::send::<OWNER, FINISH, Park1>());
    let slot0 = g::route(g::seq(leases0.roll(), close0), close0);
    let slot1 = g::route(g::seq(leases1.roll(), close1), close1);
    let busy = g::route(
        g::seq(broadcast::<Busy>().roll(), broadcast::<Closed>()),
        broadcast::<Closed>(),
    );
    let completion = g::seq(
        g::par(
            g::send::<0, FINISH, Done>(),
            g::par(g::send::<1, FINISH, Done>(), g::send::<2, FINISH, Done>()),
        ),
        g::send::<FINISH, OWNER, Halt>(),
    );
    g::par(
        input,
        g::par(slot0, g::par(slot1, g::par(busy, completion))),
    )
}

// A sum of actual received I/O values. It stores no owner or protocol phase.
enum Incoming<'r> {
    Request(u8),
    Returned0(Endpoint<'r, TOOL0>),
    Returned1(Endpoint<'r, TOOL1>),
    Halt,
}

#[hibana_runtime::main]
async fn main() -> choreofix_handoff_check::error::ShellResult<()> {
    use choreofix_handoff_check::{bus::Bus, error::ShellError};
    use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
    use std::{cell::Cell, time::Duration};
    let mut audits = Vec::new();
    let inputs = std::env::args_os().skip(1).collect::<Vec<_>>();
    if inputs.is_empty() {
        for first_free in [TOOL0, TOOL1] {
            for repetitions in [1, 2, 4] {
                let source = source();
                assert!(g::diagnose(&source).is_none(), "{:?}", g::diagnose(&source));
                let owner_program = project::<OWNER>(&source);
                let tool0_program = project::<TOOL0>(&source);
                let tool1_program = project::<TOOL1>(&source);
                let a_program = project::<0>(&source);
                let b_program = project::<1>(&source);
                let c_program = project::<2>(&source);
                let service_program = project::<SERVICE>(&source);
                let finish_program = project::<FINISH>(&source);
                let session = SessionId::new(115);
                let (bus, _, _) = Bus::open(["A", "B", "C"], [(session, TOOL0), (session, TOOL1)]);
                let captured: [Cell<Vec<hibana::runtime::tap::TapEvent>>; 6] =
                    core::array::from_fn(|_| Cell::new(Vec::new()));
                let observed_order = Cell::new(Vec::new());
                let effects = Cell::new(Vec::new());
                let observers = [0usize, 1, 2, 3, 4, 5].map(|producer| {
                    let captured = &captured;
                    let observed_order = &observed_order;
                    move |event| {
                        let mut rows = captured[producer].take();
                        rows.push(event);
                        captured[producer].set(rows);
                        let mut order = observed_order.take();
                        order.push(producer);
                        observed_order.set(order);
                    }
                });
                let mut slabs: [Vec<u8>; 6] = core::array::from_fn(|_| vec![0; 512 * 1024]);
                let mut storages: [SessionKitStorage<_>; 6] =
                    core::array::from_fn(|_| SessionKitStorage::uninit());
                let mut contexts = storages.each_mut().map(|storage| storage.init());
                let runtimes = contexts
                    .each_mut()
                    .into_iter()
                    .zip(slabs.each_mut())
                    .map(|(storage, slab)| storage.rendezvous(slab, &bus.transport))
                    .collect::<core::result::Result<Vec<_>, _>>()?;
                for (runtime, observer) in runtimes.iter().zip(&observers) {
                    runtime.observe(observer);
                }
                let mut owner = runtimes[0].enter(session, &owner_program)?;
                let tool0 = runtimes[0].enter(session, &tool0_program)?;
                let tool1 = runtimes[0].enter(session, &tool1_program)?;
                let mut a = runtimes[1].enter(session, &a_program)?;
                let mut b = runtimes[2].enter(session, &b_program)?;
                let mut c = runtimes[3].enter(session, &c_program)?;
                let mut service = runtimes[4].enter(session, &service_program)?;
                let mut finish = runtimes[5].enter(session, &finish_program)?;
                hibana_runtime::race(async {
            async {
                let [mut ia,mut ib,mut ic,mut slot0,mut slot1,mut busy,mut halt]=owner.par()?;
                let(mut tool0,mut tool1)=(Some(tool0),Some(tool1));
                loop {
                    let incoming=hibana_runtime::race(
                        async {
                            hibana_runtime::race(
                                async {
                                    hibana_runtime::race(
                                        async {Ok::<_,ShellError>(Incoming::Request(ia.recv::<Intent<70>>().await?))},
                                        async {Ok::<_,ShellError>(Incoming::Request(ib.recv::<Intent<71>>().await?))},
                                    ).await
                                },
                                async {
                                    hibana_runtime::race(
                                        async {Ok::<_,ShellError>(Incoming::Request(ic.recv::<Intent<72>>().await?))},
                                        async {halt.recv::<Halt>().await?;Ok::<_,ShellError>(Incoming::Halt)},
                                    ).await
                                },
                            ).await
                        },
                        async {
                            hibana_runtime::race(
                                async {
                                    if tool0.is_none(){Ok::<_,ShellError>(Incoming::Returned0(slot0.recv::<Return0>().await?))}
                                    else {std::future::pending().await}
                                },
                                async {
                                    if tool1.is_none(){Ok::<_,ShellError>(Incoming::Returned1(slot1.recv::<Return1>().await?))}
                                    else {std::future::pending().await}
                                },
                            ).await
                        },
                    ).await?;
                    match incoming {
                        Incoming::Request(worker)=>{
                            if let Some(tool)=tool0.take() {
                                match worker {
                                    0=>{slot0.send::<Assigned<10>>(&worker).await?;
slot0.send::<Assigned<10>>(&worker).await?;
slot0.send::<Assigned<10>>(&worker).await?;
slot0.send::<Assigned<10>>(&worker).await?;},
                                    1=>{slot0.send::<Assigned<11>>(&worker).await?;
slot0.send::<Assigned<11>>(&worker).await?;
slot0.send::<Assigned<11>>(&worker).await?;
slot0.send::<Assigned<11>>(&worker).await?;},
                                    2=>{slot0.send::<Assigned<12>>(&worker).await?;
slot0.send::<Assigned<12>>(&worker).await?;
slot0.send::<Assigned<12>>(&worker).await?;
slot0.send::<Assigned<12>>(&worker).await?;},
                                    _=>unreachable!(),
                                }
                                slot0.send::<Loan0>(tool).await?;
                            } else if let Some(tool)=tool1.take() {
                                match worker {
                                    0=>{slot1.send::<Assigned<13>>(&worker).await?;
slot1.send::<Assigned<13>>(&worker).await?;
slot1.send::<Assigned<13>>(&worker).await?;
slot1.send::<Assigned<13>>(&worker).await?;},
                                    1=>{slot1.send::<Assigned<14>>(&worker).await?;
slot1.send::<Assigned<14>>(&worker).await?;
slot1.send::<Assigned<14>>(&worker).await?;
slot1.send::<Assigned<14>>(&worker).await?;},
                                    2=>{slot1.send::<Assigned<15>>(&worker).await?;
slot1.send::<Assigned<15>>(&worker).await?;
slot1.send::<Assigned<15>>(&worker).await?;
slot1.send::<Assigned<15>>(&worker).await?;},
                                    _=>unreachable!(),
                                }
                                slot1.send::<Loan1>(tool).await?;
                            } else {
                                busy.send::<Busy>(&worker).await?;
busy.send::<Busy>(&worker).await?;
busy.send::<Busy>(&worker).await?;
busy.send::<Busy>(&worker).await?;
                            }
                        }
                        Incoming::Returned0(tool)=>{
                            assert!(tool0.replace(tool).is_none());
                            slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
                        }
                        Incoming::Returned1(tool)=>{
                            assert!(tool1.replace(tool).is_none());
                            slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
                        }
                        Incoming::Halt=>break,
                    }
                }
                let tool0=match tool0 {Some(tool)=>tool,None=>{
                    let tool=slot0.recv::<Return0>().await?;
                    slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
slot0.send::<Released0>(&()).await?;
                    tool
                }};
                let tool1=match tool1 {Some(tool)=>tool,None=>{
                    let tool=slot1.recv::<Return1>().await?;
                    slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
slot1.send::<Released1>(&()).await?;
                    tool
                }};
                slot0.send::<Stop0>(&()).await?;
slot0.send::<Stop0>(&()).await?;
slot0.send::<Stop0>(&()).await?;
slot0.send::<Stop0>(&()).await?;
                slot0.send::<Park0>(tool0).await?;
                slot1.send::<Stop1>(&()).await?;
slot1.send::<Stop1>(&()).await?;
slot1.send::<Stop1>(&()).await?;
slot1.send::<Stop1>(&()).await?;
                slot1.send::<Park1>(tool1).await?;
                busy.send::<Closed>(&()).await?;
busy.send::<Closed>(&()).await?;
busy.send::<Closed>(&()).await?;
busy.send::<Closed>(&()).await?;
            };
            async {
                let [mut input,mut slot0,mut slot1,mut busy,mut done]=a.par()?;
                input.send::<Intent<70>>(&0).await?;
                slot0.recv::<Assigned<10>>().await?;
                let mut tool=slot0.recv::<Loan0>().await?;
                tool.send::<Request0>(&0).await?;
                assert_eq!(tool.recv::<Reply0>().await?,0);
                slot0.send::<Return0>(tool).await?;
                slot0.recv::<Released0>().await?;
                done.send::<Done>(&()).await?;
                async {
                    loop {
                        let branch=slot0.offer().await?;
                        match branch.label() {
                            10=>{branch.recv::<Assigned<10>>().await?;}
                            11=>{branch.recv::<Assigned<11>>().await?;}
                            12=>{branch.recv::<Assigned<12>>().await?;}
                            80=>{branch.recv::<Released0>().await?;}
                            60=>{branch.recv::<Stop0>().await?;break;}
                            label=>return Err(ShellError::message(format!("A/0 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=slot1.offer().await?;
                        match branch.label() {
                            13=>{branch.recv::<Assigned<13>>().await?;}
                            14=>{branch.recv::<Assigned<14>>().await?;}
                            15=>{branch.recv::<Assigned<15>>().await?;}
                            81=>{branch.recv::<Released1>().await?;}
                            61=>{branch.recv::<Stop1>().await?;break;}
                            label=>return Err(ShellError::message(format!("A/1 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=busy.offer().await?;
                        match branch.label() {
                            31=>{assert_eq!(branch.recv::<Busy>().await?,2);}
                            63=>{branch.recv::<Closed>().await?;break;}
                            label=>return Err(ShellError::message(format!("A/busy label {label}"))),
                        }
                    }
                };
            };
            async {
                assert_eq!(b.recv::<Assigned<10>>().await?,0);
                let [mut input,mut slot0,mut slot1,mut busy,mut done]=b.par()?;
                input.send::<Intent<71>>(&1).await?;
                slot1.recv::<Assigned<14>>().await?;
                let mut tool=slot1.recv::<Loan1>().await?;
                tool.send::<Request1>(&1).await?;
                assert_eq!(tool.recv::<Reply1>().await?,1);
                slot1.send::<Return1>(tool).await?;
                slot1.recv::<Released1>().await?;
                done.send::<Done>(&()).await?;
                async {
                    loop {
                        let branch=slot0.offer().await?;
                        match branch.label() {
                            10=>{branch.recv::<Assigned<10>>().await?;}
                            11=>{branch.recv::<Assigned<11>>().await?;}
                            12=>{branch.recv::<Assigned<12>>().await?;}
                            80=>{branch.recv::<Released0>().await?;}
                            60=>{branch.recv::<Stop0>().await?;break;}
                            label=>return Err(ShellError::message(format!("B/0 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=slot1.offer().await?;
                        match branch.label() {
                            13=>{branch.recv::<Assigned<13>>().await?;}
                            14=>{branch.recv::<Assigned<14>>().await?;}
                            15=>{branch.recv::<Assigned<15>>().await?;}
                            81=>{branch.recv::<Released1>().await?;}
                            61=>{branch.recv::<Stop1>().await?;break;}
                            label=>return Err(ShellError::message(format!("B/1 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=busy.offer().await?;
                        match branch.label() {
                            31=>{assert_eq!(branch.recv::<Busy>().await?,2);}
                            63=>{branch.recv::<Closed>().await?;break;}
                            label=>return Err(ShellError::message(format!("B/busy label {label}"))),
                        }
                    }
                };
            };
            async {
                c.recv::<Assigned<10>>().await?;
                c.recv::<Assigned<14>>().await?;
                let [mut input,mut slot0,mut slot1,mut busy,mut done]=c.par()?;
                if first_free==TOOL0 {
                    for _ in 0..repetitions {
                        let mut tool=loop {
                            input.send::<Intent<72>>(&2).await?;
                            let (channel,branch)=hibana_runtime::race(
                                async {Ok::<_,ShellError>((0,slot0.offer().await?))},
                                async {Ok::<_,ShellError>((1,busy.offer().await?))},
                            ).await?;
                            match (channel,branch.label()) {
                                (0,12)=>{branch.recv::<Assigned<12>>().await?;break slot0.recv::<Loan0>().await?;}
                                (1,31)=>{
                                    assert_eq!(branch.recv::<Busy>().await?,2);
                                    slot0.recv::<Released0>().await?;
                                }
                                (_,label)=>return Err(ShellError::message(format!("C/0 admission label {label}"))),
                            }
                        };
                        tool.send::<Request0>(&2).await?;
                        assert_eq!(tool.recv::<Reply0>().await?,2);
                        slot0.send::<Return0>(tool).await?;
                        slot0.recv::<Released0>().await?;
                    }
                } else {
                    for _ in 0..repetitions {
                        let mut tool=loop {
                            input.send::<Intent<72>>(&2).await?;
                            let (channel,branch)=hibana_runtime::race(
                                async {Ok::<_,ShellError>((0,slot1.offer().await?))},
                                async {Ok::<_,ShellError>((1,busy.offer().await?))},
                            ).await?;
                            match (channel,branch.label()) {
                                (0,15)=>{branch.recv::<Assigned<15>>().await?;break slot1.recv::<Loan1>().await?;}
                                (1,31)=>{
                                    assert_eq!(branch.recv::<Busy>().await?,2);
                                    slot1.recv::<Released1>().await?;
                                }
                                (_,label)=>return Err(ShellError::message(format!("C/1 admission label {label}"))),
                            }
                        };
                        tool.send::<Request1>(&2).await?;
                        assert_eq!(tool.recv::<Reply1>().await?,2);
                        slot1.send::<Return1>(tool).await?;
                        slot1.recv::<Released1>().await?;
                    }
                }
                done.send::<Done>(&()).await?;
                async {
                    loop {
                        let branch=slot0.offer().await?;
                        match branch.label() {
                            10=>{branch.recv::<Assigned<10>>().await?;}
                            11=>{branch.recv::<Assigned<11>>().await?;}
                            12=>{branch.recv::<Assigned<12>>().await?;}
                            80=>{branch.recv::<Released0>().await?;}
                            60=>{branch.recv::<Stop0>().await?;break;}
                            label=>return Err(ShellError::message(format!("C/0 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=slot1.offer().await?;
                        match branch.label() {
                            13=>{branch.recv::<Assigned<13>>().await?;}
                            14=>{branch.recv::<Assigned<14>>().await?;}
                            15=>{branch.recv::<Assigned<15>>().await?;}
                            81=>{branch.recv::<Released1>().await?;}
                            61=>{branch.recv::<Stop1>().await?;break;}
                            label=>return Err(ShellError::message(format!("C/1 label {label}"))),
                        }
                    }
                };
                async {
                    loop {
                        let branch=busy.offer().await?;
                        match branch.label() {
                            31=>{branch.recv::<Busy>().await?;}
                            63=>{branch.recv::<Closed>().await?;break;}
                            label=>return Err(ShellError::message(format!("C/busy label {label}"))),
                        }
                    }
                };
            };
            async {
                let [mut slot0,mut slot1,mut busy]=service.par()?;
                async {
                    loop {
                        let branch=slot0.offer().await?;
                        let worker=match branch.label() {
                            10=>branch.recv::<Assigned<10>>().await?,
                            11=>branch.recv::<Assigned<11>>().await?,
                            12=>branch.recv::<Assigned<12>>().await?,
                            80=>{branch.recv::<Released0>().await?;continue;}
                            60=>{branch.recv::<Stop0>().await?;break;}
                            label=>return Err(ShellError::message(format!("service/0 label {label}"))),
                        };
                        assert_eq!(slot0.recv::<Request0>().await?,worker);
                        let mut rows=effects.take();rows.push(serde_json::json!({"kind":"start","worker":worker,"tool":TOOL0}));effects.set(rows);
                        hibana_runtime::host::sleep(Duration::from_millis(if worker==0 {if first_free==TOOL0 {40}else{300}}else{5})).await?;
                        let mut rows=effects.take();rows.push(serde_json::json!({"kind":"finish","worker":worker,"tool":TOOL0}));effects.set(rows);
                        slot0.send::<Reply0>(&worker).await?;
                    }
                };
                async {
                    loop {
                        let branch=slot1.offer().await?;
                        let worker=match branch.label() {
                            13=>branch.recv::<Assigned<13>>().await?,
                            14=>branch.recv::<Assigned<14>>().await?,
                            15=>branch.recv::<Assigned<15>>().await?,
                            81=>{branch.recv::<Released1>().await?;continue;}
                            61=>{branch.recv::<Stop1>().await?;break;}
                            label=>return Err(ShellError::message(format!("service/1 label {label}"))),
                        };
                        assert_eq!(slot1.recv::<Request1>().await?,worker);
                        let mut rows=effects.take();rows.push(serde_json::json!({"kind":"start","worker":worker,"tool":TOOL1}));effects.set(rows);
                        hibana_runtime::host::sleep(Duration::from_millis(if worker==1 {if first_free==TOOL1 {40}else{300}}else{5})).await?;
                        let mut rows=effects.take();rows.push(serde_json::json!({"kind":"finish","worker":worker,"tool":TOOL1}));effects.set(rows);
                        slot1.send::<Reply1>(&worker).await?;
                    }
                };
                async {
                    loop {
                        let branch=busy.offer().await?;
                        match branch.label() {
                            31=>{branch.recv::<Busy>().await?;}
                            63=>{branch.recv::<Closed>().await?;break;}
                            label=>return Err(ShellError::message(format!("service/busy label {label}"))),
                        }
                    }
                };
            };
            async {
                finish.recv::<Done>().await?;
                finish.recv::<Done>().await?;
                finish.recv::<Done>().await?;
                finish.send::<Halt>(&()).await?;
                drop(finish.recv::<Park0>().await?);
                drop(finish.recv::<Park1>().await?);
            };
            Ok::<(),ShellError>(())
        },hibana_runtime::host::deadline(Duration::from_secs(5))).await?;
                let effects = effects.take();
                assert_eq!(effects.len(), 2 * (2 + repetitions));
                let (mut active, mut maximum) = (0, 0);
                let mut tool_counts = [0usize; 2];
                for event in &effects {
                    let index = usize::from(event["tool"].as_u64().unwrap() as u8 - TOOL0);
                    if event["kind"] == "start" {
                        active += 1;
                        tool_counts[index] += 1;
                        maximum = maximum.max(active);
                    } else {
                        active -= 1;
                        tool_counts[index] -= 1;
                    }
                    assert!(active <= 2 && tool_counts.iter().all(|count| *count <= 1));
                }
                assert_eq!((active, maximum, tool_counts), (0, 2, [0, 0]));
                let third = effects
                    .iter()
                    .position(|event| event["kind"] == "start" && event["worker"] == 2)
                    .unwrap();
                let slow = effects
                    .iter()
                    .position(|event| event["kind"] == "finish" && event["tool"] != first_free)
                    .unwrap();
                assert!(
                    third < slow,
                    "returned tool must be reused while the other is busy"
                );
                let checkpoints = runtimes
                    .iter()
                    .map(|runtime| runtime.tap().checkpoint())
                    .collect::<Vec<_>>();
                let records = captured.each_ref().map(|records| records.take());

                let snapshot = hibana::runtime::tap::projected(&owner_program).collect::<Vec<_>>();
                let count = records.iter().map(Vec::len).sum::<usize>();
                let observed_order = observed_order.take();
                let report = serde_json::json!({"controlled_native_fixture":true,"first_free":first_free,"repetitions":repetitions,
                "epf":"not_run","captured_records":count,"effects":effects,"maximum_overlapping_effects":maximum,
                "session":session.raw(),"observed_producers":observed_order,"checkpoints":checkpoints,
                "records":records.iter().map(|producer|producer.iter().map(|event|<[u8;16]>::from(*event)).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "snapshot":snapshot.iter().map(|event|<[u8;16]>::from(*event)).collect::<Vec<_>>(),
                "frames":bus.transport.take_observations(),"full_11M_qualified":false,"domain_or_models_executed":false});
                if let Some(directory) = std::env::var_os("CHOREOFIX_POOL_OUTPUT") {
                    std::fs::write(
                        std::path::PathBuf::from(directory)
                            .join(format!("tool-{first_free}-visits-{repetitions}.json")),
                        serde_json::to_vec(&report)
                            .map_err(|error| ShellError::message(error.to_string()))?,
                    )?;
                }
                println!(
                    "{}",
                    serde_json::json!({"first_free":first_free,"repetitions":repetitions,
                "native":"passed","captured_records":count,"epf":"not_run","full_11M_qualified":false})
                );
                audits.push((
                    first_free,
                    repetitions,
                    session.raw(),
                    snapshot,
                    records,
                    checkpoints,
                    observed_order,
                    report,
                ));
            }
        }
    } else {
        // Recheck saved captures without repeating or changing the native run.
        for input in inputs {
            let report: serde_json::Value = serde_json::from_slice(&std::fs::read(input)?)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let first_free: u8 = serde_json::from_value(report["first_free"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let repetitions: usize = serde_json::from_value(report["repetitions"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let session: u32 = serde_json::from_value(report["session"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let raw: Vec<[u8; 16]> = serde_json::from_value(report["snapshot"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let snapshot = raw
                .into_iter()
                .map(hibana::runtime::tap::TapEvent::from)
                .collect::<Vec<_>>();
            let raw: [Vec<[u8; 16]>; 6] = serde_json::from_value(report["records"].clone())
                .map_err(|error| ShellError::message(error.to_string()))?;
            let records = raw.map(|producer| {
                producer
                    .into_iter()
                    .map(hibana::runtime::tap::TapEvent::from)
                    .collect::<Vec<_>>()
            });
            let checkpoints: Vec<Option<u32>> =
                serde_json::from_value(report["checkpoints"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
            let observed_order: Vec<usize> =
                serde_json::from_value(report["observed_producers"].clone())
                    .map_err(|error| ShellError::message(error.to_string()))?;
            audits.push((
                first_free,
                repetitions,
                session,
                snapshot,
                records,
                checkpoints,
                observed_order,
                report,
            ));
        }
    }
    // Save every completed native execution before starting synchronous audit.
    // An interrupted audit leaves raw evidence explicitly marked as unverified.
    for (
        first_free,
        repetitions,
        session,
        snapshot,
        records,
        checkpoints,
        observed_order,
        mut report,
    ) in audits
    {
        let captures = records
            .iter()
            .zip(&checkpoints)
            .map(|(records, checkpoint)| hibana_epf::Capture::new(records, *checkpoint).unwrap())
            .collect::<Vec<_>>();
        use hibana_epf::{
            Acceptance, AuditStorage, ExecutionStorage, Protocol, RecordId, RouteChoice, Unfold,
        };
        let mut source_space = vec![0; Protocol::workspace_len(&snapshot).unwrap()];
        let protocol = Protocol::new(&snapshot, &mut source_space).unwrap();
        let count = records.iter().map(Vec::len).sum::<usize>();
        let capacity = 2 * count + 1;
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
        let audit = protocol
            .audit(
                &captures,
                AuditStorage {
                    unfolding: &mut unfolding,
                    choices: &mut choices,
                    execution: ExecutionStorage {
                        session,
                        observed_producers: &observed_order,
                        order: &mut order,
                        occurrences: &mut occurrences,
                        acceptances: &mut acceptances,
                    },
                },
                &mut scratch,
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
        assert_eq!(audit.execution().order.len(), count);

        report["epf"] = serde_json::json!("passed");
        if let Some(directory) = std::env::var_os("CHOREOFIX_POOL_OUTPUT") {
            std::fs::write(
                std::path::PathBuf::from(directory)
                    .join(format!("tool-{first_free}-visits-{repetitions}.json")),
                serde_json::to_vec(&report)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            )?;
        }
        println!(
            "{}",
            serde_json::json!({"first_free":first_free,"repetitions":repetitions,
            "epf":"passed","captured_records":count,"full_11M_qualified":false})
        );
    }
    Ok(())
}
