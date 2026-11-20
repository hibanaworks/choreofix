use super::preparation::*;
use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    global::{Exit, FileExit, Loan, file_metadata, private_io},
};
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};
pub(super) const ARCHITECT_DATA: u8 = 22;
pub(super) const ARCHITECT_VM: u8 = 23;
pub(super) const ARCHITECT_IO: u8 = 24;
pub(super) const A_DATA: u8 = 25;
pub(super) const A_VM: u8 = 26;
pub(super) const A_IO: u8 = 27;
pub(super) const B_DATA: u8 = 28;
pub(super) const B_VM: u8 = 29;
pub(super) const B_IO: u8 = 30;
pub(super) const C_DATA: u8 = 31;
pub(super) const C_VM: u8 = 32;
pub(super) const C_IO: u8 = 33;
pub(super) const REVIEW_DATA: u8 = 34;
pub(super) const REVIEW_VM: u8 = 35;
pub(super) const REVIEW_IO: u8 = 36;
pub(super) const TEST_DATA: u8 = 37;
pub(super) const TEST_VM: u8 = 38;
pub(super) const TEST_IO: u8 = 39;
pub(super) const DEVOPS_DATA: u8 = 43;
pub(super) const DEVOPS_VM: u8 = 44;
pub(super) const DEVOPS_IO: u8 = 45;
const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        FdBinding {
            read: Some(FdReadRow::Base),
            write: Some(FdWriteRow::Object),
            readdir: None,
        },
    )
}
const fn material(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}
pub(super) const ARCHITECT_FILES: [ChoreoFsObject<'static>; 1] =
    [operation(b"service/design_proposal", 4)];
pub(super) const DEVELOPER_FILES: [ChoreoFsObject<'static>; 4] = [
    material(b"service/plan", 4),
    operation(b"service/implement_feature", 5),
    material(b"service/review_result", 6),
    material(b"service/test_result", 7),
];
pub(super) const REVIEW_FILES: [ChoreoFsObject<'static>; 6] = [
    material(b"service/candidate/microservice_a", 4),
    material(b"service/candidate/microservice_b", 5),
    material(b"service/candidate/microservice_c", 6),
    operation(b"service/code_review/microservice_a", 7),
    operation(b"service/code_review/microservice_b", 8),
    operation(b"service/code_review/microservice_c", 9),
];
pub(super) const DEVOPS_FILES: [ChoreoFsObject<'static>; 4] = [
    material(b"service/plan", 4),
    operation(b"service/migrate_schema/microservice_a", 5),
    operation(b"service/migrate_schema/microservice_b", 6),
    operation(b"service/migrate_schema/microservice_c", 7),
];
pub(super) const TEST_FILES: [ChoreoFsObject<'static>; 6] = [
    material(b"service/candidate/microservice_a", 4),
    material(b"service/candidate/microservice_b", 5),
    material(b"service/candidate/microservice_c", 6),
    operation(b"service/run_tests/microservice_a", 7),
    operation(b"service/run_tests/microservice_b", 8),
    operation(b"service/run_tests/microservice_c", 9),
];
fn metadata<const FILE: u8, const DATA: u8>() -> impl Projectable {
    g::seq(
        file_metadata::<FILE, DATA>().roll(),
        g::send::<FILE, DATA, FileExit>(),
    )
}
fn terminal<const VM: u8, const IO: u8>() -> impl Projectable {
    g::seq(private_io::<VM, IO>(), g::send::<VM, IO, Exit>())
}
pub(super) fn global() -> impl Projectable {
    let files = g::par(
        g::send::<ARCHITECT_DATA, ARCHITECT_VM, Loan<ARCHITECT_FILE>>(),
        g::par(
            g::send::<A_DATA, A_VM, Loan<DEVELOPER_A_FILE>>(),
            g::par(
                g::send::<B_DATA, B_VM, Loan<DEVELOPER_B_FILE>>(),
                g::par(
                    g::send::<C_DATA, C_VM, Loan<DEVELOPER_C_FILE>>(),
                    g::par(
                        g::send::<REVIEW_DATA, REVIEW_VM, Loan<REVIEW_FILE>>(),
                        g::send::<TEST_DATA, TEST_VM, Loan<TEST_FILE>>(),
                    ),
                ),
            ),
        ),
    );
    let io = g::par(
        g::par(
            metadata::<ARCHITECT_FILE, ARCHITECT_DATA>(),
            terminal::<ARCHITECT_VM, ARCHITECT_IO>(),
        ),
        g::par(
            g::par(
                metadata::<DEVELOPER_A_FILE, A_DATA>(),
                terminal::<A_VM, A_IO>(),
            ),
            g::par(
                g::par(
                    metadata::<DEVELOPER_B_FILE, B_DATA>(),
                    terminal::<B_VM, B_IO>(),
                ),
                g::par(
                    g::par(
                        metadata::<DEVELOPER_C_FILE, C_DATA>(),
                        terminal::<C_VM, C_IO>(),
                    ),
                    g::par(
                        g::par(
                            metadata::<REVIEW_FILE, REVIEW_DATA>(),
                            terminal::<REVIEW_VM, REVIEW_IO>(),
                        ),
                        g::par(
                            metadata::<TEST_FILE, TEST_DATA>(),
                            terminal::<TEST_VM, TEST_IO>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::par(
            files,
            g::send::<DEVOPS_DATA, DEVOPS_VM, Loan<DEVOPS_FILE>>(),
        ),
        g::par(
            preparation(),
            g::par(
                io,
                g::par(
                    metadata::<DEVOPS_FILE, DEVOPS_DATA>(),
                    terminal::<DEVOPS_VM, DEVOPS_IO>(),
                ),
            ),
        ),
    )
}

pub(super) async fn architect_guest(
    module: &[u8],
    mut root: Endpoint<'_, ARCHITECT_VM>,
) -> ShellResult<()> {
    let endpoint = root.recv::<Loan<ARCHITECT_FILE>>().await?;
    let [mut request_1, mut result_1] = [(); 2].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(ARCHITECT_FILES[0], endpoint)
        .write_as::<Propose>(&mut request_1)
        .read_as::<Proposed<ARCHITECT_FILE>>(&mut result_1);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    file.into_endpoint().send::<FileExit>(&status).await?;

    root.send::<Exit>(&status).await?;
    Ok(())
}

pub(super) async fn developer_guest<const VM: u8, const FILE: u8, const DEVELOPER: u8>(
    module: &[u8],
    mut root: Endpoint<'_, VM>,
) -> ShellResult<()> {
    let endpoint = root.recv::<Loan<FILE>>().await?;
    let [
        mut result_1,
        mut request_2,
        mut result_2,
        mut result_3,
        mut tested,
    ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(DEVELOPER_FILES[0], endpoint)
        .read_as::<Proposed<DEVELOPER>>(&mut result_1)
        .file(DEVELOPER_FILES[1])?
        .write_as::<Implement<DEVELOPER>>(&mut request_2)
        .read_as::<Implemented>(&mut result_2)
        .file(DEVELOPER_FILES[2])?
        .read_as::<Reviewed<DEVELOPER>>(&mut result_3)
        .file(DEVELOPER_FILES[3])?
        .read_as::<TestReport>(&mut tested);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    file.into_endpoint().send::<FileExit>(&status).await?;

    root.send::<Exit>(&status).await?;
    Ok(())
}

pub(super) async fn review_guest(
    module: &[u8],
    mut root: Endpoint<'_, REVIEW_VM>,
) -> ShellResult<()> {
    let endpoint = root.recv::<Loan<REVIEW_FILE>>().await?;
    let [
        mut result_1,
        mut result_2,
        mut result_3,
        mut request_4,
        mut result_4,
        mut request_5,
        mut result_5,
        mut request_6,
        mut result_6,
    ] = [(); 9].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(REVIEW_FILES[0], endpoint)
        .read_as::<Candidate<SERVICE_A>>(&mut result_1)
        .file(REVIEW_FILES[1])?
        .read_as::<Candidate<SERVICE_B>>(&mut result_2)
        .file(REVIEW_FILES[2])?
        .read_as::<Candidate<SERVICE_C>>(&mut result_3)
        .file(REVIEW_FILES[3])?
        .write_as::<Review<SERVICE_A>>(&mut request_4)
        .read_as::<Reviewed<DEVELOPER_A>>(&mut result_4)
        .file(REVIEW_FILES[4])?
        .write_as::<Review<SERVICE_B>>(&mut request_5)
        .read_as::<Reviewed<DEVELOPER_B>>(&mut result_5)
        .file(REVIEW_FILES[5])?
        .write_as::<Review<SERVICE_C>>(&mut request_6)
        .read_as::<Reviewed<DEVELOPER_C>>(&mut result_6);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    file.into_endpoint().send::<FileExit>(&status).await?;

    root.send::<Exit>(&status).await?;
    Ok(())
}
pub(super) async fn test_guest(module: &[u8], mut root: Endpoint<'_, TEST_VM>) -> ShellResult<()> {
    let endpoint = root.recv::<Loan<TEST_FILE>>().await?;
    let [
        mut result_1,
        mut result_2,
        mut result_3,
        mut request_4,
        mut result_4,
        mut request_5,
        mut result_5,
        mut request_6,
        mut result_6,
    ] = [(); 9].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(TEST_FILES[0], endpoint)
        .read_as::<TestCandidate<SERVICE_A>>(&mut result_1)
        .file(TEST_FILES[1])?
        .read_as::<TestCandidate<SERVICE_B>>(&mut result_2)
        .file(TEST_FILES[2])?
        .read_as::<TestCandidate<SERVICE_C>>(&mut result_3)
        .file(TEST_FILES[3])?
        .write_as::<RunTests<TESTER_A>>(&mut request_4)
        .read_as::<Tested<TESTER_A>>(&mut result_4)
        .file(TEST_FILES[4])?
        .write_as::<RunTests<TESTER_B>>(&mut request_5)
        .read_as::<Tested<TESTER_B>>(&mut result_5)
        .file(TEST_FILES[5])?
        .write_as::<RunTests<TESTER_C>>(&mut request_6)
        .read_as::<Tested<TESTER_C>>(&mut result_6);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    let mut file = file.into_endpoint();
    file.send::<Exit>(&status).await?;
    file.send::<FileExit>(&status).await?;
    root.send::<Exit>(&status).await?;
    Ok(())
}

pub(super) async fn devops_guest(
    module: &[u8],
    mut root: Endpoint<'_, DEVOPS_VM>,
) -> ShellResult<()> {
    let endpoint = root.recv::<Loan<DEVOPS_FILE>>().await?;
    let [
        mut plan,
        mut request_a,
        mut reply_a,
        mut request_b,
        mut reply_b,
        mut request_c,
        mut reply_c,
    ] = [(); 7].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(DEVOPS_FILES[0], endpoint)
        .read_as::<Proposed<DEVOPS_FILE>>(&mut plan)
        .file(DEVOPS_FILES[1])?
        .write_as::<Migrate<SCHEMA_A>>(&mut request_a)
        .read_as::<Migrated<SCHEMA_A>>(&mut reply_a)
        .file(DEVOPS_FILES[2])?
        .write_as::<Migrate<SCHEMA_B>>(&mut request_b)
        .read_as::<Migrated<SCHEMA_B>>(&mut reply_b)
        .file(DEVOPS_FILES[3])?
        .write_as::<Migrate<SCHEMA_C>>(&mut request_c)
        .read_as::<Migrated<SCHEMA_C>>(&mut reply_c);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    file.into_endpoint().send::<FileExit>(&status).await?;
    root.send::<Exit>(&status).await?;
    Ok(())
}
