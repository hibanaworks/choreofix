//! Mechanism fixtures through five actual wash guests.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process, projection,
    qualification::LAB_CASCADING_CASES,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Batch {
    SynthesisFailed,
    QualityRejected,
    Approved,
    Exhausted,
}

struct Case {
    name: &'static str,
    depth: u8,
    analyst_first: bool,
    failures: &'static [(&'static str, u32)],
    batches: &'static [Batch],
    effects: usize,
}

use Batch::{Approved, Exhausted, QualityRejected, SynthesisFailed};

const CASES: [Case; LAB_CASCADING_CASES] = [
    Case {
        name: "analyst-first",
        depth: 0,
        analyst_first: true,
        failures: &[],
        batches: &[Approved],
        effects: 12,
    },
    Case {
        name: "qc-first",
        depth: 0,
        analyst_first: false,
        failures: &[],
        batches: &[Approved],
        effects: 12,
    },
    Case {
        name: "qc-first-stale-batch",
        depth: 0,
        analyst_first: false,
        failures: &[],
        batches: &[Approved],
        effects: 12,
    },
    Case {
        name: "qc-retry-1",
        depth: 0,
        analyst_first: false,
        failures: &[("verify_result", 0)],
        batches: &[QualityRejected, Approved],
        effects: 18,
    },
    Case {
        name: "qc-retry-2",
        depth: 0,
        analyst_first: true,
        failures: &[("verify_result", 0), ("verify_result", 1)],
        batches: &[QualityRejected, QualityRejected, Approved],
        effects: 28,
    },
    Case {
        name: "synthesis-retry-1",
        depth: 0,
        analyst_first: false,
        failures: &[("run_synthesis", 0)],
        batches: &[SynthesisFailed, Approved],
        effects: 13,
    },
    Case {
        name: "synthesis-retry-2",
        depth: 0,
        analyst_first: true,
        failures: &[("run_synthesis", 0), ("run_synthesis", 1)],
        batches: &[SynthesisFailed, SynthesisFailed, Approved],
        effects: 14,
    },
    Case {
        name: "synthesis-exhausted",
        depth: 0,
        analyst_first: true,
        failures: &[
            ("run_synthesis", 0),
            ("run_synthesis", 1),
            ("run_synthesis", 2),
        ],
        batches: &[SynthesisFailed, SynthesisFailed, SynthesisFailed, Exhausted],
        effects: 7,
    },
    Case {
        name: "mixed-retry",
        depth: 0,
        analyst_first: false,
        failures: &[("run_synthesis", 0), ("verify_result", 0)],
        batches: &[SynthesisFailed, QualityRejected, Approved],
        effects: 19,
    },
    Case {
        name: "depth-1",
        depth: 1,
        analyst_first: true,
        failures: &[],
        batches: &[SynthesisFailed, QualityRejected, Approved],
        effects: 21,
    },
    Case {
        name: "depth-2-exhausted",
        depth: 2,
        analyst_first: false,
        failures: &[],
        batches: &[SynthesisFailed, SynthesisFailed, QualityRejected, Exhausted],
        effects: 12,
    },
];

