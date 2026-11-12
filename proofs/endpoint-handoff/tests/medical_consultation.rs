//! Compile and drive the actual 5E sources without unrelated task globals.
#[path = "../../../src/imp/json.rs"]
pub(crate) mod json;
#[path = "../../../src/imp/patient.rs"]
pub(crate) mod patient;
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{json, patient, projection, transport};
}
#[path = "../../../src/tasks/medical_consultation/mod.rs"]
pub mod medical_consultation;
