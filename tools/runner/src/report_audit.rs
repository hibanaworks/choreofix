//! Actual report requests, content and results; EPF checks source execution.
//! Effect boundaries use checked Endpoint receives. Queued transport log rows
//! do not establish source ordering or the time at which a request was accepted.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, Frame, one, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn name(compiled: &Value, group: &str, key: &str) -> Result<u64> {
    compiled[group][key]
        .as_u64()
        .filter(|value| *value <= 255)
        .ok_or_else(|| format!("missing compiled {group} {key}").into())
}
fn target<'a>(call: &Call<'a>) -> Result<&'a str> {
    let key = match call.tool {
        "research_topic" => "topic",
        "write_section" | "revise_section" | "update_references" | "review_sections" => {
            "section_name"
        }
        "combine_sections" => "sections",
        _ => return Err("unknown report effect".into()),
    };
    call.request["arguments"][key]
        .as_str()
        .ok_or_else(|| "missing report target".into())
}
fn sections(value: &str) -> Vec<&str> {
    value.split(',').map(str::trim).collect()
}
fn assignment(call: &Call, fixture: &Value) -> Result<()> {
    let tool = call.tool;
    let schema = fixture["tool_schemas"]
        .as_array()
        .ok_or("original report schemas")?
        .iter()
        .map(|item| &item["function"])
        .find(|item| item["name"] == tool)
        .ok_or("unknown original report tool")?;
    if !schema["agent_ids"]
        .as_array()
        .ok_or("original tool participants")?
        .contains(&call.request["agent"])
    {
        return Err("report tool belongs to another participant".into());
    }
    let arguments = call.request["arguments"]
        .as_object()
        .ok_or("report argument object")?;
    let parameters = &schema["parameters"];
    if parameters["required"]
        .as_array()
        .ok_or("required original arguments")?
        .iter()
        .any(|key| key.as_str().is_none_or(|key| !arguments.contains_key(key)))
        || arguments.iter().any(|(key, value)| {
            parameters["properties"][key]["type"] != "string" || !value.is_string()
        })
    {
        return Err("report arguments differ from the recorded schema".into());
    }
    let actor = fixture["fixtures"]
        .as_array()
        .ok_or("report participants")?
        .iter()
        .find(|actor| actor["agent"] == call.request["agent"])
        .ok_or("unknown report participant")?;
    for expected in actor["calls"]
        .as_array()
        .ok_or("original report assignments")?
        .iter()
        .filter(|expected| expected["tool"] == tool)
    {
        if tool == "combine_sections" {
            let mut actual = sections(target(call)?);
            let mut expected = sections(
                expected["arguments"]["sections"]
                    .as_str()
                    .ok_or("original report sections")?,
            );
            actual.sort_unstable();
            expected.sort_unstable();
            if actual == expected {
                return Ok(());
            }
        } else if expected["arguments"]
            .as_object()
            .ok_or("original report argument object")?
            .iter()
            .all(|(key, value)| call.request["arguments"][key] == *value)
        {
            return Ok(());
        }
    }
    Err("report target differs from its original assignment".into())
}

