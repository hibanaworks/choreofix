//! Validated request data carried through the infrastructure endpoint.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Arguments<const OPERATION: u8>(Vec<u8>);

impl<const OPERATION: u8> Arguments<OPERATION> {
    const KEY: &'static str = match OPERATION {
        1 => "component",
        2 => "suite",
        3 => "target",
        _ => panic!("unknown CI/CD operation"),
    };

    const TARGET: &'static str = match OPERATION {
        1 => "application",
        2 => "integration",
        3 => "production",
        _ => panic!("unknown CI/CD operation"),
    };

    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
        Ok(Self(bytes.to_vec()))
    }

    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated CI/CD arguments")
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
    const SCHEMA_ID: u32 = 0xac16_0000 | OPERATION as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if object.len() != 1 || object.get(Self::KEY).and_then(Value::as_str) != Some(Self::TARGET)
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
