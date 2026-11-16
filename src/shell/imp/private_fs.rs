use super::bus::{Bus, Terminal};
use crate::shell::{ShellError, ShellResult};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsWrite, FdSpec, ObjectId},
    protocol::{self, FdBinding, FdReadRow, FdReaddirRow, FdWriteRow},
};

const READ: u64 = 1 << 1;
pub const WRITE: u64 = 1 << 6;
const READDIR: u64 = 1 << 14;
const DIRECTORY_OBJECT_START: usize = 14;
const ROOT_DIRECTORY: usize = 1;
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

// File contents, paths, descriptors and offsets are filesystem data. Every
// operation still consumes its admitted Endpoint request through ChoreoFS.
pub struct PrivateFs<'a> {
    pub input: Vec<u8>,
    pub work: [Vec<u8>; 8],
    pub paths: [Vec<u8>; 8],
    projected: &'a [ChoreoFsObject<'static>],
    object_fds: Vec<u8>,
    directories: Vec<(Vec<u8>, Vec<u8>)>,
    pub offsets: Vec<usize>,
    pub fd_flags: Vec<u16>,
    pub output: Vec<u8>,
    pub terminal: Terminal,
    pub bus: &'a Bus,
}

impl<'a> PrivateFs<'a> {
    pub fn new(terminal: Terminal, bus: &'a Bus, files: &'a [ChoreoFsObject<'static>]) -> Self {
        let mut directories = vec![(b"work".to_vec(), Vec::new()), (Vec::new(), Vec::new())];
        for file in files {
            for (at, _) in file
                .path()
                .iter()
                .enumerate()
                .filter(|(_, byte)| **byte == b'/')
            {
                let path = &file.path()[..at];
                if !path.is_empty() && !directories.iter().any(|(existing, _)| existing == path) {
                    directories.push((path.to_vec(), Vec::new()));
                }
            }
        }
        let mut available =
            (4..=u8::MAX).filter(|fd| files.iter().all(|file| file.fd().fd() != *fd));
        let object_fds = (0..UNBOUND_PATHS.len() + directories.len())
            .map(|_| {
                available
                    .next()
                    .expect("private and projected objects exceed the descriptor domain")
            })
            .collect();
        let count = DIRECTORY_OBJECT_START + directories.len();
        let mut fs = Self {
            input: Vec::new(),
            work: std::array::from_fn(|_| Vec::new()),
            paths: std::array::from_fn(|_| Vec::new()),
            projected: files,
            object_fds,
            directories,
            offsets: vec![0; count],
            fd_flags: vec![0; count],
            output: Vec::new(),
            terminal,
            bus,
        };
        fs.refresh_listing();
        fs
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
        for directory in 0..self.directories.len() {
            let parent = &self.directories[directory].0;
            let entries = self
                .projected
                .iter()
                .map(|file| (file.path(), file.fd().fd(), file.file_type()))
                .chain(
                    self.paths
                        .iter()
                        .enumerate()
                        .filter(|(_, path)| !path.is_empty())
                        .map(|(index, path)| {
                            (
                                path.as_slice(),
                                self.object_fds[index],
                                protocol::WASIP1_FILETYPE_REGULAR_FILE,
                            )
                        }),
                )
                .chain(
                    self.directories
                        .iter()
                        .enumerate()
                        .map(|(index, (path, _))| {
                            (
                                path.as_slice(),
                                self.object_fds[UNBOUND_PATHS.len() + index],
                                protocol::WASIP1_FILETYPE_DIRECTORY,
                            )
                        }),
                );
            let mut children = Vec::new();
            for (path, fd, kind) in entries {
                let name = if parent.is_empty() {
                    path
                } else {
                    let Some(name) = path
                        .strip_prefix(parent.as_slice())
                        .and_then(|name| name.strip_prefix(b"/"))
                    else {
                        continue;
                    };
                    name
                };
                if name.is_empty() || name.contains(&b'/') {
                    continue;
                }
                children.push((name.to_vec(), fd, kind));
            }
            children.sort_by(|left, right| left.0.cmp(&right.0));
            let mut listing = Vec::new();
            let mut previous: Option<&[u8]> = None;
            for (name, fd, kind) in &children {
                assert_ne!(previous, Some(name.as_slice()), "duplicate filesystem path");
                directory_entry(&mut listing, name, u64::from(*fd), *kind);
                previous = Some(name);
            }
            self.directories[directory].1 = listing;
        }
    }

    pub fn objects(&self) -> ChoreoFsObjectSet<Vec<ChoreoFsObject<'_>>> {
        let mut objects = vec![
            ChoreoFsObject::readable(
                b"llm/stdin",
                ObjectId(0),
                FdSpec::new(0, READ),
                &self.input,
                FdBinding::read(FdReadRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stdout",
                ObjectId(1),
                FdSpec::new(1, WRITE),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stderr",
                ObjectId(2),
                FdSpec::new(2, WRITE),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::readdir(
                b"\0preopen",
                ObjectId(3),
                FdSpec::new(3, READDIR),
                &self.directories[ROOT_DIRECTORY].1,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
        ];
        for index in 0..UNBOUND_PATHS.len() {
            objects.push(ChoreoFsObject::read_write(
                self.work_path(index),
                ObjectId(6 + index as u32),
                FdSpec::new(self.object_fds[index], READ | WRITE),
                &self.work[index],
                READ_WRITE,
            ));
        }
        for (index, (path, listing)) in self.directories.iter().enumerate() {
            objects.push(ChoreoFsObject::readdir(
                path,
                ObjectId((DIRECTORY_OBJECT_START + index) as u32),
                FdSpec::new(self.object_fds[UNBOUND_PATHS.len() + index], READDIR),
                listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ));
        }
        ChoreoFsObjectSet::new(objects)
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
                FdSpec::new(reserved, READ),
                b"",
                FdBinding::read(FdReadRow::Base),
            )];
            let (bus, _, [terminal]) = Bus::open(["TEST"], []);
            let io = PrivateFs::new(terminal, &bus, &files);
            let descriptors: std::collections::BTreeSet<_> = io.object_fds.into_iter().collect();
            assert_eq!(
                descriptors.len(),
                UNBOUND_PATHS.len() + io.directories.len()
            );
            assert!(descriptors.iter().all(|fd| *fd >= 4 && *fd != reserved));
        }
    }

