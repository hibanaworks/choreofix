//! Exact content, occurrence, dependency and rollback checks for all three services.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "staged_integration/material.rs"]
mod material;
#[path = "../../../src/tasks/staged_integration/mod.rs"]
pub mod staged_integration;

use serde_json::{Value, json};
use staged_integration::{arguments::FEATURES, imp as data};

#[test]
fn each_deployment_requires_its_exact_ci_and_validated_dependency_before_effect() {
    let mut previous = [Value::Null, Value::Null, Value::Null];
    for (feature, name) in FEATURES.into_iter().enumerate() {
        let (inputs, deployment, validation, _) = material::material(feature);
        let request = json!({"service":name});
        data::deploy_input(&request, &inputs, (feature, &previous)).unwrap();
        data::deploy_input(
            &json!({"service":name,"environment":"staging"}),
            &inputs,
            (feature, &previous),
        )
        .unwrap();
        data::deployed(&deployment, &inputs, (feature, &previous)).unwrap();
        data::validated(&validation, &deployment, feature).unwrap();
        for kind in 0..12 {
            let mut altered = inputs.clone();
            let mut actual_request = request.clone();
            match kind {
                0 => actual_request["service"] = json!("another service"),
                1 => altered["ci"]["success"] = json!(false),
                2 => altered["ci"]["data"]["repository_sha256"] = json!("old repository"),
                3 => altered["ci"]["data"]["publication_event"] = json!(0),
                4 => altered["ci"]["data"]["published_candidates"][name] = json!("substitution"),
                5 => {
                    altered["ci"]["data"]["artifact_id"] =
                        altered["publication"]["data"]["artifact_id"].clone()
                }
                6 => {
                    altered["ci"]["data"]["test_inputs"][name]["candidate_sha256"] =
                        json!("old source")
                }
                7 => altered["publication"]["data"]["agent"] = json!("REVIEWER"),
                8 => altered["ci"]["data"]["test_receipts"][name] = Value::Null,
                9 => altered["publication"]["data"]["candidate_sha256"] = json!("different source"),
                10 => altered["ci"]["data"]["suite"] = json!("another suite"),
                11 => actual_request["environment"] = json!("production"),
                _ => unreachable!(),
            }
            assert!(
                data::deploy_input(&actual_request, &altered, (feature, &previous)).is_err(),
                "{feature}/{kind}"
            );
        }
        if feature > 0 {
            for kind in 0..4 {
                let mut stale = previous.clone();
                match kind {
                    0 => stale[feature - 1] = Value::Null,
                    1 => stale[feature - 1]["validation"]["success"] = json!(false),
                    2 => {
                        stale[feature - 1]["deployment"]["data"]["candidate_sha256"] =
                            json!("replaced predecessor")
                    }
                    3 => stale[feature - 1]["validation"]["data"]["deployment_event"] = json!(0),
                    _ => unreachable!(),
                }
                assert!(
                    data::deploy_input(&request, &inputs, (feature, &stale)).is_err(),
                    "dependency {feature}/{kind}"
                );
            }
        }
        previous[feature] = json!({"deployment":deployment,"validation":validation});
    }
}
#[test]
fn validation_and_rollback_bind_the_actual_deployment_and_negative_result() {
    for (feature, name) in FEATURES.into_iter().enumerate() {
        let (_, deployment, mut validation, rollback) = material::material(feature);
        assert!(
            data::rollback_input(&json!({"service":name}), &deployment, &validation, feature)
                .is_err()
        );
        validation["success"] = json!(false);
        data::rollback_input(&json!({"service":name}), &deployment, &validation, feature).unwrap();
        data::rolled_back(&rollback, &deployment, &validation, feature).unwrap();
        for kind in 0..6 {
            let mut old = validation.clone();
            match kind {
                0 => old["data"]["service"] = json!("another service"),
                1 => old["data"]["candidate_sha256"] = json!("old candidate"),
                2 => old["data"]["deployment_event"] = json!(0),
                3 => old["data"]["artifact_id"] = deployment["data"]["artifact_id"].clone(),
                4 => old["data"]["agent"] = json!("DEVELOPER_A"),
                5 => old["tool_name"] = json!("code_review"),
                _ => unreachable!(),
            }
            assert!(data::validated(&old, &deployment, feature).is_err());
        }
        for field in [
            "candidate_sha256",
            "repository_sha256",
            "deployment_event",
            "ci_receipt",
        ] {
            let mut substitute = rollback.clone();
            substitute["data"]["removed_deployment"][field] = json!("substitution");
            assert!(data::rolled_back(&substitute, &deployment, &validation, feature).is_err());
        }
    }
}
