//! Actual file requests, material receipts, effects and replies; EPF checks execution.
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

fn named(compiled: &Value, group: &str, name: &str) -> Result<u64> {
    compiled[group][name]
        .as_u64()
        .ok_or_else(|| format!("compiled {group} {name}").into())
}
fn target(call: &Call) -> Result<&str> {
    let key = match call.tool()? {
        "implement_feature" => "feature_name",
        "merge_code" => "branch",
        "code_review" => "pr_id",
        "write_tests" => "target",
        "run_tests" => "suite",
        _ => return Err("unknown development effect".into()),
    };
    call.request["arguments"][key]
        .as_str()
        .ok_or_else(|| "effect target".into())
}
fn feature(call: &Call) -> Result<usize> {
    match target(call)? {
        "user_authentication" | "full_test_suite_after_auth" => Ok(0),
        "payment_processing" | "full_test_suite_after_payment" => Ok(1),
        _ => Err("unassigned development target".into()),
    }
}
fn receipt(value: &Value) -> Result<u64> {
    value["data"]["artifact_id"]
        .as_str()
        .and_then(|s| s.strip_prefix("development:"))
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "actual effect receipt".into())
}
fn material<'a>(calls: &'a BTreeMap<u64, Call>, tool: &str, id: u64) -> Result<&'a Call> {
    one(calls.values().filter(|call| {
        call.tool().is_ok_and(|t| t == tool)
            && call
                .result()
                .is_ok_and(|(_, result)| receipt(result).ok() == Some(id))
    }))
}
fn consume<'a>(
    (frames, used): (&'a BTreeMap<u64, Frame>, &mut BTreeSet<usize>),
    (from, to, label): (u64, u64, u64),
    (value, after, before): (&Value, usize, usize),
) -> Result<&'a Frame> {
    let frame = one(frames.values().filter(|f| {
        f.direction(from, to)
            && u64::from(f.logical_label) == label
            && after < f.sent
            && f.sent < before
            && !used.contains(&f.sent)
            && f.value().is_ok_and(|v| v == *value)
    }))?;
    frame.received()?;
    used.insert(frame.sent);
    Ok(frame)
}
const BUSINESS_ROLES: [&str; 18] = [
    "MERGE_REQUESTS",
    "REPO_OWNER",
    "REVIEW_INITIAL",
    "REVIEW_REPAIR_A",
    "REVIEW_REPAIR_B",
    "REPO",
    "TEST_ENV",
    "BUILD_SERVER",
    "REVIEW_A",
    "REVIEW_B",
    "A_FILE",
    "B_FILE",
    "A_MERGE_FILE",
    "B_MERGE_FILE",
    "A_REVIEW_FILE",
    "B_REVIEW_FILE",
    "TEST_FILE",
    "CI_FILE",
];

fn business(frame: &Frame, roles: &BTreeSet<u64>) -> bool {
    frame.identity["source"]
        .as_u64()
        .is_some_and(|r| roles.contains(&r))
        && frame.identity["target"]
            .as_u64()
            .is_some_and(|r| roles.contains(&r))
        && frame.value().is_ok_and(|v| v.is_object())
}

