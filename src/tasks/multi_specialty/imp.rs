//! The actual patient request bytes; validation contains no protocol progress.
use crate::imp::patient;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

#[derive(Debug)]
pub struct Arguments<const OPERATION: u8>(Vec<u8>);

impl<const OPERATION: u8> Arguments<OPERATION> {
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("request differs from this patient operation"))?;
        Ok(Self(bytes.to_vec()))
    }

    pub fn from_value(value: Value) -> ShellResult<Self> {
        let bytes =
            serde_json::to_vec(&value).map_err(|error| ShellError::message(error.to_string()))?;
        Self::validate_payload(Payload::new(&bytes))
            .map_err(|_| ShellError::message("request differs from this patient operation"))?;
        Ok(Self(bytes))
    }

    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated patient request")
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
    const SCHEMA_ID: u32 = 0xac05_1000 | OPERATION as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let arguments: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        patient::patient(if OPERATION == 12 {
            &arguments["arguments"]
        } else {
            &arguments
        })
        .map_err(|_| CodecError::Malformed)?;
        let valid = match OPERATION {
            0 => true,
            1 => patient::tests(&arguments, &["vitals", "basic_labs"]).is_ok(),
            2 => patient::tests(&arguments, &["ecg", "cardiac_enzymes"]).is_ok(),
            3 => patient::tests(&arguments, &["neuro_exam", "nerve_conduction"]).is_ok(),
            4 | 5 => arguments["modality"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("CT")),
            6 | 7 => arguments["modality"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("MRI")),
            8 | 9 => arguments.get("symptoms").is_none_or(Value::is_string),
            10 => arguments.get("medications").is_none_or(Value::is_string),
            11 => arguments.get("attendees").is_none_or(Value::is_string),
            12 => {
                arguments["case"].is_object()
                    && arguments["arguments"]
                        .get("attendees")
                        .is_none_or(Value::is_string)
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(CodecError::Malformed)
        }
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
