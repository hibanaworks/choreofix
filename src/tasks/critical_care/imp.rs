//! Patient/result codecs and actual data validation; no protocol operations.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;
use std::cell::Cell;

pub use crate::imp::json::{decode, encode, same};

pub fn file_arguments(
    file: &crate::shell::imp::file_wire::ClosedFile,
    assigned: (&[u8], &str),
) -> ShellResult<Value> {
    if file.completion.0.errno() != 0 || file.path != assigned.0 {
        return Err(ShellError::message(
            "file close does not match the requested service",
        ));
    }
    let arguments = decode(&file.bytes)?;
    patient(&arguments, assigned.1)?;
    Ok(arguments)
}

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
