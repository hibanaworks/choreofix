//! JSON-RPC correlation and concurrent tool replies, without task authority.
use super::{
    Conversation, EFFORT, MODEL, cancelled_tool, check_model_item, check_tool_completion,
    same_tool_call,
};
use crate::{
    evidence::Result,
    process::Guest,
    wire::{self, Wire},
};
use hibana_runtime::{host::notification, race};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::mpsc};

enum Incoming {
    Frame(Value),
    Reply(Value, Result<Value>),
}

struct ToolCall {
    participant: usize,
    original: Value,
    reader: Option<Value>,
    response: Option<Value>,
}

impl ToolCall {
    fn reply(&self) -> Option<Value> {
        let reader = self.reader.as_ref()?;
        let mut result = self.response.clone()?;
        if reader["params"]["callId"] != self.original["params"]["callId"] {
            let origin = &self.original["params"];
            let receipt =
                json!({"original_call":origin["callId"],"command":origin["arguments"]["command"]});
            result["contentItems"]
                .as_array_mut()?
                .push(json!({"type":"inputText","text":receipt.to_string()}));
        }
        Some(json!({"id":reader["id"],"result":result}))
    }
}

pub(super) async fn run(
    wire: &mut Wire,
    people: &mut [Conversation],
    completed: &mut Vec<Value>,
    tool_limit: u64,
    guest: &Guest,
) -> Result<()> {
    wire.request(1,"initialize",json!({"clientInfo":{"name":"choreofix","title":"ChoreoFix","version":"0.1.0"},"capabilities":{"experimentalApi":true}})).await?;
    wire.send(&json!({"method":"initialized"})).await?;
    let mut threads = BTreeMap::new();
    for (index, person) in people.iter_mut().enumerate() {
        let id = 2u64
            .checked_add(u64::try_from(index)?)
            .ok_or("RPC id overflow")?;
        let request = json!({"id":id,"method":"thread/start","params":{
            "model":MODEL,"allowProviderModelFallback":false,
            "cwd":person.participant.output.canonicalize()?,"ephemeral":true,
            "environments":[],"sandbox":"read-only","approvalPolicy":"never",
            "baseInstructions":format!("{}\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response.",person.participant.instructions),
            "config":{"model_reasoning_effort":EFFORT,"features.shell_tool":false,"features.multi_agent":false,"project_doc_max_bytes":0},
            "dynamicTools":[{"type":"function","name":"shell","description":"Run a command in your WASI wash guest over ChoreoFS. An empty object {} retrieves the output of your oldest interrupted command, even after the guest has exited. After an interruption, repeat {} until no_interrupted_command; this never runs a guest command.","inputSchema":{"type":"object","properties":{"command":{"type":"string"}},"additionalProperties":false}}]
        }});
        person.trace.record("send", &request)?;
        let reply = wire
            .request(id, "thread/start", request["params"].clone())
            .await?;
        person
            .trace
            .record("receive", &json!({"id":id,"result":reply}))?;
        person.observed.returned_model = reply["model"].clone();
        person.observed.returned_effort = reply["reasoningEffort"].clone();
        if person.observed.returned_model != MODEL || person.observed.returned_effort != EFFORT {
            return Err("participant model or reasoning effort was substituted".into());
        }
        let thread = reply["thread"]["id"]
            .as_str()
            .ok_or("thread id missing")?
            .to_owned();
        person.observed.thread_id = json!(thread);
        if threads.insert(thread, index).is_some() {
            return Err("duplicate thread id".into());
        }
    }
    let mut starts = BTreeMap::new();
    let first_turn = 2u64
        .checked_add(u64::try_from(people.len())?)
        .ok_or("RPC id overflow")?;
    for (index, person) in people.iter().enumerate() {
        let id = first_turn
            .checked_add(u64::try_from(index)?)
            .ok_or("RPC id overflow")?;
        starts.insert(id, index);
        let request = json!({"id":id,"method":"turn/start","params":{
            "threadId":person.observed.thread_id,"model":MODEL,"effort":EFFORT,
            "input":[{"type":"text","text":"Begin your assigned work. Use the experiment tools to carry it out."}]
        }});
        person.trace.record("send", &request)?;
        wire.send(&request).await?;
    }
    let (replies, receive) = mpsc::channel::<(Value, Result<Value>)>();
    let (notify, mut ready) = notification()?;
    let mut pending = Vec::<ToolCall>::new();
    while !threads.is_empty() {
        let incoming = race(async { wire.next().await.map(Incoming::Frame) }, async {
            let (id, result) = wire::receive(&receive, &mut ready)
                .await?
                .ok_or("tool response direction closed")?;
            Ok::<Incoming, Box<dyn std::error::Error + Send + Sync>>(Incoming::Reply(id, result))
        })
        .await?;
        let message = match incoming {
            Incoming::Frame(message) => message,
            Incoming::Reply(id, outcome) => {
                let call = pending
                    .iter_mut()
                    .find(|call| call.original["id"] == id)
                    .ok_or("tool reply has no matching request")?;
                if call.response.is_some() {
                    return Err("duplicate guest tool reply".into());
                }
                let outcome = outcome?;
                call.response = Some(
                    json!({"success":true,"contentItems":[{"type":"inputText","text":outcome.to_string()}]}),
                );
                if let Some(reply) = call.reply() {
                    people[call.participant].trace.record("send", &reply)?;
                    wire.send(&reply).await?;
                }
                continue;
            }
        };
        if message.get("method").is_none() {
            let id = message["id"].as_u64().ok_or("RPC response id")?;
            let index = starts.remove(&id).ok_or("unexpected RPC response")?;
            people[index].trace.record("receive", &message)?;
            if message.get("error").is_some() {
                return Err(format!("model RPC error: {}", message["error"]).into());
            }
            if !message["result"]["turn"]["id"].is_string() {
                return Err("turn start result missing".into());
            }
            continue;
        }
        let index = message["params"]["threadId"]
            .as_str()
            .and_then(|id| threads.get(id))
            .copied();
        let Some(index) = index else {
            if message.get("id").is_some() || message["method"] == "turn/completed" {
                return Err("model request or completion has no active conversation".into());
            }
            continue;
        };
        let person = &mut people[index];
        person.trace.record("receive", &message)?;
        check_model_item(&message)?;
        match message["method"].as_str() {
            Some("item/tool/call") => {
                person.observed.tool_calls += 1;
                if person.observed.tool_calls > tool_limit {
                    return Err("participant tool-call budget exhausted".into());
                }
                let id = message
                    .get("id")
                    .cloned()
                    .ok_or("tool request id missing")?;
                let params = &message["params"];
                let item_id = params["callId"]
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .ok_or("missing tool callId")?;
                if pending.iter().any(|call| {
                    call.original["id"] == id
                        || call.reader.as_ref().is_some_and(|reader| {
                            reader["id"] == id
                                || (call.participant == index
                                    && reader["params"]["callId"] == item_id)
                        })
                        || (call.participant == index
                            && call.original["params"]["callId"] == item_id)
                }) {
                    return Err("duplicate pending model tool call".into());
                }
                let arguments = &params["arguments"];
                let collect = params["tool"] == "shell"
                    && arguments.as_object().is_some_and(|a| a.is_empty());
                if collect
                    && let Some(call) = pending
                        .iter_mut()
                        .find(|c| c.participant == index && c.reader.is_none())
                {
                    call.reader = Some(message);
                    if let Some(reply) = call.reply() {
                        person.trace.record("send", &reply)?;
                        wire.send(&reply).await?;
                    }
                    continue;
                }
                let response = if collect {
                    Some(
                        json!({"success":true,"contentItems":[{"type":"inputText","text":r#"{"status":"no_interrupted_command"}"#}]}),
                    )
                } else if params["tool"] != "shell"
                    || arguments.as_object().is_none_or(|a| a.len() != 1)
                    || !arguments["command"].is_string()
                {
                    Some(
                        json!({"success":false,"contentItems":[{"type":"inputText","text":"expected a shell command or an empty object to collect interrupted output"}]}),
                    )
                } else {
                    let command = arguments["command"].as_str().unwrap();
                    if command.len() > 4096 {
                        Some(
                            json!({"success":true,"contentItems":[{"type":"inputText","text":r#"{"limit_utf8_bytes":4096,"status":"script_too_long"}"#}]}),
                        )
                    } else {
                        guest.send(
                            json!({"id":id,"agent":person.participant.agent,"command":command}),
                            (replies.clone(), notify.try_clone()?),
                        )?;
                        None
                    }
                };
                let call = ToolCall {
                    participant: index,
                    original: message.clone(),
                    reader: Some(message),
                    response,
                };
                if let Some(reply) = call.reply() {
                    person.trace.record("send", &reply)?;
                    wire.send(&reply).await?;
                }
                pending.push(call);
            }
            Some("item/completed") if message["params"]["item"]["type"] == "dynamicToolCall" => {
                let completed = &message["params"];
                let slot = pending
                    .iter()
                    .position(|call| {
                        call.participant == index
                            && call
                                .reader
                                .as_ref()
                                .is_some_and(|reader| same_tool_call(&reader["params"], completed))
                    })
                    .ok_or("model completed an unknown shell call")?;
                let call = &mut pending[slot];
                if cancelled_tool(completed) {
                    if call.original["params"]["tool"] == "shell"
                        && call.original["params"]["arguments"]
                            .as_object()
                            .is_some_and(|arguments| arguments.len() == 1)
                        && call.original["params"]["arguments"]["command"]
                            .as_str()
                            .is_some_and(|s| s.len() <= 4096)
                    {
                        call.reader.take();
                    } else {
                        pending.remove(slot);
                    }
                    continue;
                }
                let sent = call
                    .reply()
                    .ok_or("model completed shell call before its reply")?;
                check_tool_completion(
                    &call.reader.as_ref().unwrap()["params"],
                    &sent["result"],
                    completed,
                )?;
                pending.remove(slot);
            }
            Some("thread/tokenUsage/updated") => {
                person.observed.usage = message["params"]["tokenUsage"]["total"].clone()
            }
            Some("model/rerouted") if message["params"]["toModel"] != MODEL => {
                return Err("participant model was rerouted".into());
            }
            Some("turn/completed") => {
                if pending.iter().any(|call| call.participant == index) {
                    return Err("turn completed with a pending tool request".into());
                }
                threads.remove(
                    message["params"]["threadId"]
                        .as_str()
                        .ok_or("completed thread id")?,
                );
                completed.push(person.finish(&message["params"]["turn"])?);
            }
            _ if message.get("id").is_some() => {
                let reply = json!({"id":message["id"],"error":{"code":-32601,"message":"Only the supplied experiment tools are available"}});
                person.trace.record("send", &reply)?;
                wire.send(&reply).await?;
            }
            _ => {}
        }
    }
    if !pending.is_empty() || !starts.is_empty() {
        return Err("unfinished model RPC requests".into());
    }
    Ok(())
}
