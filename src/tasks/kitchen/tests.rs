use super::{global, imp};
use hibana::runtime::wire::{Payload, WireEncode, WirePayload};
use serde_json::json;

#[test]
fn all_actual_roles_project_and_the_nine_operations_keep_their_assigned_recipes() {
    assert_eq!(hibana::g::diagnose(&global::source()), None);
    let p = global::programs();
    assert_eq!(p.observation()["programs"].as_array().unwrap().len(), 32);
    for index in 0..9 {
        assert!(imp::expected(index).is_some());
    }
    assert!(imp::expected(9).is_none());
}

#[test]
fn typed_file_arguments_preserve_bytes_and_reject_substitution() {
    let bytes = b" {\"dish\":\"appetizer\"} ";
    let input = imp::Arguments::<0>::from_bytes(bytes).unwrap();
    let mut encoded = [0; 64];
    let length = input.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..length], bytes);
    assert_eq!(input.value(), json!({"dish":"appetizer"}));
    for bad in [
        b"{\"dish\":\"dessert\"}".as_slice(),
        b"{\"dish\":\"appetizer\",\"extra\":true}",
        b"{",
    ] {
        assert!(imp::Arguments::<0>::validate_payload(Payload::new(bad)).is_err());
    }
    let garnish = imp::expected(4).unwrap();
    assert_eq!(
        imp::Arguments::<4>::from_bytes(&serde_json::to_vec(&garnish).unwrap())
            .unwrap()
            .value(),
        garnish
    );
    assert!(imp::Arguments::<3>::from_bytes(&serde_json::to_vec(&garnish).unwrap()).is_err());
}

#[test]
fn actual_dish_results_require_distinct_completed_input_receipts() {
    let args = imp::expected(6).unwrap();
    let mut reply = json!({"tool_name":"combine_dish","success":true,"data":{
        "chef":"CHEF_A","dish":"appetizer","ingredient":"garnish","status":"dish completed","base_event":1,"ingredient_event":2}});
    imp::completed("combine_dish", "CHEF_A", &args, &reply).unwrap();
    reply["data"]["ingredient_event"] = json!(1);
    assert!(imp::completed("combine_dish", "CHEF_A", &args, &reply).is_err());
    reply["data"]["ingredient_event"] = json!(2);
    reply["success"] = json!(false);
    assert!(imp::completed("combine_dish", "CHEF_A", &args, &reply).is_err());
}
