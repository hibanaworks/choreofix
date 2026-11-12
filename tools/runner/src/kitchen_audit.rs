//! Bind actual kitchen effects to unique WASI, resource and material receipts.
//! This is offline evidence checking; EPF owns dynamic source interpretation.
use crate::{
    evidence::{Result, digest, save},
    journal::{Frame, check_close_pairs, close_arguments, records},
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
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty kitchen trace")?["message"];
    if compiled["task"] != "12E" || fixture["task"] != "12E" || fixture["type"] != "fixtures" {
        return Err("kitchen source or input differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("kitchen actors")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|actor| actor["agent"].as_str())
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
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() != 9
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("kitchen completion or effect population differs".into());
    }
    check_close_pairs(
        &frames,
        &[
            (name("roles", "CHEF_A")?, name("roles", "A_IO")?),
            (name("roles", "CHEF_B")?, name("roles", "B_IO")?),
            (name("roles", "CHEF_C")?, name("roles", "C_IO")?),
        ],
    )?;
    let recipes = [
        (
            "CHEF_A",
            "A",
            "appetizer",
            "sauce",
            "CHEF_C",
            "garnish",
            "A_IO",
        ),
        (
            "CHEF_B",
            "B",
            "main_course",
            "garnish",
            "CHEF_A",
            "glaze",
            "B_IO",
        ),
        ("CHEF_C", "C", "dessert", "glaze", "CHEF_B", "sauce", "C_IO"),
    ];
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut scored_used = BTreeSet::new();
    let mut bases = BTreeMap::new();
    let mut ingredients = BTreeMap::new();
    let mut combinations = Vec::new();
    let cooking = name("roles", "COOKING")?;
    let prep = name("roles", "INGREDIENTS")?;
    let finish = name("roles", "FINISH")?;
    let oven = name("roles", "OVEN")?;
    let stove = name("roles", "STOVETOP")?;
    let scored = score["events"]
        .as_array()
        .ok_or("kitchen original effects")?;
    let mut oven_loans = Vec::new();
    let mut stove_loans = Vec::new();
    let mut intervals = Vec::new();
    for (chef, letter, dish, ingredient, recipient, received, io) in recipes {
        let actor = name("roles", chef)?;
        let io = name("roles", io)?;
        for (tool, args, request_name, reply_name, service) in [
            (
                "prepare_base_dish",
                json!({"dish":dish}),
                format!("BaseIntent{letter}"),
                String::new(),
                cooking,
            ),
            (
                "prepare_ingredient",
                json!({"ingredient":ingredient,"for_chef":recipient}),
                format!(
                    "Prepare{}",
                    match letter {
                        "A" => "Sauce",
                        "B" => "Garnish",
                        _ => "Glaze",
                    }
                ),
                format!(
                    "{}Prepared",
                    match letter {
                        "A" => "Sauce",
                        "B" => "Garnish",
                        _ => "Glaze",
                    }
                ),
                prep,
            ),
            (
                "combine_dish",
                json!({"dish":dish,"ingredient":received}),
                format!("Combine{letter}"),
                format!("Dish{letter}"),
                finish,
            ),
        ] {
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
                f.direction(actor, service) && f.logical_label == request_label
            })?;
            if request.value()? != args || request.received()? >= call.emitted {
                return Err("effect before its actual kitchen request".into());
            }
            close_arguments(
                &frames,
                &mut closes,
                actor,
                io,
                request.sent,
                &format!("service/{tool}"),
                &args,
            )?;
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
                .ok_or("original world differs from the kitchen effect")?;
            scored_used.insert(score_index);
            let (start_at, start) = call.started()?;
            let (end_at, end) = call.ended()?;
            if tool == "prepare_base_dish" {
                for (resource, request_name, loan_name, response_name, return_name) in [
                    (
                        oven,
                        format!("OvenRequest{letter}"),
                        "OvenLoan",
                        "OvenResult",
                        "ReturnOven",
                    ),
                    (
                        stove,
                        format!("StoveRequest{letter}"),
                        "StoveLoan",
                        "StoveResult",
                        "ReturnStove",
                    ),
                ] {
                    if letter == "C" && resource == stove {
                        continue;
                    }
                    let loan_label = name("labels", loan_name)? as u8;
                    let actual_label = name("labels", &request_name)? as u8;
                    let response_label = name("labels", response_name)? as u8;
                    let return_label = name("labels", return_name)? as u8;
                    let (_, loan) = take(&frames, &mut used, |f| {
                        f.direction(cooking, actor) && f.logical_label == loan_label
                    })?;
                    let (_, actual) = take(&frames, &mut used, |f| {
                        f.direction(resource, cooking)
                            && f.logical_label == actual_label
                            && f.value().is_ok_and(|value| value == args)
                    })?;
                    let (_, response) = take(&frames, &mut used, |f| {
                        f.direction(cooking, resource)
                            && f.logical_label == response_label
                            && f.value().is_ok_and(|value| value == *result)
                    })?;
                    let (_, returned) = take(&frames, &mut used, |f| {
                        f.direction(actor, cooking) && f.logical_label == return_label
                    })?;
                    if request.received()? >= loan.sent
                        || loan.received()? >= actual.sent
                        || actual.received()? >= call.emitted
                        || response.sent <= result_at
                        || response.received()? >= returned.sent
                    {
                        return Err("resource admission, result or return order differs".into());
                    }
                    if resource == oven {
                        oven_loans.push((loan.sent, returned.received()?));
                    } else {
                        stove_loans.push((loan.sent, returned.received()?));
                    }
                }
                bases.insert(chef, (score_index, end, request.sent));
                intervals.push((chef, start, end));
            } else {
                let reply_label = name("labels", &reply_name)? as u8;
                let (_, reply) = take(&frames, &mut used, |f| {
                    f.direction(service, actor) && f.logical_label == reply_label
                })?;
                if reply.value()? != *result || reply.sent <= result_at {
                    return Err("kitchen reply differs from actual result".into());
                }
                reply.received()?;
                if tool == "prepare_ingredient" {
                    let ready_name = match letter {
                        "A" => "Sauce",
                        "B" => "Garnish",
                        _ => "Glaze",
                    };
                    let consumer = name("roles", recipient)?;
                    let ready_label = name("labels", ready_name)? as u8;
                    let (_, ready) = take(&frames, &mut used, |f| {
                        f.direction(prep, consumer) && f.logical_label == ready_label
                    })?;
                    if ready.value()? != *result || ready.sent <= result_at {
                        return Err(
                            "consumer material differs from the completed ingredient".into()
                        );
                    }
                    ingredients.insert(ingredient, (score_index, end, ready.received()?));
                } else {
                    combinations.push((
                        chef,
                        received,
                        start,
                        start_at,
                        request.sent,
                        result.clone(),
                        score_index,
                        end_at,
                    ));
                }
            }
        }
    }
    for (chef, ingredient, start, _start_at, sent, result, score_index, _end_at) in combinations {
        let (base_index, base_end, _) = bases[chef];
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
            return Err(
                "dish does not use its actual completed base and delivered ingredient".into(),
            );
        }
    }
    for loans in [&mut oven_loans, &mut stove_loans] {
        loans.sort_unstable();
        if loans.windows(2).any(|pair| pair[0].1 >= pair[1].0) {
            return Err("physical equipment authority overlaps".into());
        }
    }
    if intervals.iter().enumerate().any(|(i, (_, start, end))| {
        intervals[i + 1..]
            .iter()
            .any(|(_, a, b)| start < b && a < end)
    }) {
        return Err("actual oven effects overlap".into());
    }
    let business = compiled["labels"]
        .as_object()
        .ok_or("kitchen labels")?
        .values()
        .map(|v| v.as_u64().ok_or("kitchen label"))
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    if scored_used.len() != scored.len()
        || frames.iter().any(|(id, frame)| {
            business.contains(&u64::from(frame.logical_label)) && !used.contains(id)
        })
    {
        return Err("unmatched kitchen effect or business occurrence".into());
    }
    let json_ids = used
        .iter()
        .copied()
        .filter(|id| frames[id].value().is_ok_and(|v| v.is_object()))
        .collect::<Vec<_>>();
    let owned_ids = used
        .iter()
        .copied()
        .filter(|id| frames[id].bytes.len() == 3)
        .collect::<Vec<_>>();
    Ok(
        json!({"status":"passed","task":"12E","domain_calls":calls.len(),"dishes":3,"violations":0,"simulator_complete":score["complete"],
        "matched_business_frame_ids":used,"matched_json_frame_ids":json_ids,"owned_frame_ids":owned_ids,
        "effect_scope":"unique actual WASI/resource/material/result receipts and original completed world inputs; EPF separately checks dynamic source execution"}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("kitchen event directory")?
            .join("projection.json"),
    )?);
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let baseline = check(&rows, &artifact)?;
    let ids = baseline["matched_json_frame_ids"]
        .as_array()
        .ok_or("JSON receipts")?
        .iter()
        .map(|v| v.as_u64().ok_or_else(|| "frame ID".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    let owned = baseline["owned_frame_ids"]
        .as_array()
        .ok_or("owned receipts")?;
    for id in owned {
        for kind in [
            "missing_owned_frame",
            "duplicate_owned_frame",
            "wrong_resource_key",
        ] {
            let mut changed = rows.clone();
            if kind == "missing_owned_frame" {
                changed.retain(|row| {
                    !(row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == *id)
                });
            } else if kind == "duplicate_owned_frame" {
                let duplicate = changed
                    .iter()
                    .filter(|row| {
                        row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == *id
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                changed.extend(duplicate);
            } else {
                for row in &mut changed {
                    let m = &mut row["message"];
                    if m["type"] == "protocol_frame" && m["frame_id"] == *id {
                        m["payload"][0] =
                            json!(m["payload"][0].as_u64().ok_or("resource key")? ^ 1);
                    }
                }
            }
            let error = check(&changed, &artifact)
                .err()
                .ok_or("accepted malformed owned receipt")?;
            checked
                .push(json!({"kind":kind,"frame_id":id,"rejected":true,"error":error.to_string()}));
        }
    }
    for field in ["frame_label", "lane"] {
        let id = ids[0];
        let mut changed = rows.clone();
        for row in &mut changed {
            let m = &mut row["message"];
            if m["type"] == "protocol_frame" && m["frame_id"] == id {
                m[field] = json!(
                    m[field].as_u64().ok_or("wire field")?
                        ^ if field == "lane" { 255 } else { 65535 }
                );
            }
        }
        if check(&changed, &artifact).is_ok() {
            return Err("accepted coherently changed kitchen header".into());
        }
        checked
            .push(json!({"kind":format!("coherent_wrong_{field}"),"frame_id":id,"rejected":true}));
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,"scope":"record mutations, not runtime attacks"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
