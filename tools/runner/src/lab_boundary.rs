//! The common world's complete finite causal-order domain, through real Wasm.
use crate::qualification::LAB_CASES as CASES;
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
const PREFIXES: [[usize; 4]; 6] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 3, 0, 2],
];
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"]
        .as_str()
        .ok_or("laboratory command output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing actual scientific result: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty scientific response")?
        .map_err(Into::into)
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires the current direct laboratory runtime, wash and source receipt"]
async fn all_causal_orders_and_parallel_instruments_use_actual_wasm() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("lab evidence root")?);
    fs::create_dir_all(&evidence)?;
    let binary = PathBuf::from(std::env::var_os("CHOREOFIX_LAB_WASI_BINARY").ok_or("lab runtime")?)
        .canonicalize()?;
    let receipt =
        PathBuf::from(std::env::var_os("CHOREOFIX_LAB_BUILD_RECEIPT").ok_or("lab build receipt")?);
    let selected = std::env::var("CHOREOFIX_LAB_CASE")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?;
    if selected.is_some_and(|n| n >= CASES) {
        return Err("unknown lab diagnostic condition".into());
    }
    let module = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut reports = Vec::new();
    for case in (0..CASES).filter(|n| selected.is_none_or(|s| s == *n)) {
        let directory = evidence.join(format!(
            "lab-wasm-{}-{case}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&directory)?;
        let build = archive_build_receipt(&binary, &receipt, &directory)?;
        crate::projection::archive(&binary, &directory, "13E")?;
        let delay = if case == 24 { 1.0 } else { 0.01 };
        save(
            &directory.join("case.json"),
            &json!({"task":"13E","depth":0,"case_index":case,
            "declared_cases":CASES,"delay_multiplier":delay,"build":build,"model_calls":0,"scored_benchmark":false,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(&module)?,
            "compiled_projection_sha256":digest(&directory.join("projection.json"))?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/lab_boundary.rs"))?}),
        )?;
        let trace = Trace::create(&directory.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("13E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) = process::guest(&binary, &module, &[], &directory, trace, simulator)?;
        let ((),runtime)=hibana_runtime::race(async{
            async{
                if case<24{
                    if case==0{
                        for (agent,command) in [
                            ("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis/final_validation"),
                            ("BIOLOGIST","echo '{\"compound\":\"compound_purification\"}' > /service/run_bioassay"),
                        ]{
                            let reply=guest.command(agent,command.into()).await?;
                            assert!(reply["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"{reply}");
                        }
                        let reply=guest.command("CHEMIST","echo '{\"sample\":\"other\"}' > /service/run_analysis/raw_materials".into()).await?;
                        assert!(reply["stdout"].as_str().is_some_and(|s|s.contains("Invalid argument")),"{reply}");
                    }
                    let preparations=[
                        ("CHEMIST","/service/run_analysis/raw_materials",json!({"sample":"raw_materials"})),
                        ("BIOLOGIST","/service/run_separation/cell_cultures",json!({"material":"cell_cultures"})),
                        ("CHEMIST","/service/run_separation/compound",json!({"material":"compound_purification"})),
                        ("BIOLOGIST","/service/run_analysis/fluorescence",json!({"sample":"fluorescence"})),
                    ];
                    for at in PREFIXES[case/4]{
                        let (agent,path,args)=&preparations[at];
                        let reply=guest.command(agent,format!("echo '{args}' > {path}; cat {path}")).await?;
                        assert_eq!(response(&reply)?["success"],true,"{reply}");
                    }
                    let sample=guest.command("BIOLOGIST","cat /service/sample".into()).await?;
                    assert!(response(&sample)?["compound"]["data"]["artifact_id"].is_u64());
                    let args=if case%2==0{json!({"compound":"compound_purification"})}
                        else {json!({"compound":"compound_purification","cell_type":"β culture ".repeat(20)})};
                    let assay=guest.command("BIOLOGIST",format!("echo '{args}' > /service/run_bioassay; cat /service/run_bioassay")).await?;
                    assert_eq!(response(&assay)?["success"],true,"{assay}");
                    let source=guest.command("ANALYST","cat /service/bioassay".into()).await?;
                    assert_eq!(response(&source)?["bioassay"]["success"],true,"{source}");
                    let quality=[
                        ("/service/run_analysis/molecular_detail",json!({"sample":"molecular_detail"})),
                        ("/service/run_separation/verification",json!({"material":"verification"})),
                    ];
                    let order=if case%4<2{[0,1]}else{[1,0]};
                    for at in order{
                        let (path,args)=&quality[at];
                        let reply=guest.command("ANALYST",format!("echo '{args}' > {path}")).await?;
                        assert!(!reply["stdout"].as_str().ok_or("quality submission")?.contains("error"),"{reply}");
                    }
                    for at in order{
                        let reply=guest.command("ANALYST",format!("cat {}",quality[at].0)).await?;
                        assert_eq!(response(&reply)?["success"],true,"{reply}");
                    }
                    let quality=guest.command("CHEMIST","cat /service/quality".into()).await?;
                    assert_eq!(response(&quality)?["molecular"]["success"],true,"{quality}");
                    let final_reply=guest.command("CHEMIST","echo '{\"sample\":\"final_validation\"}' > /service/run_analysis/final_validation; cat /service/run_analysis/final_validation".into()).await?;
                    assert_eq!(response(&final_reply)?["success"],true,"{final_reply}");
                    if case==0{
                        let reply=guest.command("CHEMIST","echo '{\"sample\":\"raw_materials\"}' > /service/run_analysis/raw_materials".into()).await?;
                        assert!(reply["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"duplicate: {reply}");
                    }
                    for agent in ["CHEMIST","BIOLOGIST","ANALYST"]{
                        assert_eq!(guest.command(agent,"exit 0".into()).await?["exit_status"],0);
                    }
                }else{
                    async{
                        async{
                            for (path,args) in [
                                ("/service/run_analysis/raw_materials",json!({"sample":"raw_materials"})),
                                ("/service/run_separation/compound",json!({"material":"compound_purification"})),
                            ]{
                                let reply=guest.command("CHEMIST",format!("echo '{args}' > {path}; cat {path}")).await?;
                                assert_eq!(response(&reply)?["success"],true,"{reply}");
                            }
                            let quality=guest.command("CHEMIST","cat /service/quality".into()).await?;
                            assert_eq!(response(&quality)?["molecular"]["success"],true);
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
                            guest.command("BIOLOGIST","cat /service/sample".into()).await?;
                            let reply=guest.command("BIOLOGIST","echo '{\"compound\":\"compound_purification\"}' > /service/run_bioassay; cat /service/run_bioassay".into()).await?;
                            assert_eq!(response(&reply)?["success"],true,"{reply}");
                            assert_eq!(guest.command("BIOLOGIST","exit 0".into()).await?["exit_status"],0);
                        };
                        async{
                            guest.command("ANALYST","cat /service/bioassay".into()).await?;
                            for (path,args) in [
                                ("/service/run_analysis/molecular_detail",json!({"sample":"molecular_detail"})),
                                ("/service/run_separation/verification",json!({"material":"verification"})),
                            ]{
                                let reply=guest.command("ANALYST",format!("echo '{args}' > {path}")).await?;
                                assert!(!reply["stdout"].as_str().ok_or("quality submit")?.contains("error"),"{reply}");
                            }
                            for path in ["/service/run_analysis/molecular_detail","/service/run_separation/verification"]{
                                let reply=guest.command("ANALYST",format!("cat {path}")).await?;
                                assert_eq!(response(&reply)?["success"],true,"{reply}");
                            }
                            assert_eq!(guest.command("ANALYST","exit 0".into()).await?["exit_status"],0);
                        };
                        Ok(())
                    }.await?;
                }
                Result::Ok(())
            };
            async{runtime.await};
        },async{
            hibana_runtime::host::sleep(Duration::from_secs(180)).await?;
            Err("actual laboratory Wasm mechanism timeout".into())
        }).await?;
        save(&directory.join("runtime-result.json"), &runtime)?;
        assert_eq!(runtime["runtime_exit_code"], 0, "{runtime}");
        assert_eq!(runtime["protocol"]["status"], "complete", "{runtime}");
        assert_eq!(runtime["score"]["complete"], true, "{runtime}");
        assert_eq!(runtime["score"]["violations"], json!([]), "{runtime}");
        let (audit, counterexamples) = if case == 0 {
            let (audit, report) = crate::lab_audit::check_mutants(
                &directory.join("events.jsonl"),
                &directory.join("record-counterexamples.json"),
            )?;
            (audit, Some(report))
        } else {
            (
                crate::lab_audit::audit(&directory.join("events.jsonl"))?,
                None,
            )
        };
        assert_eq!(audit["domain_calls"], 8);
        assert_eq!(audit["violations"], 0);
        if case == 24 {
            let events = runtime["score"]["events"]
                .as_array()
                .ok_or("actual effect intervals")?;
            let overlaps = events
                .iter()
                .enumerate()
                .flat_map(|(i, a)| events[i + 1..].iter().map(move |b| (a, b)))
                .filter(|(a, b)| {
                    a["timestamp"]
                        .as_f64()
                        .zip(b["completed_at"].as_f64())
                        .is_some_and(|(s, e)| s < e)
                        && b["timestamp"]
                            .as_f64()
                            .zip(a["completed_at"].as_f64())
                            .is_some_and(|(s, e)| s < e)
                })
                .count();
            assert!(
                overlaps >= 2,
                "independent instruments did not overlap: {overlaps}"
            );
        }
        save(&directory.join("boundary.json"), &audit)?;
        reports.push(json!({"case_index":case,"status":audit["status"],"evidence":directory,
            "domain_calls":audit["domain_calls"],"runtime_binary_sha256":audit["runtime_binary_sha256"],
            "captured_records":audit["dynamic_protocol"]["captured_records"],"counterexamples":counterexamples}));
    }
    save(
        &evidence.join("lab-mechanism.json"),
        &json!({"task":"13E","declared_cases":CASES,
        "complete_finite_domain":selected.is_none(),"cases":reports,"model_calls":0,"scored_benchmark":false,
        "source_conformance_certified":true}),
    )?;
    Ok(())
}
