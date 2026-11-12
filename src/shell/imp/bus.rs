//! The external connection owns terminal output and simulator RPC correlation.
use crate::{
    shell::imp::service::Domain,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{ids::SessionId, transport::EndpointBinding};
use hibana_runtime::host::Io;
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
    future::poll_fn,
    io::{Stdin, Stdout},
    os::fd::AsFd,
    task::{Context, Poll, Waker},
};

pub struct Bus {
    pub transport: crate::imp::transport::InMemoryTransport,
    connection: Cell<Connection>,
}

type Commands = (VecDeque<(u64, String)>, Option<Waker>);
type Response = (Option<Value>, Option<Waker>);

/// Bytes and correlations of the external connection, not task permissions.
#[derive(Default)]
struct Connection {
    sequence: u64,
    outgoing: VecDeque<(Vec<Value>, Option<u64>)>,
    output_waiter: Option<Waker>,
    responses: BTreeMap<u64, Response>,
    terminals: BTreeMap<&'static str, Option<Commands>>,
}

/// Own one outstanding RPC or output-flush receipt, including its cancellation.
struct Reply<'a> {
    bus: &'a Bus,
    id: u64,
}

pub struct ExternalIo {
    input: Stdin,
    output: Option<Stdout>,
    // The accepted byte prefix survives cancellation of the output future.
    output_bytes: Vec<u8>,
    output_offset: usize,
    output_receipt: Option<u64>,
}

pub struct Terminal {
    pub agent: &'static str,
    // The current command RPC identifier, not a task phase or permission.
    identifier: Option<u64>,
}

impl Bus {
    /// Open the sole connection to a fresh external simulator trial.
    pub fn open<const N: usize, const R: usize>(
        agents: [&'static str; N],
        resources: [(SessionId, u8); R],
    ) -> (Self, ExternalIo, [Terminal; N]) {
        let mut connection = Connection::default();
        let terminals = agents.map(|agent| {
            assert!(
                connection
                    .terminals
                    .insert(agent, Some(Default::default()))
                    .is_none()
            );
            Terminal {
                agent,
                identifier: None,
            }
        });
        let bindings = resources.map(|(session, role)| {
            // SAFETY: one fresh simulator trial assigns each listed role a
            // distinct resource or request object. Every runtime reaches these same
            // bindings. Endpoint transfers and failures never reissue them.
            #[allow(unsafe_code)]
            let binding = unsafe { EndpointBinding::new() };
            (session, role, binding)
        });
        (
            Self {
                transport: crate::imp::transport::InMemoryTransport::new(bindings),
                connection: Cell::new(connection),
            },
            ExternalIo {
                input: std::io::stdin(),
                output: Some(std::io::stdout()),
                output_bytes: Vec::new(),
                output_offset: 0,
                output_receipt: None,
            },
            terminals,
        )
    }

    fn connection<T>(&self, apply: impl FnOnce(&mut Connection) -> T) -> T {
        let mut connection = self.connection.take();
        let result = apply(&mut connection);
        self.connection.set(connection);
        result
    }

    fn reply(&self) -> ShellResult<Reply<'_>> {
        let id = self
            .connection(|connection| {
                let id = connection.sequence.checked_add(1)?;
                connection.sequence = id;
                assert!(connection.responses.insert(id, (None, None)).is_none());
                Some(id)
            })
            .ok_or_else(|| ShellError::message("RPC id overflow"))?;
        Ok(Reply { bus: self, id })
    }

    fn enqueue(&self, values: Vec<Value>, flushed: Option<u64>) {
        let wake = self.connection(|connection| {
            connection.outgoing.push_back((values, flushed));
            connection.output_waiter.take()
        });
        if let Some(wake) = wake {
            wake.wake();
        }
    }

    /// Queue an immutable diagnostic observation with the I/O owner.
    pub fn observe(&self, value: Value) {
        self.enqueue(vec![value], None);
    }

    fn respond(&self, id: u64, value: Value) -> ShellResult<()> {
        let wake = self.connection(|connection| {
            let (result, wake) = connection
                .responses
                .get_mut(&id)
                .ok_or_else(|| ShellError::message("unexpected or cancelled RPC result"))?;
            if result.is_some() {
                return Err(ShellError::message("duplicate RPC result"));
            }
            *result = Some(value);
            Ok(wake.take())
        })?;
        if let Some(wake) = wake {
            wake.wake();
        }
        Ok(())
    }

    pub async fn emit(&self, value: Value) -> ShellResult<()> {
        let mut values = self.transport.take_observations();
        values.push(value);
        let reply = self.reply()?;
        self.enqueue(values, Some(reply.id));
        reply.receive().await?;
        Ok(())
    }
}

