//! Component checks for the new 6H source; these are not whole-task qualification.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/staged_integration/mod.rs"]
pub mod staged_integration;

#[test]
fn staged_integration_whole_source_static_contract() {
    assert_eq!(
        hibana::g::diagnose(&staged_integration::global::source()),
        None
    );
}
#[test]
fn staged_integration_source_projects_all_resource_file_and_wasm_views() {
    let observed = staged_integration::global::programs().observation();
    let programs = observed["programs"].as_array().unwrap();
    assert_eq!(programs.len(), 46);
    assert_eq!(programs.len(), observed["roles"].as_object().unwrap().len());
    let count = programs[0]["rows"][0][3].as_u64().unwrap() >> 16;
    println!(
        "staged integration source: views={} events={count}",
        programs.len()
    );
    assert!(count > 600);
    for program in programs {
        let rows = program["rows"].as_array().unwrap();
        assert!(rows.len() > 2);
        assert_eq!(rows[0][3].as_u64().unwrap() >> 16, count);
    }
}
#[test]
fn staged_integration_operation_paths_fit_actual_wasip1_requests() {
    use staged_integration::global::*;
    for file in DEVELOPER_A_FILES
        .into_iter()
        .chain(DEVELOPER_B_FILES)
        .chain(DEVELOPER_C_FILES)
        .chain(REVIEW_FILES)
        .chain(TEST_FILES)
        .chain(CI_FILES)
        .chain(RELEASE_FILES)
    {
        assert!(file.path().len() < hibana_wasip1_runtime::protocol::WASIP1_PATH_CHUNK_CAPACITY);
    }
}
