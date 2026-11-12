//! Correlate shell commands and already-admitted effects over child stdio.
use crate::{
    evidence::{Result, Trace},
    oracle::Simulator,
    wire::{self, Lines, Reply},
};
use hibana_runtime::{
    host::{Io, Notification, Process, notification},
    race,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
};

type Request = (Value, Reply);

pub async fn simulator(
    task: &str,
    depth: u8,
    output: &Path,
    trace: Trace,
) -> Result<(Simulator, Value)> {
    let config: Value = serde_json::from_slice(&std::fs::read("experiment.json")?)?;
    if config["python_hash_seed"] != 0 {
        return Err("the recorded upstream interpreter seed must be zero".into());
    }
    let delay = config["domain_tool_delay_multiplier"]
        .as_f64()
        .ok_or("tool delay multiplier")?;
    let result = Simulator::new(task, depth, delay, trace, &[])?;
    crate::evidence::save(&output.join("simulator-fixture.json"), &result.1)?;
    Ok(result)
}

pub struct Guest {
    input: mpsc::Sender<Request>,
    notify: Notification,
}

impl Guest {
    #[cfg(test)]
    pub(crate) fn test_input() -> Result<(Self, mpsc::Receiver<Request>, Io)> {
        let (input, receive) = mpsc::channel();
        let (notify, ready) = notification()?;
        Ok((Self { input, notify }, receive, ready))
    }
    pub(crate) fn send(&self, message: Value, reply: Reply) -> Result<()> {
        self.input
            .send((message, reply))
            .map_err(|_| "guest runtime closed")?;
        self.notify.notify()?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn command(&self, agent: &str, command: String) -> Result<Value> {
        let (send, receive) = mpsc::channel();
        let (notify, mut ready) = notification()?;
        self.send(
            json!({"agent":agent,"command":command,"id":null}),
            (send, notify),
        )?;
        let (id, result) = wire::receive(&receive, &mut ready)
            .await?
            .ok_or("guest command direction closed")?;
        if !id.is_null() {
            return Err("guest command reply id mismatch".into());
        }
        result
    }
}

pub fn guest(
    runtime: &Path,
    module: &Path,
    arguments: &[&str],
    output: &Path,
    trace: Trace,
    simulator: Simulator,
) -> Result<(Guest, impl Future<Output = Result<Value>> + use<>)> {
    let mut child = Process::spawn(
        Command::new(runtime)
            .arg(module)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create_new(output.join("runtime-stderr.log"))?),
    )?;
    let (input, output) = child.io()?;
    let (send, receive) = mpsc::channel();
    let (notify, ready) = notification()?;
    Ok((
        Guest {
            input: send,
            notify,
        },
        runtime_loop(
            child,
            input,
            Lines::new(output),
            receive,
            ready,
            trace,
            simulator,
        ),
    ))
}

enum Input {
    Command(Request),
    Frame(Value),
    Effect(Value, Result<Value>),
}

