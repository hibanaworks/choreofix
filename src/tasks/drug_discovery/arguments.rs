use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub const SYNTHESIS_QC: u8 = 1;
pub const MOLECULAR_ID: u8 = 2;
pub const CULTURE_PREP: u8 = 3;
pub const BIOASSAY: u8 = 4;
pub const STABILITY: u8 = 5;
pub const BASELINE: u8 = 6;
pub const PURITY: u8 = 7;
pub const CYTOTOXICITY: u8 = 8;
pub const REFERENCE: u8 = 9;
pub const STRUCTURE: u8 = 10;
pub const CHARACTERIZATION: u8 = 11;
pub const DECISION: u8 = 12;
pub const DOCUMENTATION: u8 = 13;
pub const COMPLIANCE: u8 = 14;
pub const REANALYSIS: u8 = 15;

/// A file request retains the exact accepted argument bytes.
pub struct Arguments<const OP: u8>(Vec<u8>);

impl<const OP: u8> Arguments<OP> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated discovery arguments")
    }
}

impl<const OP: u8> TryFrom<&[u8]> for Arguments<OP> {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
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
    const SCHEMA_ID: u32 = 0xac14_0000 | OP as u32;
    type Decoded<'a> = Self;

    fn validate_payload(payload: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(payload.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = usize::from(OP == REANALYSIS) + 1;
        if value.as_object().map_or(0, serde_json::Map::len) != fields {
            return Err(CodecError::Malformed);
        }
        let valid = match OP {
            SYNTHESIS_QC => value["sample"] == "synthesis_qc",
            MOLECULAR_ID => value["sample"] == "molecular_id",
            CULTURE_PREP => value["assay_type"] == "culture_prep",
            BIOASSAY => value["assay_type"] == "bioassay",
            STABILITY => value["sample"] == "stability",
            BASELINE => value["sample"] == "baseline_markers",
            PURITY => value["sample"] == "purity_check",
            CYTOTOXICITY => value["assay_type"] == "cytotoxicity",
            REFERENCE => value["sample"] == "reference_methods",
            STRUCTURE => value["sample"] == "structural_confirmation",
            CHARACTERIZATION => value["sample"] == "characterization",
            DECISION => value["go"].is_boolean(),
            DOCUMENTATION => value["sample"] == "documentation",
            COMPLIANCE => value["decision"] == "go_no_go",
            REANALYSIS => value["biology"].is_boolean() && value["toxicology"].is_boolean(),
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(CodecError::Malformed)
        }
    }

    fn decode_validated_payload(payload: ReceivedPayload<'_, Self>) -> Self {
        Self(payload.as_bytes().to_vec())
    }
}
