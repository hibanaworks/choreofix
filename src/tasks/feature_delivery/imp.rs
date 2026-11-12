//! Actual repository and validation data; no operation ordering.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn same(first: &Value, second: &Value) -> ShellResult<()> {
    if first != second {
        return Err(ShellError::message(
            "resource request differs from admitted arguments",
        ));
    }
    Ok(())
}
pub fn candidate(value: &Value, feature: &str) -> ShellResult<()> {
    if value["success"] != true
        || value["data"]["feature"] != feature
        || !value["data"]["source"].is_string()
        || !value["data"]["candidate_sha256"].is_string()
    {
        return Err(ShellError::message(
            "expected the completed candidate for this feature",
        ));
    }
    Ok(())
}
pub fn arguments(value: &Value, assigned: (&str, &str)) -> ShellResult<()> {
    if value[assigned.0] != assigned.1 {
        return Err(ShellError::message(
            "request names another development target",
        ));
    }
    Ok(())
}
pub fn tested(value: &Value, candidate: &Value, feature: &str) -> ShellResult<()> {
    if value["success"] != true
        || value["data"]["component"] != feature
        || value["data"]["candidate_sha256"] != candidate["data"]["candidate_sha256"]
        || !value["data"]["test_source"].is_object()
        || !value["data"]["test_sha256"].is_string()
    {
        return Err(ShellError::message(
            "tests do not name the received candidate",
        ));
    }
    Ok(())
}
pub fn reviewed(value: &Value, candidate: &Value, feature: &str) -> ShellResult<()> {
    if !value["success"].is_boolean()
        || value["data"]["target"] != feature
        || value["data"]["candidate_sha256"] != candidate["data"]["candidate_sha256"]
    {
        return Err(ShellError::message(
            "review does not name the received candidate",
        ));
    }
    Ok(())
}

pub fn merged(value: &Value, feature: &str) -> ShellResult<()> {
    if value["tool_name"] != "merge_code"
        || value["success"] != true
        || value["data"]["feature"] != feature
        || value["data"]["branch"] != feature
        || !value["data"]["candidate_sha256"].is_string()
        || !value["data"]["repository_sha256"].is_string()
        || !value["data"]["review_receipt"].is_u64()
        || !value["data"]["publication_event"].is_u64()
    {
        return Err(ShellError::message(
            "CI needs the actual reviewed publication",
        ));
    }
    Ok(())
}

