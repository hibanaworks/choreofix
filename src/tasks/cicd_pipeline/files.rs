use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

const fn read(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: None,
            readdir: None,
        },
    )
}

const fn write(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}

pub const DEVELOPER: [ChoreoFsObject<'static>; 10] = [
    read(b"service/start", 4),
    write(b"service/compile_code", 5),
    write(b"service/run_unit_tests", 6),
    write(b"service/publish_artifact", 7),
    read(b"service/result", 8),
    read(b"service/revise", 9),
    read(b"service/workspace", 10),
    read(b"service/build_verified", 11),
    read(b"service/qa_done", 12),
    read(b"service/qa_revise", 13),
];
pub const QA: [ChoreoFsObject<'static>; 5] = [
    read(b"service/start", 4),
    write(b"service/pull_artifacts", 5),
    write(b"service/run_integration_tests", 6),
    write(b"service/run_e2e_tests", 7),
    read(b"service/result", 8),
];
pub const RELEASE: [ChoreoFsObject<'static>; 10] = [
    read(b"service/start", 4),
    write(b"service/pull_artifacts", 5),
    write(b"service/deploy_staging", 6),
    write(b"service/run_staging_smoke", 7),
    read(b"service/promote", 8),
    read(b"service/abort", 9),
    write(b"service/deploy_production", 10),
    write(b"service/run_production_smoke", 11),
    read(b"service/staging_ready", 12),
    read(b"service/result", 13),
];
pub const MASTER: [ChoreoFsObject<'static>; 5] = [
    ChoreoFsObject::writable(
        b"service/build_frontend_first",
        ObjectId(4),
        FdSpec::new(4, 64),
        protocol::FdBinding::write(protocol::FdWriteRow::Object),
    ),
    ChoreoFsObject::writable(
        b"service/build_backend_first",
        ObjectId(5),
        FdSpec::new(5, 64),
        protocol::FdBinding::write(protocol::FdWriteRow::Object),
    ),
    read(b"service/result", 6),
    read(b"service/qa_report", 7),
    ChoreoFsObject::writable(
        b"service/repair",
        ObjectId(8),
        FdSpec::new(8, 64),
        protocol::FdBinding::write(protocol::FdWriteRow::Object),
    ),
];
