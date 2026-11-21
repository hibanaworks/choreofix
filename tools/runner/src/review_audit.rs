//! Actual review requests, results and effects. EPF checks source execution.
use crate::evidence::{Result, save};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::journal::{Call, one};

impl Call<'_> {
    fn target(&self) -> Result<&str> {
        let key = match self.tool {
            "implement_feature" => "feature_name",
            "code_review" => "pr_id",
            "merge_code" => "branch",
            _ => return Err("unexpected review-task tool".into()),
        };
        self.request["arguments"][key]
            .as_str()
            .ok_or_else(|| "effect target".into())
    }
}

fn role(projection: &Value, name: &str) -> Result<u64> {
    projection["compiled"]["roles"][name]
        .as_u64()
        .ok_or_else(|| format!("missing compiled review role {name}").into())
}

fn business_roles(projection: &Value) -> Result<BTreeSet<u64>> {
    [
        "REPO",
        "REVIEW_A",
        "REVIEW_B",
        "REVIEW_OWNER_A",
        "REVIEW_OWNER_B",
        "A_FILE",
        "B_FILE",
        "A_REVIEW_FILE",
        "B_REVIEW_FILE",
    ]
    .into_iter()
    .map(|name| role(projection, name))
    .collect()
}

pub(crate) fn check(rows: &[Value], projection: &Value, dynamic: &Value) -> Result<Value> {
    if projection["compiled"]["task"] != "6E" {
        return Err("review compiled protocol task".into());
    }
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if fixture["type"] != "fixtures"
        || fixture["task"] != "6E"
        || fixture["depth"].as_u64().is_none_or(|d| d > 2)
    {
        return Err("6E fixture".into());
    }
    let journal = crate::journal::read(rows, projection, dynamic)?;
    let actors = journal.participants;

    let crate::journal::Journal {
        frames,
        calls,
        scored_events,
        simulator_complete,
        ..
    } = journal;
    if frames.values().any(|frame| frame.identity["session"] != 6) {
        return Err("wrong session".into());
    }
    for call in calls.values() {
        let m = call.request;
        let actor = actors
            .iter()
            .find(|a| a["agent"] == m["agent"])
            .ok_or("unknown effect actor")?;
        if !actor["calls"]
            .as_array()
            .ok_or("original assignments")?
            .iter()
            .any(|a| {
                a["tool"] == m["tool"]
                    && a["arguments"].as_object().is_some_and(|expected| {
                        let Some(actual) = m["arguments"].as_object() else {
                            return false;
                        };
                        expected
                            .iter()
                            .all(|(key, value)| actual.get(key) == Some(value))
                            && actual.iter().all(|(key, value)| {
                                expected.contains_key(key)
                                    || (m["tool"] == "implement_feature"
                                        && key == "source"
                                        && value
                                            .as_str()
                                            .is_some_and(|text| !text.trim().is_empty()))
                            })
                    })
            })
        {
            return Err("effect differs from original assignment".into());
        }
    }
    let depth = fixture["depth"].as_u64().ok_or("depth")? as usize;
    let overrides = fixture["failure_overrides"]
        .as_array()
        .ok_or("failure overrides")?;
    let mut indices = BTreeSet::new();
    for failure in overrides {
        let index = failure[1].as_u64().ok_or("review failure index")? as usize;
        if failure[0] != "code_review" || !indices.insert(index) {
            return Err("unsupported review failure override".into());
        }
    }
    let rejections = depth + indices.len();
    if indices.iter().copied().ne(depth..rejections) {
        return Err("review overrides must extend the original failure prefix".into());
    }
    let expected = 6 + 2 * rejections;
    if !simulator_complete || calls.len() != expected {
        return Err("original completion or call count".into());
    }
    let mut scored = scored_events.iter().collect::<Vec<_>>();
    if scored.len() != expected {
        return Err("original event count".into());
    }
    for call in calls.values() {
        let (_, result) = call.result;
        let target = call.target()?;
        let args = match call.tool {
            "implement_feature" => json!({"feature":target,"source":result["data"]["source"]}),
            "code_review" => json!({"target":target}),
            "merge_code" => json!({"branch":target}),
            _ => unreachable!(),
        };
        if call.tool == "implement_feature"
            && call.request["arguments"]
                .get("source")
                .is_some_and(|source| *source != result["data"]["source"])
        {
            return Err("implementation changed actual caller source".into());
        }
        call.take_score(&mut scored, &args)?;
    }

    let repo = role(projection, "REPO")?;
    let business = business_roles(projection)?;
    let mut used_calls = BTreeSet::new();
    let mut checked_json = BTreeSet::new();
    let repository_calls = calls
        .values()
        .filter(|call| call.request["tool"] != "code_review")
        .count();
    let review_calls = calls
        .values()
        .filter(|call| call.request["tool"] == "code_review")
        .collect::<Vec<_>>();
    let mut ordered_calls = calls.iter().collect::<Vec<_>>();
    ordered_calls.sort_by_key(|(_, call)| call.emitted);
    for (id, call) in ordered_calls {
        let feature = match call.target()? {
            "user_authentication" | "dev_a_feature" => 0,
            "notification_feature" | "dev_b_feature" => 1,
            _ => return Err("unknown operation target".into()),
        };
        let (file, reviewer, owner, review_file) = if feature == 0 {
            ("A_FILE", "REVIEW_A", "REVIEW_OWNER_A", "A_REVIEW_FILE")
        } else {
            ("B_FILE", "REVIEW_B", "REVIEW_OWNER_B", "B_REVIEW_FILE")
        };
        let (file, reviewer, owner, review_file) = (
            role(projection, file)?,
            role(projection, reviewer)?,
            role(projection, owner)?,
            role(projection, review_file)?,
        );
        let (result_at, result) = call.result;
        let next = calls
            .values()
            .filter(|other| {
                other.emitted > call.emitted
                    && other.request["tool"] == call.request["tool"]
                    && other.target().ok() == call.target().ok()
            })
            .map(|other| other.emitted)
            .min()
            .unwrap_or(rows.len());
        let (from, to, request_label, reply_label) = match call.tool {
            "implement_feature" => (file, repo, 200 + feature, 208 + feature),
            "merge_code" => (file, repo, 202 + feature, 210 + feature),
            "code_review" => (review_file, reviewer, 5 + feature, 5 + feature),
            _ => return Err("unknown effect".into()),
        };
        let request = one(frames.values().filter(|f| {
            f.direction(from, to)
                && u64::from(f.logical_label) == request_label
                && !checked_json.contains(&f.sent)
                && f.received < call.emitted
                && f.value().is_ok_and(|v| v == call.request["arguments"])
        }))?;
        checked_json.insert(request.sent);
        let reply = one(frames.values().filter(|f| {
            f.direction(to, from)
                && u64::from(f.logical_label) == reply_label
                && !checked_json.contains(&f.sent)
                && f.sent > result_at
                && f.sent < next
                && f.value().is_ok_and(|v| v == *result)
        }))?;
        checked_json.insert(reply.sent);
        used_calls.insert(*id);
        match call.tool {
            "implement_feature" => {
                if result["success"] != true {
                    return Err("implementation failed".into());
                }
                let ready = one(frames.values().filter(|f| {
                    f.direction(repo, reviewer)
                        && u64::from(f.logical_label) == 5 + feature
                        && f.sent > result_at
                        && f.sent < next
                        && f.value().is_ok_and(|v| v == *result)
                }))?;
                if !checked_json.insert(ready.sent) {
                    return Err("reused implementation readiness".into());
                }
            }
            "code_review" => {
                let previous = review_calls
                    .iter()
                    .filter(|c| c.target().ok() == call.target().ok() && c.emitted < call.emitted)
                    .map(|c| c.result.0)
                    .max()
                    .unwrap_or(0);
                let implementation = calls
                    .values()
                    .filter(|c| {
                        c.tool == "implement_feature"
                            && c.target().ok() == call.target().ok()
                            && previous < c.started.0
                            && c.started.0 < call.emitted
                    })
                    .max_by_key(|c| c.started.0)
                    .ok_or("review without a fresh implementation")?;
                if implementation.ended.0 >= call.started.0 {
                    return Err("review before implementation".into());
                }
                let (impl_at, impl_result) = implementation.result;
                one(frames.values().filter(|f| {
                    f.direction(repo, reviewer)
                        && u64::from(f.logical_label) == 5 + feature
                        && f.sent > impl_at
                        && f.received < call.emitted
                        && f.value().is_ok_and(|v| v == *impl_result)
                }))?;
                if result["data"]["target"] != call.request["arguments"]["pr_id"]
                    || result["data"]["candidate_sha256"] != impl_result["data"]["candidate_sha256"]
                {
                    return Err("review names another actual candidate".into());
                }
                let success = result["success"].as_bool().ok_or("review outcome")?;
                let label = if feature == 0 {
                    if success { 30 } else { 31 }
                } else if success {
                    40
                } else {
                    41
                };
                let approval = one(frames.values().filter(|f| {
                    f.direction(owner, repo)
                        && f.logical_label == label
                        && f.sent > result_at
                        && f.sent < next
                        && !checked_json.contains(&f.sent)
                        && f.value().is_ok_and(|v| v == *result)
                }))?;
                let approved_at = approval.received;
                checked_json.insert(approval.sent);
                let feedback = one(frames.values().filter(|f| {
                    f.direction(repo, file)
                        && u64::from(f.logical_label) == file
                        && f.sent > approved_at
                        && f.sent < next
                        && !checked_json.contains(&f.sent)
                        && f.value().is_ok_and(|v| v == *result)
                }))?;
                checked_json.insert(feedback.sent);
            }
            "merge_code" => {
                let feature = if feature == 0 {
                    "user_authentication"
                } else {
                    "notification_feature"
                };
                let reviewed = review_calls
                    .iter()
                    .filter(|c| c.target().is_ok_and(|v| v == feature) && c.ended.0 < call.emitted)
                    .max_by_key(|c| c.ended.0)
                    .ok_or("merge without review")?;
                let implemented = calls
                    .values()
                    .filter(|c| {
                        c.tool == "implement_feature"
                            && c.target().is_ok_and(|v| v == feature)
                            && c.started.0 < call.emitted
                    })
                    .max_by_key(|c| c.started.0)
                    .ok_or("merge without implementation")?;
                if result["success"] != true
                    || reviewed.result.1["success"] != true
                    || implemented.ended.0 >= reviewed.started.0
                {
                    return Err("merge lacks approval for the current implementation".into());
                }
            }
            _ => unreachable!(),
        }
    }
    if used_calls.len() != calls.len() {
        return Err("unmatched effect".into());
    }
    let actual_json = frames
        .values()
        .filter(|f| {
            let Some(from) = f.identity["source"].as_u64() else {
                return false;
            };
            let Some(to) = f.identity["target"].as_u64() else {
                return false;
            };
            business.contains(&from)
                && business.contains(&to)
                && serde_json::from_slice::<Value>(&f.bytes).is_ok_and(|v| v.is_object())
        })
        .map(|f| f.sent)
        .collect::<BTreeSet<_>>();
    if actual_json != checked_json {
        return Err("missing, duplicate or extra actual business data".into());
    }
    let reviews = review_calls.len();
    let effects: Vec<_> = calls.values().collect();
    let mut overlap = BTreeMap::<String, usize>::new();
    for (i, first) in effects.iter().enumerate() {
        for second in effects.iter().skip(i + 1) {
            if first.started.1.max(second.started.1) < first.ended.1.min(second.ended.1) {
                let mut names = [first.tool, second.tool];
                names.sort();
                *overlap.entry(names.join("+")).or_default() += 1;
            }
        }
    }
    let mut selections: Vec<_> = calls
        .values()
        .filter(|call| call.tool == "code_review")
        .collect();
    selections.sort_by_key(|call| call.emitted);
    let targets = selections
        .iter()
        .map(|call| call.target())
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"wire_validation":"compiled wire membership, review contracts and EPF source-execution interpretation","status":"passed","task":"6E","kind":"completed_actual_wasi_review_trace","scored_benchmark":false,
        "depth":fixture["depth"],"carrier_frames":frames.len(),"domain_calls":calls.len(),"actual_resource_endpoint_requests":repository_calls,
        "repository_endpoint_lifetime":"one initial loan, retained by its owner until run end","actual_review_requests":reviews,"guest_exits":actors.len(),"simulator_complete":true,
        "violations":0,"overlapping_effect_pairs":overlap,"review_targets":targets,
        "direct_file_requests_and_replies":true,"effect_receives_bound_to_checked_occurrences":true,"dynamic_protocol":dynamic}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let evidence = crate::journal::Evidence::open(path)?;
    let audited = evidence.audit("6E")?;
    let (rows, projection) = (evidence.rows, evidence.projection);
    let dynamic = &audited["dynamic_protocol"];
    let journal = crate::journal::read(&rows, &projection, dynamic)?;
    let business = business_roles(&projection)?;
    let selected = journal
        .frames
        .iter()
        .filter(|(_, f)| {
            let Some(from) = f.identity["source"].as_u64() else {
                return false;
            };
            let Some(to) = f.identity["target"].as_u64() else {
                return false;
            };
            business.contains(&from)
                && business.contains(&to)
                && serde_json::from_slice::<Value>(&f.bytes).is_ok_and(|v| v.is_object())
        })
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
    fs::create_dir(output)?;
    let mut verdicts = crate::journal::json_frame_mutants(&rows, &selected, |changed| {
        check(changed, &projection, dynamic)
    })?;
    let id = *selected.first().ok_or("business frame")?;
    for (field, value) in [("frame_label", 65535), ("lane", 255)] {
        let mut changed = rows.clone();
        for row in &mut changed {
            if row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id {
                row["message"][field] = json!(value);
            }
        }
        let error = check(&changed, &projection, dynamic)
            .err()
            .ok_or("coherent invalid wire header accepted")?;
        verdicts.push(json!({"frame_id":id,"kind":format!("coherent_wrong_{field}"),"rejected":true,"error":error.to_string()}));
    }
    let result = json!({"status":"passed","task":"6E","source_events_sha256":evidence.events_hash,"audit":audited,"count":verdicts.len(),"mutants":verdicts,"scope":"Saved-record typed file, admission, actual request/result, feedback and reply mutations with checked receive occurrences; static wire-header membership. Does not repeat EPF replay or certify runtime attacks.","model_calls":0});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
