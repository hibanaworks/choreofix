//! Decode evidence and delegate source interpretation to hibana-epf.
use crate::evidence::Result;
use hibana::runtime::tap::{self, TapEvent};
use hibana_epf::{
    Acceptance, AuditStorage, Capture, ExecutionStorage, Protocol, RecordId, RouteChoice, Unfold,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn record(value: &Value) -> Result<TapEvent> {
    let fields = value
        .as_array()
        .filter(|v| v.len() == 5)
        .ok_or("tap record layout")?;
    let words: Vec<_> = fields
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("tap record word")
        })
        .collect::<std::result::Result<_, _>>()?;
    let kind = u16::try_from(words[1])?;
    let key = u16::try_from(words[2])?;
    let mut bytes = [0; 16];
    bytes[..4].copy_from_slice(&words[0].to_be_bytes());
    bytes[4..6].copy_from_slice(&kind.to_be_bytes());
    bytes[6..8].copy_from_slice(&key.to_be_bytes());
    bytes[8..12].copy_from_slice(&words[3].to_be_bytes());
    bytes[12..].copy_from_slice(&words[4].to_be_bytes());
    Ok(bytes.into())
}

pub(super) fn snapshots(compiled: &Value) -> Result<Vec<Vec<TapEvent>>> {
    let mut snapshots = Vec::new();
    let mut common = None;
    for program in compiled["programs"].as_array().ok_or("compiled programs")? {
        let rows: Vec<_> = program["rows"]
            .as_array()
            .ok_or("compiled rows")?
            .iter()
            .map(record)
            .collect::<Result<_>>()?;
        let mut workspace = vec![
            0;
            Protocol::workspace_len(&rows)
                .map_err(|e| format!("EPF protocol header: {e:?}"))?
        ];
        Protocol::new(&rows, &mut workspace)
            .map_err(|e| format!("EPF compiled protocol: {e:?}"))?;
        if program["role"] != u64::from(rows[0].causal_key() as u8) {
            return Err("compiled role disagrees with snapshot".into());
        }
        let local = rows[1].ts() as usize;
        let body: Vec<_> = rows[2..rows.len() - local]
            .iter()
            .map(|row| {
                let key = if matches!(
                    row.id(),
                    hibana::runtime::tap::PROJECTED_BINDING
                        | hibana::runtime::tap::PROJECTED_INITIAL_ACTOR
                ) {
                    0
                } else {
                    row.causal_key()
                };
                [
                    row.ts(),
                    u32::from(row.id()),
                    u32::from(key),
                    row.arg0(),
                    row.arg1(),
                ]
            })
            .collect();
        if common.as_ref().is_some_and(|previous| previous != &body) {
            return Err("compiled programs disagree about their global source".into());
        }
        common = Some(body);
        snapshots.push(rows);
    }
    if snapshots.is_empty() {
        return Err("missing compiled protocol".into());
    }
    Ok(snapshots)
}

