//! Immutable transport and effect observations, shared by task auditors.
pub(crate) mod file;

use crate::evidence::Result;
use hibana::runtime::wire::{Payload, WirePayload};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(crate) struct Frame<'a> {
    pub(crate) logical_label: u8,
    pub(crate) identity: &'a Value,
    pub(crate) bytes: Vec<u8>,
    pub(crate) sent: usize,
    pub(crate) first_poll: usize,
    pub(crate) received: usize,
    pub(crate) receipt: &'a Value,
}
impl Frame<'_> {
    pub(crate) fn direction(&self, from: u64, to: u64) -> bool {
        self.identity["source"] == from && self.identity["target"] == to
    }
    pub(crate) fn validate<P: WirePayload>(&self) -> Result<()> {
        P::validate_payload(Payload::new(&self.bytes))
            .map_err(|error| format!("WASI payload: {error:?}").into())
    }
    pub(crate) fn value(&self) -> Result<Value> {
        Ok(serde_json::from_slice(&self.bytes)?)
    }
}
pub(crate) struct Call {
    pub(crate) request: Value,
    pub(crate) emitted: usize,
    pub(crate) started: Option<(usize, f64)>,
    pub(crate) result: Option<(usize, Value)>,
    pub(crate) domain_result: Option<Value>,
    pub(crate) ended: Option<(usize, f64)>,
}
impl Call {
    pub(crate) fn started(&self) -> Result<(usize, f64)> {
        self.started.ok_or_else(|| "missing domain start".into())
    }
    pub(crate) fn ended(&self) -> Result<(usize, f64)> {
        self.ended.ok_or_else(|| "missing domain end".into())
    }
    pub(crate) fn result(&self) -> Result<(usize, &Value)> {
        self.result
            .as_ref()
            .map(|(at, value)| (*at, value))
            .ok_or_else(|| "missing original result".into())
    }
    /// Match the scoring event to the recorded call and reply. Some simulator
    /// tools publish normalized arguments; the caller supplies those exact values.
    pub(crate) fn matches_score(&self, event: &Value, scored_arguments: &Value) -> bool {
        self.result.as_ref().is_some_and(|(_, result)| {
            event["agent"] == self.request["agent"]
                && event["tool"] == self.request["tool"]
                && event.get("args") == Some(scored_arguments)
                && result["success"]
                    .as_bool()
                    .is_some_and(|success| event["success"] == success)
                && event
                    .get("result")
                    .is_some_and(|value| value == &result["data"])
                && event["violations"].as_array().is_some_and(Vec::is_empty)
        })
    }
    /// Consume one scored occurrence. Equal calls can each use a separate equal event.
    pub(crate) fn take_score(&self, remaining: &mut Vec<&Value>, arguments: &Value) -> Result<()> {
        let index = remaining
            .iter()
            .position(|event| self.matches_score(event, arguments))
            .ok_or_else(|| format!("missing matching score for effect {}", self.request["id"]))?;
        remaining.swap_remove(index);
        Ok(())
    }
    pub(crate) fn tool(&self) -> Result<&str> {
        self.request["tool"]
            .as_str()
            .ok_or_else(|| "tool name".into())
    }
}
/// Take the unique unused JSON frame sent on this edge inside the interval.
/// The task checker compares its receive commit with the actual effect time.
pub(crate) fn take_json_frame<'a>(
    (frames, used): (&'a BTreeMap<u64, Frame<'a>>, &mut BTreeSet<usize>),
    (from, to, labels): (u64, u64, &[u64]),
    (value, after, before): (&Value, usize, usize),
) -> Result<&'a Frame<'a>> {
    let frame = one(frames.values().filter(|frame| {
        frame.direction(from, to)
            && labels.contains(&u64::from(frame.logical_label))
            && after < frame.sent
            && frame.sent < before
            && !used.contains(&frame.sent)
            && frame.value().is_ok_and(|actual| actual == *value)
    }))?;
    used.insert(frame.sent);
    Ok(frame)
}

