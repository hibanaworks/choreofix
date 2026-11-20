//! Exact scientific data correspondence, without task progress or permissions.
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn record(
    value: &Value,
    (tool, scientist, target): (&str, &str, &str),
    inputs: &Value,
) -> ShellResult<u64> {
    let key = match tool {
        "run_analysis" => "sample",
        "run_separation" => "material",
        "run_bioassay" => "compound",
        _ => return Err(ShellError::message("unknown laboratory effect")),
    };
    let data = &value["data"];
    if value["tool_name"] != tool
        || value["success"] != true
        || data["scientist"] != scientist
        || data[key] != target
        || data["status"] != "complete"
        || data["inputs"] != *inputs
        || data["missing_inputs"] != json!([])
    {
        return Err(ShellError::message(
            "laboratory result does not match the actual completed scientific inputs",
        ));
    }
    data["artifact_id"]
        .as_u64()
        .ok_or_else(|| ShellError::message("missing actual scientific artifact"))
}
pub fn compound(value: &Value) -> ShellResult<u64> {
    let raw = record(
        &value["raw"],
        ("run_analysis", "CHEMIST", "raw_materials"),
        &json!({}),
    )?;
    record(
        &value["compound"],
        ("run_separation", "CHEMIST", "compound_purification"),
        &json!({"raw":raw}),
    )
}
pub fn assay(value: &Value) -> ShellResult<(u64, u64)> {
    let compound = compound(value)?;
    let cells = record(
        &value["cells"],
        ("run_separation", "BIOLOGIST", "cell_cultures"),
        &json!({}),
    )?;
    let calibrated = record(
        &value["calibration"],
        ("run_analysis", "BIOLOGIST", "fluorescence"),
        &json!({"prepared_cells":cells}),
    )?;
    let assay = record(
        &value["bioassay"],
        ("run_bioassay", "BIOLOGIST", "compound_purification"),
        &json!({"compound":compound,"prepared_cells":cells,"calibration":calibrated}),
    )?;
    Ok((compound, assay))
}
pub fn quality(value: &Value) -> ShellResult<Value> {
    let (compound, assay) = assay(&value["assay"])?;
    let inputs = json!({"compound":compound,"bioassay":assay});
    let molecular = record(
        &value["molecular"],
        ("run_analysis", "ANALYST", "molecular_detail"),
        &inputs,
    )?;
    let verification = record(
        &value["verification"],
        ("run_separation", "ANALYST", "verification"),
        &inputs,
    )?;
    Ok(
        json!({"compound":compound,"bioassay":assay,"molecular":molecular,"verification":verification}),
    )
}
