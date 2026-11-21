//! Join actual 7M effects to the EPF-checked receives that admitted them.
use crate::evidence::Result;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

type Edge = (&'static str, &'static str, &'static str);

pub(crate) fn check(history: crate::journal::Journal<'_>) -> Result<Value> {
    let dynamic = history.execution;
    let artifact = history.projection;
    let journal_end = history.row_count();
    let compiled = &artifact["compiled"];
    let scored = history.scored_events;
    if !history.simulator_complete {
        return Err("incomplete 7M score or resource violation".into());
    }
    let role = |name: &str| crate::projection::named_id(&compiled["roles"], name).map(u64::from);
    let mut used = BTreeSet::new();
    let mut take = |(from, to, label): Edge, range: Range<usize>, value: &Value| -> Result<Value> {
        let (source, target) = (role(from)?, role(to)?);
        let label_id = compiled["labels"][label].as_u64().ok_or("7M message")?;
        let (id, frame) = history
            .frames
            .iter()
            .filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(source, target)
                    && u64::from(frame.logical_label) == label_id
                    && range.contains(&frame.received)
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| format!("missing committed 7M frame {from}/{to}/{label}"))?;
        if &frame.value()? != value {
            return Err(format!("7M frame data differs at {from}/{to}/{label}").into());
        }
        used.insert(*id);
        Ok(json!({"frame":id,"occurrence":history.frames[id].receipt["occurrence"]}))
    };
    let mut ordered = history.calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut scored_calls = BTreeSet::new();
    let mut checked = Vec::new();
    let mut publication = None;
    let mut figures = BTreeMap::new();
    for call in &ordered {
        let who = call.request["agent"].as_str().ok_or("7M effect actor")?;
        let args = &call.request["arguments"];
        let tool = call.tool;
        let (returned, result) = call.result;
        let chapter = args["section_name"].as_str().unwrap_or_default();
        let (file, reader) = match who {
            "WRITER_A" => ("WRITER_FILE", "WRITER_A_SNAPSHOTS"),
            "WRITER_B" => ("WRITER_B_FILE", "WRITER_B_SNAPSHOTS"),
            _ => ("", ""),
        };
        let (request, reply): (Edge, Edge) = match (who, tool, chapter) {
            ("WRITER_A", "write_section", "CHAPTER_1") => {
                ((file, "DOCUMENT", "draft_1"), ("DOCUMENT", file, "draft_1"))
            }
            ("WRITER_A", "write_section", "CHAPTER_2") => {
                ((file, "DOCUMENT", "draft_2"), ("DOCUMENT", file, "draft_2"))
            }
            ("WRITER_B", "write_section", "CHAPTER_3") => {
                ((file, "DOCUMENT", "draft_3"), ("DOCUMENT", file, "draft_3"))
            }
            ("WRITER_A", "update_references", "CHAPTER_1") => (
                (file, "REFERENCES", "references_1"),
                ("REFERENCES", file, "references_1"),
            ),
            ("WRITER_A", "update_references", "CHAPTER_2") => (
                (file, "REFERENCES", "references_2"),
                ("REFERENCES", file, "references_2"),
            ),
            ("WRITER_B", "update_references", "CHAPTER_3") => (
                (file, "REFERENCES", "references_3"),
                ("REFERENCES", file, "references_3"),
            ),
            ("WRITER_A" | "WRITER_B", "read_section", _) => {
                ((file, reader, "read"), (reader, file, "snapshot"))
            }
            ("WRITER_A" | "WRITER_B", "revise_section", _) => {
                ((file, "DOCUMENT", "revise"), ("DOCUMENT", file, "revised"))
            }
            ("FIGURE_CREATOR", "create_figure", _) => {
                let source = role("FIGURE_STORE")?;
                let target = role("FIGURE_FILE")?;
                let mut matched = Vec::new();
                for (chapter, label) in [("CHAPTER_1", "create_1"), ("CHAPTER_3", "create_3")] {
                    let label_id = compiled["labels"][label]
                        .as_u64()
                        .ok_or("7M creation message")?;
                    for frame in history.frames.values().filter(|frame| {
                        frame.direction(source, target)
                            && u64::from(frame.logical_label) == label_id
                            && frame.received > returned
                    }) {
                        if &frame.value()? == result {
                            matched.push((chapter, label));
                        }
                    }
                }
                let [(chapter, label)] = matched.as_slice() else {
                    return Err("7M creation result must match one scoped file response".into());
                };
                if result["success"] != true
                    || figures
                        .insert(*chapter, &result["data"]["figure_id"])
                        .is_some()
                {
                    return Err("7M creation must complete once for that chapter".into());
                }
                (
                    ("FIGURE_FILE", "FIGURE_STORE", label),
                    ("FIGURE_STORE", "FIGURE_FILE", label),
                )
            }
            ("FIGURE_CREATOR", "insert_figure", "CHAPTER_1" | "CHAPTER_3") => {
                let label = if chapter == "CHAPTER_1" {
                    "insert_1"
                } else {
                    "insert_3"
                };
                if figures.get(chapter).copied() != Some(&args["figure_id"]) {
                    return Err("7M insertion does not use that chapter's actual figure".into());
                }
                (
                    ("FIGURE_FILE", "DOCUMENT", label),
                    ("FIGURE_STORE", "FIGURE_FILE", label),
                )
            }
            ("FACT_CHECKER", "fact_check", "CHAPTERS_1_2") => (
                ("FACT_FILE", "FACTS_12", "check_12"),
                ("FACTS_12", "FACT_FILE", "check_12"),
            ),
            ("FACT_CHECKER", "fact_check", "CHAPTER_3") => (
                ("FACT_FILE", "FACTS_3", "check_3"),
                ("FACTS_3", "FACT_FILE", "check_3"),
            ),
            ("EDITOR", "review_sections", "CHAPTER_1") => (
                ("EDITOR_FILE", "EDITOR", "review_1"),
                ("EDITOR", "EDITOR_FILE", "review_1"),
            ),
            ("EDITOR", "review_sections", "CHAPTER_2") => (
                ("EDITOR_FILE", "EDITOR", "review_2"),
                ("EDITOR", "EDITOR_FILE", "review_2"),
            ),
            ("EDITOR", "review_sections", "CHAPTER_3") => (
                ("EDITOR_FILE", "EDITOR", "review_3"),
                ("EDITOR", "EDITOR_FILE", "review_3"),
            ),
            ("EDITOR", "combine_sections", _) => (
                ("EDITOR_FILE", "DOCUMENT_OWNER", "combine"),
                ("DOCUMENT_OWNER", "EDITOR_FILE", "combined"),
            ),
            _ => return Err("unassigned 7M effect".into()),
        };
        let admitted = take(request, 0..call.emitted, args)?;
        let replied = take(reply, returned + 1..journal_end, result)?;
        let authorities = match tool {
            "insert_figure" => vec![
                take(
                    ("DOCUMENT", "FIGURE_STORE", request.2),
                    0..call.emitted,
                    args,
                )?,
                take(
                    ("FIGURE_STORE", "DOCUMENT", request.2),
                    0..call.emitted,
                    args,
                )?,
            ],
            "fact_check" => {
                let source = role("DOCUMENT_OWNER")?;
                let target = role("FACT_FILE")?;
                let label = compiled["labels"]["fact_outcome"]
                    .as_u64()
                    .ok_or("7M fact outcome label")?;
                let (id, frame) = history
                    .frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(source, target)
                            && u64::from(frame.logical_label) == label
                            && frame.received < call.emitted
                    })
                    .max_by_key(|(_, frame)| frame.received)
                    .ok_or("7M fact check lacks its delivered material")?;
                if frame.value()?[chapter] != result["data"]["input_snapshot"] {
                    return Err("7M fact check differs from its delivered material".into());
                }
                vec![json!({"frame": id, "occurrence": history.frames[id].receipt["occurrence"]})]
            }
            "combine_sections" => {
                if publication.replace(*call).is_some() {
                    return Err("7M publication repeated".into());
                }
                vec![
                    take(
                        ("DOCUMENT", "DOCUMENT_OWNER", "combine_document"),
                        0..call.emitted,
                        args,
                    )?,
                    take(
                        ("DOCUMENT_OWNER", "FACT_FILE", "fact_outcome"),
                        returned + 1..journal_end,
                        result,
                    )?,
                    take(
                        ("DOCUMENT_OWNER", "WRITER_FILE", "writer_a_outcome"),
                        returned + 1..journal_end,
                        result,
                    )?,
                    take(
                        ("DOCUMENT_OWNER", "WRITER_B_FILE", "writer_b_outcome"),
                        returned + 1..journal_end,
                        result,
                    )?,
                ]
            }
            _ => Vec::new(),
        };
        let (score_index, _) = scored
            .iter()
            .enumerate()
            .find(|(index, entry)| !scored_calls.contains(index) && call.matches_score(entry, args))
            .ok_or("7M effect differs from the common score")?;
        scored_calls.insert(score_index);
        checked.push(
            json!({"tool":tool,"agent":who,"request":admitted,"reply":replied,
            "authorities":authorities,"score_event":score_index}),
        );
    }

    let publication = publication.ok_or("missing 7M publication")?;
    let (_, result) = publication.result;
    let data = &result["data"];
    let mut prepared = ordered
        .iter()
        .filter_map(|call| {
            let (at, value) = call.result;
            (at < publication.emitted).then_some((at, call.request, value))
        })
        .collect::<Vec<_>>();
    prepared.sort_by_key(|(at, _, _)| *at);
    let mut document = json!({"sections":{},"references":{},"figures":{}});
    for (_, request, value) in &prepared {
        let category = match request["tool"].as_str() {
            Some("write_section" | "revise_section") => "sections",
            Some("update_references") => "references",
            Some("insert_figure") => "figures",
            _ => continue,
        };
        let chapter = request["arguments"]["section_name"]
            .as_str()
            .ok_or("7M material chapter")?;
        if value["success"] != true || !value["data"]["artifact"]["id"].is_string() {
            return Err("7M material is not an actual completed artifact".into());
        }
        document[category][chapter] = value["data"]["artifact"].clone();
    }
    let chapters = ["CHAPTER_1", "CHAPTER_2", "CHAPTER_3"];
    let mut fact_receipts = serde_json::Map::new();
    for (scope, covered) in [
        ("CHAPTERS_1_2", &chapters[..2]),
        ("CHAPTER_3", &chapters[2..]),
    ] {
        let (_, _, actual) = prepared
            .iter()
            .rev()
            .find(|(_, request, _)| {
                request["tool"] == "fact_check" && request["arguments"]["section_name"] == scope
            })
            .ok_or("missing current 7M fact check")?;
        let mut scoped = document.clone();
        for category in ["sections", "references", "figures"] {
            scoped[category]
                .as_object_mut()
                .unwrap()
                .retain(|chapter, _| covered.contains(&chapter.as_str()));
        }
        if actual["success"] != true || actual["data"]["input_snapshot"] != scoped {
            return Err("7M publication uses stale or rejected fact evidence".into());
        }
        fact_receipts.insert(scope.to_owned(), actual["data"]["check_id"].clone());
    }
    let fact_receipts = Value::Object(fact_receipts);
    let mut review_receipts = serde_json::Map::new();
    for chapter in chapters {
        let (_, _, actual) = prepared
            .iter()
            .rev()
            .find(|(_, request, _)| {
                request["tool"] == "review_sections"
                    && request["arguments"]["section_name"] == chapter
            })
            .ok_or("missing current 7M review")?;
        if actual["success"] != true
            || actual["data"]["input_snapshot"] != document
            || actual["data"]["fact_receipts"] != fact_receipts
        {
            return Err("7M publication uses stale or rejected reviews".into());
        }
        review_receipts.insert(chapter.to_owned(), actual["data"]["review_id"].clone());
    }
    let sections = publication.request["arguments"]["sections"]
        .as_str()
        .ok_or("7M publication sections")?
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    if result["success"] != true
        || data["input_snapshot"] != document
        || data["fact_receipts"] != fact_receipts
        || data["review_receipts"] != Value::Object(review_receipts)
        || data["sections"] != json!(sections)
    {
        return Err("7M publication differs from its actual current evidence".into());
    }
    Ok(
        json!({"status":"passed","task":"7M","simulator_complete":true,"violations":0,
        "dynamic_protocol":dynamic,"effects":checked}),
    )
}
