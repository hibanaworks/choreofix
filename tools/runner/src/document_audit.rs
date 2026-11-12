//! Actual document requests, material and receipts; EPF checks source execution.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, check_close_pairs, close_arguments, json_frame_mutants, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

const CHAPTERS: [&str; 2] = ["CHAPTER_1", "CHAPTER_2"];

fn snapshot(calls: &[&Call], before: usize) -> Result<Value> {
    let mut completed = calls
        .iter()
        .filter(|call| call.result.as_ref().is_some_and(|(at, _)| *at < before))
        .copied()
        .collect::<Vec<_>>();
    completed.sort_by_key(|call| call.result.as_ref().map(|(at, _)| *at));
    let mut material = json!({"sections":{},"references":{}});
    for call in completed {
        let tool = call.tool()?;
        let category = match tool {
            "write_section" | "revise_section" => "sections",
            "update_references" => "references",
            _ => continue,
        };
        let (_, result) = call.result()?;
        let chapter = call.request["arguments"]["section_name"]
            .as_str()
            .filter(|name| CHAPTERS.contains(name))
            .ok_or("material chapter")?;
        if result["success"] != true
            || !result["data"]["artifact"]["id"].is_string()
            || !result["data"]["artifact"]["content"].is_string()
        {
            return Err("material lacks its actual successful result".into());
        }
        material[category][chapter] = result["data"]["artifact"].clone();
    }
    Ok(material)
}

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty document evidence")?["message"];
    if compiled["task"] != "7E" || fixture["task"] != "7E" || fixture["type"] != "fixtures" {
        return Err("document task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("document participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("WRITER_A"), Some("WRITER_B"), Some("EDITOR")])
    {
        return Err("document participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let service = name("roles", "DOCUMENT_SERVICE")?;
    let document = name("roles", "DOCUMENT")?;
    let editor = name("roles", "EDITOR")?;
    let writer_a = name("roles", "WRITER_A")?;
    let session = compiled["session"].as_u64().ok_or("document session")?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if frames.values().any(|f| f.identity["session"] != session)
        || calls.is_empty()
        || score["complete"] != true
        || score["violations"] != json!([])
    {
        return Err("document session or completion differs".into());
    }
    let pairs = [
        (name("roles", "WRITER_A")?, name("roles", "A_IO")?),
        (name("roles", "WRITER_B")?, name("roles", "B_IO")?),
        (editor, name("roles", "EDITOR_IO")?),
    ];
    check_close_pairs(&frames, &pairs)?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut originals = score["events"]
        .as_array()
        .ok_or("document score events")?
        .clone();
    let mut resources = 0;
    let mut reviews = 0;
    let mut publications = 0;
    let mut material_ids = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for call in &ordered {
        let agent = call.request["agent"]
            .as_str()
            .ok_or("document effect actor")?;
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let (result_at, result) = call.result()?;
        let actor = name("roles", agent)?;
        let (io, chapter) = match agent {
            "WRITER_A" => ("A_IO", CHAPTERS[0]),
            "WRITER_B" => ("B_IO", CHAPTERS[1]),
            "EDITOR" => ("EDITOR_IO", ""),
            _ => return Err("unassigned document actor".into()),
        };
        let object = args.as_object().ok_or("document arguments")?;
        let primary = if tool == "combine_sections" {
            "sections"
        } else {
            "section_name"
        };
        let optional = match tool {
            "write_section" => Some("content_notes"),
            "revise_section" => Some("feedback"),
            _ => None,
        };
        if !args[primary].is_string()
            || object.iter().any(|(key, value)| {
                !value.is_string() || (key != primary && Some(key.as_str()) != optional)
            })
        {
            return Err("document argument codec differs".into());
        }
        let scope = args["section_name"].as_str().unwrap_or("");
        let (admission, request, reply, target) = match (agent, tool) {
            ("WRITER_A", "write_section") if scope == chapter => {
                (Some("WriteA"), "WriteOnDocument", "Written", document)
            }
            ("WRITER_B", "write_section") if scope == chapter => {
                (Some("WriteB"), "WriteOnDocument", "Written", document)
            }
            ("WRITER_A", "revise_section") if CHAPTERS.contains(&scope) => {
                (Some("ReviseA"), "ReviseOnDocument", "Revised", document)
            }
            ("WRITER_B", "revise_section") if CHAPTERS.contains(&scope) => {
                (Some("ReviseB"), "ReviseOnDocument", "Revised", document)
            }
            ("WRITER_A", "update_references") if scope == chapter => {
                (None, "ReferencesA", "ReferencedA", actor)
            }
            ("WRITER_B", "update_references") if scope == chapter => {
                (None, "ReferencesB", "ReferencedB", actor)
            }
            ("EDITOR", "review_sections") if scope == CHAPTERS[0] => {
                (None, "ReviewA", "ReviewedA", editor)
            }
            ("EDITOR", "review_sections") if scope == CHAPTERS[1] => {
                (None, "ReviewB", "ReviewedB", editor)
            }
            ("EDITOR", "combine_sections") => {
                (Some("Combine"), "CombineOnDocument", "Combined", document)
            }
            _ => return Err("unassigned document effect".into()),
        };
        let request_label = name("labels", request)? as u8;
        let (id, request) = frames
            .iter()
            .filter(|(_, frame)| {
                frame.direction(target, service)
                    && frame.logical_label == request_label
                    && frame.received.is_some_and(|at| at < call.emitted)
            })
            .max_by_key(|(_, frame)| frame.received)
            .ok_or("missing received document request")?;
        if request.value()? != *args || !used.insert(*id) {
            return Err("document request altered or reused".into());
        }
        let mut before = request.sent;
        if let Some(admission) = admission {
            resources += 1;
            let label = name("labels", admission)? as u8;
            let (id, admitted) = frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(actor, service)
                        && frame.logical_label == label
                        && frame.sent < request.sent
                        && frame.received.is_some_and(|at| at < call.emitted)
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("missing admitted document arguments")?;
            if admitted.value()? != *args || !used.insert(*id) {
                return Err("admitted document arguments altered or reused".into());
            }
            before = before.min(admitted.sent);
        }
        close_arguments(
            &frames,
            &mut closes,
            actor,
            name("roles", io)?,
            before,
            &format!("service/{tool}"),
            args,
        )?;
        let label = name("labels", reply)? as u8;
        let (id, reply) = frames
            .iter()
            .filter(|(_, frame)| {
                frame.direction(service, target)
                    && frame.logical_label == label
                    && frame.sent > result_at
            })
            .min_by_key(|(_, frame)| frame.sent)
            .ok_or("missing actual document reply")?;
        if reply.value()? != *result || !used.insert(*id) {
            return Err("document reply altered or reused".into());
        }
        reply.received()?;
        let index = originals
            .iter()
            .position(|event| {
                event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == *args
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("original document effect differs")?;
        originals.remove(index);
        if result["data"]["agent"] != agent {
            return Err("document result actor differs".into());
        }
        match tool {
            "write_section" | "revise_section" | "update_references" => {
                let data = &result["data"];
                if result["success"] != true
                    || data["section"] != scope
                    || !data["artifact"]["content"].is_string()
                    || !material_ids.insert(
                        data["artifact"]["id"]
                            .as_str()
                            .ok_or("material identity")?
                            .to_owned(),
                    )
                {
                    return Err("document material altered or reused".into());
                }
            }
            "review_sections" => {
                reviews += 1;
                let current = snapshot(&ordered, call.emitted)?;
                if CHAPTERS.iter().any(|name| {
                    !current["sections"][name]["id"].is_string()
                        || !current["references"][name]["id"].is_string()
                }) || result["data"]["input_snapshot"] != current
                    || result["data"]["scope"] != scope
                    || !result["success"].is_boolean()
                    || result["data"]["status"]
                        != if result["success"] == true {
                            "reviewed"
                        } else {
                            "needs_revision"
                        }
                    || !review_ids.insert(
                        result["data"]["review_id"]
                            .as_str()
                            .ok_or("review identity")?
                            .to_owned(),
                    )
                {
                    return Err("review differs from its complete current material".into());
                }
            }
            "combine_sections" => {
                publications += 1;
                let mut chapters = args["sections"]
                    .as_str()
                    .ok_or("combined chapters")?
                    .split(',')
                    .map(str::trim)
                    .collect::<Vec<_>>();
                chapters.sort_unstable();
                let current = snapshot(&ordered, call.emitted)?;
                let mut actual = result["data"]["sections"]
                    .as_array()
                    .ok_or("published chapters")?
                    .iter()
                    .map(|s| s.as_str().ok_or("published chapter"))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                actual.sort_unstable();
                if chapters != CHAPTERS
                    || actual != CHAPTERS
                    || result["success"] != true
                    || result["data"]["input_snapshot"] != current
                    || !result["data"]["artifact_id"].is_string()
                {
                    return Err("published document differs from current material".into());
                }
                for (scope, confirmation) in CHAPTERS
                    .into_iter()
                    .zip(["FinalApprovedA", "FinalApprovedB"])
                {
                    let review = ordered
                        .iter()
                        .filter(|review| {
                            review.request["tool"] == "review_sections"
                                && review.request["arguments"]["section_name"] == scope
                                && review
                                    .result
                                    .as_ref()
                                    .is_some_and(|(at, _)| *at < call.emitted)
                        })
                        .max_by_key(|review| review.result.as_ref().map(|(at, _)| *at))
                        .ok_or("publication without actual review")?;
                    let (at, value) = review.result()?;
                    if value["success"] != true
                        || value["data"]["input_snapshot"] != current
                        || result["data"]["review_receipts"][scope] != value["data"]["review_id"]
                    {
                        return Err("publication reused a stale or negative review".into());
                    }
                    let label = name("labels", confirmation)? as u8;
                    let (id, frame) = frames
                        .iter()
                        .filter(|(_, frame)| {
                            frame.direction(editor, service)
                                && frame.logical_label == label
                                && frame.sent > at
                                && frame
                                    .received
                                    .is_some_and(|received| received < call.emitted)
                        })
                        .max_by_key(|(_, frame)| frame.received)
                        .ok_or("publication lacks actual review confirmation")?;
                    if frame.value()? != *value || !used.insert(*id) {
                        return Err("review confirmation altered or reused".into());
                    }
                }
                let label = name("labels", "CombineForward")? as u8;
                let (id, forward) = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(editor, writer_a)
                            && frame.logical_label == label
                            && frame.sent < request.sent
                            && frame.received.is_some_and(|at| at < call.emitted)
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("resource request without actual editor input")?;
                if forward.value()? != *args || !used.insert(*id) {
                    return Err("forwarded publication arguments altered or reused".into());
                }
            }
            _ => unreachable!(),
        }
    }
    if !originals.is_empty() || publications != 1 {
        return Err("unmatched effect or repeated publication".into());
    }
    let required = [
        "WriteA",
        "WriteB",
        "WriteOnDocument",
        "Written",
        "ReviseA",
        "ReviseB",
        "ReviseOnDocument",
        "Revised",
        "ReferencesA",
        "ReferencesB",
        "ReferencedA",
        "ReferencedB",
        "ReviewA",
        "ReviewB",
        "ReviewedA",
        "ReviewedB",
        "Combine",
        "CombineForward",
        "CombineOnDocument",
        "Combined",
        "FinalApprovedA",
        "FinalApprovedB",
    ]
    .into_iter()
    .map(|label| name("labels", label).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, frame)| required.contains(&frame.logical_label) && !used.contains(id))
    {
        return Err("unused document request, receipt or result".into());
    }
    Ok(
        json!({"status":"passed","task":"7E","domain_calls":calls.len(),
        "actual_resource_endpoint_requests":resources,"actual_review_requests":reviews,
        "simulator_complete":true,"violations":0,"matched_effect_frame_ids":used,
        "effect_scope":"one-use actual admitted requests and replies; current chapter/reference material; completed matching review receipts at publication"}),
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
    let baseline = check(&rows, &artifact)?;
    let ids = baseline["matched_effect_frame_ids"]
        .as_array()
        .ok_or("effect frame IDs")?
        .iter()
        .map(|id| id.as_u64().ok_or("effect frame ID"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    fs::create_dir(output)?;
    let mut verdicts = json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(if field == "lane" { 255 } else { 65535 });
            }
        }
        let error = check(&changed, &artifact)
            .err()
            .ok_or("coherent invalid wire header accepted")?;
        verdicts.push(json!({"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","rejected":verdicts.len(),"mutants":verdicts,
        "scope":"immutable record mutations of static wire and actual effect correspondence; not runtime attacks or general refinement"});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
