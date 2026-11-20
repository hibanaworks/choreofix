//! Freely chosen shell commands operate directly on projected operation files.
mod arguments;
pub mod global;
mod imp;
mod localside;
use crate::shell::{ShellError, ShellResult};
use core::cell::Cell;
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    resolver::{DecisionArm, ResolverError, ResolverRef},
};
use serde_json::Value;

fn actual_outcome(input: &Cell<Option<Value>>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let result = match value.as_ref().and_then(|value| value["success"].as_bool()) {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}
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
            a_terminal,
            b_terminal,
            review_terminal,
            test_terminal,
            ci_terminal,
        ],
    ) = Bus::open(
        [
            "DEVELOPER_A",
            "DEVELOPER_B",
            "REVIEWER",
            "TEST_WRITER",
            "CI_RUNNER",
        ],
        [
            (session, REPO),
            (session, MERGE_REQUESTS),
            (session, TEST_ENV),
            (session, BUILD_SERVER),
            (session, REVIEW_A),
            (session, REVIEW_B),
            (session, A_FILE),
            (session, A_MERGE_FILE),
            (session, B_FILE),
            (session, B_MERGE_FILE),
            (session, A_REVIEW_FILE),
            (session, B_REVIEW_FILE),
            (session, TEST_FILE),
            (session, CI_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let review_a_actual = Cell::new(None);
        let review_b_actual = Cell::new(None);
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.review_a,
            ResolverRef::<REVIEW_DECISION_A>::decision_state(&review_a_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_b,
            ResolverRef::<REVIEW_DECISION_B>::decision_state(&review_b_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_a,
            ResolverRef::<REPAIR_DECISION_A>::decision_state(&review_a_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_b,
            ResolverRef::<REPAIR_DECISION_B>::decision_state(&review_b_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_a,
            ResolverRef::<CI_DECISION_A>::decision_state(&review_a_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_b,
            ResolverRef::<CI_DECISION_B>::decision_state(&review_b_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let result = async {
            let mut repo_owner = rv.enter(session, &p.repo_owner)?;
            let mut resource_init = rv.enter(session, &p.resource_init)?;
            let mut review_initial = rv.enter(session, &p.review_initial)?;
            let mut review_repair_a = rv.enter(session, &p.review_repair_a)?;
            let mut review_repair_b = rv.enter(session, &p.review_repair_b)?;
            let review_a = rv.enter(session, &p.review_a)?;
            let review_b = rv.enter(session, &p.review_b)?;
            let mut test_owner = rv.enter(session, &p.test_owner)?;
            let mut ci_owner = rv.enter(session, &p.ci_owner)?;
            let mut a_io = rv.enter(session, &p.a_io)?;
            let mut b_io = rv.enter(session, &p.b_io)?;
            let mut review_io = rv.enter(session, &p.review_io)?;
            let mut test_io = rv.enter(session, &p.test_io)?;
            let mut ci_io = rv.enter(session, &p.ci_io)?;
            let mut a_vm = rv.enter(session, &p.a_vm)?;
            let mut b_vm = rv.enter(session, &p.b_vm)?;
            let mut review_vm = rv.enter(session, &p.review_vm)?;
            let mut test_vm = rv.enter(session, &p.test_vm)?;
            let mut ci_vm = rv.enter(session, &p.ci_vm)?;
            let mut a_data = rv.enter(session, &p.a_data)?;
            let mut a_merge_data = rv.enter(session, &p.a_merge_data)?;
            let a_merge_file = rv.enter(session, &p.a_merge_file)?;
            let mut b_data = rv.enter(session, &p.b_data)?;
            let mut b_merge_data = rv.enter(session, &p.b_merge_data)?;
            let b_merge_file = rv.enter(session, &p.b_merge_file)?;
            let mut a_review_data = rv.enter(session, &p.a_review_data)?;
            let mut b_review_data = rv.enter(session, &p.b_review_data)?;
            let mut test_data = rv.enter(session, &p.test_data)?;
            let mut ci_data = rv.enter(session, &p.ci_data)?;
            let merge_requests = rv.enter(session, &p.merge_requests)?;
            let repo_resource = rv.enter(session, &p.repo)?;
            let environment = rv.enter(session, &p.environment)?;
            let build = rv.enter(session, &p.build)?;
            let a_file = rv.enter(session, &p.a_file)?;
            let b_file = rv.enter(session, &p.b_file)?;
            let a_review_file = rv.enter(session, &p.a_review_file)?;
            let b_review_file = rv.enter(session, &p.b_review_file)?;
            let test_file = rv.enter(session, &p.test_file)?;
            let ci_file = rv.enter(session, &p.ci_file)?;
            async {
                let [mut repo, mut test, mut ci, mut reviews] = resource_init.par()?;
                async {
                    repo.send::<RepoLoan>(repo_resource).await?;
                    repo.send::<crate::shell::global::Loan<MERGE_REQUESTS>>(merge_requests)
                        .await?;
                };
                async {
                    test.send::<TestLoan>(environment).await?;
                };
                async {
                    ci.send::<BuildLoan>(build).await?;
                };
                async {
                    reviews
                        .send::<crate::shell::global::Loan<REVIEW_A>>(review_a)
                        .await?;
                    reviews
                        .send::<crate::shell::global::Loan<REVIEW_B>>(review_b)
                        .await?;
                };
                Ok(())
            }
            .await?;
            async {
                localside::repository::run(&mut repo_owner, &simulator).await?;
            };
            async {
                localside::reviews::initial(
                    &mut review_initial,
                    (&simulator, &review_a_actual, &review_b_actual),
                )
                .await?;
            };
            async {
                localside::tests::run(&mut test_owner, &simulator).await?;
            };
            async {
                let mut build = ci_owner.recv::<BuildLoan>().await?;
                localside::ci::run(&mut build, &simulator).await?;
            };
            async {
                file_metadata::run(&mut a_merge_data, a_merge_file, FEATURE_FILES).await?;
            };
            let a_merge_file = review_repair_a
                .recv::<crate::shell::global::Loan<A_MERGE_FILE>>()
                .await?;
            async {
                localside::reviews::feature_a(&mut review_repair_a, (&simulator, &review_a_actual))
                    .await?;
            };
            async {
                file_metadata::run(&mut b_merge_data, b_merge_file, FEATURE_FILES).await?;
            };
            let b_merge_file = review_repair_b
                .recv::<crate::shell::global::Loan<B_MERGE_FILE>>()
                .await?;
            async {
                localside::reviews::feature_b(&mut review_repair_b, (&simulator, &review_b_actual))
                    .await?;
            };
            async {
                file_metadata::run(&mut a_data, a_file, UPDATE_FILES).await?;
            };
            async {
                file_metadata::run(&mut b_data, b_file, UPDATE_FILES).await?;
            };
            async {
                file_metadata::run(
                    &mut a_review_data,
                    a_review_file,
                    [REVIEW_FILES[0], REVIEW_FILES[2], REVIEW_FILES[4]],
                )
                .await?;
            };
            async {
                file_metadata::run(
                    &mut b_review_data,
                    b_review_file,
                    [REVIEW_FILES[1], REVIEW_FILES[3], REVIEW_FILES[5]],
                )
                .await?;
            };
            async {
                file_metadata::run(&mut test_data, test_file, TEST_FILES).await?;
            };
            async {
                file_metadata::run(&mut ci_data, ci_file, CI_FILES).await?;
            };
            let (a_file, b_file, (a_review_file, b_review_file), test_file, ci_file) = async {
                async { a_vm.recv::<crate::shell::global::Loan<A_FILE>>().await };
                async { b_vm.recv::<crate::shell::global::Loan<B_FILE>>().await };
                async {
                    let [mut a, mut b] = review_vm.par()?;
                    async { a.recv::<crate::shell::global::Loan<A_REVIEW_FILE>>().await };
                    async { b.recv::<crate::shell::global::Loan<B_REVIEW_FILE>>().await };
                };
                async {
                    test_vm
                        .recv::<crate::shell::global::Loan<TEST_FILE>>()
                        .await
                };
                async { ci_vm.recv::<crate::shell::global::Loan<CI_FILE>>().await };
            }
            .await?;
            async {
                let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut implementation = vec![0; incoming_capacity];
                let mut merge = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut merged = vec![0; incoming_capacity];
                let mut reviewed = vec![0; incoming_capacity];
                let mut ready = vec![0; incoming_capacity];
                let mut ci = vec![0; incoming_capacity];
                let mut interface = vec![0; incoming_capacity];
                let mut peer_review = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(UPDATE_FILES[0], a_file)
                    .read_as::<CiUpdate>(&mut ci)
                    .file(UPDATE_FILES[1])?
                    .read_as::<InterfaceB>(&mut interface)
                    .file(UPDATE_FILES[2])?
                    .read_as::<ReviewFeedbackB>(&mut peer_review);
                let merge_file = ChoreoFsFile::new(FEATURE_FILES[0], a_merge_file)
                    .write_as::<ImplementA>(&mut request)
                    .read_as::<ImplementationA>(&mut implementation)
                    .file(FEATURE_FILES[1])?
                    .write_as::<MergeA>(&mut merge)
                    .read_as::<MergedA>(&mut merged)
                    .file(FEATURE_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut reviewed)
                    .file(FEATURE_FILES[3])?
                    .read_as::<MergeReadyA>(&mut ready);
                let files = ChoreoFsFiles::new(a_vm).file(file)?.file(merge_file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (merge_file, (file, ()))) = files.into_parts();
                merge_file
                    .into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
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
                let mut request = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut implementation = vec![0; incoming_capacity];
                let mut merge = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut merged = vec![0; incoming_capacity];
                let mut reviewed = vec![0; incoming_capacity];
                let mut ready = vec![0; incoming_capacity];
                let mut ci = vec![0; incoming_capacity];
                let mut interface = vec![0; incoming_capacity];
                let mut peer_review = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(UPDATE_FILES[0], b_file)
                    .read_as::<CiUpdate>(&mut ci)
                    .file(UPDATE_FILES[1])?
                    .read_as::<InterfaceA>(&mut interface)
                    .file(UPDATE_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut peer_review);
                let merge_file = ChoreoFsFile::new(FEATURE_FILES[0], b_merge_file)
                    .write_as::<ImplementB>(&mut request)
                    .read_as::<ImplementationB>(&mut implementation)
                    .file(FEATURE_FILES[1])?
                    .write_as::<MergeB>(&mut merge)
                    .read_as::<MergedB>(&mut merged)
                    .file(FEATURE_FILES[2])?
                    .read_as::<ReviewFeedbackB>(&mut reviewed)
                    .file(FEATURE_FILES[3])?
                    .read_as::<MergeReadyB>(&mut ready);
                let files = ChoreoFsFiles::new(b_vm).file(file)?.file(merge_file)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (merge_file, (file, ()))) = files.into_parts();
                merge_file
                    .into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
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
                let mut request_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut request_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_a = vec![0; incoming_capacity];
                let mut result_b = vec![0; incoming_capacity];
                let mut ci_a = vec![0; incoming_capacity];
                let mut ci_b = vec![0; incoming_capacity];
                let mut candidate_a = vec![0; incoming_capacity];
                let mut candidate_b = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(REVIEW_FILES[0], a_review_file)
                    .write_as::<ReviewRequestA>(&mut request_a)
                    .read_as::<FileReviewResultA>(&mut result_a)
                    .file(REVIEW_FILES[2])?
                    .read_as::<CiUpdate>(&mut ci_a)
                    .file(REVIEW_FILES[4])?
                    .read_as::<FileReviewCandidateA>(&mut candidate_a);
                let other = ChoreoFsFile::new(REVIEW_FILES[1], b_review_file)
                    .write_as::<ReviewRequestB>(&mut request_b)
                    .read_as::<FileReviewResultB>(&mut result_b)
                    .file(REVIEW_FILES[3])?
                    .read_as::<CiUpdate>(&mut ci_b)
                    .file(REVIEW_FILES[5])?
                    .read_as::<FileReviewCandidateB>(&mut candidate_b);
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
            async {
                let mut request_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut request_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_a = vec![0; incoming_capacity];
                let mut result_b = vec![0; incoming_capacity];
                let mut ci = vec![0; incoming_capacity];
                let mut candidate_a = vec![0; incoming_capacity];
                let mut candidate_b = vec![0; incoming_capacity];
                let mut review_a = vec![0; incoming_capacity];
                let mut review_b = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(TEST_FILES[0], test_file)
                    .write_as::<TestRequestA>(&mut request_a)
                    .read_as::<FileTestedA>(&mut result_a)
                    .file(TEST_FILES[1])?
                    .write_as::<TestRequestB>(&mut request_b)
                    .read_as::<FileTestedB>(&mut result_b)
                    .file(TEST_FILES[2])?
                    .read_as::<CiUpdate>(&mut ci)
                    .file(TEST_FILES[3])?
                    .read_as::<FileTestCandidateA>(&mut candidate_a)
                    .file(TEST_FILES[4])?
                    .read_as::<FileTestCandidateB>(&mut candidate_b)
                    .file(TEST_FILES[5])?
                    .read_as::<ReviewFeedbackA>(&mut review_a)
                    .file(TEST_FILES[6])?
                    .read_as::<ReviewFeedbackB>(&mut review_b);
                let files = ChoreoFsFiles::new(test_vm).file(file)?;
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
                private_io::run(
                    &mut test_io,
                    PrivateFs::new(test_terminal, bus, &TEST_FILES),
                )
                .await?;
            };
            async {
                let mut request_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut request_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut result_a = vec![0; incoming_capacity];
                let mut result_b = vec![0; incoming_capacity];
                let mut published_a = vec![0; incoming_capacity];
                let mut published_b = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(CI_FILES[0], ci_file)
                    .write_as::<CiRequestA>(&mut request_a)
                    .read_as::<FileCiResultA>(&mut result_a)
                    .file(CI_FILES[1])?
                    .write_as::<CiRequestB>(&mut request_b)
                    .read_as::<FileCiResultB>(&mut result_b)
                    .file(CI_FILES[2])?
                    .read_as::<FilePublicationA>(&mut published_a)
                    .file(CI_FILES[3])?
                    .read_as::<FilePublicationB>(&mut published_b);
                let files = ChoreoFsFiles::new(ci_vm).file(file)?;
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
                private_io::run(&mut ci_io, PrivateFs::new(ci_terminal, bus, &CI_FILES)).await?;
            };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    io.finish(bus, result, None).await
}
pub fn projection() -> Value {
    global::programs().observation()
}
