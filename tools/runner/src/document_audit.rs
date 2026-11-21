//! Actual document requests, material and receipts; EPF checks source execution.
use crate::{
    evidence::{Result, save},
    journal::{Call, json_frame_mutants},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

const CHAPTERS: [&str; 2] = ["CHAPTER_1", "CHAPTER_2"];

fn snapshot(calls: &[&Call], before: usize) -> Result<Value> {
    let mut completed = calls
        .iter()
        .filter(|call| call.result.0 < before)
        .copied()
        .collect::<Vec<_>>();
    completed.sort_by_key(|call| call.result.0);
    let mut material = json!({"sections":{},"references":{}});
    for call in completed {
        let tool = call.tool;
        let category = match tool {
            "write_section" | "revise_section" => "sections",
            "update_references" => "references",
            _ => continue,
        };
        let (_, result) = call.result;
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

pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = journal.execution;
    let artifact = journal.projection;
    let compiled = &artifact["compiled"];

    let name =
        |group: &str, key: &str| crate::projection::named_id(&compiled[group], key).map(u64::from);
    let service = name("roles", "DOCUMENT_SERVICE")?;
    let document = name("roles", "DOCUMENT")?;
    let editor = name("roles", "EDITOR")?;
    let writer_a = name("roles", "WRITER_A")?;
    let editor_file = name("roles", "ALLOCATION_FILE")?;
    let session = compiled["session"].as_u64().ok_or("document session")?;
    let crate::journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if frames.values().any(|f| f.identity["session"] != session)
        || calls.is_empty()
        || !simulator_complete
    {
        return Err("document session or completion differs".into());
    }
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut used = BTreeSet::new();
    let mut originals = scored_events.iter().collect::<Vec<_>>();
    let mut resources = 0;
    let mut reviews = 0;
    let mut publications = 0;
    let mut material_ids = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for call in &ordered {
        let agent = call.request["agent"]
            .as_str()
            .ok_or("document effect actor")?;
        let tool = call.tool;
        let args = &call.request["arguments"];
        let (result_at, result) = call.result;
        let actor = name("roles", agent)?;
        let (file, chapter) = match agent {
            "WRITER_A" => ("A_FILE", CHAPTERS[0]),
            "WRITER_B" => ("B_FILE", CHAPTERS[1]),
            "EDITOR" => ("ALLOCATION_FILE", ""),
            _ => return Err("unassigned document actor".into()),
        };
        let file = name("roles", file)?;
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
                (None, "ReferencesA", "ReferencedA", file)
            }
            ("WRITER_B", "update_references") if scope == chapter => {
                (None, "ReferencesB", "ReferencedB", file)
            }
            ("EDITOR", "review_sections") if scope == CHAPTERS[0] => {
                (None, "ReviewA", "ReviewedA", file)
            }
            ("EDITOR", "review_sections") if scope == CHAPTERS[1] => {
                (None, "ReviewB", "ReviewedB", file)
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
                    && frame.received < call.emitted
            })
            .max_by_key(|(_, frame)| frame.received)
            .ok_or("missing received document request")?;
        if request.value()? != *args || !used.insert(*id) {
            return Err("document request altered or reused".into());
        }
        if let Some(admission) = admission {
            resources += 1;
            let label = name("labels", admission)? as u8;
            let (id, admitted) = frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(
                        if tool == "combine_sections" {
                            actor
                        } else {
                            file
                        },
                        service,
                    ) && frame.logical_label == label
                        && frame.sent < request.sent
                        && frame.received < call.emitted
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("missing admitted document arguments")?;
            if admitted.value()? != *args || !used.insert(*id) {
                return Err("admitted document arguments altered or reused".into());
            }
            if tool == "revise_section" {
                let labels = if agent == "WRITER_A" {
                    ["RequestReconsiderA", "ReconsiderA"]
                } else {
                    ["RequestReconsiderB", "ReconsiderB"]
                };
                let label = name("labels", labels[0])? as u8;
                let (id, chosen) = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(editor_file, editor)
                            && frame.logical_label == label
                            && frame.received < admitted.sent
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("revision lacks an actual reconsideration request")?;
                let actual_review = ordered
                    .iter()
                    .filter(|review| {
                        review.request["tool"] == "review_sections"
                            && review.request["arguments"]["section_name"] == chapter
                            && review.result.0 < chosen.sent
                    })
                    .max_by_key(|review| review.result.0)
                    .ok_or("reconsideration lacks its actual review")?;
                let (_, actual_review) = actual_review.result;
                if chosen.value()? != json!({"review_id":actual_review["data"]["review_id"]})
                    || actual_review["data"]["input_snapshot"] != snapshot(&ordered, call.emitted)?
                    || !used.insert(*id)
                {
                    return Err("reconsideration reused or altered its current review".into());
                }
                let label = name("labels", labels[1])? as u8;
                let (id, delivered) = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(editor, service)
                            && frame.logical_label == label
                            && frame.sent > chosen.received
                            && frame.received < admitted.sent
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("actual reconsidered review was not delivered")?;
                if delivered.value()? != *actual_review || !used.insert(*id) {
                    return Err("reconsidered review altered or reused".into());
                }
            }
            if matches!(tool, "write_section" | "revise_section") {
                let message = match (agent, tool) {
                    ("WRITER_A", "write_section") => "WriteArgumentsA",
                    ("WRITER_B", "write_section") => "WriteArgumentsB",
                    ("WRITER_A", "revise_section") => "RevisionArgumentsA",
                    ("WRITER_B", "revise_section") => "RevisionArgumentsB",
                    _ => unreachable!(),
                };
                let label = name("labels", message)? as u8;
                let (id, forwarded) = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(service, actor)
                            && frame.logical_label == label
                            && frame.sent > admitted.received
                            && frame.received < request.sent
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("missing actual worker arguments")?;
                if forwarded.value()? != *args || !used.insert(*id) {
                    return Err("worker arguments altered or reused".into());
                }
            } else {
                let label = name("labels", "CombineInput")? as u8;
                let (id, input) = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(editor_file, editor)
                            && frame.logical_label == label
                            && frame.received < admitted.sent
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("missing actual editor file input")?;
                if input.value()? != *args || !used.insert(*id) {
                    return Err("editor file input altered or reused".into());
                }
            }
        }
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
        let reply_at = reply.received;
        let copied = match (agent, tool) {
            ("WRITER_A", "write_section") => Some((actor, file, "WriteResultA")),
            ("WRITER_B", "write_section") => Some((actor, file, "WriteResultB")),
            ("WRITER_A", "revise_section") => Some((actor, file, "RevisionResultA")),
            ("WRITER_B", "revise_section") => Some((actor, file, "RevisionResultB")),
            ("EDITOR", "review_sections") if scope == CHAPTERS[0] => {
                Some((service, editor, "ReviewEvidenceA"))
            }
            ("EDITOR", "review_sections") => Some((service, editor, "ReviewEvidenceB")),
            _ => None,
        };
        if let Some((from, to, message)) = copied {
            let label = name("labels", message)? as u8;
            let (id, copy) = frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(from, to)
                        && frame.logical_label == label
                        && frame.sent
                            > if tool == "review_sections" {
                                result_at
                            } else {
                                reply_at
                            }
                })
                .min_by_key(|(_, frame)| frame.sent)
                .ok_or("missing actual result delivery")?;
            if copy.value()? != *result || !used.insert(*id) {
                return Err("actual result delivery altered or reused".into());
            }
        }
        call.take_score(&mut originals, args)?;
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
                                && review.result.0 < call.emitted
                        })
                        .max_by_key(|review| review.result.0)
                        .ok_or("publication without actual review")?;
                    let (at, value) = review.result;
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
                                && frame.received < call.emitted
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
                            && frame.received < call.emitted
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
        "WriteArgumentsA",
        "WriteArgumentsB",
        "WriteResultA",
        "WriteResultB",
        "WriteOnDocument",
        "Written",
        "ReviseA",
        "ReviseB",
        "RevisionArgumentsA",
        "RevisionArgumentsB",
        "RevisionResultA",
        "RevisionResultB",
        "ReviseOnDocument",
        "Revised",
        "ReferencesA",
        "ReferencesB",
        "ReferencedA",
        "ReferencedB",
        "ReviewA",
        "ReviewB",
        "ReviewEvidenceA",
        "ReviewEvidenceB",
        "ReviewedA",
        "ReviewedB",
        "CombineInput",
        "Combine",
        "CombineForward",
        "CombineOnDocument",
        "Combined",
        "FinalApprovedA",
        "FinalApprovedB",
        "RequestReconsiderA",
        "RequestReconsiderB",
        "ReconsiderA",
        "ReconsiderB",
    ]
    .into_iter()
    .map(|label| name("labels", label).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    // Logical labels are scoped to their communication contract. In particular,
    // engine memory growth uses the same label bytes as revision results.
    // Static wire matching and EPF check the I/O edges independently.
    let business_roles = BTreeSet::from([
        document,
        service,
        editor,
        writer_a,
        name("roles", "WRITER_B")?,
        editor_file,
        name("roles", "A_FILE")?,
        name("roles", "B_FILE")?,
    ]);
    if let Some((id, frame)) = frames.iter().find(|(id, frame)| {
        required.contains(&frame.logical_label)
            && frame.identity["source"]
                .as_u64()
                .is_some_and(|role| business_roles.contains(&role))
            && frame.identity["target"]
                .as_u64()
                .is_some_and(|role| business_roles.contains(&role))
            && !used.contains(id)
    }) {
        return Err(format!(
            "unused document request, receipt or result: frame {id}, identity {}",
            frame.identity
        )
        .into());
    }
    Ok(
        json!({"dynamic_protocol":dynamic,"status":"passed","task":"7E","domain_calls":calls.len(),
        "actual_resource_endpoint_requests":resources,"actual_review_requests":reviews,
        "simulator_complete":true,"violations":0,"matched_effect_frame_ids":used,
        "effect_scope":"one-use actual admitted requests and replies; current chapter/reference material; completed matching review receipts at publication"}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let evidence = crate::journal::Evidence::open(path)?;
    let baseline = evidence.audit("7E")?;
    let (rows, artifact) = (evidence.rows, evidence.projection);
    let dynamic = &baseline["dynamic_protocol"];
    let ids = baseline["matched_effect_frame_ids"]
        .as_array()
        .ok_or("effect frame IDs")?
        .iter()
        .map(|id| id.as_u64().ok_or("effect frame ID"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    fs::create_dir(output)?;
    let mut verdicts = json_frame_mutants(&rows, &ids, |changed| {
        crate::journal::check_effects(changed, &artifact, dynamic)
    })?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(if field == "lane" { 255 } else { 65535 });
            }
        }
        let error = crate::journal::check_effects(&changed, &artifact, dynamic)
            .err()
            .ok_or("coherent invalid wire header accepted")?;
        verdicts.push(json!({"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","rejected":verdicts.len(),"mutants":verdicts,
        "scope":"immutable record mutations of static wire and actual effect correspondence; not runtime attacks or general refinement"});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}
