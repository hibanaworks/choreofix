use crate::{
    evidence::{Result, archive_build_receipt, digest, save},
    sources,
};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn command(program: &str, arguments: &[&str]) -> Result<String> {
    let result = Command::new(program).args(arguments).output()?;
    if !result.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

/// Verify the archived application and local dependencies against the build inputs.
/// A binary hash alone cannot identify which source a build receipt describes.
pub fn runtime_sources(source: &Path, manifest: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let entries = sources::manifest(manifest)?;
    for (name, hash) in &entries {
        if digest(&source.join(name))? != *hash {
            return Err(
                format!("runtime source differs from its build: {}", name.display()).into(),
            );
        }
    }
    let mut required = Vec::new();
    let mut packages = BTreeSet::from([PathBuf::from("source")]);
    for name in entries.keys() {
        packages.insert(PathBuf::from(
            name.components()
                .next()
                .ok_or("source package")?
                .as_os_str(),
        ));
    }
    for package in packages {
        let directory = source.join(package);
        sources::files(&directory.join("src"), &mut required)?;
        required.extend([directory.join("Cargo.toml"), directory.join("Cargo.lock")]);
        for name in [".cargo/config", ".cargo/config.toml", "build.rs"] {
            let path = directory.join(name);
            if path.exists() {
                required.push(path);
            }
        }
    }
    if required.iter().any(|path| {
        path.strip_prefix(source)
            .ok()
            .is_none_or(|name| !entries.contains_key(name))
    }) {
        return Err("build-source manifest omits runtime source files".into());
    }
    Ok(entries)
}

/// Preserve the selected executable, its checked build receipt and experiment inputs.
/// This selects artifacts before a run; it cannot authorize a guest operation.
pub fn freeze(task: &str, output: &Path) -> Result<Value> {
    choreofix_inputs::verify_benchmark(Path::new("."))?;
    let family = match task {
        "1E" | "1M" | "1H" => "CODEBASE",
        "2E" | "2M" | "2H" => "WRITING",
        "3E" => "REPORT",
        "7E" => "DOCUMENT",
        "10E" => "BUILD",
        "10M" => "CASCADING_BUILD",
        "11E" => "MANUFACTURING",
        "12E" => "KITCHEN",
        "4E" => "COORDINATION",
        "4M" => "REFACTOR",
        "14E" => "DISCOVERY",
        "14M" => "REGULATORY_DISCOVERY",
        "14H" => "MULTI_ROUND_DISCOVERY",
        "13E" => "LAB",
        "13M" => "LAB_QUALITY",
        "5E" => "MEDICAL",
        "5M" => "MULTI_SPECIALTY",
        "5H" => "CRITICAL_CARE",
        "6E" => "REVIEW",
        "6M" => "FEATURE_DELIVERY",
        "8E" => "API",
        "9E" | "9M" | "9H" => "PHILOSOPHERS",
        "16E" => "CICD",
        _ => return Err("task has no delegated WASI implementation".into()),
    };
    let runtime_key = format!("CHOREOFIX_{family}_WASI_BINARY");
    let receipt_key = format!("CHOREOFIX_{family}_BUILD_RECEIPT");
    let runtime = PathBuf::from(std::env::var_os(&runtime_key).ok_or(runtime_key)?);
    let input_receipt = PathBuf::from(std::env::var_os(&receipt_key).ok_or(receipt_key)?);
    let receipt = archive_build_receipt(&runtime, &input_receipt, output)?;
    let root = std::env::current_dir()?.canonicalize()?;
    let mut inputs = sources::inputs(&root)?;
    if sources::manifest(&output.join("build-source.sha256"))? != sources::hashes(&inputs)? {
        return Err("runtime or local dependency source differs from its build".into());
    }
    let mut paths = Vec::new();
    for directory in [
        "src",
        "tools/runner",
        "tools/inputs",
        "vendor/hibana-wash/src",
        "benchmark",
    ] {
        sources::files(Path::new(directory), &mut paths)?;
    }
    paths.extend(
        [
            "Cargo.toml",
            "Cargo.lock",
            "experiment.json",
            "requirements.lock",
            "AGENTS.md",
            "plan.md",
            "benchmark/upstream-manifest.json",
            "benchmark/simulator-manifest.json",
            "benchmark/experiment-contract.json",
            "vendor/hibana-wash/Cargo.toml",
            "vendor/hibana-wash/Cargo.lock",
        ]
        .map(PathBuf::from),
    );
    for path in paths {
        inputs.insert(Path::new("source").join(&path), root.join(path));
    }
    let mut manifest = Map::new();
    for (path, original) in inputs {
        let saved = output.join(&path);
        fs::create_dir_all(saved.parent().ok_or("source parent")?)?;
        fs::copy(original, &saved)?;
        manifest.insert(path.to_string_lossy().into_owned(), json!(digest(&saved)?));
    }
    runtime_sources(output, &output.join("build-source.sha256"))?;
    save(&output.join("source-manifest.json"), &json!(manifest))?;
    let artifacts = [
        ("runtime", runtime),
        (
            "wash.wasm",
            PathBuf::from("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
        ),
        ("runner", std::env::current_exe()?),
    ];
    let mut binaries = Map::new();
    for (name, source) in artifacts {
        let saved = output.join(name);
        fs::copy(source, &saved)?;
        binaries.insert(name.into(), json!(digest(&saved)?));
    }
    if binaries["runtime"] != receipt["binary_sha256"] {
        return Err("runtime changed while archiving its build".into());
    }
    let projection = crate::projection::archive(&output.join("runtime"), output, task)?;
    if projection["runtime_binary_sha256"] != binaries["runtime"] {
        return Err("compiled projection executable differs from the snapshot".into());
    }
    let projection = json!(digest(&output.join("projection.json"))?);
    let versions = json!({
        "format_version":5,
        "checkout_head":command("git", &["rev-parse", "HEAD"])?,
        "host_toolchain":command("rustc", &["+stable", "-Vv"])?,
        "runtime_build_receipt":receipt,
        "build_receipt_sha256":digest(&output.join("build-receipt.json"))?,
        "input_build_receipt_sha256":digest(&output.join("input-build-receipt.json"))?,
        "runtime_source_manifest_sha256":digest(&output.join("build-source.sha256"))?,
        "upstream_interpreter":crate::oracle::interpreter_identity()?,
        "binaries":binaries,
        "source_manifest_sha256":digest(&output.join("source-manifest.json"))?,
        "compiled_projection_sha256":projection,
    });
    fs::write(
        output.join("dependency-tree.txt"),
        command("cargo", &["+stable", "tree", "--locked", "-i", "hibana"])?,
    )?;
    save(&output.join("versions.json"), &versions)?;
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn a_build_manifest_must_cover_the_actual_runtime_and_reject_changed_or_escaping_sources() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-runtime-source-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("source/src")).unwrap();
        for (name, bytes) in [
            ("source/src/lib.rs", "pub fn execute() {}\n"),
            ("source/Cargo.toml", "[package]\n"),
            ("source/Cargo.lock", "version = 4\n"),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        let entries: Vec<_> = [
            "source/src/lib.rs",
            "source/Cargo.toml",
            "source/Cargo.lock",
        ]
        .map(|name| format!("{}  {name}\n", digest(&root.join(name)).unwrap()))
        .into();
        let manifest = root.join("build-source.sha256");
        let original = entries.concat();
        fs::write(&manifest, &original).unwrap();
        assert_eq!(runtime_sources(&root, &manifest).unwrap().len(), 3);
        for bad in [
            String::new(),
            entries[1..].concat(),
            format!("{original}{}", entries[0]),
            format!("{}  ../Cargo.toml\n", "0".repeat(64)),
            format!("{}  /tmp/source.rs\n", "0".repeat(64)),
        ] {
            fs::write(&manifest, bad).unwrap();
            assert!(runtime_sources(&root, &manifest).is_err());
        }
        fs::write(&manifest, &original).unwrap();
        fs::write(root.join("source/src/lib.rs"), "pub fn bypass() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::write(root.join("source/src/lib.rs"), "pub fn execute() {}\n").unwrap();
        fs::write(root.join("source/src/receiver.rs"), "pub fn effect() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::remove_file(root.join("source/src/receiver.rs")).unwrap();
        fs::create_dir(root.join("source/.cargo")).unwrap();
        let config = root.join("source/.cargo/config.toml");
        fs::write(&config, "[build]\njobs = 1\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::write(
            &manifest,
            format!(
                "{original}{}  source/.cargo/config.toml\n",
                digest(&config).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(runtime_sources(&root, &manifest).unwrap().len(), 4);
        fs::create_dir_all(root.join("hibana/src")).unwrap();
        for (name, bytes) in [
            ("hibana/Cargo.toml", "[package]\n"),
            ("hibana/Cargo.lock", "version = 4\n"),
            ("hibana/src/lib.rs", "pub fn current_core() {}\n"),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        let mut complete = fs::read_to_string(&manifest).unwrap();
        for name in [
            "hibana/Cargo.toml",
            "hibana/Cargo.lock",
            "hibana/src/lib.rs",
        ] {
            complete.push_str(&format!("{}  {name}\n", digest(&root.join(name)).unwrap()));
        }
        fs::write(&manifest, &complete).unwrap();
        assert_eq!(runtime_sources(&root, &manifest).unwrap().len(), 7);
        fs::write(root.join("hibana/src/lib.rs"), "pub fn other_core() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::write(root.join("hibana/src/lib.rs"), "pub fn current_core() {}\n").unwrap();
        fs::write(root.join("hibana/src/added.rs"), "pub fn added() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::remove_file(root.join("hibana/src/added.rs")).unwrap();
        let relocated = root.with_extension("relocated");
        fs::rename(&root, &relocated).unwrap();
        assert_eq!(
            runtime_sources(&relocated, &relocated.join("build-source.sha256"))
                .unwrap()
                .len(),
            7
        );
        fs::remove_dir_all(relocated).unwrap();
    }
}
