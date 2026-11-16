//! Inspection input is ordinary optional, freely chosen notes.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct InspectionArguments(Vec<u8>);

impl InspectionArguments {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated inspection arguments")
    }
}

impl TryFrom<&[u8]> for InspectionArguments {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl WireEncode for InspectionArguments {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl WirePayload for InspectionArguments {
    const SCHEMA_ID: u32 = 0xac13_2010;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if object.keys().any(|key| key != "notes")
            || object.get("notes").is_some_and(|notes| !notes.is_string())
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