    #[test]
    fn multiple_projected_files_leave_work_and_directory_descriptors_distinct() {
        let files = [
            (4, b"service/input0"),
            (5, b"service/input1"),
            (6, b"service/input2"),
            (7, b"service/input3"),
            (8, b"service/input4"),
            (255, b"service/input5"),
        ]
        .map(|(fd, path)| {
            ChoreoFsObject::readable(
                path,
                ObjectId(u32::from(fd)),
                FdSpec::new(fd, READ),
                b"",
                FdBinding::read(FdReadRow::Base),
            )
        });
        let (bus, _, [terminal]) = Bus::open(["TEST"], []);
        let io = PrivateFs::new(terminal, &bus, &files);
        let descriptors: std::collections::BTreeSet<_> = io.object_fds.into_iter().collect();
        assert_eq!(
            descriptors.len(),
            UNBOUND_PATHS.len() + io.directories.len()
        );
        assert!(descriptors.iter().all(|fd| *fd >= 4));
        assert!(
            files
                .iter()
                .all(|file| !descriptors.contains(&file.fd().fd()))
        );
    }

    fn entries(bytes: &[u8]) -> Vec<(&[u8], u64, u8)> {
        let mut result = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            let next = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize;
            let inode = u64::from_le_bytes(bytes[at + 8..at + 16].try_into().unwrap());
            let len = u32::from_le_bytes(bytes[at + 16..at + 20].try_into().unwrap()) as usize;
            assert_eq!(next, at + 24 + len);
            result.push((&bytes[at + 24..next], inode, bytes[at + 20]));
            at = next;
        }
        result
    }

    #[test]
    fn projected_paths_expose_each_parent_and_only_its_immediate_children() {
        let files = [
            (b"service/review/a".as_slice(), 61),
            (b"service/review/b".as_slice(), 62),
            (b"service/interface/a".as_slice(), 63),
            (b"service/implement".as_slice(), 64),
        ]
        .map(|(path, fd)| {
            ChoreoFsObject::new(path, ObjectId(u32::from(fd)), FdSpec::new(fd, READ))
        });
        let (bus, _, [terminal]) = Bus::open(["TEST"], []);
        let io = PrivateFs::new(terminal, &bus, &files);
        let objects = io.objects();
        let fs = objects.choreofs();
        for (path, names) in [
            (
                b"".as_slice(),
                vec![(b"service".as_slice(), 3), (b"work".as_slice(), 3)],
            ),
            (
                b"service".as_slice(),
                vec![
                    (b"implement".as_slice(), 4),
                    (b"interface".as_slice(), 3),
                    (b"review".as_slice(), 3),
                ],
            ),
            (
                b"service/review".as_slice(),
                vec![(b"a".as_slice(), 4), (b"b".as_slice(), 4)],
            ),
            (b"service/interface".as_slice(), vec![(b"a".as_slice(), 4)]),
        ] {
            let (index, (_, listing)) = io
                .directories
                .iter()
                .enumerate()
                .find(|(_, (name, _))| name == path)
                .unwrap();
            assert_eq!(
                fs.resolve(path),
                Some(ObjectId((DIRECTORY_OBJECT_START + index) as u32))
            );
            let actual = entries(listing);
            assert_eq!(
                actual
                    .iter()
                    .map(|(name, _, kind)| (*name, *kind))
                    .collect::<Vec<_>>(),
                names
            );
            for (name, inode, _) in actual {
                let mut child_path = path.to_vec();
                if !path.is_empty() {
                    child_path.push(b'/');
                }
                child_path.extend_from_slice(name);
                let child = io
                    .projected
                    .iter()
                    .find(|file| file.path() == child_path)
                    .map(|file| file.fd().fd())
                    .unwrap_or_else(|| {
                        let child = io
                            .directories
                            .iter()
                            .position(|(directory, _)| *directory == child_path)
                            .unwrap();
                        io.object_fds[UNBOUND_PATHS.len() + child]
                    });
                assert_eq!(inode, u64::from(child));
            }
        }
    }

    #[test]
    fn creating_a_private_file_preserves_the_projected_directory_tree() {
        let files = [ChoreoFsObject::new(
            b"service/review/a",
            ObjectId(99),
            FdSpec::new(4, READ),
        )];
        let (bus, _, [terminal]) = Bus::open(["TEST"], []);
        let mut io = PrivateFs::new(terminal, &bus, &files);
        io.paths[0] = b"work/request.json".to_vec();
        io.refresh_listing();
        let work = &io.directories[0].1;
        assert_eq!(
            entries(work),
            vec![(
                b"request.json".as_slice(),
                u64::from(io.object_fds[0]),
                protocol::WASIP1_FILETYPE_REGULAR_FILE
            )]
        );
        let objects = io.objects();
        let fs = objects.choreofs();
        assert_eq!(fs.resolve(b"work/request.json"), Some(ObjectId(6)));
        assert!(fs.resolve(b"service/review").is_some());
        assert!(fs.resolve(b"service/review/a").is_none());
    }
}
