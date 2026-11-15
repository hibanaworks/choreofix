use super::imp::Arguments;
use crate::imp::patient;
use serde_json::{Value, json};
fn encoded<const OP: u8>(value: Value) -> bool {
    Arguments::<OP>::from_bytes(&serde_json::to_vec(&value).unwrap()).is_ok()
}
#[test]
fn operation_payloads_distinguish_actual_patient_test_and_resource_arguments() {
    assert!(encoded::<0>(json!({"patient_id":"patient_1"})));
    assert!(encoded::<1>(
        json!({"patient_id":"patient_1","test_type":"basic_labs,vitals"})
    ));
    assert!(encoded::<2>(
        json!({"patient_id":"patient_1","test_type":"ecg"})
    ));
    assert!(encoded::<3>(
        json!({"patient_id":"patient_1","test_type":"nerve_conduction,neuro_exam"})
    ));
    assert!(!encoded::<1>(
        json!({"patient_id":"patient_1","test_type":"ecg"})
    ));
    assert!(!encoded::<2>(
        json!({"patient_id":"patient_1","test_type":"neuro_exam"})
    ));
    assert!(!encoded::<3>(
        json!({"patient_id":"patient_1","test_type":"vitals"})
    ));
    assert!(encoded::<4>(
        json!({"patient_id":"patient_1","modality":"CT"})
    ));
    assert!(!encoded::<4>(
        json!({"patient_id":"patient_1","modality":"MRI"})
    ));
    assert!(encoded::<5>(
        json!({"patient_id":"patient_1","modality":"ct"})
    ));
    assert!(!encoded::<5>(json!({"patient_id":"other","modality":"CT"})));
    assert!(encoded::<6>(
        json!({"patient_id":"patient_1","modality":"MRI"})
    ));
    assert!(!encoded::<6>(
        json!({"patient_id":"patient_1","modality":"CT"})
    ));
    assert!(encoded::<7>(
        json!({"patient_id":"patient_1","modality":"mri"})
    ));
    assert!(!encoded::<7>(
        json!({"patient_id":"patient_1","modality":"PET"})
    ));
    assert!(encoded::<8>(
        json!({"patient_id":"patient_1","symptoms":"actual caller data"})
    ));
    assert!(!encoded::<8>(
        json!({"patient_id":"patient_1","symptoms":[]})
    ));
    assert!(encoded::<9>(json!({"patient_id":"patient_1"})));
    assert!(!encoded::<9>(json!({"patient_id":"patient_2"})));
    assert!(encoded::<10>(
        json!({"patient_id":"patient_1","medications":"caller data"})
    ));
    assert!(!encoded::<10>(
        json!({"patient_id":"patient_1","medications":[]})
    ));
    assert!(encoded::<11>(
        json!({"patient_id":"patient_1","attendees":"caller data"})
    ));
    assert!(!encoded::<11>(
        json!({"patient_id":"patient_1","attendees":[]})
    ));
    assert!(!encoded::<12>(
        json!({"arguments":{"patient_id":"other"},"case":{}})
    ));
}
#[test]
fn invalid_panel_updates_do_not_replace_actual_completed_data() {
    let result = json!({"success":true,"data":{"patient_id":"patient_1","tests":["ecg"],"artifact_id":"ecg:0","inputs":{"assessment":"assessment:0"}}});
    let mut records = Value::Null;
    patient::record_panel(&mut records, &result, &["ecg", "cardiac_enzymes"]).unwrap();
    let before = records.clone();
    let mut wrong = result;
    wrong["data"]["patient_id"] = json!("other");
    assert!(patient::record_panel(&mut records, &wrong, &["ecg", "cardiac_enzymes"]).is_err());
    assert_eq!(records, before);
}
