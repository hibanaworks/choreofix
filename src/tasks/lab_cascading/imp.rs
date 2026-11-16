//! JSON values only; resource authority stays with the projected Endpoints.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}

/// Validate actual synthesis data, including ordinary failure/exhaustion.
pub fn synthesis_result(result: &Value, arguments: &Value) -> ShellResult<()> {
    let success = result["success"]
        .as_bool()
        .ok_or_else(|| ShellError::message("synthesis has no actual verdict"))?;
    let status = result["data"]["status"].as_str();
    let remaining = result["data"]["reagent_units_remaining"].as_u64();
    if result["tool_name"] != "run_synthesis"
        || result["data"]["scientist"] != "CHEMIST"
        || result["data"]["compound"] != arguments["compound"]
        || result["data"]["inputs"] != serde_json::json!({})
        || result["data"]["missing_inputs"] != serde_json::json!([])
        || remaining.is_none()
        || if success {
            status != Some("complete") || result["data"]["artifact_id"].as_u64().is_none()
        } else {
            !matches!(status, Some("synthesis_failed" | "reagent_exhausted"))
                || !result["data"]["artifact_id"].is_null()
                || (status == Some("reagent_exhausted") && remaining != Some(0))
        }
    {
        return Err(ShellError::message(
            "synthesis result differs from the actual request or material outcome",
        ));
    }
    Ok(())
}

/// The actual terminal result, not a completion flag or a new stock authority.
pub struct SynthesisTerminal<const SUCCESS: bool>(Vec<u8>);

impl<const SUCCESS: bool> SynthesisTerminal<SUCCESS> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated synthesis terminal result")
    }
}

impl<const SUCCESS: bool> TryFrom<&[u8]> for SynthesisTerminal<SUCCESS> {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const SUCCESS: bool> WireEncode for SynthesisTerminal<SUCCESS> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const SUCCESS: bool> WirePayload for SynthesisTerminal<SUCCESS> {
    const SCHEMA_ID: u32 = 0xac13_2100 | SUCCESS as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let result: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let arguments = serde_json::json!({"compound":result["data"]["compound"]});
        super::arguments::Arguments::<0>::try_from(
            encode(&arguments)
                .map_err(|_| CodecError::Malformed)?
                .as_slice(),
        )?;
        synthesis_result(&result, &arguments).map_err(|_| CodecError::Malformed)?;
        if result["success"] != SUCCESS
            || (!SUCCESS && result["data"]["status"] != "reagent_exhausted")
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
