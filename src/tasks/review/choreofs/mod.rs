//! Freely chosen shell operations use four actual file views and one repository owner.
pub mod global;
mod imp;
pub(crate) mod localside;
#[cfg(test)]
mod tests;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(6);
    let (mut connection, mut io, [a_terminal, b_terminal, review_terminal]) = Bus::open(
        ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"],
        [
            (session, REPO),
            (session, A_FILE),
            (session, B_FILE),
            (session, A_REVIEW_FILE),
            (session, REVIEW_A),
            (session, B_REVIEW_FILE),
            (session, REVIEW_B),
        ],
    );

    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let actual_a = super::imp::ReviewInput::default();
        let actual_b = super::imp::ReviewInput::default();
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.review_owner_a,
            ResolverRef::<DECIDE_A>::decision_state(&actual_a, super::imp::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_owner_b,
            ResolverRef::<DECIDE_B>::decision_state(&actual_b, super::imp::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let result = async {
            let mut repo_owner = rv.enter(session, &p.repo_owner)?;
            let mut repo_init = rv.enter(session, &p.repo_init)?;
            let repository = rv.enter(session, &p.repo)?;
            let mut review_owner_a = rv.enter(session, &p.review_owner_a)?;
            let review_a = rv.enter(session, &p.review_a)?;
            let mut review_owner_b = rv.enter(session, &p.review_owner_b)?;
            let review_b = rv.enter(session, &p.review_b)?;
            let mut a_io = rv.enter(session, &p.a_io)?;
            let mut b_io = rv.enter(session, &p.b_io)?;
            let mut review_io = rv.enter(session, &p.review_io)?;
            let mut a_vm = rv.enter(session, &p.a_vm)?;
            let mut b_vm = rv.enter(session, &p.b_vm)?;
            let mut review_vm = rv.enter(session, &p.review_vm)?;
            let mut a_data = rv.enter(session, &p.a_data)?;
            let mut b_data = rv.enter(session, &p.b_data)?;
            let mut a_review_data = rv.enter(session, &p.a_review_data)?;
            let mut b_review_data = rv.enter(session, &p.b_review_data)?;
            let a_file = rv.enter(session, &p.a_file)?;
            let b_file = rv.enter(session, &p.b_file)?;
            let a_review_file = rv.enter(session, &p.a_review_file)?;
            let b_review_file = rv.enter(session, &p.b_review_file)?;
            repo_init.send::<RepoLoan>(repository).await?;
            async {
                localside::repo::run(&mut repo_owner, &simulator).await?;
            };
            async {
                file_metadata::run(&mut a_data, a_file, DEVELOPER_FILES).await?;
            };
            async {
                file_metadata::run(&mut b_data, b_file, DEVELOPER_FILES).await?;
            };
            async {
                file_metadata::run(&mut a_review_data, a_review_file, [REVIEW_FILES[0]]).await?;
            };
            async {
                file_metadata::run(&mut b_review_data, b_review_file, [REVIEW_FILES[1]]).await?;
            };
            let (a_file, b_file, (a_review_file, b_review_file)) = async {
                async { a_vm.recv::<crate::shell::global::Loan<A_FILE>>().await };
                async { b_vm.recv::<crate::shell::global::Loan<B_FILE>>().await };
                async {
                    let [mut a, mut b] = review_vm.par()?;
                    async { a.recv::<crate::shell::global::Loan<A_REVIEW_FILE>>().await };
                    async { b.recv::<crate::shell::global::Loan<B_REVIEW_FILE>>().await };
                };
            }
            .await?;
            async {
                localside::reviewer::run_a(&mut review_owner_a, review_a, (&simulator, &actual_a))
                    .await?;
            };
            async {
                localside::reviewer::run_b(&mut review_owner_b, review_b, (&simulator, &actual_b))
                    .await?;
            };
            async {
                let mut implementation = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut implementation_reply = vec![0; incoming_capacity];
                let mut merge = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut merge_reply = vec![0; incoming_capacity];
                let mut feedback = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DEVELOPER_FILES[0], a_file)
                    .write_as::<FileImplementA>(&mut implementation)
                    .read_as::<FileImplementationA>(&mut implementation_reply)
                    .file(DEVELOPER_FILES[1])?
                    .write_as::<FileMergeA>(&mut merge)
                    .read_as::<FileMergedA>(&mut merge_reply)
                    .file(DEVELOPER_FILES[2])?
                    .read_as::<FileUpdate<A_FILE>>(&mut feedback);
                let files = ChoreoFsFiles::new(a_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &DEVELOPER_FILES))
                    .await?;
            };
            async {
                let mut implementation = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut implementation_reply = vec![0; incoming_capacity];
                let mut merge = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut merge_reply = vec![0; incoming_capacity];
                let mut feedback = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DEVELOPER_FILES[0], b_file)
                    .write_as::<FileImplementB>(&mut implementation)
                    .read_as::<FileImplementationB>(&mut implementation_reply)
                    .file(DEVELOPER_FILES[1])?
                    .write_as::<FileMergeB>(&mut merge)
                    .read_as::<FileMergedB>(&mut merge_reply)
                    .file(DEVELOPER_FILES[2])?
                    .read_as::<FileUpdate<B_FILE>>(&mut feedback);
                let files = ChoreoFsFiles::new(b_vm).file(file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file, ())) = files.into_parts();
                file.into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &DEVELOPER_FILES))
                    .await?;
            };
            async {
                let mut a_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut a_reply = vec![0; incoming_capacity];
                let mut b_request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut b_reply = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(REVIEW_FILES[0], a_review_file)
                    .write_as::<FileReview<TARGET_A>>(&mut a_request)
                    .read_as::<FileReviewResult<TARGET_A>>(&mut a_reply);
                let other = ChoreoFsFile::new(REVIEW_FILES[1], b_review_file)
                    .write_as::<FileReview<TARGET_B>>(&mut b_request)
                    .read_as::<FileReviewResult<TARGET_B>>(&mut b_reply);
                let files = ChoreoFsFiles::new(review_vm).file(file)?.file(other)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (other, (file, ()))) = files.into_parts();
                file.into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                other
                    .into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(review_terminal, bus, &REVIEW_FILES),
                )
                .await?;
            };
            Ok(())
        }
        .await;
        bus.checkpoint(session.raw(), observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    match &result {
        Ok(()) => io.emit(bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
