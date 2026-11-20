//! Run every migrated family's scripted WASI boundary against one source snapshot.
use crate::{
    evidence::{Result, digest, save},
    sources,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub const REFACTOR_CASES: usize = 1 << 4;
pub const LAB_CASES: usize = 25;
pub const LAB_QUALITY_CASES: usize = 8;
pub const LAB_CASCADING_CASES: usize = 11;
const REGULATORY_DISCOVERY_CASES: usize = 7;
const MULTI_ROUND_DISCOVERY_CASES: usize = 3;
const RESEARCH_PAPER_CASES: usize = 2;
const SURVEY_PAPER_DEPTHS: [usize; 2] = [0, 2];

const FAMILIES: [&str; 30] = [
    "large_document",
    "cicd_pipeline",
    "full_stack_api",
    "survey_paper",
    "research_paper",
    "multi_round_discovery",
    "regulatory_discovery",
    "drug_discovery",
    "lab_cascading",
    "lab_quality",
    "lab",
    "refactor",
    "kitchen",
    "manufacturing",
    "cascading_build",
    "build",
    "document",
    "feature_delivery",
    "staged_integration",
    "critical_care",
    "multi_specialty",
    "medical",
    "coordination",
    "report",
    "review",
    "codebase",
    "writing",
    "api",
    "philosophers",
    "cicd",
];

pub const DINING_9E_ORDERS: &[(&str, &str)] = &[
    ("reconsider", "abBacCaA"),
    ("ABC", "aAbBcC"),
    ("ACB", "aAcCbB"),
    ("BAC", "bBaAcC"),
    ("BCA", "bBcCaA"),
    ("CAB", "cCaAbB"),
    ("CBA", "cCbBaA"),
];
pub const DINING_9M_ORDERS: &[(&str, &str)] = &[
    ("ABCDE", "aAbBcCdDeE"),
    ("AEDCB", "aAeEdDcCbB"),
    ("BCDEA", "bBcCdDeEaA"),
    ("BAEDC", "bBaAeEdDcC"),
    ("CDEAB", "cCdDeEaAbB"),
    ("CBAED", "cCbBaAeEdD"),
    ("DEABC", "dDeEaAbBcC"),
    ("DCBAE", "dDcCbBaAeE"),
    ("EABCD", "eEaAbBcCdD"),
    ("EDCBA", "eEdDcCbBaA"),
];
pub const DINING_9H_ORDERS: &[(&str, &str)] = &[
    ("ABCDEFG", "aAbBcCdDeEfFgG"),
    ("AGFEDCB", "aAgGfFeEdDcCbB"),
    ("BCDEFGA", "bBcCdDeEfFgGaA"),
    ("BAGFEDC", "bBaAgGfFeEdDcC"),
    ("CDEFGAB", "cCdDeEfFgGaAbB"),
    ("CBAGFED", "cCbBaAgGfFeEdD"),
    ("DEFGABC", "dDeEfFgGaAbBcC"),
    ("DCBAGFE", "dDcCbBaAgGfFeE"),
    ("EFGABCD", "eEfFgGaAbBcCdD"),
    ("EDCBAGF", "eEdDcCbBaAgGfF"),
    ("FGABCDE", "fFgGaAbBcCdDeE"),
    ("FEDCBAG", "fFeEdDcCbBaAgG"),
    ("GABCDEF", "gGaAbBcCdDeEfF"),
    ("GFEDCBA", "gGfFeEdDcCbBaA"),
];

fn dining_reports(reports: &[Value], runtime_hash: &str) -> Result<()> {
    let mut completed = BTreeSet::new();
    for report in reports {
        let task = report["task"].as_str().ok_or("dining task")?;
        let count = match task {
            "9E" => 3,
            "9M" => 5,
            "9H" => 7,
            _ => return Err("unknown dining condition".into()),
        };
        let adversarial = report["adversarial_inputs"]
            .as_bool()
            .ok_or("dining input condition")?;
        let audit = &report["audit"];
        let mutants = report["mutants"]["mutants"]
            .as_array()
            .ok_or("dining record-counterexample results")?;
        if audit["status"] != "passed"
            || audit["participants"] != count
            || audit["dynamic_protocol"]["status"] != "passed"
            || audit["actual_resource_endpoint_returns"] != 2 * count
            || audit["simulator_complete"] != true
            || mutants.len() != 20
            || mutants.iter().any(|result| result["rejected"] != true)
            || !completed.insert((task.to_owned(), adversarial))
        {
            return Err("dining qualification has a missing, failed or duplicate condition".into());
        }
        if audit["runtime_binary_sha256"] != runtime_hash {
            return Err("dining report belongs to a different runtime binary".into());
        }
    }
    let expected = ["9E", "9M", "9H"]
        .into_iter()
        .flat_map(|task| [false, true].map(|input| (task.to_owned(), input)))
        .collect();
    if completed != expected {
        return Err("dining evidence omits conditions or combines runtime binaries".into());
    }
    Ok(())
}

pub const KITCHEN_CASES: usize = KITCHEN_ORDERS.len() * 8 + 3;
pub const KITCHEN_ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

fn kitchen_reports(reports: &[Value]) -> Result<()> {
    let mut completed = BTreeSet::new();
    for report in reports {
        let condition = report["assigned_condition"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("missing assigned kitchen condition")?;
        let cases = report["cases"]
            .as_array()
            .ok_or("missing kitchen case results")?;
        if report["task"] != "12E"
            || report["status"] != "passed"
            || report["declared_conditions"] != KITCHEN_CASES
            || condition >= KITCHEN_CASES
            || cases.len() != 1
            || cases[0]["case_index"] != condition
            || cases[0]["audit"]["status"] != "passed"
            || cases[0]["audit"]["domain_calls"] != 9
            || !completed.insert(condition)
        {
            return Err("kitchen evidence has a missing, failed or duplicate condition".into());
        }
    }
    if completed != (0..KITCHEN_CASES).collect() {
        return Err("kitchen evidence omits part of the declared input domain".into());
    }
    Ok(())
}

// Independent review, test and initial merge orders for the two developers.
pub const FEATURE_DELIVERY_INITIAL_ORDERS: [(bool, bool, bool); 8] = [
    (false, false, false),
    (false, true, false),
    (true, false, true),
    (true, true, true),
    (false, false, true),
    (false, true, true),
    (true, false, false),
    (true, true, false),
];
type FailureSchedule = &'static [(&'static str, u32)];
pub const FEATURE_DELIVERY_RECOVERY_CASES: [(u8, bool, bool, FailureSchedule); 3] = [
    (1, false, true, &[("code_review", 3)]),
    (2, true, false, &[("code_review", 4), ("code_review", 5)]),
    // Both initial features are rejected before either is approved.
    (2, false, true, &[]),
];

