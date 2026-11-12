use super::bus::{Bus, Terminal};
use crate::shell::{ShellError, ShellResult};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsWrite, FdSpec, ObjectId},
    protocol::{self, FdBinding, FdReadRow, FdReaddirRow, FdWriteRow},
};

const READ: u64 = 1 << 1;
pub const WRITE: u64 = 1 << 6;
const READDIR: u64 = 1 << 14;
const ROOT_DIRECTORY_FD: usize = 16;
const ROOT_FD_SLOTS: usize = ROOT_DIRECTORY_FD + 1;
const READ_WRITE: FdBinding = FdBinding {
    read: Some(FdReadRow::Base),
    write: Some(FdWriteRow::Base),
    readdir: None,
};
// Unbound slots have unrepresentable guest paths. They are not files or
// permissions. A successful private create binds a caller-chosen name.
const UNBOUND_PATHS: [&[u8]; 8] = [
    b"\0slot0", b"\0slot1", b"\0slot2", b"\0slot3", b"\0slot4", b"\0slot5", b"\0slot6", b"\0slot7",
];

// File contents, offsets and flags are filesystem data, never task progress or
// ownership authority. All file operations consume ChoreoFS operation results.
pub struct PrivateFs<'a> {
    pub input: Vec<u8>,
    pub work: [Vec<u8>; 8],
    pub paths: [Vec<u8>; 8],
    listing: Vec<u8>,
    service_listing: Vec<u8>,
    root_listing: Vec<u8>,
    pub offsets: [usize; ROOT_FD_SLOTS],
    pub fd_flags: [u16; ROOT_FD_SLOTS],
    pub output: Vec<u8>,
    pub terminal: Terminal,
    pub bus: &'a Bus,
}

impl<'a> PrivateFs<'a> {
    pub fn new(terminal: Terminal, bus: &'a Bus, files: &[ChoreoFsObject<'static>]) -> Self {
        let mut service_listing = Vec::new();
        for file in files {
            let name = file
                .path()
                .rsplit(|byte| *byte == b'/')
                .next()
                .expect("file name");
            directory_entry(
                &mut service_listing,
                name,
                file.object().0 as u64,
                protocol::WASIP1_FILETYPE_REGULAR_FILE,
            );
        }
        let mut root_listing = Vec::new();
        for (name, inode) in [(b"service".as_slice(), 16), (b"work".as_slice(), 14)] {
            directory_entry(
                &mut root_listing,
                name,
                inode,
                protocol::WASIP1_FILETYPE_DIRECTORY,
            );
        }
        Self {
            input: Vec::new(),
            work: std::array::from_fn(|_| Vec::new()),
            paths: std::array::from_fn(|_| Vec::new()),
            listing: Vec::new(),
            service_listing,
            root_listing,
            offsets: [0; ROOT_FD_SLOTS],
            fd_flags: [0; ROOT_FD_SLOTS],
            output: Vec::new(),
            terminal,
            bus,
        }
    }
}

impl PrivateFs<'_> {
    fn work_path(&self, index: usize) -> &[u8] {
        if self.paths[index].is_empty() {
            UNBOUND_PATHS[index]
        } else {
            &self.paths[index]
        }
    }
    pub fn refresh_listing(&mut self) {
        self.listing.clear();
        for (index, path) in self.paths.iter().enumerate() {
            let Some(name) = path.strip_prefix(b"work/") else {
                continue;
            };
            let next = (self.listing.len() + 24 + name.len()) as u64;
            self.listing.extend_from_slice(&next.to_le_bytes());
            self.listing
                .extend_from_slice(&(index as u64 + 6).to_le_bytes());
            self.listing
                .extend_from_slice(&(name.len() as u32).to_le_bytes());
            self.listing
                .extend_from_slice(&[protocol::WASIP1_FILETYPE_REGULAR_FILE, 0, 0, 0]);
            self.listing.extend_from_slice(name);
        }
    }

    pub fn objects(&self) -> ChoreoFsObjectSet<'_, 15> {
        ChoreoFsObjectSet::new([
            ChoreoFsObject::readable(
                b"llm/stdin",
                ObjectId(0),
                FdSpec::new(0, READ, 1),
                &self.input,
                FdBinding::read(FdReadRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stdout",
                ObjectId(1),
                FdSpec::new(1, WRITE, 1),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stderr",
                ObjectId(2),
                FdSpec::new(2, WRITE, 1),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::readdir(
                b"\0preopen",
                ObjectId(3),
                FdSpec::new(3, READDIR, 1),
                b"",
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::read_write(
                self.work_path(0),
                ObjectId(6),
                FdSpec::new(6, READ | WRITE, 1),
                &self.work[0],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(1),
                ObjectId(7),
                FdSpec::new(7, READ | WRITE, 1),
                &self.work[1],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(2),
                ObjectId(8),
                FdSpec::new(8, READ | WRITE, 1),
                &self.work[2],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(3),
                ObjectId(9),
                FdSpec::new(9, READ | WRITE, 1),
                &self.work[3],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(4),
                ObjectId(10),
                FdSpec::new(10, READ | WRITE, 1),
                &self.work[4],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(5),
                ObjectId(11),
                FdSpec::new(11, READ | WRITE, 1),
                &self.work[5],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(6),
                ObjectId(12),
                FdSpec::new(12, READ | WRITE, 1),
                &self.work[6],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(7),
                ObjectId(13),
                FdSpec::new(13, READ | WRITE, 1),
                &self.work[7],
                READ_WRITE,
            ),
            ChoreoFsObject::readdir(
                b"work",
                ObjectId(14),
                FdSpec::new(14, READDIR, 1),
                &self.listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            // An opened directory is guest-owned. Its close must not replace
            // or remove the preopen used to resolve subsequent paths.
            ChoreoFsObject::readdir(
                b"",
                ObjectId(15),
                FdSpec::new(15, READDIR, 1),
                &self.root_listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::readdir(
                b"service",
                ObjectId(16),
                FdSpec::new(16, READDIR, 1),
                &self.service_listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
        ])
    }

    pub fn write(&mut self, operation: ChoreoFsWrite<'_>) -> ShellResult<protocol::FdWriteDoneRet> {
        let write = operation.request();
        if !operation.is_writable() {
            return Ok(operation.written());
        }
        match operation.object() {
            Some(ObjectId(1 | 2)) => self.output.extend_from_slice(operation.bytes()),
            Some(ObjectId(fd @ 6..=13)) => {
                let fd = fd as usize;
                let file = &mut self.work[fd - 6];
                let start = if self.fd_flags[fd] & 1 != 0 {
                    file.len()
                } else {
                    self.offsets[fd]
                };
                let Some(end) = start
                    .checked_add(operation.bytes().len())
                    .filter(|end| *end <= 8192)
                else {
                    return Ok(protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(write.fd(), 0, 22),
                    ));
                };
                if end > file.len() {
                    file.resize(end, 0);
                }
                file[start..end].copy_from_slice(operation.bytes());
                self.offsets[fd] = end;
            }
            _ => {
                return Err(ShellError::message(
                    "writable object has no answering owner",
                ));
            }
        }
        Ok(operation.written())
    }
}

fn directory_entry(bytes: &mut Vec<u8>, name: &[u8], inode: u64, kind: u8) {
    let next = (bytes.len() + 24 + name.len()) as u64;
    bytes.extend_from_slice(&next.to_le_bytes());
    bytes.extend_from_slice(&inode.to_le_bytes());
    bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&[kind, 0, 0, 0]);
    bytes.extend_from_slice(name);
}
