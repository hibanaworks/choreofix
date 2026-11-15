//! ChoreoFS object bytes and offsets, without task progress or permission state.
use super::global::{OUTCOME_FD, OUTCOME_PATH};
use crate::shell::ShellResult;
use hibana::runtime::wire::ReceivedPayload;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId},
    protocol,
};
pub(crate) fn outcome_objects(bytes: &[u8]) -> ChoreoFsObjectSet<'_, 1> {
    ChoreoFsObjectSet::new([ChoreoFsObject::readable(
        OUTCOME_PATH,
        ObjectId(u32::from(OUTCOME_FD)),
        FdSpec::new(OUTCOME_FD, 1 << 1, 1),
        bytes,
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )])
}

pub const MIGRATE: u8 = 1;
pub const TEST: u8 = 2;

/// The canonical arguments received through a resource Endpoint.
#[derive(Debug, PartialEq, Eq)]
pub struct ApiArguments<const OPERATION: u8>(Vec<u8>);

impl<const OPERATION: u8> ApiArguments<OPERATION> {
    const ENCODED: &'static [u8] = match OPERATION {
        MIGRATE => br#"{"migration_name":"add_new_tables"}"#,
        TEST => br#"{"suite":"backend_schema_integration"}"#,
        _ => panic!("unknown API operation"),
    };

    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        use hibana::runtime::wire::{Payload, WirePayload};
        Self::validate_payload(Payload::new(bytes))
            .map_err(|error| crate::shell::ShellError::message(format!("{error:?}")))?;
        Ok(Self(bytes.to_vec()))
    }

    pub fn into_value(self) -> serde_json::Value {
        serde_json::from_slice(&self.0).expect("validated API arguments")
    }
}

impl<const OPERATION: u8> hibana::runtime::wire::WireEncode for ApiArguments<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, hibana::runtime::wire::CodecError> {
        use hibana::runtime::wire::CodecError;
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const OPERATION: u8> hibana::runtime::wire::WirePayload for ApiArguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac08_0000 | OPERATION as u32;
    type Decoded<'a> = Self;

    fn validate_payload(
        input: hibana::runtime::wire::Payload<'_>,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        if input.as_bytes() != Self::ENCODED {
            return Err(hibana::runtime::wire::CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
pub(crate) struct OutcomeFile {
    pub bytes: Vec<u8>,
    offset: usize,
}

impl OutcomeFile {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            offset: 0,
        }
    }
    pub fn open(
        &mut self,
        request: ReceivedPayload<'_, protocol::PathOpenReq>,
    ) -> ShellResult<protocol::PathOpenedRet> {
        let objects = outcome_objects(&self.bytes);
        let op = objects.choreofs().path_open(request);
        if op.is_open() {
            self.offset = 0
        }
        Ok(op.opened_ret())
    }
    pub fn read(
        &mut self,
        request: ReceivedPayload<'_, protocol::FdReadReq>,
    ) -> ShellResult<protocol::FdReadDoneRet> {
        let (response, next) = outcome_objects(&self.bytes)
            .choreofs()
            .fd_read(request)
            .read_from(self.offset)?;
        self.offset = next;
        Ok(response)
    }
    pub fn stat(
        &self,
        request: ReceivedPayload<'_, protocol::FdFdstatGetReq>,
    ) -> protocol::FdStatRet {
        outcome_objects(&self.bytes)
            .choreofs()
            .fd_fdstat_get(request)
    }
    pub fn file_stat(
        &self,
        request: ReceivedPayload<'_, protocol::FdFilestatGetReq>,
    ) -> protocol::FdFilestatRet {
        outcome_objects(&self.bytes)
            .choreofs()
            .fd_filestat_get(request)
    }
    pub fn close(
        &self,
        request: ReceivedPayload<'_, protocol::FdCloseReq>,
    ) -> protocol::FdClosedRet {
        outcome_objects(&self.bytes).choreofs().fd_close(request)
    }
    pub fn path_stat(
        &self,
        request: ReceivedPayload<'_, protocol::PathFilestatGetReq>,
    ) -> protocol::PathFilestatRet {
        outcome_objects(&self.bytes)
            .choreofs()
            .path_filestat_get(request)
    }
}
