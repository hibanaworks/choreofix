//! Actual task arguments and original result bytes, without fd or workflow state.
use super::input::assignment;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub struct Arguments<const ROLE: u8, const OPERATION: u8>(Vec<u8>);
impl<const ROLE: u8, const OPERATION: u8> Arguments<ROLE, OPERATION> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated codebase arguments")
    }
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
    type Decoded<'a> = Self;
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
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

/// The two-module request keeps the original argument bytes.
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleArguments(Vec<u8>);
impl ModuleArguments {
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("invalid module arguments"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated module arguments")
    }
}
impl WireEncode for ModuleArguments {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl WirePayload for ModuleArguments {
    const SCHEMA_ID: u32 = 0x4342_ff02;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if value.as_object().is_none_or(|object| object.len() != 1) || !value["modules"].is_string()
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
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

        let early = Arguments::<A, 2>(br#"{"modules":"AUTH_MODULE,DATABASE_MODULE"}"#.to_vec());
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
            owner.recv::<DesignRequest<A>>().await?.into_value(),
            serde_json::json!({"feature_name":"user_authentication"})
        );
        Ok(())
    }
}
