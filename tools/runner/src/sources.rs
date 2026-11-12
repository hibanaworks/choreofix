//! Build inputs and their portable locations in one archived workspace.
use crate::evidence::{Result, digest};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub fn files(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if ["target", "__pycache__", ".git"].contains(&entry.file_name().to_string_lossy().as_ref())
        {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_dir() {
            files(&entry.path(), result)?;
        } else if kind.is_file() {
            result.push(entry.path());
        } else {
            return Err(format!("source is not a regular file: {}", entry.path().display()).into());
        }
    }
    Ok(())
}

fn from_metadata(root: &Path, metadata: &Value) -> Result<BTreeMap<PathBuf, PathBuf>> {
    let root = root.canonicalize()?;
    let workspace = root.parent().ok_or("source workspace parent")?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("resolved source packages")?;
    let core = packages
        .iter()
        .filter(|package| package["name"] == "hibana")
        .collect::<Vec<_>>();
    let [core] = core.as_slice() else {
        return Err("one canonical core is required by the source snapshot".into());
    };
    if !core["source"].is_null()
        || Path::new(core["manifest_path"].as_str().ok_or("core manifest")?).canonicalize()?
            != workspace.join("hibana/Cargo.toml").canonicalize()?
    {
        return Err("the source snapshot must use the current canonical core".into());
    }
    let mut paths = Vec::new();
    for directory in [
        "src",
        "tests",
        "benchmark",
        "tools/runner",
        "tools/inputs",
        "vendor/hibana-wash/src",
        "vendor/hibana-wash/scripts",
    ] {
        files(&root.join(directory), &mut paths)?;
    }
    paths.extend(
        [
            "Cargo.toml",
            "Cargo.lock",
            "tools/build_shell.sh",
            "vendor/hibana-wash/Cargo.toml",
            "vendor/hibana-wash/Cargo.lock",
            "experiment.json",
            "requirements.lock",
            "benchmark/upstream-manifest.json",
            "benchmark/simulator-manifest.json",
            "benchmark/experiment-contract.json",
        ]
        .map(|name| root.join(name)),
    );
    for directory in ["", "tools/runner", "tools/inputs", "vendor/hibana-wash"] {
        for name in [".cargo/config", ".cargo/config.toml", "build.rs"] {
            let path = root.join(directory).join(name);
            if path.exists() {
                paths.push(path);
            }
        }
    }
    let mut inputs = BTreeMap::new();
    for path in paths {
        inputs.insert(Path::new("source").join(path.strip_prefix(&root)?), path);
    }
    let mut repositories = BTreeSet::new();
    for package in packages {
        if !package["source"].is_null() {
            continue;
        }
        let manifest = Path::new(
            package["manifest_path"]
                .as_str()
                .ok_or("package manifest")?,
        )
        .canonicalize()?;
        if manifest.starts_with(&root) {
            continue;
        }
        let directory = manifest.parent().ok_or("dependency directory")?;
        let output = Command::new("git")
            .current_dir(directory)
            .args(["rev-parse", "--show-toplevel"])
            .output()?;
        if !output.status.success() {
            return Err("local dependency has no recorded source repository".into());
        }
        let repository = PathBuf::from(String::from_utf8(output.stdout)?.trim()).canonicalize()?;
        if repository.parent() != Some(workspace) || repository == root {
            return Err("local dependency is outside the canonical sibling workspace".into());
        }
        repositories.insert(repository);
    }
    for repository in repositories {
        let prefix = repository.file_name().ok_or("dependency source name")?;
        if prefix == "source" {
            return Err("dependency collides with archived application sources".into());
        }
        let output = Command::new("git")
            .current_dir(&repository)
            .args([
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ])
            .output()?;
        if !output.status.success() {
            return Err("cannot enumerate the actual local dependency source".into());
        }
        for name in String::from_utf8(output.stdout)?
            .split('\0')
            .filter(|name| !name.is_empty())
        {
            let relative = Path::new(name);
            if !relative
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            {
                return Err("dependency source path escapes its repository".into());
            }
            let path = repository.join(relative);
            match fs::symlink_metadata(&path) {
                Ok(info) if info.file_type().is_file() => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                _ => {
                    return Err(format!(
                        "dependency source is not a regular file: {}",
                        path.display()
                    )
                    .into());
                }
            }
            let actual = path.canonicalize()?;
            if !actual.starts_with(&repository) {
                return Err("dependency source escapes its repository".into());
            }
            inputs.insert(Path::new(prefix).join(relative), actual);
        }
    }
    Ok(inputs)
}

