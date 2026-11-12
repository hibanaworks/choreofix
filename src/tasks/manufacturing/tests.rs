use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use crate::{global::*, localside, manufacturing_data as imp};
use hibana::{
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef},
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};
#[cfg(miri)]
#[path = "../handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// This is an effect-data fixture, not a simulator permission gate or AI trial.
struct MachineData {
    events: Cell<Vec<Value>>,
    delay: Duration,
}
impl Domain for MachineData {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut events = self.events.take();
        let id = events.len();
        events.push(
            json!({"id":id,"boundary":"start","agent":agent,"tool":tool,"arguments":arguments}),
        );
        self.events.set(events);
        #[cfg(not(miri))]
        hibana_runtime::host::sleep(self.delay)
            .await
            .map_err(|e| e.to_string())?;
        #[cfg(miri)]
        {
            let started = std::time::Instant::now();
            std::future::poll_fn(|_| {
                if started.elapsed() >= self.delay {
                    std::task::Poll::Ready(())
                } else {
                    std::task::Poll::Pending
                }
            })
            .await;
        }
        let mut events = self.events.take();
        let data = if tool == "do_job" {
            let job = arguments["job_id"].as_str().ok_or("job id")?;
            let machine = if job.starts_with("lathe_") {
                "LATHE"
            } else {
                "MILL"
            };
            json!({"job_id":job,"agent":agent,"machine":machine,"status":"completed"})
        } else {
            json!({"agent":agent,"scope":"all_jobs","status":"collected",
                "jobs":events.iter().filter(|e| e["boundary"]=="finish" && e["tool"]=="do_job")
                    .map(|e| e["result"]["data"].clone()).collect::<Vec<_>>()})
        };
        let result = json!({"tool_name":tool,"success":true,"data":data});
        events.push(
            json!({"id":id,"boundary":"finish","agent":agent,"tool":tool,
            "arguments":arguments,"result":result}),
        );
        self.events.set(events);
        Ok(result)
    }
}

