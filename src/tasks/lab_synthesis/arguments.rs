//! The original laboratory arguments; culture text remains caller data.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Arguments<const OP: u8>(Vec<u8>);
impl<const OP: u8> Arguments<OP> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated laboratory arguments")
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
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
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
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
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
