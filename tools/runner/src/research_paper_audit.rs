//! Join the 3M operation files, their committed Endpoint receives and real effects.
use crate::{evidence::Result, journal};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ops::Range};

const CHAPTERS: [(&str, &str, &str, &str, &str); 3] = [
    (
        "section_a",
        "subtopic_a",
        "RESEARCHER_A",
        "AUTHOR_A_FILE",
        "SECTION_A",
    ),
    (
        "section_b",
        "subtopic_b",
        "RESEARCHER_B",
        "AUTHOR_B_FILE",
        "SECTION_B",
    ),
    (
        "section_c",
        "subtopic_c",
        "RESEARCHER_C",
        "AUTHOR_C_FILE",
        "SECTION_C",
    ),
];

pub(crate) fn check(history: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = history.execution;
    let artifact = history.projection;
    let rows = history.records();
    let compiled = &artifact["compiled"];
    let actors = history.participants;
    let role = |name: &str| crate::projection::named_id(&compiled["roles"], name).map(u64::from);
    let label = |table: &Value, name: &str| crate::projection::named_id(table, name).map(u64::from);
    let labels = &compiled["labels"];
    let evaluated = history.scored_events;
    if !history.simulator_complete {
        return Err("incomplete paper score or resource violation".into());
    }
    journal::check_close_pairs(
        &history.frames,
        &[
            (role("AUTHOR_A_VM")?, role("AUTHOR_A_IO")?),
            (role("AUTHOR_B_VM")?, role("AUTHOR_B_IO")?),
            (role("AUTHOR_C_VM")?, role("AUTHOR_C_IO")?),
            (role("FACT_VM")?, role("FACT_IO")?),
            (role("EDITOR_VM")?, role("EDITOR_IO")?),
        ],
    )?;
    let metadata: Value = serde_json::from_str(include_str!(
        "../../../benchmark/descriptions/3M/metadata.json"
    ))?;
    let mut used = BTreeSet::new();
    let mut take = |from: &str,
                    to: &str,
                    message: u64,
                    within: Range<usize>,
                    resource: Option<u64>|
     -> Result<(u64, &journal::Frame)> {
        let from_role = role(from)?;
        let to_role = role(to)?;
        let (id, frame) = history
            .frames
            .iter()
            .filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(from_role, to_role)
                    && u64::from(frame.logical_label) == message
                    && within.contains(&frame.received)
                    && resource.is_none_or(|resource| {
                        frame.delegated_role.map(u64::from) == Some(resource)
                    })
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| {
                format!("missing unused paper receive {from} -> {to} label {message} in {within:?}")
            })?;
        used.insert(*id);
        Ok((*id, frame))
    };
    let mut calls = history.calls.iter().collect::<Vec<_>>();
    calls.sort_by_key(|(_, call)| call.emitted);
    let mut used_scores = BTreeSet::new();
    let mut effects = Vec::new();
    let mut references = json!({});
    let mut facts = json!({});
    let mut reviews = json!({});
    let mut publication = None;
    let mut intervals = Vec::new();
    for (&effect_id, call) in calls {
        let agent = call.request["agent"].as_str().ok_or("paper actor")?;
        let tool = call.tool;
        let arguments = &call.request["arguments"];
        let (start, started) = call.started;
        let (_, ended) = call.ended;
        let (result_at, output) = call.result;
        let success = output["success"].as_bool().ok_or("paper verdict")?;
        let before = call.emitted.min(start);
        let score = journal::one(evaluated.iter().enumerate().filter(|(index, event)| {
            !used_scores.contains(index) && call.matches_score(event, arguments)
        }))?;
        used_scores.insert(score.0);
        intervals.push((
            crate::oracle::resources_for_call(&metadata, agent, call.request)?,
            started,
            ended,
        ));
        let ((request_id, request), (reply_id, reply), authorities) = if tool == "combine_sections"
            && agent == "EDITOR"
        {
            let request = take(
                "EDITOR_FILE",
                "DOCUMENT",
                label(labels, "Combine")?,
                0..before,
                None,
            )?;
            let admitted = request.1.received;
            let mut authorities = Vec::new();
            for (_, _, _, _, section_role) in CHAPTERS {
                authorities.push(
                    take(
                        "ARCHIVE",
                        "DOCUMENT",
                        label(labels, "Section")?,
                        admitted + 1..before,
                        Some(role(section_role)?),
                    )?
                    .0,
                );
            }
            let (_, frozen_references) = take(
                "DATABASE",
                "DOCUMENT",
                label(labels, "FrozenReferences")?,
                admitted + 1..before,
                None,
            )?;
            let (_, frozen_reviews) = take(
                "REVIEW_SERVICE",
                "DOCUMENT",
                label(labels, "FrozenReviews")?,
                0..before,
                None,
            )?;
            if frozen_references.value()? != references
                || frozen_reviews.value()? != json!({"facts":facts,"reviews":reviews})
                || !success
                || publication.replace(effect_id).is_some()
            {
                return Err(
                    "publication differs from actual references, review receipts or result".into(),
                );
            }
            let reply = take(
                "DOCUMENT",
                "EDITOR_FILE",
                label(labels, "Published")?,
                result_at + 1..rows.len(),
                None,
            )?;
            (request, reply, authorities)
        } else {
            let chapter = CHAPTERS
                .iter()
                .position(|(chapter, _, author, _, _)| {
                    if tool == "research_topic" {
                        agent == *author
                    } else {
                        arguments["section_name"] == *chapter
                    }
                })
                .ok_or("paper effect names no chapter")?;
            let (section, topic, author, author_file, section_role) = CHAPTERS[chapter];
            let table = &compiled["chapter_labels"][chapter];
            let (file, owner, request_name, reply_name) = match (agent, tool) {
                (who, "research_topic") if who == author && arguments["topic"] == topic => {
                    (author_file, "RESEARCH_SERVICE", "Research", "Researched")
                }
                (who, "research_topic") if who == author => (
                    author_file,
                    "RESEARCH_SERVICE",
                    "Supplement",
                    "Supplemented",
                ),
                (who, "write_section") if who == author => {
                    (author_file, "DOCUMENT", "Write", "Written")
                }
                (who, "revise_section") if who == author => {
                    (author_file, "DOCUMENT", "Revise", "Revised")
                }
                (who, "update_references") if who == author => {
                    (author_file, "DATABASE", "References", "Referenced")
                }
                ("FACT_CHECKER", "fact_check") => {
                    ("FACT_FILE", "REVIEW_SERVICE", "FactRequest", "FactResult")
                }
                ("EDITOR", "review_sections") => (
                    "EDITOR_FILE",
                    "REVIEW_SERVICE",
                    "ReviewRequest",
                    "ReviewResult",
                ),
                _ => return Err(format!("unexpected paper effect {agent} {tool}").into()),
            };
            let request = take(file, owner, label(table, request_name)?, 0..before, None)?;
            let reply = take(
                owner,
                file,
                label(table, reply_name)?,
                result_at + 1..rows.len(),
                None,
            )?;
            let admitted = request.1.received;
            let authorities = match tool {
                "fact_check" | "review_sections" => {
                    let (input_name, role_name) = if tool == "fact_check" {
                        ("FactInput", "FACT_CHECKER")
                    } else {
                        ("ReviewInput", "EDITOR")
                    };
                    let (_, input) = take(
                        role_name,
                        file,
                        label(table, input_name)?,
                        0..admitted,
                        None,
                    )?;
                    let input = input.value()?;
                    let snapshot = &output["data"]["input_snapshot"];
                    if (tool == "fact_check" && input != *snapshot)
                        || (tool == "review_sections"
                            && (input["snapshot"] != *snapshot || input["facts"] != facts[section]))
                    {
                        return Err("review input differs from the actual reviewed material".into());
                    }
                    let authority = take(
                        role_name,
                        "REVIEW_SERVICE",
                        label(labels, "Section")?,
                        0..admitted,
                        Some(role(section_role)?),
                    )?
                    .0;
                    if tool == "fact_check" {
                        facts[section] = output.clone();
                    } else {
                        reviews[section] = output.clone();
                    }
                    if !success || tool == "review_sections" {
                        let (_, notice) = take(
                            "DOCUMENT",
                            author_file,
                            label(table, "AuthorNotice")?,
                            result_at + 1..rows.len(),
                            None,
                        )?;
                        if notice.value()? != *output {
                            return Err("author notice differs from the actual review".into());
                        }
                    }
                    if tool == "review_sections" && success {
                        let (_, final_notice) = take(
                            "FACT_CHECKER",
                            "FACT_FILE",
                            label(table, "FactFinished")?,
                            result_at + 1..rows.len(),
                            None,
                        )?;
                        if final_notice.value()? != *output {
                            return Err("chapter approval differs from the actual review".into());
                        }
                    }
                    vec![authority]
                }
                "update_references" => {
                    references[section] = output["data"]["artifact"].clone();
                    Vec::new()
                }
                _ => Vec::new(),
            };
            (request, reply, authorities)
        };
        if request.value()? != *arguments || reply.value()? != *output {
            return Err(
                format!("{tool} differs from the admitted request or actual file reply").into(),
            );
        }
        effects.push(
            json!({"effect_id":effect_id,"tool":tool,"agent":agent,"success":success,
            "request_frame":request_id,"request_commit":request.received,
            "request_occurrence":history.frames[&request_id].receipt["occurrence"],"result_frame":reply_id,
            "result_commit":reply.received,"result_occurrence":history.frames[&reply_id].receipt["occurrence"],
            "chapter_receipts":authorities,"score_event":score.0}),
        );
    }
    for (chapter, (_, _, _, file, _)) in CHAPTERS.iter().enumerate() {
        let (_, exit) = take(
            file,
            "RESEARCH_SERVICE",
            label(&compiled["chapter_labels"][chapter], "ResearchDone")?,
            0..rows.len(),
            None,
        )?;
        if exit.bytes != [0; 4] {
            return Err("supplemental research closed with a different guest exit status".into());
        }
    }
    let paper_files = [
        "AUTHOR_A_FILE",
        "AUTHOR_B_FILE",
        "AUTHOR_C_FILE",
        "FACT_FILE",
        "EDITOR_FILE",
    ]
    .map(role)
    .into_iter()
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let services = [
        "RESEARCH_SERVICE",
        "DOCUMENT",
        "DATABASE",
        "REVIEW_SERVICE",
        "FACT_CHECKER",
        "EDITOR",
    ]
    .map(role)
    .into_iter()
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    if publication.is_none()
        || history.frames.iter().any(|(id, frame)| {
            let source = frame.source;
            let target = frame.target;
            ((paper_files.contains(&source) && services.contains(&target))
                || (paper_files.contains(&target) && services.contains(&source)))
                && !used.contains(id)
        })
    {
        return Err("missing publication or unpaired paper file traffic".into());
    }
    let mut physical_overlap = 0;
    for (index, (resources, start, finish)) in intervals.iter().enumerate() {
        for (other, next_start, next_finish) in &intervals[index + 1..] {
            if start < next_finish && next_start < finish {
                if resources.iter().any(|r| other.contains(r)) {
                    return Err("overlapping effects on one physical resource".into());
                }
                physical_overlap += usize::from(!resources.is_empty() && !other.is_empty());
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"3M","participants":actors.len(),"domain_calls":effects.len(),
        "domain_effects":effects,"dynamic_protocol":dynamic,"simulator_complete":true,"violations":0,
        "cross_resource_overlaps":physical_overlap,
        }),
    )
}
