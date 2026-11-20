//! JSON values only; resource authority stays with the projected Endpoints.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};

pub fn same_value(actual: &Value, expected: &Value) -> ShellResult<()> {
    if actual != expected {
        return Err(ShellError::message(
            "scientific data differs from the admitted material",
        ));
    }
    Ok(())
}

pub fn arguments<const OP: u8>(value: &Value) -> ShellResult<Value> {
    Ok(super::arguments::Arguments::<OP>::try_from(encode(value)?.as_slice())?.into_value())
}

fn analyzed_inputs(material: &Value) -> ShellResult<Value> {
    let synthesis = &material["synthesis"];
    let synthesized = scientific_id(
        synthesis,
        (
            "run_synthesis",
            "CHEMIST",
            &json!({"compound":synthesis["data"]["compound"]}),
        ),
        &json!({}),
    )?;
    let analysis = scientific_id(
        &material["analysis"],
        (
            "run_analysis",
            "CHEMIST",
            &json!({"sample":"synthesized_compound"}),
        ),
        &json!({"synthesis":synthesized}),
    )?;
    Ok(json!({"synthesis":synthesized,"analysis":analysis}))
}

fn purified_inputs(material: &Value) -> ShellResult<Value> {
    let chemical = analyzed_inputs(material)?;
    let purified = scientific_id(
        &material["purification"],
        (
            "run_separation",
            "CHEMIST",
            &json!({"material":"compound_purification"}),
        ),
        &chemical,
    )?;
    same_value(
        &material["purification"]["data"]["compound"],
        &material["synthesis"]["data"]["compound"],
    )?;
    Ok(json!({"synthesis":chemical["synthesis"],"purification":purified}))
}

/// Actual scientific inputs, independent of source order and Endpoint permission.
pub fn instrument_inputs<const OP: u8>(packet: &Value) -> ShellResult<Value> {
    let args = arguments::<OP>(&packet["arguments"])?;
    let material = &packet["material"];
    match OP {
        1 => {
            let synthesis = &material["synthesis"];
            let id = scientific_id(
                synthesis,
                (
                    "run_synthesis",
                    "CHEMIST",
                    &json!({"compound":synthesis["data"]["compound"]}),
                ),
                &json!({}),
            )?;
            Ok(json!({"synthesis":id}))
        }
        2 => analyzed_inputs(material),
        3 => Ok(json!({})),
        4 => {
            let id = scientific_id(
                &material["cells"],
                (
                    "run_separation",
                    "BIOLOGIST",
                    &json!({"material":"cell_cultures"}),
                ),
                &json!({}),
            )?;
            Ok(json!({"prepared_cells":id}))
        }
        5 => {
            same_value(
                &args["compound"],
                &material["synthesis"]["data"]["compound"],
            )?;
            let mut inputs = purified_inputs(material)?;
            let cells = scientific_id(
                &material["cells"],
                (
                    "run_separation",
                    "BIOLOGIST",
                    &json!({"material":"cell_cultures"}),
                ),
                &json!({}),
            )?;
            let calibration = scientific_id(
                &material["calibration"],
                (
                    "run_analysis",
                    "BIOLOGIST",
                    &json!({"sample":"fluorescence"}),
                ),
                &json!({"prepared_cells":cells}),
            )?;
            inputs["prepared_cells"] = json!(cells);
            inputs["calibration"] = json!(calibration);
            Ok(inputs)
        }
        6..=8 => {
            let mut inputs = quality_input(&json!({
                "arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"},
                "material":material
            }))?;
            if OP == 6 {
                same_value(&args["sample"], &material["synthesis"]["data"]["compound"])?;
            }
            if OP == 7 {
                let chromatography = scientific_id(
                    &packet["chromatography"],
                    (
                        "run_chromatography",
                        "ANALYST",
                        &json!({"sample":material["synthesis"]["data"]["compound"]}),
                    ),
                    &inputs,
                )?;
                inputs["chromatography"] = json!(chromatography);
            }
            Ok(inputs)
        }
        10 => {
            let report = &packet["report"];
            check_report(report)?;
            if report["quality"]["success"] != true {
                return Err(ShellError::message(
                    "final validation requires the actual positive QC result",
                ));
            }
            let mut inputs = instrument_inputs::<8>(
                &json!({"arguments":{"material":"verification"},"material":report["material"]}),
            )?;
            inputs["bioassay_qc"] = report["quality"]["data"]["artifact_id"].clone();
            inputs["chromatography"] = report["chromatography"]["data"]["artifact_id"].clone();
            inputs["molecular"] = report["molecular"]["data"]["artifact_id"].clone();
            inputs["verification"] = report["verification"]["data"]["artifact_id"].clone();
            Ok(inputs)
        }
        _ => Err(ShellError::message("unknown scientific instrument input")),
    }
}

