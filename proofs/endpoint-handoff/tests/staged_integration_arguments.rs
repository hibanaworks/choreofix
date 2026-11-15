//! Typed shell arguments for every assigned 6H effect.
#[path = "../../../src/tasks/staged_integration/arguments.rs"]
mod arguments;
use arguments::{Arguments, FEATURES, SUITES};
use hibana::runtime::wire::{Payload, WireEncode, WirePayload};
use serde_json::json;

fn check<const OP: u8>() {
    let feature = usize::from(OP / 8);
    let key = match OP % 8 {
        0 => "feature_name",
        1 => "target",
        2 => "pr_id",
        3 => "branch",
        4 => "suite",
        5..=7 => "service",
        _ => unreachable!(),
    };
    let expected = if OP % 8 == 4 {
        SUITES[feature]
    } else {
        FEATURES[feature]
    };
    let bytes = serde_json::to_vec(&json!({key:expected})).unwrap();
    let arguments = Arguments::<OP>::from_bytes(&bytes).unwrap();
    let mut buffer = vec![0; bytes.len()];
    assert_eq!(arguments.encode_into(&mut buffer).unwrap(), bytes.len());
    assert_eq!(buffer, bytes);
    assert_eq!(arguments.into_value()[key], expected);
    for invalid in [
        json!({}),
        json!({key:12}),
        json!({key:"unassigned"}),
        json!({key:expected,"stage":"approved"}),
        json!({key:expected,"source":""}),
        json!({key:expected,"source":null}),
    ] {
        assert!(
            Arguments::<OP>::validate_payload(Payload::new(&serde_json::to_vec(&invalid).unwrap()))
                .is_err()
        );
    }
    let revision =
        serde_json::to_vec(&json!({key:expected,"source":"agent-chosen new content"})).unwrap();
    assert_eq!(
        Arguments::<OP>::from_bytes(&revision).is_ok(),
        OP.is_multiple_of(8)
    );
    let staged = serde_json::to_vec(&json!({key:expected,"environment":"staging"})).unwrap();
    assert_eq!(Arguments::<OP>::from_bytes(&staged).is_ok(), OP % 8 == 5);
    for environment in [json!("production"), json!(""), json!(null), json!(42)] {
        let bytes = serde_json::to_vec(&json!({key:expected,"environment":environment})).unwrap();
        assert!(Arguments::<OP>::from_bytes(&bytes).is_err());
    }
    for other in FEATURES
        .into_iter()
        .chain(SUITES)
        .filter(|other| *other != expected)
    {
        assert!(
            Arguments::<OP>::from_bytes(&serde_json::to_vec(&json!({key:other})).unwrap()).is_err()
        );
    }
}
#[test]
fn all_twenty_four_effect_schemas_reject_wrong_targets_and_preserve_actual_bytes() {
    check::<0>();
    check::<1>();
    check::<2>();
    check::<3>();
    check::<4>();
    check::<5>();
    check::<6>();
    check::<7>();
    check::<8>();
    check::<9>();
    check::<10>();
    check::<11>();
    check::<12>();
    check::<13>();
    check::<14>();
    check::<15>();
    check::<16>();
    check::<17>();
    check::<18>();
    check::<19>();
    check::<20>();
    check::<21>();
    check::<22>();
    check::<23>();
    assert!(Arguments::<24>::from_bytes(b"{}").is_err());
    assert!(Arguments::<255>::from_bytes(b"{}").is_err());
}
