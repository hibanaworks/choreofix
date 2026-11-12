//! Immutable benchmark inputs and experiment identities, outside task execution.
pub mod matrix;
mod restore;
pub mod wasm;

pub use matrix::experiment_matrix;
pub use restore::restore;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const SIMULATOR_CONTRACT: &str = "common-environment-v18";
pub const LATEST: &str = "36b4bd108fcd74f7efb17613aebbe3fdf55499a9";
pub const PAPER: &str = "87b01a2c0ae53c5045c9ce3d43cd0235b1831275";

struct Source {
    name: &'static str,
    revision: &'static str,
    directories: &'static [&'static str],
    files: &'static [&'static str],
}

const SOURCES: [Source; 2] = [
    Source {
        name: "tracefix",
        revision: LATEST,
        directories: &["tracefix/runtime", "tracefix/pipeline", "tracefix/cli"],
        files: &[
            "LICENSE",
            "README.md",
            "pyproject.toml",
            "tracefix/__init__.py",
            "scripts/download_tla2tools.sh",
        ],
    },
    Source {
        name: "tracefix-paper",
        revision: PAPER,
        directories: &[
            "runtime_A",
            "runtime_B",
            "runtime_base_1",
            "runtime_base_2",
            "v3_agent_test",
            "tla_verify_pluscal",
        ],
        files: &["LICENSE", "README.md", "pyproject.toml"],
    },
];

fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&bytes[..n]);
    }
}

fn files(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !fs::symlink_metadata(directory)?.file_type().is_dir() {
        return Err(format!(
            "source directory is not an ordinary directory: {}",
            directory.display()
        )
        .into());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_name() == "__pycache__" {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(format!("unexpected source symlink: {}", entry.path().display()).into());
        }
        if kind.is_dir() {
            files(&entry.path(), paths)?;
        } else if kind.is_file() {
            paths.push(entry.path());
        } else {
            return Err("unexpected source file type".into());
        }
    }
    Ok(())
}

fn hashes(source: &Path, mut paths: Vec<PathBuf>) -> Result<Value> {
    paths.sort();
    let mut result = Map::new();
    for path in paths {
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(format!("source is not an ordinary file: {}", path.display()).into());
        }
        let name = path
            .strip_prefix(source)?
            .to_str()
            .ok_or("source path UTF-8")?;
        result.insert(name.into(), json!(digest(&path)?));
    }
    Ok(json!(result))
}

pub fn task_ids() -> Vec<String> {
    (1..=16)
        .flat_map(|number| ['E', 'M', 'H'].map(|level| format!("{number}{level}")))
        .collect()
}

fn directories(path: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err("source directory is a symlink".into());
        }
        if kind.is_dir() && entry.file_name() != "__pycache__" {
            names.insert(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "task name UTF-8")?,
            );
        }
    }
    Ok(names)
}

