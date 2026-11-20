//! The actual patient request bytes; validation contains no protocol progress.
use crate::imp::patient;
use crate::shell::imp::service::SimulatorOperation;
#[cfg(test)]
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
    const SCHEMA_ID: u32 = 0xac05_0000 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let arguments: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        patient::patient(&arguments).map_err(|_| CodecError::Malformed)?;
        let valid = match OPERATION {
            0 => true,
            1 => patient::tests(&arguments, &["vitals", "basic_labs"]).is_ok(),
            2 => arguments["modality"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("CT")),
            3 => patient::tests(&arguments, &["ecg", "cardiac_enzymes"]).is_ok(),
            4 => arguments.get("symptoms").is_none_or(Value::is_string),
            5 => arguments.get("medications").is_none_or(Value::is_string),
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(CodecError::Malformed)
        }
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

/// File admission supplies data for the later request on the loaned resource.
pub struct Submission<const OPERATION: u8>(pub Arguments<OPERATION>);

impl<const OPERATION: u8> WireEncode for Submission<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.encode_into(output)
    }
}

impl<const OPERATION: u8> WirePayload for Submission<OPERATION> {
    const SCHEMA_ID: u32 = 0xac05_0100 | OPERATION as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<OPERATION>::validate_payload(input)
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self {
        Self(Arguments(input.as_bytes().to_vec()))
    }
}

impl SimulatorOperation for Arguments<0> {
    const RPC: (&'static str, &'static str) = ("ER_DOCTOR", "assess_patient");
}

impl SimulatorOperation for Arguments<1> {
    const RPC: (&'static str, &'static str) = ("ER_DOCTOR", "order_tests");
}

impl SimulatorOperation for Arguments<2> {
    const RPC: (&'static str, &'static str) = ("ER_DOCTOR", "perform_imaging");
}

impl SimulatorOperation for Arguments<3> {
    const RPC: (&'static str, &'static str) = ("CARDIOLOGIST", "order_tests");
}

impl SimulatorOperation for Arguments<4> {
    const RPC: (&'static str, &'static str) = ("CARDIOLOGIST", "diagnose");
}

impl SimulatorOperation for Arguments<5> {
    const RPC: (&'static str, &'static str) = ("PHARMACIST", "review_prescriptions");
}
