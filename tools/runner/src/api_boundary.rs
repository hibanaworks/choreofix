//! Scripted checks of result-file I/O through the actual wash guests.
use crate::{
    evidence::{Result, Trace, digest, save},
    oracle, process, snapshot,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let stdout = reply["stdout"].as_str().ok_or("terminal stdout")?;
    let start = stdout.find('{').ok_or("result JSON missing")?;
    serde_json::Deserializer::from_str(&stdout[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty result")?
        .map_err(Into::into)
}

async fn participant(guest: &process::Guest, actor: Value, depth: u8) -> Result<Value> {
    let agent = actor["agent"].as_str().ok_or("original participant")?;
    if agent != "TESTER" {
        let early = guest
            .command(agent, "cd /service/outcome || echo metadata-before".into())
            .await?;
        let stdout = early["stdout"].as_str().ok_or("early metadata stdout")?;
        assert!(
            stdout.contains("not a directory") && stdout.contains("metadata-before"),
            "{early}"
        );
    }
    let call = &actor["calls"][0];
    let tool = call["tool"].as_str().ok_or("original tool")?;
    let invalid = guest
        .command(
            agent,
            format!("echo '{{}}' > /service/{tool}; cat /service/result"),
        )
        .await?;
    assert_eq!(response(&invalid)?["status"], "invalid_arguments");
    let arguments = serde_json::to_string(&call["arguments"])?;
    let quoted = format!(
        "'{}'",
        format!("{}{}", " ".repeat(512), arguments).replace('\'', "'\"'\"'")
    );
    let result = guest
        .command(
            agent,
            format!("printf '%s' {quoted} > /service/{tool}; cat /service/result"),
        )
        .await?;
    let result = response(&result)?;
    assert_eq!(result["success"], agent != "TESTER" || depth == 0);
    let outcome = if agent != "TESTER" {
        let read = guest.command(agent, "cat /service/outcome".into()).await?;
        let first = response(&read)?;
        let again = guest
            .command(
                agent,
                "cd /service/outcome || echo metadata-after; cat /service/outcome".into(),
            )
            .await?;
        let stdout = again["stdout"].as_str().ok_or("late metadata stdout")?;
        assert!(stdout.contains("not a directory") && stdout.contains("metadata-after"));
        assert_eq!(response(&again)?, first);
        Some(first)
    } else {
        None
    };
    let exit = guest.command(agent, "exit 0".into()).await?;
    assert_eq!(exit["exit_status"], 0);
    Ok(json!({"agent":agent,"operation":result,"outcome":outcome}))
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified API binary and actual wash module"]
async fn real_wasi_outcome_metadata_preserves_all_original_results() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "api-outcome-owner-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let mut cases = Vec::new();
    for depth in 0..=2 {
        let output = directory.join(format!("depth-{depth}"));
        fs::create_dir(&output)?;
        let versions = snapshot::freeze("8E", &output)?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) = oracle::Simulator::new("8E", depth, 0.01, trace.clone(), &[])?;
        save(
            &output.join("case.json"),
            &json!({"task":"8E","depth":depth,
            "scope":"resource Endpoint delegation, actual requests and results, single-owner outcome files",
            "model_calls":0,"scored_benchmark":false,"versions":versions,
            "test_barriers":"both outcome opens precede the tester command"}),
        )?;
        let (guest, runtime_loop) = process::guest(
            &output.join("runtime").canonicalize()?,
            &output.join("wash.wasm").canonicalize()?,
            &["8E"],
            &output,
            trace.clone(),
            simulator.clone(),
        )?;
        let actors = fixture["fixtures"]
            .as_array()
            .ok_or("original participants")?;
        assert_eq!(
            actors
                .iter()
                .filter(|actor| actor["agent"] == "TESTER")
                .count(),
            1
        );
        let tester = actors
            .iter()
            .find(|actor| actor["agent"] == "TESTER")
            .ok_or("tester")?;
        let [first, second]: [&Value; 2] = actors
            .iter()
            .filter(|actor| actor["agent"] != "TESTER")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "API worker count")?;
        let (runtime_result, participants) = hibana_runtime::race(
            async {
                let (first, second, runtime_result, tester) = (async {
                    async { (participant(&guest, first.clone(), depth)).await };
                    async { (participant(&guest, second.clone(), depth)).await };
                    async { (runtime_loop).await };
                    async {
                        // Hold only the scripted tester input. Both actual WASI reads
                        // must wait at the file owner until the test result exists.
                        hibana_runtime::race(async {
                    loop {
                        let bytes = fs::read(output.join("events.jsonl"))?;
                        let rows = bytes
                            .split_inclusive(|&byte| byte == b'\n')
                            .filter(|row| row.last() == Some(&b'\n'))
                            .map(serde_json::from_slice::<Value>)
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        if [(13, 11), (14, 12)].iter().all(|&(vm, owner)| {
                            rows.iter().any(|row| {
                                let message = &row["message"];
                                message["type"] == "protocol_frame"
                                    && message["action"] == "send"
                                    && message["source"] == vm
                                    && message["target"] == owner
                            })
                        }) {
                            assert!(!rows.iter().any(|row| {
                                let message = &row["message"];
                                message["type"] == "execute"
                                    && message["tool"] == "run_integration_tests"
                            }));
                            for (owner, vm) in [(11, 13), (12, 14)] {
                                assert!(
                                    !rows.iter().any(|row| {
                                        let message = &row["message"];
                                        message["type"] == "protocol_frame"
                                            && message["action"] == "send"
                                            && message["source"] == owner
                                            && message["target"] == vm
                                    }),
                                    "outcome owner replied before the actual test result"
                                );
                            }
                            trace.record(
                                "test_control",
                                &json!({"type":"both_outcome_reads_pending","model_calls":0}),
                            )?;
                            break;
                        }
                        // Transport observations are flushed with terminal I/O.
                        // Private work also exercises another guest while both
                        // outcome reads wait for the withheld test operation.
                        let private = guest
                            .command("TESTER", "echo pending-read-observation".into())
                            .await?;
                        assert!(
                            private["stdout"]
                                .as_str()
                                .ok_or("private tester output")?
                                .contains("pending-read-observation")
                        );
                    }
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
                }, hibana_runtime::host::deadline(Duration::from_secs(10)))
                .await?;
                        participant(&guest, tester.clone(), depth).await
                    };
                })
                .await?;
                Result::Ok((runtime_result, [first, second, tester]))
            },
            hibana_runtime::host::deadline(Duration::from_secs(60)),
        )
        .await?;
        assert_eq!(runtime_result["runtime_exit_code"], 0, "{runtime_result}");
        assert_eq!(runtime_result["protocol"]["status"], "complete");
        assert_eq!(runtime_result["score"]["complete"], depth == 0);
        assert_eq!(runtime_result["score"]["violations"], json!([]));
        let tester = &participants
            .iter()
            .find(|reply| reply["agent"] == "TESTER")
            .ok_or("tester result")?["operation"];
        for reply in &participants {
            if reply["agent"] != "TESTER" {
                assert_eq!(&reply["outcome"], tester);
            }
        }
        let events = simulator.observed_events().await?;
        assert_eq!(events.len(), 3);
        assert_eq!(
            events
                .iter()
                .map(|event| event["tool"].as_str().expect("original tool"))
                .collect::<Vec<_>>(),
            [
                "migrate_schema",
                "implement_backend",
                "run_integration_tests"
            ]
        );
        let frames: Vec<Value> = fs::read_to_string(output.join("events.jsonl"))?
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        for (io, owner) in [(8, 11), (9, 12)] {
            let forwarded = frames
                .iter()
                .filter(|row| {
                    let message = &row["message"];
                    message["type"] == "protocol_frame"
                        && message["action"] == "send"
                        && message["source"] == io
                        && message["target"] == owner
                        && message["payload"]
                            .as_array()
                            .is_some_and(|bytes| bytes.len() == 6 + b"service/outcome".len())
                })
                .count();
            assert!(forwarded >= 2, "metadata did not reach file owner {owner}");
        }
        let audit = crate::api_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(audit["actual_resource_endpoint_requests"], 2);
        assert_eq!(audit["actual_resource_endpoint_returns"], 2);
        let mutants =
            crate::api_audit::check_mutants(&output.join("events.jsonl"), &output.join("mutants"))?;
        assert_eq!(mutants["mutants"].as_array().ok_or("mutants")?.len(), 16);
        save(&output.join("audit.json"), &audit)?;
        let result = json!({"task":"8E","depth":depth,"status":"passed","model_calls":0,
            "scored_benchmark":false,"participants":participants,"runtime":runtime_result,
            "audit":audit,"mutants":mutants,"events_sha256":digest(&output.join("events.jsonl"))?});
        save(&output.join("result.json"), &result)?;
        cases.push(result);
        save(
            &directory.join("summary.json"),
            &json!({"cases":cases,"scored_benchmark":false}),
        )?;
    }
    println!("{}", directory.display());
    Ok(())
}
