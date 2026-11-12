//! Actual 14H scientific values. No Endpoint operations or workflow state.
pub use crate::imp::json::{decode, encode, same};
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

/// Scientific operations, not a participant's current position. Dynamic
/// compound names are checked against the received identity at the effect owner.
pub fn analysis(
    step: u8,
) -> ShellResult<(
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
)> {
    Ok(match step {
        0 => (
            "CHEMIST",
            "run_hplc_analysis",
            "sample",
            Some("synthesis_qc"),
        ),
        1 => (
            "BIOLOGIST",
            "run_cell_assay",
            "assay_type",
            Some("culture_prep"),
        ),
        2 => (
            "TOXICOLOGIST",
            "run_mass_spec_analysis",
            "sample",
            Some("baseline_markers"),
        ),
        3 => (
            "CHEMIST",
            "run_mass_spec_analysis",
            "sample",
            Some("molecular_id"),
        ),
        4 => (
            "FORMULATION_SCIENTIST",
            "prepare_formulation",
            "compound",
            None,
        ),
        5 => (
            "BIOLOGIST",
            "run_cell_assay",
            "assay_type",
            Some("bioassay"),
        ),
        6 => (
            "BIOLOGIST",
            "run_hplc_analysis",
            "sample",
            Some("stability"),
        ),
        7 => (
            "TOXICOLOGIST",
            "run_hplc_analysis",
            "sample",
            Some("purity_check"),
        ),
        8 => (
            "TOXICOLOGIST",
            "run_cell_assay",
            "assay_type",
            Some("cytotoxicity"),
        ),
        9 => (
            "CLINICAL_LEAD",
            "review_clinical_data",
            "data_type",
            Some("combined_review"),
        ),
        10 => (
            "REGULATORY_SPECIALIST",
            "verify_regulatory_compliance",
            "review_type",
            Some("initial_review"),
        ),
        11 => (
            "REGULATORY_SPECIALIST",
            "verify_regulatory_compliance",
            "review_type",
            Some("hold_review"),
        ),
        12 => (
            "PROJECT_DIRECTOR",
            "submit_project_decision",
            "outcome",
            Some("continue"),
        ),
        13 => (
            "PROJECT_DIRECTOR",
            "submit_project_decision",
            "outcome",
            Some("retry"),
        ),
        14 => (
            "PROJECT_DIRECTOR",
            "submit_project_decision",
            "outcome",
            Some("abort"),
        ),
        15 => (
            "CHEMIST",
            "run_hplc_analysis",
            "sample",
            Some("documentation"),
        ),
        _ => {
            return Err(ShellError::message(
                "unknown multi-round scientific operation",
            ));
        }
    })
}

pub fn arguments(step: u8, value: &Value) -> ShellResult<()> {
    let (_, _, key, expected) = analysis(step)?;
    if value.as_object().is_none_or(|object| object.len() != 1)
        || value[key].as_str().is_none_or(|value| {
            value.is_empty() || expected.is_some_and(|expected| value != expected)
        })
    {
        return Err(ShellError::message(
            "arguments differ from the scientific operation",
        ));
    }
    Ok(())
}

/// Decode the actual argument file returned by the admitted ChoreoFS close.
/// This validates data only; the caller performs the Endpoint operations.
pub fn analysis_file<const STEP: u8>(
    file: &crate::shell::imp::file_wire::ClosedFile,
) -> ShellResult<Value> {
    let (_, tool, key, _) = analysis(STEP)?;
    let path = format!("service/{tool}");
    let value = file.arguments(path.as_bytes(), key)?;
    arguments(STEP, &value)?;
    Ok(value)
}

/// A guest's explicit evidence submission requests the existing Endpoint
/// transfer. This decodes data only; the localside performs that transfer.
pub fn evidence_file(
    file: &crate::shell::imp::file_wire::ClosedFile,
    expected: &str,
) -> ShellResult<()> {
    let value = file.arguments(b"service/submit_evidence", "evidence")?;
    same(&value["evidence"], &Value::String(expected.into()))
}

