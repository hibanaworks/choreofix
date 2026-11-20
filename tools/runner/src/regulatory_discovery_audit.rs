//! Bind actual 14M requests, effects and replies to EPF-checked receives.
use crate::{
    drug_discovery_audit::operation,
    evidence::{Result, digest},
    journal, projection,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    path::Path,
};

pub fn audit(path: &Path) -> Result<Value> {
    let rows = journal::records(path)?;
    let artifact = projection::load(path)?;
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty regulatory history")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("14M participants")?;
    let expected = BTreeSet::from([
        "CHEMIST",
        "BIOLOGIST",
        "TOXICOLOGIST",
        "LEAD_SCIENTIST",
        "REGULATORY_SPECIALIST",
    ]);
    if compiled["task"] != "14M"
        || fixture["type"] != "fixtures"
        || fixture["task"] != "14M"
        || actors.len() != expected.len()
        || actors
            .iter()
            .filter_map(|actor| actor["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Err("wrong regulatory task or participant set".into());
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
    let headers = projection::frames(&artifact, &rows)?;
    let dynamic = projection::check_dynamic(&artifact, &rows)?;
    let mut history = journal::read(&rows, &headers, actors)?;
    history.bind_receives(&dynamic)?;
    let evaluated = history.score["events"]
        .as_array()
        .ok_or("14M score events")?;
    if history.score["complete"] != true
        || history.score["violations"] != json!([])
        || evaluated.len() != history.calls.len()
        || history
            .frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete regulatory score or resource violation".into());
    }
    journal::check_close_pairs(
        &history.frames,
        &[
            (role("CHEM_VM")?, role("CHEM_IO")?),
            (role("BIO_VM")?, role("BIO_IO")?),
            (role("TOX_VM")?, role("TOX_IO")?),
            (role("LEAD_VM")?, role("LEAD_IO")?),
            (role("REG_VM")?, role("REG_IO")?),
        ],
    )?;

    let mut consumed = BTreeSet::new();
    let mut checked_edges = BTreeSet::new();
    let mut take = |(source, target): (&str, &str),
                    labels: &[u64],
                    within: Range<usize>|
     -> Result<&journal::Frame> {
        let from = role(source)?;
        let to = role(target)?;
        checked_edges.extend(labels.iter().map(|&label| (from, to, label)));
        let (id, frame) = history
            .frames
            .iter()
            .filter(|(id, frame)| {
                !consumed.contains(*id)
                    && frame.direction(from, to)
                    && labels.contains(&u64::from(frame.logical_label))
                    && frame.received.is_some_and(|index| within.contains(&index))
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| {
                format!("no unused committed {source} -> {target} {labels:?} in {within:?}")
            })?;
        consumed.insert(*id);
        Ok(frame)
    };
    let mut score_ids = BTreeSet::new();
    let mut decisions = BTreeMap::new();
    let mut approval = None;
    let mut review_denials = 0;
    let mut analysis_failures = 0;
    let mut intervals = Vec::new();
    let mut audited_effects = Vec::new();
    let mut calls = history.calls.values().collect::<Vec<_>>();
    calls.sort_by_key(|call| call.emitted);
    for call in calls {
        let agent = call.request["agent"]
            .as_str()
            .ok_or("missing effect agent")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let (end, output) = call.result()?;
        let (start, started) = call.started()?;
        let (_, ended) = call.ended()?;
        let success = output["success"].as_bool().ok_or("missing effect result")?;
        let (file, owner, instrument, name) = if agent == "REGULATORY_SPECIALIST"
            && tool == "verify_regulatory_compliance"
            && arguments == &json!({"decision":"go_no_go"})
        {
            ("REG_FILE", "REVIEW_OWNER", None, "Compliance")
        } else {
            operation(agent, tool, arguments)
                .ok_or_else(|| format!("unknown 14M effect {agent} {tool}"))?
        };
        let code = label(name)?;
        let before = call.emitted.min(start);
        let accepted = take((file, owner), &[code], 0..before)?;
        let request_at = accepted.received()?;
        let shown = take((agent, file), &[code], end + 1..rows.len())?;
        if accepted.value()? != *arguments || shown.value()? != *output {
            return Err(format!("{name} differs from the actual request or result").into());
        }
        if output["data"]["scientist"] != agent
            || (success && output["data"]["status"] != "complete")
            || (!success && output["data"]["status"] != "injected_failure")
        {
            return Err(format!("{name} has an inconsistent actual receipt").into());
        }
        let id = output["data"]["event_id"]
            .as_u64()
            .ok_or("missing simulator event ID")?;
        let event = evaluated
            .get(usize::try_from(id)?)
            .ok_or("score omits effect")?;
        if !score_ids.insert(id)
            || event["agent"] != agent
            || event["tool"] != tool
            || event["args"] != *arguments
            || event["result"] != output["data"]
            || event["success"] != success
            || event["violations"] != json!([])
        {
            return Err("scorer does not contain the actual effect exactly once".into());
        }
        let mut use_at = None;
        if let Some(instrument) = instrument {
            let used = take((instrument, owner), &[code], request_at + 1..before)?;
            if !used.bytes.is_empty() {
                return Err("instrument use has an unexpected payload".into());
            }
            use_at = Some(used.received()?);
            intervals.push((instrument, started, ended));
            let (source, result_labels) = match name {
                "SynthesisQc" => (
                    "SYNTHESIS_VIEW",
                    if success {
                        vec![label("Passed")?]
                    } else {
                        vec![label("Failed")?]
                    },
                ),
                "CulturePrep" => (
                    "CULTURE_VIEW",
                    if success {
                        vec![label("Passed")?]
                    } else {
                        vec![label("Failed")?]
                    },
                ),
                "Bioassay" | "Stability" | "Purity" | "Cytotoxicity" | "Structure" => {
                    let material = match name {
                        "Bioassay" => "ASSAY_VIEW",
                        "Stability" => "STABILITY_VIEW",
                        "Purity" => "PURITY_VIEW",
                        "Cytotoxicity" => "TOXICITY_VIEW",
                        _ => "STRUCTURE_VIEW",
                    };
                    (
                        material,
                        if success {
                            vec![label("Passed")?]
                        } else {
                            vec![label("Failed")?, label("ReanalysisFailed")?]
                        },
                    )
                }
                "Documentation" => {
                    let (approved_at, verdict): (usize, &Value) =
                        approval.ok_or("documentation without a positive review")?;
                    let approval_received = take(
                        ("STUDY", "HPLC_OWNER"),
                        &[label("ApprovalNotice")?],
                        approved_at + 1..before,
                    )?;
                    if approval_received.value()? != *verdict
                        || output["data"]["inputs"]["regulatory"] != verdict["data"]["artifact_id"]
                        || output["data"]["inputs"]["compound"]
                            != verdict["data"]["inputs"]["compound"]
                        || output["data"]["inputs"]["decision"]
                            != verdict["data"]["inputs"]["decision"]
                    {
                        return Err("documentation did not use the actual approved decision".into());
                    }
                    (
                        "STUDY",
                        vec![label(if success {
                            "Passed"
                        } else {
                            "DocumentationFailed"
                        })?],
                    )
                }
                _ if success => (owner, vec![code]),
                _ => return Err("unexpected mass spectrometer failure".into()),
            };
            let receiver = if source == owner { instrument } else { agent };
            let result = take(
                (source, receiver),
                &result_labels,
                end + 1..shown.received()?,
            )?;
            if result.value()? != *output {
                return Err(format!("{name} did not deliver the actual instrument result").into());
            }
            analysis_failures += usize::from(!success);
        } else if name == "Decision" {
            if !success
                || output["data"]["go"] != arguments["go"]
                || decisions.insert(id, (end, output)).is_some()
            {
                return Err("invalid lead decision".into());
            }
        } else {
            let decided = output["data"]["inputs"]["decision"]
                .as_u64()
                .ok_or("review omits decision ID")?;
            let (decided_at, decision) = decisions
                .remove(&decided)
                .ok_or("review reuses or omits a lead decision")?;
            let forwarded = take(
                ("STUDY", "REGULATORY_SPECIALIST"),
                &[label("DecisionReceipt")?],
                decided_at + 1..request_at,
            )?;
            let displayed = take(
                ("REGULATORY_SPECIALIST", "REG_FILE"),
                &[label("DecisionReceipt")?],
                forwarded.received()? + 1..request_at,
            )?;
            let submitted = take(
                ("STUDY", "REVIEW_OWNER"),
                &[label("ReviewInput")?],
                request_at + 1..before,
            )?;
            if forwarded.value()? != *decision
                || displayed.value()? != *decision
                || submitted.value()? != *decision
                || output["data"]["inputs"]["compound"] != decision["data"]["inputs"]["compound"]
            {
                return Err("review did not consume its actual current lead decision".into());
            }
            let result = take(
                ("STUDY", "REGULATORY_SPECIALIST"),
                &[label(if success {
                    "Passed"
                } else {
                    "ComplianceRejected"
                })?],
                end + 1..shown.received()?,
            )?;
            let lead = take(
                ("STUDY", "LEAD_SCIENTIST"),
                &[label(if success {
                    "ApprovedForLead"
                } else {
                    "ComplianceRejected"
                })?],
                end + 1..rows.len(),
            )?;
            let lead_file = take(
                ("LEAD_SCIENTIST", "LEAD_FILE"),
                &[label("ReviewResult")?],
                lead.received()? + 1..rows.len(),
            )?;
            if result.value()? != *output
                || lead.value()? != *output
                || lead_file.value()? != *output
            {
                return Err("regulatory result differs from the actual review".into());
            }
            if success {
                if approval.replace((end, output)).is_some() {
                    return Err("multiple positive reviews".into());
                }
                for (agent, file, message) in [
                    ("BIOLOGIST", "BIO_FILE", "ApprovedForBio"),
                    ("TOXICOLOGIST", "TOX_FILE", "ApprovedForTox"),
                ] {
                    let approved = take(("STUDY", agent), &[label(message)?], end + 1..rows.len())?;
                    let shown = take(
                        (agent, file),
                        &[label("StudyNotice")?],
                        approved.received()? + 1..rows.len(),
                    )?;
                    if approved.value()? != *output || shown.value()? != *output {
                        return Err("scientist received a different approval".into());
                    }
                }
                let chemist = take(
                    ("STUDY", "CHEMIST"),
                    &[label("ApprovedForChemist")?],
                    end + 1..rows.len(),
                )?;
                let chem_file = take(
                    ("CHEMIST", "CHEM_FILE"),
                    &[label("ApprovalNotice")?],
                    chemist.received()? + 1..rows.len(),
                )?;
                if chemist.value()? != *output || chem_file.value()? != *output {
                    return Err("chemist received a different approval".into());
                }
            } else {
                review_denials += 1;
            }
        }
        audited_effects.push(json!({
            "event_id":id, "operation":name, "request_commit":request_at, "instrument_commit":use_at,
            "effect_start":start,"effect_result":end,"file_reply_commit":shown.received()?, "success":success,
        }));
    }
    let plan_file = role("LEAD_FILE")?;
    let plan_recipient = role("LEAD_SCIENTIST")?;
    let plan_label = label("Reanalysis")?;
    let reconsidered = history
        .frames
        .values()
        .filter(|frame| {
            frame.direction(plan_file, plan_recipient)
                && u64::from(frame.logical_label) == plan_label
        })
        .count();
    if reconsidered != review_denials || approval.is_none() || !decisions.is_empty() {
        return Err("a rejected review lacks a fresh lead decision and reconsideration".into());
    }
    let mut choices = Vec::new();
    for _ in 0..reconsidered {
        let requested = take(
            ("LEAD_FILE", "LEAD_SCIENTIST"),
            &[label("Reanalysis")?],
            0..rows.len(),
        )?;
        let choice = requested.value()?;
        let biology = choice["biology"].as_bool().ok_or("biology choice")?;
        let toxicology = choice["toxicology"].as_bool().ok_or("toxicology choice")?;
        let mask = u64::from(biology) | (u64::from(toxicology) << 1);
        let after = requested.received()? + 1;
        let next_structure = audited_effects
            .iter()
            .filter(|effect| effect["operation"] == "Structure")
            .filter_map(|effect| {
                effect["request_commit"]
                    .as_u64()
                    .and_then(|row| usize::try_from(row).ok())
            })
            .filter(|&row| row >= after)
            .min()
            .ok_or("reconsideration has no new structural confirmation")?;
        for agent in ["BIOLOGIST", "TOXICOLOGIST", "HPLC_OWNER", "CELL_OWNER"] {
            let plan = take(("STUDY", agent), &[mask], after..next_structure)?;
            if plan.value()? != choice {
                return Err("reanalysis plan differs from the admitted lead choice".into());
            }
        }
        for (selected, sci, file) in [
            (biology, "BIOLOGIST", "BIO_FILE"),
            (toxicology, "TOXICOLOGIST", "TOX_FILE"),
        ] {
            if selected {
                let shown = take((sci, file), &[label("StudyNotice")?], after..next_structure)?;
                if shown.value()? != choice {
                    return Err("scientist received a different reanalysis request".into());
                }
            }
        }
        choices.push(choice);
    }
    if history.frames.iter().any(|(id, frame)| {
        let key = (
            frame.identity["source"].as_u64().unwrap_or(u64::MAX),
            frame.identity["target"].as_u64().unwrap_or(u64::MAX),
            u64::from(frame.logical_label),
        );
        checked_edges.contains(&key) && !consumed.contains(id)
    }) {
        return Err("unused committed effect request or response".into());
    }
    let mut overlaps = 0;
    for (i, &(instrument, start, end)) in intervals.iter().enumerate() {
        for &(other, other_start, other_end) in &intervals[i + 1..] {
            if start < other_end && other_start < end {
                if instrument == other {
                    return Err(format!("overlapping effects on {instrument}").into());
                }
                overlaps += 1;
            }
        }
    }
    Ok(json!({
        "status":"passed","task":"14M","domain_calls":history.calls.len(),"guest_exits":actors.len(),
        "simulator_complete":true,"violations":0,"rejections":review_denials,"analysis_failures":analysis_failures,
        "reanalysis_choices":choices,"independent_effect_overlaps":overlaps,"effects":audited_effects,
        "effect_receives_bound_to_checked_occurrences":true,"dynamic_protocol":dynamic,
        "source_events_sha256":digest(path)?,"runtime_binary_sha256":artifact["runtime_binary_sha256"],
        "compiled_projection_sha256":digest(&path.parent().ok_or("effect evidence directory")?.join("projection.json"))?,
        "scored_benchmark":false,
    }))
}
