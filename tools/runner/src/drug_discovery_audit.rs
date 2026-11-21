//! Match committed discovery requests and replies to the actual common-world effects.
use crate::{
    evidence::Result,
    journal::{self, one},
    projection,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Bind observed scientific tool arguments to the file and physical instrument
/// contracts. The regulatory task adds only its own approval operation.
pub(super) fn operation(
    agent: &str,
    tool: &str,
    arguments: &Value,
) -> Option<(
    &'static str,
    &'static str,
    Option<&'static str>,
    &'static str,
)> {
    let observed = match (
        agent,
        tool,
        arguments["sample"]
            .as_str()
            .or_else(|| arguments["assay_type"].as_str()),
    ) {
        ("CHEMIST", "run_hplc_analysis", Some("synthesis_qc")) => {
            ("CHEM_FILE", "HPLC_OWNER", Some("HPLC"), "SynthesisQc")
        }
        ("CHEMIST", "run_mass_spec_analysis", Some("molecular_id")) => {
            ("CHEM_FILE", "MASS_OWNER", Some("MASS_SPEC"), "MolecularId")
        }
        ("CHEMIST", "run_hplc_analysis", Some("documentation")) => {
            ("CHEM_FILE", "HPLC_OWNER", Some("HPLC"), "Documentation")
        }
        ("BIOLOGIST", "run_cell_assay", Some("culture_prep")) => {
            ("BIO_FILE", "CELL_OWNER", Some("CELL_LAB"), "CulturePrep")
        }
        ("BIOLOGIST", "run_cell_assay", Some("bioassay")) => {
            ("BIO_FILE", "CELL_OWNER", Some("CELL_LAB"), "Bioassay")
        }
        ("BIOLOGIST", "run_hplc_analysis", Some("stability")) => {
            ("BIO_FILE", "HPLC_OWNER", Some("HPLC"), "Stability")
        }
        ("TOXICOLOGIST", "run_mass_spec_analysis", Some("baseline_markers")) => {
            ("TOX_FILE", "MASS_OWNER", Some("MASS_SPEC"), "Baseline")
        }
        ("TOXICOLOGIST", "run_hplc_analysis", Some("purity_check")) => {
            ("TOX_FILE", "HPLC_OWNER", Some("HPLC"), "Purity")
        }
        ("TOXICOLOGIST", "run_cell_assay", Some("cytotoxicity")) => {
            ("TOX_FILE", "CELL_OWNER", Some("CELL_LAB"), "Cytotoxicity")
        }
        ("LEAD_SCIENTIST", "run_mass_spec_analysis", Some("reference_methods")) => {
            ("LEAD_FILE", "MASS_OWNER", Some("MASS_SPEC"), "Reference")
        }
        ("LEAD_SCIENTIST", "run_hplc_analysis", Some("structural_confirmation")) => {
            ("LEAD_FILE", "HPLC_OWNER", Some("HPLC"), "Structure")
        }
        ("LEAD_SCIENTIST", "run_mass_spec_analysis", Some("characterization")) => (
            "LEAD_FILE",
            "MASS_OWNER",
            Some("MASS_SPEC"),
            "Characterization",
        ),
        ("LEAD_SCIENTIST", "submit_decision", None) if arguments["go"].is_boolean() => {
            ("LEAD_FILE", "LEAD_SCIENTIST", None, "Decision")
        }
        _ => return None,
    };
    Some(observed)
}

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty discovery history")?["message"];
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("discovery participants")?;
    if compiled["task"] != "14E"
        || fixture["type"] != "fixtures"
        || fixture["task"] != "14E"
        || actors.len() != 4
        || actors
            .iter()
            .filter_map(|actor| actor["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from(["CHEMIST", "BIOLOGIST", "TOXICOLOGIST", "LEAD_SCIENTIST"])
    {
        return Err("foreign discovery task or participants".into());
    }
    let role = |name: &str| {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("missing compiled role {name}"))
    };
    let label = |name: &str| {
        compiled["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("missing compiled message {name}"))
    };
    let headers = projection::frames(artifact, rows)?;
    let history = journal::read(rows, &headers, actors, dynamic)?;
    let score = &history.score;
    let evaluated = score["events"].as_array().ok_or("discovery score events")?;
    if history.calls.len() != 13
        || evaluated.len() != history.calls.len()
        || score["complete"] != true
        || score["violations"] != json!([])
        || history
            .frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("discovery history has missing effects or resource violations".into());
    }
    journal::check_close_pairs(
        &history.frames,
        &[
            (role("CHEM_VM")?, role("CHEM_IO")?),
            (role("BIO_VM")?, role("BIO_IO")?),
            (role("TOX_VM")?, role("TOX_IO")?),
            (role("LEAD_VM")?, role("LEAD_IO")?),
        ],
    )?;
    let receive = |from: &str, to: &str, message: &str| -> Result<&journal::Frame> {
        let (from, to, label) = (role(from)?, role(to)?, label(message)?);
        one(history
            .frames
            .values()
            .filter(|frame| frame.direction(from, to) && u64::from(frame.logical_label) == label))
    };

    let mut results = BTreeMap::new();
    let mut assigned = BTreeSet::new();
    let mut decision = None;
    let mut intervals = Vec::new();
    for call in history.calls.values() {
        let agent = call.request["agent"].as_str().ok_or("discovery agent")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let (end, output) = call.result()?;
        let (start, started) = call.started()?;
        let (_, ended) = call.ended()?;
        let (file, owner, instrument, operation) = operation(agent, tool, arguments)
            .ok_or_else(|| format!("unexpected discovery effect {agent} {tool} {arguments}"))?;
        let request = receive(file, owner, operation)?;
        let displayed = receive(agent, file, operation)?;
        if results.insert(operation, (end, output)).is_some()
            || request.received >= call.emitted.min(start)
            || request.value()? != *arguments
            || displayed.received <= end
            || displayed.value()? != *output
            || output["success"] != true
        {
            return Err(
                format!("{operation} differs from the admitted request or actual reply").into(),
            );
        }
        if let Some(instrument) = instrument {
            let used = receive(instrument, owner, operation)?;
            let reply = receive(owner, instrument, operation)?;
            if used.received >= call.emitted.min(start)
                || used.received <= request.received
                || !used.bytes.is_empty()
                || reply.received <= end
                || reply.value()? != *output
            {
                return Err(format!(
                    "{operation} has an invalid committed instrument use or reply"
                )
                .into());
            }
            intervals.push((instrument, started, ended));
        } else {
            for (from, to) in [("LEAD_SCIENTIST", "CHEMIST"), ("CHEMIST", "CHEM_FILE")] {
                let delivered = receive(from, to, "DecisionReceipt")?;
                if delivered.received <= end || delivered.value()? != *output {
                    return Err("chemist did not receive the actual decision result".into());
                }
            }
            decision = Some(output["data"]["go"].clone());
        }
        let (receipt_index, _) = one(evaluated.iter().enumerate().filter(|(index, event)| {
            !assigned.contains(index)
                && call.matches_score(event, arguments)
                && event["success"] == true
        }))?;
        assigned.insert(receipt_index);
    }
    let actual = |name: &str| {
        results
            .get(name)
            .copied()
            .ok_or("missing discovery receipt")
    };
    let compound = json!({"qc":actual("SynthesisQc")?.1,"identity":actual("MolecularId")?.1});
    for (agent, file, message) in [
        ("BIOLOGIST", "BIO_FILE", "BiologicalSample"),
        ("TOXICOLOGIST", "TOX_FILE", "ToxicologySample"),
    ] {
        let published = receive("CHEMIST", agent, message)?;
        let shown = receive(agent, file, message)?;
        if published.received <= actual("MolecularId")?.0
            || published.value()? != compound
            || shown.value()? != compound
        {
            return Err("scientist did not receive the actual compound".into());
        }
    }
    for (agent, message, expected, completion) in [
        (
            "BIOLOGIST",
            "BioReport",
            json!({
                "compound":compound,"cells":actual("CulturePrep")?.1,"assay":actual("Bioassay")?.1,
                "stability":actual("Stability")?.1,
            }),
            actual("Stability")?.0,
        ),
        (
            "TOXICOLOGIST",
            "ToxReport",
            json!({
                "compound":compound,"baseline":actual("Baseline")?.1,"purity":actual("Purity")?.1,
                "toxicity":actual("Cytotoxicity")?.1,
            }),
            actual("Cytotoxicity")?.0,
        ),
    ] {
        let report = receive(agent, "LEAD_SCIENTIST", message)?;
        let shown = receive("LEAD_SCIENTIST", "LEAD_FILE", message)?;
        if report.received <= completion
            || report.value()? != expected
            || shown.value()? != expected
        {
            return Err("lead scientist did not receive the report from the actual effects".into());
        }
    }
    let mut independent_overlap = 0;
    for (i, &(instrument, start, end)) in intervals.iter().enumerate() {
        for &(other, other_start, other_end) in &intervals[i + 1..] {
            if start < other_end && other_start < end {
                if instrument == other {
                    return Err(format!("overlapping effects on {instrument}").into());
                }
                independent_overlap += 1;
            }
        }
    }
    Ok(json!({
        "status":"passed", "task":"14E", "domain_calls":history.calls.len(),
        "violations":0, "simulator_complete":true, "guest_exits":actors.len(),
        "effect_receives_bound_to_checked_occurrences":true,
        "independent_effect_overlaps":independent_overlap,
        "decision":decision.ok_or("missing actual go/no-go receipt")?,
        "dynamic_protocol":dynamic,
        "scored_benchmark":false,
    }))
}
