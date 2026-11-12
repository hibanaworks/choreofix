//! Actual paper inputs and content-bound evidence. No protocol operations.
pub use crate::imp::json::{decode, encode, same};
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};

pub const AUTHORS: [&str; 3] = ["RESEARCHER_A", "RESEARCHER_B", "RESEARCHER_C"];
pub const SECTIONS: [&str; 3] = ["section_a", "section_b", "section_c"];
pub const TOPICS: [&str; 3] = ["subtopic_a", "subtopic_b", "subtopic_c"];

pub(crate) fn tool(operation: u8) -> ShellResult<&'static str> {
    match operation {
        1 => Ok("research_topic"),
        2 => Ok("write_section"),
        3 => Ok("revise_section"),
        4 => Ok("update_references"),
        5 => Ok("fact_check"),
        6 => Ok("review_sections"),
        _ => Err(ShellError::message("unknown paper operation")),
    }
}

pub fn arguments(author: u8, operation: u8, value: &Value) -> ShellResult<()> {
    let section = SECTIONS
        .get(usize::from(author))
        .ok_or_else(|| ShellError::message("unknown paper author"))?;
    tool(operation)?;
    let key = if operation == 1 {
        "topic"
    } else {
        "section_name"
    };
    let optional = match operation {
        2 => Some("content_notes"),
        3 => Some("feedback"),
        _ => None,
    };
    let fields = value
        .as_object()
        .ok_or_else(|| ShellError::message("paper arguments must be an object"))?;
    if fields.get(key).is_none_or(|value| {
        value
            .as_str()
            .is_none_or(|value| value.trim().is_empty() || (operation != 1 && value != *section))
    }) || fields
        .iter()
        .any(|(name, value)| (name != key && Some(name.as_str()) != optional) || !value.is_string())
    {
        return Err(ShellError::message(
            "paper arguments name another section or operation",
        ));
    }
    Ok(())
}

/// Exact admitted bytes, including the author's freely chosen notes/feedback.
pub struct Arguments<const AUTHOR: u8, const OPERATION: u8>(Vec<u8>);
impl<const AUTHOR: u8, const OPERATION: u8> Arguments<AUTHOR, OPERATION> {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("invalid paper request"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated paper arguments")
    }
}
impl<const AUTHOR: u8, const OPERATION: u8> WireEncode for Arguments<AUTHOR, OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
pub(crate) const fn arguments_schema(author: u8, operation: u8) -> u32 {
    0xac30_0000 | ((author as u32) << 8) | operation as u32
}
impl<const AUTHOR: u8, const OPERATION: u8> WirePayload for Arguments<AUTHOR, OPERATION> {
    const SCHEMA_ID: u32 = arguments_schema(AUTHOR, OPERATION);
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        arguments(AUTHOR, OPERATION, &value).map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

pub fn result(author: u8, operation: u8, value: &Value) -> ShellResult<()> {
    let name = AUTHORS
        .get(usize::from(author))
        .ok_or_else(|| ShellError::message("unknown paper author"))?;
    let expected_actor: &str = match operation {
        5 => "FACT_CHECKER",
        6 => "EDITOR",
        _ => name,
    };
    let target = match operation {
        1 => value["data"]["topic"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty()),
        6 => value["data"]["scope"] == SECTIONS[usize::from(author)],
        _ => value["data"]["section"] == SECTIONS[usize::from(author)],
    };
    if value["tool_name"] != tool(operation)?
        || !value["success"].is_boolean()
        || value["data"]["agent"] != expected_actor
        || !target
    {
        return Err(ShellError::message(
            "paper result differs from its actual request",
        ));
    }
    Ok(())
}

/// An actual owned result; sends borrow this value and receives own it.
pub struct Reply<const AUTHOR: u8, const OPERATION: u8>(Value);
impl<const AUTHOR: u8, const OPERATION: u8> Reply<AUTHOR, OPERATION> {
    pub fn new(value: Value) -> ShellResult<Self> {
        result(AUTHOR, OPERATION, &value)?;
        Ok(Self(value))
    }
    pub fn into_value(self) -> Value {
        self.0
    }
}
fn encode_value(value: &Value, out: &mut [u8]) -> Result<usize, CodecError> {
    let mut output = std::io::Cursor::new(out);
    serde_json::to_writer(&mut output, value).map_err(|_| CodecError::Truncated)?;
    Ok(output.position() as usize)
}

/// Current chapter material and its content-bound evidence, owned by the receiver.
pub struct Material(Value);
impl Material {
    pub fn new(value: Value) -> ShellResult<Self> {
        if !value.is_object() {
            return Err(ShellError::message("chapter material must be an object"));
        }
        Ok(Self(value))
    }
    pub fn into_value(self) -> Value {
        self.0
    }
}
impl WireEncode for Material {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        encode_value(&self.0, out)
    }
}
impl WirePayload for Material {
    const SCHEMA_ID: u32 = 0xac33_0000;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if !value.is_object() {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(serde_json::from_slice(input.as_bytes()).expect("validated chapter material"))
    }
}
impl<const AUTHOR: u8, const OPERATION: u8> WireEncode for Reply<AUTHOR, OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        encode_value(&self.0, out)
    }
}
impl<const AUTHOR: u8, const OPERATION: u8> WirePayload for Reply<AUTHOR, OPERATION> {
    const SCHEMA_ID: u32 = 0xac31_0000 | ((AUTHOR as u32) << 8) | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value = serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        result(AUTHOR, OPERATION, &value).map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(serde_json::from_slice(input.as_bytes()).expect("validated paper reply"))
    }
}

