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

const FAMILIES: [&str; 23] = [
    "multi_round_discovery",
    "kitchen",
    "manufacturing",
    "cascading_build",
    "build",
    "document",
    "feature_delivery",
    "critical_care",
    "multi_specialty",
    "medical",
    "lab_quality",
    "lab",
    "discovery",
    "regulatory_discovery",
    "refactor",
    "coordination",
    "report",
    "review",
    "codebase",
    "writing",
    "api",
    "philosophers",
    "cicd",
];

pub const MULTI_ROUND_DISCOVERY_CASES: usize = 12;

fn multi_round_discovery_reports(reports: &[Value]) -> Result<()> {
    let mut completed = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for report in reports {
        let condition = report["case"]
            .as_u64()
            .ok_or("missing scientific condition")?;
        if report["task"] != "14H"
            || report["status"] != "mechanism_case_passed"
            || report["declared_conditions"] != MULTI_ROUND_DISCOVERY_CASES
            || condition >= MULTI_ROUND_DISCOVERY_CASES as u64
            || report["source_unchanged"] != true
            || report["dynamic_protocol_qualified"] != true
            || report["separate_effect_audit_qualified"] != true
            || report["effect_receives_bound_to_checked_occurrences"] != true
            || report["scientific_effects"].as_u64().is_none_or(|n| n == 0)
            || report["frame_contracts"].as_u64().is_none_or(|n| n == 0)
            || !completed.insert(condition)
        {
            return Err("scientific evidence has a missing, failed or duplicate condition".into());
        }
        let identity = [
            "runtime_binary_sha256",
            "compiled_projection_sha256",
            "test_driver_sha256",
        ]
        .map(|key| {
            report[key]
                .as_str()
                .filter(|s| s.len() == 64)
                .ok_or("missing scientific artifact hash")
        })
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()?;
        artifacts.insert(identity);
    }
    if completed != (0..MULTI_ROUND_DISCOVERY_CASES as u64).collect() || artifacts.len() != 1 {
        return Err("scientific evidence omits conditions or combines different artifacts".into());
    }
    Ok(())
}

pub const KITCHEN_CASES: usize = KITCHEN_ORDERS.len() * 2 + 1;
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

pub const FEATURE_DELIVERY_INITIAL_ORDERS: [(bool, bool); 4] =
    [(false, false), (false, true), (true, false), (true, true)];
type FailureSchedule = &'static [(&'static str, u32)];
pub const FEATURE_DELIVERY_RECOVERY_CASES: [(u8, bool, bool, FailureSchedule); 2] = [
    (1, false, true, &[("code_review", 3)]),
    (2, true, false, &[("code_review", 4), ("code_review", 5)]),
];

