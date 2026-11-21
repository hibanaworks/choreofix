//! Scientific input/effect correspondence. EPF alone checks protocol execution.
use crate::{
    evidence::Result,
    journal::{self, Call, Frame, one},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use {crate::evidence::digest, std::path::Path};
fn target(call: &Call) -> Result<&str> {
    let key = match call.tool()? {
        "run_analysis" => "sample",
        "run_separation" => "material",
        "run_bioassay" => "compound",
        _ => return Err("unassigned scientific tool".into()),
    };
    call.request["arguments"][key]
        .as_str()
        .ok_or_else(|| "scientific target".into())
}
fn result<'a>(
    calls: &'a BTreeMap<u64, Call>,
    (actor, tool, name): (&str, &str, &str),
) -> Result<&'a Value> {
    one(calls.values().filter(|c| {
        c.request["agent"] == actor && c.tool().ok() == Some(tool) && target(c).ok() == Some(name)
    }))?
    .result()
    .map(|(_, v)| v)
}
fn frame<'a>(
    (frames, used): (&'a BTreeMap<u64, Frame<'a>>, &mut BTreeSet<usize>),
    (from, to, label): (u64, u64, u64),
    (value, after, before): (&Value, usize, usize),
) -> Result<&'a Frame<'a>> {
    let frame = one(frames.values().filter(|f| {
        f.direction(from, to)
            && u64::from(f.logical_label) == label
            && after < f.sent
            && f.sent < before
            && !used.contains(&f.sent)
            && f.value().is_ok_and(|v| v == *value)
    }))?;
    used.insert(frame.sent);
    Ok(frame)
}
pub(crate) fn check(rows: &[Value], projection: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &projection["compiled"];
    let fixture = &rows.first().ok_or("empty lab capture")?["message"];
    if compiled["task"] != "13E" || fixture["type"] != "fixtures" || fixture["task"] != "13E" {
        return Err("foreign laboratory source/run".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u64> {
        compiled["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled message {name}").into())
    };
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("original scientists")?;
    if actors
        .iter()
        .filter_map(|a| a["agent"].as_str())
        .collect::<BTreeSet<_>>()
        != BTreeSet::from(["CHEMIST", "BIOLOGIST", "ANALYST"])
    {
        return Err("laboratory participants differ".into());
    }
    let contracts = crate::projection::frames(projection, rows)?;
    let journal = journal::read(rows, &contracts, actors, dynamic)?;
    let journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    let events = score["events"]
        .as_array()
        .ok_or("scientific score events")?;
    if calls.len() != 8
        || events.len() != 8
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err(
            "incomplete laboratory world, extra effects or a physical resource violation".into(),
        );
    }
    let pairs = ["CHEM", "BIO", "ANALYST"]
        .map(|p| Ok((role(&format!("{p}_VM"))?, role(&format!("{p}_IO"))?)))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(&frames, &pairs)?;
    let raw = result(&calls, ("CHEMIST", "run_analysis", "raw_materials"))?;
    let cells = result(&calls, ("BIOLOGIST", "run_separation", "cell_cultures"))?;
    let purified = result(
        &calls,
        ("CHEMIST", "run_separation", "compound_purification"),
    )?;
    let calibrated = result(&calls, ("BIOLOGIST", "run_analysis", "fluorescence"))?;
    let assay = result(
        &calls,
        ("BIOLOGIST", "run_bioassay", "compound_purification"),
    )?;
    let molecular = result(&calls, ("ANALYST", "run_analysis", "molecular_detail"))?;
    let verified = result(&calls, ("ANALYST", "run_separation", "verification"))?;
    let specimen = json!({"raw":raw,"compound":purified});
    let assay_data = json!({"raw":raw,"compound":purified,"cells":cells,"calibration":calibrated,"bioassay":assay});
    let quality = json!({"assay":assay_data,"molecular":molecular,"verification":verified});
    let mut used = BTreeSet::new();
    let mut assigned = BTreeSet::new();
    let mut scored = BTreeSet::new();
    for call in calls.values() {
        let actor = call.request["agent"].as_str().ok_or("effect scientist")?;
        let tool = call.tool()?;
        let name = target(call)?;
        let args = &call.request["arguments"];
        let original = one(actors.iter().filter(|a| a["agent"] == actor))?;
        if !original["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .any(|c| {
                if c["tool"] != tool {
                    return false;
                }
                let key = match tool {
                    "run_analysis" => "sample",
                    "run_separation" => "material",
                    _ => "compound",
                };
                c["arguments"][key] == args[key]
            })
            || !assigned.insert((actor.to_owned(), tool.to_owned(), name.to_owned()))
        {
            return Err("unassigned or duplicated scientific effect".into());
        }
        let allowed = if tool == "run_bioassay" { 2 } else { 1 };
        let argument_key = match tool {
            "run_analysis" => "sample",
            "run_separation" => "material",
            _ => "compound",
        };
        if args.as_object().is_none_or(|a| {
            a.is_empty()
                || a.len() > allowed
                || a.keys().any(|key| {
                    key != argument_key && !(tool == "run_bioassay" && key == "cell_type")
                })
        }) || (args.get("cell_type").is_some() && !args["cell_type"].is_string())
        {
            return Err("scientific arguments contain invented fields".into());
        }
        let (ended, value) = call.result()?;
        let data = &value["data"];
        let key = match tool {
            "run_analysis" => "sample",
            "run_separation" => "material",
            _ => "compound",
        };
        if value["tool_name"] != tool
            || value["success"] != true
            || data["scientist"] != actor
            || data[key] != name
            || data["status"] != "complete"
            || data["missing_inputs"] != json!([])
        {
            return Err("scientific effect result differs from its actual request".into());
        }
        let (file, native, resource, request, operation, reply, file_reply, inputs, packet) = match (
            actor, tool, name,
        ) {
            ("CHEMIST", "run_analysis", "raw_materials") => (
                "CHEM_FILE",
                "CHEMIST",
                Some("SPECTROMETER"),
                "Raw",
                Some("RawOperation"),
                "RawResult",
                "FileRawResult",
                json!({}),
                json!({"arguments":args}),
            ),
            ("BIOLOGIST", "run_separation", "cell_cultures") => (
                "BIO_FILE",
                "BIOLOGIST",
                Some("CENTRIFUGE"),
                "Cells",
                Some("CellsOperation"),
                "CellsResult",
                "FileCellsResult",
                json!({}),
                json!({"arguments":args}),
            ),
            ("CHEMIST", "run_separation", "compound_purification") => (
                "CHEM_FILE",
                "CHEMIST",
                Some("CENTRIFUGE"),
                "Purify",
                Some("PurifyOperation"),
                "PurifiedResult",
                "FilePurifiedResult",
                json!({"raw":raw["data"]["artifact_id"]}),
                json!({"arguments":args,"raw":raw}),
            ),
            ("BIOLOGIST", "run_analysis", "fluorescence") => (
                "BIO_FILE",
                "BIOLOGIST",
                Some("SPECTROMETER"),
                "Calibrate",
                Some("CalibrationOperation"),
                "CalibrationResult",
                "FileCalibrationResult",
                json!({"prepared_cells":cells["data"]["artifact_id"]}),
                json!({"arguments":args,"cells":cells}),
            ),
            ("BIOLOGIST", "run_bioassay", "compound_purification") => (
                "BIO_FILE",
                "BIOLOGIST",
                None,
                "Bioassay",
                None,
                "BioassayResult",
                "BioassayResult",
                json!({"compound":purified["data"]["artifact_id"],"prepared_cells":cells["data"]["artifact_id"],"calibration":calibrated["data"]["artifact_id"]}),
                Value::Null,
            ),
            ("ANALYST", "run_analysis", "molecular_detail") => (
                "ANALYST_FILE",
                "ANALYST",
                Some("SPECTROMETER"),
                "Molecular",
                Some("MolecularOperation"),
                "MolecularResult",
                "FileMolecularResult",
                json!({"compound":purified["data"]["artifact_id"],"bioassay":assay["data"]["artifact_id"]}),
                json!({"arguments":args,"assay":assay_data}),
            ),
            ("ANALYST", "run_separation", "verification") => (
                "ANALYST_FILE",
                "ANALYST",
                Some("CENTRIFUGE"),
                "Verify",
                Some("VerificationOperation"),
                "VerificationResult",
                "FileVerificationResult",
                json!({"compound":purified["data"]["artifact_id"],"bioassay":assay["data"]["artifact_id"]}),
                json!({"arguments":args,"assay":assay_data}),
            ),
            ("CHEMIST", "run_analysis", "final_validation") => (
                "CHEM_FILE",
                "CHEMIST",
                Some("SPECTROMETER"),
                "Final",
                Some("FinalOperation"),
                "FinalResult",
                "FileFinalResult",
                json!({"compound":purified["data"]["artifact_id"],"bioassay":assay["data"]["artifact_id"],
                    "molecular":molecular["data"]["artifact_id"],"verification":verified["data"]["artifact_id"]}),
                json!({"arguments":args,"quality":quality}),
            ),
            _ => return Err("unassigned scientific target".into()),
        };
        if data["inputs"] != inputs {
            return Err("scientific effect used missing, stale or foreign material".into());
        }
        for receipt in inputs.as_object().ok_or("scientific inputs")?.values() {
            let previous = one(calls.values().filter(|c| {
                c.result()
                    .is_ok_and(|(_, v)| v["data"]["artifact_id"] == *receipt)
            }))?;
            if previous.ended()?.0 >= call.started()?.0 {
                return Err("scientific input was not complete before use".into());
            }
        }
        let admitted = frame(
            (&frames, &mut used),
            (role(file)?, role(native)?, label(request)?),
            (args, 0, call.emitted),
        )?;
        if admitted.received >= call.emitted {
            return Err("scientific effect preceded its actual file request receive".into());
        }
        if let (Some(resource), Some(operation)) = (resource, operation) {
            let received = frame(
                (&frames, &mut used),
                (role(native)?, role(resource)?, label(operation)?),
                (&packet, admitted.received, call.emitted),
            )?;
            if received.received >= call.emitted {
                return Err(
                    "instrument effect preceded its evidence-bearing resource receive".into(),
                );
            }
            frame(
                (&frames, &mut used),
                (role(resource)?, role(native)?, label(reply)?),
                (value, ended, usize::MAX),
            )?;
        }
        frame(
            (&frames, &mut used),
            (role(native)?, role(file)?, label(file_reply)?),
            (value, ended, usize::MAX),
        )?;
        let mut score_args = args.clone();
        if tool == "run_bioassay" && score_args.get("cell_type").is_none() {
            score_args["cell_type"] = json!("");
        }
        let (index, _) = one(events.iter().enumerate().filter(|(i, event)| {
            !scored.contains(i)
                && call.matches_score(event, &score_args)
                && event["success"] == true
        }))?;
        if data["artifact_id"] != index {
            return Err("scientific artifact identifies another actual world occurrence".into());
        }
        scored.insert(index);
    }
    let find_call = |(actor, tool, name): (&str, &str, &str)| -> Result<&Call> {
        one(calls.values().filter(|c| {
            c.request["agent"] == actor
                && c.tool().ok() == Some(tool)
                && target(c).ok() == Some(name)
        }))
    };
    let first_quality = find_call(("ANALYST", "run_analysis", "molecular_detail"))?
        .emitted
        .min(find_call(("ANALYST", "run_separation", "verification"))?.emitted);
    for (from, to, message, value, after, before) in [
        (
            "CHEMIST",
            "BIOLOGIST",
            "Purified",
            &specimen,
            find_call(("CHEMIST", "run_separation", "compound_purification"))?
                .ended()?
                .0,
            find_call(("BIOLOGIST", "run_bioassay", "compound_purification"))?.emitted,
        ),
        (
            "BIOLOGIST",
            "BIO_FILE",
            "Sample",
            &specimen,
            find_call(("CHEMIST", "run_separation", "compound_purification"))?
                .ended()?
                .0,
            find_call(("BIOLOGIST", "run_bioassay", "compound_purification"))?.emitted,
        ),
        (
            "BIOLOGIST",
            "ANALYST",
            "AssayNotice",
            &assay_data,
            find_call(("BIOLOGIST", "run_bioassay", "compound_purification"))?
                .ended()?
                .0,
            first_quality,
        ),
        (
            "ANALYST",
            "ANALYST_FILE",
            "FileAssay",
            &assay_data,
            find_call(("BIOLOGIST", "run_bioassay", "compound_purification"))?
                .ended()?
                .0,
            first_quality,
        ),
        (
            "ANALYST",
            "CHEMIST",
            "Quality",
            &quality,
            find_call(("ANALYST", "run_analysis", "molecular_detail"))?
                .ended()?
                .0
                .max(
                    find_call(("ANALYST", "run_separation", "verification"))?
                        .ended()?
                        .0,
                ),
            find_call(("CHEMIST", "run_analysis", "final_validation"))?.emitted,
        ),
        (
            "CHEMIST",
            "CHEM_FILE",
            "FileQuality",
            &quality,
            find_call(("ANALYST", "run_analysis", "molecular_detail"))?
                .ended()?
                .0
                .max(
                    find_call(("ANALYST", "run_separation", "verification"))?
                        .ended()?
                        .0,
                ),
            find_call(("CHEMIST", "run_analysis", "final_validation"))?.emitted,
        ),
    ] {
        let material = frame(
            (&frames, &mut used),
            (role(from)?, role(to)?, label(message)?),
            (value, after, before),
        )?;
        if material.received >= before {
            return Err("scientific effect used undelivered material".into());
        }
    }
    let business = [
        "CHEMIST",
        "BIOLOGIST",
        "ANALYST",
        "SPECTROMETER",
        "CENTRIFUGE",
        "CHEM_FILE",
        "BIO_FILE",
        "ANALYST_FILE",
    ]
    .into_iter()
    .map(role)
    .collect::<Result<BTreeSet<_>>>()?;
    for frame in frames.values() {
        if business.contains(&frame.identity["source"].as_u64().ok_or("frame source")?)
            && business.contains(&frame.identity["target"].as_u64().ok_or("frame target")?)
            && !used.contains(&frame.sent)
        {
            return Err("unused or extra scientific request/result".into());
        }
    }
    for (owner, message) in [("SPEC_OWNER", "SpecLoan"), ("CENT_OWNER", "CentLoan")] {
        let (owner, init, message) = (role(owner)?, role("INIT")?, label(message)?);
        one(frames
            .values()
            .filter(|f| f.direction(owner, init) && u64::from(f.logical_label) == message))?;
    }
    Ok(
        json!({"status":"passed","task":"13E","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
        "scientific_receipts_match_actual_completed_inputs":true,"effect_receives_bound_to_checked_occurrences":true,
        "actual_instrument_endpoint_returns":2,"guest_exits":3,"dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}
#[cfg(test)]
pub fn check_mutants(path: &Path, output: &Path) -> Result<(Value, Value)> {
    let evidence = crate::journal::Evidence::open(path)?;
    let baseline = evidence.audit(check)?;
    let (rows, artifact) = (evidence.rows, evidence.projection);
    let dynamic = &baseline["dynamic_protocol"];
    let final_effect = rows
        .iter()
        .position(|r| {
            r["direction"] == "runtime"
                && r["message"]["type"] == "execute"
                && r["message"]["arguments"]["sample"] == "final_validation"
        })
        .ok_or("final effect")?;
    let id = rows[final_effect]["message"]["id"].clone();
    let result = rows
        .iter()
        .position(|r| r["direction"] == "domain_end" && r["message"]["id"] == id)
        .ok_or("actual final result")?;
    let actual_label = artifact["compiled"]["labels"]["FinalOperation"].clone();
    let headers = crate::projection::contracts(&artifact["compiled"])?;
    let wire = one(headers.iter().filter(|(_, h)| h[3] == actual_label))?.0;
    let packet = rows
        .iter()
        .position(|r| {
            r["message"]["type"] == "protocol_frame" && r["message"]["frame_label"] == *wire
        })
        .ok_or("actual final resource request")?;
    let score = rows
        .iter()
        .position(|r| r["message"]["type"] == "score")
        .ok_or("world score")?;
    let mut reports = Vec::new();
    for name in [
        "changed_final_target",
        "negative_final_result",
        "stale_final_inputs",
        "substituted_resource_arguments",
        "foreign_assay_in_quality",
        "missing_resource_request",
        "missing_completed_assay",
        "forged_world_receipt",
        "missing_world_score",
    ] {
        let mut changed = rows.clone();
        match name {
            "changed_final_target" => {
                changed[final_effect]["message"]["arguments"]["sample"] = json!("raw_materials")
            }
            "negative_final_result" => {
                changed[result]["message"]["result"]["success"] = json!(false)
            }
            "stale_final_inputs" => {
                changed[result]["message"]["result"]["data"]["inputs"]["bioassay"] = json!(999)
            }
            "substituted_resource_arguments" | "foreign_assay_in_quality" => {
                let bytes: Vec<u8> =
                    serde_json::from_value(changed[packet]["message"]["payload"].clone())?;
                let mut input: Value = serde_json::from_slice(&bytes)?;
                if name == "substituted_resource_arguments" {
                    input["arguments"]["sample"] = json!("raw_materials");
                } else {
                    input["quality"]["assay"]["bioassay"]["data"]["artifact_id"] = json!(999);
                }
                changed[packet]["message"]["payload"] = json!(serde_json::to_vec(&input)?);
            }
            "missing_resource_request" => {
                changed.remove(packet);
            }
            "missing_completed_assay" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        r["direction"] == "domain_end" && r["message"]["tool"] == "run_bioassay"
                    })
                    .ok_or("assay result")?;
                changed.remove(at);
            }
            "forged_world_receipt" => {
                let event = changed[score]["message"]["events"]
                    .as_array_mut()
                    .ok_or("score events")?
                    .iter_mut()
                    .find(|e| e["args"]["sample"] == "final_validation")
                    .ok_or("scored final")?;
                event["result"]["inputs"]["bioassay"] = json!(999);
            }
            "missing_world_score" => {
                changed.remove(score);
            }
            _ => unreachable!(),
        }
        match check(&changed, &artifact, dynamic) {
            Ok(_) => return Err(format!("lab auditor accepted {name}").into()),
            Err(error) => {
                reports.push(json!({"name":name,"rejected":true,"error":error.to_string()}))
            }
        }
    }
    let report = json!({"status":"passed","task":"13E","record_counterexamples":reports,"original_events_sha256":digest(path)?,
        "dynamic_capture_checked_once":true,"runtime_attacks":false,"model_calls":0,"scored_runs":0});
    crate::evidence::save(output, &report)?;
    Ok((baseline, report))
}