pub fn instrument_result<const OP: u8>(actual: &Value, packet: &Value) -> ShellResult<u64> {
    let inputs = instrument_inputs::<OP>(packet)?;
    let (tool, scientist) = match OP {
        1 | 10 => ("run_analysis", "CHEMIST"),
        2 => ("run_separation", "CHEMIST"),
        3 => ("run_separation", "BIOLOGIST"),
        4 => ("run_analysis", "BIOLOGIST"),
        5 => ("run_bioassay", "BIOLOGIST"),
        6 => ("run_chromatography", "ANALYST"),
        7 => ("run_analysis", "ANALYST"),
        8 => ("run_separation", "ANALYST"),
        _ => return Err(ShellError::message("unknown scientific instrument result")),
    };
    scientific_id(actual, (tool, scientist, &packet["arguments"]), &inputs)
}

/// Completed analytical records and their actual QC decision, including a
/// rejected batch whose unstarted chromatography was intentionally omitted.
pub fn check_report(report: &Value) -> ShellResult<()> {
    quality_result(
        &report["quality"],
        &json!({
            "arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"},
            "material":report["material"]
        }),
    )?;
    instrument_result::<8>(
        &report["verification"],
        &json!({
            "arguments":{"material":"verification"}, "material":report["material"]
        }),
    )?;
    match (
        report["chromatography"].is_null(),
        report["molecular"].is_null(),
    ) {
        (false, false) => {
            instrument_result::<6>(
                &report["chromatography"],
                &json!({
                    "arguments":{"sample":report["material"]["synthesis"]["data"]["compound"]},
                    "material":report["material"]
                }),
            )?;
            instrument_result::<7>(
                &report["molecular"],
                &json!({
                    "arguments":{"sample":"molecular_detail"}, "material":report["material"],
                    "chromatography":report["chromatography"]
                }),
            )?;
        }
        (true, true) if report["quality"]["success"] == false => (),
        _ => {
            return Err(ShellError::message(
                "analyst report omits required actual measurements",
            ));
        }
    }
    Ok(())
}

pub fn batch_cause(cause: &Value, batch: &Value) -> ShellResult<()> {
    if cause == batch {
        return Ok(());
    }
    if batch["success"] != true
        || cause["tool_name"] != "verify_result"
        || cause["data"]["scientist"] != "BIOLOGIST"
        || cause["data"]["data_type"] != "bioassay"
        || cause["data"]["qc_agent"] != "QC_CONTROLLER"
        || cause["data"]["inputs"]["synthesis"] != batch["data"]["artifact_id"]
        || cause["data"]["missing_inputs"] != json!([])
    {
        return Err(ShellError::message(
            "result names another actual scientific batch",
        ));
    }
    Ok(())
}

pub fn decide(
    actual: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = actual.take();
    let selected = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    actual.set(value);
    selected
}

pub fn decide_synthesis_failure(
    actual: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = actual.take();
    let selected = match value["data"]["status"].as_str() {
        Some("synthesis_failed") => Ok(DecisionArm::Left),
        Some("reagent_exhausted") => Ok(DecisionArm::Right),
        _ => Err(ResolverError::reject()),
    };
    actual.set(value);
    selected
}

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|error| ShellError::message(error.to_string()))
}

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}

/// Actual completed scientific data, with exact material inputs and arguments.
pub fn scientific_id(
    result: &Value,
    (tool, scientist, arguments): (&str, &str, &Value),
    inputs: &Value,
) -> ShellResult<u64> {
    let id = result["data"]["artifact_id"].as_u64();
    if result["tool_name"] != tool
        || result["success"] != true
        || result["data"]["scientist"] != scientist
        || result["data"]["status"] != "complete"
        || result["data"]["inputs"] != *inputs
        || result["data"]["missing_inputs"] != serde_json::json!([])
        || arguments.as_object().is_none_or(|arguments| {
            arguments
                .iter()
                .any(|(key, value)| result["data"][key] != *value)
        })
        || id.is_none()
    {
        return Err(ShellError::message(
            "scientific result has different actual material inputs",
        ));
    }
    Ok(id.expect("validated scientific artifact"))
}

