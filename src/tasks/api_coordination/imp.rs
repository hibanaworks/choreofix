//! Actual JSON input and candidate data; no task progress or ownership state.
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};

pub fn arguments(bytes: &[u8], key: &str, expected: &str) -> ShellResult<Value> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| ShellError::message("API coordination arguments must be an object"))?;
    if object.len() != 1 || object.get(key).and_then(Value::as_str) != Some(expected) {
        return Err(ShellError::message(
            "API coordination arguments differ from the assignment",
        ));
    }
    Ok(value)
}

pub fn candidate(api: &Value, backend: &Value, frontend: &Value) -> Value {
    json!({"api":api,"backend":backend,"frontend":frontend})
}
