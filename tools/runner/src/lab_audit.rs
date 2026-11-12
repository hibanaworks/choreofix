//! Actual accepted arguments, specimens and quality results; EPF owns replay.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn actual(calls: &[&Call], at: usize, kind: (&str, &str, &str, &str)) -> Result<Value> {
    let (actor, tool, key, value) = kind;
    let call = one(calls.iter().copied().filter(|c| {
        c.request["agent"] == actor
            && c.request["tool"] == tool
            && c.request["arguments"][key] == value
            && c.result.as_ref().is_some_and(|(end, _)| *end < at)
    }))?;
    Ok(call.result()?.1.clone())
}
fn sample(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(
        json!({"raw":actual(calls,at,("CHEMIST","run_analysis","sample","raw_materials"))?,
        "purification":actual(calls,at,("CHEMIST","run_separation","material","compound_purification"))?}),
    )
}
fn biology(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"sample":sample(calls,at)?,
        "prep":actual(calls,at,("BIOLOGIST","run_separation","material","cell_cultures"))?,
        "calibration":actual(calls,at,("BIOLOGIST","run_analysis","sample","fluorescence"))?,
        "assay":actual(calls,at,("BIOLOGIST","run_bioassay","compound","compound_purification"))?}))
}
fn quality(calls: &[&Call], at: usize) -> Result<Value> {
    Ok(json!({"biology":biology(calls,at)?,
        "molecular":actual(calls,at,("ANALYST","run_analysis","sample","molecular_detail"))?,
        "separation":actual(calls,at,("ANALYST","run_separation","material","verification"))?}))
}
const BUSINESS: [&str; 14] = [
    "SpecIntent",
    "SpecRequest",
    "SpecResult",
    "CentIntent",
    "CentRequest",
    "CentResult",
    "Sample",
    "BioRequest",
    "BioResult",
    "BioProof",
    "BioReport",
    "Verification",
    "Quality",
    "QualityForOwner",
];

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty laboratory evidence")?["message"];
    if compiled["task"] != "13E" || fixture["task"] != "13E" || fixture["type"] != "fixtures" {
        return Err("laboratory task differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("scientists")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("CHEMIST"), Some("BIOLOGIST"), Some("ANALYST")])
    {
        return Err("laboratory participants differ".into());
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
    if calls.len() != 8
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("laboratory outcome or session differs".into());
    }
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let pairs = [
        ("CHEMIST", "CHEMIST_IO"),
        ("BIOLOGIST", "BIOLOGIST_IO"),
        ("ANALYST", "ANALYST_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((names("roles", vm)?, names("roles", io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut assigned = BTreeSet::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("laboratory score events")?
        .clone();
    for call in &ordered {
        let actor = call.request["agent"].as_str().ok_or("scientist name")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let scientist = one(actors.iter().filter(|a| a["agent"] == actor))?;
        if tool == "run_bioassay" {
            let object = arguments.as_object().ok_or("bioassay argument object")?;
            if actor != "BIOLOGIST"
                || arguments["compound"] != "compound_purification"
                || object.keys().any(|k| k != "compound" && k != "cell_type")
                || object.get("cell_type").is_some_and(|v| !v.is_string())
            {
                return Err(
                    "bioassay arguments differ from the delivered specimen or schema".into(),
                );
            }
        } else {
            one(scientist["calls"]
                .as_array()
                .ok_or("scientist assignments")?
                .iter()
                .filter(|a| a["tool"] == tool && a["arguments"] == *arguments))?;
        }
        if !assigned.insert((actor, tool, serde_json::to_string(arguments)?)) {
            return Err("duplicate assigned laboratory effect".into());
        }
        let (ended, result) = call.result()?;
        if result["success"] != true {
            return Err("unexpected terminal laboratory failure".into());
        }
        let mut normalized = arguments.clone();
        if tool == "run_bioassay" && normalized.get("cell_type").is_none() {
            normalized["cell_type"] = json!("");
        }
        let position = scored
            .iter()
            .position(|e| {
                e["agent"] == actor
                    && e["tool"] == tool
                    && e["args"] == normalized
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("laboratory effect differs from common outcome record")?;
        scored.remove(position);
        let vm = names("roles", actor)?;
        let io = match actor {
            "CHEMIST" => "CHEMIST_IO",
            "BIOLOGIST" => "BIOLOGIST_IO",
            "ANALYST" => "ANALYST_IO",
            _ => return Err("unknown scientist".into()),
        };
        let mut expected = Vec::new();
        if tool == "run_bioassay" {
            let owner = names("roles", "CENT_SERVICE")?;
            let source = sample(&ordered, call.emitted)?;
            let prep = actual(
                &ordered,
                call.emitted,
                ("BIOLOGIST", "run_separation", "material", "cell_cultures"),
            )?;
            expected.push((
                vm,
                owner,
                "BioRequest".into(),
                json!({"arguments":arguments,"purification":source["purification"],"prep":prep}),
                false,
            ));
            expected.push((owner, vm, "BioResult".into(), result.clone(), true));
        } else {
            let (prefix, resource, owner) = match tool {
                "run_analysis" => ("Spec", "SPECTROMETER", "SPEC_SERVICE"),
                "run_separation" => ("Cent", "CENTRIFUGE", "CENT_SERVICE"),
                _ => return Err("unknown laboratory tool".into()),
            };
            let owner = names("roles", owner)?;
            let resource = names("roles", resource)?;
            let envelope = if actor == "CHEMIST"
                && tool == "run_analysis"
                && arguments["sample"] == "final_validation"
            {
                json!({"arguments":arguments,"quality":quality(&ordered,call.emitted)?})
            } else {
                arguments.clone()
            };
            expected.push((
                vm,
                owner,
                format!("{prefix}Intent"),
                envelope.clone(),
                false,
            ));
            expected.push((resource, owner, format!("{prefix}Request"), envelope, false));
            expected.push((
                owner,
                resource,
                format!("{prefix}Result"),
                result.clone(),
                true,
            ));
        }
        let mut before = call.emitted;
        for (from, to, key, value, reply) in expected {
            let label = names("labels", &key)? as u8;
            let candidates = frames.iter().filter(|(id, f)| {
                !used_frames.contains(*id)
                    && f.direction(from, to)
                    && f.logical_label == label
                    && if reply {
                        f.sent > ended
                    } else {
                        f.received.is_some_and(|n| n < call.emitted)
                    }
            });
            let (id, frame) = if reply {
                candidates.min_by_key(|(_, f)| f.sent)
            } else {
                candidates.max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing {key} for {actor} {tool}"))?;
            if frame.value()? != value {
                return Err(format!("altered {key} for {actor} {tool}").into());
            }
            frame.received()?;
            used_frames.insert(*id);
            if !reply {
                before = before.min(frame.sent);
            }
        }
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
        return Err("unused laboratory score effect".into());
    }
    for key in [
        "Sample",
        "BioProof",
        "BioReport",
        "Verification",
        "Quality",
        "QualityForOwner",
    ] {
        let label = names("labels", key)? as u8;
        one(frames.values().filter(|f| f.logical_label == label))?;
    }
    for (id, frame) in &frames {
        let key = [
            "Sample",
            "BioProof",
            "BioReport",
            "Verification",
            "Quality",
            "QualityForOwner",
        ]
        .into_iter()
        .find(|key| compiled["labels"][*key] == frame.logical_label);
        if let Some(key) = key {
            let expected = match key {
                "Sample" => sample(&ordered, frame.sent)?,
                "BioReport" => biology(&ordered, frame.sent)?,
                "Verification" => actual(
                    &ordered,
                    frame.sent,
                    ("ANALYST", "run_separation", "material", "verification"),
                )?,
                "BioProof" => {
                    let assay = one(ordered
                        .iter()
                        .copied()
                        .filter(|c| c.request["tool"] == "run_bioassay"))?;
                    if assay.result()?.0 >= frame.sent {
                        return Err("bioassay proof precedes the actual result".into());
                    }
                    json!({"purification":sample(&ordered,frame.sent)?["purification"],
                        "prep":actual(&ordered,frame.sent,("BIOLOGIST","run_separation","material","cell_cultures"))?,
                        "arguments":assay.request["arguments"],"result":assay.result()?.1})
                }
                _ => quality(&ordered, frame.sent)?,
            };
            if frame.value()? != expected {
                return Err(format!("{key} differs from actual completed laboratory data").into());
            }
            frame.received()?;
            used_frames.insert(*id);
        }
    }
    let effect_labels = BUSINESS
        .into_iter()
        .map(|key| names("labels", key).map(|n| n as u8))
        .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, f)| effect_labels.contains(&f.logical_label) && !used_frames.contains(id))
    {
        return Err("unused or extra laboratory request/data frame".into());
    }
    let mut overlaps = 0;
    for (index, a) in ordered.iter().enumerate() {
        for b in &ordered[index + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                if a.tool()? == b.tool()? && a.tool()? != "run_bioassay" {
                    return Err("overlapping exclusive laboratory instrument".into());
                }
                overlaps += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"13E","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
        "common_causal_score":"passed","independent_instrument_overlaps":overlaps,
        "scope":"one-use actual WASI admissions, instrument requests, specimen and quality data; EPF owns dynamic order"}),
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
            .ok_or("laboratory evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    check(&rows, &artifact)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let business = BUSINESS
        .into_iter()
        .map(|key| {
            artifact["compiled"]["labels"][key]
                .as_u64()
                .ok_or("laboratory label")
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let selected = labels
        .iter()
        .filter(|(_, label)| business.contains(&u64::from(**label)))
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    if selected.len() != 29 {
        return Err(
            "expected seven instrument triples, bioassay request/reply and six data deliveries"
                .into(),
        );
    }
    fs::create_dir(output)?;
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,
        "scope":"static saved-record audit, not runtime attacks or EPF mutation replay","model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