/// Positive/negative result data is checked against its wire contract.
pub struct Verdict<const AUTHOR: u8, const OPERATION: u8, const SUCCESS: bool>(Value);
impl<const AUTHOR: u8, const OPERATION: u8, const SUCCESS: bool>
    Verdict<AUTHOR, OPERATION, SUCCESS>
{
    pub fn new(value: Value) -> ShellResult<Self> {
        result(AUTHOR, OPERATION, &value)?;
        if value["success"] != SUCCESS {
            return Err(ShellError::message("paper verdict differs from its branch"));
        }
        Ok(Self(value))
    }
    pub fn into_value(self) -> Value {
        self.0
    }
}
impl<const AUTHOR: u8, const OPERATION: u8, const SUCCESS: bool> WireEncode
    for Verdict<AUTHOR, OPERATION, SUCCESS>
{
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        encode_value(&self.0, out)
    }
}
impl<const AUTHOR: u8, const OPERATION: u8, const SUCCESS: bool> WirePayload
    for Verdict<AUTHOR, OPERATION, SUCCESS>
{
    const SCHEMA_ID: u32 =
        0xac32_0000 | ((AUTHOR as u32) << 8) | ((OPERATION as u32) << 1) | SUCCESS as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let actual: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        result(AUTHOR, OPERATION, &actual).map_err(|_| CodecError::Malformed)?;
        if actual["success"] != SUCCESS {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(serde_json::from_slice(input.as_bytes()).expect("validated paper verdict"))
    }
}

pub fn research(author: u8, value: &Value) -> ShellResult<()> {
    result(author, 1, value)?;
    if value["success"] != true
        || value["data"]["topic"] != TOPICS[usize::from(author)]
        || !value["data"]["artifact"]["id"].is_string()
        || !value["data"]["artifact"]["content"].is_string()
    {
        return Err(ShellError::message(
            "assigned research needs its actual completed artifact",
        ));
    }
    Ok(())
}

