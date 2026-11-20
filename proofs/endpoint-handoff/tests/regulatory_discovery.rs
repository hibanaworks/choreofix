use choreofix::{imp, shell};

#[path = "../../../src/tasks/drug_discovery/arguments.rs"]
pub mod arguments;
#[path = "../../../src/tasks/regulatory_discovery/global.rs"]
pub mod global;
#[path = "../../../src/tasks/regulatory_discovery/reanalysis.rs"]
pub mod reanalysis;

#[test]
fn regulatory_review_projects_all_reanalysis_choices() {
    assert_eq!(hibana::g::diagnose(&global::discovery()), None);
    let projection = global::programs().observation();
    let programs = projection["programs"].as_array().unwrap();
    assert_eq!(
        programs.len(),
        projection["roles"].as_object().unwrap().len()
    );
    for role in [
        global::BIO_FILE,
        global::TOX_FILE,
        global::LEAD_FILE,
        global::REG_FILE,
    ] {
        let file = programs
            .iter()
            .find(|program| program["role"] == role)
            .unwrap();
        assert!(!file["rows"].as_array().unwrap().is_empty());
    }
}
