//! The external connection owns terminal output and simulator RPC correlation.
use crate::shell::imp::service::ResourceReceipt;
use crate::{
    shell::imp::service::{Domain, PairedEffects, ReceivedEffects, SimulatorOperation},
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
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// Terminal and observation I/O shared by a trial.
///
/// ```compile_fail
/// use choreofix::shell::imp::{bus::Bus, service::Domain};
/// fn fabricate(io: &Bus) {
///     let _ = io.execute("REVIEWER", "code_review", serde_json::json!({}));
/// }
/// ```
///
/// ```compile_fail
/// use choreofix::shell::imp::bus::Bus;
/// use choreofix::shell::imp::service::{ReceivedEffects, SimulatorOperation};
/// use hibana::runtime::wire::ReceivedPayload;
/// fn submit<P: SimulatorOperation>(io: &Bus, receipt: ReceivedPayload<P>) {
///     let _ = io.apply(receipt);
/// }
/// ```
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
    outgoing: VecDeque<(Vec<u8>, Option<u64>)>,
    output_waiter: Option<Waker>,
    responses: BTreeMap<u64, Response>,
    terminals: BTreeMap<&'static str, Option<Commands>>,
}

/// Own one outstanding RPC or output-flush receipt, including its cancellation.
pub struct Reply<'a> {
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
    agent: &'static str,
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

    /// Split the trial owner's simulator connection from ordinary terminal and capture I/O.
    pub fn split(&mut self) -> (&Self, Simulator<'_>) {
        (self, Simulator(self))
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

    fn enqueue(&self, values: impl IntoIterator<Item = Value>, flushed: Option<u64>) {
        let mut bytes = Vec::new();
        for value in values {
            serde_json::to_writer(&mut bytes, &value)
                .expect("JSON values serialize to an in-memory byte buffer");
            bytes.push(b'\n');
        }
        self.connection(|connection| {
            connection.outgoing.push_back((bytes, flushed));
        });
    }

    fn wake_output(&self) {
        if let Some(wake) = self.connection(|connection| connection.output_waiter.take()) {
            wake.wake();
        }
    }

    /// Queue an immutable diagnostic observation with the I/O owner.
    pub fn projected(&self, mut projection: Value) {
        projection["type"] = json!("compiled_projection");
        self.observe(projection);
    }

    pub fn capture(&self, event: hibana::runtime::tap::TapEvent) {
        self.observe(json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}));
    }

    pub fn checkpoint(&self, session: u32, checkpoint: Option<u32>) {
        self.observe(json!({"type":"protocol_checkpoint","producer":0,"session":session,"checkpoint":checkpoint}));
    }

    fn observe(&self, value: Value) {
        let values = self.transport.take_observations().chain([value]);
        self.enqueue(values, None);
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

    async fn emit(&self, value: Value) -> ShellResult<()> {
        let values = self.transport.take_observations().chain([value]);
        let reply = self.reply()?;
        self.enqueue(values, Some(reply.id));
        self.wake_output();
        reply.await?;
        Ok(())
    }
}

impl Future for Reply<'_> {
    type Output = ShellResult<Value>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.bus.connection(|connection| {
            let std::collections::btree_map::Entry::Occupied(mut response) =
                connection.responses.entry(self.id)
            else {
                panic!("completed RPC receipt polled again");
            };
            let (result, wake) = response.get_mut();
            if let Some(value) = result.take() {
                response.remove();
                Poll::Ready(Ok(value))
            } else {
                *wake = Some(cx.waker().clone());
                Poll::Pending
            }
        })
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
            let Some((bytes, receipt)) =
                bus.connection(|connection| connection.outgoing.pop_front())
            else {
                return Ok(());
            };
            self.output_bytes = bytes;
            self.output_receipt = receipt;
        }
    }

    /// Flush preceding observations and the final trial outcome before
    /// returning the participants' result. Partial trials name their scope.
    pub async fn finish(
        mut self,
        bus: &Bus,
        result: ShellResult<()>,
        scope: Option<&str>,
    ) -> ShellResult<()> {
        let value = match &result {
            Ok(()) => {
                let mut report = json!({"type":"run_result","status":"complete"});
                if let Some(scope) = scope {
                    report["scope"] = json!(scope);
                }
                report
            }
            Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
        };
        let stdout = self
            .output
            .as_ref()
            .ok_or_else(|| ShellError::message("external output is closed"))?;
        let mut output = Io::new(stdout.as_fd().try_clone_to_owned()?)?;
        let values = bus.transport.take_observations().chain([value]);
        bus.enqueue(values, None);
        self.flush_output(bus, &mut output).await?;
        result
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

/// The simulator connection held by the native resource receivers of a trial.
pub struct Simulator<'bus>(&'bus Bus);

