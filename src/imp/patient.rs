//! Patient argument/result data only; no Endpoint operations or progress.
use crate::imp::json::decode;
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn file_arguments(
    file: &crate::shell::imp::file_wire::ClosedFile,
    path: &[u8],
) -> ShellResult<Value> {
    if file.completion.0.errno() != 0 || file.path != path {
        return Err(ShellError::message(
            "service file does not match the request",
        ));
    }
    let arguments = decode(&file.bytes)?;
    patient(&arguments)?;
    Ok(arguments)
}
pub fn patient(arguments: &Value) -> ShellResult<()> {
    if !arguments.is_object() || arguments["patient_id"] != "patient_1" {
        return Err(ShellError::message("expected the assigned patient"));
    }
    Ok(())
}
pub fn tests(arguments: &Value, expected: &[&str]) -> ShellResult<()> {
    patient(arguments)?;
    let tests = arguments["test_type"]
        .as_str()
        .ok_or_else(|| ShellError::message("test_type must be a string"))?;
    if tests.trim().is_empty()
        || tests
            .split(',')
            .any(|test| !expected.contains(&test.trim()))
    {
        return Err(ShellError::message("unknown or foreign test"));
    }
    Ok(())
}
pub fn actual_result(value: &Value) -> ShellResult<()> {
    if value["success"] != true
        || value["data"]["patient_id"] != "patient_1"
        || !value["data"]["artifact_id"].is_string()
    {
        return Err(ShellError::message(
            "missing successful assigned-patient result",
        ));
    }
    Ok(())
}
/// Keep the latest actual receipt per test; storage depends on the panel,
/// not the number of reconsiderations. Validate the full update before writing.
pub fn record_panel(results: &mut Value, result: &Value, expected: &[&str]) -> ShellResult<()> {
    actual_result(result)?;
    let assessment = result["data"]["inputs"]["assessment"]
        .as_str()
        .ok_or_else(|| ShellError::message("panel has no actual assessment"))?;
    let tests = result["data"]["tests"]
        .as_array()
        .filter(|tests| !tests.is_empty())
        .ok_or_else(|| ShellError::message("panel has no actual tests"))?;
    if tests
        .iter()
        .any(|test| test.as_str().is_none_or(|test| !expected.contains(&test)))
    {
        return Err(ShellError::message("unknown actual panel test"));
    }
    if !results.is_null() {
        let previous = results
            .as_object()
            .ok_or_else(|| ShellError::message("malformed actual panel"))?;
        for result in previous.values() {
            if result["data"]["inputs"]["assessment"] != assessment {
                return Err(ShellError::message(
                    "panel mixes different patient assessments",
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

/// Derive completeness from real panel receipts, not a stored progress flag.
pub fn panel_complete(results: &Value, expected: &[&str]) -> ShellResult<bool> {
    let results = results
        .as_object()
        .filter(|results| !results.is_empty())
        .ok_or_else(|| ShellError::message("missing actual panel results"))?;
    let mut assessments = results
        .values()
        .map(|result| &result["data"]["inputs"]["assessment"]);
    let assessment = assessments
        .next()
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("panel has no actual assessment"))?;
    if assessments.any(|other| other != assessment) {
        return Err(ShellError::message(
            "panel mixes different patient assessments",
        ));
    }
    for (test, result) in results {
        actual_result(result)?;
        if !expected.contains(&test.as_str())
            || !result["data"]["tests"]
                .as_array()
                .is_some_and(|tests| tests.iter().any(|name| name == test))
        {
            return Err(ShellError::message(
                "test does not match its actual receipt",
            ));
        }
    }
    Ok(expected.iter().all(|test| results.contains_key(*test)))
}

pub fn panel_arm(
    input: &std::cell::Cell<Value>,
    expected: &[&str],
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let result = panel_complete(&value, expected)
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
