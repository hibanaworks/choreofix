use crate::{Result, SOURCES, verify, verify_benchmark, verify_runtime_source};
use serde_json::Value;
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
pub(crate) struct Scratch(pub PathBuf);
impl Scratch {
    pub(crate) fn new() -> Result<Self> {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "choreofix-inputs-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err("could not create an owned source staging directory".into())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("source staging cleanup {}: {error}", self.0.display());
        }
    }
}

fn output(command: &mut Command) -> Result<Vec<u8>> {
    let result = command.output()?;
    if !result.status.success() {
        return Err(format!(
            "source command failed: {:?}: {}",
            command,
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(result.stdout)
}

pub(crate) fn extract(repository: &Path, revision: &str, destination: &Path) -> Result<()> {
    if destination.exists() {
        return Err("source destination already exists".into());
    }
    let tree = output(
        Command::new("git")
            .arg("-C")
            .arg(repository)
            .args(["ls-tree", "-rz", revision]),
    )?;
    for entry in tree.split(|b| *b == 0).filter(|e| !e.is_empty()) {
        let separator = entry
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("invalid Git tree row")?;
        let (header, path) = entry.split_at(separator);
        let path = &path[1..];
        let mode = header.split(|b| *b == b' ').next().ok_or("Git tree mode")?;
        if ![b"100644".as_slice(), b"100755".as_slice()].contains(&mode) {
            return Err(
                "source archives must contain ordinary tracked files, not links or submodules"
                    .into(),
            );
        }
        let path = Path::new(std::str::from_utf8(path)?);
        if !path
            .components()
            .all(|c| matches!(c,Component::Normal(name) if name != ".git"))
        {
            return Err("source archive path escapes its directory".into());
        }
    }
    let archive = Scratch::new()?;
    let path = archive.0.join("source.tar");
    output(
        Command::new("git")
            .arg("-C")
            .arg(repository)
            .args(["archive", "--format=tar", "--output"])
            .arg(&path)
            .arg(revision),
    )?;
    fs::create_dir(destination)?;
    output(
        Command::new("tar")
            .arg("-xf")
            .arg(path)
            .arg("-C")
            .arg(destination),
    )?;
    Ok(())
}

/// Restore missing immutable archives only; never repair or overwrite a source.
pub fn restore(root: &Path) -> Result<Value> {
    verify_benchmark(root)?;
    let mut missing = Vec::new();
    for spec in &SOURCES {
        let path = root.join("vendor").join(spec.name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.file_type().is_dir() {
                    return Err("existing upstream destination is not a directory".into());
                }
                verify_runtime_source(root, &path, spec)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => missing.push(spec),
            Err(error) => return Err(error.into()),
        }
    }
    if !missing.is_empty() {
        let scratch = Scratch::new()?;
        let repository = scratch.0.join("repository");
        output(
            Command::new("git")
                .args([
                    "clone",
                    "--quiet",
                    "--no-checkout",
                    "https://github.com/Sensing-And-Reasoning/TraceFix.git",
                ])
                .arg(&repository),
        )?;
        fs::create_dir_all(root.join("vendor"))?;
        for spec in missing {
            let staged = scratch.0.join(spec.name);
            extract(&repository, spec.revision, &staged)?;
            verify_runtime_source(root, &staged, spec)?;
            let destination = root.join("vendor").join(spec.name);
            match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    return Err(
                        "upstream destination appeared while restoring; left untouched".into(),
                    );
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            fs::rename(staged, destination)?;
        }
    }
    verify(root)
}
