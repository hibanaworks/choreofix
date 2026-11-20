//! 7M's document and figure-store effect boundary.
pub mod global;
pub mod localside;
#[cfg(not(test))]
mod run;
#[cfg(not(test))]
pub use run::{projection, run_stdio};

use crate::{imp::json::decode, shell::ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub const CHAPTERS: [&str; 3] = ["CHAPTER_1", "CHAPTER_2", "CHAPTER_3"];
pub const FACT_SCOPES: [&str; 2] = ["CHAPTERS_1_2", "CHAPTER_3"];

fn quality_scope(operation: u8) -> Option<&'static str> {
    use global::*;
    match operation {
        CHECK_12 => Some(FACT_SCOPES[0]),
        CHECK_3 => Some(FACT_SCOPES[1]),
        REVIEW_1 => Some(CHAPTERS[0]),
        REVIEW_2 => Some(CHAPTERS[1]),
        REVIEW_3 => Some(CHAPTERS[2]),
        _ => None,
    }
}

fn sections(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// JSON arguments for one of the task's typed file operations.
#[derive(Debug)]
pub struct Arguments<const OP: u8>(Vec<u8>);

impl<const OP: u8> Arguments<OP> {
    #[cfg(test)]
    fn new(value: &Value) -> ShellResult<Self> {
        let bytes = crate::imp::json::encode(value)?;
        Self::validate_payload(Payload::new(&bytes))
            .map_err(|_| crate::shell::ShellError::message("invalid figure document arguments"))?;
        Ok(Self(bytes))
    }

    fn value(&self) -> ShellResult<Value> {
        decode(&self.0)
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
    const SCHEMA_ID: u32 = 0xac7d_0000 | OP as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let args = value.as_object().ok_or(CodecError::Malformed)?;
        let chapter = value["section_name"]
            .as_str()
            .is_some_and(|name| CHAPTERS.contains(&name));
        let valid = match OP {
            global::WRITE | global::DRAFT_1 | global::DRAFT_2 | global::DRAFT_3 => {
                chapter
                    && (OP == global::WRITE
                        || value["section_name"] == CHAPTERS[usize::from(OP - global::DRAFT_1)])
                    && args.iter().all(|(key, value)| {
                        matches!(key.as_str(), "section_name" | "content_notes")
                            && value.is_string()
                    })
            }
            global::CREATE_1 | global::CREATE_3 => {
                args.len() == 1
                    && value["description"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty())
            }
            global::INSERT_1 | global::INSERT_3 => {
                let section = if OP == global::INSERT_1 {
                    CHAPTERS[0]
                } else {
                    CHAPTERS[2]
                };
                args.len() == 2
                    && value["section_name"] == section
                    && value["figure_id"]
                        .as_str()
                        .is_some_and(|id| !id.trim().is_empty())
            }
            global::READ => chapter && args.len() == 1,
            global::REFERENCES_1 | global::REFERENCES_2 | global::REFERENCES_3 => {
                args.len() == 1
                    && value["section_name"] == CHAPTERS[usize::from(OP - global::REFERENCES_1)]
            }
            global::REVISE_A | global::REVISE_B => {
                (if OP == global::REVISE_A {
                    matches!(
                        value["section_name"].as_str(),
                        Some("CHAPTER_1" | "CHAPTER_2")
                    )
                } else {
                    value["section_name"] == CHAPTERS[2]
                }) && args.iter().all(|(key, value)| {
                    matches!(key.as_str(), "section_name" | "feedback") && value.is_string()
                })
            }
            global::COMBINE => {
                args.len() == 1
                    && value["sections"].as_str().is_some_and(|text| {
                        let mut requested: Vec<_> = sections(text).collect();
                        requested.sort_unstable();
                        requested == CHAPTERS
                    })
            }
            operation if quality_scope(operation).is_some() => {
                args.len() == 1 && value["section_name"].as_str() == quality_scope(operation)
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

#[cfg(test)]
mod drafting_tests;
#[cfg(test)]
mod preparation_tests;
#[cfg(test)]
mod quality_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
