//! Exact admitted request bytes; no file dispatch or protocol state.
use super::{
    global::{BUILDER_A, BUILDER_B, INTEGRATOR},
    imp,
};
use crate::shell::imp::service::SimulatorOperation;
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const ROLE: u8, const RESOURCE: bool = false>(Vec<u8>);
impl<const ROLE: u8, const RESOURCE: bool> Arguments<ROLE, RESOURCE> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
    #[cfg(test)]
    pub fn into_value(self) -> serde_json::Value {
        serde_json::from_slice(&self.0).expect("validated build arguments")
    }
}
impl<const ROLE: u8, const RESOURCE: bool> WireEncode for Arguments<ROLE, RESOURCE> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const ROLE: u8, const RESOURCE: bool> WirePayload for Arguments<ROLE, RESOURCE> {
    const SCHEMA_ID: u32 = 0x5342_0000 | ((RESOURCE as u32) << 8) | ROLE as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        match ROLE {
            BUILDER_A => imp::compile_arguments(value, "module_a"),
            BUILDER_B => imp::compile_arguments(value, "module_b"),
            INTEGRATOR if !RESOURCE => imp::link_arguments(value),
            _ => return Err(CodecError::Malformed),
        }
        .map(|_| ())
        .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

impl<const ROLE: u8> SimulatorOperation<2> for Arguments<ROLE, true> {
    const RPC: (&'static str, &'static str) = (
        match ROLE {
            BUILDER_A => "BUILDER_A",
            BUILDER_B => "BUILDER_B",
            _ => panic!("unknown build resource receiver"),
        },
        "compile_module",
    );
}
impl SimulatorOperation for Arguments<INTEGRATOR> {
    const RPC: (&'static str, &'static str) = ("INTEGRATOR", "link_modules");
}
