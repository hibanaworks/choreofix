//! Actual tool Endpoints and production receiver, not full 11M qualification.
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::transport;
}
#[path = "../../../src/tasks/manufacturing_rework/mod.rs"]
pub mod manufacturing_rework;
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use manufacturing_rework::{global::*, imp as data, localside};
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

/// Completed external material records and controlled quality outcomes. This
/// fixture has no Endpoint permission, station scheduler or application cursor.
struct QualityEffects {
    job: usize,
    rejected_inspections: usize,
    events: Cell<Vec<Value>>,
}
impl Domain for QualityEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let target = data::JOBS[self.job];
        let worker = data::AGENTS[self.job % data::AGENTS.len()];
        let station =
            ["STATION_ALPHA", "STATION_BETA", "STATION_GAMMA"][[0, 1, 2, 1, 2, 0][self.job]];
        if arguments["job_id"] != target
            || agent
                != match tool {
                    "do_job" => worker,
                    "inspect_quality" => "INSPECTOR",
                    "dispatch_rework" => "DISPATCHER",
                    _ => return Err("unknown quality-fixture operation".into()),
                }
        {
            return Err("wrong quality-fixture caller or job".into());
        }
        let events = self.events.take();
        let result = (|| -> Result<Value, String> {
            let count = |tool: &str| events.iter().filter(|row| row["tool"] == tool).count();
            let data = match tool {
                "do_job" => {
                    let attempt = count(tool) + 1;
                    let order = events
                        .iter()
                        .rev()
                        .find(|row| row["tool"] == "dispatch_rework");
                    json!({"status":"completed","rework_order":order.map(|row|row["result"]["data"]["order"].clone()),
                        "part":{"artifact_id":format!("part/{target}/{attempt}"),"job_id":target,
                        "worker":worker,"station":station,"attempt":attempt}})
                }
                "inspect_quality" => {
                    let part = &events
                        .iter()
                        .rev()
                        .find(|row| row["tool"] == "do_job")
                        .ok_or("inspection without actual fixture material")?["result"]["data"]["part"];
                    json!({"job_id":target,"input_part":part,"receipt":format!("inspection/{}",count(tool)+1),
                        "inspector":"INSPECTOR","status":if count(tool)<self.rejected_inspections{"failed_inspection"}else{"inspected"}})
                }
                "dispatch_rework" => {
                    let measured = &events
                        .iter()
                        .rev()
                        .find(|row| row["tool"] == "inspect_quality")
                        .ok_or("dispatch without actual fixture inspection")?["result"];
                    if measured["success"] != false {
                        return Err("dispatch of an approved fixture part".into());
                    }
                    json!({"job_id":target,"worker":worker,"input_part":measured["data"]["input_part"],
                        "inspection":measured["data"]["receipt"],"order":format!("rework/{}",count(tool)+1),
                        "status":"rework_dispatched","reason":arguments.get("reason").cloned().unwrap_or(json!(""))})
                }
                _ => unreachable!(),
            };
            Ok(
                json!({"tool_name":tool,"success":tool!="inspect_quality" || count(tool)>=self.rejected_inspections,"data":data}),
            )
        })();
        self.events.set(events);
        let result = result?;
        let mut events = self.events.take();
        events.push(json!({"agent":agent,"tool":tool,"arguments":arguments,"result":result}));
        self.events.set(events);
        Ok(result)
    }
}

