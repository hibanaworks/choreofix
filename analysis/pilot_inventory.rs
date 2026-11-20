//! Source-bound pilot evidence, without rerunning or upgrading saved verdicts.
mod effect_overlap;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
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

fn simulator_score(path: &Path) -> Result<(String, Option<Value>)> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut line = Vec::new();
    let mut hash = Sha256::new();
    let mut score = None;
    while reader.read_until(b'\n', &mut line)? != 0 {
        hash.update(&line);
        let event: Value = serde_json::from_slice(&line)?;
        if event["direction"] == "simulator" && event["message"]["type"] == "score" {
            let message = &event["message"];
            if score.is_some() {
                return Err("multiple simulator scores in the same trace".into());
            }
            if !message["complete"].is_boolean()
                || !message["events"].is_array()
                || !message["violations"].is_array()
            {
                return Err("incomplete simulator score".into());
            }
            score = Some(message.clone());
        }
        line.clear();
    }
    Ok((format!("{:x}", hash.finalize()), score))
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
        } else if kind.is_file() {
            let name = entry.file_name();
            if name == "result.json"
                || (name == "harness-error.json" && !root.join("result.json").try_exists()?)
                || (name == "simulator-fixture.json"
                    && !root.join("result.json").try_exists()?
                    && !root.join("harness-error.json").try_exists()?
                    && root
                        .join("model-connection")
                        .symlink_metadata()
                        .is_ok_and(|m| m.is_dir())
                    && root
                        .join("model-connection/events.jsonl")
                        .symlink_metadata()
                        .is_ok_and(|m| m.is_file()))
            {
                found.push(entry.path());
            }
        }
    }
    Ok(())
}