impl Reply<'_> {
    fn poll_response(&self, cx: &mut Context<'_>) -> Poll<ShellResult<Value>> {
        self.bus.connection(|connection| {
            let (result, wake) = connection
                .responses
                .get_mut(&self.id)
                .expect("an owned RPC receipt remains registered");
            if let Some(value) = result.take() {
                Poll::Ready(Ok(value))
            } else {
                *wake = Some(cx.waker().clone());
                Poll::Pending
            }
        })
    }

    fn receive(self) -> impl Future<Output = ShellResult<Value>> {
        poll_fn(move |cx| self.poll_response(cx))
    }
}

impl Drop for Reply<'_> {
    fn drop(&mut self) {
        self.bus.connection(|connection| {
            connection.responses.remove(&self.id);
        });
    }
}

impl ExternalIo {
    async fn flush_output(&mut self, bus: &Bus, output: &mut Io) -> ShellResult<()> {
        if self.output.is_none() {
            return Err(ShellError::message("external output is closed"));
        }
        loop {
            while self.output_offset < self.output_bytes.len() {
                let count = match output.write(&self.output_bytes[self.output_offset..]).await {
                    Ok(0) => {
                        self.output.take();
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::WriteZero,
                            "external output accepted no bytes",
                        )
                        .into());
                    }
                    Ok(count) => count,
                    Err(error) => {
                        self.output.take();
                        return Err(error.into());
                    }
                };
                self.output_offset += count;
            }
            self.output_bytes.clear();
            self.output_offset = 0;
            // Cancelling the local flush waiter cannot retract queued bytes.
            // Complete only a live waiter; external RPC replies remain strict.
            if let Some(id) = self.output_receipt.take()
                && bus.connection(|connection| connection.responses.contains_key(&id))
            {
                bus.respond(id, Value::Null)?;
            }
            let Some((values, receipt)) =
                bus.connection(|connection| connection.outgoing.pop_front())
            else {
                return Ok(());
            };
            let mut bytes = Vec::new();
            for value in values {
                serde_json::to_writer(&mut bytes, &value)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                bytes.push(b'\n');
            }
            self.output_bytes = bytes;
            self.output_receipt = receipt;
        }
    }

    pub async fn emit(&mut self, bus: &Bus, value: Value) -> ShellResult<()> {
        let stdout = self
            .output
            .as_ref()
            .ok_or_else(|| ShellError::message("external output is closed"))?;
        let mut output = Io::new(stdout.as_fd().try_clone_to_owned()?)?;
        let mut values = bus.transport.take_observations();
        values.push(value);
        bus.enqueue(values, None);
        self.flush_output(bus, &mut output).await
    }

    #[hibana_runtime::scope]
    pub async fn run(&mut self, bus: &Bus) -> ShellResult<()> {
        let mut input = Io::new(self.input.as_fd().try_clone_to_owned()?)?;
        let stdout = self
            .output
            .as_ref()
            .ok_or_else(|| ShellError::message("external output is closed"))?;
        let mut output = Io::new(stdout.as_fd().try_clone_to_owned()?)?;
        async {
            let mut buffered = Vec::new();
            let mut bytes = [0; 4096];
            loop {
                let count = input.read(&mut bytes).await?;
                if count == 0 {
                    return Err(ShellError::message(
                        "terminal transport closed before guest completion",
                    ));
                }
                buffered.extend_from_slice(&bytes[..count]);
                while let Some(end) = buffered.iter().position(|byte| *byte == b'\n') {
                    let value: Value = serde_json::from_slice(&buffered[..end])
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    buffered.drain(..=end);
                    let id = value["id"]
                        .as_u64()
                        .ok_or_else(|| ShellError::message("RPC id missing"))?;
                    match value["type"].as_str() {
                        Some("command") => {
                            let agent = value["agent"]
                                .as_str()
                                .ok_or_else(|| ShellError::message("RPC agent missing"))?;
                            let command = value["command"]
                                .as_str()
                                .ok_or_else(|| ShellError::message("RPC command missing"))?;
                            let delivery = bus.connection(|connection| {
                                let terminal = connection
                                    .terminals
                                    .get_mut(agent)
                                    .ok_or_else(|| ShellError::message("unknown terminal"))?;
                                Ok::<_, ShellError>(terminal.as_mut().map(|(commands, wake)| {
                                    commands.push_back((id, command.to_owned()));
                                    wake.take()
                                }))
                            })?;
                            match delivery {
                                Some(Some(wake)) => wake.wake(),
                                Some(None) => {}
                                None => {
                                    bus.emit(
                                        json!({"type":"terminal_closed", "id":id, "agent":agent}),
                                    )
                                    .await?
                                }
                            }
                        }
                        Some("simulator_result") => bus.respond(id, value["result"].clone())?,
                        _ => return Err(ShellError::message("unknown direct WASI RPC message")),
                    }
                }
            }
        };
        async {
            loop {
                self.flush_output(bus, &mut output).await?;
                poll_fn(|cx| {
                    bus.connection(|connection| {
                        if !connection.outgoing.is_empty() {
                            Poll::Ready(())
                        } else {
                            connection.output_waiter = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    })
                })
                .await;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "bus/tests.rs"]
mod tests;

impl Terminal {
    pub async fn read_command(&mut self, bus: &Bus, output: Vec<u8>) -> ShellResult<String> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        bus.emit(
            json!({"type":"terminal", "agent":self.agent, "id":self.identifier, "stdout":stdout}),
        )
        .await?;
        let (id, command) = poll_fn(|cx| {
            bus.connection(|connection| {
                let (commands, wake) = connection
                    .terminals
                    .get_mut(self.agent)
                    .and_then(Option::as_mut)
                    .expect("an active terminal remains registered");
                if let Some(command) = commands.pop_front() {
                    Poll::Ready(command)
                } else {
                    *wake = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
        })
        .await;
        self.identifier = Some(id);
        Ok(command)
    }

    pub async fn finish(self, bus: &Bus, status: i32, output: Vec<u8>) -> ShellResult<()> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        bus.connection(|connection| {
            connection
                .terminals
                .get_mut(self.agent)
                .expect("a terminal remains registered")
                .take();
        });
        bus.emit(json!({"type":"exit", "agent":self.agent, "id":self.identifier, "status":status, "stdout":stdout})).await
    }
}

impl Domain for Bus {
    fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>> {
        // The unpolled request stays lazy. Once accepted, the actual RPC
        // receipt can be borrowed by a race without cancelling the operation.
        let mut request =
            Some(json!({"type":"execute", "agent":agent, "tool":tool, "arguments":arguments}));
        let mut receipt = None;
        poll_fn(move |cx| {
            if let Some(mut request) = request.take() {
                let reply = match self.reply() {
                    Ok(reply) => reply,
                    Err(error) => return Poll::Ready(Err(error.to_string())),
                };
                request["id"] = json!(reply.id);
                let mut values = self.transport.take_observations();
                values.push(request);
                self.enqueue(values, None);
                receipt = Some(reply);
            }
            let result = receipt
                .as_ref()
                .expect("completed RPC future polled again")
                .poll_response(cx);
            if result.is_ready() {
                receipt.take();
            }
            result.map(|value| value.map_err(|error| error.to_string()))
        })
    }
}

#[cfg(test)]
impl Bus {
    /// A native world peer consumes the same RPC queue and replies as stdio.
    /// It never reads or advances Endpoint progress. N is the fixture's actual
    /// effect population, not a production task/completion counter.
    #[hibana_runtime::scope]
    pub(crate) async fn fixture_peer<const N: usize>(
        &self,
        domain: &impl Domain,
        tools: &[&'static str],
    ) -> ShellResult<()> {
        for _ in [(); N] {
            async {
                let request = poll_fn(|cx| {
                    self.connection(|connection| {
                        while let Some((values, receipt)) = connection.outgoing.pop_front() {
                            if receipt.is_some() {
                                return Poll::Ready(Err(ShellError::message(
                                    "effect fixture cannot acknowledge terminal output",
                                )));
                            }
                            if let Some(request) =
                                values.into_iter().find(|v| v["type"] == "execute")
                            {
                                return Poll::Ready(Ok(request));
                            }
                        }
                        connection.output_waiter = Some(cx.waker().clone());
                        Poll::Pending
                    })
                })
                .await?;
                let agent = self
                    .connection(|connection| {
                        connection
                            .terminals
                            .keys()
                            .copied()
                            .find(|name| request["agent"] == *name)
                    })
                    .ok_or_else(|| ShellError::message("unknown fixture actor"))?;
                let tool = tools
                    .iter()
                    .copied()
                    .find(|name| request["tool"] == *name)
                    .ok_or_else(|| ShellError::message("unknown fixture tool"))?;
                let result = domain
                    .execute(agent, tool, request["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                self.respond(
                    request["id"]
                        .as_u64()
                        .ok_or_else(|| ShellError::message("fixture RPC id missing"))?,
                    result,
                )?;
            };
        }
        Ok(())
    }
}
