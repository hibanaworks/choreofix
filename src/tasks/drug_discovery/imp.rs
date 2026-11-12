//! Codecs and validation of real argument/result data; no Endpoint operations.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn assigned(value: &Value, key: &str, expected: &str) -> ShellResult<()> {
    if value.as_object().is_none_or(|v| v.len() != 1) || value[key].as_str() != Some(expected) {
        return Err(ShellError::message(
            "instrument arguments differ from the assigned sample or assay",
        ));
    }
    Ok(())
}
pub fn same(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        return Err(ShellError::message(
            "instrument request differs from admitted real data",
        ));
    }
    Ok(())
}
pub fn successful(value: &Value) -> ShellResult<()> {
    if value["success"] != true {
        return Err(ShellError::message("terminal instrument operation failed"));
    }
    Ok(())
}
pub fn decision(file: &crate::shell::imp::file_wire::ClosedFile) -> ShellResult<Value> {
    if file.path != b"service/submit_decision" || file.completion.0.errno() != 0 {
        return Err(ShellError::message("expected the actual decision file"));
    }
    let value = decode(&file.bytes)?;
    if value.as_object().is_none_or(|v| v.len() != 1) || !value["go"].is_boolean() {
        return Err(ShellError::message(
            "expected one boolean go/no-go decision",
        ));
    }
    Ok(value)
}

pub fn decision_receipt(decision: &Value, receipt: &Value) -> ShellResult<()> {
    successful(receipt)?;
    same(&receipt["data"]["go"], &decision["go"])?;
    for (key, artifact) in [
        ("compound", &decision["compound"]["identity"]),
        ("bioassay", &decision["bio"]["assay"]),
        ("stability", &decision["bio"]["stability"]),
        ("toxicity", &decision["tox"]["cytotoxicity"]),
        ("structural", &decision["analyses"]["structural"]),
        (
            "characterization",
            &decision["analyses"]["characterization"],
        ),
    ] {
        let identity = &artifact["data"]["artifact_id"];
        if !identity.is_u64() {
            return Err(ShellError::message(
                "analysis has no actual artifact receipt",
            ));
        }
        same(&receipt["data"]["inputs"][key], identity)?;
    }
    Ok(())
}
