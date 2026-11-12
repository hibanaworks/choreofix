//! Repository request data and the actual selected review target.
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use std::cell::Cell;
#[derive(Default)]
pub struct ReviewTarget(Cell<Option<&'static str>>);
impl ReviewTarget {
    pub fn record(&self, feature: &'static str) {
        self.0.set(Some(feature));
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        match self.0.get() {
            Some(super::global::FEATURE_A) => Ok(DecisionArm::Left),
            Some(super::global::FEATURE_B) => Ok(DecisionArm::Right),
            _ => Err(ResolverError::reject()),
        }
    }
}
