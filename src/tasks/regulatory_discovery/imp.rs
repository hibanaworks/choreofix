use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

pub use super::receipts::{actual, artifact, compound, decode, encode, inputs};

pub fn result(receipt: &Value, request: (&str, &str, &Value)) -> ShellResult<()> {
    if receipt["success"] == true {
        return actual(receipt, request);
    }
    super::receipts::request_matches(receipt, request)?;
    let data = &receipt["data"];
    if receipt["success"] != false
        || data["status"] != "injected_failure"
        || !data["event_id"].is_u64()
        || !data["artifact_id"].is_null()
        || !data["missing_inputs"].as_array().is_some_and(Vec::is_empty)
    {
        return Err(ShellError::message(
            "regulatory discovery returned an invalid failure receipt",
        ));
    }
    Ok(())
}

pub fn select_biology(arguments: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = arguments.take();
    let selected = match value["biology"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    arguments.set(value);
    selected
}

pub fn select_toxicology(arguments: &Cell<Value>) -> Result<DecisionArm, ResolverError> {
    let value = arguments.take();
    let selected = match value["toxicology"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    arguments.set(value);
    selected
}