impl<'bus> Simulator<'bus> {
    /// Enqueue this received instrument request before handing its physical
    /// resource to the next participant. The reply may still be outstanding.
    pub fn submit<P: SimulatorOperation>(
        &self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> ShellResult<Reply<'bus>> {
        let (agent, tool) = P::RPC;
        let args = received_arguments::<P>(received)?;
        self.0.queue_rpc(agent, tool, args)
    }
}

fn received_arguments<P: SimulatorOperation>(
    received: hibana::runtime::wire::ReceivedPayload<'_, P>,
) -> ShellResult<Value> {
    let value = serde_json::from_slice(received.as_bytes())
        .map_err(|e| ShellError::message(format!("simulator request JSON: {e}")))?;
    rpc_arguments::<1, P>(value)
}

fn rpc_arguments<const N: usize, P: SimulatorOperation<N>>(
    mut request: Value,
) -> ShellResult<Value> {
    match P::ARGUMENT_FIELD {
        None => Ok(request),
        Some(field) => request
            .as_object_mut()
            .and_then(|object| object.remove(field))
            .ok_or_else(|| ShellError::message(format!("received request lacks {field}"))),
    }
}

impl Domain for Simulator<'_> {
    fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>> {
        self.0.rpc(agent, tool, arguments)
    }
}

impl<'bus> ReceivedEffects for Simulator<'bus> {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'bus, 'io, P> {
        let bus: &'io Bus = self.0;
        let arguments = received_arguments::<P>(received);
        async move {
            let (agent, tool) = P::RPC;
            bus.rpc(agent, tool, arguments?)
                .await
                .map_err(ShellError::message)
        }
    }
}

impl PairedEffects for Simulator<'_> {
    async fn apply_pair<'a, 'b, P: SimulatorOperation<2>>(
        &self,
        first: impl Into<ResourceReceipt<'a, P>>,
        second: impl Into<ResourceReceipt<'b, P>>,
    ) -> ShellResult<Value> {
        let first = first.into();
        let second = second.into();

        let decode = |bytes: &[u8]| {
            serde_json::from_slice::<Value>(bytes)
                .map_err(|error| ShellError::message(error.to_string()))
        };
        let arguments = decode(first.as_bytes())?;
        if first.as_bytes() != second.as_bytes() && decode(second.as_bytes())? != arguments {
            return Err(ShellError::message(
                "resource requests have different arguments",
            ));
        }
        let (agent, tool) = P::RPC;
        self.0
            .rpc(agent, tool, rpc_arguments::<2, P>(arguments)?)
            .await
            .map_err(ShellError::message)
    }
}

impl Bus {
    fn queue_rpc(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> ShellResult<Reply<'_>> {
        let reply = self.reply()?;
        let request =
            json!({"type":"execute","id":reply.id,"agent":agent,"tool":tool,"arguments":arguments});
        self.enqueue(self.transport.take_observations().chain([request]), None);
        self.wake_output();
        Ok(reply)
    }

    // A borrowed race may drop its waiter and later resume this same request.
    fn rpc(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>> + Unpin {
        let mut arguments = Some(arguments);
        let mut receipt = None;
        poll_fn(move |cx| {
            if let Some(args) = arguments.take() {
                match self.queue_rpc(agent, tool, args) {
                    Ok(reply) => receipt = Some(reply),
                    Err(error) => return Poll::Ready(Err(error.to_string())),
                }
            }
            let result =
                Pin::new(receipt.as_mut().expect("completed RPC future polled again")).poll(cx);
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
                        while let Some((bytes, receipt)) = connection.outgoing.pop_front() {
                            if receipt.is_some() {
                                return Poll::Ready(Err(ShellError::message(
                                    "effect fixture cannot acknowledge terminal output",
                                )));
                            }
                            for value in
                                serde_json::Deserializer::from_slice(&bytes).into_iter::<Value>()
                            {
                                let request = match value {
                                    Ok(value) => value,
                                    Err(error) => {
                                        return Poll::Ready(Err(ShellError::message(
                                            error.to_string(),
                                        )));
                                    }
                                };
                                if request["type"] == "execute" {
                                    return Poll::Ready(Ok(request));
                                }
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
