//! Offline actual-data/receipt correspondence; EPF alone interprets the source.
use crate::{
    evidence::{Result, save},
    journal::records,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
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
    let journal = crate::journal::read(rows, &frame_labels, actors, dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if calls.is_empty()
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete cascading build execution".into());
    }
    let mut used = BTreeSet::new();
    let mut file_requests = 0;
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
            .position(|event| call.matches_score(event, args))
            .ok_or("score differs from actual build effect")?;
        scored.remove(matched);
        if args.as_object().is_none_or(|fields| fields.len() != 1)
            || result["tool_name"] != tool
            || result["data"]["agent"] != agent
        {
            return Err("build effect arguments or actual result differ".into());
        }
        let (source, target, request, file_role, index) = match (agent, tool) {
            ("BUILDER_A", "compile_module") => {
                (role(agent)?, compiler, "CompileA", "A_FILE", Some(0))
            }
            ("BUILDER_B", "compile_module") => {
                (role(agent)?, compiler, "CompileB", "B_FILE", Some(1))
            }
            ("BUILDER_C", "compile_module") => {
                (role(agent)?, compiler, "CompileC", "C_FILE", Some(2))
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
                    "TESTER_FILE",
                    Some(index),
                )
            }
            ("INTEGRATOR", "link_modules") => {
                (integrator, compiler, "Link", "INTEGRATOR_FILE", None)
            }
            _ => return Err("unassigned cascading build effect".into()),
        };
        let request_label = label(request)?;
        let (id, admitted) = frames
            .iter()
            .filter(|(_, f)| {
                f.direction(source, target)
                    && f.logical_label == request_label
                    && f.received < call.emitted
            })
            .max_by_key(|(_, f)| f.received)
            .ok_or("effect has no actual admitted request")?;
        if !used.insert(*id) {
            return Err("effect reuses an earlier admitted request".into());
        }
        let file = role(file_role)?;
        let (file_request, file_reply) = match tool {
            "compile_module" => (
                ["CompileFileA", "CompileFileB", "CompileFileC"][index.ok_or("compile index")?],
                "FileCompiled",
            ),
            "run_build_tests" => (
                ["TestFileA", "TestFileB", "TestFileC"][index.ok_or("test index")?],
                "FileTested",
            ),
            "link_modules" => ("LinkFile", "FileLinked"),
            _ => return Err("unknown actual file operation".into()),
        };
        let request_label = label(file_request)?;
        let (file_id, file_input) = frames
            .iter()
            .filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(file, source)
                    && frame.logical_label == request_label
                    && frame.received < admitted.sent
            })
            .max_by_key(|(_, frame)| frame.received)
            .ok_or("effect lacks its typed file input")?;
        if file_input.value()? != *args || !used.insert(*file_id) {
            return Err("typed file arguments changed or reused".into());
        }
        file_requests += 1;
        let reply_label = label(file_reply)?;
        let (id, reply) = frames
            .iter()
            .filter(|(id, frame)| {
                !used.contains(*id)
                    && frame.direction(source, file)
                    && frame.logical_label == reply_label
                    && frame.sent > at
            })
            .min_by_key(|(_, frame)| frame.sent)
            .ok_or("actual file result missing")?;
        if reply.value()? != *result || !used.insert(*id) {
            return Err("actual operation file result changed or reused".into());
        }
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
                            && f.received < call.emitted
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
                            && f.sent > frame.received
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
    for (recipient, name) in [
        (compiler, "AllForCompiler"),
        (integrator, "AllForIntegrator"),
    ] {
        let aggregate_label = label(name)?;
        let aggregates: Vec<_> = frames
            .iter()
            .filter(|(_, f)| f.direction(service, recipient) && f.logical_label == aggregate_label)
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
    }
    let integrator_file = role("INTEGRATOR_FILE")?;
    let file_repair = label("FileRepair")?;
    let file_ready = label("FileReady")?;
    let choose_repair = label("ChooseRepair")?;
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
        let mut expected = json!({"objects":objects});
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
                .filter(|(at, _, _)| *at > frame.received)
                .min_by_key(|(at, _, _)| *at)
                .ok_or("repair without another actual build")?;
            let index = next.1;
            if current(index) {
                return Err("repair requests an already current checked object".into());
            }
            let eligible: Vec<_> = (0..3)
                .filter(|&i| !current(i))
                .map(|i| modules[i])
                .collect();
            let mut expected_notice = json!({"objects":objects, "tests":tests});
            expected_notice["success"] = json!(false);
            expected_notice["rebuild"] = json!(eligible);
            let mut offers = frames
                .iter()
                .filter(|(_, item)| {
                    item.sent > frame.sent
                        && item.sent < next.0
                        && item.direction(integrator, integrator_file)
                        && item.logical_label == file_repair
                })
                .collect::<Vec<_>>();
            offers.sort_by_key(|(_, item)| item.sent);
            let mut choices = frames
                .iter()
                .filter(|(_, item)| {
                    item.sent > frame.sent
                        && item.sent < next.0
                        && item.direction(integrator_file, integrator)
                        && item.logical_label == choose_repair
                })
                .collect::<Vec<_>>();
            choices.sort_by_key(|(_, item)| item.sent);
            if offers.is_empty() || offers.len() != choices.len() {
                return Err("missing candidate read or shell repair choice".into());
            }
            let attempts = offers.len();
            let mut previous = frame.received;
            for (attempt, ((offer_id, offer), (choice_id, choice))) in
                offers.into_iter().zip(choices).enumerate()
            {
                if offer.value()? != expected_notice
                    || offer.sent <= previous
                    || choice.sent <= offer.received
                    || !used.insert(*offer_id)
                    || !used.insert(*choice_id)
                {
                    return Err(
                        "repair choice differs from the offered candidate or its occurrence".into(),
                    );
                }
                previous = choice.received;
                let requested = std::str::from_utf8(&choice.bytes).ok().map(str::trim);
                if attempt + 1 == attempts {
                    if requested != Some(modules[index]) {
                        return Err(
                            "actual rebuild differs from the integrator shell choice".into()
                        );
                    }
                } else if requested.is_some_and(|name| eligible.contains(&name)) {
                    return Err("an eligible shell repair choice was discarded".into());
                } else {
                    expected_notice["error"] = json!("choose a module listed in rebuild");
                }
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
                            && f.sent > previous
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
    let (objects, tests) = snapshot(ready.sent)?;
    let proof = json!({"success":true,"objects":objects,"tests":tests});
    let file_link = label("LinkFile")?;
    let candidates = frames
        .iter()
        .filter(|(_, frame)| {
            frame.direction(integrator, integrator_file)
                && frame.logical_label == file_ready
                && frame.sent > ready.sent
        })
        .collect::<Vec<_>>();
    let (id, candidate) = *candidates.first().ok_or("actual candidate file missing")?;
    let request = frames
        .values()
        .find(|frame| {
            frame.direction(integrator_file, integrator) && frame.logical_label == file_link
        })
        .ok_or("link file input missing")?;
    if candidates.len() != 1
        || candidate.value()? != proof
        || candidate.sent <= ready.received
        || candidate.received >= request.sent
        || !used.insert(*id)
    {
        return Err(
            "candidate file differs from the actual checked data or request occurrence".into(),
        );
    }

    let finish_label = label("FinishChecks")?;
    let (id, finish) = frames
        .iter()
        .find(|(_, f)| f.direction(integrator, compiler) && f.logical_label == finish_label)
        .ok_or("final actual checks missing")?;
    if finish.value()? != proof || finish.sent <= ready.received || !used.insert(*id) {
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
            || candidates[0].1.sent <= finish.received
            || candidates[0].1.received >= links[0].emitted
        {
            return Err("link begins before actual editing Endpoint return".into());
        }
    }
    let update = label("FileUpdated")?;
    let feedback = label("FileFeedback")?;
    let reconsidered = label("FileReconsidered")?;
    let finished = label("FileFinished")?;
    let object_notice = label("ObjectForTesting")?;
    let builder_roles = agents
        .iter()
        .map(|agent| role(agent))
        .collect::<Result<Vec<_>>>()?;
    let notice_inputs = [
        "UpdateA",
        "UpdateB",
        "UpdateC",
        "Feedback",
        "RetryA",
        "RetryB",
        "RetryC",
        "RetryAToB",
        "RetryAToC",
        "RetryBToA",
        "RetryBToC",
        "RetryCToA",
        "RetryCToB",
        "FinishedA",
        "FinishedB",
        "FinishedC",
    ]
    .into_iter()
    .map(label)
    .collect::<Result<BTreeSet<_>>>()?;
    let object_inputs = [
        "TestObjectA",
        "TestObjectB",
        "TestObjectC",
        "FinishedTester",
    ]
    .into_iter()
    .map(label)
    .collect::<Result<BTreeSet<_>>>()?;
    for frame in frames.values() {
        let source = frame.identity["source"].as_u64().ok_or("frame source")?;
        let target = frame.identity["target"].as_u64().ok_or("frame target")?;
        let forwarded = if notice_inputs.contains(&frame.logical_label)
            && [compiler, service, integrator].contains(&source)
            && builder_roles.contains(&target)
        {
            let index = builder_roles
                .iter()
                .position(|role| *role == target)
                .ok_or("builder")?;
            Some((
                target,
                role(["A_FILE", "B_FILE", "C_FILE"][index])?,
                if frame.logical_label == label("Feedback")? {
                    feedback
                } else if [label("RetryA")?, label("RetryB")?, label("RetryC")?]
                    .contains(&frame.logical_label)
                {
                    reconsidered
                } else if [
                    label("FinishedA")?,
                    label("FinishedB")?,
                    label("FinishedC")?,
                ]
                .contains(&frame.logical_label)
                {
                    finished
                } else {
                    update
                },
            ))
        } else if object_inputs.contains(&frame.logical_label)
            && [compiler, integrator].contains(&source)
            && target == tester
        {
            Some((tester, role("TESTER_FILE")?, object_notice))
        } else {
            None
        };
        if let Some((source, target, label)) = forwarded {
            let value = frame.value()?;
            let received = frame.received;
            let (id, _) = frames
                .iter()
                .filter(|(id, reply)| {
                    !used.contains(*id)
                        && reply.direction(source, target)
                        && reply.logical_label == label
                        && reply.sent > received
                        && reply.value().ok().as_ref() == Some(&value)
                })
                .min_by_key(|(_, reply)| reply.sent)
                .ok_or("actual incoming file data missing")?;
            if !used.insert(*id) {
                return Err("incoming file data reused".into());
            }
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
        "A_FILE",
        "B_FILE",
        "C_FILE",
        "INTEGRATOR_FILE",
        "TESTER_FILE",
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
        json!({"dynamic_protocol_checked":true,"status":"passed","task":"10M","domain_calls":calls.len(),"violations":0,
        "simulator_complete":score["complete"],
        "matched_effect_frame_ids":ids,"matched_typed_file_requests":file_requests,"dynamic_protocol":dynamic,
        "effect_scope":"unique actual admitted, resource, test, candidate and link occurrences; editing Endpoint return before link"}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    // Establish the complete baseline once. Mutations retain this checked
    // source execution and independently challenge effect/argument matching.
    let baseline = crate::journal::Evidence::open(path)?.audit(check)?;
    let (artifact, _) = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = &baseline["dynamic_protocol"];
    let ids = baseline["matched_effect_frame_ids"]
        .as_array()
        .ok_or("effect frame IDs")?
        .iter()
        .map(|id| id.as_u64().ok_or("effect frame ID".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = crate::journal::json_frame_mutants(&rows, &ids, |changed| {
        check(changed, &artifact, dynamic)
    })?;
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
        let error = check(&changed, &artifact, dynamic)
            .err()
            .ok_or_else(|| format!("accepted {kind}"))?;
        checked.push(json!({"kind":kind,"rejected":true,"error":error.to_string()}));
    }
    // Preserve every transport observation of the write while substituting another module.
    let choice_label = artifact["compiled"]["labels"]["ChooseRepair"]
        .as_u64()
        .ok_or("choice label")?;
    let (choice, replacement) = sends
        .iter()
        .find_map(|frame| {
            let id = frame["frame_id"].as_u64()?;
            if labels.get(&id).copied().map(u64::from) != Some(choice_label) {
                return None;
            }
            ["module_a", "module_b", "module_c"]
                .into_iter()
                .find_map(|name| {
                    let bytes = format!("{name}\n");
                    (frame["payload"] == json!(bytes.as_bytes())).then_some((
                        id,
                        if name == "module_a" {
                            b"module_b\n"
                        } else {
                            b"module_a\n"
                        },
                    ))
                })
        })
        .ok_or("scripted shell repair choice")?;
    let mut changed = rows.clone();
    for row in &mut changed {
        if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == choice {
            row["message"]["payload"] = json!(replacement);
        }
    }
    let error = check(&changed, &artifact, dynamic)
        .err()
        .ok_or("accepted substituted repair choice")?;
    checked.push(
        json!({"kind":"substituted_shell_repair_choice","rejected":true,"error":error.to_string()}),
    );
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == ids[0] {
                row["message"][field] = json!(65535);
            }
        }
        let error = check(&changed, &artifact, dynamic)
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
