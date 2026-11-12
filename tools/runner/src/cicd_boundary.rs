//! Scripted shell qualification through the actual CI/CD Wasm participants.
use crate::{
    cicd_audit,
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(value: &Value) -> Result<Value> {
    let output = value["stdout"].as_str().ok_or("terminal stdout")?;
    let start = output.find('{').ok_or("missing result JSON")?;
    serde_json::Deserializer::from_str(&output[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty result")?
        .map_err(Into::into)
}
async fn participant(guest: &process::Guest, actor: Value) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("participant")?;
    let call = &actor["calls"][0];
    let tool = call["tool"].as_str().ok_or("tool")?;
    let private = guest
        .command(
            agent,
            r#"
ls /
ls /missing || :
printf '%s' private-work > /work/notes
cat /work/notes
ls /
cat /work/notes
"#
            .into(),
        )
        .await?;
    let output = private["stdout"].as_str().ok_or("private stdout")?;
    assert_eq!(output.matches("private-work").count(), 2);
    assert!(!output.contains("Permission denied"));
    let empty = guest.command(agent, "cat /service/result".into()).await?;
    assert_eq!(empty["stdout"], "wash> ");
    let forged = guest
        .command(
            agent,
            "echo forged > /service/result; cat /service/result".into(),
        )
        .await?;
    assert!(forged["stdout"].as_str().unwrap().contains("wash:"));
    assert_eq!(
        guest.command(agent, "cat /service/result".into()).await?["stdout"],
        "wash> "
    );
    let unknown = guest.command(agent, "not_a_command".into()).await?;
    assert!(
        unknown["stdout"]
            .as_str()
            .unwrap()
            .contains("command not found")
    );
    if agent == "DEVELOPER" {
        let prepared = guest
            .command(
                agent,
                r#"
value='{"component":"application"}'
printf '%s\n' "$value" > /work/0
for word in alpha beta gamma; do printf '%s\n' "$word" >> /work/2; done
if cat /work/2 | grep beta; then echo found; else echo wrong; fi
cat < /work/2 | wc
false && echo wrong
cat /missing || echo recovered
n=0
while [ "$n" -lt 3 ]; do printf '%s\n' "$n"; expr "$n" + 1 | read n; done
"#
                .into(),
            )
            .await?;
        let output = prepared["stdout"].as_str().ok_or("script stdout")?;
        assert!(output.contains("beta\nfound\n17\n"), "{prepared}");
        assert!(output.contains("recovered\n0\n1\n2\n"));
        assert!(!output.contains("wrong"));
        guest
            .command(
                agent,
                "echo notes > /work/agent-notes.txt; echo approved > /work/grant".into(),
            )
            .await?;
        let listing = guest.command(agent, "ls /work".into()).await?;
        assert!(
            listing["stdout"]
                .as_str()
                .unwrap()
                .contains("agent-notes.txt\n")
        );
        assert!(listing["stdout"].as_str().unwrap().contains("grant\n"));
        let grown = guest.command(agent, format!("pad='{}'; n=0\nwhile [ \"$n\" -lt 100 ]; do printf '%s' \"$pad\"; expr \"$n\" + 1 | read n; done | wc", "x".repeat(512))).await?;
        assert!(grown["stdout"].as_str().unwrap().contains("51200\n"));
        let bounded = guest
            .command(agent, "while true; do :; done".into())
            .await?;
        assert!(
            bounded["stdout"]
                .as_str()
                .unwrap()
                .contains("budget exhausted")
        );
        assert!(
            guest.command(agent, "echo still-running".into()).await?["stdout"]
                .as_str()
                .unwrap()
                .contains("still-running")
        );
        let malformed = guest
            .command(
                agent,
                "cat /work/0 > /service/compile_and_test\nif true; then echo unfinished".into(),
            )
            .await?;
        assert!(malformed["stdout"].as_str().unwrap().contains("wash:"));
        guest
            .command(
                agent,
                r#"printf '%s\n' 'echo from-script' > /work/1"#.into(),
            )
            .await?;
        let sourced = guest.command(agent, ". /work/1".into()).await?;
        assert!(
            sourced["stdout"]
                .as_str()
                .unwrap()
                .contains("from-script\n")
        );
        let denied = guest
            .command(agent, "echo '{}' > /service/deploy_application".into())
            .await?;
        assert!(denied["stdout"].as_str().unwrap().contains("wash:"));
    } else {
        let private = guest.command(agent, "cat /work/0".into()).await?;
        assert!(private["stdout"].as_str().unwrap().contains("No such file"));
    }
    let rejected = guest
        .command(
            agent,
            format!("echo '{{}}' > /service/{tool}; cat /service/result"),
        )
        .await?;
    assert_eq!(response(&rejected)?["status"], "invalid_arguments");
    let result = if agent == "DEVELOPER" {
        guest
            .command(
                agent,
                "cp /work/0 /service/compile_and_test; cat /service/result".into(),
            )
            .await?
    } else {
        guest
            .command(
                agent,
                format!(
                    "printf '%s' '{}' > /service/{tool}; cat /service/result",
                    serde_json::to_string(&call["arguments"])?
                ),
            )
            .await?
    };
    assert_eq!(response(&result)?["success"], true);
    let duplicate = guest
        .command(
            agent,
            format!(
                "echo '{}' > /service/{tool}",
                serde_json::to_string(&call["arguments"])?
            ),
        )
        .await?;
    assert!(duplicate["stdout"].as_str().unwrap().contains("os error 8"));
    let exit = guest
        .command(agent, "cat /service/result; printf bye; exit 0".into())
        .await?;
    assert_eq!(exit["exit_status"], 0);
    assert!(exit["stdout"].as_str().unwrap().ends_with("bye"));
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified CI/CD binary, actual wash and unchanged upstream simulator"]
async fn real_wasi_cicd_uses_resource_endpoints_and_keeps_shell_freedom() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_WASI_BINARY").ok_or("CI/CD binary missing")?,
    );
    let receipt_path = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_BUILD_RECEIPT").ok_or("build receipt missing")?,
    );
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "cicd-resource-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let receipt = archive_build_receipt(&runtime, &receipt_path, &directory)?;
    let mut outcomes = Vec::new();
    for depth in 0..=2 {
        let output = directory.join(format!("depth-{depth}"));
        fs::create_dir(&output)?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) = oracle::Simulator::new("16E", depth, 0.01, trace.clone(), &[])?;
        let projection = crate::projection::archive(&runtime, &output, "16E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"16E","depth":depth,"model_calls":0,"runtime_binary_sha256":projection["runtime_binary_sha256"],"compiled_projection_sha256":digest(&output.join("projection.json"))?,"scored_benchmark":false,"build":receipt,"wasm_sha256":digest(wasm)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,"harness_source_sha256":digest(Path::new("tools/runner/src/cicd_boundary.rs"))?,"oracle_source_sha256":digest(Path::new("tools/runner/src/oracle.rs"))?,"audit_source_sha256":digest(Path::new("tools/runner/src/cicd_audit.rs"))?}),
        )?;
        let (guest, runtime_loop) =
            process::guest(&runtime, wasm, &["16E"], &output, trace, simulator.clone())?;
        let actors: [Value; 3] = fixture["fixtures"]
            .as_array()
            .ok_or("original participants")?
            .clone()
            .try_into()
            .map_err(|_| "CI/CD participant count")?;
        let result = hibana_runtime::race(
            async {
                let guest = &guest;

                for actor in actors {
                    async move {
                        participant(guest, actor).await?;
                    };
                }
                let result = runtime_loop.await?;
                Ok(result)
            },
            hibana_runtime::host::deadline(Duration::from_secs(90)),
        )
        .await?;
        assert_eq!(result["runtime_exit_code"], 0, "{result}");
        assert_eq!(result["protocol"]["status"], "complete");
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        assert_eq!(result["score"]["events"].as_array().unwrap().len(), 3);
        let audit = cicd_audit::audit(&output.join("events.jsonl"))?;
        let mutants =
            cicd_audit::check_mutants(&output.join("events.jsonl"), &output.join("mutants"))?;
        assert_eq!(mutants["mutants"].as_array().ok_or("mutants")?.len(), 16);
        save(&output.join("audit.json"), &audit)?;
        save(&output.join("result.json"), &result)?;
        outcomes.push(json!({"depth":depth,"runtime":result,"audit":audit,"mutants":mutants}));
        save(
            &directory.join("summary.json"),
            &json!({"cases":outcomes,"model_calls":0,"scored_benchmark":false}),
        )?;
    }
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the source-qualified CI/CD binary and actual wash; verifies rejection before any original effect"]
async fn real_wasi_cicd_early_exit_has_no_domain_effect() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_WASI_BINARY").ok_or("CI/CD binary missing")?,
    );
    let receipt_path = PathBuf::from(
        std::env::var_os("CHOREOFIX_CICD_BUILD_RECEIPT").ok_or("build receipt missing")?,
    );
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root missing")?);
    fs::create_dir_all(&root)?;
    let output = root.join(format!(
        "cicd-early-exit-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&output)?;
    let receipt = archive_build_receipt(&runtime, &receipt_path, &output)?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let (simulator, _) = oracle::Simulator::new("16E", 0, 0.01, trace.clone(), &[])?;
    save(
        &output.join("case.json"),
        &json!({"task":"16E","model_calls":0,"scored_benchmark":false,
        "expected":"guest exit rejected before any task effect","build":receipt,
        "harness_binary_sha256":digest(&std::env::current_exe()?)?,
        "harness_source_sha256":digest(Path::new("tools/runner/src/cicd_boundary.rs"))?,
        "oracle_source_sha256":digest(Path::new("tools/runner/src/oracle.rs"))?,"wasm_sha256":digest(module)?}),
    )?;
    let (guest, runtime_loop) = process::guest(
        &runtime,
        module,
        &["16E"],
        &output,
        trace,
        simulator.clone(),
    )?;
    let (command, result) = hibana_runtime::race(
        async {
            async { Ok(guest.command("DEVELOPER", "exit 0".into()).await) };
            async { (runtime_loop).await };
        },
        hibana_runtime::host::deadline(Duration::from_secs(10)),
    )
    .await?;
    assert!(command.is_err());
    assert_eq!(result["protocol"]["status"], "failed", "{result}");
    assert_eq!(result["runtime_exit_code"], 1);
    assert_eq!(result["score"]["complete"], false);
    assert_eq!(result["score"]["events"], json!([]));
    assert_eq!(result["score"]["violations"], json!([]));
    save(
        &output.join("result.json"),
        &json!({"runtime":result,"command_error":command.unwrap_err().to_string(),
        "model_calls":0,"scored_benchmark":false,"status":"expected_rejection"}),
    )?;
    println!("{}", output.display());
    Ok(())
}
