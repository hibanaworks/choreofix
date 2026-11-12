//! Seven actual Wasm participants, clinical data causality and physical resources.
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
    let text = reply["stdout"].as_str().ok_or("terminal output")?;
    let at = text.find('{').ok_or("missing clinical response")?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty response")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires current 5H binary, source receipt, actual wash and common simulator"]
async fn actual_wasi_two_patient_care_and_all_ct_orders() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "critical-care-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_CRITICAL_CARE_WASI_BINARY").ok_or("critical-care binary")?,
    )
    .canonicalize()?;
    let receipt = archive_build_receipt(
        &binary,
        &PathBuf::from(
            std::env::var_os("CHOREOFIX_CRITICAL_CARE_BUILD_RECEIPT").ok_or("source receipt")?,
        ),
        &directory,
    )?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let orders = [
        [
            ("patient_a", true),
            ("patient_a", false),
            ("patient_b", true),
            ("patient_b", false),
        ],
        [
            ("patient_a", true),
            ("patient_b", true),
            ("patient_a", false),
            ("patient_b", false),
        ],
        [
            ("patient_a", true),
            ("patient_b", true),
            ("patient_b", false),
            ("patient_a", false),
        ],
        [
            ("patient_b", true),
            ("patient_b", false),
            ("patient_a", true),
            ("patient_a", false),
        ],
        [
            ("patient_b", true),
            ("patient_a", true),
            ("patient_b", false),
            ("patient_a", false),
        ],
        [
            ("patient_b", true),
            ("patient_a", true),
            ("patient_a", false),
            ("patient_b", false),
        ],
    ];
    let mut outcomes = Vec::new();
    for index in 0..8 {
        let mut order = orders[index % orders.len()]
            .map(|(patient, perform)| (patient, "CT", perform))
            .to_vec();
        let mri_at = 2 * (index % 3);
        order.splice(
            mri_at..mri_at,
            [("patient_b", "MRI", true), ("patient_b", "MRI", false)],
        );
        let invalid = index == 7;
        let combined = index == 6;
        let delay = if index == 0 { 1.0 } else { 0.01 };
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "5H")?;
        save(
            &output.join("case.json"),
            &json!({
                "task":"5H", "depth":0, "imaging_order":order,
                "combined_panels":combined, "invalid_and_repeated_partial":invalid,
                "delay_multiplier":delay, "runtime_binary_sha256":digest(&binary)?,
                "compiled_projection_sha256":digest(&output.join("projection.json"))?,
                "guest_wasm_sha256":digest(module)?,
                "harness_binary_sha256":digest(&std::env::current_exe()?)?,
                "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,
                "build":receipt, "model_calls":0, "scored_benchmark":false
            }),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("5H", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["5H"], &output, trace, simulator)?;
        let guest = &guest;
        let ([(), ()], (), [(), (), (), ()], result) = hibana_runtime::race(async {
            for (actor, patient, panel, foreign) in [
                ("ER_DOCTOR_A", "patient_a", &["trauma_panel", "blood_type", "coag"][..], "stroke_panel"),
                ("ER_DOCTOR_B", "patient_b", &["stroke_panel", "ct_angio"][..], "trauma_panel"),
            ] { async move {
                if invalid {
                    let reply = guest.command(actor, "echo '{\"patient_id\":\"other\"}' > /service/assess_patient; cat /service/result".into()).await?;
                    assert_eq!(response(&reply)?["status"], "invalid_arguments");
                }
                let args = json!({"patient_id":patient});
                let reply = guest.command(actor, format!("echo '{args}' > /service/assess_patient; cat /service/result")).await?;
                assert_eq!(response(&reply)?["success"], true);
                if invalid {
                    let args = json!({"patient_id":patient,"test_type":foreign});
                    let reply = guest.command(actor, format!("echo '{args}' > /service/order_tests; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["status"], "invalid_arguments");
                }
                let requests = if combined {
                    vec![panel.join(",")]
                } else {
                    let mut requests = panel.iter().map(ToString::to_string).collect::<Vec<_>>();
                    if index % 2 == 1 { requests.reverse(); }
                    if invalid { requests.insert(0, requests[0].clone()); }
                    requests
                };
                for test in requests {
                    let args = json!({"patient_id":patient,"test_type":test});
                    let reply = guest.command(actor, format!("echo '{args}' > /service/order_tests; cat /service/result")).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"], true);
                    assert_eq!(actual["data"]["patient_id"], patient);
                }
                if invalid {
                    let args = json!({"patient_id":patient,"symptoms":[]});
                    let reply = guest.command(actor, format!("echo '{args}' > /service/diagnose; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["status"], "invalid_arguments");
                }
                let args = json!({"patient_id":patient,"symptoms":format!("actual {actor} findings")});
                let reply = guest.command(actor, format!("echo '{args}' > /service/diagnose; cat /service/result")).await?;
                let actual = response(&reply)?;
                assert_eq!(actual["success"], true);
                assert_eq!(actual["diagnosis"]["data"]["patient_id"], patient);
                assert_eq!(actual["diagnosis"]["data"]["symptoms"], args["symptoms"]);
                assert_eq!(actual["care"]["success"], true);
                assert_eq!(actual["care"]["data"]["patient_id"], patient);
                assert_eq!(actual["prescription_review"]["success"], true);
                assert_eq!(guest.command(actor, "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            }; }
            async {
                if invalid {
                    for (tool, args) in [
                        ("interpret_imaging", json!({"patient_id":"patient_a","modality":"CT"})),
                        ("perform_imaging", json!({"patient_id":"patient_a","modality":"MRI"})),
                    ] {
                        let reply = guest.command("RADIOLOGIST", format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;
                        assert_eq!(response(&reply)?["status"], "invalid_arguments");
                    }
                }
                for (patient, modality, perform) in order {
                    let args = json!({"patient_id":patient,"modality":modality});
                    let tool = if perform { "perform_imaging" } else { "interpret_imaging" };
                    let reply = guest.command("RADIOLOGIST", format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"], true);
                    assert_eq!(actual["data"]["patient_id"], patient);
                    assert_eq!(actual["data"]["modality"], modality);
                }
                assert_eq!(guest.command("RADIOLOGIST", "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            };
            for (actor, patient, tool, fields) in [
                ("ANESTHESIOLOGIST", "patient_a", "prepare_anesthesia", json!({})),
                ("SURGEON", "patient_a", "perform_surgery", json!({"procedure":"trauma_surgery"})),
                ("ICU_COORDINATOR", "patient_b", "manage_icu", json!({})),
                ("PHARMACIST", "patient_A_and_B", "review_prescriptions", json!({"medications":"caller selected medications"})),
            ] { async move {
                let mut args = fields;
                args["patient_id"] = json!(patient);
                if invalid {
                    let mut changed = args.clone();
                    changed["patient_id"] = json!("other");
                    let reply = guest.command(actor, format!("echo '{changed}' > /service/{tool}; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["status"], "invalid_arguments");
                }
                let reply = guest.command(actor, format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;
                let actual = response(&reply)?;
                assert_eq!(actual["success"], true);
                assert_eq!(actual["data"]["patient_id"], patient);
                if invalid {
                    let reply = guest.command(actor, format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["status"], "out_of_order");
                }
                assert_eq!(guest.command(actor, "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            }; }
            async { runtime.await };
        }, hibana_runtime::host::deadline(Duration::from_secs(300))).await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        outcomes.push(json!({"case":index,"domain_calls":result["score"]["events"].as_array().ok_or("actual effects")?.len(),"dynamic_and_effect_audit_qualified":false}));
        save(
            &directory.join("progress.json"),
            &json!({"runtime_completed":outcomes.len(),"completed":0,"outcomes":outcomes}),
        )?;
    }
    for index in 0..outcomes.len() {
        let output = directory.join(format!("case{index}"));
        let audit = crate::critical_care_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        if index == 0 {
            assert!(
                audit["independent_effect_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0)
            );
            crate::critical_care_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
        }
        outcomes[index]["audit"] = audit;
        outcomes[index]["dynamic_and_effect_audit_qualified"] = json!(true);
        save(
            &directory.join("progress.json"),
            &json!({"runtime_completed":outcomes.len(),"completed":index+1,"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","completed":outcomes.len(),"outcomes":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
