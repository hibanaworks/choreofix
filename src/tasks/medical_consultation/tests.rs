//! Typed file inputs and actual panel data. Real Wasm covers the whole workflow.
use super::imp::Arguments;
use crate::imp::patient;
use serde_json::{Value, json};

#[test]
fn typed_operation_requests_reject_foreign_or_malformed_patient_data() {
    for bytes in [
        b"{}".as_slice(),
        b"null",
        b"[]",
        b"not-json",
        b"{\"patient_id\":\"other\"}",
    ] {
        assert!(Arguments::<0>::from_bytes(bytes).is_err());
        assert!(Arguments::<1>::from_bytes(bytes).is_err());
        assert!(Arguments::<2>::from_bytes(bytes).is_err());
        assert!(Arguments::<3>::from_bytes(bytes).is_err());
        assert!(Arguments::<4>::from_bytes(bytes).is_err());
        assert!(Arguments::<5>::from_bytes(bytes).is_err());
    }
    for tests in [
        "vitals",
        "basic_labs",
        "vitals,basic_labs",
        "basic_labs,vitals",
        "vitals,vitals",
        " vitals , basic_labs ",
    ] {
        let bytes =
            serde_json::to_vec(&json!({"patient_id":"patient_1","test_type":tests})).unwrap();
        assert!(Arguments::<1>::from_bytes(&bytes).is_ok());
        assert!(Arguments::<3>::from_bytes(&bytes).is_err());
    }
    for tests in [
        "ecg",
        "cardiac_enzymes",
        "ecg,cardiac_enzymes",
        "cardiac_enzymes,ecg",
        "ecg,ecg",
    ] {
        let bytes =
            serde_json::to_vec(&json!({"patient_id":"patient_1","test_type":tests})).unwrap();
        assert!(Arguments::<3>::from_bytes(&bytes).is_ok());
        assert!(Arguments::<1>::from_bytes(&bytes).is_err());
    }
    assert!(Arguments::<2>::from_bytes(br#"{"patient_id":"patient_1","modality":"ct"}"#).is_ok());
    assert!(Arguments::<2>::from_bytes(br#"{"patient_id":"patient_1","modality":"MRI"}"#).is_err());
    assert!(Arguments::<4>::from_bytes(br#"{"patient_id":"patient_1","symptoms":[]}"#).is_err());
    assert!(Arguments::<5>::from_bytes(br#"{"patient_id":"patient_1","medications":[]}"#).is_err());
    for bytes in [
        br#"{"patient_id":"patient_1"}"#.as_slice(),
        br#"{"patient_id":"patient_1","symptoms":"","medications":"caller data"}"#,
    ] {
        assert!(Arguments::<0>::from_bytes(bytes).is_ok());
        assert!(Arguments::<4>::from_bytes(bytes).is_ok());
        assert!(Arguments::<5>::from_bytes(bytes).is_ok());
    }
}

#[test]
fn actual_panel_replacements_remain_bounded_and_invalid_updates_are_atomic() {
    let mut panel = Value::Null;
    for revision in 0..200 {
        let actual = json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":format!("vitals-{revision}"),"tests":["vitals"],"inputs":{"assessment":"assessment-1"}}});
        patient::record_panel(&mut panel, &actual, &["vitals", "basic_labs"]).unwrap();
        assert_eq!(panel.as_object().unwrap().len(), 1);
        assert!(!patient::panel_complete(&panel, &["vitals", "basic_labs"]).unwrap());
    }
    let before = panel.clone();
    for actual in [
        json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"bad","tests":["vitals","ecg"],"inputs":{"assessment":"assessment-1"}}}),
        json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"bad","tests":["basic_labs"],"inputs":{"assessment":"assessment-2"}}}),
        json!({"success":true,"data":{"patient_id":"other","artifact_id":"bad","tests":["basic_labs"],"inputs":{"assessment":"assessment-1"}}}),
    ] {
        assert!(patient::record_panel(&mut panel, &actual, &["vitals", "basic_labs"]).is_err());
        assert_eq!(panel, before);
    }
    let actual = json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"basic-labs","tests":["basic_labs"],"inputs":{"assessment":"assessment-1"}}});
    patient::record_panel(&mut panel, &actual, &["vitals", "basic_labs"]).unwrap();
    assert!(patient::panel_complete(&panel, &["vitals", "basic_labs"]).unwrap());
    assert_eq!(panel.as_object().unwrap().len(), 2);
}
