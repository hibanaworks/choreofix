use super::*;
use std::collections::BTreeSet;

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
