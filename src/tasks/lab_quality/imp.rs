//! Scientific value correspondence, without ownership or workflow state.
use crate::shell::{ShellError, ShellResult};
use serde_json::{Value, json};

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn record(
    value: &Value,
    (tool, actor, arguments): (&str, &str, &Value),
    inputs: &Value,
) -> ShellResult<u64> {
    let data = &value["data"];
    if value["tool_name"] != tool
        || value["success"] != true
        || data["scientist"] != actor
        || data["status"] != "complete"
        || data["inputs"] != *inputs
        || data["missing_inputs"] != json!([])
        || arguments.as_object().is_none_or(|arguments| {
            arguments
                .iter()
                .any(|(key, expected)| data[key] != *expected)
        })
    {
        return Err(ShellError::message(
            "scientific result differs from its actual material inputs",
        ));
    }
    data["artifact_id"]
        .as_u64()
        .ok_or_else(|| ShellError::message("scientific result has no completed artifact"))
}
pub fn specimen(materials: &Value) -> ShellResult<(u64, u64)> {
    let synthesis = &materials["synthesis"];
    let compound = synthesis["data"]["compound"]
        .as_str()
        .ok_or_else(|| ShellError::message("missing actual synthesized batch name"))?;
    let synthesis = record(
        synthesis,
        ("run_synthesis", "CHEMIST", &json!({"compound":compound})),
        &json!({}),
    )?;
    let analysis = record(
        &materials["initial_analysis"],
        (
            "run_analysis",
            "CHEMIST",
            &json!({"sample":"synthesized_compound"}),
        ),
        &json!({"synthesis":synthesis}),
    )?;
    let purification = record(
        &materials["purification"],
        (
            "run_separation",
            "CHEMIST",
            &json!({"material":"compound_purification"}),
        ),
        &json!({"synthesis":synthesis,"analysis":analysis}),
    )?;
    if materials["purification"]["data"]["compound"] != compound {
        return Err(ShellError::message(
            "purification substituted the actual batch name",
        ));
    }
    Ok((synthesis, purification))
}
pub fn qc_inputs(arguments: &Value, materials: &Value) -> ShellResult<Value> {
    let (synthesis, purification) = specimen(materials)?;
    match arguments["scientist"].as_str() {
        Some("CHEMIST") => {
            let bytes = encode(arguments)?;
            super::arguments::Arguments::<9>::try_from(bytes.as_slice())?;
            let analysis = record(
                &materials["analysis"],
                (
                    "run_analysis",
                    "CHEMIST",
                    &json!({"sample":"synthesized_compound"}),
                ),
                &json!({"synthesis":synthesis}),
            )?;
            Ok(json!({"synthesis":synthesis,"purification":purification,"analysis":analysis}))
        }
        Some("BIOLOGIST") => {
            let bytes = encode(arguments)?;
            super::arguments::Arguments::<10>::try_from(bytes.as_slice())?;
            let cells = record(
                &materials["cells"],
                (
                    "run_separation",
                    "BIOLOGIST",
                    &json!({"material":"cell_cultures"}),
                ),
                &json!({}),
            )?;
            let calibration = record(
                &materials["calibration"],
                (
                    "run_analysis",
                    "BIOLOGIST",
                    &json!({"sample":"fluorescence"}),
                ),
                &json!({"prepared_cells":cells}),
            )?;
            let compound = &materials["synthesis"]["data"]["compound"];
            let bioassay = record(
                &materials["bioassay"],
                ("run_bioassay", "BIOLOGIST", &json!({"compound":compound})),
                &json!({"synthesis":synthesis,"purification":purification,
                    "prepared_cells":cells,"calibration":calibration}),
            )?;
            Ok(json!({"synthesis":synthesis,"purification":purification,"bioassay":bioassay}))
        }
        _ => Err(ShellError::message("unknown scientific QC subject")),
    }
}
pub fn qc_packet(packet: &Value, actual_materials: &Value) -> ShellResult<Value> {
    if packet["materials"] != *actual_materials
        || packet.as_object().is_none_or(|fields| fields.len() != 2)
    {
        return Err(ShellError::message(
            "QC request substituted the actual scientific material",
        ));
    }
    qc_inputs(&packet["arguments"], actual_materials)
}
pub fn qc_result(result: &Value, arguments: &Value, inputs: &Value) -> ShellResult<()> {
    let data = &result["data"];
    let verdict = result["success"]
        .as_bool()
        .ok_or_else(|| ShellError::message("QC response lacks its actual decision"))?;
    if result["tool_name"] != "verify_result"
        || data["scientist"] != arguments["scientist"]
        || data["data_type"] != arguments["data_type"]
        || data["qc_agent"] != "QC_CONTROLLER"
        || data["inputs"] != *inputs
        || data["missing_inputs"] != json!([])
        || data["status"] != if verdict { "complete" } else { "rejected" }
        || if verdict {
            !data["artifact_id"].is_u64()
        } else {
            !data["artifact_id"].is_null()
        }
    {
        return Err(ShellError::message(
            "QC decision differs from the actual subject, material or result",
        ));
    }
    Ok(())
}

/// The final instrument sees the same actual batch, approved trials and quality data.
pub fn final_inputs(quality: &Value) -> ShellResult<Value> {
    let materials = &quality["materials"];
    let (synthesis, purification) = specimen(materials)?;
    let chem_args = json!({"scientist":"CHEMIST","data_type":"purification"});
    let bio_args = json!({"scientist":"BIOLOGIST","data_type":"bioassay"});
    let chem_inputs = qc_inputs(&chem_args, materials)?;
    let bio_inputs = qc_inputs(&bio_args, materials)?;
    qc_result(&quality["chemist_qc"], &chem_args, &chem_inputs)?;
    qc_result(&quality["biologist_qc"], &bio_args, &bio_inputs)?;
    if quality["chemist_qc"]["success"] != true || quality["biologist_qc"]["success"] != true {
        return Err(ShellError::message(
            "final validation lacks two actual positive QC results",
        ));
    }
    let bioassay = bio_inputs["bioassay"]
        .as_u64()
        .ok_or_else(|| ShellError::message("missing actual successful biological trial"))?;
    let measured_inputs =
        json!({"synthesis":synthesis,"purification":purification,"bioassay":bioassay});
    let molecular = record(
        &quality["molecular"],
        (
            "run_analysis",
            "ANALYST",
            &json!({"sample":"molecular_detail"}),
        ),
        &measured_inputs,
    )?;
    let verification = record(
        &quality["verification"],
        (
            "run_separation",
            "ANALYST",
            &json!({"material":"verification"}),
        ),
        &measured_inputs,
    )?;
    Ok(
        json!({"synthesis":synthesis,"purification":purification,"bioassay":bioassay,
        "purification_qc":quality["chemist_qc"]["data"]["artifact_id"],
        "bioassay_qc":quality["biologist_qc"]["data"]["artifact_id"],
        "molecular":molecular,"verification":verification}),
    )
}
