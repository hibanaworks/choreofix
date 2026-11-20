//! Validated request data carried through the infrastructure endpoint.
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

// The close intent and the resource request carry the same data. Only the
// latter decodes to a receipt accepted by the simulator.
pub struct Arguments<const OPERATION: u8, const RESOURCE: bool = false>(Vec<u8>);

impl<const OPERATION: u8, const RESOURCE: bool> Arguments<OPERATION, RESOURCE> {
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

    fn validate(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if object.len() != 1 || object.get(Self::KEY).and_then(Value::as_str) != Some(Self::TARGET)
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
}

impl<const OPERATION: u8> Arguments<OPERATION> {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
        Ok(Self(bytes.to_vec()))
    }

    pub fn into_request(self) -> Arguments<OPERATION, true> {
        Arguments(self.0)
    }
}

impl<const OPERATION: u8, const RESOURCE: bool> WireEncode for Arguments<OPERATION, RESOURCE> {
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
    type Decoded<'a> = Value;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Self::validate(input)
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Value {
        serde_json::from_slice(input.as_bytes()).expect("validated CI/CD intent")
    }
}

impl<const OPERATION: u8> WirePayload for Arguments<OPERATION, true> {
    const SCHEMA_ID: u32 = 0xac16_0100 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Self::validate(input)
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

impl<const OPERATION: u8> SimulatorOperation for Arguments<OPERATION, true> {
    const RPC: (&'static str, &'static str) = match OPERATION {
        1 => ("DEVELOPER", "compile_and_test"),
        2 => ("TESTER", "run_integration_tests"),
        3 => ("DEPLOYER", "deploy_application"),
        _ => panic!("unknown CI/CD operation"),
    };
}
