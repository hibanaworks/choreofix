//! The original laboratory arguments; culture text remains caller data.
use crate::shell::imp::service::SimulatorOperation;
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Arguments<const OP: u8>(Vec<u8>);
impl<const OP: u8> Arguments<OP> {
    fn validate(value: &Value) -> Result<(), CodecError> {
        let (key, target) = match OP {
            0 => ("sample", "raw_materials"),
            1 => ("material", "compound_purification"),
            2 => ("material", "cell_cultures"),
            3 => ("sample", "fluorescence"),
            4 => ("compound", "compound_purification"),
            5 => ("sample", "molecular_detail"),
            6 => ("material", "verification"),
            7 => ("sample", "final_validation"),
            _ => return Err(CodecError::Malformed),
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if value[key] != target
            || object
                .keys()
                .any(|k| k != key && !(OP == 4 && k == "cell_type"))
            || (object.contains_key("cell_type") && !value["cell_type"].is_string())
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
}
impl<const OP: u8> TryFrom<&[u8]> for Arguments<OP> {
    type Error = CodecError;
    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}
impl<const OP: u8> WireEncode for Arguments<OP> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OP: u8> WirePayload for Arguments<OP> {
    const SCHEMA_ID: u32 = 0xac0d_1000 | OP as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        Self::validate(&value)
    }
    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

impl SimulatorOperation for Arguments<4> {
    const RPC: (&'static str, &'static str) = ("BIOLOGIST", "run_bioassay");
}

/// A resource request carries the admitted file arguments and the material they refer to.
pub struct InstrumentRequest<'a, const OP: u8>(pub &'a [u8]);

impl<const OP: u8> WireEncode for InstrumentRequest<'_, OP> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        self.0.encode_into(out)
    }
}

impl<const OP: u8> WirePayload for InstrumentRequest<'_, OP> {
    const SCHEMA_ID: u32 = Arguments::<OP>::SCHEMA_ID | 0x100;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let packet: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        Arguments::<OP>::validate(&packet["arguments"])
    }
    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

impl<const OP: u8> SimulatorOperation for InstrumentRequest<'_, OP> {
    const ARGUMENT_FIELD: Option<&'static str> = Some("arguments");
    const RPC: (&'static str, &'static str) = match OP {
        0 | 7 => ("CHEMIST", "run_analysis"),
        1 => ("CHEMIST", "run_separation"),
        2 => ("BIOLOGIST", "run_separation"),
        3 => ("BIOLOGIST", "run_analysis"),
        5 => ("ANALYST", "run_analysis"),
        6 => ("ANALYST", "run_separation"),
        _ => panic!("not an instrument request"),
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_requests_accept_arbitrary_culture_data_and_reject_substitutions() {
        for value in [
            serde_json::json!({"compound":"compound_purification"}),
            serde_json::json!({"compound":"compound_purification","cell_type":"β culture ".repeat(40)}),
        ] {
            assert!(
                Arguments::<4>::try_from(serde_json::to_vec(&value).unwrap().as_slice()).is_ok()
            );
        }
        for value in [
            serde_json::json!({"compound":"other"}),
            serde_json::json!({"compound":"compound_purification","cell_type":7}),
            serde_json::json!({"compound":"compound_purification","phase":true}),
        ] {
            assert!(
                Arguments::<4>::try_from(serde_json::to_vec(&value).unwrap().as_slice()).is_err()
            );
        }
        assert!(Arguments::<0>::try_from(br#"{"sample":"raw_materials"}"#.as_slice()).is_ok());
        assert!(Arguments::<7>::try_from(br#"{"sample":"raw_materials"}"#.as_slice()).is_err());
    }
}
