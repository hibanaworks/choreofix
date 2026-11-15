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
        7 => "first_writer",
        8..=14 => "section_name",
        16 => "sections",
        17 => "review",
        18 => "review_id",
        19 | 20 => "topic",
        21..=28 => "section_name",
        29 => "sections",
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
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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
        if OPERATION == 17 {
            if value["tool_name"] != "review_sections"
                || !value["success"].is_boolean()
                || !matches!(
                    value["data"]["scope"].as_str(),
                    Some("CHAPTER_1" | "CHAPTER_2")
                )
            {
                return Err(CodecError::Malformed);
            }
            return Ok(());
        }
        let optional = match OPERATION {
            2 | 8 | 9 | 21 | 22 => Some("content_notes"),
            3 | 14 | 23 | 24 => Some("feedback"),
            _ => None,
        };
        if !object.get(Self::KEY).is_some_and(Value::is_string)
            || object.iter().any(|(name, value)| {
                (name != Self::KEY && Some(name.as_str()) != optional) || !value.is_string()
            })
        {
            return Err(CodecError::Malformed);
        }
        if OPERATION == 18 && value[Self::KEY].as_str().is_none_or(str::is_empty) {
            return Err(CodecError::Malformed);
        }
        let chapter = match OPERATION {
            8 | 10 | 12 => Some("CHAPTER_1"),
            9 | 11 | 13 => Some("CHAPTER_2"),
            _ => None,
        };
        if chapter.is_some_and(|name| value[Self::KEY] != name)
            || (OPERATION == 14
                && !matches!(value[Self::KEY].as_str(), Some("CHAPTER_1" | "CHAPTER_2")))
        {
            return Err(CodecError::Malformed);
        }
        let expected = match OPERATION {
            19 => Some("subtopic_a"),
            20 => Some("subtopic_b"),
            21 | 23 | 25 | 27 => Some("section_a"),
            22 | 24 | 26 | 28 => Some("section_b"),
            _ => None,
        };
        if expected.is_some_and(|value| object[Self::KEY] != value) {
            return Err(CodecError::Malformed);
        }
        if OPERATION == 29 {
            let mut sections: Vec<_> = value[Self::KEY]
                .as_str()
                .ok_or(CodecError::Malformed)?
                .split(',')
                .map(str::trim)
                .collect();
            sections.sort_unstable();
            if sections != ["section_a", "section_b"] {
                return Err(CodecError::Malformed);
            }
        }
        if OPERATION == 16 {
            let mut sections: Vec<_> = value[Self::KEY]
                .as_str()
                .ok_or(CodecError::Malformed)?
                .split(',')
                .map(str::trim)
                .collect();
            sections.sort_unstable();
            if sections != ["CHAPTER_1", "CHAPTER_2"] {
                return Err(CodecError::Malformed);
            }
        }
        if OPERATION == 7 && !matches!(object[Self::KEY].as_str(), Some("WRITER_A" | "WRITER_B")) {
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

    #[test]
    fn reconsideration_carries_an_actual_review_with_either_verdict() {
        for verdict in [true, false] {
            for chapter in ["CHAPTER_1", "CHAPTER_2"] {
                let value = serde_json::json!({
                    "tool_name":"review_sections", "success":verdict,
                    "data":{"scope":chapter,"review_id":"current-review"}
                });
                let bytes = serde_json::to_vec(&value).unwrap();
                let arguments = ReportArguments::<17>::from_bytes(&bytes).unwrap();
                assert_eq!(arguments.as_bytes(), bytes);
                assert_eq!(arguments.into_value(), value);
            }
        }
        for bytes in [
            br#"{"tool_name":"review_sections","success":"false","data":{"scope":"CHAPTER_1"}}"#
                .as_slice(),
            br#"{"tool_name":"review_sections","success":false,"data":{"scope":"OUTSIDE"}}"#
                .as_slice(),
            br#"{"tool_name":"combine_sections","success":true,"data":{"scope":"CHAPTER_1"}}"#
                .as_slice(),
        ] {
            assert!(ReportArguments::<17>::from_bytes(bytes).is_err());
        }
    }
}

#[cfg(test)]
mod report_section_tests {
    use super::*;
    #[test]
    fn direct_report_files_validate_their_assigned_target_before_admission() {
        assert!(ReportArguments::<19>::from_bytes(br#"{"topic":"subtopic_a"}"#).is_ok());
        assert!(ReportArguments::<19>::from_bytes(br#"{"topic":"subtopic_b"}"#).is_err());
        assert!(ReportArguments::<20>::from_bytes(br#"{"topic":"subtopic_b"}"#).is_ok());
        assert!(ReportArguments::<20>::from_bytes(br#"{"topic":"subtopic_a"}"#).is_err());
        let a = br#"{"section_name":"section_a"}"#;
        let b = br#"{"section_name":"section_b"}"#;
        assert!(ReportArguments::<21>::from_bytes(a).is_ok());
        assert!(ReportArguments::<21>::from_bytes(b).is_err());
        assert!(ReportArguments::<22>::from_bytes(b).is_ok());
        assert!(ReportArguments::<22>::from_bytes(a).is_err());
        assert!(ReportArguments::<23>::from_bytes(a).is_ok());
        assert!(ReportArguments::<23>::from_bytes(b).is_err());
        assert!(ReportArguments::<24>::from_bytes(b).is_ok());
        assert!(ReportArguments::<24>::from_bytes(a).is_err());
        assert!(ReportArguments::<25>::from_bytes(a).is_ok());
        assert!(ReportArguments::<25>::from_bytes(b).is_err());
        assert!(ReportArguments::<26>::from_bytes(b).is_ok());
        assert!(ReportArguments::<26>::from_bytes(a).is_err());
        assert!(ReportArguments::<27>::from_bytes(a).is_ok());
        assert!(ReportArguments::<27>::from_bytes(b).is_err());
        assert!(ReportArguments::<28>::from_bytes(b).is_ok());
        assert!(ReportArguments::<28>::from_bytes(a).is_err());
        for sections in ["section_a,section_b", "section_b,section_a"] {
            let bytes = serde_json::to_vec(&serde_json::json!({"sections":sections})).unwrap();
            assert!(ReportArguments::<29>::from_bytes(&bytes).is_ok());
        }
        for sections in ["section_a", "section_a,section_a", "section_b,section_c"] {
            let bytes = serde_json::to_vec(&serde_json::json!({"sections":sections})).unwrap();
            assert!(ReportArguments::<29>::from_bytes(&bytes).is_err());
        }
    }
}
