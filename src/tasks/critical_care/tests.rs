//! Input contracts and actual patient data; protocol qualification uses real Wasm.
use super::imp::{self, Arguments};
use serde_json::{Value, json};
fn valid<const OP: u8>(value: Value) -> bool {
    Arguments::<OP>::from_bytes(&serde_json::to_vec(&value).unwrap()).is_ok()
}
#[test]
fn patient_operation_bytes_keep_patient_modality_and_field_types() {
    assert!(valid::<0>(json!({"patient_id":"patient_a"})));
    assert!(!valid::<0>(json!({"patient_id":"patient_b"})));
    assert!(valid::<1>(json!({"patient_id":"patient_b"})));
    assert!(!valid::<1>(json!({"patient_id":"other"})));
    assert!(valid::<2>(
        json!({"patient_id":"patient_a","test_type":"coag,trauma_panel,blood_type"})
    ));
    assert!(!valid::<2>(
        json!({"patient_id":"patient_a","test_type":"stroke_panel"})
    ));
    assert!(valid::<3>(
        json!({"patient_id":"patient_b","test_type":"ct_angio,stroke_panel"})
    ));
    assert!(!valid::<3>(
        json!({"patient_id":"patient_b","test_type":"coag"})
    ));
    assert!(valid::<4>(
        json!({"patient_id":"patient_a","modality":"CT"})
    ));
    assert!(!valid::<4>(
        json!({"patient_id":"patient_b","modality":"CT"})
    ));
    assert!(valid::<5>(
        json!({"patient_id":"patient_a","modality":"ct"})
    ));
    assert!(!valid::<5>(
        json!({"patient_id":"patient_a","modality":"MRI"})
    ));
    assert!(valid::<6>(
        json!({"patient_id":"patient_b","modality":"CT"})
    ));
    assert!(valid::<7>(
        json!({"patient_id":"patient_b","modality":"CT"})
    ));
    assert!(valid::<8>(
        json!({"patient_id":"patient_b","modality":"MRI"})
    ));
    assert!(valid::<9>(
        json!({"patient_id":"patient_b","modality":"mri"})
    ));
    assert!(!valid::<8>(
        json!({"patient_id":"patient_a","modality":"MRI"})
    ));
    assert!(valid::<10>(
        json!({"patient_id":"patient_a","type":"caller text"})
    ));
    assert!(!valid::<10>(json!({"patient_id":"patient_a","type":[]})));
    assert!(valid::<11>(
        json!({"patient_id":"patient_a","procedure":"trauma_surgery"})
    ));
    assert!(!valid::<11>(
        json!({"patient_id":"patient_b","procedure":"trauma_surgery"})
    ));
    assert!(valid::<12>(json!({"patient_id":"patient_b"})));
    assert!(!valid::<12>(json!({"patient_id":"patient_a"})));
    assert!(valid::<13>(
        json!({"patient_id":"patient_A_and_B","medications":"actual caller text"})
    ));
    assert!(!valid::<13>(
        json!({"patient_id":"patient_A_and_B","medications":[]})
    ));
    assert!(valid::<14>(
        json!({"patient_id":"patient_a","symptoms":"caller findings"})
    ));
    assert!(!valid::<14>(
        json!({"patient_id":"patient_a","symptoms":[]})
    ));
    assert!(valid::<15>(json!({"patient_id":"patient_b"})));
}
#[test]
fn a_foreign_or_malformed_panel_update_preserves_actual_receipts() {
    let attempt = json!({"success":true,"data":{"patient_id":"patient_a","artifact_id":"panel:0","tests":["coag"],"inputs":{"assessment":"assessment:0"}}});
    let mut panel = Value::Null;
    imp::record_panel(
        &mut panel,
        &attempt,
        ("patient_a", &["coag", "blood_type", "trauma_panel"]),
    )
    .unwrap();
    let before = panel.clone();
    let mut other = attempt;
    other["data"]["patient_id"] = json!("patient_b");
    assert!(
        imp::record_panel(
            &mut panel,
            &other,
            ("patient_a", &["coag", "blood_type", "trauma_panel"])
        )
        .is_err()
    );
    assert_eq!(panel, before);
}
