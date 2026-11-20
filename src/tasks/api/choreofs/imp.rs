//! ChoreoFS object bytes and offsets, without task progress or permission state.
use super::global::{OUTCOME_FD, OUTCOME_PATH};
use crate::shell::ShellResult;
use hibana::runtime::wire::ReceivedPayload;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId},
    protocol,
};
pub(crate) fn outcome_objects(bytes: &[u8]) -> ChoreoFsObjectSet<[ChoreoFsObject<'_>; 1]> {
    ChoreoFsObjectSet::new([ChoreoFsObject::readable(
        OUTCOME_PATH,
        ObjectId(u32::from(OUTCOME_FD)),
        FdSpec::new(OUTCOME_FD, 1 << 1),
        bytes,
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )])
}

pub const MIGRATE: u8 = 1;
pub const TEST: u8 = 2;

pub const IMPLEMENT: u8 = 3;

/// Both stages preserve the submitted bytes. Only a resource request can be applied.
pub struct ApiArguments<const OP: u8, const RESOURCE: bool = true>(pub(super) Vec<u8>);

impl<const OP: u8, const RESOURCE: bool> ApiArguments<OP, RESOURCE> {
    fn validate(
        input: hibana::runtime::wire::Payload,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        use hibana::runtime::wire::CodecError;
        let value: serde_json::Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (key, target) = match OP {
            MIGRATE => ("migration_name", "add_new_tables"),
            TEST => ("suite", "backend_schema_integration"),
            IMPLEMENT => ("component", "new_api_endpoints"),
            _ => return Err(CodecError::Malformed),
        };
        match value.as_object() {
            Some(fields)
                if fields.len() == 1
                    && fields.get(key).and_then(|v| v.as_str()) == Some(target) =>
            {
                Ok(())
            }
            _ => Err(CodecError::Malformed),
        }
    }
}
impl<const OP: u8, const RESOURCE: bool> hibana::runtime::wire::WireEncode
    for ApiArguments<OP, RESOURCE>
{
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, hibana::runtime::wire::CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(hibana::runtime::wire::CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OP: u8> hibana::runtime::wire::WirePayload for ApiArguments<OP, false> {
    const SCHEMA_ID: u32 = 0xac08_0100 | OP as u32;
    type Decoded<'a> = Self;
    fn validate_payload(
        input: hibana::runtime::wire::Payload,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        Self::validate(input)
    }
    fn decode_validated_payload(input: ReceivedPayload<Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
impl<const OP: u8> hibana::runtime::wire::WirePayload for ApiArguments<OP> {
    const SCHEMA_ID: u32 = 0xac08_0000 | OP as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(
        input: hibana::runtime::wire::Payload,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        Self::validate(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl<const OP: u8> crate::shell::imp::service::SimulatorOperation for ApiArguments<OP> {
    const RPC: (&'static str, &'static str) = match OP {
        MIGRATE => ("DB_ADMIN", "migrate_schema"),
        TEST => ("TESTER", "run_integration_tests"),
        IMPLEMENT => ("BACKEND_DEV", "implement_backend"),
        _ => panic!("unknown API operation"),
    };
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
