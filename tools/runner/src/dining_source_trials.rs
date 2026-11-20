//! Paper experiment over the upstream dining model's complete visible traces.
use crate::{
    evidence::{Result, digest, save},
    philosophers_boundary::verify_action_words,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[hibana_runtime::main]
#[test]
#[ignore = "requires the upstream TLC state dump and the built Wasm dining runtime"]
async fn upstream_visible_actions() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let model = PathBuf::from(std::env::var_os("CHOREOFIX_DINING_TLC").ok_or("TLC directory")?)
        .canonicalize()?;
    for (name, original) in [
        (
            "Protocol.tla",
            "vendor/tracefix-paper/v3_agent_test/tests/fixtures/9E/Protocol_translated.tla",
        ),
        ("dining_orders.tla", "analysis/dining_orders.tla"),
        ("dining_actions.cfg", "analysis/dining_actions.cfg"),
    ] {
        if digest(&model.join(name))? != digest(&root.join(original))? {
            return Err(format!("TLC input differs from {original}").into());
        }
    }
    let dump = model.join("source-states.txt.dump");
    let mut words = BTreeSet::new();
    for state in fs::read_to_string(&dump)?.split("\n\n") {
        if !state
            .lines()
            .any(|line| line.starts_with("/\\ pc = ") && line.matches("\"Done\"").count() == 3)
        {
            continue;
        }
        let history = state
            .lines()
            .find_map(|line| line.strip_prefix("/\\ actions = <<")?.strip_suffix(">>"))
            .ok_or("terminal state without an action history")?;
        let actions: Vec<char> = serde_json::from_str(&format!("[{history}]"))?;
        let word: String = actions.iter().collect();
        if actions.len() != 6
            || !['a', 'A', 'b', 'B', 'c', 'C']
                .iter()
                .all(|a| actions.contains(a))
        {
            return Err(format!("unexpected terminal history {word}").into());
        }
        words.insert(word);
    }
    assert_eq!(
        words.len(),
        90,
        "number of distinct upstream terminal histories"
    );
    verify_action_words(
        "9E",
        "upstream-actions",
        words.iter().map(|word| (word.as_str(), word.as_str())),
    )
    .await?;
    let evidence =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("evidence root")?);
    save(
        &evidence.join("upstream-actions/result.json"),
        &serde_json::json!({
            "source_states_sha256": digest(&dump)?, "visible_actions": words,
            "completed_and_audited": words.len(), "model_calls":0, "scored_benchmark":false
        }),
    )
}