// Missing aggregate reports do not supply a verdict, token total, or model-call count.
fn unreported(path: &Path, record: &Value) -> Result<Value> {
    let root = path.parent().ok_or("attempt directory")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("attempt evidence filename")?;
    let reason = match name {
        "harness-error.json" if record["kind"] == "harness_error" => "harness_error",
        "simulator-fixture.json" if record["fixtures"].is_array() => "no_aggregate_result",
        "result.json" if record["kind"] == "unscored_direct_wasi_model_preflight" => {
            "no_participant_report"
        }
        _ => return Err("unexpected model-run evidence".into()),
    };
    let mut row = json!({"path":root, "reason":reason, "task":record["task"],
        "depth":record["depth"],"error":record["error"],"evidence_file":name,
        "evidence_sha256":digest(path)?});
    if name == "harness-error.json" && root.join("simulator-fixture.json").try_exists()? {
        let fixture = archived_path(root, "simulator-fixture.json")?;
        let input = load(&fixture)?;
        row["task"] = input["task"].clone();
        row["depth"] = input["depth"].clone();
        row["fixture_sha256"] = json!(digest(&fixture)?);
    }
    for (name, field) in [
        ("events.jsonl", "events_sha256"),
        ("model-connection/events.jsonl", "connection_trace_sha256"),
    ] {
        if root.join(name).try_exists()? {
            row[field] = json!(digest(&archived_path(root, name)?)?);
        }
    }
    Ok(row)
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

fn all_participants_reported(fixture: &Value, reports: &[&Value]) -> bool {
    let Some(agents) = fixture["fixtures"].as_array() else {
        return false;
    };
    let expected: BTreeSet<_> = agents
        .iter()
        .filter_map(|actor| actor["agent"].as_str())
        .filter(|name| !name.is_empty())
        .collect();
    let reported: BTreeSet<_> = reports
        .iter()
        .filter_map(|report| report["participant"].as_str())
        .collect();
    !expected.is_empty()
        && expected.len() == agents.len()
        && expected.len() == reports.len()
        && expected == reported
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
    let protocol = &audit["protocol"];
    let dynamic = protocol
        .get("dynamic_protocol")
        .unwrap_or(&protocol["dynamic"]);
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
        ("saved_dynamic_audit_status", &dynamic["status"]),
        (
            "captured_records",
            dynamic
                .get("captured_records")
                .unwrap_or(&protocol["captured_records"]),
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
    let all_reported = all_participants_reported(&fixture, &participants);
    if !all_reported {
        errors.push("participant reports differ from the simulator roster".to_owned());
    }
    let model_completed = all_reported
        && participants
            .iter()
            .all(|p| p["status"] == "completed" && p["error"].is_null());
    row["model_participants_completed"] = json!(model_completed);
    let tokens = if all_reported {
        inclusive_usage(&participants)
    } else {
        None
    };
    row["inclusive_tokens"] = json!(tokens.filter(|_| model_completed));
    row["interrupted_reported_tokens"] = json!(tokens.filter(|_| !model_completed));
    match tokens {
        Some(total) if !audit.is_null() && audit["inclusive_tokens"] != total => {
            errors.push("saved audit token total differs from participant totals".to_owned())
        }
        None => {
            errors.push("missing, invalid or overflowing inclusive participant usage".to_owned())
        }
        _ => {}
    }
    let mut score = None;
    if let [guest] = guests.as_slice() {
        if let Some(value) = guest.get("score").filter(|value| value.is_object()) {
            score = Some((value.clone(), "guest_runtime"));
        }
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
        let file = root.join(name);
        let evidence = if name == "events.jsonl" && guests.is_empty() {
            simulator_score(&file)
        } else {
            digest(&file).map(|hash| (hash, None))
        };
        match evidence {
            Ok((actual, observed)) => {
                if expected.as_str() != Some(actual.as_str()) {
                    errors.push(format!("{name}: recorded hash absent or different"));
                }
                if name == "events.jsonl" {
                    row["events_sha256"] = json!(actual);
                    if !audit.is_null() && audit["events_sha256"] != actual {
                        errors.push("saved audit refers to other raw events".to_owned());
                    } else if let Some(value) = observed
                        && reference["result_sha256"] == row["result_sha256"]
                        && expected.as_str() == Some(actual.as_str())
                    {
                        score = Some((value, "simulator_trace"));
                    }
                }
            }
            Err(error) => {
                errors.push(format!("{name}: {error}"));
                if name == "events.jsonl"
                    && guests.is_empty()
                    && let Ok(hash) = digest(&file)
                {
                    row["events_sha256"] = json!(hash);
                }
            }
        }
    }
    if let Some((value, source)) = score {
        row["world_complete"] = value["complete"].clone();
        row["world_violations"] = json!(value["violations"].as_array().map(Vec::len));
        row["domain_calls"] = json!(value["events"].as_array().map(Vec::len));
        row["world_score_source"] = json!(source);
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
    "interrupted_reported_tokens",
    "participant_count",
    "domain_calls",
    "world_complete",
    "world_violations",
    "world_score_source",
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
    if let [mode, cohort, output] = args.as_slice()
        && mode == "--effect-overlap"
    {
        return effect_overlap::write_report(Path::new(cohort), Path::new(output));
    }
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
    let candidate_count = paths.iter().filter(|p| p.ends_with("result.json")).count();
    let mut rows = Vec::new();
    let mut unreported_attempts = Vec::new();
    let mut malformed = Vec::new();
    let mut used_references = BTreeSet::new();
    for path in paths {
        match load(&path) {
            Ok(result)
                if path.ends_with("result.json")
                    && result["outcomes"].as_array().is_some_and(|outcomes| {
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
            Ok(record)
                if !path.ends_with("result.json")
                    || record["kind"] == "unscored_direct_wasi_model_preflight" =>
            {
                match unreported(&path, &record) {
                    Ok(row) => unreported_attempts.push(row),
                    Err(error) => malformed.push(json!({"path":path,"error":error.to_string()})),
                }
            }
            Ok(_) => {}
            Err(error) => malformed.push(json!({"path":path,"error":error.to_string()})),
        }
    }
    if rows.is_empty() && unreported_attempts.is_empty() {
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
        "pilots":rows,"unreported_attempts":unreported_attempts,
        "malformed_candidates":malformed,"skipped_symlinks":skipped,
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
        "| Task | Depth | Seconds | Inclusive tokens | Last reported at interruption | Participants | Effects | World complete | Score source | Saved audit / dynamic | Hashes agree |\n|---|---:|---:|---:|---:|---:|---:|---|---|---|---|"
    )?;
    for row in &rows {
        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} / {} | {} |",
            cell(&row["task"]),
            cell(&row["depth"]),
            cell(&row["elapsed_seconds"]),
            cell(&row["inclusive_tokens"]),
            cell(&row["interrupted_reported_tokens"]),
            cell(&row["participant_count"]),
            cell(&row["domain_calls"]),
            cell(&row["world_complete"]),
            cell(&row["world_score_source"]),
            cell(&row["saved_audit_status"]),
            cell(&row["saved_dynamic_audit_status"]),
            cell(&row["artifact_hashes_agree"])
        )?;
    }
    if !unreported_attempts.is_empty() {
        writeln!(report, "\n## Attempts without an aggregate model report\n")?;
        writeln!(
            report,
            "| Run | Task | Depth | Recorded status |\n|---|---|---|---|"
        )?;
        for row in &unreported_attempts {
            writeln!(
                report,
                "| {} | {} | {} | {} |",
                cell(&row["path"]),
                cell(&row["task"]),
                cell(&row["depth"]),
                cell(&row["reason"])
            )?;
        }
    }
    println!(
        "{}",
        json!({"pilot_records":rows.len(),"hashes_agree":valid,"distinct_raw_event_streams":unique.len(),
        "unreported_attempts":unreported_attempts.len(),"tasks":tasks,"scored_comparisons_counted":0})
    );
    if rows.is_empty() {
        return Err(
            "attempts recorded, but no aggregate participant results were available".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_live_trial_is_counted_once_and_mechanism_fixture_is_excluded() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("choreofix-incomplete-{}", std::process::id()));
        fs::create_dir(&root)?;
        let fixture = root.join("simulator-fixture.json");
        fs::write(&fixture, r#"{"task":"7H","depth":0,"fixtures":[]}"#)?;
        let scan = || -> Result<Vec<PathBuf>> {
            let mut found = Vec::new();
            candidates(&root, &mut found, &mut Vec::new())?;
            Ok(found)
        };
        assert!(scan()?.is_empty());
        fs::create_dir(root.join("model-connection"))?;
        fs::write(root.join("model-connection/events.jsonl"), b"")?;
        assert_eq!(scan()?.as_slice(), std::slice::from_ref(&fixture));
        let missing = unreported(&fixture, &load(&fixture)?)?;
        assert_eq!(missing["reason"], "no_aggregate_result");
        assert!(missing.get("inclusive_tokens").is_none());
        let error = root.join("harness-error.json");
        fs::write(
            &error,
            r#"{"kind":"harness_error","error":"connection closed"}"#,
        )?;
        assert_eq!(scan()?.as_slice(), std::slice::from_ref(&error));
        let failed = unreported(&error, &load(&error)?)?;
        assert_eq!(failed["task"], "7H");
        assert_eq!(failed["depth"], 0);
        assert_eq!(failed["error"], "connection closed");
        let result = root.join("result.json");
        fs::write(
            &result,
            r#"{"kind":"unscored_direct_wasi_model_preflight","outcomes":[]}"#,
        )?;
        assert_eq!(scan()?.as_slice(), std::slice::from_ref(&result));
        assert_eq!(
            unreported(&result, &load(&result)?)?["reason"],
            "no_participant_report"
        );
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    #[test]
    fn trial_totals_require_each_expected_participant_exactly_once() {
        let fixture = json!({"fixtures":[{"agent":"A"},{"agent":"B"}]});
        let a = json!({"participant":"A"});
        let b = json!({"participant":"B"});
        assert!(all_participants_reported(&fixture, &[&b, &a]));
        assert!(!all_participants_reported(&fixture, &[&a]));
        assert!(!all_participants_reported(&fixture, &[&a, &a]));
        assert!(!all_participants_reported(
            &fixture,
            &[&a, &json!({"participant":"C"})]
        ));
        assert!(!all_participants_reported(&json!({"fixtures":[]}), &[]));
    }

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
    fn a_timeout_keeps_last_reported_usage_separate_from_a_final_total() -> Result<()> {
        let root = std::env::temp_dir().join(format!("choreofix-usage-{}", std::process::id()));
        fs::create_dir(&root)?;
        fs::write(
            root.join("simulator-fixture.json"),
            r#"{"fixtures":[{"agent":"WRITER"},{"agent":"EDITOR"}]}"#,
        )?;
        let path = root.join("result.json");
        for (status, final_total, interrupted_total) in [
            ("timeout", Value::Null, json!(280)),
            ("completed", json!(280), Value::Null),
        ] {
            let mut result = json!({"outcomes":[{"kind":"guest_runtime",
                "result":{"score":{"complete":false}}}]});
            for (agent, status) in [("WRITER", "completed"), ("EDITOR", status)] {
                result["outcomes"].as_array_mut().unwrap().push(json!({
                    "kind":"model_participant","result":{"participant":agent,"status":status,
                    "usage":{"inputTokens":100,"outputTokens":40,"totalTokens":140}}
                }));
            }
            fs::write(&path, serde_json::to_vec(&result)?)?;
            let row = inventory((&root, &path), &result, &Value::Null)?;
            assert_eq!(row["world_complete"], false);
            assert_eq!(row["inclusive_tokens"], final_total);
            assert_eq!(row["interrupted_reported_tokens"], interrupted_total);
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn interrupted_trial_preserves_only_a_unique_hash_bound_simulator_score() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("choreofix-trace-score-{}", std::process::id()));
        fs::create_dir(&root)?;
        let path = root.join("result.json");
        let trace = root.join("events.jsonl");
        let score = json!({"type":"score","complete":false,"events":[{}],"violations":[]});
        let observed = json!({"direction":"simulator","message":score}).to_string();
        let unrelated = json!({"direction":"to_runtime","message":score}).to_string();
        for (content, alter_hash, has_score) in [
            (format!("{unrelated}\n{observed}\n"), false, true),
            (unrelated, false, false),
            (format!("{observed}\n{observed}\n"), false, false),
            (observed.clone(), true, false),
            (format!("{observed}\n{{"), false, false),
        ] {
            fs::write(&trace, content)?;
            let mut result = json!({"outcomes":[],"events_sha256":digest(&trace)?});
            if alter_hash {
                result["events_sha256"] = json!("wrong hash");
            }
            fs::write(&path, serde_json::to_vec(&result)?)?;
            let reference = json!({"result_sha256":digest(&path)?,"audit":null});
            let row = inventory((&root, &path), &result, &reference)?;
            assert_eq!(row["guest_runtime_completed"], false);
            assert_eq!(row["artifact_hashes_agree"], false);
            if has_score {
                assert_eq!(row["world_complete"], false);
                assert_eq!(row["world_violations"], 0);
                assert_eq!(row["domain_calls"], 1);
                assert_eq!(row["world_score_source"], "simulator_trace");
            } else {
                assert!(row["world_complete"].is_null());
                assert!(row["world_score_source"].is_null());
            }
        }
        fs::remove_dir_all(root)?;
        Ok(())
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
