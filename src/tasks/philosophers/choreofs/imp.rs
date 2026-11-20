use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
use hibana::runtime::wire::{CodecError, Payload, WireEncode, WirePayload};
use serde_json::Value;

/// Canonical JSON with the original task's required string argument.
#[derive(Debug, PartialEq, Eq)]
pub struct PhilosopherArguments(Vec<u8>);

impl PhilosopherArguments {
    pub fn from_value(value: Value) -> ShellResult<Self> {
        if !valid_object(&value) {
            return Err(ShellError::message(
                "expected a string argument: philosopher",
            ));
        }
        serde_json::to_vec(&value)
            .map(Self)
            .map_err(|error| ShellError::message(error.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        let value = serde_json::from_slice(bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        Self::from_value(value)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated canonical philosopher arguments")
    }
}

fn valid_object(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.get("philosopher").is_some_and(Value::is_string))
}

impl WireEncode for PhilosopherArguments {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        let bytes = output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?;
        bytes.copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl WirePayload for PhilosopherArguments {
    const SCHEMA_ID: u32 = 0x5048_4901;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if !valid_object(&value)
            || serde_json::to_vec(&value).map_err(|_| CodecError::Malformed)? != input.as_bytes()
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(
        input: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> Self::Decoded<'a> {
        Self(input.as_bytes().to_vec())
    }
}

/// Shell JSON keeps the user's whitespace and additional fields on the wire.
pub struct Thought<'a, const VIEW: u8>(&'a [u8]);
impl<const VIEW: u8> WireEncode for Thought<'_, VIEW> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const VIEW: u8> WirePayload for Thought<'_, VIEW> {
    const SCHEMA_ID: u32 = 0x5048_5000 | VIEW as u32;
    type Decoded<'a> = hibana::runtime::wire::ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        valid_object(&value)
            .then_some(())
            .ok_or(CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(
        input: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> Self::Decoded<'a> {
        input
    }
}

impl<const VIEW: u8> SimulatorOperation for Thought<'_, VIEW> {
    const RPC: (&'static str, &'static str) = (
        super::identity::AGENTS[(VIEW - super::identity::roles::THINK_A) as usize],
        "think",
    );
}

/// A physical fork's contribution to the projected diner's meal.
pub struct ForkRequest<'a, const VIEW: u8>(pub &'a PhilosopherArguments);
impl<const VIEW: u8> ForkRequest<'_, VIEW> {
    // A diner's ordinary view, its close Endpoint, and its own fork identify
    // the same physical simulator participant on the two incoming requests.
    const DINER: u8 = {
        use super::identity::roles::*;
        match VIEW {
            A..=G => VIEW - A,
            A_EAT_CLOSE..=G_EAT_CLOSE => VIEW - A_EAT_CLOSE,
            FORK_A..=FORK_G => VIEW - FORK_A,
            _ => panic!("not a dining request view"),
        }
    };
}
impl<const DINER: u8> WireEncode for ForkRequest<'_, DINER> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.encode_into(output)
    }
}
impl<const DINER: u8> WirePayload for ForkRequest<'_, DINER> {
    const SCHEMA_ID: u32 = 0x5048_5100 | Self::DINER as u32;
    type Decoded<'a> = hibana::runtime::wire::ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        PhilosopherArguments::validate_payload(input)
    }
    fn decode_validated_payload<'a>(
        input: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> Self::Decoded<'a> {
        input
    }
}
impl<const DINER: u8> SimulatorOperation<2> for ForkRequest<'_, DINER> {
    const RPC: (&'static str, &'static str) =
        (super::identity::AGENTS[Self::DINER as usize], "eat");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_arguments_round_trip_and_reject_inexact_data() {
        let original =
            serde_json::json!({"philosopher":"any model-chosen string","extra":{"preserved":true}});
        let arguments = PhilosopherArguments::from_value(original.clone()).unwrap();
        let mut bytes = vec![0; 128];
        let len = arguments.encode_into(&mut bytes).unwrap();
        let bytes = Payload::new(&bytes[..len]);
        PhilosopherArguments::validate_payload(bytes).unwrap();
        assert_eq!(
            PhilosopherArguments::from_bytes(bytes.as_bytes())
                .unwrap()
                .into_value(),
            original
        );
        for invalid in [
            b"{}".as_slice(),
            br#"{"philosopher":1}"#,
            br#"{"philosopher":null,"extra":1}"#,
            br#"{"philosopher":"a","philosopher":"b"}"#,
            br#"{ "philosopher":"a"}"#,
            br#"{"philosopher":"a"}null"#,
        ] {
            assert_eq!(
                PhilosopherArguments::validate_payload(Payload::new(invalid)),
                Err(CodecError::Malformed)
            );
        }
    }
}
