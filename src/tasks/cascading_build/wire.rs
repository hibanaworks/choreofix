//! Exact request data. Conversion never grants an operation.
use super::{
    global::{A, B, C, INTEGRATOR, TESTER},
    imp,
};
use crate::shell::imp::service::SimulatorOperation;
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

/// The resource message carries the acknowledged compile arguments, test input object, or link request.
pub struct BuildRequest<'a, const ROLE: u8>(pub &'a [u8]);
impl<const ROLE: u8> WireEncode for BuildRequest<'_, ROLE> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.encode_into(output)
    }
}
impl<const ROLE: u8> WirePayload for BuildRequest<'_, ROLE> {
    const SCHEMA_ID: u32 = 0x4342_0100 | ROLE as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        if ROLE != TESTER {
            return Arguments::<ROLE>::validate_payload(input);
        }
        let mut value: serde_json::Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = value
            .as_object_mut()
            .filter(|fields| fields.len() == 2)
            .ok_or(CodecError::Malformed)?;
        let arguments = fields.remove("arguments").ok_or(CodecError::Malformed)?;
        let object = fields.get("object").ok_or(CodecError::Malformed)?;
        let index = imp::module(&object["module"]).map_err(|_| CodecError::Malformed)?;
        imp::arguments(arguments, index)
            .and_then(|_| imp::validate_object(object, index))
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl SimulatorOperation<2> for BuildRequest<'_, A> {
    const RPC: (&'static str, &'static str) = (imp::AGENTS[A as usize], "compile_module");
}
impl SimulatorOperation<2> for BuildRequest<'_, B> {
    const RPC: (&'static str, &'static str) = (imp::AGENTS[B as usize], "compile_module");
}
impl SimulatorOperation<2> for BuildRequest<'_, C> {
    const RPC: (&'static str, &'static str) = (imp::AGENTS[C as usize], "compile_module");
}
impl SimulatorOperation for BuildRequest<'_, INTEGRATOR> {
    const RPC: (&'static str, &'static str) = ("INTEGRATOR", "link_modules");
}
impl SimulatorOperation for BuildRequest<'_, TESTER> {
    const RPC: (&'static str, &'static str) = ("TEST_RUNNER", "run_build_tests");
    const ARGUMENT_FIELD: Option<&'static str> = Some("arguments");
}
