//! Exercise canonical 5H bodies without unrelated application globals.
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
#[path = "../../../src/tasks/critical_care/mod.rs"]
pub mod critical_care;