pub fn material(
    author: u8,
    operation: u8,
    actual: &Value,
    (request, researched): (&Value, &Value),
) -> ShellResult<()> {
    arguments(author, operation, request)?;
    research(author, researched)?;
    result(author, operation, actual)?;
    if !matches!(operation, 2..=4)
        || actual["success"] != true
        || !actual["data"]["artifact"]["id"].is_string()
        || !actual["data"]["artifact"]["content"].is_string()
        || actual["data"]["artifact"]["research_inputs"]
            != json!({TOPICS[usize::from(author)]: researched["data"]["artifact"]})
        || request
            .get("content_notes")
            .is_some_and(|notes| actual["data"]["artifact"]["content"] != *notes)
        || request
            .get("feedback")
            .is_some_and(|feedback| actual["data"]["artifact"]["feedback"] != *feedback)
    {
        return Err(ShellError::message(
            "paper material is not bound to its admitted input and research",
        ));
    }
    Ok(())
}

pub fn snapshot(author: u8, written: &Value, references: &Value) -> ShellResult<Value> {
    let operation = match written["tool_name"].as_str() {
        Some("write_section") => 2,
        Some("revise_section") => 3,
        _ => {
            return Err(ShellError::message(
                "section snapshot lacks actual writing or revision",
            ));
        }
    };
    result(author, operation, written)?;
    result(author, 4, references)?;
    if written["success"] != true
        || references["success"] != true
        || !written["data"]["artifact"]["id"].is_string()
        || !references["data"]["artifact"]["id"].is_string()
        || written["data"]["artifact"]["id"] == references["data"]["artifact"]["id"]
        || written["data"]["artifact"]["research_inputs"]
            != references["data"]["artifact"]["research_inputs"]
    {
        return Err(ShellError::message(
            "section and references need the same completed research",
        ));
    }
    let snapshot = json!({"sections":{SECTIONS[usize::from(author)]: written["data"]["artifact"]},
        "references":{SECTIONS[usize::from(author)]: references["data"]["artifact"]}});
    check_snapshot(author, &snapshot)?;
    Ok(snapshot)
}

fn check_snapshot(author: u8, snapshot: &Value) -> ShellResult<()> {
    let section = SECTIONS
        .get(usize::from(author))
        .ok_or_else(|| ShellError::message("unknown paper author"))?;
    if snapshot.as_object().is_none_or(|fields| fields.len() != 2) {
        return Err(ShellError::message(
            "fact evidence needs exactly this chapter and references",
        ));
    }
    for category in ["sections", "references"] {
        let artifact = &snapshot[category][*section];
        let research = &artifact["research_inputs"];
        let input = &research[TOPICS[usize::from(author)]];
        if snapshot[category]
            .as_object()
            .is_none_or(|fields| fields.len() != 1)
            || !artifact["id"].is_string()
            || !artifact["content"].is_string()
            || research.as_object().is_none_or(|fields| fields.len() != 1)
            || !input["id"].is_string()
            || !input["content"].is_string()
        {
            return Err(ShellError::message(
                "incomplete or foreign chapter material",
            ));
        }
    }
    if snapshot["sections"][*section]["research_inputs"]
        != snapshot["references"][*section]["research_inputs"]
    {
        return Err(ShellError::message(
            "chapter and reference research differs",
        ));
    }
    Ok(())
}

pub fn checked(author: u8, actual: &Value, snapshot: &Value) -> ShellResult<()> {
    check_snapshot(author, snapshot)?;
    result(author, 5, actual)?;
    if !actual["data"]["check_id"].is_string() || actual["data"]["input_snapshot"] != *snapshot {
        return Err(ShellError::message("fact check names other paper material"));
    }
    Ok(())
}

pub fn reviewed(
    author: u8,
    actual: &Value,
    (snapshot, facts): (&Value, &Value),
) -> ShellResult<()> {
    checked(author, facts, snapshot)?;
    result(author, 6, actual)?;
    if facts["success"] != true
        || !actual["data"]["review_id"].is_string()
        || actual["data"]["input_snapshot"] != *snapshot
        || actual["data"]["fact_receipts"]
            != json!({SECTIONS[usize::from(author)]:facts["data"]["check_id"]})
    {
        return Err(ShellError::message(
            "editorial result lacks this section's actual positive fact check",
        ));
    }
    Ok(())
}