fn boundary_assignments(family: &str, filter: &str) -> Result<Vec<(String, Option<usize>)>> {
    if family == "cicd" {
        let assignments = [
            "adversarial_depth_0",
            "ordinary_depth_1",
            "normal_delay_depth_2",
            "real_wasi_cicd_early_exit_has_no_domain_effect",
        ]
        .map(|name| format!("cicd_boundary::{name}"))
        .into_iter()
        .filter(|name| name.contains(filter))
        .map(|name| (name, None))
        .collect::<Vec<_>>();
        if assignments.is_empty() {
            return Err("unknown CI/CD finite assignment filter".into());
        }
        return Ok(assignments);
    }
    if family == "coordination" {
        let assignments = [
            "backend_first",
            "frontend_first",
            "one_amendment",
            "extended_four_amendments",
            "extended_normal_delay",
        ]
        .map(|name| format!("coordination_boundary::{name}"))
        .into_iter()
        .filter(|name| name.contains(filter))
        .map(|name| (name, None))
        .collect::<Vec<_>>();
        if assignments.is_empty() {
            return Err("unknown coordination finite assignment filter".into());
        }
        return Ok(assignments);
    }
    if family == "philosophers" {
        let assignments = [
            "three_philosophers_all_meal_orders",
            "five_philosophers_any_first_seat",
            "seven_philosophers_any_first_seat",
            "three_philosophers_normal",
            "five_philosophers_normal",
            "seven_philosophers_normal",
            "three_philosophers_adversarial",
            "five_philosophers_adversarial",
            "seven_philosophers_adversarial",
        ]
        .map(|name| format!("philosophers_boundary::{name}"))
        .into_iter()
        .filter(|name| name.contains(filter))
        .map(|name| (name, None))
        .collect::<Vec<_>>();
        if assignments.is_empty() {
            return Err("unknown dining finite assignment filter".into());
        }
        return Ok(assignments);
    }
    if family == "kitchen" {
        let name = "kitchen_boundary::all_equipment_and_recipe_input_orders_use_actual_wasm";
        if !name.contains(filter) {
            return Err("unknown kitchen finite assignment filter".into());
        }
        return Ok((0..KITCHEN_CASES)
            .map(|index| (name.to_owned(), Some(index)))
            .collect());
    }
    if family == "regulatory_discovery" {
        let name = "regulatory_discovery_boundary::actual_wasm_reconsideration";
        if !name.contains(filter) {
            return Err("unknown regulatory discovery finite assignment filter".into());
        }
        return Ok((0..REGULATORY_DISCOVERY_CASES)
            .map(|index| (name.to_owned(), Some(index)))
            .collect());
    }
    if family == "multi_round_discovery" {
        let name = "multi_round_discovery_boundary::actual_wasm_runs_multi_round_study";
        if !name.contains(filter) {
            return Err("unknown multi-round discovery finite assignment filter".into());
        }
        return Ok((0..MULTI_ROUND_DISCOVERY_CASES)
            .map(|index| (name.to_owned(), Some(index)))
            .collect());
    }
    if family == "research_paper" {
        let name = "research_paper_boundary::actual_wasm_publishes_three_independent_chapters";
        if !name.contains(filter) {
            return Err("unknown research paper finite assignment filter".into());
        }
        return Ok((0..RESEARCH_PAPER_CASES)
            .map(|index| (name.to_owned(), Some(index)))
            .collect());
    }
    if family == "survey_paper" {
        let name = "survey_paper_boundary::seven_wasm_guests_publish_the_current_survey";
        if !name.contains(filter) {
            return Err("unknown survey paper finite assignment filter".into());
        }
        return Ok(SURVEY_PAPER_DEPTHS
            .into_iter()
            .map(|depth| (name.to_owned(), Some(depth)))
            .collect());
    }
    if family != "feature_delivery" {
        return Ok(vec![(filter.to_owned(), None)]);
    }
    let mut assignments = Vec::new();
    for (name, conditions) in [
        (
            "feature_delivery_boundary::actual_wasi_initial_orders_and_integrated_ci",
            FEATURE_DELIVERY_INITIAL_ORDERS.len() + 2,
        ),
        (
            "feature_delivery_boundary::actual_wasi_review_rejection_and_ci_repair",
            FEATURE_DELIVERY_RECOVERY_CASES.len(),
        ),
    ] {
        if name.contains(filter) {
            assignments.extend((0..conditions).map(|index| (name.to_owned(), Some(index))));
        }
    }
    if assignments.is_empty() {
        return Err("no declared feature-delivery conditions match the filter".into());
    }
    Ok(assignments)
}

