use choreofix::{imp, shell};

#[path = "../../../src/tasks/multi_round_discovery/mod.rs"]
pub mod multi_round_discovery;

#[test]
fn multi_round_direct_file_source_has_no_projection_diagnostic() {
    assert_eq!(
        hibana::g::diagnose(&multi_round_discovery::global::discovery()),
        None
    );
}
