//! Canonical JSON bytes for one assigned operation; no progress or permission state.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub const FEATURES: [&str; 3] = [
    "user_authentication",
    "payment_processing",
    "notification_system",
];
pub const SUITES: [&str; 3] = [
    "full_suite_after_auth",
    "full_suite_after_payment",
    "full_suite_after_notification",
];

pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated staged-integration arguments")
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
    const SCHEMA_ID: u32 = 0xac06_3000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        if OPERATION >= 24 {
            return Err(CodecError::Malformed);
        }
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let feature = usize::from(OPERATION / 8);
        let operation = OPERATION % 8;
        let key = match operation {
            0 => "feature_name",
            1 => "target",
            2 => "pr_id",
            3 => "branch",
            4 => "suite",
            5..=7 => "service",
            _ => unreachable!(),
        };
        let expected = if operation == 4 {
            SUITES[feature]
        } else {
            FEATURES[feature]
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if value[key] != expected
            || object.keys().any(|key_name| {
                key_name != key
                    && !(operation == 0 && key_name == "source")
                    && !(operation == 5 && key_name == "environment")
            })
            || object.get("source").is_some_and(|source| {
                source
                    .as_str()
                    .is_none_or(|source| source.trim().is_empty())
            })
            || object
                .get("environment")
                .is_some_and(|environment| environment != "staging")
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