// Evidence consumption, never application authority or protocol progress.
#[track_caller]
fn take<'a>(
    frames: &'a BTreeMap<u64, Frame<'a>>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<&'a Frame<'a>> {
    let caller = std::panic::Location::caller();
    let (id, frame) = one(frames
        .iter()
        .filter(|(id, frame)| !used.contains(*id) && matches(frame)))
    .map_err(|error| {
        format!(
            "{error} at {}:{} ({} consumed frames)",
            caller.file(),
            caller.line(),
            used.len()
        )
    })?;
    if !used.insert(*id) {
        return Err("report frame occurrence reused".into());
    }
    Ok(frame)
}
fn material(calls: &[&Call], before: usize) -> Result<Value> {
    let mut completed = calls
        .iter()
        .copied()
        .filter(|call| call.result.0 < before)
        .collect::<Vec<_>>();
    completed.sort_by_key(|call| call.result.0);
    let mut snapshot = json!({"sections":{},"references":{}});
    for call in completed {
        let category = match call.tool {
            "write_section" | "revise_section" => "sections",
            "update_references" => "references",
            _ => continue,
        };
        let scope = target(call)?;
        let (_, result) = call.result;
        if !matches!(scope, "section_a" | "section_b")
            || result["success"] != true
            || !result["data"]["artifact"]["id"].is_string()
            || !result["data"]["artifact"]["content"].is_string()
        {
            return Err("report material lacks its actual successful artifact".into());
        }
        snapshot[category][scope] = result["data"]["artifact"].clone();
    }
    Ok(snapshot)
}
fn scoped(snapshot: &Value, scope: &str) -> Value {
    json!({"sections":{scope:snapshot["sections"][scope]},"references":{scope:snapshot["references"][scope]}})
}
pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty report evidence")?["message"];
    if fixture["type"] != "fixtures" || fixture["task"] != "3E" || compiled["task"] != "3E" {
        return Err("report task differs".into());
    }
    let actors = crate::journal::participants(
        fixture,
        include_str!("../../../benchmark/descriptions/3E/metadata.json"),
    )?;

    let headers = crate::projection::contracts(compiled)?;
    let labels = crate::projection::frames(artifact, rows)?;
    let journal = crate::journal::read(rows, &labels, actors, dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    let session = compiled["session"].as_u64().ok_or("report session")?;
    if frames
        .values()
        .any(|frame| frame.identity["session"] != session)
        || calls.is_empty()
        || score["complete"] != true
        || score["violations"] != json!([])
    {
        return Err("report session or actual outcome differs".into());
    }
    let role = |key| name(compiled, "roles", key);
    let label = |key| name(compiled, "labels", key).map(|n| n as u8);
    let (a, b, editor, doc, doc_service, db, db_service) = (
        role("RESEARCHER_A")?,
        role("RESEARCHER_B")?,
        role("EDITOR")?,
        role("DOCUMENT")?,
        role("DOCUMENT_SERVICE")?,
        role("DATABASE")?,
        role("DATABASE_SERVICE")?,
    );
    let initial_doc = role("INITIAL_DOCUMENT")?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut used = BTreeSet::new();
    let mut originals = score["events"]
        .as_array()
        .ok_or("report score events")?
        .iter()
        .collect::<Vec<_>>();
    let mut resource_requests = 0;
    let mut review_count = 0;
    let mut publications = 0;
    for call in &ordered {
        assignment(call, fixture)?;
        let agent = call.request["agent"].as_str().ok_or("report actor")?;
        let tool = call.tool;
        let args = &call.request["arguments"];
        let (result_at, result) = call.result;
        let actor = role(agent)?;
        let (file, section) = match agent {
            "RESEARCHER_A" => (role("A_FILE")?, "section_a"),
            "RESEARCHER_B" => (role("B_FILE")?, "section_b"),
            "EDITOR" => (role("EDITOR_FILE")?, ""),
            _ => return Err("unassigned report actor".into()),
        };
        if tool != "review_sections" && result["success"] != true {
            return Err("unexpected report effect failure".into());
        }
        if result["tool_name"] != tool {
            return Err("report result names another effect".into());
        }
        match tool {
            "research_topic" => {
                let research = actor;
                let request_label = label(if agent == "RESEARCHER_A" {
                    "ResearchA"
                } else {
                    "ResearchB"
                })?;
                let research_result_label = label("ResearchResult")?;
                let request = take(&frames, &mut used, |f| {
                    f.direction(file, research)
                        && f.logical_label == request_label
                        && f.received < call.emitted
                        && f.value().is_ok_and(|v| v == *args)
                })?;
                let reply = take(&frames, &mut used, |f| {
                    f.direction(research, file)
                        && f.logical_label == research_result_label
                        && f.received > result_at
                        && f.value().is_ok_and(|v| v == *result)
                })?;
                if request.received >= call.started.0
                    || reply.received <= result_at
                    || !result["data"]["artifact"]["id"].is_string()
                {
                    return Err("research result lacks its actual request and artifact".into());
                }
            }
            "write_section" | "revise_section" | "update_references" | "combine_sections" => {
                let (
                    resource,
                    service,
                    request_name,
                    admitted_name,
                    forward_name,
                    result_name,
                    file_result,
                ) = match (agent, tool) {
                    ("RESEARCHER_A", "write_section") => (
                        doc,
                        initial_doc,
                        "WriteOnDocument",
                        "WriteA",
                        Some("WriteArgumentsA"),
                        "DocumentResult",
                        "WriteResultA",
                    ),
                    ("RESEARCHER_B", "write_section") => (
                        doc,
                        initial_doc,
                        "WriteOnDocument",
                        "WriteB",
                        Some("WriteArgumentsB"),
                        "DocumentResult",
                        "WriteResultB",
                    ),
                    ("RESEARCHER_A", "revise_section") => (
                        doc,
                        doc_service,
                        "ReviseOnDocument",
                        "ReviseA",
                        Some("RevisionArgumentsA"),
                        "DocumentResult",
                        "RevisionResultA",
                    ),
                    ("RESEARCHER_B", "revise_section") => (
                        doc,
                        doc_service,
                        "ReviseOnDocument",
                        "ReviseB",
                        Some("RevisionArgumentsB"),
                        "DocumentResult",
                        "RevisionResultB",
                    ),
                    ("RESEARCHER_A", "update_references") => (
                        db,
                        db_service,
                        "ReferencesOnDatabase",
                        "ReferencesA",
                        Some("ReferenceArgumentsA"),
                        "DatabaseResult",
                        "ReferenceResultA",
                    ),
                    ("RESEARCHER_B", "update_references") => (
                        db,
                        db_service,
                        "ReferencesOnDatabase",
                        "ReferencesB",
                        Some("ReferenceArgumentsB"),
                        "DatabaseResult",
                        "ReferenceResultB",
                    ),
                    ("EDITOR", "combine_sections") => (
                        doc,
                        doc_service,
                        "CombineOnDocument",
                        "Combine",
                        None,
                        "DocumentResult",
                        "CombinedReply",
                    ),
                    _ => return Err("report resource effect belongs to another participant".into()),
                };
                let request_label = label(request_name)?;
                take(&frames, &mut used, |f| {
                    f.direction(resource, service)
                        && f.logical_label == request_label
                        && f.received < call.emitted
                        && f.value().is_ok_and(|v| v == *args)
                })?;
                let admission_label = label(admitted_name)?;
                let admission_target = match tool {
                    "write_section" => name(compiled, "roles", "WRITE_REQUESTS")?,
                    "update_references" => name(compiled, "roles", "REFERENCE_REQUESTS")?,
                    _ => service,
                };
                take(&frames, &mut used, |f| {
                    f.direction(
                        if tool == "combine_sections" {
                            editor
                        } else {
                            file
                        },
                        admission_target,
                    ) && f.logical_label == admission_label
                        && f.received < call.emitted
                        && f.value().is_ok_and(|v| v == *args)
                })?;
                if let Some(forward_name) = forward_name {
                    let forwarded_label = label(forward_name)?;
                    take(&frames, &mut used, |f| {
                        f.direction(service, actor)
                            && f.logical_label == forwarded_label
                            && f.received < call.emitted
                            && f.value().is_ok_and(|v| v == *args)
                    })?;
                } else {
                    take(&frames, &mut used, |f| {
                        f.direction(file, editor)
                            && f.logical_label == admission_label
                            && f.received < call.emitted
                            && f.value().is_ok_and(|v| v == *args)
                    })?;
                }
                let reply_label = label(result_name)?;
                take(&frames, &mut used, |f| {
                    f.direction(service, resource)
                        && f.logical_label == reply_label
                        && f.received > result_at
                        && f.value().is_ok_and(|v| v == *result)
                })?;
                let file_reply_label = label(file_result)?;
                take(&frames, &mut used, |f| {
                    f.direction(actor, file)
                        && f.logical_label == file_reply_label
                        && f.received > result_at
                        && f.value().is_ok_and(|v| v == *result)
                })?;
                resource_requests += 1;
                if tool == "update_references" {
                    let forwarded_label = label(if section == "section_a" {
                        "ReferenceMaterialA"
                    } else {
                        "ReferenceMaterialB"
                    })?;
                    take(&frames, &mut used, |f| {
                        f.direction(db_service, initial_doc)
                            && f.logical_label == forwarded_label
                            && f.received > result_at
                            && f.value().is_ok_and(|v| v == *result)
                    })?;
                }
                if tool == "revise_section" {
                    let prior = ordered
                        .iter()
                        .filter(|p| {
                            p.request["tool"] == "review_sections"
                                && target(p).ok() == Some(section)
                                && p.result.0 < call.emitted
                        })
                        .max_by_key(|p| p.emitted)
                        .ok_or("revision lacks its actual review")?;
                    let (_, value) = prior.result;
                    if value["success"] != false
                        || value["data"]["input_snapshot"]
                            != scoped(&material(&ordered, call.emitted)?, section)
                    {
                        return Err("revision reused a stale or positive review".into());
                    }
                }
                if tool == "combine_sections" {
                    publications += 1;
                    let current = material(&ordered, call.emitted)?;
                    let mut names = sections(target(call)?);
                    names.sort_unstable();
                    if names != ["section_a", "section_b"]
                        || result["data"]["input_snapshot"] != current
                        || !result["data"]["artifact_id"].is_string()
                    {
                        return Err("published report differs from current material".into());
                    }
                    for (section, service, confirmation, notice, author) in [
                        (
                            "section_a",
                            role("RESEARCH_A")?,
                            "ConfirmApprovedA",
                            "CombinedA",
                            a,
                        ),
                        (
                            "section_b",
                            role("RESEARCH_B")?,
                            "ConfirmApprovedB",
                            "CombinedB",
                            b,
                        ),
                    ] {
                        let prior = ordered
                            .iter()
                            .filter(|p| {
                                p.request["tool"] == "review_sections"
                                    && target(p).ok() == Some(section)
                                    && p.result.0 < call.emitted
                            })
                            .max_by_key(|p| p.emitted)
                            .ok_or("publication lacks a current review")?;
                        let (at, value) = prior.result;
                        if value["success"] != true
                            || value["data"]["input_snapshot"] != scoped(&current, section)
                            || result["data"]["review_receipts"][section]
                                != value["data"]["review_id"]
                        {
                            return Err(
                                "publication reused stale or negative review material".into()
                            );
                        }
                        let confirmation = label(confirmation)?;
                        take(&frames, &mut used, |f| {
                            f.direction(editor, service)
                                && f.logical_label == confirmation
                                && f.received > at
                                && f.received < call.emitted
                                && f.value().is_ok_and(|v| v == *value)
                        })?;
                        take(&frames, &mut used, |f| {
                            f.direction(service, doc_service)
                                && f.logical_label == confirmation
                                && f.received > at
                                && f.received < call.emitted
                                && f.value().is_ok_and(|v| v == *value)
                        })?;
                        let notice = label(notice)?;
                        take(&frames, &mut used, |f| {
                            f.direction(editor, author)
                                && f.logical_label == notice
                                && f.received > result_at
                                && f.value().is_ok_and(|v| v == *result)
                        })?;
                    }
                } else {
                    if result["data"]["section"] != section || result["data"]["agent"] != agent {
                        return Err("report material belongs to another section".into());
                    }
                    let research = ordered
                        .iter()
                        .find(|p| {
                            p.request["tool"] == "research_topic" && p.request["agent"] == agent
                        })
                        .ok_or("material lacks actual research")?;
                    if research.result.0 >= call.emitted
                        || result["data"]["artifact"]["research_inputs"][target(research)?]
                            != research.result.1["data"]["artifact"]
                    {
                        return Err("material did not consume its actual research".into());
                    }
                }
            }
            "review_sections" => {
                review_count += 1;
                let section = target(call)?;
                let (service, outcome, own, other, own_file, feedback, result_name) = match section
                {
                    "section_a" => (
                        role("RESEARCH_A")?,
                        role("REVIEW_A")?,
                        a,
                        b,
                        role("A_FILE")?,
                        "FeedbackA",
                        "ReviewResultA",
                    ),
                    "section_b" => (
                        role("RESEARCH_B")?,
                        role("REVIEW_B")?,
                        b,
                        a,
                        role("B_FILE")?,
                        "FeedbackB",
                        "ReviewResultB",
                    ),
                    _ => return Err("review has another report scope".into()),
                };
                let request_label = label(if section == "section_a" {
                    "ReviewA"
                } else {
                    "ReviewB"
                })?;
                for (from, to) in [(file, editor), (editor, own), (own, service)] {
                    take(&frames, &mut used, |f| {
                        f.direction(from, to)
                            && f.logical_label == request_label
                            && f.received < call.emitted
                            && f.value().is_ok_and(|v| v == *args)
                    })?;
                }
                // The author admits the review of its own prepared material.
                take(&frames, &mut used, |f| {
                    f.direction(editor, doc_service)
                        && f.logical_label == request_label
                        && f.sent < call.emitted
                        && f.value().is_ok_and(|v| v == *args)
                })?;
                let verdict = result["success"]
                    .as_bool()
                    .ok_or("review lacks an actual verdict")?;
                if result["data"]["agent"] != "EDITOR"
                    || result["data"]["scope"] != section
                    || !result["data"]["review_id"].is_string()
                    || result["data"]["input_snapshot"]
                        != scoped(&material(&ordered, call.emitted)?, section)
                {
                    return Err("review does not refer to its current actual section".into());
                }
                let response_name = match (section, verdict) {
                    ("section_a", true) => "ApprovedA",
                    ("section_a", false) => "RejectedA",
                    (_, true) => "ApprovedB",
                    (_, false) => "RejectedB",
                };
                let reply_label = label(response_name)?;
                if review_count == 1 {
                    let forwarded = take(&frames, &mut used, |f| {
                        f.direction(doc_service, initial_doc)
                            && f.logical_label == reply_label
                            && f.received > result_at
                            && f.value().is_ok_and(|v| v == *result)
                    })?;
                    let handoff_label = label("ReportMaterial")?;
                    let handoff = take(&frames, &mut used, |f| {
                        f.direction(initial_doc, doc_service)
                            && f.logical_label == handoff_label
                            && f.sent > forwarded.received
                    })?;
                    let actual = material(&ordered, handoff.sent)?;
                    if ["section_a", "section_b"].into_iter().any(|key| {
                        actual["sections"][key].is_null() || actual["references"][key].is_null()
                    }) || handoff.value()? != actual
                    {
                        return Err(
                            "document transfer differs from the actual initial material".into()
                        );
                    }
                }
                for recipient in [doc_service, own, other, editor] {
                    take(&frames, &mut used, |f| {
                        f.direction(outcome, recipient)
                            && f.logical_label == reply_label
                            && f.received > result_at
                            && f.value().is_ok_and(|v| v == *result)
                    })?;
                    if recipient == own || recipient == other {
                        take(&frames, &mut used, |f| {
                            f.direction(recipient, outcome)
                                && f.logical_label == reply_label
                                && f.received > result_at
                                && f.value().is_ok_and(|v| v == *result)
                        })?;
                    }
                    if recipient == own {
                        let feedback = label(feedback)?;
                        take(&frames, &mut used, |f| {
                            f.direction(own, own_file)
                                && f.logical_label == feedback
                                && f.received > result_at
                                && f.value().is_ok_and(|v| v == *result)
                        })?;
                    }
                    if recipient == editor {
                        let file_result = label(result_name)?;
                        take(&frames, &mut used, |f| {
                            f.direction(editor, file)
                                && f.logical_label == file_result
                                && f.received > result_at
                                && f.value().is_ok_and(|v| v == *result)
                        })?;
                    }
                }
            }
            _ => return Err("unexpected report operation".into()),
        }
        call.take_score(&mut originals, args)?;
    }
    if !originals.is_empty() || publications != 1 {
        return Err("unmatched report effect or publication".into());
    }
    let business = [
        a,
        b,
        editor,
        doc,
        doc_service,
        initial_doc,
        role("WRITE_REQUESTS")?,
        role("REFERENCE_REQUESTS")?,
        db,
        db_service,
        role("A_FILE")?,
        role("B_FILE")?,
        role("EDITOR_FILE")?,
        role("RESEARCH_A")?,
        role("RESEARCH_B")?,
        role("REVIEW_A")?,
        role("REVIEW_B")?,
    ];
    for (id, frame) in &frames {
        let wire = frame.identity["frame_label"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .ok_or("report wire identity")?;
        let header = headers.get(&wire).ok_or("report compiled header")?;
        if header[4] == 0
            && business.contains(&u64::from(header[1]))
            && business.contains(&u64::from(header[2]))
            && !used.contains(id)
        {
            return Err("unused or extra report request or result".into());
        }
    }
    let mut overlaps = BTreeMap::<String, usize>::new();
    for (index, first) in ordered.iter().enumerate() {
        for second in ordered.iter().skip(index + 1) {
            if first.started.1.max(second.started.1) < first.ended.1.min(second.ended.1) {
                let mut pair = [first.tool, second.tool];
                pair.sort_unstable();
                *overlaps.entry(pair.join("+")).or_default() += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"3E","kind":"direct_file_actual_wasi_report_trace","scored_benchmark":false,"domain_calls":calls.len(),"actual_resource_endpoint_requests":resource_requests,"actual_review_requests":review_count,"forwarded_approvals":2,"guest_exits":actors.len(),"simulator_complete":true,"violations":0,"overlapping_effect_pairs":overlaps,"dynamic_protocol":dynamic}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let rows = records(path)?;
    let (projection, _) = crate::projection::load(path)?;
    let dynamic = crate::projection::check_dynamic(&projection, &rows)?;
    let mut audit = check(&rows, &projection, &dynamic)?;
    audit["source_events_sha256"] = json!(digest(path)?);
    audit["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("report directory")?
            .join("projection.json")
    )?);
    audit["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("report actors")?;
    let labels = crate::projection::frames(&projection, &rows)?;
    let journal = crate::journal::read(&rows, &labels, actors, &dynamic)?;
    let ids = journal
        .frames
        .iter()
        .filter(|(_, frame)| frame.value().is_ok())
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    save(&output.join("audit.json"), &audit)?;
    // These mutations change only frame rows. Producer captures are unchanged;
    // bind_receives still requires every changed journal to match the checked
    // receive occurrences. Real audits always check their own capture above.
    let mut rejected = crate::journal::json_frame_mutants(&rows, &ids, |changed| {
        check(changed, &projection, &dynamic)
    })?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(if field == "lane" { 255 } else { 65535 });
            }
        }
        let error = check(&changed, &projection, &dynamic)
            .err()
            .ok_or("coherent invalid report wire header accepted")?;
        rejected.push(json!({"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let call = journal
        .calls
        .values()
        .filter(|call| call.tool == "research_topic")
        .min_by_key(|call| call.emitted)
        .ok_or("missing actual research for effect-boundary counterexamples")?;
    let file = name(
        &projection["compiled"],
        "roles",
        if call.request["agent"] == "RESEARCHER_A" {
            "A_FILE"
        } else {
            "B_FILE"
        },
    )?;
    let request = one(journal.frames.values().filter(|frame| {
        frame.identity["source"] == file
            && frame
                .value()
                .is_ok_and(|value| value == call.request["arguments"])
    }))?;
    let (result_at, result) = call.result;
    let reply = one(journal.frames.values().filter(|frame| {
        frame.identity["target"] == file && frame.value().is_ok_and(|value| value == *result)
    }))?;
    for (kind, from, to) in [
        (
            "effect_before_actual_receive",
            call.emitted,
            rows[..request.received]
                .iter()
                .rposition(|row| {
                    row["direction"] == "runtime" && row["message"]["type"] == "terminal"
                })
                .ok_or("missing earlier terminal output for effect-boundary counterexample")?,
        ),
        (
            "reply_before_actual_result",
            result_at,
            rows.iter()
                .enumerate()
                .skip(reply.received + 1)
                .find(|(_, row)| {
                    row["direction"] == "to_runtime" && row["message"]["type"] == "command"
                })
                .map(|(index, _)| index)
                .ok_or("missing later command for effect-boundary counterexample")?,
        ),
    ] {
        let mut changed = rows.clone();
        changed.swap(from, to);
        // Capture rows and their positions are unchanged. The journal must
        // remain internally valid; rejection must come from the actual
        // request/effect/result boundary, not malformed effect intervals.
        crate::journal::read(&changed, &labels, actors, &dynamic)?;
        let error = check(&changed, &projection, &dynamic)
            .err()
            .ok_or_else(|| format!("accepted {kind}"))?;
        if !error.to_string().starts_with("missing matching frame") {
            return Err(format!("{kind} did not reach the effect-boundary check: {error}").into());
        }
        rejected.push(json!({"kind":kind,"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","task":"3E","record_mutations_rejected":rejected.len(),"mutations":rejected});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
