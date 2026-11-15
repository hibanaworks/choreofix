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
// Actual development material. These checks do not choose a protocol branch.
pub fn candidate(value: &Value, feature: usize) -> ShellResult<()> {
    result(
        value,
        "implement_feature",
        ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"][feature],
    )?;
    if value["success"] != true
        || value["data"]["feature"] != FEATURES[feature]
        || value["data"]["source"].as_str().is_none_or(str::is_empty)
        || value["data"]["candidate_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err(ShellError::message(
            "missing actual staged-development candidate",
        ));
    }
    Ok(())
}
pub fn tested(value: &Value, source: &Value, feature: usize) -> ShellResult<()> {
    candidate(source, feature)?;
    result(value, "write_tests", "TEST_WRITER")?;
    if value["success"] != true
        || value["data"]["component"] != FEATURES[feature]
        || value["data"]["candidate_sha256"] != source["data"]["candidate_sha256"]
        || value["data"]["source_receipt"].as_u64() != Some(receipt(source)?)
        || !value["data"]["test_source"].is_object()
        || value["data"]["test_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || receipt(value)? <= receipt(source)?
    {
        return Err(ShellError::message(
            "tests do not describe the received source occurrence",
        ));
    }
    Ok(())
}
pub fn review_inputs(inputs: &Value, feature: usize) -> ShellResult<()> {
    tested(&inputs["tests"], &inputs["candidate"], feature)
}
pub fn reviewed(value: &Value, source: &Value, feature: usize) -> ShellResult<()> {
    candidate(source, feature)?;
    review_notice(value, feature)?;
    if value["data"]["candidate_sha256"] != source["data"]["candidate_sha256"]
        || value["data"]["source_receipt"].as_u64() != Some(receipt(source)?)
        || receipt(value)? <= receipt(source)?
    {
        return Err(ShellError::message(
            "review differs from the actual source occurrence",
        ));
    }
    Ok(())
}
pub fn review_notice(value: &Value, feature: usize) -> ShellResult<()> {
    result(value, "code_review", "REVIEWER")?;
    if value["data"]["target"] != FEATURES[feature]
        || !value["data"]["source_receipt"].is_u64()
        || value["data"]["candidate_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err(ShellError::message("unrelated review result"));
    }
    Ok(())
}
pub fn merge_inputs(
    request: &Value,
    (feature, source, tests, review): (usize, &Value, &Value, &Value),
) -> ShellResult<()> {
    assigned(request, "branch", FEATURES[feature])?;
    tested(tests, source, feature)?;
    reviewed(review, source, feature)?;
    if review["success"] != true || receipt(review)? <= receipt(tests)? {
        return Err(ShellError::message(
            "merge requires positive review after current tests",
        ));
    }
    Ok(())
}
pub fn merged(value: &Value, feature: usize) -> ShellResult<()> {
    result(
        value,
        "merge_code",
        ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"][feature],
    )?;
    if value["success"] != true
        || value["data"]["feature"] != FEATURES[feature]
        || value["data"]["branch"] != FEATURES[feature]
        || value["data"]["candidate_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || value["data"]["repository_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || value["data"]["publication_event"].as_u64() != Some(receipt(value)?)
        || !value["data"]["review_receipt"].is_u64()
    {
        return Err(ShellError::message("missing actual reviewed publication"));
    }
    Ok(())
}
pub fn merge_result(
    value: &Value,
    (feature, source, tests, review): (usize, &Value, &Value, &Value),
    prior_ci: &Value,
) -> ShellResult<()> {
    merge_inputs(
        &json!({"branch":FEATURES[feature]}),
        (feature, source, tests, review),
    )?;
    merged(value, feature)?;
    let data = &value["data"];
    if data["source_receipt"].as_u64() != Some(receipt(source)?)
        || data["review_receipt"].as_u64() != Some(receipt(review)?)
        || data["source"] != source["data"]["source"]
        || data["candidate_sha256"] != source["data"]["candidate_sha256"]
        || data["published_candidates"][FEATURES[feature]] != source["data"]["candidate_sha256"]
        || receipt(value)? <= receipt(review)?
    {
        return Err(ShellError::message(
            "publication substituted its reviewed material",
        ));
    }
    if prior_ci.is_null() {
        if !data["base_repository_sha256"].is_null()
            || !data["base_test_receipt"].is_null()
            || !data["base_failure_receipt"].is_null()
            || data["base_publication_events"]
                .as_object()
                .is_none_or(|map| !map.is_empty())
        {
            return Err(ShellError::message(
                "first publication names an unknown base",
            ));
        }
    } else {
        result(prior_ci, "run_tests", "CI_RUNNER")?;
        if data["base_repository_sha256"] != prior_ci["data"]["repository_sha256"]
            || data["base_publication_events"] != prior_ci["data"]["publication_events"]
        {
            return Err(ShellError::message(
                "publication differs from the actual integrated base",
            ));
        }
        match prior_ci["success"].as_bool() {
            Some(true) => {
                if data["base_test_receipt"].as_u64() != Some(receipt(prior_ci)?)
                    || !data["base_failure_receipt"].is_null()
                    || !data["repaired_publication_event"].is_null()
                {
                    return Err(ShellError::message(
                        "publication lost its successful base CI",
                    ));
                }
            }
            Some(false) => {
                let replaced = prior_ci["data"]["publication_event"]
                    .as_u64()
                    .ok_or_else(|| ShellError::message("failed publication occurrence"))?;
                if prior_ci["data"]["feature"] != FEATURES[feature]
                    || !data["base_test_receipt"].is_null()
                    || data["base_failure_receipt"].as_u64() != Some(receipt(prior_ci)?)
                    || data["repaired_publication_event"].as_u64() != Some(replaced)
                    || receipt(source)? <= replaced
                {
                    return Err(ShellError::message(
                        "repair did not replace the actual failed publication",
                    ));
                }
            }
            None => return Err(ShellError::message("base CI has no outcome")),
        }
    }
    Ok(())
}
fn publication_events(value: &Value, feature: usize) -> ShellResult<Value> {
    let mut events = value["data"]["base_publication_events"]
        .as_object()
        .ok_or_else(|| ShellError::message("missing publication base receipts"))?
        .clone();
    events.insert(FEATURES[feature].into(), json!(receipt(value)?));
    Ok(Value::Object(events))
}
pub fn ci_notice(value: &Value, feature: usize) -> ShellResult<()> {
    result(value, "run_tests", "CI_RUNNER")?;
    if value["data"]["feature"] != FEATURES[feature] || value["data"]["suite"] != SUITES[feature] {
        return Err(ShellError::message("unrelated integrated CI result"));
    }
    Ok(())
}
pub fn ci_inputs(publication: &Value, tests: &[Value; 3], feature: usize) -> ShellResult<()> {
    merged(publication, feature)?;
    let candidates = publication["data"]["published_candidates"]
        .as_object()
        .ok_or_else(|| ShellError::message("integrated candidate map"))?;
    if !candidates.contains_key(FEATURES[feature]) {
        return Err(ShellError::message(
            "integrated publication lacks its own feature",
        ));
    }
    for (name, identity) in candidates {
        let index = FEATURES
            .iter()
            .position(|feature| feature == name)
            .ok_or_else(|| ShellError::message("unassigned integrated feature"))?;
        let test = &tests[index];
        result(test, "write_tests", "TEST_WRITER")?;
        if test["success"] != true
            || test["data"]["component"] != *name
            || test["data"]["candidate_sha256"] != *identity
            || !test["data"]["test_source"].is_object()
            || !test["data"]["test_sha256"].is_string()
            || receipt(test)? >= receipt(publication)?
        {
            return Err(ShellError::message(
                "CI lacks the actual published inputs' completed tests",
            ));
        }
    }
    Ok(())
}
pub fn ci_result(
    value: &Value,
    publication: &Value,
    tests: &[Value; 3],
    feature: usize,
) -> ShellResult<()> {
    ci_inputs(publication, tests, feature)?;
    ci_notice(value, feature)?;
    let data = &value["data"];
    let published = &publication["data"];
    if data["repository_sha256"] != published["repository_sha256"]
        || data["published_candidates"] != published["published_candidates"]
        || data["publication_event"] != published["publication_event"]
        || data["publication_events"] != publication_events(publication, feature)?
        || data["publication_events"][FEATURES[feature]] != published["publication_event"]
        || receipt(value)? <= receipt(publication)?
    {
        return Err(ShellError::message("CI checked another publication"));
    }
    let candidates = published["published_candidates"]
        .as_object()
        .expect("checked input map");
    for field in ["publication_events", "test_receipts", "test_inputs"] {
        if data[field].as_object().is_none_or(|map| {
            map.len() != candidates.len() || candidates.keys().any(|key| !map.contains_key(key))
        }) {
            return Err(ShellError::message(
                "CI receipt map differs from published inputs",
            ));
        }
    }
    for name in candidates.keys() {
        let index = FEATURES
            .iter()
            .position(|feature| feature == name)
            .expect("checked feature");
        if data["test_receipts"][name].as_u64() != Some(receipt(&tests[index])?)
            || data["test_inputs"][name] != tests[index]["data"]
            || data["publication_events"][name]
                .as_u64()
                .is_none_or(|id| id > receipt(publication).expect("checked receipt"))
        {
            return Err(ShellError::message(
                "CI did not use the exact received input receipts",
            ));
        }
    }
    Ok(())
}
pub fn stage_notice(value: &Value, feature: usize) -> ShellResult<()> {
    result(value, "validate_staging", "RELEASE_MANAGER")?;
    if value["data"]["service"] != FEATURES[feature]
        || !value["data"]["deployment_event"].is_u64()
        || value["data"]["candidate_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err(ShellError::message("unrelated actual staging result"));
    }
    Ok(())
}
pub fn publication_stage(
    value: &Value,
    publication: &Value,
    ci: &Value,
    feature: usize,
) -> ShellResult<()> {
    stage_notice(value, feature)?;
    staging_inputs(&json!({"publication":publication,"ci":ci}), feature)?;
    let deployment = value["data"]["deployment_event"]
        .as_u64()
        .ok_or_else(|| ShellError::message("staging deployment occurrence"))?;
    if value["data"]["candidate_sha256"] != publication["data"]["candidate_sha256"]
        || deployment <= receipt(ci)?
        || receipt(value)? <= deployment
    {
        return Err(ShellError::message(
            "staging result names an older publication or CI",
        ));
    }
    Ok(())
}
pub fn rollback_notice(
    value: &Value,
    validation: &Value,
    publication: &Value,
    ci: &Value,
    feature: usize,
) -> ShellResult<()> {
    publication_stage(validation, publication, ci, feature)?;
    result(value, "rollback_service", "RELEASE_MANAGER")?;
    let removed = &value["data"]["removed_deployment"];
    if validation["success"] != false
        || value["success"] != true
        || value["data"]["service"] != FEATURES[feature]
        || removed["service"] != FEATURES[feature]
        || removed["deployment_event"] != validation["data"]["deployment_event"]
        || removed["candidate_sha256"] != publication["data"]["candidate_sha256"]
        || removed["publication_event"] != publication["data"]["publication_event"]
        || removed["ci_receipt"].as_u64() != Some(receipt(ci)?)
        || receipt(value)? <= receipt(validation)?
    {
        return Err(ShellError::message(
            "rollback did not remove the actually rejected staging material",
        ));
    }
    Ok(())
}
pub fn actual_bytes(input: &core::cell::Cell<Option<Value>>) -> ShellResult<Vec<u8>> {
    let value = input.take();
    let encoded = match value.as_ref() {
        Some(value) => encode(value),
        None => Err(ShellError::message("missing actual effect outcome")),
    };
    input.set(value);
    encoded
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
        || checked["publication_events"] != publication_events(publication, feature)?
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
        if !FEATURES.contains(&name.as_str()) {
            return Err(ShellError::message(
                "unknown feature in integrated publication",
            ));
        }
        if checked["publication_events"][name]
            .as_u64()
            .is_none_or(|id| id >= ci_receipt)
            || checked["test_receipts"][name]
                .as_u64()
                .is_none_or(|id| id >= ci_receipt)
            || checked["test_receipts"][name].as_u64()
                != Some(receipt(&json!({"data":checked["test_inputs"][name]}))?)
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
