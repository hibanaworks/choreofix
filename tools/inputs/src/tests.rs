use super::*;
use crate::restore::{Scratch, extract};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixture() -> Result<Scratch> {
    let temp = Scratch::new()?;
    fs::create_dir(temp.0.join("benchmark"))?;
    for name in [
        "upstream-manifest.json",
        "simulator-manifest.json",
        "runtime-manifest.json",
    ] {
        fs::copy(
            root().join("benchmark").join(name),
            temp.0.join("benchmark").join(name),
        )?;
    }
    fs::copy(
        root().join("experiment.json"),
        temp.0.join("experiment.json"),
    )?;
    fs::copy(
        root().join("benchmark/experiment-contract.json"),
        temp.0.join("benchmark/experiment-contract.json"),
    )?;
    let benchmark = read(&root().join("benchmark/simulator-manifest.json"))?;
    let runtime = read(&root().join("benchmark/runtime-manifest.json"))?;
    for mapping in benchmark["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| &v["files"])
        .chain([&benchmark["simulator_files"]])
    {
        for path in mapping.as_object().unwrap().keys() {
            let destination = temp.0.join(path);
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::copy(root().join(path), destination)?;
        }
    }
    for source in ["tracefix", "tracefix-paper"] {
        for path in runtime[source]["files"].as_object().unwrap().keys() {
            let destination = temp.0.join("vendor").join(source).join(path);
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::copy(root().join("vendor").join(source).join(path), destination)?;
        }
    }
    Ok(temp)
}

#[test]
fn every_original_input_and_both_runtimes_match_the_recorded_manifests() -> Result<()> {
    let result = verify(&root())?;
    assert_eq!(result["benchmark"]["tasks"], 48);
    assert_eq!(result["benchmark"]["checklist_items"], 825);
    assert_eq!(
        result["runtimes"],
        json!({"tracefix":182,"tracefix-paper":127})
    );
    assert_eq!(result["baselines"]["conditions"], 5);
    Ok(())
}

#[test]
fn baseline_selection_rejects_missing_sources_entrypoints_and_changed_model_conditions()
-> Result<()> {
    let temp = fixture()?;
    let path = temp.0.join("benchmark/experiment-contract.json");
    let contract = read(&path)?;
    verify_baselines(&temp.0)?;
    for (field, value) in [
        ("source", json!("unknown-source")),
        ("revision", json!(LATEST)),
        ("module", json!("runtime_base_2.missing")),
        ("module", json!("../runtime_B")),
    ] {
        let mut changed = contract.clone();
        changed["baseline_runtimes"]["prompt_only"][field] = value;
        fs::write(&path, serde_json::to_vec(&changed)?)?;
        assert!(verify_baselines(&temp.0).is_err());
    }
    let mut changed = contract.clone();
    changed["baseline_runtimes"]
        .as_object_mut()
        .unwrap()
        .remove("chat_only");
    fs::write(&path, serde_json::to_vec(&changed)?)?;
    assert!(verify_baselines(&temp.0).is_err());
    for (field, value) in [
        ("model", json!("another-model")),
        ("reasoning_effort", json!("none")),
        ("required_endpoint", json!("chat_completions")),
        ("authentication", json!("api_key")),
        ("paid_api_calls", json!(true)),
    ] {
        let mut changed = contract.clone();
        changed["participant_api"][field] = value;
        fs::write(&path, serde_json::to_vec(&changed)?)?;
        assert!(verify_baselines(&temp.0).is_err());
    }
    fs::write(&path, serde_json::to_vec(&contract)?)?;
    verify_baselines(&temp.0)?;
    Ok(())
}

#[test]
fn changed_bytes_metadata_files_and_task_sets_are_rejected() -> Result<()> {
    let temp = fixture()?;
    verify(&temp.0)?;
    let input = temp.0.join("benchmark/descriptions/1E/description.md");
    let bytes = fs::read(&input)?;
    fs::write(&input, b"modified task")?;
    assert!(verify_benchmark(&temp.0).is_err());
    fs::write(input, bytes)?;
    let extra = temp.0.join("benchmark/environments/1E/new-policy.py");
    fs::write(&extra, b"different environment")?;
    assert!(verify_benchmark(&temp.0).is_err());
    fs::remove_file(extra)?;
    let extra = temp.0.join("benchmark/environments/17E");
    fs::create_dir(&extra)?;
    assert!(verify_benchmark(&temp.0).is_err());
    fs::remove_dir(extra)?;
    let extra = temp.0.join("benchmark/tools/new-tool.py");
    fs::write(&extra, b"new tool")?;
    assert!(verify_benchmark(&temp.0).is_err());
    fs::remove_file(extra)?;
    verify_benchmark(&temp.0)?;
    let path = temp.0.join("benchmark/simulator-manifest.json");
    let mut manifest = read(&path)?;
    manifest["tasks"][0]["agents"] = json!(["REPLACED_PARTICIPANT"]);
    fs::write(path, serde_json::to_vec(&manifest)?)?;
    assert!(verify_benchmark(&temp.0).is_err());
    Ok(())
}

#[test]
fn a_saved_world_keeps_its_declared_contract_and_checks_its_actual_bytes() -> Result<()> {
    let temp = fixture()?;
    let contract = "archived-world-fixture";
    let manifest = simulator_manifest(&temp.0, contract)?;
    let path = temp.0.join("benchmark/simulator-manifest.json");
    fs::write(&path, serde_json::to_vec(&manifest)?)?;
    let verified = verify_benchmark(&temp.0)?;
    assert_eq!(verified["simulator_contract"], contract);
    assert_eq!(verified["tasks"], 48);
    let source = temp.0.join("benchmark/tools/sim_document.py");
    let bytes = fs::read(&source)?;
    fs::write(&source, b"changed scientific world")?;
    assert!(verify_benchmark(&temp.0).is_err());
    fs::write(&source, bytes)?;
    verify_benchmark(&temp.0)?;
    for invalid in [Value::Null, json!(""), json!(9)] {
        let mut changed = manifest.clone();
        changed["simulator_contract"] = invalid;
        fs::write(&path, serde_json::to_vec(&changed)?)?;
        assert!(verify_benchmark(&temp.0).is_err());
    }
    Ok(())
}

#[test]
fn existing_sources_are_never_overwritten_or_silently_repaired() -> Result<()> {
    let temp = fixture()?;
    restore(&temp.0)?;
    let extra = temp.0.join("vendor/tracefix/tracefix/runtime/extra.py");
    fs::write(&extra, b"unexpected implementation")?;
    assert!(restore(&temp.0).is_err());
    assert_eq!(fs::read(&extra)?, b"unexpected implementation");
    fs::remove_file(extra)?;
    let license = temp.0.join("vendor/tracefix-paper/LICENSE");
    fs::write(&license, b"changed license")?;
    assert!(verify_runtimes(&temp.0).is_err());
    assert!(restore(&temp.0).is_err());
    assert_eq!(fs::read(license)?, b"changed license");
    Ok(())
}

#[test]
fn generator_cli_schema_and_installation_inputs_are_checked_without_repair() -> Result<()> {
    let temp = fixture()?;
    for (source, file) in [
        ("tracefix", "tracefix/cli/cli.py"),
        ("tracefix", "tracefix/pipeline/pipeline/schema.json"),
        ("tracefix", "scripts/download_tla2tools.sh"),
        ("tracefix", "pyproject.toml"),
        ("tracefix-paper", "v3_agent_test/tool_client.py"),
        ("tracefix-paper", "v3_agent_test/pipeline/schema.json"),
        ("tracefix-paper", "tla_verify_pluscal/cli.py"),
        ("tracefix-paper", "pyproject.toml"),
    ] {
        let path = temp.0.join("vendor").join(source).join(file);
        let original = fs::read(&path)?;
        fs::write(&path, b"changed compiler input")?;
        assert!(
            verify_runtimes(&temp.0).is_err(),
            "accepted changed {source}/{file}"
        );
        assert!(restore(&temp.0).is_err());
        assert_eq!(fs::read(&path)?, b"changed compiler input");
        fs::write(&path, &original)?;
        fs::remove_file(&path)?;
        assert!(
            verify_runtimes(&temp.0).is_err(),
            "accepted missing {source}/{file}"
        );
        fs::write(&path, original)?;
        verify_runtimes(&temp.0)?;
    }
    let extra = temp
        .0
        .join("vendor/tracefix-paper/v3_agent_test/pipeline/new-schema.json");
    fs::write(&extra, b"unexpected data dependency")?;
    assert!(verify_runtimes(&temp.0).is_err());
    fs::remove_file(extra)?;
    verify_runtimes(&temp.0)?;
    Ok(())
}

#[test]
#[cfg(unix)]
fn matching_bytes_do_not_allow_source_file_or_package_directory_symlinks() -> Result<()> {
    use std::os::unix::fs::symlink;
    let temp = fixture()?;
    let license = temp.0.join("vendor/tracefix/LICENSE");
    let original = fs::read(&license)?;
    let external = temp.0.join("license-copy");
    fs::write(&external, &original)?;
    fs::remove_file(&license)?;
    symlink(&external, &license)?;
    assert!(verify_runtimes(&temp.0).is_err());
    assert!(restore(&temp.0).is_err());
    assert!(fs::symlink_metadata(&license)?.file_type().is_symlink());
    fs::remove_file(&license)?;
    fs::write(license, original)?;

    let package = temp.0.join("vendor/tracefix/tracefix/cli");
    let external = temp.0.join("compiler-copy");
    fs::rename(&package, &external)?;
    symlink(&external, &package)?;
    assert!(verify_runtimes(&temp.0).is_err());
    assert!(restore(&temp.0).is_err());
    assert!(fs::symlink_metadata(&package)?.file_type().is_symlink());
    fs::remove_file(&package)?;
    fs::rename(external, package)?;
    verify_runtimes(&temp.0)?;
    Ok(())
}

#[test]
fn compiler_qualification_requires_both_exact_artifacts_and_never_repairs_them() -> Result<()> {
    let temp = fixture()?;
    let contract_path = temp.0.join("benchmark/experiment-contract.json");
    let mut contract = read(&contract_path)?;
    let bytes = b"controlled compiler artifact fixture";
    contract["compiler_toolchain"]["sha256"] = json!(format!("{:x}", Sha256::digest(bytes)));
    fs::write(&contract_path, serde_json::to_vec(&contract)?)?;
    for source in ["tracefix", "tracefix-paper"] {
        let path = temp.0.join("vendor").join(source).join("lib/tla2tools.jar");
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)?;
    }
    verify_compiler(&temp.0)?;
    let jar = temp.0.join("vendor/tracefix-paper/lib/tla2tools.jar");
    fs::write(&jar, b"different compiler")?;
    assert!(verify_compiler(&temp.0).is_err());
    assert_eq!(fs::read(&jar)?, b"different compiler");
    fs::remove_file(&jar)?;
    assert!(verify_compiler(&temp.0).is_err());
    fs::write(&jar, bytes)?;
    verify_compiler(&temp.0)?;
    #[cfg(unix)]
    {
        let external = temp.0.join("compiler-jar-copy");
        fs::write(&external, bytes)?;
        fs::remove_file(&jar)?;
        std::os::unix::fs::symlink(&external, &jar)?;
        assert!(verify_compiler(&temp.0).is_err());
        assert!(fs::symlink_metadata(&jar)?.file_type().is_symlink());
        fs::remove_file(&jar)?;
        fs::write(&jar, bytes)?;
    }
    contract["compiler_toolchain"]["sha256"] = json!("not-a-digest");
    fs::write(contract_path, serde_json::to_vec(&contract)?)?;
    assert!(verify_compiler(&temp.0).is_err());
    Ok(())
}

