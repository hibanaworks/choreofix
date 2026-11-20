use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}

pub fn artifact(receipt: &Value) -> ShellResult<u64> {
    if receipt["success"] != true || receipt["data"]["status"] != "complete" {
        return Err(ShellError::message("discovery effect did not complete"));
    }
    receipt["data"]["artifact_id"]
        .as_u64()
        .ok_or_else(|| ShellError::message("discovery effect has no artifact"))
}

pub fn actual(receipt: &Value, (agent, tool, arguments): (&str, &str, &Value)) -> ShellResult<()> {
    artifact(receipt)?;
    request_matches(receipt, (agent, tool, arguments))
}

pub fn request_matches(
    receipt: &Value,
    (agent, tool, arguments): (&str, &str, &Value),
) -> ShellResult<()> {
    if receipt["tool_name"] != tool
        || receipt["data"]["scientist"] != agent
        || arguments.as_object().is_none_or(|fields| {
            fields
                .iter()
                .any(|(key, value)| receipt["data"][key] != *value)
        })
    {
        return Err(ShellError::message(
            "discovery receipt differs from the accepted request",
        ));
    }
    Ok(())
}

pub fn inputs(receipt: &Value, inputs: &Value) -> ShellResult<()> {
    if receipt["data"]["inputs"] != *inputs {
        return Err(ShellError::message(
            "discovery receipt names different inputs",
        ));
    }
    Ok(())
}

pub fn compound(packet: &Value) -> ShellResult<u64> {
    let qc = artifact(&packet["qc"])?;
    let identity = &packet["identity"];
    inputs(identity, &serde_json::json!({"qc":qc}))?;
    artifact(identity)
}