/// Check the three actual merge inputs against the repository's own data.
pub fn merge_inputs(
    admitted: &Value,
    requests: (&Value, &Value),
    (feature, candidate, review): (&str, &Value, &Value),
) -> ShellResult<()> {
    arguments(admitted, ("branch", feature))?;
    same(admitted, requests.0)?;
    same(admitted, requests.1)?;
    self::candidate(candidate, feature)?;
    reviewed(review, candidate, feature)?;
    let source_receipt = candidate["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse::<u64>().ok());
    if candidate["tool_name"] != "implement_feature"
        || review["tool_name"] != "code_review"
        || review["success"] != true
        || review["data"]["agent"] != "REVIEWER"
        || source_receipt.is_none()
        || review["data"]["source_receipt"].as_u64() != source_receipt
    {
        return Err(ShellError::message(
            "merge requires the actual approval of the current candidate",
        ));
    }
    Ok(())
}

/// A returned publication must preserve the admitted candidate, approval and
/// previous integrated repository. These are data receipts, not progress flags.
pub fn merge_result(
    value: &Value,
    (feature, candidate, review): (&str, &Value, &Value),
    prior_ci: &Value,
) -> ShellResult<()> {
    merged(value, feature)?;
    let review_receipt = review["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse::<u64>().ok());
    let source_receipt = candidate["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse::<u64>().ok());
    if review_receipt.is_none()
        || source_receipt.is_none()
        || value["data"]["source_receipt"].as_u64() != source_receipt
        || value["data"]["review_receipt"].as_u64() != review_receipt
        || value["data"]["candidate_sha256"] != candidate["data"]["candidate_sha256"]
        || value["data"]["published_candidates"][feature] != candidate["data"]["candidate_sha256"]
        || value["data"]["source"] != candidate["data"]["source"]
    {
        return Err(ShellError::message(
            "publication differs from the admitted candidate or approval",
        ));
    }
    if prior_ci.is_null() {
        if !value["data"]["base_repository_sha256"].is_null()
            || !value["data"]["base_test_receipt"].is_null()
            || !value["data"]["base_failure_receipt"].is_null()
            || value["data"]["base_publication_events"]
                .as_object()
                .is_none_or(|events| !events.is_empty())
        {
            return Err(ShellError::message("first publication has an unknown base"));
        }
    } else {
        let receipt = prior_ci["data"]["artifact_id"]
            .as_str()
            .and_then(|id| id.strip_prefix("development:"))
            .and_then(|id| id.parse::<u64>().ok());
        if prior_ci["tool_name"] != "run_tests"
            || receipt.is_none()
            || value["data"]["base_repository_sha256"] != prior_ci["data"]["repository_sha256"]
            || value["data"]["base_publication_events"] != prior_ci["data"]["publication_events"]
        {
            return Err(ShellError::message(
                "publication did not extend the actual integrated repository",
            ));
        }
        match prior_ci["success"].as_bool() {
            Some(true) => {
                if value["data"]["base_test_receipt"].as_u64() != receipt
                    || !value["data"]["base_failure_receipt"].is_null()
                    || !value["data"]["repaired_publication_event"].is_null()
                {
                    return Err(ShellError::message(
                        "publication lost the actual successful base validation",
                    ));
                }
            }
            Some(false) => {
                let replaced = prior_ci["data"]["publication_event"].as_u64();
                if prior_ci["data"]["feature"] != feature
                    || !value["data"]["base_test_receipt"].is_null()
                    || value["data"]["base_failure_receipt"].as_u64() != receipt
                    || value["data"]["repaired_publication_event"].as_u64() != replaced
                    || !matches!((source_receipt, replaced), (Some(source), Some(publication)) if source > publication)
                {
                    return Err(ShellError::message(
                        "repair does not replace the actually failed publication",
                    ));
                }
            }
            None => return Err(ShellError::message("base validation has no actual result")),
        }
    }
    Ok(())
}

pub fn ci_tests(merge: &Value, tests: (&Value, &Value)) -> ShellResult<()> {
    use super::global::{FEATURE_A, FEATURE_B};
    let candidates = merge["data"]["published_candidates"]
        .as_object()
        .ok_or_else(|| ShellError::message("missing published repository inputs"))?;
    let merged_feature = merge["data"]["feature"]
        .as_str()
        .ok_or_else(|| ShellError::message("missing merged feature"))?;
    for (feature, identity) in candidates {
        let test = match feature.as_str() {
            FEATURE_A => tests.0,
            FEATURE_B => tests.1,
            _ => return Err(ShellError::message("unknown published feature")),
        };
        if test["tool_name"] != "write_tests"
            || test["success"] != true
            || test["data"]["agent"] != "TEST_WRITER"
            || test["data"]["component"] != *feature
            || test["data"]["candidate_sha256"] != *identity
            || !test["data"]["test_source"].is_object()
            || !test["data"]["test_sha256"].is_string()
        {
            return Err(ShellError::message(
                "CI test data differ from the published candidate",
            ));
        }
    }
    if candidates.get(merged_feature) != Some(&merge["data"]["candidate_sha256"]) {
        return Err(ShellError::message("CI inputs omit the merged candidate"));
    }
    Ok(())
}

/// Match the trusted CI service's original result to this actual publication.
pub fn ci_publication(result: &Value, publication: &Value) -> ShellResult<()> {
    use super::global::{FEATURE_A, FEATURE_B, SUITE_A, SUITE_B};
    let feature = publication["data"]["feature"]
        .as_str()
        .ok_or_else(|| ShellError::message("missing published feature"))?;
    let suite = match feature {
        FEATURE_A => SUITE_A,
        FEATURE_B => SUITE_B,
        _ => return Err(ShellError::message("unknown published feature")),
    };
    let occurrence = result["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse::<u64>().ok());
    let publication_event = publication["data"]["publication_event"].as_u64();
    if result["tool_name"] != "run_tests"
        || result["data"]["agent"] != "CI_RUNNER"
        || !result["success"].is_boolean()
        || result["data"]["suite"] != suite
        || result["data"]["feature"] != feature
        || result["data"]["repository_sha256"] != publication["data"]["repository_sha256"]
        || result["data"]["published_candidates"][feature]
            != publication["data"]["candidate_sha256"]
        || result["data"]["published_candidates"] != publication["data"]["published_candidates"]
        || result["data"]["publication_event"].as_u64() != publication_event
        || result["data"]["publication_events"][feature].as_u64() != publication_event
        || result["data"]["publication_events"]
            .as_object()
            .is_none_or(|events| {
                publication["data"]["published_candidates"]
                    .as_object()
                    .is_none_or(|candidates| {
                        events.len() != candidates.len()
                            || !candidates
                                .keys()
                                .all(|feature| events.get(feature).is_some_and(Value::is_u64))
                    })
            })
        || !matches!((occurrence, publication_event), (Some(ci), Some(merged)) if ci > merged)
    {
        return Err(ShellError::message(
            "CI result differs from the admitted repository",
        ));
    }
    Ok(())
}

pub fn ci_result(result: &Value, publication: &Value, tests: (&Value, &Value)) -> ShellResult<()> {
    use super::global::{FEATURE_A, FEATURE_B};
    ci_publication(result, publication)?;
    if result["success"] != true {
        return Ok(());
    }
    let candidates = result["data"]["published_candidates"]
        .as_object()
        .ok_or_else(|| ShellError::message("missing CI repository inputs"))?;
    for (feature, identity) in candidates {
        let test = match feature.as_str() {
            FEATURE_A => tests.0,
            FEATURE_B => tests.1,
            _ => return Err(ShellError::message("CI names an unknown published feature")),
        };
        let receipt = test["data"]["artifact_id"]
            .as_str()
            .and_then(|id| id.strip_prefix("development:"))
            .and_then(|id| id.parse::<u64>().ok());
        if test["success"] != true
            || test["data"]["component"] != *feature
            || test["data"]["candidate_sha256"] != *identity
            || receipt.is_none()
            || result["data"]["test_receipts"][feature].as_u64() != receipt
        {
            return Err(ShellError::message(
                "successful CI did not use the received test occurrence",
            ));
        }
        if result["data"]["test_inputs"][feature] != test["data"] {
            return Err(ShellError::message(
                "successful CI used different test definition data",
            ));
        }
    }
    if result["data"]["test_receipts"]
        .as_object()
        .is_none_or(|receipts| receipts.len() != candidates.len())
        || result["data"]["test_inputs"]
            .as_object()
            .is_none_or(|inputs| inputs.len() != candidates.len())
    {
        return Err(ShellError::message(
            "CI test receipts do not match the published inputs",
        ));
    }
    Ok(())
}
