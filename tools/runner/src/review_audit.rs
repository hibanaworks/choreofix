//! Checks completed 6E WASI histories against resource transfers and original
//! simulator calls. This module is used only after a run.
use crate::evidence::{Result, digest, save};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

struct Frame {
    logical_label: u8,
    identity: Value,
    bytes: Vec<u8>,
    sent: usize,
    received: Option<usize>,
    queued: bool,
}
impl Frame {
    fn direction(&self, from: u64, to: u64) -> bool {
        self.identity["source"] == from && self.identity["target"] == to
    }
    fn received(&self) -> Result<usize> {
        self.received.ok_or_else(|| "unreceived frame".into())
    }
    fn value(&self) -> Result<Value> {
        Ok(serde_json::from_slice(&self.bytes)?)
    }
}
struct Call {
    request: Value,
    emitted: usize,
    started: Option<(usize, f64)>,
    result: Option<(usize, Value)>,
    domain_result: Option<Value>,
    ended: Option<(usize, f64)>,
}
impl Call {
    fn started(&self) -> Result<(usize, f64)> {
        self.started.ok_or_else(|| "missing domain start".into())
    }
    fn ended(&self) -> Result<(usize, f64)> {
        self.ended.ok_or_else(|| "missing domain end".into())
    }
    fn result(&self) -> Result<(usize, &Value)> {
        self.result
            .as_ref()
            .map(|(at, value)| (*at, value))
            .ok_or_else(|| "missing original result".into())
    }
    fn tool(&self) -> Result<&str> {
        self.request["tool"]
            .as_str()
            .ok_or_else(|| "tool name".into())
    }
    fn target(&self) -> Result<&str> {
        let key = match self.tool()? {
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
fn one<'a>(items: impl Iterator<Item = &'a Frame>) -> Result<&'a Frame> {
    let mut items = items;
    let frame = items.next().ok_or("missing matching frame")?;
    if items.next().is_some() {
        return Err("multiple matching frames".into());
    }
    Ok(frame)
}
fn records(path: &Path) -> Result<Vec<Value>> {
    fs::read_to_string(path)?
        .lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}
fn time(row: &Value) -> Result<f64> {
    row["elapsed_seconds"]
        .as_f64()
        .filter(|t| t.is_finite() && *t >= 0.0)
        .ok_or_else(|| "invalid observation time".into())
}
fn close_arguments(
    frames: &BTreeMap<u64, Frame>,
    used: &mut BTreeSet<usize>,
    vm: u64,
    io: u64,
    before: usize,
    path: &str,
    arguments: &Value,
) -> Result<()> {
    let closed = frames
        .values()
        .filter(|f| {
            f.direction(io, vm)
                && f.bytes.first() == Some(&4)
                && f.bytes.len() >= 4
                && f.received.is_some_and(|at| at < before)
        })
        .max_by_key(|f| f.received)
        .ok_or("missing actual WASI close result")?;
    let length = usize::from(closed.bytes[3]);
    if closed.bytes[1..3] != [0, 0]
        || closed.bytes.get(4..4 + length) != Some(path.as_bytes())
        || serde_json::from_slice::<Value>(closed.bytes.get(4 + length..).ok_or("close pathname")?)?
            != *arguments
    {
        return Err("WASI close differs from actual business arguments".into());
    }
    if !frames.values().any(|f| {
        f.direction(vm, io) && f.bytes == [4] && f.received.is_some_and(|at| at < closed.sent)
    }) {
        return Err("close result without projected WASI request".into());
    }
    if !used.insert(closed.sent) {
        return Err("WASI close reused for another effect".into());
    }
    Ok(())
}

fn check_close_pairs(frames: &BTreeMap<u64, Frame>) -> Result<()> {
    let mut used = BTreeSet::new();
    for vm in [0, 1, 2] {
        for reply in frames.values().filter(|f| {
            f.direction(vm + 8, vm)
                && f.bytes.first() == Some(&4)
                && f.bytes.len() >= 4
                && f.bytes
                    .get(4..4 + usize::from(f.bytes[3]))
                    .is_some_and(|path| path.starts_with(b"service/"))
        }) {
            let request = frames
                .values()
                .filter(|f| f.direction(vm, vm + 8) && f.sent < reply.sent)
                .max_by_key(|f| f.sent)
                .ok_or("business close without WASI request")?;
            if request.bytes != [4]
                || request.received()? >= reply.sent
                || !used.insert(request.sent)
            {
                return Err("business close request reused or out of order".into());
            }
        }
    }
    Ok(())
}

fn check(rows: &[Value], projection: &Value) -> Result<Value> {
    if projection["compiled"]["task"] != "6E" {
        return Err("review compiled protocol task".into());
    }
    let labels = crate::projection::frames(projection, rows)?;
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if fixture["type"] != "fixtures"
        || fixture["task"] != "6E"
        || fixture["depth"].as_u64().is_none_or(|d| d > 2)
    {
        return Err("6E fixture".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("original actors")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str().unwrap_or(""))
            .collect::<BTreeSet<_>>()
            != BTreeSet::from(["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"])
    {
        return Err("6E participants".into());
    }
    let mut frames = BTreeMap::<u64, Frame>::new();
    let mut calls = BTreeMap::<u64, Call>::new();
    let mut exits = BTreeSet::new();
    let mut verdict = None;
    let mut score = None;
    for (index, row) in rows.iter().enumerate() {
        let m = &row["message"];
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                if m["session"] != 6 {
                    return Err("wrong session".into());
                }
                let id = m["frame_id"].as_u64().ok_or("frame ID")?;
                let mut identity = m.clone();
                identity
                    .as_object_mut()
                    .ok_or("frame object")?
                    .remove("action");
                match m["action"].as_str() {
                    Some("send") => {
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
                        if frames
                            .insert(
                                id,
                                Frame {
                                    logical_label: *labels.get(&id).ok_or("unmapped frame")?,
                                    identity,
                                    bytes,
                                    sent: index,
                                    received: None,
                                    queued: true,
                                },
                            )
                            .is_some()
                        {
                            return Err("duplicate frame send".into());
                        }
                    }
                    Some("receive" | "requeue") => {
                        let frame = frames.get_mut(&id).ok_or("frame without send")?;
                        if frame.identity != identity {
                            return Err("altered frame identity".into());
                        }
                        if m["action"] == "receive" && frame.queued {
                            frame.received = Some(index);
                            frame.queued = false;
                        } else if m["action"] == "requeue"
                            && !frame.queued
                            && frame.received.is_some()
                        {
                            frame.queued = true;
                        } else {
                            return Err("invalid frame observation".into());
                        }
                    }
                    _ => return Err("unknown frame action".into()),
                }
            }
            (Some("runtime"), Some("execute")) => {
                let id = m["id"].as_u64().ok_or("effect ID")?;
                let actor = actors
                    .iter()
                    .find(|a| a["agent"] == m["agent"])
                    .ok_or("unknown effect actor")?;
                if !actor["calls"]
                    .as_array()
                    .ok_or("original assignments")?
                    .iter()
                    .any(|a| a["tool"] == m["tool"] && a["arguments"] == m["arguments"])
                {
                    return Err("effect differs from original assignment".into());
                }
                if calls
                    .insert(
                        id,
                        Call {
                            request: m.clone(),
                            emitted: index,
                            started: None,
                            result: None,
                            domain_result: None,
                            ended: None,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate effect ID".into());
                }
            }
            (Some("domain_start"), Some("execute")) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("domain ID")?)
                    .ok_or("domain start without runtime request")?;
                if *m != call.request || call.started.replace((index, time(row)?)).is_some() {
                    return Err("altered domain request".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("result ID")?)
                    .ok_or("result without request")?;
                if call.domain_result.as_ref() != Some(&m["result"])
                    || call.result.replace((index, m["result"].clone())).is_some()
                {
                    return Err("duplicate original result".into());
                }
            }
            (Some("domain_end"), _) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("domain end ID")?)
                    .ok_or("domain end without request")?;
                if call.request["agent"] != m["agent"]
                    || call.request["tool"] != m["tool"]
                    || call.ended.replace((index, time(row)?)).is_some()
                {
                    return Err("altered domain result".into());
                }
                call.domain_result = Some(m["result"].clone());
            }
            (Some("runtime"), Some("exit")) => {
                let agent = m["agent"].as_str().ok_or("exit agent")?;
                if !actors.iter().any(|a| a["agent"] == agent)
                    || m["status"] != 0
                    || !exits.insert(agent.to_owned())
                {
                    return Err("invalid guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if m["status"] != "complete" || verdict.replace(index).is_some() {
                    return Err("incomplete runtime verdict".into());
                }
            }
            (Some("simulator"), Some("score")) if score.replace(m).is_some() => {
                return Err("duplicate original score".into());
            }
            _ => {}
        }
    }
    if frames.values().any(|f| f.queued || f.received.is_none())
        || exits.len() != 3
        || verdict.is_none()
    {
        return Err("incomplete observed execution".into());
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
    let score = score.ok_or("missing original score")?;
    if score["complete"] != true || score["violations"] != json!([]) || calls.len() != expected {
        return Err("original completion or call count".into());
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("original scored events")?
        .clone();
    if scored.len() != expected {
        return Err("original event count".into());
    }
    for call in calls.values() {
        let (start, started) = call.started()?;
        let (end, finished) = call.ended()?;
        let (result_at, result) = call.result()?;
        if !(call.emitted < start
            && start < end
            && end < result_at
            && result_at < verdict.unwrap()
            && started <= finished)
            || result["tool_name"] != call.request["tool"]
        {
            return Err("invalid effect/result interval".into());
        }
        let target = call.target()?;
        let args = match call.tool()? {
            "implement_feature" => json!({"feature":target}),
            "code_review" => json!({"target":target}),
            "merge_code" => json!({"branch":target}),
            _ => unreachable!(),
        };
        let at = scored
            .iter()
            .position(|e| {
                e["agent"] == call.request["agent"]
                    && e["tool"] == call.request["tool"]
                    && e["args"] == args
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("score differs from original calls")?;
        scored.remove(at);
    }

    let is_loan = |f: &&Frame| f.bytes.len() == 3 && f.bytes[0] == 13;
    let mut loans: Vec<_> = frames
        .values()
        .filter(|f| f.direction(7, 0) || f.direction(7, 1))
        .filter(is_loan)
        .collect();
    loans.sort_by_key(|f| f.sent);
    if loans.len() != 4 + rejections {
        return Err("repository loan count".into());
    }
    let mut previous_return = 0;
    let mut used = BTreeSet::new();
    let mut used_closes = BTreeSet::new();
    check_close_pairs(&frames)?;
    for (number, loan) in loans.iter().enumerate() {
        let vm = loan.identity["target"].as_u64().ok_or("loan target")?;
        let next = loans.get(number + 1).map_or(rows.len(), |f| f.sent);
        let returned = one(frames.values().filter(|f| {
            f.direction(vm, 7)
                && is_loan(f)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < next
        }))?;
        let request = one(frames.values().filter(|f| {
            f.direction(13, 7)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < returned.sent
        }))?;
        let reply = one(frames.values().filter(|f| {
            f.direction(7, 13)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < returned.sent
        }))?;
        let admitted = one(frames.values().filter(|f| {
            f.direction(vm, 7)
                && f.bytes.first() == Some(&b'{')
                && f.received
                    .is_some_and(|at| at > previous_return && at < loan.sent)
        }))?;
        let arguments = request.value()?;
        let selected: Vec<_> = calls
            .iter()
            .filter(|(_, c)| {
                c.request["agent"]
                    == if vm == 0 {
                        "DEVELOPER_A"
                    } else {
                        "DEVELOPER_B"
                    }
                    && c.request["arguments"] == arguments
                    && c.emitted > request.received().unwrap_or(usize::MAX)
                    && c.emitted < returned.sent
            })
            .collect();
        if selected.len() != 1 {
            return Err("resource request does not select one actual effect".into());
        }
        let (id, call) = selected[0];
        let (result_at, result) = call.result()?;
        if !used.insert(*id)
            || admitted.value()? != arguments
            || reply.value()? != *result
            || result["success"] != true
            || !(previous_return < loan.sent
                && loan.received()? < request.sent
                && request.received()? < call.emitted
                && result_at < reply.sent
                && reply.received()? < returned.sent
                && returned.received()? < next)
        {
            return Err("loan/request/result/return order or contents".into());
        }
        let path = if call.tool()? == "implement_feature" {
            "service/implement_feature"
        } else {
            "service/merge_code"
        };
        close_arguments(
            &frames,
            &mut used_closes,
            vm,
            vm + 8,
            admitted.sent,
            path,
            &arguments,
        )?;
        if call.tool()? == "merge_code" {
            let feature = if vm == 0 {
                "user_authentication"
            } else {
                "notification_feature"
            };
            let reviewed = calls
                .values()
                .filter(|c| {
                    c.tool().is_ok_and(|s| s == "code_review")
                        && c.target().is_ok_and(|s| s == feature)
                        && c.ended.is_some_and(|(at, _)| at < loan.sent)
                })
                .max_by_key(|c| c.ended.map(|v| v.0))
                .ok_or("merge without review")?;
            let implementation = calls
                .values()
                .filter(|c| {
                    c.tool().is_ok_and(|s| s == "implement_feature")
                        && c.target().is_ok_and(|s| s == feature)
                        && c.started.is_some_and(|(at, _)| at < loan.sent)
                })
                .max_by_key(|c| c.started.map(|v| v.0))
                .ok_or("merge without implementation")?;
            if reviewed.result()?.1["success"] != true
                || implementation.ended()?.0 >= reviewed.started()?.0
            {
                return Err("merge without approval of the latest implementation".into());
            }
        }
        previous_return = returned.received()?;
    }
    let returns = frames
        .values()
        .filter(|f| f.direction(0, 7) || f.direction(1, 7))
        .filter(is_loan)
        .count();
    let resource_requests = frames.values().filter(|f| f.direction(13, 7)).count();
    let resource_replies = frames.values().filter(|f| f.direction(7, 13)).count();
    let admissions = frames
        .values()
        .filter(|f| (f.direction(0, 7) || f.direction(1, 7)) && f.bytes.first() == Some(&b'{'))
        .count();
    if returns != loans.len()
        || resource_requests != loans.len()
        || resource_replies != loans.len()
        || admissions != loans.len()
    {
        return Err("extra or missing resource operation".into());
    }

    let mut reviews = 0;
    let mut used_review_requests = BTreeSet::new();
    let mut used_review_results = BTreeSet::new();
    let mut review_calls: Vec<_> = calls
        .values()
        .filter(|c| c.tool().is_ok_and(|s| s == "code_review"))
        .collect();
    review_calls.sort_by_key(|c| c.emitted);
    let mut previous_review = 0;
    for call in &review_calls {
        let service = if call.target()? == "user_authentication" {
            5
        } else {
            6
        };
        let previous = review_calls
            .iter()
            .filter(|c| c.target().ok() == call.target().ok() && c.emitted < call.emitted)
            .map(|c| c.result().map(|(at, _)| at))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(0);
        let implementation = calls
            .values()
            .filter(|c| {
                c.tool().is_ok_and(|s| s == "implement_feature")
                    && c.target().ok() == call.target().ok()
                    && c.started
                        .is_some_and(|(at, _)| previous < at && at < call.emitted)
            })
            .max_by_key(|c| c.started.map(|v| v.0))
            .ok_or("review without implementation")?;
        if implementation.ended()?.0 >= call.emitted {
            return Err("review before the latest implementation completed".into());
        }
        let mut submitted: Vec<_> = frames
            .values()
            .filter(|f| {
                f.identity["source"] == 2
                    && matches!(f.identity["target"].as_u64(), Some(0 | 1 | 5 | 6 | 7))
                    && previous_review < f.sent
                    && f.sent < call.emitted
                    && f.value().is_ok_and(|v| v == call.request["arguments"])
            })
            .collect();
        submitted.sort_by_key(|f| f.sent);
        if submitted.len() != 6 {
            return Err("review requires five intent deliveries and one distinct request".into());
        }
        let mut after = previous_review;
        for (frame, target) in submitted.iter().zip([7, 5, 6, 0, 1, service]) {
            if !frame.direction(2, target)
                || frame.sent <= after
                || !used_review_requests.insert(frame.sent)
            {
                return Err("review intent/request order or reused occurrence".into());
            }
            after = frame.sent;
        }
        let request = submitted[5];
        let intent_label = submitted[0].logical_label;
        if !matches!(intent_label, 80..=89 | 92..=95)
            || submitted[..5]
                .iter()
                .any(|frame| frame.logical_label != intent_label)
            || !(if service == 5 { [71, 77] } else { [171, 177] }).contains(&request.logical_label)
        {
            return Err("compiled review message differs from intent or request".into());
        }
        if request.received()? >= call.emitted {
            return Err("review effect before request receipt".into());
        }
        close_arguments(
            &frames,
            &mut used_closes,
            2,
            10,
            request.sent,
            &format!("service/code_review/{}", call.target()?),
            &call.request["arguments"],
        )?;
        let next = calls
            .values()
            .filter(|c| {
                c.tool().is_ok_and(|s| s == "code_review")
                    && c.target().ok() == call.target().ok()
                    && c.emitted > call.emitted
            })
            .map(|c| c.emitted)
            .min()
            .unwrap_or(rows.len());
        let (result_at, result) = call.result()?;
        previous_review = result_at;
        for target in [0, 1, 2, if service == 5 { 6 } else { 5 }, 7] {
            let reply = one(frames.values().filter(|f| {
                f.direction(service, target)
                    && f.sent > result_at
                    && f.sent < next
                    && f.value().is_ok_and(|v| v == *result)
            }))?;
            let label = match (service, result["success"].as_bool()) {
                (5, Some(true)) => 30,
                (5, Some(false)) => 31,
                (6, Some(true)) => 40,
                (6, Some(false)) => 41,
                _ => return Err("review result has no boolean success".into()),
            };
            if reply.logical_label != label {
                return Err("compiled review reply differs from actual result".into());
            }
            if !used_review_results.insert(reply.sent) {
                return Err("review result reused for another effect".into());
            }
        }
        reviews += 1;
    }
    if used.len() + reviews != calls.len()
        || frames
            .values()
            .filter(|f| {
                f.identity["source"] == 2
                    && matches!(f.identity["target"].as_u64(), Some(0 | 1 | 5 | 6 | 7))
                    && f.bytes.first() == Some(&b'{')
            })
            .count()
            != used_review_requests.len()
        || frames
            .values()
            .filter(|f| f.identity["source"] == 5 || f.identity["source"] == 6)
            .count()
            != reviews * 5
    {
        return Err("missing or extra review result".into());
    }
    let effects: Vec<_> = calls.values().collect();
    let mut overlap = BTreeMap::<String, usize>::new();
    for (i, first) in effects.iter().enumerate() {
        for second in effects.iter().skip(i + 1) {
            if first.started()?.1.max(second.started()?.1) < first.ended()?.1.min(second.ended()?.1)
            {
                let mut names = [first.tool()?, second.tool()?];
                names.sort();
                *overlap.entry(names.join("+")).or_default() += 1;
            }
        }
    }
    let mut selections: Vec<_> = calls
        .values()
        .filter(|call| call.tool().is_ok_and(|tool| tool == "code_review"))
        .collect();
    selections.sort_by_key(|call| call.emitted);
    let targets = selections
        .iter()
        .map(|call| call.target())
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"status":"passed","task":"6E","kind":"completed_actual_wasi_review_trace","scored_benchmark":false,
        "depth":fixture["depth"],"carrier_frames":frames.len(),"domain_calls":calls.len(),"actual_resource_endpoint_requests":loans.len(),
        "actual_resource_endpoint_returns":returns,"actual_review_requests":reviews,"guest_exits":exits.len(),"simulator_complete":true,
        "violations":0,"overlapping_effect_pairs":overlap,"review_targets":targets}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let projection = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &projection)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&projection, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    result["wire_validation"] =
        json!("compiled wire membership, review contracts and EPF source-execution interpretation");
    Ok(result)
}

fn review_occurrence_frames(rows: &[Value], repeated: bool) -> Result<(u64, u64, u64)> {
    let mut seen = BTreeSet::new();
    let (at, call) = rows
        .iter()
        .enumerate()
        .find(|(_, row)| {
            let m = &row["message"];
            row["direction"] == "runtime"
                && m["type"] == "execute"
                && m["tool"] == "code_review"
                && (!repeated || !seen.insert(m["arguments"].to_string()))
        })
        .ok_or("matching actual review occurrence")?;
    let call = &call["message"];
    let service = if call["arguments"]["pr_id"] == "user_authentication" {
        5
    } else {
        6
    };
    let (request_at, request) = rows[..at]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, row)| {
            let m = &row["message"];
            m["type"] == "protocol_frame"
                && m["action"] == "send"
                && m["source"] == 2
                && m["target"] == service
                && serde_json::from_value::<Vec<u8>>(m["payload"].clone())
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .is_some_and(|args| args == call["arguments"])
        })
        .ok_or("actual review request frame")?;
    let closed = rows[..request_at]
        .iter()
        .rev()
        .find(|row| {
            let m = &row["message"];
            m["type"] == "protocol_frame"
                && m["action"] == "receive"
                && m["source"] == 10
                && m["target"] == 2
                && m["payload"][0] == 4
                && m["payload"][1] == 0
                && m["payload"][2] == 0
                && m["payload"][3].as_u64().is_some_and(|n| n > 0)
        })
        .ok_or("actual review close frame")?;
    let result = rows[at..]
        .iter()
        .find(|row| {
            let m = &row["message"];
            m["type"] == "protocol_frame"
                && m["action"] == "send"
                && m["source"] == service
                && m["target"] == 2
        })
        .ok_or("actual review result frame")?;
    Ok((
        request["message"]["frame_id"]
            .as_u64()
            .ok_or("request ID")?,
        closed["message"]["frame_id"].as_u64().ok_or("close ID")?,
        result["message"]["frame_id"].as_u64().ok_or("result ID")?,
    ))
}

