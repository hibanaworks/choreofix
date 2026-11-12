//! JSON value encoding and exact admitted-value comparison.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn same(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        return Err(ShellError::message(
            "request differs from its actual admitted data",
        ));
    }
    Ok(())
}
