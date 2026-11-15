//! Patient/result codecs and actual data validation; no protocol operations.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;
use std::cell::Cell;

pub use crate::imp::json::{decode, encode, same};

pub fn patient(arguments: &Value, expected: &str) -> ShellResult<()> {
    if !arguments.is_object() || arguments["patient_id"] != expected {
        return Err(ShellError::message("request names another patient"));
    }
    Ok(())
}
pub fn optional_text(arguments: &Value, field: &str) -> ShellResult<()> {
    if arguments.get(field).is_some_and(|value| !value.is_string()) {
        return Err(ShellError::message(format!("{field} must be a string")));
    }
    Ok(())
}
pub fn imaging(arguments: &Value, assigned: (&str, &str)) -> ShellResult<()> {
    patient(arguments, assigned.0)?;
    if arguments["modality"]
        .as_str()
        .is_none_or(|value| !value.eq_ignore_ascii_case(assigned.1))
    {
        return Err(ShellError::message(
            "request names another imaging modality",
        ));
    }
    Ok(())
}
pub fn actual_result(value: &Value, expected: &str) -> ShellResult<()> {
    if value["success"] != true
        || value["data"]["patient_id"] != expected
        || !value["data"]["artifact_id"].is_string()
    {
        return Err(ShellError::message(
            "missing successful actual patient result",
        ));
    }
    Ok(())
}
pub fn tests(arguments: &Value, assigned: (&str, &[&str])) -> ShellResult<()> {
    patient(arguments, assigned.0)?;
    let tests = arguments["test_type"]
        .as_str()
        .filter(|tests| !tests.trim().is_empty())
        .ok_or_else(|| ShellError::message("test_type must name actual tests"))?;
    if tests
        .split(',')
        .any(|test| !assigned.1.contains(&test.trim()))
    {
        return Err(ShellError::message(
            "request names unknown or foreign tests",
        ));
    }
    Ok(())
}

/// Validate the complete update before replacing any bounded per-test receipt.
pub fn record_panel(
    results: &mut Value,
    result: &Value,
    assigned: (&str, &[&str]),
) -> ShellResult<()> {
    actual_result(result, assigned.0)?;
    let assessment = result["data"]["inputs"]["assessment"]
        .as_str()
        .ok_or_else(|| ShellError::message("test result lacks its actual assessment"))?;
    let tests = result["data"]["tests"]
        .as_array()
        .filter(|tests| !tests.is_empty())
        .ok_or_else(|| ShellError::message("test result contains no actual tests"))?;
    if tests
        .iter()
        .any(|test| test.as_str().is_none_or(|name| !assigned.1.contains(&name)))
    {
        return Err(ShellError::message("result names foreign tests"));
    }
    if !results.is_null() {
        let previous = results
            .as_object()
            .ok_or_else(|| ShellError::message("malformed actual panel"))?;
        for receipt in previous.values() {
            actual_result(receipt, assigned.0)?;
            if receipt["data"]["inputs"]["assessment"] != assessment {
                return Err(ShellError::message(
                    "panel mixes patient assessment occurrences",
                ));
            }
        }
    }
    if results.is_null() {
        *results = Value::Object(serde_json::Map::new());
    }
    let panel = results.as_object_mut().expect("validated panel object");
    for test in tests {
        panel.insert(
            test.as_str().expect("validated test name").to_owned(),
            result.clone(),
        );
    }
    Ok(())
}
pub fn panel_complete(results: &Value, assigned: (&str, &[&str])) -> ShellResult<bool> {
    let receipts = results
        .as_object()
        .filter(|receipts| !receipts.is_empty())
        .ok_or_else(|| ShellError::message("missing actual test panel"))?;
    let assessment = receipts
        .values()
        .next()
        .and_then(|r| r["data"]["inputs"]["assessment"].as_str())
        .ok_or_else(|| ShellError::message("panel has no actual assessment"))?;
    for (test, receipt) in receipts {
        actual_result(receipt, assigned.0)?;
        if !assigned.1.contains(&test.as_str())
            || receipt["data"]["inputs"]["assessment"] != assessment
            || !receipt["data"]["tests"]
                .as_array()
                .is_some_and(|tests| tests.iter().any(|name| name == test))
        {
            return Err(ShellError::message(
                "panel has a foreign or stale actual test",
            ));
        }
    }
    Ok(assigned.1.iter().all(|test| receipts.contains_key(*test)))
}
pub fn panel_arm(
    input: &Cell<Value>,
    assigned: (&str, &[&str]),
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let result = panel_complete(&value, assigned)
        .map(|complete| {
            if complete {
                DecisionArm::Left
            } else {
                DecisionArm::Right
            }
        })
        .map_err(|_| ResolverError::reject());
    input.set(value);
    result
}

/// Own the actual request bytes; the contract selects patient and argument shape.
#[derive(Debug)]
pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("invalid patient operation arguments"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated patient operation")
    }
}
impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac05_2000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let arguments: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let assigned = match OPERATION {
            0 | 2 | 4 | 5 | 10 | 11 | 14 => "patient_a",
            1 | 3 | 6 | 7 | 8 | 9 | 12 | 15 => "patient_b",
            13 => "patient_A_and_B",
            _ => return Err(CodecError::Malformed),
        };
        patient(&arguments, assigned).map_err(|_| CodecError::Malformed)?;
        let valid = match OPERATION {
            0 | 1 | 12 => true,
            2 => tests(
                &arguments,
                (assigned, &["trauma_panel", "blood_type", "coag"]),
            )
            .is_ok(),
            3 => tests(&arguments, (assigned, &["stroke_panel", "ct_angio"])).is_ok(),
            4..=7 => imaging(&arguments, (assigned, "CT")).is_ok(),
            8 | 9 => imaging(&arguments, (assigned, "MRI")).is_ok(),
            10 => optional_text(&arguments, "type").is_ok(),
            11 => arguments["procedure"]
                .as_str()
                .is_some_and(|s| s == "trauma_surgery"),
            13 => optional_text(&arguments, "medications").is_ok(),
            14 | 15 => optional_text(&arguments, "symptoms").is_ok(),
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(CodecError::Malformed)
        }
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
