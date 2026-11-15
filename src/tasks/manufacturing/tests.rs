use super::{global::*, imp};
use hibana::runtime::wire::WireEncode;
use serde_json::{Value, json};

#[test]
fn operation_files_and_both_affine_machines_project_from_one_global() {
    assert_eq!(hibana::g::diagnose(&source()), None);
    let compiled = programs().observation();
    assert_eq!(compiled["programs"].as_array().unwrap().len(), 23);
}

#[test]
fn typed_job_and_report_arguments_preserve_actual_input_and_reject_substitution() {
    let bytes = br#" { "job_id" : "lathe_job_a" } "#;
    let admitted = imp::Arguments::<0>::from_bytes(bytes).unwrap();
    let mut output = [0; 128];
    let len = admitted.encode_into(&mut output).unwrap();
    assert_eq!(&output[..len], bytes);
    assert_eq!(admitted.into_value(), json!({"job_id":"lathe_job_a"}));
    assert!(imp::Arguments::<0>::from_bytes(br#"{"job_id":"lathe_job_b"}"#).is_err());
    assert!(imp::Arguments::<1>::from_bytes(br#"{"job_id":"lathe_job_b"}"#).is_ok());
    assert!(imp::Arguments::<2>::from_bytes(br#"{"job_id":"mill_job_a"}"#).is_ok());
    assert!(imp::Arguments::<3>::from_bytes(br#"{"job_id":"mill_job_b"}"#).is_ok());
    assert!(
        imp::Arguments::<2>::from_bytes(br#"{"job_id":"mill_job_a","owner":"TECHNICIAN_A"}"#)
            .is_err()
    );
    assert!(imp::Arguments::<3>::from_bytes(br#"{"job_id":0}"#).is_err());
    assert_eq!(
        imp::Arguments::<4>::from_bytes(b" { } ")
            .unwrap()
            .into_value(),
        json!({})
    );
    assert!(imp::Arguments::<4>::from_bytes(br#"{"scope":"all_jobs"}"#).is_err());
    assert!(imp::Arguments::<5>::from_bytes(b"{}").is_err());
}

#[test]
fn report_contents_must_be_the_four_distinct_actual_machine_results() {
    let jobs: Vec<Value> = imp::JOBS
        .iter()
        .enumerate()
        .map(|(index, id)| {
            json!({"tool_name":"do_job","success":true,
        "data":{"job_id":id,"agent":if index.is_multiple_of(2){"TECHNICIAN_A"}else{"TECHNICIAN_B"},
        "machine":if index<2{"LATHE"}else{"MILL"},"status":"completed"}})
        })
        .collect();
    let expected = imp::completed_jobs(&jobs).unwrap();
    let report = json!({"tool_name":"collect_reports","success":true,
        "data":{"agent":"SUPERVISOR","scope":"all_jobs","status":"collected","jobs":expected}});
    imp::report_result(&expected, &report).unwrap();
    assert!(imp::completed_jobs(&jobs[..3]).is_err());
    let mut duplicate = jobs.clone();
    duplicate[3] = duplicate[0].clone();
    assert!(imp::completed_jobs(&duplicate).is_err());
    let mut wrong_machine = jobs;
    wrong_machine[0]["data"]["machine"] = json!("MILL");
    assert!(imp::completed_jobs(&wrong_machine).is_err());
    let mut wrong_report = report;
    wrong_report["data"]["jobs"][0]["job_id"] = json!("unperformed_job");
    assert!(imp::report_result(&expected, &wrong_report).is_err());
}
