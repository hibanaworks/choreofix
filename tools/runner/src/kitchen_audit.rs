//! Actual file/resource/material receives and effects; EPF owns source execution.
use crate::{
    evidence::{Result, digest, save},
    journal::{Frame, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn take<'a>(
    frames: &'a BTreeMap<u64, Frame>,
    used: &mut BTreeSet<u64>,
    predicate: impl Fn(&Frame) -> bool,
) -> Result<(u64, &'a Frame)> {
    let mut candidates = frames.iter().filter(|(_, frame)| predicate(frame));
    let (&id, frame) = candidates.next().ok_or("missing kitchen receipt")?;
    if candidates.next().is_some() || !used.insert(id) {
        return Err("duplicate or reused kitchen receipt".into());
    }
    Ok((id, frame))
}
fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty kitchen trace")?["message"];
    if compiled["task"] != "12E" || fixture["task"] != "12E" || fixture["type"] != "fixtures" {
        return Err("kitchen source or input differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("kitchen actors")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("CHEF_A"), Some("CHEF_B"), Some("CHEF_C")])
    {
        return Err("kitchen participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let mut journal = crate::journal::read(rows, &labels, actors)?;
    journal.bind_receives(dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if calls.len() != 9
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("kitchen completion or effect population differs".into());
    }
    let recipes = [
        ("CHEF_A", "A", "appetizer", "sauce", "CHEF_C", "garnish"),
        ("CHEF_B", "B", "main_course", "garnish", "CHEF_A", "glaze"),
        ("CHEF_C", "C", "dessert", "glaze", "CHEF_B", "sauce"),
    ];
    let mut used = BTreeSet::new();
    let mut scored_used = BTreeSet::new();
    let mut bases = BTreeMap::new();
    let mut ingredients = BTreeMap::new();
    let mut combinations = Vec::new();
    let prep = name("roles", "INGREDIENTS")?;
    let finish = name("roles", "FINISH")?;
    let oven = name("roles", "OVEN")?;
    let stove = name("roles", "STOVETOP")?;
    let scored = score["events"]
        .as_array()
        .ok_or("kitchen original effects")?;
    let mut intervals = Vec::new();
    for (chef, letter, dish, ingredient, recipient, received) in recipes {
        let actor = name("roles", chef)?;
        let material = match letter {
            "A" => "Sauce",
            "B" => "Garnish",
            _ => "Glaze",
        };
        for (tool, args, file, request_name, reply_name, service) in [
            (
                "prepare_base_dish",
                json!({"dish":dish}),
                format!("BASE_FILE_{letter}"),
                format!("BaseIntent{letter}"),
                format!("BaseResult{letter}"),
                actor,
            ),
            (
                "prepare_ingredient",
                json!({"ingredient":ingredient,"for_chef":recipient}),
                format!("INGREDIENT_FILE_{letter}"),
                format!("Prepare{material}"),
                format!("{material}Prepared"),
                prep,
            ),
            (
                "combine_dish",
                json!({"dish":dish,"ingredient":received}),
                format!("COMBINE_FILE_{letter}"),
                format!("Combine{letter}"),
                format!("Dish{letter}"),
                finish,
            ),
        ] {
            let file = name("roles", &file)?;
            let call = crate::journal::one(
                calls
                    .values()
                    .filter(|call| call.request["agent"] == chef && call.request["tool"] == tool),
            )?;
            if call.request["arguments"] != args {
                return Err("kitchen effect differs from the original assignment".into());
            }
            let request_label = name("labels", &request_name)? as u8;
            let (_, request) = take(&frames, &mut used, |f| {
                f.direction(file, service) && f.logical_label == request_label
            })?;
            if request.value()? != args || request.received()? >= call.emitted {
                return Err("effect before its actual operation-file receive".into());
            }
            let (result_at, result) = call.result()?;
            let status = match tool {
                "prepare_base_dish" => "base dish prepared",
                "prepare_ingredient" => "ingredient prepared",
                _ => "dish completed",
            };
            if result["tool_name"] != tool
                || result["success"] != true
                || result["data"]["chef"] != chef
                || result["data"]["status"] != status
                || args
                    .as_object()
                    .ok_or("recipe arguments")?
                    .iter()
                    .any(|(key, value)| result["data"][key] != *value)
            {
                return Err("kitchen result differs from actual assigned effect".into());
            }
            let score_index = scored
                .iter()
                .enumerate()
                .find(|(index, event)| {
                    !scored_used.contains(index)
                        && event["agent"] == chef
                        && event["tool"] == tool
                        && event["args"] == args
                        && event["success"] == true
                        && event["result"] == result["data"]
                        && event["violations"] == json!([])
                })
                .map(|(index, _)| index)
                .ok_or("world score differs from the kitchen effect")?;
            scored_used.insert(score_index);
            let (_, start) = call.started()?;
            let (_, end) = call.ended()?;
            let reply_label = name("labels", &reply_name)? as u8;
            let (_, reply) = take(&frames, &mut used, |f| {
                f.direction(service, file) && f.logical_label == reply_label
            })?;
            if reply.value()? != *result || reply.sent <= result_at {
                return Err("operation-file reply differs from actual result".into());
            }
            reply.received()?;
            if tool == "prepare_base_dish" {
                for (resource, prefix) in [(oven, "Oven"), (stove, "Stove")] {
                    if letter == "C" && resource == stove {
                        continue;
                    }
                    let actual_label = name("labels", &format!("{prefix}Request{letter}"))? as u8;
                    let response_label = name("labels", &format!("{prefix}Result{letter}"))? as u8;
                    let (_, actual) = take(&frames, &mut used, |f| {
                        f.direction(resource, actor) && f.logical_label == actual_label
                    })?;
                    let (_, response) = take(&frames, &mut used, |f| {
                        f.direction(actor, resource) && f.logical_label == response_label
                    })?;
                    if actual.value()? != args
                        || actual.received()? >= call.emitted
                        || response.value()? != *result
                        || response.sent <= result_at
                    {
                        return Err("equipment request or reply differs from actual effect".into());
                    }
                    response.received()?;
                }
                bases.insert(chef, (score_index, end));
                intervals.push((chef, start, end));
            } else if tool == "prepare_ingredient" {
                let consumer = name("roles", recipient)?;
                let ready_label = name("labels", material)? as u8;
                let (_, ready) = take(&frames, &mut used, |f| {
                    f.direction(prep, consumer) && f.logical_label == ready_label
                })?;
                if ready.value()? != *result || ready.sent <= result_at {
                    return Err(
                        "incoming ingredient differs from its actual completed effect".into(),
                    );
                }
                ingredients.insert(ingredient, (score_index, end, ready.received()?));
            } else {
                combinations.push((
                    chef,
                    received,
                    start,
                    request.sent,
                    result.clone(),
                    score_index,
                ));
            }
        }
    }
    for (chef, ingredient, start, sent, result, score_index) in combinations {
        let (base_index, base_end) = bases[chef];
        let (ingredient_index, ingredient_end, received) = ingredients[ingredient];
        if base_end > start
            || ingredient_end > start
            || received >= sent
            || result["data"]["base_event"] != base_index
            || result["data"]["ingredient_event"] != ingredient_index
            || [base_index, ingredient_index].into_iter().any(|index| {
                scored[index]["completed_at"]
                    .as_f64()
                    .is_none_or(|completed| {
                        scored[score_index]["timestamp"]
                            .as_f64()
                            .is_none_or(|began| completed > began)
                    })
            })
        {
            return Err("dish lacks its actual completed base and delivered ingredient".into());
        }
    }
    if intervals.iter().enumerate().any(|(i, (_, start, end))| {
        intervals[i + 1..]
            .iter()
            .any(|(_, a, b)| start < b && a < end)
    }) {
        return Err("actual oven effects overlap".into());
    }
    let owned_labels = [
        "OvenLoanA",
        "StoveLoanA",
        "OvenLoanB",
        "StoveLoanB",
        "OvenLoanC",
        "ReturnOvenA",
        "ReturnStoveA",
        "ReturnOvenB",
        "ReturnStoveB",
        "ReturnOvenC",
    ]
    .into_iter()
    .map(|label| name("labels", label))
    .collect::<Result<BTreeSet<_>>>()?;
    let business = compiled["labels"]
        .as_object()
        .ok_or("kitchen labels")?
        .values()
        .map(|value| value.as_u64().ok_or("kitchen label"))
        .collect::<std::result::Result<BTreeSet<_>, _>>()?
        .difference(&owned_labels)
        .copied()
        .collect::<BTreeSet<_>>();
    if scored_used.len() != scored.len()
        || frames.iter().any(|(id, frame)| {
            business.contains(&u64::from(frame.logical_label)) && !used.contains(id)
        })
    {
        return Err("unmatched kitchen effect or business occurrence".into());
    }
    let return_labels = [
        "ReturnOvenA",
        "ReturnStoveA",
        "ReturnOvenB",
        "ReturnStoveB",
        "ReturnOvenC",
    ]
    .into_iter()
    .map(|label| name("labels", label).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    let returns = frames
        .values()
        .filter(|frame| return_labels.contains(&frame.logical_label) && frame.received.is_some())
        .count();
    if returns != 5 {
        return Err("incomplete kitchen equipment return coverage".into());
    }
    let mut overlaps = 0;
    for (i, left) in scored.iter().enumerate() {
        for right in &scored[i + 1..] {
            if (left["tool"] == "prepare_base_dish" && right["tool"] == "prepare_ingredient"
                || right["tool"] == "prepare_base_dish" && left["tool"] == "prepare_ingredient")
                && left["timestamp"].as_f64().ok_or("effect start")?
                    < right["completed_at"].as_f64().ok_or("effect end")?
                && right["timestamp"].as_f64().ok_or("effect start")?
                    < left["completed_at"].as_f64().ok_or("effect end")?
            {
                overlaps += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"12E","domain_calls":calls.len(),"dishes":3,
        "violations":0,"simulator_complete":true,"actual_typed_file_requests":9,
        "actual_resource_endpoint_requests":5,"actual_resource_endpoint_returns":returns,
        "base_ingredient_overlap_pairs":overlaps,"matched_json_frame_ids":used,
        "dynamic_protocol":dynamic,
        "effect_scope":"unique committed operation-file/resource/material receives, actual effect/result and completed world inputs; EPF owns source execution and delegation"}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = crate::projection::check_dynamic(&artifact, &rows)?;
    let mut result = check(&rows, &artifact, &dynamic)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("kitchen event directory")?
            .join("projection.json")
    )?);
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    // Mutations below leave source capture untouched. Check that capture once.
    let dynamic = crate::projection::check_dynamic(&artifact, &rows)?;
    let mut baseline = check(&rows, &artifact, &dynamic)?;
    baseline["source_events_sha256"] = json!(digest(path)?);
    baseline["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    baseline["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("kitchen event directory")?
            .join("projection.json")
    )?);
    let ids = baseline["matched_json_frame_ids"]
        .as_array()
        .ok_or("JSON receipts")?
        .iter()
        .map(|id| id.as_u64().ok_or_else(|| "frame ID".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = crate::journal::json_frame_mutants(&rows, &ids, |changed| {
        check(changed, &artifact, &dynamic)
    })?;
    let first = *ids.first().ok_or("missing kitchen frame")?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        let mut count = 0;
        for row in &mut changed {
            let m = &mut row["message"];
            if m["type"] == "protocol_frame" && m["frame_id"] == first {
                m[field] = json!(
                    m[field].as_u64().ok_or("wire field")?
                        ^ if field == "lane" { 255 } else { 65535 }
                );
                count += 1;
            }
        }
        if count < 2 || check(&changed, &artifact, &dynamic).is_ok() {
            return Err("accepted coherently changed kitchen header".into());
        }
        checked.push(
            json!({"kind":format!("coherent_wrong_{field}"),"frame_id":first,"rejected":true}),
        );
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,
        "scope":"record mutations, not runtime attacks; unchanged source capture checked once"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