#[test]
fn both_matrices_retain_every_recorded_identity_and_reject_changed_conditions() -> Result<()> {
    matrix::check(&root())?;
    let config = read(&root().join("experiment.json"))?;
    let matrices = experiment_matrix(&config)?;
    let main = matrices["matrix.jsonl"]
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let latest = matrices["latest-matrix.jsonl"]
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(main.len(), 2160);
    assert_eq!(latest.len(), 432);
    let mut identities = BTreeSet::new();
    for run in main.iter().chain(&latest) {
        assert!(identities.insert(run["run_id"].clone().to_string()));
    }
    for task in task_ids() {
        for depth in 0..3 {
            for seed in 0..3 {
                assert_eq!(
                    main.iter()
                        .filter(|r| r["task"] == task
                            && r["scenario_depth"] == depth
                            && r["seed"] == seed)
                        .count(),
                    5
                );
                assert_eq!(
                    latest
                        .iter()
                        .filter(|r| r["task"] == task
                            && r["scenario_depth"] == depth
                            && r["seed"] == seed
                            && r["upstream_revision"] == LATEST)
                        .count(),
                    1
                );
            }
        }
    }
    for (field, value) in [
        ("model", json!("another-model")),
        ("reasoning_effort", json!("low")),
        ("scenario_depths", json!([0])),
        ("expected_runs", json!(432)),
        ("latest_baseline_revision", json!(PAPER)),
        (
            "conditions",
            json!([
                "hibana",
                "hibana",
                "prompt_only",
                "chat_only",
                "mediator_enforced"
            ]),
        ),
    ] {
        let mut changed = config.clone();
        changed[field] = value;
        assert!(
            experiment_matrix(&changed).is_err(),
            "accepted changed {field}"
        );
    }
    let temp = Scratch::new()?;
    fs::create_dir(temp.0.join("benchmark"))?;
    fs::copy(
        root().join("experiment.json"),
        temp.0.join("experiment.json"),
    )?;
    matrix::write(&temp.0)?;
    let path = temp.0.join("benchmark/latest-matrix.jsonl");
    fs::write(
        &path,
        matrices["latest-matrix.jsonl"]
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    let before = fs::read(&path)?;
    assert!(matrix::check(&temp.0).is_err());
    assert_eq!(fs::read(path)?, before);
    Ok(())
}

fn git(repository: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args([
            "-c",
            "user.name=Hayato Ohhashi",
            "-c",
            "user.email=o8@vmm.dev",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

#[test]
fn git_archives_preserve_bytes_without_nested_git_or_overwriting_existing_files() -> Result<()> {
    let temp = Scratch::new()?;
    let repository = temp.0.join("repo");
    fs::create_dir(&repository)?;
    git(&repository, &["init", "--quiet"])?;
    fs::write(repository.join("input.bin"), [0, 17, 0, 255])?;
    git(&repository, &["add", "input.bin"])?;
    git(
        &repository,
        &["commit", "--quiet", "-m", "Record fixture input"],
    )?;
    let revision = git(&repository, &["rev-parse", "HEAD"])?;
    let destination = temp.0.join("source");
    extract(&repository, &revision, &destination)?;
    assert_eq!(fs::read(destination.join("input.bin"))?, [0, 17, 0, 255]);
    assert!(!destination.join(".git").exists());
    fs::write(destination.join("keep"), b"existing source")?;
    assert!(extract(&repository, &revision, &destination).is_err());
    assert_eq!(fs::read(destination.join("keep"))?, b"existing source");
    let missing = temp.0.join("missing-revision");
    assert!(extract(&repository, "does-not-exist", &missing).is_err());
    assert!(!missing.exists());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/outside-source", repository.join("link"))?;
        git(&repository, &["add", "link"])?;
        git(
            &repository,
            &["commit", "--quiet", "-m", "Record unsupported archive link"],
        )?;
        let linked = temp.0.join("linked-source");
        assert!(extract(&repository, "HEAD", &linked).is_err());
        assert!(!linked.exists());
    }
    Ok(())
}