fn response(reply: &Value) -> Result<Value> {
    let output = reply["stdout"]
        .as_str()
        .ok_or("scientific command output")?;
    let start = output
        .find('{')
        .ok_or_else(|| format!("no scientific reply: {reply}"))?;
    serde_json::Deserializer::from_str(&output[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty scientific response")?
        .map_err(Into::into)
}

async fn receive_notice(guest: &process::Guest, agent: &str, path: &str) -> Result<Value> {
    let unavailable = format!("wash: read {path}: Permission denied (os error 2)\nwash> ");
    loop {
        let reply = guest.command(agent, format!("cat {path}")).await?;
        if reply["stdout"] != unavailable {
            return response(&reply);
        }
        hibana_runtime::host::sleep(Duration::from_millis(20)).await?;
    }
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current 13H Wasm runtime, wash and a build receipt"]
async fn actual_wasm_scientists_reconsider_and_retire() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("13H evidence root")?);
    fs::create_dir_all(&evidence)?;
    let directory = evidence.join("lab-cascading-wasi");
    fs::create_dir(&directory)?;
    let auditor = PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("13H auditor")?)
        .canonicalize()?;
    let auditor_sha256 = digest(&auditor)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_CASCADING_WASI_BINARY").ok_or("13H runtime")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_CASCADING_BUILD_RECEIPT").ok_or("13H build receipt")?,
    );
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let build = archive_build_receipt(&binary, &receipt, &directory)?;
    projection::archive(&binary, &directory, "13H")?;
    save(
        &directory.join("fixture-build.json"),
        &json!({
            "build":build,"guest_wasm_sha256":digest(&module)?,"runtime_binary_sha256":digest(&binary)?,
            "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
            "fixture_sha256":digest(Path::new("tools/runner/src/lab_cascading_boundary.rs"))?,
            "declared_cases":CASES.len(),"model_calls":0,"scored_benchmark":false
        }),
    )?;

    let mut verified = Vec::with_capacity(CASES.len());
    for Case {
        name: case,
        depth,
        analyst_first,
        failures,
        batches,
        effects,
    } in CASES
    {
        let stale_qc_read = case == "qc-first-stale-batch";
        let complete = batches.last() == Some(&Approved);
        let output = directory.join(case);
        fs::create_dir(&output)?;
        fs::hard_link(
            directory.join("projection.json"),
            output.join("projection.json"),
        )?;
        save(
            &output.join("case.json"),
            &json!({
                "task":"13H","depth":depth,"analyst_first":analyst_first,"failure_overrides":failures,
                "stale_qc_read":stale_qc_read,
                "model_calls":0,"scored_benchmark":false,"build":build,
                "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"delay_multiplier":0.01,
                "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
                "compiled_projection_sha256":digest(&output.join("projection.json"))?,
                "harness_binary_sha256":digest(&std::env::current_exe()?)?,
                "harness_source_sha256":digest(Path::new("tools/runner/src/lab_cascading_boundary.rs"))?
            }),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, fixture) =
            oracle::Simulator::new("13H", depth, 0.01, trace.clone(), failures)?;
        save(&output.join("fixture.json"), &fixture)?;
        let (guest, runtime) = process::guest(&binary, &module, &[], &output, trace, simulator)?;
        let ((), outcome) = hibana_runtime::race(async {
            async {

                let (final_result, (), (), (), ()) = async {
                    async {
                        for (index, &batch) in batches.iter().enumerate() {
                            loop {
                                let sent = guest.command("CHEMIST", "echo '{\"compound\":\"β fixture material\"}' > /service/run_synthesis".into()).await?;
                                if index != 0 && sent["stdout"] == "wash: close output /service/run_synthesis: Permission denied (os error 2)\nwash> " {
                                    // The prior result is known. Retry only a refused write
                                    // while QC and the instrument owners return authority.
                                    hibana_runtime::host::sleep(Duration::from_millis(20)).await?;
                                    continue;
                                }
                                assert_eq!(sent["stdout"], "wash> ", "{case}: {sent}");
                                break;
                            }
                            let reply = guest.command("CHEMIST", "cat /service/run_synthesis".into()).await?;
                            let synthesis = response(&reply)?;
                            let expected = match batch {
                                SynthesisFailed => "synthesis_failed",
                                Exhausted => "reagent_exhausted",
                                QualityRejected | Approved => "complete",
                            };
                            assert_eq!(synthesis["data"]["status"], expected, "{case}: {synthesis}");
                            if batch == Exhausted {
                                assert_eq!(guest.command("CHEMIST", "exit 0".into()).await?["exit_status"], 0);
                                return Ok(synthesis);
                            }
                            if batch == SynthesisFailed {
                                continue;
                            }
                            for (path, args) in [
                                ("run_analysis/synthesized_compound", json!({"sample":"synthesized_compound"})),
                                ("run_separation/compound_purification", json!({"material":"compound_purification"})),
                            ] {
                                let reply = guest.command("CHEMIST", format!("echo '{args}' > /service/{path}; cat /service/{path}")).await?;
                                assert_eq!(response(&reply)?["success"], true, "{case}: {reply}");
                            }
                            let quality = if batch == QualityRejected && index == 0 {
                                // Reproduce the ordinary blocking read before the
                                // first rejection that led to the live stop.
                                let reply = guest.command("CHEMIST", "cat /service/qc".into()).await?;
                                response(&reply)?
                            } else {
                                // At the boundary between visits this read may be
                                // unavailable until the next verdict is enabled.
                                receive_notice(&guest, "CHEMIST", "/service/qc").await?
                            };
                            if batch == QualityRejected {
                                assert_eq!(quality["data"]["status"], "rejected", "{case}: {quality}");
                                continue;
                            }
                            assert_eq!(quality["success"], true, "{case}: {quality}");
                            let reply = guest.command("CHEMIST", "echo '{\"sample\":\"final_validation\"}' > /service/run_analysis/final_validation; cat /service/run_analysis/final_validation".into()).await?;
                            let result = response(&reply)?;
                            assert_eq!(result["success"], true, "{case}: {reply}");
                            assert_eq!(guest.command("CHEMIST", "exit 0".into()).await?["exit_status"], 0);
                            return Ok(result);
                        }
                        Err(format!("{case}: missing terminal chemistry result").into())
                    };
                    async {
                        for (path, args) in [
                            ("run_separation/cell_cultures", json!({"material":"cell_cultures"})),
                            ("run_analysis/fluorescence", json!({"sample":"fluorescence"})),
                        ] {
                            let reply = guest.command("BIOLOGIST", format!("echo '{args}' > /service/{path}; cat /service/{path}")).await?;
                            assert_eq!(response(&reply)?["success"], true, "{case}: {reply}");
                        }
                        for &batch in batches {
                            if batch == Exhausted {
                                break;
                            }
                            let material = receive_notice(&guest, "BIOLOGIST", "/service/batch").await?;
                            if batch == SynthesisFailed {
                                assert_eq!(material["data"]["status"], "synthesis_failed", "{case}: {material}");
                                continue;
                            }
                            let args = json!({"compound":material["synthesis"]["data"]["compound"], "cell_type":"β fixture culture"});
                            let reply = guest.command("BIOLOGIST", format!("echo '{args}' > /service/run_bioassay; cat /service/run_bioassay")).await?;
                            let assay = response(&reply)?;
                            assert_eq!(assay["success"], true, "{case}: {reply}");
                            let publication = json!({"artifact_id":assay["data"]["artifact_id"]});
                            if analyst_first {
                                let reply = guest.command("BIOLOGIST", format!("echo '{publication}' > /service/publish/analyst; cat /service/publish/analyst")).await?;
                                assert_eq!(response(&reply)?["bioassay"], assay, "{case}: {reply}");
                            }
                            let reply = guest.command("BIOLOGIST", format!("echo '{publication}' > /service/publish/qc; cat /service/publish/qc")).await?;
                            assert_eq!(response(&reply)?["bioassay"], assay, "{case}: {reply}");
                            if batch == QualityRejected {
                                let rejection = receive_notice(&guest, "BIOLOGIST", "/service/batch").await?;
                                assert_eq!(rejection["data"]["status"], "rejected", "{case}: {rejection}");
                                assert_eq!(rejection["data"]["inputs"]["bioassay"], assay["data"]["artifact_id"], "{case}: {rejection}");
                                continue;
                            }
                            let approved = receive_notice(&guest, "BIOLOGIST", "/service/batch").await?;
                            assert_eq!(approved["success"], true, "{case}: {approved}");
                            assert_eq!(approved["data"]["inputs"]["bioassay"], assay["data"]["artifact_id"], "{case}: {approved}");
                            if !analyst_first {
                                let reply = guest.command("BIOLOGIST", format!("echo '{publication}' > /service/publish/analyst; cat /service/publish/analyst")).await?;
                                assert_eq!(response(&reply)?["bioassay"], assay, "{case}: {reply}");
                            }
                            assert_eq!(guest.command("BIOLOGIST", "exit 0".into()).await?["exit_status"], 0);
                        }
                        Ok(())
                    };
                    async {
                        for &batch in batches {
                            if batch == Exhausted {
                                break;
                            }
                            let material = receive_notice(&guest, "ANALYST", "/service/batch").await?;
                            if batch == SynthesisFailed {
                                assert_eq!(material["data"]["status"], "synthesis_failed", "{case}: {material}");
                                continue;
                            }
                            let mut operations = [
                                ("run_separation/verification", json!({"material":"verification"})),
                                ("run_chromatography", json!({"sample":material["synthesis"]["data"]["compound"]})),
                                ("run_analysis/molecular_detail", json!({"sample":"molecular_detail"})),
                            ];
                            let count = if analyst_first || batch == Approved { 3 } else { 1 };
                            if case == "depth-1" && batch == Approved {
                                operations.rotate_left(1);
                            }
                            for (path, args) in operations.into_iter().take(count) {
                                let reply = guest.command("ANALYST", format!("echo '{args}' > /service/{path}; cat /service/{path}")).await?;
                                assert_eq!(response(&reply)?["success"], true, "{case}: {reply}");
                            }
                            if batch == QualityRejected {
                                let retry = receive_notice(&guest, "ANALYST", "/service/batch").await?;
                                assert_eq!(retry["data"]["status"], "rejected", "{case}: {retry}");
                            } else {
                                let approved = receive_notice(&guest, "ANALYST", "/service/batch").await?;
                                assert_eq!(approved["success"], true, "{case}: {approved}");
                                assert_eq!(approved["data"]["inputs"]["bioassay"], material["bioassay"]["data"]["artifact_id"], "{case}: {approved}");
                                if case == "depth-1" {
                                    let stale = hibana_runtime::race(
                                        guest.command("ANALYST", "cat /service/batch".into()),
                                        async {
                                            hibana_runtime::host::sleep(Duration::from_secs(8)).await?;
                                            Err("analyst's batch read after final approval remained blocked".into())
                                        },
                                    ).await?;
                                    assert_eq!(stale["stdout"], "wash: read /service/batch: Permission denied (os error 2)\nwash> ");
                                }
                                assert_eq!(guest.command("ANALYST", "exit 0".into()).await?["exit_status"], 0);
                            }
                        }
                        Ok(())
                    };
                    async {
                        for &batch in batches {
                            if !matches!(batch, Approved | QualityRejected) {
                                continue;
                            }
                            let material = receive_notice(&guest, "QC_CONTROLLER", "/service/bioassay").await?;
                            let reply = guest.command("QC_CONTROLLER", "echo '{\"scientist\":\"BIOLOGIST\",\"data_type\":\"bioassay\"}' > /service/verify_result/BIOLOGIST; cat /service/verify_result/BIOLOGIST".into()).await?;
                            let quality = response(&reply)?;
                            assert_eq!(quality["success"], batch == Approved, "{case}: {reply}");
                            assert_eq!(quality["data"]["inputs"]["bioassay"], material["bioassay"]["data"]["artifact_id"], "{case}: {reply}");
                        }
                        if stale_qc_read {
                            let stale = hibana_runtime::race(
                                guest.command("QC_CONTROLLER", "cat /service/bioassay".into()),
                                async {
                                    hibana_runtime::host::sleep(Duration::from_secs(8)).await?;
                                    Err("stale QC batch read remained blocked".into())
                                },
                            ).await?;
                            assert_eq!(stale["stdout"], "wash: read /service/bioassay: Permission denied (os error 2)\nwash> ");
                        }
                        if complete {
                            assert_eq!(guest.command("QC_CONTROLLER", "exit 0".into()).await?["exit_status"], 0);
                        }
                        Ok(())
                    };
                    async {
                        let arguments = if depth == 2 { json!({}) } else { json!({"notes":"β fixture inspection"}) };
                        let reply = guest.command("SAFETY_OFFICER", format!("echo '{arguments}' > /service/perform_inspection; cat /service/perform_inspection")).await?;
                        assert_eq!(response(&reply)?["success"], true, "{case}: {reply}");
                        assert_eq!(guest.command("SAFETY_OFFICER", "exit 0".into()).await?["exit_status"], 0);
                    };
                }.await?;
                if !complete {
                    let guest = &guest;
                    let final_result = &final_result;
                    async {
                        for (role, path) in [
                            ("BIOLOGIST", "/service/batch"),
                            ("ANALYST", "/service/batch"),
                            ("QC_CONTROLLER", "/service/qc"),
                        ] {
                            async {
                                let outcome = receive_notice(guest, role, path).await?;
                                assert_eq!(&outcome, final_result, "{case}: {outcome}");
                                assert_eq!(guest.command(role, "exit 0".into()).await?["exit_status"], 0);
                            };
                        }
                        Ok(())
                    }.await?;
                }
                Ok(())
            };
            async { runtime.await };
        },async {
            hibana_runtime::host::sleep(Duration::from_secs(90)).await?;
            Err(format!("actual 13H Wasm deadline: {case}").into())
        }).await?;
        save(&output.join("runtime-result.json"), &outcome)?;
        assert_eq!(outcome["runtime_exit_code"], 0, "{outcome}");
        assert_eq!(outcome["protocol"]["status"], "complete", "{outcome}");
        assert_eq!(outcome["score"]["complete"], complete, "{case}: {outcome}");
        assert_eq!(outcome["score"]["violations"], json!([]), "{outcome}");
        // Release the independent auditor's temporary workspace after each case.
        let checked = Command::new(&auditor)
            .args(["--audit", "13H"])
            .arg(&events)
            .output()?;
        fs::write(output.join("audit-stderr.log"), &checked.stderr)?;
        if !checked.status.success() {
            return Err(format!(
                "{case}: scientific audit ended with {}. {}",
                checked.status,
                String::from_utf8_lossy(&checked.stderr)
            )
            .into());
        }
        let audit: Value = serde_json::from_slice(&checked.stdout)?;
        assert_eq!(audit["status"], "passed", "{case}: {audit}");
        assert_eq!(audit["simulator_complete"], complete, "{case}: {audit}");
        assert_eq!(audit["domain_calls"], effects, "{case}: {audit}");
        save(&output.join("audit.json"), &audit)?;
        verified.push(json!({
            "case":case,"depth":depth,"simulator_complete":complete,
            "domain_calls":effects,"violations":audit["violations"],
            "complete_capture_records":audit["dynamic_protocol"]["captured_records"],
            "runtime_binary_sha256":audit["runtime_binary_sha256"],
            "audit_sha256":digest(&output.join("audit.json"))?,"events_sha256":digest(&events)?
        }));
    }
    save(
        &directory.join("mechanism.json"),
        &json!({
            "task":"13H","status":"passed","declared_cases":CASES.len(),"cases":verified,
            "auditor_sha256":auditor_sha256,"model_calls":0,"scored_benchmark":false
        }),
    )?;
    Ok(())
}