/// Complete native fixture for one job continuation, not the six-job station/
/// tool allocator or a Wasm/model implementation of 11M.
#[hibana_runtime::scope]
async fn quality_case<const JOB: u8, const WORKER: u8, const CAP: u8>(
    rejected: usize,
) -> ShellResult<Value> {
    use hibana::g::Message;
    use hibana::{g, runtime::program::project};
    let work = g::seq(
        g::send::<CAP, DEPOT, JobIntent<JOB>>(),
        g::send::<DEPOT, CAP, JobResult>(),
    );
    let source = job_quality::<JOB, WORKER, CAP>(work, work);
    let worker_program = project::<WORKER>(&source);
    let job_program = project::<CAP>(&source);
    let inspector_program = project::<INSPECTOR>(&source);
    let dispatcher_program = project::<DISPATCHER>(&source);
    let depot_program = project::<DEPOT>(&source);
    let session = SessionId::new(SESSION + 1);
    let (bus, _, _) = Bus::open(data::AGENTS, [(session, CAP)]);
    let verdict = Cell::new(Value::Null);
    let records: [Cell<Vec<hibana::runtime::tap::TapEvent>>; 4] =
        core::array::from_fn(|_| Cell::new(Vec::new()));
    let observed_producers = Cell::new(Vec::new());
    let observers = [0usize, 1, 2, 3].map(|producer| {
        let records = &records;
        let observed_producers = &observed_producers;
        move |event: hibana::runtime::tap::TapEvent| {
            let mut order = observed_producers.take();
            order.push(producer);
            observed_producers.set(order);
            let mut capture = records[producer].take();
            capture.push(event);
            records[producer].set(capture);
        }
    });
    let mut worker_slab = vec![0; 1024 * 1024];
    let mut inspector_slab = vec![0; 1024 * 1024];
    let mut dispatcher_slab = vec![0; 1024 * 1024];
    let mut depot_slab = vec![0; 1024 * 1024];
    let mut worker_storage = SessionKitStorage::uninit();
    let mut inspector_storage = SessionKitStorage::uninit();
    let mut dispatcher_storage = SessionKitStorage::uninit();
    let mut depot_storage = SessionKitStorage::uninit();
    let worker_runtime = worker_storage
        .init()
        .rendezvous(&mut worker_slab, &bus.transport)?;
    let inspector_runtime = inspector_storage
        .init()
        .rendezvous(&mut inspector_slab, &bus.transport)?;
    let dispatcher_runtime = dispatcher_storage
        .init()
        .rendezvous(&mut dispatcher_slab, &bus.transport)?;
    let depot_runtime = depot_storage
        .init()
        .rendezvous(&mut depot_slab, &bus.transport)?;
    worker_runtime.observe(&observers[0]);
    inspector_runtime.observe(&observers[1]);
    dispatcher_runtime.observe(&observers[2]);
    depot_runtime.observe(&observers[3]);
    inspector_runtime
        .set_resolver(
            &inspector_program,
            ResolverRef::<QUALITY_FIRST>::decision_state(&verdict, data::initial_quality),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let mut worker = worker_runtime.enter(session, &worker_program)?;
    let mut job = worker_runtime.enter(session, &job_program)?;
    let mut inspector = inspector_runtime.enter(session, &inspector_program)?;
    let mut dispatcher = dispatcher_runtime.enter(session, &dispatcher_program)?;
    let mut depot = depot_runtime.enter(session, &depot_program)?;
    let arguments_bytes = data::encode(&json!({"job_id":data::JOBS[usize::from(JOB)]}))?;
    let arguments = data::JobArguments::<JOB>::from_bytes(&arguments_bytes)?;
    let admitted = data::admitted(&arguments)?;
    let effects = QualityEffects {
        job: usize::from(JOB),
        rejected_inspections: rejected,
        events: Cell::new(Vec::new()),
    };
    let ((), (), (), mut retired) = hibana_runtime::race(async {
        async {
            job.send::<JobIntent<JOB>>(&arguments).await?;
            let result = data::decode(job.recv::<JobResult>().await?)?;
            data::completed(&admitted, &result, None)?;
            let bytes = data::encode(&result["data"]["part"])?;
            job.send::<PartReady>(&bytes.as_slice()).await?;
            worker.send::<JobLoan<CAP>>(job).await?;
            loop {
                let branch = worker.offer().await?;
                match branch.label() {
                    label if label == ApprovalForWorker::<JOB>::LOGICAL_LABEL => {
                        branch.recv::<ApprovalForWorker<JOB>>().await?;
                        break;
                    }
                    OrderForWorker::LOGICAL_LABEL => {
                        let previous = data::decode(branch.recv::<OrderForWorker>().await?)?;
                        let mut job = worker.recv::<JobLoan<CAP>>().await?;
                        job.send::<JobIntent<JOB>>(&arguments).await?;
                        let result = data::decode(job.recv::<JobResult>().await?)?;
                        data::completed(&admitted, &result, Some((&previous["inspection"],&previous["arguments"],&previous["result"])))?;
                        let bytes = data::encode(&result["data"]["part"])?;
                        job.send::<PartReady>(&bytes.as_slice()).await?;
                        worker.send::<JobLoan<CAP>>(job).await?;
                    }
                    _ => return Err(ShellError::message("unexpected worker quality continuation")),
                }
            }
        };
        async {
            loop {
                let part = data::decode(inspector.recv::<PartReady>().await?)?;
                let mut job = inspector.recv::<JobLoan<CAP>>().await?;
                job.send::<InspectRequest<JOB>>(&arguments).await?;
                let received = inspector.recv::<InspectRequest<JOB>>().await?.value();
                data::inspection_input(&part, &received)?;
                let result = effects.execute("INSPECTOR", "inspect_quality", received.clone()).await.map_err(ShellError::message)?;
                data::inspected(&part, &received, &result)?;
                let bytes = data::encode(&result)?;
                inspector.send::<InspectionResult>(&bytes.as_slice()).await?;
                let actual = data::decode(job.recv::<InspectionResult>().await?)?;
                assert_eq!(actual, result);
                verdict.set(actual.clone());
                if actual["success"] == true {
                    job.send::<Approved<JOB>>(&data::InspectionReply::<JOB,true>::from_bytes(&bytes)?).await?;
                    inspector.send::<JobLoan<CAP>>(job).await?;
                    break;
                }
                job.send::<Failed<JOB>>(&data::InspectionReply::<JOB,false>::from_bytes(&bytes)?).await?;
                inspector.send::<JobLoan<CAP>>(job).await?;
            }
        };
        async {
            loop {
                let branch = dispatcher.offer().await?;
                match branch.label() {
                    label if label == Approved::<JOB>::LOGICAL_LABEL => {
                        let bytes = branch.recv::<Approved<JOB>>().await?.to_vec();
                        let mut job = dispatcher.recv::<JobLoan<CAP>>().await?;
                        async {
                            let [mut to_worker,mut to_depot] = job.par()?;
                            async { to_worker.send::<ApprovalForWorker<JOB>>(&data::InspectionReply::<JOB,true>::from_bytes(&bytes)?).await?; };
                            async { to_depot.send::<Approved<JOB>>(&data::InspectionReply::<JOB,true>::from_bytes(&bytes)?).await?; };
                            Ok::<(),ShellError>(())
                        }.await?;
                        dispatcher.send::<JobLoan<CAP>>(job).await?;
                        break;
                    }
                    label if label == Failed::<JOB>::LOGICAL_LABEL => {
                        let inspection = data::decode(branch.recv::<Failed<JOB>>().await?)?;
                        let mut job = dispatcher.recv::<JobLoan<CAP>>().await?;
                        let bytes = data::encode(&json!({"job_id":data::JOBS[usize::from(JOB)],"reason":"repair the measured defect"}))?;
                        let dispatch = data::DispatchArguments::<JOB>::from_bytes(&bytes)?;
                        job.send::<DispatchRequest<JOB>>(&dispatch).await?;
                        let received = dispatcher.recv::<DispatchRequest<JOB>>().await?.value();
                        data::rework_input(&inspection, &received)?;
                        let result = effects.execute("DISPATCHER", "dispatch_rework", received.clone()).await.map_err(ShellError::message)?;
                        data::dispatched(&inspection, &received, &result)?;
                        let bytes = data::encode(&result)?;
                        dispatcher.send::<Dispatched>(&bytes.as_slice()).await?;
                        let actual = data::decode(job.recv::<Dispatched>().await?)?;
                        assert_eq!(actual, result);
                        let bytes = data::encode(&json!({"inspection":inspection,"arguments":received,"result":actual}))?;
                        async {
                            let [mut to_depot,mut to_worker] = job.par()?;
                            async { to_depot.send::<OrderEvidence>(&bytes.as_slice()).await?; };
                            async { to_worker.send::<OrderForWorker>(&bytes.as_slice()).await?; };
                            Ok::<(),ShellError>(())
                        }.await?;
                        dispatcher.send::<JobLoan<CAP>>(job).await?;
                    }
                    _ => return Err(ShellError::message("unexpected dispatch continuation")),
                }
            }
        };
        async {
            let received = depot.recv::<JobIntent<JOB>>().await?.value();
            let result = effects.execute(data::AGENTS[usize::from(JOB)%3], "do_job", received).await.map_err(ShellError::message)?;
            data::completed(&admitted, &result, None)?;
            let mut part = result["data"]["part"].clone();
            let bytes = data::encode(&result)?;
            depot.send::<JobResult>(&bytes.as_slice()).await?;
            loop {
                let branch = depot.offer().await?;
                match branch.label() {
                    label if label == Approved::<JOB>::LOGICAL_LABEL => {
                        let approved = data::decode(branch.recv::<Approved<JOB>>().await?)?;
                        data::inspected(&part, &arguments.value(), &approved)?;
                        let retired = depot.recv::<JobLoan<CAP>>().await?;
                        return Ok::<_, ShellError>(retired);
                    }
                    OrderEvidence::LOGICAL_LABEL => {
                        let previous = data::decode(branch.recv::<OrderEvidence>().await?)?;
                        data::inspected(&part, &arguments.value(), &previous["inspection"])?;
                        data::dispatched(&previous["inspection"],&previous["arguments"],&previous["result"])?;
                        let received = depot.recv::<JobIntent<JOB>>().await?.value();
                        let result = effects.execute(data::AGENTS[usize::from(JOB)%3], "do_job", received).await.map_err(ShellError::message)?;
                        data::completed(&admitted,&result,Some((&previous["inspection"],&previous["arguments"],&previous["result"])))?;
                        part=result["data"]["part"].clone();
                        let bytes=data::encode(&result)?;
                        depot.send::<JobResult>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected physical job continuation")),
                }
            }
        };
    },deadline(Duration::from_secs(if cfg!(miri){600}else{5}))).await?;
    let checkpoints = [
        worker_runtime.tap().checkpoint(),
        inspector_runtime.tap().checkpoint(),
        dispatcher_runtime.tap().checkpoint(),
        depot_runtime.tap().checkpoint(),
    ];
    let captured = records.each_ref().map(|records| records.take());
    let observed_producers = observed_producers.take();
    let captures: [hibana_epf::Capture; 4] = core::array::from_fn(|producer| {
        hibana_epf::Capture::new(&captured[producer], checkpoints[producer])
            .expect("complete quality capture")
    });
    let resolver_calls = captured
        .iter()
        .flatten()
        .filter(|event| {
            event.id() == hibana::runtime::tap::RESOLVER_AUDIT
                && (event.arg1() & 0xffff) == u32::from(QUALITY_FIRST)
        })
        .count();
    assert_eq!(
        resolver_calls, 1,
        "an inner rework must not rerun the outer decision"
    );
    let snapshot: Vec<_> = hibana::runtime::tap::projected(&depot_program).collect();
    #[cfg(not(miri))]
    let dynamic = {
        use hibana_epf::{
            Acceptance, AuditStorage, ExecutionStorage, Protocol, RecordId, RouteChoice, Unfold,
        };
        let mut source_storage = vec![0; Protocol::workspace_len(&snapshot).unwrap()];
        let protocol = Protocol::new(&snapshot, &mut source_storage).unwrap();
        let count = captured.iter().map(Vec::len).sum::<usize>();
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
            .map_err(|error| ShellError::message(format!("quality source execution: {error:?}")))?;
        json!({"status":"passed","captured_records":audit.execution().order.len(),
            "application_effects_certified":false,"scope":"complete native job-continuation source execution; fixture effects are checked separately"})
    };
    #[cfg(miri)]
    let dynamic = {
        let _ = &captures;
        json!({"status":"not_repeated_under_miri","scope":"Miri checks real Endpoint and data operations; native checks the same source execution"})
    };
    // The terminal probe runs after every legitimate participant has finished.
    // An undeclared operation on this retired actor/view may end the closed run;
    // it must not remain Pending or produce another work frame/effect.
    let mut frames = bus.transport.take_observations();
    let denied = hibana_runtime::race(
        async {
            retired
                .send::<JobIntent<JOB>>(&arguments)
                .await
                .map_err(ShellError::from)
        },
        deadline(Duration::from_secs(1)),
    )
    .await;
    let rejection = match denied {
        Err(ShellError::Endpoint(error)) => format!("{error:?}"),
        other => {
            return Err(ShellError::message(format!(
                "retired work must reject promptly: {other:?}"
            )));
        }
    };
    let after = bus.transport.take_observations();
    assert!(
        !after.iter().any(|row| row["action"] == "send"),
        "denied work emitted a frame"
    );
    frames.extend(after);
    let terminal_probe_checkpoints = [
        worker_runtime.tap().checkpoint(),
        inspector_runtime.tap().checkpoint(),
        dispatcher_runtime.tap().checkpoint(),
        depot_runtime.tap().checkpoint(),
    ];
    let terminal_probe_records = records.each_ref().map(|records| records.take());
    let events = effects.events.take();
    assert_eq!(events.len(), 3 * rejected + 2);
    frames.extend(bus.transport.take_observations());
    let outcome = json!({"job":data::JOBS[usize::from(JOB)],"rejected_inspections":rejected,
        "events":events,"frames":frames,"retired_request_rejected":true,"retired_request_error":rejection,"controlled_native_fixture":true,
        "full_11M_task":false,"model_calls":0,"dynamic_protocol":dynamic,"resolver_calls":resolver_calls,
        "session":session.raw(),"checkpoints":checkpoints,"observed_producers":observed_producers,
        "capture_scope":"complete producer prefixes through legitimate participants; the later rejected terminal probe is retained separately",
        "terminal_probe_checkpoints":terminal_probe_checkpoints,
        "terminal_probe_records":terminal_probe_records.iter().map(|producer|producer.iter().map(|event|<[u8;16]>::from(*event)).collect::<Vec<_>>()).collect::<Vec<_>>(),
        "protocol":snapshot.iter().map(|event|<[u8;16]>::from(*event)).collect::<Vec<_>>(),
        "records":captured.iter().map(|producer|producer.iter().map(|event|<[u8;16]>::from(*event)).collect::<Vec<_>>()).collect::<Vec<_>>()});
    if let Some(directory) = std::env::var_os("CHOREOFIX_QUALITY_CASE_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory)?;
        std::fs::write(
            directory.join(format!("job{JOB}-reject{rejected}.json")),
            data::encode(&outcome)?,
        )?;
    }
    Ok(outcome)
}

#[hibana_runtime::main]
#[test]
#[cfg(not(miri))]
async fn all_six_quality_continuations_preserve_actual_rework_and_close_on_approval()
-> ShellResult<()> {
    for rejected in 0..=3 {
        quality_case::<0, WORKER_A, 9>(rejected).await?;
        quality_case::<1, WORKER_B, 10>(rejected).await?;
        quality_case::<2, WORKER_C, 11>(rejected).await?;
        quality_case::<3, WORKER_A, 12>(rejected).await?;
        quality_case::<4, WORKER_B, 13>(rejected).await?;
        quality_case::<5, WORKER_C, 14>(rejected).await?;
    }
    Ok(())
}

#[cfg(miri)]
#[hibana_runtime::main(clock::Clock)]
#[test]
async fn quality_continuation_moves_between_runtimes_and_rejects_retired_work() -> ShellResult<()> {
    quality_case::<0, WORKER_A, 9>(2).await?;
    Ok(())
}

/// Controlled I/O data and append-only measurements; it grants no operations.
struct ToolEffects {
    events: Cell<Vec<Value>>,
    delays: [Duration; 3],
}
impl Domain for ToolEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let worker = data::AGENTS
            .iter()
            .position(|name| *name == agent)
            .ok_or("unknown fixture worker")?;
        if tool != "do_job" || arguments != json!({"job_id":data::JOBS[worker]}) {
            return Err("fixture received wrong effect arguments".to_owned());
        }
        let mut events = self.events.take();
        events.push(json!({"boundary":"start", "worker":worker, "arguments":arguments}));
        self.events.set(events);
        hibana_runtime::host::sleep(self.delays[worker])
            .await
            .map_err(|error| error.to_string())?;
        let station = ["STATION_ALPHA", "STATION_BETA", "STATION_GAMMA"][worker];
        let result = json!({"tool_name":tool,"success":true,
            "data":{"status":"completed","rework_order":null,"part":{
                "artifact_id":format!("part/{}/1",data::JOBS[worker]),
                "job_id":arguments["job_id"],"worker":agent,
                "station":station,
                "attempt":1}}});
        let mut events = self.events.take();
        events.push(json!({"boundary":"finish","worker":worker,"result":result}));
        self.events.set(events);
        Ok(result)
    }
}

