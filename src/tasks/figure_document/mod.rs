//! 7M's document and figure-store effect boundary.
pub mod global;
pub mod localside;
mod run;
pub use run::{projection, run_stdio};

#[cfg(test)]
use crate::shell::ShellResult;
use crate::shell::imp::service::SimulatorOperation;
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

#[cfg(test)]
impl<const OP: u8> Arguments<OP> {
    fn new(value: &Value) -> ShellResult<Self> {
        let bytes = crate::imp::json::encode(value)?;
        Self::validate_payload(Payload::new(&bytes))
            .map_err(|_| crate::shell::ShellError::message("invalid figure document arguments"))?;
        Ok(Self(bytes))
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
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let args = value.as_object().ok_or(CodecError::Malformed)?;
        let chapter = value["section_name"]
            .as_str()
            .is_some_and(|name| CHAPTERS.contains(&name));
        let valid = match OP {
            global::DRAFT_1 | global::DRAFT_2 | global::DRAFT_3 => {
                chapter
                    && value["section_name"] == CHAPTERS[usize::from(OP - global::DRAFT_1)]
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
            global::WRITER_A_SNAPSHOTS | global::WRITER_B_SNAPSHOTS => chapter && args.len() == 1,
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

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

// Each effect accepts only its own wire type. Insertion requires both resource receives.
macro_rules! operations {
    ($count:literal; $($op:ident => ($agent:literal, $tool:literal)),+ $(,)?) => {
        $(impl SimulatorOperation<$count> for Arguments<{ global::$op }> {
            const RPC: (&'static str, &'static str) = ($agent, $tool);
        })+
    };
}

operations!(1;
    DRAFT_1 => ("WRITER_A", "write_section"),
    DRAFT_2 => ("WRITER_A", "write_section"),
    DRAFT_3 => ("WRITER_B", "write_section"),
    WRITER_A_SNAPSHOTS => ("WRITER_A", "read_section"),
    WRITER_B_SNAPSHOTS => ("WRITER_B", "read_section"),
    REFERENCES_1 => ("WRITER_A", "update_references"),
    REFERENCES_2 => ("WRITER_A", "update_references"),
    REFERENCES_3 => ("WRITER_B", "update_references"),
    CREATE_1 => ("FIGURE_CREATOR", "create_figure"),
    CREATE_3 => ("FIGURE_CREATOR", "create_figure"),
    CHECK_12 => ("FACT_CHECKER", "fact_check"),
    CHECK_3 => ("FACT_CHECKER", "fact_check"),
    REVIEW_1 => ("EDITOR", "review_sections"),
    REVIEW_2 => ("EDITOR", "review_sections"),
    REVIEW_3 => ("EDITOR", "review_sections"),
    REVISE_A => ("WRITER_A", "revise_section"),
    REVISE_B => ("WRITER_B", "revise_section"),
    COMBINE => ("EDITOR", "combine_sections"),
);
operations!(2;
    INSERT_1 => ("FIGURE_CREATOR", "insert_figure"),
    INSERT_3 => ("FIGURE_CREATOR", "insert_figure"),
);

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
