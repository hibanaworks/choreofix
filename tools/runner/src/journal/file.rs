//! Reconstruct one file's contents from recorded WASI requests and replies.
use super::Frame;
use crate::evidence::Result;
use hibana_wasip1_runtime::protocol;

pub(crate) fn same_path(bytes: &[u8], path: &str) -> bool {
    use hibana_wasip1_runtime::choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId};
    let objects = ChoreoFsObjectSet::new([ChoreoFsObject::new(
        path.as_bytes(),
        ObjectId(0),
        FdSpec::new(0, 0),
    )]);
    objects.choreofs().resolve(bytes) == Some(ObjectId(0))
}

pub(crate) struct FileContents {
    fd: u8,
    pub(crate) bytes: Vec<u8>,
    pub(crate) offset: usize,
    flags: u16,
}

impl FileContents {
    pub(crate) fn new(fd: u8) -> Self {
        Self {
            fd,
            bytes: Vec::new(),
            offset: 0,
            flags: 0,
        }
    }

    pub(crate) fn open(
        &mut self,
        request: &Frame,
        reply: &Frame,
        matches_path: impl FnOnce(&[u8]) -> bool,
    ) -> Result<()> {
        request.validate::<protocol::PathOpenReq>()?;
        reply.validate::<protocol::PathOpenedRet>()?;
        if reply.bytes[1..3] == [0, 0] {
            if reply.bytes[0] != self.fd
                || request.bytes[0] != 3
                || !matches_path(&request.bytes[14..])
            {
                return Err("successful open names a different file".into());
            }
            if u16::from_be_bytes(request.bytes[9..11].try_into()?) & 8 != 0 {
                self.bytes.clear();
            }
            self.flags = u16::from_be_bytes(request.bytes[11..13].try_into()?);
            self.offset = 0;
        }
        Ok(())
    }

    pub(crate) fn write(&mut self, request: &Frame, reply: &Frame) -> Result<()> {
        request.validate::<protocol::FdWriteObjectReq>()?;
        reply.validate::<protocol::FdWriteDoneRet>()?;
        let count = usize::from(reply.bytes[1]);
        let input = &request.bytes[2..];
        let errno = u16::from_be_bytes(reply.bytes[2..4].try_into()?);
        if request.bytes[0] != self.fd
            || reply.bytes[0] != self.fd
            || count > input.len()
            || (errno != 0 && count != 0)
        {
            return Err("write fd or accepted byte count differs".into());
        }
        if errno == 0 {
            let start = if self.flags & 1 != 0 {
                self.bytes.len()
            } else {
                self.offset
            };
            let end = start.checked_add(count).ok_or("file offset overflow")?;
            self.bytes.resize(self.bytes.len().max(end), 0);
            self.bytes[start..end].copy_from_slice(&input[..count]);
            self.offset = end;
        }
        Ok(())
    }

    pub(crate) fn read(&mut self, request: &Frame, reply: &Frame) -> Result<()> {
        request.validate::<protocol::FdReadReq>()?;
        reply.validate::<protocol::FdReadDoneRet>()?;
        let end = self
            .offset
            .saturating_add(usize::from(request.bytes[1]))
            .min(self.bytes.len());
        if request.bytes[0] != self.fd
            || reply.bytes[0] != self.fd
            || reply.bytes[1..3] != [0, 0]
            || self.bytes.get(self.offset..end) != Some(&reply.bytes[4..])
        {
            return Err("read fd or returned file contents differ".into());
        }
        self.offset = end;
        Ok(())
    }

    pub(crate) fn close(&self, request: &Frame, reply: &Frame) -> Result<()> {
        request.validate::<protocol::FdCloseReq>()?;
        reply.validate::<protocol::FdClosedRet>()?;
        if request.bytes != [self.fd] || reply.bytes != [self.fd, 0, 0] {
            return Err("close fd or result differs".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn frame(bytes: &[u8]) -> Frame<'static> {
        Frame {
            logical_label: 0,
            identity: &Value::Null,
            bytes: bytes.to_vec(),
            sent: 0,
            first_poll: 0,
            received: 0,
            receipt: &Value::Null,
        }
    }

    #[test]
    fn reconstruct_only_accepted_bytes_across_reopens() -> Result<()> {
        let mut file = FileContents::new(7);
        let mut open = frame(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, b'x']);
        let opened = frame(&[7, 0, 0, 0]);
        let write = frame(&[7, 3, b'a', b'b', b'c']);
        let one = frame(&[7, 1, 0, 0]);
        file.open(&open, &opened, |p| p == b"x")?;
        file.write(&write, &one)?;
        assert_eq!(file.bytes, b"a");
        assert!(file.write(&write, &frame(&[7, 4, 0, 0])).is_err());
        assert!(file.write(&write, &frame(&[7, 1, 0, 28])).is_err());
        assert!(file.write(&write, &frame(&[6, 0, 0, 0])).is_err());
        open.bytes[12] = 1; // append
        file.open(&open, &opened, |p| p == b"x")?;
        file.write(&write, &frame(&[7, 2, 0, 0]))?;
        assert_eq!(file.bytes, b"aab");
        open.bytes[12] = 0;
        file.open(&open, &opened, |p| p == b"x")?;
        let read = frame(&[7, 2]);
        assert!(file.read(&read, &frame(&[7, 0, 0, 2, b'a', b'b'])).is_err());
        file.read(&read, &frame(&[7, 0, 0, 2, b'a', b'a']))?;
        open.bytes[10] = 8; // truncate
        file.open(&open, &frame(&[0, 0, 2, 0]), |_| false)?;
        file.read(&read, &frame(&[7, 0, 0, 1, b'b']))?;
        assert!(file.open(&open, &opened, |p| p == b"y").is_err());
        file.open(&open, &opened, |p| p == b"x")?;
        file.read(&read, &frame(&[7, 0, 0, 0]))?;
        assert!(file.close(&frame(&[7]), &frame(&[6, 0, 0])).is_err());
        Ok(())
    }
}
