//! Exact request data. Conversion never grants an operation.
use super::{global::INTEGRATOR, imp};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const ROLE: u8>(Vec<u8>);
impl<const ROLE: u8> Arguments<ROLE> {
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> serde_json::Value {
        serde_json::from_slice(&self.0).expect("validated cascading build arguments")
    }
}
impl<const ROLE: u8> WireEncode for Arguments<ROLE> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const ROLE: u8> WirePayload for Arguments<ROLE> {
    const SCHEMA_ID: u32 = 0x4342_0000 | ROLE as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if ROLE == INTEGRATOR {
            imp::link_arguments(value)
        } else if ROLE <= 2 {
            imp::arguments(value, ROLE as usize)
        } else {
            return Err(CodecError::Malformed);
        }
        .map(|_| ())
        .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
