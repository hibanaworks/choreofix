//! Actual request values and candidate data, without protocol progress state.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};

pub const DEFINE: u8 = 1;
pub const BACKEND: u8 = 2;
pub const FRONTEND: u8 = 3;
pub const TEST: u8 = 4;

/// Canonical arguments carried by the actual resource request.
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    fn expected() -> Value {
        match OPERATION {
            DEFINE => json!({"api_name":"feature_api_spec"}),
            BACKEND => json!({"feature_name":"backend_endpoint"}),
            FRONTEND => json!({"feature_name":"frontend_component"}),
            TEST => json!({"suite":"integration_tests"}),
            _ => panic!("unknown coordination operation"),
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        Ok(Self(
            serde_json::to_vec(&value).map_err(|error| ShellError::message(error.to_string()))?,
        ))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated coordination arguments")
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
    const SCHEMA_ID: u32 = 0xac04_0000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if value != Self::expected() {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

pub fn agreement(bytes: &[u8]) -> ShellResult<Value> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| ShellError::message("agreement must be an object"))?;
    if object.len() != 2
        || object.get("api_name").and_then(Value::as_str) != Some("feature_api_spec")
        || !matches!(
            object.get("first").and_then(Value::as_str),
            Some("BACKEND_DEV" | "FRONTEND_DEV")
        )
    {
        return Err(ShellError::message(
            "agreement must identify the API and an implementation order",
        ));
    }
    Ok(value)
}
pub fn candidate(api: &Value, backend: &Value, frontend: &Value) -> Value {
    json!({"api":api,"backend":backend,"frontend":frontend})
}
