//! Five autonomous wash guests use their actual operation-file Endpoints.
mod arguments;
pub mod global;
mod imp;
mod localside;
#[cfg(test)]
mod tests;
use crate::shell::ShellResult;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            arch_terminal,
            a_terminal,
            b_terminal,
            review_terminal,
            test_terminal,
        ],
    ) = Bus::open(
        [
            "ARCHITECT",
            "DEVELOPER_A",
            "DEVELOPER_B",
            "REVIEWER",
            "TESTER",
        ],
        [
            (session, REPO),
            (session, TEST_ENV),
            (session, ARCH_FILE),
            (session, A_FILE),
            (session, B_FILE),
            (session, REVIEW_FILE),
            (session, TEST_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        let result = async {
            let mut repo_owner = rv.enter(session, &p.repo_owner)?;
            let mut test_owner = rv.enter(session, &p.test_owner)?;
            let mut architect = rv.enter(session, &p.architect)?;
            let mut reviewer = rv.enter(session, &p.reviewer)?;
            let mut resource_init = rv.enter(session, &p.resource_init)?;
            let mut arch_data = rv.enter(session, &p.arch_data)?;
            let mut a_data = rv.enter(session, &p.a_data)?;
            let mut b_data = rv.enter(session, &p.b_data)?;
            let mut review_data = rv.enter(session, &p.review_data)?;
            let mut test_data = rv.enter(session, &p.test_data)?;
            let mut arch_vm = rv.enter(session, &p.arch_vm)?;
            let mut a_vm = rv.enter(session, &p.a_vm)?;
            let mut b_vm = rv.enter(session, &p.b_vm)?;
            let mut review_vm = rv.enter(session, &p.review_vm)?;
            let mut test_vm = rv.enter(session, &p.test_vm)?;
            let mut arch_io = rv.enter(session, &p.arch_io)?;
            let mut a_io = rv.enter(session, &p.a_io)?;
            let mut b_io = rv.enter(session, &p.b_io)?;
            let mut review_io = rv.enter(session, &p.review_io)?;
            let mut test_io = rv.enter(session, &p.test_io)?;
            let repo = rv.enter(session, &p.repo)?;
            let environment = rv.enter(session, &p.environment)?;
            let arch_file = rv.enter(session, &p.arch_file)?;
            let a_file = rv.enter(session, &p.a_file)?;
            let b_file = rv.enter(session, &p.b_file)?;
            let review_file = rv.enter(session, &p.review_file)?;
            let test_file = rv.enter(session, &p.test_file)?;
            async {
                resource_init.send::<RepoLoan>(repo).await?;
                resource_init.send::<TestLoan>(environment).await?;
            };
            async {
                localside::repository::run(&mut repo_owner, &simulator).await?;
            };
            async {
                localside::tests::run(&mut test_owner, &simulator).await?;
            };
            async {
                localside::architect::run(&mut architect, &simulator).await?;
            };
            async {
                localside::reviews::run(&mut reviewer, &simulator).await?;
            };
            async {
                file_metadata::run(&mut arch_data, arch_file, ARCH_FILES).await?;
            };
            async {
                file_metadata::run(&mut a_data, a_file, A_FILES).await?;
            };
            async {
                file_metadata::run(&mut b_data, b_file, B_FILES).await?;
            };
            async {
                file_metadata::run(&mut review_data, review_file, REVIEW_FILES).await?;
            };
            async {
                file_metadata::run(&mut test_data, test_file, TEST_FILES).await?;
            };
            let (arch_file, a_file, b_file, review_file, test_file) = async {
                async { arch_vm.recv::<Loan<ARCH_FILE>>().await };
                async { a_vm.recv::<Loan<A_FILE>>().await };
                async { b_vm.recv::<Loan<B_FILE>>().await };
                async { review_vm.recv::<Loan<REVIEW_FILE>>().await };
                async { test_vm.recv::<Loan<TEST_FILE>>().await };
            }
            .await?;
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(ARCH_FILES[0], arch_file)
                    .write_as::<DesignA>(&mut request_0)
                    .read_as::<DesignedA>(&mut result_0)
                    .file(ARCH_FILES[1])?
                    .write_as::<DesignB>(&mut request_1)
                    .read_as::<DesignedB>(&mut result_1);
                let files = ChoreoFsFiles::new(arch_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut arch_io,
                    PrivateFs::new(arch_terminal, bus, &ARCH_FILES),
                )
                .await?;
            };
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let mut request_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_2 = vec![0; incoming_capacity];
                let mut reviewed = vec![0; incoming_capacity];
                let mut tested = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(A_FILES[0], a_file)
                    .write_as::<RefactorA>(&mut request_0)
                    .read_as::<RefactoredA>(&mut result_0)
                    .file(A_FILES[1])?
                    .write_as::<RefactorSharedA>(&mut request_1)
                    .read_as::<SharedChangedA>(&mut result_1)
                    .file(A_FILES[2])?
                    .write_as::<MergeA>(&mut request_2)
                    .read_as::<MergedA>(&mut result_2)
                    .file(A_FILES[3])?
                    .read_as::<ReviewFeedbackA>(&mut reviewed)
                    .file(A_FILES[4])?
                    .read_as::<TestFeedbackA>(&mut tested);
                let files = ChoreoFsFiles::new(a_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &A_FILES)).await?;
            };
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let mut request_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_2 = vec![0; incoming_capacity];
                let mut reviewed = vec![0; incoming_capacity];
                let mut tested = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(B_FILES[0], b_file)
                    .write_as::<RefactorB>(&mut request_0)
                    .read_as::<RefactoredB>(&mut result_0)
                    .file(B_FILES[1])?
                    .write_as::<RefactorSharedB>(&mut request_1)
                    .read_as::<SharedChangedB>(&mut result_1)
                    .file(B_FILES[2])?
                    .write_as::<MergeB>(&mut request_2)
                    .read_as::<MergedB>(&mut result_2)
                    .file(B_FILES[3])?
                    .read_as::<ReviewFeedbackB>(&mut reviewed)
                    .file(B_FILES[4])?
                    .read_as::<TestFeedbackB>(&mut tested);
                let files = ChoreoFsFiles::new(b_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &B_FILES)).await?;
            };
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(REVIEW_FILES[0], review_file)
                    .write_as::<ReviewRequestA>(&mut request_0)
                    .read_as::<FileReviewedA>(&mut result_0)
                    .file(REVIEW_FILES[1])?
                    .write_as::<ReviewRequestB>(&mut request_1)
                    .read_as::<FileReviewedB>(&mut result_1);
                let files = ChoreoFsFiles::new(review_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(review_terminal, bus, &REVIEW_FILES),
                )
                .await?;
            };
            async {
                let mut request_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_0 = vec![0; incoming_capacity];
                let mut request_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_1 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(TEST_FILES[0], test_file)
                    .write_as::<TestRequestA>(&mut request_0)
                    .read_as::<FileTestedA>(&mut result_0)
                    .file(TEST_FILES[1])?
                    .write_as::<TestRequestB>(&mut request_1)
                    .read_as::<FileTestedB>(&mut result_1);
                let files = ChoreoFsFiles::new(test_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint().send::<FileExit>(&status).await?;
                root.send::<Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut test_io,
                    PrivateFs::new(test_terminal, bus, &TEST_FILES),
                )
                .await?;
            };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    match &result {
 Ok(())=>io.emit(bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
 Err(error)=>io.emit(bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
 }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
