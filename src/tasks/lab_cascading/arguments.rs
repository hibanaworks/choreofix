//! Inspection input is ordinary optional, freely chosen notes.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct InspectionArguments(Vec<u8>);

impl InspectionArguments {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated inspection arguments")
    }
}

impl TryFrom<&[u8]> for InspectionArguments {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl WireEncode for InspectionArguments {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl WirePayload for InspectionArguments {
    const SCHEMA_ID: u32 = 0xac13_2010;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if object.keys().any(|key| key != "notes")
            || object.get("notes").is_some_and(|notes| !notes.is_string())
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

/// Original scientific tool arguments; compound names and culture text remain data.
pub struct Arguments<const OP: u8>(Vec<u8>);

impl<const OP: u8> Arguments<OP> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated cascading laboratory arguments")
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
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const OP: u8> WirePayload for Arguments<OP> {
    const SCHEMA_ID: u32 = 0xac13_2000 | OP as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        let (key, fixed) = match OP {
            0 | 5 => ("compound", None),
            1 => ("sample", Some("synthesized_compound")),
            2 => ("material", Some("compound_purification")),
            3 => ("material", Some("cell_cultures")),
            4 => ("sample", Some("fluorescence")),
            6 => ("sample", None),
            7 => ("sample", Some("molecular_detail")),
            8 => ("material", Some("verification")),
            9 => ("scientist", Some("BIOLOGIST")),
            10 => ("sample", Some("final_validation")),
            _ => return Err(CodecError::Malformed),
        };
        if !value[key].is_string()
            || fixed.is_some_and(|expected| value[key] != expected)
            || object.keys().any(|field| {
                field != key
                    && !(OP == 5 && field == "cell_type")
                    && !(OP == 9 && field == "data_type")
            })
            || (OP == 0
                && value[key]
                    .as_str()
                    .is_none_or(|name| name.trim().is_empty()))
            || object
                .get("cell_type")
                .is_some_and(|culture| !culture.is_string())
            || (OP == 9 && value["data_type"] != "bioassay")
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
