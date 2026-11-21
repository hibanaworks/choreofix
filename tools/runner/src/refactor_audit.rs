//! Code-content and actual effect correspondence; EPF owns protocol execution.
use crate::{
    evidence::Result,
    journal::{self, Call, one, take_json_frame},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};
#[cfg(test)]
use {crate::evidence::digest, std::path::Path};

fn python_string(value: &str) -> Result<String> {
    let mut out = String::new();
    for ch in serde_json::to_string(value)?.chars() {
        let n = ch as u32;
        if n < 127 {
            out.push(ch);
        } else if n <= 65535 {
            write!(out, "\\u{n:04x}")?;
        } else {
            let n = n - 65536;
            write!(
                out,
                "\\u{:04x}\\u{:04x}",
                0xd800 + (n >> 10),
                0xdc00 + (n & 1023)
            )?;
        }
    }
    Ok(out)
}
fn identity(module: &str, source: &str, common: &str) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(format!(
            "{{\"common\": {}, \"{module}\": {}}}",
            python_string(common)?,
            python_string(source)?
        ))
    ))
}
fn code_before<'a>(calls: &'a BTreeMap<u64, Call>, target: &str, before: usize) -> Result<&'a str> {
    let call = calls
        .values()
        .filter(|c| {
            c.tool == "refactor_code"
                && c.request["arguments"]["target"] == target
                && c.ended.0 < before
        })
        .max_by_key(|c| c.ended.0)
        .ok_or("missing completed refactor material")?;
    call.result.1["data"]["source"]
        .as_str()
        .ok_or_else(|| "refactor source bytes".into())
}
fn candidate(calls: &BTreeMap<u64, Call>, module: &str, before: usize) -> Result<String> {
    identity(
        module,
        code_before(calls, module, before)?,
        code_before(calls, "SHARED_LIBRARY", before)?,
    )
}
fn checked(value: &Value, module: &str, hash: &str) -> Result<()> {
    if value["success"] != true
        || value["data"]["candidate_sha256"] != hash
        || (value["tool_name"] == "code_review" && value["data"]["target"] != module)
        || (value["tool_name"] == "run_tests"
            && value["data"]["suite"] != format!("{module}_tests"))
    {
        return Err("validation is not for the actual current module/library bytes".into());
    }
    let outputs = value["data"]["outputs"]
        .as_array()
        .ok_or("executed code outputs")?;
    let offset = if module == "module_a" { 1 } else { 2 };
    if outputs.len() != 3
        || outputs.iter().zip([-3, 0, 7]).any(|(row, input)| {
            row["input"] != input
                || row["actual"] != input * 2 + offset
                || row["expected"] != input * 2 + offset
        })
    {
        return Err("validation outputs differ from the common task's actual arithmetic".into());
    }
    Ok(())
}
pub(crate) fn check(rows: &[Value], projection: &Value, dynamic: &Value) -> Result<Value> {
    let c = &projection["compiled"];
    let fixture = &rows.first().ok_or("empty refactor record")?["message"];
    if c["task"] != "4M" || fixture["task"] != "4M" || fixture["type"] != "fixtures" {
        return Err("shared-refactor source and run differ".into());
    }
    let role = |name: &str| -> Result<u64> {
        c["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled role {name}").into())
    };
    let label = |name: &str| -> Result<u64> {
        c["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled label {name}").into())
    };
    let journal = crate::journal::read(rows, projection, dynamic)?;
    let actors = journal.participants;

    let journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if !simulator_complete
        || calls.len() != 12
        || frames
            .values()
            .any(|f| f.identity["session"] != c["session"])
    {
        return Err(
            "incomplete/foreign refactor run, extra effects or common-world violation".into(),
        );
    }
    let pairs = ["ARCH", "A", "B", "REVIEW", "TEST"]
        .map(|n| Ok((role(&format!("{n}_VM"))?, role(&format!("{n}_IO"))?)))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(&frames, &pairs)?;
    let mut scored = scored_events.iter().collect::<Vec<_>>();
    let mut used = BTreeSet::new();
    for (suffix, module) in [("A", "module_a"), ("B", "module_b")] {
        let candidate_label = label(&format!("Candidate{suffix}"))?;
        let (repo, owner) = (role("REPO")?, role("REPO_OWNER")?);
        let snapshot = one(frames.values().filter(|f| {
            f.direction(repo, owner) && u64::from(f.logical_label) == candidate_label
        }))?;
        let request_label = label(&format!("ReadCandidate{suffix}"))?;
        let request = one(frames
            .values()
            .filter(|f| f.direction(owner, repo) && u64::from(f.logical_label) == request_label))?;
        if !request.bytes.is_empty() || request.received >= snapshot.sent {
            return Err("repository snapshot does not answer its typed read".into());
        }
        used.insert(request.sent);
        let at = snapshot.sent;
        let latest = calls
            .values()
            .filter(|c| c.tool == "refactor_code")
            .map(|c| (c.ended.0, c))
            .filter(|(end, _)| *end < at)
            .max_by_key(|(end, _)| *end)
            .ok_or("candidate lacks repository edit")?
            .1;
        let snapshot_value = snapshot.value()?;
        if snapshot_value != latest.result.1["data"]["working_candidates"]
            || snapshot_value[module] != candidate(&calls, module, at)?
        {
            return Err("snapshot differs from the actual repository contents".into());
        }
        used.insert(at);
    }
    let mut matched = BTreeSet::new();
    for call in calls.values() {
        let agent = call.request["agent"].as_str().ok_or("effect agent")?;
        let tool = call.tool;
        let args = &call.request["arguments"];
        let actor = one(actors.iter().filter(|a| a["agent"] == agent))?;
        if !actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .any(|a| a["tool"] == tool && a["arguments"] == *args)
            || !matched.insert((
                agent.to_owned(),
                tool.to_owned(),
                serde_json::to_string(args)?,
            ))
        {
            return Err("unassigned or duplicated original effect".into());
        }
        let (at, value) = call.result;
        let data = &value["data"];
        if value["tool_name"] != tool || value["success"] != true {
            return Err("actual effect result differs".into());
        }
        let module = match tool {
            "design_proposal" => args["title"]
                .as_str()
                .and_then(|s| s.strip_suffix("_refactor")),
            "refactor_code" => args["target"].as_str(),
            "code_review" => args["pr_id"].as_str(),
            "run_tests" => args["suite"]
                .as_str()
                .and_then(|s| s.strip_suffix("_tests")),
            "merge_code" => args["branch"].as_str(),
            _ => None,
        }
        .ok_or("assigned target")?;
        let suffix = if module == "module_a" || agent == "DEVELOPER_A" {
            "A"
        } else {
            "B"
        };
        let (from, to, request, reply) = match tool {
            "design_proposal" => (
                role("ARCH_FILE")?,
                role("ARCHITECT")?,
                format!("Design{suffix}"),
                format!("Designed{suffix}"),
            ),
            "refactor_code" => (
                role(&format!("{suffix}_FILE"))?,
                role("REPO")?,
                if module == "SHARED_LIBRARY" {
                    format!("RefactorShared{suffix}")
                } else {
                    format!("Refactor{suffix}")
                },
                if module == "SHARED_LIBRARY" {
                    format!("SharedChanged{suffix}")
                } else {
                    format!("Refactored{suffix}")
                },
            ),
            "code_review" => (
                role("REVIEW_FILE")?,
                role("REVIEWER")?,
                format!("ReviewRequest{suffix}"),
                format!("FileReviewed{suffix}"),
            ),
            "run_tests" => (
                role("TEST_FILE")?,
                role("TEST_ENV")?,
                format!("TestRequest{suffix}"),
                format!("FileTested{suffix}"),
            ),
            "merge_code" => (
                role(&format!("{suffix}_FILE"))?,
                role("REPO_OWNER")?,
                format!("Merge{suffix}"),
                format!("Merged{suffix}"),
            ),
            _ => return Err("unknown original effect".into()),
        };
        let input = take_json_frame(
            (&frames, &mut used),
            (from, to, &[label(&request)?]),
            (args, 0, call.emitted),
        )?;
        if input.received >= call.emitted {
            return Err("effect preceded its checked receive".into());
        }
        take_json_frame(
            (&frames, &mut used),
            (to, from, &[label(&reply)?]),
            (value, at, usize::MAX),
        )?;
        let score_args = match tool {
            "design_proposal" => json!({"proposal":args["title"]}),
            "code_review" => json!({"target":args["pr_id"]}),
            _ => args.clone(),
        };
        call.take_score(&mut scored, &score_args)?;
        match tool {
            "design_proposal" => {
                if data["proposal"] != args["title"] || !data["document"].is_string() {
                    return Err("design data differs".into());
                }
            }
            "refactor_code" => {
                let source = data["source"].as_str().ok_or("actual changed source")?;
                if data["target"] != module
                    || data["content_sha256"] != format!("{:x}", Sha256::digest(source.as_bytes()))
                {
                    return Err("changed source content hash differs".into());
                }
            }
            "code_review" | "run_tests" => {
                let hash = candidate(&calls, module, call.started.0)?;
                checked(value, module, &hash)?;
                let repo = role("REPO_OWNER")?;
                let ready_label = label(&format!(
                    "Ready{}{suffix}",
                    if tool == "code_review" {
                        "Review"
                    } else {
                        "Test"
                    }
                ))?;
                let ready = one(frames.values().filter(|f| {
                    f.direction(repo, to)
                        && u64::from(f.logical_label) == ready_label
                        && f.sent < call.emitted
                }))?;
                if ready.value()?[module] != hash || ready.received >= call.emitted {
                    return Err("validation input differs from actual current code".into());
                }
                used.insert(ready.sent);
                take_json_frame(
                    (&frames, &mut used),
                    (
                        to,
                        role("REPO_OWNER")?,
                        &[label(&format!(
                            "{}{suffix}",
                            if tool == "code_review" {
                                "Reviewed"
                            } else {
                                "Tested"
                            }
                        ))?],
                    ),
                    (value, at, usize::MAX),
                )?;
                take_json_frame(
                    (&frames, &mut used),
                    (
                        role("REPO_OWNER")?,
                        role(&format!("{suffix}_FILE"))?,
                        &[label(&format!(
                            "{}Feedback{suffix}",
                            if tool == "code_review" {
                                "Review"
                            } else {
                                "Test"
                            }
                        ))?],
                    ),
                    (value, at, usize::MAX),
                )?;
            }
            "merge_code" => {
                let hash = candidate(&calls, module, call.started.0)?;
                if data["branch"] != module
                    || data["candidate_sha256"] != hash
                    || data["published_candidates"][module] != hash
                {
                    return Err("publication does not contain its current validated code".into());
                }
                let mut validation = BTreeMap::new();
                for (kind, operation, actor, key, target) in [
                    (
                        "review",
                        "code_review",
                        "REVIEWER",
                        "target",
                        module.to_owned(),
                    ),
                    (
                        "test",
                        "run_tests",
                        "TESTER",
                        "suite",
                        format!("{module}_tests"),
                    ),
                ] {
                    let index = data["validation_receipts"][module][kind]
                        .as_u64()
                        .ok_or("publication validation occurrence")?
                        as usize;
                    let event = scored_events.get(index).ok_or("validation event index")?;
                    if event["agent"] != actor
                        || event["tool"] != operation
                        || event["result"][key] != target
                        || event["result"]["candidate_sha256"] != hash
                        || event["success"] != true
                    {
                        return Err("publication reused unrelated validation".into());
                    }
                    let checked_call = one(calls.values().filter(|p| {
                        p.tool == operation
                            && p.request["agent"] == actor
                            && p.result.1["data"] == event["result"]
                    }))?;
                    if checked_call.ended.0 >= call.started.0 {
                        return Err("publication used unfinished validation".into());
                    }
                    validation.insert(kind, checked_call.result.1.clone());
                }
                let admitted = json!({"arguments":args,
                    "review":validation["review"],"test":validation["test"]});
                let publication = take_json_frame(
                    (&frames, &mut used),
                    (
                        role("REPO_OWNER")?,
                        role("REPO")?,
                        &[label(&format!("Publication{suffix}"))?],
                    ),
                    (&admitted, input.received, call.emitted),
                )?;
                if publication.received >= call.emitted {
                    return Err("publication effect preceded its evidence-bearing receive".into());
                }
                take_json_frame(
                    (&frames, &mut used),
                    (
                        role("REPO")?,
                        role("REPO_OWNER")?,
                        &[label(&format!("Published{suffix}"))?],
                    ),
                    (value, at, usize::MAX),
                )?;
            }
            _ => unreachable!(),
        }
    }
    let business_roles = [
        "ARCH_FILE",
        "A_FILE",
        "B_FILE",
        "REVIEW_FILE",
        "TEST_FILE",
        "ARCHITECT",
        "REVIEWER",
        "REPO",
        "REPO_OWNER",
        "TEST_ENV",
    ]
    .into_iter()
    .map(role)
    .collect::<Result<BTreeSet<_>>>()?;
    for frame in frames.values() {
        let from = frame.identity["source"].as_u64().ok_or("frame source")?;
        let to = frame.identity["target"].as_u64().ok_or("frame target")?;
        if business_roles.contains(&from)
            && business_roles.contains(&to)
            && frame.identity["delegated_role"].is_null()
            && ![label("ReviewExit")?, label("TestExit")?, label("RepoExit")?]
                .contains(&u64::from(frame.logical_label))
            && !used.contains(&frame.sent)
        {
            return Err("unused or extra business observation".into());
        }
    }
    if !scored.is_empty() {
        return Err("unused common-scored effects".into());
    }
    Ok(
        json!({"status":"passed","task":"4M","domain_calls":calls.len(),"violations":0,"simulator_complete":true,
        "actual_code_hashes_checked":true,"publication_uses_completed_current_review_and_tests":true,
        "effect_receives_bound_to_checked_occurrences":true,"guest_exits":5,
        "compiled_projection_sha256":projection["projection_sha256"],"runtime_binary_sha256":projection["runtime_binary_sha256"],
        "dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}

/// Data/effect counterexamples reuse this capture's one EPF check.
#[cfg(test)]
pub fn check_mutants(path: &Path, output: &Path) -> Result<(Value, Value)> {
    let evidence = crate::journal::Evidence::open(path)?;
    let baseline = evidence.audit("4M")?;
    let (rows, artifact) = (evidence.rows, evidence.projection);
    let dynamic = &baseline["dynamic_protocol"];
    let effect = rows
        .iter()
        .position(|r| {
            r["direction"] == "runtime"
                && r["message"]["type"] == "execute"
                && r["message"]["tool"] == "merge_code"
        })
        .ok_or("missing publication counterexample input")?;
    let id = rows[effect]["message"]["id"].clone();
    let end = rows
        .iter()
        .position(|r| r["direction"] == "domain_end" && r["message"]["id"] == id)
        .ok_or("missing effect result")?;
    let score = rows
        .iter()
        .position(|r| r["message"]["type"] == "score")
        .ok_or("missing actual score")?;
    let publication_label = artifact["compiled"]["labels"]["PublicationA"].clone();
    let contracts = crate::projection::contracts(&artifact["compiled"])?;
    let publication_wire = one(contracts
        .iter()
        .filter(|(_, header)| header[3] == publication_label))?
    .0;
    let publication = rows
        .iter()
        .position(|r| {
            r["message"]["type"] == "protocol_frame"
                && r["message"]["frame_label"] == *publication_wire
        })
        .ok_or("missing publication frame")?;
    let mut reports = Vec::new();
    for name in [
        "changed_effect_target",
        "changed_effect_result",
        "negative_effect_result",
        "missing_publication_request",
        "substituted_publication_arguments",
        "stale_publication_review",
        "missing_actual_review",
        "forged_publication_score",
        "missing_common_score",
    ] {
        let mut changed = rows.clone();
        match name {
            "changed_effect_target" => {
                changed[effect]["message"]["arguments"]["branch"] = json!("other")
            }
            "changed_effect_result" => {
                changed[end]["message"]["result"]["data"]["candidate_sha256"] = json!("old")
            }
            "negative_effect_result" => changed[end]["message"]["result"]["success"] = json!(false),
            "missing_publication_request" => {
                changed.remove(publication);
            }
            "substituted_publication_arguments" | "stale_publication_review" => {
                let bytes: Vec<u8> =
                    serde_json::from_value(changed[publication]["message"]["payload"].clone())?;
                let mut value: Value = serde_json::from_slice(&bytes)?;
                if name == "substituted_publication_arguments" {
                    value["arguments"]["branch"] = json!("module_b");
                } else {
                    value["review"]["data"]["candidate_sha256"] = json!("old");
                }
                changed[publication]["message"]["payload"] = json!(serde_json::to_vec(&value)?);
            }
            "missing_actual_review" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        r["direction"] == "domain_end" && r["message"]["tool"] == "code_review"
                    })
                    .ok_or("review result")?;
                changed.remove(at);
            }
            "forged_publication_score" => {
                let events = changed[score]["message"]["events"]
                    .as_array_mut()
                    .ok_or("score events")?;
                let event = events
                    .iter_mut()
                    .find(|e| e["tool"] == "merge_code")
                    .ok_or("scored publication")?;
                event["result"]["candidate_sha256"] = json!("old");
            }
            "missing_common_score" => {
                changed.remove(score);
            }
            _ => unreachable!(),
        }
        match check(&changed, &artifact, dynamic) {
            Ok(_) => return Err(format!("refactor auditor accepted {name}").into()),
            Err(error) => {
                reports.push(json!({"name":name,"rejected":true,"error":error.to_string()}))
            }
        }
    }
    let report = json!({"task":"4M","status":"passed","record_counterexamples":reports,
        "original_events_sha256":digest(path)?,"dynamic_capture_checked_once":true,
        "runtime_attacks":false,"model_calls":0,"scored_runs":0});
    crate::evidence::save(output, &report)?;
    Ok((baseline, report))
}
