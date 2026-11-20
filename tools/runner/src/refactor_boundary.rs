//! Free-shell stimuli through five actual Wasm guests; no model calls.
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

use crate::qualification::REFACTOR_CASES as CASES;
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("refactor command output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing actual response: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty actual response")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current direct-file runtime, wash, source receipt and common world"]
async fn independent_and_interleaved_modules_preserve_current_publications() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("refactor evidence root")?);
    fs::create_dir_all(&evidence)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_REFACTOR_WASI_BINARY").ok_or("refactor runtime")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_REFACTOR_BUILD_RECEIPT").ok_or("refactor build receipt")?,
    );
    let selected = std::env::var("CHOREOFIX_REFACTOR_CASE")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?;
    if selected.is_some_and(|case| case >= CASES) {
        return Err("unknown refactor diagnostic case".into());
    }
    let mut reports = Vec::new();
    for case in (0..CASES).filter(|case| selected.is_none_or(|selected| selected == *case)) {
        let directory = evidence.join(format!(
            "refactor-wasm-{}-{case}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&directory)?;
        let archived = archive_build_receipt(&binary, &receipt, &directory)?;
        let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
        crate::projection::archive(&binary, &directory, "4M")?;
        save(
            &directory.join("case.json"),
            &json!({"task":"4M","depth":0,"case_index":case,"declared_cases":CASES,
            "model_calls":0,"scored_benchmark":false,"build":archived,"simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,
            "delay_multiplier":0.01,"runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
            "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/refactor_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/refactor_audit.rs"))?}),
        )?;
        let trace = Trace::create(&directory.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("4M", 0, 0.01, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, &module, &[], &directory, trace, simulator)?;
        let run = async {
            async {
                if case < 8 {
                    async {
                        async {
                            let order=if case&4==0 {["module_a","module_b"]} else {["module_b","module_a"]};
                            for module in order {
                                let reply=guest.command("ARCHITECT",format!("echo '{}' > /service/design_proposal/{module}; cat /service/design_proposal/{module}",json!({"title":format!("{module}_refactor")}))).await?;
                                assert_eq!(response(&reply)?["success"],true,"{reply}");
                            }
                            assert_eq!(guest.command("ARCHITECT","exit 0".into()).await?["exit_status"],0);
                            Result::Ok(())
                        };
                        async {
                            let early=guest.command("DEVELOPER_A","echo '{\"branch\":\"module_a\"}' > /service/merge_code".into()).await?;
                            assert!(early["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"early publication: {early}");
                            let wrong=guest.command("DEVELOPER_A","echo '{\"target\":\"module_b\"}' > /service/refactor_code/module_a".into()).await?;
                            assert!(wrong["stdout"].as_str().is_some_and(|s|s.contains("Invalid argument")),"substituted target: {wrong}");
                            let a=if case&1==0 {["module_a","SHARED_LIBRARY"]} else {["SHARED_LIBRARY","module_a"]};
                            let b=if case&2==0 {["module_b","SHARED_LIBRARY"]} else {["SHARED_LIBRARY","module_b"]};
                            let order=if case&4==0 {[("DEVELOPER_A",a[0]),("DEVELOPER_B",b[0]),("DEVELOPER_A",a[1]),("DEVELOPER_B",b[1])]}
                                else {[("DEVELOPER_B",b[0]),("DEVELOPER_A",a[0]),("DEVELOPER_B",b[1]),("DEVELOPER_A",a[1])]};
                            for (agent,target) in order {
                                let reply=guest.command(agent,format!("echo '{}' > /service/refactor_code/{target}; cat /service/refactor_code/{target}",json!({"target":target}))).await?;
                                assert_eq!(response(&reply)?["success"],true,"{reply}");
                            }
                            async {
                                async {
                            let order=if case&4==0 {[("DEVELOPER_A","module_a"),("DEVELOPER_B","module_b")]}
                                else {[("DEVELOPER_B","module_b"),("DEVELOPER_A","module_a")]};
                            for (agent,module) in order {
                                let wrong=guest.command(agent,"echo '{}' > /service/review".into()).await?;
                                assert!(wrong["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"approval overwrite: {wrong}");
                                for path in ["review","test_result"] {
                                    let reply=guest.command(agent,format!("cat /service/{path}")).await?;
                                    assert_eq!(response(&reply)?["success"],true,"{reply}");
                                }
                                let merged=guest.command(agent,format!("echo '{}' > /service/merge_code; cat /service/merge_code",json!({"branch":module}))).await?;
                                assert_eq!(response(&merged)?["success"],true,"{merged}");
                                assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                            }
                                    Result::Ok(())
                                };
                                async {
                                        let order=if (case&1==0)==(case&4==0) {["module_a","module_b"]} else {["module_b","module_a"]};
                                        for module in order {
                                            let reply=guest.command("REVIEWER",format!("echo '{}' > /service/code_review/{module}",json!({"pr_id":module}))).await?;
                                            assert!(!reply["stdout"].as_str().ok_or("review submit")?.contains("denied"),"{reply}");
                                        }
                                        for module in order {
                                            let reply=guest.command("REVIEWER",format!("cat /service/code_review/{module}")).await?;
                                            assert_eq!(response(&reply)?["success"],true,"{reply}");
                                        }
                                        assert_eq!(guest.command("REVIEWER","exit 0".into()).await?["exit_status"],0);
                                    Result::Ok(())
                                };
                                async {
                                        let order=if (case&2==0)==(case&4==0) {["module_a","module_b"]} else {["module_b","module_a"]};
                                        for module in order {
                                            let reply=guest.command("TESTER",format!("echo '{}' > /service/run_tests/{module}_tests",json!({"suite":format!("{module}_tests")}))).await?;
                                            assert!(!reply["stdout"].as_str().ok_or("test submit")?.contains("denied"),"{reply}");
                                        }
                                        for module in order {
                                            let reply=guest.command("TESTER",format!("cat /service/run_tests/{module}_tests")).await?;
                                            assert_eq!(response(&reply)?["success"],true,"{reply}");
                                        }
                                        assert_eq!(guest.command("TESTER","exit 0".into()).await?["exit_status"],0);
                                    Result::Ok(())
                                };
                                Ok(())
                            }.await?;
                            Result::Ok(())
                        };
                        Ok(())
                    }.await?;
                } else {
                    let modules = if case & 4 == 0 {
                        [("DEVELOPER_A", "module_a"), ("DEVELOPER_B", "module_b")]
                    } else {
                        [("DEVELOPER_B", "module_b"), ("DEVELOPER_A", "module_a")]
                    };
                    for (developer, module) in modules {
                        let design = guest.command("ARCHITECT", format!(
                            "echo '{}' > /service/design_proposal/{module}; cat /service/design_proposal/{module}",
                            json!({"title":format!("{module}_refactor")})
                        )).await?;
                        assert_eq!(response(&design)?["success"], true, "{design}");
                        let edits = if case & 1 == 0 {
                            [module, "SHARED_LIBRARY"]
                        } else {
                            ["SHARED_LIBRARY", module]
                        };
                        for target in edits {
                            let reply = guest.command(developer, format!(
                                "echo '{}' > /service/refactor_code/{target}; cat /service/refactor_code/{target}",
                                json!({"target":target})
                            )).await?;
                            assert_eq!(response(&reply)?["success"], true, "{reply}");
                        }
                        let review = (
                            "REVIEWER",
                            format!("code_review/{module}"),
                            json!({"pr_id":module}),
                        );
                        let test = (
                            "TESTER",
                            format!("run_tests/{module}_tests"),
                            json!({"suite":format!("{module}_tests")}),
                        );
                        let validations = if case & 2 == 0 {
                            [review, test]
                        } else {
                            [test, review]
                        };
                        for (agent, path, request) in validations {
                            let reply = guest
                                .command(
                                    agent,
                                    format!(
                                        "echo '{request}' > /service/{path}; cat /service/{path}"
                                    ),
                                )
                                .await?;
                            assert_eq!(response(&reply)?["success"], true, "{reply}");
                        }
                        for path in ["review", "test_result"] {
                            let reply = guest
                                .command(developer, format!("cat /service/{path}"))
                                .await?;
                            assert_eq!(response(&reply)?["success"], true, "{reply}");
                        }
                        let reply = guest
                            .command(
                                developer,
                                format!(
                                    "echo '{}' > /service/merge_code; cat /service/merge_code",
                                    json!({"branch":module})
                                ),
                            )
                            .await?;
                        assert_eq!(response(&reply)?["success"], true, "{reply}");
                        assert_eq!(
                            guest.command(developer, "exit 0".into()).await?["exit_status"],
                            0
                        );
                    }
                    for role in ["ARCHITECT", "REVIEWER", "TESTER"] {
                        assert_eq!(
                            guest.command(role, "exit 0".into()).await?["exit_status"],
                            0
                        );
                    }
                }
                Result::Ok(())
            };
            async { runtime.await };
        };
        let (_, runtime) = hibana_runtime::race(run, async {
            hibana_runtime::host::sleep(Duration::from_secs(180)).await?;
            Err("refactor actual-Wasm mechanism timeout".into())
        })
        .await?;
        assert_eq!(runtime["runtime_exit_code"], 0, "{runtime}");
        assert_eq!(runtime["protocol"]["status"], "complete", "{runtime}");
        save(&directory.join("runtime-result.json"), &runtime)?;
        let (audit, counterexamples) = if case == 0 {
            let (audit, report) = crate::refactor_audit::check_mutants(
                &directory.join("events.jsonl"),
                &directory.join("record-counterexamples.json"),
            )?;
            (audit, Some(report))
        } else {
            (
                crate::refactor_audit::audit(&directory.join("events.jsonl"))?,
                None,
            )
        };
        assert_eq!(audit["domain_calls"], 12);
        assert_eq!(audit["violations"], 0);
        save(&directory.join("boundary.json"), &audit)?;
        reports.push(json!({"case_index":case,"evidence":directory,"boundary_sha256":digest(&directory.join("boundary.json"))?,
            "runtime_binary_sha256":audit["runtime_binary_sha256"],"status":audit["status"],"domain_calls":audit["domain_calls"],"counterexamples":counterexamples}));
    }
    save(
        &evidence.join("refactor-mechanism.json"),
        &json!({"task":"4M","declared_cases":CASES,"cases":reports,
        "complete_finite_domain":selected.is_none(),"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
