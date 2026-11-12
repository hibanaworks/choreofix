//! Canonical repository payload and its actual message declarations.
#[path = "../../../src/tasks/review/choreofs/global.rs"]
pub mod global;
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}

#[test]
fn implementation_arguments_retain_actual_source_bytes_and_reject_other_fields() {
    use hibana::runtime::wire::{Payload, WireEncode, WirePayload};
    use shell::imp::repository_arguments::RepositoryArguments;
    let bytes = br#"{"feature_name":"user_authentication","source":"actual candidate bytes"}"#;
    let arguments = RepositoryArguments::<1>::from_bytes(bytes).unwrap();
    let mut encoded = [0; 160];
    let length = arguments.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..length], bytes);
    assert_eq!(arguments.into_value()["source"], "actual candidate bytes");
    for invalid in [
        br#"{"feature_name":"user_authentication","source":[]}"#.as_slice(),
        br#"{"feature_name":"user_authentication","source":" "}"#,
        br#"{"feature_name":"user_authentication","unrelated":"data"}"#,
    ] {
        assert!(RepositoryArguments::<1>::validate_payload(Payload::new(invalid)).is_err());
    }
    assert!(
        RepositoryArguments::<2>::validate_payload(Payload::new(
            br#"{"branch":"dev_a_feature","source":"not a merge argument"}"#,
        ))
        .is_err()
    );
}

#[test]
fn test_and_ci_arguments_keep_their_distinct_targets() {
    use hibana::runtime::wire::{Payload, WirePayload};
    use shell::imp::repository_arguments::RepositoryArguments;
    let tests = br#"{"target":"payment_processing"}"#;
    let ci = br#"{"suite":"full_test_suite_after_payment"}"#;
    assert_eq!(
        RepositoryArguments::<4>::from_bytes(tests)
            .unwrap()
            .into_value()["target"],
        "payment_processing"
    );
    assert_eq!(
        RepositoryArguments::<5>::from_bytes(ci)
            .unwrap()
            .into_value()["suite"],
        "full_test_suite_after_payment"
    );
    assert!(RepositoryArguments::<4>::validate_payload(Payload::new(ci)).is_err());
    assert!(RepositoryArguments::<5>::validate_payload(Payload::new(tests)).is_err());
    for invalid in [
        br#"{"target":42}"#.as_slice(),
        br#"{"target":"payment_processing","source":"extra input"}"#,
    ] {
        assert!(RepositoryArguments::<4>::validate_payload(Payload::new(invalid)).is_err());
    }
}
