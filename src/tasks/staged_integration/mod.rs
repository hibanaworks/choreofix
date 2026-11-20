//! Direct staged-integration source and staging effect receiver.
//! Seven-Wasm execution is connected; full real-model task qualification remains incomplete.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod ci;
    pub mod repository;
    pub mod reviews;
    pub mod staging;
    pub mod tests;
}
use core::cell::Cell;
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;

pub fn actual_outcome(input: &Cell<Option<Value>>) -> Result<DecisionArm, ResolverError> {
    let value = input.take();
    let decision = match value.as_ref().and_then(|value| value["success"].as_bool()) {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    decision
}

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> crate::shell::ShellResult<()> {
    use crate::shell::{
        ShellError,
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            a_terminal,
            b_terminal,
            c_terminal,
            review_terminal,
            test_terminal,
            ci_terminal,
            release_terminal,
        ],
    ) = Bus::open(
        [
            "DEVELOPER_A",
            "DEVELOPER_B",
            "DEVELOPER_C",
            "REVIEWER",
            "TEST_WRITER",
            "CI_RUNNER",
            "RELEASE_MANAGER",
        ],
        [
            (session, REPO),
            (session, MERGE_REQUESTS),
            (session, TEST_ENV),
            (session, BUILD_SERVER),
            (session, STAGING_ENV),
            (session, A_FILE),
            (session, B_FILE),
            (session, C_FILE),
            (session, A_REVIEW_FILE),
            (session, B_REVIEW_FILE),
            (session, C_REVIEW_FILE),
            (session, TEST_FILE),
            (session, CI_FILE),
            (session, RELEASE_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let p = programs();
        bus.projected(p.observation());
        let review_a_actual = Cell::new(None);
        let review_b_actual = Cell::new(None);
        let review_c_actual = Cell::new(None);
        let ci_actual = Cell::new(None);
        let stage_actual = Cell::new(None);
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
            &p.build,
            ResolverRef::<CI_DECISION_A>::decision_state(&ci_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.staging,
            ResolverRef::<STAGE_DECISION_A>::decision_state(&stage_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_b,
            ResolverRef::<REVIEW_DECISION_B>::decision_state(&review_b_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.build,
            ResolverRef::<CI_DECISION_B>::decision_state(&ci_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.staging,
            ResolverRef::<STAGE_DECISION_B>::decision_state(&stage_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_c,
            ResolverRef::<REVIEW_DECISION_C>::decision_state(&review_c_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.build,
            ResolverRef::<CI_DECISION_C>::decision_state(&ci_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.staging,
            ResolverRef::<STAGE_DECISION_C>::decision_state(&stage_actual, actual_outcome),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let result = async {
            let mut repo_owner = rv.enter(session, &p.repo_owner)?;
            let mut test_owner = rv.enter(session, &p.test_owner)?;
            let mut ci_owner = rv.enter(session, &p.ci_owner)?;
            let mut stage_owner = rv.enter(session, &p.stage_owner)?;
            let mut stage_owner_b = rv.enter(session, &p.stage_owner_b)?;
            let mut stage_owner_c = rv.enter(session, &p.stage_owner_c)?;
            let mut review_a = rv.enter(session, &p.review_a)?;
            let mut review_b = rv.enter(session, &p.review_b)?;
            let mut review_c = rv.enter(session, &p.review_c)?;
            let mut resource_init = rv.enter(session, &p.resource_init)?;
            let mut a_io = rv.enter(session, &p.a_io)?;
            let mut b_io = rv.enter(session, &p.b_io)?;
            let mut c_io = rv.enter(session, &p.c_io)?;
            let mut review_io = rv.enter(session, &p.review_io)?;
            let mut test_io = rv.enter(session, &p.test_io)?;
            let mut ci_io = rv.enter(session, &p.ci_io)?;
            let mut release_io = rv.enter(session, &p.release_io)?;
            let mut a_vm = rv.enter(session, &p.a_vm)?;
            let mut b_vm = rv.enter(session, &p.b_vm)?;
            let mut c_vm = rv.enter(session, &p.c_vm)?;
            let mut review_vm = rv.enter(session, &p.review_vm)?;
            let mut test_vm = rv.enter(session, &p.test_vm)?;
            let mut ci_vm = rv.enter(session, &p.ci_vm)?;
            let mut release_vm = rv.enter(session, &p.release_vm)?;
            let mut a_data = rv.enter(session, &p.a_data)?;
            let mut b_data = rv.enter(session, &p.b_data)?;
            let mut c_data = rv.enter(session, &p.c_data)?;
            let mut a_review_data = rv.enter(session, &p.a_review_data)?;
            let mut b_review_data = rv.enter(session, &p.b_review_data)?;
            let mut c_review_data = rv.enter(session, &p.c_review_data)?;
            let mut test_data = rv.enter(session, &p.test_data)?;
            let mut ci_data = rv.enter(session, &p.ci_data)?;
            let mut release_data = rv.enter(session, &p.release_data)?;
            let repo_resource = rv.enter(session, &p.repo)?;
            let merge_requests = rv.enter(session, &p.merge_requests)?;
            let environment = rv.enter(session, &p.environment)?;
            let build = rv.enter(session, &p.build)?;
            let staging = rv.enter(session, &p.staging)?;
            let a_file = rv.enter(session, &p.a_file)?;
            let b_file = rv.enter(session, &p.b_file)?;
            let c_file = rv.enter(session, &p.c_file)?;
            let a_review_file = rv.enter(session, &p.a_review_file)?;
            let b_review_file = rv.enter(session, &p.b_review_file)?;
            let c_review_file = rv.enter(session, &p.c_review_file)?;
            let test_file = rv.enter(session, &p.test_file)?;
            let ci_file = rv.enter(session, &p.ci_file)?;
            let release_file = rv.enter(session, &p.release_file)?;
            async {
                let [mut repo, mut test, mut ci, mut stage] = resource_init.par()?;
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
                    stage.send::<StageLoan>(staging).await?;
                };
                Ok(())
            }
            .await?;
            async {
                localside::repository::run(&mut repo_owner, &simulator).await?;
            };
            async {
                localside::tests::run(&mut test_owner, &simulator).await?;
            };
            async {
                localside::ci::run(&mut ci_owner, (&simulator, &ci_actual)).await?;
            };
            async {
                localside::staging::run_a(&mut stage_owner, (&simulator, &stage_actual)).await?;
            };
            async {
                localside::staging::run_b(&mut stage_owner_b, (&simulator, &stage_actual)).await?;
            };
            async {
                localside::staging::run_c(&mut stage_owner_c, (&simulator, &stage_actual)).await?;
            };
            async {
                localside::reviews::feature_a(&mut review_a, (&simulator, &review_a_actual))
                    .await?;
            };
            async {
                localside::reviews::feature_b(&mut review_b, (&simulator, &review_b_actual))
                    .await?;
            };
            async {
                localside::reviews::feature_c(&mut review_c, (&simulator, &review_c_actual))
                    .await?;
            };
            async {
                file_metadata::run(&mut a_data, a_file, DEVELOPER_A_FILES).await?;
            };
            async {
                file_metadata::run(&mut b_data, b_file, DEVELOPER_B_FILES).await?;
            };
            async {
                file_metadata::run(&mut c_data, c_file, DEVELOPER_C_FILES).await?;
            };
            async {
                file_metadata::run(
                    &mut a_review_data,
                    a_review_file,
                    [
                        REVIEW_FILES[0],
                        REVIEW_FILES[3],
                        REVIEW_FILES[6],
                        REVIEW_FILES[9],
                    ],
                )
                .await?;
            };
            async {
                file_metadata::run(
                    &mut b_review_data,
                    b_review_file,
                    [
                        REVIEW_FILES[1],
                        REVIEW_FILES[4],
                        REVIEW_FILES[7],
                        REVIEW_FILES[10],
                    ],
                )
                .await?;
            };
            async {
                file_metadata::run(
                    &mut c_review_data,
                    c_review_file,
                    [
                        REVIEW_FILES[2],
                        REVIEW_FILES[5],
                        REVIEW_FILES[8],
                        REVIEW_FILES[11],
                    ],
                )
                .await?;
            };
            async {
                file_metadata::run(&mut test_data, test_file, TEST_FILES).await?;
            };
            async {
                file_metadata::run(&mut ci_data, ci_file, CI_FILES).await?;
            };
            async {
                file_metadata::run(&mut release_data, release_file, RELEASE_FILES).await?;
            };
            let (
                a_file,
                b_file,
                c_file,
                (a_review_file, b_review_file, c_review_file),
                test_file,
                ci_file,
                release_file,
            ) = async {
                async { a_vm.recv::<crate::shell::global::Loan<A_FILE>>().await };
                async { b_vm.recv::<crate::shell::global::Loan<B_FILE>>().await };
                async { c_vm.recv::<crate::shell::global::Loan<C_FILE>>().await };
                async {
                    let [mut a, mut b, mut c] = review_vm.par()?;
                    async { a.recv::<crate::shell::global::Loan<A_REVIEW_FILE>>().await };
                    async { b.recv::<crate::shell::global::Loan<B_REVIEW_FILE>>().await };
                    async { c.recv::<crate::shell::global::Loan<C_REVIEW_FILE>>().await };
                };
                async {
                    test_vm
                        .recv::<crate::shell::global::Loan<TEST_FILE>>()
                        .await
                };
                async { ci_vm.recv::<crate::shell::global::Loan<CI_FILE>>().await };
                async {
                    release_vm
                        .recv::<crate::shell::global::Loan<RELEASE_FILE>>()
                        .await
                };
            }
            .await?;
            async {
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_read_8 = vec![0; incoming_capacity];
                let mut file_read_9 = vec![0; incoming_capacity];
                let mut file_read_10 = vec![0; incoming_capacity];
                let mut file_read_11 = vec![0; incoming_capacity];
                let mut file_read_12 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DEVELOPER_A_FILES[0], a_file)
                    .write_as::<ImplementA>(&mut file_write_0)
                    .read_as::<ImplementationA>(&mut file_read_0)
                    .file(DEVELOPER_A_FILES[1])?
                    .write_as::<MergeRequestA>(&mut file_write_1)
                    .read_as::<MergedA>(&mut file_read_1)
                    .file(DEVELOPER_A_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut file_read_2)
                    .file(DEVELOPER_A_FILES[3])?
                    .read_as::<ReviewFeedbackB>(&mut file_read_3)
                    .file(DEVELOPER_A_FILES[4])?
                    .read_as::<ReviewFeedbackC>(&mut file_read_4)
                    .file(DEVELOPER_A_FILES[5])?
                    .read_as::<CiUpdateA>(&mut file_read_5)
                    .file(DEVELOPER_A_FILES[6])?
                    .read_as::<CiUpdateB>(&mut file_read_6)
                    .file(DEVELOPER_A_FILES[7])?
                    .read_as::<CiUpdateC>(&mut file_read_7)
                    .file(DEVELOPER_A_FILES[8])?
                    .read_as::<StageUpdateA>(&mut file_read_8)
                    .file(DEVELOPER_A_FILES[9])?
                    .read_as::<StageUpdateB>(&mut file_read_9)
                    .file(DEVELOPER_A_FILES[10])?
                    .read_as::<StageUpdateC>(&mut file_read_10)
                    .file(DEVELOPER_A_FILES[11])?
                    .read_as::<InterfaceBToA>(&mut file_read_11)
                    .file(DEVELOPER_A_FILES[12])?
                    .read_as::<InterfaceCToA>(&mut file_read_12);
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
                    PrivateFs::new(a_terminal, bus, &DEVELOPER_A_FILES),
                )
                .await?;
            };
            async {
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_read_8 = vec![0; incoming_capacity];
                let mut file_read_9 = vec![0; incoming_capacity];
                let mut file_read_10 = vec![0; incoming_capacity];
                let mut file_read_11 = vec![0; incoming_capacity];
                let mut file_read_12 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DEVELOPER_B_FILES[0], b_file)
                    .write_as::<ImplementB>(&mut file_write_0)
                    .read_as::<ImplementationB>(&mut file_read_0)
                    .file(DEVELOPER_B_FILES[1])?
                    .write_as::<MergeRequestB>(&mut file_write_1)
                    .read_as::<MergedB>(&mut file_read_1)
                    .file(DEVELOPER_B_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut file_read_2)
                    .file(DEVELOPER_B_FILES[3])?
                    .read_as::<ReviewFeedbackB>(&mut file_read_3)
                    .file(DEVELOPER_B_FILES[4])?
                    .read_as::<ReviewFeedbackC>(&mut file_read_4)
                    .file(DEVELOPER_B_FILES[5])?
                    .read_as::<CiUpdateA>(&mut file_read_5)
                    .file(DEVELOPER_B_FILES[6])?
                    .read_as::<CiUpdateB>(&mut file_read_6)
                    .file(DEVELOPER_B_FILES[7])?
                    .read_as::<CiUpdateC>(&mut file_read_7)
                    .file(DEVELOPER_B_FILES[8])?
                    .read_as::<StageUpdateA>(&mut file_read_8)
                    .file(DEVELOPER_B_FILES[9])?
                    .read_as::<StageUpdateB>(&mut file_read_9)
                    .file(DEVELOPER_B_FILES[10])?
                    .read_as::<StageUpdateC>(&mut file_read_10)
                    .file(DEVELOPER_B_FILES[11])?
                    .read_as::<InterfaceAToB>(&mut file_read_11)
                    .file(DEVELOPER_B_FILES[12])?
                    .read_as::<InterfaceCToB>(&mut file_read_12);
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
                    PrivateFs::new(b_terminal, bus, &DEVELOPER_B_FILES),
                )
                .await?;
            };
            async {
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_read_8 = vec![0; incoming_capacity];
                let mut file_read_9 = vec![0; incoming_capacity];
                let mut file_read_10 = vec![0; incoming_capacity];
                let mut file_read_11 = vec![0; incoming_capacity];
                let mut file_read_12 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(DEVELOPER_C_FILES[0], c_file)
                    .write_as::<ImplementC>(&mut file_write_0)
                    .read_as::<ImplementationC>(&mut file_read_0)
                    .file(DEVELOPER_C_FILES[1])?
                    .write_as::<MergeRequestC>(&mut file_write_1)
                    .read_as::<MergedC>(&mut file_read_1)
                    .file(DEVELOPER_C_FILES[2])?
                    .read_as::<ReviewFeedbackA>(&mut file_read_2)
                    .file(DEVELOPER_C_FILES[3])?
                    .read_as::<ReviewFeedbackB>(&mut file_read_3)
                    .file(DEVELOPER_C_FILES[4])?
                    .read_as::<ReviewFeedbackC>(&mut file_read_4)
                    .file(DEVELOPER_C_FILES[5])?
                    .read_as::<CiUpdateA>(&mut file_read_5)
                    .file(DEVELOPER_C_FILES[6])?
                    .read_as::<CiUpdateB>(&mut file_read_6)
                    .file(DEVELOPER_C_FILES[7])?
                    .read_as::<CiUpdateC>(&mut file_read_7)
                    .file(DEVELOPER_C_FILES[8])?
                    .read_as::<StageUpdateA>(&mut file_read_8)
                    .file(DEVELOPER_C_FILES[9])?
                    .read_as::<StageUpdateB>(&mut file_read_9)
                    .file(DEVELOPER_C_FILES[10])?
                    .read_as::<StageUpdateC>(&mut file_read_10)
                    .file(DEVELOPER_C_FILES[11])?
                    .read_as::<InterfaceAToC>(&mut file_read_11)
                    .file(DEVELOPER_C_FILES[12])?
                    .read_as::<InterfaceBToC>(&mut file_read_12);
                let files = ChoreoFsFiles::new(c_vm).file(file)?;
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
                    &mut c_io,
                    PrivateFs::new(c_terminal, bus, &DEVELOPER_C_FILES),
                )
                .await?;
            };
            async {
                let mut file_a_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_a_read_0 = vec![0; incoming_capacity];
                let mut file_a_read_3 = vec![0; incoming_capacity];
                let mut file_a_read_6 = vec![0; incoming_capacity];
                let mut file_a_read_9 = vec![0; incoming_capacity];
                let mut file_b_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_b_read_1 = vec![0; incoming_capacity];
                let mut file_b_read_4 = vec![0; incoming_capacity];
                let mut file_b_read_7 = vec![0; incoming_capacity];
                let mut file_b_read_10 = vec![0; incoming_capacity];
                let mut file_c_write_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_c_read_2 = vec![0; incoming_capacity];
                let mut file_c_read_5 = vec![0; incoming_capacity];
                let mut file_c_read_8 = vec![0; incoming_capacity];
                let mut file_c_read_11 = vec![0; incoming_capacity];
                let file_a = ChoreoFsFile::new(REVIEW_FILES[0], a_review_file)
                    .write_as::<ReviewRequestA>(&mut file_a_write_0)
                    .read_as::<FileReviewResultA>(&mut file_a_read_0)
                    .file(REVIEW_FILES[3])?
                    .read_as::<FileReviewCandidateA>(&mut file_a_read_3)
                    .file(REVIEW_FILES[6])?
                    .read_as::<CiUpdateA>(&mut file_a_read_6)
                    .file(REVIEW_FILES[9])?
                    .read_as::<StageUpdateA>(&mut file_a_read_9);
                let file_b = ChoreoFsFile::new(REVIEW_FILES[1], b_review_file)
                    .write_as::<ReviewRequestB>(&mut file_b_write_1)
                    .read_as::<FileReviewResultB>(&mut file_b_read_1)
                    .file(REVIEW_FILES[4])?
                    .read_as::<FileReviewCandidateB>(&mut file_b_read_4)
                    .file(REVIEW_FILES[7])?
                    .read_as::<CiUpdateB>(&mut file_b_read_7)
                    .file(REVIEW_FILES[10])?
                    .read_as::<StageUpdateB>(&mut file_b_read_10);
                let file_c = ChoreoFsFile::new(REVIEW_FILES[2], c_review_file)
                    .write_as::<ReviewRequestC>(&mut file_c_write_2)
                    .read_as::<FileReviewResultC>(&mut file_c_read_2)
                    .file(REVIEW_FILES[5])?
                    .read_as::<FileReviewCandidateC>(&mut file_c_read_5)
                    .file(REVIEW_FILES[8])?
                    .read_as::<CiUpdateC>(&mut file_c_read_8)
                    .file(REVIEW_FILES[11])?
                    .read_as::<StageUpdateC>(&mut file_c_read_11);
                let files = ChoreoFsFiles::new(review_vm)
                    .file(file_a)?
                    .file(file_b)?
                    .file(file_c)?;
                let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                let (exit, files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root, (file_c, (file_b, (file_a, ())))) = files.into_parts();
                file_a
                    .into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                file_b
                    .into_endpoint()
                    .send::<crate::shell::global::FileExit>(&status)
                    .await?;
                file_c
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
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_write_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_read_8 = vec![0; incoming_capacity];
                let mut file_read_9 = vec![0; incoming_capacity];
                let mut file_read_10 = vec![0; incoming_capacity];
                let mut file_read_11 = vec![0; incoming_capacity];
                let mut file_read_12 = vec![0; incoming_capacity];
                let mut file_read_13 = vec![0; incoming_capacity];
                let mut file_read_14 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(TEST_FILES[0], test_file)
                    .write_as::<TestRequestA>(&mut file_write_0)
                    .read_as::<FileTestedA>(&mut file_read_0)
                    .file(TEST_FILES[1])?
                    .write_as::<TestRequestB>(&mut file_write_1)
                    .read_as::<FileTestedB>(&mut file_read_1)
                    .file(TEST_FILES[2])?
                    .write_as::<TestRequestC>(&mut file_write_2)
                    .read_as::<FileTestedC>(&mut file_read_2)
                    .file(TEST_FILES[3])?
                    .read_as::<FileTestCandidateA>(&mut file_read_3)
                    .file(TEST_FILES[4])?
                    .read_as::<FileTestCandidateB>(&mut file_read_4)
                    .file(TEST_FILES[5])?
                    .read_as::<FileTestCandidateC>(&mut file_read_5)
                    .file(TEST_FILES[6])?
                    .read_as::<ReviewFeedbackA>(&mut file_read_6)
                    .file(TEST_FILES[7])?
                    .read_as::<ReviewFeedbackB>(&mut file_read_7)
                    .file(TEST_FILES[8])?
                    .read_as::<ReviewFeedbackC>(&mut file_read_8)
                    .file(TEST_FILES[9])?
                    .read_as::<CiUpdateA>(&mut file_read_9)
                    .file(TEST_FILES[10])?
                    .read_as::<CiUpdateB>(&mut file_read_10)
                    .file(TEST_FILES[11])?
                    .read_as::<CiUpdateC>(&mut file_read_11)
                    .file(TEST_FILES[12])?
                    .read_as::<StageUpdateA>(&mut file_read_12)
                    .file(TEST_FILES[13])?
                    .read_as::<StageUpdateB>(&mut file_read_13)
                    .file(TEST_FILES[14])?
                    .read_as::<StageUpdateC>(&mut file_read_14);
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
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_write_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_read_8 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(CI_FILES[0], ci_file)
                    .write_as::<CiRequestA>(&mut file_write_0)
                    .read_as::<FileCiResultA>(&mut file_read_0)
                    .file(CI_FILES[1])?
                    .write_as::<CiRequestB>(&mut file_write_1)
                    .read_as::<FileCiResultB>(&mut file_read_1)
                    .file(CI_FILES[2])?
                    .write_as::<CiRequestC>(&mut file_write_2)
                    .read_as::<FileCiResultC>(&mut file_read_2)
                    .file(CI_FILES[3])?
                    .read_as::<FilePublicationA>(&mut file_read_3)
                    .file(CI_FILES[4])?
                    .read_as::<FilePublicationB>(&mut file_read_4)
                    .file(CI_FILES[5])?
                    .read_as::<FilePublicationC>(&mut file_read_5)
                    .file(CI_FILES[6])?
                    .read_as::<StageUpdateA>(&mut file_read_6)
                    .file(CI_FILES[7])?
                    .read_as::<StageUpdateB>(&mut file_read_7)
                    .file(CI_FILES[8])?
                    .read_as::<StageUpdateC>(&mut file_read_8);
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
            async {
                let mut file_write_0 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_0 = vec![0; incoming_capacity];
                let mut file_write_1 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_1 = vec![0; incoming_capacity];
                let mut file_write_2 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_2 = vec![0; incoming_capacity];
                let mut file_write_3 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_3 = vec![0; incoming_capacity];
                let mut file_write_4 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_4 = vec![0; incoming_capacity];
                let mut file_write_5 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_5 = vec![0; incoming_capacity];
                let mut file_write_6 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_6 = vec![0; incoming_capacity];
                let mut file_write_7 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_7 = vec![0; incoming_capacity];
                let mut file_write_8 = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut file_read_8 = vec![0; incoming_capacity];
                let mut file_read_9 = vec![0; incoming_capacity];
                let mut file_read_10 = vec![0; incoming_capacity];
                let mut file_read_11 = vec![0; incoming_capacity];
                let file = ChoreoFsFile::new(RELEASE_FILES[0], release_file)
                    .write_as::<DeployRequestA>(&mut file_write_0)
                    .read_as::<DeployedA>(&mut file_read_0)
                    .file(RELEASE_FILES[1])?
                    .write_as::<DeployRequestB>(&mut file_write_1)
                    .read_as::<DeployedB>(&mut file_read_1)
                    .file(RELEASE_FILES[2])?
                    .write_as::<DeployRequestC>(&mut file_write_2)
                    .read_as::<DeployedC>(&mut file_read_2)
                    .file(RELEASE_FILES[3])?
                    .write_as::<ValidateRequestA>(&mut file_write_3)
                    .read_as::<FileStageResultA>(&mut file_read_3)
                    .file(RELEASE_FILES[4])?
                    .write_as::<ValidateRequestB>(&mut file_write_4)
                    .read_as::<FileStageResultB>(&mut file_read_4)
                    .file(RELEASE_FILES[5])?
                    .write_as::<ValidateRequestC>(&mut file_write_5)
                    .read_as::<FileStageResultC>(&mut file_read_5)
                    .file(RELEASE_FILES[6])?
                    .write_as::<RollbackRequestA>(&mut file_write_6)
                    .read_as::<RolledBackA>(&mut file_read_6)
                    .file(RELEASE_FILES[7])?
                    .write_as::<RollbackRequestB>(&mut file_write_7)
                    .read_as::<RolledBackB>(&mut file_read_7)
                    .file(RELEASE_FILES[8])?
                    .write_as::<RollbackRequestC>(&mut file_write_8)
                    .read_as::<RolledBackC>(&mut file_read_8)
                    .file(RELEASE_FILES[9])?
                    .read_as::<FileStagingReadyA>(&mut file_read_9)
                    .file(RELEASE_FILES[10])?
                    .read_as::<FileStagingReadyB>(&mut file_read_10)
                    .file(RELEASE_FILES[11])?
                    .read_as::<FileStagingReadyC>(&mut file_read_11);
                let files = ChoreoFsFiles::new(release_vm).file(file)?;
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
                    &mut release_io,
                    PrivateFs::new(release_terminal, bus, &RELEASE_FILES),
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
        Ok(()) => io.emit(bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> Value {
    global::programs().observation()
}
