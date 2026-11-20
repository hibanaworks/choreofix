use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

/// The physical simulator operation named by this projected wire payload.
pub trait SimulatorOperation<const RECEIPTS: usize = 1>:
    hibana::runtime::wire::WirePayload
{
    const RPC: (&'static str, &'static str);
    /// JSON member containing the simulator arguments when the wire request also carries evidence.
    const ARGUMENT_FIELD: Option<&'static str> = None;
}

/// Effect I/O available to receivers whose message type fixes the operation.
/// A receipt supplies both the payload type and the bytes to submit.
///
/// ```compile_fail
/// use choreofix::shell::imp::service::ReceivedEffects;
/// fn fabricate(receiver: &impl ReceivedEffects) {
///     let _ = receiver.apply(serde_json::json!({"pr_id": "user_authentication"}));
/// }
/// ```
///
/// ```compile_fail
/// use choreofix::shell::imp::service::{Domain, ReceivedEffects};
/// fn substitute(receiver: &impl ReceivedEffects) {
///     let _ = receiver.execute("REVIEWER", "code_review", serde_json::json!({}));
/// }
/// ```
pub trait ReceivedEffects {
    /// Consumes the receive now. The simulator request is sent when the returned future is polled.
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, Self, P>;
}

/// A single actual resource receive. A receiver can retain its bytes when the
/// same Endpoint must receive a delegated right before the effect may run.
pub struct ResourceReceipt<'a, P> {
    bytes: std::borrow::Cow<'a, [u8]>,
    payload: core::marker::PhantomData<fn(&P) -> &P>,
}
impl<'a, P> From<hibana::runtime::wire::ReceivedPayload<'a, P>> for ResourceReceipt<'a, P> {
    fn from(received: hibana::runtime::wire::ReceivedPayload<'a, P>) -> Self {
        Self {
            bytes: received.as_bytes().into(),
            payload: core::marker::PhantomData,
        }
    }
}
impl<P> ResourceReceipt<'_, P> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn retain(
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> ResourceReceipt<'static, P> {
        ResourceReceipt {
            bytes: received.as_bytes().to_vec().into(),
            payload: core::marker::PhantomData,
        }
    }
}

/// An operation on two resources consumes two actual receives of the same request.
///
/// ```compile_fail
/// use choreofix::shell::imp::service::{PairedEffects, SimulatorOperation};
/// use hibana::runtime::wire::ReceivedPayload;
/// fn only_one<P: SimulatorOperation<2>>(io: &impl PairedEffects, one: ReceivedPayload<'_, P>) {
///     let _ = io.apply(one);
/// }
/// ```
///
/// ```compile_fail
/// use choreofix::shell::imp::service::{PairedEffects, SimulatorOperation};
/// use hibana::runtime::wire::ReceivedPayload;
/// fn reuse<P: SimulatorOperation<2>>(io: &impl PairedEffects, one: ReceivedPayload<'_, P>) {
///     let _ = io.apply_pair(one, one);
/// }
/// ```
pub trait PairedEffects: ReceivedEffects {
    fn apply_pair<'a, 'b, P: SimulatorOperation<2>>(
        &self,
        first: impl Into<ResourceReceipt<'a, P>>,
        second: impl Into<ResourceReceipt<'b, P>>,
    ) -> impl Future<Output = ShellResult<Value>>;
}

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
