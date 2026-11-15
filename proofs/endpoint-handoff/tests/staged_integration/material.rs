//! Material fixtures for component tests; no task driver or protocol decisions.
use crate::staged_integration::arguments::{FEATURES, SUITES};
use serde_json::{Value, json};

pub fn material(feature: usize) -> (Value, Value, Value, Value) {
    let name = FEATURES[feature];
    let id = 10 + feature as u64 * 10;
    let mut candidates = serde_json::Map::new();
    let mut publications = serde_json::Map::new();
    let mut tests = serde_json::Map::new();
    let mut test_inputs = serde_json::Map::new();
    for (index, feature) in FEATURES.iter().enumerate().take(feature + 1) {
        let hash = format!("candidate:{index}");
        candidates.insert((*feature).into(), json!(hash));
        publications.insert((*feature).into(), json!(10 + index as u64 * 10));
        tests.insert((*feature).into(), json!(8 + index as u64 * 10));
        test_inputs.insert((*feature).into(), json!({"component":feature,"candidate_sha256":hash,
            "test_source":{"target":feature,"expected_candidate":hash},"test_sha256":format!("test:{index}")}));
    }
    let inputs = json!({
        "publication":{"tool_name":"merge_code","success":true,"data":{
            "agent":(["DEVELOPER_A","DEVELOPER_B","DEVELOPER_C"][feature]),"feature":name,"branch":name,
            "candidate_sha256":format!("candidate:{feature}"),"repository_sha256":format!("repository:{feature}"),
            "publication_event":id,"published_candidates":candidates,"artifact_id":format!("development:{id}")}},
        "ci":{"tool_name":"run_tests","success":true,"data":{
            "agent":"CI_RUNNER","feature":name,"suite":SUITES[feature],
            "published_candidates":candidates,"repository_sha256":format!("repository:{feature}"),
            "publication_event":id,"publication_events":publications,"test_receipts":tests,
            "test_inputs":test_inputs,"artifact_id":format!("development:{}",id+2)}}});
    let mut dependencies = serde_json::Map::new();
    if feature > 0 {
        dependencies.insert(FEATURES[feature - 1].into(), json!(id - 10 + 4));
    }
    let deployment = json!({"tool_name":"deploy_service","success":true,"data":{
        "agent":"RELEASE_MANAGER","service":name,"environment":"staging",
        "candidate_sha256":format!("candidate:{feature}"),"repository_sha256":format!("repository:{feature}"),
        "publication_event":id,"ci_receipt":id+2,"dependency_receipts":dependencies,
        "deployment_event":id+3,"artifact_id":format!("development:{}",id+3)}});
    let validation = json!({"tool_name":"validate_staging","success":true,"data":{
        "agent":"RELEASE_MANAGER","service":name,"candidate_sha256":format!("candidate:{feature}"),
        "deployment_event":id+3,"artifact_id":format!("development:{}",id+4)}});
    let removed = json!({"service":name,"environment":"staging","candidate_sha256":format!("candidate:{feature}"),
        "repository_sha256":format!("repository:{feature}"),"publication_event":id,"ci_receipt":id+2,
        "dependency_receipts":dependencies,"deployment_event":id+3});
    let rollback = json!({"tool_name":"rollback_service","success":true,"data":{
        "agent":"RELEASE_MANAGER","service":name,"removed_deployment":removed,
        "artifact_id":format!("development:{}",id+5)}});
    (inputs, deployment, validation, rollback)
}
