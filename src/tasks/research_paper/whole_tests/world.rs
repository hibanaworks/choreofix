//! The real common world supplies every effect; no material is prepared beforehand.
use super::*;
use hibana_runtime::host::{Io, Process};
use std::{
    net::SocketAddr,
    path::Path,
    process::{Command, Stdio},
};

// One owned connection per actual RPC: no shared response table or task state.
struct World<'a> {
    address: SocketAddr,
    observed: &'a Cell<Vec<tap::TapEvent>>,
    calls: Cell<Vec<Value>>,
}
async fn read_json(io: &mut Io) -> ShellResult<Value> {
    let mut bytes = Vec::new();
    loop {
        let mut chunk = [0u8; 4096];
        let count = io.read(&mut chunk).await?;
        if count == 0 {
            return Err(ShellError::message("world disconnected before its reply"));
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.contains(&b'\n') {
            return imp::decode(&bytes);
        }
    }
}
impl World<'_> {
    async fn request(&self, call: Value) -> ShellResult<Value> {
        let mut socket = Io::connect(self.address).await?;
        let mut bytes = imp::encode(&call)?;
        bytes.push(b'\n');
        socket.write_all(&bytes).await?;
        let response = read_json(&mut socket).await?;
        if let Some(error) = response.get("error") {
            return Err(ShellError::message(format!("world RPC: {error}")));
        }
        Ok(response)
    }
}
impl Domain for World<'_> {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let observations = self.observed.take();
        let before = observations.len();
        self.observed.set(observations);
        let mut calls = self.calls.take();
        let request_row = calls.len();
        calls.push(json!({"kind":"request","actor":actor,"tool":tool,"arguments":arguments,"capture_before":before}));
        self.calls.set(calls);
        let response = self
            .request(json!({"actor":actor,"tool":tool,"arguments":arguments}))
            .await
            .map_err(|e| e.to_string())?;
        let result = response
            .get("result")
            .cloned()
            .ok_or("missing actual world result")?;
        let mut calls = self.calls.take();
        calls.push(json!({"kind":"reply","request_row":request_row,"event_index":response["event_index"],"result":result}));
        self.calls.set(calls);
        Ok(result)
    }
}

fn dynamic_audit(evidence: &Value) -> ShellResult<Value> {
    use hibana_epf::{
        Acceptance, AuditStorage, Capture, ExecutionStorage, Protocol, RecordId, RouteChoice,
        Unfold,
    };
    // Only decode the canonical five-word record format. All eligibility,
    // visits, custody and occurrence correspondence are checked by EPF.
    let decode = |value: &Value| {
        let words = value.as_array().expect("native five-word record");
        assert_eq!(words.len(), 5);
        let word = |i: usize| u32::try_from(words[i].as_u64().unwrap()).unwrap();
        let mut bytes = [0u8; 16];
        bytes[..4].copy_from_slice(&word(0).to_be_bytes());
        bytes[4..6].copy_from_slice(&u16::try_from(word(1)).unwrap().to_be_bytes());
        bytes[6..8].copy_from_slice(&u16::try_from(word(2)).unwrap().to_be_bytes());
        bytes[8..12].copy_from_slice(&word(3).to_be_bytes());
        bytes[12..].copy_from_slice(&word(4).to_be_bytes());
        tap::TapEvent::from(bytes)
    };
    let compiled = evidence["compiled"]
        .as_array()
        .unwrap()
        .iter()
        .map(decode)
        .collect::<Vec<_>>();
    let records = evidence["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(decode)
        .collect::<Vec<_>>();
    let checkpoint = evidence["checkpoint"]
        .as_u64()
        .map(|n| u32::try_from(n).unwrap());
    let mut source_storage = vec![
        0;
        Protocol::workspace_len(&compiled).map_err(|e| {
            ShellError::message(format!("source header: {e:?}"))
        })?
    ];
    let protocol = Protocol::new(&compiled, &mut source_storage)
        .map_err(|e| ShellError::message(format!("source snapshot: {e:?}")))?;
    let captures = [Capture::new(&records, checkpoint)
        .map_err(|e| ShellError::message(format!("capture: {e:?}")))?];
    let count = records.len();
    let capacity = count
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .expect("native capture capacity");
    let mut unfolding = vec![Unfold::Source; capacity];
    let mut choices = vec![
        RouteChoice {
            span: [0, 0],
            right: false
        };
        protocol
            .audit_choice_capacity(capacity)
            .map_err(|e| ShellError::message(format!("choices: {e:?}")))?
    ];
    let producer = vec![0; count];
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
            .map_err(|e| ShellError::message(format!("audit space: {e:?}")))?
    ];
    let audit = protocol
        .audit(
            &captures,
            AuditStorage {
                unfolding: &mut unfolding,
                choices: &mut choices,
                execution: ExecutionStorage {
                    session: SESSION,
                    observed_producers: &producer,
                    order: &mut order,
                    occurrences: &mut occurrences,
                    acceptances: &mut acceptances,
                },
            },
            &mut workspace,
        )
        .map_err(|e| ShellError::message(format!("dynamic EPF: {e:?}")))?;
    let source = audit.path().unfolding();
    let correspondence=audit.execution().order.iter().zip(audit.execution().occurrences).map(|(record,point)|json!({
        "ordinal":record.ordinal,"occurrence":point,"source_event":point.and_then(|event|source.origin(event))
    })).collect::<Vec<_>>();
    Ok(
        json!({"complete_capture":true,"dynamic_source_compatible":true,"records":count,
        "source_events":compiled[0].arg0()>>16,"expanded_events":source.event_count(),"correspondence":correspondence,
        "application_effects_certified":false,"session_complete_certified":false}),
    )
}

