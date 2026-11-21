use super::*;
use std::collections::BTreeSet;

#[hibana_runtime::main]
#[test]
async fn coordination_inputs_come_from_the_actual_world_data() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = std::env::temp_dir().join(format!(
        "coordination-world-inputs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let (world, actual) = Simulator::new("4E", 0, 0.0, trace, &[])?;
    let actors = actual["fixtures"].as_array().ok_or("actual participants")?;
    assert_eq!(actors.len(), 3);
    let backend = actors
        .iter()
        .find(|p| p["agent"] == "BACKEND_DEV")
        .ok_or("backend")?;
    let frontend = actors
        .iter()
        .find(|p| p["agent"] == "FRONTEND_DEV")
        .ok_or("frontend")?;
    let tester = actors
        .iter()
        .find(|p| p["agent"] == "TESTER")
        .ok_or("tester")?;
    assert_eq!(
        backend["calls"][0],
        json!({"tool":"define_api","arguments":{"api_name":"feature_api_spec"}})
    );
    assert_eq!(
        backend["calls"][1],
        json!({"tool":"implement_feature","arguments":{"feature_name":"backend_endpoint"}})
    );
    assert_eq!(
        frontend["calls"][0],
        json!({"tool":"implement_feature","arguments":{"feature_name":"frontend_component"}})
    );
    assert_eq!(
        tester["calls"][0],
        json!({"tool":"run_tests","arguments":{"suite":"integration_tests"}})
    );
    assert!(
        world.observed_events().await?.is_empty(),
        "reading inputs must not run an operation"
    );
    drop(world);
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn multi_round_inputs_preserve_public_operations_without_inventing_a_call_recipe() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmark/descriptions/14H");
    let metadata: Value =
        serde_json::from_slice(&fs::read(source.join("metadata.json")).unwrap()).unwrap();
    let schemas: Value =
        serde_json::from_slice(&fs::read(source.join("tools.json")).unwrap()).unwrap();
    let inputs = json!({"tool_schemas":schemas});
    let recorded = fixture("14H", &metadata, &inputs).unwrap();
    let actors = recorded["fixtures"].as_array().unwrap();
    assert_eq!(actors.len(), 7);
    for actor in actors {
        let name = actor["agent"].as_str().unwrap();
        assert_eq!(actor["resources"], metadata["agent_resources"][name]);
        assert!(actor.get("calls").is_none());
        let operations = actor["operations"].as_array().unwrap();
        assert!(!operations.is_empty());
        for operation in operations {
            assert!(
                schemas
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|tool| tool["function"] == *operation)
            );
            assert!(
                operation["agent_ids"]
                    .as_array()
                    .unwrap()
                    .contains(&actor["agent"])
            );
            assert!(
                operation.get("arguments").is_none(),
                "dynamic data must not be fabricated"
            );
        }
    }
    let chemist = actors
        .iter()
        .find(|actor| actor["agent"] == "CHEMIST")
        .unwrap();
    assert_eq!(
        chemist["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|op| op["name"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["run_hplc_analysis", "run_mass_spec_analysis"])
    );
    assert!(fixture("14H", &metadata, &json!({})).is_err());
    let mut missing = inputs;
    missing["tool_schemas"]
        .as_array_mut()
        .unwrap()
        .retain(|tool| {
            !tool["function"]["agent_ids"]
                .as_array()
                .unwrap()
                .contains(&json!("PROJECT_DIRECTOR"))
        });
    assert!(fixture("14H", &metadata, &missing).is_err());
}

#[test]
fn kitchen_fixtures_describe_the_original_materials_and_physical_resources() {
    let metadata = json!({"agents":["CHEF_A"],
        "agent_resources":{"CHEF_A":["OVEN","STOVETOP"]},
        "tool_resource_map":{"prepare_base_dish":"@agent_resources",
        "prepare_ingredient":[],"combine_dish":[]}});
    let inputs = json!({"_DISHES":[{"chef_id":"CHEF_A","dish_name":"original_dish",
        "receives_ingredient":"original_input"}],
        "_INGREDIENTS":[{"producer":"CHEF_A","name":"original_output","consumer":"CHEF_C"}]});
    let actual = fixture("12E", &metadata, &inputs).unwrap();
    let actor = &actual["fixtures"][0];
    assert_eq!(actor["resources"], json!(["OVEN", "STOVETOP"]));
    assert_eq!(
        actor["calls"],
        json!([
            {"tool":"prepare_base_dish","arguments":{"dish":"original_dish"}},
            {"tool":"prepare_ingredient","arguments":{"ingredient":"original_output","for_chef":"CHEF_C"}},
            {"tool":"combine_dish","arguments":{"dish":"original_dish","ingredient":"original_input"}}
        ])
    );
    let mut missing = inputs.clone();
    missing["_DISHES"] = json!([]);
    assert!(fixture("12E", &metadata, &missing).is_err());
    missing = inputs;
    missing["_INGREDIENTS"][0]["producer"] = json!("CHEF_B");
    assert!(fixture("12E", &metadata, &missing).is_err());
}

#[test]
fn physical_resource_metadata_follows_actual_module_arguments_and_rejects_missing_bindings() {
    let metadata = json!({"agent_resources":{"BUILDER_A":["CORE_LIB"]},
        "tool_resource_map":{"compile_module":{"argument":"module_name","values":{
            "module_a":["CORE_LIB","SHARED_TYPES"],"module_b":["SHARED_TYPES"]}},
            "link_modules":[],"legacy":"@agent_resources"}});
    let a = json!({"tool":"compile_module","arguments":{"module_name":"Module A"}});
    let b = json!({"tool":"compile_module","arguments":{"module_name":"module_b"}});
    assert_eq!(
        resources_for_call(&metadata, "BUILDER_A", &a).unwrap(),
        ["CORE_LIB", "SHARED_TYPES"]
    );
    assert_eq!(
        resources_for_call(&metadata, "BUILDER_A", &b).unwrap(),
        ["SHARED_TYPES"]
    );
    assert!(
        resources_for_call(
            &metadata,
            "BUILDER_A",
            &json!({"tool":"compile_module",
        "arguments":{"module_name":"missing"}})
        )
        .is_err()
    );
    assert!(
        resources_for_call(
            &metadata,
            "INTEGRATOR",
            &json!({"tool":"link_modules","arguments":{}})
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        resources_for_call(
            &metadata,
            "BUILDER_A",
            &json!({"tool":"legacy","arguments":{}})
        )
        .unwrap(),
        ["CORE_LIB"]
    );
}
use crate::evidence::save;
use hibana_runtime::host::deadline;
use std::io;
use std::time::Duration;

#[hibana_runtime::main]
#[test]
#[ignore = "runs the source-pinned common 13M world through Rust; no Wasm or model"]
async fn qc_world_binds_real_inputs_and_recovers_at_both_subject_orders() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    choreofix_inputs::verify_benchmark(&root)?;
    let directory = root
        .join("output/experiments/lab-quality-development-v1")
        .join(format!(
            "rust-world-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
    fs::create_dir(&directory)?;
    let inputs = crate::sources::inputs(&root)?;
    let before = crate::sources::hashes(&inputs)?;
    for (name, path) in &inputs {
        let saved = directory.join(name);
        fs::create_dir_all(saved.parent().ok_or("world source parent")?)?;
        fs::copy(path, saved)?;
    }
    save(
        &directory.join("source.json"),
        &json!({"files":before,
        "harness_sha256":crate::evidence::digest(&std::env::current_exe()?)?,
        "scope":"common-world input/response qualification; not Endpoint, Wasm or model execution"}),
    )?;
    let mut outcomes = Vec::new();
    for depth in 0..=2 {
        for order in [["CHEMIST", "BIOLOGIST"], ["BIOLOGIST", "CHEMIST"]] {
            let case = directory.join(format!("depth{depth}-{}", order[0]));
            fs::create_dir(&case)?;
            let trace = Trace::create(&case.join("events.jsonl"))?;
            let (simulator, fixture) = Simulator::new("13M", depth, 0.0, trace, &[])?;
            assert_eq!(
                fixture["fixtures"]
                    .as_array()
                    .ok_or("QC scientist fixtures")?
                    .len(),
                4
            );
            let qc = fixture["fixtures"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["agent"] == "QC_CONTROLLER")
                .ok_or("QC participant")?;
            assert_eq!(qc["resources"], json!(["FUME_HOOD", "SPECTROMETER"]));
            // IDs correlate external RPC replies; they grant no task operation.
            let mut id = 0;
            for (tool, actor, arguments) in [
                (
                    "run_synthesis",
                    "CHEMIST",
                    json!({"compound":"free_named_batch"}),
                ),
                (
                    "run_analysis",
                    "CHEMIST",
                    json!({"sample":"synthesized_compound"}),
                ),
                (
                    "run_separation",
                    "CHEMIST",
                    json!({"material":"compound_purification"}),
                ),
                (
                    "run_separation",
                    "BIOLOGIST",
                    json!({"material":"cell_cultures"}),
                ),
                (
                    "run_analysis",
                    "BIOLOGIST",
                    json!({"sample":"fluorescence"}),
                ),
            ] {
                let reply = simulator.call(json!({"type":"execute","id":id,"tool":tool,"agent":actor,"arguments":arguments})).await?;
                assert_eq!(reply["result"]["success"], true, "{reply}");
                id += 1;
            }
            loop {
                let reply = simulator.call(json!({"type":"execute","id":id,"agent":"BIOLOGIST",
                    "tool":"run_bioassay","arguments":{"compound":"free_named_batch","cell_type":"arbitrary_culture"}})).await?;
                id += 1;
                if reply["result"]["success"] == true {
                    break;
                }
                assert_eq!(
                    reply["result"]["data"]["status"], "bioassay_failed",
                    "{reply}"
                );
                assert!(id < 20, "world failure injection did not allow recovery");
            }
            for scientist in order {
                loop {
                    let reply = simulator.call(json!({"type":"execute","id":id,"agent":"QC_CONTROLLER","tool":"verify_result",
                        "arguments":{"scientist":scientist,"data_type":if scientist=="CHEMIST" {"purification"} else {"bioassay"}}})).await?;
                    id += 1;
                    if reply["result"]["success"] == true {
                        break;
                    }
                    assert_eq!(reply["result"]["data"]["status"], "rejected", "{reply}");
                    let (tool, args) = if scientist == "CHEMIST" {
                        ("run_analysis", json!({"sample":"synthesized_compound"}))
                    } else {
                        ("run_bioassay", json!({"compound":"free_named_batch"}))
                    };
                    let revised = simulator.call(json!({"type":"execute","id":id,"agent":scientist,"tool":tool,"arguments":args})).await?;
                    id += 1;
                    assert_eq!(revised["result"]["success"], true, "{revised}");
                    assert!(id < 40, "world QC did not allow recovery");
                }
            }
            for (tool, actor, args) in [
                (
                    "run_separation",
                    "ANALYST",
                    json!({"material":"verification"}),
                ),
                (
                    "run_analysis",
                    "ANALYST",
                    json!({"sample":"molecular_detail"}),
                ),
                (
                    "run_analysis",
                    "CHEMIST",
                    json!({"sample":"final_validation"}),
                ),
            ] {
                let reply = simulator.call(json!({"type":"execute","id":id,"agent":actor,"tool":tool,"arguments":args})).await?;
                id += 1;
                assert_eq!(reply["result"]["success"], true, "{reply}");
            }
            let score = simulator.call(json!({"type":"score","id":id})).await?;
            assert_eq!(score["complete"], true, "{score}");
            assert_eq!(score["violations"], json!([]), "{score}");
            save(&case.join("result.json"), &score)?;
            outcomes.push(json!({"depth":depth,"first":order[0],"effects":id,"complete":true}));
        }
    }
    assert_eq!(
        crate::sources::hashes(&crate::sources::inputs(&root)?)?,
        before
    );
    save(
        &directory.join("summary.json"),
        &json!({"status":"passed","conditions":outcomes.len(),"outcomes":outcomes,
        "source_unchanged":true,"model_calls":0,"wasm":false,"scored_benchmark":false}),
    )?;
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::scope]
async fn sensitivity_case<const N: usize>(
    directory: &Path,
    name: &str,
    calls: [(usize, usize); N],
    conflict: bool,
) -> Result<Value> {
    let case = directory.join(name);
    fs::create_dir(&case)?;
    let trace = Trace::create(&case.join("events.jsonl"))?;
    let (simulator, fixture) = Simulator::new("1H", 0, 1.0, trace, &[])?;
    let mut replies: [Value; N] = core::array::from_fn(|_| Value::Null);
    let mut calls = calls.into_iter().enumerate();
    let inputs = replies.each_mut().map(|output| {
        let (id, (agent, tool)) = calls.next().expect("one input per reply");
        let actor = &fixture["fixtures"][agent];
        let call = &actor["calls"][tool];
        let message = json!({"type":"execute","id":id,"agent":actor["agent"],
            "tool":call["tool"],"arguments":call["arguments"]});
        (message, output)
    });
    async {
        let simulator = &simulator;

        for (message, output) in inputs {
            async move {
                *output = simulator.call(message).await?;
            };
        }
        Ok(())
    }
    .await?;
    let latest_start = replies
        .iter()
        .map(|r| r["started_ns"].as_u64().unwrap())
        .max()
        .unwrap();
    let earliest_end = replies
        .iter()
        .map(|r| r["finished_ns"].as_u64().unwrap())
        .min()
        .unwrap();
    assert!(latest_start < earliest_end, "{name} calls did not overlap");
    let score = simulator.call(json!({"type":"score"})).await?;
    let violations = score["violations"]
        .as_array()
        .ok_or("original violations")?;
    if conflict {
        assert!(
            violations
                .iter()
                .any(|v| v["violation_type"] == "concurrent_resource_use")
        );
    } else {
        assert!(violations.is_empty(), "independent work rejected: {score}");
    }
    let outcome = json!({"case":name,"replies":replies.as_slice(),"score":score,
        "overlap":true,"expected_resource_conflict":conflict,
        "model_calls":0,"scored_benchmark":false,
        "scope":"original simulator sensitivity; no Hibana protocol is under test"});
    save(&case.join("result.json"), &outcome)?;
    Ok(outcome)
}

#[hibana_runtime::main]
#[test]
#[ignore = "checks original simulator resource sensitivity with real concurrent calls"]
async fn codebase_simulator_allows_independent_work_and_reports_shared_module_overlap() -> Result<()>
{
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = std::env::var_os("CHOREOFIX_EVIDENCE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("output").to_owned());
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "codebase-simulator-sensitivity-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let mut outcomes = Vec::new();
    outcomes.push(
        sensitivity_case(
            &directory,
            "disjoint_commits",
            [(0, 2), (2, 2), (4, 2)],
            false,
        )
        .await?,
    );
    outcomes.push(sensitivity_case(&directory, "shared_database", [(0, 2), (1, 2)], true).await?);
    outcomes
        .push(sensitivity_case(&directory, "test_and_next_commit", [(0, 3), (1, 2)], false).await?);
    save(&directory.join("summary.json"), &json!({"cases":outcomes}))?;
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "calls the original registry error path through the Rust callback"]
async fn original_tool_exception_reaches_rust_without_effects_or_a_hung_reply() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = Path::new("output").join(format!(
        "upstream-rust-error-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let (simulator, _) = Simulator::new("6E", 0, 0.0, trace, &[])?;
    let result: io::Result<Result<Value>> = hibana_runtime::race(
        async {
            Ok(simulator
                .call(json!({"type":"execute","id":1,"agent":"DEVELOPER_A",
            "tool":"not_an_original_tool","arguments":{}}))
                .await)
        },
        deadline(Duration::from_secs(5)),
    )
    .await;
    let result = result?;
    assert!(result.is_err());
    assert!(simulator.observed_events().await?.is_empty());
    save(
        &directory.join("result.json"),
        &json!({"kind":"upstream_api_error_test",
        "model_calls":0,"scored_benchmark":false,"error":result.unwrap_err().to_string(),
        "events":simulator.observed_events().await?}),
    )?;
    println!("{}", directory.display());
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn cancelled_replies_leave_later_calls_usable() -> Result<()> {
    let (send, receive) = mpsc::channel::<Request>();
    let simulator = Simulator(send);
    let worker = std::thread::spawn(move || -> Result<()> {
        let (message, reply) = receive.recv()?;
        assert_eq!(message["id"], 1);
        std::thread::sleep(Duration::from_millis(20));
        wire::respond(&reply, message["id"].clone(), Ok(json!({"id":1})))?;
        let (message, reply) = receive.recv()?;
        wire::respond(&reply, message["id"].clone(), Ok(message))?;
        Ok(())
    });
    let error = hibana_runtime::race(
        simulator.call(json!({"id":1})),
        deadline(Duration::from_millis(1)),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::TimedOut
    );
    let result = hibana_runtime::race(
        simulator.call(json!({"id":2})),
        deadline(Duration::from_secs(2)),
    )
    .await?;
    assert_eq!(result, json!({"id":2}));
    worker.join().map_err(|_| "reply worker panicked")??;
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn an_external_reply_wakes_a_pending_call_without_a_shared_rust_waker() -> Result<()> {
    let (send, receive) = mpsc::channel::<Request>();
    let simulator = Simulator(send);
    let worker = std::thread::spawn(move || -> Result<()> {
        let (message, reply) = receive.recv()?;
        std::thread::sleep(Duration::from_millis(5));
        wire::respond(&reply, message["id"].clone(), Ok(message))?;
        Ok(())
    });
    let result = hibana_runtime::race(
        simulator.call(json!({"payload":"owned response"})),
        deadline(Duration::from_secs(2)),
    )
    .await?;
    assert_eq!(result, json!({"payload":"owned response"}));
    worker.join().map_err(|_| "reply worker panicked")??;
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "initializes the common simulator environments at all 144 task-depth combinations"]
async fn all_original_environment_classes_load_from_verified_sources() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let manifest: Value = serde_json::from_slice(&fs::read("benchmark/simulator-manifest.json")?)?;
    let directory = Path::new("output").join(format!(
        "upstream-rust-api-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let mut results = Vec::new();
    for task in manifest["tasks"].as_array().ok_or("48 task records")? {
        let id = task["id"].as_str().ok_or("task id")?;
        for depth in 0..=2 {
            let (simulator, fixture) = Simulator::new(id, depth, 0.0, trace.clone(), &[])?;
            let original: Value = serde_json::from_slice(&fs::read(
                Path::new("benchmark/descriptions")
                    .join(id)
                    .join("metadata.json"),
            )?)?;
            assert_eq!(
                fixture["fixtures"]
                    .as_array()
                    .ok_or("fixtures")?
                    .iter()
                    .map(|v| v["agent"].clone())
                    .collect::<Vec<_>>(),
                original["agents"]
                    .as_array()
                    .ok_or("original agents")?
                    .clone()
            );
            let score = simulator.call(json!({"type":"score"})).await?;
            assert_eq!(score["events"], json!([]));
            assert_eq!(score["violations"], json!([]));
            results.push(json!({"task":id,"depth":depth,"fixture":fixture,"initial_score":score}));
        }
    }
    assert_eq!(results.len(), 144);
    save(
        &directory.join("initializations.json"),
        &json!({"cases":results,"model_calls":0,
        "scored_benchmark":false,"scope":"upstream import, construction and initial scorer only; no task port acceptance"}),
    )?;
    println!("{}", directory.display());
    Ok(())
}

#[test]
fn multi_specialty_fixture_preserves_both_modalities_and_the_actual_case() -> Result<()> {
    let metadata: Value = serde_json::from_str(include_str!(
        "../../../../benchmark/descriptions/5M/metadata.json"
    ))?;
    let inputs = json!({
        "_ASSESS_STEPS":[{"agent_id":"ER_DOCTOR","patient_id":"source_patient"}],
        "_ORDER_TESTS_STEPS":[{"agent_id":"ER_DOCTOR","patient_id":"source_patient","tests":["vitals","basic_labs"]},{"agent_id":"CARDIOLOGIST","patient_id":"source_patient","tests":["ecg","cardiac_enzymes"]},{"agent_id":"NEUROLOGIST","patient_id":"source_patient","tests":["neuro_exam","nerve_conduction"]}],
        "_IMAGING_STEPS":[{"agent_id":"RADIOLOGIST","patient_id":"source_patient","modality":"CT"},{"agent_id":"RADIOLOGIST","patient_id":"source_patient","modality":"MRI"}],
        "_INTERPRET_IMAGING_STEPS":[{"agent_id":"RADIOLOGIST","patient_id":"source_patient","modality":"CT"},{"agent_id":"RADIOLOGIST","patient_id":"source_patient","modality":"MRI"}],
        "_DIAGNOSE_STEPS":[{"agent_id":"CARDIOLOGIST","patient_id":"source_patient"},{"agent_id":"NEUROLOGIST","patient_id":"source_patient"}],
        "_PRESCRIPTION_REVIEW_STEPS":[{"agent_id":"PHARMACIST","patient_id":"source_patient"}],
        "_CASE_CONFERENCE_STEPS":[{"agent_id":"ER_DOCTOR","case_id":"source_patient"}]
    });
    let actual = fixture("5M", &metadata, &inputs)?;
    let actors = actual["fixtures"].as_array().ok_or("five actors")?;
    assert_eq!(actors.len(), 5);
    let radiologist = actors
        .iter()
        .find(|a| a["agent"] == "RADIOLOGIST")
        .ok_or("radiologist")?;
    assert_eq!(radiologist["resources"], json!(["CT_SCANNER", "MRI"]));
    for tool in ["perform_imaging", "interpret_imaging"] {
        assert_eq!(
            radiologist["calls"]
                .as_array()
                .ok_or("imaging assignments")?
                .iter()
                .filter(|c| c["tool"] == tool)
                .map(|c| c["arguments"]["modality"].clone())
                .collect::<Vec<_>>(),
            vec![json!("CT"), json!("MRI")]
        );
    }
    let er = actors
        .iter()
        .find(|a| a["agent"] == "ER_DOCTOR")
        .ok_or("ER doctor")?;
    assert_eq!(er["resources"], json!(["CONSULTATION_ROOM"]));
    assert_eq!(
        er["calls"]
            .as_array()
            .ok_or("ER assignments")?
            .iter()
            .find(|c| c["tool"] == "conduct_case_conference")
            .ok_or("conference")?["arguments"],
        json!({"patient_id":"source_patient"})
    );
    assert!(
        actors.iter().all(
            |a| a["calls"].as_array().is_some_and(|calls| !calls.is_empty()
                && calls
                    .iter()
                    .all(|c| c["arguments"]["patient_id"] == "source_patient"))
        )
    );
    Ok(())
}

#[test]
fn critical_care_assignments_preserve_both_patients_and_all_actual_tools() -> Result<()> {
    let metadata: Value = serde_json::from_str(include_str!(
        "../../../../benchmark/descriptions/5H/metadata.json"
    ))?;
    let imaging = json!([
        {"agent_id":"RADIOLOGIST","patient_id":"patient_a","modality":"CT"},
        {"agent_id":"RADIOLOGIST","patient_id":"patient_b","modality":"CT"},
        {"agent_id":"RADIOLOGIST","patient_id":"patient_b","modality":"MRI"}
    ]);
    let mut inputs = json!({
        "_ASSESS_STEPS":[{"agent_id":"ER_DOCTOR_A","patient_id":"patient_a"},
            {"agent_id":"ER_DOCTOR_B","patient_id":"patient_b"}],
        "_ORDER_TESTS_STEPS":[{"agent_id":"ER_DOCTOR_A","patient_id":"patient_a",
            "tests":["trauma_panel","blood_type","coag"]},
            {"agent_id":"ER_DOCTOR_B","patient_id":"patient_b","tests":["stroke_panel","ct_angio"]}],
        "_IMAGING_STEPS":imaging,"_INTERPRET_IMAGING_STEPS":imaging,
        "_DIAGNOSE_STEPS":[{"agent_id":"ER_DOCTOR_A","patient_id":"patient_a"},
            {"agent_id":"ER_DOCTOR_B","patient_id":"patient_b"}],
        "_ANESTHESIA_STEPS":[{"agent_id":"ANESTHESIOLOGIST","patient_id":"patient_a"}],
        "_SURGERY_STEPS":[{"agent_id":"SURGEON","patient_id":"patient_a","procedure":"trauma_surgery"}],
        "_ICU_STEPS":[{"agent_id":"ICU_COORDINATOR","patient_id":"patient_b"}],
        "_PRESCRIPTION_REVIEW_STEPS":[{"agent_id":"PHARMACIST","patient_id":"patient_A_and_B"}]
    });
    let declared = fixture("5H", &metadata, &inputs)?;
    let actors = declared["fixtures"]
        .as_array()
        .ok_or("seven actual participants")?;
    assert_eq!(actors.len(), 7);
    assert_eq!(
        actors
            .iter()
            .map(|a| a["calls"].as_array().map_or(0, Vec::len))
            .sum::<usize>(),
        16
    );
    let actor = |name| {
        actors
            .iter()
            .find(|a| a["agent"] == name)
            .ok_or("assigned actor")
    };
    assert_eq!(
        actor("ICU_COORDINATOR")?["resources"],
        json!(["ICU_BEDS", "VENTILATOR"])
    );
    assert_eq!(
        actor("RADIOLOGIST")?["resources"],
        json!(["CT_SCANNER", "MRI"])
    );
    assert_eq!(
        actor("RADIOLOGIST")?["calls"]
            .as_array()
            .ok_or("six imaging calls")?
            .len(),
        6
    );
    assert_eq!(
        actor("SURGEON")?["calls"][0]["arguments"],
        json!({"patient_id":"patient_a","procedure":"trauma_surgery"})
    );
    assert_eq!(
        actor("PHARMACIST")?["calls"][0]["arguments"],
        json!({"patient_id":"patient_A_and_B"})
    );
    for (name, patient) in [("ER_DOCTOR_A", "patient_a"), ("ER_DOCTOR_B", "patient_b")] {
        assert!(
            actor(name)?["calls"]
                .as_array()
                .ok_or("patient calls")?
                .iter()
                .all(|call| call["arguments"]["patient_id"] == patient)
        );
    }
    inputs
        .as_object_mut()
        .ok_or("input tables")?
        .remove("_SURGERY_STEPS");
    assert!(
        fixture("5H", &metadata, &inputs).is_err(),
        "a declared tool cannot silently lose its assigned inputs"
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn staged_release_inputs_preserve_all_seven_agents_and_original_operation_targets()
-> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = std::env::temp_dir().join(format!(
        "staged-integration-inputs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Trace::create(&directory.join("events.jsonl"))?;
    let (world, fixture) = Simulator::new("6H", 0, 0.0, trace, &[])?;
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("seven staged-release participants")?;
    assert_eq!(actors.len(), 7);
    assert_eq!(
        actors
            .iter()
            .map(|a| a["calls"].as_array().unwrap().len())
            .sum::<usize>(),
        24
    );
    let actor = |name| {
        actors
            .iter()
            .find(|a| a["agent"] == name)
            .ok_or("original actor")
    };
    for (name, feature) in [
        ("DEVELOPER_A", "user_authentication"),
        ("DEVELOPER_B", "payment_processing"),
        ("DEVELOPER_C", "notification_system"),
    ] {
        assert_eq!(
            actor(name)?["calls"],
            json!([
                {"tool":"implement_feature","arguments":{"feature_name":feature}},
                {"tool":"merge_code","arguments":{"branch":feature}}
            ])
        );
        assert_eq!(actor(name)?["resources"], json!(["REPO"]));
    }
    let release = actor("RELEASE_MANAGER")?;
    assert_eq!(release["resources"], json!(["STAGING_ENV"]));
    let calls = release["calls"]
        .as_array()
        .ok_or("original release operations")?;
    for feature in [
        "user_authentication",
        "payment_processing",
        "notification_system",
    ] {
        for tool in ["deploy_service", "validate_staging", "rollback_service"] {
            assert_eq!(
                calls
                    .iter()
                    .filter(|c| c["tool"] == tool && c["arguments"] == json!({"service":feature}))
                    .count(),
                1
            );
        }
    }
    assert_eq!(actor("REVIEWER")?["resources"], json!([]));
    assert_eq!(actor("TEST_WRITER")?["resources"], json!(["TEST_ENV"]));
    assert_eq!(actor("CI_RUNNER")?["resources"], json!(["BUILD_SERVER"]));
    assert_eq!(
        actor("CI_RUNNER")?["calls"],
        json!([
            {"tool":"run_tests","arguments":{"suite":"full_suite_after_auth"}},
            {"tool":"run_tests","arguments":{"suite":"full_suite_after_payment"}},
            {"tool":"run_tests","arguments":{"suite":"full_suite_after_notification"}}
        ])
    );
    assert!(
        world.observed_events().await?.is_empty(),
        "reading assignments must not execute a tool or prescribe its order"
    );
    drop(world);
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the built wafer runtime and wash guest"]
async fn wash_uses_the_same_reference_wafer_for_both_spot_checks() -> Result<()> {
    use crate::{
        evidence::{archive_build_receipt, digest},
        process, projection,
    };
    use std::path::PathBuf;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let output =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("wafer evidence root")?);
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_WAFER_WASI_BINARY").ok_or("wafer runtime")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_WAFER_BUILD_RECEIPT").ok_or("wafer build receipt")?,
    );
    let wash = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    fs::create_dir(&output)?;
    archive_build_receipt(&binary, &receipt, &output)?;
    let film = ("run_deposition", "film_integrity_check");
    let etch = ("run_etch_process", "etch_uniformity_check");
    let recipients = [
        ("LITHOGRAPHER", "/service/lot_lithographer"),
        ("DEPOSITOR", "/service/lot_depositor"),
        ("ETCHER", "/service/lot_etcher"),
    ];
    for (case, order, retry, delivery, deferred_reads) in [
        ("film-first", [film, etch], false, [0, 1, 2], false),
        ("etch-first", [etch, film], false, [1, 2, 0], false),
        ("overlay-retry", [film, etch], true, [2, 0, 1], false),
        ("litho-etch-dep", [etch, film], false, [0, 2, 1], true),
        ("dep-litho-etch", [film, etch], false, [1, 0, 2], true),
        ("etch-dep-litho", [etch, film], false, [2, 1, 0], true),
        ("reactive-etch-retry", [film, etch], false, [2, 0, 1], false),
    ] {
        let directory = output.join(case);
        fs::create_dir(&directory)?;
        let verdict = if retry { "hold for review" } else { "release" };
        let journal = directory.join("events.jsonl");
        let artifact = projection::archive(&binary, &directory, "15E")?;
        let trace = Trace::create(&journal)?;
        let retry_control = case == "litho-etch-dep";
        let retry_etch = case == "reactive-etch-retry";
        let failures: &[(&str, u32)] = if retry {
            &[("run_metrology_measurement", 0)]
        } else if retry_control {
            &[("check_monitoring", 0)]
        } else {
            &[]
        };
        let (world, fixture) = Simulator::new("15E", 0, 1.0, trace.clone(), failures)?;
        let actors = fixture["fixtures"].as_array().ok_or("wafer participants")?;
        assert_eq!(actors.len(), 6);
        assert!(actors.iter().all(|a| {
            a.get("calls").is_none()
                && a["operations"]
                    .as_array()
                    .is_some_and(|ops| !ops.is_empty())
        }));
        let retry_pattern = case == "dep-litho-etch";
        let retry_endpoint = case == "etch-dep-litho";
        if retry_pattern {
            world.call(json!({"type":"test_failure","tool":"run_etch_process","agent":"LITHOGRAPHER","call_index":0})).await?;
        }
        if retry_endpoint {
            world.call(json!({"type":"test_failure","tool":"check_monitoring","agent":"LITHOGRAPHER","call_index":1})).await?;
        }
        if retry_etch {
            world.call(json!({"type":"test_failure","tool":"run_etch_process","agent":"ETCHER","call_index":0})).await?;
        }
        world
            .set_test_delays(json!({"run_stepper_exposure":[0.5,0.5],
            "run_metrology_measurement":[0.0,0.0],"check_monitoring":[0.0,0.0],
            "run_etch_process":[0.0,0.0],"run_deposition":[0.5,0.5]}))
            .await?;
        let guest_effects = 19
            + 4 * usize::from(retry)
            + 4 * usize::from(retry_control)
            + usize::from(retry_etch)
            + usize::from(retry_pattern)
            + usize::from(retry_endpoint);
        let exposure_first = match case {
            "dep-litho-etch" => Some(true),
            "etch-dep-litho" => Some(false),
            _ => None,
        };
        let (guest, runtime) =
            process::guest(&binary, &wash, &[], &directory, trace, world.clone())?;
        let (rejected_confirmations, outcome) = hibana_runtime::race(
            async {
                async {
                    let agent = "INTEGRATION_ENGINEER";
                    let controller = "PROCESS_CONTROLLER";
                    let measurer = "METROLOGY_ENGINEER";
                    let materials = ["pattern", "film", "etch"];
                    let dimensions = [
                        "cd_measurement",
                        "thickness_measurement",
                        "profile_measurement",
                    ];
                    let mut measure_order = delivery;
                    let schedule: Vec<_> = if deferred_reads {
                        measure_order.rotate_left(1);
                        delivery
                            .into_iter()
                            .map(|i| (i, true))
                            .chain(measure_order.into_iter().map(|i| (i, false)))
                            .collect()
                    } else {
                        delivery
                            .into_iter()
                            .flat_map(|i| [(i, true), (i, false)])
                            .collect()
                    };
                    let mut inputs = [Value::Null, Value::Null, Value::Null];
                    let mut readings = [Value::Null, Value::Null, Value::Null];
                    let expose = "echo '{\"process\":\"photo_exposure\"}' > /service/run_stepper_exposure";
                    let mut first_measure = true;
                    for (i, prepare) in schedule {
                        let who = recipients[i].0;
                        let material = materials[i];
                        let dimension = dimensions[i];
                        if prepare {
                            let mut receipts = Vec::new();
                            if who == "LITHOGRAPHER" {
                                if exposure_first.is_none() {
                                    let wrong = guest.command(who, "echo '{\"process\":\"overlay_alignment\"}' | dd of=/service/run_stepper_exposure oflag=nonblock".into()).await?;
                                    assert!(wrong["stdout"].as_str().is_some_and(|s| s.contains("Invalid argument")), "{wrong}");
                                    assert_eq!(guest.command(who, expose.into()).await?["stdout"], "wash> ");
                                }
                                let monitor_path="/service/check_monitoring/exposure_dose";
                                let sample=format!("echo '{{\"check_type\":\"exposure_dose\"}}' > {monitor_path}; cat {monitor_path}");
                                let mut reading=shell_json_result(&guest.command(who,sample.clone()).await?)?;
                                if retry_control {
                                    assert_eq!(reading["data"]["status"],"check_failed");
                                    let early=guest.command(who,"echo ready | dd of=/service/fab_ready oflag=nonblock".into()).await?;
                                    assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied") || text.contains("Resource temporarily unavailable")), "{early}");
                                    reading=shell_json_result(&guest.command(who,sample).await?)?;
                                }
                                assert_eq!(reading["success"],true,"{reading}");
                                assert_eq!(reading["data"]["check_type"],"exposure_dose");
                                let result = shell_json_result(&guest.command(who, "cat /service/run_stepper_exposure".into()).await?)?;
                                assert_eq!(result["success"], true, "{result}");
                                assert_eq!(result["data"]["process"], "photo_exposure");
                                assert_eq!(reading["data"]["inputs"],json!([result["data"]["receipt"]]));
                                let etch = r#"echo '{"process":"pattern_transfer_etch"}' > /service/run_etch_process; cat /service/run_etch_process"#;
                                let mut etched = shell_json_result(&guest.command(who, etch.into()).await?)?;
                                if retry_pattern {
                                    assert_eq!(etched["data"]["status"], "etch_failed", "{etched}");
                                    etched = shell_json_result(&guest.command(who, etch.into()).await?)?;
                                }
                                assert_eq!(etched["success"], true, "{etched}");
                                assert_eq!(etched["data"]["inputs"], json!([result["data"]["receipt"], reading["data"]["receipt"]]));
                                let inspect = r#"echo '{"check_type":"etch_endpoint"}' > /service/check_monitoring/etch_endpoint; cat /service/check_monitoring/etch_endpoint"#;
                                let mut endpoint = shell_json_result(&guest.command(who, inspect.into()).await?)?;
                                if retry_endpoint {
                                    assert_eq!(endpoint["data"]["status"], "check_failed", "{endpoint}");
                                    endpoint = shell_json_result(&guest.command(who, inspect.into()).await?)?;
                                }
                                assert_eq!(endpoint["success"], true, "{endpoint}");
                                assert_eq!(endpoint["data"]["inputs"], json!([etched["data"]["receipt"]]));
                                receipts.extend([etched["data"]["receipt"].clone(), endpoint["data"]["receipt"].clone()]);
                            }
                            if who == "DEPOSITOR" {
                                let growth = "/service/run_deposition";
                                let rate = "/service/check_monitoring";
                                let grow = format!("echo '{{\"process\":\"cvd_growth\"}}' > {growth}");
                                let sample = format!("echo '{{\"check_type\":\"deposition_rate\"}}' > {rate}; cat {rate}");
                                let early = guest.command(who, format!("echo '{{\"check_type\":\"deposition_rate\"}}' | dd of={rate} oflag=nonblock")).await?;
                                assert!(early["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{early}");
                                if exposure_first == Some(true) {
                                    assert_eq!(guest.command("LITHOGRAPHER", expose.into()).await?["stdout"], "wash> ");
                                }
                                assert_eq!(guest.command(who, grow).await?["stdout"], "wash> ");
                                if exposure_first == Some(false) {
                                    assert_eq!(guest.command("LITHOGRAPHER", expose.into()).await?["stdout"], "wash> ");
                                }
                                let mut measured = shell_json_result(&guest.command(who, sample.clone()).await?)?;
                                if retry_control {
                                    assert_eq!(measured["data"]["status"], "check_failed");
                                    let early = guest.command(who, "echo ready | dd of=/service/fab_ready oflag=nonblock".into()).await?;
                                    assert!(early["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{early}");
                                    measured = shell_json_result(&guest.command(who, sample).await?)?;
                                }
                                let grown = shell_json_result(&guest.command(who, format!("cat {growth}")).await?)?;
                                assert_eq!(grown["success"], true, "{grown}");
                                assert_eq!(measured["success"], true, "{measured}");
                                assert_eq!(measured["data"]["inputs"], json!([grown["data"]["receipt"]]));
                                receipts.extend([grown["data"]["receipt"].clone(), measured["data"]["receipt"].clone()]);
                            }
                            if who == "ETCHER" {
                                let path = "/service/run_etch_process";
                                let invalid = guest.command(who, format!("echo '{{\"process\":\"pattern_transfer_etch\"}}' | dd of={path} oflag=nonblock")).await?;
                                assert!(invalid["stdout"].as_str().is_some_and(|s| s.contains("Invalid argument")), "{invalid}");
                                let process = format!("echo '{{\"process\":\"reactive_ion_etch\"}}' > {path}; cat {path}");
                                let mut result = shell_json_result(&guest.command(who, process.clone()).await?)?;
                                if retry_etch {
                                    assert_eq!(result["success"], false);
                                    assert_eq!(result["data"]["status"], "etch_failed");
                                    let early = guest.command(who, "echo ready | dd of=/service/fab_ready oflag=nonblock".into()).await?;
                                    assert!(early["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{early}");
                                    result = shell_json_result(&guest.command(who, process).await?)?;
                                }
                                assert_eq!(result["success"], true, "{result}");
                                receipts.push(result["data"]["receipt"].clone());
                                let repeat = guest.command(who, format!("echo '{{\"process\":\"reactive_ion_etch\"}}' | dd of={path} oflag=nonblock")).await?;
                                assert!(repeat["stdout"].as_str().is_some_and(|s| s.contains("Permission denied")), "{repeat}");
                                let monitor_path="/service/check_monitoring";
                                let sample=format!("echo '{{\"check_type\":\"etch_rate\"}}' > {monitor_path}; cat {monitor_path}");
                                let mut reading=shell_json_result(&guest.command(who,sample.clone()).await?)?;
                                if retry_control {
                                    assert_eq!(reading["data"]["status"],"check_failed");
                                    let early=guest.command(who,"echo ready | dd of=/service/fab_ready oflag=nonblock".into()).await?;
                                    assert!(early["stdout"].as_str().is_some_and(|text|text.contains("Permission denied") || text.contains("Resource temporarily unavailable")), "{early}");
                                    reading=shell_json_result(&guest.command(who,sample).await?)?;
                                }
                                assert_eq!(reading["success"],true,"{reading}");
                                assert_eq!(reading["data"]["check_type"],"etch_rate");
                                assert_eq!(reading["data"]["inputs"],json!([result["data"]["receipt"]]));
                                receipts.push(reading["data"]["receipt"].clone());
                            }

                            inputs[i] = json!({"receipts": &receipts[receipts.len()-2..]});
                            let sent = guest
                                .command(
                                    who,
                                    format!("printf '%s' '{}' > /service/fab_ready", inputs[i]),
                                )
                                .await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                            continue;
                        }
                        let ready = guest
                            .command(measurer, format!("cat /service/{material}"))
                            .await?;
                        assert_eq!(shell_json_result(&ready)?, inputs[i]);
                        let command = format!(
                            "echo '{{\"measurement\":\"{dimension}\"}}' > /service/{dimension}; cat /service/{dimension}"
                        );
                        let attempts = if first_measure {
                            2 + usize::from(retry)
                        } else {
                            1
                        };
                        for n in 0..attempts {
                            let read = guest.command(measurer, command.clone()).await?;
                            let value = shell_json_result(&read)?;
                            if first_measure && retry && n == 0 {
                                assert_eq!(value["success"], false, "{value}");
                                assert_eq!(value["data"]["status"], "measurement_failed");
                                let denied = guest
                                    .command(
                                        measurer,
                                        format!("printf '%s' '{value}' > /service/{material}"),
                                    )
                                    .await?;
                                assert!(
                                    denied["stdout"]
                                        .as_str()
                                        .is_some_and(|s| s.contains("Invalid argument")),
                                    "{denied}"
                                );
                                continue;
                            }
                            assert_eq!(value["success"], true, "{value}");
                            assert_eq!(value["data"]["inputs"], inputs[i]["receipts"]);
                            if !readings[i].is_null() {
                                assert_ne!(value["data"]["receipt"], readings[i]["data"]["receipt"]);
                            }
                            readings[i] = value;
                        }
                        first_measure = false;
                        if !deferred_reads {
                            let sent = guest
                                .command(
                                    measurer,
                                    format!("printf '%s' '{}' > /service/{material}", readings[i]),
                                )
                                .await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                        }
                    }
                    if deferred_reads {
                        for i in delivery {
                            let material = materials[i];
                            let wrong = &readings[(i + 1) % 3];
                            let denied = guest
                                .command(
                                    measurer,
                                    format!("printf '%s' '{wrong}' > /service/{material}"),
                                )
                                .await?;
                            assert!(
                                denied["stdout"]
                                    .as_str()
                                    .is_some_and(|s| s.contains("Invalid argument")),
                                "{denied}"
                            );
                            let sent = guest
                                .command(
                                    measurer,
                                    format!("printf '%s' '{}' > /service/{material}", readings[i]),
                                )
                                .await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                        }
                    }
                    assert_eq!(
                        guest.command(measurer, "exit 0".into()).await?["exit_status"],
                        0
                    );
                    for (position, i) in delivery.into_iter().rev().enumerate() {
                        if position == 2 {
                            let premature=guest.command(controller,"echo '{\"check_type\":\"cross_reference\"}' | dd of=/service/check_monitoring oflag=nonblock".into()).await?;
                            assert!(
                                premature["stdout"]
                                    .as_str()
                                    .is_some_and(|s| s.contains("Permission denied")
                                        || s.contains("Resource temporarily unavailable")),
                                "{premature}"
                            );
                        }
                        let read = guest
                            .command(controller, format!("cat /service/{}_report", materials[i]))
                            .await?;
                        assert_eq!(shell_json_result(&read)?, readings[i]);
                    }
                    let check = "echo '{\"check_type\":\"cross_reference\"}' > /service/check_monitoring; cat /service/check_monitoring";
                    let prospective = json!({"tool_name":"check_monitoring","success":true,"data":{
                        "engineer":controller,"check_type":"cross_reference","receipt":1,
                        "inputs":readings.each_ref().map(|r| &r["data"]["receipt"]),
                    }});
                    let early = format!("printf '%s' '{prospective}' | dd of=/service/all_validated oflag=nonblock");
                    let premature = guest.command(controller, early.clone()).await?;
                    assert!(premature["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{premature}");
                    let mut validation = shell_json_result(&guest.command(controller, check.into()).await?)?;
                    let first_validation = validation.clone();
                    if retry_control {
                        assert_eq!(validation["success"], false, "{validation}");
                        assert_eq!(validation["data"]["status"], "check_failed");
                        let premature = guest.command(controller, early).await?;
                        assert!(premature["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{premature}");
                        validation = shell_json_result(&guest.command(controller, check.into()).await?)?;
                    }
                    assert_eq!(validation["success"], true, "{validation}");
                    assert_eq!(validation["data"]["inputs"], json!(readings.each_ref().map(|r| &r["data"]["receipt"])));
                    let notices = ["validated_lithographer", "validated_depositor", "validated_etcher", "all_validated"];
                    let mut invalid = [readings[0].clone(), validation.clone(), first_validation, validation.clone()];
                    invalid[1]["data"]["inputs"][1] = invalid[1]["data"]["inputs"][0].clone();
                    if !retry_control {
                        invalid[2]["data"]["engineer"] = json!("DEPOSITOR");
                    }
                    invalid[3]["data"]["inputs"].as_array_mut().unwrap().pop();
                    let mut rejected = Vec::new();
                    for (path, value) in notices.iter().zip(&invalid) {
                        let reply = guest.command(controller, format!("printf '%s' '{value}' > /service/{path}")).await?;
                        assert!(reply["stdout"].as_str().is_some_and(|s| s.contains("Invalid argument")), "{reply}");
                        rejected.push(json!({"path":path,"payload":value,"stdout":reply["stdout"]}));
                    }
                    let mut broadcast = delivery.to_vec();
                    broadcast.insert(if deferred_reads { 0 } else { 3 }, 3);
                    if deferred_reads {
                        broadcast.retain(|i| *i != 2);
                    }
                    for &i in &broadcast {
                        let path = notices[i];
                        let sent = guest.command(controller, format!("printf '%s' '{validation}' > /service/{path}")).await?;
                        assert_eq!(sent["stdout"], "wash> ", "{sent}");
                    }
                    for i in broadcast.into_iter().rev() {
                        let (who, file) = if i == 3 { (agent, "all_validated") } else { (recipients[i].0, "process_validation") };
                        let read = guest.command(who, format!("cat /service/{file}")).await?;
                        assert_eq!(shell_json_result(&read)?, validation, "{read}");
                    }
                    let calibration = async |which: usize| -> Result<()> {
                        let (who, measurement) = [("LITHOGRAPHER", "alignment_check"), ("DEPOSITOR", "thickness_verification")][which];
                        let path = "/service/run_metrology_measurement";
                        let args = json!({"measurement":measurement});
                        let command = format!("printf '%s' '{args}' > {path}; cat {path}");
                        let mut result = shell_json_result(&guest.command(who, command.clone()).await?)?;
                        if result["success"] == false {
                            assert_eq!(result["data"]["status"], "measurement_failed");
                            result = shell_json_result(&guest.command(who, command).await?)?;
                        }
                        assert_eq!(result["success"], true);
                        assert_eq!(result["data"]["measurement"], measurement);
                        assert_eq!(result["data"]["inputs"], json!([validation["data"]["receipt"]]));
                        Ok(())
                    };
                    let overlay = delivery.iter().position(|&i| i == 2).unwrap();
                    for &which in &delivery[..overlay] { calibration(which).await?; }
                    let mount = "/service/run_stepper_exposure";
                    let measure = "/service/run_metrology_measurement";
                    let mount_write = format!("echo '{{\"process\":\"overlay_alignment\"}}' > {mount}");
                    let measure_write = format!("echo '{{\"measurement\":\"overlay_alignment\"}}' > {measure}");
                    let refused = guest.command(agent,
                        format!("echo premature | dd of={} oflag=nonblock", recipients[delivery[0]].1)).await?;
                    assert!(refused["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{refused}");
                    let first = guest.command(agent, mount_write).await?;
                    assert_eq!(first["stdout"], "wash> ", "{first}");
                    let sent = guest.command(agent, measure_write.clone()).await?;
                    assert_eq!(sent["stdout"], "wash> ", "{sent}");
                    if retry {
                        let failed = shell_json_result(&guest.command(agent, format!("cat {measure}")).await?)?;
                        assert_eq!(failed["success"], false, "{failed}");
                        assert_eq!(failed["data"]["status"], "measurement_failed");
                        let refused = guest.command(agent,
                            "echo '{\"process\":\"film_integrity_check\"}' | dd of=/service/run_deposition oflag=nonblock".into()).await?;
                        let output = refused["stdout"].as_str().ok_or("missing refused command output")?;
                        assert!(output.contains("Permission denied") || output.contains("Resource temporarily unavailable"), "{refused}");
                        let sent = guest.command(agent, measure_write).await?;
                        assert_eq!(sent["stdout"], "wash> ", "{sent}");
                    }
                    let (mounted, measured) = if order[0] == etch {
                        let mounted = shell_json_result(&guest.command(agent, format!("cat {mount}")).await?)?;
                        (mounted, shell_json_result(&guest.command(agent, format!("cat {measure}")).await?)?)
                    } else {
                        let measured = shell_json_result(&guest.command(agent, format!("cat {measure}")).await?)?;
                        (shell_json_result(&guest.command(agent, format!("cat {mount}")).await?)?, measured)
                    };
                    assert_eq!(mounted["success"], true, "{mounted}");
                    assert_eq!(measured["success"], true, "{measured}");
                    assert_eq!(mounted["data"]["inputs"], json!([validation["data"]["receipt"]]));
                    assert_eq!(measured["data"]["inputs"], json!([mounted["data"]["receipt"]]));
                    for &which in &delivery[overlay + 1..] { calibration(which).await?; }
                    let mut inspection = serde_json::Map::new();
                    for (tool, operation) in order {
                        let path = format!("/service/{tool}");
                        let early = guest
                            .command(agent, format!("dd if={path} iflag=nonblock"))
                            .await?;
                        let stdout = early["stdout"].as_str().ok_or("early instrument read")?;
                        assert!(
                            stdout.contains("Permission denied")
                                || stdout.contains("Resource temporarily unavailable"),
                            "{early}"
                        );
                        let wrong = if tool == film.0 { etch.1 } else { film.1 };
                        let refused = guest
                            .command(
                                agent,
                                format!("echo '{{\"process\":\"{wrong}\"}}' > {path}"),
                            )
                            .await?;
                        assert!(
                            refused["stdout"]
                                .as_str()
                                .is_some_and(|s| s.contains("Invalid argument")),
                            "{refused}"
                        );
                        let reply = guest
                            .command(
                                agent,
                                format!(
                                    "echo '{{\"process\":\"{operation}\"}}' > {path}; cat {path}"
                                ),
                            )
                            .await?;
                        let result = shell_json_result(&reply)?;
                        assert_eq!(result["tool_name"], tool);
                        assert_eq!(result["success"], true);
                        inspection.insert(operation.into(), result["data"]["receipt"].clone());
                        assert_eq!(result["data"]["process"], operation);
                        assert_eq!(
                            result["data"]["inputs"],
                            json!([mounted["data"]["receipt"], measured["data"]["receipt"]])
                        );
                    }
                    let decision = json!({"lot_disposition":verdict, "inspection":inspection});
                    if deferred_reads {
                        let sent = guest.command(controller, format!("printf '%s' '{validation}' > /service/validated_etcher")).await?;
                        assert_eq!(sent["stdout"], "wash> ", "{sent}");
                        let read = guest.command("ETCHER", "cat /service/process_validation".into()).await?;
                        assert_eq!(shell_json_result(&read)?, validation);
                    }
                    assert_eq!(guest.command(controller, "exit 0".into()).await?["exit_status"], 0);
                    let writes = delivery.map(|i| (i, true));
                    let schedule: Vec<_> = if deferred_reads {
                        writes.into_iter().chain(delivery.into_iter().rev().map(|i| (i, false))).collect()
                    } else {
                        delivery.into_iter().flat_map(|i| [(i, true), (i, false)]).collect()
                    };
                    for (i, write) in schedule {
                        let (recipient, path) = recipients[i];
                        if write {
                            let sent = guest.command(agent, format!("printf '%s' '{decision}' > {path}")).await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                        } else {
                            let read = guest.command(recipient, "cat /service/lot_disposition".into()).await?;
                            assert_eq!(shell_json_result(&read)?, decision, "{read}");
                            assert_eq!(guest.command(recipient, "exit 0".into()).await?["exit_status"], 0);
                        }
                    }
                    assert_eq!(
                        guest.command(agent, "exit 0".into()).await?["exit_status"],
                        0
                    );
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(rejected)
                };
                async { runtime.await };
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(30)),
        )
        .await?;
        save(&directory.join("runtime-result.json"), &outcome)?;
        assert_eq!(outcome["runtime_exit_code"], 0);
        assert_eq!(
            outcome["protocol"],
            json!({"type":"run_result","status":"complete","scope":"reference_wafer_integration"})
        );
        assert_eq!(outcome["backend_errors"], json!([]));
        assert!(outcome["error"].is_null());
        assert_eq!(
            outcome["score"]["events"].as_array().unwrap().len(),
            guest_effects
        );
        assert_eq!(outcome["score"]["complete"], true);
        assert_eq!(outcome["score"]["violations"], json!([]));
        let rows = fs::read_to_string(&journal)?
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<Vec<Value>, _>>()?;
        let labels = projection::frames(&artifact, &rows)?;
        let (guest_calls, fixture_calls): (Vec<_>, Vec<_>) = rows
            .iter()
            .filter(|r| r["direction"] == "domain_start")
            .partition(|r| r["message"]["id"].is_u64());
        assert_eq!(guest_calls.len(), guest_effects);
        assert!(
            fixture_calls.is_empty(),
            "every physical effect must originate at a wash guest"
        );
        let epf = projection::check_dynamic(&artifact, &rows)?;
        let audited = crate::wafer_audit::effects(&rows, &artifact, &labels, &epf)?;
        let compiled = &artifact["compiled"];
        let roles = &compiled["roles"];
        let probe_effects =
            check_probe_effects(&epf, &labels, compiled, &rows, &outcome["score"], delivery)?;
        let effects = check_wafer_effects(
            &labels,
            &epf,
            &rows,
            &outcome["score"],
            compiled,
            [
                retry,
                retry_control,
                retry_etch,
                retry_pattern,
                retry_endpoint,
            ],
        )?;
        if let Some(photo_first) = exposure_first {
            let interval = |process: &str| -> Result<(u64, u64)> {
                let effect = effects
                    .iter()
                    .find(|e| e["arguments"]["process"] == process)
                    .ok_or("missing fabrication interval")?;
                Ok((
                    effect["effect_start_row"].as_u64().ok_or("missing start")?,
                    effect["effect_end_row"].as_u64().ok_or("missing end")?,
                ))
            };
            let photo = interval("photo_exposure")?;
            let growth = interval("cvd_growth")?;
            assert_eq!(photo.0 < growth.0, photo_first, "unexpected load order");
            assert!(
                photo.0.max(growth.0) < photo.1.min(growth.1),
                "both independent instruments must be active together"
            );
        }
        check_spot_check_effects(roles, &epf, &rows, &outcome["score"], order[0] == etch)?;
        let captures = epf["sessions"][0]["acceptances"]
            .as_array()
            .ok_or("missing EPF deliveries")?;
        let validation = effects.last().ok_or("missing validation")?;
        assert_eq!(validation["tool"], "check_monitoring");
        assert_eq!(validation["success"], true);
        let measurements = check_dimension_reports(
            compiled,
            captures,
            &labels,
            &rows,
            &outcome["score"],
            validation,
        )?;
        let mut measure_order = delivery;
        if deferred_reads {
            measure_order.rotate_left(1);
        }
        for pair in measure_order.windows(2) {
            let previous = measurements[pair[0]]["attempts"]
                .as_array()
                .unwrap()
                .last()
                .unwrap();
            let next = &measurements[pair[1]]["attempts"][0];
            assert!(
                previous["effect_end_row"].as_u64().unwrap()
                    < next["effect_start_row"].as_u64().unwrap()
            );
        }
        for (index, &material) in measure_order.iter().enumerate() {
            assert_eq!(
                measurements[material]["attempts"].as_array().unwrap().len(),
                if index == 0 {
                    2 + usize::from(retry)
                } else {
                    1
                }
            );
        }
        for ordered in delivery.windows(2) {
            assert!(
                measurements[ordered[0]]["ready_send"].as_u64().unwrap()
                    < measurements[ordered[1]]["ready_send"].as_u64().unwrap()
            );
            assert!(
                measurements[ordered[0]]["report_send"].as_u64().unwrap()
                    < measurements[ordered[1]]["report_send"].as_u64().unwrap()
            );
            if !deferred_reads {
                assert!(
                    measurements[ordered[0]]["report_send"].as_u64().unwrap()
                        < measurements[ordered[1]]["first_fabrication_start"]
                            .as_u64()
                            .unwrap()
                );
            }
        }
        let mut confirmations = Vec::new();
        for (view, actor) in [
            ("LITHO_INBOX", "LITHO"),
            ("DEP_INBOX", "DEP"),
            ("ETCH_INBOX", "ETCH"),
            ("WAFER", "ENGINEER"),
        ] {
            let mut matches = captures.iter().filter(|c| {
                c["send_view"] == roles["CONTROL_FILE"]
                    && c["receive_commit"]["view"] == roles[view]
            });
            let call = matches
                .next()
                .ok_or("missing process validation notification")?;
            assert!(matches.next().is_none());
            assert_eq!(call["send_actor"], roles["CONTROL"]);
            assert_eq!(call["receive_commit"]["actor"], roles[actor]);
            let (sent, bytes) = json_frame(&rows, call)?;
            assert_eq!(bytes, validation["result"]);
            assert!(validation["reply_received_row"].as_u64().unwrap() < sent as u64);
            if view == "WAFER" {
                assert!(
                    call["receive_commit"]["journal_row"].as_u64().unwrap()
                        < effects[0]["effect_start_row"].as_u64().unwrap()
                );
            }
            confirmations.push(json!({"view":view,"frame":call["frame_id"],"send_row":sent,"receive_row":call["receive_commit"]["journal_row"]}));
        }
        let mut delivered = Vec::new();
        let mut inspections = serde_json::Map::new();
        let mut last_inspection_read = 0;
        for call in captures {
            if call["receive_commit"]["view"] != roles["WAFER"]
                || !["DEPOSITION_CHAMBER", "ETCH_CHAMBER"]
                    .iter()
                    .any(|role| call["send_view"] == roles[*role])
            {
                continue;
            }
            let (_, reply) = json_frame(&rows, call)?;
            inspections.insert(
                reply["data"]["process"]
                    .as_str()
                    .ok_or("inspection target")?
                    .into(),
                reply["data"]["receipt"].clone(),
            );
            last_inspection_read = last_inspection_read.max(
                call["receive_commit"]["journal_row"]
                    .as_u64()
                    .ok_or("missing inspection receive")?,
            );
        }
        assert_eq!(inspections.len(), 2);
        if deferred_reads {
            let etcher = confirmations
                .iter()
                .find(|notice| notice["view"] == "ETCH_INBOX")
                .ok_or("missing etcher confirmation")?;
            assert!(last_inspection_read < etcher["send_row"].as_u64().unwrap());
        }
        let decision = json!({"lot_disposition":verdict, "inspection":inspections});
        for role in delivery {
            let (recipient, _) = recipients[role];
            let inbox = artifact["compiled"]["roles"]
                [["LITHO_INBOX", "DEP_INBOX", "ETCH_INBOX"][role]]
                .as_u64()
                .ok_or("missing inbox view")?;
            let vm = artifact["compiled"]["roles"][["LITHO", "DEP", "ETCH"][role]]
                .as_u64()
                .ok_or("missing receiving participant")?;
            let mut matched = captures.iter().filter(|c| {
                c["send_view"] == roles["WAFER"] && c["receive_commit"]["view"] == inbox
            });
            let call = matched.next().ok_or("missing lot disposition")?;
            assert!(matched.next().is_none());
            assert_eq!(call["send_actor"], roles["ENGINEER"]);
            assert_eq!(call["receive_commit"]["actor"], vm);
            let (sent, payload) = json_frame(&rows, call)?;
            assert_eq!(payload, decision);
            assert!(sent as u64 > last_inspection_read);
            delivered.push(json!({"recipient":recipient,"frame":call["frame_id"],"send_row":sent,"receive_row":call["receive_commit"]["journal_row"]}));
        }
        assert!(
            delivered
                .windows(2)
                .all(|p| p[0]["send_row"].as_u64().unwrap() < p[1]["send_row"].as_u64().unwrap())
        );
        if deferred_reads {
            assert!(
                delivered.last().unwrap()["send_row"].as_u64().unwrap()
                    < delivered.last().unwrap()["receive_row"].as_u64().unwrap()
            );
            assert!(
                delivered
                    .windows(2)
                    .all(|p| p[0]["receive_row"].as_u64().unwrap()
                        > p[1]["receive_row"].as_u64().unwrap())
            );
        } else {
            assert!(delivered.windows(2).all(
                |p| p[0]["receive_row"].as_u64().unwrap() < p[1]["send_row"].as_u64().unwrap()
            ));
        }
        save(
            &directory.join("result.json"),
            &json!({"task":"15E","order":order,"wasm_participants":6,"disposition":decision,"deliveries":delivered,"deferred_reads":deferred_reads,
            "model_calls":0,"scored_benchmark":false,"fabrication_fixture_calls":0, "fixture_post_validation_calls":0,"audited_effects":audited,"probe_effects":probe_effects,
                "shell_effects":guest_effects,"concurrent_load_exposure_first":exposure_first,"measurements":measurements,"instrument_effects":effects,"confirmations":confirmations,"rejected_confirmations":rejected_confirmations,"etcher_confirmation_after_inspections":deferred_reads,"injected_first_check_failure":retry_control,"injected_first_measurement_failure":retry,"injected_first_etch_failure":retry_etch,"early_reads":2,"rejected_wrong_operations":3,"rejected_duplicate_etch":true,"score":outcome["score"],"epf":epf,
            "events_sha256":digest(&journal)?,"runtime_binary_sha256":digest(&binary)?,
            "wash_sha256":digest(&wash)?,"build_receipt_sha256":digest(&output.join("build-receipt.json"))?}),
        )?;
    }
    Ok(())
}

fn shell_json_result(reply: &Value) -> Result<Value> {
    let stdout = reply["stdout"].as_str().ok_or("missing shell output")?;
    let offset = stdout
        .find('{')
        .ok_or_else(|| format!("missing instrument result: {reply}"))?;
    Ok(serde_json::Deserializer::from_str(&stdout[offset..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty instrument result")??)
}

fn json_frame(journal_rows: &[Value], call: &Value) -> Result<(usize, Value)> {
    let (index, row) = journal_rows
        .iter()
        .enumerate()
        .find(|(_, row)| {
            let message = &row["message"];
            row["direction"] == "runtime"
                && message["type"] == "protocol_frame"
                && message["action"] == "send"
                && message["frame_id"] == call["frame_id"]
        })
        .ok_or("missing accepted frame")?;
    let bytes: Vec<u8> = serde_json::from_value(row["message"]["payload"].clone())?;
    Ok((index, serde_json::from_slice(&bytes)?))
}

fn received_instrument_effect(
    rows: &[Value],
    request: &Value,
    reply: &Value,
    (start, invoked): (usize, &Value),
    scored: &Value,
) -> Result<Value> {
    let message = &invoked["message"];
    let id = &message["id"];
    let mut ends = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["direction"] == "domain_end" && r["message"]["id"] == *id);
    let (end, finished) = ends.next().ok_or("missing instrument completion")?;
    assert!(ends.next().is_none(), "duplicate instrument completion");
    let finished = &finished["message"];
    let (sent, args) = json_frame(rows, request)?;
    let (reply_sent, result) = json_frame(rows, reply)?;
    assert_eq!(invoked["direction"], "domain_start");
    assert_eq!(finished["agent"], message["agent"]);
    assert_eq!(finished["tool"], message["tool"]);
    assert_eq!(scored["agent"], message["agent"]);
    assert_eq!(scored["tool"], message["tool"]);
    assert_eq!(args, message["arguments"]);
    assert_eq!(args, scored["args"]);
    assert_eq!(result, finished["result"]);
    assert_eq!(result["data"], scored["result"]);
    assert_eq!(result["success"], scored["success"]);
    let received = request["receive_commit"]["journal_row"]
        .as_u64()
        .ok_or("missing instrument receive row")?;
    assert!(received < start as u64 && start < end && end < reply_sent);
    Ok(
        json!({"tool":message["tool"],"rpc_id":id,"success":result["success"],"request_frame":request["frame_id"],"reply_frame":reply["frame_id"],
        "request_sent_row":sent,"receive_row":received,"effect_start_row":start,"effect_end_row":end,"reply_row":reply_sent,"reply_received_row":reply["receive_commit"]["journal_row"],
        "arguments":args,"receipt":result["data"]["receipt"],"inputs":result["data"]["inputs"],"result":result}),
    )
}

fn check_probe_effects(
    epf: &Value,
    labels: &std::collections::BTreeMap<u64, u8>,
    compiled: &Value,
    rows: &[Value],
    score: &Value,
    order: [usize; 3],
) -> Result<Vec<Value>> {
    let roles = &compiled["roles"];
    let message = |name: &str| {
        u8::try_from(
            compiled["labels"][name]
                .as_u64()
                .expect("compiled wafer message"),
        )
        .unwrap()
    };
    let calls = epf["sessions"][0]["acceptances"]
        .as_array()
        .ok_or("meter acceptances")?;
    let mut requests: Vec<_> = calls
        .iter()
        .filter(|r| {
            r["send_view"] == roles["PROBE"] && r["receive_commit"]["view"] == roles["READOUT"]
        })
        .collect();
    let mut replies: Vec<_> = calls
        .iter()
        .filter(|r| {
            r["send_view"] == roles["READOUT"] && r["receive_commit"]["view"] == roles["PROBE"]
        })
        .collect();
    let step = |r: &Value| r["receive_commit"]["journal_row"].as_u64().unwrap();
    requests.sort_by_key(|r| step(r));
    replies.sort_by_key(|r| step(r));
    let starts: Vec<_> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r["direction"] == "domain_start" && r["message"]["tool"] == "run_metrology_measurement"
        })
        .collect();
    let scored: Vec<_> = score["events"]
        .as_array()
        .ok_or("meter scores")?
        .iter()
        .filter(|r| r["tool"] == "run_metrology_measurement")
        .collect();
    assert!(!requests.is_empty());
    assert_eq!(
        [replies.len(), starts.len(), scored.len()],
        [requests.len(); 3]
    );
    let mut effects = Vec::new();
    let mut finished = Vec::new();
    for (i, request) in requests.into_iter().enumerate() {
        let effect = received_instrument_effect(rows, request, replies[i], starts[i], scored[i])?;
        let measurement = effect["arguments"]["measurement"]
            .as_str()
            .ok_or("measurement")?;
        let (label, file, kind) = match measurement {
            "cd_measurement" => (message("cd"), "MEASURE_FILE", None),
            "thickness_measurement" => (message("thickness"), "MEASURE_FILE", None),
            "profile_measurement" => (message("profile"), "MEASURE_FILE", None),
            "alignment_check" => (message("alignment_check"), "LITHO_INBOX", Some(0)),
            "thickness_verification" => (message("thickness_verification"), "DEP_INBOX", Some(1)),
            "overlay_alignment" => (message("Measure"), "WAFER", Some(2)),
            _ => return Err("unexpected meter operation".into()),
        };
        assert_eq!(
            request["send_actor"],
            u8::try_from(roles["METROLOGY"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            labels.get(&request["frame_id"].as_u64().unwrap()),
            Some(&label)
        );
        let previous = i.checked_sub(1).map(|p| step(replies[p])).unwrap_or(0);
        assert!(previous < effect["request_sent_row"].as_u64().unwrap());
        let inputs: Vec<_> = calls
            .iter()
            .filter(|r| {
                r["send_view"] == roles[file]
                    && r["receive_commit"]["view"] == roles["METROLOGY"]
                    && labels.get(&r["frame_id"].as_u64().unwrap()) == Some(&label)
                    && previous < step(r)
                    && step(r) < effect["request_sent_row"].as_u64().unwrap()
            })
            .collect();
        let [input] = inputs.as_slice() else {
            return Err("probe needs one corresponding file request".into());
        };
        assert_eq!(json_frame(rows, input)?.1, effect["arguments"]);
        if let Some(k) = kind
            && effect["success"] == true
        {
            finished.push(k);
        }
        effects.push(effect);
    }
    assert_eq!(finished, order);
    Ok(effects)
}

fn check_dimension_reports(
    compiled: &Value,
    captures: &[Value],
    labels: &std::collections::BTreeMap<u64, u8>,
    rows: &[Value],
    score: &Value,
    validation: &Value,
) -> Result<Vec<Value>> {
    let roles = &compiled["roles"];
    let message = |name: &str| {
        u8::try_from(
            compiled["labels"][name]
                .as_u64()
                .expect("compiled wafer message"),
        )
        .unwrap()
    };
    let deliveries = |from: &str, to: &str, label: u8| {
        let mut matched: Vec<_> = captures
            .iter()
            .filter(|call| {
                call["send_view"] == roles[from]
                    && call["receive_commit"]["view"] == roles[to]
                    && call["frame_id"].as_u64().and_then(|id| labels.get(&id)) == Some(&label)
            })
            .collect();
        matched.sort_by_key(|call| call["receive_commit"]["journal_row"].as_u64());
        matched
    };
    let events = score["events"].as_array().ok_or("missing wafer score")?;
    let mut evidence = Vec::new();
    for (inbox, vm, who, name, request_label, reply_label, report_label, physical_inputs) in [
        (
            "LITHO_INBOX",
            "LITHO",
            "LITHOGRAPHER",
            "cd_measurement",
            message("cd"),
            message("cd"),
            message("cd"),
            ["pattern_transfer_etch", "etch_endpoint"],
        ),
        (
            "DEP_INBOX",
            "DEP",
            "DEPOSITOR",
            "thickness_measurement",
            message("thickness"),
            message("thickness"),
            message("thickness"),
            ["cvd_growth", "deposition_rate"],
        ),
        (
            "ETCH_INBOX",
            "ETCH",
            "ETCHER",
            "profile_measurement",
            message("profile"),
            message("profile"),
            message("profile"),
            ["reactive_ion_etch", "etch_rate"],
        ),
    ] {
        let ready = deliveries(
            inbox,
            "MEASURE_FILE",
            serde_json::from_value(roles[inbox].clone())?,
        );
        let [ready] = ready.as_slice() else {
            return Err("missing or duplicate fab-ready operation".into());
        };
        let (ready_send, notice) = json_frame(rows, ready)?;
        assert_eq!(ready["send_actor"], roles[vm]);
        assert_eq!(ready["receive_commit"]["actor"], roles["MEASURER"]);
        let inputs: Vec<_> = physical_inputs
            .iter()
            .map(|op| {
                let source = events
                    .iter()
                    .find(|e| {
                        e["agent"] == who
                            && e["success"] == true
                            && (e["result"]["process"] == *op || e["result"]["check_type"] == *op)
                    })
                    .ok_or("missing fabrication effect")?;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                    source["result"]["receipt"].clone(),
                )
            })
            .collect::<Result<_>>()?;
        assert_eq!(notice["receipts"], json!(inputs));
        let requests = deliveries("MEASURE_FILE", "METROLOGY", request_label);
        let replies = deliveries("METROLOGY", "MEASURE_FILE", reply_label);
        let starts: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row["direction"] == "domain_start"
                    && row["message"]["agent"] == "METROLOGY_ENGINEER"
                    && row["message"]["tool"] == "run_metrology_measurement"
                    && row["message"]["arguments"]["measurement"] == name
            })
            .collect();
        let scored: Vec<_> = events
            .iter()
            .filter(|e| {
                e["agent"] == "METROLOGY_ENGINEER"
                    && e["tool"] == "run_metrology_measurement"
                    && e["args"]["measurement"] == name
            })
            .collect();
        assert!(!starts.is_empty());
        assert_eq!(
            [requests.len(), replies.len(), scored.len()],
            [starts.len(); 3]
        );
        let mut attempts = Vec::new();
        for (i, (start, invoked)) in starts.into_iter().enumerate() {
            assert_eq!(requests[i]["send_actor"], roles["MEASURER"]);
            assert_eq!(requests[i]["receive_commit"]["actor"], roles["METROLOGY"]);
            assert_eq!(replies[i]["send_actor"], roles["METROLOGY"]);
            assert_eq!(replies[i]["receive_commit"]["actor"], roles["MEASURER"]);
            let effect = received_instrument_effect(
                rows,
                requests[i],
                replies[i],
                (start, invoked),
                scored[i],
            )?;
            assert_eq!(effect["arguments"], json!({"measurement":name}));
            assert!(
                ready["receive_commit"]["journal_row"].as_u64().unwrap()
                    < effect["request_sent_row"].as_u64().unwrap()
            );
            if effect["success"] == true {
                assert_eq!(effect["inputs"], json!(inputs));
            }
            attempts.push(effect);
        }
        let actual = &attempts.last().ok_or("missing measurement")?["result"];
        let reports = deliveries("MEASURE_FILE", "CONTROL_FILE", report_label);
        let [report] = reports.as_slice() else {
            return Err("missing or duplicate measurement report".into());
        };
        let (report_send, reported) = json_frame(rows, report)?;
        assert_eq!(report["send_actor"], roles["MEASURER"]);
        assert_eq!(report["receive_commit"]["actor"], roles["CONTROL"]);
        assert_eq!(actual["success"], true);
        assert_eq!(&reported, actual);
        assert!(
            replies.last().unwrap()["receive_commit"]["journal_row"]
                .as_u64()
                .unwrap()
                < report_send as u64
        );
        assert!(
            report["receive_commit"]["journal_row"].as_u64().unwrap()
                < validation["receive_row"].as_u64().unwrap()
        );
        assert_eq!(
            validation["inputs"][evidence.len()],
            actual["data"]["receipt"]
        );
        let first_fabrication_start = rows
            .iter()
            .position(|row| row["direction"] == "domain_start" && row["message"]["agent"] == who)
            .ok_or("missing first fabrication start")?;
        evidence.push(json!({"measurement":name,"ready":ready["frame_id"],"ready_send":ready_send,"report":report["frame_id"],
            "first_fabrication_start":first_fabrication_start,"report_send":report_send,"receipt":actual["data"]["receipt"],"attempts":attempts}));
    }
    Ok(evidence)
}

fn check_wafer_effects(
    labels: &std::collections::BTreeMap<u64, u8>,
    epf: &Value,
    rows: &[Value],
    score: &Value,
    compiled: &Value,
    [
        retry,
        retry_control,
        retry_etch,
        retry_pattern,
        retry_endpoint,
    ]: [bool; 5],
) -> Result<Vec<Value>> {
    let roles = &compiled["roles"];
    let message = |name: &str| {
        u8::try_from(
            compiled["labels"][name]
                .as_u64()
                .expect("compiled wafer message"),
        )
        .unwrap()
    };

    let calls = epf["sessions"][0]["acceptances"]
        .as_array()
        .ok_or("EPF deliveries")?;
    let by_label = |label| -> Vec<&Value> {
        let mut found: Vec<_> = calls
            .iter()
            .filter(|call| {
                call["frame_id"]
                    .as_u64()
                    .and_then(|id| labels.get(&id))
                    .copied()
                    == Some(label)
            })
            .collect();
        found.sort_by_key(|c| c["receive_commit"]["journal_row"].as_u64());
        found
    };
    let events = score["events"].as_array().ok_or("wafer events")?;
    let mut evidence = Vec::new();
    let mut reports = by_label(message("OverlayFailed"));
    assert_eq!(reports.len(), usize::from(retry));
    let passed = by_label(message("OverlayPassed"));
    assert_eq!(passed.len(), 1);
    reports.extend(passed);
    let mut checks = by_label(message("CrossReferenceFailed"));
    assert_eq!(checks.len(), usize::from(retry_control));
    let completed = by_label(message("CrossReferencePassed"));
    assert_eq!(completed.len(), 1);
    checks.extend(completed);
    let mut rates = by_label(message("RateFailed"));
    assert_eq!(rates.len(), usize::from(retry_control));
    let stable = by_label(message("RatePassed"));
    assert_eq!(stable.len(), 1);
    rates.extend(stable);
    let mut etch_results = by_label(message("EtchFailed"));
    assert_eq!(etch_results.len(), usize::from(retry_etch));
    let loaded = by_label(message("EtchLoaded"));
    assert_eq!(loaded.len(), 1);
    let transfers: Vec<_> = calls
        .iter()
        .filter(|c| {
            c["send_view"] == roles["ETCH_OPERATOR"]
                && c["receive_commit"]["view"] == roles["ETCH_METER"]
        })
        .collect();
    let [transfer] = transfers.as_slice() else {
        return Err("expected one etch chamber transfer".into());
    };
    assert_eq!(
        labels.get(&transfer["frame_id"].as_u64().ok_or("etch loan frame")?),
        Some(&message("EtchLoan"))
    );
    let step = |c: &Value| c["receive_commit"]["journal_row"].as_u64().unwrap();
    // The holder sends the acknowledgement before the loan. Their receivers are independent.
    assert_eq!(loaded[0]["producer"], transfer["producer"]);
    assert!(loaded[0]["before"].as_u64().unwrap() < transfer["before"].as_u64().unwrap());
    let returns: Vec<_> = by_label(message("EtchLoan"))
        .into_iter()
        .filter(|c| {
            c["send_view"] == roles["ETCH_METER"]
                && c["receive_commit"]["view"] == roles["ETCH_STATION"]
        })
        .collect();
    let [returned] = returns.as_slice() else {
        return Err("expected one inspected etch chamber transfer".into());
    };
    let inspected: Vec<_> = by_label(message("Uniformity"))
        .into_iter()
        .filter(|c| c["send_view"] == u8::try_from(roles["WAFER"].as_u64().unwrap()).unwrap())
        .collect();
    assert_eq!(inspected.len(), 1);
    assert_eq!(
        inspected[0]["receive_commit"]["actor"],
        roles["ETCH_STATION"]
    );
    assert!(step(transfer) < step(returned) && step(returned) < step(inspected[0]));
    etch_results.extend(loaded);
    let depositor_file: u8 = serde_json::from_value(roles["DEP_INBOX"].clone())?;
    let operator: u8 = serde_json::from_value(roles["CONTROL_OPERATOR"].clone())?;
    let control_file: u8 = serde_json::from_value(roles["CONTROL_FILE"].clone())?;
    for (agent, tool, actor, resource, caller, responder, request_label, reply_label, count) in [
        (
            "INTEGRATION_ENGINEER",
            "run_stepper_exposure",
            u8::try_from(roles["STEPPER"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["STAGE"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["WAFER"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["STEPPER"].as_u64().unwrap()).unwrap(),
            message("Mount"),
            message("Mounted"),
            1,
        ),
        (
            "INTEGRATION_ENGINEER",
            "run_metrology_measurement",
            u8::try_from(roles["METROLOGY"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["METROLOGY"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["WAFER"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["STEPPER"].as_u64().unwrap()).unwrap(),
            message("Measure"),
            message("Measured"),
            reports.len(),
        ),
        (
            "LITHOGRAPHER",
            "run_stepper_exposure",
            u8::try_from(roles["STEPPER"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["STAGE"].as_u64().unwrap()).unwrap(),
            serde_json::from_value(roles["LITHO_INBOX"].clone())?,
            u8::try_from(roles["STEPPER"].as_u64().unwrap()).unwrap(),
            message("Expose"),
            message("Exposed"),
            1,
        ),
        (
            "DEPOSITOR",
            "run_deposition",
            u8::try_from(roles["DEPOSITION_CHAMBER"].as_u64().unwrap()).unwrap(),
            u8::try_from(roles["CVD_STAGE"].as_u64().unwrap()).unwrap(),
            depositor_file,
            u8::try_from(roles["DEPOSITION_CHAMBER"].as_u64().unwrap()).unwrap(),
            message("Growth"),
            message("Grown"),
            1,
        ),
        (
            "ETCHER",
            "run_etch_process",
            serde_json::from_value(roles["ETCH_OPERATOR"].clone())?,
            u8::try_from(roles["ETCH_CHAMBER"].as_u64().unwrap()).unwrap(),
            serde_json::from_value(roles["ETCH_INBOX"].clone())?,
            u8::try_from(roles["PLASMA_INPUT"].as_u64().unwrap()).unwrap(),
            message("ReactiveEtch"),
            message("EtchResult"),
            etch_results.len(),
        ),
        (
            "LITHOGRAPHER",
            "run_etch_process",
            serde_json::from_value(roles["PATTERN_OPERATOR"].clone())?,
            serde_json::from_value(roles["PATTERN_WAFER"].clone())?,
            serde_json::from_value(roles["LITHO_INBOX"].clone())?,
            u8::try_from(roles["PLASMA_INPUT"].as_u64().unwrap()).unwrap(),
            message("PatternEtch"),
            message("PatternResult"),
            1 + usize::from(retry_pattern),
        ),
        (
            "LITHOGRAPHER",
            "check_monitoring",
            serde_json::from_value(roles["PATTERN_METER"].clone())?,
            serde_json::from_value(roles["PATTERN_WAFER"].clone())?,
            serde_json::from_value(roles["LITHO_INBOX"].clone())?,
            serde_json::from_value(roles["MONITOR_INPUT"].clone())?,
            message("etch_endpoint"),
            message("etch_endpoint"),
            1 + usize::from(retry_endpoint),
        ),
        (
            "LITHOGRAPHER",
            "check_monitoring",
            serde_json::from_value(roles["PHOTO_METER"].clone())?,
            u8::try_from(roles["STAGE"].as_u64().unwrap()).unwrap(),
            serde_json::from_value(roles["LITHO_INBOX"].clone())?,
            serde_json::from_value(roles["MONITOR_INPUT"].clone())?,
            message("ExposureDose"),
            message("ExposureDose"),
            1 + usize::from(retry_control),
        ),
        (
            "ETCHER",
            "check_monitoring",
            serde_json::from_value(roles["ETCH_METER"].clone())?,
            u8::try_from(roles["ETCH_CHAMBER"].as_u64().unwrap()).unwrap(),
            serde_json::from_value(roles["ETCH_INBOX"].clone())?,
            serde_json::from_value(roles["MONITOR_INPUT"].clone())?,
            message("etch_rate"),
            message("etch_rate"),
            1 + usize::from(retry_control),
        ),
        (
            "DEPOSITOR",
            "check_monitoring",
            operator,
            u8::try_from(roles["CVD_STAGE"].as_u64().unwrap()).unwrap(),
            depositor_file,
            serde_json::from_value(roles["MONITOR_INPUT"].clone())?,
            message("Rate"),
            message("RateResult"),
            rates.len(),
        ),
        (
            "PROCESS_CONTROLLER",
            "check_monitoring",
            operator,
            u8::try_from(roles["MONITOR"].as_u64().unwrap()).unwrap(),
            control_file,
            serde_json::from_value(roles["MONITOR_INPUT"].clone())?,
            message("CrossReference"),
            message("CrossReferenceResult"),
            1 + usize::from(retry_control),
        ),
    ] {
        let requests: Vec<_> = by_label(request_label)
            .into_iter()
            .filter(|r| r["send_view"] == caller)
            .collect();
        let replies: Vec<_> = by_label(reply_label)
            .into_iter()
            .filter(|r| r["receive_commit"]["view"] == caller)
            .collect();
        let scored: Vec<_> = events
            .iter()
            .filter(|e| {
                e["agent"] == agent
                    && e["tool"] == tool
                    && (request_label != message("ExposureDose")
                        || e["args"]["check_type"] == "exposure_dose")
                    && (request_label != message("etch_endpoint")
                        || e["args"]["check_type"] == "etch_endpoint")
            })
            .collect();
        let starts: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r["direction"] == "domain_start"
                    && r["message"]["agent"] == agent
                    && r["message"]["tool"] == tool
                    && (request_label != message("ExposureDose")
                        || r["message"]["arguments"]["check_type"] == "exposure_dose")
                    && (request_label != message("etch_endpoint")
                        || r["message"]["arguments"]["check_type"] == "etch_endpoint")
            })
            .collect();
        assert_eq!(
            [requests.len(), replies.len(), scored.len(), starts.len()],
            [count; 4]
        );
        for i in 0..count {
            let receive = &requests[i]["receive_commit"];
            assert_eq!(receive["view"], resource);
            assert_eq!(receive["actor"], actor);
            assert_eq!(requests[i]["send_view"], caller);
            assert_eq!(replies[i]["send_actor"], responder);
            assert_eq!(replies[i]["receive_commit"]["view"], caller);
            let effect =
                received_instrument_effect(rows, requests[i], replies[i], starts[i], scored[i])?;
            let confirmation = if request_label == message("Measure") {
                Some(reports[i])
            } else if request_label == message("CrossReference") {
                Some(checks[i])
            } else if request_label == message("Rate") {
                Some(rates[i])
            } else if request_label == message("ReactiveEtch") {
                Some(etch_results[i])
            } else {
                None
            };
            if let Some(report) = confirmation {
                let (reported_at, reported) = json_frame(rows, report)?;
                assert_eq!(report["send_actor"], actor);
                assert_eq!(report["receive_commit"]["actor"], responder);
                assert!(
                    effect["effect_end_row"].as_u64().unwrap() < reported_at as u64
                        && report["receive_commit"]["journal_row"].as_u64().unwrap()
                            < effect["reply_row"].as_u64().unwrap()
                );
                assert_eq!(reported, effect["result"]);
            }
            evidence.push(effect);
        }
    }
    let growth = evidence
        .iter()
        .find(|e| e["arguments"]["process"] == "cvd_growth")
        .ok_or("missing CVD growth")?;
    let rate = evidence
        .iter()
        .find(|e| e["arguments"]["check_type"] == "deposition_rate")
        .ok_or("missing in-situ rate")?;
    assert!(
        growth["effect_start_row"].as_u64().unwrap() < rate["effect_start_row"].as_u64().unwrap()
            && rate["effect_start_row"].as_u64().unwrap()
                < growth["effect_end_row"].as_u64().unwrap()
    );
    let mount = &evidence[0];
    let first_measure = &evidence[1];
    assert!(
        mount["effect_start_row"].as_u64().unwrap()
            < first_measure["effect_start_row"].as_u64().unwrap()
            && first_measure["effect_start_row"].as_u64().unwrap()
                < mount["effect_end_row"].as_u64().unwrap(),
        "the in-situ measurement must start during the physical load call"
    );
    Ok(evidence)
}

fn check_spot_check_effects(
    roles: &Value,
    epf: &Value,
    journal_rows: &[Value],
    score: &Value,
    etch_first: bool,
) -> Result<()> {
    let calls = epf["sessions"][0]["acceptances"]
        .as_array()
        .ok_or("EPF source occurrences")?;
    let paths = [
        (
            "run_deposition",
            u8::try_from(roles["DEPOSITION_CHAMBER"].as_u64().unwrap()).unwrap(),
        ),
        (
            "run_etch_process",
            u8::try_from(roles["ETCH_CHAMBER"].as_u64().unwrap()).unwrap(),
        ),
    ];
    let mut previous_reply = None;
    for (name, chamber) in if etch_first {
        [paths[1], paths[0]]
    } else {
        paths
    } {
        let accepted = |from: u8, to: u8| -> Result<&Value> {
            let mut matches = calls
                .iter()
                .filter(|c| c["send_view"] == from && c["receive_commit"]["view"] == to);
            let call = matches.next().ok_or("missing instrument delivery")?;
            assert!(matches.next().is_none(), "duplicate instrument delivery");
            Ok(call)
        };
        let request = accepted(
            u8::try_from(roles["WAFER"].as_u64().unwrap()).unwrap(),
            chamber,
        )?;
        let reply = accepted(
            chamber,
            u8::try_from(roles["WAFER"].as_u64().unwrap()).unwrap(),
        )?;
        let (request_sent, _) = json_frame(journal_rows, request)?;
        if let Some(previous) = previous_reply {
            assert!(
                previous < request_sent,
                "the next check used the wafer before receiving its result"
            );
        }
        previous_reply = Some(
            reply["receive_commit"]["journal_row"]
                .as_u64()
                .ok_or("wafer reply row")? as usize,
        );
        let effects: Vec<_> = journal_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                let m = &row["message"];
                m["agent"] == "INTEGRATION_ENGINEER"
                    && m["tool"] == name
                    && (row["direction"] == "domain_start" || row["direction"] == "domain_end")
            })
            .collect();
        let [(start, invoked), (end, finished)] = effects[..] else {
            panic!("two effect boundaries required for {name}");
        };
        assert_eq!(invoked["direction"], "domain_start");
        assert_eq!(finished["direction"], "domain_end");
        assert_eq!(invoked["message"]["id"], finished["message"]["id"]);
        assert!(start < end);
        let mut scored = score["events"]
            .as_array()
            .ok_or("missing scored wafer events")?
            .iter()
            .filter(|e| e["agent"] == "INTEGRATION_ENGINEER" && e["tool"] == name);
        let effect = scored.next().ok_or("missing scored spot check")?;
        assert!(scored.next().is_none(), "extra scored spot check");
        received_instrument_effect(journal_rows, request, reply, (start, invoked), effect)?;
        assert_eq!(effect["success"], true);
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the built manufacturing runtime and wash guest"]
async fn wash_manufactures_all_six_jobs_on_their_shared_workstations() -> Result<()> {
    use crate::{evidence::archive_build_receipt, process, projection};
    use std::path::PathBuf;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_WASI_BINARY").ok_or("manufacturing runtime")?,
    )
    .canonicalize()?;
    let output =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("manufacturing evidence")?);
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_MANUFACTURING_BUILD_RECEIPT")
            .ok_or("manufacturing build receipt")?,
    );
    let wash = root.join("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    fs::create_dir(&output)?;
    archive_build_receipt(&binary, &receipt, &output)?;
    let artifact = projection::archive(&binary, &output, "11M")?;
    let projection_path = output.join("projection.json");
    let projection_sha256 = crate::evidence::digest(&projection_path)?;
    let jobs = [
        ("WORKER_A", "alpha_job_a", "ALPHA"),
        ("WORKER_B", "beta_job_b", "BETA"),
        ("WORKER_C", "gamma_job_c", "GAMMA"),
    ];
    for (index, order) in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
    .into_iter()
    .enumerate()
    {
        for (profile, failures) in [
            ("serial", &[][..]),
            (
                "other-station-rejections",
                &[
                    ("inspect_quality", 3),
                    ("inspect_quality", 4),
                    ("inspect_quality", 5),
                ][..],
            ),
            ("contended", &[("inspect_quality", 0)][..]),
            ("dispatch-during-work", &[("inspect_quality", 0)][..]),
            (
                "three-rejections",
                &[
                    ("inspect_quality", 0),
                    ("inspect_quality", 1),
                    ("inspect_quality", 2),
                ][..],
            ),
        ] {
            let other_jobs = [
                ("WORKER_C", "alpha_job_c", "ALPHA_C", "ALPHA_SERVICE"),
                ("WORKER_A", "beta_job_a", "BETA_A", "BETA_SERVICE"),
                ("WORKER_B", "gamma_job_b", "GAMMA_B", "GAMMA_SERVICE"),
            ];
            let before = (index * 4
                + match profile {
                    "serial" => 0,
                    "contended" => 1,
                    "dispatch-during-work" => 2,
                    _ => 3,
                })
                % 8;
            let batch = matches!(profile, "contended" | "three-rejections");
            let streamed = profile == "dispatch-during-work";
            let directory = output.join(format!("{index}-{profile}"));
            fs::create_dir(&directory)?;
            let journal = directory.join("events.jsonl");
            let trace = Trace::create(&journal)?;
            fs::hard_link(&projection_path, directory.join("projection.json"))?;
            crate::evidence::save(
                &directory.join("case.json"),
                &json!({
                    "runtime_binary_sha256": artifact["runtime_binary_sha256"],
                    "compiled_projection_sha256": projection_sha256,
                }),
            )?;
            let (world, _) = Simulator::new("11M", 0, 1.0, trace.clone(), failures)?;
            let job_seconds = if streamed { 1.6 } else { 0.4 };
            world
                .set_test_delays(json!({"do_job":[job_seconds,job_seconds],"inspect_quality":[0.05,0.05],"dispatch_rework":[0.05,0.05]}))
                .await?;
            let (guest, runtime) =
                process::guest(&binary, &wash, &["11M"], &directory, trace, world)?;
            let ((parts, inspections, orders), outcome) = hibana_runtime::race(
                async {
                    async {
                        let mut parts = Vec::new();
                        let mut inspections = Vec::new();
                        let mut orders = Vec::new();
                        let disposition = async |agent: &str, job: &str, work_file: &str, inspection: &Value| -> Result<Option<Value>> {
                            let accepted = inspection["success"].as_bool().ok_or("inspection verdict")?;
                            let verdict_dir = if accepted { "approved" } else { "failed" };
                            let notice = shell_json_result(&guest.command("DISPATCHER", format!("cat /service/{verdict_dir}/{job}")).await?)?;
                            assert_eq!(&notice, inspection);
                            let args = json!({"job_id":job,"reason":format!("inspect {} again",inspection["data"]["input_part"]["artifact_id"].as_str().ok_or("part ID")?)});
                            let path = format!("/service/dispatch_rework/{job}");
                            let nonblocking = format!("printf '%s' '{args}' | dd of={path} oflag=nonblock");
                            if accepted {
                                let refused = guest.command("DISPATCHER", nonblocking).await?;
                                assert!(refused["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")), "{refused}");
                                return Ok(None);
                            }
                            let job_args = json!({"job_id":job});
                            let premature = guest.command(agent, format!("printf '%s' '{job_args}' | dd of={work_file} oflag=nonblock")).await?;
                            assert!(premature["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{premature}");
                            let other = json!({"job_id":"unassigned_job"});
                            let invalid = guest.command("DISPATCHER", format!("printf '%s' '{other}' > {path}")).await?;
                            assert!(invalid["stdout"].as_str().is_some_and(|s|s.contains("Invalid argument")), "{invalid}");
                            let sent = guest.command("DISPATCHER", format!("printf '%s' '{args}' > {path}")).await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                            let order = shell_json_result(&guest.command("DISPATCHER", format!("cat {path}")).await?)?;
                            assert_eq!(order["data"]["inspection"], notice["data"]["receipt"]);
                            assert_eq!(order["data"]["input_part"], notice["data"]["input_part"]);
                            assert_eq!(order["data"]["reason"], args["reason"]);
                            let duplicate = guest.command("DISPATCHER", nonblocking).await?;
                            assert!(duplicate["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")), "{duplicate}");
                            Ok(Some(order))
                        };
                        let inspect = async |i: usize, part: &Value| -> Result<Value> {
                            let job = jobs[i].1;
                            let path = format!("/service/inspect_quality/{job}");
                            let wrong = json!({"job_id":jobs[(i+1)%3].1});
                            let refused = guest.command("INSPECTOR", format!("printf '%s' '{wrong}' > {path}")).await?;
                            assert!(refused["stdout"].as_str().is_some_and(|s| s.contains("Invalid argument")), "{refused}");
                            let arguments = json!({"job_id":job});
                            let sent = guest.command("INSPECTOR", format!("printf '%s' '{arguments}' > {path}")).await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                            let duplicate = guest.command("INSPECTOR", format!("printf '%s' '{arguments}' | dd of={path} oflag=nonblock")).await?;
                            assert!(duplicate["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")), "{duplicate}");
                            let result = shell_json_result(&guest.command("INSPECTOR", "cat /service/inspection_result".into()).await?)?;
                            assert_eq!(result["data"]["input_part"], part["data"]["part"]);
                            let repeat = guest.command("INSPECTOR", format!("printf '%s' '{arguments}' | dd of={path} oflag=nonblock")).await?;
                            assert!(repeat["stdout"].as_str().is_some_and(|s| s.contains("Permission denied")), "{repeat}");
                            Ok(result)
                        };
                        if index == 0 && profile == "serial" {
                            for (agent, _, _) in jobs {
                                let early = guest.command(agent, "dd if=/service/do_job iflag=nonblock".into()).await?;
                                assert!(early["stdout"].as_str().is_some_and(|s| s.contains("Permission denied")), "result read before submission: {agent}: {early}");
                            }
                        }
                        let early = guest.command("INSPECTOR", "dd if=/service/parts/alpha_job_a iflag=nonblock".into()).await?;
                        assert!(early["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{early}");
                        let premature = guest.command("DISPATCHER", "printf '%s' '{\"job_id\":\"alpha_job_a\"}' | dd of=/service/dispatch_rework/alpha_job_a oflag=nonblock".into()).await?;
                        assert!(premature["stdout"].as_str().is_some_and(|s| s.contains("Permission denied") || s.contains("Resource temporarily unavailable")), "{premature}");
                        let submit = async |i: usize| -> Result<()> {
                            let (agent, job, _) = jobs[i];
                            let invalid = json!({"job_id":jobs[(i+1)%3].1});
                            let refusal = guest
                                .command(
                                    agent,
                                    format!("printf '%s' '{invalid}' > /service/do_job"),
                                )
                                .await?;
                            assert!(
                                refusal["stdout"]
                                    .as_str()
                                    .is_some_and(|s| s.contains("Invalid argument")),
                                "{refusal}"
                            );
                            let args = json!({"job_id":job});
                            let write = guest
                                .command(agent, format!("printf '%s' '{args}' > /service/do_job"))
                                .await?;
                            assert_eq!(write["stdout"], "wash> ", "{write}");
                            Ok(())
                        };
                        let submit_other = async |i: usize| -> Result<()> {
                            let (agent, job, _, _) = other_jobs[i];
                            let args = json!({"job_id":job});
                            let write = guest.command(agent,format!("printf '%s' '{args}' > /service/do_job_at_other_station")).await?;
                            assert_eq!(write["stdout"], "wash> ", "{write}");
                            Ok(())
                        };
                        let read_other = async |i: usize| -> Result<()> {
                            let (agent,job,_,_) = other_jobs[i];
                            let part = shell_json_result(&guest.command(agent, "cat /service/do_job_at_other_station".into()).await?)?;
                            assert_eq!(part["data"]["part"]["job_id"], job);
                            assert_eq!(part["data"]["part"]["attempt"], 1);
                            let received = shell_json_result(&guest.command("INSPECTOR", format!("cat /service/parts/{job}")).await?)?;
                            assert_eq!(received, part["data"]["part"]);
                            Ok(())
                        };
                        for &i in &order {
                            if before & (1 << i) != 0 { submit_other(i).await?; }
                        }
                        for &i in &order {
                            if before & (1 << i) != 0 { read_other(i).await?; }
                        }
                        for &i in &order {
                            let (agent, job, _) = jobs[i];
                            if !(streamed && i == order[1]) {
                                submit(i).await?;
                            }
                            if index == 0 && profile == "serial" && i == order[0] {
                                let (other, job, _, _) = other_jobs[i];
                                let args = json!({"job_id":job});
                                let busy = guest.command(other, format!("printf '%s' '{args}' | dd of=/service/do_job_at_other_station oflag=nonblock")).await?;
                                let output = busy["stdout"].as_str().ok_or("competing workstation write")?;
                                assert!(output.contains("Resource temporarily unavailable") || output.contains("Permission denied"), "{busy}");
                            }
                            if !batch {
                                let part = shell_json_result(
                                    &guest.command(agent, "cat /service/do_job".into()).await?,
                                )?;
                                let delivered = shell_json_result(&guest.command("INSPECTOR", format!("cat /service/parts/{job}")).await?)?;
                                assert_eq!(delivered, part["data"]["part"]);
                                if streamed && i == order[0] {
                                    submit(order[1]).await?;
                                }
                                let inspected = inspect(i, &part).await?;
                                if inspected["success"] == false {
                                    let dispatched = disposition(jobs[i].0, jobs[i].1, "/service/do_job", &inspected).await?.ok_or("missing rework order")?;
                                    let update = shell_json_result(&guest.command(agent, "cat /service/job_update".into()).await?)?;
                                    assert_eq!(update, dispatched);
                                    orders.push(dispatched);
                                } else {
                                    let update = shell_json_result(&guest.command(agent, "cat /service/job_update".into()).await?)?;
                                    assert_eq!(update, inspected);
                                    assert!(disposition(jobs[i].0, jobs[i].1, "/service/do_job", &inspected).await?.is_none());
                                }
                                inspections.push((i, inspected));
                                parts.push((i, part));
                            }
                        }
                        if batch {
                            for i in order.into_iter().rev() {
                                let part = shell_json_result(
                                    &guest
                                        .command(jobs[i].0, "cat /service/do_job".into())
                                        .await?,
                                )?;
                                parts.push((i, part));
                            }
                            for (i, part) in parts.iter().cycle().skip(1).take(parts.len()) {
                                let inspected = inspect(*i, part).await?;
                                if index % 2 == 0 && let Some(order) = disposition(jobs[*i].0, jobs[*i].1, "/service/do_job", &inspected).await? {
                                    orders.push(order);
                                }
                                inspections.push((*i, inspected));
                            }
                            for (i, part) in &parts {
                                let job = jobs[*i].1;
                                let delivered = shell_json_result(&guest.command("INSPECTOR", format!("cat /service/parts/{job}")).await?)?;
                                assert_eq!(delivered, part["data"]["part"]);
                            }
                            if index % 2 == 1 {
                                for (i, inspected) in inspections.iter().rev() {
                                    if let Some(order) = disposition(jobs[*i].0, jobs[*i].1, "/service/do_job", inspected).await? {
                                        orders.push(order);
                                    }
                                }
                            }
                        }
                        for (worker, inspection) in inspections.iter().filter(|_| batch) {
                            let update = shell_json_result(&guest.command(jobs[*worker].0, "cat /service/job_update".into()).await?)?;
                            let expected = if inspection["success"] == true { inspection } else {
                                orders.iter().find(|order| order["data"]["inspection"]==inspection["data"]["receipt"]).ok_or("missing work order")?
                            };
                            assert_eq!(&update, expected);
                        }
                        for &i in &order {
                            if before & (1 << i) == 0 { submit_other(i).await?; read_other(i).await?; }
                        }
                        for i in order {
                            let job = other_jobs[i].1;
                            let args = json!({"job_id":job});
                            let sent = guest.command("INSPECTOR",format!("printf '%s' '{args}' > /service/inspect_quality/{job}")).await?;
                            assert_eq!(sent["stdout"], "wash> ", "{sent}");
                        }
                        let mut returned = std::collections::BTreeSet::new();
                        for position in 3..6 {
                            let reading = shell_json_result(&guest.command("INSPECTOR", "cat /service/inspection_result".into()).await?)?;
                            let job = reading["data"]["job_id"].as_str().ok_or("missing measured job")?;
                            assert!(other_jobs.iter().any(|entry|entry.1 == job) && returned.insert(job.to_owned()));
                            assert_eq!(reading["data"]["input_part"]["artifact_id"],format!("part/{job}/1"));
                            assert_eq!(reading["success"], !failures.contains(&("inspect_quality",position)));
                            let agent = other_jobs.iter().find(|entry| entry.1 == job).ok_or("unassigned station job")?.0;
                            let order = disposition(agent, job, "/service/do_job_at_other_station", &reading).await?;
                            let update = shell_json_result(&guest.command(agent, "cat /service/other_job_update".into()).await?)?;
                            assert_eq!(update, *order.as_ref().unwrap_or(&reading));
                            orders.extend(order);
                        }
                        assert_eq!(guest.command("DISPATCHER", "exit 0".into()).await?["exit_status"], 0);
                        assert_eq!(guest.command("INSPECTOR", "exit 0".into()).await?["exit_status"], 0);
                        for (i, inspected) in &inspections {
                            let (agent, job, _) = jobs[*i];
                            if inspected["success"] == true {
                                let args = json!({"job_id":job});
                                let repeat = guest.command(agent, format!("printf '%s' '{args}' | dd of=/service/do_job oflag=nonblock")).await?;
                                assert!(repeat["stdout"].as_str().is_some_and(|s|s.contains("Permission denied")),"{repeat}");
                            }
                            assert_eq!(
                                guest.command(agent, "exit 0".into()).await?["exit_status"],
                                0
                            );
                        }
                        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((parts, inspections, orders))
                    };
                    async { runtime.await };
                },
                hibana_runtime::host::deadline(std::time::Duration::from_secs(30)),
            )
            .await?;
            save(&directory.join("runtime-result.json"), &outcome)?;
            assert_eq!(
                outcome["protocol"],
                json!({"type":"run_result","status":"complete","scope":"workstation_tool_jobs"}),
                "{outcome}"
            );
            assert_eq!(outcome["runtime_exit_code"], 0);
            assert_eq!(outcome["backend_errors"], json!([]));
            assert_eq!(outcome["score"]["complete"], failures.is_empty());
            assert_eq!(outcome["score"]["violations"], json!([]));
            let events = outcome["score"]["events"]
                .as_array()
                .ok_or("station events")?;
            assert_eq!(orders.len(), failures.len());
            assert_eq!(events.len(), 12 + orders.len());
            let rows: Vec<Value> = fs::read_to_string(&journal)?
                .lines()
                .map(serde_json::from_str)
                .collect::<std::result::Result<_, _>>()?;
            let labels = projection::frames(&artifact, &rows)?;
            let epf = projection::check_dynamic(&artifact, &rows)?;
            let verified =
                crate::manufacturing_rework_audit::verify(&artifact, &rows, &labels, &epf)?;
            assert_eq!(verified["simulator_complete"], outcome["score"]["complete"]);
            assert_eq!(verified["domain_calls"], events.len());
            let calls = epf["sessions"][0]["acceptances"]
                .as_array()
                .ok_or("EPF accepted calls")?;
            let roles = &artifact["compiled"]["roles"];
            for direction in ["domain_start", "domain_end"] {
                assert_eq!(
                    rows.iter()
                        .filter(|row| row["direction"] == direction)
                        .count(),
                    12 + orders.len()
                );
            }
            let transfer = |source: &str, destination: &str| {
                let mut transfers: Vec<_> = calls
                    .iter()
                    .filter(|call| {
                        call["send_view"] == roles[source]
                            && call["receive_commit"]["view"] == roles[destination]
                    })
                    .collect();
                transfers
                    .sort_by_key(|call| call["receive_commit"]["journal_row"].as_u64().unwrap());
                transfers
            };
            let samples = transfer("GAUGE", "READOUT");
            let readings = transfer("READOUT", "GAUGE");
            let published = transfer("GAUGE", "INSPECTOR");
            let measured: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r["direction"] == "domain_start" && r["message"]["tool"] == "inspect_quality"
                })
                .collect();
            assert_eq!(
                [
                    samples.len(),
                    readings.len(),
                    published.len(),
                    measured.len()
                ],
                [6; 4]
            );
            let mut gauge_effects = std::collections::BTreeMap::new();
            let mut previous = 0;
            for (((sample, reading), published), start) in samples
                .into_iter()
                .zip(readings)
                .zip(published)
                .zip(measured)
            {
                let job = start.1["message"]["arguments"]["job_id"]
                    .as_str()
                    .ok_or("missing inspected job")?;
                let scored = events
                    .iter()
                    .find(|event| {
                        event["tool"] == "inspect_quality" && event["args"]["job_id"] == job
                    })
                    .ok_or("unscored inspection")?;
                let effect = received_instrument_effect(&rows, sample, reading, start, scored)?;
                assert_eq!(sample["send_actor"], roles["BENCH"]);
                let (sent, value) = json_frame(&rows, published)?;
                assert_eq!(value, effect["result"]);
                assert!(previous < effect["request_sent_row"].as_u64().unwrap());
                assert!(effect["reply_received_row"].as_u64().unwrap() < sent as u64);
                previous = sent as u64;
                assert!(gauge_effects.insert(job.to_owned(), effect).is_none());
            }
            let starts: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row["direction"] == "domain_start"
                        && row["message"]["tool"] == "do_job"
                        && jobs
                            .iter()
                            .any(|(_, job, _)| row["message"]["arguments"]["job_id"] == *job)
                })
                .collect();
            assert_eq!(starts.len(), 3);
            let mut effects = Vec::new();
            let mut checked_inspections = Vec::new();
            let mut checked_orders = Vec::new();
            for (i, part) in parts {
                let (agent, job, station) = jobs[i];
                assert_eq!(part["success"], true);
                let (start, entry) = *starts
                    .iter()
                    .find(|(_, row)| row["message"]["agent"] == agent)
                    .ok_or("missing worker call")?;
                let find = |permitted: &[u8], sender: &Value, receiver: &Value| -> Result<&Value> {
                    let mut matches = calls.iter().filter(|call| {
                        call["send_actor"] == *sender
                            && call["receive_commit"]["actor"] == *receiver
                            && call["frame_id"]
                                .as_u64()
                                .and_then(|id| labels.get(&id))
                                .is_some_and(|l| permitted.contains(l))
                    });
                    let call = matches
                        .next()
                        .ok_or("missing EPF workstation or tool delivery")?;
                    assert!(matches.next().is_none());
                    Ok(call)
                };
                let vm = &roles[format!("{station}_OWNER")];
                let worker = &roles[agent];
                let depot = &roles["DEPOT"];
                let submitted = find(&[35], vm, worker)?;
                let returned = find(&[21], worker, vm)?;
                let delivered = find(&[i as u8], worker, &roles["INSPECTOR_VM"])?;
                assert_eq!(delivered["receive_commit"]["view"], roles["INSPECTOR"]);
                assert_eq!(json_frame(&rows, delivered)?.1, part["data"]["part"]);
                let check = find(&[i as u8], &roles["INSPECTOR_VM"], worker)?;
                let presented = find(&[i as u8], worker, &roles["BENCH"])?;
                let measured = find(&[i as u8], &roles["BENCH"], worker)?;
                assert_eq!(check["send_view"], roles["INSPECTOR"]);
                let args = json!({"job_id":job});
                let inspection = gauge_effects.get(job).ok_or("tool job was not measured")?;
                assert_eq!(args, json_frame(&rows, check)?.1);
                assert_eq!(args, json_frame(&rows, presented)?.1);
                assert!(
                    check["receive_commit"]["journal_row"].as_u64().unwrap()
                        < json_frame(&rows, presented)?.0 as u64
                );
                assert!(
                    presented["receive_commit"]["journal_row"].as_u64().unwrap()
                        < inspection["request_sent_row"].as_u64().unwrap()
                );
                assert_eq!(json_frame(&rows, measured)?.1, inspection["result"]);
                let (position, (_, shell)) = inspections
                    .iter()
                    .enumerate()
                    .find(|(_, (job_index, _))| *job_index == i)
                    .ok_or("shell inspection")?;
                assert_eq!(inspection["result"], *shell);
                assert_eq!(
                    shell["success"],
                    !failures.contains(&("inspect_quality", position as u32))
                );
                assert_eq!(shell["data"]["input_part"], part["data"]["part"]);
                checked_inspections.push(inspection.clone());
                assert_eq!(submitted["send_view"], roles[station]);
                assert_eq!(returned["receive_commit"]["view"], roles[station]);
                let using_tool = find(&[3, 4], worker, depot)?;
                let tool_reply = find(&[5, 6], depot, worker)?;
                assert_eq!(
                    using_tool["send_view"],
                    tool_reply["receive_commit"]["view"]
                );
                assert_eq!(json_frame(&rows, submitted)?.1, json!({"job_id":job}));
                assert_eq!(json_frame(&rows, returned)?.1, part);
                assert!(
                    submitted["receive_commit"]["journal_row"]
                        .as_u64()
                        .ok_or("station receipt")?
                        < json_frame(&rows, using_tool)?.0 as u64
                );
                assert!(
                    tool_reply["receive_commit"]["journal_row"]
                        .as_u64()
                        .ok_or("tool result receipt")?
                        < json_frame(&rows, returned)?.0 as u64
                );
                assert!(
                    tool_reply["receive_commit"]["journal_row"]
                        .as_u64()
                        .ok_or("missing tool reply")?
                        < json_frame(&rows, delivered)?.0 as u64
                );
                let event = events
                    .iter()
                    .find(|e| {
                        e["agent"] == agent && e["tool"] == "do_job" && e["args"]["job_id"] == job
                    })
                    .ok_or("worker score")?;
                let effect = received_instrument_effect(
                    &rows,
                    using_tool,
                    tool_reply,
                    (start, entry),
                    event,
                )?;
                assert_eq!(effect["arguments"], json!({"job_id":job}));
                assert_eq!(effect["result"], part);
                assert_eq!(
                    part["data"]["part"]["station"],
                    format!("STATION_{station}")
                );
                let machine = &roles[format!("{station}_MACHINE")];
                let request = find(&[10], worker, depot)?;
                let reply = find(
                    &[machine.as_u64().ok_or("physical station role")? as u8],
                    depot,
                    worker,
                )?;
                assert_eq!(request["send_view"], *machine);
                assert_eq!(reply["receive_commit"]["view"], *machine);
                let station_effect =
                    received_instrument_effect(&rows, request, reply, (start, entry), event)?;
                assert_eq!(station_effect["rpc_id"], effect["rpc_id"]);
                effects.push(json!({"worker":agent,"machine_effect":station_effect,"station_request":submitted["frame_id"],"station_result":returned["frame_id"],"part_delivery":delivered["frame_id"],"tool":using_tool["send_view"],"effect":effect}));
            }
            let mut other_effects = Vec::new();
            for (i, (agent, job, view, service)) in other_jobs.iter().copied().enumerate() {
                let station = jobs[i].2;
                let owner = &roles[format!("{station}_OWNER")];
                let machine = &roles[format!("{station}_MACHINE")];
                let cpu = &roles[service];
                let file = &roles[view];
                let wire = |label: u8, from: &Value, to: &Value| -> Result<&Value> {
                    let mut found = calls.iter().filter(|c| {
                        c["send_view"] == *from
                            && c["receive_commit"]["view"] == *to
                            && c["frame_id"].as_u64().and_then(|f| labels.get(&f)) == Some(&label)
                    });
                    let row = found.next().ok_or("missing workstation receipt")?;
                    assert!(found.next().is_none());
                    Ok(row)
                };
                let request = wire(35, file, cpu)?;
                let actual = wire(20, machine, cpu)?;
                let reply = wire(21, cpu, machine)?;
                let published = wire(21, cpu, file)?;
                assert_eq!(request["send_actor"], *owner);
                let score = events
                    .iter()
                    .find(|v| v["tool"] == "do_job" && v["args"]["job_id"] == job)
                    .ok_or("unscored station job")?;
                let primary = events
                    .iter()
                    .find(|v| v["tool"] == "do_job" && v["args"]["job_id"] == jobs[i].1)
                    .ok_or("unscored tool job")?;
                let (earlier, later) = if before & (1 << i) != 0 {
                    (score, primary)
                } else {
                    (primary, score)
                };
                assert!(
                    earlier["completed_at"].as_f64().ok_or("finish")?
                        <= later["timestamp"].as_f64().ok_or("start")?
                );
                let start = rows
                    .iter()
                    .enumerate()
                    .find(|(_, r)| {
                        r["direction"] == "domain_start"
                            && r["message"]["tool"] == "do_job"
                            && r["message"]["arguments"]["job_id"] == job
                    })
                    .ok_or("missing station effect")?;
                let effect = received_instrument_effect(&rows, actual, reply, start, score)?;
                assert_eq!(json_frame(&rows, request)?.1, effect["arguments"]);
                assert_eq!(json_frame(&rows, published)?.1, effect["result"]);
                let job_id = match i {
                    0 => 5,
                    1 => 3,
                    2 => 4,
                    _ => unreachable!(),
                };
                let delivered = wire(job_id, cpu, &roles["INSPECTOR"])?;
                assert_eq!(
                    json_frame(&rows, delivered)?.1,
                    effect["result"]["data"]["part"]
                );
                assert!(
                    json_frame(&rows, delivered)?.0 as u64
                        > effect["effect_end_row"]
                            .as_u64()
                            .ok_or("manufacturing completion")?
                );
                assert_eq!(score["agent"], agent);
                let inspection = gauge_effects
                    .get(job)
                    .ok_or("station job was not measured")?;
                let check = wire(job_id, &roles["INSPECTOR"], cpu)?;
                let presented = wire(job_id, cpu, &roles["BENCH"])?;
                let measured = wire(job_id, &roles["BENCH"], cpu)?;
                assert_eq!(json_frame(&rows, check)?.1, inspection["arguments"]);
                assert_eq!(json_frame(&rows, presented)?.1, inspection["arguments"]);
                assert!(
                    presented["receive_commit"]["journal_row"].as_u64().unwrap()
                        < inspection["request_sent_row"].as_u64().unwrap()
                );
                assert!(
                    check["receive_commit"]["journal_row"].as_u64().unwrap()
                        < json_frame(&rows, presented)?.0 as u64
                );
                assert_eq!(json_frame(&rows, measured)?.1, inspection["result"]);
                assert_eq!(
                    inspection["result"]["data"]["input_part"],
                    effect["result"]["data"]["part"]
                );
                checked_inspections.push(inspection.clone());
                other_effects.push(effect);
            }
            for (job, worker, file, job_id) in jobs
                .iter()
                .enumerate()
                .map(|(id, (worker, job, file))| (*job, *worker, *file, id as u8))
                .chain(other_jobs.iter().map(|(_, job, file, service)| {
                    let id = match *job {
                        "beta_job_a" => 3,
                        "gamma_job_b" => 4,
                        "alpha_job_c" => 5,
                        _ => unreachable!(),
                    };
                    (*job, *service, *file, id)
                }))
            {
                let result = &gauge_effects[job]["result"];
                let approved = result["success"] == true;
                let wire = |label: u8, from: &Value, to: &Value| -> Result<&Value> {
                    let mut found = calls.iter().filter(|call| {
                        call["send_view"] == *from
                            && call["receive_commit"]["view"] == *to
                            && call["frame_id"].as_u64().and_then(|f| labels.get(&f))
                                == Some(&label)
                    });
                    let only = found.next().ok_or("missing disposition delivery")?;
                    assert!(found.next().is_none());
                    Ok(only)
                };
                let worker = &roles[worker];
                let dispatcher = &roles["DISPATCHER"];
                let notice = wire(if approved { 25 } else { 26 }, worker, dispatcher)?;
                assert_eq!(notice["receive_commit"]["actor"], roles["DISPATCHER_VM"]);
                assert_eq!(json_frame(&rows, notice)?.1, *result);
                let (_, update) = json_frame(&rows, wire(36, worker, &roles[file])?)?;
                if approved {
                    assert_eq!(update, *result);
                    continue;
                }
                let request = wire(27, dispatcher, worker)?;
                let reply = wire(job_id, worker, dispatcher)?;
                let rpc = rows
                    .iter()
                    .enumerate()
                    .find(|(_, r)| {
                        r["direction"] == "domain_start"
                            && r["message"]["tool"] == "dispatch_rework"
                            && r["message"]["arguments"]["job_id"] == job
                    })
                    .ok_or("dispatch RPC")?;
                let scored = events
                    .iter()
                    .find(|e| e["tool"] == "dispatch_rework" && e["args"]["job_id"] == job)
                    .ok_or("dispatch score")?;
                let effect = received_instrument_effect(&rows, request, reply, rpc, scored)?;
                assert!(orders.contains(&effect["result"]));
                assert_eq!(update, effect["result"]);
                assert_eq!(update["data"]["input_part"], result["data"]["input_part"]);
                assert_eq!(update["data"]["inspection"], result["data"]["receipt"]);
                checked_orders.push(effect);
            }
            if before == 7 {
                let last_start = other_effects
                    .iter()
                    .map(|e| e["effect_start_row"].as_u64().unwrap())
                    .max()
                    .unwrap();
                let first_finish = other_effects
                    .iter()
                    .map(|e| e["effect_end_row"].as_u64().unwrap())
                    .min()
                    .unwrap();
                assert!(
                    last_start < first_finish,
                    "three independent workstations: {other_effects:?}"
                );
            }
            if !batch {
                assert_eq!(
                    starts
                        .iter()
                        .map(|(_, r)| r["message"]["agent"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    order.map(|i| jobs[i].0)
                );
            }
            let peak = starts
                .iter()
                .map(|(t, _)| {
                    effects
                        .iter()
                        .filter(|e| {
                            e["effect"]["effect_start_row"].as_u64().unwrap() <= *t as u64
                                && e["effect"]["effect_end_row"].as_u64().unwrap() > *t as u64
                        })
                        .count()
                })
                .max()
                .unwrap();
            assert_eq!(peak, if batch { 2 } else { 1 }, "{directory:?} {effects:?}");
            let intervals = events
                .iter()
                .filter(|event| {
                    event["tool"] == "do_job"
                        && jobs
                            .iter()
                            .any(|(_, job, _)| event["args"]["job_id"] == *job)
                })
                .map(|event| {
                    let start = event["timestamp"].as_f64().ok_or("simulator job start")?;
                    let end = event["completed_at"]
                        .as_f64()
                        .ok_or("simulator job finish")?;
                    if end <= start {
                        return Err("invalid simulator job interval");
                    }
                    Ok((start, end))
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let simulator_peak = intervals
                .iter()
                .map(|&(t, _)| {
                    intervals
                        .iter()
                        .filter(|&&(start, end)| start <= t && t < end)
                        .count()
                })
                .max()
                .ok_or("no job intervals")?;
            assert_eq!(simulator_peak, peak);
            let dispatch_overlap = if streamed {
                let job = |i: usize| {
                    &effects
                        .iter()
                        .find(|entry| entry["worker"] == jobs[i].0)
                        .unwrap()["effect"]
                };
                let order_effect = &checked_orders[0];
                let other = job(order[1]);
                let last = job(order[2]);
                let row = |e: &Value, field: &str| e[field].as_u64().unwrap();
                assert!(row(other, "effect_start_row") < row(order_effect, "effect_start_row"));
                assert!(row(order_effect, "effect_end_row") < row(other, "effect_end_row"));
                assert!(row(order_effect, "effect_end_row") < row(last, "effect_start_row"));
                let score_event = |tool, i: usize| {
                    events
                        .iter()
                        .find(|e| e["tool"] == tool && e["args"]["job_id"] == jobs[i].1)
                        .unwrap()
                };
                let dispatch = score_event("dispatch_rework", order[0]);
                let manufacturing = score_event("do_job", order[1]);
                let start = dispatch["timestamp"].as_f64().ok_or("dispatch start")?;
                let end = dispatch["completed_at"].as_f64().ok_or("dispatch end")?;
                assert!(
                    manufacturing["timestamp"]
                        .as_f64()
                        .ok_or("other job start")?
                        < start
                        && start < end
                );
                assert!(
                    end < manufacturing["completed_at"]
                        .as_f64()
                        .ok_or("other job end")?
                );
                Some(
                    json!({"rejected_job":jobs[order[0]].1,"active_job":jobs[order[1]].1,"unsubmitted_job":jobs[order[2]].1,
                    "dispatch_start_row":order_effect["effect_start_row"],"dispatch_end_row":order_effect["effect_end_row"],
                    "other_job_start_row":other["effect_start_row"],"other_job_end_row":other["effect_end_row"],
                    "last_job_start_row":last["effect_start_row"]}),
                )
            } else {
                None
            };
            save(
                &directory.join("checked.json"),
                &json!({"order":order,"other_job_first_mask":before,"other_job_effects":other_effects,"dispatch_while_other_job_active":dispatch_overlap,"submitted_before_reads":batch,"maximum_outstanding_jobs":peak,"maximum_simultaneous_effects":simulator_peak,"epf_records":epf["captured_records"],"effects":effects,"inspections":checked_inspections,"dispatches":checked_orders,"inspection_order":events.iter().filter(|e|e["tool"]=="inspect_quality").map(|e|&e["args"]["job_id"]).collect::<Vec<_>>(),"score_complete":outcome["score"]["complete"],"model_calls":0}),
            )?;
        }
    }
    Ok(())
}