/// Failure and exhaustion are real occurrences, but do not create a positive
/// artifact. Their run-local event IDs let clinical data cite those outcomes.
pub fn reply(step: u8, value: &Value) -> ShellResult<u64> {
    let (actor, tool, key, _) = analysis(step)?;
    arguments(step, &serde_json::json!({key:value["data"][key]}))?;
    let event = value["data"]["event_id"]
        .as_u64()
        .ok_or_else(|| ShellError::message("scientific result has no occurrence"))?;
    let success = value["success"]
        .as_bool()
        .ok_or_else(|| ShellError::message("scientific result has no actual verdict"))?;
    let status = value["data"]["status"].as_str();
    let completed = if success {
        status == Some("complete")
    } else {
        match step {
            5 | 8 => matches!(status, Some("injected_failure" | "exhausted")),
            10 | 11 => status == Some("clinical_hold"),
            0 | 1 | 4 | 6 | 7 | 15 => status == Some("injected_failure"),
            _ => false,
        }
    };
    if value["tool_name"] != tool
        || value["data"]["scientist"] != actor
        || !completed
        || (success && value["data"]["artifact_id"] != event)
        || (!success && !value["data"]["artifact_id"].is_null())
    {
        return Err(ShellError::message(
            "result differs from the actual scientific occurrence",
        ));
    }
    Ok(event)
}

pub fn formulation_input(identity: &Value, input: &Value) -> ShellResult<()> {
    let id = reply(3, identity)?;
    arguments(4, input)?;
    if identity["success"] != true
        || identity["data"]["compound"] != format!("compound-{id}")
        || input["compound"] != identity["data"]["compound"]
    {
        return Err(ShellError::message(
            "formulation differs from the actual compound",
        ));
    }
    Ok(())
}

pub fn formulation_receipt(identity: &Value, actual: &Value) -> ShellResult<()> {
    let compound = reply(3, identity)?;
    reply(4, actual)?;
    formulation_input(
        identity,
        &serde_json::json!({"compound":actual["data"]["compound"]}),
    )?;
    if actual["data"]["inputs"]["compound"] != compound {
        return Err(ShellError::message(
            "formulation result cites another compound occurrence",
        ));
    }
    Ok(())
}

/// Actual inputs retained by the cell-lab effect owner. Sample availability is
/// the common world's physical data, never an application permission counter.
pub fn assay_inputs(step: u8, material: &Value) -> ShellResult<Value> {
    let formulation = reply(4, &material["formulation"])?;
    let cells = reply(1, &material["cells"])?;
    if material["formulation"]["success"] != true || material["cells"]["success"] != true {
        return Err(ShellError::message(
            "assay requires actual formulation and cells",
        ));
    }
    let mut inputs = serde_json::json!({"formulation":formulation,"cells":cells});
    match step {
        5 => {}
        8 => {
            let baseline = reply(2, &material["baseline"])?;
            let purity = reply(7, &material["purity"])?;
            if material["baseline"]["success"] != true
                || material["purity"]["success"] != true
                || material["purity"]["data"]["inputs"]["formulation"] != formulation
            {
                return Err(ShellError::message(
                    "toxicity requires the actual baseline and purity",
                ));
            }
            inputs["baseline"] = baseline.into();
            inputs["purity"] = purity.into();
        }
        _ => return Err(ShellError::message("unknown biological assay")),
    }
    Ok(inputs)
}

pub fn assay_receipt(step: u8, inputs: &Value, actual: &Value) -> ShellResult<()> {
    if !matches!(step, 5 | 8) {
        return Err(ShellError::message("unknown biological assay"));
    }
    reply(step, actual)?;
    same(&actual["data"]["inputs"], inputs)?;
    let sample = &actual["data"]["sample_unit"];
    if (actual["data"]["status"] == "exhausted" && !sample.is_null())
        || (actual["data"]["status"] != "exhausted" && sample.as_u64().is_none())
    {
        return Err(ShellError::message(
            "assay result changes its actual sample consumption",
        ));
    }
    Ok(())
}

/// Match the actual material and shared cells before the clinical effect.
/// This is a relation between received values, not a second protocol cursor.
pub fn clinical_inputs(candidate: &Value, baseline: &Value) -> ShellResult<()> {
    let formulation = reply(4, &candidate["formulation"])?;
    let baseline_id = reply(2, baseline)?;
    let purity = reply(7, &candidate["purity"])?;
    let cells = candidate["bioassay"]["data"]["inputs"]["cells"]
        .as_u64()
        .ok_or_else(|| ShellError::message("bioassay has no actual prepared cells"))?;
    if candidate["formulation"]["success"] != true || baseline["success"] != true {
        return Err(ShellError::message(
            "clinical inputs require actual formulation and baseline",
        ));
    }
    for (name, step) in [
        ("bioassay", 5),
        ("purity", 7),
        ("stability", 6),
        ("toxicity", 8),
    ] {
        let result = &candidate[name];
        reply(step, result)?;
        if result["data"]["inputs"]["formulation"] != formulation
            || (step != 7 && result["data"]["inputs"]["cells"] != cells)
            || (matches!(step, 6 | 7) && result["success"] != true)
        {
            return Err(ShellError::message(
                "clinical evidence uses different materials",
            ));
        }
    }
    if candidate["toxicity"]["data"]["inputs"]["purity"] != purity
        || candidate["toxicity"]["data"]["inputs"]["baseline"] != baseline_id
    {
        return Err(ShellError::message(
            "toxicity evidence uses different purity or baseline",
        ));
    }
    Ok(())
}