// Application receipts are observation data. Eligibility and custody come from
// the checked EPF correspondence; no second protocol/workflow is implemented.
fn effect_receipts(
    (evidence, checked): (&Value, &Value),
    world: &Value,
    calls: &Value,
) -> Result<Value, String> {
    use hibana::runtime::wire::WirePayload;
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    if checked["dynamic_source_compatible"] != true {
        return Err("missing checked EPF execution".into());
    }
    let compiled = evidence["compiled"].as_array().ok_or("compiled source")?;
    let capture = evidence["records"].as_array().ok_or("capture")?;
    let events = world["events"].as_array().ok_or("world effects")?;
    let calls = calls.as_array().ok_or("RPC observations")?;
    let source = |event: u16| -> Result<&Value, String> {
        let row = compiled.get(2 + usize::from(event)).ok_or("source event")?;
        if row[1] != u64::from(tap::PROJECTED_SOURCE) || row[0] != u64::from(event) {
            return Err("source identity".into());
        }
        Ok(row)
    };
    let word = |row: &Value, index: usize| -> Result<u32, String> {
        row.get(index)
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("record word".into())
    };
    let mut sent = BTreeMap::new();
    let mut delivered = BTreeMap::new();
    for (index, frame) in evidence["frames"]
        .as_array()
        .ok_or("frames")?
        .iter()
        .enumerate()
    {
        let id = frame["frame_id"].as_u64().ok_or("frame ID")?;
        if frame["action"] == "send" {
            if sent.insert(id, frame).is_some() {
                return Err("reused send frame ID".into());
            }
        } else if frame["action"] == "receive" {
            delivered.insert(id, (index, frame));
        }
    }
    if sent.len() != delivered.len() {
        return Err("unreceived final frame".into());
    }
    let mut delivered = delivered.into_iter().collect::<Vec<_>>();
    delivered.sort_by_key(|(_, (index, _))| *index);
    let mut lanes = BTreeMap::<u16, VecDeque<(u64, &Value)>>::new();
    for (id, (_, frame)) in delivered {
        let before = sent.get(&id).ok_or("receive without send")?;
        for name in [
            "session",
            "source",
            "target",
            "lane",
            "frame_label",
            "payload",
        ] {
            if before[name] != frame[name] {
                return Err("send/receive contents changed".into());
            }
        }
        let origin = frame["frame_label"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .ok_or("frame origin")?;
        let row = source(origin)?;
        let key = word(row, 3)?;
        if frame["session"] != u64::from(SESSION)
            || frame["source"] != u64::from(key >> 24)
            || frame["target"] != u64::from((key >> 16) as u8)
            || frame["lane"] != u64::from((key >> 8) as u8)
        {
            return Err("frame disagrees with the compiled source".into());
        }
        lanes.entry(origin).or_default().push_back((id, frame));
    }
    let mut receipts = Vec::new();
    for (ordinal, row) in capture.iter().enumerate() {
        let kind = word(row, 1)? as u16;
        if kind != tap::ENDPOINT_RECV
            && !(kind == tap::ENDPOINT_SESSION && word(row, 2)? as u8 == 2)
        {
            continue;
        }
        let origin = word(row, 4)? as u16;
        let point = &checked["correspondence"][ordinal];
        if point["ordinal"] != ordinal
            || point["source_event"] != u64::from(origin)
            || !point["occurrence"].is_number()
        {
            return Err("receive has no checked occurrence".into());
        }
        let (id, frame) = lanes
            .get_mut(&origin)
            .and_then(VecDeque::pop_front)
            .ok_or("receive has no frame occurrence")?;
        receipts.push((ordinal, source(origin)?, frame, word(row, 2)? >> 8, id));
    }
    if lanes.values().any(|v| !v.is_empty()) {
        return Err("frame has no receive commit".into());
    }
    let mut used_requests = BTreeSet::new();
    let mut used_authorities = BTreeSet::new();
    let mut used_events = BTreeSet::new();
    let mut used_replies = BTreeSet::new();
    let mut links = Vec::new();
    let mut requests = 0;
    for (request_row, call) in calls.iter().enumerate() {
        if call["kind"] != "request" {
            continue;
        }
        requests += 1;
        let before = call["capture_before"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| *n <= capture.len())
            .ok_or("effect capture boundary")?;
        let matching = calls
            .iter()
            .enumerate()
            .filter(|(_, row)| row["kind"] == "reply" && row["request_row"] == request_row)
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err("RPC request needs one actual reply".into());
        }
        let (reply_row, reply) = matching[0];
        if !used_replies.insert(reply_row) {
            return Err("RPC reply reused".into());
        }
        let event_index = reply["event_index"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("world event index")?;
        if !used_events.insert(event_index) {
            return Err("world effect occurrence reused".into());
        }
        let event = events.get(event_index).ok_or("world event")?;
        if call["actor"] != event["agent"]
            || call["tool"] != event["tool"]
            || call["arguments"] != event["args"]
            || reply["result"]["success"] != event["success"]
            || reply["result"]["data"] != event["result"]
        {
            return Err("RPC differs from the actual world effect".into());
        }
        let tool = call["tool"].as_str().ok_or("effect tool")?;
        let (schema, author, operation) = if tool == "combine_sections" {
            if call["actor"] != "EDITOR" {
                return Err("publication actor".into());
            }
            (ReportArguments::<6>::SCHEMA_ID, None, None)
        } else {
            let operation = (1..=6)
                .find(|op| imp::tool(*op).is_ok_and(|name| name == tool))
                .ok_or("unknown effect contract")?;
            let author = if operation <= 4 {
                imp::AUTHORS.iter().position(|name| call["actor"] == *name)
            } else {
                imp::SECTIONS
                    .iter()
                    .position(|name| call["arguments"]["section_name"] == *name)
            }
            .ok_or("effect participant")?;
            imp::result(author as u8, operation, &reply["result"]).map_err(|e| e.to_string())?;
            if call["actor"] != reply["result"]["data"]["agent"] {
                return Err("effect actor differs".into());
            }
            (
                imp::arguments_schema(author as u8, operation),
                Some(author),
                Some(operation),
            )
        };
        let request = receipts
            .iter()
            .rev()
            .find(|(ordinal, meta, _, _, _)| {
                *ordinal < before
                    && !used_requests.contains(ordinal)
                    && word(meta, 2).is_ok_and(|kind| kind >> 8 == 0)
                    && meta[4] == schema
            })
            .ok_or("effect lacks a fresh accepted request")?;
        let bytes = request.2["payload"]
            .as_array()
            .ok_or("request payload")?
            .iter()
            .map(|n| {
                n.as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or("payload byte")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())? != call["arguments"]
        {
            return Err("effect arguments differ from the accepted request".into());
        }
        used_requests.insert(request.0);
        let needed = if operation.is_some_and(|op| op == 5 || op == 6) {
            vec![[SECTION_A, SECTION_B, SECTION_C][author.unwrap()]]
        } else if operation.is_none() {
            vec![SECTION_A, SECTION_B, SECTION_C]
        } else {
            Vec::new()
        };
        let mut custody = Vec::new();
        for role in needed {
            let receipt = receipts
                .iter()
                .rev()
                .find(|(ordinal, meta, _, actor, _)| {
                    request.0 < *ordinal
                        && *ordinal < before
                        && *actor == request.3
                        && !used_authorities.contains(ordinal)
                        && word(meta, 2).is_ok_and(|kind| kind >> 8 == 1)
                        && meta[4] == u64::from(role)
                })
                .ok_or("effect lacks its consumed chapter authority")?;
            used_authorities.insert(receipt.0);
            custody.push(json!({"ordinal":receipt.0,"frame_id":receipt.4,"logical_role":role}));
        }
        links.push(json!({"request_row":request_row,"event_index":event_index,"receive_ordinal":request.0,
            "frame_id":request.4,"source_event":request.1[0],"occurrence":checked["correspondence"][request.0]["occurrence"],"custody":custody}));
    }
    let declared = receipts
        .iter()
        .filter(|(_, meta, _, _, _)| {
            meta[4] == u64::from(ReportArguments::<6>::SCHEMA_ID)
                || (0..3).any(|author| {
                    (1..=6).any(|op| meta[4] == u64::from(imp::arguments_schema(author, op)))
                })
        })
        .count();
    if requests != events.len()
        || requests != used_replies.len()
        || calls.len() != 2 * requests
        || declared != used_requests.len()
    {
        return Err("missing, extra or reused request/effect/reply".into());
    }
    Ok(
        json!({"accepted_request_effect_pairs":requests,"unique_world_effects":used_events.len(),
        "chapter_authority_receipts":used_authorities.len(),"links":links,
        "scope":"Finite native RPC/accepted-request/actual-world occurrence correspondence, with EPF-checked chapter custody.",
        "arbitrary_rust_receiver_certified":false,"whole_task_application_qualified":false}),
    )
}

fn reject_receipt_mutations(
    (evidence, checked): (&Value, &Value),
    world: &Value,
    calls: &Value,
) -> Value {
    let rows = calls.as_array().unwrap();
    let request = rows.iter().position(|r| r["kind"] == "request").unwrap();
    let reply = rows
        .iter()
        .position(|r| r["kind"] == "reply" && r["request_row"] == request)
        .unwrap();
    let event = rows[reply]["event_index"].as_u64().unwrap() as usize;
    let mut mutants = Vec::new();

    let mut altered = calls.clone();
    altered.as_array_mut().unwrap().remove(reply);
    mutants.push((
        "missing_rpc_reply",
        evidence.clone(),
        world.clone(),
        altered,
    ));

    let mut altered = calls.clone();
    altered[request]["capture_before"] = json!(0);
    mutants.push((
        "effect_before_request",
        evidence.clone(),
        world.clone(),
        altered,
    ));

    // Change the independent effect journal too: checking only RPC/world
    // agreement would miss this substitution of the admitted request.
    let mut altered = calls.clone();
    let mut changed_world = world.clone();
    altered[request]["arguments"]["topic"] = json!("unadmitted_research");
    changed_world["events"][event]["args"] = altered[request]["arguments"].clone();
    mutants.push((
        "changed_admitted_arguments",
        evidence.clone(),
        changed_world,
        altered,
    ));

    let mut altered = calls.clone();
    altered[reply]["result"]["success"] = json!(false);
    mutants.push((
        "changed_actual_result",
        evidence.clone(),
        world.clone(),
        altered,
    ));

    let mut altered = calls.clone();
    let next_reply = rows
        .iter()
        .enumerate()
        .find(|(i, r)| *i != reply && r["kind"] == "reply")
        .unwrap()
        .0;
    altered[next_reply]["event_index"] = rows[reply]["event_index"].clone();
    mutants.push((
        "reused_world_occurrence",
        evidence.clone(),
        world.clone(),
        altered,
    ));

    let mut changed_world = world.clone();
    changed_world["events"].as_array_mut().unwrap().pop();
    mutants.push((
        "missing_world_effect",
        evidence.clone(),
        changed_world,
        calls.clone(),
    ));

    for field in ["frame_label", "lane"] {
        let mut altered = evidence.clone();
        let first = altered["frames"].as_array().unwrap()[0]["frame_id"].clone();
        for frame in altered["frames"].as_array_mut().unwrap() {
            if frame["frame_id"] == first {
                frame[field] = json!(u16::MAX);
            }
        }
        mutants.push((field, altered, world.clone(), calls.clone()));
    }

    let mut altered = evidence.clone();
    let authority = altered["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|frame| {
            frame["action"] == "send" && {
                let origin = frame["frame_label"].as_u64().unwrap() as usize;
                evidence["compiled"][2 + origin][2].as_u64().unwrap() >> 8 == 1
            }
        })
        .unwrap()["frame_id"]
        .clone();
    altered["frames"]
        .as_array_mut()
        .unwrap()
        .retain(|frame| frame["frame_id"] != authority);
    mutants.push((
        "missing_authority_delivery",
        altered,
        world.clone(),
        calls.clone(),
    ));

    // Repeated reviews use identical arguments. An older admitted occurrence
    // must not stand in for the later request even when all values agree.
    if let Some((later, earlier)) = rows.iter().enumerate().find_map(|(later, call)| {
        if call["kind"] != "request" {
            return None;
        }
        rows[..later]
            .iter()
            .position(|old| {
                old["kind"] == "request"
                    && old["actor"] == call["actor"]
                    && old["tool"] == call["tool"]
                    && old["arguments"] == call["arguments"]
            })
            .map(|earlier| (later, earlier))
    }) {
        let mut altered = calls.clone();
        altered[later]["capture_before"] = rows[earlier]["capture_before"].clone();
        mutants.push((
            "reused_same_arguments_request",
            evidence.clone(),
            world.clone(),
            altered,
        ));
    }

    json!(
        mutants
            .into_iter()
            .map(|(name, altered, changed_world, changed_calls)| {
                let error = effect_receipts((&altered, checked), &changed_world, &changed_calls)
                    .expect_err(name);
                json!({"mutation":name,"rejected":true,"error":error})
            })
            .collect::<Vec<_>>()
    )
}

#[hibana_runtime::main]
#[test]
async fn all_effects_use_the_real_common_world_under_the_whole_source() -> ShellResult<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut qualified = Vec::new();
    for (depth, starts) in [
        (0, [0, 3, 6]),
        (0, [6, 0, 3]),
        (0, [3, 6, 0]),
        (1, [6, 3, 0]),
        (2, [0, 6, 3]),
    ] {
        let mut command = Command::new(root.join(".venv/bin/python"));
        command
            .arg("-u")
            .arg("-m")
            .arg("benchmark.tests.fixtures.research_paper_world")
            .arg(depth.to_string())
            .arg("0.01")
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = Process::spawn(&mut command)?;
        let (_input, mut output) = child.io()?;
        let hello = hibana_runtime::race(
            read_json(&mut output),
            hibana_runtime::host::deadline(Duration::from_secs(30)),
        )
        .await?;
        let address =
            SocketAddr::from(([127, 0, 0, 1], hello["address"][1].as_u64().unwrap() as u16));
        let observed = Cell::new(Vec::new());
        let world = World {
            address,
            observed: &observed,
            calls: Cell::new(Vec::new()),
        };
        let (run, evidence) = exercise(&world, (starts, None), &observed).await?;
        let actual = world.request(json!({"inspect":true})).await?;
        let directory = Path::new(evidence["evidence_directory"].as_str().unwrap());
        std::fs::write(directory.join("world-start.json"), imp::encode(&hello)?)?;
        std::fs::write(directory.join("world-result.json"), imp::encode(&actual)?)?;
        let calls = world.calls.take();
        std::fs::write(
            directory.join("effect-rpc.json"),
            imp::encode(&json!(&calls))?,
        )?;
        // The owner reaps its server only after the complete result is saved.
        drop(child);
        run?;
        let checked = dynamic_audit(&evidence);
        match &checked {
            Ok(result) => {
                std::fs::write(directory.join("dynamic-audit.json"), imp::encode(result)?)?
            }
            Err(error) => std::fs::write(
                directory.join("dynamic-audit.json"),
                imp::encode(&json!({"error":error.to_string(),"dynamic_source_compatible":false}))?,
            )?,
        }
        let checked = checked?;
        let calls = json!(calls);
        let application =
            effect_receipts((&evidence, &checked), &actual, &calls).map_err(ShellError::message)?;
        std::fs::write(
            directory.join("effect-audit.json"),
            imp::encode(&application)?,
        )?;
        let mutations = reject_receipt_mutations((&evidence, &checked), &actual, &calls);
        std::fs::write(
            directory.join("effect-audit-mutations.json"),
            imp::encode(&mutations)?,
        )?;
        assert_eq!(actual["complete"], true);
        assert_eq!(actual["violations"], json!([]));
        assert_eq!(hello["source_files"], actual["source_files"]);
        let events = actual["events"].as_array().unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|r| r["tool"] == "combine_sections")
                .count(),
            1
        );
        assert!(events.iter().all(|r| r["completed_at"].is_number()));
        qualified.push(json!({"depth":depth,"starts":starts,"evidence_directory":directory,
            "effects":events.len(),"complete":true,"violations":0,"model_calls":0,"wasm_conditions":0,"scored_runs":0}));
    }
    let last = Path::new(
        qualified.last().unwrap()["evidence_directory"]
            .as_str()
            .unwrap(),
    );
    let summary = last.parent().unwrap().join(format!(
        "real-world-series-{}.json",
        last.file_name().unwrap().to_str().unwrap()
    ));
    std::fs::write(
        summary,
        imp::encode(
            &json!({"task":"3M","cases":qualified,"scope":"All effects passed through the whole native Endpoint source and actual receivers; scripted participants, not AI/Wasm qualification.","model_calls":0,"scored_runs":0,"counted_as_accepted_port":false}),
        )?,
    )?;
    Ok(())
}
