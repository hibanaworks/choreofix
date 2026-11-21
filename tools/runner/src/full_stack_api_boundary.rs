//! Exercise the actual 8M Wasm guests through the ordinary simulator and journal.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 8M runtime, build receipt and wash guest"]
async fn actual_wasm_delivers_repair_decisions_and_revalidates() -> Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(workspace)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("8M evidence root")?);
    fs::create_dir_all(&evidence)?;
    let output = evidence.join("full-stack-api-wasi");
    fs::create_dir(&output)?;
    let runtime = PathBuf::from(
        std::env::var_os("CHOREOFIX_FULL_STACK_API_WASI_BINARY").ok_or("8M executable")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_FULL_STACK_API_BUILD_RECEIPT").ok_or("8M build receipt")?,
    );
    let wasm = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let build = archive_build_receipt(&runtime, &receipt, &output)?;
    projection::archive(&runtime, &output, "8M")?;
    save(
        &output.join("case.json"),
        &json!({"task":"8M","depth":0,"model_calls":0,"scored_benchmark":false,"build":build,
        "runtime_binary_sha256":digest(&runtime)?,"guest_wasm_sha256":digest(wasm)?,
        "compiled_projection_sha256":digest(&output.join("projection.json"))?,
        "harness_binary_sha256":digest(&std::env::current_exe()?)?,
        "harness_source_sha256":digest(Path::new("tools/runner/src/full_stack_api_boundary.rs"))?,
        "audit_source_sha256":digest(Path::new("tools/runner/src/full_stack_api_audit.rs"))?}),
    )?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let (world, _) = oracle::Simulator::new(
        "8M",
        0,
        0.05,
        trace.clone(),
        &[
            ("code_review", 0),
            ("run_integration_tests", 0),
            ("run_integration_tests", 1),
            ("run_integration_tests", 2),
        ],
    )?;
    let (guest, finish) = process::guest(&runtime, wasm, &["8M"], &output, trace, world)?;

    let ((), result) = hibana_runtime::race(async {
        async {
            let response = |reply: Value| -> Result<Value> {
                let text = reply["stdout"].as_str().ok_or("missing wash output")?;
                let at = text.find('{').ok_or_else(|| format!("missing result: {reply}"))?;
                Ok(serde_json::Deserializer::from_str(&text[at..]).into_iter::<Value>()
                    .next().ok_or("empty wash result")??)
            };
            let migrate = r#"echo '{"migration_name":"feature_schema"}' > /service/migrate_schema; cat /service/migrate_schema"#;
            let implement = r#"echo '{"component":"api_endpoints"}' > /service/implement_backend; cat /service/implement_backend"#;
            let frontend = r#"echo '{"component":"user_interface"}' > /service/implement_frontend; cat /service/implement_frontend"#;
            let integration = r#"echo '{"suite":"full_stack_integration"}' > /service/run_integration_tests; cat /service/run_integration_tests"#;
            let (schema, api) = async {
                async { response(guest.command("DB_ADMIN", migrate.into()).await?) };
                async { response(guest.command("BACKEND_DEV", r#"echo '{"endpoint":"feature_api_spec","method":"POST"}' > /service/define_api; cat /service/define_api"#.into()).await?) };
            }.await?;
            assert_eq!(api["data"]["contract"]["method"], "POST");
            assert_eq!(response(guest.command("BACKEND_DEV", "cat /service/schema".into()).await?)?, schema);
            let rejected = guest.command("BACKEND_DEV", r#"echo '{"reason":" "}' > /service/request_migration"#.into()).await?;
            assert!(rejected["stdout"].as_str().is_some_and(|text| text.contains("Invalid argument")));
            let requested = guest.command("BACKEND_DEV", r#"echo '{"reason":"add the missing relationship"}' > /service/request_migration"#.into()).await?;
            assert_eq!(requested["stdout"], "wash> ");
            let need = response(guest.command("DB_ADMIN", "cat /service/schema_status".into()).await?)?;
            assert_eq!(need, json!({"reason":"add the missing relationship","schema":schema["data"]["revision"],"status":"migration_requested"}));
            let revised = response(guest.command("DB_ADMIN", migrate.into()).await?)?;
            assert_ne!(schema["data"]["revision"], revised["data"]["revision"]);
            assert_eq!(response(guest.command("BACKEND_DEV", "cat /service/schema".into()).await?)?, revised);
            let implementation = response(guest.command("BACKEND_DEV", implement.into()).await?)?;
            assert_eq!(implementation["data"]["inputs"]["schema"], revised["data"]["revision"]);
            assert_eq!(response(guest.command("DB_ADMIN", "cat /service/schema_status".into()).await?)?, implementation);
            let stale = guest.command("BACKEND_DEV", r#"echo '{"reason":"already implemented"}' > /service/request_migration"#.into()).await?;
            assert!(stale["stdout"].as_str().is_some_and(|text| text.contains("Permission denied")));

            let review_backend = r#"echo '{"pr_id":"backend_api"}' > /service/code_review/backend_api; cat /service/code_review/backend_api"#;
            let subject = response(guest.command("REVIEWER", "cat /service/backend_candidate".into()).await?)?;
            let rejected = response(guest.command("REVIEWER", review_backend.into()).await?)?;
            assert_eq!(rejected["data"]["inputs"], subject);
            assert_eq!(rejected["data"]["status"], "changes_requested");
            assert_eq!(response(guest.command("BACKEND_DEV", "cat /service/outcome".into()).await?)?, rejected);
            let revised = response(guest.command("BACKEND_DEV", implement.into()).await?)?;
            assert_ne!(revised["data"]["revision"], implementation["data"]["revision"]);

            let mut candidates = Vec::new();
            for (correction, next) in [("initial", Some("backend")), ("backend", Some("schema")), ("schema", Some("api")), ("api", None)] {
                if correction == "backend" {
                    response(guest.command("BACKEND_DEV", implement.into()).await?)?;
                }
                if correction == "schema" {
                    let new_schema = response(guest.command("DB_ADMIN", migrate.into()).await?)?;
                    assert_eq!(response(guest.command("BACKEND_DEV", "cat /service/schema".into()).await?)?, new_schema);
                    let code = response(guest.command("BACKEND_DEV", implement.into()).await?)?;
                    assert_eq!(code["data"]["inputs"]["schema"], new_schema["data"]["revision"]);
                }
                if correction == "api" {
                    let defined = response(guest.command("BACKEND_DEV", r#"echo '{"endpoint":"feature_api_spec","method":"PUT"}' > /service/define_api; cat /service/define_api"#.into()).await?)?;
                    assert_eq!(defined["data"]["contract"]["method"], "PUT");
                }
                let subject = response(guest.command("REVIEWER", "cat /service/backend_candidate".into()).await?)?;
                let reviewed = response(guest.command("REVIEWER", review_backend.into()).await?)?;
                assert_eq!(reviewed["data"]["inputs"], subject);
                assert_eq!(reviewed["success"], true);
                assert_eq!(response(guest.command("BACKEND_DEV", "cat /service/outcome".into()).await?)?, reviewed);
                if correction != "backend" {
                    assert_eq!(response(guest.command("FRONTEND_DEV", "cat /service/backend".into()).await?)?, reviewed);
                    let implemented = response(guest.command("FRONTEND_DEV", frontend.into()).await?)?;
                    let subject = response(guest.command("REVIEWER", "cat /service/frontend_candidate".into()).await?)?;
                    assert_eq!(subject["code"], implemented["data"]["revision"]);
                    let accepted = response(guest.command("REVIEWER", r#"echo '{"pr_id":"frontend_ui"}' > /service/code_review/frontend_ui; cat /service/code_review/frontend_ui"#.into()).await?)?;
                    assert_eq!(accepted["success"], true);
                    assert_eq!(response(guest.command("FRONTEND_DEV", "cat /service/outcome".into()).await?)?, accepted);
                }
                let candidate = response(guest.command("TESTER", "cat /service/test_candidate".into()).await?)?;
                let tested = response(guest.command("TESTER", integration.into()).await?)?;
                assert_eq!(candidate, tested["data"]["inputs"]);
                candidates.push(candidate);
                assert_eq!(response(guest.command("REVIEWER", "cat /service/outcome".into()).await?)?, tested);
                if let Some(next) = next {
                    assert_eq!(tested["success"], false);
                    let arguments = json!({"component":next});
                    let (db, backend, frontend, decision) = async {
                        async { response(guest.command("DB_ADMIN", "cat /service/outcome".into()).await?) };
                        async { response(guest.command("BACKEND_DEV", "cat /service/outcome".into()).await?) };
                        async { response(guest.command("FRONTEND_DEV", "cat /service/outcome".into()).await?) };
                        async { guest.command("REVIEWER", format!("echo '{}' > /service/repair", arguments)).await };
                    }.await?;
                    assert_eq!(decision["stdout"], "wash> ");
                    for notice in [db, backend, frontend] {
                        assert_eq!(notice, json!({"repair":arguments,"test":tested}));
                    }
                } else {
                    assert_eq!(tested["success"], true);
                    for who in ["DB_ADMIN","BACKEND_DEV","FRONTEND_DEV"] {
                        assert_eq!(response(guest.command(who, "cat /service/outcome".into()).await?)?, tested);
                    }
                }
            }
            assert_ne!(candidates[0]["candidate"]["backend"], candidates[1]["candidate"]["backend"]);
            assert_eq!(candidates[0]["frontend_review"], candidates[1]["frontend_review"]);
            assert_ne!(candidates[1]["candidate"]["schema"], candidates[2]["candidate"]["schema"]);
            assert_eq!(candidates[2]["candidate"]["backend"], candidates[3]["candidate"]["backend"]);
            assert_eq!(candidates[3]["candidate"]["api"]["method"], "PUT");
            for who in ["DB_ADMIN","BACKEND_DEV","FRONTEND_DEV","REVIEWER","TESTER"] {
                assert_eq!(guest.command(who, "exit 0".into()).await?["exit_status"], 0);
            }
            Ok(())
        };
        async { finish.await };
    }, async {
        hibana_runtime::host::sleep(Duration::from_secs(50)).await?;
        Err("8M Wasm mechanism timeout".into())
    }).await?;
    assert_eq!(result["error"], Value::Null);
    save(&output.join("runtime-result.json"), &result)?;
    let checked = crate::journal::Evidence::open(&output.join("events.jsonl"))?
        .audit(crate::full_stack_api_audit::check)?;
    assert_eq!(checked["status"], "passed");
    assert_eq!(
        checked["early_migration_requests"]
            .as_array()
            .ok_or("early request evidence")?
            .len(),
        1
    );
    save(&output.join("boundary.json"), &checked)?;
    Ok(())
}
