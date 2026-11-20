use super::global::{BACKEND_DEV, FRONTEND_DEV};
#[cfg(test)]
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub const FRONTEND_COMPONENT: u8 = 0;
pub const BACKEND_COMPONENT: u8 = 1;
pub const QA_WORKSPACE: u8 = 16;
pub const RELEASE_WORKSPACE: u8 = 17;
pub const INTEGRATION_SUITE: u8 = 18;
pub const E2E_SUITE: u8 = 19;
pub const FULL_STACK: u8 = 20;
pub const STAGING: u8 = 21;
pub const PRODUCTION: u8 = 22;
pub const FRONTEND_FIRST: u8 = 23;
pub const BACKEND_FIRST: u8 = 24;
pub const REPAIR_SELECTION: u8 = 25;

/// Original tool arguments, checked at the projected file's close.
pub struct Arguments<const INPUT: u8>(Vec<u8>);

impl<const INPUT: u8> Arguments<INPUT> {
    pub const AGENT: &'static str = match INPUT {
        FRONTEND_DEV => "FRONTEND_DEV",
        BACKEND_DEV => "BACKEND_DEV",
        _ => panic!("this argument does not identify a developer"),
    };
    const FIELD: (&'static str, &'static str) = match INPUT {
        FRONTEND_COMPONENT => ("component", "frontend"),
        BACKEND_COMPONENT => ("component", "backend"),
        FRONTEND_DEV => ("artifact", "frontend_image"),
        BACKEND_DEV => ("artifact", "backend_image"),
        QA_WORKSPACE => ("target", "test_harness"),
        RELEASE_WORKSPACE => ("target", "staging_deploy"),
        INTEGRATION_SUITE => ("suite", "api_integration"),
        E2E_SUITE => ("suite", "user_flows"),
        FULL_STACK => ("component", "full_stack"),
        STAGING => ("target", "staging"),
        PRODUCTION => ("target", "production"),
        FRONTEND_FIRST => ("first", "frontend"),
        BACKEND_FIRST => ("first", "backend"),
        REPAIR_SELECTION => ("component", "frontend"),
        _ => panic!("unknown pipeline input"),
    };

    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated pipeline arguments")
    }

    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes)).map_err(ShellError::from)?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const INPUT: u8> WireEncode for Arguments<INPUT> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const INPUT: u8> WirePayload for Arguments<INPUT> {
    const SCHEMA_ID: u32 = 0xac16_2000 | INPUT as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        let (key, argument) = Self::FIELD;
        let selected = object.get(key).and_then(Value::as_str);
        let valid = selected == Some(argument)
            || INPUT == REPAIR_SELECTION && matches!(selected, Some("backend" | "both"));
        if object.len() != 1 || !valid {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
