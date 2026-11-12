//! Offline actual-data/receipt correspondence; EPF alone interprets the source.
use crate::{
    evidence::{Result, digest, save},
    journal::{check_close_pairs, close_arguments, records},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty build record")?["message"];
    if compiled["task"] != "10M" || fixture["task"] != "10M" || fixture["type"] != "fixtures" {
        return Err("cascading build task differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("build participants")?;
    let agents = ["BUILDER_A", "BUILDER_B", "BUILDER_C"];
    let modules = ["module_a", "module_b", "module_c"];
    if actors.len() != 5
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("BUILDER_A"),
                Some("BUILDER_B"),
                Some("BUILDER_C"),
                Some("TEST_RUNNER"),
                Some("INTEGRATOR"),
            ])
    {
        return Err("cascading build participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let role = |key: &str| name("roles", key);
    let label = |key: &str| name("labels", key).map(|v| v as u8);
    let compiler = role("COMPILER")?;
    let service = role("TEST_SERVICE")?;
    let tester = role("TEST_RUNNER")?;
    let integrator = role("INTEGRATOR")?;
    let resource_names = [
        ["CORE_LIB", "CONFIG"],
        ["CORE_LIB", "DATA_MODELS"],
        ["CONFIG", "DATA_MODELS"],
    ];
    let request_names = [
        ["CompileCore", "CompileConfig"],
        ["CompileCore", "CompileModels"],
        ["CompileConfig", "CompileModels"],
    ];
    let reply_names = [
        ["CoreBuilt", "ConfigBuilt"],
        ["CoreBuilt", "ModelsBuilt"],
        ["ConfigBuilt", "ModelsBuilt"],
    ];
    let frame_labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &frame_labels, actors)?;
    if calls.is_empty()
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete cascading build execution".into());
    }
    let io_names = ["A_IO", "B_IO", "C_IO", "TESTER_IO", "INTEGRATOR_IO"];
    let pairs = [agents[0], agents[1], agents[2], "TEST_RUNNER", "INTEGRATOR"]
        .into_iter()
        .zip(io_names)
        .map(|(agent, io)| Ok((role(agent)?, role(io)?)))
        .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut delivered_results = BTreeMap::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("build score events")?
        .clone();
    let mut ordered: Vec<_> = calls.values().collect();
    ordered.sort_by_key(|call| call.emitted);
    let mut objects = BTreeMap::<String, Value>::new();
    let mut object_ids = BTreeSet::new();
    let mut tested_objects = BTreeSet::new();
    let mut receipts = BTreeSet::new();
    let mut compile_history = Vec::<(usize, usize, Value)>::new();
    let mut test_history = Vec::<(usize, usize, Value)>::new();
    let mut links = Vec::new();
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("effect agent")?;
        let tool = call.tool()?;
        let args = &call.request["arguments"];
        let (at, result) = call.result()?;
        let matched = scored
            .iter()
            .position(|event| {
                event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == *args
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("score differs from actual build effect")?;
        scored.remove(matched);
        if args.as_object().is_none_or(|fields| fields.len() != 1)
            || result["tool_name"] != tool
            || result["data"]["agent"] != agent
        {
            return Err("build effect arguments or actual result differ".into());
        }
        let (source, target, request, io, index) = match (agent, tool) {
            ("BUILDER_A", "compile_module") => {
                (role(agent)?, compiler, "CompileA", "A_IO", Some(0))
            }
            ("BUILDER_B", "compile_module") => {
                (role(agent)?, compiler, "CompileB", "B_IO", Some(1))
            }
            ("BUILDER_C", "compile_module") => {
                (role(agent)?, compiler, "CompileC", "C_IO", Some(2))
            }
            ("TEST_RUNNER", "run_build_tests") => {
                let index = modules
                    .iter()
                    .position(|module| {
                        args["module_name"]
                            .as_str()
                            .is_some_and(|s| s.trim().to_lowercase().replace(' ', "_") == *module)
                    })
                    .ok_or("unassigned test module")?;
                (
                    tester,
                    service,
                    ["TestA", "TestB", "TestC"][index],
                    "TESTER_IO",
                    Some(index),
                )
            }
            ("INTEGRATOR", "link_modules") => (integrator, compiler, "Link", "INTEGRATOR_IO", None),
            _ => return Err("unassigned cascading build effect".into()),
        };
        let request_label = label(request)?;
        let (id, admitted) = frames
            .iter()
            .filter(|(_, f)| {
                f.direction(source, target)
                    && f.logical_label == request_label
                    && f.received.is_some_and(|before| before < call.emitted)
            })
            .max_by_key(|(_, f)| f.received)
            .ok_or("effect has no actual admitted request")?;
        if !used.insert(*id) {
            return Err("effect reuses an earlier admitted request".into());
        }
        let previous_closes = closes.clone();
        close_arguments(
            &frames,
            &mut closes,
            source,
            role(io)?,
            admitted.sent,
            &format!("service/{tool}"),
            args,
        )?;
        let close = *closes
            .difference(&previous_closes)
            .next()
            .ok_or("missing unique close occurrence")?;
        delivered_results.insert(close, result.clone());
        let admitted_value = admitted.value()?;
        if tool == "compile_module" {
            let index = index.ok_or("compile module index")?;
            let supplied = args["module_name"]
                .as_str()
                .ok_or("compile argument")?
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            if supplied != modules[index]
                || admitted_value != *args
                || result["success"] != true
                || result["data"]["module"] != modules[index]
                || result["data"]["status"] != "compiled"
            {
                return Err("compile names a different actual module".into());
            }
            let object = &result["data"]["object"];
            let bytes = object["bytes"].as_str().ok_or("actual object bytes")?;
            let content: Value = serde_json::from_str(bytes)?;
            if object["builder"] != agent
                || object["module"] != modules[index]
                || object["inputs"].as_object().is_none_or(|inputs| {
                    inputs.len() != 2
                        || resource_names[index]
                            .iter()
                            .any(|name| !inputs.contains_key(*name))
                })
                || content != json!({"module":modules[index],"inputs":object["inputs"]})
                || object["content_sha256"] != format!("{:x}", Sha256::digest(bytes.as_bytes()))
                || !object_ids.insert(object["id"].as_str().ok_or("object occurrence")?.to_owned())
            {
                return Err("actual object input, occurrence or bytes differ".into());
            }
            for pair in 0..2 {
                let resource = role(resource_names[index][pair])?;
                let request_label = label(request_names[index][pair])?;
                let (id, frame) = frames
                    .iter()
                    .filter(|(_, f)| {
                        f.direction(resource, compiler)
                            && f.logical_label == request_label
                            && f.received.is_some_and(|before| before < call.emitted)
                    })
                    .max_by_key(|(_, f)| f.received)
                    .ok_or("compile lacks an actual resource request")?;
                if frame.value()? != *args || !used.insert(*id) {
                    return Err("resource request altered or reused".into());
                }
                let reply_label = label(reply_names[index][pair])?;
                let (id, frame) = frames
                    .iter()
                    .filter(|(_, f)| {
                        f.direction(compiler, resource)
                            && f.logical_label == reply_label
                            && f.sent > at
                    })
                    .min_by_key(|(_, f)| f.sent)
                    .ok_or("actual resource reply missing")?;
                if frame.value()? != *result || !used.insert(*id) {
                    return Err("resource reply altered or reused".into());
                }
                frame.received()?;
            }
            let report_label = label(["TestObjectA", "TestObjectB", "TestObjectC"][index])?;
            let (id, frame) = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(compiler, tester) && f.logical_label == report_label && f.sent > at
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or("actual completed build report missing")?;
            if frame.value()? != *result || !used.insert(*id) {
                return Err("build report altered or reused".into());
            }
            frame.received()?;
            if result["data"]["artifacts_modified"] == true {
                for (other, target_name) in agents
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                {
                    let target = role(target_name)?;
                    let update_label = label(["UpdateA", "UpdateB", "UpdateC"][other])?;
                    let (id, frame) = frames
                        .iter()
                        .filter(|(_, f)| {
                            f.direction(compiler, target)
                                && f.logical_label == update_label
                                && f.sent > at
                        })
                        .min_by_key(|(_, f)| f.sent)
                        .ok_or("actual shared input update missing")?;
                    if frame.value()? != *result || !used.insert(*id) {
                        return Err("shared input update altered or reused".into());
                    }
                    frame.received()?;
                }
            }
            objects.insert(modules[index].into(), object.clone());
            compile_history.push((at, index, result.clone()));
        } else if tool == "run_build_tests" {
            let index = index.ok_or("test module index")?;
            let object = objects
                .get(modules[index])
                .ok_or("test before actual build")?;
            if admitted_value != json!({"arguments":args,"object":object})
                || result["data"]["module"] != modules[index]
                || !result["success"].is_boolean()
                || result["data"]["input_object"] != *object
                || !tested_objects
                    .insert(object["id"].as_str().ok_or("tested object ID")?.to_owned())
                || !receipts.insert(
                    result["data"]["receipt"]
                        .as_str()
                        .ok_or("actual test receipt")?
                        .to_owned(),
                )
            {
                return Err("test does not consume its fresh actual submitted object".into());
            }
            for (target, reply) in [(tester, "Tested"), (role(agents[index])?, "Feedback")] {
                let reply_label = label(reply)?;
                let (id, frame) = frames
                    .iter()
                    .filter(|(_, f)| {
                        f.direction(service, target)
                            && f.logical_label == reply_label
                            && f.sent > at
                    })
                    .min_by_key(|(_, f)| f.sent)
                    .ok_or("actual test result delivery missing")?;
                if frame.value()? != *result || !used.insert(*id) {
                    return Err("test result delivery altered or reused".into());
                }
                frame.received()?;
            }
            test_history.push((at, index, result.clone()));
        } else {
            if admitted_value != *args || result["success"] != true {
                return Err("link input altered".into());
            }
            let mut names = args["modules"]
                .as_str()
                .ok_or("linked modules")?
                .split(',')
                .map(|name| name.trim().to_lowercase().replace(' ', "_"))
                .collect::<Vec<_>>();
            names.sort_unstable();
            if names != modules || objects.len() != 3 {
                return Err("link lacks all actual current modules".into());
            }
            let mut checks = BTreeMap::new();
            for (index, module) in modules.iter().enumerate() {
                let test = test_history
                    .iter()
                    .rev()
                    .find(|(_, i, _)| *i == index)
                    .ok_or("link before actual test")?;
                if test.2["success"] != true || test.2["data"]["input_object"] != objects[*module] {
                    return Err("link reuses an obsolete or negative test".into());
                }
                checks.insert(
                    *module,
                    json!({"run_build_tests":test.2["data"]["receipt"]}),
                );
                for resource in resource_names[index] {
                    for other in objects
                        .values()
                        .filter(|obj| obj["inputs"].get(resource).is_some())
                    {
                        if other["inputs"][resource] != objects[*module]["inputs"][resource] {
                            return Err(
                                "linked objects disagree about current shared inputs".into()
                            );
                        }
                    }
                }
            }
            let inputs = json!(objects);
            let bytes = result["data"]["binary"]["bytes"]
                .as_str()
                .ok_or("actual linked binary bytes")?;
            if result["data"]["status"] != "linked"
                || result["data"]["issues"] != json!([])
                || result["data"]["inputs"] != inputs
                || result["data"]["checks"] != json!(checks)
                || serde_json::from_str::<Value>(bytes)? != inputs
            {
                return Err("linked binary differs from the actual current checked objects".into());
            }
            let linked_label = label("Linked")?;
            let (id, frame) = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(compiler, integrator)
                        && f.logical_label == linked_label
                        && f.sent > at
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or("actual link result missing")?;
            if frame.value()? != *result || !used.insert(*id) {
                return Err("actual link result altered or reused".into());
            }
            for (target, reply) in [
                (role(agents[0])?, "FinishedA"),
                (role(agents[1])?, "FinishedB"),
                (role(agents[2])?, "FinishedC"),
                (tester, "FinishedTester"),
                (service, "FinishedService"),
            ] {
                let reply_label = label(reply)?;
                let (id, frame) = frames
                    .iter()
                    .filter(|(_, f)| {
                        f.direction(integrator, target)
                            && f.logical_label == reply_label
                            && f.sent > frame.received.unwrap_or(at)
                    })
                    .min_by_key(|(_, f)| f.sent)
                    .ok_or("final actual link notification missing")?;
                if frame.value()? != *result || !used.insert(*id) {
                    return Err("final link notification altered or reused".into());
                }
            }
            links.push(call);
        }
    }
    if !scored.is_empty()
        || links.len() != 1
        || ordered
            .last()
            .is_none_or(|last| last.tool().ok() != Some("link_modules"))
    {
        return Err("extra or missing actual build/test/link effect".into());
    }
    let snapshot = |before: usize| -> Result<(Vec<Value>, Vec<Value>)> {
        let mut objects = Vec::new();
        let mut tests = Vec::new();
        for index in 0..3 {
            objects.push(
                compile_history
                    .iter()
                    .rev()
                    .find(|(at, i, _)| *at < before && *i == index)
                    .ok_or("candidate before actual build")?
                    .2["data"]["object"]
                    .clone(),
            );
            tests.push(
                test_history
                    .iter()
                    .rev()
                    .find(|(at, i, _)| *at < before && *i == index)
                    .ok_or("candidate before actual test")?
                    .2
                    .clone(),
            );
        }
        Ok((objects, tests))
    };
    let aggregate_label = label("AllForCompiler")?;
    let aggregates: Vec<_> = frames
        .iter()
        .filter(|(_, f)| f.direction(service, compiler) && f.logical_label == aggregate_label)
        .collect();
    if aggregates.len()
        != test_history
            .len()
            .checked_sub(2)
            .ok_or("initial test bundle missing")?
    {
        return Err("missing or extra test receipt bundle".into());
    }
    for (id, frame) in aggregates {
        let (_, tests) = snapshot(frame.sent)?;
        if frame.value()? != json!(tests) || !used.insert(*id) {
            return Err("aggregate reuses or changes actual tests".into());
        }
    }
    let repair_label = label("RepairCandidate")?;
    let ready_label = label("ReadyCandidate")?;
    let mut ready = Vec::new();
    for (id, frame) in frames.iter().filter(|(_, f)| {
        f.direction(compiler, integrator) && [repair_label, ready_label].contains(&f.logical_label)
    }) {
        let (objects, tests) = snapshot(frame.sent)?;
        let value = frame.value()?;
        let current = |index: usize| {
            tests[index]["success"] == true
                && tests[index]["data"]["input_object"] == objects[index]
                && objects[index]["inputs"].as_object().is_some_and(|inputs| {
                    inputs.iter().all(|(resource, value)| {
                        compile_history
                            .iter()
                            .filter(|(at, _, result)| {
                                *at < frame.sent
                                    && result["data"]["object"]["inputs"].get(resource).is_some()
                            })
                            .max_by_key(|(at, _, _)| *at)
                            .is_some_and(|(_, _, latest)| {
                                latest["data"]["object"]["inputs"][resource] == *value
                            })
                    })
                })
        };
        let mut expected = json!({"objects":objects,"tests":tests});
        if frame.logical_label == ready_label {
            if (0..3).any(|index| !current(index)) {
                return Err("ready candidate has stale or failed actual checks".into());
            }
            expected["success"] = json!(true);
            ready.push(frame);
        }
        if value != expected || !used.insert(*id) {
            return Err("candidate differs from actual object/test occurrences".into());
        }
        if frame.logical_label == repair_label {
            let next = compile_history
                .iter()
                .filter(|(at, _, _)| *at > frame.received.unwrap_or(frame.sent))
                .min_by_key(|(at, _, _)| *at)
                .ok_or("repair without another actual build")?;
            let index = next.1;
            if current(index) {
                return Err("repair requests an already current checked object".into());
            }
            let request =
                json!({"module":modules[index],"object":objects[index],"test":tests[index]});
            let directives = match index {
                0 => [
                    (role(agents[0])?, "RetryA"),
                    (compiler, "RetryAForCompiler"),
                    (role(agents[1])?, "RetryAToB"),
                    (role(agents[2])?, "RetryAToC"),
                ],
                1 => [
                    (role(agents[1])?, "RetryB"),
                    (compiler, "RetryBForCompiler"),
                    (role(agents[0])?, "RetryBToA"),
                    (role(agents[2])?, "RetryBToC"),
                ],
                _ => [
                    (role(agents[2])?, "RetryC"),
                    (compiler, "RetryCForCompiler"),
                    (role(agents[0])?, "RetryCToA"),
                    (role(agents[1])?, "RetryCToB"),
                ],
            };
            for (target, name) in directives {
                let selected_label = label(name)?;
                let (id, directive) = frames
                    .iter()
                    .filter(|(_, f)| {
                        f.direction(integrator, target)
                            && f.logical_label == selected_label
                            && f.sent > frame.received.unwrap_or(frame.sent)
                            && f.sent < next.0
                    })
                    .min_by_key(|(_, f)| f.sent)
                    .ok_or("repair context delivery missing")?;
                if directive.value()? != request || !used.insert(*id) {
                    return Err("repair context altered or reused".into());
                }
            }
        }
    }
    if ready.len() != 1 {
        return Err("ready candidate missing or duplicated".into());
    }
    let ready = ready[0];
    let finish_label = label("FinishChecks")?;
    let (id, finish) = frames
        .iter()
        .find(|(_, f)| f.direction(integrator, compiler) && f.logical_label == finish_label)
        .ok_or("final actual checks missing")?;
    if finish.value()? != ready.value()? || finish.sent <= ready.received()? || !used.insert(*id) {
        return Err("final checks changed the actual ready candidate".into());
    }
    for (resource, name) in [
        ("CORE_LIB", "FrozenCore"),
        ("CONFIG", "FrozenConfig"),
        ("DATA_MODELS", "FrozenModels"),
    ] {
        let frozen_label = label(name)?;
        let candidates = frames
            .iter()
            .filter(|(_, f)| f.direction(compiler, integrator) && f.logical_label == frozen_label)
            .collect::<Vec<_>>();
        if candidates.len() != 1
            || candidates[0].1.bytes.first() != Some(&(role(resource)? as u8))
            || candidates[0].1.sent <= finish.received()?
            || candidates[0].1.received()? >= links[0].emitted
        {
            return Err("link begins before actual editing Endpoint return".into());
        }
    }
    // Argument-file closes may legitimately be rejected. Every accepted tool
    // call must return its actual result bytes; unmatched closes must return a
    // recoverable rejection, never an invented success or missing response.
    let mut result_chunks = BTreeSet::new();
    for &(vm, io) in &pairs {
        let mut closed: Vec<_> = frames
            .values()
            .filter(|f| {
                f.direction(io, vm)
                    && f.logical_label == hibana_wasip1_runtime::protocol::LABEL_WASI_FD_CLOSE_RET
                    && f.bytes.first() == Some(&4)
            })
            .collect();
        closed.sort_by_key(|f| f.sent);
        for (index, close) in closed.iter().enumerate() {
            let until = closed.get(index + 1).map_or(usize::MAX, |f| f.sent);
            let mut chunks: Vec<_> = frames
                .iter()
                .filter(|(_, f)| {
                    f.direction(vm, io)
                        && f.logical_label == 241
                        && f.sent > close.received.unwrap_or(close.sent)
                        && f.sent < until
                })
                .collect();
            chunks.sort_by_key(|(_, f)| f.sent);
            let mut bytes = Vec::new();
            for (id, chunk) in chunks {
                let offset = chunk.bytes.get(..8).ok_or("result chunk offset missing")?;
                if u64::from_be_bytes(offset.try_into()?) != bytes.len() as u64
                    || !result_chunks.insert(*id)
                {
                    return Err("result bytes missing, reordered or reused".into());
                }
                bytes.extend_from_slice(&chunk.bytes[8..]);
            }
            let mut actual: Value = serde_json::from_slice(&bytes)?;
            if let Some(expected) = delivered_results.get(&close.sent) {
                actual
                    .as_object_mut()
                    .ok_or("result must be an object")?
                    .remove("coordination");
                if actual != *expected {
                    return Err("guest result differs from actual effect".into());
                }
            } else if !matches!(
                actual["status"].as_str(),
                Some("invalid_arguments" | "out_of_order")
            ) {
                return Err("unmatched argument close produced a non-rejection".into());
            }
        }
        if frames.iter().any(|(id, f)| {
            f.direction(vm, io) && f.logical_label == 241 && !result_chunks.contains(id)
        }) {
            return Err("unmatched result bytes".into());
        }
    }
    let json_labels = compiled["labels"]
        .as_object()
        .ok_or("business labels")?
        .iter()
        .filter(|(name, _)| {
            !name.starts_with("Loan") && !name.starts_with("Return") && !name.starts_with("Frozen")
        })
        .map(|(_, value)| {
            value
                .as_u64()
                .map(|v| v as u8)
                .ok_or("business label".into())
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let business = [
        "BUILDER_A",
        "BUILDER_B",
        "BUILDER_C",
        "INTEGRATOR",
        "TEST_RUNNER",
        "COMPILER",
        "TEST_SERVICE",
        "CORE_LIB",
        "CONFIG",
        "DATA_MODELS",
    ]
    .into_iter()
    .map(role)
    .collect::<Result<BTreeSet<_>>>()?;
    if frames.iter().any(|(id, f)| {
        json_labels.contains(&f.logical_label)
            && f.identity["source"]
                .as_u64()
                .is_some_and(|v| business.contains(&v))
            && f.identity["target"]
                .as_u64()
                .is_some_and(|v| business.contains(&v))
            && !used.contains(id)
    }) {
        return Err("unmatched or extra actual business request/result".into());
    }
    let ids = used
        .iter()
        .filter(|id| {
            frames[id]
                .value()
                .ok()
                .is_some_and(|value| value.is_object())
        })
        .copied()
        .collect::<Vec<_>>();
    Ok(
        json!({"status":"passed","task":"10M","domain_calls":calls.len(),"violations":0,
        "simulator_complete":score["complete"],
        "matched_effect_frame_ids":ids,"matched_wasi_closes":closes.len(),
        "effect_scope":"unique actual admitted, resource, test, candidate and link occurrences; editing Endpoint return before link"}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("record directory")?
            .join("projection.json")
    )?);
    result["dynamic_protocol_checked_separately"] = json!(true);
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
        .map(|id| id.as_u64().ok_or("effect frame ID".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let sends = rows
        .iter()
        .filter(|row| {
            row["message"]["type"] == "protocol_frame" && row["message"]["action"] == "send"
        })
        .map(|row| &row["message"])
        .collect::<Vec<_>>();
    let decode = |message: &Value| -> Result<Value> {
        let bytes = message["payload"]
            .as_array()
            .ok_or("frame bytes")?
            .iter()
            .map(|v| {
                v.as_u64()
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or("frame byte")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(serde_json::from_slice(&bytes)?)
    };
    let aggregate = sends
        .iter()
        .find(|m| {
            m["frame_id"].as_u64().is_some_and(|id| {
                labels.get(&id).copied().map(u64::from)
                    == artifact["compiled"]["labels"]["AllForCompiler"].as_u64()
            })
        })
        .ok_or("test receipt aggregate")?;
    let mut swapped = decode(aggregate)?;
    swapped.as_array_mut().ok_or("receipt array")?.swap(0, 1);
    let repair_label = artifact["compiled"]["labels"]["RepairCandidate"]
        .as_u64()
        .ok_or("repair label")?;
    let candidates = sends
        .iter()
        .filter(|m| {
            m["frame_id"]
                .as_u64()
                .is_some_and(|id| labels.get(&id).copied().map(u64::from) == Some(repair_label))
        })
        .copied()
        .collect::<Vec<_>>();
    if candidates.len() < 2 {
        return Err("two actual reconsiderations are required by this qualification".into());
    }
    for (kind, frame, value) in [
        ("swapped_actual_test_receipts", *aggregate, swapped),
        (
            "previous_candidate_reused",
            candidates[1],
            decode(candidates[0])?,
        ),
    ] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame"
                && row["message"]["frame_id"] == frame["frame_id"]
            {
                row["message"]["payload"] = json!(serde_json::to_vec(&value)?);
            }
        }
        let error = check(&changed, &artifact)
            .err()
            .ok_or_else(|| format!("accepted {kind}"))?;
        checked.push(json!({"kind":kind,"rejected":true,"error":error.to_string()}));
    }
    let chunk = sends
        .iter()
        .find(|m| {
            m["frame_id"]
                .as_u64()
                .is_some_and(|id| labels.get(&id) == Some(&241))
        })
        .ok_or("actual guest result chunk")?;
    let mut changed = rows.clone();
    for row in &mut changed {
        if row["message"]["type"] == "protocol_frame"
            && row["message"]["frame_id"] == chunk["frame_id"]
        {
            let bytes = row["message"]["payload"]
                .as_array_mut()
                .ok_or("result chunk bytes")?;
            bytes[8] = json!(b'[');
        }
    }
    let error = check(&changed, &artifact)
        .err()
        .ok_or("accepted altered guest result bytes")?;
    checked.push(
        json!({"kind":"coherent_guest_result_change","rejected":true,"error":error.to_string()}),
    );
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(65535);
            }
        }
        let error = check(&changed, &artifact)
            .err()
            .ok_or("accepted coherently changed header")?;
        checked.push(json!({"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,
        "scope":"immutable record mutations, not runtime attacks"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
