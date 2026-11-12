//! The production 14H instrument receivers, without unrelated task globals.
#[path = "../../../src/imp/json.rs"]
pub(crate) mod json;
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{json, projection, transport};
}
#[path = "../../../src/tasks/multi_round_discovery/mod.rs"]
pub mod multi_round_discovery;

#[cfg(miri)]
#[path = "../../../src/tasks/handoff/tests/clock.rs"]
mod clock;
