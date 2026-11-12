//! Exact report request data admitted by ChoreoFS.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

/// Exact bytes admitted by the guest's ChoreoFS close, including optional data.
#[derive(Debug)]
pub struct ReportArguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> ReportArguments<OPERATION> {
    const KEY: &'static str = match OPERATION {
        1 => "topic",
        2..=5 => "section_name",
        6 => "sections",
        _ => panic!("unknown report operation"),
    };
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("invalid report arguments"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated report arguments")
    }
}
impl<const OPERATION: u8> WireEncode for ReportArguments<OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for ReportArguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac03_0000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        let optional = match OPERATION {
            2 => Some("content_notes"),
            3 => Some("feedback"),
            _ => None,
        };
        if !object.get(Self::KEY).is_some_and(Value::is_string)
            || object.iter().any(|(name, value)| {
                (name != Self::KEY && Some(name.as_str()) != optional) || !value.is_string()
            })
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
    fn report_requests_preserve_optional_notes_and_reject_other_operations_fields() {
        let bytes = br#"{"section_name":"section_a","content_notes":"independent notes"}"#;
        assert_eq!(
            ReportArguments::<2>::from_bytes(bytes)
                .unwrap()
                .into_value()["content_notes"],
            "independent notes"
        );
        assert!(ReportArguments::<3>::from_bytes(bytes).is_err());
        assert!(
            ReportArguments::<2>::from_bytes(br#"{"section_name":"section_a","content_notes":2}"#)
                .is_err()
        );
        assert!(
            ReportArguments::<4>::from_bytes(br#"{"section_name":"section_a","feedback":"stale"}"#)
                .is_err()
        );
        assert!(
            ReportArguments::<1>::from_bytes(
                br#"{"topic":"subtopic_a","section_name":"section_b"}"#
            )
            .is_err()
        );
        assert!(ReportArguments::<6>::from_bytes(br#"{"sections":[]}"#).is_err());
    }
}
