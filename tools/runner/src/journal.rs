//! Immutable transport and effect observations, shared by task auditors.
use crate::evidence::Result;
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(crate) struct Frame {
    pub(crate) logical_label: u8,
    pub(crate) identity: Value,
    pub(crate) bytes: Vec<u8>,
    pub(crate) sent: usize,
    pub(crate) received: Option<usize>,
    queued: bool,
}
impl Frame {
    pub(crate) fn direction(&self, from: u64, to: u64) -> bool {
        self.identity["source"] == from && self.identity["target"] == to
    }
    pub(crate) fn received(&self) -> Result<usize> {
        self.received.ok_or_else(|| "unreceived frame".into())
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
    pub(crate) fn tool(&self) -> Result<&str> {
        self.request["tool"]
            .as_str()
            .ok_or_else(|| "tool name".into())
    }
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
    fs::read_to_string(path)?
        .lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

/// Run a saved-record counterexample in the same production auditor, releasing
/// its search workspace on exit. This neither rebuilds nor restores any source.
pub(crate) fn reject_mutant(original: &Path, changed: &Path, command: &str) -> Result<String> {
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
        .arg(command)
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
pub(crate) fn close_arguments(
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
                && f.logical_label == protocol::LABEL_WASI_FD_CLOSE_RET
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
    let request = frames
        .values()
        .filter(|frame| {
            frame.direction(vm, io)
                && frame.logical_label == protocol::LABEL_WASI_FD_CLOSE
                && frame.sent < closed.sent
        })
        .max_by_key(|frame| frame.sent)
        .ok_or("close result without projected WASI request")?;
    if request.bytes != [4] || request.received()? >= closed.sent {
        return Err("close result without projected WASI request".into());
    }
    if !used.insert(closed.sent) {
        return Err("WASI close reused for another effect".into());
    }
    Ok(())
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
        let mut previous = None;
        for (request, reply) in requests.into_iter().zip(replies) {
            if request.bytes.len() != 1
                || reply.bytes.len() < 4
                || request.bytes[0] != reply.bytes[0]
                || request.received()? >= reply.sent
                || previous.is_some_and(|at| at >= request.sent)
            {
                return Err("WASI close request/result mismatch or overlapping import".into());
            }
            previous = Some(reply.received()?);
        }
    }
    Ok(())
}

/// Correlated observations owned by the offline auditor, never runtime authority.
pub(crate) struct Journal {
    pub(crate) frames: BTreeMap<u64, Frame>,
    pub(crate) calls: BTreeMap<u64, Call>,
    pub(crate) score: Value,
}

impl Journal {
    /// Use EPF's successful Endpoint receives for effect correspondence. The
    /// transport journal also contains previews and can be delivered later.
    pub(crate) fn bind_receives<'a>(
        &mut self,
        dynamic: &'a Value,
    ) -> Result<BTreeMap<u64, &'a Value>> {
        let receipts = self.receipts(dynamic)?;
        for (&id, receipt) in &receipts {
            let at = usize::try_from(
                receipt["receive_commit"]["journal_row"]
                    .as_u64()
                    .ok_or("committed receive journal row")?,
            )?;
            self.frames.get_mut(&id).ok_or("accepted frame")?.received = Some(at);
        }
        Ok(receipts)
    }

    /// Bind complete, EPF-checked receives to physical frames. These are
    /// immutable evidence references, not another source-execution replay.
    /// Transport observations can be queued after the commit capture; their
    /// journal arrival order is not the protocol's causal order.
    pub(crate) fn receipts<'a>(&self, dynamic: &'a Value) -> Result<BTreeMap<u64, &'a Value>> {
        let mut receipts = BTreeMap::new();
        for session in dynamic["sessions"].as_array().ok_or("EPF sessions")? {
            for acceptance in session["acceptances"].as_array().ok_or("EPF acceptances")? {
                let id = acceptance["frame_id"].as_u64().ok_or("accepted frame")?;
                let received = &acceptance["receive_commit"];
                let frame = self.frames.get(&id).ok_or("missing accepted frame")?;
                if session["session"] != frame.identity["session"]
                    || received["source_event"] != frame.identity["frame_label"]
                    || received["view"] != frame.identity["target"]
                    || acceptance["send_view"] != frame.identity["source"]
                    || receipts.insert(id, acceptance).is_some()
                {
                    return Err("missing or reused committed receive occurrence".into());
                }
            }
        }
        if receipts.len() != self.frames.len() {
            return Err("frame without a checked committed receive".into());
        }
        Ok(receipts)
    }
}

