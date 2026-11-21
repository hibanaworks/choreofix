//! Actual wash guests and the common build world; scripted, with no model calls.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("build terminal output")?;
    let offset = text
        .find('{')
        .ok_or_else(|| format!("missing build reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[offset..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty build reply")?
        .map_err(Into::into)
}

async fn compile(
    guest: &process::Guest,
    (agent, module): (&str, &str),
    adversarial: bool,
) -> Result<Value> {
    if agent == "BUILDER_B" {
        let update = guest
            .command(agent, "dd if=/service/updates iflag=nonblock".into())
            .await?;
        if update["stdout"]
            .as_str()
            .ok_or("type update output")?
            .contains("wash:")
        {
            // Before A's actual result exists, the initial B request remains
            // legal and its old object will require a later real update.
            assert!(
                update["stdout"]
                    .as_str()
                    .ok_or("type update rejection")?
                    .contains("(os error 2)")
                    || update["stdout"]
                        .as_str()
                        .ok_or("nonblocking type update output")?
                        .contains("(os error 6)"),
                "unexpected update read failure: {update}"
            );
        } else {
            let update = response(&update)?;
            assert_eq!(update["success"], true);
            assert_eq!(update["data"]["agent"], "BUILDER_A");
        }
    }
    if adversarial {
        let bad = guest
            .command(
                agent,
                "echo '{\"module_name\":\"outside\"}' > /service/compile_module".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("rejected write output")?
                .contains("wash:"),
            "{bad}"
        );
        let private = guest
            .command(
                agent,
                "echo 'private calculation' > /work/notes; cat /work/notes".into(),
            )
            .await?;
        assert!(
            private["stdout"]
                .as_str()
                .ok_or("private output")?
                .contains("private calculation")
        );
    }
    let arguments = json!({"module_name":module});
    let reply = guest
        .command(
            agent,
            format!(
                "printf '%s' '{arguments}' > /service/compile_module; cat /service/compile_module"
            ),
        )
        .await?;
    let actual = response(&reply)?;
    assert_eq!(actual["tool_name"], "compile_module");
    assert_eq!(actual["success"], true, "{reply}");
    Ok(actual)
}

async fn builder(
    guest: &process::Guest,
    (agent, module, adversarial): (&str, &str, bool),
    mut actual: Value,
) -> Result<()> {
    loop {
        // Actual returned type contents distinguish B's old initial object
        // from a current build. There is no simulated participant-stage flag.
        if actual["data"]["object"]["inputs"]["SHARED_TYPES"]["modules"]["module_a"].is_string() {
            break;
        }
        assert_eq!(agent, "BUILDER_B", "A's build must produce the new types");
        actual = compile(guest, (agent, module), false).await?;
    }
    if adversarial {
        let args = json!({"module_name":module});
        let again = guest
            .command(agent, format!("echo '{args}' > /service/compile_module"))
            .await?;
        assert!(
            again["stdout"]
                .as_str()
                .ok_or("duplicate write output")?
                .contains("wash:"),
            "{again}"
        );
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

async fn integrator(guest: &process::Guest, adversarial: bool) -> Result<()> {
    if adversarial {
        let bad = guest
            .command(
                "INTEGRATOR",
                "echo '{\"modules\":\"module_a,module_a\"}' > /service/link_modules".into(),
            )
            .await?;
        assert!(
            bad["stdout"]
                .as_str()
                .ok_or("rejected link output")?
                .contains("wash:"),
            "{bad}"
        );
    }
    let reply = guest.command("INTEGRATOR", "echo '{\"modules\":\"module_b,module_a\"}' > /service/link_modules; cat /service/link_modules".into()).await?;
    let actual = response(&reply)?;
    assert_eq!(actual["success"], true, "{reply}");
    assert_eq!(actual["data"]["status"], "linked");
    if adversarial {
        let duplicate = guest
            .command(
                "INTEGRATOR",
                "echo '{\"modules\":\"module_a,module_b\"}' > /service/link_modules".into(),
            )
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate link output")?
                .contains("wash:"),
            "{duplicate}"
        );
    }
    assert_eq!(
        guest.command("INTEGRATOR", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified build runtime, actual wash and common world"]
async fn both_first_builders_rebuild_current_types_and_reject_invalid_or_duplicate_shell_effects()
-> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("build evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "build-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_BUILD_WASI_BINARY").ok_or("build binary")?)
            .canonicalize()?;
    let input =
        PathBuf::from(std::env::var_os("CHOREOFIX_BUILD_BUILD_RECEIPT").ok_or("build receipt")?);
    let receipt = archive_build_receipt(&binary, &input, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for (depth, first, adversarial) in (0..=2)
        .flat_map(|depth| ["BUILDER_A", "BUILDER_B"].map(move |first| (depth, first, false)))
        .chain([(0, "BUILDER_A", true), (0, "BUILDER_B", true)])
    {
        let output = directory.join(format!("case{}", outcomes.len()));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "10E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"10E","depth":depth,"initial_builder":first,
            "adversarial_inputs":adversarial,"model_calls":0,"scored_benchmark":false,
            "initial_order_control":"actual_shell_commands",
            "build":receipt,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,"test_barriers":"none"}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("10E", depth, 0.01, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["10E"], &output, trace, simulator)?;
        let ((), result) = hibana_runtime::race(
            async {
                async {
                    async { integrator(&guest, adversarial).await };
                    // Select the fixture's order with actual shell requests,
                    // never a runtime argument or resolver override. Both
                    // initial results exist before B considers a stale rebuild.
                    let (a, b) = if first == "BUILDER_A" {
                        let a = compile(&guest, ("BUILDER_A", "module_a"), adversarial).await?;
                        let b = compile(&guest, ("BUILDER_B", "module_b"), adversarial).await?;
                        (a, b)
                    } else {
                        let b = compile(&guest, ("BUILDER_B", "module_b"), adversarial).await?;
                        let a = compile(&guest, ("BUILDER_A", "module_a"), adversarial).await?;
                        (a, b)
                    };
                    async { builder(&guest, ("BUILDER_A", "module_a", adversarial), a).await };
                    async { builder(&guest, ("BUILDER_B", "module_b", adversarial), b).await };
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        assert_eq!(
            result["score"]["events"]
                .as_array()
                .ok_or("build effects")?
                .len(),
            if first == "BUILDER_A" { 3 } else { 4 }
        );
        let checked = crate::journal::Evidence::open(&output.join("events.jsonl"))?.audit("10E")?;
        save(&output.join("audit.json"), &checked)?;
        let mutants = crate::build_audit::check_mutants(
            &output.join("events.jsonl"),
            &output.join("record-mutants"),
        )?;
        outcomes.push(json!({"case":output,"task":"10E","depth":depth,"initial_builder":first,
            "initial_order_control":"actual_shell_commands",
            "adversarial_inputs":adversarial,"protocol_complete":true,"common_score_complete":true,
            "audit":checked,"record_mutants_rejected":mutants["checked"].as_array().ok_or("build mutants")?.len()}));
    }
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,
        "scored_benchmark":false,"cases":outcomes}),
    )?;
    Ok(())
}
