//! One-use WASI admissions, actual resource requests and development effects.
//! EPF checks the source execution; this checker never drives an Endpoint.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn completed<'a>(
    calls: &[&'a Call],
    before: usize,
    tool: &str,
    key: &str,
    target: &str,
) -> Result<&'a Call> {
    calls
        .iter()
        .copied()
        .filter(|call| {
            call.request["tool"] == tool
                && call.request["arguments"][key] == target
                && call.result.as_ref().is_some_and(|(at, _)| *at < before)
        })
        .max_by_key(|call| call.emitted)
        .ok_or_else(|| format!("missing actual {tool} for {target}").into())
}

fn receipt(result: &Value) -> Result<u64> {
    result["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .ok_or("missing development occurrence")?
        .parse()
        .map_err(Into::into)
}

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty feature-delivery evidence")?["message"];
    if compiled["task"] != "6M" || fixture["task"] != "6M" || fixture["type"] != "fixtures" {
        return Err("feature-delivery task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("feature-delivery participants")?;
    if actors
        .iter()
        .map(|actor| actor["agent"].as_str())
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            Some("DEVELOPER_A"),
            Some("DEVELOPER_B"),
            Some("REVIEWER"),
            Some("TEST_WRITER"),
            Some("CI_RUNNER"),
        ])
        || actors.len() != 5
    {
        return Err("feature-delivery participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|value| *value <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if score["complete"] != true
        || score["violations"] != json!([])
        || calls.is_empty()
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("feature-delivery completion, effect or session differs".into());
    }
    let pairs = [
        ("DEVELOPER_A", "A_IO"),
        ("DEVELOPER_B", "B_IO"),
        ("TEST_WRITER", "WRITER_IO"),
        ("REVIEWER", "REVIEWER_IO"),
        ("CI_RUNNER", "RUNNER_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((name("roles", vm)?, name("roles", io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut scored = score["events"]
        .as_array()
        .ok_or("development score events")?
        .clone();
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let repository = name("roles", "REPOSITORY")?;
    let repo = name("roles", "REPO")?;
    let ci = name("roles", "CI_SERVICE")?;
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("development actor")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let actor = one(actors.iter().filter(|actor| actor["agent"] == agent))?;
        one(actor["calls"]
            .as_array()
            .ok_or("development assignments")?
            .iter()
            .filter(|assigned| {
                assigned["tool"] == tool
                    && assigned["arguments"].as_object().is_some_and(|object| {
                        object.iter().all(|(key, value)| arguments[key] == *value)
                    })
            }))?;
        let vm = name("roles", agent)?;
        let io = name(
            "roles",
            match agent {
                "DEVELOPER_A" => "A_IO",
                "DEVELOPER_B" => "B_IO",
                "TEST_WRITER" => "WRITER_IO",
                "REVIEWER" => "REVIEWER_IO",
                "CI_RUNNER" => "RUNNER_IO",
                _ => return Err("unknown participant".into()),
            },
        )?;
        let feature = match tool {
            "implement_feature" => arguments["feature_name"].as_str(),
            "merge_code" => arguments["branch"].as_str(),
            "write_tests" => arguments["target"].as_str(),
            "code_review" => arguments["pr_id"].as_str(),
            "run_tests" => match arguments["suite"].as_str() {
                Some("full_test_suite_after_auth") => Some("user_authentication"),
                Some("full_test_suite_after_payment") => Some("payment_processing"),
                _ => None,
            },
            _ => None,
        }
        .ok_or("unknown development operation or target")?;
        let a = feature == "user_authentication";
        if !a && feature != "payment_processing" {
            return Err("unassigned feature".into());
        }
        let authority = name("roles", if a { "AUTH_A" } else { "AUTH_B" })?;
        let review_service = name("roles", if a { "REVIEW_A" } else { "REVIEW_B" })?;
        let test_service = name("roles", if a { "TEST_DATA_A" } else { "TEST_DATA_B" })?;
        let (result_at, result) = call.result()?;
        if result["tool_name"] != tool
            || result["data"]["agent"] != agent
            || !result["success"].is_boolean()
        {
            return Err("actual development result identity differs".into());
        }
        let score_arguments = match tool {
            "implement_feature" => {
                json!({"feature":feature,"source":arguments.get("source").cloned()
                .unwrap_or_else(|| json!(format!("Implementation of {feature}")))})
            }
            "write_tests" => json!({"component":feature}),
            "code_review" => json!({"target":feature}),
            _ => arguments.clone(),
        };
        let at = scored
            .iter()
            .position(|event| {
                event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == score_arguments
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("development effect differs from the independent simulator score")?;
        scored.remove(at);
        // Requests precede the actual effect; result frames follow its original reply.
        let mut expected = Vec::<(u64, u64, &str, Value, bool)>::new();
        match tool {
            "implement_feature" => {
                if result["success"] != true
                    || result["data"]["feature"] != feature
                    || !result["data"]["candidate_sha256"].is_string()
                    || !result["data"]["source"].is_string()
                {
                    return Err("implementation lacks its actual candidate".into());
                }
                expected.extend([
                    (
                        vm,
                        repository,
                        if a { "ImplementA" } else { "ImplementB" },
                        arguments.clone(),
                        false,
                    ),
                    (authority, repository, "Implement", arguments.clone(), false),
                    (
                        repo,
                        repository,
                        "ImplementOnRepo",
                        arguments.clone(),
                        false,
                    ),
                    (repository, repo, "RepoResult", result.clone(), true),
                ]);
            }
            "write_tests" | "code_review" => {
                let candidate = completed(
                    &ordered,
                    call.emitted,
                    "implement_feature",
                    "feature_name",
                    feature,
                )?
                .result()?
                .1;
                if result["data"]["candidate_sha256"] != candidate["data"]["candidate_sha256"]
                    || result["data"]["source_receipt"].as_u64() != Some(receipt(candidate)?)
                {
                    return Err(
                        "test or review receipt differs from the actual candidate occurrence"
                            .into(),
                    );
                }
                if tool == "write_tests" {
                    if result["success"] != true
                        || result["data"]["component"] != feature
                        || !result["data"]["test_source"].is_object()
                        || !result["data"]["test_sha256"].is_string()
                    {
                        return Err("test definition lacks its actual content".into());
                    }
                    expected.extend([
                        (
                            vm,
                            test_service,
                            if a { "TestIntentA" } else { "TestIntentB" },
                            arguments.clone(),
                            false,
                        ),
                        (
                            name("roles", "TEST_ENV")?,
                            test_service,
                            if a { "TestRequestA" } else { "TestRequestB" },
                            arguments.clone(),
                            false,
                        ),
                        (
                            test_service,
                            name("roles", "TEST_ENV")?,
                            if a { "TestResultA" } else { "TestResultB" },
                            result.clone(),
                            true,
                        ),
                    ]);
                } else {
                    if result["data"]["target"] != feature {
                        return Err("review targets another feature".into());
                    }
                    expected.extend([
                        (
                            authority,
                            review_service,
                            "Review",
                            arguments.clone(),
                            false,
                        ),
                        (
                            review_service,
                            authority,
                            if a { "ReviewResultA" } else { "ReviewResultB" },
                            result.clone(),
                            true,
                        ),
                        (
                            review_service,
                            repository,
                            if a { "ReviewResultA" } else { "ReviewResultB" },
                            result.clone(),
                            true,
                        ),
                    ]);
                }
            }
            "merge_code" => {
                let candidate = completed(
                    &ordered,
                    call.emitted,
                    "implement_feature",
                    "feature_name",
                    feature,
                )?
                .result()?
                .1;
                let review = completed(&ordered, call.emitted, "code_review", "pr_id", feature)?
                    .result()?
                    .1;
                if review["success"] != true
                    || result["success"] != true
                    || review["data"]["source_receipt"].as_u64() != Some(receipt(candidate)?)
                    || result["data"]["review_receipt"].as_u64() != Some(receipt(review)?)
                    || result["data"]["source_receipt"].as_u64() != Some(receipt(candidate)?)
                    || result["data"]["source"] != candidate["data"]["source"]
                    || result["data"]["published_candidates"][feature]
                        != candidate["data"]["candidate_sha256"]
                {
                    return Err(
                        "publication differs from its current candidate or actual approval".into(),
                    );
                }
                expected.extend([
                    (
                        vm,
                        repository,
                        if a {
                            "MergeAdmissionA"
                        } else {
                            "MergeAdmissionB"
                        },
                        arguments.clone(),
                        false,
                    ),
                    (authority, repository, "Merge", arguments.clone(), false),
                    (repo, repository, "MergeOnRepo", arguments.clone(), false),
                    (repository, authority, "Merged", result.clone(), true),
                    (repository, ci, "Merged", result.clone(), true),
                ]);
            }
            "run_tests" => {
                let publication =
                    completed(&ordered, call.emitted, "merge_code", "branch", feature)?
                        .result()?
                        .1;
                let mut publication_events = serde_json::Map::new();
                for published in ordered.iter().filter(|published| {
                    published.request["tool"] == "merge_code"
                        && published
                            .result
                            .as_ref()
                            .is_some_and(|(at, _)| *at < call.emitted)
                }) {
                    let value = published.result()?.1;
                    publication_events.insert(
                        value["data"]["feature"]
                            .as_str()
                            .ok_or("published feature")?
                            .to_owned(),
                        value["data"]["publication_event"]
                            .as_u64()
                            .ok_or("publication occurrence")?
                            .into(),
                    );
                }
                if result["data"]["repository_sha256"] != publication["data"]["repository_sha256"]
                    || result["data"]["publication_events"] != Value::Object(publication_events)
                    || result["data"]["publication_event"]
                        != publication["data"]["publication_event"]
                    || result["data"]["published_candidates"]
                        != publication["data"]["published_candidates"]
                {
                    return Err("CI result differs from the actual published repository".into());
                }
                for (target, identity) in publication["data"]["published_candidates"]
                    .as_object()
                    .ok_or("published feature data")?
                {
                    let tests = completed(&ordered, call.emitted, "write_tests", "target", target)?
                        .result()?
                        .1;
                    if tests["data"]["candidate_sha256"] != *identity
                        || result["data"]["test_inputs"][target] != tests["data"]
                    {
                        return Err(
                            "CI inputs differ from actual completed test definitions".into()
                        );
                    }
                }
                expected.extend([
                    (
                        vm,
                        ci,
                        if a { "CiIntentA" } else { "CiIntentB" },
                        arguments.clone(),
                        false,
                    ),
                    (
                        authority,
                        ci,
                        if a {
                            "RunCiOnFeatureA"
                        } else {
                            "RunCiOnFeatureB"
                        },
                        arguments.clone(),
                        false,
                    ),
                    (
                        name("roles", "BUILD_SERVER")?,
                        ci,
                        if a { "RunCiA" } else { "RunCiB" },
                        arguments.clone(),
                        false,
                    ),
                    (
                        ci,
                        name("roles", "BUILD_SERVER")?,
                        "CiResult",
                        result.clone(),
                        true,
                    ),
                    (ci, authority, "CiResult", result.clone(), true),
                    (ci, repository, "CiResult", result.clone(), true),
                ]);
            }
            _ => unreachable!(),
        }
        let mut before = call.emitted;
        for (from, to, key, value, reply) in expected {
            let label = name("labels", key)? as u8;
            let candidates = frames.iter().filter(|(id, frame)| {
                !used_frames.contains(*id)
                    && frame.direction(from, to)
                    && frame.logical_label == label
                    && if reply {
                        frame.sent > result_at
                    } else {
                        frame.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, frame) = if reply {
                candidates.min_by_key(|(_, frame)| frame.sent)
            } else {
                candidates.max_by_key(|(_, frame)| frame.received)
            }
            .ok_or_else(|| format!("missing {key} for {agent} {tool}"))?;
            if frame.value()? != value {
                return Err(format!("altered {key} for {agent} {tool}").into());
            }
            frame.received()?;
            if !reply {
                before = before.min(frame.sent);
            }
            used_frames.insert(*id);
        }
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            io,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
    }
    if !scored.is_empty() {
        return Err("unmatched scored development effects".into());
    }
    let effect_labels = [
        "ImplementA",
        "ImplementB",
        "Implement",
        "ImplementOnRepo",
        "RepoResult",
        "TestIntentA",
        "TestIntentB",
        "TestRequestA",
        "TestRequestB",
        "TestResultA",
        "TestResultB",
        "Review",
        "ReviewResultA",
        "ReviewResultB",
        "MergeAdmissionA",
        "MergeAdmissionB",
        "Merge",
        "MergeOnRepo",
        "CiIntentA",
        "CiIntentB",
        "RunCiOnFeatureA",
        "RunCiOnFeatureB",
        "RunCiA",
        "RunCiB",
        "CiResult",
    ]
    .into_iter()
    .map(|key| name("labels", key))
    .collect::<Result<BTreeSet<_>>>()?;
    if frames.iter().any(|(id, frame)| {
        !pairs
            .iter()
            .any(|(vm, io)| frame.direction(*vm, *io) || frame.direction(*io, *vm))
            && effect_labels.contains(&u64::from(frame.logical_label))
            && !used_frames.contains(id)
    }) {
        return Err("unused or extra effect admission, request or result".into());
    }
    Ok(
        json!({"status":"passed","task":"6M","domain_calls":calls.len(),"simulator_complete":true,"violations":0,
        "consumed_effect_frames":used_frames,"effect_scope":"one-use WASI arguments and real feature/resource requests; actual candidate, approval, publication and CI data; EPF owns dynamic order"}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}

/// Saved-record mutations, not runtime attacks or changes to the source protocol.
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let baseline = check(&rows, &artifact)?;
    fs::create_dir(output)?;
    let ids = baseline["consumed_effect_frames"]
        .as_array()
        .ok_or("effect frame inventory")?
        .iter()
        .map(|id| id.as_u64().ok_or_else(|| "effect frame ID".into()))
        .collect::<Result<Vec<_>>>()?;
    let verdicts =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    let result = json!({"baseline_passed":true,"mutants_rejected":verdicts.len(),"verdicts":verdicts,
        "scope":"effect-frame records; dynamic protocol is separately checked on the baseline"});
    save(&output.join("mutation-results.json"), &result)?;
    Ok(result)
}
