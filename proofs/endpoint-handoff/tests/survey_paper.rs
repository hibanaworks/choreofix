//! Compile the actual 3H editor boundary without unrelated task globals.
pub use choreofix::{imp, shell};

#[path = "../../../src/tasks/survey_paper/mod.rs"]
pub mod survey_paper;