pub(crate) fn one<T>(items: impl Iterator<Item = T>) -> Result<T> {
    let mut items = items;
    let frame = items.next().ok_or("missing matching frame")?;
    if items.next().is_some() {
        return Err("multiple matching frames".into());
    }
    Ok(frame)
}
pub(crate) fn records(path: &Path) -> Result<Vec<Value>> {
    parse_records(&fs::read_to_string(path)?)
}

fn parse_records(text: &str) -> Result<Vec<Value>> {
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

/// The exact files used for a saved run's evidence and their content hashes.
pub(crate) struct Evidence {
    pub(crate) rows: Vec<Value>,
    pub(crate) projection: Value,
    pub(crate) events_hash: String,
    projection_hash: String,
}

impl Evidence {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let (projection, projection_hash) = crate::projection::load(path)?;
        let text = fs::read_to_string(path)?;
        let events_hash = format!("{:x}", Sha256::digest(text.as_bytes()));
        Ok(Self {
            rows: parse_records(&text)?,
            projection,
            events_hash,
            projection_hash,
        })
    }

    pub(crate) fn audit(
        &self,
        effects: fn(&[Value], &Value, &Value) -> Result<Value>,
    ) -> Result<Value> {
        let execution = crate::projection::check_dynamic(&self.projection, &self.rows)?;
        let mut report = effects(&self.rows, &self.projection, &execution)?;
        report["source_events_sha256"] = json!(self.events_hash);
        report["compiled_projection_sha256"] = json!(self.projection_hash);
        report["runtime_binary_sha256"] = self.projection["runtime_binary_sha256"].clone();
        Ok(report)
    }
}

/// Run a saved-record counterexample in the same production auditor, releasing
/// its search workspace on exit. This neither rebuilds nor restores any source.
pub(crate) fn reject_mutant(original: &Path, changed: &Path, task: &str) -> Result<String> {
    let source = original.parent().ok_or("original evidence directory")?;
    let output = changed.parent().ok_or("mutant evidence directory")?;
    for name in ["projection.json", "case.json", "versions.json"] {
        let from = source.join(name);
        if !from.exists() {
            continue;
        }
        let to = output.join(name);
        if !to.exists() {
            fs::copy(&from, &to)?;
        }
        if crate::evidence::digest(&from)? != crate::evidence::digest(&to)? {
            return Err("mutant context differs from original evidence".into());
        }
    }
    #[cfg(test)]
    let auditor = std::env::var_os("CHOREOFIX_AUDITOR").ok_or("qualified auditor missing")?;
    #[cfg(not(test))]
    let auditor = std::env::current_exe()?;
    let result = std::process::Command::new(auditor)
        .args(["--audit", task])
        .arg(changed)
        .output()?;
    let error = String::from_utf8(result.stderr)?;
    if result.status.code() != Some(1) || !error.starts_with("Error:") {
        return Err(format!(
            "record counterexample was accepted or its auditor failed: status={}, stderr={error}",
            result.status
        )
        .into());
    }
    Ok(error)
}

