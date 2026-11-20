//! Match 7H effects to the actual file operations and material received by their actors.
use crate::{
    evidence::{Result, digest},
    journal::{self, Journal, one},
    projection,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(crate) fn audit(path: &Path) -> Result<Value> {
    let rows = journal::records(path)?;
    let artifact = projection::load(path)?;
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("missing 7H evidence")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("7H participants")?;
    if fixture["type"] != "fixtures"
        || fixture["task"] != "7H"
        || compiled["task"] != "7H"
        || actors.len() != 7
        || actors
            .iter()
            .filter_map(|actor| actor["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                "WRITER_A",
                "WRITER_B",
                "WRITER_C",
                "FIGURE_CREATOR",
                "FACT_CHECKER",
                "REVIEWER",
                "EDITOR_IN_CHIEF",
            ])
    {
        return Err("7H task or participants differ".into());
    }
    let headers = projection::frames(&artifact, &rows)?;
    let dynamic = projection::check_dynamic(&artifact, &rows)?;
    let mut history = journal::read(&rows, &headers, actors)?;
    let receipts = history.bind_receives(&dynamic)?;
    let mut report = check(compiled, &history, &receipts)?;
    report["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    report["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("missing evidence directory")?
            .join("projection.json")
    )?);
    report["events_sha256"] = json!(digest(path)?);
    report["captured_records"] = dynamic["captured_records"].clone();
    Ok(report)
}

