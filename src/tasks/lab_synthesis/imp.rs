//! Data codecs and specimen checks; no protocol operations.
use crate::shell::{ShellError, ShellResult, imp::file_wire::ClosedFile};
use serde_json::Value;
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn same(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        return Err(ShellError::message(
            "laboratory request differs from admitted actual data",
        ));
    }
    Ok(())
}
pub fn assigned(value: &Value, key: &str, expected: &str) -> ShellResult<()> {
    if value.as_object().is_none_or(|o| o.len() != 1) || value[key].as_str() != Some(expected) {
        return Err(ShellError::message(
            "laboratory arguments differ from the assigned specimen",
        ));
    }
    Ok(())
}
pub fn successful(result: &Value) -> ShellResult<()> {
    if result["success"] != true {
        return Err(ShellError::message("terminal laboratory operation failed"));
    }
    Ok(())
}
pub fn assay_arguments(value: &Value) -> ShellResult<()> {
    let object = value
        .as_object()
        .ok_or_else(|| ShellError::message("expected bioassay arguments"))?;
    if object.keys().any(|k| k != "compound" && k != "cell_type")
        || value["compound"].as_str() != Some("compound_purification")
        || object.get("cell_type").is_some_and(|v| !v.is_string())
    {
        return Err(ShellError::message(
            "bioassay must name the actual purified compound; optional cell_type must be a string",
        ));
    }
    Ok(())
}
pub fn assay_file(file: &ClosedFile) -> ShellResult<Value> {
    if file.path != b"service/run_bioassay" || file.completion.0.errno() != 0 {
        return Err(ShellError::message("expected actual bioassay file"));
    }
    let value = decode(&file.bytes)?;
    assay_arguments(&value)?;
    Ok(value)
}
