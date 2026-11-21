//! Actual tool Endpoints and production receiver, not full 11M qualification.
#[path = "../../../src/imp/projection.rs"]
pub mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::projection;
    pub(crate) use crate::transport;
}
#[path = "../../../src/tasks/manufacturing_rework/mod.rs"]
pub mod manufacturing_rework;
use hibana::g;
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    program::{RoleProgram, project},
    resolver::{DecisionArm, ResolverError, ResolverRef},
    tap::TapEvent,
};
use hibana_runtime::host::deadline;
use manufacturing_rework::{global::*, imp as data, localside};
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{Domain, ReceivedEffects, SimulatorOperation},
    },
};
use std::{cell::Cell, time::Duration};

struct Programs {
    a_files: RoleProgram<A_FILES>,
    alpha_c_files: RoleProgram<ALPHA_C_FILES>,
    alpha_service: RoleProgram<ALPHA_SERVICE>,
    alpha_owner: RoleProgram<ALPHA_OWNER>,
    alpha_machine: RoleProgram<ALPHA_MACHINE>,
    alpha_c: RoleProgram<ALPHA_C>,
    b_files: RoleProgram<B_FILES>,
    beta_a_files: RoleProgram<BETA_A_FILES>,
    beta_service: RoleProgram<BETA_SERVICE>,
    beta_owner: RoleProgram<BETA_OWNER>,
    beta_machine: RoleProgram<BETA_MACHINE>,
    beta_a: RoleProgram<BETA_A>,
    c_files: RoleProgram<C_FILES>,
    gamma_b_files: RoleProgram<GAMMA_B_FILES>,
    gamma_service: RoleProgram<GAMMA_SERVICE>,
    gamma_owner: RoleProgram<GAMMA_OWNER>,
    gamma_machine: RoleProgram<GAMMA_MACHINE>,
    gamma_b: RoleProgram<GAMMA_B>,
    dispatcher: RoleProgram<DISPATCHER>,
    inspector: RoleProgram<INSPECTOR>,
    a: RoleProgram<WORKER_A>,
    b: RoleProgram<WORKER_B>,
    c: RoleProgram<WORKER_C>,
    alpha: RoleProgram<ALPHA>,
    beta: RoleProgram<BETA>,
    gamma: RoleProgram<GAMMA>,
    depot: RoleProgram<DEPOT>,
    tool_0: RoleProgram<TOOL_0>,
    tool_1: RoleProgram<TOOL_1>,
}
fn programs() -> Programs {
    let source = g::seq(workstation_files(), source());
    Programs {
        a_files: project::<A_FILES>(&source),
        alpha_c_files: project::<ALPHA_C_FILES>(&source),
        alpha_service: project::<ALPHA_SERVICE>(&source),
        alpha_owner: project::<ALPHA_OWNER>(&source),
        alpha_machine: project::<ALPHA_MACHINE>(&source),
        alpha_c: project::<ALPHA_C>(&source),
        b_files: project::<B_FILES>(&source),
        beta_a_files: project::<BETA_A_FILES>(&source),
        beta_service: project::<BETA_SERVICE>(&source),
        beta_owner: project::<BETA_OWNER>(&source),
        beta_machine: project::<BETA_MACHINE>(&source),
        beta_a: project::<BETA_A>(&source),
        c_files: project::<C_FILES>(&source),
        gamma_b_files: project::<GAMMA_B_FILES>(&source),
        gamma_service: project::<GAMMA_SERVICE>(&source),
        gamma_owner: project::<GAMMA_OWNER>(&source),
        gamma_machine: project::<GAMMA_MACHINE>(&source),
        gamma_b: project::<GAMMA_B>(&source),
        dispatcher: project::<DISPATCHER>(&source),
        inspector: project::<INSPECTOR>(&source),
        a: project::<WORKER_A>(&source),
        b: project::<WORKER_B>(&source),
        c: project::<WORKER_C>(&source),
        alpha: project::<ALPHA>(&source),
        beta: project::<BETA>(&source),
        gamma: project::<GAMMA>(&source),
        depot: project::<DEPOT>(&source),
        tool_0: project::<TOOL_0>(&source),
        tool_1: project::<TOOL_1>(&source),
    }
}

/// Completed external material records and controlled quality outcomes. This
/// fixture has no Endpoint permission, station scheduler or application cursor.
struct QualityEffects {
    job: usize,
    rejected_inspections: usize,
    events: Cell<Vec<Value>>,
}
impl ReceivedEffects for QualityEffects {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let arguments = data::decode(received.as_bytes());
        async move {
            let (agent, tool) = P::RPC;
            let arguments = arguments?;
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
                        _ => return Err(ShellError::message("unknown quality-fixture operation")),
                    }
            {
                return Err(ShellError::message("wrong quality-fixture caller or job"));
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
                            .ok_or("inspection without actual fixture material")?["result"]["data"]
                            ["part"];
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
            let result = result.map_err(ShellError::message)?;
            let mut events = self.events.take();
            events.push(json!({"agent":agent,"tool":tool,"arguments":arguments,"result":result}));
            self.events.set(events);
            Ok(result)
        }
    }
}

