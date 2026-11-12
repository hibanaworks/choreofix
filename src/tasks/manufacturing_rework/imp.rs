//! Exact admitted inputs and actual RPC results; no custody or workflow state.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};
use std::cell::Cell;

pub const AGENTS: [&str; 3] = ["WORKER_A", "WORKER_B", "WORKER_C"];
pub const JOBS: [&str; 6] = [
    "alpha_job_a",
    "beta_job_b",
    "gamma_job_c",
    "beta_job_a",
    "gamma_job_b",
    "alpha_job_c",
];

/// Exact submitted bytes. The payload codec checks the assignment before recv commits.
#[derive(Debug)]
pub struct JobArguments<const JOB: u8>(Vec<u8>);
impl<const JOB: u8> JobArguments<JOB> {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("arguments differ from the worker's assigned job"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn value(&self) -> Value {
        serde_json::from_slice(&self.0).expect("validated job arguments")
    }
}
impl<const JOB: u8> WireEncode for JobArguments<JOB> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const JOB: u8> WirePayload for JobArguments<JOB> {
    const SCHEMA_ID: u32 = 0xac11_1000 | JOB as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if JOBS
            .get(usize::from(JOB))
            .is_none_or(|job| value != json!({"job_id":job}))
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

/// The dispatcher's actual optional reason is data, not a retry or phase flag.
pub struct DispatchArguments<const JOB: u8>(Vec<u8>);
impl<const JOB: u8> DispatchArguments<JOB> {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("dispatch arguments differ from the assigned job"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn value(&self) -> Value {
        serde_json::from_slice(&self.0).expect("validated dispatch arguments")
    }
}
impl<const JOB: u8> WireEncode for DispatchArguments<JOB> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const JOB: u8> WirePayload for DispatchArguments<JOB> {
    const SCHEMA_ID: u32 = 0xac11_2000 | JOB as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if JOBS
            .get(usize::from(JOB))
            .is_none_or(|job| value["job_id"] != *job)
            || value
                .as_object()
                .is_none_or(|args| args.keys().any(|key| key != "job_id" && key != "reason"))
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

/// A verdict for this job's real material, with success bound to the wire type.
pub struct InspectionReply<'a, const JOB: u8, const APPROVED: bool>(&'a [u8]);
impl<'a, const JOB: u8, const APPROVED: bool> InspectionReply<'a, JOB, APPROVED> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("inspection target or verdict differs"))?;
        Ok(Self(bytes))
    }
}
impl<const JOB: u8, const APPROVED: bool> WireEncode for InspectionReply<'_, JOB, APPROVED> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const JOB: u8, const APPROVED: bool> WirePayload for InspectionReply<'_, JOB, APPROVED> {
    const SCHEMA_ID: u32 = 0xac11_3000 | ((JOB as u32) << 1) | APPROVED as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let job = JOBS.get(usize::from(JOB)).ok_or(CodecError::Malformed)?;
        if value["success"] != APPROVED
            || value["data"]["job_id"] != *job
            || inspected(&value["data"]["input_part"], &json!({"job_id":job}), &value).is_err()
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}
pub fn admitted<const JOB: u8>(arguments: &JobArguments<JOB>) -> ShellResult<Value> {
    Ok(json!({"agent":AGENTS[usize::from(JOB) % AGENTS.len()], "arguments":arguments.value()}))
}
pub fn agent(admitted: &Value) -> ShellResult<&'static str> {
    AGENTS
        .into_iter()
        .find(|agent| admitted["agent"] == *agent)
        .ok_or_else(|| ShellError::message("admitted worker missing"))
}
pub fn arguments(admitted: &Value, arguments: Value) -> ShellResult<Value> {
    if arguments != admitted["arguments"] {
        return Err(ShellError::message(
            "resource request differs from admitted arguments",
        ));
    }
    Ok(arguments)
}
fn part_attempt(part: &Value, job: usize) -> ShellResult<u64> {
    let station = match job {
        0 | 5 => "STATION_ALPHA",
        1 | 3 => "STATION_BETA",
        2 | 4 => "STATION_GAMMA",
        _ => unreachable!(),
    };
    let attempt = part["attempt"].as_u64().filter(|attempt| *attempt > 0);
    if part["job_id"] != JOBS[job]
        || part["worker"] != AGENTS[job % AGENTS.len()]
        || part["station"] != station
        || attempt
            .is_none_or(|attempt| part["artifact_id"] != format!("part/{}/{attempt}", JOBS[job]))
    {
        return Err(ShellError::message(
            "part identity differs from the actual job, worker or station",
        ));
    }
    Ok(attempt.expect("checked positive attempt"))
}

