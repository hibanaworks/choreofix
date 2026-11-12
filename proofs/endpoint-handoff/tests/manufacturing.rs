//! Compile and drive the canonical manufacturing global and its actual receivers.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/manufacturing/mod.rs"]
pub mod manufacturing;
pub(crate) use manufacturing::{global, imp as manufacturing_data, localside};
#[path = "../../../src/tasks/manufacturing/tests.rs"]
mod tests;
