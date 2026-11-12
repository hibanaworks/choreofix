//! Actual instrument effects and transported data; EPF owns execution legality.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn actual(calls: &[&Call], at: usize, kind: (&str, &str, &str, &str)) -> Result<Value> {
    let (agent, tool, key, value) = kind;
    let call = one(calls.iter().copied().filter(|c| {
        c.request["agent"] == agent
            && c.request["tool"] == tool
            && c.request["arguments"][key] == value
            && c.result.as_ref().is_some_and(|(end, _)| *end < at)
    }))?;
    Ok(call.result()?.1.clone())
}
fn compound(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(
        json!({"qc":actual(calls,at,("CHEMIST","run_hplc_analysis","sample","synthesis_qc"))?,
 "identity":actual(calls,at,("CHEMIST","run_mass_spec_analysis","sample","molecular_id"))?}),
    )
}
fn bio(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"compound":compound(calls,at)?,
 "prep":actual(calls,at,("BIOLOGIST","run_cell_assay","assay_type","culture_prep"))?,
 "assay":actual(calls,at,("BIOLOGIST","run_cell_assay","assay_type","bioassay"))?,
 "stability":actual(calls,at,("BIOLOGIST","run_hplc_analysis","sample","stability"))?}))
}
fn tox(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"compound":compound(calls,at)?,
 "baseline":actual(calls,at,("TOXICOLOGIST","run_mass_spec_analysis","sample","baseline_markers"))?,
 "purity":actual(calls,at,("TOXICOLOGIST","run_hplc_analysis","sample","purity_check"))?,
 "cytotoxicity":actual(calls,at,("TOXICOLOGIST","run_cell_assay","assay_type","cytotoxicity"))?}))
}
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty instrument evidence")?["message"];
    if compiled["task"] != "14E" || fixture["task"] != "14E" || fixture["type"] != "fixtures" {
        return Err("instrument task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("instrument participants")?;
    if actors.len() != 4
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("BIOLOGIST"),
                Some("CHEMIST"),
                Some("LEAD_SCIENTIST"),
                Some("TOXICOLOGIST"),
            ])
    {
        return Err("instrument participants differ".into());
    }
    let names = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() != 13
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("instrument completion or session differs".into());
    }
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let pairs = [
        ("CHEMIST", "CHEMIST_IO"),
        ("BIOLOGIST", "BIOLOGIST_IO"),
        ("TOXICOLOGIST", "TOXICOLOGIST_IO"),
        ("LEAD_SCIENTIST", "LEAD_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((names("roles", vm)?, names("roles", io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut used_assignments = BTreeSet::new();
    let decision_label = names("labels", "Decision")? as u8;
    let lead = names("roles", "LEAD_SCIENTIST")?;
    let chemist = names("roles", "CHEMIST")?;
    let decision_frame = one(frames
        .values()
        .filter(|f| f.direction(lead, chemist) && f.logical_label == decision_label))?;
    let decision = decision_frame.value()?;
    if !decision["go"].is_boolean() {
        return Err("missing actual go/no-go decision".into());
    }
    let submitted_call = one(ordered
        .iter()
        .copied()
        .filter(|call| call.request["tool"] == "submit_decision"))?;
    let (submitted_at, submitted) = submitted_call.result()?;
    let owner = names("roles", "HPLC_SERVICE")?;
    let request_frame = one(frames.values().filter(|f| {
        f.direction(lead, owner) && f.logical_label == compiled["labels"]["DecisionForOwner"]
    }))?;
    let submitted_frame = one(frames.values().filter(|f| {
        f.direction(owner, lead) && f.logical_label == compiled["labels"]["DecisionSubmitted"]
    }))?;
    if submitted_call.request["agent"] != "LEAD_SCIENTIST"
        || submitted_call.request["arguments"] != json!({"go":decision["go"]})
        || request_frame.received()? >= submitted_call.emitted
        || submitted_at >= submitted_frame.sent
        || submitted_frame.received()? >= decision_frame.sent
        || submitted_frame.value()? != *submitted
    {
        return Err("decision does not consume its actual submitted result".into());
    }
    let at = submitted_call.emitted;
    let mut expected_decision = json!({"go":decision["go"],"compound":compound(&ordered,at)?,"bio":bio(&ordered,at)?,"tox":tox(&ordered,at)?,
  "reference":actual(&ordered,at,("LEAD_SCIENTIST","run_mass_spec_analysis","sample","reference_methods"))?,
  "analyses":{"structural":actual(&ordered,at,("LEAD_SCIENTIST","run_hplc_analysis","sample","structural_confirmation"))?,
  "characterization":actual(&ordered,at,("LEAD_SCIENTIST","run_mass_spec_analysis","sample","characterization"))?}});
    if request_frame.value()? != expected_decision {
        return Err("submitted decision differs from actual analysis data".into());
    }
    expected_decision["receipt"] = submitted.clone();
    if decision != expected_decision {
        return Err("decision differs from actual completed analysis data".into());
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("instrument score events")?
        .clone();
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("effect scientist")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        if tool != "submit_decision" {
            one(actor["calls"]
                .as_array()
                .ok_or("instrument assignments")?
                .iter()
                .filter(|a| a["tool"] == tool && a["arguments"] == *arguments))?;
        }
        if !used_assignments.insert((agent, tool, serde_json::to_string(arguments)?)) {
            return Err("repeated assigned analysis".into());
        }
        let (result_at, result) = call.result()?;
        if result["success"] != true {
            return Err("unexpected terminal analysis failure".into());
        }
        let at = scored
            .iter()
            .position(|e| {
                e["agent"] == agent
                    && e["tool"] == tool
                    && e["args"] == *arguments
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("actual analysis differs from original score")?;
        scored.remove(at);
        if tool == "submit_decision" {
            close_arguments(
                &frames,
                &mut used_closes,
                lead,
                names("roles", "LEAD_IO")?,
                request_frame.sent,
                "service/submit_decision",
                arguments,
            )?;
            continue;
        }
        let (prefix, resource, service) = match tool {
            "run_hplc_analysis" => ("Hplc", "HPLC", "HPLC_SERVICE"),
            "run_mass_spec_analysis" => ("Mass", "MASS_SPEC", "MASS_SERVICE"),
            "run_cell_assay" => ("Cell", "CELL_LAB", "CELL_SERVICE"),
            _ => return Err("unknown instrument operation".into()),
        };
        let vm = names("roles", agent)?;
        let owner = names("roles", service)?;
        let resource = names("roles", resource)?;
        let envelope = if agent == "CHEMIST"
            && tool == "run_hplc_analysis"
            && arguments["sample"] == "documentation"
        {
            if decision_frame.received()? >= call.emitted {
                return Err("documentation lacks the actual decision delivery".into());
            }
            json!({"arguments":arguments,"decision":decision})
        } else {
            arguments.clone()
        };
        let mut before = call.emitted;
        for (from, to, key, value, reply) in [
            (
                vm,
                owner,
                format!("{prefix}Intent"),
                envelope.clone(),
                false,
            ),
            (resource, owner, format!("{prefix}Request"), envelope, false),
            (
                owner,
                resource,
                format!("{prefix}Result"),
                result.clone(),
                true,
            ),
        ] {
            let label = names("labels", &key)? as u8;
            let candidates = frames.iter().filter(|(id, f)| {
                !used_frames.contains(*id)
                    && f.direction(from, to)
                    && f.logical_label == label
                    && if reply {
                        f.sent > result_at
                    } else {
                        f.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, f) = if reply {
                candidates.min_by_key(|(_, f)| f.sent)
            } else {
                candidates.max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing {key} for {agent} {tool}"))?;
            if f.value()? != value {
                return Err(format!("altered {key} for {agent} {tool}").into());
            }
            f.received()?;
            used_frames.insert(*id);
            if !reply {
                before = before.min(f.sent);
            }
        }
        let io = match agent {
            "CHEMIST" => "CHEMIST_IO",
            "BIOLOGIST" => "BIOLOGIST_IO",
            "TOXICOLOGIST" => "TOXICOLOGIST_IO",
            "LEAD_SCIENTIST" => "LEAD_IO",
            _ => return Err("scientist role".into()),
        };
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            names("roles", io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
    }
    if !scored.is_empty() {
        return Err("unmatched actual instrument score event".into());
    }
    for key in [
        "CompoundBio",
        "CompoundTox",
        "BioReport",
        "ToxReport",
        "Decision",
        "DecisionForOwner",
        "DecisionSubmitted",
    ] {
        let label = names("labels", key)? as u8;
        one(frames.values().filter(|f| f.logical_label == label))?;
    }
    for (id, f) in &frames {
        let key = [
            "CompoundBio",
            "CompoundTox",
            "BioReport",
            "ToxReport",
            "Decision",
            "DecisionForOwner",
            "DecisionSubmitted",
        ]
        .into_iter()
        .find(|key| compiled["labels"][*key] == f.logical_label);
        if let Some(key) = key {
            let expected = match key {
                "CompoundBio" | "CompoundTox" => compound(&ordered, f.sent)?,
                "BioReport" => bio(&ordered, f.sent)?,
                "ToxReport" => tox(&ordered, f.sent)?,
                "DecisionForOwner" => {
                    let mut v = expected_decision.clone();
                    v.as_object_mut()
                        .ok_or("decision object")?
                        .remove("receipt");
                    v
                }
                "DecisionSubmitted" => submitted.clone(),
                _ => expected_decision.clone(),
            };
            if f.value()? != expected {
                return Err(format!("{key} differs from actual analysis data").into());
            }
            f.received()?;
            used_frames.insert(*id);
        }
    }
    let effect_labels = [
        "HplcIntent",
        "HplcRequest",
        "HplcResult",
        "MassIntent",
        "MassRequest",
        "MassResult",
        "CellIntent",
        "CellRequest",
        "CellResult",
        "CompoundBio",
        "CompoundTox",
        "BioReport",
        "ToxReport",
        "Decision",
        "DecisionForOwner",
        "DecisionSubmitted",
    ]
    .into_iter()
    .map(|key| names("labels", key).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, f)| effect_labels.contains(&f.logical_label) && !used_frames.contains(id))
    {
        return Err("unused or reused instrument operation/data delivery".into());
    }
    let mut independent_overlaps = 0;
    for (i, a) in ordered.iter().enumerate() {
        for b in &ordered[i + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                if a.tool()? == b.tool()? {
                    return Err("overlapping use of one exclusive instrument".into());
                }
                independent_overlaps += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"14E","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
  "go":decision["go"],"independent_instrument_overlaps":independent_overlaps,"common_causal_score":"passed",
  "effect_scope":"actual one-use admissions, instrument requests, original results, compound/report/decision data; EPF owns dynamic order"}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}

/// Change saved observations only; these are auditor checks, not runtime attacks.
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    check(&rows, &artifact)?;
    fs::create_dir(output)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let business = [
        "HplcIntent",
        "HplcRequest",
        "HplcResult",
        "MassIntent",
        "MassRequest",
        "MassResult",
        "CellIntent",
        "CellRequest",
        "CellResult",
        "CompoundBio",
        "CompoundTox",
        "BioReport",
        "ToxReport",
        "Decision",
        "DecisionForOwner",
        "DecisionSubmitted",
    ]
    .into_iter()
    .map(|key| {
        artifact["compiled"]["labels"][key]
            .as_u64()
            .ok_or("compiled analysis label")
    })
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let selected = labels
        .iter()
        .filter(|(_, label)| business.contains(&u64::from(**label)))
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    if selected.len() != 43 {
        return Err("expected twelve analysis triples and seven decision/data deliveries".into());
    }
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),
        "source_events_sha256":digest(path)?,"scope":"static saved-record effect/data audit, not runtime attacks or EPF mutation replay",
        "model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
