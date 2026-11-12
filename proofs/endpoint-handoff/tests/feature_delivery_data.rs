//! Data correspondence at merge, separately from actual Endpoint execution.
#[path = "../../../src/tasks/feature_delivery/imp.rs"]
mod data;
mod global {
    pub const FEATURE_A: &str = "user_authentication";
    pub const FEATURE_B: &str = "payment_processing";
    pub const SUITE_A: &str = "full_test_suite_after_auth";
    pub const SUITE_B: &str = "full_test_suite_after_payment";
}
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::transport;
}
use serde_json::{Value, json};

#[test]
fn merge_data_requires_the_exact_candidate_approval_and_integrated_base() {
    for feature in [global::FEATURE_A, global::FEATURE_B] {
        let candidate = json!({"tool_name":"implement_feature", "success":true,
            "data":{"feature":feature,"source":"actual source", "candidate_sha256":"source hash",
                "artifact_id":"development:10"}});
        let review = json!({"tool_name":"code_review","success":true,
            "data":{"agent":"REVIEWER", "target":feature,"candidate_sha256":"source hash",
                "source_receipt":10,"artifact_id":"development:12"}});
        let request = json!({"branch":feature});
        data::merge_inputs(
            &request,
            (&request, &request),
            (feature, &candidate, &review),
        )
        .unwrap();
        for trial in 0..7 {
            let mut feature_request = request.clone();
            let mut repo_request = request.clone();
            let mut approval = review.clone();
            match trial {
                0 => feature_request["branch"] = json!("other feature"),
                1 => repo_request["branch"] = json!("other feature"),
                2 => approval["success"] = json!(false),
                3 => approval["data"]["candidate_sha256"] = json!("old source"),
                4 => approval["data"]["source_receipt"] = json!(9),
                5 => approval["tool_name"] = json!("run_tests"),
                6 => approval["data"]["agent"] = json!("DEVELOPER_A"),
                _ => unreachable!(),
            }
            assert!(
                data::merge_inputs(
                    &request,
                    (&feature_request, &repo_request),
                    (feature, &candidate, &approval)
                )
                .is_err(),
                "{feature} input {trial}"
            );
        }
        let publication = json!({"tool_name":"merge_code","success":true,
            "data":{"feature":feature,"branch":feature,"source":"actual source",
                "candidate_sha256":"source hash","repository_sha256":"published hash",
                "source_receipt":10,"base_publication_events":{},
                "review_receipt":12,"publication_event":13,
                "published_candidates":{feature:"source hash"},
                "base_repository_sha256":null,"base_test_receipt":null}});
        data::merge_result(&publication, (feature, &candidate, &review), &Value::Null).unwrap();
        let prior_ci = json!({"tool_name":"run_tests","success":true,
            "data":{"repository_sha256":"integrated base","artifact_id":"development:9",
                "publication_event":8,"publication_events":{feature:8}}});
        let mut next = publication.clone();
        next["data"]["base_repository_sha256"] = json!("integrated base");
        next["data"]["base_test_receipt"] = json!(9);
        next["data"]["base_publication_events"] = json!({feature:8});
        data::merge_result(&next, (feature, &candidate, &review), &prior_ci).unwrap();
        for trial in 0..8 {
            let mut changed = next.clone();
            let mut base = prior_ci.clone();
            match trial {
                0 => changed["data"]["review_receipt"] = json!(11),
                1 => changed["data"]["source"] = json!("substituted source"),
                2 => changed["data"]["candidate_sha256"] = json!("old candidate"),
                3 => changed["data"]["published_candidates"][feature] = json!("other input"),
                4 => changed["data"]["base_test_receipt"] = json!(8),
                5 => changed["data"]["base_repository_sha256"] = json!("other repository"),
                6 => base["success"] = json!(false),
                7 => base["tool_name"] = json!("code_review"),
                _ => unreachable!(),
            }
            assert!(
                data::merge_result(&changed, (feature, &candidate, &review), &base).is_err(),
                "{feature} result {trial}"
            );
        }
        let mut failed_ci = prior_ci.clone();
        failed_ci["success"] = json!(false);
        failed_ci["data"]["feature"] = json!(feature);
        let mut repaired = next.clone();
        repaired["data"]["base_test_receipt"] = Value::Null;
        repaired["data"]["base_failure_receipt"] = json!(9);
        repaired["data"]["repaired_publication_event"] = json!(8);
        data::merge_result(&repaired, (feature, &candidate, &review), &failed_ci).unwrap();
        for trial in 0..8 {
            let mut changed = repaired.clone();
            let mut base = failed_ci.clone();
            match trial {
                0 => changed["data"]["base_failure_receipt"] = json!(7),
                1 => changed["data"]["repaired_publication_event"] = json!(7),
                2 => changed["data"]["base_publication_events"][feature] = json!(7),
                3 => base["data"]["feature"] = json!("other feature"),
                4 => base["data"]["publication_event"] = json!(10),
                5 => base["success"] = json!("not a decision"),
                6 => changed["data"]["base_test_receipt"] = json!(9),
                7 => changed["data"]["source_receipt"] = json!(7),
                _ => unreachable!(),
            }
            assert!(
                data::merge_result(&changed, (feature, &candidate, &review), &base).is_err(),
                "{feature} repair {trial}"
            );
        }
    }
}