/// Require complete producer prefixes, then construct and check every named
/// run against the actual executable's compiled global. No task grammar or
/// local progress/ownership implementation belongs here.
pub fn check(artifact: &Value, rows: &[Value]) -> Result<Value> {
    let compiled = snapshots(&artifact["compiled"])?;
    let snapshot = &compiled[0];
    let mut source_storage =
        vec![0; Protocol::workspace_len(snapshot).map_err(|e| format!("EPF workspace: {e:?}"))?];
    let protocol =
        Protocol::new(snapshot, &mut source_storage).map_err(|e| format!("EPF protocol: {e:?}"))?;
    let mut streams = BTreeMap::<u64, Vec<TapEvent>>::new();
    let mut journal_positions = BTreeMap::<u64, Vec<usize>>::new();
    let mut checkpoints = BTreeMap::<u64, u32>::new();
    let mut observations = Vec::new();
    let mut sessions = BTreeSet::new();
    for (journal_row, row) in rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["direction"] == "runtime")
    {
        let message = &row["message"];
        match message["type"].as_str() {
            Some("protocol_capture") => {
                let producer = message["producer"].as_u64().ok_or("capture producer")?;
                if checkpoints.contains_key(&producer) {
                    return Err("records after producer checkpoint".into());
                }
                let records = message["records"].as_array().ok_or("capture records")?;
                let stream = streams.entry(producer).or_default();
                for value in records {
                    stream.push(record(value)?);
                    journal_positions
                        .entry(producer)
                        .or_default()
                        .push(journal_row);
                    observations.push(producer);
                }
            }
            Some("protocol_checkpoint") => {
                let producer = message["producer"].as_u64().ok_or("checkpoint producer")?;
                let checkpoint = message["checkpoint"]
                    .as_u64()
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or("missing/exhausted producer checkpoint")?;
                let session = message["session"]
                    .as_u64()
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or("checkpoint session")?;
                if checkpoints.insert(producer, checkpoint).is_some() {
                    return Err("duplicate producer checkpoint".into());
                }
                streams.entry(producer).or_default();
                sessions.insert(session);
            }
            _ => {}
        }
    }
    if streams.is_empty() || checkpoints.len() != streams.len() {
        return Err("incomplete protocol capture inventory".into());
    }
    let producer_ids: Vec<_> = streams.keys().copied().collect();
    let observed_producers: Vec<_> = observations
        .into_iter()
        .map(|producer| {
            producer_ids
                .binary_search(&producer)
                .expect("captured producer")
        })
        .collect();
    let captures: Vec<_> = streams
        .iter()
        .map(|(producer, records)| {
            Capture::new(records, checkpoints.get(producer).copied())
                .map_err(|e| format!("EPF producer {producer}: {e:?}").into())
        })
        .collect::<Result<_>>()?;
    for record in captures.iter().flat_map(|capture| capture.records()) {
        if matches!(
            record.id(),
            tap::SEND_POLL
                | tap::ENDPOINT_SEND
                | tap::ENDPOINT_RECV
                | tap::ENDPOINT_SESSION
                | tap::SCOPE_REENTRY
                | tap::ROUTE_ARM_SELECTION
                | tap::RESOLVER_AUDIT
        ) && !sessions.contains(&record.arg0())
        {
            return Err("capture contains an undeclared session".into());
        }
    }
    let mut frames = BTreeMap::<(u32, u16), VecDeque<u64>>::new();
    let mut frame_ids = BTreeSet::new();
    for row in rows.iter().filter(|r| {
        r["direction"] == "runtime"
            && r["message"]["type"] == "protocol_frame"
            && r["message"]["action"] == "send"
    }) {
        let frame = &row["message"];
        let session = frame["session"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("frame session")?;
        let origin = frame["frame_label"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .ok_or("frame label")?;
        let source = snapshot
            .get(2 + usize::from(origin))
            .filter(|r| r.id() == tap::PROJECTED_SOURCE)
            .ok_or("frame source occurrence")?;
        for (field, value) in [
            ("lane", (source.arg0() >> 8) as u8),
            ("source", (source.arg0() >> 24) as u8),
            ("target", (source.arg0() >> 16) as u8),
        ] {
            if frame[field] != value {
                return Err("frame disagrees with the EPF source occurrence".into());
            }
        }
        let id = frame["frame_id"].as_u64().ok_or("frame ID")?;
        if !sessions.contains(&session) || !frame_ids.insert(id) {
            return Err("unmatched session or reused send frame".into());
        }
        frames.entry((session, origin)).or_default().push_back(id);
    }
    // A successful nonlocal commit needs its actual transport frame. Reject
    // missing journal evidence before allocating a source-search workspace.
    // Accepted sends without a sender commit or receive remain valid prefixes.
    let mut commits = BTreeMap::<(u32, u16), [usize; 2]>::new();
    for event in captures.iter().flat_map(|capture| capture.records()) {
        let direction = match event.id() {
            tap::ENDPOINT_SEND => 0,
            tap::ENDPOINT_RECV => 1,
            _ => continue,
        };
        let origin = event.arg1() as u16;
        let source = snapshot
            .get(2 + usize::from(origin))
            .filter(|row| row.id() == tap::PROJECTED_SOURCE)
            .ok_or("committed source occurrence")?;
        if source.arg0() >> 24 == (source.arg0() >> 16) & 255 {
            continue;
        }
        let key = (event.arg0(), origin);
        let count = &mut commits.entry(key).or_default()[direction];
        *count += 1;
        if *count > frames.get(&key).map_or(0, VecDeque::len) {
            return Err("committed operation lacks its transport frame".into());
        }
    }
    let count: usize = captures.iter().map(|c| c.records().len()).sum();
    let capacity = count
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .ok_or("source proof capacity overflow")?;
    let mut reports = Vec::new();
    for session in sessions {
        let mut derivation = vec![Unfold::Source; capacity];
        let mut choices = vec![
            RouteChoice {
                span: [0, 0],
                right: false
            };
            protocol
                .audit_choice_capacity(capacity)
                .map_err(|e| format!("EPF choices: {e:?}"))?
        ];
        let mut order = vec![
            RecordId {
                producer: 0,
                ordinal: 0
            };
            count
        ];
        let mut occurrences = vec![None; count];
        let mut acceptances = vec![
            Acceptance {
                first_poll: RecordId {
                    producer: 0,
                    ordinal: 0
                },
                before: 0
            };
            count
        ];
        let mut workspace = vec![
            0;
            protocol
                .audit_workspace_len(&captures, capacity)
                .map_err(|e| format!("EPF workspace: {e:?}"))?
        ];
        let audit = protocol
            .audit(
                &captures,
                AuditStorage {
                    unfolding: &mut derivation,
                    choices: &mut choices,
                    execution: ExecutionStorage {
                        session,
                        observed_producers: &observed_producers,
                        order: &mut order,
                        occurrences: &mut occurrences,
                        acceptances: &mut acceptances,
                    },
                },
                &mut workspace,
            )
            .map_err(|e| format!("EPF dynamic eligibility: {e:?}"))?;
        let source = audit.path().unfolding();
        let correspondence: Vec<_> = audit.execution().order.iter().zip(audit.execution().occurrences).map(|(id, occurrence)|
            json!({"producer":streams.keys().nth(id.producer),"ordinal":id.ordinal,"occurrence":occurrence,
                "source_event":occurrence.and_then(|event| source.origin(event))})
        ).collect();
        // These are provenance links from the checked EPF witness, not another
        // execution model. A finite prefix may have an acceptance without a
        // receive commit. An effect auditor must require its own requests'
        // receive commits and the producer's ordered effect/capture contract.
        let mut poll_occurrences = BTreeMap::new();
        let mut receive_commits = BTreeMap::new();
        for (id, occurrence) in audit
            .execution()
            .order
            .iter()
            .zip(audit.execution().occurrences)
        {
            let Some(occurrence) = occurrence else {
                continue;
            };
            poll_occurrences.insert((id.producer, id.ordinal), *occurrence);
            let event = captures[id.producer].records()[id.ordinal];
            if event.id() == tap::ENDPOINT_RECV {
                let producer = producer_ids[id.producer];
                let journal_row = journal_positions
                    .get(&producer)
                    .and_then(|positions| positions.get(id.ordinal))
                    .ok_or("receive commit has no original journal observation")?;
                let receipt = json!({"producer":producer,"ordinal":id.ordinal,
                    "journal_row":journal_row,"occurrence":occurrence,
                    "source_event":source.origin(*occurrence),"actor":event.causal_key()>>8,
                    "view":event.arg1()>>24});
                if receive_commits.insert(*occurrence, receipt).is_some() {
                    return Err("one source occurrence has multiple receive commits".into());
                }
            }
        }
        let mut intervals = Vec::new();
        for acceptance in audit.execution().acceptances {
            let call =
                captures[acceptance.first_poll.producer].records()[acceptance.first_poll.ordinal];
            let origin = call.arg1() as u16;
            let frame = frames
                .get_mut(&(session, origin))
                .and_then(VecDeque::pop_front)
                .ok_or("accepted call has no unique corresponding journal frame")?;
            let occurrence = poll_occurrences
                .get(&(
                    acceptance.first_poll.producer,
                    acceptance.first_poll.ordinal,
                ))
                .ok_or("acceptance lacks a checked source occurrence")?;
            intervals.push(json!({"producer":streams.keys().nth(acceptance.first_poll.producer),
                "first_poll":acceptance.first_poll.ordinal,"before":acceptance.before,"frame_id":frame,
                "occurrence":occurrence,"send_actor":call.causal_key()>>8,
                "send_view":call.causal_key() as u8,
                "receive_commit":receive_commits.get(occurrence)}));
        }
        reports.push(json!({"session":session,"expanded_events":source.event_count(),"order":correspondence,"acceptances":intervals}));
    }
    if frames.values().any(|remaining| !remaining.is_empty()) {
        return Err("journal frame has no checked acceptance occurrence".into());
    }
    Ok(
        json!({"status":"passed","kind":"epf_source_interpretation","producer_count":captures.len(),
        "captured_records":count,"sessions":reports,"application_effects_certified":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Value, Vec<Value>) {
        use hibana::{
            g,
            runtime::{program::project, tap},
        };
        let global = g::send::<7, 7, g::Msg<1, ()>>();
        let snapshot: Vec<_> = tap::projected(&project::<7>(&global))
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
        let lane = (snapshot[2][3] >> 8) as u8;
        let artifact = json!({"compiled":{"programs":[{"role":7,"rows":snapshot}]}});
        let rows = vec![
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[0,tap::ENDPOINT_SEND,7u16 << 8 | 1,1,7u32 << 24 | u32::from(lane) << 16]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":42,"session":1,"checkpoint":1}}),
        ];
        (artifact, rows)
    }

    #[test]
    fn complete_prefix_uses_the_compiled_source_without_an_audit_choreography() -> Result<()> {
        let (artifact, rows) = fixture();
        let report = check(&artifact, &rows)?;
        assert_eq!(report["status"], "passed");
        assert_eq!(report["sessions"][0]["order"][0]["producer"], 42);
        assert_eq!(report["sessions"][0]["order"][0]["source_event"], 0);
        assert_eq!(report["application_effects_certified"], false);
        Ok(())
    }

    #[test]
    fn acceptance_links_to_its_checked_receive_and_original_observation() -> Result<()> {
        use hibana::{g, runtime::program::project};
        let source = g::send::<7, 8, g::Msg<1, ()>>();
        let snapshot = tap::projected(&project::<7>(&source))
            .map(|r| {
                [
                    r.ts(),
                    u32::from(r.id()),
                    u32::from(r.causal_key()),
                    r.arg0(),
                    r.arg1(),
                ]
            })
            .collect::<Vec<_>>();
        let lane = (snapshot[2][3] >> 8) as u8;
        let artifact = json!({"compiled":{"programs":[{"role":7,"rows":snapshot}]}});
        let mut rows = vec![
            json!({"direction":"runtime","message":{"type":"protocol_frame","action":"send",
                "session":1,"frame_id":19,"frame_label":0,"lane":lane,"source":7,"target":8}}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[0,tap::SEND_POLL,7u16<<8|7,1,0xffff0000u32]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[1,tap::ENDPOINT_SEND,7u16<<8|1,1,7u32<<24|u32::from(lane)<<16]]}}),
            json!({"direction":"external","message":{"type":"unrelated_observation"}}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":42,
                "records":[[2,tap::ENDPOINT_RECV,8u16<<8|2,1,8u32<<24|u32::from(lane)<<16]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":42,"session":1,"checkpoint":3}}),
        ];
        let report = check(&artifact, &rows)?;
        let delivery = &report["sessions"][0]["acceptances"][0];
        assert_eq!(delivery["frame_id"], 19);
        assert_eq!(delivery["occurrence"], 0);
        assert_eq!(delivery["send_actor"], 7);
        assert_eq!(delivery["send_view"], 7);
        assert_eq!(
            delivery["receive_commit"],
            json!({"producer":42,"ordinal":2,"journal_row":4,
            "occurrence":0,"source_event":0,"actor":8,"view":8})
        );
        rows.remove(4);
        rows[4]["message"]["checkpoint"] = json!(2);
        let prefix = check(&artifact, &rows)?;
        assert!(
            prefix["sessions"][0]["acceptances"][0]["receive_commit"].is_null(),
            "a finite accepted prefix does not invent the receiver's commit"
        );
        Ok(())
    }

    #[test]
    fn observation_arrival_order_survives_producer_grouping() -> Result<()> {
        use hibana::{g, runtime::program::project};
        let global = g::par(
            g::send::<7, 7, g::Msg<1, ()>>(),
            g::send::<8, 8, g::Msg<2, ()>>(),
        );
        let snapshot: Vec<_> = tap::projected(&project::<7>(&global))
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
        let lanes = [(snapshot[2][3] >> 8) as u8, (snapshot[3][3] >> 8) as u8];
        let artifact = json!({"compiled":{"programs":[{"role":7,"rows":snapshot}]}});
        let mut rows = vec![
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":90,
                "records":[[0,tap::ENDPOINT_SEND,7u16 << 8 | 1,1,7u32 << 24 | u32::from(lanes[0]) << 16]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_capture","producer":10,
                "records":[[0,tap::ENDPOINT_SEND,8u16 << 8 | 1,1,8u32 << 24 | u32::from(lanes[1]) << 16 | 1]]}}),
            json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":90,"session":1,"checkpoint":1}}),
            json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":10,"session":1,"checkpoint":1}}),
        ];
        assert_eq!(
            check(&artifact, &rows)?["sessions"][0]["order"][0]["producer"],
            90
        );
        rows.swap(0, 1);
        assert_eq!(
            check(&artifact, &rows)?["sessions"][0]["order"][0]["producer"],
            10
        );
        Ok(())
    }

    #[test]
    fn missing_or_reused_producer_evidence_never_yields_a_dynamic_certificate() {
        let (artifact, rows) = fixture();
        assert!(check(&artifact, &rows[..1]).is_err());
        assert!(check(&artifact, &rows[1..]).is_err());
        let mut duplicate = rows.clone();
        duplicate.push(rows[1].clone());
        assert!(check(&artifact, &duplicate).is_err());
        let mut missing = rows.clone();
        missing[0]["message"]["records"] = json!([]);
        assert!(check(&artifact, &missing).is_err());
        let mut exhausted = rows.clone();
        exhausted[1]["message"]["checkpoint"] = Value::Null;
        assert!(check(&artifact, &exhausted).is_err());
    }
}
