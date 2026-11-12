//! File contents carried by the private I/O protocol.
use crate::shell::{ShellError, ShellResult};
use hibana::{
    g,
    runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use hibana_wasip1_runtime::{choreofs::ChoreoFsRequest, protocol};
use serde_json::Value;

pub const FILE_BYTES: usize = 8192;
pub const SERVICE_FD: u8 = 4;
pub const RESULT_FD: u8 = 5;
pub const RESULT_CHUNK_LABEL: u8 = 241;
pub type ClosedFileMsg = g::Msg<{ protocol::LABEL_WASI_FD_CLOSE_RET }, ClosedFile>;
pub type ResultChunkMsg = g::Msg<RESULT_CHUNK_LABEL, ResultChunk>;

/// The close result and, for the argument file, its actual pathname and bytes.
pub struct ClosedFile {
    pub completion: protocol::FdClosedRet,
    pub path: Vec<u8>,
    pub bytes: Vec<u8>,
}

impl hibana_wasip1_runtime::FileReply for ClosedFile {
    fn completion(&self) -> protocol::FdClosedRet {
        self.completion
    }
}

impl ClosedFile {
    pub fn arguments(&self, expected_path: &[u8], key: &str) -> ShellResult<Value> {
        if self.completion.0.errno() != 0 || self.path != expected_path {
            return Err(ShellError::message(
                "service file does not match the request",
            ));
        }
        let value: Value = serde_json::from_slice(&self.bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        let object = value
            .as_object()
            .ok_or_else(|| ShellError::message("expected an argument object"))?;
        if object.len() != 1 || !object.get(key).is_some_and(Value::is_string) {
            return Err(ShellError::message(format!(
                "expected one string argument: {key}"
            )));
        }
        Ok(value)
    }
}

impl WireEncode for ClosedFile {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if self.path.len() > protocol::WASIP1_PATH_CHUNK_CAPACITY || self.bytes.len() > FILE_BYTES {
            return Err(CodecError::Malformed);
        }
        if self.completion.0.fd() != SERVICE_FD && (!self.path.is_empty() || !self.bytes.is_empty())
        {
            return Err(CodecError::Malformed);
        }
        let len = 4 + self.path.len() + self.bytes.len();
        let out = out.get_mut(..len).ok_or(CodecError::Truncated)?;
        out[0] = self.completion.0.fd();
        out[1..3].copy_from_slice(&self.completion.0.errno().to_be_bytes());
        out[3] = self.path.len() as u8;
        out[4..4 + self.path.len()].copy_from_slice(&self.path);
        out[4 + self.path.len()..].copy_from_slice(&self.bytes);
        Ok(len)
    }
}
impl WirePayload for ClosedFile {
    const SCHEMA_ID: u32 = 0xcf00_0001;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() < 4 {
            return Err(CodecError::Truncated);
        }
        let path_len = bytes[3] as usize;
        if path_len > protocol::WASIP1_PATH_CHUNK_CAPACITY
            || bytes.len() < 4 + path_len
            || bytes.len() - 4 - path_len > FILE_BYTES
        {
            return Err(CodecError::Malformed);
        }
        if bytes[0] != SERVICE_FD && bytes.len() != 4 {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        let bytes = input.as_bytes();
        let path_end = 4 + bytes[3] as usize;
        Self {
            completion: protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                bytes[0],
                u16::from_be_bytes([bytes[1], bytes[2]]),
            )),
            path: bytes[4..path_end].to_vec(),
            bytes: bytes[path_end..].to_vec(),
        }
    }
}

/// One write to the guest's result file, at an explicit byte offset.
pub struct ResultChunk {
    offset: u64,
    write: protocol::FdWrite,
}
impl ResultChunk {
    pub fn new(offset: usize, bytes: &[u8]) -> ShellResult<Self> {
        offset
            .checked_add(bytes.len())
            .ok_or_else(|| ShellError::message("result offset overflow"))?;
        Ok(Self {
            offset: u64::try_from(offset)
                .map_err(|_| ShellError::message("result offset exceeds WASI filesize"))?,
            write: protocol::FdWrite::new(RESULT_FD, bytes)?,
        })
    }
    pub fn offset(input: &ReceivedPayload<'_, Self>) -> usize {
        let bytes = input.as_bytes();
        usize::try_from(u64::from_be_bytes(
            bytes[..8].try_into().expect("validated result offset"),
        ))
        .expect("validated native result offset")
    }
}
impl WireEncode for ResultChunk {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = 8 + self.write.as_bytes().len();
        let out = out.get_mut(..len).ok_or(CodecError::Truncated)?;
        out[..8].copy_from_slice(&self.offset.to_be_bytes());
        out[8..].copy_from_slice(self.write.as_bytes());
        Ok(len)
    }
}
impl WirePayload for ResultChunk {
    const SCHEMA_ID: u32 = 0xcf00_0003;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() < 8 {
            return Err(CodecError::Truncated);
        }
        let offset = usize::try_from(u64::from_be_bytes(
            bytes[..8].try_into().expect("checked result offset length"),
        ))
        .map_err(|_| CodecError::Malformed)?;
        if bytes.len() - 8 > protocol::WASIP1_IO_CHUNK_CAPACITY
            || offset.checked_add(bytes.len() - 8).is_none()
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl ChoreoFsRequest for ResultChunk {
    type Request = (protocol::FdWrite, protocol::FdWriteRow);
    fn request(input: &ReceivedPayload<'_, Self>) -> Self::Request {
        (
            protocol::FdWrite::new(RESULT_FD, &input.as_bytes()[8..])
                .expect("validated result chunk"),
            protocol::FdWriteRow::Base,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_chunks_keep_offsets_beyond_the_old_file_and_u16_limits() {
        for offset in [8192, usize::from(u16::MAX) + 17] {
            let chunk = ResultChunk::new(offset, b"result bytes").unwrap();
            let mut out = [0; 8 + protocol::WASIP1_IO_CHUNK_CAPACITY];
            let len = chunk.encode_into(&mut out).unwrap();
            let bytes = &out[..len];
            ResultChunk::validate_payload(Payload::new(bytes)).unwrap();
            assert_eq!(
                u64::from_be_bytes(bytes[..8].try_into().unwrap()),
                offset as u64
            );
            assert_eq!(&bytes[8..], b"result bytes");
        }
    }

    #[test]
    fn result_chunks_reject_offset_overflow_and_oversized_packets() {
        assert!(ResultChunk::new(usize::MAX, b"x").is_err());
        let mut overflow = u64::MAX.to_be_bytes().to_vec();
        overflow.push(b'x');
        assert!(ResultChunk::validate_payload(Payload::new(&overflow)).is_err());
        assert!(ResultChunk::new(0, &[0; protocol::WASIP1_IO_CHUNK_CAPACITY + 1]).is_err());
        let too_long = [0; 8 + protocol::WASIP1_IO_CHUNK_CAPACITY + 1];
        assert!(ResultChunk::validate_payload(Payload::new(&too_long)).is_err());
        assert!(ResultChunk::validate_payload(Payload::new(&[0; 7])).is_err());
    }
}