fn observer<'a>(
    producer: usize,
    records: &'a Cell<Vec<TapEvent>>,
    observed_producers: &'a Cell<Vec<usize>>,
) -> impl Fn(TapEvent) + 'a {
    move |event| {
        let mut order = observed_producers.take();
        order.push(producer);
        observed_producers.set(order);
        let mut capture = records.take();
        capture.push(event);
        records.set(capture);
    }
}

fn audit_execution<const N: usize>(
    session: SessionId,
    snapshot: &[TapEvent],
    captured: &[Vec<TapEvent>; N],
    checkpoints: [Option<u32>; N],
    observed_producers: &[usize],
) -> ShellResult<usize> {
    use hibana_epf::{Acceptance, AuditStorage, ExecutionStorage, Protocol, RecordId, Unfold};
    let captures = captured
        .iter()
        .zip(checkpoints)
        .map(|(records, end)| hibana_epf::Capture::new(records, end))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ShellError::message(format!("incomplete actor capture: {error:?}")))?;
    let mut source_storage = vec![0; Protocol::workspace_len(snapshot).unwrap()];
    let protocol = Protocol::new(snapshot, &mut source_storage).unwrap();
    let count = captured.iter().map(Vec::len).sum::<usize>();
    let capacity = count.checked_mul(2).and_then(|n| n.checked_add(1)).unwrap();
    let mut unfolding = vec![Unfold::Source; capacity];
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
                execution: ExecutionStorage {
                    session: session.raw(),
                    observed_producers,
                    order: &mut order,
                    occurrences: &mut occurrences,
                    acceptances: &mut acceptances,
                },
            },
            &mut scratch,
        )
        .map_err(|error| ShellError::message(format!("source execution: {error:?}")))?;
    Ok(audit.execution().order.len())
}