pub fn clinical_receipt(candidate: &Value, actual: &Value) -> ShellResult<()> {
    reply(9, actual)?;
    let mut supported = true;
    for (name, step) in [
        ("formulation", 4),
        ("bioassay", 5),
        ("stability", 6),
        ("purity", 7),
        ("toxicity", 8),
    ] {
        let result = &candidate[name];
        let id = reply(step, result)?;
        if actual["data"]["inputs"][name] != id
            || (step != 5 && step != 8 && result["success"] != true)
        {
            return Err(ShellError::message(
                "clinical assessment cites another scientific outcome",
            ));
        }
        supported &= result["success"] == true;
    }
    let expected = if supported {
        "supported"
    } else {
        "insufficient_data"
    };
    if actual["success"] != true || actual["data"]["assessment"] != expected {
        return Err(ShellError::message(
            "clinical assessment changes the actual assay verdicts",
        ));
    }
    Ok(())
}

pub fn decision_receipt(candidate: &Value, actual: &Value) -> ShellResult<()> {
    let step = match candidate["outcome"].as_str() {
        Some("continue") => 12,
        Some("retry") => 13,
        Some("abort") => 14,
        _ => return Err(ShellError::message("unknown actual director outcome")),
    };
    reply(step, actual)?;
    let review_step = match candidate["regulatory"]["data"]["review_type"].as_str() {
        Some("initial_review") => 10,
        Some("hold_review") => 11,
        _ => return Err(ShellError::message("missing actual regulatory review")),
    };
    for (name, step) in [
        ("compound", 3),
        ("formulation", 4),
        ("clinical", 9),
        ("regulatory", review_step),
    ] {
        let result = &candidate[name];
        if actual["data"]["inputs"][name] != reply(step, result)?
            || (name != "regulatory" && result["success"] != true)
        {
            return Err(ShellError::message("director outcome cites another study"));
        }
    }
    for (result, input, evidence) in [
        ("formulation", "compound", "compound"),
        ("clinical", "formulation", "formulation"),
        ("regulatory", "clinical", "clinical"),
    ] {
        if candidate[result]["data"]["inputs"][input] != candidate[evidence]["data"]["event_id"] {
            return Err(ShellError::message(
                "director evidence belongs to different scientific inputs",
            ));
        }
    }
    if actual["success"] != true || (step == 12 && candidate["regulatory"]["success"] != true) {
        return Err(ShellError::message(
            "continuation requires actual positive approval",
        ));
    }
    Ok(())
}

pub fn regulatory_receipt(input: &Value, clinical: &Value, actual: &Value) -> ShellResult<()> {
    let step = match input["review_type"].as_str() {
        Some("initial_review") => 10,
        Some("hold_review") => 11,
        _ => return Err(ShellError::message("unknown admitted regulatory review")),
    };
    arguments(step, input)?;
    let id = reply(9, clinical)?;
    reply(step, actual)?;
    if clinical["success"] != true
        || !matches!(
            clinical["data"]["assessment"].as_str(),
            Some("supported" | "insufficient_data")
        )
        || actual["data"]["inputs"]["clinical"] != id
        || (actual["success"] == true && clinical["data"]["assessment"] != "supported")
    {
        return Err(ShellError::message(
            "regulatory verdict differs from the actual current clinical assessment",
        ));
    }
    Ok(())
}

pub fn documentation_receipt(candidate: &Value, actual: &Value) -> ShellResult<()> {
    if !matches!(candidate["outcome"].as_str(), Some("continue" | "abort")) {
        return Err(ShellError::message(
            "retry is not a terminal project outcome",
        ));
    }
    decision_receipt(candidate, &candidate["decision"])?;
    reply(15, actual)?;
    if actual["success"] != true {
        return Err(ShellError::message("final documentation has not succeeded"));
    }
    for name in [
        "compound",
        "formulation",
        "clinical",
        "regulatory",
        "decision",
    ] {
        if actual["data"]["inputs"][name] != candidate[name]["data"]["event_id"] {
            return Err(ShellError::message(
                "documentation archives another project outcome",
            ));
        }
    }
    Ok(())
}

