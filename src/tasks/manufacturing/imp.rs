//! Original job identifiers, exact request bytes and actual result validation.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};

pub const JOBS: [&str; 4] = ["lathe_job_a", "lathe_job_b", "mill_job_a", "mill_job_b"];

/// Owns the exact JSON bytes of one assigned job; it stores no protocol state.
#[derive(Debug)]
pub struct JobArguments<const JOB: u8>(Vec<u8>);
impl<const JOB: u8> JobArguments<JOB> {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("arguments differ from the assigned machine job"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated manufacturing request")
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
    const SCHEMA_ID: u32 = 0xac11_0000 | JOB as u32;
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
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
/// Input is the actual received request and its sender, never a custody flag.
pub fn first_request(
    input: &core::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let arm = match value["agent"].as_str() {
        Some("TECHNICIAN_A") => Ok(DecisionArm::Left),
        Some("TECHNICIAN_B") => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    input.set(value);
    arm
}
/// Distinguishes the actual completed RPC from the other received request.
pub fn arrival(
    input: &core::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let arm = if value["tool_name"] == "do_job" {
        Ok(DecisionArm::Left)
    } else if matches!(
        value["agent"].as_str(),
        Some("TECHNICIAN_A" | "TECHNICIAN_B")
    ) && value["arguments"].is_object()
    {
        Ok(DecisionArm::Right)
    } else {
        Err(ResolverError::reject())
    };
    input.set(value);
    arm
}
pub fn completed(result: &Value, agent: &str, machine: &str, arguments: &Value) -> ShellResult<()> {
    if result["tool_name"] != "do_job"
        || result["success"] != true
        || result["data"]
            != json!({"job_id":arguments["job_id"],"agent":agent,
            "machine":machine,"status":"completed"})
    {
        return Err(ShellError::message(
            "machine result differs from the actual assigned job",
        ));
    }
    Ok(())
}
pub fn report_arguments(value: Value) -> ShellResult<Value> {
    if value != json!({}) {
        return Err(ShellError::message(
            "collect_reports takes an empty argument object",
        ));
    }
    Ok(value)
}
pub fn completed_jobs(receipts: &Value) -> ShellResult<Vec<Value>> {
    let jobs = receipts
        .as_array()
        .ok_or_else(|| ShellError::message("job receipts missing"))?;
    let mut expected = Vec::new();
    for (index, id) in JOBS.iter().enumerate() {
        let agent = if index.is_multiple_of(2) {
            "TECHNICIAN_A"
        } else {
            "TECHNICIAN_B"
        };
        let machine = if index < 2 { "LATHE" } else { "MILL" };
        let matched: Vec<_> = jobs
            .iter()
            .filter(|job| job["data"]["job_id"] == *id)
            .collect();
        let [job] = matched.as_slice() else {
            return Err(ShellError::message(
                "missing or duplicated actual job receipt",
            ));
        };
        completed(job, agent, machine, &json!({"job_id":id}))?;
        expected.push(job["data"].clone());
    }
    if jobs.len() != 4 {
        return Err(ShellError::message("extra actual job receipt"));
    }
    expected.sort_by_key(|job| job["job_id"].as_str().unwrap().to_owned());
    Ok(expected)
}
pub fn report_result(expected: &[Value], actual: &Value) -> ShellResult<()> {
    if actual["tool_name"] != "collect_reports"
        || actual["success"] != true
        || actual["data"]["agent"] != "SUPERVISOR"
        || actual["data"]["scope"] != "all_jobs"
        || actual["data"]["status"] != "collected"
    {
        return Err(ShellError::message(
            "supervisor report is not the completed four-job report",
        ));
    }
    let mut received = actual["data"]["jobs"]
        .as_array()
        .cloned()
        .ok_or_else(|| ShellError::message("supervisor report contents missing"))?;
    received.sort_by_key(|job| job["job_id"].as_str().unwrap_or("").to_owned());
    if expected != received {
        return Err(ShellError::message(
            "supervisor report differs from the four actual results",
        ));
    }
    Ok(())
}
