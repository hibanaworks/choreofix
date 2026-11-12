//! Saved 3E effects and resource exchanges; this never drives an Endpoint.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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
fn target(call: &Call) -> Result<&str> {
    let key = match call.tool()? {
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
fn score_arguments(call: &Call) -> Result<Value> {
    Ok(match call.tool()? {
        "research_topic" => json!({"topic":target(call)?}),
        "write_section" | "revise_section" => json!({"section":target(call)?}),
        "update_references" | "review_sections" => json!({"scope":target(call)?}),
        "combine_sections" => json!({"sections":sections(target(call)?)}),
        _ => return Err("unknown report score operation".into()),
    })
}

fn assignment(call: &Call, fixture: &Value) -> Result<()> {
    let tool = call.tool()?;
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

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty report evidence")?["message"];
    if compiled["task"] != "3E"
        || fixture["type"] != "fixtures"
        || fixture["task"] != "3E"
        || fixture["depth"].as_u64().is_none_or(|depth| depth > 2)
    {
        return Err("report task or failure condition".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("report participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|actor| actor["agent"].as_str().unwrap_or(""))
            .collect::<BTreeSet<_>>()
            != BTreeSet::from(["RESEARCHER_A", "RESEARCHER_B", "EDITOR"])
    {
        return Err("original report participants differ".into());
    }
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    let session = compiled["session"]
        .as_u64()
        .ok_or("compiled report session")?;
    if frames
        .values()
        .any(|frame| frame.identity["session"] != session)
    {
        return Err("report session differs".into());
    }
    let a = name(compiled, "roles", "RESEARCHER_A")?;
    let b = name(compiled, "roles", "RESEARCHER_B")?;
    let editor = name(compiled, "roles", "EDITOR")?;
    let a_io = name(compiled, "roles", "A_IO")?;
    let b_io = name(compiled, "roles", "B_IO")?;
    let editor_io = name(compiled, "roles", "EDITOR_IO")?;
    let review_a = name(compiled, "roles", "REVIEW_A")?;
    let review_b = name(compiled, "roles", "REVIEW_B")?;
    let document = name(compiled, "roles", "DOCUMENT")?;
    let document_service = name(compiled, "roles", "DOCUMENT_SERVICE")?;
    let database = name(compiled, "roles", "DATABASE")?;
    let database_service = name(compiled, "roles", "DATABASE_SERVICE")?;
    let label = |key| name(compiled, "labels", key).map(|value| value as u8);
    check_close_pairs(&frames, &[(a, a_io), (b, b_io), (editor, editor_io)])?;

    let depth = fixture["depth"].as_u64().ok_or("report depth")? as usize;
    let mut failures = BTreeSet::new();
    for item in fixture["failure_overrides"]
        .as_array()
        .ok_or("report failure overrides")?
    {
        if item[0] != "review_sections"
            || !failures.insert(usize::try_from(
                item[1].as_u64().ok_or("review failure index")?,
            )?)
        {
            return Err("invalid report failure override".into());
        }
    }
    let rejected = depth + failures.len();
    if failures.iter().copied().ne(depth..rejected) {
        return Err("report overrides must extend the original rejection prefix".into());
    }
    if score["complete"] != true
        || score["violations"] != json!([])
        || calls.len() != 9 + 2 * rejected
    {
        return Err("report completion, violations or effect count".into());
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("original report events")?
        .clone();
    for call in calls.values() {
        assignment(call, fixture)?;
        let (_, result) = call.result()?;
        if call.tool()? != "review_sections" && result["success"] != true {
            return Err("unexpected report operation failure".into());
        }
        let args = score_arguments(call)?;
        let index = scored
            .iter()
            .position(|event| {
                event["agent"] == call.request["agent"]
                    && event["tool"] == call.request["tool"]
                    && event["args"] == args
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("original report score differs from actual effects")?;
        scored.remove(index);
    }
    if !scored.is_empty() {
        return Err("unused original report event".into());
    }
    let mut used_calls = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut admitted_frames = BTreeSet::new();
    let mut loan_total = 0;
    for (resource, service, result_label, expected) in [
        (
            document,
            document_service,
            label("DocumentResult")?,
            3 + rejected,
        ),
        (database, database_service, label("DatabaseResult")?, 2),
    ] {
        let is_loan = |frame: &&crate::journal::Frame| {
            frame.bytes.len() == 3 && u64::from(frame.bytes[0]) == resource
        };
        let mut loans = frames
            .values()
            .filter(|frame| {
                frame.identity["source"] == service
                    && [a, b, editor]
                        .iter()
                        .any(|vm| frame.identity["target"] == *vm)
            })
            .filter(is_loan)
            .collect::<Vec<_>>();
        loans.sort_by_key(|frame| frame.sent);
        if loans.len() != expected {
            return Err("report resource loan count".into());
        }
        let mut previous = None;
        for (index, loan) in loans.iter().enumerate() {
            let vm = loan.identity["target"].as_u64().ok_or("loan recipient")?;
            let next = loans.get(index + 1).map_or(rows.len(), |frame| frame.sent);
            let returned = one(frames.values().filter(|frame| {
                frame.direction(vm, service)
                    && is_loan(frame)
                    && frame.sent > loan.received.unwrap_or(usize::MAX)
                    && frame.sent < next
            }))?;
            let request = one(frames.values().filter(|frame| {
                frame.direction(resource, service)
                    && frame.sent > loan.received.unwrap_or(usize::MAX)
                    && frame.sent < returned.sent
            }))?;
            let response = one(frames.values().filter(|frame| {
                frame.direction(service, resource)
                    && frame.sent > loan.received.unwrap_or(usize::MAX)
                    && frame.sent < returned.sent
            }))?;
            let tool = if request.logical_label == label("WriteOnDocument")? && resource == document
            {
                "write_section"
            } else if request.logical_label == label("ReviseOnDocument")? && resource == document {
                "revise_section"
            } else if request.logical_label == label("CombineOnDocument")? && resource == document {
                "combine_sections"
            } else if request.logical_label == label("ReferencesOnDatabase")?
                && resource == database
            {
                "update_references"
            } else {
                return Err("unexpected compiled report resource request".into());
            };
            let admission_label = match (vm, tool) {
                (vm, "write_section") if vm == a => label("WriteA")?,
                (vm, "write_section") if vm == b => label("WriteB")?,
                (vm, "revise_section") if vm == a => label("ReviseA")?,
                (vm, "revise_section") if vm == b => label("ReviseB")?,
                (vm, "update_references") if vm == a => label("ReferencesA")?,
                (vm, "update_references") if vm == b => label("ReferencesB")?,
                (vm, "combine_sections") if vm == editor => label("Combine")?,
                _ => return Err("resource operation belongs to another report participant".into()),
            };
            let admitted = one(frames.values().filter(|frame| {
                frame.direction(vm, service)
                    && frame.logical_label == admission_label
                    && frame.received.is_some_and(|at| {
                        previous.is_none_or(|before| before < at) && at < loan.sent
                    })
            }))?;
            let arguments = request.value()?;
            let agent = if vm == a {
                "RESEARCHER_A"
            } else if vm == b {
                "RESEARCHER_B"
            } else {
                "EDITOR"
            };
            let matching = calls
                .iter()
                .filter(|(_, call)| {
                    call.request["agent"] == agent
                        && call.request["tool"] == tool
                        && call.request["arguments"] == arguments
                        && call.emitted > request.received.unwrap_or(usize::MAX)
                        && call.emitted < returned.sent
                })
                .collect::<Vec<_>>();
            let [(id, call)] = matching.as_slice() else {
                return Err("resource request must select one actual report effect".into());
            };
            let (result_at, result) = call.result()?;
            if !used_calls.insert(**id)
                || !admitted_frames.insert(admitted.sent)
                || admitted.value()? != arguments
                || response.value()? != *result
                || response.logical_label != result_label
                || !(previous.is_none_or(|before| before < loan.sent)
                    && loan.received()? < request.sent
                    && request.received()? < call.emitted
                    && result_at < response.sent
                    && response.received()? < returned.sent
                    && returned.received()? < next)
            {
                return Err("report loan/admission/request/result/return mismatch".into());
            }
            close_arguments(
                &frames,
                &mut used_closes,
                vm,
                if vm == a {
                    a_io
                } else if vm == b {
                    b_io
                } else {
                    editor_io
                },
                admitted.sent,
                &format!("service/{tool}"),
                &arguments,
            )?;
            previous = Some(returned.received()?);
        }
        if frames
            .values()
            .filter(|frame| {
                frame.identity["target"] == service
                    && [a, b, editor]
                        .iter()
                        .any(|vm| frame.identity["source"] == *vm)
            })
            .filter(is_loan)
            .count()
            != loans.len()
            || frames
                .values()
                .filter(|frame| frame.direction(resource, service))
                .count()
                != loans.len()
            || frames
                .values()
                .filter(|frame| frame.direction(service, resource))
                .count()
                != loans.len()
        {
            return Err("unused or extra report resource frame".into());
        }
        loan_total += loans.len();
    }

    for (agent, vm, io, service_name) in [
        ("RESEARCHER_A", a, a_io, "RESEARCH_A"),
        ("RESEARCHER_B", b, b_io, "RESEARCH_B"),
    ] {
        let service = name(compiled, "roles", service_name)?;
        let matching = calls
            .iter()
            .filter(|(_, call)| {
                call.request["agent"] == agent && call.request["tool"] == "research_topic"
            })
            .collect::<Vec<_>>();
        let [(id, call)] = matching.as_slice() else {
            return Err("report requires one research effect per researcher".into());
        };
        let request = one(frames.values().filter(|frame| frame.direction(vm, service)))?;
        let reply = one(frames.values().filter(|frame| frame.direction(service, vm)))?;
        if !used_calls.insert(**id)
            || request.logical_label != label("Research")?
            || reply.logical_label != label("ResearchResult")?
            || request.value()? != call.request["arguments"]
            || reply.value()? != *call.result()?.1
            || !(request.received()? < call.emitted && call.result()?.0 < reply.sent)
        {
            return Err("research request/result differs from its actual effect".into());
        }
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            io,
            request.sent,
            "service/research_topic",
            &call.request["arguments"],
        )?;
    }

    for author in ["RESEARCHER_A", "RESEARCHER_B"] {
        let research = calls
            .values()
            .filter(|call| {
                call.request["agent"] == author && call.request["tool"] == "research_topic"
            })
            .collect::<Vec<_>>();
        let write = calls
            .values()
            .filter(|call| {
                call.request["agent"] == author && call.request["tool"] == "write_section"
            })
            .collect::<Vec<_>>();
        let references = calls
            .values()
            .filter(|call| {
                call.request["agent"] == author && call.request["tool"] == "update_references"
            })
            .collect::<Vec<_>>();
        let ([research], [write], [references]) =
            (research.as_slice(), write.as_slice(), references.as_slice())
        else {
            return Err("missing or repeated report preparation".into());
        };
        if research.ended()?.0 >= write.started()?.0 || write.ended()?.0 >= references.started()?.0
        {
            return Err("report preparation order differs".into());
        }
    }

    let communication = crate::projection::contracts(compiled)?;
    let intents = compiled["review_intents"]
        .as_array()
        .ok_or("compiled report review intents")?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or("review intent label")
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let mut reviews = calls
        .iter()
        .filter(|(_, call)| call.request["tool"] == "review_sections")
        .collect::<Vec<_>>();
    reviews.sort_by_key(|(_, call)| call.emitted);
    if reviews.len() != 2 + rejected {
        return Err("report review count".into());
    }
    let mut readiness = BTreeSet::new();
    for (agent, author, ready) in [
        ("RESEARCHER_A", a, label("ReadyA")?),
        ("RESEARCHER_B", b, label("ReadyB")?),
    ] {
        let references = one(calls.values().filter(|call| {
            call.request["agent"] == agent && call.request["tool"] == "update_references"
        }))?;
        let (result_at, result) = references.result()?;
        for (section, service) in [("section_a", review_a), ("section_b", review_b)] {
            let first_review = reviews
                .iter()
                .find(|(_, call)| target(call).ok() == Some(section))
                .ok_or("missing first report review")?
                .1;
            let frame = one(frames.values().filter(|frame| {
                frame.direction(author, service)
                    && frame.logical_label == ready
                    && result_at < frame.sent
                    && frame.received.is_some_and(|at| at < first_review.emitted)
                    && frame.value().is_ok_and(|value| value == *result)
            }))?;
            if !readiness.insert(frame.sent) {
                return Err("report submission reused by another review service".into());
            }
        }
    }
    let mut review_requests = BTreeSet::new();
    let mut review_replies = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    let mut previous_review = None;
    for (id, call) in &reviews {
        let section = target(call)?;
        let service = if section == "section_a" {
            review_a
        } else if section == "section_b" {
            review_b
        } else {
            return Err("unknown reviewed section".into());
        };
        for author in ["RESEARCHER_A", "RESEARCHER_B"] {
            for tool in ["write_section", "update_references"] {
                let prior = calls
                    .values()
                    .filter(|candidate| {
                        candidate.request["agent"] == author && candidate.request["tool"] == tool
                    })
                    .collect::<Vec<_>>();
                let [prior] = prior.as_slice() else {
                    return Err("report review requires both actual submissions".into());
                };
                if prior.ended()?.0 >= call.started()?.0 {
                    return Err("report review precedes both submissions".into());
                }
            }
        }
        let earlier = reviews
            .iter()
            .filter(|(_, prior)| {
                target(prior).ok() == Some(section) && prior.emitted < call.emitted
            })
            .max_by_key(|(_, prior)| prior.emitted);
        if let Some((_, prior)) = earlier {
            if prior.result()?.1["success"] != false {
                return Err("report re-reviewed an already approved section".into());
            }
            let changed = calls
                .iter()
                .filter(|(_, change)| {
                    change.request["tool"] == "revise_section"
                        && target(change).ok() == Some(section)
                        && change.started.is_some_and(|(at, _)| {
                            prior.result().is_ok_and(|(before, _)| before < at)
                        })
                        && change.ended.is_some_and(|(at, _)| at < call.emitted)
                })
                .collect::<Vec<_>>();
            let [(change_id, change)] = changed.as_slice() else {
                return Err("report re-review requires one intervening actual revision".into());
            };
            if !revisions.insert(**change_id) {
                return Err("revision reused by another report review".into());
            }
            let (result_at, result) = change.result()?;
            let author = if service == review_a { a } else { b };
            let ready = label(if service == review_a {
                "ReadyA"
            } else {
                "ReadyB"
            })?;
            let frame = one(frames.values().filter(|frame| {
                frame.direction(author, service)
                    && frame.logical_label == ready
                    && result_at < frame.sent
                    && frame.received.is_some_and(|at| at < call.emitted)
                    && frame.value().is_ok_and(|value| value == *result)
            }))?;
            if !readiness.insert(frame.sent) {
                return Err("report revision readiness reused".into());
            }
        }
        let mut requests = frames
            .values()
            .filter(|frame| {
                frame.identity["source"] == editor
                    && [a, b, review_a, review_b, document_service]
                        .iter()
                        .any(|role| frame.identity["target"] == *role)
                    && previous_review.is_none_or(|before| before < frame.sent)
                    && frame.sent < call.emitted
                    && frame
                        .value()
                        .is_ok_and(|value| value == call.request["arguments"])
            })
            .collect::<Vec<_>>();
        requests.sort_by_key(|frame| frame.sent);
        if requests.len() < 2 {
            return Err("report review requires actual intents and one unique request".into());
        }
        let intent = requests[0].logical_label;
        let author = if service == review_a { a } else { b };
        let recipients = communication
            .values()
            .filter(|header| {
                u64::from(header[1]) == editor
                    && header[3] == intent
                    && [document_service, review_a, review_b, a, b].contains(&u64::from(header[2]))
            })
            .map(|header| u64::from(header[2]))
            .collect::<BTreeSet<_>>();
        let (request, deliveries) = requests.split_last().ok_or("missing review request")?;
        let observed = deliveries
            .iter()
            .map(|frame| frame.identity["target"].as_u64().ok_or("intent recipient"))
            .collect::<std::result::Result<BTreeSet<_>, _>>()?;
        if !BTreeSet::from([document_service, service, author]).is_subset(&recipients)
            || observed != recipients
            || deliveries.len() != recipients.len()
            || !request.direction(editor, service)
            || requests
                .iter()
                .any(|frame| !review_requests.insert(frame.sent))
        {
            return Err("report intent recipients, ordering or reuse".into());
        }
        let request_labels = if service == review_a {
            [label("ReviewA")?, label("ReReviewA")?]
        } else {
            [label("ReviewB")?, label("ReReviewB")?]
        };
        if !intents.contains(&intent)
            || deliveries.iter().any(|frame| frame.logical_label != intent)
            || !request_labels.contains(&request.logical_label)
            || request.received()? >= call.emitted
            || !used_calls.insert(**id)
        {
            return Err("report review differs from its compiled request or occurrence".into());
        }
        close_arguments(
            &frames,
            &mut used_closes,
            editor,
            editor_io,
            requests[0].sent,
            "service/review_sections",
            &call.request["arguments"],
        )?;
        let result = call.result()?;
        let next = reviews
            .iter()
            .filter(|(_, later)| {
                target(later).ok() == Some(section) && later.emitted > call.emitted
            })
            .map(|(_, later)| later.emitted)
            .min()
            .unwrap_or(rows.len());
        let reply_label = match (section, result.1["success"].as_bool()) {
            ("section_a", Some(true)) => label("ApprovedA")?,
            ("section_a", Some(false)) => label("RejectedA")?,
            ("section_b", Some(true)) => label("ApprovedB")?,
            ("section_b", Some(false)) => label("RejectedB")?,
            _ => return Err("report result lacks a boolean outcome".into()),
        };
        let other_section = if service == review_a {
            "section_b"
        } else {
            "section_a"
        };
        let other_approved = reviews.iter().any(|(_, earlier)| {
            target(earlier).ok() == Some(other_section)
                && earlier
                    .result()
                    .is_ok_and(|(at, result)| at < call.emitted && result["success"] == true)
        });
        let recipients = if other_approved {
            vec![document_service, author, editor]
        } else {
            vec![
                if service == review_a {
                    review_b
                } else {
                    review_a
                },
                document_service,
                a,
                b,
                editor,
            ]
        };
        for recipient in recipients {
            let reply = one(frames.values().filter(|frame| {
                frame.direction(service, recipient)
                    && frame.sent > result.0
                    && frame.sent < next
                    && frame.value().is_ok_and(|value| value == *result.1)
            }))?;
            if reply.logical_label != reply_label || !review_replies.insert(reply.sent) {
                return Err("reused or altered report review reply".into());
            }
        }
        previous_review = Some(result.0);
    }
    if calls
        .iter()
        .filter(|(_, call)| call.request["tool"] == "revise_section")
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>()
        != revisions
    {
        return Err("unmatched or repeated report revision".into());
    }
    let ready_a = label("ReadyA")?;
    let ready_b = label("ReadyB")?;
    let observed_ready = frames
        .values()
        .filter(|frame| {
            (frame.logical_label == ready_a || frame.logical_label == ready_b)
                && (frame.identity["source"] == a || frame.identity["source"] == b)
                && (frame.identity["target"] == review_a || frame.identity["target"] == review_b)
        })
        .map(|frame| frame.sent)
        .collect::<BTreeSet<_>>();
    if readiness != observed_ready || readiness.len() != 4 + rejected {
        return Err("unmatched or repeated report readiness".into());
    }
    let combine = calls
        .values()
        .filter(|call| call.request["tool"] == "combine_sections")
        .collect::<Vec<_>>();
    let [combine] = combine.as_slice() else {
        return Err("report requires exactly one final combination".into());
    };
    let mut confirmations = BTreeSet::new();
    for (section, confirmation) in [
        ("section_a", "ConfirmApprovedA"),
        ("section_b", "ConfirmApprovedB"),
    ] {
        let reviewed = reviews
            .iter()
            .filter(|(_, call)| {
                target(call).ok() == Some(section) && call.emitted < combine.emitted
            })
            .max_by_key(|(_, call)| call.emitted)
            .ok_or("combine without actual report review")?
            .1;
        let changed = calls
            .values()
            .filter(|call| {
                matches!(call.tool().ok(), Some("write_section" | "revise_section"))
                    && target(call).ok() == Some(section)
                    && call.emitted < combine.emitted
            })
            .max_by_key(|call| call.emitted)
            .ok_or("combine without current section")?;
        if reviewed.result()?.1["success"] != true || changed.ended()?.0 >= reviewed.started()?.0 {
            return Err("combine lacks approval of the current section".into());
        }
        let confirmation_label = label(confirmation)?;
        let (review_at, review_result) = reviewed.result()?;
        let receipt = one(frames.values().filter(|frame| {
            frame.direction(editor, document_service)
                && frame.logical_label == confirmation_label
                && frame.sent > review_at
                && frame.received.is_some_and(|at| at < combine.emitted)
        }))?;
        if receipt.value()? != *review_result || !confirmations.insert(receipt.sent) {
            return Err("report confirmation differs from the actual review".into());
        }
    }
    let admission_labels = [
        "WriteA",
        "WriteB",
        "ReviseA",
        "ReviseB",
        "ReferencesA",
        "ReferencesB",
        "Combine",
    ]
    .map(label)
    .into_iter()
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .values()
        .filter(|frame| {
            [document_service, database_service]
                .iter()
                .any(|service| frame.identity["target"] == *service)
                && [a, b, editor]
                    .iter()
                    .any(|vm| frame.identity["source"] == *vm)
                && admission_labels.contains(&frame.logical_label)
        })
        .count()
        != admitted_frames.len()
    {
        return Err("unused or extra report admission".into());
    }
    if used_calls.len() != calls.len()
        || frames
            .values()
            .filter(|frame| {
                frame.identity["source"] == editor
                    && [a, b, review_a, review_b, document_service]
                        .iter()
                        .any(|role| frame.identity["target"] == *role)
                    && frame.bytes.first() == Some(&b'{')
            })
            .count()
            != review_requests.len() + confirmations.len() + 1
        || frames
            .values()
            .filter(|frame| {
                frame.identity["source"] == review_a || frame.identity["source"] == review_b
            })
            .count()
            != review_replies.len()
    {
        return Err("unused or extra report business request/result".into());
    }
    let mut overlaps = BTreeMap::<String, usize>::new();
    let values = calls.values().collect::<Vec<_>>();
    for (index, first) in values.iter().enumerate() {
        for second in values.iter().skip(index + 1) {
            if first.started()?.1.max(second.started()?.1) < first.ended()?.1.min(second.ended()?.1)
            {
                let mut pair = [first.tool()?, second.tool()?];
                pair.sort_unstable();
                *overlaps.entry(pair.join("+")).or_default() += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"3E","kind":"completed_actual_wasi_report_trace","scored_benchmark":false,
        "domain_calls":calls.len(),"actual_resource_endpoint_requests":loan_total,"actual_resource_endpoint_returns":loan_total,
        "actual_review_requests":reviews.len(),"forwarded_approvals":confirmations.len(),"guest_exits":actors.len(),
        "simulator_complete":true,"violations":0,"overlapping_effect_pairs":overlaps}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let projection = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &projection)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&projection, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("report directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}

/// Check trace mutations against the static wire and business-effect audit.
/// Dynamic source execution is a separate qualification of the original trace.
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let baseline = check(&rows, &projection)?;
    let compiled = &projection["compiled"];
    let actors = rows[0]["message"]["fixtures"]
        .as_array()
        .ok_or("report participants")?;
    let headers = crate::projection::frames(&projection, &rows)?;
    let journal = crate::journal::read(&rows, &headers, actors)?;
    let private = ["A_IO", "B_IO", "EDITOR_IO"]
        .map(|key| name(compiled, "roles", key))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let resources = [
        name(compiled, "roles", "DOCUMENT")?,
        name(compiled, "roles", "DATABASE")?,
    ];
    let document_service = name(compiled, "roles", "DOCUMENT_SERVICE")?;
    let write_label = name(compiled, "labels", "WriteOnDocument")? as u8;
    let fresh = journal
        .frames
        .keys()
        .max()
        .ok_or("report frames")?
        .checked_add(1)
        .ok_or("frame ID overflow")?;
    let mut mutations = Vec::new();
    for (&id, frame) in &journal.frames {
        let source = frame.identity["source"].as_u64().ok_or("frame source")?;
        let target = frame.identity["target"].as_u64().ok_or("frame target")?;
        if !private.contains(&source) && !private.contains(&target) {
            if frame.bytes.first() == Some(&b'{') {
                mutations.extend([
                    ("missing_business_frame", Some(id)),
                    ("forged_business_frame", Some(id)),
                    ("duplicate_business_frame", Some(id)),
                ]);
            }
            if frame.bytes.len() == 3 && resources.contains(&u64::from(frame.bytes[0])) {
                mutations.extend([
                    ("missing_endpoint_frame", Some(id)),
                    ("forged_endpoint_frame", Some(id)),
                ]);
            }
        }
        if private.contains(&source)
            && frame.logical_label == hibana_wasip1_runtime::protocol::LABEL_WASI_FD_CLOSE_RET
            && frame.bytes.first() == Some(&4)
            && frame.bytes.len() > 4
        {
            mutations.push(("missing_wasi_close", Some(id)));
        }
    }
    let first = *journal.frames.keys().next().ok_or("report frame header")?;
    mutations.extend([
        ("wrong_frame_label", Some(first)),
        ("wrong_lane", Some(first)),
        ("missing_original_result", None),
        ("forged_original_result", None),
        ("missing_guest_exit", None),
        ("false_score", None),
        ("effect_before_resource_request", None),
    ]);
    fs::create_dir(output)?;
    let mut verdicts = Vec::new();
    let mut coverage = BTreeMap::<&str, usize>::new();
    for (kind, id) in mutations {
        let mut changed = rows.clone();
        if let Some(id) = id {
            let matches = |row: &Value| {
                row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id
            };
            if kind.starts_with("missing_") {
                changed.retain(|row| !matches(row));
            } else if kind == "duplicate_business_frame" {
                changed = changed
                    .into_iter()
                    .flat_map(|row| {
                        let copy = matches(&row).then(|| {
                            let mut copy = row.clone();
                            copy["message"]["frame_id"] = json!(fresh);
                            copy
                        });
                        core::iter::once(row).chain(copy)
                    })
                    .collect();
            } else {
                let original = &journal.frames[&id];
                for row in changed.iter_mut().filter(|row| matches(row)) {
                    match kind {
                        "forged_business_frame" => {
                            row["message"]["payload"] =
                                json!(serde_json::to_vec(&json!({"forged":true}))?)
                        }
                        "forged_endpoint_frame" => {
                            let mut payload = original.bytes.clone();
                            payload[0] = if u64::from(payload[0]) == resources[0] {
                                resources[1] as u8
                            } else {
                                resources[0] as u8
                            };
                            row["message"]["payload"] = json!(payload);
                        }
                        "wrong_frame_label" => row["message"]["frame_label"] = json!(u16::MAX),
                        "wrong_lane" => {
                            row["message"]["lane"] = json!(
                                (original.identity["lane"].as_u64().ok_or("wire lane")? + 1) % 256
                            )
                        }
                        _ => return Err("unknown report frame mutation".into()),
                    }
                }
            }
        } else {
            match kind {
                "missing_original_result" => {
                    let at = changed
                        .iter()
                        .position(|row| row["message"]["type"] == "simulator_result")
                        .ok_or("original report result")?;
                    changed.remove(at);
                }
                "forged_original_result" => {
                    let row = changed
                        .iter_mut()
                        .find(|row| {
                            row["message"]["type"] == "simulator_result"
                                && row["message"]["result"]["tool_name"] == "review_sections"
                        })
                        .ok_or("actual report review result")?;
                    row["message"]["result"]["data"] = json!({"forged":true});
                }
                "missing_guest_exit" => {
                    let at = changed
                        .iter()
                        .position(|row| {
                            row["direction"] == "runtime" && row["message"]["type"] == "exit"
                        })
                        .ok_or("report guest exit")?;
                    changed.remove(at);
                }
                "false_score" => {
                    changed
                        .iter_mut()
                        .find(|row| row["message"]["type"] == "score")
                        .ok_or("report score")?["message"]["complete"] = json!(false)
                }
                "effect_before_resource_request" => {
                    let request = journal
                        .frames
                        .values()
                        .filter(|frame| {
                            frame.logical_label == write_label
                                && frame.direction(resources[0], document_service)
                        })
                        .min_by_key(|frame| frame.sent)
                        .ok_or("actual report document request")?;
                    let arguments = request.value()?;
                    let call = one(journal.calls.values().filter(|call| {
                        call.request["tool"] == "write_section"
                            && call.request["arguments"] == arguments
                    }))?;
                    let before = request.received()?;
                    let mut effect = changed.remove(call.emitted);
                    effect["elapsed_seconds"] = changed[before]["elapsed_seconds"].clone();
                    changed.insert(before, effect);
                }
                _ => return Err("unknown report record mutation".into()),
            }
        }
        if changed == rows {
            return Err(format!("empty report mutation {kind} {id:?}").into());
        }
        let rejection = check(&changed, &projection)
            .err()
            .ok_or_else(|| format!("undetected report mutation {kind} {id:?}"))?;
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&changed)?));
        verdicts.push(json!({"kind":kind,"frame_id":id,"status":"rejected","parsed_records_sha256":hash,"reason":rejection.to_string()}));
        *coverage.entry(kind).or_default() += 1;
    }
    for kind in [
        "missing_business_frame",
        "forged_business_frame",
        "duplicate_business_frame",
        "missing_endpoint_frame",
        "forged_endpoint_frame",
        "missing_wasi_close",
        "wrong_frame_label",
        "wrong_lane",
        "missing_original_result",
        "forged_original_result",
        "missing_guest_exit",
        "false_score",
        "effect_before_resource_request",
    ] {
        if !coverage.contains_key(kind) {
            return Err(format!("report mutation domain did not exercise {kind}").into());
        }
    }
    let result = json!({"status":"passed","task":"3E","scope":"static wire contracts, business/effect correspondence and completion; dynamic execution checking is separate","source_events_sha256":digest(path)?,"baseline_static_audit":baseline,"count":verdicts.len(),"coverage":coverage,"mutants":verdicts,"representation":"Each mutant is regenerated from its original trace and frame ID; redundant JSONL copies are not stored"});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
