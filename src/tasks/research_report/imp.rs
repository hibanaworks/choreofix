//! Actual result data used by the registered resolvers.
use crate::shell::imp::review_reply::ReviewReply;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

/// The editor owns its actual requested section and replies received from the
/// review services. This stores input data, not a second protocol cursor.
pub struct ReviewTarget {
    requested: Cell<Option<&'static str>>,
    replies: [Cell<Option<Value>>; 2],
}
impl Default for ReviewTarget {
    fn default() -> Self {
        Self {
            requested: Cell::new(None),
            replies: std::array::from_fn(|_| Cell::new(None)),
        }
    }
}
impl ReviewTarget {
    pub fn record(&self, section: &'static str) {
        self.requested.set(Some(section));
    }
    pub fn received(&self, section: &'static str, bytes: &[u8]) -> ShellResult<()> {
        let index = match section {
            super::global::SECTION_A => 0,
            super::global::SECTION_B => 1,
            _ => return Err(ShellError::message("unassigned reviewed section")),
        };
        let reply: Value = serde_json::from_slice(bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        if !reply["success"].is_boolean() {
            return Err(ShellError::message("missing review outcome"));
        }
        self.replies[index].set(Some(reply));
        Ok(())
    }
    fn reply(&self, index: usize) -> Option<Value> {
        let value = self.replies[index].take();
        let copy = value.clone();
        self.replies[index].set(value);
        copy
    }
    pub fn approvals(&self) -> ShellResult<(Vec<u8>, Vec<u8>)> {
        let a = serde_json::to_vec(
            &self
                .reply(0)
                .ok_or_else(|| ShellError::message("missing section A review"))?,
        )
        .map_err(|error| ShellError::message(error.to_string()))?;
        let b = serde_json::to_vec(
            &self
                .reply(1)
                .ok_or_else(|| ShellError::message("missing section B review"))?,
        )
        .map_err(|error| ShellError::message(error.to_string()))?;
        ReviewReply::<true>::from_bytes(&a)?;
        ReviewReply::<true>::from_bytes(&b)?;
        Ok((a, b))
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        let (index, arm) = match self.requested.get() {
            Some(super::global::SECTION_A) => (0, DecisionArm::Left),
            Some(super::global::SECTION_B) => (1, DecisionArm::Right),
            _ => return Err(ResolverError::reject()),
        };
        if self
            .reply(index)
            .is_some_and(|reply| reply["success"] == true)
        {
            return Err(ResolverError::reject());
        }
        Ok(arm)
    }
}

#[derive(Default)]
pub struct ReviewInput(Cell<Option<Value>>);
impl ReviewInput {
    pub fn record(&self, result: Value) {
        self.0.set(Some(result));
    }
}
pub fn decide(input: &ReviewInput) -> Result<DecisionArm, ResolverError> {
    let result = input.0.take();
    let decision = match result.as_ref().and_then(|reply| reply["success"].as_bool()) {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.0.set(result);
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_approvals_come_from_both_actual_received_positive_replies() {
        let input = ReviewTarget::default();
        assert!(input.approvals().is_err());
        input
            .received(super::super::global::SECTION_A, br#"{"success":true}"#)
            .unwrap();
        input
            .received(super::super::global::SECTION_B, br#"{"success":false}"#)
            .unwrap();
        assert!(input.approvals().is_err());
        input.record(super::super::global::SECTION_A);
        assert!(input.decide().is_err());
        input.record(super::super::global::SECTION_B);
        assert_eq!(input.decide(), Ok(DecisionArm::Right));
        input
            .received(super::super::global::SECTION_B, br#"{"success":true}"#)
            .unwrap();
        let (a, b) = input.approvals().unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&a).unwrap()["success"],
            true
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&b).unwrap()["success"],
            true
        );
        assert!(input.decide().is_err());
    }
}
