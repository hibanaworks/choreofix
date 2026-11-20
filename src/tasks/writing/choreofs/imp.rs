//! Original task inputs and wire bytes, without workflow or permission state.
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Assignment {
    pub agent: &'static str,
    pub topic: &'static str,
    pub section: &'static str,
    pub sections: [&'static str; 2],
}

/// Immutable original task data. Each role identifies one 3/5/7-person input.
pub const fn assignment(role: u8) -> Assignment {
    let (agent, topic, section, sections) = match role {
        0 => (
            "RESEARCHER_A",
            "intro_methods_connection",
            "sec_intro_methods",
            ["INTRO_SECTION", "METHODS_SECTION"],
        ),
        1 => (
            "RESEARCHER_B",
            "methods_results_connection",
            "sec_methods_results",
            ["METHODS_SECTION", "RESULTS_SECTION"],
        ),
        2 => (
            "RESEARCHER_C",
            "results_intro_analysis",
            "sec_results_intro",
            ["INTRO_SECTION", "RESULTS_SECTION"],
        ),
        3 => (
            "RESEARCHER_A",
            "intro_background_connection",
            "sec_intro_background",
            ["BACKGROUND_SECTION", "INTRO_SECTION"],
        ),
        4 => (
            "RESEARCHER_B",
            "background_methods_connection",
            "sec_background_methods",
            ["BACKGROUND_SECTION", "METHODS_SECTION"],
        ),
        5 => (
            "RESEARCHER_C",
            "methods_experiments_connection",
            "sec_methods_experiments",
            ["EXPERIMENTS_SECTION", "METHODS_SECTION"],
        ),
        6 => (
            "RESEARCHER_D",
            "experiments_results_connection",
            "sec_experiments_results",
            ["EXPERIMENTS_SECTION", "RESULTS_SECTION"],
        ),
        7 => (
            "RESEARCHER_E",
            "results_intro_analysis",
            "sec_results_intro",
            ["INTRO_SECTION", "RESULTS_SECTION"],
        ),
        8 => (
            "RESEARCHER_A",
            "intro_background_connection",
            "sec_intro_background",
            ["BACKGROUND_SECTION", "INTRO_SECTION"],
        ),
        9 => (
            "RESEARCHER_B",
            "background_related_connection",
            "sec_background_related",
            ["BACKGROUND_SECTION", "RELATED_SECTION"],
        ),
        10 => (
            "RESEARCHER_C",
            "related_methods_connection",
            "sec_related_methods",
            ["METHODS_SECTION", "RELATED_SECTION"],
        ),
        11 => (
            "RESEARCHER_D",
            "methods_experiments_connection",
            "sec_methods_experiments",
            ["EXPERIMENTS_SECTION", "METHODS_SECTION"],
        ),
        12 => (
            "RESEARCHER_E",
            "experiments_results_connection",
            "sec_experiments_results",
            ["EXPERIMENTS_SECTION", "RESULTS_SECTION"],
        ),
        13 => (
            "RESEARCHER_F",
            "results_discussion_connection",
            "sec_results_discussion",
            ["DISCUSSION_SECTION", "RESULTS_SECTION"],
        ),
        14 => (
            "RESEARCHER_G",
            "discussion_intro_connection",
            "sec_discussion_intro",
            ["DISCUSSION_SECTION", "INTRO_SECTION"],
        ),
        _ => panic!("unknown writing task input"),
    };
    Assignment {
        agent,
        topic,
        section,
        sections,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const ROLE: u8, const OPERATION: u8>(Vec<u8>);
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 0> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "read_literature");
}
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 1> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "draft_content");
}
impl<const ROLE: u8> SimulatorOperation<2> for Arguments<ROLE, 2> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "write_to_document");
}
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 3> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "review_own_work");
}
impl<const ROLE: u8, const OPERATION: u8> WireEncode for Arguments<ROLE, OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const ROLE: u8, const OPERATION: u8> WirePayload for Arguments<ROLE, OPERATION> {
    const SCHEMA_ID: u32 = 0x5752_0000 | ((ROLE as u32) << 8) | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let input = assignment(ROLE);
        let key = match OPERATION {
            0 => "topic",
            1 | 3 => "section_name",
            2 => "sections",
            _ => panic!("unknown writing request"),
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        let text = value[key].as_str().ok_or(CodecError::Malformed)?;
        if object.len() != 1 || !object.contains_key(key) {
            return Err(CodecError::Malformed);
        }
        let valid = if OPERATION == 2 {
            let mut actual: Vec<_> = text.split(',').map(str::trim).collect();
            let mut expected = input.sections;
            actual.sort_unstable();
            expected.sort_unstable();
            actual == expected
        } else {
            text == if OPERATION == 0 {
                input.topic
            } else {
                input.section
            }
        };
        if !valid {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

/// A writing submission can be forwarded to the repository or a module owner.
/// Only the request received from a section can be spent on the document write.
#[derive(Debug)]
pub struct WriteSubmission<const ROLE: u8> {
    pub arguments: Arguments<ROLE, 2>,
}
impl<const ROLE: u8> WriteSubmission<ROLE> {
    pub fn value(&self) -> Value {
        serde_json::from_slice(&self.arguments.0).expect("validated writing submission")
    }
}
impl<const ROLE: u8> WireEncode for WriteSubmission<ROLE> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.arguments.encode_into(output)
    }
}
impl<const ROLE: u8> WirePayload for WriteSubmission<ROLE> {
    const SCHEMA_ID: u32 = 0x5753_0002 | ((ROLE as u32) << 8);
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<ROLE, 2>::validate_payload(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self {
            arguments: Arguments(input.as_bytes().to_vec()),
        }
    }
}

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn result(value: &Value, tool: &str, agent: &str) -> ShellResult<()> {
    if value["tool_name"] != tool
        || value["data"]["agent"] != agent
        || !value["success"].is_boolean()
    {
        return Err(ShellError::message(
            "effect result differs from its actual request",
        ));
    }
    Ok(())
}
