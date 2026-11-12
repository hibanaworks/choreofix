//! Real three-guest requests; schedules are test inputs, never world permissions.
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
    let text = reply["stdout"].as_str().ok_or("medical terminal output")?;
    let start = text.find('{').ok_or("missing medical response")?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty medical response")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires current 5E binary, source receipt, actual wash and common simulator"]
async fn actual_wasi_partial_panels_and_independent_ct_orders() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("medical evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "medical-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_MEDICAL_WASI_BINARY").ok_or("medical binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_MEDICAL_BUILD_RECEIPT").ok_or("medical source receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    // CT can appear at each of the three cuts between two ordered panel
    // requests. Reverse panel order, combined panels and repeated partial
    // requests use the same production global and localsides.
    let cases: &[(&[&str], &[&str], f64, bool)] = &[
        (
            &["CT", "vitals", "basic_labs"],
            &["ecg", "cardiac_enzymes"],
            1.0,
            false,
        ),
        (
            &["vitals", "CT", "basic_labs"],
            &["cardiac_enzymes", "ecg"],
            0.01,
            false,
        ),
        (
            &["vitals", "basic_labs", "CT"],
            &["ecg", "cardiac_enzymes"],
            0.01,
            false,
        ),
        (
            &["CT", "basic_labs", "vitals"],
            &["cardiac_enzymes", "ecg"],
            0.01,
            false,
        ),
        (
            &["basic_labs", "CT", "vitals"],
            &["ecg", "cardiac_enzymes"],
            0.01,
            false,
        ),
        (
            &["basic_labs", "vitals", "CT"],
            &["cardiac_enzymes", "ecg"],
            0.01,
            false,
        ),
        (
            &["vitals,basic_labs", "CT"],
            &["ecg,cardiac_enzymes"],
            0.01,
            false,
        ),
        (
            &["CT", "vitals,basic_labs"],
            &["ecg,cardiac_enzymes"],
            0.01,
            false,
        ),
        (
            &["vitals", "vitals", "CT", "vitals", "basic_labs"],
            &[
                "cardiac_enzymes",
                "cardiac_enzymes",
                "cardiac_enzymes",
                "ecg",
            ],
            0.01,
            true,
        ),
    ];
    let mut outcomes = Vec::new();
    for (index, &(er_requests, cardiac_requests, delay, invalid)) in cases.iter().enumerate() {
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "5E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"5E","depth":0,"er_requests":er_requests,
            "cardiac_requests":cardiac_requests,"delay_multiplier":delay,"invalid_prefixes":invalid,
            "runtime_binary_sha256":digest(&binary)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "guest_wasm_sha256":digest(module)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,
            "simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"build":receipt,
            "model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("5E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["5E"], &output, trace, simulator)?;
        let ((),(),(),result) = hibana_runtime::race(async {
            async {
                if invalid {
                    let reply = guest.command("ER_DOCTOR","echo '{\"patient_id\":\"patient_other\"}' > /service/assess_patient; cat /service/result".into()).await?;
                    assert_eq!(response(&reply)?["status"],"invalid_arguments");
                }
                let reply = guest.command("ER_DOCTOR","echo '{\"patient_id\":\"patient_1\"}' > /service/assess_patient; cat /service/result".into()).await?;
                assert_eq!(response(&reply)?["success"],true);
                for (at,request) in er_requests.iter().enumerate() {
                    if invalid {
                        let reply = guest.command("ER_DOCTOR","echo '{\"patient_id\":\"patient_1\",\"test_type\":\"ecg\"}' > /service/order_tests; cat /service/result".into()).await?;
                        assert_eq!(response(&reply)?["status"],"invalid_arguments");
                    }
                    let (tool,arguments) = if *request == "CT" {
                        ("perform_imaging",json!({"patient_id":"patient_1","modality":"CT"}))
                    } else {
                        ("order_tests",json!({"patient_id":"patient_1","test_type":request}))
                    };
                    let reply = guest.command("ER_DOCTOR",format!("echo '{arguments}' > /service/{tool}; cat /service/result")).await?;
                    let actual = response(&reply)?;
                    assert_eq!(actual["success"],true);
                    if at + 1 == er_requests.len() {
                        assert_eq!(actual["consultation"]["cardiac_diagnosis"]["success"],true);
                        assert_eq!(actual["consultation"]["medication_review"]["success"],true);
                    }
                }
                assert_eq!(guest.command("ER_DOCTOR","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async {
                for request in cardiac_requests {
                    if invalid {
                        let reply = guest.command("CARDIOLOGIST","echo '{\"patient_id\":\"patient_1\",\"test_type\":\"vitals\"}' > /service/order_tests; cat /service/result".into()).await?;
                        assert_eq!(response(&reply)?["status"],"invalid_arguments");
                    }
                    let arguments = json!({"patient_id":"patient_1","test_type":request});
                    let reply = guest.command("CARDIOLOGIST",format!("echo '{arguments}' > /service/order_tests; cat /service/result")).await?;
                    assert_eq!(response(&reply)?["success"],true);
                }
                let mut arguments = json!({"patient_id":"patient_1"});
                if index % 2 == 0 { arguments["symptoms"] = json!("caller symptom notes"); }
                let reply = guest.command("CARDIOLOGIST",format!("echo '{arguments}' > /service/diagnose; cat /service/result")).await?;
                let actual = response(&reply)?;
                assert_eq!(actual["success"],true);
                assert_eq!(actual["data"]["symptoms"],arguments["symptoms"]);
                assert_eq!(actual["consultation"]["imaging"]["success"],true);
                assert_eq!(guest.command("CARDIOLOGIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async {
                if invalid {
                    let reply = guest.command("PHARMACIST","echo '{\"patient_id\":\"patient_1\",\"medications\":[]}' > /service/review_prescriptions; cat /service/result".into()).await?;
                    assert_eq!(response(&reply)?["status"],"invalid_arguments");
                }
                let mut arguments = json!({"patient_id":"patient_1"});
                if index % 2 == 0 { arguments["medications"] = json!("supplied medication data"); }
                let reply = guest.command("PHARMACIST",format!("echo '{arguments}' > /service/review_prescriptions; cat /service/result")).await?;
                let actual = response(&reply)?;
                assert_eq!(actual["success"],true);
                assert_eq!(actual["data"]["medications"],arguments["medications"]);
                assert_eq!(guest.command("PHARMACIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async { runtime.await };
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        outcomes.push(json!({"case":index,"model_calls":0,
            "domain_calls":result["score"]["events"].as_array().ok_or("actual effects")?.len(),
            "dynamic_and_effect_audit_qualified":false}));
        save(
            &directory.join("progress.json"),
            &json!({"runtime_completed":outcomes.len(),"completed":0,"outcomes":outcomes}),
        )?;
    }
    for index in 0..outcomes.len() {
        let output = directory.join(format!("case{index}"));
        let audit = crate::medical_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        if index == 0 {
            assert!(
                audit["panel_imaging_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0)
            );
            let mutants = crate::medical_audit::check_mutants(
                &output.join("events.jsonl"),
                &output.join("record-mutants"),
            )?;
            assert!(mutants["mutants_rejected"].as_u64().is_some_and(|n| n > 0));
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
        &json!({"status":"passed","completed":outcomes.len(),
        "outcomes":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
