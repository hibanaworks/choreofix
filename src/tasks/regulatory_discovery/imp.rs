//! Scientific arguments and effect receipts, with no Endpoint operations.
pub use crate::imp::json::{decode, encode, same};
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;
#[cfg(test)]
mod tests;

pub fn analysis_file<const STEP: u8>(
    file: &crate::shell::imp::file_wire::ClosedFile,
) -> ShellResult<Value> {
    let (_, tool, key, _) = analysis(STEP)?;
    let expected = format!("service/{tool}");
    let value = file.arguments(expected.as_bytes(), key)?;
    AnalysisArguments::<STEP>::from_bytes(&file.bytes)?;
    Ok(value)
}
pub fn input_file(
    file: &crate::shell::imp::file_wire::ClosedFile,
    path: &[u8],
) -> ShellResult<Value> {
    if file.completion.0.errno() != 0 || file.path != path {
        return Err(ShellError::message("unexpected scientific service file"));
    }
    decode(&file.bytes)
}
pub fn plan_file(file: &crate::shell::imp::file_wire::ClosedFile) -> ShellResult<Value> {
    let value = input_file(file, b"service/reconsider")?;
    if value.as_object().is_none_or(|o| o.len() != 2)
        || !value["biology"].is_boolean()
        || !value["toxicology"].is_boolean()
    {
        return Err(ShellError::message(
            "expected biology and toxicology reanalysis choices",
        ));
    }
    Ok(value)
}
pub fn choose_biology(
    value: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let input = value.take();
    let arm = match input["biology"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    value.set(input);
    arm
}
pub fn choose_toxicology(
    value: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let input = value.take();
    let arm = match input["toxicology"].as_bool() {
        Some(false) => Ok(DecisionArm::Left),
        Some(true) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    value.set(input);
    arm
}

pub fn decide(
    result: &std::cell::Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = result.take();
    let arm = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    result.set(value);
    arm
}

pub fn successful(value: &Value) -> ShellResult<()> {
    if value["success"] != true {
        return Err(ShellError::message(
            "expected an actual successful scientific result",
        ));
    }
    Ok(())
}

pub fn decision_receipt(candidate: &Value, actual: &Value) -> ShellResult<()> {
    successful(actual)?;
    same(&candidate["go"], &actual["data"]["go"])?;
    if actual["tool_name"] != "submit_decision" || !candidate["go"].is_boolean() {
        return Err(ShellError::message(
            "expected the lead scientist's actual decision",
        ));
    }
    for (name, result) in [
        ("compound", &candidate["compound"]["identity"]),
        ("bioassay", &candidate["bio"]["assay"]),
        ("stability", &candidate["bio"]["stability"]),
        ("toxicity", &candidate["tox"]["cytotoxicity"]),
        ("structural", &candidate["analyses"]["structural"]),
        (
            "characterization",
            &candidate["analyses"]["characterization"],
        ),
    ] {
        successful(result)?;
        let id = &result["data"]["artifact_id"];
        if !id.is_u64() {
            return Err(ShellError::message(
                "decision input has no scientific artifact receipt",
            ));
        }
        same(id, &actual["data"]["inputs"][name])?;
    }
    Ok(())
}

pub fn regulatory_receipt(admitted: &Value, actual: &Value) -> ShellResult<()> {
    if actual["tool_name"] != "verify_regulatory_compliance"
        || actual["data"]["scientist"] != "REGULATORY_SPECIALIST"
        || !actual["success"].is_boolean()
    {
        return Err(ShellError::message(
            "expected an actual regulatory review result",
        ));
    }
    for (name, id) in [
        ("decision", &admitted["receipt"]["data"]["artifact_id"]),
        (
            "compound",
            &admitted["decision"]["compound"]["identity"]["data"]["artifact_id"],
        ),
    ] {
        if !id.is_u64() {
            return Err(ShellError::message(
                "regulatory input has no current artifact receipt",
            ));
        }
        same(id, &actual["data"]["inputs"][name])?;
    }
    Ok(())
}
pub fn analysis(step: u8) -> ShellResult<(&'static str, &'static str, &'static str, &'static str)> {
    match step {
        15 => Ok(("CHEMIST", "run_hplc_analysis", "sample", "documentation")),
        0 => Ok(("CHEMIST", "run_hplc_analysis", "sample", "synthesis_qc")),
        1 => Ok(("BIOLOGIST", "run_cell_assay", "assay_type", "culture_prep")),
        2 => Ok((
            "TOXICOLOGIST",
            "run_mass_spec_analysis",
            "sample",
            "baseline_markers",
        )),
        3 => Ok((
            "LEAD_SCIENTIST",
            "run_mass_spec_analysis",
            "sample",
            "reference_methods",
        )),
        4 | 11 => Ok(("BIOLOGIST", "run_cell_assay", "assay_type", "bioassay")),
        5 | 12 => Ok(("BIOLOGIST", "run_hplc_analysis", "sample", "stability")),
        6 | 13 => Ok((
            "TOXICOLOGIST",
            "run_hplc_analysis",
            "sample",
            "purity_check",
        )),
        7 | 14 => Ok((
            "TOXICOLOGIST",
            "run_cell_assay",
            "assay_type",
            "cytotoxicity",
        )),
        8 => Ok((
            "CHEMIST",
            "run_mass_spec_analysis",
            "sample",
            "molecular_id",
        )),
        9 => Ok((
            "LEAD_SCIENTIST",
            "run_hplc_analysis",
            "sample",
            "structural_confirmation",
        )),
        10 => Ok((
            "LEAD_SCIENTIST",
            "run_mass_spec_analysis",
            "sample",
            "characterization",
        )),
        _ => Err(ShellError::message("unknown scientific analysis")),
    }
}

/// The borrowed JSON is the actual request, validated against its scientific
/// operation. STEP distinguishes occurrences without a runtime phase cursor.
pub struct AnalysisArguments<'a, const STEP: u8>(&'a [u8]);
impl<'a, const STEP: u8> AnalysisArguments<'a, STEP> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        let (_, _, key, target) = analysis(STEP)?;
        let value = decode(bytes)?;
        if value.as_object().is_none_or(|object| object.len() != 1) || value[key] != target {
            return Err(ShellError::message(
                "analysis arguments differ from the assigned specimen",
            ));
        }
        Ok(Self(bytes))
    }
}
impl<const STEP: u8> WireEncode for AnalysisArguments<'_, STEP> {
    fn encode_into(&self, dst: &mut [u8]) -> Result<usize, CodecError> {
        dst.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const STEP: u8> WirePayload for AnalysisArguments<'_, STEP> {
    const SCHEMA_ID: u32 = 0xad14_1000 | STEP as u32;
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
        let value = decode(bytes)?;
        let (actor, tool, key, target) = analysis(STEP)?;
        if value["success"] != SUCCESS
            || value["tool_name"] != tool
            || value["data"][key] != target
            || value["data"]["scientist"] != actor
        {
            return Err(ShellError::message(
                "analysis result differs from the actual operation or branch",
            ));
        }
        Ok(Self(bytes))
    }
}
impl<const STEP: u8, const SUCCESS: bool> WireEncode for AnalysisReply<'_, STEP, SUCCESS> {
    fn encode_into(&self, dst: &mut [u8]) -> Result<usize, CodecError> {
        dst.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const STEP: u8, const SUCCESS: bool> WirePayload for AnalysisReply<'_, STEP, SUCCESS> {
    const SCHEMA_ID: u32 = 0xad14_2000 | ((STEP as u32) << 1) | SUCCESS as u32;
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

pub struct ReanalysisPlan<'a, const MASK: u8, const DEST: u8>(&'a [u8]);
impl<'a, const MASK: u8, const DEST: u8> ReanalysisPlan<'a, MASK, DEST> {
    pub fn from_bytes(bytes: &'a [u8]) -> ShellResult<Self> {
        let value = decode(bytes)?;
        if MASK > 3
            || value.as_object().is_none_or(|object| object.len() != 2)
            || value["biology"] != (MASK & 1 != 0)
            || value["toxicology"] != (MASK & 2 != 0)
        {
            return Err(ShellError::message(
                "reanalysis plan differs from the selected analyses",
            ));
        }
        Ok(Self(bytes))
    }
}
impl<const MASK: u8, const DEST: u8> WireEncode for ReanalysisPlan<'_, MASK, DEST> {
    fn encode_into(&self, dst: &mut [u8]) -> Result<usize, CodecError> {
        dst.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl<const MASK: u8, const DEST: u8> WirePayload for ReanalysisPlan<'_, MASK, DEST> {
    const SCHEMA_ID: u32 = 0xad14_3000 | ((MASK as u32) << 8) | DEST as u32;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        ReanalysisPlan::<MASK, DEST>::from_bytes(input.as_bytes())
            .map(|_| ())
            .map_err(|_| CodecError::Malformed)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}
