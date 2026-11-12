//! One wire contract for actual positive and negative review replies.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

/// Original review response whose success field agrees with its message label.
pub struct ReviewReply<'a, const APPROVED: bool>(&'a [u8]);
impl<'a, const APPROVED: bool> ReviewReply<'a, APPROVED> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("review result differs from its outcome"))?;
        Ok(Self(bytes))
    }
}
impl<const APPROVED: bool> WireEncode for ReviewReply<'_, APPROVED> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const APPROVED: bool> WirePayload for ReviewReply<'_, APPROVED> {
    const SCHEMA_ID: u32 = 0xac06_1000 | APPROVED as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if value["success"].as_bool() != Some(APPROVED) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}
