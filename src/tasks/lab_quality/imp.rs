//! Actual request/result data and payload validation; no protocol operations.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn same(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        return Err(ShellError::message(
            "laboratory request differs from admitted actual data",
        ));
    }
    Ok(())
}
pub fn assigned(value: &Value, key: &str, expected: &str) -> ShellResult<()> {
    if value.as_object().is_none_or(|object| object.len() != 1)
        || value[key].as_str() != Some(expected)
    {
        return Err(ShellError::message(
            "laboratory arguments differ from the assigned specimen",
        ));
    }
    Ok(())
}
pub fn successful(value: &Value) -> ShellResult<()> {
    if value["success"].as_bool() != Some(true) {
        return Err(ShellError::message("terminal laboratory operation failed"));
    }
    Ok(())
}
pub fn assay_arguments(value: &Value, compound: &Value) -> ShellResult<()> {
    let object = value
        .as_object()
        .ok_or_else(|| ShellError::message("expected biological trial arguments"))?;
    if object
        .keys()
        .any(|key| key != "compound" && key != "cell_type")
        || !compound.is_string()
        || value["compound"] != *compound
        || object
            .get("cell_type")
            .is_some_and(|value| !value.is_string())
    {
        return Err(ShellError::message(
            "biological trial must name the actual synthesized compound",
        ));
    }
    Ok(())
}

pub fn assay_file(
    file: &crate::shell::imp::file_wire::ClosedFile,
    compound: &Value,
) -> ShellResult<Value> {
    if file.completion.0.errno() != 0 || file.path != b"service/run_bioassay" {
        return Err(ShellError::message(
            "expected an actual biological trial file",
        ));
    }
    let value = decode(&file.bytes)?;
    assay_arguments(&value, compound)?;
    Ok(value)
}

pub fn qc_arguments(value: &Value, scientist: &str) -> ShellResult<()> {
    let data_type = match scientist {
        "CHEMIST" => "purification",
        "BIOLOGIST" => "bioassay",
        _ => return Err(ShellError::message("unknown QC subject")),
    };
    if value.as_object().is_none_or(|object| object.len() != 2)
        || value["scientist"] != scientist
        || value["data_type"] != data_type
    {
        return Err(ShellError::message(
            "QC arguments differ from the actual subject",
        ));
    }
    Ok(())
}

pub fn qc_file(file: &crate::shell::imp::file_wire::ClosedFile) -> ShellResult<Value> {
    if file.completion.0.errno() != 0 || file.path != b"service/verify_result" {
        return Err(ShellError::message("expected an actual QC request file"));
    }
    let value = decode(&file.bytes)?;
    let scientist = value["scientist"]
        .as_str()
        .ok_or_else(|| ShellError::message("expected the actual QC subject"))?;
    qc_arguments(&value, scientist)?;
    Ok(value)
}

pub fn qc_admission(
    admitted: &Value,
    selected: &Value,
    (synthesis, purification, evidence): (&Value, &Value, &Value),
) -> ShellResult<()> {
    same(&admitted["arguments"], selected)?;
    same(&admitted["sample"]["synthesis"], synthesis)?;
    same(&admitted["sample"]["purification"], purification)?;
    match selected["scientist"].as_str() {
        Some("CHEMIST") => same(&admitted["sample"]["analysis"], evidence),
        Some("BIOLOGIST") => same(&admitted["result"], evidence),
        _ => Err(ShellError::message("unknown actual QC subject")),
    }
}

pub fn qc_result(result: &Value, arguments: &Value) -> ShellResult<()> {
    if result["tool_name"] != "verify_result" || !result["success"].is_boolean() {
        return Err(ShellError::message("missing actual QC decision"));
    }
    same(&result["data"]["scientist"], &arguments["scientist"])?;
    same(&result["data"]["data_type"], &arguments["data_type"])
}

/// Only the actual result value is exposed to the result resolver.
pub fn decide(
    result: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = result.take();
    let decision = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    result.set(value);
    decision
}

/// Actual QC reply, bound to both its scientific subject and its verdict.
pub struct QcReply<'a, const WHO: u8, const APPROVED: bool>(&'a [u8]);
impl<'a, const WHO: u8, const APPROVED: bool> QcReply<'a, WHO, APPROVED> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes)).map_err(|_| {
            ShellError::message("QC reply differs from its actual subject or result")
        })?;
        Ok(Self(bytes))
    }
}
impl<'r, const WHO: u8, const APPROVED: bool> WireEncode for QcReply<'r, WHO, APPROVED> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<'r, const WHO: u8, const APPROVED: bool> WirePayload for QcReply<'r, WHO, APPROVED> {
    const SCHEMA_ID: u32 = 0xac13_0000 | ((WHO as u32) << 1) | APPROVED as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (scientist, data_type) = match WHO {
            super::global::CHEMIST => ("CHEMIST", "purification"),
            super::global::BIOLOGIST => ("BIOLOGIST", "bioassay"),
            _ => return Err(CodecError::Malformed),
        };
        if value["tool_name"] != "verify_result"
            || value["success"].as_bool() != Some(APPROVED)
            || value["data"]["scientist"] != scientist
            || value["data"]["data_type"] != data_type
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}
