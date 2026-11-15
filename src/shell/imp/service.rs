use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

pub trait Domain {
    fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>>;
}

/// The resolver reads only the actual validation result for this close.
pub struct Validation(Cell<Option<ShellResult<Value>>>);
impl Validation {
    pub fn new() -> Self {
        Self(Cell::new(None))
    }
    pub fn record(&self, result: ShellResult<Value>) {
        self.0.replace(Some(result));
    }
    pub fn take(&self) -> ShellResult<Value> {
        self.0
            .take()
            .ok_or_else(|| ShellError::message("missing validation result"))?
    }
    pub fn rejection(&self) -> ShellResult<String> {
        let result = self.0.take();
        let rejection = match result.as_ref() {
            Some(Err(error)) => Ok(error.to_string()),
            _ => Err(ShellError::message("missing argument rejection")),
        };
        self.0.set(result);
        rejection
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        let result = self.0.take();
        let decision = match result.as_ref() {
            Some(Ok(_)) => Ok(DecisionArm::Left),
            Some(Err(_)) => Ok(DecisionArm::Right),
            None => Err(ResolverError::reject()),
        };
        self.0.set(result);
        decision
    }
}
impl Default for Validation {
    fn default() -> Self {
        Self::new()
    }
}
