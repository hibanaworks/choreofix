//! Build arguments and actual result data, independent of protocol progress.
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn compile_arguments(value: Value, module: &str) -> ShellResult<Value> {
    let supplied = value["module_name"]
        .as_str()
        .map(|name| name.trim().to_lowercase().replace(' ', "_"));
    if value.as_object().is_none_or(|fields| fields.len() != 1)
        || supplied.as_deref() != Some(module)
    {
        return Err(ShellError::message(
            "build arguments differ from the assigned module",
        ));
    }
    Ok(value)
}
pub fn compiled(result: &Value, agent: &str, module: &str) -> ShellResult<Value> {
    let data = &result["data"];
    let object = &data["object"];
    if result["tool_name"] != "compile_module"
        || result["success"] != true
        || data["agent"] != agent
        || data["module"] != module
        || data["status"] != "compiled"
        || object["builder"] != agent
        || object["module"] != module
        || !object["id"].is_string()
        || !object["bytes"].is_string()
        || !object["content_sha256"].is_string()
        || !object["inputs"]["CORE_LIB"].is_object()
        || !object["inputs"]["SHARED_TYPES"].is_object()
    {
        return Err(ShellError::message(
            "compile result is not this module's actual object",
        ));
    }
    let content = decode(object["bytes"].as_str().unwrap().as_bytes())?;
    if content["module"] != module || content["inputs"] != object["inputs"] {
        return Err(ShellError::message(
            "object bytes differ from the reported build inputs",
        ));
    }
    Ok(object.clone())
}
pub fn link_arguments(value: Value) -> ShellResult<Value> {
    let mut modules: Vec<_> = value["modules"]
        .as_str()
        .unwrap_or("")
        .split(',')
        .map(|name| name.trim().to_lowercase().replace(' ', "_"))
        .collect();
    modules.sort_unstable();
    if value.as_object().is_none_or(|fields| fields.len() != 1)
        || modules != ["module_a", "module_b"]
    {
        return Err(ShellError::message(
            "link requires both actual modules exactly once",
        ));
    }
    Ok(value)
}
pub fn linked(result: &Value, objects: &[Value; 2]) -> ShellResult<()> {
    let data = &result["data"];
    let inputs = serde_json::json!({"module_a":objects[0], "module_b":objects[1]});
    if result["tool_name"] != "link_modules"
        || result["success"] != true
        || data["agent"] != "INTEGRATOR"
        || data["status"] != "linked"
        || data["issues"] != serde_json::json!([])
        || data["inputs"] != inputs
        || !data["binary"]["id"].is_string()
        || data["binary"]["bytes"]
            .as_str()
            .is_none_or(|bytes| decode(bytes.as_bytes()).ok().as_ref() != Some(&inputs))
        || objects[0]["inputs"]["SHARED_TYPES"] != objects[1]["inputs"]["SHARED_TYPES"]
    {
        return Err(ShellError::message(
            "linked binary differs from the completed current objects",
        ));
    }
    Ok(())
}