#[hibana_runtime::main]
#[test]
#[cfg(not(miri))]
async fn all_arrival_orders_preserve_two_tool_capacity_and_streaming() -> ShellResult<()> {
    let programs = programs();
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    // Delays are a native scheduling fixture, never inserted in model trials.
    let profiles = [
        ([0, 2, 4], [30, 100, 10], 2),
        ([0, 2, 4], [100, 30, 10], 2),
        ([0, 30, 60], [5, 5, 5], 1),
        ([0, 30, 32], [5, 60, 30], 2),
        ([0, 2, 80], [20, 40, 10], 2),
    ];
    let mut checked = 0;
    for order in orders {
        for (profile, (arrivals, durations, expected_maximum)) in profiles.iter().enumerate() {
            let session = SessionId::new(SESSION);
            let (bus, _, _) = Bus::open(data::AGENTS, [(session, TOOL_0), (session, TOOL_1)]);
            let inputs = [const { Cell::new(Value::Null) }; 5];
            let next = (&inputs[0], &inputs[1]);
            let mut host_slab = vec![0; 1024 * 1024];
            let mut caller_slab = vec![0; 1024 * 1024];
            let mut host_storage = SessionKitStorage::uninit();
            let mut caller_storage = SessionKitStorage::uninit();
            let host = host_storage
                .init()
                .rendezvous(&mut host_slab, &bus.transport)?;
            let caller = caller_storage
                .init()
                .rendezvous(&mut caller_slab, &bus.transport)?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<FIRST_A>::decision_state(&inputs[0], data::worker_a),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<FIRST_B>::decision_state(&inputs[0], data::worker_b),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<NEXT>::decision_state(&next, data::next),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<ARRIVAL_0>::decision_state(&inputs[2], data::arrival),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<ARRIVAL_1>::decision_state(&inputs[3], data::arrival),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            host.set_resolver(
                &programs.depot,
                ResolverRef::<FINISHED_FIRST>::decision_state(&inputs[4], data::finished),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
            let mut depot = host.enter(session, &programs.depot)?;
            let tool0 = host.enter(session, &programs.tool_0)?;
            let tool1 = host.enter(session, &programs.tool_1)?;
            let mut a = caller.enter(session, &programs.a)?;
            let mut b = caller.enter(session, &programs.b)?;
            let mut c = caller.enter(session, &programs.c)?;
            let delays: [Duration; 3] = std::array::from_fn(|worker| {
                Duration::from_millis(durations[order.iter().position(|id| *id == worker).unwrap()])
            });
            let effects = ToolEffects {
                events: Cell::new(Vec::new()),
                delays,
            };
            let delays: [Duration; 3] = std::array::from_fn(|worker| {
                Duration::from_millis(arrivals[order.iter().position(|id| *id == worker).unwrap()])
            });
            let a_bytes = data::encode(&json!({"job_id":data::JOBS[0]}))?;
            let b_bytes = data::encode(&json!({"job_id":data::JOBS[1]}))?;
            let c_bytes = data::encode(&json!({"job_id":data::JOBS[2]}))?;
            let execution = hibana_runtime::race(
                async {
                    async {
                        localside::tool_depot::run(&mut depot, (tool0, tool1), (&bus, &inputs))
                            .await
                            .map_err(|error| ShellError::message(format!("depot: {error}")))?;
                    };
                    async {
                        bus.fixture_peer::<3>(&effects, &["do_job"]).await?;
                    };
                    async {
                        // Release the next fixture input after an actual effect
                        // starts; timer ordering alone does not order ready peers.
                        if let Some(rank) = order
                            .iter()
                            .position(|id| *id == 0)
                            .filter(|rank| *rank > 0)
                        {
                            loop {
                                let events = effects.events.take();
                                let started = events.iter().any(|event| {
                                    event["boundary"] == "start"
                                        && event["worker"] == order[rank - 1]
                                });
                                effects.events.set(events);
                                if started {
                                    break;
                                }
                                hibana_runtime::host::sleep(Duration::from_millis(1)).await?;
                            }
                        }
                        hibana_runtime::host::sleep(delays[0]).await?;
                        localside::worker::run(&mut a, &a_bytes)
                            .await
                            .map_err(|error| ShellError::message(format!("worker A: {error}")))?;
                    };
                    async {
                        if let Some(rank) = order
                            .iter()
                            .position(|id| *id == 1)
                            .filter(|rank| *rank > 0)
                        {
                            loop {
                                let events = effects.events.take();
                                let started = events.iter().any(|event| {
                                    event["boundary"] == "start"
                                        && event["worker"] == order[rank - 1]
                                });
                                effects.events.set(events);
                                if started {
                                    break;
                                }
                                hibana_runtime::host::sleep(Duration::from_millis(1)).await?;
                            }
                        }
                        hibana_runtime::host::sleep(delays[1]).await?;
                        localside::worker::run(&mut b, &b_bytes)
                            .await
                            .map_err(|error| ShellError::message(format!("worker B: {error}")))?;
                    };
                    async {
                        if let Some(rank) = order
                            .iter()
                            .position(|id| *id == 2)
                            .filter(|rank| *rank > 0)
                        {
                            loop {
                                let events = effects.events.take();
                                let started = events.iter().any(|event| {
                                    event["boundary"] == "start"
                                        && event["worker"] == order[rank - 1]
                                });
                                effects.events.set(events);
                                if started {
                                    break;
                                }
                                hibana_runtime::host::sleep(Duration::from_millis(1)).await?;
                            }
                        }
                        hibana_runtime::host::sleep(delays[2]).await?;
                        localside::worker::run(&mut c, &c_bytes)
                            .await
                            .map_err(|error| ShellError::message(format!("worker C: {error}")))?;
                    };
                    Ok::<(), ShellError>(())
                },
                hibana_runtime::host::deadline(Duration::from_secs(5)),
            )
            .await;
            let observed_inputs = inputs.each_ref().map(|input| {
                let value = input.take();
                input.set(value.clone());
                value
            });
            let events = effects.events.take();
            if let Some(directory) = std::env::var_os("CHOREOFIX_TOOL_CASE_OUTPUT") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory)?;
                std::fs::write(
                    directory.join(format!(
                        "{}{}{}-{profile}.json",
                        order[0], order[1], order[2]
                    )),
                    data::encode(&json!({"order":order,"profile":profile,"events":events,
                        "inputs":observed_inputs,"frames":bus.transport.take_observations(),
                        "error":execution.as_ref().err().map(ToString::to_string),
                        "controlled_native_fixture":true,"model_calls":0}))?,
                )?;
            }
            execution.map_err(|error| {
                ShellError::message(format!(
                    "order={order:?} profile={profile}: {error}; inputs={observed_inputs:?}"
                ))
            })?;
            assert_eq!(
                events.len(),
                6,
                "order={order:?} profile={profile}: {events:?}"
            );
            let (mut active, mut maximum) = (0, 0);
            let mut starts = Vec::new();
            for event in &events {
                if event["boundary"] == "start" {
                    active += 1;
                    maximum = maximum.max(active);
                    starts.push(event["worker"].as_u64().unwrap() as usize);
                    if profile < 2 && starts.len() == 3 {
                        assert_eq!(
                            active, 2,
                            "third job must start before the slower one ends: {events:?}"
                        );
                    }
                } else {
                    active -= 1;
                }
                assert!(
                    (0..=2).contains(&active),
                    "actual two-tool capacity: {events:?}"
                );
            }
            assert_eq!(
                starts, order,
                "effect order comes from actual received requests"
            );
            assert_eq!(active, 0);
            assert_eq!(
                maximum, *expected_maximum,
                "order={order:?} profile={profile}: {events:?}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 30);
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn production_receiver_rejects_changed_tool_requests_before_effect() -> ShellResult<()> {
    use hibana::{
        g,
        runtime::{
            tap,
            wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
        },
    };
    // Claim the correct wire schema without validating the attacker's bytes.
    // The actual receiving Endpoint must apply JobArguments' validator.
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
        const SCHEMA_ID: u32 = <data::JobArguments<WORKER_A> as WirePayload>::SCHEMA_ID;
        type Decoded<'a> = &'a [u8];
        fn validate_payload(_: Payload<'_>) -> Result<(), CodecError> {
            Ok(())
        }
        fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
            input.as_bytes()
        }
    }
    type UntrustedRequest = g::Msg<3, Untrusted>;
    let programs = programs();
    for bytes in [
        br#"{"job_id":"beta_job_b"}"#.as_slice(),
        br#"{"job_id":"alpha_job_a","extra":true}"#.as_slice(),
        b"{".as_slice(),
        b"null".as_slice(),
    ] {
        let session = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(data::AGENTS, [(session, TOOL_0), (session, TOOL_1)]);
        let inputs = [const { Cell::new(Value::Null) }; 5];
        let mut host_slab = vec![0; 1024 * 1024];
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut host_storage = SessionKitStorage::uninit();
        let mut caller_storage = SessionKitStorage::uninit();
        let host = host_storage
            .init()
            .rendezvous(&mut host_slab, &bus.transport)?;
        let caller = caller_storage
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        host.set_resolver(
            &programs.depot,
            ResolverRef::<FIRST_A>::decision_state(&inputs[0], data::worker_a),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let mut depot = host.enter(session, &programs.depot)?;
        let tool0 = host.enter(session, &programs.tool_0)?;
        let tool1 = host.enter(session, &programs.tool_1)?;
        let mut worker = caller.enter(session, &programs.a)?;
        let effects = ToolEffects {
            events: Cell::new(Vec::new()),
            delays: [Duration::ZERO; 3],
        };
        let error = hibana_runtime::race(
            async {
                async {
                    localside::tool_depot::run(&mut depot, (tool0, tool1), (&bus, &inputs)).await?;
                };
                async {
                    bus.fixture_peer::<1>(&effects, &["do_job"]).await?;
                };
                async {
                    let [mut intent, mut work] = worker.par()?;
                    let admitted =
                        data::JobArguments::<WORKER_A>::from_bytes(br#"{"job_id":"alpha_job_a"}"#)?;
                    intent.send::<Intent<WORKER_A>>(&admitted).await?;
                    let mut tool = work.recv::<Loan0>().await?;
                    tool.send::<UntrustedRequest>(&Untrusted(bytes)).await?;
                    core::future::pending::<ShellResult<()>>().await?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(Duration::from_secs(5)),
        )
        .await
        .expect_err("changed actual resource request must fail");
        assert!(
            matches!(error, ShellError::Endpoint(_))
                && error.to_string().contains("Codec(Malformed)"),
            "must reject input rather than time out: {error}"
        );
        assert_eq!(
            host.tap()
                .filter(|event| event.id() == tap::ENDPOINT_RECV)
                .count(),
            1,
            "only the valid intent may commit; the malformed resource request must not"
        );
        assert!(
            effects.events.take().is_empty(),
            "production receiver must reject before the actual RPC"
        );
    }
    Ok(())
}
