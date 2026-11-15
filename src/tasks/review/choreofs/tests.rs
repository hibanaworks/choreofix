//! Operation data contracts; protocol and rejection schedules use actual Wasm tests.
#[test]
fn review_reply_retains_the_response_and_rejects_the_opposite_outcome() {
    use crate::shell::imp::review_reply::ReviewReply;
    use hibana::runtime::wire::{Payload, WireEncode, WirePayload};

    let approved = br#"{"success":true,"feedback":"merge this revision"}"#;
    let rejected = br#"{"success":false,"feedback":"revise this change"}"#;
    assert!(ReviewReply::<true>::from_bytes(approved).is_ok());
    assert!(ReviewReply::<false>::from_bytes(rejected).is_ok());
    assert!(ReviewReply::<true>::from_bytes(rejected).is_err());
    assert!(ReviewReply::<false>::from_bytes(approved).is_err());
    for malformed in [b"{}".as_slice(), b"null", br#"{"success":"true"}"#] {
        assert!(ReviewReply::<true>::validate_payload(Payload::new(malformed)).is_err());
        assert!(ReviewReply::<false>::validate_payload(Payload::new(malformed)).is_err());
    }
    let response = ReviewReply::<true>::from_bytes(approved).unwrap();
    let mut encoded = [0; 128];
    let written = response.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..written], approved);
}

#[test]
fn file_contract_keeps_target_and_source_and_rejects_foreign_operations() {
    use super::imp::Arguments;
    use hibana::runtime::wire::{Payload, WirePayload};
    let input = br#"{"feature_name":"user_authentication","source":"actual candidate"}"#;
    assert_eq!(
        Arguments::<0>::from_bytes(input).unwrap().into_value()["source"],
        "actual candidate"
    );
    assert!(Arguments::<2>::validate_payload(Payload::new(input)).is_err());
    for input in [
        br#"{}"#.as_slice(),
        br#"{"feature_name":"notification_feature"}"#,
        br#"{"feature_name":"user_authentication","source":[]}"#,
        br#"{"feature_name":"user_authentication","source":" "}"#,
    ] {
        assert!(Arguments::<0>::validate_payload(Payload::new(input)).is_err());
    }
    assert!(
        Arguments::<5>::validate_payload(Payload::new(br#"{"pr_id":"user_authentication"}"#))
            .is_ok()
    );
    assert!(
        Arguments::<6>::validate_payload(Payload::new(br#"{"pr_id":"user_authentication"}"#))
            .is_err()
    );
    assert!(
        Arguments::<1>::validate_payload(Payload::new(br#"{"branch":"dev_a_feature"}"#)).is_ok()
    );
    assert!(
        Arguments::<3>::validate_payload(Payload::new(br#"{"branch":"dev_a_feature"}"#)).is_err()
    );
}
