//! Content and effect receipts, independent of protocol progression.
use super::arguments::{FEATURES, SUITES};
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn receipt(value: &Value) -> ShellResult<u64> {
    value["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| ShellError::message("missing actual development effect receipt"))
}
fn result(value: &Value, tool: &str, agent: &str) -> ShellResult<()> {
    if value["tool_name"] != tool
        || value["data"]["agent"] != agent
        || !value["success"].is_boolean()
    {
        return Err(ShellError::message("effect tool, actor or outcome differs"));
    }
    receipt(value)?;
    Ok(())
}
pub fn assigned(arguments: &Value, key: &str, expected: &str) -> ShellResult<()> {
    if arguments[key] != expected {
        return Err(ShellError::message(
            "operation names another staged-release target",
        ));
    }
    Ok(())
}
pub fn staging_inputs(value: &Value, feature: usize) -> ShellResult<()> {
    let publication = &value["publication"];
    let ci = &value["ci"];
    let author = ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"][feature];
    result(publication, "merge_code", author)?;
    result(ci, "run_tests", "CI_RUNNER")?;
    let material = &publication["data"];
    let checked = &ci["data"];
    if publication["success"] != true
        || ci["success"] != true
        || material["feature"] != FEATURES[feature]
        || material["branch"] != FEATURES[feature]
        || material["candidate_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || material["repository_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || material["publication_event"].as_u64() != Some(receipt(publication)?)
        || material["published_candidates"][FEATURES[feature]] != material["candidate_sha256"]
        || checked["feature"] != FEATURES[feature]
        || checked["suite"] != SUITES[feature]
        || checked["publication_event"] != material["publication_event"]
        || checked["publication_events"][FEATURES[feature]] != material["publication_event"]
        || checked["repository_sha256"] != material["repository_sha256"]
        || checked["published_candidates"] != material["published_candidates"]
        || receipt(ci)? <= receipt(publication)?
    {
        return Err(ShellError::message(
            "staging requires completed CI for the exact publication",
        ));
    }
    let candidates = material["published_candidates"]
        .as_object()
        .ok_or_else(|| ShellError::message("published input map"))?;
    if candidates.len() != feature + 1
        || FEATURES[..=feature]
            .iter()
            .any(|name| !candidates.contains_key(*name))
    {
        return Err(ShellError::message(
            "staged publication lacks a preceding feature",
        ));
    }
    let ci_receipt = receipt(ci)?;
    for field in ["publication_events", "test_receipts", "test_inputs"] {
        if checked[field].as_object().is_none_or(|map| {
            map.len() != candidates.len() || candidates.keys().any(|key| !map.contains_key(key))
        }) {
            return Err(ShellError::message(
                "CI input receipt map differs from its publication",
            ));
        }
    }
    for (name, identity) in candidates {
        let Some(index) = FEATURES.iter().position(|feature| feature == name) else {
            return Err(ShellError::message(
                "unknown feature in integrated publication",
            ));
        };
        if index > feature
            || checked["publication_events"][name]
                .as_u64()
                .is_none_or(|id| id >= ci_receipt)
            || checked["test_receipts"][name]
                .as_u64()
                .is_none_or(|id| id >= ci_receipt)
            || checked["test_inputs"][name]["component"] != *name
            || checked["test_inputs"][name]["candidate_sha256"] != *identity
            || checked["test_inputs"][name]["test_sha256"]
                .as_str()
                .is_none_or(str::is_empty)
            || checked["test_inputs"][name]["test_source"]
                .as_object()
                .is_none()
        {
            return Err(ShellError::message(
                "CI has incomplete or different material inputs",
            ));
        }
    }
    Ok(())
}
pub fn deploy_input(
    arguments: &Value,
    inputs: &Value,
    (feature, prior): (usize, &[Value; 3]),
) -> ShellResult<()> {
    assigned(arguments, "service", FEATURES[feature])?;
    if arguments
        .get("environment")
        .is_some_and(|environment| environment != "staging")
    {
        return Err(ShellError::message(
            "the staging Endpoint cannot deploy to another environment",
        ));
    }
    staging_inputs(inputs, feature)?;
    if feature > 0 {
        let previous = &prior[feature - 1];
        validated(
            &previous["validation"],
            &previous["deployment"],
            feature - 1,
        )?;
        if previous["validation"]["success"] != true
            || previous["deployment"]["data"]["candidate_sha256"]
                != inputs["publication"]["data"]["published_candidates"][FEATURES[feature - 1]]
        {
            return Err(ShellError::message(
                "deployment lacks its actual validated predecessor",
            ));
        }
    }
    Ok(())
}
pub fn deployed(
    value: &Value,
    inputs: &Value,
    (feature, prior): (usize, &[Value; 3]),
) -> ShellResult<()> {
    deploy_input(
        &json!({"service":FEATURES[feature]}),
        inputs,
        (feature, prior),
    )?;
    result(value, "deploy_service", "RELEASE_MANAGER")?;
    let publication = &inputs["publication"]["data"];
    let data = &value["data"];
    let mut dependencies = serde_json::Map::new();
    if feature > 0 {
        let name = FEATURES[feature - 1];
        let previous = &prior[feature - 1];
        validated(
            &previous["validation"],
            &previous["deployment"],
            feature - 1,
        )?;
        if previous["validation"]["success"] != true
            || previous["deployment"]["data"]["candidate_sha256"]
                != publication["published_candidates"][name]
        {
            return Err(ShellError::message(
                "staged predecessor differs from current published material",
            ));
        }
        dependencies.insert(name.into(), json!(receipt(&previous["validation"])?));
    }
    if value["success"] != true
        || data["service"] != FEATURES[feature]
        || data["environment"] != "staging"
        || data["candidate_sha256"] != publication["candidate_sha256"]
        || data["repository_sha256"] != publication["repository_sha256"]
        || data["publication_event"] != publication["publication_event"]
        || data["ci_receipt"].as_u64() != Some(receipt(&inputs["ci"])?)
        || data["dependency_receipts"] != Value::Object(dependencies)
        || data["deployment_event"].as_u64() != Some(receipt(value)?)
        || receipt(value)? <= receipt(&inputs["ci"])?
    {
        return Err(ShellError::message(
            "deployment does not match received publication and CI",
        ));
    }
    Ok(())
}
pub fn validated(value: &Value, deployment: &Value, feature: usize) -> ShellResult<()> {
    result(deployment, "deploy_service", "RELEASE_MANAGER")?;
    result(value, "validate_staging", "RELEASE_MANAGER")?;
    if deployment["success"] != true
        || deployment["data"]["service"] != FEATURES[feature]
        || value["data"]["service"] != FEATURES[feature]
        || value["data"]["candidate_sha256"] != deployment["data"]["candidate_sha256"]
        || value["data"]["deployment_event"] != deployment["data"]["deployment_event"]
        || deployment["data"]["deployment_event"].as_u64() != Some(receipt(deployment)?)
        || receipt(value)? <= receipt(deployment)?
    {
        return Err(ShellError::message(
            "staging validation names a different deployment",
        ));
    }
    Ok(())
}
pub fn rollback_input(
    arguments: &Value,
    deployment: &Value,
    validation: &Value,
    feature: usize,
) -> ShellResult<()> {
    assigned(arguments, "service", FEATURES[feature])?;
    validated(validation, deployment, feature)?;
    if validation["success"] != false {
        return Err(ShellError::message(
            "rollback requires the actual negative staging result",
        ));
    }
    Ok(())
}
pub fn rolled_back(
    value: &Value,
    deployment: &Value,
    validation: &Value,
    feature: usize,
) -> ShellResult<()> {
    rollback_input(
        &json!({"service":FEATURES[feature]}),
        deployment,
        validation,
        feature,
    )?;
    result(value, "rollback_service", "RELEASE_MANAGER")?;
    let data = &deployment["data"];
    let removed = json!({"service":data["service"],"candidate_sha256":data["candidate_sha256"],
        "repository_sha256":data["repository_sha256"],"publication_event":data["publication_event"],
        "ci_receipt":data["ci_receipt"],"dependency_receipts":data["dependency_receipts"],
        "deployment_event":data["deployment_event"],"environment":data["environment"]});
    if value["success"] != true
        || value["data"]["service"] != FEATURES[feature]
        || value["data"]["removed_deployment"] != removed
        || receipt(value)? <= receipt(validation)?
    {
        return Err(ShellError::message(
            "rollback did not remove the actually rejected deployment",
        ));
    }
    Ok(())
}
