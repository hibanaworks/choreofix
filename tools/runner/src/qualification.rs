//! Run every migrated family's scripted WASI boundary against one source snapshot.
use crate::evidence::{Result, digest, save};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const FAMILIES: [&str; 6] = [
    "review",
    "codebase",
    "writing",
    "api",
    "philosophers",
    "cicd",
];

fn files(root: &Path, path: &Path, hashes: &mut BTreeMap<PathBuf, String>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if ["target", ".git", "__pycache__"].contains(&entry.file_name().to_string_lossy().as_ref())
        {
            continue;
        }
        if kind.is_dir() {
            files(root, &entry.path(), hashes)?;
        } else if kind.is_file() {
            hashes.insert(
                entry.path().strip_prefix(root)?.to_owned(),
                digest(&entry.path())?,
            );
        } else {
            return Err(format!(
                "source is not a regular file or directory: {}",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(())
}

fn sources(root: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let mut hashes = BTreeMap::new();
    for directory in [
        "src",
        "tests",
        "tools/runner",
        "tools/inputs",
        "vendor/hibana-wash/src",
        "vendor/hibana-wash/scripts",
    ] {
        files(root, &root.join(directory), &mut hashes)?;
    }
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "tools/build_shell.sh",
        "vendor/hibana-wash/Cargo.toml",
        "vendor/hibana-wash/Cargo.lock",
        "experiment.json",
        "requirements.lock",
        "benchmark/upstream-manifest.json",
        "benchmark/experiment-contract.json",
    ] {
        hashes.insert(PathBuf::from(name), digest(&root.join(name))?);
    }
    for directory in ["", "tools/runner", "tools/inputs", "vendor/hibana-wash"] {
        for name in [".cargo/config", ".cargo/config.toml", "build.rs"] {
            let path = Path::new(directory).join(name);
            if root.join(&path).exists() {
                hashes.insert(path.clone(), digest(&root.join(path))?);
            }
        }
    }
    Ok(hashes)
}

fn unchanged(root: &Path, before: &BTreeMap<PathBuf, String>) -> Result<()> {
    if &sources(root)? != before {
        return Err(
            "qualification source changed; results are not a fixed-source qualification".into(),
        );
    }
    Ok(())
}

fn execute(command: &mut Command, output: &Path, name: &str) -> Result<String> {
    save(
        &output.join(format!("{name}-command.json")),
        &json!({
            "program":command.get_program().to_string_lossy(),
            "arguments":command.get_args().map(|a| a.to_string_lossy()).collect::<Vec<_>>()
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

/// Builds the real guests and six hosts; no model connection or scored run is started.
pub fn run(output: &Path) -> Result<Value> {
    let root = std::env::current_dir()?.canonicalize()?;
    let output = root.join(output);
    fs::create_dir_all(output.parent().ok_or("qualification output parent")?)?;
    fs::create_dir(&output)?;
    let output = output.canonicalize()?;
    let before = sources(&root)?;
    save(&output.join("source-before.json"), &json!(before))?;
    for name in before.keys() {
        let saved = output.join("source").join(name);
        fs::create_dir_all(saved.parent().ok_or("source parent")?)?;
        fs::copy(root.join(name), saved)?;
    }
    fs::copy(std::env::current_exe()?, output.join("qualifier"))?;
    let app = std::env::var_os("CHOREOFIX_APP_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("target"));
    let runner = std::env::var_os("CHOREOFIX_RUNNER_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(root.join("tools/runner/target"));
    execute(
        Command::new("sh")
            .arg("tools/build_shell.sh")
            .env("CARGO_TARGET_DIR", &app)
            .env("CARGO_BUILD_JOBS", "1")
            .env("CARGO_INCREMENTAL", "0"),
        &output,
        "build",
    )?;
    unchanged(&root, &before)?;
    let manifest = before
        .iter()
        .map(|(name, hash)| format!("{hash}  {}\n", name.display()))
        .collect::<String>();
    fs::write(output.join("runtime-source.sha256"), manifest)?;
    let mut tests = BTreeMap::new();
    for family in FAMILIES {
        unchanged(&root, &before)?;
        let binary = app
            .join("debug")
            .join(format!("{family}-wasi"))
            .canonicalize()?;
        let receipt = output.join(format!("{family}-build-receipt.json"));
        save(
            &receipt,
            &json!({
                "binary_sha256":digest(&binary)?, "source_manifest":"runtime-source.sha256",
                "source_manifest_sha256":digest(&output.join("runtime-source.sha256"))?,
                "source_unchanged":true, "scored_benchmark":false
            }),
        )?;
        let log = execute(
            Command::new("cargo")
                .args([
                    "+stable",
                    "test",
                    "--locked",
                    "--manifest-path",
                    "tools/runner/Cargo.toml",
                    &format!("{family}_boundary::"),
                    "--",
                    "--ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("CARGO_TARGET_DIR", &runner)
                .env("CARGO_BUILD_JOBS", "1")
                .env("CARGO_INCREMENTAL", "0")
                .env("CHOREOFIX_AUDITOR", output.join("qualifier"))
                .env("CHOREOFIX_EVIDENCE_ROOT", output.join("evidence"))
                .env(
                    format!("CHOREOFIX_{}_WASI_BINARY", family.to_uppercase()),
                    binary,
                )
                .env(
                    format!("CHOREOFIX_{}_BUILD_RECEIPT", family.to_uppercase()),
                    receipt,
                ),
            &output,
            family,
        )?;
        tests.insert(family, completed_tests(&log)?);
        unchanged(&root, &before)?;
    }
    save(&output.join("source-after.json"), &json!(sources(&root)?))?;
    let result = json!({
        "all_six_families_passed":true, "source_unchanged":true,
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

    #[test]
    fn qualification_detects_changed_added_and_removed_source() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "choreofix-source-qualification-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&root)?;
        let result = (|| -> Result<()> {
            for directory in [
                "src",
                "tests",
                "tools/runner",
                "tools/inputs",
                "vendor/hibana-wash/src",
                "vendor/hibana-wash/scripts",
            ] {
                fs::create_dir_all(root.join(directory))?;
            }
            for name in [
                "Cargo.toml",
                "Cargo.lock",
                "tools/build_shell.sh",
                "vendor/hibana-wash/Cargo.toml",
                "vendor/hibana-wash/Cargo.lock",
                "experiment.json",
                "requirements.lock",
                "benchmark/upstream-manifest.json",
                "benchmark/experiment-contract.json",
                "src/lib.rs",
            ] {
                let path = root.join(name);
                fs::create_dir_all(path.parent().ok_or("fixture parent")?)?;
                fs::write(path, b"original")?;
            }
            let original = sources(&root)?;
            unchanged(&root, &original)?;
            fs::write(root.join("src/lib.rs"), b"changed")?;
            assert!(unchanged(&root, &original).is_err());
            fs::write(root.join("src/lib.rs"), b"original")?;
            fs::write(root.join("src/added.rs"), b"added")?;
            assert!(unchanged(&root, &original).is_err());
            fs::remove_file(root.join("src/added.rs"))?;
            fs::remove_file(root.join("src/lib.rs"))?;
            assert!(unchanged(&root, &original).is_err());
            Ok(())
        })();
        fs::remove_dir_all(root)?;
        result
    }
}
