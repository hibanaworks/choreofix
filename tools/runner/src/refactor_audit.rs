//! Actual admitted values, resource requests and effects; EPF checks choreography.
use crate::{
    evidence::{Result, digest},
    journal::{Call, check_close_pairs, close_arguments, one, records},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

// Reconstruct data from completed effects, rather than trusting a candidate packet.
fn candidate(calls: &[&Call], before: usize) -> Result<Value> {
    let changes = calls
        .iter()
        .filter(|c| c.request["tool"] == "refactor_code")
        .filter(|c| c.result.as_ref().is_some_and(|(at, _)| *at < before))
        .map(|c| Ok(json!({"arguments":c.request["arguments"],"result":c.result()?.1})))
        .collect::<Result<Vec<_>>>()?;
    if !matches!(changes.len(), 2 | 4) {
        return Err("candidate lacks its completed repository changes".into());
    }
    let mut repository = Value::Null;
    for pair in changes.as_chunks::<2>().0 {
        repository = json!({"previous":repository,"changes":pair});
    }
    if changes.len() == 2 {
        return Ok(repository);
    }
    let designs = calls
        .iter()
        .filter(|c| c.request["tool"] == "design_proposal")
        .filter(|c| c.result.as_ref().is_some_and(|(at, _)| *at < before))
        .map(|c| Ok(c.result()?.1.clone()))
        .collect::<Result<Vec<_>>>()?;
    if designs.len() != 2 {
        return Err("candidate lacks completed design data".into());
    }
    Ok(json!({"designs":designs,"repository":repository}))
}

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty refactor evidence")?["message"];
    if compiled["task"] != "4M" || fixture["task"] != "4M" || fixture["type"] != "fixtures" {
        return Err("shared-refactor task differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("refactor participants")?;
    if actors.len() != 5
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                Some("ARCHITECT"),
                Some("DEVELOPER_A"),
                Some("DEVELOPER_B"),
                Some("REVIEWER"),
                Some("TESTER"),
            ])
    {
        return Err("shared-refactor participants differ".into());
    }
    let names = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let store = names("roles", "STORE")?;
    let repo = names("roles", "REPO")?;
    let environment = names("roles", "TEST_ENV")?;
    let reviewer = names("roles", "REVIEW_SERVICE")?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if score["complete"] != true
        || score["violations"] != json!([])
        || calls.len() != 14
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("shared-refactor completion or session differs".into());
    }
    let pairs = [
        ("ARCHITECT", "ARCHITECT_IO"),
        ("DEVELOPER_A", "A_IO"),
        ("DEVELOPER_B", "B_IO"),
        ("REVIEWER", "REVIEWER_IO"),
        ("TESTER", "TESTER_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((names("roles", vm)?, names("roles", io)?)))
    .collect::<Result<Vec<_>>>()?;
    check_close_pairs(&frames, &pairs)?;
    let mut used_frames = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("simulator score events")?
        .clone();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    let mut tests = Vec::<Value>::new();
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("effect participant")?;
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        let vm = names("roles", agent)?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        one(actor["calls"]
            .as_array()
            .ok_or("participant assignments")?
            .iter()
            .filter(|c| c["tool"] == tool && c["arguments"] == *arguments))?;
        let (result_at, result) = call.result()?;
        if result["success"] != true {
            return Err("refactor effect failed".into());
        }
        let score_args = match tool {
            "design_proposal" => json!({"proposal":arguments["title"]}),
            "code_review" => json!({"target":arguments["pr_id"]}),
            _ => arguments.clone(),
        };
        let index = scored
            .iter()
            .position(|e| {
                e["agent"] == agent
                    && e["tool"] == tool
                    && e["args"] == score_args
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("actual refactor effect differs from simulator score")?;
        scored.remove(index);
        // Each tuple names a real frame, its payload and which side of the effect it occupies.
        let mut expected = Vec::<(u64, u64, &str, Value, bool)>::new();
        let module = if matches!(arguments["target"].as_str(), Some("module_b"))
            || matches!(arguments["title"].as_str(), Some("module_b_refactor"))
            || matches!(arguments["pr_id"].as_str(), Some("module_b"))
            || matches!(arguments["branch"].as_str(), Some("module_b"))
        {
            "DEVELOPER_B"
        } else {
            "DEVELOPER_A"
        };
        let io = match agent {
            "ARCHITECT" => "ARCHITECT_IO",
            "DEVELOPER_A" => "A_IO",
            "DEVELOPER_B" => "B_IO",
            "REVIEWER" => "REVIEWER_IO",
            "TESTER" => "TESTER_IO",
            _ => return Err("refactor participant".into()),
        };
        match tool {
            "design_proposal" => {
                expected.push((
                    vm,
                    store,
                    if module == "DEVELOPER_A" {
                        "DesignA"
                    } else {
                        "DesignB"
                    },
                    arguments.clone(),
                    false,
                ));
                expected.push((store, vm, "DesignResult", result.clone(), true));
                expected.push((
                    store,
                    names("roles", module)?,
                    "DesignResult",
                    result.clone(),
                    true,
                ));
            }
            "refactor_code" => {
                expected.push((
                    vm,
                    store,
                    if arguments["target"] == "SHARED_LIBRARY" {
                        "SharedIntent"
                    } else {
                        "OwnIntent"
                    },
                    arguments.clone(),
                    false,
                ));
                expected.push((repo, store, "Refactor", arguments.clone(), false));
                expected.push((store, repo, "Refactored", result.clone(), true));
            }
            "run_tests" => {
                let c = candidate(&ordered, call.emitted)?;
                let envelope = json!({"candidate":c,"arguments":arguments});
                let tested = json!({"candidate":c,"arguments":arguments,"result":result});
                expected.push((
                    vm,
                    store,
                    if arguments["suite"] == "module_a_tests" {
                        "TestA"
                    } else {
                        "TestB"
                    },
                    arguments.clone(),
                    false,
                ));
                expected.push((repo, store, "ReadSnapshot", envelope.clone(), false));
                expected.push((environment, store, "RunTests", envelope, false));
                expected.push((store, repo, "RepositoryTested", tested.clone(), true));
                expected.push((
                    store,
                    environment,
                    "EnvironmentTested",
                    tested.clone(),
                    true,
                ));
                tests.push(tested);
            }
            "code_review" => {
                let c = candidate(&ordered, call.emitted)?;
                let reply =
                    json!({"success":true,"candidate":c,"arguments":arguments,"result":result});
                expected.push((
                    vm,
                    reviewer,
                    if module == "DEVELOPER_A" {
                        "ReviewA"
                    } else {
                        "ReviewB"
                    },
                    arguments.clone(),
                    false,
                ));
                expected.push((
                    reviewer,
                    store,
                    if module == "DEVELOPER_A" {
                        "AcceptedA"
                    } else {
                        "AcceptedB"
                    },
                    arguments.clone(),
                    false,
                ));
                for to in [vm, store, names("roles", module)?] {
                    expected.push((reviewer, to, "Approved", reply.clone(), true));
                }
            }
            "merge_code" => {
                if tests.len() != 4
                    || ordered.iter().any(|c| {
                        c.request["tool"] == "run_tests"
                            && c.result.as_ref().is_none_or(|(at, _)| *at >= call.emitted)
                    })
                {
                    return Err("merge lacks completed tests".into());
                }
                let c = candidate(&ordered, call.emitted)?;
                let reviewed = one(ordered.iter().copied().filter(|c| {
                    c.request["tool"] == "code_review"
                        && c.request["arguments"]["pr_id"] == arguments["branch"]
                        && c.result.as_ref().is_some_and(|(at, _)| *at < call.emitted)
                }))?;
                let approval = json!({"success":true,"candidate":c,"arguments":reviewed.request["arguments"],"result":reviewed.result()?.1});
                let summary = json!({"candidate":c,"rounds":[&tests[..2],&tests[2..]]});
                let ready = json!({"candidate":c,"tests":summary,"review":approval});
                expected.push((store, vm, "MergeReady", ready.clone(), false));
                expected.push((vm, store, "MergeIntent", arguments.clone(), false));
                expected.push((
                    repo,
                    store,
                    "MergeRequest",
                    json!({"arguments":arguments,"checked":ready}),
                    false,
                ));
                expected.push((store, repo, "Merged", result.clone(), true));
            }
            _ => return Err("unassigned shared-refactor effect".into()),
        }
        let mut before = call.emitted;
        for (from, to, key, value, reply) in expected {
            let label = names("labels", key)? as u8;
            let mut candidates = frames.iter().filter(|(id, f)| {
                !used_frames.contains(*id)
                    && f.direction(from, to)
                    && f.logical_label == label
                    && if reply {
                        f.sent > result_at
                    } else if matches!(key, "AcceptedA" | "AcceptedB") {
                        // The service has sent this notice before executing the effect.
                        // Send acceptance does not require STORE to have consumed it yet.
                        f.sent < call.emitted
                    } else {
                        f.received.is_some_and(|at| at < call.emitted)
                    }
            });
            let (id, frame) = if reply {
                candidates.min_by_key(|(_, f)| f.sent)
            } else {
                candidates.by_ref().max_by_key(|(_, f)| f.received)
            }
            .ok_or_else(|| format!("missing {key} for {agent} {tool}"))?;
            if frame.value()? != value {
                return Err(format!("altered {key} for {agent} {tool}").into());
            }
            frame.received()?;
            used_frames.insert(*id);
            // MergeReady is data delivery before the guest's merge request, not its admission.
            if !reply && key != "MergeReady" {
                before = before.min(frame.sent);
            }
        }
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            names("roles", io)?,
            before,
            &format!("service/{tool}"),
            arguments,
        )?;
    }
    if !scored.is_empty() {
        return Err("unmatched simulator effect".into());
    }
    let effect_labels = [
        "DesignA",
        "DesignB",
        "DesignResult",
        "OwnIntent",
        "SharedIntent",
        "Refactor",
        "Refactored",
        "TestA",
        "TestB",
        "ReadSnapshot",
        "RunTests",
        "RepositoryTested",
        "EnvironmentTested",
        "ReviewA",
        "ReviewB",
        "AcceptedA",
        "AcceptedB",
        "Approved",
        "MergeReady",
        "MergeIntent",
        "MergeRequest",
        "Merged",
    ]
    .into_iter()
    .map(|key| names("labels", key).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if frames
        .iter()
        .any(|(id, f)| effect_labels.contains(&f.logical_label) && !used_frames.contains(id))
    {
        return Err("unused or extra effect request, admission or reply".into());
    }
    for f in frames.values() {
        let key = ["TestPlan", "Candidate", "CandidateForOwner"]
            .into_iter()
            .find(|key| compiled["labels"][*key] == f.logical_label);
        if key.is_some() && f.value()? != candidate(&ordered, f.sent)? {
            return Err("candidate delivery differs from actual completed data".into());
        }
        if ["TestSummary", "TestSummaryB"]
            .into_iter()
            .any(|key| compiled["labels"][key] == f.logical_label)
            && (tests.len() != 4
                || ordered.iter().any(|c| {
                    c.request["tool"] == "run_tests"
                        && c.result.as_ref().is_none_or(|(at, _)| *at >= f.sent)
                })
                || f.value()?
                    != json!({"candidate":candidate(&ordered,f.sent)?,"rounds":[&tests[..2],&tests[2..]]}))
        {
            return Err("test summary differs from actual results".into());
        }
    }
    let mut review_test = 0;
    let mut merge_review = 0;
    for a in &ordered {
        for b in &ordered {
            if a.started()?.1 < b.ended()?.1 && b.started()?.1 < a.ended()?.1 {
                if a.tool()? == "code_review" && b.tool()? == "run_tests" {
                    review_test += 1;
                }
                if a.tool()? == "merge_code" && b.tool()? == "code_review" {
                    merge_review += 1;
                }
            }
        }
    }
    Ok(
        json!({"status":"passed","task":"4M","domain_calls":calls.len(),"simulator_complete":true,"violations":0,
        "review_test_overlaps":review_test,"merge_review_overlaps":merge_review,
        "effect_scope":"one-use admitted arguments, real resource requests, completed actual results and content data; EPF owns dynamic order"}),
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

/// Mutate saved observations only. These are auditor tests, not runtime attacks.
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    check(&rows, &artifact)?;
    fs::create_dir(output)?;
    let labels = crate::projection::frames(&artifact, &rows)?;
    let compiled = &artifact["compiled"];
    let business = [
        "DesignA",
        "DesignB",
        "DesignResult",
        "OwnIntent",
        "SharedIntent",
        "Refactor",
        "Refactored",
        "TestA",
        "TestB",
        "ReadSnapshot",
        "RunTests",
        "RepositoryTested",
        "EnvironmentTested",
        "ReviewA",
        "ReviewB",
        "AcceptedA",
        "AcceptedB",
        "Approved",
        "MergeReady",
        "MergeIntent",
        "MergeRequest",
        "Merged",
    ]
    .into_iter()
    .map(|key| {
        compiled["labels"][key]
            .as_u64()
            .ok_or("compiled effect label")
    })
    .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let selected = labels
        .iter()
        .filter(|(_, label)| business.contains(&u64::from(**label)))
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err("no actual business frames to mutate".into());
    }
    let mut verdicts = Vec::new();
    for id in selected {
        for kind in [
            "missing_frame",
            "coherent_payload_change",
            "duplicate_frame",
        ] {
            let mut changed = rows.clone();
            match kind {
                "missing_frame" => changed.retain(|row| {
                    !(row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == id)
                }),
                "coherent_payload_change" => {
                    for row in &mut changed {
                        let m = &mut row["message"];
                        if m["type"] == "protocol_frame" && m["frame_id"] == id {
                            let bytes = m["payload"]
                                .as_array()
                                .ok_or("frame payload")?
                                .iter()
                                .map(|v| {
                                    v.as_u64()
                                        .and_then(|v| u8::try_from(v).ok())
                                        .ok_or("frame byte")
                                })
                                .collect::<std::result::Result<Vec<_>, _>>()?;
                            let mut value: Value = serde_json::from_slice(&bytes)?;
                            let field = value
                                .as_object_mut()
                                .ok_or("business JSON object")?
                                .values_mut()
                                .next()
                                .ok_or("empty business payload")?;
                            *field = json!("unadmitted_value");
                            m["payload"] = json!(serde_json::to_vec(&value)?);
                        }
                    }
                }
                "duplicate_frame" => {
                    let duplicate = changed
                        .iter()
                        .filter(|row| {
                            row["message"]["type"] == "protocol_frame"
                                && row["message"]["frame_id"] == id
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    changed.extend(duplicate);
                }
                _ => unreachable!(),
            }
            let rejection = check(&changed, &artifact)
                .err()
                .ok_or("record mutation escaped actual effect audit")?;
            verdicts.push(json!({"kind":kind,"frame_id":id,"mutated_input_sha256":format!("{:x}",Sha256::digest(serde_json::to_vec(&changed)?)),"rejection":rejection.to_string()}));
        }
    }
    let result = json!({"status":"passed","task":"4M","rejected":verdicts.len(),"mutations":verdicts,
        "source_events_sha256":digest(path)?,"auditor_binary_sha256":digest(&std::env::current_exe()?)?,
        "scope":"static wire and one-use effect/admission correspondence; each change preserves all observations of its frame",
        "dynamic_mutations_checked":false,"runtime_attacks":0,"model_calls":0});
    crate::evidence::save(&output.join("summary.json"), &result)?;
    Ok(result)
}