fn copy_bytes(bytes: &[u8], destination: &mut [u8]) -> Result<usize, CodecError> {
    destination
        .get_mut(..bytes.len())
        .ok_or(CodecError::Truncated)?
        .copy_from_slice(bytes);
    Ok(bytes.len())
}

/// Only actual tool arguments; STEP names the scientific operation, not progress.
pub struct AnalysisArguments<'a, const STEP: u8>(&'a [u8]);
impl<'a, const STEP: u8> AnalysisArguments<'a, STEP> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        arguments(
            STEP,
            &serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))?,
        )?;
        Ok(Self(bytes))
    }
}
impl<const STEP: u8> WireEncode for AnalysisArguments<'_, STEP> {
    fn encode_into(&self, destination: &mut [u8]) -> Result<usize, CodecError> {
        copy_bytes(self.0, destination)
    }
}
impl<const STEP: u8> WirePayload for AnalysisArguments<'_, STEP> {
    const SCHEMA_ID: u32 = 0xad14_4000 | STEP as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        AnalysisArguments::<STEP>::from_bytes(input.as_bytes())
            .map(|_| ())
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}

pub struct AnalysisReply<'a, const STEP: u8, const SUCCESS: bool>(&'a [u8]);
impl<'a, const STEP: u8, const SUCCESS: bool> AnalysisReply<'a, STEP, SUCCESS> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))?;
        reply(STEP, &value)?;
        if value["success"] != SUCCESS {
            return Err(ShellError::message(
                "scientific reply differs from its branch",
            ));
        }
        Ok(Self(bytes))
    }
}
impl<const STEP: u8, const SUCCESS: bool> WireEncode for AnalysisReply<'_, STEP, SUCCESS> {
    fn encode_into(&self, destination: &mut [u8]) -> Result<usize, CodecError> {
        copy_bytes(self.0, destination)
    }
}
impl<const STEP: u8, const SUCCESS: bool> WirePayload for AnalysisReply<'_, STEP, SUCCESS> {
    const SCHEMA_ID: u32 = 0xad14_5000 | ((STEP as u32) << 1) | SUCCESS as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        AnalysisReply::<STEP, SUCCESS>::from_bytes(input.as_bytes())
            .map(|_| ())
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}

/// A borrowed result object addressed to one participant. The target schema
/// distinguishes concurrent sends; this contains no endpoint or authority state.
pub struct JsonTo<'a, const DEST: u8>(&'a [u8]);
impl<'a, const DEST: u8> JsonTo<'a, DEST> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))?;
        if !value.is_object() {
            return Err(ShellError::message("scientific evidence must be an object"));
        }
        Ok(Self(bytes))
    }
}
impl<const DEST: u8> WireEncode for JsonTo<'_, DEST> {
    fn encode_into(&self, destination: &mut [u8]) -> Result<usize, CodecError> {
        copy_bytes(self.0, destination)
    }
}
impl<const DEST: u8> WirePayload for JsonTo<'_, DEST> {
    const SCHEMA_ID: u32 = 0xad14_6000 | DEST as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        JsonTo::<DEST>::from_bytes(input.as_bytes())
            .map(|_| ())
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}

/// Only a real, recorded terminal director outcome can be announced as final.
pub struct FinalOutcome<'a, const DEST: u8>(&'a [u8]);
impl<'a, const DEST: u8> FinalOutcome<'a, DEST> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))?;
        decision_receipt(&value, &value["decision"])?;
        if !matches!(value["outcome"].as_str(), Some("continue" | "abort")) {
            return Err(ShellError::message("recorded outcome is not terminal"));
        }
        Ok(Self(bytes))
    }
}
impl<const DEST: u8> WireEncode for FinalOutcome<'_, DEST> {
    fn encode_into(&self, destination: &mut [u8]) -> Result<usize, CodecError> {
        copy_bytes(self.0, destination)
    }
}
impl<const DEST: u8> WirePayload for FinalOutcome<'_, DEST> {
    const SCHEMA_ID: u32 = 0xad14_7000 | DEST as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        FinalOutcome::<DEST>::from_bytes(input.as_bytes())
            .map(|_| ())
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}

#[cfg(test)]
#[path = "imp/tests.rs"]
mod tests;
