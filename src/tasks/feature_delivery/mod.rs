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
fn approval_order(input: &Cell<Option<[Value; 2]>>) -> Result<DecisionArm, ResolverError> {
    let values = input.take();
    let result = values
        .as_ref()
        .and_then(|reviews| {
            let receipt = |review: &Value| {
                if review["success"] != true {
                    return None;
                }
                review["data"]["artifact_id"]
                    .as_str()?
                    .strip_prefix("development:")?
                    .parse::<u64>()
                    .ok()
            };
            let a = receipt(&reviews[0])?;
            let b = receipt(&reviews[1])?;
            if a == b {
                None
            } else if a < b {
                Some(DecisionArm::Left)
            } else {
                Some(DecisionArm::Right)
            }
        })
        .ok_or_else(ResolverError::reject);
    input.set(values);
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
        bus,
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
            (session, TEST_ENV),
            (session, BUILD_SERVER),
            (session, A_FILE),
            (session, B_FILE),
            (session, A_REVIEW_FILE),
            (session, B_REVIEW_FILE),
            (session, TEST_FILE),
            (session, CI_FILE),
        ],
    );
    let participants = async {
        let p = programs();
        bus.observe(p.observation());
        let order = Cell::new(None);
        let review_a_actual = Cell::new(None);
        let review_b_actual = Cell::new(None);
        let ci_actual = Cell::new(None);
        let capture = |event| {
            bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}))
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        rv.set_resolver(
            &p.repo,
            ResolverRef::<MERGE_ORDER>::decision_state(&order, approval_order),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
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
            &p.build,
            ResolverRef::<CI_DECISION_A>::decision_state(&ci_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.build,
            ResolverRef::<CI_DECISION_B>::decision_state(&ci_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let result = async {
            let mut repo_owner = rv.enter(session, &p.repo_owner)?;
            let mut resource_init = rv.enter(session, &p.resource_init)?;
            let mut review_a = rv.enter(session, &p.review_a)?;
            let mut review_b = rv.enter(session, &p.review_b)?;
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
            let mut b_data = rv.enter(session, &p.b_data)?;
            let mut a_review_data = rv.enter(session, &p.a_review_data)?;
            let mut b_review_data = rv.enter(session, &p.b_review_data)?;
            let mut test_data = rv.enter(session, &p.test_data)?;
            let mut ci_data = rv.enter(session, &p.ci_data)?;
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
                let [mut repo, mut test, mut ci] = resource_init.par()?;
                async {
                    repo.send::<RepoLoan>(repo_resource).await?;
                };
                async {
                    test.send::<TestLoan>(environment).await?;
                };
                async {
                    ci.send::<BuildLoan>(build).await?;
                };
                Ok(())
            }
            .await?;
            async {
                localside::repository::run(&mut repo_owner, (&bus, &order)).await?;
            };
            async {
                localside::reviews::feature_a(&mut review_a, (&bus, &review_a_actual)).await?;
            };
            async {
                localside::reviews::feature_b(&mut review_b, (&bus, &review_b_actual)).await?;
            };
            async {
                localside::tests::run(&mut test_owner, &bus).await?;
            };
            async {
                localside::ci::run(&mut ci_owner, (&bus, &ci_actual)).await?;
            };
            async {
                file_metadata::run(&mut a_data, a_file, DEVELOPER_FILES).await?;
            };
            async {
                file_metadata::run(&mut b_data, b_file, DEVELOPER_FILES).await?;
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
                let file = ChoreoFsFile::new(DEVELOPER_FILES[0], a_file)
                    .write_as::<ImplementA>(&mut request)
                    .read_as::<ImplementationA>(&mut implementation)
                    .file(DEVELOPER_FILES[1])?
                    .write_as::<MergeA>(&mut merge)
                    .read_as::<MergedA>(&mut merged)
                    .file(DEVELOPER_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut reviewed)
                    .file(DEVELOPER_FILES[3])?
                    .read_as::<MergeReadyA>(&mut ready)
                    .file(DEVELOPER_FILES[4])?
                    .read_as::<CiUpdate>(&mut ci)
                    .file(DEVELOPER_FILES[5])?
                    .read_as::<InterfaceB>(&mut interface)
                    .file(DEVELOPER_FILES[6])?
                    .read_as::<ReviewFeedbackB>(&mut peer_review);
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
                private_io::run(
                    &mut a_io,
                    PrivateFs::new(a_terminal, &bus, &DEVELOPER_FILES),
                )
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
                let file = ChoreoFsFile::new(DEVELOPER_FILES[0], b_file)
                    .write_as::<ImplementB>(&mut request)
                    .read_as::<ImplementationB>(&mut implementation)
                    .file(DEVELOPER_FILES[1])?
                    .write_as::<MergeB>(&mut merge)
                    .read_as::<MergedB>(&mut merged)
                    .file(DEVELOPER_FILES[2])?
                    .read_as::<ReviewFeedbackB>(&mut reviewed)
                    .file(DEVELOPER_FILES[3])?
                    .read_as::<MergeReadyB>(&mut ready)
                    .file(DEVELOPER_FILES[4])?
                    .read_as::<CiUpdate>(&mut ci)
                    .file(DEVELOPER_FILES[5])?
                    .read_as::<InterfaceA>(&mut interface)
                    .file(DEVELOPER_FILES[6])?
                    .read_as::<ReviewFeedbackA>(&mut peer_review);
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
                private_io::run(
                    &mut b_io,
                    PrivateFs::new(b_terminal, &bus, &DEVELOPER_FILES),
                )
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
                    PrivateFs::new(review_terminal, &bus, &REVIEW_FILES),
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
                    PrivateFs::new(test_terminal, &bus, &TEST_FILES),
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
                private_io::run(&mut ci_io, PrivateFs::new(ci_terminal, &bus, &CI_FILES)).await?;
            };
            Ok(())
        }
        .await;
        bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
        result
    };
    let result = hibana_runtime::race(participants, io.run(&bus)).await;
    match &result {
        Ok(()) => io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> Value {
    global::programs().observation()
}