fn occurrence(value: &Value, prefix: &str) -> bool {
    value.as_str().is_some_and(|value| {
        value
            .strip_prefix(prefix)
            .and_then(|number| number.parse::<u64>().ok())
            .is_some_and(|number| number > 0 && value == format!("{prefix}{number}"))
    })
}

pub fn inspection_input(part: &Value, arguments: &Value) -> ShellResult<()> {
    let job = JOBS
        .iter()
        .position(|job| part["job_id"] == *job)
        .ok_or_else(|| ShellError::message("unknown inspected part"))?;
    part_attempt(part, job)?;
    if arguments != &json!({"job_id":JOBS[job]}) {
        return Err(ShellError::message("inspection requested another material"));
    }
    Ok(())
}

/// Match the actual inspection reply to the requested, completed material.
pub fn inspected(part: &Value, arguments: &Value, result: &Value) -> ShellResult<()> {
    inspection_input(part, arguments)?;
    let status = match result["success"].as_bool() {
        Some(true) => "inspected",
        Some(false) => "failed_inspection",
        None => return Err(ShellError::message("inspection has no actual verdict")),
    };
    if result["tool_name"] != "inspect_quality"
        || result["data"]["status"] != status
        || result["data"]["job_id"] != part["job_id"]
        || result["data"]["inspector"] != "INSPECTOR"
        || result["data"]["input_part"] != *part
        || !occurrence(&result["data"]["receipt"], "inspection/")
    {
        return Err(ShellError::message(
            "inspection differs from the actual requested part or verdict",
        ));
    }
    Ok(())
}

pub fn rework_input(inspection: &Value, arguments: &Value) -> ShellResult<()> {
    let part = &inspection["data"]["input_part"];
    let job = JOBS
        .iter()
        .position(|job| part["job_id"] == *job)
        .ok_or_else(|| ShellError::message("unknown rework part"))?;
    inspected(part, &json!({"job_id":JOBS[job]}), inspection)?;
    if inspection["success"] != false
        || arguments["job_id"] != JOBS[job]
        || arguments
            .as_object()
            .is_none_or(|args| args.keys().any(|key| key != "job_id" && key != "reason"))
    {
        return Err(ShellError::message(
            "rework requires the failed inspection of this job",
        ));
    }
    Ok(())
}

/// Match a real dispatch reply to its failed inspection and admitted arguments.
pub fn dispatched(inspection: &Value, arguments: &Value, result: &Value) -> ShellResult<()> {
    rework_input(inspection, arguments)?;
    let part = &inspection["data"]["input_part"];
    if result["tool_name"] != "dispatch_rework"
        || result["success"] != true
        || result["data"]["status"] != "rework_dispatched"
        || result["data"]["job_id"] != part["job_id"]
        || result["data"]["worker"] != part["worker"]
        || result["data"]["input_part"] != *part
        || result["data"]["inspection"] != inspection["data"]["receipt"]
        || result["data"]["reason"] != arguments.get("reason").cloned().unwrap_or(json!(""))
        || !occurrence(&result["data"]["order"], "rework/")
    {
        return Err(ShellError::message(
            "rework order differs from the failed inspection or admitted arguments",
        ));
    }
    Ok(())
}