#[test]
fn every_machine_and_worker_projects_from_the_same_source() {
    let compiled = programs().observation();
    let programs = compiled["programs"].as_array().unwrap();
    assert_eq!(programs.len(), 15);
    assert!(
        programs
            .iter()
            .all(|p| !p["rows"].as_array().unwrap().is_empty())
    );
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn all_twenty_four_job_orders_use_actual_machine_endpoints_and_completed_reports()
-> ShellResult<()> {
    let p = programs();
    let mut tested = Vec::new();
    for one in 0..4 {
        for two in 0..4 {
            for three in 0..4 {
                for four in 0..4 {
                    let order = [one, two, three, four];
                    let mut sorted = order;
                    sorted.sort_unstable();
                    if sorted != [0, 1, 2, 3] {
                        continue;
                    }
                    let session = SessionId::new(SESSION);
                    let (bus, _io, _terminals) = Bus::open(
                        ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
                        [
                            (session, LATHE),
                            (session, MILL),
                            (session, LATHE_REQUEST_A),
                            (session, LATHE_REQUEST_B),
                            (session, MILL_REQUEST_A),
                            (session, MILL_REQUEST_B),
                        ],
                    );
                    let mut owner_slab = vec![0; 1024 * 1024];
                    let mut caller_slab = vec![0; 1024 * 1024];
                    let first_lathe = Cell::new(Value::Null);
                    let first_mill = Cell::new(Value::Null);
                    let arrival_lathe = Cell::new(Value::Null);
                    let arrival_mill = Cell::new(Value::Null);
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
                            &p.lathe_service,
                            ResolverRef::<FIRST_LATHE>::decision_state(
                                &first_lathe,
                                imp::first_request,
                            ),
                        )
                        .map_err(|e| ShellError::message(format!("{e:?}")))?;
                    owner
                        .set_resolver(
                            &p.mill_service,
                            ResolverRef::<FIRST_MILL>::decision_state(
                                &first_mill,
                                imp::first_request,
                            ),
                        )
                        .map_err(|e| ShellError::message(format!("{e:?}")))?;
                    owner
                        .set_resolver(
                            &p.lathe_service,
                            ResolverRef::<LATHE_ARRIVAL>::decision_state(
                                &arrival_lathe,
                                imp::arrival,
                            ),
                        )
                        .map_err(|e| ShellError::message(format!("{e:?}")))?;
                    owner
                        .set_resolver(
                            &p.mill_service,
                            ResolverRef::<MILL_ARRIVAL>::decision_state(
                                &arrival_mill,
                                imp::arrival,
                            ),
                        )
                        .map_err(|e| ShellError::message(format!("{e:?}")))?;
                    let mut lathe_service = owner.enter(session, &p.lathe_service)?;
                    let mut mill_service = owner.enter(session, &p.mill_service)?;
                    let mut report_service = owner.enter(session, &p.report_service)?;
                    let lathe = owner.enter(session, &p.lathe)?;
                    let mill = owner.enter(session, &p.mill)?;
                    let lathe_request_a = owner.enter(session, &p.lathe_request_a)?;
                    let lathe_request_b = owner.enter(session, &p.lathe_request_b)?;
                    let mill_request_a = owner.enter(session, &p.mill_request_a)?;
                    let mill_request_b = owner.enter(session, &p.mill_request_b)?;
                    let mut a = caller.enter(session, &p.a)?;
                    let mut b = caller.enter(session, &p.b)?;
                    let mut supervisor = caller.enter(session, &p.supervisor)?;
                    let [mut a, _a_private] = a
                        .par()
                        .map_err(|e| ShellError::message(format!("A root: {e:?}")))?;
                    let [mut b, _b_private] = b
                        .par()
                        .map_err(|e| ShellError::message(format!("B root: {e:?}")))?;
                    let [mut supervisor, _supervisor_private] = supervisor
                        .par()
                        .map_err(|e| ShellError::message(format!("supervisor root: {e:?}")))?;
                    let [
                        mut lathe_a_intent,
                        mut lathe_a_work,
                        mut mill_a_intent,
                        mut mill_a_work,
                    ] = a.par()?;
                    let [
                        mut lathe_b_intent,
                        mut lathe_b_work,
                        mut mill_b_intent,
                        mut mill_b_work,
                    ] = b.par()?;
                    let effects = MachineData {
                        events: Cell::new(Vec::new()),
                        delay: Duration::ZERO,
                    };
                    hibana_runtime::race(
                        async {
                            async {
                                localside::lathe::run(
                                    &mut lathe_service,
                                    (lathe, lathe_request_a, lathe_request_b),
                                    (&bus, &first_lathe, &arrival_lathe),
                                )
                                .await?;
                            };
                            async {
                                localside::mill::run(
                                    &mut mill_service,
                                    (mill, mill_request_a, mill_request_b),
                                    (&bus, &first_mill, &arrival_mill),
                                )
                                .await?;
                            };
                            async {
                                localside::reports::run(&mut report_service, &bus).await?;
                            };
                            async {
                                bus.fixture_peer::<5>(&effects, &["do_job", "collect_reports"])
                                    .await?;
                            };
                            async {
                                let (lathe, mill) = async {
                                    let [mut lathe, mut mill] = supervisor.par().map_err(|e| {
                                        ShellError::message(format!("supervisor reports: {e:?}"))
                                    })?;
                                    async {
                                        let branch = lathe.offer().await?;
                                        let results = match branch.label() {
                                            LatheDoneA::LOGICAL_LABEL => {
                                                let a = imp::decode(
                                                    branch.recv::<LatheDoneA>().await?,
                                                )?;
                                                let b =
                                                    imp::decode(lathe.recv::<LatheDoneB>().await?)?;
                                                [a, b]
                                            }
                                            LatheDoneB::LOGICAL_LABEL => {
                                                let b = imp::decode(
                                                    branch.recv::<LatheDoneB>().await?,
                                                )?;
                                                let a =
                                                    imp::decode(lathe.recv::<LatheDoneA>().await?)?;
                                                [a, b]
                                            }
                                            _ => {
                                                return Err(ShellError::message(
                                                    "lathe reports differ",
                                                ));
                                            }
                                        };
                                        Ok::<_, ShellError>(results)
                                    };
                                    async {
                                        let branch = mill.offer().await?;
                                        let results = match branch.label() {
                                            MillDoneA::LOGICAL_LABEL => {
                                                let a =
                                                    imp::decode(branch.recv::<MillDoneA>().await?)?;
                                                let b =
                                                    imp::decode(mill.recv::<MillDoneB>().await?)?;
                                                [a, b]
                                            }
                                            MillDoneB::LOGICAL_LABEL => {
                                                let b =
                                                    imp::decode(branch.recv::<MillDoneB>().await?)?;
                                                let a =
                                                    imp::decode(mill.recv::<MillDoneA>().await?)?;
                                                [a, b]
                                            }
                                            _ => {
                                                return Err(ShellError::message(
                                                    "mill reports differ",
                                                ));
                                            }
                                        };
                                        Ok::<_, ShellError>(results)
                                    };
                                }
                                .await?;
                                let receipts = json!([lathe[0], lathe[1], mill[0], mill[1]]);
                                let bytes = imp::encode(&receipts)?;
                                supervisor.send::<CompletedJobs>(&bytes.as_slice()).await?;
                                supervisor.send::<CollectReports>(&b"{}".as_slice()).await?;
                                let report = imp::decode(supervisor.recv::<Reports>().await?)?;
                                imp::report_result(&imp::completed_jobs(&receipts)?, &report)?;
                                Ok::<_, ShellError>(())
                            };
                            async {
                                // Public operation scripts enumerate inputs; they do not authorize
                                // production calls or tell the receivers which branch to choose.
                                for job in order {
                                    let arguments = imp::encode(&json!({"job_id":imp::JOBS[job]}))?;
                                    match job {
                                        0 => {
                                            let arguments =
                                                imp::JobArguments::<0>::from_bytes(&arguments)?;
                                            lathe_a_intent.send::<LatheIntentA>(&arguments).await?;
                                            let mut request =
                                                lathe_a_work.recv::<LatheRequestLoanA>().await?;
                                            request.recv::<LatheReady>().await?;
                                            lathe_a_work
                                                .send::<ReturnLatheRequestA>(request)
                                                .await?;
                                            let mut lathe =
                                                lathe_a_work.recv::<LatheLoan>().await?;
                                            lathe.send::<LatheJobA>(&arguments).await?;
                                            let result =
                                                imp::decode(lathe.recv::<LatheResult>().await?)?;
                                            let bytes = imp::encode(&result)?;
                                            async {
                                                let [mut returned, mut report] =
                                                    lathe_a_work.par()?;
                                                async {
                                                    returned.send::<ReturnLathe>(lathe).await?;
                                                };
                                                async {
                                                    report
                                                        .send::<LatheDoneA>(&bytes.as_slice())
                                                        .await?;
                                                };
                                                Ok::<(), ShellError>(())
                                            }
                                            .await?;
                                        }
                                        1 => {
                                            let arguments =
                                                imp::JobArguments::<1>::from_bytes(&arguments)?;
                                            lathe_b_intent.send::<LatheIntentB>(&arguments).await?;
                                            let mut request =
                                                lathe_b_work.recv::<LatheRequestLoanB>().await?;
                                            request.recv::<LatheReady>().await?;
                                            lathe_b_work
                                                .send::<ReturnLatheRequestB>(request)
                                                .await?;
                                            let mut lathe =
                                                lathe_b_work.recv::<LatheLoan>().await?;
                                            lathe.send::<LatheJobB>(&arguments).await?;
                                            let result =
                                                imp::decode(lathe.recv::<LatheResult>().await?)?;
                                            let bytes = imp::encode(&result)?;
                                            async {
                                                let [mut returned, mut report] =
                                                    lathe_b_work.par()?;
                                                async {
                                                    returned.send::<ReturnLathe>(lathe).await?;
                                                };
                                                async {
                                                    report
                                                        .send::<LatheDoneB>(&bytes.as_slice())
                                                        .await?;
                                                };
                                                Ok::<(), ShellError>(())
                                            }
                                            .await?;
                                        }
                                        2 => {
                                            let arguments =
                                                imp::JobArguments::<2>::from_bytes(&arguments)?;
                                            mill_a_intent.send::<MillIntentA>(&arguments).await?;
                                            let mut request =
                                                mill_a_work.recv::<MillRequestLoanA>().await?;
                                            request.recv::<MillReady>().await?;
                                            mill_a_work.send::<ReturnMillRequestA>(request).await?;
                                            let mut mill = mill_a_work.recv::<MillLoan>().await?;
                                            mill.send::<MillJobA>(&arguments).await?;
                                            let result =
                                                imp::decode(mill.recv::<MillResult>().await?)?;
                                            let bytes = imp::encode(&result)?;
                                            async {
                                                let [mut returned, mut report] =
                                                    mill_a_work.par()?;
                                                async {
                                                    returned.send::<ReturnMill>(mill).await?;
                                                };
                                                async {
                                                    report
                                                        .send::<MillDoneA>(&bytes.as_slice())
                                                        .await?;
                                                };
                                                Ok::<(), ShellError>(())
                                            }
                                            .await?;
                                        }
                                        3 => {
                                            let arguments =
                                                imp::JobArguments::<3>::from_bytes(&arguments)?;
                                            mill_b_intent.send::<MillIntentB>(&arguments).await?;
                                            let mut request =
                                                mill_b_work.recv::<MillRequestLoanB>().await?;
                                            request.recv::<MillReady>().await?;
                                            mill_b_work.send::<ReturnMillRequestB>(request).await?;
                                            let mut mill = mill_b_work.recv::<MillLoan>().await?;
                                            mill.send::<MillJobB>(&arguments).await?;
                                            let result =
                                                imp::decode(mill.recv::<MillResult>().await?)?;
                                            let bytes = imp::encode(&result)?;
                                            async {
                                                let [mut returned, mut report] =
                                                    mill_b_work.par()?;
                                                async {
                                                    returned.send::<ReturnMill>(mill).await?;
                                                };
                                                async {
                                                    report
                                                        .send::<MillDoneB>(&bytes.as_slice())
                                                        .await?;
                                                };
                                                Ok::<(), ShellError>(())
                                            }
                                            .await?;
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                Ok::<_, ShellError>(())
                            };
                            Ok::<_, ShellError>(())
                        },
                        deadline(Duration::from_secs(10)),
                    )
                    .await?;
                    let events = effects.events.take();
                    let actual = events
                        .iter()
                        .filter(|e| e["boundary"] == "start" && e["tool"] == "do_job")
                        .map(|e| e["arguments"]["job_id"].as_str().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(actual, order.map(|j| imp::JOBS[j]));
                    assert_eq!(
                        events.iter().filter(|e| e["boundary"] == "finish").count(),
                        5
                    );
                    tested.push(order);
                }
            }
        }
    }
    assert_eq!(tested.len(), 24);
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn disjoint_machine_effects_overlap_and_each_machine_keeps_one_actual_user() -> ShellResult<()>
{
    let p = programs();
    let session = SessionId::new(SESSION);
    let (bus, _io, _terminals) = Bus::open(
        ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
        [
            (session, LATHE),
            (session, MILL),
            (session, LATHE_REQUEST_A),
            (session, LATHE_REQUEST_B),
            (session, MILL_REQUEST_A),
            (session, MILL_REQUEST_B),
        ],
    );
    let first_lathe = Cell::new(Value::Null);
    let first_mill = Cell::new(Value::Null);
    let arrival_lathe = Cell::new(Value::Null);
    let arrival_mill = Cell::new(Value::Null);
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
            &p.lathe_service,
            ResolverRef::<FIRST_LATHE>::decision_state(&first_lathe, imp::first_request),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
    owner
        .set_resolver(
            &p.mill_service,
            ResolverRef::<FIRST_MILL>::decision_state(&first_mill, imp::first_request),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
    owner
        .set_resolver(
            &p.lathe_service,
            ResolverRef::<LATHE_ARRIVAL>::decision_state(&arrival_lathe, imp::arrival),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
    owner
        .set_resolver(
            &p.mill_service,
            ResolverRef::<MILL_ARRIVAL>::decision_state(&arrival_mill, imp::arrival),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let mut lathe_service = owner.enter(session, &p.lathe_service)?;
    let mut mill_service = owner.enter(session, &p.mill_service)?;
    let lathe = owner.enter(session, &p.lathe)?;
    let mill = owner.enter(session, &p.mill)?;
    let lathe_request_a = owner.enter(session, &p.lathe_request_a)?;
    let lathe_request_b = owner.enter(session, &p.lathe_request_b)?;
    let mill_request_a = owner.enter(session, &p.mill_request_a)?;
    let mill_request_b = owner.enter(session, &p.mill_request_b)?;
    let mut a = caller.enter(session, &p.a)?;
    let mut b = caller.enter(session, &p.b)?;
    // This test stops after the machine phase. The complete supervisor report
    // is exercised for every total order by the preceding end-to-end fixture.
    let _supervisor = caller.enter(session, &p.supervisor)?;
    let [mut a, _a_private] = a.par()?;
    let [mut b, _b_private] = b.par()?;
    let effects = MachineData {
        events: Cell::new(Vec::new()),
        delay: Duration::from_millis(20),
    };
    hibana_runtime::race(
        async {
            async {
                localside::lathe::run(
                    &mut lathe_service,
                    (lathe, lathe_request_a, lathe_request_b),
                    (&bus, &first_lathe, &arrival_lathe),
                )
                .await?;
            };
            async {
                localside::mill::run(
                    &mut mill_service,
                    (mill, mill_request_a, mill_request_b),
                    (&bus, &first_mill, &arrival_mill),
                )
                .await?;
            };
            async {
                bus.fixture_peer::<4>(&effects, &["do_job"]).await?;
            };
            async {
                let arguments = imp::JobArguments::<0>::from_bytes(br#"{"job_id":"lathe_job_a"}"#)?;
                a.send::<LatheIntentA>(&arguments).await?;
                let mut request = a.recv::<LatheRequestLoanA>().await?;
                request.recv::<LatheReady>().await?;
                a.send::<ReturnLatheRequestA>(request).await?;
                let mut lathe = a.recv::<LatheLoan>().await?;
                lathe.send::<LatheJobA>(&arguments).await?;
                let result = imp::decode(lathe.recv::<LatheResult>().await?)?;
                let bytes = imp::encode(&result)?;
                a.send::<ReturnLathe>(lathe).await?;
                a.send::<LatheDoneA>(&bytes.as_slice()).await?;

                let arguments = imp::JobArguments::<2>::from_bytes(br#"{"job_id":"mill_job_a"}"#)?;
                a.send::<MillIntentA>(&arguments).await?;
                let mut request = a.recv::<MillRequestLoanA>().await?;
                request.recv::<MillReady>().await?;
                a.send::<ReturnMillRequestA>(request).await?;
                let mut mill = a.recv::<MillLoan>().await?;
                mill.send::<MillJobA>(&arguments).await?;
                let result = imp::decode(mill.recv::<MillResult>().await?)?;
                let bytes = imp::encode(&result)?;
                a.send::<ReturnMill>(mill).await?;
                a.send::<MillDoneA>(&bytes.as_slice()).await?;
                Ok::<_, ShellError>(())
            };
            async {
                let arguments = imp::JobArguments::<3>::from_bytes(br#"{"job_id":"mill_job_b"}"#)?;
                b.send::<MillIntentB>(&arguments).await?;
                let mut request = b.recv::<MillRequestLoanB>().await?;
                request.recv::<MillReady>().await?;
                b.send::<ReturnMillRequestB>(request).await?;
                let mut mill = b.recv::<MillLoan>().await?;
                mill.send::<MillJobB>(&arguments).await?;
                let result = imp::decode(mill.recv::<MillResult>().await?)?;
                let bytes = imp::encode(&result)?;
                b.send::<ReturnMill>(mill).await?;
                b.send::<MillDoneB>(&bytes.as_slice()).await?;

                let arguments = imp::JobArguments::<1>::from_bytes(br#"{"job_id":"lathe_job_b"}"#)?;
                b.send::<LatheIntentB>(&arguments).await?;
                let mut request = b.recv::<LatheRequestLoanB>().await?;
                request.recv::<LatheReady>().await?;
                b.send::<ReturnLatheRequestB>(request).await?;
                let mut lathe = b.recv::<LatheLoan>().await?;
                lathe.send::<LatheJobB>(&arguments).await?;
                let result = imp::decode(lathe.recv::<LatheResult>().await?)?;
                let bytes = imp::encode(&result)?;
                b.send::<ReturnLathe>(lathe).await?;
                b.send::<LatheDoneB>(&bytes.as_slice()).await?;
                Ok::<_, ShellError>(())
            };
            Ok::<_, ShellError>(())
        },
        deadline(Duration::from_secs(10)),
    )
    .await?;
    let events = effects.events.take();
    let spans = events
        .iter()
        .enumerate()
        .filter(|(_, e)| e["boundary"] == "start")
        .map(|(start, e)| {
            let end = events
                .iter()
                .position(|f| f["id"] == e["id"] && f["boundary"] == "finish")
                .unwrap();
            (
                start,
                end,
                e["arguments"]["job_id"]
                    .as_str()
                    .unwrap()
                    .starts_with("lathe_"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 4);
    let mut overlaps = Vec::new();
    for (index, (start, end, machine)) in spans.iter().enumerate() {
        for (other_start, other_end, other_machine) in &spans[index + 1..] {
            if start < other_end && other_start < end {
                assert_ne!(
                    machine, other_machine,
                    "two actual users overlap on one machine"
                );
                overlaps.push((*start, *other_start));
            }
        }
    }
    assert!(
        overlaps.len() >= 2,
        "independent actual effects were serialized"
    );
    Ok(())
}

#[test]
fn job_wire_values_preserve_actual_bytes_and_reject_other_objects() {
    use hibana::runtime::wire::WireEncode;
    let bytes = br#"{ "job_id": "lathe_job_a" }"#;
    let request = imp::JobArguments::<0>::from_bytes(bytes).unwrap();
    let mut encoded = [0; 128];
    let count = request.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..count], bytes);
    assert!(imp::JobArguments::<1>::from_bytes(bytes).is_err());
    assert!(imp::JobArguments::<2>::from_bytes(bytes).is_err());
    assert!(
        imp::JobArguments::<0>::from_bytes(br#"{"job_id":"lathe_job_a","owner":"TECHNICIAN_A"}"#)
            .is_err()
    );
    assert!(imp::JobArguments::<0>::from_bytes(br#"{"job_id":0}"#).is_err());
}

#[test]
fn supervisor_receipts_reject_missing_duplicate_and_forged_actual_results() {
    let jobs = imp::JOBS
        .iter()
        .enumerate()
        .map(|(index, job)| {
            json!({"tool_name":"do_job","success":true,"data":{
            "job_id":job,"agent":if index.is_multiple_of(2) {"TECHNICIAN_A"}else{"TECHNICIAN_B"},
            "machine":if index<2 {"LATHE"}else{"MILL"},"status":"completed"}})
        })
        .collect::<Vec<_>>();
    let expected = imp::completed_jobs(&json!(jobs)).unwrap();
    let report = json!({"tool_name":"collect_reports","success":true,"data":{
        "agent":"SUPERVISOR","scope":"all_jobs","status":"collected","jobs":expected}});
    imp::report_result(&expected, &report).unwrap();
    assert!(imp::completed_jobs(&json!(&jobs[..3])).is_err());
    let mut duplicate = jobs.clone();
    duplicate[3] = duplicate[0].clone();
    assert!(imp::completed_jobs(&json!(duplicate)).is_err());
    let mut forged = jobs.clone();
    forged[0]["data"]["machine"] = json!("MILL");
    assert!(imp::completed_jobs(&json!(forged)).is_err());
    let mut incomplete = report.clone();
    incomplete["data"]["jobs"].as_array_mut().unwrap().pop();
    assert!(imp::report_result(&expected, &incomplete).is_err());
}
#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn a_busy_machine_keeps_its_rpc_while_the_other_technician_uses_the_free_machine()
-> ShellResult<()> {
    for retry_before_other_job in [false, true] {
        let p = programs();
        let session = SessionId::new(SESSION);
        let (bus, _io, _terminals) = Bus::open(
            ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
            [
                (session, LATHE),
                (session, MILL),
                (session, LATHE_REQUEST_A),
                (session, LATHE_REQUEST_B),
                (session, MILL_REQUEST_A),
                (session, MILL_REQUEST_B),
            ],
        );
        let first_lathe = Cell::new(Value::Null);
        let first_mill = Cell::new(Value::Null);
        let arrival_lathe = Cell::new(Value::Null);
        let arrival_mill = Cell::new(Value::Null);
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
                &p.lathe_service,
                ResolverRef::<FIRST_LATHE>::decision_state(&first_lathe, imp::first_request),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        owner
            .set_resolver(
                &p.mill_service,
                ResolverRef::<FIRST_MILL>::decision_state(&first_mill, imp::first_request),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        owner
            .set_resolver(
                &p.lathe_service,
                ResolverRef::<LATHE_ARRIVAL>::decision_state(&arrival_lathe, imp::arrival),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        owner
            .set_resolver(
                &p.mill_service,
                ResolverRef::<MILL_ARRIVAL>::decision_state(&arrival_mill, imp::arrival),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut lathe_service = owner.enter(session, &p.lathe_service)?;
        let mut mill_service = owner.enter(session, &p.mill_service)?;
        let lathe = owner.enter(session, &p.lathe)?;
        let mill = owner.enter(session, &p.mill)?;
        let lathe_request_a = owner.enter(session, &p.lathe_request_a)?;
        let lathe_request_b = owner.enter(session, &p.lathe_request_b)?;
        let mill_request_a = owner.enter(session, &p.mill_request_a)?;
        let mill_request_b = owner.enter(session, &p.mill_request_b)?;
        let mut a = caller.enter(session, &p.a)?;
        let mut b = caller.enter(session, &p.b)?;
        // This test stops after the machine phase. The complete supervisor report
        // is exercised for every total order by the preceding end-to-end fixture.
        let _supervisor = caller.enter(session, &p.supervisor)?;
        let [mut a, _a_private] = a.par()?;
        let [mut b, _b_private] = b.par()?;
        let effects = MachineData {
            events: Cell::new(Vec::new()),
            delay: Duration::from_millis(40),
        };
        hibana_runtime::race(
            async {
                async {
                    localside::lathe::run(
                        &mut lathe_service,
                        (lathe, lathe_request_a, lathe_request_b),
                        (&bus, &first_lathe, &arrival_lathe),
                    )
                    .await?;
                };
                async {
                    localside::mill::run(
                        &mut mill_service,
                        (mill, mill_request_a, mill_request_b),
                        (&bus, &first_mill, &arrival_mill),
                    )
                    .await?;
                };
                async {
                    bus.fixture_peer::<4>(&effects, &["do_job"]).await?;
                };
                async {
                    let arguments =
                        imp::JobArguments::<0>::from_bytes(br#"{"job_id":"lathe_job_a"}"#)?;
                    a.send::<LatheIntentA>(&arguments).await?;
                    let mut request = a.recv::<LatheRequestLoanA>().await?;
                    request.recv::<LatheReady>().await?;
                    a.send::<ReturnLatheRequestA>(request).await?;
                    let mut lathe = a.recv::<LatheLoan>().await?;
                    lathe.send::<LatheJobA>(&arguments).await?;
                    let result = imp::decode(lathe.recv::<LatheResult>().await?)?;
                    let bytes = imp::encode(&result)?;
                    a.send::<ReturnLathe>(lathe).await?;
                    a.send::<LatheDoneA>(&bytes.as_slice()).await?;

                    let arguments =
                        imp::JobArguments::<2>::from_bytes(br#"{"job_id":"mill_job_a"}"#)?;
                    a.send::<MillIntentA>(&arguments).await?;
                    let mut request = a.recv::<MillRequestLoanA>().await?;
                    loop {
                        let branch = request.offer().await?;
                        match branch.label() {
                            MillReady::LOGICAL_LABEL => {
                                branch.recv::<MillReady>().await?;
                                break;
                            }
                            MillBusy::LOGICAL_LABEL => {
                                branch.recv::<MillBusy>().await?;
                            }
                            MillRetryA::LOGICAL_LABEL => {
                                branch.send::<MillRetryA>(&arguments).await?;
                            }
                            _ => return Err(ShellError::message("unexpected mill availability")),
                        }
                    }
                    a.send::<ReturnMillRequestA>(request).await?;
                    let mut mill = a.recv::<MillLoan>().await?;
                    mill.send::<MillJobA>(&arguments).await?;
                    let result = imp::decode(mill.recv::<MillResult>().await?)?;
                    let bytes = imp::encode(&result)?;
                    a.send::<ReturnMill>(mill).await?;
                    a.send::<MillDoneA>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>(())
                };
                async {
                    // Fixture scheduling waits for actual I/O acceptance, not an app flag.
                    std::future::poll_fn(|cx| {
                        let events = effects.events.take();
                        let started = events.iter().any(|e| {
                            e["boundary"] == "start" && e["arguments"]["job_id"] == "lathe_job_a"
                        });
                        effects.events.set(events);
                        if started {
                            std::task::Poll::Ready(())
                        } else {
                            cx.waker().wake_by_ref();
                            std::task::Poll::Pending
                        }
                    })
                    .await;
                    let lathe_arguments =
                        imp::JobArguments::<1>::from_bytes(br#"{"job_id":"lathe_job_b"}"#)?;
                    b.send::<LatheIntentB>(&lathe_arguments).await?;
                    let mut lathe_request = b.recv::<LatheRequestLoanB>().await?;
                    lathe_request.recv::<LatheBusy>().await?;
                    if retry_before_other_job {
                        lathe_request.send::<LatheRetryB>(&lathe_arguments).await?;
                        lathe_request.recv::<LatheBusy>().await?;
                    }
                    // The blocked machine's real request Endpoint remains owned here.
                    let arguments =
                        imp::JobArguments::<3>::from_bytes(br#"{"job_id":"mill_job_b"}"#)?;
                    b.send::<MillIntentB>(&arguments).await?;
                    let mut request = b.recv::<MillRequestLoanB>().await?;
                    request.recv::<MillReady>().await?;
                    b.send::<ReturnMillRequestB>(request).await?;
                    let mut mill = b.recv::<MillLoan>().await?;
                    mill.send::<MillJobB>(&arguments).await?;
                    let result = imp::decode(mill.recv::<MillResult>().await?)?;
                    let bytes = imp::encode(&result)?;
                    b.send::<ReturnMill>(mill).await?;
                    b.send::<MillDoneB>(&bytes.as_slice()).await?;

                    loop {
                        let branch = lathe_request.offer().await?;
                        match branch.label() {
                            LatheReady::LOGICAL_LABEL => {
                                // Keep the already arrived Ready while a final retry crosses it.
                                drop(branch);
                                lathe_request.send::<LatheRetryB>(&lathe_arguments).await?;
                                lathe_request.recv::<LatheBusy>().await?;
                                lathe_request.recv::<LatheReady>().await?;
                                break;
                            }
                            LatheBusy::LOGICAL_LABEL => {
                                branch.recv::<LatheBusy>().await?;
                            }
                            LatheRetryB::LOGICAL_LABEL => {
                                branch.send::<LatheRetryB>(&lathe_arguments).await?;
                            }
                            _ => return Err(ShellError::message("unexpected lathe availability")),
                        }
                    }
                    // A valid late retry can cross the already queued Ready.
                    lathe_request.send::<LatheRetryB>(&lathe_arguments).await?;
                    lathe_request.recv::<LatheBusy>().await?;
                    b.send::<ReturnLatheRequestB>(lathe_request).await?;
                    let mut lathe = b.recv::<LatheLoan>().await?;
                    lathe.send::<LatheJobB>(&lathe_arguments).await?;
                    let result = imp::decode(lathe.recv::<LatheResult>().await?)?;
                    let bytes = imp::encode(&result)?;
                    b.send::<ReturnLathe>(lathe).await?;
                    b.send::<LatheDoneB>(&bytes.as_slice()).await?;
                    Ok::<_, ShellError>(())
                };
                Ok::<_, ShellError>(())
            },
            deadline(Duration::from_secs(10)),
        )
        .await?;
        let events = effects.events.take();
        let started = events
            .iter()
            .filter(|e| e["boundary"] == "start")
            .collect::<Vec<_>>();
        assert_eq!(
            started.len(),
            4,
            "a cancelled borrowed wait must never reissue an effect"
        );
        for job in imp::JOBS {
            assert_eq!(
                started
                    .iter()
                    .filter(|e| e["arguments"]["job_id"] == job)
                    .count(),
                1
            );
        }
        let index = |job: &str, boundary: &str| {
            events
                .iter()
                .position(|e| e["arguments"]["job_id"] == job && e["boundary"] == boundary)
                .unwrap()
        };
        assert!(
            index("mill_job_b", "start") < index("lathe_job_a", "finish"),
            "Busy prevented choosing the idle machine"
        );
        assert!(index("lathe_job_b", "start") > index("lathe_job_a", "finish"));
        assert!(index("mill_job_a", "start") > index("mill_job_b", "finish"));
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn physical_receiver_rejects_untrusted_job_bytes_before_any_effect() -> ShellResult<()> {
    use hibana::{
        g,
        runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    };
    // A peer can claim a schema on the wire. Only the production receiver's
    // actual payload validator can establish that these bytes describe its job.
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
        const SCHEMA_ID: u32 = <imp::JobArguments<0> as WirePayload>::SCHEMA_ID;
        type Decoded<'a> = &'a [u8];
        fn validate_payload(_: Payload<'_>) -> Result<(), CodecError> {
            Ok(())
        }
        fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
            input.as_bytes()
        }
    }
    type UntrustedJob = g::Msg<7, Untrusted>;
    for bytes in [
        br#"{"job_id":"mill_job_a"}"#.as_slice(),
        br#"{"job_id":"lathe_job_b"}"#.as_slice(),
        br#"{"job_id":"lathe_job_a","owner":"TECHNICIAN_A"}"#.as_slice(),
        b"{".as_slice(),
    ] {
        let p = programs();
        let session = SessionId::new(SESSION);
        let (bus, _io, _terminals) = Bus::open(
            ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
            [
                (session, LATHE),
                (session, LATHE_REQUEST_A),
                (session, LATHE_REQUEST_B),
            ],
        );
        let first = Cell::new(Value::Null);
        let arrival = Cell::new(Value::Null);
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
                &p.lathe_service,
                ResolverRef::<FIRST_LATHE>::decision_state(&first, imp::first_request),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        owner
            .set_resolver(
                &p.lathe_service,
                ResolverRef::<LATHE_ARRIVAL>::decision_state(&arrival, imp::arrival),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut service = owner.enter(session, &p.lathe_service)?;
        let lathe = owner.enter(session, &p.lathe)?;
        let query_a = owner.enter(session, &p.lathe_request_a)?;
        let query_b = owner.enter(session, &p.lathe_request_b)?;
        let mut a = caller.enter(session, &p.a)?;
        let [mut a, _private] = a.par()?;
        let effects = MachineData {
            events: Cell::new(Vec::new()),
            delay: Duration::ZERO,
        };
        hibana_runtime::race(
            async {
                let error = localside::lathe::run(
                    &mut service,
                    (lathe, query_a, query_b),
                    (&bus, &first, &arrival),
                )
                .await;
                assert!(matches!(error, Err(ShellError::Endpoint(_))));
                Ok::<(), ShellError>(())
            },
            async {
                async {
                    let valid = imp::JobArguments::<0>::from_bytes(br#"{"job_id":"lathe_job_a"}"#)?;
                    a.send::<LatheIntentA>(&valid).await?;
                    let mut query = a.recv::<LatheRequestLoanA>().await?;
                    query.recv::<LatheReady>().await?;
                    a.send::<ReturnLatheRequestA>(query).await?;
                    let mut actual_resource = a.recv::<LatheLoan>().await?;
                    actual_resource
                        .send::<UntrustedJob>(&Untrusted(bytes))
                        .await?;
                    core::future::pending::<ShellResult<()>>().await
                };
                async {
                    bus.fixture_peer::<1>(&effects, &["do_job"]).await?;
                };
                Ok::<(), ShellError>(())
            },
        )
        .await?;
        assert!(
            effects.events.take().is_empty(),
            "untrusted wire bytes reached the machine effect"
        );
    }
    Ok(())
}
