//! Independent Luna conversations over one App Server connection.
use crate::{
    evidence::{Result, Trace, digest, save},
    process::Guest,
    wire::Wire,
};
use hibana_runtime::{
    host::{Io, Process, deadline},
    race,
};
use serde_json::{Value, json};
use std::{
    fs, io,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

mod driver;
#[cfg(test)]
mod tests;

pub const MODEL: &str = "gpt-6-luna";
pub const EFFORT: &str = "xhigh";
pub const CLI: &str = "/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex";

pub(crate) fn same_tool_call(request: &Value, completed: &Value) -> bool {
    let item = &completed["item"];
    request["callId"].as_str().is_some_and(|id| !id.is_empty())
        && request["threadId"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
        && request["turnId"].as_str().is_some_and(|id| !id.is_empty())
        && item["id"] == request["callId"]
        && completed["threadId"] == request["threadId"]
        && completed["turnId"] == request["turnId"]
        && item["tool"] == request["tool"]
        && item["namespace"] == request["namespace"]
        && item["arguments"] == request["arguments"]
}

pub(crate) fn cancelled_tool(completed: &Value) -> bool {
    let item = &completed["item"];
    item["status"] == "failed"
        && item["success"] == false
        && item["contentItems"].as_array().is_some_and(Vec::is_empty)
}

/// The server reports the contents it delivered to this particular tool call.
pub(crate) fn check_tool_completion(
    request: &Value,
    sent: &Value,
    completed: &Value,
) -> Result<()> {
    let item = &completed["item"];
    let success = sent["success"]
        .as_bool()
        .ok_or("shell reply lacks a success value")?;
    if !same_tool_call(request, completed)
        || item["status"] != if success { "completed" } else { "failed" }
        || item["success"] != success
        || !sent["contentItems"].is_array()
        || item["contentItems"] != sent["contentItems"]
    {
        return Err(format!(
            "model did not acknowledge the shell reply for {}",
            request["callId"]
        )
        .into());
    }
    Ok(())
}

pub(crate) fn check_model_item(message: &Value) -> Result<()> {
    let item = &message["params"]["item"];
    match item["type"].as_str() {
        None | Some("dynamicToolCall" | "reasoning" | "userMessage" | "agentMessage") => Ok(()),
        Some("sleep")
            if message.get("id").is_none()
                && item.as_object().is_some_and(|fields| {
                    fields.len() == 5 && fields.contains_key("completedAtMs")
                })
                && item["id"].is_string()
                && item["durationMs"].as_u64().is_some_and(|ms| ms > 0)
                && item["startedAtMs"]
                    .as_u64()
                    .is_some_and(|started| match message["method"].as_str() {
                        Some("item/started") => item["completedAtMs"].is_null(),
                        Some("item/completed") => item["completedAtMs"]
                            .as_u64()
                            .is_some_and(|completed| completed >= started),
                        _ => false,
                    }) =>
        {
            Ok(())
        }
        Some(kind) => Err(format!("unexpected native model item: {kind}").into()),
    }
}

pub fn identity() -> Result<Value> {
    let version = std::process::Command::new(CLI).arg("--version").output()?;
    if !version.status.success() {
        return Err("model client version query failed".into());
    }
    let version = String::from_utf8(version.stdout)?.trim().to_owned();
    if version.is_empty() {
        return Err("model client returned an empty version".into());
    }
    Ok(json!({
        "model":MODEL,"effort":EFFORT,
        "client_version":version,
        "client_sha256":digest(std::path::Path::new(CLI))?,
    }))
}

pub struct Participant {
    pub agent: String,
    pub instructions: String,
    pub output: PathBuf,
}

pub struct Limits {
    pub timeout: Duration,
    pub tool_calls: u64,
}

#[derive(Default)]
struct Observed {
    thread_id: Value,
    returned_model: Value,
    returned_effort: Value,
    usage: Value,
    tool_calls: u64,
}

struct Conversation {
    participant: Participant,
    trace: Trace,
    started: Instant,
    observed: Observed,
}

impl Conversation {
    fn finish(&self, turn: &Value) -> Result<Value> {
        let result = json!({
            "participant":self.participant.agent,"status":turn["status"],"error":turn["error"],
            "thread_id":self.observed.thread_id,"requested_model":MODEL,
            "authentication":"chatgpt","model_connection":"codex_app_server",
            "returned_model":self.observed.returned_model,"requested_effort":EFFORT,
            "returned_effort":self.observed.returned_effort,"usage":self.observed.usage,
            "tool_calls":self.observed.tool_calls,"elapsed_seconds":self.started.elapsed().as_secs_f64(),
            "raw_events_sha256":digest(&self.participant.output.join("events.jsonl"))?
        });
        save(&self.participant.output.join("participant.json"), &result)?;
        Ok(result)
    }
}

pub async fn run(
    people: Vec<Participant>,
    limits: Limits,
    guest: &Guest,
    guest_failure: &mut Io,
) -> Result<Vec<Value>> {
    let parent = people
        .first()
        .ok_or("empty model group")?
        .output
        .parent()
        .ok_or("model output parent")?
        .to_owned();
    if people
        .iter()
        .any(|p| p.output.parent() != Some(parent.as_path()))
    {
        return Err("model outputs must share one trial directory".into());
    }
    let directory = parent.join("model-connection");
    fs::create_dir(&directory)?;
    let mut conversations = Vec::new();
    for participant in people {
        fs::create_dir(&participant.output)?;
        conversations.push(Conversation {
            trace: Trace::create(&participant.output.join("events.jsonl"))?,
            participant,
            started: Instant::now(),
            observed: Observed::default(),
        });
    }
    let mut child = Process::spawn(
        Command::new(CLI)
            .args([
                "-c",
                "forced_login_method=\"chatgpt\"",
                "-c",
                "model_provider=\"openai\"",
                "app-server",
                "--listen",
                "stdio://",
            ])
            .env_remove("OPENAI_API_KEY")
            .env_remove("CODEX_API_KEY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create_new(directory.join("stderr.log"))?),
    )?;
    let (input, output) = child.io()?;
    let mut wire = Wire::new(
        input,
        output,
        Trace::create(&directory.join("events.jsonl"))?,
    );
    let mut completed = Vec::new();
    let outcome: io::Result<Result<()>> = race(
        async {
            Ok(driver::run(
                &mut wire,
                &mut conversations,
                &mut completed,
                limits.tool_calls,
                guest,
            )
            .await)
        },
        race(deadline(limits.timeout), async {
            guest_failure.read(&mut [0]).await?;
            Ok(Err("guest runtime failed; see guest-outcome.json".into()))
        }),
    )
    .await;
    let outcome = match child.terminate() {
        Ok(_) => outcome,
        Err(error) => Ok(Err(error.into())),
    };
    finish_reports(&conversations, completed, outcome)
}

fn finish_reports(
    conversations: &[Conversation],
    mut completed: Vec<Value>,
    outcome: io::Result<Result<()>>,
) -> Result<Vec<Value>> {
    let error = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(("harness_error", error.to_string())),
        Err(error) if error.kind() == io::ErrorKind::TimedOut => {
            Some(("timeout", "participant group timeout".to_owned()))
        }
        Err(error) => Some(("harness_error", error.to_string())),
    };
    if let Some((status, error)) = error {
        if conversations.iter().all(|conversation| {
            completed
                .iter()
                .any(|result| result["participant"] == conversation.participant.agent)
        }) {
            return Err(format!("model participant group {status}: {error}").into());
        }
        for conversation in conversations {
            if !completed
                .iter()
                .any(|result| result["participant"] == conversation.participant.agent)
            {
                completed.push(conversation.finish(&json!({"status":status,"error":error}))?);
            }
        }
    }
    Ok(completed)
}
