//! Actual Wasm calls, pre-admission errors and real CI/CD result messages.
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
async fn participant(guest: &process::Guest, actor: Value, adversarial: bool) -> Result<()> {
    let agent = actor["agent"].as_str().ok_or("participant")?;
    let call = &actor["calls"][0];
    let tool = call["tool"].as_str().ok_or("tool")?;
    let path = format!("/service/{tool}");
    let arguments = serde_json::to_string(&call["arguments"])?;
    let private = guest
        .command(
            agent,
            "printf private-work > /work/notes; cat /work/notes".into(),
        )
        .await?;
    assert!(
        private["stdout"]
            .as_str()
            .ok_or("private stdout")?
            .contains("private-work")
    );
    if adversarial {
        let unknown = guest
            .command(agent, "echo '{}' > /service/not_assigned".into())
            .await?;
        assert!(
            unknown["stdout"]
                .as_str()
                .ok_or("unknown-file error")?
                .contains("wash:")
        );
        if agent != "DEVELOPER" {
            // The dependency has not been received by this shell. This is an actual
            // early effect attempt, followed by another command in the same guest.
            let early = guest
                .command(agent, format!("printf '%s' '{arguments}' > {path}"))
                .await?;
            assert!(
                early["stdout"]
                    .as_str()
                    .ok_or("early error")?
                    .contains("Permission denied"),
                "{early}"
            );
        }
    }
    if agent != "DEVELOPER" {
        let input = response(&guest.command(agent, format!("cat {path}")).await?)?;
        assert_eq!(
            input["kind"],
            if agent == "TESTER" {
                "build_complete"
            } else {
                "tests_passed"
            }
        );
        assert_eq!(input["result"]["success"], true);
    }
    if adversarial {
        let malformed = guest
            .command(agent, format!("echo '{{}}' > {path}"))
            .await?;
        assert!(
            malformed["stdout"]
                .as_str()
                .ok_or("argument error")?
                .contains("os error 28"),
            "{malformed}"
        );
        let key = call["arguments"]
            .as_object()
            .ok_or("assigned argument")?
            .keys()
            .next()
            .ok_or("argument key")?;
        let wrong = guest
            .command(
                agent,
                format!("echo '{{\"{key}\":\"another_target\"}}' > {path}"),
            )
            .await?;
        assert!(
            wrong["stdout"]
                .as_str()
                .ok_or("target error")?
                .contains("os error 28"),
            "{wrong}"
        );
    }
    let padding = if adversarial {
        " ".repeat(192)
    } else {
        String::new()
    };
    let result = response(
        &guest
            .command(
                agent,
                format!("printf '%s' '{arguments}{padding}' > {path}; cat {path}"),
            )
            .await?,
    )?;
    assert_eq!(
        result["kind"],
        match agent {
            "DEVELOPER" => "compile_result",
            "TESTER" => "test_result",
            _ => "deploy_result",
        }
    );
    assert_eq!(result["result"]["success"], true, "{result}");
    if adversarial {
        let duplicate = guest
            .command(agent, format!("printf '%s' '{arguments}' > {path}"))
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate error")?
                .contains("Permission denied"),
            "{duplicate}"
        );
    }
    if agent != "DEPLOYER" {
        let deployed = response(&guest.command(agent, format!("cat {path}")).await?)?;
        assert_eq!(deployed["kind"], "deployment_complete");
        assert_eq!(deployed["result"]["success"], true);
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current CI/CD binary, receipt, actual wash and common simulator"]
async fn adversarial_depth_0() -> Result<()> {
    run(0, 0.01, true).await
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current CI/CD binary, receipt, actual wash and common simulator"]
async fn ordinary_depth_1() -> Result<()> {
    run(1, 0.01, false).await
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current CI/CD binary, receipt, actual wash and common simulator"]
async fn normal_delay_depth_2() -> Result<()> {
    run(2, 1.0, false).await
}

#[hibana_runtime::scope]
async fn run(depth: u8, delay: f64, adversarial: bool) -> Result<()> {
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
    let output = root.join(format!(
        "cicd-direct-{}-{}",
        depth,
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&output)?;
    let receipt = archive_build_receipt(&runtime, &receipt_path, &output)?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let (simulator, fixture) = oracle::Simulator::new("16E", depth, delay, trace.clone(), &[])?;
    let projection = crate::projection::archive(&runtime, &output, "16E")?;
    save(
        &output.join("case.json"),
        &json!({"task":"16E","depth":depth,"delay_multiplier":delay,
 "adversarial_inputs":adversarial,"model_calls":0,"scored_benchmark":false,"build":receipt,
 "runtime_binary_sha256":projection["runtime_binary_sha256"],
 "compiled_projection_sha256":digest(&output.join("projection.json"))?,
 "wasm_sha256":digest(wasm)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,
 "harness_source_sha256":digest(Path::new("tools/runner/src/cicd_boundary.rs"))?,
 "oracle_source_sha256":digest(Path::new("tools/runner/src/oracle.rs"))?,
 "audit_source_sha256":digest(Path::new("tools/runner/src/cicd_audit.rs"))?}),
    )?;
    let (guest, runtime_loop) =
        process::guest(&runtime, wasm, &["16E"], &output, trace, simulator)?;
    let actors: [Value; 3] = fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?
        .clone()
        .try_into()
        .map_err(|_| "CI/CD participants")?;
    let result = hibana_runtime::race(
        async {
            let guest = &guest;
            for actor in actors {
                async move {
                    participant(guest, actor, adversarial).await?;
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
    assert_eq!(
        result["score"]["events"]
            .as_array()
            .ok_or("actual effects")?
            .len(),
        3
    );
    save(&output.join("result.json"), &result)?;
    let audit = crate::journal::Evidence::open(&output.join("events.jsonl"))?
        .audit(crate::cicd_audit::check)?;
    save(&output.join("audit.json"), &audit)?;
    let mutants = cicd_audit::check_mutants(&output.join("events.jsonl"), &output.join("mutants"))?;
    assert_eq!(
        mutants["mutants"]
            .as_array()
            .ok_or("record counterexamples")?
            .len(),
        16
    );
    save(
        &output.join("summary.json"),
        &json!({"status":"passed","task":"16E","depth":depth,"delay_multiplier":delay,"adversarial_inputs":adversarial,"audit":audit,"mutants":mutants,"model_calls":0,"scored_benchmark":false}),
    )?;
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
