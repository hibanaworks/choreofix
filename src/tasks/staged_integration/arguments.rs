//! Canonical JSON bytes for one assigned operation; no progress or permission state.
use crate::shell::imp::service::SimulatorOperation;
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
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated staged-integration arguments")
    }
}
impl<const OPERATION: u8> TryFrom<&[u8]> for Arguments<OPERATION> {
    type Error = CodecError;
    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
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

/// The request consumed by the physical receiver. Forwarding and stage checks carry Arguments.
pub struct EffectRequest<const OPERATION: u8>(pub Arguments<OPERATION>);

impl<const OPERATION: u8> WireEncode for EffectRequest<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.encode_into(output)
    }
}
impl<const OPERATION: u8> WirePayload for EffectRequest<OPERATION> {
    const SCHEMA_ID: u32 = 0xac06_3100 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<OPERATION>::validate_payload(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl<const OPERATION: u8> SimulatorOperation for EffectRequest<OPERATION> {
    const RPC: (&'static str, &'static str) = {
        let developer = ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"][(OPERATION / 8) as usize];
        match OPERATION % 8 {
            0 => (developer, "implement_feature"),
            1 => ("TEST_WRITER", "write_tests"),
            2 => ("REVIEWER", "code_review"),
            3 => (developer, "merge_code"),
            4 => ("CI_RUNNER", "run_tests"),
            5 => ("RELEASE_MANAGER", "deploy_service"),
            6 => ("RELEASE_MANAGER", "validate_staging"),
            _ => ("RELEASE_MANAGER", "rollback_service"),
        }
    };
}