async fn runtime_loop(
    mut child: Process,
    mut input: Io,
    mut lines: Lines,
    commands: mpsc::Receiver<Request>,
    mut commands_ready: Io,
    trace: Trace,
    simulator: Simulator,
) -> Result<Value> {
    let (effects, results) = mpsc::channel();
    let (notify, mut effects_ready) = notification()?;
    let mut pending = BTreeMap::new();
    let mut admitted = BTreeSet::new();
    let mut backend_errors = Vec::new();
    let verdict: Result<Value> = async {
        let mut sequence = 0u64;
        loop {
            // These are physical input directions. The task's Endpoint operations
            // run in the child; this owner only transfers their actual bytes.
            let event = race(async {
                let line = lines.next_line().await?.ok_or("guest runtime closed before verdict")?;
                Ok::<Input, Box<dyn std::error::Error + Send + Sync>>(Input::Frame(serde_json::from_str(&line)?))
            }, async {
                race(async {
                    match wire::receive(&commands, &mut commands_ready).await? {
                        Some(command) => Ok(Input::Command(command)),
                        None => core::future::pending().await,
                    }
                }, async {
                    let (id, result) = wire::receive(&results, &mut effects_ready).await?.ok_or("effect response direction closed")?;
                    Ok(Input::Effect(id, result))
                }).await
            }).await?;
            match event {
                Input::Command((mut command, response)) => {
                    sequence = sequence.checked_add(1).ok_or("guest command id overflow")?;
                    let caller_id = command["id"].take();
                    command["type"] = json!("command");
                    command["id"] = json!(sequence);
                    pending.insert(sequence, (caller_id, response));
                    trace.record("to_runtime", &command)?;
                    wire::send(&mut input, &command).await?;
                }
                Input::Frame(message) => {
                    trace.record("runtime", &message)?;
                    match message["type"].as_str() {
                        Some("protocol_frame" | "compiled_projection" | "protocol_capture" | "protocol_checkpoint") => {},
                        Some("execute") => {
                            let id = message["id"].as_u64().ok_or("effect request id")?;
                            if admitted.contains(&id) { return Err("duplicate effect request id".into()); }
                            simulator.send(message, (effects.clone(), notify.try_clone()?))?;
                            admitted.insert(id);
                        }
                        Some("terminal" | "exit" | "terminal_closed") => {
                            if message["id"].is_null() { continue; }
                            let id = message["id"].as_u64().ok_or("terminal reply id")?;
                            let response = match message["type"].as_str() {
                                Some("terminal") => json!({"stdout":message["stdout"]}),
                                Some("exit") => json!({"stdout":message["stdout"],"exit_status":message["status"]}),
                                _ => json!({"status":"guest_closed"}),
                            };
                            let (id, reply) = pending.remove(&id).ok_or("unknown terminal reply id")?;
                            wire::respond(&reply, id, Ok(response))?;
                        }
                        Some("run_result") => break Ok(message),
                        _ => return Err(format!("unknown guest event: {message}").into()),
                    }
                }
                Input::Effect(id, result) => {
                    let id = id.as_u64().ok_or("effect reply id")?;
                    if !admitted.remove(&id) { return Err("unknown or duplicate effect reply".into()); }
                    let response = match result {
                        Ok(response) if response["id"] == id => response,
                        Ok(_) => return Err("effect reply id mismatch".into()),
                        Err(error) => {
                            backend_errors.push(error.to_string());
                            return Err(error);
                        }
                    };
                    wire::send(&mut input, &response).await?;
                }
            }
        }
    }.await;
    // A model driver retains its shared reply producer. Dropping these
    // per-command producers alone would leave their callers waiting forever.
    for (id, reply) in pending.into_values().chain(
        commands
            .try_iter()
            .map(|(mut command, reply)| (command["id"].take(), reply)),
    ) {
        if let Err(error) = wire::respond(
            &reply,
            id,
            Err("guest runtime closed before command reply".into()),
        ) {
            backend_errors.push(error.to_string());
        }
    }
    drop(commands);
    drop(input);
    if !verdict.as_ref().is_ok_and(|v| v["status"] == "complete") {
        child.terminate()?;
    }
    let exit = child.wait().await?;
    // Observe the response for each actual accepted RPC before scoring, even
    // if the guest has exited. Python may retain completed callback objects;
    // their destruction is not evidence that an effect finished.
    while !admitted.is_empty() {
        let (id, result) = wire::receive(&results, &mut effects_ready)
            .await?
            .ok_or("missing accepted effect replies")?;
        if !id.as_u64().is_some_and(|id| admitted.remove(&id)) {
            backend_errors.push("unknown or duplicate drained effect reply".to_owned());
        }
        if let Err(error) = result {
            backend_errors.push(error.to_string());
        }
    }
    drop(effects);
    drop(notify);
    let score = simulator.call(json!({"type":"score"})).await?;
    Ok(
        json!({"protocol":verdict.as_ref().ok(),"error":verdict.err().map(|e|e.to_string()),
        "runtime_exit_code":exit.code(),"score":score,"backend_errors":backend_errors}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[hibana_runtime::main]
    #[test]
    #[ignore = "isolated Python world and child stdio; explicitly run by ordinary CI"]
    async fn runtime_shutdown_completes_pending_and_queued_shared_channel_replies() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?;
        std::env::set_current_dir(&root)?;
        let output = root.join("output/experiments").join(format!(
            "runtime-close-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&output)?;
        for (name, response) in [
            (
                "verdict",
                "printf '%s\\n' '{\"type\":\"run_result\",\"status\":\"failed\",\"error\":\"fixture closed before terminal reply\"}'\n",
            ),
            ("eof", ""),
        ] {
            let directory = output.join(name);
            std::fs::create_dir(&directory)?;
            let script = directory.join("runtime.sh");
            std::fs::write(&script, format!("IFS= read -r request\n{response}exit 1\n"))?;
            let trace = Trace::create(&directory.join("events.jsonl"))?;
            let (world, _) = Simulator::new("9E", 0, 0.0, trace.clone(), &[])?;
            let (guest, runtime) =
                guest(Path::new("/bin/sh"), &script, &[], &directory, trace, world)?;
            let (send, receive) = mpsc::channel();
            let (notify, mut ready) = notification()?;
            // The model driver keeps a shared response producer alive. Merely
            // dropping one request's producer cannot complete its pending RPC.
            for id in [41, 42] {
                guest.send(
                    json!({"id":id,"agent":"PHILOSOPHER_1","command":"exit 0"}),
                    (send.clone(), notify.try_clone()?),
                )?;
            }
            race(
                async {
                    let (result, mut ids) = async {
                        async { runtime.await };
                        async {
                            let mut ids = Vec::new();
                            for _ in 0..2 {
                                let (id, result) = wire::receive(&receive, &mut ready)
                                    .await?
                                    .ok_or("shared response channel closed")?;
                                ids.push(id.as_u64().ok_or("caller id")?);
                                assert!(result.is_err(), "an interrupted command cannot succeed");
                            }
                            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(ids)
                        };
                    }
                    .await?;
                    ids.sort_unstable();
                    assert_eq!(ids, [41, 42]);
                    assert_eq!(result["runtime_exit_code"], 1);
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                },
                hibana_runtime::host::deadline(std::time::Duration::from_secs(2)),
            )
            .await?;
            drop(send);
            drop(notify);
            drop(guest);
        }
        std::fs::remove_dir_all(output)?;
        Ok(())
    }

    #[hibana_runtime::main]
    #[test]
    #[ignore = "requires the built writing runtime, wash module and original simulator environment"]
    async fn original_simulator_overlaps_disjoint_actual_wasi_effects() -> Result<()> {
        std::env::set_current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()?,
        )?;
        let output = Path::new("output").join(format!(
            "writing-rust-runner-probe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir_all(output.parent().ok_or("probe output parent")?)?;
        std::fs::create_dir(&output)?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) = simulator("2M", 2, &output, trace.clone()).await?;
        let runtime = std::env::var_os("CHOREOFIX_WRITING_RUNTIME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "target/debug/writing-wasi".into());
        crate::evidence::save(
            &output.join("case.json"),
            &json!({
                "task":"2M","depth":2,"model_calls":0,"scored_benchmark":false,
                "kind":"rust_upstream_api_and_existing_wasi_process_probe",
                "runtime_binary_sha256":crate::evidence::digest(&runtime)?,
                "guest_wasm_sha256":crate::evidence::digest(Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"))?,
                "runtime_source_scope":"existing binary; this bridge test does not establish its Hibana source pin"
            }),
        )?;
        let (guest, guest_loop) = guest(
            &runtime,
            Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
            &["2M"],
            &output,
            trace,
            simulator,
        )?;
        let actors: [Value; 5] = fixture["fixtures"]
            .as_array()
            .ok_or("fixtures")?
            .clone()
            .try_into()
            .map_err(|_| "writing actor count")?;
        let result = hibana_runtime::race(
            async {
                let guest = &guest;

                for actor in actors {
                    async move {
                        let name = actor["agent"].as_str().ok_or("actor")?;
                        for call in actor["calls"].as_array().ok_or("calls")? {
                            let arguments = serde_json::to_string(&call["arguments"])?;
                            let quote = format!("'{}'", arguments.replace('\'', "'\"'\"'"));
                            let response = guest
                                .command(
                                    name,
                                    format!(
                                        "printf '%s' {quote} > /service/{}; cat /service/result",
                                        call["tool"].as_str().ok_or("tool")?
                                    ),
                                )
                                .await?;
                            if !response["stdout"]
                                .as_str()
                                .ok_or("stdout")?
                                .contains("\"success\":true")
                            {
                                return Err(format!("probe operation failed: {response}").into());
                            }
                        }
                        let exit = guest.command(name, "exit 0".into()).await?;
                        if exit["exit_status"] != 0 {
                            return Err("probe guest exit failed".into());
                        }
                    };
                }
                let result = guest_loop.await?;
                Ok(result)
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(120)),
        )
        .await?;
        if result["protocol"]["status"] != "complete" || result["score"]["complete"] != true {
            return Err(format!("probe failed: {result}").into());
        }
        let audit = crate::writing_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(audit["max_disjoint_writes"], 2);
        crate::evidence::save(
            &output.join("probe.json"),
            &json!({"kind":"deterministic_rust_process_transport_probe","model_calls":0,"scored_benchmark":false,"audit":audit,"result":result}),
        )?;
        println!("{}", output.display());
        Ok(())
    }
}