pub fn inputs(root: &Path) -> Result<BTreeMap<PathBuf, PathBuf>> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args([
            "+stable",
            "metadata",
            "--offline",
            "--locked",
            "--format-version",
            "1",
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "cannot resolve build sources: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    from_metadata(root, &serde_json::from_slice(&output.stdout)?)
}

pub fn hashes(inputs: &BTreeMap<PathBuf, PathBuf>) -> Result<BTreeMap<PathBuf, String>> {
    inputs
        .iter()
        .map(|(name, path)| Ok((name.clone(), digest(path)?)))
        .collect()
}

pub fn git_provenance(root: &Path, inputs: &BTreeMap<PathBuf, PathBuf>) -> Result<Value> {
    let root = root.canonicalize()?;
    let workspace = root.parent().ok_or("source workspace parent")?;
    let mut provenance = serde_json::Map::new();
    for name in inputs.keys() {
        let prefix = name
            .components()
            .next()
            .ok_or("source repository")?
            .as_os_str();
        let key = prefix.to_str().ok_or("source repository name")?;
        if provenance.contains_key(key) {
            continue;
        }
        let directory = if key == "source" {
            root.clone()
        } else {
            workspace.join(prefix)
        };
        let mut values = Vec::new();
        for arguments in [vec!["rev-parse", "HEAD"], vec!["diff", "--binary", "HEAD"]] {
            let result = Command::new("git")
                .current_dir(&directory)
                .args(arguments)
                .output()?;
            if !result.status.success() {
                return Err("cannot record source Git provenance".into());
            }
            values.push(String::from_utf8(result.stdout)?);
        }
        provenance.insert(
            key.to_owned(),
            serde_json::json!({
                "git_base":values[0].trim(), "dirty_patch":values[1],
            }),
        );
    }
    Ok(Value::Object(provenance))
}

pub fn manifest(path: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let mut entries = BTreeMap::new();
    for row in fs::read_to_string(path)?.lines() {
        let (hash, name) = row.split_once("  ").ok_or("invalid build-source row")?;
        let name = Path::new(name);
        if hash.len() != 64
            || !hash.bytes().all(|c| c.is_ascii_hexdigit())
            || name.as_os_str().is_empty()
            || !name.components().all(|c| matches!(c, Component::Normal(_)))
            || entries.insert(name.to_owned(), hash.to_owned()).is_some()
        {
            return Err("invalid or duplicate build-source entry".into());
        }
    }
    if entries.is_empty() {
        return Err("empty build-source manifest".into());
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn local_dependency_changes_are_build_changes_and_archived_paths_relocate() -> Result<()> {
        let workspace = std::env::temp_dir().join(format!(
            "choreofix-workspace-source-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&workspace)?;
        let result = (|| -> Result<()> {
            let app = workspace.join("app");
            let core = workspace.join("hibana");
            for directory in [
                "src",
                "tests",
                "tools/runner",
                "tools/inputs",
                "vendor/hibana-wash/src",
                "vendor/hibana-wash/scripts",
            ] {
                fs::create_dir_all(app.join(directory))?;
            }
            for name in [
                "tools/build_shell.sh",
                "vendor/hibana-wash/Cargo.toml",
                "vendor/hibana-wash/Cargo.lock",
                "experiment.json",
                "requirements.lock",
                "benchmark/upstream-manifest.json",
                "benchmark/simulator-manifest.json",
                "benchmark/experiment-contract.json",
            ] {
                let path = app.join(name);
                fs::create_dir_all(path.parent().ok_or("fixture parent")?)?;
                fs::write(path, b"fixture")?;
            }
            fs::write(
                app.join("Cargo.toml"),
                "[package]\nname='source-fixture'\nversion='0.0.0'\nedition='2024'\n[dependencies]\nhibana={path='../hibana'}\n",
            )?;
            fs::write(app.join("src/lib.rs"), "pub fn application() {}\n")?;
            fs::create_dir_all(core.join("src"))?;
            fs::write(
                core.join("Cargo.toml"),
                "[package]\nname='hibana'\nversion='0.10.0'\nedition='2024'\n",
            )?;
            fs::write(core.join("Cargo.lock"), "version = 4\n")?;
            let implementation = core.join("src/lib.rs");
            fs::write(&implementation, "pub fn current() {}\n")?;
            if !Command::new("git")
                .args(["init", "--quiet"])
                .arg(&core)
                .status()?
                .success()
            {
                return Err("cannot initialize the dependency fixture".into());
            }
            let resolved = Command::new("cargo")
                .current_dir(&app)
                .args(["+stable", "metadata", "--offline", "--format-version", "1"])
                .output()?;
            if !resolved.status.success() {
                return Err(String::from_utf8_lossy(&resolved.stderr)
                    .into_owned()
                    .into());
            }
            let original = hashes(&inputs(&app)?)?;
            let application = app.join("src/lib.rs");
            fs::write(&application, "pub fn changed_application() {}\n")?;
            assert_ne!(hashes(&inputs(&app)?)?, original);
            fs::write(&application, "pub fn application() {}\n")?;
            fs::remove_file(&application)?;
            let missing = inputs(&app).and_then(|inputs| hashes(&inputs));
            assert!(!missing.as_ref().is_ok_and(|current| current == &original));
            fs::write(&application, "pub fn application() {}\n")?;
            fs::write(&implementation, "pub fn changed() {}\n")?;
            assert_ne!(hashes(&inputs(&app)?)?, original);
            fs::write(&implementation, "pub fn current() {}\n")?;
            let added = core.join("src/added.rs");
            fs::write(&added, "pub fn added() {}\n")?;
            assert_ne!(hashes(&inputs(&app)?)?, original);
            fs::remove_file(&added)?;
            fs::remove_file(&implementation)?;
            let missing = inputs(&app).and_then(|inputs| hashes(&inputs));
            assert!(!missing.as_ref().is_ok_and(|current| current == &original));
            fs::write(&implementation, "pub fn current() {}\n")?;
            let added = app.join("src/added.rs");
            fs::write(&added, "pub fn added() {}\n")?;
            assert_ne!(hashes(&inputs(&app)?)?, original);
            fs::remove_file(&added)?;
            assert_eq!(hashes(&inputs(&app)?)?, original);
            let archive = workspace.join("archive");
            for (name, path) in inputs(&app)? {
                let saved = archive.join(name);
                fs::create_dir_all(saved.parent().ok_or("archive parent")?)?;
                fs::copy(path, saved)?;
            }
            let relocated = workspace.join("relocated");
            fs::rename(archive, &relocated)?;
            let metadata = Command::new("cargo")
                .current_dir(relocated.join("source"))
                .args([
                    "+stable",
                    "metadata",
                    "--offline",
                    "--locked",
                    "--format-version",
                    "1",
                ])
                .output()?;
            if !metadata.status.success() {
                return Err(String::from_utf8_lossy(&metadata.stderr)
                    .into_owned()
                    .into());
            }
            let metadata: Value = serde_json::from_slice(&metadata.stdout)?;
            let core = metadata["packages"]
                .as_array()
                .ok_or("archived packages")?
                .iter()
                .filter(|package| package["name"] == "hibana")
                .collect::<Vec<_>>();
            let [core] = core.as_slice() else {
                return Err("archived core is not unique".into());
            };
            assert_eq!(
                Path::new(core["manifest_path"].as_str().ok_or("archived manifest")?)
                    .canonicalize()?,
                relocated.join("hibana/Cargo.toml").canonicalize()?
            );
            Ok(())
        })();
        fs::remove_dir_all(workspace)?;
        result
    }
}