#[test]
#[ignore = "requires CHOREOFIX_REVIEW_TRACE from a completed actual-WASI repeated review"]
fn a_repeated_review_cannot_reuse_an_earlier_request() -> Result<()> {
    let path = std::env::var_os("CHOREOFIX_REVIEW_TRACE").ok_or("review trace path")?;
    let mut rows = records(Path::new(&path))?;
    let projection = crate::projection::load(Path::new(&path))?;
    check(&rows, &projection)?;
    let (id, _, _) = review_occurrence_frames(&rows, true)?;
    rows.retain(|row| {
        !(row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id)
    });
    if let Some(output) = std::env::var_os("CHOREOFIX_REVIEW_MUTANT_OUTPUT") {
        let lines = rows
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        fs::write(output, lines.join("\n") + "\n")?;
    }
    assert!(
        check(&rows, &projection).is_err(),
        "the earlier identical request was reused"
    );
    Ok(())
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let audited = audit(path)?;
    fs::create_dir(output)?;
    let rows = records(path)?;
    let projection = crate::projection::load(path)?;
    let mut verdicts = Vec::new();
    let mut kinds = vec![
        "missing_loan",
        "wrong_resource",
        "wrong_frame_label",
        "wrong_lane",
        "wrong_review_message",
        "missing_admission",
        "wrong_admission",
        "missing_request",
        "forged_request",
        "missing_reply",
        "forged_reply",
        "missing_return",
        "overlapping_loans",
        "effect_before_request",
        "missing_original_result",
        "forged_review_result",
        "missing_wasi_close",
        "missing_exit",
        "false_score",
        "missing_review_request",
        "extra_review_request",
        "missing_review_close",
        "extra_review_close",
        "missing_review_result",
        "extra_review_result",
    ];
    if review_occurrence_frames(&rows, true).is_ok() {
        kinds.extend([
            "missing_repeated_review_request",
            "missing_repeated_review_close",
            "missing_repeated_review_result",
        ]);
    }
    for kind in kinds {
        let mut changed = rows.clone();
        let frame_target = |r: &Value| -> bool {
            let m = &r["message"];
            if m["type"] != "protocol_frame" {
                return false;
            }
            let source = m["source"].as_u64();
            let target = m["target"].as_u64();
            match kind {
                "missing_loan" | "wrong_resource" | "overlapping_loans" => {
                    source == Some(7) && matches!(target, Some(0 | 1))
                }
                "missing_admission" | "wrong_admission" => {
                    matches!(source, Some(0 | 1)) && target == Some(7) && m["payload"][0] == 123
                }
                "missing_request" | "forged_request" | "effect_before_request" => {
                    source == Some(13) && target == Some(7)
                }
                "missing_reply" | "forged_reply" => source == Some(7) && target == Some(13),
                "missing_return" => {
                    matches!(source, Some(0 | 1)) && target == Some(7) && m["payload"][0] == 13
                }
                _ => false,
            }
        };
        match kind {
            "wrong_review_message" => {
                let (request, _, _) = review_occurrence_frames(&changed, false)?;
                let message = &changed
                    .iter()
                    .find(|row| {
                        row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == request
                    })
                    .ok_or("review request to mutate")?["message"];
                let source = message["source"].as_u64().ok_or("review source")?;
                let target = message["target"].as_u64().ok_or("review target")?;
                let replacement = projection["compiled"]["programs"]
                    .as_array()
                    .ok_or("projection programs")?
                    .iter()
                    .flat_map(|program| program["rows"].as_array().into_iter().flatten())
                    .filter(|row| row[1] == 0x0301)
                    .find(|row| {
                        let peers = row[4].as_u64().unwrap_or(0);
                        let packed = row[3].as_u64().unwrap_or(0);
                        peers >> 24 == source
                            && (peers >> 16) & 0xff == target
                            && matches!(packed & 0xff, 80..=89 | 92..=95)
                    })
                    .ok_or("compiled intent sharing the review request's peers")?;
                let packed = replacement[3].as_u64().ok_or("compiled intent fields")?;
                for row in changed.iter_mut().filter(|row| {
                    row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == request
                }) {
                    row["message"]["frame_label"] = json!(packed >> 16);
                    row["message"]["lane"] = json!((packed >> 8) & 0xff);
                }
                // The replacement is a real static wire contract with matching
                // peers and payload. Business-message checks must still reject it.
                crate::projection::frames(&projection, &changed)?;
            }
            "wrong_frame_label" | "wrong_lane" => {
                let first = changed
                    .iter()
                    .find(|row| row["message"]["type"] == "protocol_frame")
                    .ok_or("frame for wire mutation")?;
                let id = first["message"]["frame_id"].clone();
                let field = if kind == "wrong_lane" {
                    "lane"
                } else {
                    "frame_label"
                };
                let value = if kind == "wrong_lane" {
                    (first["message"][field].as_u64().ok_or("frame lane")? + 1) % 256
                } else {
                    u64::from(u16::MAX)
                };
                for row in changed.iter_mut().filter(|row| {
                    row["message"]["type"] == "protocol_frame" && row["message"]["frame_id"] == id
                }) {
                    row["message"][field] = json!(value);
                }
            }
            "missing_review_request"
            | "extra_review_request"
            | "missing_review_close"
            | "extra_review_close"
            | "missing_review_result"
            | "extra_review_result"
            | "missing_repeated_review_request"
            | "missing_repeated_review_close"
            | "missing_repeated_review_result" => {
                let (request, closed, result) =
                    review_occurrence_frames(&changed, kind.contains("repeated"))?;
                let id = if kind.ends_with("request") {
                    request
                } else if kind.ends_with("close") {
                    closed
                } else {
                    result
                };
                if kind.starts_with("missing") {
                    changed.retain(|row| {
                        !(row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id)
                    });
                } else {
                    let fresh = changed
                        .iter()
                        .filter_map(|row| row["message"]["frame_id"].as_u64())
                        .max()
                        .ok_or("frame IDs")?
                        .checked_add(1)
                        .ok_or("frame ID overflow")?;
                    changed = changed
                        .into_iter()
                        .flat_map(|row| {
                            let mut copies = vec![row];
                            if copies[0]["message"]["type"] == "protocol_frame"
                                && copies[0]["message"]["frame_id"] == id
                            {
                                let mut extra = copies[0].clone();
                                extra["message"]["frame_id"] = json!(fresh);
                                copies.push(extra);
                            }
                            copies
                        })
                        .collect();
                }
            }
            "missing_wasi_close" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 0
                            && m["target"] == 7
                            && m["payload"][0] == 123
                    })
                    .ok_or("actual admission for close mutation")?;
                let id = changed[..at]
                    .iter()
                    .rev()
                    .find(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 8
                            && m["target"] == 0
                            && m["payload"][0] == 4
                            && m["payload"].as_array().is_some_and(|v| v.len() > 4)
                    })
                    .ok_or("actual WASI close mutation")?["message"]["frame_id"]
                    .clone();
                changed.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id)
                });
            }
            "missing_original_result" => {
                let at = changed
                    .iter()
                    .position(|r| r["message"]["type"] == "simulator_result")
                    .ok_or("result mutant")?;
                changed.remove(at);
            }
            "missing_exit" => {
                let at = changed
                    .iter()
                    .position(|r| r["direction"] == "runtime" && r["message"]["type"] == "exit")
                    .ok_or("exit mutant")?;
                changed.remove(at);
            }
            "false_score" => {
                let row = changed
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "score")
                    .ok_or("score mutant")?;
                row["message"]["complete"] = json!(false);
            }
            "forged_review_result" => {
                let row = changed
                    .iter_mut()
                    .find(|r| {
                        r["message"]["type"] == "simulator_result"
                            && r["message"]["result"]["tool_name"] == "code_review"
                    })
                    .ok_or("review mutant")?;
                row["message"]["result"]["data"] = json!({"forged":true});
            }
            "overlapping_loans" => {
                let ids: Vec<_> = changed
                    .iter()
                    .filter(|r| frame_target(r) && r["message"]["action"] == "send")
                    .map(|r| r["message"]["frame_id"].clone())
                    .collect();
                let id = ids.get(1).ok_or("second loan")?.clone();
                let moved: Vec<_> = changed
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id
                    })
                    .cloned()
                    .collect();
                changed.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id)
                });
                let at = changed
                    .iter()
                    .position(|r| frame_target(r) && r["message"]["action"] == "receive")
                    .ok_or("loan receipt")?;
                changed.splice(at + 1..at + 1, moved);
            }
            "effect_before_request" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        r["direction"] == "runtime"
                            && r["message"]["type"] == "execute"
                            && r["message"]["tool"] == "implement_feature"
                    })
                    .ok_or("effect mutant")?;
                let effect = changed.remove(at);
                let before = changed
                    .iter()
                    .position(|r| frame_target(r) && r["message"]["action"] == "receive")
                    .ok_or("request receipt")?;
                changed.insert(before, effect);
            }
            _ => {
                let id = changed
                    .iter()
                    .find(|r| frame_target(r))
                    .ok_or("mutation frame")?["message"]["frame_id"]
                    .clone();
                if kind.starts_with("missing_") {
                    changed.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == id)
                    });
                } else {
                    for row in changed.iter_mut().filter(|r| {
                        r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id
                    }) {
                        row["message"]["payload"] = match kind {
                            "wrong_resource" => json!([12, 0, 0]),
                            "wrong_admission" | "forged_request" => {
                                json!(serde_json::to_vec(&json!({"feature_name":"forged"}))?)
                            }
                            "forged_reply" => json!(serde_json::to_vec(
                                &json!({"success":true,"data":{"forged":true}})
                            )?),
                            _ => return Err("unknown mutation".into()),
                        };
                    }
                }
            }
        }
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&changed)?));
        let rejection = check(&changed, &projection)
            .err()
            .ok_or_else(|| format!("undetected review mutation {kind}"))?;
        verdicts.push(json!({"kind":kind,"status":"rejected","parsed_records_sha256":hash,"reason":rejection.to_string()}));
    }
    let result = json!({"status":"passed","task":"6E","source_events_sha256":digest(path)?,"audit":audited,"count":verdicts.len(),"mutants":verdicts,
        "representation":"Mutants are regenerated from the original JSONL by this Rust tool; redundant raw copies are not stored"});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
