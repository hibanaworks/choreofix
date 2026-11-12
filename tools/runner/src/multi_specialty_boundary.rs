//! Real five-Wasm clinical requests under all six CT/MRI causal interleavings.
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
#[ignore = "requires current 5M binary, source receipt, actual wash and common simulator"]
async fn actual_wasi_all_imaging_orders_and_partial_panels() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "multi-specialty-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MULTI_SPECIALTY_WASI_BINARY")
            .ok_or("multi-specialty binary")?,
    )
    .canonicalize()?;
    let receipt = archive_build_receipt(
        &binary,
        &PathBuf::from(
            std::env::var_os("CHOREOFIX_MULTI_SPECIALTY_BUILD_RECEIPT").ok_or("source receipt")?,
        ),
        &directory,
    )?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let orders = [
        [("CT", true), ("CT", false), ("MRI", true), ("MRI", false)],
        [("CT", true), ("MRI", true), ("CT", false), ("MRI", false)],
        [("CT", true), ("MRI", true), ("MRI", false), ("CT", false)],
        [("MRI", true), ("MRI", false), ("CT", true), ("CT", false)],
        [("MRI", true), ("CT", true), ("MRI", false), ("CT", false)],
        [("MRI", true), ("CT", true), ("CT", false), ("MRI", false)],
    ];
    let mut outcomes = Vec::new();
    for index in 0..8 {
        let order = orders[index % orders.len()];
        let invalid = index == 7;
        let combined = index == 6;
        let delay = if index == 0 { 1.0 } else { 0.01 };
        let output = directory.join(format!("case{index}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "5M")?;
        save(
            &output.join("case.json"),
            &json!({"task":"5M","depth":0,"imaging_order":order,"combined_panels":combined,"invalid_and_repeated_partial":invalid,"delay_multiplier":delay,
            "runtime_binary_sha256":digest(&binary)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,"guest_wasm_sha256":digest(module)?,"harness_binary_sha256":digest(&std::env::current_exe()?)?,"simulator_contract":choreofix_inputs::SIMULATOR_CONTRACT,"build":receipt,"model_calls":0,"scored_benchmark":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("5M", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, module, &["5M"], &output, trace, simulator)?;
        let guest = &guest;
        let ((),[(),()],(),(),result)=hibana_runtime::race(async{
            async{
                if invalid{let reply=guest.command("ER_DOCTOR","echo '{\"patient_id\":\"other\"}' > /service/assess_patient; cat /service/result".into()).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");}
                let reply=guest.command("ER_DOCTOR","echo '{\"patient_id\":\"patient_1\"}' > /service/assess_patient; cat /service/result".into()).await?;assert_eq!(response(&reply)?["success"],true);
                let requests=if combined{vec!["vitals,basic_labs"]}else if invalid{vec!["vitals","vitals","basic_labs"]}else if index%2==0{vec!["vitals","basic_labs"]}else{vec!["basic_labs","vitals"]};
                for(at,test)in requests.iter().enumerate(){let args=json!({"patient_id":"patient_1","test_type":test});let reply=guest.command("ER_DOCTOR",format!("echo '{args}' > /service/order_tests; cat /service/result")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);if at+1==requests.len(){assert_eq!(actual["consultation"]["cardiac"]["result"]["success"],true);assert_eq!(actual["consultation"]["neurology"]["result"]["success"],true);assert_eq!(actual["consultation"]["review"]["success"],true);}}
                if invalid{let reply=guest.command("ER_DOCTOR","echo '{\"patient_id\":\"patient_1\",\"attendees\":[]}' > /service/conduct_case_conference; cat /service/result".into()).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");}
                let args=json!({"patient_id":"patient_1","attendees":"caller-selected conference participants"});let reply=guest.command("ER_DOCTOR",format!("echo '{args}' > /service/conduct_case_conference; cat /service/result")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["attendees"],args["attendees"]);
                assert_eq!(guest.command("ER_DOCTOR","exit 0".into()).await?["exit_status"],0);Ok(())
            };
            for(actor,panel,other)in [("CARDIOLOGIST",["ecg","cardiac_enzymes"],"vitals"),("NEUROLOGIST",["neuro_exam","nerve_conduction"],"ecg")]{async move{
                if invalid{let args=json!({"patient_id":"patient_1","test_type":other});let reply=guest.command(actor,format!("echo '{args}' > /service/order_tests; cat /service/result")).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");}
                let requests=if combined{vec![panel.join(",")]}else if invalid{vec![panel[0].to_owned(),panel[0].to_owned(),panel[1].to_owned()]}else if index%2==0{panel.into_iter().map(str::to_owned).collect()}else{panel.into_iter().rev().map(str::to_owned).collect()};
                for test in requests{let args=json!({"patient_id":"patient_1","test_type":test});let reply=guest.command(actor,format!("echo '{args}' > /service/order_tests; cat /service/result")).await?;assert_eq!(response(&reply)?["success"],true);}
                let args=json!({"patient_id":"patient_1","symptoms":format!("caller {actor} findings")});let reply=guest.command(actor,format!("echo '{args}' > /service/diagnose; cat /service/result")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["symptoms"],args["symptoms"]);assert_eq!(actual["consultation"]["conference"]["success"],true);
                assert_eq!(guest.command(actor,"exit 0".into()).await?["exit_status"],0);Ok(())
            };}
            async{
                if invalid{let reply=guest.command("RADIOLOGIST","echo '{\"patient_id\":\"patient_1\",\"modality\":\"CT\"}' > /service/interpret_imaging; cat /service/result".into()).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");let reply=guest.command("RADIOLOGIST","echo '{\"patient_id\":\"patient_1\",\"modality\":\"PET\"}' > /service/perform_imaging; cat /service/result".into()).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");}
                for(at,(modality,perform))in order.into_iter().enumerate(){let args=json!({"patient_id":"patient_1","modality":modality});let tool=if perform{"perform_imaging"}else{"interpret_imaging"};let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["modality"],modality);if at==3{assert_eq!(actual["consultation"]["conference"]["success"],true);}
if invalid&&perform{let reply=guest.command("RADIOLOGIST",format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;assert_eq!(response(&reply)?["status"],"out_of_order");}}
                assert_eq!(guest.command("RADIOLOGIST","exit 0".into()).await?["exit_status"],0);Ok(())
            };
            async{
                if invalid{let reply=guest.command("PHARMACIST","echo '{\"patient_id\":\"patient_1\",\"medications\":[]}' > /service/review_prescriptions; cat /service/result".into()).await?;assert_eq!(response(&reply)?["status"],"invalid_arguments");}
                let args=json!({"patient_id":"patient_1","medications":"caller supplied medications"});let reply=guest.command("PHARMACIST",format!("echo '{args}' > /service/review_prescriptions; cat /service/result")).await?;let actual=response(&reply)?;assert_eq!(actual["success"],true);assert_eq!(actual["data"]["medications"],args["medications"]);assert_eq!(guest.command("PHARMACIST","exit 0".into()).await?["exit_status"],0);Ok(())
            };
            async{runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(240))).await?;
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
        let audit = crate::multi_specialty_audit::audit(&output.join("events.jsonl"))?;
        save(&output.join("audit.json"), &audit)?;
        if index == 0 {
            assert!(
                audit["independent_effect_overlaps"]
                    .as_u64()
                    .is_some_and(|n| n > 0)
            );
            crate::multi_specialty_audit::check_mutants(
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