/// Derive the actual QC inputs from the submitted scientific records.
/// No artifact ID is accepted merely because a request names that number.
pub fn quality_input(packet: &Value) -> ShellResult<Value> {
    use serde_json::json;
    if packet.as_object().is_none_or(|object| {
        object.len() != 2 || !object.contains_key("arguments") || !object.contains_key("material")
    }) {
        return Err(ShellError::message(
            "QC requires actual arguments and material records",
        ));
    }
    super::arguments::Arguments::<9>::try_from(encode(&packet["arguments"])?.as_slice())?;
    let material = &packet["material"];
    let synthesis = &material["synthesis"];
    let compound = &synthesis["data"]["compound"];
    let synthesis_args = json!({"compound":compound});
    synthesis_result(synthesis, &synthesis_args)?;
    let synthesized = scientific_id(
        synthesis,
        ("run_synthesis", "CHEMIST", &synthesis_args),
        &json!({}),
    )?;
    let analysis = scientific_id(
        &material["analysis"],
        (
            "run_analysis",
            "CHEMIST",
            &json!({"sample":"synthesized_compound"}),
        ),
        &json!({"synthesis":synthesized}),
    )?;
    let purified = scientific_id(
        &material["purification"],
        (
            "run_separation",
            "CHEMIST",
            &json!({"material":"compound_purification"}),
        ),
        &json!({"synthesis":synthesized,"analysis":analysis}),
    )?;
    if material["purification"]["data"]["compound"] != *compound {
        return Err(ShellError::message("purification names another compound"));
    }
    let cells = scientific_id(
        &material["cells"],
        (
            "run_separation",
            "BIOLOGIST",
            &json!({"material":"cell_cultures"}),
        ),
        &json!({}),
    )?;
    let calibrated = scientific_id(
        &material["calibration"],
        (
            "run_analysis",
            "BIOLOGIST",
            &json!({"sample":"fluorescence"}),
        ),
        &json!({"prepared_cells":cells}),
    )?;
    let assay_args =
        json!({"compound":compound,"cell_type":material["bioassay"]["data"]["cell_type"]});
    super::arguments::Arguments::<5>::try_from(encode(&assay_args)?.as_slice())?;
    let assay = scientific_id(
        &material["bioassay"],
        ("run_bioassay", "BIOLOGIST", &assay_args),
        &json!({"synthesis":synthesized,"purification":purified,"prepared_cells":cells,"calibration":calibrated}),
    )?;
    Ok(json!({"synthesis":synthesized,"purification":purified,"bioassay":assay}))
}

pub fn quality_result(result: &Value, packet: &Value) -> ShellResult<()> {
    let expected = quality_input(packet)?;
    let success = result["success"].as_bool();
    if result["tool_name"] != "verify_result"
        || result["data"]["scientist"] != "BIOLOGIST"
        || result["data"]["data_type"] != "bioassay"
        || result["data"]["qc_agent"] != "QC_CONTROLLER"
        || result["data"]["inputs"] != expected
        || result["data"]["missing_inputs"] != serde_json::json!([])
        || match success {
            Some(true) => {
                result["data"]["status"] != "complete"
                    || result["data"]["artifact_id"].as_u64().is_none()
            }
            Some(false) => {
                result["data"]["status"] != "rejected" || !result["data"]["artifact_id"].is_null()
            }
            None => true,
        }
    {
        return Err(ShellError::message(
            "QC result differs from the actual submitted bioassay",
        ));
    }
    Ok(())
}

/// Validate actual synthesis data, including ordinary failure/exhaustion.
pub fn synthesis_result(result: &Value, arguments: &Value) -> ShellResult<()> {
    let success = result["success"]
        .as_bool()
        .ok_or_else(|| ShellError::message("synthesis has no actual verdict"))?;
    let status = result["data"]["status"].as_str();
    let remaining = result["data"]["reagent_units_remaining"].as_u64();
    if result["tool_name"] != "run_synthesis"
        || result["data"]["scientist"] != "CHEMIST"
        || result["data"]["compound"] != arguments["compound"]
        || result["data"]["inputs"] != serde_json::json!({})
        || result["data"]["missing_inputs"] != serde_json::json!([])
        || remaining.is_none()
        || if success {
            status != Some("complete") || result["data"]["artifact_id"].as_u64().is_none()
        } else {
            !matches!(status, Some("synthesis_failed" | "reagent_exhausted"))
                || !result["data"]["artifact_id"].is_null()
                || (status == Some("reagent_exhausted") && remaining != Some(0))
        }
    {
        return Err(ShellError::message(
            "synthesis result differs from the actual request or material outcome",
        ));
    }
    Ok(())
}

/// The actual terminal result, not a completion flag or a new stock authority.
pub struct SynthesisTerminal<const SUCCESS: bool>(Vec<u8>);

impl<const SUCCESS: bool> SynthesisTerminal<SUCCESS> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated synthesis terminal result")
    }
}

impl<const SUCCESS: bool> TryFrom<&[u8]> for SynthesisTerminal<SUCCESS> {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const SUCCESS: bool> WireEncode for SynthesisTerminal<SUCCESS> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const SUCCESS: bool> WirePayload for SynthesisTerminal<SUCCESS> {
    const SCHEMA_ID: u32 = 0xac13_2100 | SUCCESS as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let result: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let arguments = serde_json::json!({"compound":result["data"]["compound"]});
        super::arguments::Arguments::<0>::try_from(
            encode(&arguments)
                .map_err(|_| CodecError::Malformed)?
                .as_slice(),
        )?;
        synthesis_result(&result, &arguments).map_err(|_| CodecError::Malformed)?;
        if result["success"] != SUCCESS
            || (!SUCCESS && result["data"]["status"] != "reagent_exhausted")
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