pub fn completed(
    admitted: &Value,
    result: &Value,
    previous_rework: Option<(&Value, &Value, &Value)>,
) -> ShellResult<()> {
    let job = JOBS
        .iter()
        .position(|job| admitted["arguments"]["job_id"] == *job)
        .ok_or_else(|| ShellError::message("unknown admitted job"))?;
    let previous_attempt = match previous_rework {
        None => {
            if result["data"].get("rework_order") != Some(&Value::Null) {
                return Err(ShellError::message(
                    "initial part claims an unrelated work order",
                ));
            }
            0
        }
        Some((inspection, arguments, order)) => {
            dispatched(inspection, arguments, order)?;
            if result["data"]["rework_order"] != order["data"]["order"] {
                return Err(ShellError::message(
                    "replacement did not use its actual work order",
                ));
            }
            part_attempt(&order["data"]["input_part"], job)?
        }
    };
    let next = previous_attempt
        .checked_add(1)
        .ok_or_else(|| ShellError::message("part attempt exhausted"))?;
    if result["tool_name"] != "do_job"
        || result["success"] != true
        || result["data"]["status"] != "completed"
        || admitted["agent"] != AGENTS[job % AGENTS.len()]
        || part_attempt(&result["data"]["part"], job)? != next
    {
        return Err(ShellError::message(
            "tool result differs from the actual admitted job or prior part",
        ));
    }
    Ok(())
}

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}
pub fn worker_a(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match value["agent"].as_str() {
        Some("WORKER_A") => Ok(DecisionArm::Left),
        Some("WORKER_B" | "WORKER_C") => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}

pub fn initial_quality(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match value["success"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}

pub fn worker_b(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match value["agent"].as_str() {
        Some("WORKER_B") => Ok(DecisionArm::Left),
        Some("WORKER_C") => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}
pub fn next(input: &(&Cell<Value>, &Cell<Value>)) -> Result<DecisionArm, ResolverError> {
    let first = input.0.take();
    let second = input.1.take();
    let result = match (first["agent"].as_str(), second["agent"].as_str()) {
        (Some("WORKER_A"), Some("WORKER_B"))
        | (Some("WORKER_B"), Some("WORKER_A"))
        | (Some("WORKER_C"), Some("WORKER_A")) => Ok(DecisionArm::Left),
        (Some("WORKER_A" | "WORKER_B"), Some("WORKER_C"))
        | (Some("WORKER_C"), Some("WORKER_B")) => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    input.0.set(first);
    input.1.set(second);
    result
}
pub fn arrival(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = if value["tool_name"] == "do_job" {
        Ok(DecisionArm::Left)
    } else if AGENTS.into_iter().any(|agent| value["agent"] == agent)
        && value["arguments"].is_object()
    {
        Ok(DecisionArm::Right)
    } else {
        Err(ResolverError::reject())
    };
    input.set(value);
    result
}
pub fn finished(input: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match (
        value["resource"].as_u64(),
        value["result"]["tool_name"].as_str(),
    ) {
        (Some(4), Some("do_job")) => Ok(DecisionArm::Left),
        (Some(5), Some("do_job")) => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quality_wire<const JOB: u8>() {
        for (target_index, target) in JOBS.into_iter().enumerate() {
            for reason in [
                None,
                Some(json!("requested repair")),
                Some(Value::Null),
                Some(json!({"measured":"defect"})),
            ] {
                let mut arguments = json!({"job_id":target});
                if let Some(reason) = reason {
                    arguments["reason"] = reason;
                }
                let encoded = serde_json::to_vec(&arguments).unwrap();
                assert_eq!(
                    DispatchArguments::<JOB>::from_bytes(&encoded).is_ok(),
                    usize::from(JOB) == target_index
                );
            }
            for success in [false, true] {
                let worker = AGENTS[target_index % AGENTS.len()];
                let station = ["STATION_ALPHA", "STATION_BETA", "STATION_GAMMA"]
                    [[0, 1, 2, 1, 2, 0][target_index]];
                let result = json!({"tool_name":"inspect_quality","success":success,"data":{
                    "job_id":target,"input_part":{"artifact_id":format!("part/{target}/1"),"job_id":target,
                    "worker":worker,"station":station,"attempt":1},"receipt":"inspection/1","inspector":"INSPECTOR",
                    "status":if success{"inspected"}else{"failed_inspection"}}});
                let encoded = serde_json::to_vec(&result).unwrap();
                assert_eq!(
                    InspectionReply::<JOB, true>::from_bytes(&encoded).is_ok(),
                    usize::from(JOB) == target_index && success
                );
                assert_eq!(
                    InspectionReply::<JOB, false>::from_bytes(&encoded).is_ok(),
                    usize::from(JOB) == target_index && !success
                );
            }
        }
        let unknown = json!({"job_id":"unknown_job"});
        assert!(
            DispatchArguments::<JOB>::from_bytes(&serde_json::to_vec(&unknown).unwrap()).is_err()
        );
        let extra = json!({"job_id":JOBS[usize::from(JOB)],"approval":true});
        assert!(
            DispatchArguments::<JOB>::from_bytes(&serde_json::to_vec(&extra).unwrap()).is_err()
        );
    }

    #[test]
    fn quality_wire_contracts_keep_the_target_reason_and_actual_outcome() {
        quality_wire::<0>();
        quality_wire::<1>();
        quality_wire::<2>();
        quality_wire::<3>();
        quality_wire::<4>();
        quality_wire::<5>();
        assert!(DispatchArguments::<6>::from_bytes(br#"{"job_id":"alpha_job_a"}"#).is_err());
        assert!(InspectionReply::<6, true>::from_bytes(br#"{}"#).is_err());
    }

    fn job_payload<const JOB: u8>() {
        let expected = json!({"job_id":JOBS[usize::from(JOB)]});
        let encoded = serde_json::to_vec(&expected).unwrap();
        let arguments = JobArguments::<JOB>::from_bytes(&encoded).unwrap();
        assert_eq!(arguments.value(), expected);
        assert_eq!(
            admitted(&arguments).unwrap()["agent"],
            AGENTS[usize::from(JOB) % AGENTS.len()]
        );
        for (other, job) in JOBS.into_iter().enumerate() {
            let bytes = serde_json::to_vec(&json!({"job_id":job})).unwrap();
            assert_eq!(
                JobArguments::<JOB>::validate_payload(Payload::new(&bytes)).is_ok(),
                other == usize::from(JOB)
            );
        }
    }

    #[test]
    fn all_six_job_contracts_keep_their_exact_target_and_worker() {
        job_payload::<0>();
        job_payload::<1>();
        job_payload::<2>();
        job_payload::<3>();
        job_payload::<4>();
        job_payload::<5>();
        assert!(JobArguments::<6>::from_bytes(br#"{"job_id":"alpha_job_a"}"#).is_err());
    }

    #[test]
    fn actual_part_results_bind_job_worker_station_and_material_occurrence() {
        for (job, target) in JOBS.into_iter().enumerate() {
            let worker = AGENTS[job % AGENTS.len()];
            let station =
                ["STATION_ALPHA", "STATION_BETA", "STATION_GAMMA"][[0, 1, 2, 1, 2, 0][job]];
            let admitted = json!({"agent":worker,"arguments":{"job_id":target}});
            let initial = json!({"tool_name":"do_job","success":true,"data":{
                "status":"completed","rework_order":null,"part":{
                    "artifact_id":format!("part/{target}/1"),"job_id":target,
                    "worker":worker,"station":station,"attempt":1}}});
            completed(&admitted, &initial, None).unwrap();
            let prior = &initial["data"]["part"];
            let inspection = json!({"tool_name":"inspect_quality","success":false,"data":{
                "job_id":target,"input_part":prior,"receipt":"inspection/1",
                "inspector":"INSPECTOR","status":"failed_inspection"}});
            let dispatch_arguments = json!({"job_id":target});
            let order = json!({"tool_name":"dispatch_rework","success":true,"data":{
                "job_id":target,"worker":worker,"input_part":prior,"inspection":"inspection/1",
                "order":"rework/1","status":"rework_dispatched","reason":""}});
            let previous = Some((&inspection, &dispatch_arguments, &order));
            let mut result = initial.clone();
            result["data"]["part"]["artifact_id"] = json!(format!("part/{target}/2"));
            result["data"]["part"]["attempt"] = json!(2);
            result["data"]["rework_order"] = json!("rework/1");
            completed(&admitted, &result, previous).unwrap();
            assert!(completed(&admitted, &result, None).is_err());
            assert!(completed(&admitted, &initial, previous).is_err());
            let mut changed_order = order.clone();
            changed_order["data"]["input_part"]["job_id"] = json!(JOBS[(job + 1) % JOBS.len()]);
            assert!(
                completed(
                    &admitted,
                    &result,
                    Some((&inspection, &dispatch_arguments, &changed_order))
                )
                .is_err()
            );
            for (path, changed) in [
                ("/tool_name", json!("inspect_quality")),
                ("/success", json!(false)),
                ("/data/status", json!("pending")),
                ("/data/part/job_id", json!(JOBS[(job + 1) % JOBS.len()])),
                ("/data/part/worker", json!(AGENTS[(job + 1) % AGENTS.len()])),
                ("/data/part/station", json!("STATION_OTHER")),
                ("/data/part/attempt", json!(0)),
                ("/data/part/artifact_id", json!(format!("part/{target}/1"))),
            ] {
                let mut altered = result.clone();
                *altered.pointer_mut(path).unwrap() = changed;
                assert!(
                    completed(&admitted, &altered, previous).is_err(),
                    "{target} {path}"
                );
            }
        }
    }

    #[test]
    fn inspection_and_rework_receipts_bind_the_same_material_and_actual_verdict() {
        for (job, target) in JOBS.into_iter().enumerate() {
            for attempt in 1..=3u64 {
                let worker = AGENTS[job % AGENTS.len()];
                let station =
                    ["STATION_ALPHA", "STATION_BETA", "STATION_GAMMA"][[0, 1, 2, 1, 2, 0][job]];
                let part = json!({"artifact_id":format!("part/{target}/{attempt}"),
                    "job_id":target,"worker":worker,"station":station,"attempt":attempt});
                let inspect_arguments = json!({"job_id":target});
                let inspection = json!({"tool_name":"inspect_quality","success":false,"data":{
                    "job_id":target,"input_part":part,"receipt":format!("inspection/{attempt}"),
                    "inspector":"INSPECTOR","status":"failed_inspection"}});
                inspected(&part, &inspect_arguments, &inspection).unwrap();
                let arguments = json!({"job_id":target,"reason":"repair the measured defect"});
                let order = json!({"tool_name":"dispatch_rework","success":true,"data":{
                    "job_id":target,"worker":worker,"input_part":part,
                    "inspection":format!("inspection/{attempt}"),"order":format!("rework/{attempt}"),
                    "status":"rework_dispatched","reason":arguments["reason"]}});
                dispatched(&inspection, &arguments, &order).unwrap();
                let admitted = json!({"agent":worker,"arguments":{"job_id":target}});
                let replacement = json!({"tool_name":"do_job","success":true,"data":{
                    "status":"completed","rework_order":order["data"]["order"],"part":{
                        "artifact_id":format!("part/{target}/{}",attempt+1),"job_id":target,
                        "worker":worker,"station":station,"attempt":attempt+1}}});
                completed(
                    &admitted,
                    &replacement,
                    Some((&inspection, &arguments, &order)),
                )
                .unwrap();

                let mut positive = inspection.clone();
                positive["success"] = json!(true);
                positive["data"]["status"] = json!("inspected");
                inspected(&part, &inspect_arguments, &positive).unwrap();
                assert!(dispatched(&positive, &arguments, &order).is_err());
                assert!(
                    completed(
                        &admitted,
                        &replacement,
                        Some((&positive, &arguments, &order))
                    )
                    .is_err()
                );

                for (path, value) in [
                    ("/tool_name", json!("dispatch_rework")),
                    ("/success", json!(true)),
                    ("/data/status", json!("rework_required")),
                    ("/data/job_id", json!(JOBS[(job + 1) % JOBS.len()])),
                    ("/data/inspector", json!(worker)),
                    (
                        "/data/input_part/artifact_id",
                        json!(format!("part/{target}/{}", attempt + 1)),
                    ),
                    ("/data/receipt", json!("inspection/0")),
                    ("/data/receipt", json!("inspection/01")),
                ] {
                    let mut changed = inspection.clone();
                    *changed.pointer_mut(path).unwrap() = value;
                    assert!(
                        inspected(&part, &inspect_arguments, &changed).is_err(),
                        "{target} {attempt} {path}"
                    );
                    assert!(
                        dispatched(&changed, &arguments, &order).is_err(),
                        "{target} {attempt} {path}"
                    );
                }
                for (path, value) in [
                    ("/tool_name", json!("inspect_quality")),
                    ("/success", json!(false)),
                    ("/data/status", json!("pending")),
                    ("/data/job_id", json!(JOBS[(job + 1) % JOBS.len()])),
                    ("/data/worker", json!(AGENTS[(job + 1) % AGENTS.len()])),
                    ("/data/input_part/attempt", json!(attempt + 1)),
                    (
                        "/data/inspection",
                        json!(format!("inspection/{}", attempt + 1)),
                    ),
                    ("/data/order", json!("rework/0")),
                    ("/data/order", json!("rework/01")),
                    ("/data/reason", json!("unadmitted reason")),
                ] {
                    let mut changed = order.clone();
                    *changed.pointer_mut(path).unwrap() = value;
                    assert!(
                        dispatched(&inspection, &arguments, &changed).is_err(),
                        "{target} {attempt} {path}"
                    );
                    assert!(
                        completed(
                            &admitted,
                            &replacement,
                            Some((&inspection, &arguments, &changed))
                        )
                        .is_err(),
                        "{target} {attempt} {path}"
                    );
                }
                let mut stale = replacement.clone();
                stale["data"]["part"] = part.clone();
                assert!(
                    completed(&admitted, &stale, Some((&inspection, &arguments, &order))).is_err()
                );
                stale = replacement.clone();
                stale["data"]["rework_order"] = json!(format!("rework/{}", attempt + 1));
                assert!(
                    completed(&admitted, &stale, Some((&inspection, &arguments, &order))).is_err()
                );
                let mut changed_arguments = arguments.clone();
                changed_arguments["reason"] = json!("different admitted request");
                assert!(dispatched(&inspection, &changed_arguments, &order).is_err());
            }
        }
        assert!(inspected(&Value::Null, &json!({}), &Value::Null).is_err());
        assert!(dispatched(&Value::Null, &json!({}), &Value::Null).is_err());
    }
}
