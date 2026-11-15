//! Finite real-Wasm input orders and refusal/retry probes, scored by the neutral world.
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
fn arguments(chef: usize, tool: &str) -> Value {
    match tool {
        "prepare_base_dish" => json!({"dish":DISHES[chef]}),
        "prepare_ingredient" => json!({"ingredient":INGREDIENTS[chef],"for_chef":RECIPIENTS[chef]}),
        "combine_dish" => json!({"dish":DISHES[chef],"ingredient":RECEIVED[chef]}),
        _ => panic!("undeclared kitchen fixture operation"),
    }
}
async fn write(guest: &process::Guest, chef: usize, tool: &str, padding: &str) -> Result<()> {
    let args = arguments(chef, tool);
    let reply = guest
        .command(
            CHEFS[chef],
            format!("printf '%s' '{args}{padding}' > /service/{tool}"),
        )
        .await?;
    let stdout = reply["stdout"].as_str().ok_or("kitchen write output")?;
    if stdout.contains("wash:") {
        return Err(format!("kitchen write failed: {reply}").into());
    }
    Ok(())
}
async fn read(guest: &process::Guest, chef: usize, tool: &str) -> Result<Value> {
    let reply = guest
        .command(CHEFS[chef], format!("cat /service/{tool}"))
        .await?;
    let stdout = reply["stdout"].as_str().ok_or("kitchen read output")?;
    let at = stdout
        .find('{')
        .ok_or_else(|| format!("missing kitchen result: {reply}"))?;
    let actual = serde_json::Deserializer::from_str(&stdout[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty kitchen result")??;
    if actual["success"] != true
        || actual["tool_name"] != tool
        || actual["data"]["chef"] != CHEFS[chef]
    {
        return Err(format!("kitchen result differs: {reply}").into());
    }
    Ok(actual)
}
#[hibana_runtime::main]
#[test]
#[ignore = "requires source-qualified operation files, actual wash and neutral world"]
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
    let assignments = selector.map_or_else(
        || {
            (KITCHEN_ORDERS.len() * 8..KITCHEN_CASES)
                .chain(0..KITCHEN_ORDERS.len() * 8)
                .collect::<Vec<_>>()
        },
        |i| vec![i],
    );
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
    let input = PathBuf::from(
        std::env::var_os("CHOREOFIX_KITCHEN_BUILD_RECEIPT").ok_or("kitchen receipt")?,
    );
    archive_build_receipt(&binary, &input, &directory)?;
    let receipt_sha256 = digest(&directory.join("build-receipt.json"))?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for case in assignments {
        let special = case >= KITCHEN_ORDERS.len() * 8;
        let first = if special {
            case - KITCHEN_ORDERS.len() * 8
        } else {
            0
        };
        let order = if special {
            [first, (first + 1) % 3, (first + 2) % 3]
        } else {
            KITCHEN_ORDERS[case / 8]
        };
        let mask = if special { 0 } else { case % 8 };
        let delay = if special { 1.0 } else { 0.01 };
        let output = directory.join(format!("case{case}"));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "12E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"12E","depth":0,"case_index":case,
            "equipment_order":order,"per_chef_ingredient_first_bits":mask,
            "adversarial_inputs":special,"per_dish_early_completion":special,
            "test_delays":if special { json!({"prepare_base_dish":[0.15,0.15],"prepare_ingredient":[0.1,0.1],"combine_dish":[0.0,0.0]}) } else { json!({}) },
            "domain_delay_multiplier":delay,"model_calls":0,"scored_benchmark":false,
            "build_receipt":"../build-receipt.json","build_receipt_sha256":receipt_sha256,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_source_sha256":digest(Path::new("tools/runner/src/kitchen_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/kitchen_audit.rs"))?,
            "input_scope":"all six equipment orders crossed with all eight per-chef input orders; three controlled-delay refusal/recovery and early-completion cases",
            "test_barriers":"scripted fixture only: each special case withholds other bases and own outgoing ingredient until first dish completes"}),
        )?;
        let events = output.join("events.jsonl");
        let trace = Trace::create(&events)?;
        let (simulator, _) = oracle::Simulator::new("12E", 0, delay, trace.clone(), &[])?;
        if special {
            simulator
                .set_test_delays(json!({"prepare_base_dish":[0.15,0.15],
                "prepare_ingredient":[0.1,0.1],"combine_dish":[0.0,0.0]}))
                .await?;
        }
        let (guest, runtime) =
            process::guest(&binary, module, &["12E"], &output, trace, simulator)?;
        let ((), result) = hibana_runtime::race(
            async {
                async {
                    if special {
                        for (i, chef) in CHEFS.into_iter().enumerate() {
                            let early = arguments(i, "combine_dish");
                            let reply = guest.command(chef, format!(
                            "printf '%s' '{early}' | dd of=/service/combine_dish oflag=nonblock"
                        )).await?;
                            let text = reply["stdout"].as_str().ok_or("early input output")?;
                            assert!(
                                text.contains("(os error 2)") || text.contains("(os error 6)"),
                                "{reply}"
                            );
                            for (tool, invalid) in [
                                ("prepare_base_dish", "{\"dish\":\"wrong\"}"),
                                (
                                    "prepare_ingredient",
                                    "{\"ingredient\":\"sauce\",\"for_chef\":\"wrong\"}",
                                ),
                                ("combine_dish", "{"),
                            ] {
                                let reply = guest
                                    .command(
                                        chef,
                                        format!("printf '%s' '{invalid}' > /service/{tool}"),
                                    )
                                    .await?;
                                assert!(
                                    reply["stdout"]
                                        .as_str()
                                        .ok_or("invalid output")?
                                        .contains("(os error 28)"),
                                    "{reply}"
                                );
                            }
                            let private = guest
                                .command(
                                    chef,
                                    "echo 'private kitchen notes' > /work/notes; cat /work/notes"
                                        .into(),
                                )
                                .await?;
                            assert!(
                                private["stdout"]
                                    .as_str()
                                    .ok_or("private output")?
                                    .contains("private kitchen notes")
                            );
                        }
                        write(&guest, first, "prepare_base_dish", &" ".repeat(1200)).await?;
                        // The incoming ingredient is another chef's independent effect.
                        let producer = (first + 1) % 3;
                        write(&guest, producer, "prepare_ingredient", "").await?;
                        read(&guest, first, "prepare_base_dish").await?;
                        read(&guest, producer, "prepare_ingredient").await?;
                        write(&guest, first, "combine_dish", "").await?;
                        read(&guest, first, "combine_dish").await?;
                        // No other base or the first chef's outgoing material was requested yet.
                        write(&guest, first, "prepare_ingredient", "").await?;
                        let last = (first + 2) % 3;
                        write(&guest, last, "prepare_ingredient", "").await?;
                        for i in [producer, last] {
                            write(&guest, i, "prepare_base_dish", "").await?;
                        }
                        for i in [last, producer] {
                            read(&guest, i, "prepare_base_dish").await?;
                        }
                        for i in [last, first] {
                            read(&guest, i, "prepare_ingredient").await?;
                        }
                        for i in [last, producer] {
                            write(&guest, i, "combine_dish", "").await?;
                            read(&guest, i, "combine_dish").await?;
                        }
                        for (i, chef) in CHEFS.into_iter().enumerate() {
                            for tool in ["prepare_base_dish", "prepare_ingredient", "combine_dish"]
                            {
                                let args = arguments(i, tool);
                                let duplicate = guest.command(chef, format!(
                                "printf '%s' '{args}' | dd of=/service/{tool} oflag=nonblock"
                            )).await?;
                                assert!(
                                    duplicate["stdout"]
                                        .as_str()
                                        .ok_or("duplicate output")?
                                        .contains("wash:"),
                                    "{duplicate}"
                                );
                            }
                        }
                    } else {
                        // Every write precedes every result read, including reverse-order reads.
                        for i in order {
                            if mask & (1 << i) != 0 {
                                write(&guest, i, "prepare_ingredient", "").await?;
                            }
                            write(&guest, i, "prepare_base_dish", "").await?;
                            if mask & (1 << i) == 0 {
                                write(&guest, i, "prepare_ingredient", "").await?;
                            }
                        }
                        for i in order.into_iter().rev() {
                            read(&guest, i, "prepare_base_dish").await?;
                            read(&guest, i, "prepare_ingredient").await?;
                        }
                        for i in order.into_iter().rev() {
                            write(&guest, i, "combine_dish", "").await?;
                        }
                        for i in order {
                            read(&guest, i, "combine_dish").await?;
                        }
                    }
                    for chef in CHEFS {
                        assert_eq!(
                            guest.command(chef, "exit 0".into()).await?["exit_status"],
                            0
                        );
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(Duration::from_secs(180)),
        )
        .await?;
        save(&output.join("runtime-result.json"), &result)?;
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let mutants = crate::kitchen_audit::check_mutants(&events, &output.join("record-mutants"))?;
        let audit = &mutants["baseline"];
        assert_eq!(audit["simulator_complete"], result["score"]["complete"]);
        assert_eq!(
            audit["compiled_projection_sha256"],
            digest(&output.join("projection.json"))?
        );
        save(&output.join("audit.json"), audit)?;
        if special {
            assert!(
                audit["base_ingredient_overlap_pairs"]
                    .as_u64()
                    .ok_or("actual overlap")?
                    >= 1
            );
            let scored = result["score"]["events"]
                .as_array()
                .ok_or("actual kitchen effects")?;
            let dish = scored
                .iter()
                .find(|e| e["agent"] == CHEFS[first] && e["tool"] == "combine_dish")
                .ok_or("first actual dish")?;
            let done = dish["completed_at"]
                .as_f64()
                .ok_or("actual dish completion")?;
            for event in scored.iter().filter(|e| {
                e["tool"] == "prepare_base_dish" && e["agent"] != CHEFS[first]
                    || e["tool"] == "prepare_ingredient" && e["agent"] == CHEFS[first]
            }) {
                assert!(
                    done < event["timestamp"]
                        .as_f64()
                        .ok_or("independent effect start")?
                );
            }
        }
        outcomes.push(json!({"case_index":case,"output":output,
            "audit":{"status":audit["status"],"domain_calls":audit["domain_calls"],"violations":audit["violations"],"base_ingredient_overlap_pairs":audit["base_ingredient_overlap_pairs"]},
            "audit_sha256":digest(&output.join("audit.json"))?,
            "record_mutants_rejected":mutants["checked"].as_array().ok_or("mutants")?.len()}));
    }
    assert_eq!(
        outcomes.len(),
        if selector.is_some() { 1 } else { KITCHEN_CASES }
    );
    save(
        &directory.join("report.json"),
        &json!({"status":"passed","model_calls":0,"scored_runs":0,
        "full_declared_domain":selector.is_none(),"assigned_condition":selector,
        "declared_conditions":KITCHEN_CASES,"task":"12E","cases":outcomes}),
    )?;
    Ok(())
}