pub(crate) fn check(
    compiled: &Value,
    history: &Journal,
    receipts: &BTreeMap<u64, &Value>,
) -> Result<Value> {
    let role = |name: &str| compiled["roles"][name].as_u64().ok_or("7H projected role");
    let scores = history.score["events"]
        .as_array()
        .ok_or("7H score events")?;
    if scores.len() != history.calls.len()
        || history.score["complete"].as_bool().is_none()
        || history.score["violations"] != json!([])
    {
        return Err("7H score differs from completed domain effects".into());
    }
    let mut ordered: Vec<_> = history.calls.values().collect();
    ordered.sort_by_key(|call| call.emitted);
    let (document, editor, editor_file, editor_vm) = (
        role("DOCUMENT")?,
        role("EDITOR")?,
        role("EDITOR_FILE")?,
        role("EDITOR_VM")?,
    );
    let mut editor_materials = Vec::new();
    for (id, frame) in &history.frames {
        if compiled["labels"]["PublicationMaterial"] != u64::from(frame.logical_label)
            || !(frame.direction(document, editor_file) || frame.direction(editor, editor_file))
        {
            continue;
        }
        let at = frame.received()?;
        let commit = &receipts[id]["receive_commit"];
        let material = frame.value()?;
        let reviews = material["reviews"]
            .as_array()
            .ok_or("7H editorial reviews")?;
        if commit["actor"] != editor_vm || reviews.len() != 3 {
            return Err("7H editorial material lacks the three scope reviews at the editor".into());
        }
        let mut snapshot = json!({"sections":{},"references":{},"figures":{}});
        let mut fact_ids = serde_json::Map::new();
        let mut review_ids = serde_json::Map::new();
        let mut amended_checks = serde_json::Map::new();
        for scope in ["CHAPTERS_1_2", "CHAPTER_3", "CHAPTER_4"] {
            let (review_at, review) = ordered
                .iter()
                .filter(|call| {
                    call.request["tool"] == "review_sections"
                        && call.request["arguments"]["section_name"] == scope
                })
                .filter_map(|call| call.result().ok())
                .filter(|(end, _)| *end < at)
                .max_by_key(|(end, _)| *end)
                .ok_or("7H editor has no current review for its scope")?;
            one(reviews.iter().filter(|received| *received == review))?;
            let data = &review["data"];
            let (fact, scoped) = if data["status"] == "minor_revision" {
                let (_, check) = ordered
                    .iter()
                    .filter(|call| {
                        call.request["tool"] == "fact_check"
                            && call.request["arguments"]["section_name"] == scope
                    })
                    .filter_map(|call| call.result().ok())
                    .filter(|(end, _)| review_at < *end && *end < at)
                    .max_by_key(|(end, _)| *end)
                    .ok_or("7H amended editorial material lacks a new fact check")?;
                if check["success"] != true {
                    return Err("7H editorial material has the wrong amended fact check".into());
                }
                amended_checks.insert(scope.to_owned(), check.clone());
                (&check["data"]["check_id"], &check["data"]["input_snapshot"])
            } else {
                (&data["fact_receipts"][scope], &data["input_snapshot"])
            };
            fact_ids.insert(scope.to_owned(), fact.clone());
            review_ids.insert(scope.to_owned(), data["review_id"].clone());
            for category in ["sections", "references", "figures"] {
                let combined = snapshot[category]
                    .as_object_mut()
                    .ok_or("7H editorial snapshot")?;
                for (chapter, value) in scoped[category]
                    .as_object()
                    .ok_or("7H scoped review snapshot")?
                {
                    if combined.insert(chapter.clone(), value.clone()).is_some() {
                        return Err("7H editorial review scopes overlap".into());
                    }
                }
            }
        }
        if material["input_snapshot"] != snapshot
            || material["fact_receipts"].as_object() != Some(&fact_ids)
            || material["review_receipts"].as_object() != Some(&review_ids)
            || material["checked_minor"].as_object() != Some(&amended_checks)
        {
            return Err(
                "7H editorial material differs from the actual current reviews and facts".into(),
            );
        }
        editor_materials.push((at, material, commit));
    }
    editor_materials.sort_by_key(|(at, _, _)| *at);
    if editor_materials.is_empty() {
        return Err("7H is missing editorial material".into());
    }
    let mut used = BTreeSet::new();
    let mut scored = BTreeSet::new();
    let mut correspondence = Vec::new();
    let mut fact_checks = Vec::new();
    let mut review_verdicts = Vec::new();
    for &call in &ordered {
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let agent = call.request["agent"].as_str().ok_or("7H effect actor")?;
        let (file, actor) = match agent {
            "WRITER_A" => ("WRITER_A_FILE", "WRITER_A_VM"),
            "WRITER_B" => ("WRITER_B_FILE", "WRITER_B_VM"),
            "WRITER_C" => ("WRITER_C_FILE", "WRITER_C_VM"),
            "FIGURE_CREATOR" => ("FIGURE_FILE", "FIGURE_VM"),
            "FACT_CHECKER" => ("FACT_FILE", "FACT_VM"),
            "REVIEWER" => ("REVIEW_FILE", "REVIEW_VM"),
            "EDITOR_IN_CHIEF" => ("EDITOR_FILE", "EDITOR_VM"),
            _ => return Err("unknown 7H effect actor".into()),
        };
        let (returned, result) = call.result()?;
        if tool == "review_sections"
            && result["data"]["review_id"].is_string()
            && result["data"].get("feedback") != args.get("feedback")
        {
            return Err("7H review feedback differs from the submitted assessment".into());
        }
        let section = args["section_name"].as_str().unwrap_or_default();
        let chapter = section.strip_prefix("CHAPTER_").unwrap_or_default();
        let (target, operations) = match (agent, tool, section) {
            ("WRITER_A" | "WRITER_B" | "WRITER_C", "write_section", _) => {
                ("DOCUMENT", vec![format!("Write{chapter}")])
            }
            ("WRITER_A" | "WRITER_B" | "WRITER_C", "update_references", _) => {
                ("REFERENCES", vec![format!("References{chapter}")])
            }
            ("WRITER_A", "revise_section", "CHAPTER_1" | "CHAPTER_2") => (
                "DOCUMENT",
                if args["review_id"].is_string() {
                    vec![format!("Minor{chapter}")]
                } else {
                    vec![format!("Finalize{chapter}"), format!("Reconsider{chapter}")]
                },
            ),
            ("WRITER_B", "revise_section", "CHAPTER_3")
            | ("WRITER_C", "revise_section", "CHAPTER_4") => (
                "DOCUMENT",
                if args["review_id"].is_string() {
                    vec![format!("Minor{chapter}")]
                } else {
                    vec![format!("Reconsider{chapter}")]
                },
            ),
            ("FIGURE_CREATOR", "create_figure", _) => (
                "FIGURE_STORE",
                ["Create1", "Create3", "Create4"]
                    .map(str::to_owned)
                    .to_vec(),
            ),
            ("FIGURE_CREATOR", "insert_figure", _) => {
                ("DOCUMENT", vec![format!("Insert{chapter}")])
            }
            ("FACT_CHECKER", "fact_check", _) | ("REVIEWER", "review_sections", _) => match section
            {
                "CHAPTERS_1_2" => ("FACT_PART_I", vec!["Check12".into()]),
                "CHAPTER_3" => ("FACT_3", vec!["Check3".into()]),
                "CHAPTER_4" => ("FACT_4", vec!["Check4".into()]),
                _ => return Err("unknown 7H assessment scope".into()),
            },
            ("EDITOR_IN_CHIEF", "make_editorial_decision", _) => {
                ("DOCUMENT", vec!["Editorial".into()])
            }
            ("EDITOR_IN_CHIEF", "combine_sections", _) => ("DOCUMENT", vec!["Combine".into()]),
            _ => return Err("effect is outside the 7H file contracts".into()),
        };
        let (from, primary) = (role(file)?, role(target)?);
        let reassessment = (target == "FACT_PART_I")
            .then(|| role("FACT_EDITORIAL"))
            .transpose()?;
        let (&request_id, request) = one(history.frames.iter().filter(|(id, frame)| {
            !used.contains(*id)
                && (frame.direction(from, primary)
                    || reassessment.is_some_and(|role| frame.direction(from, role)))
                && operations
                    .iter()
                    .any(|label| compiled["labels"][label] == u64::from(frame.logical_label))
                && frame.received.is_some_and(|at| at < call.emitted)
                && frame.value().is_ok_and(|value| value == *args)
        }))
        .map_err(|err| format!("7H request {agent} {tool} {section}: {err}"))?;
        let to = request.identity["target"]
            .as_u64()
            .ok_or("7H request target")?;
        let target = if to == primary {
            target
        } else {
            "FACT_EDITORIAL"
        };
        let request_commit = &receipts[&request_id]["receive_commit"];
        let request_at = request.received()?;
        let operation = operations
            .iter()
            .find(|name| compiled["labels"][name.as_str()] == u64::from(request.logical_label))
            .ok_or("7H effect operation")?;
        let performer = match tool {
            "write_section" | "insert_figure" => &["DRAFTING"][..],
            "create_figure" => &["FIGURE_CREATOR"][..],
            "revise_section" if operation.starts_with("Finalize") => &["DRAFTING"][..],
            "revise_section" if section == "CHAPTER_3" => &["FACT_3"][..],
            "revise_section" if section == "CHAPTER_4" => &["FACT_4"][..],
            "revise_section" if operation.starts_with("Minor") => {
                &["FACT_PART_I", "FACT_EDITORIAL"][..]
            }
            "revise_section" => &["FACT_PART_I", "FACT_EDITORIAL", "DOCUMENT_OWNER"][..],
            "make_editorial_decision" | "combine_sections" => &["EDITOR"][..],
            _ => std::slice::from_ref(&target),
        };
        if !performer
            .iter()
            .any(|name| role(name).is_ok_and(|expected| request_commit["actor"] == expected))
        {
            return Err("7H effect runs outside its receiving actor".into());
        }
        used.insert(request_id);

        let reply_owners = match tool {
            "insert_figure" => &["FIGURE_STORE"][..],
            "review_sections" if section != "CHAPTERS_1_2" && result["success"] == true => {
                &["DOCUMENT"][..]
            }
            "fact_check" if result["success"] == false => &[target, "DOCUMENT"][..],
            "make_editorial_decision" if result["success"] == false => &["DOCUMENT_OWNER"][..],
            "revise_section" if matches!(operation.as_str(), "Minor1" | "Minor2") => {
                &["FACT_PART_I", "FACT_EDITORIAL", "DOCUMENT"][..]
            }
            _ => std::slice::from_ref(&target),
        };
        let reply_value = if operation.starts_with("Finalize") {
            &result["data"]["artifact"]
        } else {
            result
        };
        let next_same_reply = ordered
            .iter()
            .find(|next| {
                next.emitted > call.emitted
                    && next.request["agent"] == agent
                    && next.request["tool"] == tool
                    && next.request["arguments"] == *args
            })
            .map(|next| next.emitted);
        let reply_label = if tool == "review_sections" {
            compiled["labels"]["ReviewInput"]
                .as_u64()
                .ok_or("7H scoped review reply label")?
        } else {
            u64::from(request.logical_label)
        };
        let (&reply_id, reply) = one(history.frames.iter().filter(|(id, frame)| {
            !used.contains(*id)
                && reply_owners
                    .iter()
                    .any(|name| role(name).is_ok_and(|source| frame.direction(source, from)))
                && u64::from(frame.logical_label) == reply_label
                && frame
                    .received
                    .is_some_and(|at| returned < at && next_same_reply.is_none_or(|next| at < next))
                && frame.value().is_ok_and(|value| value == *reply_value)
        }))
        .map_err(|err| format!("7H reply {agent} {tool} {section}: {err}"))?;
        let reply_commit = &receipts[&reply_id]["receive_commit"];
        if reply_commit["actor"] != role(actor)? {
            return Err("7H effect result went to the wrong actor".into());
        }
        used.insert(reply_id);

        let (score, _) = scores
            .iter()
            .enumerate()
            .find(|(index, event)| {
                !scored.contains(index)
                    && event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == *args
                    && event["result"] == result["data"]
                    && event["success"] == result["success"]
            })
            .ok_or("7H domain effect is missing its common score event")?;
        scored.insert(score);

        let mut resource_receives = Vec::new();
        if tool == "insert_figure" {
            let figure = args["figure_id"].as_str().ok_or("7H insertion figure ID")?;
            let (_, creation) = one(ordered
                .iter()
                .filter(|prior| {
                    prior.request["tool"] == "create_figure" && prior.request["agent"] == agent
                })
                .filter_map(|prior| prior.result().ok())
                .filter(|(at, value)| {
                    *at < request_at
                        && value["success"] == true
                        && value["data"]["figure_id"] == figure
                }))?;
            if result["success"] != true
                || result["data"]["artifact"] != creation["data"]["artifact"]
            {
                return Err("7H insertion does not contain the created figure".into());
            }
            let store = role("FIGURE_STORE")?;
            for (source, target, actor) in [
                (to, store, role("FIGURE_CREATOR")?),
                (store, to, role("DRAFTING")?),
            ] {
                let (&id, _) = one(history.frames.iter().filter(|(id, frame)| {
                    !used.contains(*id)
                        && frame.direction(source, target)
                        && frame.logical_label == request.logical_label
                        && frame
                            .received
                            .is_some_and(|at| request_at < at && at < call.emitted)
                        && frame.value().is_ok_and(|value| value == *args)
                }))?;
                let commit = &receipts[&id]["receive_commit"];
                if commit["actor"] != actor {
                    return Err("7H insertion was not received by the resource".into());
                }
                used.insert(id);
                resource_receives.push(commit.clone());
            }
            let (&id, working_reply) = one(history.frames.iter().filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(to, store)
                    && frame.logical_label == request.logical_label
                    && frame.received.is_some_and(|at| returned < at)
                    && frame.value().is_ok_and(|value| value == *result)
            }))?;
            if receipts[&id]["receive_commit"]["actor"] != role("FIGURE_CREATOR")?
                || working_reply.received()? >= reply.received()?
            {
                return Err("7H insertion result did not pass through the figure resource".into());
            }
            used.insert(id);
        }

        if matches!(tool, "fact_check" | "review_sections") {
            let chapters: &[&str] = match section {
                "CHAPTERS_1_2" => &["CHAPTER_1", "CHAPTER_2"],
                "CHAPTER_3" => &["CHAPTER_3"],
                "CHAPTER_4" => &["CHAPTER_4"],
                _ => return Err("unknown 7H material scope".into()),
            };
            let material_label = if tool == "fact_check" {
                "FactInput"
            } else {
                "ReviewInput"
            };
            let snapshot = &result["data"]["input_snapshot"];
            let (&material, input) = history
                .frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(to, from)
                        && compiled["labels"][material_label] == u64::from(frame.logical_label)
                        && (tool != "fact_check"
                            || frame.value().is_ok_and(|v| v["sections"].is_object()))
                        && frame.received.is_some_and(|at| at < request_at)
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("7H assessment has no material receive")?;
            let material_commit = &receipts[&material]["receive_commit"];
            if material_commit["actor"] != role(actor)? {
                return Err("7H assessment material went to the wrong actor".into());
            }
            if tool == "fact_check" {
                if input.value()? != *snapshot {
                    return Err("7H fact check used different material".into());
                }
                for (category, tools) in [
                    ("sections", &["write_section", "revise_section"][..]),
                    ("references", &["update_references"]),
                    ("figures", &["insert_figure"]),
                ] {
                    let items = snapshot[category]
                        .as_object()
                        .ok_or("7H material category")?;
                    let expected = chapters
                        .iter()
                        .copied()
                        .filter(|&chapter| category != "figures" || chapter != "CHAPTER_2");
                    if items.len() != expected.clone().count() {
                        return Err("7H fact snapshot scope differs".into());
                    }
                    for chapter in expected {
                        let latest = ordered
                            .iter()
                            .filter(|prior| {
                                prior.ended.is_some_and(|(at, _)| at < call.emitted)
                                    && prior.request["arguments"]["section_name"] == chapter
                                    && prior.tool().is_ok_and(|name| tools.contains(&name))
                            })
                            .max_by_key(|prior| prior.emitted)
                            .ok_or("7H snapshot lacks a domain effect")?;
                        if items[chapter] != latest.result()?.1["data"]["artifact"] {
                            return Err("7H snapshot has stale domain material".into());
                        }
                    }
                }
                fact_checks.push(
                    json!({"scope":section,"check_id":result["data"]["check_id"],
                    "material_commit":material_commit,"ended_row":call.ended()?.0}),
                );
            } else {
                let checked = input.value()?;
                let facts = one(ordered.iter().filter(|prior| {
                    prior.tool().ok() == Some("fact_check")
                        && prior.result().is_ok_and(|(at, value)| {
                            input.received.is_some_and(|received| at < received)
                                && *value == checked
                        })
                }))?;
                if checked["success"] != true
                    || checked["data"]["section"] != section
                    || checked["data"]["input_snapshot"] != *snapshot
                    || result["data"]["fact_receipts"]
                        != json!({(section):checked["data"]["check_id"]})
                {
                    return Err("7H review lacks the current successful fact receipt".into());
                }
                review_verdicts.push(json!({"scope":section,"decision":args["decision"],
                    "status":result["data"]["status"],"review_id":result["data"]["review_id"],
                    "fact_id":facts.result()?.1["data"]["check_id"],"ended_row":call.ended()?.0,
                    "material_commit":material_commit}));
            }
        }
        if operation.starts_with("Minor") {
            let receipt = one(ordered
                .iter()
                .filter_map(|prior| prior.result().ok())
                .filter(|(at, evidence)| {
                    *at < request_at
                        && evidence["tool_name"] == "review_sections"
                        && evidence["data"]["status"] == "minor_revision"
                        && evidence["data"]["review_id"] == args["review_id"]
                }))?
            .1;
            let prior = &receipt["data"]["input_snapshot"]["sections"][section];
            let amended = &result["data"]["artifact"];
            let mut expected = prior.clone();
            expected["id"] = amended["id"].clone();
            expected["content"] = receipt["data"]["minor_replacements"][section].clone();
            expected["previous_id"] = prior["id"].clone();
            expected["minor_review_id"] = args["review_id"].clone();
            if result["success"] != true
                || !expected["content"].is_string()
                || amended["id"] == prior["id"]
                || *amended != expected
            {
                return Err("7H minor amendment differs from its original review".into());
            }
        }
        if matches!(tool, "make_editorial_decision" | "combine_sections") {
            let (_, material) = history
                .frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(to, from)
                        && compiled["labels"]["PublicationMaterial"]
                            == u64::from(frame.logical_label)
                        && frame.received.is_some_and(|at| at < request_at)
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("7H editor has no admitted current material")?;
            let material = material.value()?;
            for field in ["input_snapshot", "review_receipts"] {
                if result["data"][field] != material[field] {
                    return Err(
                        format!("7H editorial {field} differs from received material").into(),
                    );
                }
            }
            let decision = if tool == "combine_sections" {
                let approved = ordered
                    .iter()
                    .filter_map(|prior| prior.result().ok())
                    .filter(|(at, value)| {
                        *at < request_at && value["tool_name"] == "make_editorial_decision"
                    })
                    .max_by_key(|(at, _)| *at)
                    .ok_or("7H publication lacks an editorial decision")?
                    .1;
                if approved["success"] != true
                    || result["data"]["editorial_receipts"]
                        != json!({"final":approved["data"]["decision_id"]})
                    || result["data"]["fact_receipts"] != material["fact_receipts"]
                {
                    return Err("7H publication uses stale approvals or fact checks".into());
                }
                approved
            } else {
                result
            };
            for field in ["input_snapshot", "review_receipts"] {
                if decision["data"][field] != material[field] {
                    return Err("7H selected an editorial approval of older material".into());
                }
            }
        }
        let revision_trigger = if tool == "revise_section"
            && matches!(
                section,
                "CHAPTER_1" | "CHAPTER_2" | "CHAPTER_3" | "CHAPTER_4"
            )
            && !operation.starts_with("Finalize")
        {
            let (scope, custodians) = match section {
                "CHAPTER_1" | "CHAPTER_2" => {
                    ("CHAPTERS_1_2", &["FACT_PART_I", "FACT_EDITORIAL"][..])
                }
                "CHAPTER_3" => ("CHAPTER_3", &["FACT_3"][..]),
                "CHAPTER_4" => ("CHAPTER_4", &["FACT_4"][..]),
                _ => unreachable!("checked manuscript section"),
            };
            let (at, assessment) = ordered
                .iter()
                .filter(|prior| {
                    (prior.request["arguments"]["section_name"] == scope
                        && matches!(prior.tool().ok(), Some("fact_check" | "review_sections")))
                        || (scope == "CHAPTERS_1_2"
                            && prior.tool().ok() == Some("make_editorial_decision"))
                })
                .filter_map(|prior| prior.result().ok())
                .filter(|(at, _)| *at < request_at)
                .max_by_key(|(at, _)| *at)
                .ok_or("7H revision occurred before any assessment")?;
            let (sources, status) = if operation.starts_with("Minor") {
                (&["DOCUMENT"][..], "minor_revision")
            } else if assessment["tool_name"] == "make_editorial_decision" {
                (&["DOCUMENT"][..], "needs_revision")
            } else {
                (custodians, "needs_revision")
            };
            if assessment["success"] != false || assessment["data"]["status"] != status {
                return Err("7H revision lacks a current assessment requiring that change".into());
            }
            // A Part I minor review carries a single notice listing the exact
            // amendment authorized for each of its chapters.
            let shared_notice = scope == "CHAPTERS_1_2" && operation.starts_with("Minor");
            let (&id, _) = one(history.frames.iter().filter(|(id, frame)| {
                (shared_notice || !used.contains(*id))
                    && sources.iter().any(|source| {
                        role(source).is_ok_and(|source| frame.direction(source, from))
                    })
                    && compiled["labels"]["AuthorOutcome"] == u64::from(frame.logical_label)
                    && frame
                        .received
                        .is_some_and(|received| at < received && received < request_at)
                    && frame.value().is_ok_and(|payload| &payload == assessment)
            }))
            .map_err(|error| {
                format!("7H revision did not receive its rejected assessment: {error}")
            })?;
            let commit = &receipts[&id]["receive_commit"];
            if commit["actor"] != role(actor)? {
                return Err("7H revision notice went to the wrong writer".into());
            }
            if !shared_notice {
                used.insert(id);
            }
            Some(json!({"effect":assessment["tool_name"],"frame":id,"receive_commit":commit}))
        } else {
            None
        };
        correspondence.push(json!({"agent":agent,"tool":tool,"chapter":args["section_name"],
            "operation":operation,"request_frame":request_id,"reply_frame":reply_id,"request_commit":request_commit,
            "resource_receives":resource_receives,"reply_commit":reply_commit,"revision_trigger":revision_trigger}));
    }
    let publications = ordered
        .iter()
        .filter(|call| {
            call.request["tool"] == "combine_sections"
                && call
                    .result()
                    .is_ok_and(|(_, result)| result["success"] == true)
        })
        .count();
    if publications != usize::from(history.score["complete"] == true) {
        return Err("7H successful publications differ from common completion".into());
    }
    Ok(
        json!({"task":"7H","status":"passed","violations":0,"effects":ordered.len(),
        "simulator_complete":history.score["complete"],"correspondence":correspondence,
        "editor_material_commits":editor_materials.iter().map(|(_, _, commit)| *commit).collect::<Vec<_>>(),
        "fact_checks":fact_checks,"review_verdicts":review_verdicts}),
    )
}
