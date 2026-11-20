//! Exact request data and validation of real job/report results.
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};
pub const JOBS: [&str; 4] = ["lathe_job_a", "lathe_job_b", "mill_job_a", "mill_job_b"];

#[derive(Debug)]
pub struct Arguments<const OPERATION: u8, const SUBMISSION: bool = false>(Vec<u8>);

impl<const OPERATION: u8> SimulatorOperation for Arguments<OPERATION> {
    const RPC: (&'static str, &'static str) = match OPERATION {
        0 | 2 => ("TECHNICIAN_A", "do_job"),
        1 | 3 => ("TECHNICIAN_B", "do_job"),
        4 => ("SUPERVISOR", "collect_reports"),
        _ => panic!("unknown manufacturing operation"),
    };
}

impl<const OPERATION: u8> Arguments<OPERATION> {
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("arguments differ from this operation"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated manufacturing arguments")
    }
}
impl<const OPERATION: u8, const SUBMISSION: bool> WireEncode for Arguments<OPERATION, SUBMISSION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac11_0000 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let valid = match OPERATION {
            0..=3 => value == json!({"job_id":JOBS[usize::from(OPERATION)]}),
            4 => value == json!({}),
            _ => false,
        };
        if !valid {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

// A file submission supplies the job to send on the machine Endpoint.
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION, true> {
    const SCHEMA_ID: u32 = 0xac11_0100 | OPERATION as u32;
    type Decoded<'a> = Arguments<OPERATION>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<OPERATION>::validate_payload(input)
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        Arguments(input.as_bytes().to_vec())
    }
}
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn completed(result: &Value, agent: &str, machine: &str, arguments: &Value) -> ShellResult<()> {
    if result["tool_name"] != "do_job"
        || result["success"] != true
        || result["data"]
            != json!({"job_id":arguments["job_id"],"agent":agent,"machine":machine,"status":"completed"})
    {
        return Err(ShellError::message(
            "machine result differs from the actual assigned job",
        ));
    }
    Ok(())
}
pub fn completed_jobs(receipts: &[Value]) -> ShellResult<Vec<Value>> {
    if receipts.len() != JOBS.len() {
        return Err(ShellError::message("four actual job receipts required"));
    }
    let mut expected = Vec::new();
    for (index, id) in JOBS.iter().enumerate() {
        let agent = if index.is_multiple_of(2) {
            "TECHNICIAN_A"
        } else {
            "TECHNICIAN_B"
        };
        let machine = if index < 2 { "LATHE" } else { "MILL" };
        let matched = receipts
            .iter()
            .filter(|job| job["data"]["job_id"] == *id)
            .collect::<Vec<_>>();
        let [job] = matched.as_slice() else {
            return Err(ShellError::message(
                "missing or duplicated actual job receipt",
            ));
        };
        completed(job, agent, machine, &json!({"job_id":id}))?;
        expected.push(job["data"].clone());
    }
    expected.sort_by_key(|job| job["job_id"].as_str().expect("checked job id").to_owned());
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
    if received.iter().any(|job| !job["job_id"].is_string()) {
        return Err(ShellError::message("report job id missing"));
    }
    received.sort_by_key(|job| {
        job["job_id"]
            .as_str()
            .expect("checked report job id")
            .to_owned()
    });
    if received != expected {
        return Err(ShellError::message(
            "supervisor report differs from the four actual results",
        ));
    }
    Ok(())
}
