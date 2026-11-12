//! Source-bound pilot evidence, without rerunning or upgrading saved verdicts.
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 32 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

fn candidates(root: &Path, found: &mut Vec<PathBuf>, skipped: &mut Vec<Value>) -> Result<()> {
    let mut entries = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            skipped.push(json!({"path":entry.path(),"reason":"symlink not traversed"}));
        } else if kind.is_dir() {
            if ![
                "source",
                "src",
                "target",
                "vendor",
                ".git",
                "__pycache__",
                "record-mutants",
                "record-mutations",
                "mutants",
                "hibana",
                "hibana-epf",
                "hibana-wasip1-runtime",
            ]
            .iter()
            .any(|name| entry.file_name() == *name)
            {
                candidates(&entry.path(), found, skipped)?;
            }
        } else if kind.is_file() && entry.file_name() == "result.json" {
            found.push(entry.path());
        }
    }
    Ok(())
}

fn auxiliary(
    root: &Path,
    name: &str,
    row: &mut Map<String, Value>,
    errors: &mut Vec<String>,
) -> Value {
    match load(&root.join(name)).and_then(|value| Ok((value, digest(&root.join(name))?))) {
        Ok((value, hash)) => {
            row.insert(
                format!(
                    "{}_sha256",
                    name.trim_end_matches(".json").replace('-', "_")
                ),
                json!(hash),
            );
            value
        }
        Err(error) => {
            errors.push(format!("{name}: {error}"));
            Value::Null
        }
    }
}

fn inclusive_usage(participants: &[&Value]) -> Option<u64> {
    if participants.is_empty() {
        return None;
    }
    participants.iter().try_fold(0u64, |sum, participant| {
        let usage = &participant["usage"];
        let input = usage["inputTokens"].as_u64()?;
        let output = usage["outputTokens"].as_u64()?;
        let total = usage["totalTokens"].as_u64()?;
        (input.checked_add(output)? == total).then_some(())?;
        sum.checked_add(total)
    })
}

fn archived_path(root: &Path, name: &str) -> Result<PathBuf> {
    let relative = Path::new(name);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("nonlocal source path".into());
    }
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root.canonicalize()?) {
        return Err("source path escapes the evidence archive".into());
    }
    Ok(path)
}

fn recorded_audit(root: &Path, reference: &Value, row: &mut Map<String, Value>) -> Result<Value> {
    let audit = &reference["audit"];
    let name = audit["path"].as_str().ok_or("no saved audit selected")?;
    let pointer = audit["pointer"].as_str().ok_or("audit JSON pointer")?;
    let file = archived_path(root, name)?;
    let hash = digest(&file)?;
    if audit["sha256"] != hash {
        return Err("selected audit hash differs".into());
    }
    let value = load(&file)?;
    let value = value
        .pointer(pointer)
        .ok_or("selected audit pointer is absent")?;
    if !value.is_object() {
        return Err("selected audit is not an object".into());
    }
    row.insert("audit_sha256".to_owned(), json!(hash));
    row.insert("saved_audit_path".to_owned(), json!(name));
    row.insert("saved_audit_pointer".to_owned(), json!(pointer));
    Ok(value.clone())
}

