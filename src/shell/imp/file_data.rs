//! Bytes and descriptor metadata for one participant's Endpoint-bound object.
//! There is no operation dispatch, permission table or workflow state here.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::ReceivedPayload;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsRequest},
    protocol,
};

pub struct FileData {
    object: ChoreoFsObject<'static>,
    pub bytes: Vec<u8>,
    offset: usize,
    flags: u16,
}
impl FileData {
    pub fn new(object: ChoreoFsObject<'static>) -> Self {
        Self {
            object,
            bytes: Vec::new(),
            offset: 0,
            flags: 0,
        }
    }
    pub fn objects(&self) -> ChoreoFsObjectSet<'_, 1> {
        ChoreoFsObjectSet::new([ChoreoFsObject::read_write(
            self.object.path(),
            self.object.object(),
            self.object.fd(),
            &self.bytes,
            protocol::FdBinding {
                read: Some(protocol::FdReadRow::Base),
                write: Some(protocol::FdWriteRow::Object),
                readdir: None,
            },
        )])
    }
    pub fn open(
        &mut self,
        received: ReceivedPayload<protocol::PathOpenReq>,
    ) -> protocol::PathOpenedRet {
        let request = protocol::PathOpenReq::request(&received).0;
        if request.preopen_fd() != 3 {
            return protocol::PathOpenedRet(protocol::PathOpened::new(0, 8));
        }
        let opened = self.objects().choreofs().path_open(received).opened_ret();
        if opened.0.errno() == 0 {
            if request.open_flags() & 8 != 0 {
                self.bytes.clear();
            }
            self.offset = 0;
            self.flags = request.fd_flags();
        }
        opened
    }
    pub fn write(
        &mut self,
        received: ReceivedPayload<protocol::FdWriteObjectReq>,
    ) -> ShellResult<protocol::FdWriteDoneRet> {
        let objects = self.objects();
        let write = objects.choreofs().fd_write(received);
        if !write.is_writable() {
            return Ok(write.written());
        }
        let start = if self.flags & 1 != 0 {
            self.bytes.len()
        } else {
            self.offset
        };
        let Some(end) = start
            .checked_add(write.bytes().len())
            .filter(|n| *n <= 8192)
        else {
            return Ok(protocol::FdWriteDoneRet(
                protocol::FdWriteDone::new_with_errno(write.request().fd(), 0, 22),
            ));
        };
        // The received request borrows Endpoint storage, not this file's bytes.
        self.bytes.resize(self.bytes.len().max(end), 0);
        self.bytes[start..end].copy_from_slice(write.bytes());
        self.offset = end;
        Ok(write.written())
    }
    pub fn read(
        &mut self,
        received: ReceivedPayload<protocol::FdReadReq>,
    ) -> ShellResult<protocol::FdReadDoneRet> {
        let (reply, next) = self
            .objects()
            .choreofs()
            .fd_read(received)
            .read_from(self.offset)?;
        self.offset = next;
        Ok(reply)
    }
    pub fn replace(&mut self, value: serde_json::Value) -> ShellResult<()> {
        self.bytes =
            serde_json::to_vec(&value).map_err(|error| ShellError::message(error.to_string()))?;
        self.offset = 0;
        Ok(())
    }
}
