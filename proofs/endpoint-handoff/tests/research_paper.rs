//! Compile the actual 3M development source, without unrelated task globals.
#[path = "../../../src/imp/json.rs"]
pub(crate) mod json;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{json, transport};
}
#[path = "../../../src/tasks/research_paper/mod.rs"]
pub mod research_paper;
