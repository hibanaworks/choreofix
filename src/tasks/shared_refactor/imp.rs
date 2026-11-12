//! Argument and artifact values only; these functions perform no communication.
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn arguments(bytes: &[u8], key: &str, expected: &str) -> ShellResult<Value> {
    let v = decode(bytes)?;
    if v.as_object().is_none_or(|o| o.len() != 1) || v[key].as_str() != Some(expected) {
        return Err(ShellError::message(
            "shared-refactor arguments differ from the assigned object",
        ));
    }
    Ok(v)
}
pub fn same(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        Err(ShellError::message(
            "shared-refactor received values differ from the admitted data",
        ))
    } else {
        Ok(())
    }
}
pub fn successful(value: &Value) -> ShellResult<()> {
    if value["success"].as_bool() != Some(true) {
        Err(ShellError::message(
            "terminal original 4M operation failed; no merge permitted",
        ))
    } else {
        Ok(())
    }
}
pub fn candidate(previous: &Value, first: &Value, second: &Value) -> Value {
    json!({"previous":previous,"changes":[first,second]})
}
