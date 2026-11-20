//! Actual task arguments and original result bytes, without fd or workflow state.
use super::input::assignment;
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const ROLE: u8, const OPERATION: u8>(Vec<u8>);
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 0> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "design_feature");
}
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 1> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "implement_code");
}
impl<const ROLE: u8> SimulatorOperation<2> for Arguments<ROLE, 2> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "commit_changes");
}
impl<const ROLE: u8> SimulatorOperation for Arguments<ROLE, 3> {
    const RPC: (&'static str, &'static str) = (assignment(ROLE).agent, "run_local_tests");
}
impl<const ROLE: u8, const OPERATION: u8> WireEncode for Arguments<ROLE, OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const ROLE: u8, const OPERATION: u8> WirePayload for Arguments<ROLE, OPERATION> {
    const SCHEMA_ID: u32 = 0x4342_0000 | ((ROLE as u32) << 8) | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        assert!(OPERATION < 4, "unknown codebase request");
        let input = assignment(ROLE);
        let key = if OPERATION == 2 {
            "modules"
        } else {
            "feature_name"
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        let text = value[key].as_str().ok_or(CodecError::Malformed)?;
        if object.len() != 1 || !object.contains_key(key) {
            return Err(CodecError::Malformed);
        }
        let valid = if OPERATION == 2 {
            let mut actual: Vec<_> = text.split(',').map(str::trim).collect();
            let mut expected = input.modules;
            actual.sort_unstable();
            expected.sort_unstable();
            actual == expected
        } else {
            text == input.feature
        };
        if !valid {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

/// A commit submission can be forwarded to the repository or a module owner.
/// Only the request received from a module can be spent on the physical commit.
#[derive(Debug)]
pub struct CommitSubmission<const ROLE: u8> {
    pub arguments: Arguments<ROLE, 2>,
}
impl<const ROLE: u8> CommitSubmission<ROLE> {
    pub fn value(&self) -> Value {
        serde_json::from_slice(&self.arguments.0).expect("validated commit submission")
    }
}
impl<const ROLE: u8> WireEncode for CommitSubmission<ROLE> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.arguments.encode_into(output)
    }
}
impl<const ROLE: u8> WirePayload for CommitSubmission<ROLE> {
    const SCHEMA_ID: u32 = 0x4343_0002 | ((ROLE as u32) << 8);
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<ROLE, 2>::validate_payload(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self {
            arguments: Arguments(input.as_bytes().to_vec()),
        }
    }
}

pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| ShellError::message(error.to_string()))
}
pub fn result(value: &Value, tool: &str, agent: &str) -> ShellResult<()> {
    if value["tool_name"] != tool
        || value["data"]["agent"] != agent
        || !value["success"].is_boolean()
    {
        return Err(ShellError::message(
            "effect result differs from its actual request",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::global::{roles1e::*, *};
    use super::*;
    use crate::imp::transport::InMemoryTransport;
    use hibana::runtime::{SessionKitStorage, ids::SessionId, transport::EndpointBinding};
    use std::time::Duration;

    #[hibana_runtime::main]
    #[test]
    async fn different_module_requests_cannot_issue_a_simulator_call() -> ShellResult<()> {
        use crate::shell::imp::{bus::Bus, service::PairedEffects};
        use hibana::{g, runtime::program::project};
        type First = ModuleRequest<40, A>;
        type Second = ModuleRequest<41, A>;
        let global = g::par(g::send::<40, A, First>(), g::send::<41, A, Second>());
        let receiver_program = project::<A>(&global);
        let first_program = project::<40>(&global);
        let second_program = project::<41>(&global);
        let (mut connection, _, _) = Bus::open([], []);
        let (bus, simulator) = connection.split();
        let mut slab = [0; 32768];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let id = SessionId::new(1);
        let mut first = rv.enter(id, &first_program)?;
        let mut second = rv.enter(id, &second_program)?;
        let mut receiver = rv.enter(id, &receiver_program)?;
        first
            .send::<First>(&Arguments(
                br#"{"modules":"AUTH_MODULE,DATABASE_MODULE"}"#.to_vec(),
            ))
            .await?;
        second
            .send::<Second>(&Arguments(
                br#"{"modules":"DATABASE_MODULE,AUTH_MODULE"}"#.to_vec(),
            ))
            .await?;
        let [mut a, mut b] = receiver.par()?;
        let left = a.recv::<First>().await?;
        let right = b.recv::<Second>().await?;
        let error = hibana_runtime::race(
            simulator.apply_pair(left, right),
            hibana_runtime::host::deadline(Duration::from_secs(2)),
        )
        .await
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "resource requests have different arguments"
        );
        Ok(())
    }

    #[hibana_runtime::main]
    #[test]
    async fn actual_file_endpoint_refuses_early_commit_then_accepts_its_design() -> ShellResult<()>
    {
        let p = project_1e();
        let session = SessionId::new(SESSION);
        let bindings = [A_FILE, B_FILE, C_FILE].map(|role| {
            // SAFETY: these are the sole physical issuers for this test's
            // three file capabilities, with no remote aliases or old run.
            #[allow(unsafe_code)]
            let binding = unsafe { EndpointBinding::new() };
            (session, role, binding)
        });
        let transport = InMemoryTransport::new(bindings);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport)?;
        let mut owner = rv.enter(session, &p.a)?;
        let mut a_data = rv.enter(session, &p.a_data)?;
        let file_a = rv.enter(session, &p.a_file)?;
        let mut a_vm = rv.enter(session, &p.a_vm)?;
        let mut b_data = rv.enter(session, &p.b_data)?;
        let file_b = rv.enter(session, &p.b_file)?;
        let mut b_vm = rv.enter(session, &p.b_vm)?;
        let mut c_data = rv.enter(session, &p.c_data)?;
        let file_c = rv.enter(session, &p.c_file)?;
        let mut c_vm = rv.enter(session, &p.c_vm)?;
        a_data.send::<Loan<A_FILE>>(file_a).await?;
        let mut file = a_vm.recv::<Loan<A_FILE>>().await?;
        b_data.send::<Loan<B_FILE>>(file_b).await?;
        let _file_b = b_vm.recv::<Loan<B_FILE>>().await?;
        c_data.send::<Loan<C_FILE>>(file_c).await?;
        let _file_c = c_vm.recv::<Loan<C_FILE>>().await?;

        let early = CommitSubmission {
            arguments: Arguments::<A, 2>(br#"{"modules":"AUTH_MODULE,DATABASE_MODULE"}"#.to_vec()),
        };
        let denied = hibana_runtime::race(
            async { Ok::<_, ShellError>(file.send::<Submitted<A>>(&early).await) },
            hibana_runtime::host::deadline(Duration::from_secs(2)),
        )
        .await?;
        assert!(
            denied.is_err(),
            "the projected file accepted a commit before its design"
        );
        let design = Arguments::<A, 0>(br#"{"feature_name":"user_authentication"}"#.to_vec());
        file.send::<DesignRequest<A>>(&design).await?;
        assert_eq!(
            serde_json::from_slice::<Value>(owner.recv::<DesignRequest<A>>().await?.as_bytes())
                .unwrap(),
            serde_json::json!({"feature_name":"user_authentication"})
        );
        Ok(())
    }
}