fn initial_quality(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match value["success"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}

/// Complete native fixture for one job continuation, not the six-job station/
/// tool allocator or a Wasm/model implementation of 11M.
#[hibana_runtime::scope]
async fn quality_case<const JOB: u8, const WORKER: u8, const CAP: u8>(
    rejected: usize,
    work: impl hibana::runtime::program::Projectable + Copy,
) -> ShellResult<Value> {
    use hibana::g::Message;
    use hibana::runtime::program::project;
    const OTHER_WORKER: u8 = 25;
    let source = hibana::g::par(
        job_quality::<JOB, WORKER, CAP>(work, work),
        hibana::g::seq(
            hibana::g::send::<DEPOT, OTHER_WORKER, Loan1>(),
            hibana::g::send::<OTHER_WORKER, DEPOT, Return1>(),
        ),
    );
    let worker_program = project::<WORKER>(&source);
    let job_program = project::<CAP>(&source);
    let inspector_program = project::<INSPECTOR>(&source);
    let dispatcher_program = project::<DISPATCHER>(&source);
    let depot_program = project::<DEPOT>(&source);
    let session = SessionId::new(SESSION + 1);
    let tool_program = project::<TOOL_0>(&source);
    let other_program = project::<OTHER_WORKER>(&source);
    let other_tool_program = project::<TOOL_1>(&source);
    let (bus, _, _) = Bus::open(
        data::AGENTS,
        [(session, CAP), (session, TOOL_0), (session, TOOL_1)],
    );
    let verdict = Cell::new(Value::Null);
    let records: [Cell<Vec<hibana::runtime::tap::TapEvent>>; 5] =
        core::array::from_fn(|_| Cell::new(Vec::new()));
    let observed_producers = Cell::new(Vec::new());
    let observers =
        [0, 1, 2, 3, 4].map(|producer| observer(producer, &records[producer], &observed_producers));
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
    let mut other_slab = vec![0; 1024 * 1024];
    let mut other_storage = SessionKitStorage::uninit();
    let other_runtime = other_storage
        .init()
        .rendezvous(&mut other_slab, &bus.transport)?;
    other_runtime.observe(&observers[4]);
    worker_runtime.observe(&observers[0]);
    inspector_runtime.observe(&observers[1]);
    dispatcher_runtime.observe(&observers[2]);
    depot_runtime.observe(&observers[3]);
    inspector_runtime
        .set_resolver(
            &inspector_program,
            ResolverRef::<QUALITY_FIRST>::decision_state(&verdict, initial_quality),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let mut worker = worker_runtime.enter(session, &worker_program)?;
    let mut job = worker_runtime.enter(session, &job_program)?;
    let mut inspector = inspector_runtime.enter(session, &inspector_program)?;
    let mut dispatcher = dispatcher_runtime.enter(session, &dispatcher_program)?;
    let mut cabinet = depot_runtime.enter(session, &depot_program)?;
    let [mut depot, mut other_loan] = cabinet.par()?;
    let mut other = other_runtime.enter(session, &other_program)?;
    let other_tool = depot_runtime.enter(session, &other_tool_program)?;
    other_loan.send::<Loan1>(other_tool).await?;
    let occupied = other.recv::<Loan1>().await?;
    let arguments_bytes = data::encode(&json!({"job_id":data::JOBS[usize::from(JOB)]}))?;
    let intent = data::JobArguments::<JOB>::from_bytes(&arguments_bytes)?;
    let arguments = data::JobArguments::<JOB, { data::DO_JOB }>::from_bytes(&arguments_bytes)?;
    let mut available = if JOB < 3 {
        Some(depot_runtime.enter(session, &tool_program)?)
    } else {
        None
    };
    let check = data::JobArguments::<JOB, { data::INSPECT_QUALITY }>::from_bytes(&arguments_bytes)?;
    let admitted = data::submission::<JOB>(arguments.value());
    let effects = QualityEffects {
        job: usize::from(JOB),
        rejected_inspections: rejected,
        events: Cell::new(Vec::new()),
    };
    let ((), (), (), mut retired) = hibana_runtime::race(async {
        async {
            let mut previous: Option<Value> = None;
            loop {
                let result = if JOB < 3 {
                    job.send::<Intent<JOB>>(&intent).await?;
                    let mut tool = worker.recv::<Loan0>().await?;
                    let requested = data::JobArguments::<WORKER, {data::DO_JOB}>::from_bytes(&arguments_bytes)?;
                    tool.send::<Request0<WORKER>>(&requested).await?;
                    let bytes = tool.recv::<Reply0>().await?.to_vec();
                    async {
                        let [mut returned, mut issued] = worker.par()?;
                        async { returned.send::<Return0>(tool).await?; };
                        async { issued.send::<JobResult>(&bytes.as_slice()).await?; };
                        Ok::<(),ShellError>(())
                    }.await?;
                    data::decode(job.recv::<JobResult>().await?)?
                } else {
                    job.send::<JobIntent<JOB>>(&arguments).await?;
                    data::decode(job.recv::<JobResult>().await?)?
                };
                data::completed(&admitted, &result, previous.as_ref().map(|order| (&order["inspection"], &order["arguments"], &order["result"])))?;
                let bytes = data::encode(&result["data"]["part"])?;
                job.send::<PartReady>(&bytes.as_slice()).await?;
                worker.send::<JobLoan<CAP>>(job).await?;
                let branch = worker.offer().await?;
                match branch.label() {
                    label if label == ApprovalForWorker::<JOB>::LOGICAL_LABEL => {
                        branch.recv::<ApprovalForWorker<JOB>>().await?;
                        break;
                    }
                    OrderForWorker::LOGICAL_LABEL => {
                        previous = Some(data::decode(branch.recv::<OrderForWorker>().await?)?);
                        job = worker.recv::<JobLoan<CAP>>().await?;
                    }
                    _ => return Err(ShellError::message("unexpected worker quality continuation")),
                }
            }
        };
        async {
            loop {
                let part = data::decode(inspector.recv::<PartReady>().await?)?;
                let mut job = inspector.recv::<JobLoan<CAP>>().await?;
                job.send::<InspectRequest<JOB>>(&check).await?;
                let request = inspector.recv::<InspectRequest<JOB>>().await?;
                let received = data::decode(request.as_bytes())?;
                data::inspection_input(&part, &received)?;
                let result = effects.apply(request).await?;
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
                        let request = dispatcher.recv::<DispatchRequest<JOB>>().await?;
                        let received = data::decode(request.as_bytes())?;
                        data::rework_input(&inspection, &received)?;
                        let result = effects.apply(request).await?;
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
            let mut previous: Option<Value> = None;
            loop {
                let result = if let Some(tool) = available.take() {
                    let submission = data::admitted(depot.recv::<Intent<JOB>>().await?)?;
                    depot.send::<Loan0>(tool).await?;
                    let request = depot.recv::<Request0<WORKER>>().await?;
                    if data::decode(request.as_bytes())? != submission["arguments"] {
                        return Err(ShellError::message("tool request differs from the submitted job"));
                    }
                    let result = effects.apply(request).await?;
                    data::completed(&submission, &result, previous.as_ref().map(|order| (&order["inspection"], &order["arguments"], &order["result"])))?;
                    let bytes = data::encode(&result)?;
                    depot.send::<Reply0>(&bytes.as_slice()).await?;
                    available = Some(depot.recv::<Return0>().await?);
                    result
                } else {
                    let result = effects.apply(depot.recv::<JobIntent<JOB>>().await?).await?;
                    data::completed(&admitted, &result, previous.as_ref().map(|order| (&order["inspection"], &order["arguments"], &order["result"])))?;
                    let bytes = data::encode(&result)?;
                    depot.send::<JobResult>(&bytes.as_slice()).await?;
                    result
                };
                let part = &result["data"]["part"];
                let branch = depot.offer().await?;
                match branch.label() {
                    label if label == Approved::<JOB>::LOGICAL_LABEL => {
                        let approved = data::decode(branch.recv::<Approved<JOB>>().await?)?;
                        data::inspected(part, &arguments.value(), &approved)?;
                        let retired = depot.recv::<JobLoan<CAP>>().await?;
                        return Ok::<_, ShellError>(retired);
                    }
                    OrderEvidence::LOGICAL_LABEL => {
                        let order = data::decode(branch.recv::<OrderEvidence>().await?)?;
                        data::inspected(part, &arguments.value(), &order["inspection"])?;
                        data::dispatched(&order["inspection"],&order["arguments"],&order["result"])?;
                        previous = Some(order);
                    }
                    _ => return Err(ShellError::message("unexpected physical job continuation")),
                }
            }
        };
    },deadline(Duration::from_secs(5))).await?;
    other.send::<Return1>(occupied).await?;
    let _returned = other_loan.recv::<Return1>().await?;
    let checkpoints = [
        worker_runtime.tap().checkpoint(),
        inspector_runtime.tap().checkpoint(),
        dispatcher_runtime.tap().checkpoint(),
        depot_runtime.tap().checkpoint(),
        other_runtime.tap().checkpoint(),
    ];
    let captured = records.each_ref().map(|records| records.take());
    let observed_producers = observed_producers.take();
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
    let count = audit_execution(
        session,
        &snapshot,
        &captured,
        checkpoints,
        &observed_producers,
    )?;
    let dynamic = json!({"status":"passed","captured_records":count,
        "application_effects_certified":false,"scope":"complete native job-continuation source execution; fixture effects are checked separately"});
    // The terminal probe runs after every legitimate participant has finished.
    // An undeclared operation on this retired actor/view may end the closed run;
    // it must not remain Pending or produce another work frame/effect.
    let mut frames: Vec<_> = bus.transport.take_observations().collect();
    let denied = hibana_runtime::race(
        async {
            if JOB < 3 {
                retired.send::<Intent<JOB>>(&intent).await
            } else {
                retired.send::<JobIntent<JOB>>(&arguments).await
            }
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
    let after: Vec<_> = bus.transport.take_observations().collect();
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
        other_runtime.tap().checkpoint(),
    ];
    let terminal_probe_records = records.each_ref().map(|records| records.take());
    let events = effects.events.take();
    assert_eq!(events.len(), 3 * rejected + 2);
    frames.extend(bus.transport.take_observations());
    let tool_requests = frames
        .iter()
        .filter(|frame| {
            frame["action"] == "send" && frame["source"] == TOOL_0 && frame["target"] == DEPOT
        })
        .count();
    assert_eq!(tool_requests, if JOB < 3 { rejected + 1 } else { 0 });
    let sent = frames
        .iter()
        .filter(|frame| frame["action"] == "send")
        .collect::<Vec<_>>();
    assert_eq!(
        (sent[0]["source"].as_u64(), sent[0]["target"].as_u64()),
        (Some(DEPOT.into()), Some(OTHER_WORKER.into()))
    );
    let last = sent.last().expect("returned second tool");
    assert_eq!(
        (last["source"].as_u64(), last["target"].as_u64()),
        (Some(OTHER_WORKER.into()), Some(DEPOT.into()))
    );
    let outcome = json!({"job":data::JOBS[usize::from(JOB)],"rejected_inspections":rejected,
        "events":events,"frames":frames,"retired_request_rejected":true,"retired_request_error":rejection,"free_tool_requests":tool_requests,"independent_loan_frame":sent[0]["frame_id"],"independent_return_frame":last["frame_id"],"controlled_native_fixture":true,
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
async fn all_six_quality_continuations_preserve_actual_rework_and_close_on_approval()
-> ShellResult<()> {
    use hibana::{g, runtime::program::Projectable};
    fn tool_job<const JOB: u8, const CAP: u8>() -> impl Projectable + Copy {
        g::seq(
            g::send::<CAP, DEPOT, Intent<JOB>>(),
            whole0::<JOB>(g::send::<JOB, CAP, JobResult>()),
        )
    }
    fn station_job<const JOB: u8, const CAP: u8>() -> impl Projectable + Copy {
        g::seq(
            g::send::<CAP, DEPOT, JobIntent<JOB>>(),
            g::send::<DEPOT, CAP, JobResult>(),
        )
    }
    for rejected in 0..=3 {
        quality_case::<0, WORKER_A, 9>(rejected, tool_job::<0, 9>()).await?;
        quality_case::<1, WORKER_B, 10>(rejected, tool_job::<1, 10>()).await?;
        quality_case::<2, WORKER_C, 11>(rejected, tool_job::<2, 11>()).await?;
        quality_case::<3, WORKER_A, 12>(rejected, station_job::<3, 12>()).await?;
        quality_case::<4, WORKER_B, 13>(rejected, station_job::<4, 13>()).await?;
        quality_case::<5, WORKER_C, 14>(rejected, station_job::<5, 14>()).await?;
    }
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
        if agent == "INSPECTOR" && tool == "inspect_quality" {
            let events = self.events.take();
            let part = events.iter().find_map(|row| {
                let part = &row["result"]["data"]["part"];
                (row["boundary"] == "finish" && part["job_id"] == arguments["job_id"])
                    .then(|| part.clone())
            });
            self.events.set(events);
            let part = part.ok_or("inspection before the tool finished")?;
            let job = data::JOBS
                .iter()
                .position(|name| arguments["job_id"] == *name)
                .ok_or("unknown inspection target")?;
            return Ok(
                json!({"tool_name":tool,"success":true,"data":{"job_id":arguments["job_id"],
                "input_part":part,"status":"inspected","inspector":agent,"receipt":format!("inspection/{}",job+1)}}),
            );
        }
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
            let (mut connection, _, _) = Bus::open(
                ["WORKER_A", "WORKER_B", "WORKER_C", "INSPECTOR"],
                [
                    (session, TOOL_0),
                    (session, TOOL_1),
                    (session, ALPHA),
                    (session, ALPHA_C),
                    (session, ALPHA_MACHINE),
                    (session, BETA),
                    (session, BETA_A),
                    (session, BETA_MACHINE),
                    (session, GAMMA),
                    (session, GAMMA_B),
                    (session, GAMMA_MACHINE),
                ],
            );
            let (bus, simulator) = connection.split();
            let inputs = [const { Cell::new(Value::Null) }; 5];
            let next = (&inputs[0], &inputs[1]);
            let records: [Cell<Vec<TapEvent>>; 2] = core::array::from_fn(|_| Cell::default());
            let observed_producers = Cell::default();
            let observers =
                [0, 1].map(|producer| observer(producer, &records[producer], &observed_producers));
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
            host.observe(&observers[0]);
            caller.observe(&observers[1]);
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

            let mut alpha_data = caller.enter(session, &programs.a_files)?;
            let mut alpha_other_data = caller.enter(session, &programs.alpha_c_files)?;
            let mut alpha_owner = caller.enter(session, &programs.alpha_owner)?;
            let alpha_machine = caller.enter(session, &programs.alpha_machine)?;
            let alpha_file = caller.enter(session, &programs.alpha)?;
            let alpha_other = caller.enter(session, &programs.alpha_c)?;
            alpha_data
                .send::<shell::global::Loan<ALPHA>>(alpha_file)
                .await?;
            alpha_other_data
                .send::<shell::global::Loan<ALPHA_C>>(alpha_other)
                .await?;
            let (mut alpha, mut alpha_other) = async {
                let [mut primary, mut other] = alpha_owner.par()?;
                async { primary.recv::<shell::global::Loan<ALPHA>>().await };
                async { other.recv::<shell::global::Loan<ALPHA_C>>().await };
            }
            .await?;

            let mut beta_data = caller.enter(session, &programs.b_files)?;
            let mut beta_other_data = caller.enter(session, &programs.beta_a_files)?;
            let mut beta_owner = caller.enter(session, &programs.beta_owner)?;
            let beta_machine = caller.enter(session, &programs.beta_machine)?;
            let beta_file = caller.enter(session, &programs.beta)?;
            let beta_other = caller.enter(session, &programs.beta_a)?;
            beta_data
                .send::<shell::global::Loan<BETA>>(beta_file)
                .await?;
            beta_other_data
                .send::<shell::global::Loan<BETA_A>>(beta_other)
                .await?;
            let (mut beta, mut beta_other) = async {
                let [mut primary, mut other] = beta_owner.par()?;
                async { primary.recv::<shell::global::Loan<BETA>>().await };
                async { other.recv::<shell::global::Loan<BETA_A>>().await };
            }
            .await?;

            let mut gamma_data = caller.enter(session, &programs.c_files)?;
            let mut gamma_other_data = caller.enter(session, &programs.gamma_b_files)?;
            let mut gamma_owner = caller.enter(session, &programs.gamma_owner)?;
            let gamma_machine = caller.enter(session, &programs.gamma_machine)?;
            let gamma_file = caller.enter(session, &programs.gamma)?;
            let gamma_other = caller.enter(session, &programs.gamma_b)?;
            gamma_data
                .send::<shell::global::Loan<GAMMA>>(gamma_file)
                .await?;
            gamma_other_data
                .send::<shell::global::Loan<GAMMA_B>>(gamma_other)
                .await?;
            let (mut gamma, mut gamma_other) = async {
                let [mut primary, mut other] = gamma_owner.par()?;
                async { primary.recv::<shell::global::Loan<GAMMA>>().await };
                async { other.recv::<shell::global::Loan<GAMMA_B>>().await };
            }
            .await?;
            let mut alpha_service = caller.enter(session, &programs.alpha_service)?;
            let alpha_work = QualityEffects {
                job: 5,
                rejected_inspections: 0,
                events: Cell::default(),
            };
            let mut beta_service = caller.enter(session, &programs.beta_service)?;
            let beta_work = QualityEffects {
                job: 3,
                rejected_inspections: 0,
                events: Cell::default(),
            };
            let mut gamma_service = caller.enter(session, &programs.gamma_service)?;
            let gamma_work = QualityEffects {
                job: 4,
                rejected_inspections: 0,
                events: Cell::default(),
            };
            let mut inspector = caller.enter(session, &programs.inspector)?;
            let mut dispatcher = caller.enter(session, &programs.dispatcher)?;
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
                        localside::tool_depot::run(
                            &mut depot,
                            (tool0, tool1),
                            (&simulator, &inputs),
                        )
                        .await
                        .map_err(|error| ShellError::message(format!("depot: {error}")))?;
                    };
                    async {
                        let mut machine = alpha_machine;
                        for _ in 0..2 {
                            drop(alpha_owner.offer().await?);
                            alpha_owner.send::<JobLoan<ALPHA_MACHINE>>(machine).await?;
                            machine = alpha_owner.recv::<JobLoan<ALPHA_MACHINE>>().await?;
                        }
                    };
                    async {
                        localside::worker::station_job::<ALPHA_SERVICE,ALPHA_MACHINE,ALPHA_JOB_C>(&mut alpha_service,&alpha_work).await?;
                    };
                    async {
                        localside::worker::run::<WORKER_A, ALPHA_MACHINE>(&mut a, &simulator)
                            .await?;
                    };
                    async {
                        let mut machine = beta_machine;
                        for _ in 0..2 {
                            drop(beta_owner.offer().await?);
                            beta_owner.send::<JobLoan<BETA_MACHINE>>(machine).await?;
                            machine = beta_owner.recv::<JobLoan<BETA_MACHINE>>().await?;
                        }
                    };
                    async {
                        localside::worker::station_job::<BETA_SERVICE,BETA_MACHINE,BETA_JOB_A>(&mut beta_service,&beta_work).await?;
                    };
                    async {
                        localside::worker::run::<WORKER_B, BETA_MACHINE>(&mut b, &simulator)
                            .await?;
                    };
                    async {
                        let mut machine = gamma_machine;
                        for _ in 0..2 {
                            drop(gamma_owner.offer().await?);
                            gamma_owner.send::<JobLoan<GAMMA_MACHINE>>(machine).await?;
                            machine = gamma_owner.recv::<JobLoan<GAMMA_MACHINE>>().await?;
                        }
                    };
                    async {
                        localside::worker::station_job::<GAMMA_SERVICE,GAMMA_MACHINE,GAMMA_JOB_B>(&mut gamma_service,&gamma_work).await?;
                    };
                    async {
                        localside::worker::run::<WORKER_C, GAMMA_MACHINE>(&mut c, &simulator)
                            .await?;
                    };
                    async {
                        for job in order {
                            let notice = match job {
                                0 => dispatcher.recv::<Approved<WORKER_A>>().await?,
                                1 => dispatcher.recv::<Approved<WORKER_B>>().await?,
                                2 => dispatcher.recv::<Approved<WORKER_C>>().await?,
                                _ => unreachable!(),
                            };
                            let notice = data::decode(notice)?;
                            assert_eq!(notice["data"]["job_id"], data::JOBS[job]);
                            assert_eq!(notice["success"], true);
                        }
                    };
                    async {
                        for worker in order {
                            let bytes = match worker {
                                0 => inspector.recv::<FinishedPart<WORKER_A>>().await?.to_vec(),
                                1 => inspector.recv::<FinishedPart<WORKER_B>>().await?.to_vec(),
                                2 => inspector.recv::<FinishedPart<WORKER_C>>().await?.to_vec(),
                                _ => unreachable!(),
                            };
                            let arguments = json!({"job_id":data::JOBS[worker]});
                            let encoded = data::encode(&arguments)?;
                            match worker {
                                0 => {
                                    inspector
                                        .send::<CheckPart<WORKER_A>>(
                                            &data::JobArguments::from_bytes(&encoded)?,
                                        )
                                        .await?
                                }
                                1 => {
                                    inspector
                                        .send::<CheckPart<WORKER_B>>(
                                            &data::JobArguments::from_bytes(&encoded)?,
                                        )
                                        .await?
                                }
                                2 => {
                                    inspector
                                        .send::<CheckPart<WORKER_C>>(
                                            &data::JobArguments::from_bytes(&encoded)?,
                                        )
                                        .await?
                                }
                                _ => unreachable!(),
                            }
                            let result = data::decode(inspector.recv::<InspectionResult>().await?)?;
                            data::inspected(&data::decode(&bytes)?, &arguments, &result)?;
                        }
                        for job in [ALPHA_JOB_C, BETA_JOB_A, GAMMA_JOB_B] {
                            let part = match job {
                                ALPHA_JOB_C => inspector.recv::<FinishedPart<ALPHA_JOB_C>>().await?,
                                BETA_JOB_A => inspector.recv::<FinishedPart<BETA_JOB_A>>().await?,
                                GAMMA_JOB_B => inspector.recv::<FinishedPart<GAMMA_JOB_B>>().await?,
                                _ => unreachable!(),
                            };
                            assert_eq!(data::decode(part)?["job_id"], data::JOBS[usize::from(job)]);
                        }
                    };
                    async {
                        bus.fixture_peer::<6>(&effects, &["do_job", "inspect_quality"])
                            .await?;
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
                        let arguments = data::JobArguments::<WORKER_A>::from_bytes(&a_bytes)?;
                        alpha.send::<StationJob<WORKER_A>>(&arguments).await?;
                        let result = data::decode(alpha.recv::<JobResult>().await?)?;
                        data::completed(
                            &data::submission::<WORKER_A>(arguments.value()),
                            &result,
                            None,
                        )?;
                        let approval = data::decode(alpha.recv::<JobUpdate>().await?)?;
                        data::inspected(&result["data"]["part"], &arguments.value(), &approval)?;
                        let arguments = data::JobArguments::<ALPHA_JOB_C>::from_bytes(&data::encode(&json!({"job_id":data::JOBS[5]}))?)?;
                        alpha_other.send::<StationJob<ALPHA_JOB_C>>(&arguments).await?;
                        let result = data::decode(alpha_other.recv::<JobResult>().await?)?;
                        data::completed(&data::submission::<ALPHA_JOB_C>(arguments.value()), &result, None)?;
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
                        let arguments = data::JobArguments::<WORKER_B>::from_bytes(&b_bytes)?;
                        beta.send::<StationJob<WORKER_B>>(&arguments).await?;
                        let result = data::decode(beta.recv::<JobResult>().await?)?;
                        data::completed(
                            &data::submission::<WORKER_B>(arguments.value()),
                            &result,
                            None,
                        )?;
                        let approval = data::decode(beta.recv::<JobUpdate>().await?)?;
                        data::inspected(&result["data"]["part"], &arguments.value(), &approval)?;
                        let arguments = data::JobArguments::<BETA_JOB_A>::from_bytes(&data::encode(&json!({"job_id":data::JOBS[3]}))?)?;
                        beta_other.send::<StationJob<BETA_JOB_A>>(&arguments).await?;
                        let result = data::decode(beta_other.recv::<JobResult>().await?)?;
                        data::completed(&data::submission::<BETA_JOB_A>(arguments.value()), &result, None)?;
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
                        let arguments = data::JobArguments::<WORKER_C>::from_bytes(&c_bytes)?;
                        gamma.send::<StationJob<WORKER_C>>(&arguments).await?;
                        let result = data::decode(gamma.recv::<JobResult>().await?)?;
                        data::completed(
                            &data::submission::<WORKER_C>(arguments.value()),
                            &result,
                            None,
                        )?;
                        let approval = data::decode(gamma.recv::<JobUpdate>().await?)?;
                        data::inspected(&result["data"]["part"], &arguments.value(), &approval)?;
                        let arguments = data::JobArguments::<GAMMA_JOB_B>::from_bytes(&data::encode(&json!({"job_id":data::JOBS[4]}))?)?;
                        gamma_other.send::<StationJob<GAMMA_JOB_B>>(&arguments).await?;
                        let result = data::decode(gamma_other.recv::<JobResult>().await?)?;
                        data::completed(&data::submission::<GAMMA_JOB_B>(arguments.value()), &result, None)?;
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
            let checkpoints = [host.tap().checkpoint(), caller.tap().checkpoint()];
            let captured = records.each_ref().map(Cell::take);
            let observed_producers = observed_producers.take();
            let snapshot: Vec<_> = hibana::runtime::tap::projected(&programs.depot).collect();
            let audit = audit_execution(
                session,
                &snapshot,
                &captured,
                checkpoints,
                &observed_producers,
            );
            if let Some(directory) = std::env::var_os("CHOREOFIX_TOOL_CASE_OUTPUT") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory)?;
                std::fs::write(
                    directory.join(format!(
                        "{}{}{}-{profile}.json",
                        order[0], order[1], order[2]
                    )),
                    data::encode(&json!({
                        "order": order,
                        "profile": profile,
                        "events": events,
                        "session": session.raw(),
                        "checkpoints": checkpoints,
                        "observed_producers": observed_producers,
                        "protocol": snapshot.iter().copied().map(<[u8; 16]>::from).collect::<Vec<_>>(),
                        "records": captured.each_ref().map(|stream| {
                            stream.iter().copied().map(<[u8; 16]>::from).collect::<Vec<_>>()
                        }),
                        "audited_records": audit.as_ref().ok(),
                        "audit_error": audit.as_ref().err().map(ToString::to_string),
                        "inputs": observed_inputs,
                        "frames": bus.transport.take_observations().collect::<Vec<_>>(),
                        "error": execution.as_ref().err().map(ToString::to_string),
                        "controlled_native_fixture": true,
                        "model_calls": 0
                    }))?,
                )?;
            }
            execution.map_err(|error| {
                ShellError::message(format!(
                    "order={order:?} profile={profile}: {error}; inputs={observed_inputs:?}"
                ))
            })?;
            assert_eq!(audit?, captured.iter().map(Vec::len).sum::<usize>());
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

#[hibana_runtime::main]
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
        const SCHEMA_ID: u32 =
            <data::JobArguments<WORKER_A, { data::DO_JOB }> as WirePayload>::SCHEMA_ID;
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
        let (mut connection, _, _) = Bus::open(
            data::AGENTS,
            [
                (session, TOOL_0),
                (session, TOOL_1),
                (session, ALPHA),
                (session, ALPHA_C),
                (session, ALPHA_MACHINE),
                (session, BETA),
                (session, BETA_A),
                (session, BETA_MACHINE),
                (session, GAMMA),
                (session, GAMMA_B),
                (session, GAMMA_MACHINE),
            ],
        );
        let (bus, simulator) = connection.split();
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

        let mut alpha_data = caller.enter(session, &programs.a_files)?;
        let mut alpha_other_data = caller.enter(session, &programs.alpha_c_files)?;
        let mut alpha_owner = caller.enter(session, &programs.alpha_owner)?;
        let alpha_machine = caller.enter(session, &programs.alpha_machine)?;
        let alpha_file = caller.enter(session, &programs.alpha)?;
        let alpha_other = caller.enter(session, &programs.alpha_c)?;
        alpha_data
            .send::<shell::global::Loan<ALPHA>>(alpha_file)
            .await?;
        alpha_other_data
            .send::<shell::global::Loan<ALPHA_C>>(alpha_other)
            .await?;
        let (mut alpha, _alpha_other) = async {
            let [mut primary, mut other] = alpha_owner.par()?;
            async { primary.recv::<shell::global::Loan<ALPHA>>().await };
            async { other.recv::<shell::global::Loan<ALPHA_C>>().await };
        }
        .await?;

        let effects = ToolEffects {
            events: Cell::new(Vec::new()),
            delays: [Duration::ZERO; 3],
        };
        let error = hibana_runtime::race(
            async {
                async {
                    localside::tool_depot::run(&mut depot, (tool0, tool1), (&simulator, &inputs))
                        .await?;
                };
                async {
                    bus.fixture_peer::<1>(&effects, &["do_job"]).await?;
                };
                async {
                    let [mut intent, mut work] = worker.par()?;
                    let admitted =
                        data::JobArguments::<WORKER_A>::from_bytes(br#"{"job_id":"alpha_job_a"}"#)?;
                    alpha.send::<StationJob<WORKER_A>>(&admitted).await?;
                    let _station = intent.recv::<StationJob<WORKER_A>>().await?;
                    alpha_owner
                        .send::<JobLoan<ALPHA_MACHINE>>(alpha_machine)
                        .await?;
                    let mut machine = intent.recv::<JobLoan<ALPHA_MACHINE>>().await?;
                    machine.send::<Intent<WORKER_A>>(&admitted).await?;
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
