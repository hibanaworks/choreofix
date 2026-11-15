//! The actual operation-file global and every sealed role image.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/feature_delivery/mod.rs"]
pub mod feature_delivery;

#[test]
fn feature_delivery_whole_source_static_contract() {
    assert_eq!(
        hibana::g::diagnose(&feature_delivery::global::source()),
        None
    );
}
#[test]
fn feature_delivery_whole_source_projects_every_view() {
    let observed = feature_delivery::projection();
    let programs = observed["programs"].as_array().unwrap();
    assert_eq!(programs.len(), observed["roles"].as_object().unwrap().len());
    assert_eq!(programs.len(), 31);
    let events = programs[0]["rows"][0][3].as_u64().unwrap() >> 16;
    assert!(events > 0);
    for program in programs {
        let rows = program["rows"].as_array().unwrap();
        assert!(rows.len() > 2);
        assert_eq!(rows[0][3].as_u64().unwrap() >> 16, events);
    }
}

#[test]
fn feature_delivery_file_paths_fit_the_wasip1_request() {
    use feature_delivery::global::*;
    for object in DEVELOPER_FILES
        .into_iter()
        .chain(REVIEW_FILES)
        .chain(TEST_FILES)
        .chain(CI_FILES)
    {
        // The shell may use an absolute path, including its leading slash.
        assert!(
            object.path().len() + 1 <= hibana_wasip1_runtime::protocol::WASIP1_PATH_CHUNK_CAPACITY
        );
    }
}