pub(crate) fn read(
    rows: &[Value],
    labels: &BTreeMap<u64, u8>,
    actors: &[Value],
) -> Result<Journal> {
    let mut frames = BTreeMap::<u64, Frame>::new();
    let mut calls = BTreeMap::<u64, Call>::new();
    let mut exits = BTreeSet::new();
    let mut verdict = None;
    let mut score = None;
    for (index, row) in rows.iter().enumerate() {
        let m = &row["message"];
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
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
    if frames.values().any(|f| f.queued || f.received.is_none())
        || exits.len() != actors.len()
        || verdict.is_none()
    {
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
        let mut received = frame.clone();
        received["action"] = json!("receive");
        let mut rows = vec![
            json!({"direction":"runtime","message":artifact["compiled"]}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[0,tap::SEND_POLL,7u16<<8|7,1,0xffff0000u32]]}}),
            json!({"direction":"runtime","message":frame}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[1,tap::ENDPOINT_SEND,7u16<<8|1,1,7u64<<24|lane<<16]]}}),
            json!({"direction":"runtime","message":received}),
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
        let mut journal = read(&rows, &labels, &actors)?;
        let checked = crate::projection::check_dynamic(&artifact, &rows)?;
        let receipts = journal.receipts(&checked)?;
        assert_eq!(receipts[&19]["receive_commit"]["journal_row"], 5);
        assert_eq!(journal.frames[&19].received, Some(4));
        journal.bind_receives(&checked)?;
        assert_eq!(journal.frames[&19].received, Some(5));
        let mut duplicate = checked.clone();
        duplicate["sessions"][0]["acceptances"]
            .as_array_mut()
            .unwrap()
            .push(receipts[&19].clone());
        assert!(journal.receipts(&duplicate).is_err());
        for (field, value) in [("source_event", 1), ("view", 7)] {
            let mut altered = checked.clone();
            altered["sessions"][0]["acceptances"][0]["receive_commit"][field] = json!(value);
            assert!(journal.receipts(&altered).is_err(), "accepted {field}");
        }
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
        let mut journal_late = read(&late, &labels, &actors)?;
        assert!(journal_late.frames[&19].sent > 3);
        assert_eq!(
            journal_late.receipts(&checked_late)?[&19]["receive_commit"]["journal_row"],
            3
        );
        journal_late.bind_receives(&checked_late)?;
        assert_eq!(journal_late.frames[&19].received, Some(3));
        // EPF correctly permits a finite source prefix with an accepted send.
        // Complete application effect evidence must additionally have its receive.
        rows.remove(5);
        rows[5]["message"]["checkpoint"] = json!(2);
        let prefix = crate::projection::check_dynamic(&artifact, &rows)?;
        assert_eq!(prefix["status"], "passed");
        assert!(read(&rows, &labels, &actors)?.receipts(&prefix).is_err());
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
        assert!(read(&rows, &BTreeMap::new(), &actors).is_ok());
        for removed in [0, 1, 2, 3, 4, 5, 6] {
            let mut changed = rows.clone();
            changed.remove(removed);
            assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        }
        let mut changed = rows.clone();
        changed.swap(2, 3);
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        let mut changed = rows.clone();
        changed.swap(5, 6);
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        let mut changed = rows.clone();
        changed[1]["message"]["arguments"]["section_name"] = json!("section_b");
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        let mut changed = rows.clone();
        changed[3]["message"]["result"]["success"] = json!(false);
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        let mut changed = rows.clone();
        changed[2]["elapsed_seconds"] = json!(0.05);
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
        let mut changed = rows.clone();
        changed.insert(1, rows[0].clone());
        assert!(read(&changed, &BTreeMap::new(), &actors).is_err());
    }

    #[test]
    fn close_records_cannot_reuse_a_request_or_substitute_another_syscall() {
        let request = Frame {
            logical_label: protocol::LABEL_WASI_FD_CLOSE,
            identity: json!({"source":0,"target":8}),
            bytes: vec![6],
            sent: 0,
            received: Some(1),
            queued: false,
        };
        let reply = Frame {
            logical_label: protocol::LABEL_WASI_FD_CLOSE_RET,
            identity: json!({"source":8,"target":0}),
            bytes: vec![6, 0, 0, 0],
            sent: 2,
            received: Some(3),
            queued: false,
        };
        let mut frames = BTreeMap::from([(1, request), (2, reply)]);
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_ok());
        frames.get_mut(&1).unwrap().logical_label = protocol::LABEL_WASI_FD_CLOSE_RET;
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
        frames.get_mut(&1).unwrap().logical_label = protocol::LABEL_WASI_FD_CLOSE;
        frames.get_mut(&2).unwrap().bytes[0] = 4;
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
        frames.get_mut(&2).unwrap().bytes[0] = 6;
        frames.insert(
            3,
            Frame {
                logical_label: protocol::LABEL_WASI_FD_CLOSE_RET,
                identity: json!({"source":8,"target":0}),
                bytes: vec![6, 0, 0, 0],
                sent: 4,
                received: Some(5),
                queued: false,
            },
        );
        assert!(check_close_pairs(&frames, &[(0, 8)]).is_err());
    }
}