fn unchanged(root: &Path, before: &BTreeMap<PathBuf, String>) -> Result<()> {
    if &sources::hashes(&sources::inputs(root)?)? != before {
        return Err(
            "qualification source changed; results are not a fixed-source qualification".into(),
        );
    }
    Ok(())
}

fn execute(command: &mut Command, output: &Path, name: &str) -> Result<String> {
    let compiler_environment: BTreeMap<_, _> = [
        "CARGO_INCREMENTAL",
        "CARGO_BUILD_JOBS",
        "CARGO_BUILD_TARGET",
        "CARGO_PROFILE_DEV_DEBUG",
        "CARGO_PROFILE_TEST_DEBUG",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "PYTHONHASHSEED",
        "CHOREOFIX_FEATURE_DELIVERY_CASE",
        "CHOREOFIX_REFACTOR_CASE",
        "CHOREOFIX_CODEBASE_CASES",
        "CHOREOFIX_LAB_CASE",
        "CHOREOFIX_LAB_QUALITY_CASE",
        "CHOREOFIX_KITCHEN_CASE",
        "CHOREOFIX_REGULATORY_DISCOVERY_CASE",
        "CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE",
    ]
    .into_iter()
    .map(|key| {
        let value = command
            .get_envs()
            .find(|(name, _)| *name == std::ffi::OsStr::new(key))
            .map_or_else(
                || std::env::var_os(key),
                |(_, value)| value.map(std::ffi::OsStr::to_os_string),
            );
        (key, value.map(|value| value.to_string_lossy().into_owned()))
    })
    .collect();
    save(
        &output.join(format!("{name}-command.json")),
        &json!({
            "program":command.get_program().to_string_lossy(),
            "arguments":command.get_args().map(|a| a.to_string_lossy()).collect::<Vec<_>>(),
            "compiler_environment":compiler_environment
        }),
    )?;
    let file = fs::File::create(output.join(format!("{name}.log")))?;
    let status = command
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file))
        .status()?;
    if !status.success() {
        return Err(format!(
            "{name} failed ({status}); inspect {}",
            output.join(format!("{name}.log")).display()
        )
        .into());
    }
    Ok(fs::read_to_string(output.join(format!("{name}.log")))?)
}

fn completed_tests(log: &str) -> Result<u64> {
    let mut passed = 0;
    for line in log.lines() {
        if line.starts_with("test result:") && !line.starts_with("test result: ok. ") {
            return Err("qualification reported an unsuccessful test suite".into());
        }
        if let Some(result) = line.strip_prefix("test result: ok. ") {
            let (count, rest) = result
                .split_once(" passed; ")
                .ok_or("malformed test result")?;
            if !rest.starts_with("0 failed;") {
                return Err("qualification reported failed tests".into());
            }
            passed += count.parse::<u64>()?;
        }
    }
    if passed == 0 {
        return Err(
            "no WASI boundary test ran; a missing test filter is not a qualification".into(),
        );
    }
    Ok(passed)
}