fn check(rows: &[Value], projection: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &projection["compiled"];
    if compiled["task"] != "6M" {
        return Err("feature-delivery compiled task".into());
    }
    let role = |name: &str| named(compiled, "roles", name);
    let label = |name: &str| named(compiled, "labels", name);
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if fixture["type"] != "fixtures" || fixture["task"] != "6M" {
        return Err("6M fixture".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?;
    let expected = BTreeSet::from([
        "DEVELOPER_A",
        "DEVELOPER_B",
        "REVIEWER",
        "TEST_WRITER",
        "CI_RUNNER",
    ]);
    if actors.len() != expected.len()
        || actors
            .iter()
            .filter_map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Err("6M participants".into());
    }
    let headers = crate::projection::frames(projection, rows)?;
    let mut journal = crate::journal::read(rows, &headers, actors)?;
    journal.bind_receives(dynamic)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    if score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("development completion, session or actual violation".into());
    }
    let pairs = [
        ("A_VM", "A_IO"),
        ("B_VM", "B_IO"),
        ("REVIEW_VM", "REVIEW_IO"),
        ("TEST_VM", "TEST_IO"),
        ("CI_VM", "CI_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    crate::journal::check_close_pairs(&frames, &pairs)?;
    let roles = BUSINESS_ROLES
        .into_iter()
        .map(role)
        .collect::<Result<BTreeSet<_>>>()?;
    let mut scored = score["events"]
        .as_array()
        .ok_or("scored actual effects")?
        .clone();
    if scored.len() != calls.len() {
        return Err("score and effect count differ".into());
    }
    let mut used = BTreeSet::new();
    let mut publication_inputs = BTreeSet::new();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    for call in &ordered {
        let tool = call.tool()?;
        let f = feature(call)?;
        let suffix = if f == 0 { "A" } else { "B" };
        let name = if f == 0 {
            "user_authentication"
        } else {
            "payment_processing"
        };
        let merge_file = role(if f == 0 {
            "A_MERGE_FILE"
        } else {
            "B_MERGE_FILE"
        })?;
        let review_file = role(if f == 0 {
            "A_REVIEW_FILE"
        } else {
            "B_REVIEW_FILE"
        })?;
        let reviewer = role(if f == 0 { "REVIEW_A" } else { "REVIEW_B" })?;
        let repo = role("REPO")?;
        let environment = role("TEST_ENV")?;
        let build = role("BUILD_SERVER")?;
        let arguments = &call.request["arguments"];
        let agent = call.request["agent"].as_str().ok_or("actual actor")?;
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        if !actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .any(|a| {
                a["tool"] == tool
                    && a["arguments"].as_object().is_some_and(|expected| {
                        arguments.as_object().is_some_and(|actual| {
                            expected
                                .iter()
                                .all(|(key, value)| actual.get(key) == Some(value))
                                && actual.iter().all(|(key, value)| {
                                    expected.contains_key(key)
                                        || (tool == "implement_feature"
                                            && key == "source"
                                            && value.as_str().is_some_and(|s| !s.trim().is_empty()))
                                })
                        })
                    })
            })
        {
            return Err("effect differs from the assigned actual arguments".into());
        }
        let (at, result) = call.result()?;
        if result["tool_name"] != tool
            || result["data"]["agent"] != agent
            || !result["success"].is_boolean()
        {
            return Err("actual result identity".into());
        }
        let score_args = match tool {
            "implement_feature" => {
                if arguments
                    .get("source")
                    .is_some_and(|source| *source != result["data"]["source"])
                {
                    return Err("caller source changed".into());
                }
                json!({"feature":name,"source":result["data"]["source"]})
            }
            "merge_code" => json!({"branch":name}),
            "code_review" => json!({"target":name}),
            "write_tests" => json!({"component":name}),
            "run_tests" => json!({"suite":target(call)?}),
            _ => unreachable!(),
        };
        let scored_at = scored
            .iter()
            .position(|e| {
                e["agent"] == agent
                    && e["tool"] == tool
                    && e["args"] == score_args
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("common score differs from actual effect")?;
        scored.remove(scored_at);
        let next = ordered
            .iter()
            .filter(|other| {
                other.emitted > call.emitted
                    && other.tool().ok() == Some(tool)
                    && feature(other).ok() == Some(f)
            })
            .map(|other| other.emitted)
            .min()
            .unwrap_or(rows.len());
        let (from, to, request_name, reply_name) = match tool {
            "implement_feature" => (
                merge_file,
                repo,
                format!("Implement{suffix}"),
                format!("Implementation{suffix}"),
            ),
            "merge_code" => (
                merge_file,
                repo,
                format!("Merge{suffix}"),
                format!("Merged{suffix}"),
            ),
            "code_review" => (
                review_file,
                reviewer,
                format!("ReviewRequest{suffix}"),
                format!("FileReviewResult{suffix}"),
            ),
            "write_tests" => (
                role("TEST_FILE")?,
                environment,
                format!("TestRequest{suffix}"),
                format!("FileTested{suffix}"),
            ),
            "run_tests" => (
                role("CI_FILE")?,
                build,
                format!("CiRequest{suffix}"),
                format!("FileCiResult{suffix}"),
            ),
            _ => unreachable!(),
        };
        let merge = tool == "merge_code";
        let request = consume(
            (&frames, &mut used),
            (
                from,
                if merge { role("MERGE_REQUESTS")? } else { to },
                label(&request_name)?,
            ),
            (arguments, 0, call.emitted),
        )?;
        if merge {
            let authorized = consume(
                (&frames, &mut used),
                (
                    role("REPO_OWNER")?,
                    repo,
                    label(&format!("InitialMerge{suffix}"))?,
                ),
                (arguments, request.received()?, call.emitted),
            )?;
            if authorized.received()? >= call.emitted {
                return Err("initial merge predates receipt by the repository".into());
            }
        }
        if request.received()? >= call.emitted {
            return Err("effect before its checked request receive".into());
        }
        consume(
            (&frames, &mut used),
            (to, from, label(&reply_name)?),
            (result, at, next),
        )?;
        match tool {
            "implement_feature" => {
                if result["success"] != true || result["data"]["feature"] != name {
                    return Err("implementation result".into());
                }
                for (peer, message) in [
                    (reviewer, format!("ReadyReview{suffix}")),
                    (review_file, format!("FileReviewCandidate{suffix}")),
                    (environment, format!("ReadyTest{suffix}")),
                    (role("TEST_FILE")?, format!("FileTestCandidate{suffix}")),
                    (
                        role(if f == 0 { "B_FILE" } else { "A_FILE" })?,
                        format!("Interface{suffix}"),
                    ),
                ] {
                    consume(
                        (&frames, &mut used),
                        (repo, peer, label(&message)?),
                        (result, at, next),
                    )?;
                }
            }
            "code_review" | "write_tests" => {
                let source = result["data"]["source_receipt"]
                    .as_u64()
                    .ok_or("candidate receipt")?;
                let implementation = material(&calls, "implement_feature", source)?;
                let implemented = implementation.result()?.1;
                if feature(implementation)? != f
                    || implementation.ended()?.0 >= call.started()?.0
                    || result["data"]["candidate_sha256"] != implemented["data"]["candidate_sha256"]
                {
                    return Err("validation differs from the actually completed candidate".into());
                }
                if tool == "write_tests" {
                    if result["success"] != true || result["data"]["component"] != name {
                        return Err("test definition result".into());
                    }
                    consume(
                        (&frames, &mut used),
                        (environment, repo, label(&format!("Tested{suffix}"))?),
                        (result, at, next),
                    )?;
                } else {
                    if result["data"]["target"] != name {
                        return Err("review target".into());
                    }
                    let outcome = if result["success"] == true {
                        "Approved"
                    } else {
                        "Rejected"
                    };
                    let outcomes = [
                        label(&format!("{outcome}{suffix}"))?,
                        label(&format!("Repair{outcome}{suffix}"))?,
                    ];
                    let accepted = one(frames.values().filter(|v| {
                        v.direction(reviewer, repo)
                            && outcomes.contains(&u64::from(v.logical_label))
                            && at < v.sent
                            && v.sent < next
                            && !used.contains(&v.sent)
                            && v.value().is_ok_and(|v| v == *result)
                    }))?;
                    let committed = accepted.received()?;
                    used.insert(accepted.sent);
                    if result["success"] == true && u64::from(accepted.logical_label) == outcomes[0]
                    {
                        consume(
                            (&frames, &mut used),
                            (
                                reviewer,
                                role(if f == 0 {
                                    "REVIEW_REPAIR_A"
                                } else {
                                    "REVIEW_REPAIR_B"
                                })?,
                                outcomes[0],
                            ),
                            (result, at, accepted.sent),
                        )?;
                    }
                    for peer in [
                        merge_file,
                        role(if f == 0 { "B_FILE" } else { "A_FILE" })?,
                        role("TEST_FILE")?,
                        environment,
                    ] {
                        consume(
                            (&frames, &mut used),
                            (repo, peer, label(&format!("ReviewFeedback{suffix}"))?),
                            (result, committed, next),
                        )?;
                    }
                }
            }
            "merge_code" => {
                let implementation = material(
                    &calls,
                    "implement_feature",
                    result["data"]["source_receipt"]
                        .as_u64()
                        .ok_or("merge candidate")?,
                )?;
                let review = material(
                    &calls,
                    "code_review",
                    result["data"]["review_receipt"]
                        .as_u64()
                        .ok_or("merge approval")?,
                )?;
                let implemented = implementation.result()?.1;
                let approved = review.result()?.1;
                if feature(implementation)? != f
                    || feature(review)? != f
                    || result["success"] != true
                    || approved["success"] != true
                    || implementation.ended()?.0 >= review.started()?.0
                    || review.ended()?.0 >= call.started()?.0
                    || approved["data"]["source_receipt"] != json!(receipt(implemented)?)
                    || approved["data"]["candidate_sha256"]
                        != implemented["data"]["candidate_sha256"]
                    || result["data"]["candidate_sha256"] != implemented["data"]["candidate_sha256"]
                    || result["data"]["source"] != implemented["data"]["source"]
                {
                    return Err(
                        "publication differs from its actual candidate or positive approval".into(),
                    );
                }
                let ready = consume(
                    (&frames, &mut used),
                    (reviewer, merge_file, label(&format!("MergeReady{suffix}"))?),
                    (approved, review.result()?.0, call.emitted),
                )?;
                if ready.received()? >= request.received()? {
                    return Err("merge before actual readiness".into());
                }
                let publication_label = label(&format!("Publication{suffix}"))?;
                let publication = one(frames.values().filter(|v| {
                    v.direction(repo, build)
                        && u64::from(v.logical_label) == publication_label
                        && at < v.sent
                        && v.sent < rows.len()
                        && !used.contains(&v.sent)
                        && v.value().is_ok_and(|v| v["publication"] == *result)
                }))?;
                let inputs = publication.value()?;
                if inputs.as_object().is_none_or(|object| {
                    object.len() != 2
                        || !object.contains_key("publication")
                        || !object.contains_key("tests")
                }) {
                    return Err("publication carries unrecorded input fields".into());
                }
                let tests = inputs["tests"]
                    .as_array()
                    .filter(|t| t.len() == 2)
                    .ok_or("publication tests")?;
                let published = result["data"]["published_candidates"]
                    .as_object()
                    .ok_or("published features")?;
                for (index, (name, tested)) in [
                    ("user_authentication", label("TestedA")?),
                    ("payment_processing", label("TestedB")?),
                ]
                .into_iter()
                .enumerate()
                {
                    let received = frames
                        .values()
                        .filter(|frame| {
                            frame.direction(environment, repo)
                                && u64::from(frame.logical_label) == tested
                                && frame.received.is_some_and(|row| row < call.emitted)
                        })
                        .max_by_key(|frame| frame.received);
                    let test = &tests[index];
                    if let Some(frame) = received {
                        if frame.value()? != *test {
                            return Err("publication changed a received test definition".into());
                        }
                        let actual = material(&calls, "write_tests", receipt(test)?)?;
                        if feature(actual)? != index
                            || actual.result()?.1 != test
                            || actual.ended()?.0 >= call.started()?.0
                        {
                            return Err("publication carries another actual test definition".into());
                        }
                    } else if !test.is_null() || published.contains_key(name) {
                        return Err("publication lacks a received test for its candidate or invents a peer test".into());
                    }
                    if let Some(candidate) = published.get(name)
                        && test["data"]["candidate_sha256"] != *candidate
                    {
                        return Err("publication carries a test for another candidate".into());
                    }
                }
                publication.received()?;
                used.insert(publication.sent);
            }
            "run_tests" => {
                let merge = material(
                    &calls,
                    "merge_code",
                    result["data"]["publication_event"]
                        .as_u64()
                        .ok_or("CI publication")?,
                )?;
                let merged = merge.result()?.1;
                if feature(merge)? != f
                    || merge.ended()?.0 >= call.started()?.0
                    || result["data"]["suite"] != arguments["suite"]
                    || result["data"]["repository_sha256"] != merged["data"]["repository_sha256"]
                    || result["data"]["published_candidates"]
                        != merged["data"]["published_candidates"]
                {
                    return Err("CI differs from its actual publication".into());
                }
                let publication_label = label(&format!("Publication{suffix}"))?;
                let publication = one(frames.values().filter(|v| {
                    v.direction(repo, build)
                        && u64::from(v.logical_label) == publication_label
                        && v.received.is_some_and(|n| n < call.emitted)
                        && v.value().is_ok_and(|v| v["publication"] == *merged)
                }))?;
                if !publication_inputs.insert(publication.sent) {
                    return Err("reused CI publication input".into());
                }
                let inputs = publication.value()?;
                for (feature, hash) in result["data"]["published_candidates"]
                    .as_object()
                    .ok_or("CI candidates")?
                {
                    let index = match feature.as_str() {
                        "user_authentication" => 0,
                        "payment_processing" => 1,
                        _ => return Err("CI feature".into()),
                    };
                    let test = &inputs["tests"][index];
                    if test["data"]["candidate_sha256"] != *hash
                        || result["data"]["test_inputs"][feature] != test["data"]
                        || result["data"]["test_receipts"][feature] != json!(receipt(test)?)
                    {
                        return Err("CI used different actual test inputs".into());
                    }
                }
                let incoming = consume(
                    (&frames, &mut used),
                    (
                        build,
                        role("CI_FILE")?,
                        label(&format!("FilePublication{suffix}"))?,
                    ),
                    (merged, merge.result()?.0, call.emitted),
                )?;
                if incoming.received()? >= call.emitted {
                    return Err("CI request before actual publication delivery".into());
                }
                let publisher = role(if f == 0 {
                    "REVIEW_REPAIR_A"
                } else {
                    "REVIEW_REPAIR_B"
                })?;
                let decision = label(&format!(
                    "Ci{}{suffix}",
                    if result["success"] == true {
                        "Passed"
                    } else {
                        "Failed"
                    }
                ))?;
                let gate = consume(
                    (&frames, &mut used),
                    (build, role("REPO_OWNER")?, decision),
                    (result, at, next),
                )?;
                let to_reviewer = consume(
                    (&frames, &mut used),
                    (build, publisher, label(&format!("CiResult{suffix}"))?),
                    (result, gate.sent, next),
                )?;
                let outcome = consume(
                    (&frames, &mut used),
                    (reviewer, repo, decision),
                    (result, to_reviewer.received()?, next),
                )?;
                let committed = outcome.received()?;
                consume(
                    (&frames, &mut used),
                    (repo, build, decision),
                    (result, committed, next),
                )?;
                for peer in [
                    role("A_FILE")?,
                    role("B_FILE")?,
                    review_file,
                    role("TEST_FILE")?,
                    environment,
                ] {
                    consume(
                        (&frames, &mut used),
                        (publisher, peer, label("CiUpdate")?),
                        (result, outcome.sent, next),
                    )?;
                }
            }
            _ => unreachable!(),
        }
    }
    let actual = frames
        .values()
        .filter(|f| business(f, &roles))
        .map(|f| f.sent)
        .collect::<BTreeSet<_>>();
    if used != actual || !scored.is_empty() {
        return Err("missing, extra, duplicated or unmatched actual business data".into());
    }
    let mut overlaps = BTreeMap::<String, usize>::new();
    for (i, a) in ordered.iter().enumerate() {
        for b in ordered.iter().skip(i + 1) {
            if a.started()?.1.max(b.started()?.1) < a.ended()?.1.min(b.ended()?.1) {
                let mut tools = [a.tool()?, b.tool()?];
                tools.sort();
                *overlaps.entry(tools.join("+")).or_default() += 1;
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"6M","kind":"completed_actual_wasi_feature_delivery_trace",
        "depth":fixture["depth"],"domain_calls":calls.len(),"carrier_frames":frames.len(),"guest_exits":actors.len(),
        "direct_file_requests_and_replies":true,"effect_receives_bound_to_checked_occurrences":true,
        "actual_candidate_review_test_and_ci_receipts":true,"simulator_complete":true,"violations":0,
        "overlapping_effect_pairs":overlaps,"dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let projection = crate::projection::load(path)?;
    let rows = records(path)?;
    let dynamic = crate::projection::check_dynamic(&projection, &rows)?;
    let mut result = check(&rows, &projection, &dynamic)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let audited = audit(path)?;
    let dynamic = &audited["dynamic_protocol"];
    let headers = crate::projection::frames(&projection, &rows)?;
    let actors = rows.first().ok_or("fixture")?["message"]["fixtures"]
        .as_array()
        .ok_or("actors")?;
    let journal = crate::journal::read(&rows, &headers, actors)?;
    let roles = BUSINESS_ROLES
        .into_iter()
        .map(|name| named(&projection["compiled"], "roles", name))
        .collect::<Result<BTreeSet<_>>>()?;
    let selected = journal
        .frames
        .iter()
        .filter(|(_, frame)| business(frame, &roles))
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let mut verdicts = crate::journal::json_frame_mutants(&rows, &selected, |changed| {
        check(changed, &projection, dynamic)
    })?;
    let id = *selected.first().ok_or("actual business frame")?;
    for (field, value) in [("frame_label", 65535), ("lane", 255)] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id {
                row["message"][field] = json!(value);
            }
        }
        let error = check(&changed, &projection, dynamic)
            .err()
            .ok_or("coherent invalid header accepted")?;
        verdicts.push(json!({"frame_id":id,"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","task":"6M","source_events_sha256":digest(path)?,"audit":audited,"count":verdicts.len(),"mutants":verdicts,
        "scope":"Recorded actual requests, material receipts, effects and replies with EPF-checked receive occurrences; compiled wire membership. Not runtime attacks.","model_calls":0});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
