//! Actual code and validation data; no permission or progress state.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn changed(value: &Value, target: &str) -> ShellResult<()> {
    if value["tool_name"] != "refactor_code"
        || value["success"] != true
        || value["data"]["target"] != target
        || !value["data"]["source"].is_string()
        || !value["data"]["content_sha256"].is_string()
        || value["data"]["working_candidates"]
            .as_object()
            .is_none_or(|v| v.len() != 2)
        || ["module_a", "module_b"].iter().any(|module| {
            value["data"]["working_candidates"][module]
                .as_str()
                .is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        })
    {
        return Err(ShellError::message(
            "refactor did not return the actual code and current content identities",
        ));
    }
    Ok(())
}
pub fn validation(value: &Value, current: &Value, module: &str) -> ShellResult<()> {
    let offset = match module {
        "module_a" => 1,
        "module_b" => 2,
        _ => return Err(ShellError::message("unknown code module")),
    };
    let tool = value["tool_name"].as_str();
    let assigned = match tool {
        Some("code_review") => value["data"]["target"] == module,
        Some("run_tests") => value["data"]["suite"] == format!("{module}_tests"),
        _ => false,
    };
    if !assigned
        || value["success"] != true
        || current[module].as_str().is_none()
        || value["data"]["candidate_sha256"] != current[module]
        || value["data"]["outputs"]
            .as_array()
            .is_none_or(|v| v.len() != 3)
        || value["data"]["outputs"].as_array().is_some_and(|v| {
            v.iter().zip([-3, 0, 7]).any(|(row, input)| {
                row["input"] != input
                    || row["actual"] != input * 2 + offset
                    || row["expected"] != input * 2 + offset
            })
        })
    {
        return Err(ShellError::message(
            "validation is not successful evidence for these current code bytes",
        ));
    }
    Ok(())
}
pub fn publication_inputs(
    arguments: &Value,
    current: &Value,
    (module, review, test): (&str, &Value, &Value),
) -> ShellResult<()> {
    if arguments["branch"] != module
        || review["tool_name"] != "code_review"
        || test["tool_name"] != "run_tests"
    {
        return Err(ShellError::message(
            "publication target or validation kind differs",
        ));
    }
    validation(review, current, module)?;
    validation(test, current, module)
}
pub fn published(value: &Value, current: &Value, module: &str) -> ShellResult<()> {
    if value["tool_name"] != "merge_code"
        || value["success"] != true
        || value["data"]["branch"] != module
        || value["data"]["candidate_sha256"] != current[module]
        || value["data"]["published_candidates"][module] != current[module]
        || value["data"]["validation_receipts"][module]["review"]
            .as_u64()
            .is_none()
        || value["data"]["validation_receipts"][module]["test"]
            .as_u64()
            .is_none()
    {
        return Err(ShellError::message(
            "publication differs from its admitted current candidate",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn publication_requires_successful_matching_review_and_test_of_current_bytes() {
        let current = json!({"module_a":"a".repeat(64),"module_b":"b".repeat(64)});
        let outputs = json!([{"input":-3,"actual":-5,"expected":-5},
            {"input":0,"actual":1,"expected":1},{"input":7,"actual":15,"expected":15}]);
        let review = json!({"tool_name":"code_review","success":true,"data":{
            "target":"module_a","candidate_sha256":current["module_a"],"outputs":outputs}});
        let test = json!({"tool_name":"run_tests","success":true,"data":{
            "suite":"module_a_tests","candidate_sha256":current["module_a"],"outputs":outputs}});
        let arguments = json!({"branch":"module_a"});
        assert!(publication_inputs(&arguments, &current, ("module_a", &review, &test)).is_ok());
        for field in ["review", "test"] {
            for kind in [
                "stale",
                "negative",
                "wrong_target",
                "wrong_output",
                "missing_outputs",
                "wrong_kind",
            ] {
                let mut changed = if field == "review" {
                    review.clone()
                } else {
                    test.clone()
                };
                match kind {
                    "stale" => changed["data"]["candidate_sha256"] = json!("old bytes"),
                    "negative" => changed["success"] = json!(false),
                    "wrong_target" => {
                        changed["data"][if field == "review" { "target" } else { "suite" }] =
                            json!("module_b")
                    }
                    "wrong_output" => changed["data"]["outputs"][0]["actual"] = json!(0),
                    "missing_outputs" => changed["data"]["outputs"] = json!([]),
                    "wrong_kind" => changed["tool_name"] = json!("design_proposal"),
                    _ => unreachable!(),
                }
                let (r, t) = if field == "review" {
                    (&changed, &test)
                } else {
                    (&review, &changed)
                };
                assert!(
                    publication_inputs(&arguments, &current, ("module_a", r, t)).is_err(),
                    "{field}/{kind}"
                );
            }
        }
        let edited = json!({"module_a":"c".repeat(64),"module_b":current["module_b"]});
        assert!(publication_inputs(&arguments, &edited, ("module_a", &review, &test)).is_err());
        assert!(
            publication_inputs(
                &json!({"branch":"module_b"}),
                &current,
                ("module_a", &review, &test)
            )
            .is_err()
        );
    }
}