fn boundary_assignments(family: &str, filter: &str) -> Result<Vec<(String, Option<usize>)>> {
    if family == "multi_round_discovery" {
        let name = "multi_round_discovery_boundary::seven_actual_wasm_guests_retest_review_exhaust_and_reconsider";
        if !name.contains(filter) {
            return Err("unknown scientific finite assignment filter".into());
        }
        return Ok((0..MULTI_ROUND_DISCOVERY_CASES)
            .map(|index| (name.to_owned(), Some(index)))
            .collect());
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
    if family != "feature_delivery" {
        return Ok(vec![(filter.to_owned(), None)]);
    }
    let mut assignments = Vec::new();
    for (name, conditions) in [
        (
            "feature_delivery_boundary::actual_wasi_initial_orders_and_integrated_ci",
            FEATURE_DELIVERY_INITIAL_ORDERS.len(),
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
    if std::env::var_os("CHOREOFIX_FEATURE_DELIVERY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_KITCHEN_CASE").is_some()
        || std::env::var_os("CHOREOFIX_REGULATORY_DISCOVERY_CASE").is_some()
        || std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE").is_some()
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
            if let Some(condition) = condition {
                command.env(
                    match family {
                        "kitchen" => "CHOREOFIX_KITCHEN_CASE",
                        "multi_round_discovery" => "CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE",
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
        if filter.is_none() && family == "report" && completed != 3 {
            return Err("report qualification must execute the ordinary, extended-rejection and adversarial groups".into());
        }
        if filter.is_none() && family == "manufacturing" && completed != 2 {
            return Err("manufacturing qualification must execute both the total-order/recovery and contended-choice groups".into());
        }
        if family == "kitchen" && completed != KITCHEN_CASES as u64 {
            return Err("kitchen qualification omitted a declared condition".into());
        }
        if family == "multi_round_discovery" {
            if completed != MULTI_ROUND_DISCOVERY_CASES as u64 {
                return Err("scientific qualification omitted a declared condition".into());
            }
            let reports = fs::read_dir(output.join("evidence"))?
                .map(|entry| {
                    let entry = entry?;
                    if !entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("multi-round-discovery-case")
                    {
                        return Ok(None);
                    }
                    Ok(Some(serde_json::from_slice(&fs::read(
                        entry.path().join("qualification.json"),
                    )?)?))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            multi_round_discovery_reports(&reports)?;
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
        "dynamic_eligibility_replay_complete":false, "qualifier_sha256":digest(&output.join("qualifier"))?
    });
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scientific_assignments_require_every_condition_with_one_fixed_artifact() {
        let assigned =
            boundary_assignments("multi_round_discovery", "multi_round_discovery_boundary::")
                .unwrap();
        assert_eq!(
            assigned.iter().map(|(_, n)| n.unwrap()).collect::<Vec<_>>(),
            (0..12).collect::<Vec<_>>()
        );
        assert!(
            boundary_assignments(
                "multi_round_discovery",
                "multi_round_discovery_boundary::missing"
            )
            .is_err()
        );
        let reports = (0..12).map(|case| json!({
            "task":"14H","case":case,"status":"mechanism_case_passed","declared_conditions":12,
            "source_unchanged":true,"dynamic_protocol_qualified":true,"separate_effect_audit_qualified":true,
            "effect_receives_bound_to_checked_occurrences":true,"scientific_effects":13,"frame_contracts":104,
            "runtime_binary_sha256":"a".repeat(64),"compiled_projection_sha256":"b".repeat(64),
            "test_driver_sha256":"c".repeat(64)
        })).collect::<Vec<_>>();
        multi_round_discovery_reports(&reports).unwrap();
        assert!(multi_round_discovery_reports(&reports[..11]).is_err());
        let mut changed = reports.clone();
        changed[11] = changed[0].clone();
        assert!(multi_round_discovery_reports(&changed).is_err());
        for (key, value) in [
            ("case", json!(12)),
            ("runtime_binary_sha256", json!("d".repeat(64))),
            ("test_driver_sha256", Value::Null),
            ("scientific_effects", json!(0)),
            ("effect_receives_bound_to_checked_occurrences", json!(false)),
            ("status", json!("failed")),
        ] {
            let mut changed = reports.clone();
            changed[11][key] = value;
            assert!(
                multi_round_discovery_reports(&changed).is_err(),
                "accepted invalid {key}"
            );
        }
    }

    #[test]
    fn kitchen_assignments_and_actual_reports_cover_every_condition_once() {
        let assigned = boundary_assignments("kitchen", "kitchen_boundary::").unwrap();
        assert_eq!(
            assigned
                .iter()
                .map(|(_, index)| index.unwrap())
                .collect::<Vec<_>>(),
            (0..13).collect::<Vec<_>>()
        );
        assert!(boundary_assignments("kitchen", "kitchen_boundary::missing").is_err());
        let reports: Vec<_> = (0..13)
            .map(|index| {
                json!({"task":"12E","status":"passed",
            "declared_conditions":13,"assigned_condition":index,
            "cases":[{"case_index":index,"audit":{"status":"passed","domain_calls":9}}]})
            })
            .collect();
        kitchen_reports(&reports).unwrap();
        assert!(kitchen_reports(&reports[..12]).is_err());
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
        assert_eq!(assigned.len(), 6);
        assert_eq!(assigned.iter().collect::<BTreeSet<_>>().len(), 6);
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
        assert_eq!(initial, [0, 1, 2, 3]);
        assert_eq!(recovery, [0, 1]);
        assert_eq!(
            FEATURE_DELIVERY_INITIAL_ORDERS
                .into_iter()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([(false, false), (false, true), (true, false), (true, true)])
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