/// Mutate immutable observations; this never drives or changes an Endpoint.
pub(crate) fn json_frame_mutants(
    rows: &[Value],
    ids: &[u64],
    verify: impl Fn(&[Value]) -> Result<Value>,
) -> Result<Vec<Value>> {
    if ids.is_empty() || ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("empty or repeated frame-mutation assignment".into());
    }
    let mut verdicts = Vec::new();
    for &id in ids {
        for kind in [
            "missing_frame",
            "coherent_payload_change",
            "duplicate_frame",
        ] {
            let mut changed = rows.to_vec();
            match kind {
                "missing_frame" => changed.retain(|row| {
                    !(row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == id)
                }),
                "coherent_payload_change" => {
                    for row in &mut changed {
                        let message = &mut row["message"];
                        if message["type"] == "protocol_frame" && message["frame_id"] == id {
                            let bytes = message["payload"]
                                .as_array()
                                .ok_or("JSON frame bytes")?
                                .iter()
                                .map(|v| {
                                    v.as_u64()
                                        .and_then(|n| u8::try_from(n).ok())
                                        .ok_or("frame byte")
                                })
                                .collect::<std::result::Result<Vec<_>, _>>()?;
                            let mut value: Value = serde_json::from_slice(&bytes)?;
                            if value
                                .as_object_mut()
                                .ok_or("business JSON object")?
                                .insert("unadmitted_value".into(), json!(true))
                                .is_some()
                            {
                                return Err("baseline already contains the mutation marker".into());
                            }
                            message["payload"] = json!(serde_json::to_vec(&value)?);
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
            let error = verify(&changed)
                .err()
                .ok_or_else(|| format!("accepted {kind} for frame {id}"))?;
            verdicts
                .push(json!({"frame_id":id,"kind":kind,"rejected":true,"error":error.to_string()}));
        }
    }
    Ok(verdicts)
}
fn time(row: &Value) -> Result<f64> {
    row["elapsed_seconds"]
        .as_f64()
        .filter(|t| t.is_finite() && *t >= 0.0)
        .ok_or_else(|| "invalid observation time".into())
}
pub(crate) fn check_close_pairs(frames: &BTreeMap<u64, Frame>, pairs: &[(u64, u64)]) -> Result<()> {
    for &(vm, io) in pairs {
        let mut requests = frames
            .values()
            .filter(|frame| {
                frame.direction(vm, io) && frame.logical_label == protocol::LABEL_WASI_FD_CLOSE
            })
            .collect::<Vec<_>>();
        let mut replies = frames
            .values()
            .filter(|frame| {
                frame.direction(io, vm) && frame.logical_label == protocol::LABEL_WASI_FD_CLOSE_RET
            })
            .collect::<Vec<_>>();
        requests.sort_by_key(|frame| frame.sent);
        replies.sort_by_key(|frame| frame.sent);
        if requests.len() != replies.len() {
            return Err("missing or extra WASI close request/result".into());
        }
        for (request, reply) in requests.into_iter().zip(replies) {
            protocol::FdCloseReq::validate_payload(Payload::new(&request.bytes))
                .map_err(|_| "invalid WASI close request")?;
            protocol::FdClosedRet::validate_payload(Payload::new(&reply.bytes))
                .map_err(|_| "invalid WASI close result")?;
            if request.bytes[0] != reply.bytes[0] {
                return Err("WASI close request/result data mismatch".into());
            }
        }
    }
    Ok(())
}

/// Correlated observations owned by the offline auditor, never runtime authority.
pub(crate) struct Journal<'a> {
    pub(crate) frames: BTreeMap<u64, Frame<'a>>,
    pub(crate) calls: BTreeMap<u64, Call>,
    pub(crate) score: Value,
}

/// A completed trial must associate each physical frame with one EPF-checked
/// receive, including when the transport owner flushes its journal after the commit.
pub(crate) fn read<'a>(
    rows: &'a [Value],
    labels: &BTreeMap<u64, u8>,
    actors: &[Value],
    dynamic: &'a Value,
) -> Result<Journal<'a>> {
    let mut acceptances = BTreeMap::new();
    for session in dynamic["sessions"].as_array().ok_or("EPF sessions")? {
        let sid = session["session"].as_u64().ok_or("EPF session ID")?;
        for receipt in session["acceptances"].as_array().ok_or("EPF acceptances")? {
            let id = receipt["frame_id"].as_u64().ok_or("accepted frame")?;
            if acceptances.insert(id, (sid, receipt)).is_some() {
                return Err("reused frame in checked execution".into());
            }
        }
    }
    let mut frames = BTreeMap::new();
    let mut calls = BTreeMap::<u64, Call>::new();
    let mut exits = BTreeSet::new();
    let mut verdict = None;
    let mut score = None;
    for (index, row) in rows.iter().enumerate() {
        let m = &row["message"];
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                let id = m["frame_id"].as_u64().ok_or("frame ID")?;
                if m["action"] != "send" {
                    return Err("unknown frame action".into());
                }
                let (sid, receipt) = acceptances
                    .remove(&id)
                    .ok_or("missing or reused EPF acceptance")?;
                let commit = &receipt["receive_commit"];
                if m["session"] != sid
                    || commit["source_event"] != m["frame_label"]
                    || commit["view"] != m["target"]
                    || receipt["send_view"] != m["source"]
                {
                    return Err("frame differs from its checked receive".into());
                }
                let first_poll = usize::try_from(
                    receipt["first_poll_row"]
                        .as_u64()
                        .ok_or("missing send poll position")?,
                )?;
                let received = usize::try_from(
                    commit["journal_row"]
                        .as_u64()
                        .ok_or("missing receive commit position")?,
                )?;
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
                frames.insert(
                    id,
                    Frame {
                        logical_label: *labels.get(&id).ok_or("unmapped frame")?,
                        identity: m,
                        bytes,
                        sent: index,
                        first_poll,
                        received,
                        receipt,
                    },
                );
            }
            (Some("runtime"), Some("execute")) => {
                let id = m["id"].as_u64().ok_or("effect ID")?;
                if !actors.iter().any(|actor| actor["agent"] == m["agent"]) {
                    return Err("unknown effect actor".into());
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
            (Some("simulator"), Some("score")) if score.replace((index, m)).is_some() => {
                return Err("duplicate original score".into());
            }
            _ => {}
        }
    }
    if !acceptances.is_empty() {
        return Err("checked acceptance without a physical frame".into());
    }
    if exits.len() != actors.len() || verdict.is_none() {
        return Err("incomplete observed execution".into());
    }
    let verdict = verdict.ok_or("missing runtime verdict")?;
    for call in calls.values() {
        let (start, started) = call.started()?;
        let (end, ended) = call.ended()?;
        let (result_at, result) = call.result()?;
        if !(call.emitted < start
            && start < end
            && end < result_at
            && result_at < verdict
            && started <= ended)
            || result["tool_name"] != call.tool()?
        {
            return Err("invalid effect/result interval or tool".into());
        }
    }
    let (score_at, score) = score.ok_or("missing original score")?;
    if score_at <= verdict {
        return Err("original score precedes runtime completion".into());
    }
    let score = score.clone();
    Ok(Journal {
        frames,
        calls,
        score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn complete_effect_evidence_requires_the_epf_receive_commit_not_a_transport_preview()
    -> Result<()> {
        use hibana::{
            g,
            runtime::{program::project, tap},
        };
        let global = g::send::<7, 8, g::Msg<1, ()>>();
        let programs = [
            tap::projected(&project::<7>(&global)).collect::<Vec<_>>(),
            tap::projected(&project::<8>(&global)).collect::<Vec<_>>(),
        ];
        let compiled: Vec<_> = programs
            .iter()
            .zip([7, 8])
            .map(|(program, role)| {
                let rows: Vec<_> = program
                    .iter()
                    .map(|r| {
                        [
                            r.ts(),
                            u32::from(r.id()),
                            u32::from(r.causal_key()),
                            r.arg0(),
                            r.arg1(),
                        ]
                    })
                    .collect();
                json!({"role":role,"rows":rows})
            })
            .collect();
        let lane = compiled[0]["rows"][2][3].as_u64().ok_or("source lane")? >> 8 & 255;
        let artifact = json!({"compiled":{"type":"compiled_projection","format_version":1,
            "task":"1E","programs":compiled}});
        let frame = json!({"type":"protocol_frame","action":"send","session":1,
            "frame_id":19,"frame_label":0,"lane":lane,"source":7,"target":8,"payload":[]});
        let mut rows = vec![
            json!({"direction":"runtime","message":artifact["compiled"]}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[0,tap::SEND_POLL,7u16<<8|7,1,0xffff0000u32]]}}),
            json!({"direction":"runtime","message":frame}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[1,tap::ENDPOINT_SEND,7u16<<8|1,1,7u64<<24|lane<<16]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[2,tap::ENDPOINT_RECV,8u16<<8|2,1,8u64<<24|lane<<16]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":42,"session":1,"checkpoint":3}}),
            json!({"direction":"runtime","message":{"type":"exit","agent":"A","status":0}}),
            json!({"direction":"runtime","message":{"type":"exit","agent":"B","status":0}}),
            json!({"direction":"runtime","message":{"type":"run_result","status":"complete"}}),
            json!({"direction":"simulator","message":{"type":"score","complete":true,"events":[],"violations":[]}}),
        ];
        let labels = crate::projection::frames(&artifact, &rows)?;
        let actors = [json!({"agent":"A"}), json!({"agent":"B"})];
        let checked = crate::projection::check_dynamic(&artifact, &rows)?;
        let journal = read(&rows, &labels, &actors, &checked)?;
        assert_eq!(journal.frames[&19].received, 4);
        assert_eq!(journal.frames[&19].first_poll, 1);
        let mut duplicate = checked.clone();
        duplicate["sessions"][0]["acceptances"]
            .as_array_mut()
            .unwrap()
            .push(journal.frames[&19].receipt.clone());
        assert!(read(&rows, &labels, &actors, &duplicate).is_err());
        for (field, value) in [("source_event", 1), ("view", 7)] {
            let mut altered = checked.clone();
            altered["sessions"][0]["acceptances"][0]["receive_commit"][field] = json!(value);
            assert!(
                read(&rows, &labels, &actors, &altered).is_err(),
                "accepted {field}"
            );
        }
        for (field, value) in [("send_view", json!(8)), ("first_poll_row", Value::Null)] {
            let mut altered = checked.clone();
            altered["sessions"][0]["acceptances"][0][field] = value;
            assert!(
                read(&rows, &labels, &actors, &altered).is_err(),
                "accepted {field}"
            );
        }
        let mut wrong_session = checked.clone();
        wrong_session["sessions"][0]["session"] = json!(123456);
        assert!(read(&rows, &labels, &actors, &wrong_session).is_err());
        let mut missing = checked.clone();
        missing["sessions"][0]["acceptances"] = json!([]);
        assert!(read(&rows, &labels, &actors, &missing).is_err());
        let mut orphan = checked.clone();
        let mut extra = journal.frames[&19].receipt.clone();
        extra["frame_id"] = json!(20);
        orphan["sessions"][0]["acceptances"]
            .as_array_mut()
            .unwrap()
            .push(extra);
        assert!(read(&rows, &labels, &actors, &orphan).is_err());
        // The transport owner can flush its observations after the capture
        // owner has already recorded a successful receive commit.
        let mut late = rows.clone();
        let transport: Vec<_> = late
            .iter()
            .filter(|row| row["message"]["type"] == "protocol_frame")
            .cloned()
            .collect();
        late.retain(|row| row["message"]["type"] != "protocol_frame");
        late.splice(4..4, transport);
        let checked_late = crate::projection::check_dynamic(&artifact, &late)?;
        let journal_late = read(&late, &labels, &actors, &checked_late)?;
        assert!(journal_late.frames[&19].sent > 3);
        assert_eq!(journal_late.frames[&19].received, 3);
        // EPF correctly permits a finite source prefix with an accepted send.
        // Complete application effect evidence must additionally have its receive.
        rows.remove(4);
        rows[4]["message"]["checkpoint"] = json!(2);
        let prefix = crate::projection::check_dynamic(&artifact, &rows)?;
        assert_eq!(prefix["status"], "passed");
        assert!(read(&rows, &labels, &actors, &prefix).is_err());
        Ok(())
    }

    #[test]
    fn actual_effect_lifecycle_requires_its_request_and_matching_ordered_result() {
        let request = json!({"type":"execute","id":1,"agent":"A","tool":"write_section","arguments":{"section_name":"section_a"}});
        let result =
            json!({"tool_name":"write_section","success":true,"data":{"section":"section_a"}});
        let rows = vec![
            json!({"direction":"runtime","message":request}),
            json!({"direction":"domain_start","elapsed_seconds":0.1,"message":request}),
            json!({"direction":"domain_end","elapsed_seconds":0.2,"message":{"id":1,"agent":"A","tool":"write_section","result":result}}),
            json!({"direction":"simulator","message":{"type":"simulator_result","id":1,"result":result}}),
            json!({"direction":"runtime","message":{"type":"exit","agent":"A","status":0}}),
            json!({"direction":"runtime","message":{"type":"run_result","status":"complete"}}),
            json!({"direction":"simulator","message":{"type":"score","complete":true,"events":[],"violations":[]}}),
        ];
        let actors = [json!({"agent":"A"})];
        let dynamic = json!({"sessions":[]});
        let journal = read(&rows, &BTreeMap::new(), &actors, &dynamic).unwrap();
        let call = &journal.calls[&1];
        let args = json!({"section":"section_a"});
        let scored = json!({"agent":"A","tool":"write_section","args":args,
            "success":true,"result":result["data"],"violations":[]});
        assert!(call.matches_score(&scored, &args));
        let mut repeated = vec![&scored, &scored];
        assert!(
            call.take_score(&mut repeated, &request["arguments"])
                .is_err()
        );
        call.take_score(&mut repeated, &args).unwrap();
        call.take_score(&mut repeated, &args).unwrap();
        assert!(call.take_score(&mut repeated, &args).is_err());
        assert!(!call.matches_score(&scored, &request["arguments"]));
        for field in ["agent", "tool", "args", "success", "result", "violations"] {
            let mut changed = scored.clone();
            changed[field] = Value::Null;
            assert!(
                !call.matches_score(&changed, &args),
                "ignored altered {field}"
            );
            changed.as_object_mut().unwrap().remove(field);
            assert!(
                !call.matches_score(&changed, &args),
                "ignored missing {field}"
            );
        }
        for removed in [0, 1, 2, 3, 4, 5, 6] {
            let mut changed = rows.clone();
            changed.remove(removed);
            assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        }
        let mut changed = rows.clone();
        changed.swap(2, 3);
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        let mut changed = rows.clone();
        changed.swap(5, 6);
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        let mut changed = rows.clone();
        changed[1]["message"]["arguments"]["section_name"] = json!("section_b");
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        let mut changed = rows.clone();
        changed[3]["message"]["result"]["success"] = json!(false);
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        let mut changed = rows.clone();
        changed[2]["elapsed_seconds"] = json!(0.05);
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
        let mut changed = rows.clone();
        changed.insert(1, rows[0].clone());
        assert!(read(&changed, &BTreeMap::new(), &actors, &dynamic).is_err());
    }

    #[test]
    fn close_records_cannot_reuse_a_request_or_substitute_another_syscall() {
        let request = Frame {
            logical_label: protocol::LABEL_WASI_FD_CLOSE,
            identity: &json!({"source":0,"target":8}),
            bytes: vec![6],
            sent: 0,
            received: 1,
            first_poll: 0,
            receipt: &Value::Null,
        };
        let reply = Frame {
            logical_label: protocol::LABEL_WASI_FD_CLOSE_RET,
            identity: &json!({"source":8,"target":0}),
            bytes: vec![6, 0, 0],
            sent: 2,
            received: 3,
            first_poll: 0,
            receipt: &Value::Null,
        };
        let mut frames = BTreeMap::from([(1, request), (2, reply)]);
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_ok());
        frames.get_mut(&2).unwrap().bytes = vec![6, 0, 2];
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_ok());
        for bytes in [vec![], vec![6], vec![6, 0], vec![6, 0, 0, 0]] {
            frames.get_mut(&2).unwrap().bytes = bytes;
            assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
        }
        frames.get_mut(&2).unwrap().bytes = vec![6, 0, 0];
        frames.get_mut(&1).unwrap().logical_label = protocol::LABEL_WASI_FD_CLOSE_RET;
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
        frames.get_mut(&1).unwrap().logical_label = protocol::LABEL_WASI_FD_CLOSE;
        frames.get_mut(&2).unwrap().bytes[0] = 4;
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
        frames.get_mut(&2).unwrap().bytes[0] = 6;
        let third_identity = json!({"source":8,"target":0});
        frames.insert(
            3,
            Frame {
                logical_label: protocol::LABEL_WASI_FD_CLOSE_RET,
                identity: &third_identity,
                bytes: vec![6, 0, 0],
                sent: 4,
                received: 5,
                first_poll: 0,
                receipt: &Value::Null,
            },
        );
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
    }
}
