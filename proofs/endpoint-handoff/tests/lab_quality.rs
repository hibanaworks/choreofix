//! Production 13M Endpoint/data boundaries; no application Miri.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/lab_quality/mod.rs"]
pub mod lab_quality;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