// The build and its receipt are shared by mechanism qualification and live
// trials. Refreshing experiment-source evidence never requires rerunning an
// already measured guest merely to obtain a build receipt.
fn build(
    output: &Path,
    families: &[&str],
) -> Result<(PathBuf, BTreeMap<PathBuf, String>, PathBuf)> {
    let root = std::env::current_dir()?.canonicalize()?;
    let output = root.join(output);
    fs::create_dir_all(output.parent().ok_or("qualification output parent")?)?;
    fs::create_dir(&output)?;
    let output = output.canonicalize()?;
    let inputs = sources::inputs(&root)?;
    let before = sources::hashes(&inputs)?;
    let provenance = sources::git_provenance(&root, &inputs)?;
    save(&output.join("source-before.json"), &json!(before))?;
    for (name, path) in inputs {
        let saved = output.join(name);
        fs::create_dir_all(saved.parent().ok_or("source parent")?)?;
        fs::copy(path, saved)?;
    }
    fs::copy(std::env::current_exe()?, output.join("qualifier"))?;
    let app = std::env::var_os("CHOREOFIX_APP_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("target"));
    execute(
        Command::new("sh")
            .arg("tools/build_shell.sh")
            .args(families.iter().flat_map(|family| {
                [
                    "--bin".to_owned(),
                    format!("{}-wasi", family.replace('_', "-")),
                ]
            }))
            .env("CARGO_TARGET_DIR", &app)
            .env("CARGO_BUILD_JOBS", "1"),
        &output,
        "build",
    )?;
    unchanged(&root, &before)?;
    let manifest = before
        .iter()
        .map(|(name, hash)| format!("{hash}  {}\n", name.display()))
        .collect::<String>();
    fs::write(output.join("runtime-source.sha256"), manifest)?;
    for family in families {
        unchanged(&root, &before)?;
        let binary = app
            .join("debug")
            .join(format!("{}-wasi", family.replace('_', "-")))
            .canonicalize()?;
        let receipt = output.join(format!("{family}-build-receipt.json"));
        save(
            &receipt,
            &json!({
                "binary_sha256":digest(&binary)?, "source_manifest":"runtime-source.sha256",
                "source_manifest_sha256":digest(&output.join("runtime-source.sha256"))?,
                "source_git":provenance,
                "source_unchanged":true, "scored_benchmark":false
            }),
        )?;
    }
    unchanged(&root, &before)?;
    save(&output.join("source-after.json"), &json!(before))?;
    Ok((output, before, app))
}

/// Build current-source artifacts without claiming that qualification tests ran.
pub fn build_only(output: &Path, selection: &str) -> Result<Value> {
    // Building an implemented Cargo target is distinct from certifying the
    // finite WASI domains declared in FAMILIES.
    let metadata = Command::new("cargo")
        .args([
            "+stable",
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()?;
    if !metadata.status.success() {
        return Err("cannot read actual application build targets".into());
    }
    let metadata: Value = serde_json::from_slice(&metadata.stdout)?;
    let binaries = metadata["packages"]
        .as_array()
        .ok_or("application packages")?
        .iter()
        .filter(|p| p["name"] == "choreofix")
        .flat_map(|p| p["targets"].as_array().into_iter().flatten())
        .filter(|t| t["kind"] == json!(["bin"]))
        .filter_map(|t| t["name"].as_str())
        .collect::<BTreeSet<_>>();
    let mut used = BTreeSet::new();
    let families = selection
        .split(',')
        .map(|family| {
            if !binaries.contains(format!("{}-wasi", family.replace('_', "-")).as_str())
                || !used.insert(family)
            {
                return Err(
                    "build families must be actual Cargo targets, nonempty and unique".into(),
                );
            }
            Ok(family)
        })
        .collect::<Result<Vec<_>>>()?;
    let (output, _, _) = build(output, &families)?;
    let result = json!({
        "status":"built_not_qualified", "families":families,
        "source_unchanged":true, "qualification_tests_run":false,
        "model_calls":0, "scored_runs":0,
        "qualifier_sha256":digest(&output.join("qualifier"))?
    });
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

/// Build the real guest and run each selected family's declared WASI boundary.
pub fn run(output: &Path, filter: Option<&str>) -> Result<Value> {
    if std::env::var_os("CHOREOFIX_LAB_CASE").is_some()
        || std::env::var_os("CHOREOFIX_LAB_QUALITY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_REFACTOR_CASE").is_some()
        || std::env::var_os("CHOREOFIX_CODEBASE_CASES").is_some()
        || std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_KITCHEN_CASE").is_some()
        || std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_RESEARCH_PAPER_CASE").is_some()
    {
        return Err(
            "WASI qualification assigns every declared condition; use the direct boundary test for a diagnostic condition".into(),
        );
    }
    let selected = if let Some(filter) = filter {
        let mut used = BTreeSet::new();
        filter
            .split(',')
            .map(|filter| {
                let family = FAMILIES
                    .iter()
                    .copied()
                    .find(|family| filter.starts_with(&format!("{family}_boundary::")))
                    .ok_or("each diagnostic filter must name a declared WASI boundary")?;
                if !used.insert(family) {
                    return Err("a diagnostic family cannot be assigned twice".into());
                }
                Ok((family, filter.to_owned()))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        FAMILIES
            .iter()
            .map(|family| (*family, format!("{family}_boundary::")))
            .collect()
    };
    let root = std::env::current_dir()?.canonicalize()?;
    let families = selected
        .iter()
        .map(|(family, _)| *family)
        .collect::<Vec<_>>();
    let (output, before, app) = build(output, &families)?;
    let runner = std::env::var_os("CHOREOFIX_RUNNER_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("tools/runner/target"));
    let mut tests = BTreeMap::new();
    for (family, test_filter) in selected {
        unchanged(&root, &before)?;
        let binary = app
            .join("debug")
            .join(format!("{}-wasi", family.replace('_', "-")))
            .canonicalize()?;
        let receipt = output.join(format!("{family}-build-receipt.json"));
        let mut completed = 0;
        for (assignment, (test_filter, condition)) in boundary_assignments(family, &test_filter)?
            .into_iter()
            .enumerate()
        {
            let mut command = Command::new("cargo");
            command
                .args([
                    "+stable",
                    "test",
                    "--locked",
                    "--manifest-path",
                    "tools/runner/Cargo.toml",
                    &test_filter,
                    "--",
                    "--ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("CARGO_TARGET_DIR", &runner)
                .env("CARGO_BUILD_JOBS", "1")
                .env("PYTHONHASHSEED", "0")
                .env("CHOREOFIX_AUDITOR", output.join("qualifier"))
                .env("CHOREOFIX_EVIDENCE_ROOT", output.join("evidence"))
                .env(
                    format!("CHOREOFIX_{}_WASI_BINARY", family.to_uppercase()),
                    &binary,
                )
                .env(
                    format!("CHOREOFIX_{}_BUILD_RECEIPT", family.to_uppercase()),
                    &receipt,
                );
            // Libtest filters are substring matches: build_boundary also
            // matches cascading_build_boundary. Keep each family's assignment
            // disjoint, including when its peers ran in an earlier group.
            command.args(
                FAMILIES
                    .iter()
                    .filter(|other| **other != family && other.ends_with(family))
                    .flat_map(|other| ["--skip".to_owned(), format!("{other}_boundary::")]),
            );
            if let Some(condition) = condition {
                command.env(
                    match family {
                        "kitchen" => "CHOREOFIX_KITCHEN_CASE",
                        "regulatory_discovery" => "CHOREOFIX_REGULATORY_CASE",
                        "multi_round_discovery" => "CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE",
                        "research_paper" => "CHOREOFIX_RESEARCH_PAPER_CASE",
                        "survey_paper" => "CHOREOFIX_SURVEY_PAPER_DEPTH",
                        _ => "CHOREOFIX_FEATURE_DELIVERY_CASE",
                    },
                    condition.to_string(),
                );
            }
            let log = execute(&mut command, &output, &format!("{family}-{assignment}"))?;
            let passed = completed_tests(&log)?;
            if condition.is_some() && passed != 1 {
                return Err("a condition assignment must execute exactly one boundary test".into());
            }
            completed += passed;
        }
        if filter.is_none() && family == "report" && completed != 5 {
            return Err("report qualification requires ordinary reviews, repeated rejections, invalid requests, independent author preparation and early section review".into());
        }
        if filter.is_none() && family == "manufacturing" && completed != 2 {
            return Err("manufacturing qualification must execute both the total-order/recovery and contended-choice groups".into());
        }
        if family == "cicd_pipeline" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/cicd-pipeline-wasi/boundary.json"),
            )?)?;
            if completed != 1
                || report["task"] != "16M"
                || report["status"] != "passed"
                || report["runtime_binary_sha256"] != digest(&binary)?
            {
                return Err("16M qualification requires its audited actual-Wasm boundary".into());
            }
        }
        if family == "large_document" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/large-document-wasi/boundary.json"),
            )?)?;
            if completed != 1
                || report["task"] != "7H"
                || report["status"] != "passed"
                || report["runtime_binary_sha256"] != digest(&binary)?
            {
                return Err("7H qualification requires its audited actual-Wasm boundary".into());
            }
        }
        if family == "full_stack_api" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/full-stack-api-wasi/boundary.json"),
            )?)?;
            if completed != 1
                || report["task"] != "8M"
                || report["status"] != "passed"
                || report["simulator_complete"] != true
                || report["violations"] != 0
                || report["runtime_binary_sha256"] != digest(&binary)?
            {
                return Err(
                    "8M qualification requires the audited actual-Wasm recovery case".into(),
                );
            }
        }
        if family == "regulatory_discovery" {
            if completed != REGULATORY_DISCOVERY_CASES as u64 {
                return Err("14M did not execute every finite condition".into());
            }
            let runtime = digest(&binary)?;
            for index in 0..REGULATORY_DISCOVERY_CASES {
                let report: Value = serde_json::from_slice(&fs::read(output.join(format!(
                    "evidence/regulatory-discovery-wasi-{index}/audit.json"
                )))?)?;
                if report["task"] != "14M"
                    || report["status"] != "passed"
                    || report["dynamic_protocol"]["status"] != "passed"
                    || report["violations"] != 0
                    || report["runtime_binary_sha256"] != runtime
                {
                    return Err("14M qualification requires all audited runs on one runtime".into());
                }
            }
        }
        if family == "multi_round_discovery" {
            if completed != MULTI_ROUND_DISCOVERY_CASES as u64 {
                return Err("14H did not execute every finite condition".into());
            }
            let runtime = digest(&binary)?;
            for index in 0..MULTI_ROUND_DISCOVERY_CASES {
                let report: Value = serde_json::from_slice(&fs::read(output.join(format!(
                    "evidence/multi-round-discovery-wasi-{index}/audit.json"
                )))?)?;
                if report["task"] != "14H"
                    || report["status"] != "passed"
                    || report["dynamic_protocol"]["status"] != "passed"
                    || report["violations"] != 0
                    || report["runtime_binary_sha256"] != runtime
                {
                    return Err("14H qualification requires all audited runs on one runtime".into());
                }
            }
        }
        if family == "research_paper" {
            if completed != RESEARCH_PAPER_CASES as u64 {
                return Err("3M did not execute every finite condition".into());
            }
            let runtime = digest(&binary)?;
            for (index, effect_count) in [16, 24].into_iter().enumerate() {
                let report: Value = serde_json::from_slice(&fs::read(output.join(format!(
                    "evidence/research-paper-wasi-{index}/effect-audit.json"
                )))?)?;
                if report["task"] != "3M"
                    || report["status"] != "passed"
                    || report["dynamic_protocol"]["status"] != "passed"
                    || report["violations"] != 0
                    || report["domain_calls"] != effect_count
                    || report["runtime_binary_sha256"] != runtime
                {
                    return Err("3M qualification requires both audited runs on one runtime".into());
                }
            }
        }
        if family == "survey_paper" {
            if completed != SURVEY_PAPER_DEPTHS.len() as u64 {
                return Err("3H did not execute every finite condition".into());
            }
            let runtime = digest(&binary)?;
            for depth in SURVEY_PAPER_DEPTHS {
                let report: Value = serde_json::from_slice(&fs::read(output.join(format!(
                    "evidence/survey-paper-wasi-{depth}/effect-audit.json"
                )))?)?;
                if report["task"] != "3H"
                    || report["status"] != "passed"
                    || report["dynamic_protocol"]["status"] != "passed"
                    || report["simulator_complete"] != true
                    || report["violations"] != 0
                    || report["domain_calls"] != 26 + 2 * depth
                    || report["runtime_binary_sha256"] != runtime
                {
                    return Err(
                        "3H qualification requires both audited conditions on one runtime".into(),
                    );
                }
            }
        }
        if family == "drug_discovery" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/drug-discovery-wasi/mechanism.json"),
            )?)?;
            let cases = report["cases"].as_array().ok_or("14E case reports")?;
            let runtime = digest(&binary)?;
            if completed != 1
                || report["task"] != "14E"
                || report["status"] != "passed"
                || cases.len() != 2
                || cases[0]["case"] != "reference-first-no-go"
                || cases[1]["case"] != "baseline-first-go"
                || cases[0]["audit"]["decision"] != false
                || cases[1]["audit"]["decision"] != true
                || cases.iter().any(|case| {
                    let audit = &case["audit"];
                    audit["status"] != "passed"
                        || audit["violations"] != 0
                        || audit["runtime_binary_sha256"] != runtime
                })
            {
                return Err("14E qualification requires both decision cases on one runtime".into());
            }
        }
        if family == "lab_cascading" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/lab-cascading-wasi/mechanism.json"),
            )?)?;
            let cases = report["cases"].as_array().ok_or("13H case reports")?;
            let names = cases
                .iter()
                .filter_map(|case| case["case"].as_str())
                .collect::<BTreeSet<_>>();
            let runtime = digest(&binary)?;
            if completed != 1
                || report["task"] != "13H"
                || report["status"] != "passed"
                || report["declared_cases"] != LAB_CASCADING_CASES
                || names.len() != LAB_CASCADING_CASES
                || cases.len() != LAB_CASCADING_CASES
                || cases
                    .iter()
                    .any(|case| case["violations"] != 0 || case["runtime_binary_sha256"] != runtime)
            {
                return Err("13H qualification requires every case on the selected runtime".into());
            }
        }
        if family == "lab_quality" && test_filter == "lab_quality_boundary::" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/lab-quality-mechanism.json"),
            )?)?;
            let cases = report["cases"].as_array().ok_or("QC condition reports")?;
            let ids = cases
                .iter()
                .filter_map(|case| case["case_index"].as_u64())
                .collect::<BTreeSet<_>>();
            if completed != 1
                || report["complete_finite_domain"] != true
                || report["application_effect_correspondence_certified"] != true
                || report["declared_cases"] != LAB_QUALITY_CASES
                || cases.len() != LAB_QUALITY_CASES
                || ids != (0..LAB_QUALITY_CASES as u64).collect()
                || cases.iter().any(|case| {
                    let depth = match case["case_index"].as_u64() {
                        Some(0..=5) => case["case_index"].as_u64().unwrap() / 2,
                        Some(6) => 2,
                        Some(7) => 0,
                        _ => return true,
                    };
                    case["status"] != "passed"
                        || case["depth"] != depth
                        || case["domain_calls"] != 11 + 3 * depth
                        || case["application_effect_correspondence_certified"] != true
                })
                || [0, 4].into_iter().any(|index| {
                    cases
                        .iter()
                        .find(|case| case["case_index"] == index)
                        .and_then(|case| {
                            case["counterexamples"]["record_counterexamples"].as_array()
                        })
                        .is_none_or(|items| {
                            items.len() != 9 || items.iter().any(|item| item["rejected"] != true)
                        })
                })
            {
                return Err(
                    "QC qualification omits a required real-Wasm condition or effect audit".into(),
                );
            }
        }
        if family == "lab" && test_filter == "lab_boundary::" {
            let report: Value =
                serde_json::from_slice(&fs::read(output.join("evidence/lab-mechanism.json"))?)?;
            let cases = report["cases"]
                .as_array()
                .ok_or("laboratory condition reports")?;
            let ids = cases
                .iter()
                .filter_map(|case| case["case_index"].as_u64())
                .collect::<BTreeSet<_>>();
            if completed != 1
                || report["complete_finite_domain"] != true
                || report["declared_cases"] != LAB_CASES
                || report["source_conformance_certified"] != true
                || cases.len() != LAB_CASES
                || ids != (0..LAB_CASES as u64).collect()
                || cases
                    .iter()
                    .any(|case| case["status"] != "passed" || case["domain_calls"] != 8)
                || cases
                    .iter()
                    .find(|case| case["case_index"] == 0)
                    .and_then(|case| case["counterexamples"]["record_counterexamples"].as_array())
                    .is_none_or(|items| {
                        items.len() != 9 || items.iter().any(|item| item["rejected"] != true)
                    })
            {
                return Err("laboratory qualification omitted a required real-Wasm case".into());
            }
        }
        if family == "refactor" && test_filter == "refactor_boundary::" {
            let report: Value = serde_json::from_slice(&fs::read(
                output.join("evidence/refactor-mechanism.json"),
            )?)?;
            let cases = report["cases"]
                .as_array()
                .ok_or("refactor condition reports")?;
            let ids = cases
                .iter()
                .filter_map(|case| case["case_index"].as_u64())
                .collect::<BTreeSet<_>>();
            if completed != 1
                || report["complete_finite_domain"] != true
                || report["declared_cases"] != REFACTOR_CASES
                || cases.len() != REFACTOR_CASES
                || ids != (0..REFACTOR_CASES as u64).collect()
                || cases
                    .iter()
                    .any(|case| case["status"] != "passed" || case["domain_calls"] != 12)
                || cases
                    .iter()
                    .find(|case| case["case_index"] == 0)
                    .and_then(|case| case["counterexamples"]["record_counterexamples"].as_array())
                    .is_none_or(|items| {
                        items.len() != 9 || items.iter().any(|item| item["rejected"] != true)
                    })
            {
                return Err("refactor qualification omits a declared real-Wasm condition".into());
            }
        }
        if family == "staged_integration"
            && test_filter == "staged_integration_boundary::"
            && completed != 4
        {
            return Err(
                "staged integration must execute concurrent, paced, reverse-merge and failed-result recovery trials".into(),
            );
        }
        if family == "kitchen" && completed != KITCHEN_CASES as u64 {
            return Err("kitchen qualification omitted a declared condition".into());
        }
        if family == "philosophers" {
            if completed != 9 {
                return Err("dining qualification omitted a declared condition".into());
            }
            let mut reports = Vec::new();
            for entry in fs::read_dir(output.join("evidence"))? {
                let entry = entry?;
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("philosophers-rust-boundary-")
                {
                    reports.extend(serde_json::from_slice::<Vec<Value>>(&fs::read(
                        entry.path().join("summary.json"),
                    )?)?);
                }
            }
            let runtime_hash = digest(&binary)?;
            dining_reports(&reports, &runtime_hash)?;
            for (task, group, seats, schedules) in [
                ("9E", "three-diner-orders", 3, DINING_9E_ORDERS),
                ("9M", "five-first-seats", 5, DINING_9M_ORDERS),
                ("9H", "seven-first-seats", 7, DINING_9H_ORDERS),
            ] {
                let directory = output.join("evidence").join(group);
                for &(name, word) in schedules {
                    let case = directory.join(name);
                    let audit: Value = serde_json::from_slice(&fs::read(case.join("audit.json"))?)?;
                    let journal = case.join("events.jsonl");
                    let records = crate::journal::records(&journal)?;
                    let effects = records
                        .iter()
                        .filter(|row| row["direction"] == "domain_start")
                        .map(|row| {
                            (
                                row["message"]["agent"].as_str(),
                                row["message"]["tool"].as_str(),
                            )
                        });
                    let expected: Vec<_> = word
                        .chars()
                        .map(|action| {
                            let seat = action.to_ascii_uppercase();
                            let tool = if action.is_ascii_lowercase() {
                                "think"
                            } else {
                                "eat"
                            };
                            (format!("PHILOSOPHER_{seat}"), tool)
                        })
                        .collect();
                    if !effects.eq(expected
                        .iter()
                        .map(|(seat, tool)| (Some(seat.as_str()), Some(*tool))))
                        || audit["task"] != task
                        || audit["status"] != "passed"
                        || audit["runtime_binary_sha256"] != runtime_hash
                        || audit["source_events_sha256"] != digest(&journal)?
                        || audit["dynamic_protocol"]["status"] != "passed"
                        || audit["actual_resource_endpoint_returns"] != 2 * seats
                        || audit["simulator_complete"] != true
                        || audit["violations"] != 0
                    {
                        return Err(
                            format!("{task} case {name} lacks matching audited evidence").into(),
                        );
                    }
                }
            }
        }
        if family == "kitchen" {
            let mut reports = Vec::new();
            for entry in fs::read_dir(output.join("evidence"))? {
                let entry = entry?;
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("kitchen-wasi-")
                {
                    reports.push(serde_json::from_slice(&fs::read(
                        entry.path().join("report.json"),
                    )?)?);
                }
            }
            kitchen_reports(&reports)?;
        }
        tests.insert(family, completed);
        unchanged(&root, &before)?;
    }
    save(
        &output.join("source-after.json"),
        &json!(sources::hashes(&sources::inputs(&root)?)?),
    )?;
    let result = json!({
        "all_families_passed":filter.is_none(), "source_unchanged":true,
        "diagnostic_filter":filter,
        "test_functions_passed":tests, "model_calls":0, "scored_runs":0,
        "qualifier_sha256":digest(&output.join("qualifier"))?
    });
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kitchen_assignments_and_actual_reports_cover_every_condition_once() {
        let assigned = boundary_assignments("kitchen", "kitchen_boundary::").unwrap();
        assert_eq!(
            assigned
                .iter()
                .map(|(_, index)| index.unwrap())
                .collect::<Vec<_>>(),
            (0..KITCHEN_CASES).collect::<Vec<_>>()
        );
        assert!(boundary_assignments("kitchen", "kitchen_boundary::missing").is_err());
        let reports: Vec<_> = (0..KITCHEN_CASES)
            .map(|index| {
                json!({"task":"12E","status":"passed",
            "declared_conditions":KITCHEN_CASES,"assigned_condition":index,
            "cases":[{"case_index":index,"audit":{"status":"passed","domain_calls":9}}]})
            })
            .collect();
        kitchen_reports(&reports).unwrap();
        assert!(kitchen_reports(&reports[..KITCHEN_CASES - 1]).is_err());
        let mut changed = reports.clone();
        changed[12] = changed[0].clone();
        assert!(kitchen_reports(&changed).is_err());
        let mut changed = reports.clone();
        changed[12]["cases"][0]["case_index"] = json!(11);
        assert!(kitchen_reports(&changed).is_err());
        let mut changed = reports;
        changed[12]["cases"][0]["audit"]["domain_calls"] = json!(8);
        assert!(kitchen_reports(&changed).is_err());
    }

    #[test]
    fn isolated_feature_conditions_cover_the_complete_declared_matrix_once() {
        let assigned =
            boundary_assignments("feature_delivery", "feature_delivery_boundary::").unwrap();
        assert_eq!(assigned.len(), 13);
        assert_eq!(assigned.iter().collect::<BTreeSet<_>>().len(), 13);
        let initial: Vec<_> = assigned
            .iter()
            .filter(|(name, _)| name.ends_with("actual_wasi_initial_orders_and_integrated_ci"))
            .map(|(_, index)| index.unwrap())
            .collect();
        let recovery: Vec<_> = assigned
            .iter()
            .filter(|(name, _)| name.ends_with("actual_wasi_review_rejection_and_ci_repair"))
            .map(|(_, index)| index.unwrap())
            .collect();
        assert_eq!(initial, (0..10).collect::<Vec<_>>());
        assert_eq!(recovery, [0, 1, 2]);
        assert_eq!(
            FEATURE_DELIVERY_INITIAL_ORDERS
                .into_iter()
                .collect::<BTreeSet<_>>(),
            [false, true]
                .into_iter()
                .flat_map(
                    |review| [false, true].into_iter().flat_map(move |test| [false, true]
                        .into_iter()
                        .map(move |merge| (review, test, merge)))
                )
                .collect()
        );
    }

    #[test]
    fn an_unknown_feature_filter_cannot_publish_an_empty_assignment() {
        assert!(boundary_assignments("feature_delivery", "missing_boundary").is_err());
        assert!(
            boundary_assignments(
                "feature_delivery",
                "feature_delivery_boundary::actual_wasi_initial_orders_and_integrated_ci"
            )
            .unwrap()
            .iter()
            .all(|(name, _)| name.ends_with("actual_wasi_initial_orders_and_integrated_ci"))
        );
    }

    #[test]
    fn a_missing_filter_or_failed_test_cannot_qualify_a_family() {
        assert!(completed_tests("test result: ok. 0 passed; 0 failed; 2 ignored;").is_err());
        assert!(completed_tests("test result: FAILED. 1 passed; 1 failed;").is_err());
        assert!(
            completed_tests(
                "test result: ok. 2 passed; 0 failed;\ntest result: FAILED. 0 passed; 1 failed;"
            )
            .is_err()
        );
        assert_eq!(
            completed_tests("test result: ok. 2 passed; 0 failed; 0 ignored;").unwrap(),
            2
        );
    }
}
