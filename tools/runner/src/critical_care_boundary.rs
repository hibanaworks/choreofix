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
    for index in 0..9 {
        let mut order = orders[index % orders.len()]
            .map(|(patient, perform)| (patient, "CT", perform))
            .to_vec();
        let mri_at = 2 * (index % 3);
        order.splice(
            mri_at..mri_at,
            [("patient_b", "MRI", true), ("patient_b", "MRI", false)],
        );
        let invalid = index == 7;
        let combined = index == 6 || index == 8;
        let delay = if index == 0 || index == 8 { 1.0 } else { 0.01 };
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
                    let reply=guest.command(actor,"cat /service/summary; cat /service/findings".into()).await?;assert!(reply["stdout"].as_str().ok_or("premature read reply")?.contains("wash:"));
                    let reply=guest.command(actor,"echo '{\"patient_id\":\"other\"}' > /service/assess_patient".into()).await?;assert!(reply["stdout"].as_str().ok_or("invalid patient reply")?.contains("wash:"));
                }
                let args=json!({"patient_id":patient});let reply=guest.command(actor,format!("echo '{args}' > /service/assess_patient; cat /service/assess_patient")).await?;assert_eq!(response(&reply)?["success"],true);
                if invalid{let args=json!({"patient_id":patient,"test_type":foreign});let reply=guest.command(actor,format!("echo '{args}' > /service/order_tests")).await?;assert!(reply["stdout"].as_str().ok_or("foreign panel reply")?.contains("wash:"));}
                let requests=if combined{vec![panel.join(",")]}else{let mut requests=panel.iter().map(ToString::to_string).collect::<Vec<_>>();if index%2==1{requests.reverse();}
                    if invalid{requests.insert(0,requests[0].clone());}requests};
                for test in requests{let args=json!({"patient_id":patient,"test_type":test});let reply=guest.command(actor,format!("echo '{args}' > /service/order_tests; cat /service/order_tests")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["patient_id"],patient);}
                let args=json!({"patient_id":patient,"test_type":panel[0]});let reply=guest.command(actor,format!("echo '{args}' > /service/order_tests")).await?;assert!(reply["stdout"].as_str().ok_or("completed panel reply")?.contains("wash:"));
                let findings=response(&guest.command(actor,"cat /service/findings".into()).await?)?;assert_eq!(findings["assessment"]["data"]["patient_id"],patient);
                if invalid{let args=json!({"patient_id":patient,"symptoms":[]});let reply=guest.command(actor,format!("echo '{args}' > /service/diagnose")).await?;assert!(reply["stdout"].as_str().ok_or("invalid diagnosis reply")?.contains("wash:"));}
                let args=json!({"patient_id":patient,"symptoms":format!("actual {actor} findings")});let reply=guest.command(actor,format!("echo '{args}' > /service/diagnose; cat /service/diagnose")).await?;let diagnosis=response(&reply)?;assert_eq!(diagnosis["success"],true);assert_eq!(diagnosis["data"]["symptoms"],args["symptoms"]);
                let reply=guest.command(actor,format!("echo '{args}' > /service/diagnose")).await?;assert!(reply["stdout"].as_str().ok_or("completed diagnosis reply")?.contains("wash:"));
                let actual=response(&guest.command(actor,"cat /service/summary".into()).await?)?;assert_eq!(actual["diagnosis"],diagnosis);assert_eq!(actual["findings"],findings);assert_eq!(actual["care"]["success"],true);assert_eq!(actual["care"]["data"]["patient_id"],patient);assert_eq!(actual["prescription_review"]["success"],true);
                assert_eq!(guest.command(actor, "exit 0".into()).await?["exit_status"], 0);
                Ok(())
            }; }
            async {
                if invalid{
                    let args=json!({"patient_id":"patient_a","modality":"CT"});let reply=guest.command("RADIOLOGIST",format!("echo '{args}' | dd of=/service/interpret_imaging_ct_a oflag=nonblock")).await?;assert!(reply["stdout"].as_str().ok_or("early interpretation reply")?.contains("wash:"));
                    let args=json!({"patient_id":"patient_a","modality":"MRI"});let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/perform_imaging_mri_b")).await?;assert!(reply["stdout"].as_str().ok_or("wrong imaging patient reply")?.contains("wash:"));
                }
                if index==8{
                    for (patient,modality,file)in [("patient_a","CT","perform_imaging_ct_a"),("patient_b","MRI","perform_imaging_mri_b")]{let args=json!({"patient_id":patient,"modality":modality});let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{file}")).await?;assert!(!reply["stdout"].as_str().ok_or("concurrent imaging request")?.contains("wash:"));}
                    for file in ["perform_imaging_ct_a","perform_imaging_mri_b"]{assert_eq!(response(&guest.command("RADIOLOGIST",format!("cat /service/{file}")).await?)?["success"],true);}
                    for (patient,modality,file)in [("patient_a","CT","interpret_imaging_ct_a"),("patient_b","MRI","interpret_imaging_mri_b")]{let args=json!({"patient_id":patient,"modality":modality});let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{file}")).await?;assert!(!reply["stdout"].as_str().ok_or("concurrent interpretation request")?.contains("wash:"));}
                    for file in ["interpret_imaging_ct_a","interpret_imaging_mri_b"]{assert_eq!(response(&guest.command("RADIOLOGIST",format!("cat /service/{file}")).await?)?["success"],true);}
                    for tool in ["perform_imaging","interpret_imaging"]{let args=json!({"patient_id":"patient_b","modality":"CT"});let file=format!("{tool}_ct_b");assert_eq!(response(&guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{file}; cat /service/{file}")).await?)?["success"],true);}
                }else{for(patient,modality,perform)in order{
                    let args=json!({"patient_id":patient,"modality":modality});let tool=if perform{"perform_imaging"}else{"interpret_imaging"};let suffix=if patient=="patient_a"{"a"}else{"b"};let file=format!("{tool}_{}_{suffix}",modality.to_ascii_lowercase());
                    let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{file}; cat /service/{file}")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["patient_id"],patient);assert_eq!(actual["data"]["modality"],modality);
                    if invalid{let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{file}")).await?;assert!(reply["stdout"].as_str().ok_or("duplicate imaging reply")?.contains("wash:"));}
                }}
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
                if invalid{
                    let mut changed=args.clone();changed["patient_id"]=json!("other");let reply=guest.command(actor,format!("echo '{changed}' > /service/{tool}")).await?;assert!(reply["stdout"].as_str().ok_or("foreign care patient")?.contains("wash:"));
                }
                if actor!="PHARMACIST"{let diagnosis=response(&guest.command(actor,"cat /service/diagnosis".into()).await?)?;assert_eq!(diagnosis["result"]["success"],true);assert_eq!(diagnosis["result"]["data"]["patient_id"],patient);}
                let reply=guest.command(actor,format!("echo '{args}' > /service/{tool}; cat /service/{tool}")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["patient_id"],patient);
                if tool=="review_prescriptions"{assert_eq!(actual["data"]["medications"],args["medications"]);}
                if tool=="perform_surgery"{assert_eq!(actual["data"]["procedure"],args["procedure"]);}
                let reply=guest.command(actor,format!("echo '{args}' > /service/{tool}")).await?;assert!(reply["stdout"].as_str().ok_or("completed care request")?.contains("wash:"));
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
        let audit = crate::journal::Evidence::open(&output.join("events.jsonl"))?
            .audit(crate::critical_care_audit::check)?;
        save(&output.join("audit.json"), &audit)?;
        if index == 8 {
            assert!(
                audit["cross_scanner_effect_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n >= 2),
                "CT/MRI actual imaging and interpretation effects must overlap"
            );
        }
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

#[hibana_runtime::main]
#[test]
#[ignore = "requires current direct-file 5H binary, receipt, actual wash and common simulator"]
async fn a_patient_finishes_before_the_other_is_assessed() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "critical-care-patient-independence-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_CRITICAL_CARE_WASI_BINARY").ok_or("critical care binary")?,
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
    let mut outcomes = Vec::new();
    for first in ["patient_a", "patient_b"] {
        let output = directory.join(first);
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "5H")?;
        save(
            &output.join("case.json"),
            &json!({"task":"5H","depth":0,"first_completed_patient":first,"late_patient_assessment_after_first_care":true,"delay_multiplier":0.01,"runtime_binary_sha256":digest(&binary)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,"guest_wasm_sha256":digest(module)?,"test_driver_sha256":digest(&std::env::current_exe()?)?,"build":receipt,"model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("5H", 0, 0.01, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["5H"], &output, trace, simulator)?;
        let guest = &guest;
        let execution = async {
            async {
                let patients = if first == "patient_a" {
                    ["patient_a", "patient_b"]
                } else {
                    ["patient_b", "patient_a"]
                };
                for patient in patients {
                    let (actor, tests, suffix, modalities) = if patient == "patient_a" {
                        (
                            "ER_DOCTOR_A",
                            "trauma_panel,blood_type,coag",
                            "a",
                            &["CT"][..],
                        )
                    } else {
                        (
                            "ER_DOCTOR_B",
                            "stroke_panel,ct_angio",
                            "b",
                            &["CT", "MRI"][..],
                        )
                    };
                    let arguments = json!({"patient_id":patient});
                    let reply=guest.command(actor,format!("echo '{arguments}' > /service/assess_patient; cat /service/assess_patient")).await?;
                    assert_eq!(response(&reply)?["success"], true);
                    let arguments = json!({"patient_id":patient,"test_type":tests});
                    let reply=guest.command(actor,format!("echo '{arguments}' > /service/order_tests; cat /service/order_tests")).await?;
                    assert_eq!(response(&reply)?["success"], true);
                    for modality in modalities {
                        for tool in ["perform_imaging", "interpret_imaging"] {
                            let arguments = json!({"patient_id":patient,"modality":modality});
                            let file = format!("{tool}_{}_{suffix}", modality.to_ascii_lowercase());
                            let reply = guest
                                .command(
                                    "RADIOLOGIST",
                                    format!(
                                        "echo '{arguments}' > /service/{file}; cat /service/{file}"
                                    ),
                                )
                                .await?;
                            assert_eq!(response(&reply)?["success"], true);
                        }
                    }
                    let reply = guest.command(actor, "cat /service/findings".into()).await?;
                    assert_eq!(
                        response(&reply)?["assessment"]["data"]["patient_id"],
                        patient
                    );
                    let arguments =
                        json!({"patient_id":patient,"symptoms":"actual completed workup"});
                    let reply = guest
                        .command(
                            actor,
                            format!(
                                "echo '{arguments}' > /service/diagnose; cat /service/diagnose"
                            ),
                        )
                        .await?;
                    assert_eq!(response(&reply)?["success"], true);
                    let reply = guest.command(actor, "cat /service/summary".into()).await?;
                    let summary = response(&reply)?;
                    assert_eq!(summary["care"]["success"], true);
                    assert_eq!(summary["care"]["data"]["patient_id"], patient);
                    assert_eq!(
                        guest.command(actor, "exit 0".into()).await?["exit_status"],
                        0
                    );
                }
                assert_eq!(
                    guest.command("RADIOLOGIST", "exit 0".into()).await?["exit_status"],
                    0
                );
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            };
            for (actor, patient, tool, fields) in [
                (
                    "ANESTHESIOLOGIST",
                    "patient_a",
                    "prepare_anesthesia",
                    json!({}),
                ),
                (
                    "SURGEON",
                    "patient_a",
                    "perform_surgery",
                    json!({"procedure":"trauma_surgery"}),
                ),
                ("ICU_COORDINATOR", "patient_b", "manage_icu", json!({})),
                (
                    "PHARMACIST",
                    "patient_A_and_B",
                    "review_prescriptions",
                    json!({}),
                ),
            ] {
                async move {
                    if actor != "PHARMACIST" {
                        let reply = guest
                            .command(actor, "cat /service/diagnosis".into())
                            .await?;
                        assert_eq!(response(&reply)?["result"]["data"]["patient_id"], patient);
                    }
                    let mut arguments = fields;
                    arguments["patient_id"] = json!(patient);
                    let reply = guest
                        .command(
                            actor,
                            format!("echo '{arguments}' > /service/{tool}; cat /service/{tool}"),
                        )
                        .await?;
                    assert_eq!(response(&reply)?["success"], true);
                    assert_eq!(
                        guest.command(actor, "exit 0".into()).await?["exit_status"],
                        0
                    );
                    Ok(())
                };
            }
            async { runtime.await };
        };
        let ((), [(), (), (), ()], result) = hibana_runtime::race(
            execution,
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let events = result["score"]["events"]
            .as_array()
            .ok_or("common effects")?;
        let first_care = events
            .iter()
            .find(|event| {
                event["args"]["patient_id"] == first
                    && ["perform_surgery", "manage_icu"]
                        .contains(&event["tool"].as_str().unwrap_or(""))
            })
            .ok_or("first patient's actual care")?;
        let late_assessment = events
            .iter()
            .find(|event| event["tool"] == "assess_patient" && event["args"]["patient_id"] != first)
            .ok_or("later patient's actual assessment")?;
        assert!(
            first_care["completed_at"]
                .as_f64()
                .ok_or("completed care timestamp")?
                < late_assessment["timestamp"]
                    .as_f64()
                    .ok_or("assessment start timestamp")?
        );
        let audit = crate::journal::Evidence::open(&output.join("events.jsonl"))?
            .audit(crate::critical_care_audit::check)?;
        save(&output.join("audit.json"), &audit)?;
        outcomes.push(json!({"first_completed_patient":first,"late_patient_assessment_after_first_care":true,"audit":audit}));
        save(
            &directory.join("progress.json"),
            &json!({"completed":outcomes.len(),"outcomes":outcomes}),
        )?;
    }
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","completed":outcomes.len(),"outcomes":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
