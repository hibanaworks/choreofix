//! Actual object versions and completed test receipts, not protocol progress.
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};

pub const MODULES: [&str; 3] = ["module_a", "module_b", "module_c"];
pub const AGENTS: [&str; 3] = ["BUILDER_A", "BUILDER_B", "BUILDER_C"];
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn module(value: &Value) -> ShellResult<usize> {
    MODULES
        .iter()
        .position(|name| value == name)
        .ok_or_else(|| ShellError::message("unknown build module"))
}
pub fn arguments(value: Value, expected: usize) -> ShellResult<Value> {
    let name = value["module_name"]
        .as_str()
        .map(|s| s.trim().to_lowercase().replace(' ', "_"));
    if value.as_object().is_none_or(|fields| fields.len() != 1)
        || name.as_deref() != Some(MODULES[expected])
    {
        return Err(ShellError::message(
            "build arguments differ from the actual input object",
        ));
    }
    Ok(value)
}
pub fn object(result: &Value, expected: usize) -> ShellResult<Value> {
    let obj = &result["data"]["object"];
    if result["tool_name"] != "compile_module"
        || result["success"] != true
        || result["data"]["agent"] != AGENTS[expected]
        || result["data"]["module"] != MODULES[expected]
    {
        return Err(ShellError::message(
            "build did not return its actual assigned object",
        ));
    }
    validate_object(obj, expected)?;
    Ok(obj.clone())
}
pub fn validate_object(obj: &Value, expected: usize) -> ShellResult<()> {
    let names = match expected {
        0 => ["CORE_LIB", "CONFIG"],
        1 => ["CORE_LIB", "DATA_MODELS"],
        _ => ["CONFIG", "DATA_MODELS"],
    };
    if obj["builder"] != AGENTS[expected]
        || obj["module"] != MODULES[expected]
        || obj["inputs"].as_object().is_none_or(|inputs| {
            inputs.len() != names.len() || names.iter().any(|name| !inputs.contains_key(*name))
        })
        || !obj["content_sha256"].is_string()
    {
        return Err(ShellError::message(
            "build object differs from its actual module",
        ));
    }
    occurrence(obj)?;
    let bytes = obj["bytes"]
        .as_str()
        .ok_or_else(|| ShellError::message("object bytes missing"))?;
    if decode(bytes.as_bytes())? != json!({"module":MODULES[expected],"inputs":obj["inputs"]}) {
        return Err(ShellError::message(
            "actual object bytes differ from reported build inputs",
        ));
    }
    Ok(())
}
// This is the common world's actual object occurrence, not a task cursor. Its
// sequence identifies which of two completed objects read a shared file later.
fn occurrence(object: &Value) -> ShellResult<u64> {
    object["id"]
        .as_str()
        .and_then(|id| id.strip_prefix("object:"))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| ShellError::message("object occurrence missing"))
}
pub fn test_result(result: &Value, expected: usize) -> ShellResult<()> {
    if result["tool_name"] != "run_build_tests"
        || !result["success"].is_boolean()
        || result["data"]["module"] != MODULES[expected]
        || result["data"]["agent"] != "TEST_RUNNER"
        || !result["data"]["receipt"].is_string()
        || result["data"]["input_object"]["module"] != MODULES[expected]
    {
        return Err(ShellError::message(
            "test receipt differs from its actual build input",
        ));
    }
    Ok(())
}
pub fn current(objects: &[Value; 3], tests: &[Value; 3], index: usize) -> ShellResult<bool> {
    let Some(inputs) = objects[index]["inputs"].as_object() else {
        return Ok(false);
    };
    if tests[index]["success"] != true || tests[index]["data"]["input_object"] != objects[index] {
        return Ok(false);
    }
    for (name, actual) in inputs {
        let latest = objects
            .iter()
            .filter(|obj| obj["inputs"].get(name).is_some())
            .map(|obj| Ok((occurrence(obj)?, &obj["inputs"][name])))
            .collect::<ShellResult<Vec<_>>>()?
            .into_iter()
            .max_by_key(|(occurrence, _)| *occurrence)
            .ok_or_else(|| ShellError::message("current shared input missing"))?;
        if actual != latest.1 {
            return Ok(false);
        }
    }
    Ok(true)
}
pub fn reconsider(objects: &[Value; 3], tests: &[Value; 3], index: usize) -> ShellResult<Value> {
    test_result(&tests[index], index)?;
    if current(objects, tests, index)? {
        return Err(ShellError::message(
            "a current positively tested object needs no repair",
        ));
    }
    Ok(json!({"module":MODULES[index],"test":tests[index],"object":objects[index]}))
}
pub fn finish(objects: &[Value; 3], tests: &[Value; 3]) -> ShellResult<Value> {
    for index in 0..3 {
        test_result(&tests[index], index)?;
        if !current(objects, tests, index)? {
            return Err(ShellError::message(
                "link needs every current tested object",
            ));
        }
    }
    Ok(json!({"success":true,"objects":objects,"tests":tests}))
}
pub fn candidate(objects: &[Value; 3], tests: &[Value; 3]) -> ShellResult<Value> {
    for index in 0..3 {
        validate_object(&objects[index], index)?;
        test_result(&tests[index], index)?;
    }
    Ok(json!({"objects":objects,"tests":tests}))
}
pub fn needed(objects: &[Value; 3], tests: &[Value; 3]) -> ShellResult<Option<usize>> {
    for index in 0..3 {
        validate_object(&objects[index], index)?;
        test_result(&tests[index], index)?;
        if !current(objects, tests, index)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
pub fn link_arguments(value: Value) -> ShellResult<Value> {
    let mut names: Vec<_> = value["modules"]
        .as_str()
        .unwrap_or("")
        .split(',')
        .map(|name| name.trim().to_lowercase().replace(' ', "_"))
        .collect();
    names.sort_unstable();
    if value.as_object().is_none_or(|fields| fields.len() != 1) || names != MODULES {
        return Err(ShellError::message(
            "link requires every module exactly once",
        ));
    }
    Ok(value)
}
pub fn linked(result: &Value, objects: &[Value; 3], tests: &[Value; 3]) -> ShellResult<()> {
    finish(objects, tests)?;
    let inputs = json!({"module_a":objects[0],"module_b":objects[1],"module_c":objects[2]});
    let checks = json!({"module_a":{"run_build_tests":tests[0]["data"]["receipt"]},
        "module_b":{"run_build_tests":tests[1]["data"]["receipt"]},
        "module_c":{"run_build_tests":tests[2]["data"]["receipt"]}});
    if result["tool_name"] != "link_modules"
        || result["success"] != true
        || result["data"]["agent"] != "INTEGRATOR"
        || result["data"]["status"] != "linked"
        || result["data"]["issues"] != json!([])
        || result["data"]["inputs"] != inputs
        || result["data"]["checks"] != checks
        || !result["data"]["binary"]["id"].is_string()
        || result["data"]["binary"]["bytes"]
            .as_str()
            .is_none_or(|bytes| decode(bytes.as_bytes()).ok().as_ref() != Some(&inputs))
    {
        return Err(ShellError::message(
            "link did not use the current objects and completed test receipts",
        ));
    }
    Ok(())
}
pub fn candidate_parts(value: &Value) -> ShellResult<([Value; 3], [Value; 3])> {
    let parse = |key: &str| {
        value[key]
            .as_array()
            .and_then(|rows| rows.clone().try_into().ok())
            .ok_or_else(|| {
                ShellError::message("candidate must contain the actual three objects and tests")
            })
    };
    let objects = parse("objects")?;
    let tests = parse("tests")?;
    candidate(&objects, &tests)?;
    Ok((objects, tests))
}

/// The cell contains the received candidate itself. It is the resolver's
/// actual decision input, not a selected-module flag or a protocol cursor.
pub fn selected<const EXPECTED: usize>(
    input: &core::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let choice = candidate_parts(&value)
        .and_then(|(objects, tests)| needed(&objects, &tests))
        .ok()
        .flatten()
        .map(|index| {
            if index == EXPECTED {
                DecisionArm::Left
            } else {
                DecisionArm::Right
            }
        })
        .ok_or_else(ResolverError::reject);
    input.set(value);
    choice
}