/// Inventory the actual world bytes under their explicitly declared contract.
/// A label identifies a source version; it does not certify its semantics.
pub fn simulator_manifest(source: &Path, contract: &str) -> Result<Value> {
    if contract.is_empty() {
        return Err("missing simulator contract identity".into());
    }
    let benchmark = source.join("benchmark");
    let ids = task_ids();
    let expected = ids.iter().cloned().collect::<BTreeSet<_>>();
    for directory in ["descriptions", "environments"] {
        if directories(&benchmark.join(directory))? != expected {
            return Err(format!("upstream {directory} task set differs from all 48 tasks").into());
        }
    }
    let mut tasks = Vec::new();
    for id in ids {
        let description = benchmark.join("descriptions").join(&id);
        let environment = benchmark.join("environments").join(&id);
        let metadata = read(&description.join("metadata.json"))?;
        let tools = read(&description.join("tools.json"))?;
        let checklist = read(&environment.join("checklist.json"))?;
        let mut paths = Vec::new();
        files(&description, &mut paths)?;
        files(&environment, &mut paths)?;
        let tools = tools
            .as_array()
            .ok_or("original tools array")?
            .iter()
            .map(|entry| {
                entry["function"]["name"]
                    .as_str()
                    .ok_or("original tool name")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if !metadata["agents"].is_array() || !metadata["resources"].is_array() {
            return Err("original participant or resource array missing".into());
        }
        tasks.push(
            json!({"id":id,"agents":metadata["agents"],"resources":metadata["resources"],
            "tools":tools,"checklist_items":checklist.as_array().ok_or("original checklist")?.len(),
            "files":hashes(source,paths)?}),
        );
    }
    let mut paths = Vec::new();
    files(&benchmark.join("tools"), &mut paths)?;
    paths.retain(|p| p.extension().is_some_and(|e| e == "py"));
    paths.push(benchmark.join("__init__.py"));
    Ok(
        json!({"upstream_revision":LATEST,"simulator_contract":contract,"tasks":tasks,"simulator_files":hashes(source,paths)?}),
    )
}

pub fn verify_benchmark(root: &Path) -> Result<Value> {
    let recorded = read(&root.join("benchmark/simulator-manifest.json"))?;
    let contract = recorded["simulator_contract"]
        .as_str()
        .ok_or("missing simulator contract identity")?;
    let actual = simulator_manifest(root, contract)?;
    if actual != recorded {
        return Err("common simulator bytes or metadata differ from the fixed manifest".into());
    }
    let tasks = actual["tasks"].as_array().ok_or("tasks")?;
    Ok(
        json!({"tasks":tasks.len(),"scenarios":16,"unchanged":false,"simulator_contract":contract,
        "checklist_items":tasks.iter().map(|t|t["checklist_items"].as_u64().expect("generated count")).sum::<u64>()}),
    )
}

fn runtime_inventory(source: &Path, spec: &Source) -> Result<Value> {
    if !fs::symlink_metadata(source)?.file_type().is_dir() {
        return Err("baseline source is not an ordinary directory".into());
    }
    let mut paths = spec.files.iter().map(|file| source.join(file)).collect();
    for directory in spec.directories {
        files(&source.join(directory), &mut paths)?;
    }
    Ok(json!({"revision":spec.revision,"files":hashes(source,paths)?}))
}

fn verify_runtime_source(root: &Path, source: &Path, spec: &Source) -> Result<usize> {
    let actual = runtime_inventory(source, spec)?;
    let expected = read(&root.join("benchmark/runtime-manifest.json"))?;
    if actual != expected[spec.name] {
        return Err(format!(
            "{} runtime/compiler bytes differ from the pinned manifest",
            spec.name
        )
        .into());
    }
    Ok(actual["files"].as_object().ok_or("runtime files")?.len())
}

pub fn verify_runtimes(root: &Path) -> Result<Value> {
    let expected = read(&root.join("benchmark/runtime-manifest.json"))?;
    let names = expected
        .as_object()
        .ok_or("runtime sources")?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if names != SOURCES.iter().map(|s| s.name).collect() {
        return Err("baseline source set changed".into());
    }
    let mut counts = Map::new();
    for spec in &SOURCES {
        counts.insert(
            spec.name.into(),
            json!(verify_runtime_source(
                root,
                &root.join("vendor").join(spec.name),
                spec
            )?),
        );
    }
    Ok(json!(counts))
}

/// Check the separately distributed compiler artifact. Source archives do not
/// contain this JAR, and source verification does not qualify its version.
pub fn verify_compiler(root: &Path) -> Result<Value> {
    let contract = read(&root.join("benchmark/experiment-contract.json"))?;
    let expected = contract["compiler_toolchain"]["sha256"]
        .as_str()
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("compiler SHA-256 pin missing or malformed")?;
    let mut checked = Map::new();
    for spec in &SOURCES {
        let path = root
            .join("vendor")
            .join(spec.name)
            .join("lib/tla2tools.jar");
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(format!(
                "compiler artifact is not an ordinary file: {}",
                path.display()
            )
            .into());
        }
        let actual = digest(&path)?;
        if actual != expected {
            return Err(format!(
                "unqualified compiler artifact {}: expected {expected}, found {actual}; left unchanged",
                path.display(),
            ).into());
        }
        checked.insert(spec.name.into(), json!(actual));
    }
    Ok(json!({"version":contract["compiler_toolchain"]["version"],"artifacts":checked}))
}

fn verify_baselines(root: &Path) -> Result<Value> {
    let config = read(&root.join("experiment.json"))?;
    let contract = read(&root.join("benchmark/experiment-contract.json"))?;
    let manifest = read(&root.join("benchmark/runtime-manifest.json"))?;
    let mut conditions = BTreeSet::new();
    for field in ["conditions", "supplementary_conditions"] {
        for condition in config[field].as_array().ok_or("comparison conditions")? {
            let condition = condition.as_str().ok_or("comparison condition name")?;
            if condition != "hibana" && !conditions.insert(condition) {
                return Err("duplicate comparison condition".into());
            }
        }
    }
    let runtimes = contract["baseline_runtimes"]
        .as_object()
        .ok_or("baseline runtime mapping")?;
    if runtimes.keys().map(String::as_str).collect::<BTreeSet<_>>() != conditions {
        return Err("baseline mapping does not cover the recorded comparison conditions".into());
    }
    for (condition, runtime) in runtimes {
        let source = runtime["source"].as_str().ok_or("baseline source")?;
        let pinned = manifest.get(source).ok_or("unknown baseline source")?;
        if runtime["revision"] != pinned["revision"] {
            return Err(
                format!("baseline revision differs from pinned source: {condition}").into(),
            );
        }
        let module = runtime["module"].as_str().ok_or("baseline module")?;
        if module.split('.').any(|part| {
            part.is_empty() || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        }) || pinned["files"]
            .get(format!("{}/__main__.py", module.replace('.', "/")))
            .is_none()
        {
            return Err(format!("baseline module has no pinned entrypoint: {condition}").into());
        }
    }
    let api = &contract["participant_api"];
    if api["model"] != config["model"]
        || api["reasoning_effort"] != config["reasoning_effort"]
        || api["required_endpoint"] != "codex_app_server"
        || api["authentication"] != "chatgpt"
        || api["paid_api_calls"] != false
    {
        return Err("participant API contract differs from the recorded model condition".into());
    }
    Ok(
        json!({"conditions":runtimes.len(),"entrypoints_pinned":true,
        "model":api["model"],"reasoning_effort":api["reasoning_effort"],
        "required_endpoint":api["required_endpoint"],
        "authentication":api["authentication"],"paid_api_calls":api["paid_api_calls"]}),
    )
}

pub fn verify(root: &Path) -> Result<Value> {
    Ok(
        json!({"benchmark":verify_benchmark(root)?,"runtimes":verify_runtimes(root)?,
        "baselines":verify_baselines(root)?}),
    )
}

#[cfg(test)]
mod tests;
