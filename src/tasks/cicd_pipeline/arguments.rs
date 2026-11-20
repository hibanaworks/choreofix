use super::global::{BACKEND_DEV, FRONTEND_DEV};
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
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

// Distinguish operations that accept identical JSON on different projected edges.
pub const COMPILE: u8 = 91;
pub const UNIT_TEST: u8 = 93;
pub const ARTIFACT_PUBLICATION: u8 = 80;
pub const STAGE: u8 = 103;
pub const DEPLOY: u8 = 107;

/// Original tool arguments, checked at the projected file's close.
pub struct Arguments<const INPUT: u8, const OP: u8 = 0x20>(Vec<u8>);

impl<const INPUT: u8, const OP: u8> Arguments<INPUT, OP> {
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

    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes)).map_err(ShellError::from)?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const INPUT: u8, const OP: u8> WireEncode for Arguments<INPUT, OP> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const INPUT: u8, const OP: u8> WirePayload for Arguments<INPUT, OP> {
    const SCHEMA_ID: u32 = 0xac16_0000 | ((OP as u32) << 8) | INPUT as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

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

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

const fn developer(component: u8) -> &'static str {
    match component {
        FRONTEND_COMPONENT => "FRONTEND_DEV",
        BACKEND_COMPONENT => "BACKEND_DEV",
        _ => panic!("unknown build component"),
    }
}

impl<const COMPONENT: u8> SimulatorOperation for Arguments<COMPONENT, COMPILE> {
    const RPC: (&'static str, &'static str) = (developer(COMPONENT), "compile_code");
}
impl<const COMPONENT: u8> SimulatorOperation for Arguments<COMPONENT, UNIT_TEST> {
    const RPC: (&'static str, &'static str) = (developer(COMPONENT), "run_unit_tests");
}
impl SimulatorOperation<2> for Arguments<FRONTEND_DEV, ARTIFACT_PUBLICATION> {
    const RPC: (&'static str, &'static str) = ("FRONTEND_DEV", "publish_artifact");
}
impl SimulatorOperation<2> for Arguments<BACKEND_DEV, ARTIFACT_PUBLICATION> {
    const RPC: (&'static str, &'static str) = ("BACKEND_DEV", "publish_artifact");
}
impl SimulatorOperation for Arguments<QA_WORKSPACE> {
    const RPC: (&'static str, &'static str) = ("QA_ENGINEER", "pull_artifacts");
}
impl SimulatorOperation for Arguments<RELEASE_WORKSPACE> {
    const RPC: (&'static str, &'static str) = ("RELEASE_ENGINEER", "pull_artifacts");
}
impl SimulatorOperation for Arguments<INTEGRATION_SUITE> {
    const RPC: (&'static str, &'static str) = ("QA_ENGINEER", "run_integration_tests");
}
impl SimulatorOperation for Arguments<E2E_SUITE> {
    const RPC: (&'static str, &'static str) = ("QA_ENGINEER", "run_e2e_tests");
}
impl SimulatorOperation for Arguments<FULL_STACK, STAGE> {
    const RPC: (&'static str, &'static str) = ("RELEASE_ENGINEER", "deploy_staging");
}
impl SimulatorOperation for Arguments<FULL_STACK, DEPLOY> {
    const RPC: (&'static str, &'static str) = ("RELEASE_ENGINEER", "deploy_production");
}
impl SimulatorOperation for Arguments<STAGING> {
    const RPC: (&'static str, &'static str) = ("RELEASE_ENGINEER", "run_smoke_tests");
}
impl SimulatorOperation for Arguments<PRODUCTION> {
    const RPC: (&'static str, &'static str) = ("RELEASE_ENGINEER", "run_smoke_tests");
}

impl<const DEVELOPER: u8> From<Arguments<DEVELOPER>>
    for Arguments<DEVELOPER, ARTIFACT_PUBLICATION>
{
    fn from(admitted: Arguments<DEVELOPER>) -> Self {
        Self(admitted.0)
    }
}
