//! Finite real-Wasm QC inputs. These scripts are mechanism fixtures, not AI recipes.
use crate::qualification::LAB_QUALITY_CASES as CASES;
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
    let text = reply["stdout"]
        .as_str()
        .ok_or("scientific command output")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("missing scientific response: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty scientific response")?
        .map_err(Into::into)
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires current direct QC runtime, wash and checked build receipt"]
async fn actual_wasm_qc_retries_subject_choices_and_instrument_overlap() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("QC evidence root")?);
    fs::create_dir_all(&evidence)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_LAB_QUALITY_WASI_BINARY").ok_or("QC runtime")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_LAB_QUALITY_BUILD_RECEIPT").ok_or("QC build receipt")?,
    );
    let selected = std::env::var("CHOREOFIX_LAB_QUALITY_CASE")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?;
    if selected.is_some_and(|n| n >= CASES) {
        return Err("unknown QC mechanism case".into());
    }
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut reports = Vec::new();
    for case in (0..CASES).filter(|n| selected.is_none_or(|s| s == *n)) {
        let depth = if case < 6 {
            (case / 2) as u8
        } else if case == 6 {
            2
        } else {
            0
        };
        let delay = if case == 6 { 1.0 } else { 0.01 };
        let batch = if case == 7 {
            ""
        } else {
            "β research batch \"free\""
        };
        let directory = evidence.join(format!(
            "lab-quality-wasm-{}-{case}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&directory)?;
        let build = archive_build_receipt(&binary, &receipt, &directory)?;
        crate::projection::archive(&binary, &directory, "13M")?;
        save(
            &directory.join("case.json"),
            &json!({"task":"13M","depth":depth,
   "case_index":case,"declared_cases":CASES,"delay_multiplier":delay,
   "model_calls":0,"scored_benchmark":false,"build":build,
   "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
   "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
   "harness_source_sha256":digest(Path::new("tools/runner/src/lab_quality_boundary.rs"))?}),
        )?;
        let trace = Trace::create(&directory.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("13M", depth, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, &module, &[], &directory, trace, simulator)?;
        let ((),runtime)=hibana_runtime::race(async{
   async{
    async{
     if case==0{
      let reply=guest.command("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis/final_validation".into()).await?;
      assert!(reply["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"{reply}");
     }
     for (path,args) in [
      ("/service/run_synthesis",json!({"compound":batch})),
      ("/service/run_analysis/synthesized_compound",json!({"sample":"synthesized_compound"})),
      ("/service/run_separation/compound_purification",json!({"material":"compound_purification"})),
     ]{
      let reply=guest.command("CHEMIST",format!("echo '{args}' > {path}; cat {path}")).await?;
      assert_eq!(response(&reply)?["success"],true,"{reply}");
     }
     loop{
      let reply=guest.command("CHEMIST","cat /service/qc".into()).await?;
      let qc=response(&reply)?;
      if qc["success"]==true{break;}
      assert_eq!(qc["data"]["status"],"rejected","{reply}");
      let reply=guest.command("CHEMIST","echo '{\"sample\":\"synthesized_compound\"}' > /service/run_analysis/synthesized_compound; cat /service/run_analysis/synthesized_compound".into()).await?;
      assert_eq!(response(&reply)?["success"],true,"{reply}");
     }
     let quality=guest.command("CHEMIST","cat /service/quality".into()).await?;
     assert_eq!(response(&quality)?["chemist_qc"]["success"],true,"{quality}");
     let reply=guest.command("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis/final_validation; cat /service/run_analysis/final_validation".into()).await?;
     assert_eq!(response(&reply)?["success"],true,"{reply}");
     assert_eq!(guest.command("CHEMIST","exit 0".into()).await?["exit_status"],0);
    };
    async{
     for (path,args) in [
      ("/service/run_separation/cell_cultures",json!({"material":"cell_cultures"})),
      ("/service/run_analysis/fluorescence",json!({"sample":"fluorescence"})),
     ]{
      let reply=guest.command("BIOLOGIST",format!("echo '{args}' > {path}; cat {path}")).await?;
      assert_eq!(response(&reply)?["success"],true,"{reply}");
     }
     let sample=guest.command("BIOLOGIST","cat /service/sample".into()).await?;
     assert_eq!(response(&sample)?["synthesis"]["data"]["compound"],batch,"{sample}");
     // A dynamic bad batch uses the existing refusal path without a tool call.
     let reply=guest.command("BIOLOGIST","echo '{\"compound\":\"foreign unprepared batch\"}' > /service/run_bioassay; cat /service/run_bioassay".into()).await?;
     assert_eq!(response(&reply)?["data"]["status"],"invalid_arguments","{reply}");
     loop{
      let args=if case%2==0{json!({"compound":batch})}
       else{json!({"compound":batch,"cell_type":"β culture ".repeat(20)})};
      let reply=guest.command("BIOLOGIST",format!("echo '{args}' > /service/run_bioassay; cat /service/run_bioassay")).await?;
      let assay=response(&reply)?;
      if assay["success"]==true{break;}
      assert_eq!(assay["data"]["status"],"bioassay_failed","{reply}");
     }
     loop{
      let reply=guest.command("BIOLOGIST","cat /service/qc".into()).await?;
      let qc=response(&reply)?;
      if qc["success"]==true{break;}
      assert_eq!(qc["data"]["status"],"rejected","{reply}");
      loop{
       let args=json!({"compound":batch,"cell_type":"retry culture"});
       let reply=guest.command("BIOLOGIST",format!("echo '{args}' > /service/run_bioassay; cat /service/run_bioassay")).await?;
       let assay=response(&reply)?;
       if assay["success"]==true{break;}
       assert_eq!(assay["data"]["status"],"bioassay_failed","{reply}");
      }
     }
     assert_eq!(guest.command("BIOLOGIST","exit 0".into()).await?["exit_status"],0);
    };
    async{
     let reply=guest.command("ANALYST","cat /service/bioassay".into()).await?;
     assert_eq!(response(&reply)?["biologist_qc"]["success"],true,"{reply}");
     let operations=[
      ("/service/run_analysis/molecular_detail",json!({"sample":"molecular_detail"})),
      ("/service/run_separation/verification",json!({"material":"verification"})),
     ];
     let order=if case%2==0{[0,1]}else{[1,0]};
     for at in order{
      let (path,args)=&operations[at];
      let reply=guest.command("ANALYST",format!("echo '{args}' > {path}")).await?;
      assert!(!reply["stdout"].as_str().ok_or("quality submit")?.contains("error"),"{reply}");
     }
     for at in order{
      let reply=guest.command("ANALYST",format!("cat {}",operations[at].0)).await?;
      assert_eq!(response(&reply)?["success"],true,"{reply}");
     }
     assert_eq!(guest.command("ANALYST","exit 0".into()).await?["exit_status"],0);
    };
    async{
     let first=if case%2==0{"CHEMIST"}else{"BIOLOGIST"};
     let other=if first=="CHEMIST"{"BIOLOGIST"}else{"CHEMIST"};
     let mut results=Vec::<Value>::new();
     for subject in [first,other,first,other]{
      if results.iter().any(|r|r["data"]["scientist"]==subject && r["success"]==true){continue;}
      loop{
       let args=json!({"scientist":subject,"data_type":if subject=="CHEMIST"{"purification"}else{"bioassay"}});
       let reply=guest.command("QC_CONTROLLER",format!("echo '{args}' > /service/verify_result/{subject}; cat /service/verify_result/{subject}")).await?;
       let result=response(&reply)?;
       let approved=result["success"]==true;
       assert_eq!(result["data"]["scientist"],subject,"{reply}");
       results.push(result);
       if approved{break;}
       assert_eq!(results.last().unwrap()["data"]["status"],"rejected","{reply}");
       if [3,4,6].contains(&case) && results.iter().filter(|r|r["success"]==false).count()<=2{break;}
      }
     }
     assert_eq!(results.iter().filter(|r|r["success"]==true).count(),2,"{results:?}");
     assert_eq!(guest.command("QC_CONTROLLER","exit 0".into()).await?["exit_status"],0);
    };
    Ok(())
   };
   async{
    let result=runtime.await?;
    save(&directory.join("runtime-result.json"),&result)?;
    if result["runtime_exit_code"]!=0 || result["protocol"]["status"]!="complete"{
     return Err(format!("QC Wasm runtime failed: {result}").into());
    }
    Result::Ok(result)
   };
  },async{
   hibana_runtime::host::sleep(Duration::from_secs(300)).await?;
   Err("actual QC Wasm mechanism deadline".into())
  }).await?;
        assert_eq!(runtime["score"]["complete"], true, "{runtime}");
        assert_eq!(runtime["score"]["violations"], json!([]), "{runtime}");
        let events = runtime["score"]["events"]
            .as_array()
            .ok_or("actual QC effects")?;
        assert_eq!(events.len(), 11 + 3 * usize::from(depth));
        for tool in ["verify_result", "run_bioassay"] {
            assert_eq!(
                events
                    .iter()
                    .filter(|e| e["tool"] == tool && e["success"] == false)
                    .count(),
                usize::from(depth)
            );
        }
        let (audit, counterexamples) = if [0, 4].contains(&case) {
            let (audit, report) = crate::lab_quality_audit::check_mutants(
                &directory.join("events.jsonl"),
                &directory.join("record-counterexamples.json"),
            )?;
            (audit, Some(report))
        } else {
            (
                crate::journal::Evidence::open(&directory.join("events.jsonl"))?
                    .audit(crate::lab_quality_audit::check)?,
                None,
            )
        };
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["input_refusals"], 1);
        if case == 6 {
            let overlap = |a: &Value, b: &Value| -> Result<bool> {
                Ok(a["timestamp"].as_f64().ok_or("effect start")?
                    < b["completed_at"].as_f64().ok_or("effect end")?
                    && b["timestamp"].as_f64().ok_or("effect start")?
                        < a["completed_at"].as_f64().ok_or("effect end")?)
            };
            let quality = events
                .iter()
                .filter(|e| e["agent"] == "ANALYST")
                .collect::<Vec<_>>();
            assert_eq!(quality.len(), 2);
            assert!(
                overlap(quality[0], quality[1])?,
                "actual quality effects did not overlap"
            );
            let independent = events
                .iter()
                .filter(|e| e["tool"] == "verify_result")
                .flat_map(|a| {
                    events
                        .iter()
                        .filter(|e| e["tool"] == "run_bioassay")
                        .map(move |b| overlap(a, b))
                })
                .collect::<Result<Vec<_>>>()?;
            assert!(
                independent.into_iter().any(core::convert::identity),
                "QC and independent biological trial effects did not overlap"
            );
        }
        save(&directory.join("boundary.json"), &audit)?;
        reports.push(json!({"case_index":case,"depth":depth,"evidence":directory,"status":audit["status"],
         "domain_calls":audit["domain_calls"],"runtime_binary_sha256":audit["runtime_binary_sha256"],
         "captured_records":audit["dynamic_protocol"]["captured_records"],"counterexamples":counterexamples,
         "application_effect_correspondence_certified":true}));
    }
    save(
        &evidence.join("lab-quality-mechanism.json"),
        &json!({
         "task":"13M","declared_cases":CASES,"cases":reports,
         "complete_finite_domain":selected.is_none(),"application_effect_correspondence_certified":true,
         "model_calls":0,"scored_runs":0
        }),
    )?;
    Ok(())
}
