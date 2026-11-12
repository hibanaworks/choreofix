//! Scripted real-Wasm inputs; the common world and EPF, not this script, judge effects.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
    qualification::{KITCHEN_CASES, KITCHEN_ORDERS},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const CHEFS: [&str; 3] = ["CHEF_A", "CHEF_B", "CHEF_C"];
const DISHES: [&str; 3] = ["appetizer", "main_course", "dessert"];
const INGREDIENTS: [&str; 3] = ["sauce", "garnish", "glaze"];
const RECIPIENTS: [&str; 3] = ["CHEF_C", "CHEF_A", "CHEF_B"];
const RECEIVED: [&str; 3] = ["garnish", "glaze", "sauce"];
fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("kitchen terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("missing kitchen reply: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty kitchen reply")?
        .map_err(Into::into)
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified kitchen CLI, actual wash and common world"]
async fn all_equipment_and_recipe_input_orders_use_actual_wasm() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let selector = std::env::var("CHOREOFIX_KITCHEN_CASE")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?;
    if selector.is_some_and(|i| i >= KITCHEN_CASES) {
        return Err("kitchen condition outside declared domain".into());
    }
    let assignments = selector.map_or_else(|| (0..KITCHEN_CASES).collect::<Vec<_>>(), |i| vec![i]);
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("kitchen evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "kitchen-wasi-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_KITCHEN_WASI_BINARY").ok_or("kitchen CLI")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_KITCHEN_BUILD_RECEIPT").ok_or("kitchen build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for case in assignments {
        let pipeline = case == KITCHEN_CASES - 1;
        let order = if pipeline {
            [2, 1, 0]
        } else {
            KITCHEN_ORDERS[case / 2]
        };
        let ingredient_first = pipeline || case.is_multiple_of(2);
        let adversarial = pipeline;
        let delay = if pipeline { 1.0 } else { 0.01 };
        let output = directory.join(format!("case{case}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "12E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"12E","depth":0,"case_index":case,"order":order,
            "ingredient_first":ingredient_first,"adversarial_inputs":adversarial,"pipeline":pipeline,
            "domain_delay_multiplier":delay,"model_calls":0,"scored_benchmark":false,"build":receipt,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/kitchen_boundary.rs"))?,"audit_source_sha256":digest(Path::new("tools/runner/src/kitchen_audit.rs"))?,"input_scope":"first twelve cases select both ordinary request orders and all six equipment orders; last case has invalid/private/duplicate inputs, overlapping ingredients and per-dish early completion"}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("12E", 0, delay, trace.clone(), &[])?;
        let (guest, runtime) =
            process::guest(&binary, module, &["12E"], &output, trace, simulator)?;
        let ((),result)=hibana_runtime::race(async {
            async {
                if adversarial {
                    for (i,chef) in CHEFS.into_iter().enumerate() {
                        let args=json!({"dish":DISHES[i],"ingredient":RECEIVED[i]});
                        let early=guest.command(chef,format!("echo '{args}' > /service/combine_dish; cat /service/result")).await?;
                        assert_eq!(response(&early)?["status"],"out_of_order","{early}");
                        let invalid=guest.command(chef,"echo '{\"dish\":\"wrong\"}' > /service/prepare_base_dish; cat /service/result".into()).await?;
                        assert_eq!(response(&invalid)?["status"],"invalid_arguments","{invalid}");
                        let private=guest.command(chef,"echo 'private kitchen notes' > /work/notes; cat /work/notes".into()).await?;
                        assert!(private["stdout"].as_str().ok_or("private output")?.contains("private kitchen notes"));
                    }
                }
                if pipeline {
                    async {
                        let guest=&guest;
                        for i in [0usize,1,2] {
                            async move {
                                let args=json!({"ingredient":INGREDIENTS[i],"for_chef":RECIPIENTS[i]});
                                let reply=guest.command(CHEFS[i],format!("echo '{args}' > /service/prepare_ingredient; cat /service/result")).await?;
                                assert_eq!(response(&reply)?["success"],true,"{reply}");
                                Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
                            };
                        }
                        Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
                    }.await?;
                    for i in order {
                        let base=json!({"dish":DISHES[i]});
                        let padding=if i==order[0]{" ".repeat(1200)}else{String::new()};
                        let reply=guest.command(CHEFS[i],format!("printf '%s' '{base}{padding}' > /service/prepare_base_dish; cat /service/result")).await?;
                        assert_eq!(response(&reply)?["success"],true,"{reply}");
                        let duplicate=guest.command(CHEFS[i],format!("echo '{base}' > /service/prepare_base_dish; cat /service/result")).await?;
                        assert_eq!(response(&duplicate)?["status"],"out_of_order","{duplicate}");
                        let args=json!({"dish":DISHES[i],"ingredient":RECEIVED[i]});
                        let invalid=guest.command(CHEFS[i],format!("echo '{{\"dish\":\"{}\",\"ingredient\":\"wrong\"}}' > /service/combine_dish; cat /service/result",DISHES[i])).await?;
                        assert_eq!(response(&invalid)?["status"],"invalid_arguments","{invalid}");
                        let done=guest.command(CHEFS[i],format!("echo '{args}' > /service/combine_dish; cat /service/result")).await?;
                        assert_eq!(response(&done)?["success"],true,"{done}");
                        let duplicate=guest.command(CHEFS[i],format!("echo '{args}' > /service/combine_dish; cat /service/result")).await?;
                        assert_eq!(response(&duplicate)?["status"],"out_of_order","{duplicate}");
                    }
                } else {
                    for tool in if ingredient_first{["prepare_ingredient","prepare_base_dish"]}else{["prepare_base_dish","prepare_ingredient"]} {
                        for i in order {
                            let args=if tool=="prepare_base_dish"{json!({"dish":DISHES[i]})}else{json!({"ingredient":INGREDIENTS[i],"for_chef":RECIPIENTS[i]})};
                            let reply=guest.command(CHEFS[i],format!("echo '{args}' > /service/{tool}; cat /service/result")).await?;
                            assert_eq!(response(&reply)?["success"],true,"{reply}");
                        }
                    }
                    for i in order.into_iter().rev() {
                        let args=json!({"dish":DISHES[i],"ingredient":RECEIVED[i]});
                        let reply=guest.command(CHEFS[i],format!("echo '{args}' > /service/combine_dish; cat /service/result")).await?;
                        assert_eq!(response(&reply)?["success"],true,"{reply}");
                    }
                }
                for chef in CHEFS {assert_eq!(guest.command(chef,"exit 0".into()).await?["exit_status"],0);}
                Ok::<(),Box<dyn std::error::Error+Send+Sync>>(())
            };
            async {runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(180))).await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let audit = crate::kitchen_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(audit["simulator_complete"], result["score"]["complete"]);
        assert_eq!(
            audit["compiled_projection_sha256"],
            digest(&output.join("projection.json"))?
        );
        save(&output.join("audit.json"), &audit)?;
        let mutants = crate::kitchen_audit::check_mutants(
            &output.join("events.jsonl"),
            &output.join("record-mutants"),
        )?;
        outcomes.push(json!({"case_index":case,"output":output,"audit":audit,"record_mutants_rejected":mutants["checked"].as_array().ok_or("mutants")?.len()}));
    }
    assert_eq!(
        outcomes.len(),
        if selector.is_some() { 1 } else { KITCHEN_CASES }
    );
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,"scored_runs":0,
        "full_declared_domain":selector.is_none(),"assigned_condition":selector,"declared_conditions":KITCHEN_CASES,"task":"12E","cases":outcomes}),
    )?;
    Ok(())
}