/// Data validation for the original 3M combine call. No progress, permissions
/// or completed-author flags are reconstructed here; all inputs are the actual
/// material and full validation results retained by their effect owners.
pub fn publication(
    arguments: &Value,
    (sections, references): (&Value, &Value),
    reviews: &Value,
) -> ShellResult<Value> {
    let requested = arguments["sections"].as_str().ok_or_else(|| {
        ShellError::message("paper publication needs the original sections string")
    })?;
    let mut requested: Vec<_> = requested
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    requested.sort_unstable();
    if arguments.as_object().is_none_or(|fields| fields.len() != 1)
        || requested != SECTIONS
        || reviews.as_object().is_none_or(|fields| fields.len() != 2)
    {
        return Err(ShellError::message(
            "publication differs from the three actual paper sections",
        ));
    }
    for values in [sections, references, &reviews["facts"], &reviews["reviews"]] {
        if values.as_object().is_none_or(|fields| {
            fields.len() != 3
                || SECTIONS
                    .iter()
                    .any(|section| !fields.contains_key(*section))
        }) {
            return Err(ShellError::message(
                "paper publication is missing actual material or reviews",
            ));
        }
    }
    let mut identities = Vec::new();
    for author in 0..3 {
        let name = SECTIONS[usize::from(author)];
        let snapshot =
            json!({"sections":{name:sections[name]},"references":{name:references[name]}});
        let facts = &reviews["facts"][name];
        let editorial = &reviews["reviews"][name];
        reviewed(author, editorial, (&snapshot, facts))?;
        if editorial["success"] != true {
            return Err(ShellError::message(
                "publication contains a rejected editorial review",
            ));
        }
        for id in [
            &sections[name]["id"],
            &references[name]["id"],
            &facts["data"]["check_id"],
            &editorial["data"]["review_id"],
        ] {
            let id = id
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(|| {
                    ShellError::message("publication evidence has no actual identity")
                })?;
            if identities.contains(&id) {
                return Err(ShellError::message(
                    "a publication receipt was reused for another input",
                ));
            }
            identities.push(id);
        }
    }
    Ok(json!({"sections":sections,"references":references}))
}

pub fn published(
    actual: &Value,
    (arguments, snapshot): (&Value, &Value),
    reviews: &Value,
) -> ShellResult<()> {
    let sections: Vec<_> = arguments["sections"]
        .as_str()
        .ok_or_else(|| ShellError::message("missing publication arguments"))?
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    let facts: serde_json::Map<String, Value> = SECTIONS
        .iter()
        .map(|name| {
            (
                name.to_string(),
                reviews["facts"][*name]["data"]["check_id"].clone(),
            )
        })
        .collect();
    let editorial: serde_json::Map<String, Value> = SECTIONS
        .iter()
        .map(|name| {
            (
                name.to_string(),
                reviews["reviews"][*name]["data"]["review_id"].clone(),
            )
        })
        .collect();
    if actual["tool_name"] != "combine_sections"
        || actual["success"] != true
        || actual["data"]["agent"] != "EDITOR"
        || actual["data"]["status"] != "combined"
        || actual["data"]["artifact_id"]
            .as_str()
            .is_none_or(|id| id.trim().is_empty())
        || actual["data"]["sections"] != json!(sections)
        || actual["data"]["input_snapshot"] != *snapshot
        || actual["data"]["fact_receipts"] != Value::Object(facts)
        || actual["data"]["review_receipts"] != Value::Object(editorial)
        || actual
            .get("violations")
            .is_some_and(|values| values.as_array().is_none_or(|values| !values.is_empty()))
    {
        return Err(ShellError::message(
            "actual publication result differs from the fenced paper and reviews",
        ));
    }
    Ok(())
}
