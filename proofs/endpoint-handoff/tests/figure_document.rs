//! Compile the actual 7M document/figure boundary without unrelated task globals.
pub use choreofix::{imp, shell};

#[path = "../../../src/tasks/figure_document/mod.rs"]
pub mod figure_document;
