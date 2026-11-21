//! Bind each survey effect to its committed file operation and chapter evidence.
use crate::{evidence::Result, journal, projection};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ops::Range};

const CHAPTERS: [(&str, &str, &str, &str, &str); 4] = [
    (
        "section_a",
        "subtopic_a",
        "RESEARCHER_A",
        "AUTHOR_FILE_A",
        "SECTION_A",
    ),
    (
        "section_b",
        "subtopic_b",
        "RESEARCHER_B",
        "AUTHOR_FILE_B",
        "SECTION_B",
    ),
    (
        "section_c",
        "subtopic_c",
        "RESEARCHER_C",
        "AUTHOR_FILE_C",
        "SECTION_C",
    ),
    (
        "section_d",
        "subtopic_d",
        "RESEARCHER_D",
        "AUTHOR_FILE_D",
        "SECTION_D",
    ),
];

pub(crate) fn check(history: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = history.execution;
    let artifact = history.projection;
    let rows = history.records();
    let compiled = &artifact["compiled"];
    let actors = history.participants;

    let role = |name: &str| crate::projection::named_id(&compiled["roles"], name).map(u64::from);
    let contracts = projection::contracts(compiled)?;
    let scores = history.scored_events;
    if !history.simulator_complete {
        return Err("incomplete survey score or world violation".into());
    }
    journal::check_close_pairs(
        &history.frames,
        &[
            (role("AUTHOR_VM_A")?, role("AUTHOR_IO_A")?),
            (role("AUTHOR_VM_B")?, role("AUTHOR_IO_B")?),
            (role("AUTHOR_VM_C")?, role("AUTHOR_IO_C")?),
            (role("AUTHOR_VM_D")?, role("AUTHOR_IO_D")?),
            (role("ANALYST_VM")?, role("ANALYST_IO")?),
            (role("REVIEW_VM")?, role("REVIEW_IO")?),
            (role("EDITOR_VM")?, role("EDITOR_IO")?),
        ],
    )?;

    let mut used = BTreeSet::new();
    let mut take = |from: &str,
                    to: &str,
                    label: u8,
                    within: Range<usize>,
                    owned: Option<u64>|
     -> Result<(u64, &journal::Frame)> {
        let from_role = role(from)?;
        let to_role = role(to)?;
        let (id, frame) = history
            .frames
            .iter()
            .filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(from_role, to_role)
                    && frame.logical_label == label
                    && within.contains(&frame.received)
                    && owned.is_none_or(|resource| {
                        frame.identity["frame_label"]
                            .as_u64()
                            .and_then(|v| u16::try_from(v).ok())
                            .and_then(|wire| contracts.get(&wire))
                            .is_some_and(|meta| meta[4] == 1 && u64::from(meta[5]) == resource)
                    })
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| {
                format!("missing survey receive {from} -> {to} label {label} in {within:?}")
            })?;
        used.insert(*id);
        Ok((*id, frame))
    };

    let metadata: Value = serde_json::from_str(include_str!(
        "../../../benchmark/descriptions/3H/metadata.json"
    ))?;
    let mut calls: Vec<_> = history.calls.iter().collect();
    calls.sort_by_key(|(_, call)| call.emitted);
    let mut used_scores = BTreeSet::new();
    let mut effects = Vec::new();
    let mut intervals = Vec::new();
    for (&id, call) in calls.iter().copied() {
        let agent = call.request["agent"]
            .as_str()
            .ok_or("survey effect actor")?;
        let args = &call.request["arguments"];
        let tool = call.tool;
        let (start, started) = call.started;
        let (_, ended) = call.ended;
        let (result_at, output) = call.result;
        let success = output["success"].as_bool().ok_or("survey effect verdict")?;
        let before = call.emitted.min(start);
        let score = journal::one(scores.iter().enumerate().filter(|(index, event)| {
            !used_scores.contains(index) && call.matches_score(event, args)
        }))?;
        used_scores.insert(score.0);
        intervals.push((
            crate::oracle::resources_for_call(&metadata, agent, call.request)?,
            started,
            ended,
        ));

        let mut authorities = Vec::new();
        let (request, reply) = if agent == "EDITOR_IN_CHIEF" {
            let (request_label, reply_label) = match tool {
                "make_editorial_decision" => (11, 12),
                "combine_sections" => (13, 14),
                _ => return Err("unknown survey editorial effect".into()),
            };
            let request = take(
                "EDITOR_FILE",
                "EDITOR_IN_CHIEF",
                request_label,
                0..before,
                None,
            )?;
            let admitted = request.1.received;
            if tool == "make_editorial_decision" {
                let input = take("EDITOR_IN_CHIEF", "EDITOR_FILE", 10, 0..admitted, None)?.1;
                let displayed = input.value()?;
                if !success
                    || displayed["input_snapshot"] != output["data"]["input_snapshot"]
                    || displayed["review_receipts"] != output["data"]["review_receipts"]
                    || output["data"]["review_summary"] != args["review_summary"]
                {
                    return Err("editorial decision differs from the approved material".into());
                }
                for (chapter, _, _, _, section) in CHAPTERS {
                    let logical = role(section)?;
                    let label = u8::try_from(logical)?;
                    let review =
                        take(section, "EDITOR_IN_CHIEF", label, 0..input.received, None)?.1;
                    let review_value = review.value()?;
                    let review_at = review.received;
                    let actual = journal::one(calls.iter().filter_map(|(_, call)| {
                        let (at, result) = call.result;
                        (at < review_at
                            && result["tool_name"] == "peer_review"
                            && result["success"] == true
                            && result["data"]["scope"] == chapter
                            && *result == review_value)
                            .then_some(result)
                    }))?;
                    if actual["data"]["review_id"] != displayed["review_receipts"][chapter] {
                        return Err("editorial review receipt names another occurrence".into());
                    }
                    for category in ["sections", "references", "figures"] {
                        if actual["data"]["input_snapshot"][category][chapter]
                            != displayed["input_snapshot"][category][chapter]
                        {
                            return Err("editorial input substitutes reviewed content".into());
                        }
                    }
                    authorities.push(
                        take(
                            "REVIEWER",
                            "EDITOR_IN_CHIEF",
                            label,
                            review.received + 1..input.received,
                            Some(logical),
                        )?
                        .0,
                    );
                }
            } else {
                let (_, decision) = journal::one(calls.iter().filter_map(|(_, call)| {
                    let (at, result) = call.result;
                    (at < admitted && result["tool_name"] == "make_editorial_decision")
                        .then_some((at, result))
                }))?;
                if !success
                    || decision["success"] != true
                    || output["data"]["editorial_receipts"]["finalize"]
                        != decision["data"]["decision_id"]
                    || output["data"]["input_snapshot"] != decision["data"]["input_snapshot"]
                    || output["data"]["review_receipts"] != decision["data"]["review_receipts"]
                {
                    return Err("combination uses a different editorial decision or content".into());
                }
            }
            (
                request,
                take(
                    "EDITOR_IN_CHIEF",
                    "EDITOR_FILE",
                    reply_label,
                    result_at + 1..rows.len(),
                    None,
                )?,
            )
        } else {
            let figure_label = if tool == "create_figure" {
                let start = calls
                    .iter()
                    .filter_map(|(_, other)| {
                        let (at, result) = other.result;
                        (at < before && result["tool_name"] == tool).then_some(at)
                    })
                    .max()
                    .map_or(0, |at| at + 1);
                let from = role("ANALYST_FILE")?;
                let to = role("FIGURES")?;
                Some(
                    journal::one(history.frames.values().filter(|frame| {
                        frame.direction(from, to) && (start..before).contains(&frame.received)
                    }))?
                    .logical_label,
                )
            } else {
                None
            };
            let chapter = CHAPTERS
                .into_iter()
                .find(|(section, topic, author, _, resource)| match tool {
                    "research_topic" => agent == *author && args["topic"] == *topic,
                    "create_figure" => {
                        figure_label.map(u64::from) == compiled["roles"][resource].as_u64()
                    }
                    "peer_review" => args["manuscript"] == *section,
                    _ => args["section_name"] == *section,
                })
                .ok_or("survey effect names no chapter")?;
            let (chapter, topic, author, author_file, section) = chapter;
            let logical = role(section)?;
            let label = u8::try_from(logical)?;
            let (file, owner, request_label, reply_label) = match (agent, tool) {
                (who, "research_topic") if who == author => (author_file, section, 30, 31),
                (who, "write_section") if who == author => (author_file, section, 32, 33),
                (who, "update_references") if who == author => (author_file, "DATABASE", label, 35),
                (who, "revise_section") if who == author => (author_file, section, 27, 28),
                ("DATA_ANALYST", "create_figure") => ("ANALYST_FILE", "FIGURES", label, 38),
                ("DATA_ANALYST", "insert_figure") => ("ANALYST_FILE", section, 41, 42),
                ("REVIEWER", "peer_review") => ("REVIEW_FILE", section, 22, 23),
                _ => return Err("unexpected survey effect".into()),
            };
            let request = take(file, owner, request_label, 0..before, None)?;
            let admitted = request.1.received;
            match tool {
                "update_references" | "create_figure" => {
                    let source = take(
                        section,
                        owner,
                        if tool == "update_references" { 34 } else { 37 },
                        admitted + 1..before,
                        None,
                    )?
                    .1
                    .value()?;
                    let research = journal::one(calls.iter().filter_map(|(_, call)| {
                        let (at, result) = call.result;
                        (at < admitted
                            && result["tool_name"] == "research_topic"
                            && result["data"]["agent"] == author)
                            .then_some(result)
                    }))?;
                    let actual = &output["data"]["artifact"]["research_inputs"];
                    let actual = if tool == "update_references" {
                        actual
                    } else {
                        &actual[chapter]
                    };
                    if source != *research || actual[topic] != research["data"]["artifact"] {
                        return Err("artifact cites different research".into());
                    }
                }
                "insert_figure" => {
                    let input = take(section, "ANALYST_FILE", 44, 0..admitted, None)?.1;
                    if input.value()?["figures"][chapter] != output["data"]["artifact"] {
                        return Err("inserted figure differs from its admitted input".into());
                    }
                    authorities.push(
                        take(
                            author,
                            "DATA_ANALYST",
                            label,
                            0..input.received,
                            Some(logical),
                        )?
                        .0,
                    );
                }
                "peer_review" => {
                    let input = take(section, "REVIEW_FILE", 21, 0..admitted, None)?.1;
                    let draft = take(section, "REVIEWER", 20, 0..input.received, None)?.1;
                    let expected = &output["data"]["input_snapshot"];
                    if input.value()? != *expected
                        || draft.value()? != *expected
                        || output["data"]["scope"] != chapter
                    {
                        return Err("peer review differs from the admitted draft".into());
                    }
                    let interval = draft.received + 1..input.received;
                    let analyst = role("DATA_ANALYST")?;
                    let reviewer = role("REVIEWER")?;
                    let initial = history.frames.values().any(|f| {
                        f.direction(analyst, reviewer)
                            && f.logical_label == label
                            && interval.contains(&f.received)
                    });
                    let sender = if initial { "DATA_ANALYST" } else { author };
                    authorities.push(take(sender, "REVIEWER", label, interval, Some(logical))?.0);
                    let notice = take(author, author_file, 26, result_at + 1..rows.len(), None)?.1;
                    if notice.value()? != *output {
                        return Err("author notice differs from actual review".into());
                    }
                }
                "revise_section" => {
                    let rejection = calls
                        .iter()
                        .filter_map(|(_, call)| {
                            let (at, result) = call.result;
                            (at < admitted
                                && result["tool_name"] == "peer_review"
                                && result["data"]["scope"] == chapter)
                                .then_some((at, result))
                        })
                        .max_by_key(|(at, _)| *at)
                        .ok_or("revision without actual review")?;
                    if rejection.1["success"] != false
                        || output["data"]["artifact"]["content"]
                            == rejection.1["data"]["input_snapshot"]["sections"][chapter]["content"]
                    {
                        return Err("revision fails to change its rejected draft".into());
                    }
                    authorities.push(
                        take(
                            "REVIEWER",
                            author,
                            label,
                            rejection.0 + 1..admitted,
                            Some(logical),
                        )?
                        .0,
                    );
                }
                _ => {}
            }
            (
                request,
                take(owner, file, reply_label, result_at + 1..rows.len(), None)?,
            )
        };
        if request.1.value()? != *args || reply.1.value()? != *output {
            return Err(format!("{tool} differs from its admitted request or file reply").into());
        }
        effects.push(
            json!({"effect_id":id,"tool":tool,"agent":agent,"success":success,
            "request_frame":request.0,"request_occurrence":history.frames[&request.0].receipt["occurrence"],
            "result_frame":reply.0,"result_occurrence":history.frames[&reply.0].receipt["occurrence"],
            "chapter_authorities":authorities,"score_event":score.0}),
        );
    }
    if calls
        .iter()
        .filter(|(_, c)| c.request["tool"] == "combine_sections")
        .count()
        != 1
    {
        return Err("survey requires exactly one publication".into());
    }
    for (index, (resources, start, end)) in intervals.iter().enumerate() {
        for (others, other_start, other_end) in &intervals[..index] {
            if *start < *other_end
                && *other_start < *end
                && resources.iter().any(|r| others.contains(r))
            {
                return Err("survey physical resource effects overlap".into());
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"3H","participants":actors.len(),"domain_calls":effects.len(),
        "violations":0,"effects":effects,"dynamic_protocol":dynamic,
        "simulator_complete":history.simulator_complete,
        "scored_benchmark":false}),
    )
}
