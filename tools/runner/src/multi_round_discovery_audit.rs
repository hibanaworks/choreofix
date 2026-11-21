//! Bind scientific task outcomes to the compiled protocol's committed receives.
use crate::{evidence::Result, journal};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

type Operation = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
    &'static str,
);

fn operation(agent: &str, tool: &str, arguments: &Value) -> Option<Operation> {
    Some(match (agent, tool) {
        ("CHEMIST", "run_hplc_analysis") if arguments["sample"] == "synthesis_qc" => {
            ("CHEM_FILE", "HPLC_SERVICE", &["HPLC"], "SynthesisQc", "QC")
        }
        ("BIOLOGIST", "run_cell_assay") if arguments["assay_type"] == "culture_prep" => (
            "BIO_FILE",
            "CELL_SERVICE",
            &["CELL_LAB"],
            "CulturePrep",
            "CULTURE",
        ),
        ("TOXICOLOGIST", "run_mass_spec_analysis") if arguments["sample"] == "baseline_markers" => {
            (
                "TOX_FILE",
                "MASS_SERVICE",
                &["MASS_SPEC"],
                "Baseline",
                "MASS_SERVICE",
            )
        }
        ("CHEMIST", "run_mass_spec_analysis") if arguments["sample"] == "molecular_id" => (
            "CHEM_FILE",
            "MASS_SERVICE",
            &["MASS_SPEC"],
            "MolecularId",
            "MASS_SERVICE",
        ),
        ("FORMULATION_SCIENTIST", "prepare_formulation") => (
            "FORM_FILE",
            "FORM_SERVICE",
            &["FORMULATION_SUITE"],
            "Formulation",
            "FORMULATION",
        ),
        ("BIOLOGIST", "run_cell_assay") if arguments["assay_type"] == "bioassay" => (
            "BIO_FILE",
            "CELL_SERVICE",
            &["CELL_LAB", "BIOLOGICAL_SAMPLES"],
            "Bioassay",
            "BIOASSAY",
        ),
        ("BIOLOGIST", "run_hplc_analysis") if arguments["sample"] == "stability" => (
            "BIO_FILE",
            "HPLC_SERVICE",
            &["HPLC"],
            "Stability",
            "STABILITY",
        ),
        ("TOXICOLOGIST", "run_hplc_analysis") if arguments["sample"] == "purity_check" => {
            ("TOX_FILE", "HPLC_SERVICE", &["HPLC"], "Purity", "PURITY")
        }
        ("TOXICOLOGIST", "run_cell_assay") if arguments["assay_type"] == "cytotoxicity" => (
            "TOX_FILE",
            "CELL_SERVICE",
            &["CELL_LAB", "BIOLOGICAL_SAMPLES"],
            "Cytotoxicity",
            "TOXICITY",
        ),
        ("CLINICAL_LEAD", "review_clinical_data")
            if arguments["data_type"] == "combined_review" =>
        {
            (
                "CLIN_FILE",
                "MASS_SERVICE",
                &["MASS_SPEC"],
                "Clinical",
                "STUDY",
            )
        }
        ("REGULATORY_SPECIALIST", "verify_regulatory_compliance")
            if arguments["review_type"] == "initial_review" =>
        {
            ("REG_FILE", "STUDY_SERVICE", &[], "RoutineReview", "STUDY")
        }
        ("REGULATORY_SPECIALIST", "verify_regulatory_compliance")
            if arguments["review_type"] == "hold_review" =>
        {
            (
                "REG_FILE",
                "STUDY_SERVICE",
                &["HPLC", "MASS_SPEC"],
                "DeepReview",
                "STUDY",
            )
        }
        ("PROJECT_DIRECTOR", "submit_project_decision") => {
            ("DIR_FILE", "STUDY_SERVICE", &[], "Decision", "STUDY")
        }
        ("CHEMIST", "run_hplc_analysis") if arguments["sample"] == "documentation" => (
            "CHEM_FILE",
            "HPLC_SERVICE",
            &["HPLC"],
            "Documentation",
            "STUDY",
        ),
        _ => return None,
    })
}

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty 14H history")?["message"];
    let history = crate::journal::read(rows, artifact, dynamic)?;
    let actors = history.participants;

    if compiled["task"] != "14H" || fixture["type"] != "fixtures" || fixture["task"] != "14H" {
        return Err("foreign task fixture or projection".into());
    }
    let role = |name: &str| {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("missing compiled role {name}"))
    };
    let label = |name: &str| {
        compiled["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("missing compiled label {name}"))
    };
    let scored = history.scored_events;
    if !history.simulator_complete {
        return Err("incomplete 14H score or resource violation".into());
    }
    journal::check_close_pairs(
        &history.frames,
        &[
            (role("CHEM_VM")?, role("CHEMIST_IO")?),
            (role("BIO_VM")?, role("BIOLOGIST_IO")?),
            (role("TOX_VM")?, role("TOXICOLOGIST_IO")?),
            (role("FORM_VM")?, role("FORMULATOR_IO")?),
            (role("CLIN_VM")?, role("CLINICIAN_IO")?),
            (role("REG_VM")?, role("REGULATOR_IO")?),
            (role("DIR_VM")?, role("DIRECTOR_IO")?),
        ],
    )?;
    let mut used_frames = BTreeSet::new();
    let mut effect_edges = BTreeSet::new();
    let mut take = |from: &str,
                    to: &str,
                    operation_label: u64,
                    within: Range<usize>|
     -> Result<&journal::Frame> {
        let from_id = role(from)?;
        let to_id = role(to)?;
        effect_edges.insert((from_id, to_id, operation_label));
        let (id, frame) = history
            .frames
            .iter()
            .filter(|(id, frame)| {
                !used_frames.contains(*id)
                    && frame.direction(from_id, to_id)
                    && u64::from(frame.logical_label) == operation_label
                    && within.contains(&frame.received)
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| {
                format!(
                    "no unused 14H receive for {from} -> {to} label {operation_label} in {within:?}"
                )
            })?;
        used_frames.insert(*id);
        Ok(frame)
    };
    let mut sample_units = BTreeSet::new();
    let mut effect_ids = BTreeSet::new();
    let mut current_reviews = BTreeMap::<u64, (usize, &Value)>::new();
    let mut terminal = None;
    let mut documentation = None;
    let mut effects = Vec::new();
    let mut intervals = Vec::new();
    let mut calls = history.calls.values().collect::<Vec<_>>();
    calls.sort_by_key(|call| call.emitted);
    for call in calls {
        let agent = call.request["agent"].as_str().ok_or("scientist")?;
        let tool = call.tool;
        let arguments = &call.request["arguments"];
        let (file, owner, resources, name, result_view) = operation(agent, tool, arguments)
            .ok_or_else(|| format!("unknown course of scientific effect {agent} {tool}"))?;
        let code = label(name)?;
        let (start, started) = call.started;
        let (_, ended) = call.ended;
        let (end, output) = call.result;
        let before = call.emitted.min(start);
        let request = take(file, owner, code, 0..before)?;
        let received = request.received;
        let shown = take(agent, file, code, end + 1..rows.len())?;
        if request.value()? != *arguments || shown.value()? != *output {
            return Err(format!("{name} differs from its actual file request or reply").into());
        }
        for &resource in resources {
            let operation = if resource == "BIOLOGICAL_SAMPLES" {
                label("SampleUse")?
            } else {
                code
            };
            let used = take(resource, owner, operation, received + 1..before)?;
            if !used.bytes.is_empty() {
                return Err(
                    format!("{name} resource invocation carried additional arguments").into(),
                );
            }
        }
        intervals.push((resources, started, ended));
        let success = output["success"].as_bool().ok_or("scientific verdict")?;
        let id = output["data"]["event_id"]
            .as_u64()
            .ok_or("scientific occurrence ID")?;
        let score_event = scored
            .get(usize::try_from(id)?)
            .ok_or("14H scorer omits effect")?;
        if !effect_ids.insert(id) || !call.matches_score(score_event, arguments) {
            return Err("14H scorer differs from the actual effect".into());
        }
        if resources.contains(&"BIOLOGICAL_SAMPLES") {
            let unit = &output["data"]["sample_unit"];
            if output["data"]["status"] == "exhausted" {
                if !unit.is_null() || success || sample_units.len() != 3 {
                    return Err("exhaustion differs from the real stock".into());
                }
            } else if !sample_units
                .insert(unit.as_u64().ok_or("sample-consuming assay has no unit")?)
                || sample_units.len() > 3
            {
                return Err("sample unit reused or stock replenished".into());
            }
        }
        let (result_target, result_label) = if result_view == "MASS_SERVICE" {
            ("MASS_SPEC", label("Passed")?)
        } else if name == "Decision" {
            (agent, 70)
        } else {
            (agent, label(if success { "Passed" } else { "Failed" })?)
        };
        let result = take(
            result_view,
            result_target,
            result_label,
            end + 1..shown.received,
        )?;
        if name == "Decision" {
            let envelope = result.value()?;
            let review_id = output["data"]["inputs"]["regulatory"]
                .as_u64()
                .ok_or("decision without a review occurrence")?;
            let (at, review) = current_reviews
                .get(&review_id)
                .ok_or("decision lacks actual review")?;
            let notice = take("STUDY", agent, 69, at + 1..received)?;
            let published = take(
                "REG_FILE",
                "REGULATORY_SPECIALIST",
                label("Publish")?,
                at + 1..notice.received,
            )?;
            if published.value()? != json!({"evidence":"regulatory"})
                || notice.value()? != **review
                || envelope["regulatory"] != **review
                || envelope["decision"] != *output
                || (arguments["outcome"] == "continue" && review["success"] != true)
            {
                return Err("director decision is detached from the submitted review".into());
            }
            if matches!(arguments["outcome"].as_str(), Some("continue" | "abort"))
                && terminal.replace(envelope).is_some()
            {
                return Err("multiple terminal decisions".into());
            }
        } else {
            if result.value()? != *output {
                return Err(format!("{name} view reply differs from its actual effect").into());
            }
            if agent == "REGULATORY_SPECIALIST" {
                current_reviews.insert(id, (end, output));
            }
            if name == "Documentation" && success {
                let terminal = terminal
                    .as_ref()
                    .ok_or("documentation without a terminal decision")?;
                for key in [
                    "compound",
                    "formulation",
                    "clinical",
                    "regulatory",
                    "decision",
                ] {
                    if output["data"]["inputs"][key] != terminal[key]["data"]["event_id"] {
                        return Err("documentation cites a different project lineage".into());
                    }
                }
                if documentation.replace(id).is_some() {
                    return Err("multiple successful documentations".into());
                }
            }
        }
        effects.push(
            json!({"event_id":id,"operation":name,"request_commit":received,
            "effect_start":start,"effect_result":end,"file_reply_commit":shown.received,
            "resources":resources,"success":success}),
        );
    }
    if history.frames.iter().any(|(id, frame)| {
        let key = (
            frame.identity["source"].as_u64().unwrap_or(u64::MAX),
            frame.identity["target"].as_u64().unwrap_or(u64::MAX),
            u64::from(frame.logical_label),
        );
        effect_edges.contains(&key) && !used_frames.contains(id)
    }) {
        return Err("unused 14H effect frame".into());
    }
    let mut overlap_count = 0;
    for (index, &(resources, start, end)) in intervals.iter().enumerate() {
        for &(other_resources, other_start, other_end) in &intervals[index + 1..] {
            if start < other_end && other_start < end {
                if let Some(resource) = resources
                    .iter()
                    .find(|resource| other_resources.contains(resource))
                {
                    return Err(format!("overlapping uses of {resource}").into());
                }
                overlap_count += 1;
            }
        }
    }
    let outcome = terminal.ok_or("missing actual terminal director decision")?;
    let documentation = documentation.ok_or("missing actual final documentation")?;
    Ok(json!({
        "status":"passed","task":"14H","effects":effects,"domain_calls":history.calls.len(),
        "guest_exits":actors.len(),"independent_effect_overlaps":overlap_count,
        "samples_consumed":sample_units.len(),"terminal_outcome":outcome["outcome"],
        "documentation":documentation,"simulator_complete":true,"violations":0,
        "effect_receives_bound_to_checked_occurrences":true,"dynamic_protocol":dynamic,
        "scored_benchmark":false,
    }))
}
