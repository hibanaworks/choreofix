use super::bus::{Bus, Terminal};
use crate::shell::{ShellError, ShellResult};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsWrite, FdSpec, ObjectId},
    protocol::{self, FdBinding, FdReadRow, FdReaddirRow, FdWriteRow},
};

const READ: u64 = 1 << 1;
pub const WRITE: u64 = 1 << 6;
const READDIR: u64 = 1 << 14;
const SERVICE_DIRECTORY_OBJECT: usize = 16;
const PRIVATE_OBJECT_SLOTS: usize = SERVICE_DIRECTORY_OBJECT + 1;
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
    // Guest descriptors are distinct from the stable private object indices.
    // Allocate them around the actual projected files, not a fixed fd range.
    object_fds: [u8; PRIVATE_OBJECT_SLOTS - 6],
    listing: Vec<u8>,
    service_listing: Vec<u8>,
    root_listing: Vec<u8>,
    pub offsets: [usize; PRIVATE_OBJECT_SLOTS],
    pub fd_flags: [u16; PRIVATE_OBJECT_SLOTS],
    pub output: Vec<u8>,
    pub terminal: Terminal,
    pub bus: &'a Bus,
}

impl<'a> PrivateFs<'a> {
    pub fn new(terminal: Terminal, bus: &'a Bus, files: &[ChoreoFsObject<'static>]) -> Self {
        let mut available =
            (4..=u8::MAX).filter(|fd| files.iter().all(|file| file.fd().fd() != *fd));
        let object_fds = std::array::from_fn(|_| {
            available
                .next()
                .expect("private and projected files exceed the guest descriptor domain")
        });
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
            object_fds,
            listing: Vec::new(),
            service_listing,
            root_listing,
            offsets: [0; PRIVATE_OBJECT_SLOTS],
            fd_flags: [0; PRIVATE_OBJECT_SLOTS],
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
                FdSpec::new(self.object_fds[0], READ | WRITE, 1),
                &self.work[0],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(1),
                ObjectId(7),
                FdSpec::new(self.object_fds[1], READ | WRITE, 1),
                &self.work[1],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(2),
                ObjectId(8),
                FdSpec::new(self.object_fds[2], READ | WRITE, 1),
                &self.work[2],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(3),
                ObjectId(9),
                FdSpec::new(self.object_fds[3], READ | WRITE, 1),
                &self.work[3],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(4),
                ObjectId(10),
                FdSpec::new(self.object_fds[4], READ | WRITE, 1),
                &self.work[4],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(5),
                ObjectId(11),
                FdSpec::new(self.object_fds[5], READ | WRITE, 1),
                &self.work[5],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(6),
                ObjectId(12),
                FdSpec::new(self.object_fds[6], READ | WRITE, 1),
                &self.work[6],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(7),
                ObjectId(13),
                FdSpec::new(self.object_fds[7], READ | WRITE, 1),
                &self.work[7],
                READ_WRITE,
            ),
            ChoreoFsObject::readdir(
                b"work",
                ObjectId(14),
                FdSpec::new(self.object_fds[8], READDIR, 1),
                &self.listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            // An opened directory is guest-owned. Its close must not replace
            // or remove the preopen used to resolve subsequent paths.
            ChoreoFsObject::readdir(
                b"",
                ObjectId(15),
                FdSpec::new(self.object_fds[9], READDIR, 1),
                &self.root_listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::readdir(
                b"service",
                ObjectId(SERVICE_DIRECTORY_OBJECT as u32),
                FdSpec::new(self.object_fds[10], READDIR, 1),
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
            Some(ObjectId(object @ 6..=13)) => {
                let index = object as usize;
                let file = &mut self.work[index - 6];
                let start = if self.fd_flags[index] & 1 != 0 {
                    file.len()
                } else {
                    self.offsets[index]
                };
                let Some(end) = start
                    .checked_add(operation.bytes().len())
                    .filter(|end| *end <= crate::shell::GUEST_MEMORY_BYTES)
                else {
                    return Ok(protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(write.fd(), 0, 22),
                    ));
                };
                if end > file.len() {
                    file.resize(end, 0);
                }
                file[start..end].copy_from_slice(operation.bytes());
                self.offsets[index] = end;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_descriptors_exclude_every_possible_projected_descriptor() {
        for reserved in 4..=u8::MAX {
            let files = [ChoreoFsObject::readable(
                b"service/input",
                ObjectId(99),
                FdSpec::new(reserved, READ, 1),
                b"",
                FdBinding::read(FdReadRow::Base),
            )];
            let (bus, _, [terminal]) = Bus::open(["TEST"], []);
            let io = PrivateFs::new(terminal, &bus, &files);
            let descriptors: std::collections::BTreeSet<_> = io.object_fds.into_iter().collect();
            assert_eq!(descriptors.len(), PRIVATE_OBJECT_SLOTS - 6);
            assert!(descriptors.iter().all(|fd| *fd >= 4 && *fd != reserved));
        }
    }

    #[test]
    fn multiple_projected_files_leave_work_and_directory_descriptors_distinct() {
        let files = [4, 5, 6, 7, 8, 255].map(|fd| {
            ChoreoFsObject::readable(
                b"service/input",
                ObjectId(u32::from(fd)),
                FdSpec::new(fd, READ, 1),
                b"",
                FdBinding::read(FdReadRow::Base),
            )
        });
        let (bus, _, [terminal]) = Bus::open(["TEST"], []);
        let io = PrivateFs::new(terminal, &bus, &files);
        let descriptors: std::collections::BTreeSet<_> = io.object_fds.into_iter().collect();
        assert_eq!(descriptors.len(), PRIVATE_OBJECT_SLOTS - 6);
        assert!(descriptors.iter().all(|fd| *fd >= 4));
        assert!(
            files
                .iter()
                .all(|file| !descriptors.contains(&file.fd().fd()))
        );
    }
}
