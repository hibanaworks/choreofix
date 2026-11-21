//! Independent actual-input/effect correspondence. EPF checks source execution.
use crate::{
    evidence::{Result, digest, save},
    journal::{Call, one, take_json_frame},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const FEATURES: [&str; 3] = [
    "user_authentication",
    "payment_processing",
    "notification_system",
];

fn target<'a>(call: &Call<'a>) -> Result<&'a str> {
    let key = match call.tool {
        "implement_feature" => "feature_name",
        "write_tests" => "target",
        "code_review" => "pr_id",
        "merge_code" => "branch",
        "run_tests" => "suite",
        "deploy_service" | "validate_staging" | "rollback_service" => "service",
        _ => return Err("unassigned staged-development effect".into()),
    };
    call.request["arguments"][key]
        .as_str()
        .ok_or_else(|| "actual assigned target".into())
}
fn feature(call: &Call) -> Result<usize> {
    match target(call)? {
        "user_authentication" | "full_suite_after_auth" => Ok(0),
        "payment_processing" | "full_suite_after_payment" => Ok(1),
        "notification_system" | "full_suite_after_notification" => Ok(2),
        _ => Err("unassigned staged-development target".into()),
    }
}
fn receipt(value: &Value) -> Result<u64> {
    value["data"]["artifact_id"]
        .as_str()
        .and_then(|id| id.strip_prefix("development:"))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| "actual effect receipt".into())
}
fn material<'a>(calls: &'a BTreeMap<u64, Call<'a>>, tool: &str, id: u64) -> Result<&'a Call<'a>> {
    one(calls
        .values()
        .filter(|call| call.tool == tool && receipt(call.result.1).ok() == Some(id)))
}
fn input<'a>(
    calls: &'a BTreeMap<u64, Call<'a>>,
    call: &Call,
    tool: &str,
    field: &str,
) -> Result<&'a Call<'a>> {
    let previous = material(
        calls,
        tool,
        call.result.1["data"][field]
            .as_u64()
            .ok_or("input receipt")?,
    )?;
    if previous.ended.0 >= call.started.0 {
        return Err("effect used an input before its actual completion".into());
    }
    Ok(previous)
}

pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    journal.check_session(&journal.projection["compiled"]["session"])?;
    let dynamic = journal.execution;
    let rows = journal.records();
    let compiled = &journal.projection["compiled"];
    let fixture = journal.fixture();
    let role = |name: &str| crate::projection::named_id(&compiled["roles"], name).map(u64::from);
    let label = |name: &str| crate::projection::named_id(&compiled["labels"], name).map(u64::from);
    let actors = journal.participants;

    let crate::journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if !simulator_complete {
        return Err("incomplete run, foreign session or actual simulator violation".into());
    }
    let pairs = ["A", "B", "C", "REVIEW", "TEST", "CI", "RELEASE"]
        .into_iter()
        .map(|name| Ok((role(&format!("{name}_VM"))?, role(&format!("{name}_IO"))?)))
        .collect::<Result<Vec<_>>>()?;
    crate::journal::check_close_pairs(&frames, &pairs)?;
    let business_roles = [
        "REPO",
        "REPO_OWNER",
        "MERGE_REQUESTS",
        "STAGE_OWNER",
        "STAGE_OWNER_B",
        "STAGE_OWNER_C",
        "TEST_ENV",
        "BUILD_SERVER",
        "CI_OWNER",
        "STAGING_ENV",
        "REVIEW_A",
        "REVIEW_B",
        "REVIEW_C",
        "A_FILE",
        "B_FILE",
        "C_FILE",
        "A_REVIEW_FILE",
        "B_REVIEW_FILE",
        "C_REVIEW_FILE",
        "TEST_FILE",
        "CI_FILE",
        "RELEASE_FILE",
    ]
    .into_iter()
    .map(role)
    .collect::<Result<BTreeSet<_>>>()?;
    let mut scored = scored_events.iter().collect::<Vec<_>>();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut used = BTreeSet::new();
    for call in &ordered {
        let tool = call.tool;
        let f = feature(call)?;
        let suffix = ["A", "B", "C"][f];
        let name = FEATURES[f];
        let developer = role(&format!("{suffix}_FILE"))?;
        let review_file = role(&format!("{suffix}_REVIEW_FILE"))?;
        let reviewer = role(&format!("REVIEW_{suffix}"))?;
        let repo = role("REPO")?;
        let owner = role("REPO_OWNER")?;
        let requests = role("MERGE_REQUESTS")?;
        let environment = role("TEST_ENV")?;
        let build = role("BUILD_SERVER")?;
        let ci_owner = role("CI_OWNER")?;
        let stage = role("STAGING_ENV")?;
        let arguments = &call.request["arguments"];
        let agent = call.request["agent"].as_str().ok_or("effect actor")?;
        let actor = one(actors.iter().filter(|actor| actor["agent"] == agent))?;
        if !actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .any(|assigned| {
                assigned["tool"] == tool
                    && assigned["arguments"].as_object().is_some_and(|expected| {
                        arguments.as_object().is_some_and(|actual| {
                            expected
                                .iter()
                                .all(|(key, value)| actual.get(key) == Some(value))
                                && actual.iter().all(|(key, value)| {
                                    expected.contains_key(key)
                                        || (tool == "implement_feature"
                                            && key == "source"
                                            && value.as_str().is_some_and(|s| !s.trim().is_empty()))
                                        || (tool == "deploy_service"
                                            && key == "environment"
                                            && value == "staging")
                                })
                        })
                    })
            })
        {
            return Err("effect differs from actual assigned operation".into());
        }
        let (at, result) = call.result;
        let data = &result["data"];
        if result["tool_name"] != tool || data["agent"] != agent || !result["success"].is_boolean()
        {
            return Err("actual result identity".into());
        }
        let score_args = match tool {
            "implement_feature" => {
                if arguments
                    .get("source")
                    .is_some_and(|value| *value != data["source"])
                {
                    return Err("caller source substitution".into());
                }
                json!({"feature":name,"source":data["source"]})
            }
            "write_tests" => json!({"component":name}),
            "code_review" => json!({"target":name}),
            "merge_code" => json!({"branch":name}),
            "run_tests" => json!({"suite":target(call)?}),
            _ => json!({"service":name}),
        };
        call.take_score(&mut scored, &score_args)?;
        let next = ordered
            .iter()
            .filter(|next| {
                next.emitted > call.emitted && next.tool == tool && feature(next).ok() == Some(f)
            })
            .map(|call| call.emitted)
            .min()
            .unwrap_or(rows.len());
        let (from, to, request, reply) = match tool {
            "implement_feature" => (
                developer,
                repo,
                format!("Implement{suffix}"),
                format!("Implementation{suffix}"),
            ),
            "write_tests" => (
                role("TEST_FILE")?,
                environment,
                format!("TestRequest{suffix}"),
                format!("FileTested{suffix}"),
            ),
            "code_review" => (
                review_file,
                reviewer,
                format!("ReviewRequest{suffix}"),
                format!("FileReviewResult{suffix}"),
            ),
            "merge_code" => (
                developer,
                requests,
                format!("MergeRequest{suffix}"),
                format!("Merged{suffix}"),
            ),
            "run_tests" => (
                role("CI_FILE")?,
                build,
                format!("CiRequest{suffix}"),
                format!("FileCiResult{suffix}"),
            ),
            "deploy_service" => (
                role("RELEASE_FILE")?,
                stage,
                format!("DeployRequest{suffix}"),
                format!("Deployed{suffix}"),
            ),
            "validate_staging" => (
                role("RELEASE_FILE")?,
                stage,
                format!("ValidateRequest{suffix}"),
                format!("FileStageResult{suffix}"),
            ),
            "rollback_service" => (
                role("RELEASE_FILE")?,
                stage,
                format!("RollbackRequest{suffix}"),
                format!("RolledBack{suffix}"),
            ),
            _ => unreachable!(),
        };
        let accepted = take_json_frame(
            (&frames, &mut used),
            (from, to, &[label(&request)?]),
            (arguments, 0, call.emitted),
        )?;
        if accepted.received >= call.emitted {
            return Err("effect before EPF-checked request receive".into());
        }
        take_json_frame(
            (&frames, &mut used),
            (
                if tool == "merge_code" { repo } else { to },
                from,
                &[label(&reply)?],
            ),
            (result, at, next),
        )?;
        match tool {
            "implement_feature" => {
                if result["success"] != true
                    || data["feature"] != name
                    || data["source"].as_str().is_none_or(str::is_empty)
                {
                    return Err("actual source result".into());
                }
                for (peer, message) in [
                    (environment, format!("TestCandidate{suffix}")),
                    (role("TEST_FILE")?, format!("FileTestCandidate{suffix}")),
                ] {
                    take_json_frame(
                        (&frames, &mut used),
                        (repo, peer, &[label(&message)?]),
                        (result, at, next),
                    )?;
                }
                for other in ["A", "B", "C"].into_iter().filter(|other| *other != suffix) {
                    take_json_frame(
                        (&frames, &mut used),
                        (
                            repo,
                            role(&format!("{other}_FILE"))?,
                            &[label(&format!("Interface{suffix}To{other}"))?],
                        ),
                        (result, at, next),
                    )?;
                }
            }
            "write_tests" | "code_review" => {
                let source = input(&calls, call, "implement_feature", "source_receipt")?;
                let candidate = source.result.1;
                if feature(source)? != f
                    || data["candidate_sha256"] != candidate["data"]["candidate_sha256"]
                {
                    return Err("test or review used different candidate bytes".into());
                }
                if tool == "write_tests" {
                    if result["success"] != true || data["component"] != name {
                        return Err("actual test result".into());
                    }
                    take_json_frame(
                        (&frames, &mut used),
                        (environment, repo, &[label(&format!("Tested{suffix}"))?]),
                        (result, at, next),
                    )?;
                } else {
                    let test = one(calls.values().filter(|test| {
                        test.tool == "write_tests"
                            && test.result.1["data"]["source_receipt"] == data["source_receipt"]
                    }))?;
                    if test.ended.0 >= call.started.0 || data["target"] != name {
                        return Err("review lacks its completed actual test input".into());
                    }
                    let context = json!({"candidate":candidate,"tests":test.result.1});
                    for (peer, message) in [
                        (reviewer, format!("ReviewCandidate{suffix}")),
                        (review_file, format!("FileReviewCandidate{suffix}")),
                    ] {
                        let ready = take_json_frame(
                            (&frames, &mut used),
                            (repo, peer, &[label(&message)?]),
                            (&context, test.result.0, call.emitted),
                        )?;
                        if ready.received >= call.emitted {
                            return Err("review before its actual candidate delivery".into());
                        }
                    }
                    let outcome = if result["success"] == true {
                        "Approved"
                    } else {
                        "Rejected"
                    };
                    let labels = ["", "Repair", "StageRepair", "StageCi"]
                        .map(|prefix| label(&format!("{prefix}{outcome}{suffix}")))
                        .into_iter()
                        .collect::<Result<Vec<_>>>()?;
                    let committed = take_json_frame(
                        (&frames, &mut used),
                        (reviewer, repo, &labels),
                        (result, at, next),
                    )?
                    .received;
                    for peer in ["A_FILE", "B_FILE", "C_FILE", "TEST_ENV", "TEST_FILE"]
                        .into_iter()
                        .map(role)
                    {
                        take_json_frame(
                            (&frames, &mut used),
                            (repo, peer?, &[label(&format!("ReviewFeedback{suffix}"))?]),
                            (result, committed, next),
                        )?;
                    }
                }
            }
            "merge_code" => {
                let admitted = take_json_frame(
                    (&frames, &mut used),
                    (owner, repo, &[label(&format!("Admit{suffix}"))?]),
                    (arguments, accepted.received, call.emitted),
                )?;
                if admitted.received >= call.emitted {
                    return Err(
                        "publication began before repository received its admitted request".into(),
                    );
                }
                let source = input(&calls, call, "implement_feature", "source_receipt")?;
                let review = input(&calls, call, "code_review", "review_receipt")?;
                let checked = review.result.1;
                if feature(source)? != f
                    || feature(review)? != f
                    || checked["success"] != true
                    || data["source"] != source.result.1["data"]["source"]
                    || data["candidate_sha256"] != checked["data"]["candidate_sha256"]
                    || data["source_receipt"] != checked["data"]["source_receipt"]
                    || data["publication_event"].as_u64() != Some(receipt(result)?)
                {
                    return Err("publication differs from completed actual approved source".into());
                }
                let base = if !data["base_test_receipt"].is_null() {
                    Some(("base_test_receipt", true))
                } else if !data["base_failure_receipt"].is_null() {
                    Some(("base_failure_receipt", false))
                } else {
                    None
                };
                if let Some((field, success)) = base {
                    let ci = input(&calls, call, "run_tests", field)?;
                    let prior = ci.result.1;
                    if prior["success"] != success
                        || data["base_repository_sha256"] != prior["data"]["repository_sha256"]
                        || data["base_publication_events"] != prior["data"]["publication_events"]
                    {
                        return Err("publication base differs from completed actual CI".into());
                    }
                    if !success
                        && (data["repaired_publication_event"]
                            != prior["data"]["publication_event"]
                            || source.started.0
                                <= input(&calls, ci, "merge_code", "publication_event")?
                                    .ended
                                    .0)
                    {
                        return Err("repair reused source preceding the failed publication".into());
                    }
                } else if !data["base_repository_sha256"].is_null()
                    || data["base_publication_events"]
                        .as_object()
                        .is_none_or(|map| !map.is_empty())
                {
                    return Err("publication has no matching base CI".into());
                }
            }
            "run_tests" => {
                let merge = input(&calls, call, "merge_code", "publication_event")?;
                let published = merge.result.1;
                if feature(merge)? != f
                    || data["repository_sha256"] != published["data"]["repository_sha256"]
                    || data["published_candidates"] != published["data"]["published_candidates"]
                {
                    return Err("CI differs from the actual publication".into());
                }
                let previous_stage = calls
                    .values()
                    .filter(|entry| {
                        entry.tool == "validate_staging"
                            && feature(entry).ok() == Some(f)
                            && entry.ended.0 < merge.emitted
                    })
                    .max_by_key(|entry| entry.emitted);
                let publication_label =
                    if previous_stage.is_some_and(|entry| entry.result.1["success"] == false) {
                        label(&format!("RepairPublication{suffix}"))?
                    } else {
                        label(&format!("Publication{suffix}"))?
                    };
                let context = one(frames.values().filter(|frame| {
                    frame.direction(repo, ci_owner)
                        && u64::from(frame.logical_label) == publication_label
                        && frame.sent < call.emitted
                        && !used.contains(&frame.sent)
                        && frame
                            .value()
                            .is_ok_and(|value| value["publication"] == *published)
                }))?;
                if context.received >= call.emitted {
                    return Err("CI before its actual inputs".into());
                }
                used.insert(context.sent);
                let inputs = context.value()?;
                let mut received_tests = Vec::new();
                for suffix in ["A", "B", "C"] {
                    let tested = label(&format!("Tested{suffix}"))?;
                    let latest = frames
                        .values()
                        .filter(|frame| {
                            frame.direction(environment, repo)
                                && u64::from(frame.logical_label) == tested
                                && frame.received < context.sent
                        })
                        .max_by_key(|frame| frame.received);
                    received_tests.push(match latest {
                        Some(frame) => frame.value()?,
                        None => Value::Null,
                    });
                }
                if inputs != json!({"publication":published,"tests":received_tests}) {
                    return Err(
                        "CI context differs from the actual received material snapshot".into(),
                    );
                }
                let mut publications = published["data"]["base_publication_events"]
                    .as_object()
                    .ok_or("publication base receipts")?
                    .clone();
                publications.insert(name.into(), json!(receipt(published)?));
                if data["publication_events"] != json!(publications) {
                    return Err("CI substituted a publication occurrence".into());
                }
                let candidates = data["published_candidates"]
                    .as_object()
                    .ok_or("actual published candidates")?;
                for field in ["publication_events", "test_receipts", "test_inputs"] {
                    if data[field].as_object().is_none_or(|map| {
                        map.len() != candidates.len()
                            || candidates.keys().any(|key| !map.contains_key(key))
                    }) {
                        return Err("CI input map omits or adds material".into());
                    }
                }
                for (feature, hash) in candidates {
                    let index = FEATURES
                        .iter()
                        .position(|name| *name == feature)
                        .ok_or("CI feature")?;
                    let test = &inputs["tests"][index];
                    let actual = material(&calls, "write_tests", receipt(test)?)?;
                    if actual.ended.0 >= call.started.0
                        || *test != *actual.result.1
                        || test["data"]["candidate_sha256"] != *hash
                        || data["test_inputs"][feature] != test["data"]
                        || data["test_receipts"][feature] != json!(receipt(test)?)
                    {
                        return Err("CI differs from the actually completed test inputs".into());
                    }
                }
                take_json_frame(
                    (&frames, &mut used),
                    (
                        build,
                        role("CI_FILE")?,
                        &[label(&format!("FilePublication{suffix}"))?],
                    ),
                    (published, merge.result.0, call.emitted),
                )?;
                let outcome = if result["success"] == true {
                    "Passed"
                } else {
                    "Failed"
                };
                let labels = [
                    format!("Ci{outcome}{suffix}"),
                    format!("StageCi{outcome}{suffix}"),
                ]
                .iter()
                .map(|name| label(name))
                .collect::<Result<Vec<_>>>()?;
                let committed = take_json_frame(
                    (&frames, &mut used),
                    (build, repo, &labels),
                    (result, at, next),
                )?
                .received;
                take_json_frame(
                    (&frames, &mut used),
                    (
                        repo,
                        owner,
                        &[label(&format!("Publication{outcome}{suffix}"))?],
                    ),
                    (result, committed, next),
                )?;
                for peer in [
                    role("A_FILE")?,
                    role("B_FILE")?,
                    role("C_FILE")?,
                    environment,
                    reviewer,
                    review_file,
                    role("TEST_FILE")?,
                ] {
                    take_json_frame(
                        (&frames, &mut used),
                        (repo, peer, &[label(&format!("CiUpdate{suffix}"))?]),
                        (result, committed, next),
                    )?;
                }
            }
            "deploy_service" => {
                let merge = input(&calls, call, "merge_code", "publication_event")?;
                let ci = input(&calls, call, "run_tests", "ci_receipt")?;
                let published = merge.result.1;
                let tested = ci.result.1;
                if result["success"] != true
                    || tested["success"] != true
                    || feature(merge)? != f
                    || data["candidate_sha256"] != published["data"]["candidate_sha256"]
                    || data["repository_sha256"] != tested["data"]["repository_sha256"]
                    || data["deployment_event"].as_u64() != Some(receipt(result)?)
                    || data["environment"] != "staging"
                {
                    return Err("deployment lacks actual matching positive CI".into());
                }
                for (dependency, id) in data["dependency_receipts"]
                    .as_object()
                    .ok_or("actual dependency receipts")?
                {
                    let checked = material(
                        &calls,
                        "validate_staging",
                        id.as_u64().ok_or("dependency receipt")?,
                    )?;
                    if checked.ended.0 >= call.started.0
                        || checked.result.1["success"] != true
                        || checked.result.1["data"]["service"] != *dependency
                        || checked.result.1["data"]["candidate_sha256"]
                            != tested["data"]["published_candidates"][dependency]
                    {
                        return Err(
                            "deployment reused stale or incomplete dependency validation".into(),
                        );
                    }
                }
                let integrated = input(&calls, ci, "merge_code", "publication_event")?
                    .result
                    .1;
                let inputs = json!({"publication":published,"integrated_publication":integrated,"ci":tested});
                let deployment_start = call.started.0;
                let own_ci = one(calls.values().filter(|own| {
                    own.tool == "run_tests"
                        && own.ended.0 < deployment_start
                        && own.result.1["success"] == true
                        && own.result.1["data"]["publication_event"]
                            == published["data"]["publication_event"]
                }))?;
                let readiness = json!({"publication":published,"integrated_publication":published,"ci":own_ci.result.1});
                let stage_owner = role(["STAGE_OWNER", "STAGE_OWNER_B", "STAGE_OWNER_C"][f])?;
                let announced = take_json_frame(
                    (&frames, &mut used),
                    (
                        build,
                        stage_owner,
                        &[label(&format!("StagingReady{suffix}"))?],
                    ),
                    (&readiness, own_ci.result.0, call.emitted),
                )?
                .received;
                take_json_frame(
                    (&frames, &mut used),
                    (
                        stage,
                        role("RELEASE_FILE")?,
                        &[label(&format!("FileStagingReady{suffix}"))?],
                    ),
                    (&readiness, announced, accepted.sent),
                )?;
                let checked = take_json_frame(
                    (&frames, &mut used),
                    (stage, requests, &[label(&format!("CheckStage{suffix}"))?]),
                    (arguments, accepted.received, call.emitted),
                )?
                .received;
                let latest = take_json_frame(
                    (&frames, &mut used),
                    (owner, repo, &[label(&format!("LatestChecked{suffix}"))?]),
                    (arguments, checked, call.emitted),
                )?
                .received;
                let before_effect = take_json_frame(
                    (&frames, &mut used),
                    (repo, stage, &[label(&format!("StageInputs{suffix}"))?]),
                    (&inputs, latest.max(ci.result.0), call.emitted),
                )?
                .received;
                if before_effect >= call.emitted {
                    return Err("deploy before receiving the checked repository head".into());
                }
                let installed = label(&format!("Installed{suffix}"))?;
                let installation = take_json_frame(
                    (&frames, &mut used),
                    (stage, repo, &[installed]),
                    (result, at, next),
                )?
                .received;
                take_json_frame(
                    (&frames, &mut used),
                    (repo, owner, &[installed]),
                    (result, installation, next),
                )?;
            }
            "validate_staging" => {
                let deployed = input(&calls, call, "deploy_service", "deployment_event")?;
                if feature(deployed)? != f
                    || data["service"] != name
                    || data["candidate_sha256"] != deployed.result.1["data"]["candidate_sha256"]
                {
                    return Err("validation used another actual deployment".into());
                }
                let outcome = if result["success"] == true {
                    "Passed"
                } else {
                    "Failed"
                };
                let committed = take_json_frame(
                    (&frames, &mut used),
                    (stage, repo, &[label(&format!("Stage{outcome}{suffix}"))?]),
                    (result, at, next),
                )?
                .received;
                let after = if result["success"] == true {
                    committed
                } else {
                    let rollback = one(calls.values().filter(|rollback| {
                        rollback.tool == "rollback_service"
                            && rollback.result.1["data"]["removed_deployment"]["deployment_event"]
                                == data["deployment_event"]
                    }))?;
                    let returned = one(frames.values().filter(|frame| {
                        frame.direction(stage, repo)
                            && u64::from(frame.logical_label)
                                == label(&format!("RollbackReceipt{suffix}"))
                                    .expect("known rollback label")
                            && frame.value().is_ok_and(|value| value == *rollback.result.1)
                    }))?;
                    returned.received
                };
                for peer in [
                    role("A_FILE")?,
                    role("B_FILE")?,
                    role("C_FILE")?,
                    environment,
                    reviewer,
                    review_file,
                    role("TEST_FILE")?,
                    role("CI_FILE")?,
                    ci_owner,
                ] {
                    take_json_frame(
                        (&frames, &mut used),
                        (repo, peer, &[label(&format!("StageUpdate{suffix}"))?]),
                        (result, after, next),
                    )?;
                }
            }
            "rollback_service" => {
                let removed = &data["removed_deployment"];
                let deployed = material(
                    &calls,
                    "deploy_service",
                    removed["deployment_event"]
                        .as_u64()
                        .ok_or("removed deployment receipt")?,
                )?;
                let actual = deployed.result.1;
                let failed = one(calls.values().filter(|validation| {
                    validation.tool == "validate_staging"
                        && validation.result.1["data"]["deployment_event"]
                            == removed["deployment_event"]
                }))?;
                if result["success"] != true
                    || failed.ended.0 >= call.started.0
                    || failed.result.1["success"] != false
                    || removed.as_object().is_none_or(|object| {
                        object
                            .iter()
                            .any(|(key, value)| actual["data"][key] != *value)
                    })
                {
                    return Err(
                        "rollback did not remove the actual negatively validated deployment".into(),
                    );
                }
                take_json_frame(
                    (&frames, &mut used),
                    (stage, repo, &[label(&format!("RollbackReceipt{suffix}"))?]),
                    (result, at, next),
                )?;
            }
            _ => unreachable!(),
        }
    }
    let actual = frames
        .values()
        .filter(|frame| {
            business_roles.contains(&frame.source)
                && business_roles.contains(&frame.target)
                && frame.value().is_ok_and(|value| value.is_object())
        })
        .map(|frame| frame.sent)
        .collect::<BTreeSet<_>>();
    if used != actual || !scored.is_empty() {
        return Err("unmatched, reused, missing or extra business effect data".into());
    }
    let mut overlaps = BTreeMap::<String, usize>::new();
    for (index, a) in ordered.iter().enumerate() {
        for b in ordered.iter().skip(index + 1) {
            if a.started.1.max(b.started.1) < a.ended.1.min(b.ended.1) {
                let mut pair = [a.tool, b.tool];
                pair.sort();
                *overlaps.entry(pair.join("+")).or_default() += 1;
            }
        }
    }
    Ok(
        json!({"verifier_binary_sha256":digest(&std::env::current_exe()?)?,"status":"passed","task":"6H","depth":fixture["depth"],"domain_calls":calls.len(),
        "carrier_frames":frames.len(),"guest_exits":actors.len(),"direct_file_requests_and_replies":true,
        "effect_receives_bound_to_checked_occurrences":true,"actual_candidate_review_test_ci_and_staging_receipts":true,
        "simulator_complete":true,"violations":0,"overlapping_effect_pairs":overlaps,
        "dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let evidence = crate::journal::Evidence::open(path)?;
    let audited = evidence.audit("6H")?;
    let (rows, projection) = (evidence.rows, evidence.projection);
    let dynamic = &audited["dynamic_protocol"];
    let journal = crate::journal::read(&rows, &projection, dynamic)?;
    let roles = [
        "REPO",
        "REPO_OWNER",
        "MERGE_REQUESTS",
        "STAGE_OWNER",
        "STAGE_OWNER_B",
        "STAGE_OWNER_C",
        "TEST_ENV",
        "BUILD_SERVER",
        "CI_OWNER",
        "STAGING_ENV",
        "REVIEW_A",
        "REVIEW_B",
        "REVIEW_C",
        "A_FILE",
        "B_FILE",
        "C_FILE",
        "A_REVIEW_FILE",
        "B_REVIEW_FILE",
        "C_REVIEW_FILE",
        "TEST_FILE",
        "CI_FILE",
        "RELEASE_FILE",
    ]
    .into_iter()
    .map(|name| {
        projection["compiled"]["roles"][name]
            .as_u64()
            .ok_or_else(|| "compiled business role".into())
    })
    .collect::<Result<BTreeSet<_>>>()?;
    let selected = journal
        .frames
        .iter()
        .filter(|(_, frame)| {
            roles.contains(&frame.source)
                && roles.contains(&frame.target)
                && frame.value().is_ok_and(|value| value.is_object())
        })
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let mut verdicts = crate::journal::json_frame_mutants(&rows, &selected, |changed| {
        crate::journal::check_effects(changed, &projection, dynamic)
    })?;
    let id = *selected.first().ok_or("actual business frame")?;
    for (field, value) in [("frame_label", 65535), ("lane", 255)] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id {
                row["message"][field] = json!(value);
            }
        }
        let error = crate::journal::check_effects(&changed, &projection, dynamic)
            .err()
            .ok_or("coherent invalid header accepted")?;
        verdicts.push(json!({"frame_id":id,"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","task":"6H","source_events_sha256":evidence.events_hash,"audit":audited,
        "count":verdicts.len(),"mutants":verdicts,"model_calls":0,
        "verifier_binary_sha256":digest(&std::env::current_exe()?)?,
        "scope":"Recorded request/effect/result and compiled wire counterexamples with EPF-checked receive occurrences; not runtime attacks"});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
