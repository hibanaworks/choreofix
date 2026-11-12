use super::*;

#[cfg(miri)]
#[path = "../../../tasks/handoff/tests/clock.rs"]
mod clock;

#[test]
fn an_unpolled_effect_creates_no_rpc_or_correlation() {
    let (bus, _, _) = Bus::open([], []);
    drop(bus.execute("A", "inspect", json!({"target":7})));
    assert!(bus.connection(|c| c.outgoing.is_empty() && c.responses.is_empty() && c.sequence == 0));
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn losing_a_borrowed_effect_race_preserves_the_original_rpc() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let mut effect = bus.execute("A", "inspect", json!({"target":7}));
    let result = hibana_runtime::race(
        async { (&mut effect).await.map(Some).map_err(ShellError::message) },
        async { Ok(None) },
    )
    .await?;
    assert!(result.is_none());
    let requests = bus.connection(|c| c.outgoing.drain(..).collect::<Vec<_>>());
    assert_eq!(requests.len(), 1);
    let id = requests[0].0.last().unwrap()["id"].as_u64().unwrap();
    bus.respond(id, json!({"actual":"original result"}))?;
    assert_eq!(
        effect.await.map_err(ShellError::message)?,
        json!({"actual":"original result"})
    );
    assert!(bus.connection(|c| c.responses.is_empty() && c.sequence == 1));
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
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
    let (bus, _, _) = Bus::open(["A", "B"], []);
    async {
        let value = bus
            .execute("A", "inspect", json!({"input":1}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(
            value,
            json!({"agent":"A","tool":"inspect","arguments":{"input":1}})
        );
    };
    async {
        let value = bus
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
    let (bus, _, _) = Bus::open([], []);
    async {
        let value = bus
            .execute("A", "inspect", json!({"target": 7}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(value, json!("first"));
    };
    async {
        let value = bus
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
        let first = requests[0].0.last().unwrap();
        let second = requests[1].0.last().unwrap();
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
        assert_eq!(values.last().unwrap()["type"], "exit");
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
