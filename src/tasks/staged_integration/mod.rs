//! Direct staged-integration source and staging effect receiver.
//! The seven-Wasm execution and full task qualification are still being connected.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod staging;
}
use core::cell::Cell;
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;

pub fn actual_outcome(input: &Cell<Option<Value>>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let decision = match value.as_ref().and_then(|value| value["success"].as_bool()) {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    decision
}
