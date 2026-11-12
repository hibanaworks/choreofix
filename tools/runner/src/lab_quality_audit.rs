//! Bind actual WASI admissions and scientific receipts to effects; EPF owns progress.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn latest<'a>(
    calls: &[&'a Call],
    at: usize,
    query: (&str, &str, Option<(&str, &str)>),
) -> Result<&'a Call> {
    let (actor, tool, argument) = query;
    calls
        .iter()
        .copied()
        .filter(|call| {
            call.request["agent"] == actor
                && call.request["tool"] == tool
                && argument.is_none_or(|(key, value)| call.request["arguments"][key] == value)
                && call.result.as_ref().is_some_and(|(end, _)| *end < at)
        })
        .max_by_key(|call| call.emitted)
        .ok_or_else(|| format!("missing completed {actor} {tool}").into())
}
fn value(calls: &[&Call], at: usize, query: (&str, &str, Option<(&str, &str)>)) -> Result<Value> {
    Ok(latest(calls, at, query)?.result()?.1.clone())
}
fn specimen(calls: &[&Call], at: usize) -> Result<Value> {
    let initial = calls
        .iter()
        .copied()
        .filter(|call| {
            call.request["agent"] == "CHEMIST"
                && call.request["tool"] == "run_analysis"
                && call.request["arguments"]["sample"] == "synthesized_compound"
                && call.result.as_ref().is_some_and(|(end, _)| *end < at)
        })
        .min_by_key(|call| call.emitted)
        .ok_or("missing initial compound measurement")?;
    Ok(
        json!({"synthesis":value(calls,at,("CHEMIST","run_synthesis",None))?,
        "analysis":initial.result()?.1,
        "purification":value(calls,at,("CHEMIST","run_separation",Some(("material","compound_purification"))))?}),
    )
}
fn biology(calls: &[&Call], at: usize) -> Result<Value> {
    let assay = latest(calls, at, ("BIOLOGIST", "run_bioassay", None))?;
    Ok(json!({"sample":specimen(calls,at)?,
        "preparation":value(calls,at,("BIOLOGIST","run_separation",Some(("material","cell_cultures"))))?,
        "calibration":value(calls,at,("BIOLOGIST","run_analysis",Some(("sample","fluorescence"))))?,
        "arguments":assay.request["arguments"],"result":assay.result()?.1}))
}
fn approvals(calls: &[&Call], at: usize) -> Result<Value> {
    let result = json!({"chemist":value(calls,at,("QC_CONTROLLER","verify_result",Some(("scientist","CHEMIST"))))?,
        "biologist":value(calls,at,("QC_CONTROLLER","verify_result",Some(("scientist","BIOLOGIST"))))?});
    if result["chemist"]["success"] != true || result["biologist"]["success"] != true {
        return Err("final approvals include an actual rejection".into());
    }
    Ok(result)
}
fn quality(calls: &[&Call], at: usize) -> Result<Value> {
    let mut actual = biology(calls, at)?;
    actual["qc"] = approvals(calls, at)?;
    Ok(json!({"biology":actual,
        "molecular":value(calls,at,("ANALYST","run_analysis",Some(("sample","molecular_detail"))))?,
        "separation":value(calls,at,("ANALYST","run_separation",Some(("material","verification"))))?}))
}

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty laboratory evidence")?["message"];
    if compiled["task"] != "13M" || fixture["task"] != "13M" || fixture["type"] != "fixtures" {
        return Err("QC laboratory task differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("scientists")?;
    if actors.len() != 4
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("CHEMIST"),
                Some("BIOLOGIST"),
                Some("ANALYST"),
                Some("QC_CONTROLLER"),
            ])
    {
        return Err("QC laboratory participants differ".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u8> {
        compiled["labels"][name]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or_else(|| format!("compiled label {name}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() < 11
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete or unsafe QC laboratory outcome".into());
    }
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let pairs = [
        ("CHEMIST", "CHEMIST_IO"),
        ("BIOLOGIST", "BIOLOGIST_IO"),
        ("ANALYST", "ANALYST_IO"),
        ("QC_CONTROLLER", "QC_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let business = |frame: &Frame| {
        !pairs
            .iter()
            .any(|(vm, io)| frame.direction(*vm, *io) || frame.direction(*io, *vm))
    };
    let contracts = crate::projection::contracts(compiled)?;
    for (name, count) in [
        ("Sample", 1),
        ("ApprovedResults", 1),
        ("ApprovedBiology", 1),
        ("Quality", 1),
        ("QualityForOwner", 1),
        ("VerificationProof", 1),
        ("PurificationProof", 1),
        ("CalibrationProof", 1),
        (
            "AnalysisProof",
            ordered
                .iter()
                .filter(|call| {
                    call.request["agent"] == "CHEMIST"
                        && call.request["tool"] == "run_analysis"
                        && call.request["arguments"]["sample"] == "synthesized_compound"
                })
                .count(),
        ),
        (
            "ChemistReport",
            ordered
                .iter()
                .filter(|call| {
                    call.request["tool"] == "verify_result"
                        && call.request["arguments"]["scientist"] == "CHEMIST"
                })
                .count(),
        ),
        (
            "BiologyReport",
            ordered
                .iter()
                .filter(|call| {
                    call.request["tool"] == "verify_result"
                        && call.request["arguments"]["scientist"] == "BIOLOGIST"
                })
                .count(),
        ),
    ] {
        let logical = label(name)?;
        if frames
            .values()
            .filter(|frame| business(frame) && frame.logical_label == logical)
            .count()
            != count
        {
            return Err(format!("missing or extra {name} delivery").into());
        }
    }
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("actual laboratory score events")?
        .clone();
    for call in &ordered {
        let actor = call.request["agent"].as_str().ok_or("scientist")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let object = arguments.as_object().ok_or("actual arguments")?;
        match (actor, tool) {
            ("CHEMIST", "run_synthesis")
                if object.len() == 1 && arguments["compound"].is_string() => {}
            ("BIOLOGIST", "run_bioassay") => {
                let synthesis = latest(&ordered, call.emitted, ("CHEMIST", "run_synthesis", None))?;
                if arguments["compound"] != synthesis.request["arguments"]["compound"]
                    || object
                        .keys()
                        .any(|key| key != "compound" && key != "cell_type")
                    || object
                        .get("cell_type")
                        .is_some_and(|value| !value.is_string())
                {
                    return Err("assay differs from the actual batch or schema".into());
                }
            }
            _ => {
                let scientist = one(actors.iter().filter(|a| a["agent"] == actor))?;
                one(scientist["calls"]
                    .as_array()
                    .ok_or("scientist assignments")?
                    .iter()
                    .filter(|assigned| {
                        assigned["tool"] == tool && assigned["arguments"] == *arguments
                    }))?;
            }
        }
        let (ended, result) = call.result()?;
        if result["success"] != true
            && !matches!(
                (tool, result["data"]["status"].as_str()),
                ("run_bioassay", Some("bioassay_failed")) | ("verify_result", Some("rejected"))
            )
        {
            return Err(
                "missing-input or unknown failure is not a valid scientific attempt".into(),
            );
        }
        let mut normalized = arguments.clone();
        if tool == "run_bioassay" && normalized.get("cell_type").is_none() {
            normalized["cell_type"] = json!("");
        }
        let position = scored
            .iter()
            .position(|event| {
                event["agent"] == actor
                    && event["tool"] == tool
                    && event["args"] == normalized
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("effect differs from the common world occurrence")?;
        scored.remove(position);
        let vm = role(actor)?;
        let mut expected = Vec::new();
        if tool == "verify_result" {
            let subject = arguments["scientist"].as_str().ok_or("QC subject")?;
            let expected_type = match subject {
                "CHEMIST" => "purification",
                "BIOLOGIST" => "bioassay",
                _ => return Err("unknown QC subject".into()),
            };
            if arguments["data_type"] != expected_type || object.len() != 2 {
                return Err("QC subject/type mismatch".into());
            }
            let sample = if subject == "CHEMIST" {
                let mut sample = specimen(&ordered, call.emitted)?;
                sample["analysis"] = value(
                    &ordered,
                    call.emitted,
                    (
                        "CHEMIST",
                        "run_analysis",
                        Some(("sample", "synthesized_compound")),
                    ),
                )?;
                sample
            } else {
                specimen(&ordered, call.emitted)?
            };
            let mut admitted = json!({"arguments":arguments,"sample":sample});
            if subject == "BIOLOGIST" {
                admitted["result"] =
                    value(&ordered, call.emitted, ("BIOLOGIST", "run_bioassay", None))?;
            }
            let owner = role("FUME_SERVICE")?;
            let selection_names = if subject == "CHEMIST" {
                ["FirstChemist", "NextChemist", "LastChemist"]
            } else {
                ["FirstBiologist", "NextBiologist", "LastBiologist"]
            };
            let selection_labels = selection_names
                .iter()
                .map(|name| label(name))
                .collect::<Result<BTreeSet<_>>>()?;
            let selected = frames
                .iter()
                .filter(|(id, frame)| {
                    !used_frames.contains(*id)
                        && frame.direction(vm, owner)
                        && selection_labels.contains(&frame.logical_label)
                        && frame.received.is_some_and(|at| at < call.emitted)
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("missing actual QC selection")?
                .1;
            let selected_name = *selection_names
                .iter()
                .find(|name| compiled["labels"][**name] == selected.logical_label)
                .ok_or("selection contract")?;
            let targets = contracts
                .values()
                .filter(|header| {
                    u64::from(header[1]) == vm
                        && header[3] == selected.logical_label
                        && !pairs
                            .iter()
                            .any(|(from, to)| *from == vm && *to == u64::from(header[2]))
                })
                .map(|header| u64::from(header[2]))
                .collect::<BTreeSet<_>>();
            if targets.is_empty() {
                return Err("QC selection has no compiled destinations".into());
            }
            for target in targets {
                expected.push((vm, target, vec![selected_name], arguments.clone(), false));
            }
            let resource = role(if subject == "CHEMIST" {
                "CHEMISTRY"
            } else {
                "BIOLOGY"
            })?;
            let owner = role("FUME_SERVICE")?;
            expected.extend([
                (
                    resource,
                    owner,
                    vec!["QcAdmission"],
                    admitted.clone(),
                    false,
                ),
                (
                    role("SPECTROMETER")?,
                    owner,
                    vec!["QcSpecRequest"],
                    admitted.clone(),
                    false,
                ),
                (
                    role("FUME_HOOD")?,
                    owner,
                    vec!["QcFumeRequest"],
                    admitted,
                    false,
                ),
                (
                    owner,
                    role("SPECTROMETER")?,
                    vec!["QcSpecResult"],
                    result.clone(),
                    true,
                ),
                (
                    owner,
                    role("FUME_HOOD")?,
                    vec!["QcFumeResult"],
                    result.clone(),
                    true,
                ),
            ]);
            let verdicts = match (subject, result["success"].as_bool()) {
                ("CHEMIST", Some(true)) => vec!["ChemistApproved", "LastChemistApproved"],
                ("CHEMIST", Some(false)) => vec!["ChemistRejected", "LastChemistRejected"],
                ("BIOLOGIST", Some(true)) => vec!["BiologistApproved", "LastBiologistApproved"],
                ("BIOLOGIST", Some(false)) => vec!["BiologistRejected", "LastBiologistRejected"],
                _ => return Err("non-boolean QC verdict".into()),
            };
            expected.push((owner, vm, verdicts.clone(), result.clone(), true));
            for target in ["CHEMIST", "BIOLOGIST", "SPEC_SERVICE", "CENT_SERVICE"] {
                expected.push((vm, role(target)?, verdicts.clone(), result.clone(), true));
            }
        } else if tool == "run_bioassay" {
            let owner = role("CENT_SERVICE")?;
            let admitted = json!({"arguments":arguments,
                "purification":specimen(&ordered,call.emitted)?["purification"],
                "preparation":value(&ordered,call.emitted,("BIOLOGIST","run_separation",Some(("material","cell_cultures"))))?,
                "calibration":value(&ordered,call.emitted,("BIOLOGIST","run_analysis",Some(("sample","fluorescence"))))?});
            expected.push((
                role("BIOLOGY")?,
                owner,
                vec!["BioRequest", "Retest", "LastRetest"],
                admitted,
                false,
            ));
            let success = result["success"] == true;
            expected.push((
                owner,
                vm,
                if success {
                    vec!["BioPassed", "RetestPassed", "LastRetestPassed"]
                } else {
                    vec!["BioFailed", "RetestFailed", "LastRetestFailed"]
                },
                result.clone(),
                true,
            ));
            expected.push((
                owner,
                role("FUME_SERVICE")?,
                if success {
                    vec!["BioFumePassed", "RetestFumePassed", "LastRetestFumePassed"]
                } else {
                    vec!["BioFumeFailed", "RetestFumeFailed", "LastRetestFumeFailed"]
                },
                result.clone(),
                true,
            ));
            expected.push((
                owner,
                role("SPEC_SERVICE")?,
                if success {
                    vec!["BioSpecPassed", "RetestSpecPassed", "LastRetestSpecPassed"]
                } else {
                    vec!["BioSpecFailed", "RetestSpecFailed", "LastRetestSpecFailed"]
                },
                result.clone(),
                true,
            ));
        } else {
            let (logical, owner, resource, intent, request) =
                match (actor, tool, arguments.get("sample").and_then(Value::as_str)) {
                    ("CHEMIST", "run_synthesis", _) => (
                        "CHEMISTRY",
                        "FUME_SERVICE",
                        "FUME_HOOD",
                        vec!["Synthesis"],
                        "SynthesisRequest",
                    ),
                    ("CHEMIST", "run_analysis", Some("synthesized_compound")) => (
                        "CHEMISTRY",
                        "SPEC_SERVICE",
                        "SPECTROMETER",
                        vec!["InitialAnalysis", "Reanalysis", "LastReanalysis"],
                        "SpecRequest",
                    ),
                    ("CHEMIST", "run_separation", _) => (
                        "CHEMISTRY",
                        "CENT_SERVICE",
                        "CENTRIFUGE",
                        vec!["Purification"],
                        "CentRequest",
                    ),
                    ("BIOLOGIST", "run_analysis", _) => (
                        "BIOLOGIST",
                        "SPEC_SERVICE",
                        "SPECTROMETER",
                        vec!["Calibration"],
                        "SpecRequest",
                    ),
                    ("BIOLOGIST", "run_separation", _) => (
                        "BIOLOGIST",
                        "CENT_SERVICE",
                        "CENTRIFUGE",
                        vec!["Preparation"],
                        "CentRequest",
                    ),
                    ("ANALYST", "run_analysis", _) => (
                        "ANALYST",
                        "SPEC_SERVICE",
                        "SPECTROMETER",
                        vec!["Molecular"],
                        "SpecRequest",
                    ),
                    ("ANALYST", "run_separation", _) => (
                        "ANALYST",
                        "CENT_SERVICE",
                        "CENTRIFUGE",
                        vec!["Separation"],
                        "CentRequest",
                    ),
                    ("CHEMIST", "run_analysis", Some("final_validation")) => (
                        "CHEMIST",
                        "SPEC_SERVICE",
                        "SPECTROMETER",
                        vec!["FinalValidation"],
                        "SpecRequest",
                    ),
                    _ => return Err("unknown laboratory effect".into()),
                };
            let envelope = if actor == "ANALYST" {
                json!({"arguments":arguments,"biology":value(&ordered,call.emitted,("BIOLOGIST","run_bioassay",None))?})
            } else if actor == "CHEMIST" && arguments["sample"] == "final_validation" {
                json!({"arguments":arguments,"quality":quality(&ordered,call.emitted)?})
            } else {
                arguments.clone()
            };
            expected.extend([
                (
                    role(logical)?,
                    role(owner)?,
                    intent,
                    envelope.clone(),
                    false,
                ),
                (
                    role(resource)?,
                    role(owner)?,
                    vec![request],
                    envelope,
                    false,
                ),
                (
                    role(owner)?,
                    role(resource)?,
                    vec!["InstrumentResult"],
                    result.clone(),
                    true,
                ),
            ]);
        }
        let mut before = call.emitted;
        for (from, to, names, expected, reply) in expected {
            let accepted = names
                .iter()
                .map(|name| label(name))
                .collect::<Result<BTreeSet<_>>>()?;
            let candidates = frames.iter().filter(|(id, frame)| {
                !used_frames.contains(*id)
                    && frame.direction(from, to)
                    && accepted.contains(&frame.logical_label)
                    && if reply {
                        frame.sent > ended
                    } else {
                        frame.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, frame) = if reply {
                candidates.min_by_key(|(_, frame)| frame.sent)
            } else {
                candidates.max_by_key(|(_, frame)| frame.received)
            }
            .ok_or_else(|| format!("missing {} for {actor} {tool}", names.join("/")))?;
            if frame.value()? != expected {
                return Err(format!("altered {} for {actor} {tool}", names.join("/")).into());
            }
            frame.received()?;
            used_frames.insert(*id);
            if !reply {
                before = before.min(frame.sent);
            }
        }
        let io = if actor == "QC_CONTROLLER" {
            "QC_IO"
        } else {
            match actor {
                "CHEMIST" => "CHEMIST_IO",
                "BIOLOGIST" => "BIOLOGIST_IO",
                "ANALYST" => "ANALYST_IO",
                _ => return Err("unknown actor".into()),
            }
        };
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            role(io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
    }
    if !scored.is_empty() {
        return Err("unused common-world effect".into());
    }
    for (id, frame) in &frames {
        if used_frames.contains(id) || !business(frame) {
            continue;
        }
        let name = compiled["labels"]
            .as_object()
            .ok_or("compiled names")?
            .iter()
            .find(|(_, number)| number.as_u64() == Some(u64::from(frame.logical_label)))
            .map(|(name, _)| name.as_str());
        let Some(name) = name else {
            continue;
        };
        let expected = match name {
            "Sample" => specimen(&ordered, frame.sent)?,
            "ChemistReport" => {
                let mut sample = specimen(&ordered, frame.sent)?;
                sample["analysis"] = value(
                    &ordered,
                    frame.sent,
                    (
                        "CHEMIST",
                        "run_analysis",
                        Some(("sample", "synthesized_compound")),
                    ),
                )?;
                sample
            }
            "BiologyReport" => biology(&ordered, frame.sent)?,
            "ApprovedResults" => approvals(&ordered, frame.sent)?,
            "ApprovedBiology" => {
                let mut report = biology(&ordered, frame.sent)?;
                report["qc"] = approvals(&ordered, frame.sent)?;
                report
            }
            "Quality" | "QualityForOwner" => quality(&ordered, frame.sent)?,
            "AnalysisProof" => value(
                &ordered,
                frame.sent,
                (
                    "CHEMIST",
                    "run_analysis",
                    Some(("sample", "synthesized_compound")),
                ),
            )?,
            "PurificationProof" => value(
                &ordered,
                frame.sent,
                (
                    "CHEMIST",
                    "run_separation",
                    Some(("material", "compound_purification")),
                ),
            )?,
            "CalibrationProof" => value(
                &ordered,
                frame.sent,
                (
                    "BIOLOGIST",
                    "run_analysis",
                    Some(("sample", "fluorescence")),
                ),
            )?,
            "VerificationProof" => value(
                &ordered,
                frame.sent,
                (
                    "ANALYST",
                    "run_separation",
                    Some(("material", "verification")),
                ),
            )?,
            _ => return Err(format!("unused or extra {name} frame").into()),
        };
        if frame.value()? != expected {
            return Err(format!("{name} differs from actual completed data").into());
        }
        frame.received()?;
        used_frames.insert(*id);
    }
    let mut overlaps = 0;
    let mut biological_qc_overlaps = 0;
    for (index, a) in ordered.iter().enumerate() {
        for b in &ordered[index + 1..] {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                let resources = |tool| match tool {
                    "run_analysis" => 1,
                    "run_separation" => 2,
                    "run_synthesis" => 4,
                    "verify_result" => 5,
                    _ => 0,
                };
                if resources(a.tool()?) & resources(b.tool()?) != 0 {
                    return Err("overlapping exclusive instrument effects".into());
                }
                overlaps += 1;
                if matches!(
                    (a.tool()?, b.tool()?),
                    ("run_bioassay", "verify_result") | ("verify_result", "run_bioassay")
                ) {
                    biological_qc_overlaps += 1;
                }
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"13M","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
        "common_causal_score":"passed","independent_instrument_overlaps":overlaps,"biological_qc_overlaps":biological_qc_overlaps,
        "scope":"one-use actual WASI admissions, both QC instruments, scientific receipts and actual results; EPF owns dynamic order"}),
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
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    check(&rows, &artifact)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let actors = rows.first().ok_or("fixture")?["message"]["fixtures"]
        .as_array()
        .ok_or("scientists")?;
    let journal = crate::journal::read(&rows, &labels, actors)?;
    let private = ["CHEMIST_IO", "BIOLOGIST_IO", "ANALYST_IO", "QC_IO"]
        .into_iter()
        .map(|name| {
            artifact["compiled"]["roles"][name]
                .as_u64()
                .ok_or("private I/O role")
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let business = artifact["compiled"]["labels"]
        .as_object()
        .ok_or("scientific labels")?
        .values()
        .map(|value| value.as_u64().ok_or("logical label"))
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let selected = labels
        .iter()
        .filter(|(id, label)| {
            business.contains(&u64::from(**label))
                && journal.frames.get(id).is_some_and(|frame| {
                    private.iter().all(|role| {
                        frame.identity["source"] != *role && frame.identity["target"] != *role
                    })
                })
        })
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &selected, |changed| check(changed, &artifact))?;
    let result = json!({"status":"passed","mutants_rejected":verdicts.len(),"source_events_sha256":digest(path)?,
        "scope":"saved-record effect audit, not runtime attacks or EPF mutation replay","model_calls":0,"verdicts":verdicts});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
