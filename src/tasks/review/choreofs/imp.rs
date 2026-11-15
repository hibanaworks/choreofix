//! Exact operation data accepted by the file Endpoint's public message contract.
#[cfg(test)]
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;
pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated operation arguments")
    }
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        Ok(Self(bytes.to_vec()))
    }
}
impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac06_1000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (key, target) = match OPERATION {
            0 => ("feature_name", super::global::FEATURE_A),
            1 => ("branch", super::global::MERGE_BRANCH_A),
            2 => ("feature_name", super::global::FEATURE_B),
            3 => ("branch", super::global::MERGE_BRANCH_B),
            5 => ("pr_id", super::global::FEATURE_A),
            6 => ("pr_id", super::global::FEATURE_B),
            _ => return Err(CodecError::Malformed),
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if value[key] != target
            || object
                .keys()
                .any(|k| k != key && !((OPERATION == 0 || OPERATION == 2) && k == "source"))
            || object
                .get("source")
                .is_some_and(|v| v.as_str().is_none_or(|v| v.trim().is_empty()))
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
