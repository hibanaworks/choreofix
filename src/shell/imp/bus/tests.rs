use super::*;

#[hibana_runtime::main]
#[test]
async fn diagnostic_observation_does_not_wake_its_own_producer() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let polls = Cell::new(0);
    let result = hibana_runtime::race(
        async {
            poll_fn(|cx| {
                polls.set(polls.get() + 1);
                bus.connection(|connection| connection.output_waiter = Some(cx.waker().clone()));
                bus.observe(json!({"type":"observation"}));
                assert!(bus.connection(|connection| connection.output_waiter.is_some()));
                Poll::<std::io::Result<()>>::Pending
            })
            .await
        },
        hibana_runtime::host::deadline(std::time::Duration::from_millis(5)),
    )
    .await;
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut);
    // Only initial entry and the independent timer can poll this producer.
    assert!(
        polls.get() <= 2,
        "diagnostic feedback caused {} polls",
        polls.get()
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn observations_deliver_transport_evidence_without_waiting_for_an_rpc() -> ShellResult<()> {
    use hibana::{
        g,
        runtime::{SessionKitStorage, ids::SessionId, program::project},
    };
    type Message = g::Msg<1, u32>;
    let source = g::send::<1, 2, Message>();
    let program = project::<1>(&source);
    let (bus, _, _) = Bus::open([], []);
    let mut slab = [0; 8192];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut sender = rv.enter(SessionId::new(17), &program)?;
    sender.send::<Message>(&42).await?;
    bus.observe(json!({"type":"observation"}));
    assert!(bus.transport.take_observations().next().is_none());
    let pending = bus
        .connection(|connection| connection.outgoing.pop_front())
        .unwrap();
    assert!(pending.1.is_none());
    let records = serde_json::Deserializer::from_slice(&pending.0)
        .into_iter::<Value>()
        .map(|value| value.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["type"], "protocol_frame");
    assert_eq!(records[0]["action"], "send");
    assert_eq!(records[1], json!({"type":"observation"}));
    Ok(())
}

#[test]
fn an_unpolled_effect_creates_no_rpc_or_correlation() {
    let (mut connection, _, _) = Bus::open([], []);
    let (bus, simulator) = connection.split();
    drop(simulator.execute("A", "inspect", json!({"target":7})));
    assert!(bus.connection(|c| c.outgoing.is_empty() && c.responses.is_empty() && c.sequence == 0));
}

#[hibana_runtime::main]
#[test]
async fn losing_a_borrowed_effect_race_preserves_the_original_rpc() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let mut effect = bus.rpc("A", "inspect", json!({"target":7}));
    let result = hibana_runtime::race(
        async { (&mut effect).await.map(Some).map_err(ShellError::message) },
        async { Ok(None) },
    )
    .await?;
    assert!(result.is_none());
    let requests = bus.connection(|c| c.outgoing.drain(..).collect::<Vec<_>>());
    assert_eq!(requests.len(), 1);
    let request = serde_json::Deserializer::from_slice(&requests[0].0)
        .into_iter::<Value>()
        .last()
        .unwrap()
        .unwrap();
    let id = request["id"].as_u64().unwrap();
    bus.respond(id, json!({"actual":"original result"}))?;
    assert_eq!(
        effect.await.map_err(ShellError::message)?,
        json!({"actual":"original result"})
    );
    assert!(bus.connection(|c| c.responses.is_empty() && c.sequence == 1));
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn native_world_peer_uses_the_actual_rpc_queue_and_receipts() -> ShellResult<()> {
    struct Echo;
    impl Domain for Echo {
        async fn execute(
            &self,
            agent: &'static str,
            tool: &'static str,
            arguments: Value,
        ) -> Result<Value, String> {
            Ok(json!({"agent":agent,"tool":tool,"arguments":arguments}))
        }
    }
    let (mut connection, _, _) = Bus::open(["A", "B"], []);
    let (bus, simulator) = connection.split();
    async {
        let value = simulator
            .execute("A", "inspect", json!({"input":1}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(
            value,
            json!({"agent":"A","tool":"inspect","arguments":{"input":1}})
        );
    };
    async {
        let value = simulator
            .execute("B", "inspect", json!({"input":2}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(
            value,
            json!({"agent":"B","tool":"inspect","arguments":{"input":2}})
        );
    };
    async {
        bus.fixture_peer::<2>(&Echo, &["inspect"]).await?;
    };
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn cancelled_emit_still_flushes_capture_checkpoint_and_final_receipt() -> ShellResult<()> {
    for partial in [false, true] {
        let (bus, mut owner, _) = Bus::open([], []);
        let (reader, writer) = std::os::unix::net::UnixStream::pair()?;
        let mut input = Io::new(reader.into())?;
        let mut output = Io::new(writer.into())?;
        let first = json!({"type":"terminal", "stdout":"x".repeat(1024 * 1024)});
        let mut expected = serde_json::to_vec(&first).unwrap();
        expected.push(b'\n');

        hibana_runtime::race(bus.emit(first), async {
            if partial {
                hibana_runtime::race(owner.flush_output(&bus, &mut output), async { Ok(()) })
                    .await?;
            }
            Ok(())
        })
        .await?;
        if partial {
            assert!(owner.output_offset > 0);
            assert!(owner.output_offset < owner.output_bytes.len());
            assert!(owner.output_receipt.is_some());
        } else {
            assert_eq!(owner.output_offset, 0);
            assert_eq!(bus.connection(|connection| connection.outgoing.len()), 1);
        }
        assert!(bus.connection(|connection| connection.responses.is_empty()));

        for value in [
            json!({"type":"protocol_capture", "producer":0,"ordinal":7}),
            json!({"type":"protocol_checkpoint", "producer":0,"checkpoint":8}),
        ] {
            expected.extend(serde_json::to_vec(&value).unwrap());
            expected.push(b'\n');
            bus.observe(value);
        }
        let verdict = json!({"type":"run_result", "status":"failed"});
        expected.extend(serde_json::to_vec(&verdict).unwrap());
        expected.push(b'\n');
        let ((), (), received) = async {
            async {
                bus.emit(verdict).await?;
            };
            async {
                hibana_runtime::yield_now().await;
                owner.flush_output(&bus, &mut output).await?;
                drop(output);
            };
            async {
                let mut received = Vec::new();
                let mut bytes = [0; 4096];
                loop {
                    let count = input.read(&mut bytes).await?;
                    if count == 0 {
                        break;
                    }
                    received.extend_from_slice(&bytes[..count]);
                }
                Ok(received)
            };
        }
        .await?;
        assert_eq!(received, expected, "partial={partial}");
        assert!(bus.connection(|connection| connection.responses.is_empty()));
        assert!(bus.connection(|connection| connection.outgoing.is_empty()));
        assert!(owner.output_receipt.is_none());
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn cancelled_output_keeps_its_prefix_before_queued_checkpoint_and_verdict() -> ShellResult<()>
{
    let (bus, mut owner, _) = Bus::open([], []);
    let (reader, writer) = std::os::unix::net::UnixStream::pair()?;
    let mut input = Io::new(reader.into())?;
    let mut output = Io::new(writer.into())?;
    let first = json!({"type":"observation","text":"x".repeat(1024 * 1024)});
    let mut expected = serde_json::to_vec(&first).unwrap();
    expected.push(b'\n');
    bus.observe(first);
    hibana_runtime::race(owner.flush_output(&bus, &mut output), async { Ok(()) }).await?;
    assert!(owner.output_offset > 0);
    assert!(owner.output_offset < owner.output_bytes.len());
    for value in [
        json!({"type":"protocol_checkpoint","producer":0,"checkpoint":7}),
        json!({"type":"run_result","status":"complete"}),
    ] {
        expected.extend(serde_json::to_vec(&value).unwrap());
        expected.push(b'\n');
        bus.observe(value);
    }
    let ((), received) = async {
        async move {
            owner.flush_output(&bus, &mut output).await?;
            drop(output);
        };
        async {
            let mut received = Vec::new();
            let mut bytes = [0; 4096];
            loop {
                let count = input.read(&mut bytes).await?;
                if count == 0 {
                    break;
                }
                received.extend_from_slice(&bytes[..count]);
            }
            Ok(received)
        };
    }
    .await?;
    assert_eq!(received, expected);
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn failed_output_cannot_be_reopened_by_a_final_verdict() -> ShellResult<()> {
    let (bus, mut owner, _) = Bus::open([], []);
    let (reader, writer) = std::os::unix::net::UnixStream::pair()?;
    drop(reader);
    let mut output = Io::new(writer.into())?;
    bus.observe(json!({"type":"observation"}));
    assert!(owner.flush_output(&bus, &mut output).await.is_err());
    assert!(owner.output.is_none());
    assert!(
        owner
            .emit(&bus, json!({"type":"run_result"}))
            .await
            .is_err()
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn identical_concurrent_effect_arguments_keep_distinct_rpc_replies() -> ShellResult<()> {
    let (mut connection, _, _) = Bus::open([], []);
    let (bus, simulator) = connection.split();
    async {
        let value = simulator
            .execute("A", "inspect", json!({"target": 7}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(value, json!("first"));
    };
    async {
        let value = simulator
            .execute("A", "inspect", json!({"target": 7}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(value, json!("second"));
    };
    async {
        hibana_runtime::yield_now().await;
        let requests =
            bus.connection(|connection| connection.outgoing.drain(..).collect::<Vec<_>>());
        assert_eq!(requests.len(), 2);
        let first = serde_json::Deserializer::from_slice(&requests[0].0)
            .into_iter::<Value>()
            .last()
            .unwrap()
            .unwrap();
        let second = serde_json::Deserializer::from_slice(&requests[1].0)
            .into_iter::<Value>()
            .last()
            .unwrap()
            .unwrap();
        assert_eq!(first["arguments"], second["arguments"]);
        assert_ne!(first["id"], second["id"]);
        bus.respond(second["id"].as_u64().unwrap(), json!("second"))?;
        bus.respond(first["id"].as_u64().unwrap(), json!("first"))?;
    };
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn output_receipt_waits_for_actual_write_completion() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    async {
        bus.emit(json!({"type": "exit", "status": 0})).await?;
    };
    async {
        hibana_runtime::yield_now().await;
        let (values, flushed) = bus
            .connection(|connection| connection.outgoing.pop_front())
            .unwrap();
        let value = serde_json::Deserializer::from_slice(&values)
            .into_iter::<Value>()
            .last()
            .unwrap()
            .unwrap();
        assert_eq!(value["type"], "exit");
        bus.respond(flushed.unwrap(), Value::Null)?;
    };
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn cancelling_one_rpc_removes_only_its_own_receipt() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let cancelled = bus.reply()?;
    let id = cancelled.id;
    let retained = bus.reply()?;
    let retained_id = retained.id;
    hibana_runtime::race(cancelled.receive(), async { Ok(Value::Null) }).await?;
    assert!(bus.respond(id, json!("late")).is_err());
    bus.respond(retained_id, json!("live"))?;
    assert_eq!(retained.receive().await?, json!("live"));
    assert!(bus.connection(|connection| connection.responses.is_empty()));
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn duplicate_rpc_reply_cannot_replace_the_first_value() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let receipt = bus.reply()?;
    let id = receipt.id;
    bus.respond(id, json!("first"))?;
    assert!(bus.respond(id, json!("replacement")).is_err());
    assert_eq!(receipt.receive().await?, json!("first"));
    assert!(bus.respond(id, json!("replay")).is_err());
    Ok(())
}