fn inventory((input, path): (&Path, &Path), result: &Value, reference: &Value) -> Result<Value> {
    let root = path.parent().ok_or("result parent")?;
    let outcomes = result["outcomes"].as_array().ok_or("model outcomes")?;
    let participants: Vec<_> = outcomes
        .iter()
        .filter(|r| r["kind"] == "model_participant")
        .map(|r| &r["result"])
        .collect();
    let guests: Vec<_> = outcomes
        .iter()
        .filter(|r| r["kind"] == "guest_runtime")
        .map(|r| &r["result"])
        .collect();
    let mut row = json!({"task":result["task"],"depth":result["depth"],"path":root,
        "elapsed_seconds":result["elapsed_seconds"],"participant_count":participants.len(),
        "scored_benchmark":result["scored_benchmark"],"run_error":result["error"],
        "result_sha256":digest(path)?,"historical_auditors_rerun":false});
    let mut errors = Vec::new();
    let object = row.as_object_mut().ok_or("inventory row")?;
    if reference["result_sha256"] != object["result_sha256"] {
        errors.push("result bytes differ from the declared evidence reference".to_owned());
    }
    let versions = auxiliary(root, "versions.json", object, &mut errors);
    let audit = match recorded_audit(input, reference, object) {
        Ok(audit) => audit,
        Err(error) => {
            errors.push(format!("saved audit: {error}"));
            Value::Null
        }
    };
    let fixture = auxiliary(root, "simulator-fixture.json", object, &mut errors);
    for (name, value) in [
        ("runtime_binary_sha256", &versions["binaries"]["runtime"]),
        (
            "runtime_source_manifest_sha256",
            &versions["runtime_source_manifest_sha256"],
        ),
        ("model", &versions["model_connection"]["model"]),
        ("effort", &versions["model_connection"]["effort"]),
        ("simulator_contract", &fixture["simulator_contract"]),
        ("simulator_seed", &fixture["seed"]),
        ("delay_multiplier", &fixture["delay_multiplier"]),
        ("failure_overrides", &fixture["failure_overrides"]),
        (
            "simulator_manifest_sha256",
            &fixture["simulator_manifest_sha256"],
        ),
        ("saved_audit_status", &audit["status"]),
        ("saved_auditor_sha256", &audit["auditor_sha256"]),
        ("saved_effect_audit_status", &audit["protocol"]["status"]),
        (
            "saved_dynamic_audit_status",
            &audit["protocol"]["dynamic_protocol"]["status"],
        ),
        (
            "captured_records",
            &audit["protocol"]["dynamic_protocol"]["captured_records"],
        ),
        (
            "max_simultaneous_model_turns",
            &audit["max_simultaneous_model_turns"],
        ),
        (
            "independent_effect_overlaps",
            &audit["protocol"]["independent_effect_overlaps"],
        ),
    ] {
        object.insert(name.to_owned(), value.clone());
    }
    row["model_settings_match"] = json!(
        !participants.is_empty()
            && participants.iter().all(|p| {
                p["requested_model"] == "gpt-6-luna"
                    && p["returned_model"] == "gpt-6-luna"
                    && versions["model_connection"]["model"] == "gpt-6-luna"
                    && p["requested_effort"] == "xhigh"
                    && p["returned_effort"] == "xhigh"
                    && versions["model_connection"]["effort"] == "xhigh"
            })
    );
    row["model_participants_completed"] = json!(
        !participants.is_empty()
            && participants
                .iter()
                .all(|p| p["status"] == "completed" && p["error"].is_null())
    );
    let tokens = inclusive_usage(&participants);
    row["inclusive_tokens"] = json!(tokens);
    match tokens {
        Some(total) if !audit.is_null() && audit["inclusive_tokens"] != total => {
            errors.push("saved audit token total differs from participant totals".to_owned())
        }
        None => {
            errors.push("missing, invalid or overflowing inclusive participant usage".to_owned())
        }
        _ => {}
    }
    if let [guest] = guests.as_slice() {
        row["world_complete"] = guest["score"]["complete"].clone();
        row["world_violations"] = json!(guest["score"]["violations"].as_array().map(Vec::len));
        row["domain_calls"] = json!(guest["score"]["events"].as_array().map(Vec::len));
        row["guest_runtime_completed"] = json!(
            guest["runtime_exit_code"] == 0
                && guest["protocol"]["status"] == "complete"
                && guest["error"].is_null()
                && guest["backend_errors"] == json!([])
        );
    } else {
        errors.push("expected exactly one guest-runtime report".to_owned());
        row["guest_runtime_completed"] = json!(false);
    }
    if result["versions"] != versions {
        errors.push("embedded and saved versions differ".to_owned());
    }
    let binaries = versions["binaries"].as_object();
    let names: BTreeSet<_> = binaries
        .into_iter()
        .flat_map(|map| map.keys().map(String::as_str))
        .collect();
    if names != BTreeSet::from(["runner", "runtime", "wash.wasm"]) {
        errors.push("archived binary inventory is incomplete or unexpected".to_owned());
    }
    for (name, expected) in binaries.into_iter().flatten() {
        if !["runner", "runtime", "wash.wasm"].contains(&name.as_str()) {
            continue;
        }
        if !matches!((expected.as_str(), digest(&root.join(name))),
            (Some(expected), Ok(actual)) if expected == actual)
        {
            errors.push(format!("{name}: archived binary missing or hash differs"));
        }
    }
    for (name, expected) in [
        ("events.jsonl", &result["events_sha256"]),
        ("projection.json", &versions["compiled_projection_sha256"]),
        ("source-manifest.json", &versions["source_manifest_sha256"]),
        (
            "source/benchmark/simulator-manifest.json",
            &fixture["simulator_manifest_sha256"],
        ),
    ] {
        match digest(&root.join(name)) {
            Ok(actual) => {
                if expected.as_str() != Some(actual.as_str()) {
                    errors.push(format!("{name}: recorded hash absent or different"));
                }
                if name == "events.jsonl" {
                    row["events_sha256"] = json!(actual);
                    if !audit.is_null() && audit["events_sha256"] != actual {
                        errors.push("saved audit refers to other raw events".to_owned());
                    }
                }
            }
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    if !audit["protocol"].is_null() {
        let runtime = &audit["protocol"]["runtime_binary_sha256"];
        if runtime.is_null() || runtime != &row["runtime_binary_sha256"] {
            errors.push("saved effect audit does not bind the archived executable".to_owned());
        }
    }
    if !audit.is_null() && audit["task"] != result["task"] {
        errors.push("saved audit refers to another task".to_owned());
    }
    let mut failures = Vec::new();
    match load(&root.join("source-manifest.json")) {
        Ok(Value::Object(manifest)) if !manifest.is_empty() => {
            row["archived_source_files_checked"] = json!(manifest.len());
            for (name, expected) in manifest {
                match archived_path(root, &name).and_then(|file| digest(&file)) {
                    Ok(actual) if expected.as_str() == Some(actual.as_str()) => {}
                    Ok(_) => failures.push(json!({"path":name,"error":"source bytes differ"})),
                    Err(error) => failures.push(json!({"path":name,"error":error.to_string()})),
                }
            }
        }
        _ => errors.push("nonempty archived source hash map is unavailable".to_owned()),
    }
    if !failures.is_empty() {
        errors.push(format!(
            "{} archived source files unavailable or different",
            failures.len()
        ));
    }
    row["archived_source_failures"] = json!(failures);
    row["artifact_hashes_agree"] = json!(errors.is_empty());
    row["integrity_errors"] = json!(errors);
    Ok(row)
}

const FIELDS: &[&str] = &[
    "task",
    "depth",
    "elapsed_seconds",
    "inclusive_tokens",
    "participant_count",
    "domain_calls",
    "world_complete",
    "world_violations",
    "simulator_seed",
    "delay_multiplier",
    "simulator_contract",
    "simulator_manifest_sha256",
    "runtime_binary_sha256",
    "model",
    "effort",
    "saved_audit_status",
    "saved_dynamic_audit_status",
    "saved_audit_path",
    "saved_audit_pointer",
    "artifact_hashes_agree",
    "same_raw_events_as",
    "path",
];

fn cell(value: &Value) -> String {
    value.as_str().map_or_else(
        || {
            if value.is_null() {
                String::new()
            } else {
                value.to_string()
            }
        },
        str::to_owned,
    )
}

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let [input, references, output] = args.as_slice() else {
        return Err(
            "usage: choreofix-pilot-inventory INPUT_ROOT AUDIT_REFERENCES NEW_OUTPUT".into(),
        );
    };
    let input = Path::new(input);
    let references_path = Path::new(references);
    let references = load(references_path)?;
    let references = references.as_object().ok_or("audit reference map")?;
    let output = Path::new(output);
    let mut paths = Vec::new();
    let mut skipped = Vec::new();
    candidates(input, &mut paths, &mut skipped)?;
    paths.sort();
    let candidate_count = paths.len();
    let mut rows = Vec::new();
    let mut malformed = Vec::new();
    let mut used_references = BTreeSet::new();
    for path in paths {
        match load(&path) {
            Ok(result)
                if result["outcomes"].as_array().is_some_and(|outcomes| {
                    outcomes
                        .iter()
                        .any(|row| row["kind"] == "model_participant")
                }) =>
            {
                let name = path
                    .strip_prefix(input)?
                    .to_str()
                    .ok_or("result reference path")?;
                let reference = references.get(name).ok_or_else(|| {
                    format!("model result has no declared evidence reference: {name}")
                })?;
                used_references.insert(name.to_owned());
                match inventory((input, &path), &result, reference) {
                    Ok(row) => rows.push(row),
                    Err(error) => malformed.push(json!({"path":path,"error":error.to_string()})),
                }
            }
            Ok(_) => {}
            Err(error) => malformed.push(json!({"path":path,"error":error.to_string()})),
        }
    }
    if rows.is_empty() {
        return Err("no model pilot records found; no inventory was certified".into());
    }
    if references.len() != used_references.len() {
        return Err("unused or missing model evidence references".into());
    }
    rows.sort_by_key(|row| (cell(&row["task"]), cell(&row["depth"]), cell(&row["path"])));
    let mut unique = BTreeMap::new();
    for row in &mut rows {
        if let Some(events) = row["events_sha256"].as_str() {
            let key = events.to_owned();
            if let Some(first) = unique.get(&key) {
                row["same_raw_events_as"] = json!(first);
            } else {
                unique.insert(key, cell(&row["path"]));
            }
        }
    }
    let valid = rows
        .iter()
        .filter(|row| row["artifact_hashes_agree"] == true)
        .count();
    let tasks: BTreeSet<_> = rows.iter().filter_map(|row| row["task"].as_str()).collect();
    fs::create_dir_all(output.parent().ok_or("output parent")?)?;
    fs::create_dir(output)?;
    let metadata = json!({"scope":"local pilot inventory; saved verdicts plus hash/usage checks, not rerun certification",
        "source_sha256":format!("{:x}",Sha256::digest(include_bytes!("pilot_inventory.rs"))),
        "cargo_manifest_sha256":format!("{:x}",Sha256::digest(include_bytes!("Cargo.toml"))),
        "cargo_lock_sha256":format!("{:x}",Sha256::digest(include_bytes!("Cargo.lock"))),
        "inventory_binary_sha256":digest(&std::env::current_exe()?)?,
        "audit_references_sha256":digest(references_path)?,
        "candidate_results":candidate_count,"pilot_records":rows.len(),"distinct_raw_event_streams":unique.len(),
        "pilots":rows,"malformed_candidates":malformed,"skipped_symlinks":skipped,
        "statistical_pools_created":false,"comparative_superiority_established":false,"scored_comparisons_counted":0});
    fs::write(
        output.join("inventory.json"),
        format!("{}\n", serde_json::to_string_pretty(&metadata)?),
    )?;
    let tool_source = output.join("source");
    fs::create_dir(&tool_source)?;
    fs::write(
        tool_source.join("pilot_inventory.rs"),
        include_bytes!("pilot_inventory.rs"),
    )?;
    fs::write(tool_source.join("Cargo.toml"), include_bytes!("Cargo.toml"))?;
    fs::write(tool_source.join("Cargo.lock"), include_bytes!("Cargo.lock"))?;
    fs::copy(references_path, tool_source.join("audit-references.json"))?;
    let mut csv = File::create(output.join("pilots.csv"))?;
    writeln!(csv, "{}", FIELDS.join(","))?;
    for row in &rows {
        let cells: Vec<_> = FIELDS
            .iter()
            .map(|field| format!("\"{}\"", cell(&row[field]).replace('"', "\"\"")))
            .collect();
        writeln!(csv, "{}", cells.join(","))?;
    }
    let mut report = File::create(output.join("inventory.md"))?;
    writeln!(
        report,
        "# Source-bound pilot inventory\n\nIndividual pilot observations; scorer and source versions remain separate. Saved verdicts are not rerun certification. Hash integrity does not establish semantic correctness or comparative superiority. Duplicate raw streams are identified, not treated as independent experiments. Inclusive tokens are input plus output; reasoning is not added again.\n"
    )?;
    writeln!(
        report,
        "| Task | Depth | Seconds | Inclusive tokens | Participants | Effects | World complete | Saved audit / dynamic | Hashes agree |\n|---|---:|---:|---:|---:|---:|---|---|---|"
    )?;
    for row in &rows {
        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} | {} | {} / {} | {} |",
            cell(&row["task"]),
            cell(&row["depth"]),
            cell(&row["elapsed_seconds"]),
            cell(&row["inclusive_tokens"]),
            cell(&row["participant_count"]),
            cell(&row["domain_calls"]),
            cell(&row["world_complete"]),
            cell(&row["saved_audit_status"]),
            cell(&row["saved_dynamic_audit_status"]),
            cell(&row["artifact_hashes_agree"])
        )?;
    }
    println!(
        "{}",
        json!({"pilot_records":rows.len(),"hashes_agree":valid,"distinct_raw_event_streams":unique.len(),
        "tasks":tasks,"scored_comparisons_counted":0})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_audit_reference_binds_the_file_and_exact_json_object() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-audit-reference-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let file = root.join("report.json");
        let baseline = json!({"task":"11E","status":"passed"});
        fs::write(&file, json!({"baseline":baseline,"count":1}).to_string()).unwrap();
        let reference = json!({"audit":{"path":"report.json","pointer":"/baseline",
            "sha256":digest(&file).unwrap()}});
        let mut row = Map::new();
        assert_eq!(
            recorded_audit(&root, &reference, &mut row).unwrap(),
            baseline
        );
        assert_eq!(row["saved_audit_pointer"], "/baseline");
        for changed in [
            json!({"audit":{"path":"report.json","pointer":"/baseline","sha256":"changed"}}),
            json!({"audit":{"path":"report.json","pointer":"/missing","sha256":digest(&file).unwrap()}}),
            json!({"audit":{"path":"report.json","pointer":"/count","sha256":digest(&file).unwrap()}}),
            json!({"audit":{"path":"../report.json","pointer":"/baseline","sha256":digest(&file).unwrap()}}),
            json!({"audit":null}),
        ] {
            assert!(recorded_audit(&root, &changed, &mut Map::new()).is_err());
        }
        fs::remove_file(file).unwrap();
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn inclusive_usage_counts_reasoning_once_and_rejects_missing_or_invalid_totals() {
        let first = json!({"usage":{"inputTokens":100,"outputTokens":40,"totalTokens":140,"reasoningTokens":30}});
        let second = json!({"usage":{"inputTokens":200,"outputTokens":60,"totalTokens":260,"reasoningTokens":40}});
        assert_eq!(inclusive_usage(&[&first, &second]), Some(400));
        assert_eq!(inclusive_usage(&[]), None);
        for usage in [
            json!({}),
            json!({"inputTokens":true,"outputTokens":40,"totalTokens":41}),
            json!({"inputTokens":100,"outputTokens":40,"totalTokens":139}),
            json!({"inputTokens":u64::MAX,"outputTokens":1,"totalTokens":0}),
        ] {
            assert_eq!(inclusive_usage(&[&json!({"usage":usage})]), None);
        }
        let maximum =
            json!({"usage":{"inputTokens":u64::MAX,"outputTokens":0,"totalTokens":u64::MAX}});
        let one = json!({"usage":{"inputTokens":1,"outputTokens":0,"totalTokens":1}});
        assert_eq!(inclusive_usage(&[&maximum, &one]), None);
    }
}
