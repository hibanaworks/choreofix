//! Exact operation data accepted by the file Endpoint's public message contract.
use crate::shell::imp::service::SimulatorOperation;
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;
pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    #[cfg(test)]
    pub(super) fn encode_request(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }

    const CONTRACT: Option<(&'static str, &'static str, &'static str, &'static str)> =
        match OPERATION {
            0 => Some((
                "DEVELOPER_A",
                "implement_feature",
                "feature_name",
                super::global::FEATURE_A,
            )),
            1 => Some((
                "DEVELOPER_A",
                "merge_code",
                "branch",
                super::global::MERGE_BRANCH_A,
            )),
            2 => Some((
                "DEVELOPER_B",
                "implement_feature",
                "feature_name",
                super::global::FEATURE_B,
            )),
            3 => Some((
                "DEVELOPER_B",
                "merge_code",
                "branch",
                super::global::MERGE_BRANCH_B,
            )),
            5 => Some(("REVIEWER", "code_review", "pr_id", super::global::FEATURE_A)),
            6 => Some(("REVIEWER", "code_review", "pr_id", super::global::FEATURE_B)),
            _ => None,
        };
}
impl<const OPERATION: u8> SimulatorOperation for Arguments<OPERATION> {
    const RPC: (&'static str, &'static str) = match Self::CONTRACT {
        Some((agent, tool, _, _)) => (agent, tool),
        None => panic!("unknown review operation"),
    };
}

impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac06_1000 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (_, tool, key, target) = Self::CONTRACT.ok_or(CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if value[key] != target
            || object
                .keys()
                .any(|k| k != key && !((tool == "implement_feature") && k == "source"))
            || object
                .get("source")
                .is_some_and(|v| v.as_str().is_none_or(|v| v.trim().is_empty()))
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
