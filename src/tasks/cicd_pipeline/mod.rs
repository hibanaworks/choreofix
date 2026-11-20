mod arguments;
mod files;
mod global;
mod localside;

#[cfg(test)]
mod tests;

use self::{
    arguments::{BACKEND_COMPONENT, FRONTEND_COMPONENT},
    global::*,
};
use crate::{
    imp::projection,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    },
};
use hibana::{
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::json;

pub fn projection() -> serde_json::Value {
    let source = with_shells();
    projection_value(vec![
        projection::rows(&project::<REGISTRY_OWNER>(&source)),
        projection::rows(&project::<BUILD_MASTER>(&source)),
        projection::rows(&project::<BUILD_SERVER>(&source)),
        projection::rows(&project::<ARTIFACT_STORE>(&source)),
        projection::rows(&project::<FRONTEND_DEV>(&source)),
        projection::rows(&project::<BACKEND_DEV>(&source)),
        projection::rows(&project::<QA>(&source)),
        projection::rows(&project::<RELEASE>(&source)),
        projection::rows(&project::<PROD_OWNER>(&source)),
        projection::rows(&project::<FRONTEND_DATA>(&source)),
        projection::rows(&project::<BACKEND_DATA>(&source)),
        projection::rows(&project::<QA_DATA>(&source)),
        projection::rows(&project::<RELEASE_DATA>(&source)),
        projection::rows(&project::<MASTER_DATA>(&source)),
        projection::rows(&project::<TEST_ENV>(&source)),
        projection::rows(&project::<PROD_ENV>(&source)),
        projection::rows(&project::<FRONTEND_FILE>(&source)),
        projection::rows(&project::<BACKEND_FILE>(&source)),
        projection::rows(&project::<QA_FILE>(&source)),
        projection::rows(&project::<RELEASE_FILE>(&source)),
        projection::rows(&project::<MASTER_FILE>(&source)),
        projection::rows(&project::<FRONTEND_VM>(&source)),
        projection::rows(&project::<BACKEND_VM>(&source)),
        projection::rows(&project::<QA_VM>(&source)),
        projection::rows(&project::<RELEASE_VM>(&source)),
        projection::rows(&project::<MASTER_VM>(&source)),
        projection::rows(&project::<FRONTEND_IO>(&source)),
        projection::rows(&project::<BACKEND_IO>(&source)),
        projection::rows(&project::<QA_IO>(&source)),
        projection::rows(&project::<RELEASE_IO>(&source)),
        projection::rows(&project::<MASTER_IO>(&source)),
    ])
}

fn projection_value(programs: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "type":"compiled_projection", "format_version":1, "task":"16M", "session":SESSION,
        "roles": {
            "REGISTRY_OWNER": REGISTRY_OWNER,
            "BUILD_SERVER": BUILD_SERVER,
            "ARTIFACT_STORE": ARTIFACT_STORE,
            "FRONTEND_DEV": FRONTEND_DEV,
            "BACKEND_DEV": BACKEND_DEV,
            "BUILD_MASTER": BUILD_MASTER,
            "QA": QA,
            "RELEASE": RELEASE,
            "PROD_OWNER": PROD_OWNER,
            "FRONTEND_DATA": FRONTEND_DATA,
            "BACKEND_DATA": BACKEND_DATA,
            "QA_DATA": QA_DATA,
            "RELEASE_DATA": RELEASE_DATA,
            "MASTER_DATA": MASTER_DATA,
            "TEST_ENV": TEST_ENV,
            "PROD_ENV": PROD_ENV,
            "FRONTEND_FILE": FRONTEND_FILE,
            "BACKEND_FILE": BACKEND_FILE,
            "QA_FILE": QA_FILE,
            "RELEASE_FILE": RELEASE_FILE,
            "MASTER_FILE": MASTER_FILE,
            "FRONTEND_VM": FRONTEND_VM,
            "BACKEND_VM": BACKEND_VM,
            "QA_VM": QA_VM,
            "RELEASE_VM": RELEASE_VM,
            "MASTER_VM": MASTER_VM,
            "FRONTEND_IO": FRONTEND_IO,
            "BACKEND_IO": BACKEND_IO,
            "QA_IO": QA_IO,
            "RELEASE_IO": RELEASE_IO,
            "MASTER_IO": MASTER_IO,
        },
        "labels": {
            "Compile": Compile::<FRONTEND_COMPONENT>::LOGICAL_LABEL,
            "UnitTest": UnitTest::<FRONTEND_COMPONENT>::LOGICAL_LABEL,
            "Publish": Publish::<FRONTEND_DEV>::LOGICAL_LABEL,
            "ReadBuild": ReadBuild::<FRONTEND_DEV>::LOGICAL_LABEL,
            "RegisterArtifact": RegisterArtifact::<FRONTEND_DEV>::LOGICAL_LABEL,
            "Compiled": Compiled::LOGICAL_LABEL,
            "UnitTested": UnitTested::LOGICAL_LABEL,
            "Published": Published::LOGICAL_LABEL,
            "PullForQa": PullForQa::LOGICAL_LABEL,
            "PullForRelease": PullForRelease::LOGICAL_LABEL,
            "QaPulled": QaPulled::LOGICAL_LABEL,
            "ReleasePulled": ReleasePulled::LOGICAL_LABEL,
            "Integration": Integration::LOGICAL_LABEL,
            "E2e": E2e::LOGICAL_LABEL,
            "Integrated": Integrated::LOGICAL_LABEL,
            "E2eTested": E2eTested::LOGICAL_LABEL,
            "Stage": Stage::LOGICAL_LABEL,
            "Staged": Staged::LOGICAL_LABEL,
            "SmokeStaging": SmokeStaging::LOGICAL_LABEL,
            "StagingResult": StagingResult::LOGICAL_LABEL,
            "Deploy": Deploy::LOGICAL_LABEL,
            "Deployed": Deployed::LOGICAL_LABEL,
            "SmokeProduction": SmokeProduction::LOGICAL_LABEL,
            "ProductionSmoke": ProductionSmoke::LOGICAL_LABEL,
        },
        "programs":programs
    })
}

#[hibana_runtime::scope]
pub async fn run_stdio(wash: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        bus,
        mut io,
        [
            front_terminal,
            back_terminal,
            qa_terminal,
            release_terminal,
            master_terminal,
        ],
    ) = Bus::open(
        [
            "FRONTEND_DEV",
            "BACKEND_DEV",
            "QA_ENGINEER",
            "RELEASE_ENGINEER",
            "BUILD_MASTER",
        ],
        [
            BUILD_SERVER,
            ARTIFACT_STORE,
            TEST_ENV,
            PROD_ENV,
            FRONTEND_FILE,
            BACKEND_FILE,
            QA_FILE,
            RELEASE_FILE,
            MASTER_FILE,
        ]
        .map(|role| (session, role)),
    );
    let result = hibana_runtime::race(
        async {
            let source = with_shells();
            let capture = |event| {
                bus.observe(json!({
                    "type":"protocol_capture", "producer":0, "records":[projection::record(event)]
                }))
            };
            let slab_bytes = 1024 * 1024;
            let mut slab = vec![0; slab_bytes];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observation = rv.tap();
            let result = async {
                let mut programs = Vec::new();
                macro_rules! attach {
                    ($role:ident) => {{
                        let program = project::<$role>(&source);
                        programs.push(projection::rows(&program));
                        rv.enter(session, &program)?
                    }};
                }
                let mut registry_owner = attach!(REGISTRY_OWNER);
                let mut master = attach!(BUILD_MASTER);
                let workspace = attach!(BUILD_SERVER);
                let store = attach!(ARTIFACT_STORE);
                let mut frontend = attach!(FRONTEND_DEV);
                let mut backend = attach!(BACKEND_DEV);
                let mut qa = attach!(QA);
                let mut release = attach!(RELEASE);
                let mut prod_owner = attach!(PROD_OWNER);
                let mut front_data = attach!(FRONTEND_DATA);
                let mut back_data = attach!(BACKEND_DATA);
                let mut qa_data = attach!(QA_DATA);
                let mut release_data = attach!(RELEASE_DATA);
                let mut master_data = attach!(MASTER_DATA);
                let test_env = attach!(TEST_ENV);
                let prod_env = attach!(PROD_ENV);
                let front_file = attach!(FRONTEND_FILE);
                let back_file = attach!(BACKEND_FILE);
                let qa_file = attach!(QA_FILE);
                let release_file = attach!(RELEASE_FILE);
                let master_file = attach!(MASTER_FILE);
                let mut front_vm = attach!(FRONTEND_VM);
                let mut back_vm = attach!(BACKEND_VM);
                let mut qa_vm = attach!(QA_VM);
                let mut release_vm = attach!(RELEASE_VM);
                let mut master_vm = attach!(MASTER_VM);
                let mut front_io = attach!(FRONTEND_IO);
                let mut back_io = attach!(BACKEND_IO);
                let mut qa_io = attach!(QA_IO);
                let mut release_io = attach!(RELEASE_IO);
                let mut master_io = attach!(MASTER_IO);
                bus.observe(projection_value(programs));
                async { localside::registry_owner(&mut registry_owner, store, &bus).await };
                async { localside::master(&mut master, workspace).await };
                async {
                    localside::developer::<FRONTEND_DEV, FRONTEND_COMPONENT>(&mut frontend, &bus)
                        .await
                };
                async {
                    localside::developer::<BACKEND_DEV, BACKEND_COMPONENT>(&mut backend, &bus).await
                };
                async { localside::qa(&mut qa, test_env, &bus).await };
                async { localside::release(&mut release, &bus).await };
                async {
                    let branch = prod_owner.offer().await?;
                    match branch.label() {
                        Promote::LOGICAL_LABEL => {
                            branch.recv::<Promote>().await?;
                            prod_owner.send::<Loan<PROD_ENV>>(prod_env).await?;
                            drop(prod_owner.recv::<Loan<PROD_ENV>>().await?);
                        }
                        StagingAborted::LOGICAL_LABEL => {
                            branch.recv::<StagingAborted>().await?;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected production decision {label}"
                            )));
                        }
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    file_metadata::run(&mut front_data, front_file, files::DEVELOPER).await?;
                };
                async {
                    private_io::run(
                        &mut front_io,
                        PrivateFs::new(front_terminal, &bus, &files::DEVELOPER),
                    )
                    .await?;
                };
                async {
                    file_metadata::run(&mut back_data, back_file, files::DEVELOPER).await?;
                };
                async {
                    private_io::run(
                        &mut back_io,
                        PrivateFs::new(back_terminal, &bus, &files::DEVELOPER),
                    )
                    .await?;
                };
                async {
                    file_metadata::run(&mut qa_data, qa_file, files::QA).await?;
                };
                async {
                    private_io::run(&mut qa_io, PrivateFs::new(qa_terminal, &bus, &files::QA))
                        .await?;
                };
                async {
                    file_metadata::run(&mut release_data, release_file, files::RELEASE).await?;
                };
                async {
                    private_io::run(
                        &mut release_io,
                        PrivateFs::new(release_terminal, &bus, &files::RELEASE),
                    )
                    .await?;
                };
                async {
                    file_metadata::run(&mut master_data, master_file, files::MASTER).await?;
                };
                async {
                    private_io::run(
                        &mut master_io,
                        PrivateFs::new(master_terminal, &bus, &files::MASTER),
                    )
                    .await?;
                };
                async {
                    let file = front_vm.recv::<Loan<FRONTEND_FILE>>().await?;
                    let mut result = vec![0; slab_bytes];
                    let mut revision = vec![0; slab_bytes];
                    let mut workspace = vec![0; slab_bytes];
                    let mut verified = vec![0; slab_bytes];
                    let mut qa_done = vec![0; slab_bytes];
                    let mut qa_revise = vec![0; slab_bytes];
                    let mut result_0 = vec![0; slab_bytes];
                    let mut arguments_1 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; slab_bytes];
                    let mut arguments_2 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; slab_bytes];
                    let mut arguments_3 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_3 = vec![0; slab_bytes];
                    let file = ChoreoFsFile::new(files::DEVELOPER[0], file)
                        .read_as::<BeginFrontend>(&mut result_0)
                        .file(files::DEVELOPER[1])?
                        .write_as::<Compile<FRONTEND_COMPONENT>>(&mut arguments_1)
                        .read_as::<Compiled>(&mut result_1)
                        .file(files::DEVELOPER[2])?
                        .write_as::<UnitTest<FRONTEND_COMPONENT>>(&mut arguments_2)
                        .read_as::<UnitTested>(&mut result_2)
                        .file(files::DEVELOPER[3])?
                        .write_as::<Publish<FRONTEND_DEV>>(&mut arguments_3)
                        .read_as::<Published>(&mut result_3)
                        .file(files::DEVELOPER[4])?
                        .read_as::<FrontendFinished>(&mut result)
                        .file(files::DEVELOPER[5])?
                        .read_as::<Revise<FRONTEND_DEV>>(&mut revision)
                        .file(files::DEVELOPER[6])?
                        .read_as::<WorkspaceAvailable>(&mut workspace)
                        .file(files::DEVELOPER[7])?
                        .read_as::<BuildVerified>(&mut verified)
                        .file(files::DEVELOPER[8])?
                        .read_as::<BuildEnded<FRONTEND_DEV>>(&mut qa_done)
                        .file(files::DEVELOPER[9])?
                        .read_as::<QaReviseFrontend>(&mut qa_revise);
                    let attached = ChoreoFsFiles::new(front_vm).file(file)?;
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(wash, GuestMemory::new(&mut memory), attached)?;
                    let (exit, attached) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut vm, (file, ())) = attached.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    vm.send::<Exit>(&status).await?;
                };
                async {
                    let file = back_vm.recv::<Loan<BACKEND_FILE>>().await?;
                    let mut result = vec![0; slab_bytes];
                    let mut revision = vec![0; slab_bytes];
                    let mut workspace = vec![0; slab_bytes];
                    let mut verified = vec![0; slab_bytes];
                    let mut qa_done = vec![0; slab_bytes];
                    let mut qa_revise = vec![0; slab_bytes];
                    let mut result_0 = vec![0; slab_bytes];
                    let mut arguments_1 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; slab_bytes];
                    let mut arguments_2 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; slab_bytes];
                    let mut arguments_3 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_3 = vec![0; slab_bytes];
                    let file = ChoreoFsFile::new(files::DEVELOPER[0], file)
                        .read_as::<BeginBackend>(&mut result_0)
                        .file(files::DEVELOPER[1])?
                        .write_as::<Compile<BACKEND_COMPONENT>>(&mut arguments_1)
                        .read_as::<Compiled>(&mut result_1)
                        .file(files::DEVELOPER[2])?
                        .write_as::<UnitTest<BACKEND_COMPONENT>>(&mut arguments_2)
                        .read_as::<UnitTested>(&mut result_2)
                        .file(files::DEVELOPER[3])?
                        .write_as::<Publish<BACKEND_DEV>>(&mut arguments_3)
                        .read_as::<Published>(&mut result_3)
                        .file(files::DEVELOPER[4])?
                        .read_as::<BackendFinished>(&mut result)
                        .file(files::DEVELOPER[5])?
                        .read_as::<Revise<BACKEND_DEV>>(&mut revision)
                        .file(files::DEVELOPER[6])?
                        .read_as::<WorkspaceAvailable>(&mut workspace)
                        .file(files::DEVELOPER[7])?
                        .read_as::<BuildVerified>(&mut verified)
                        .file(files::DEVELOPER[8])?
                        .read_as::<BuildEnded<BACKEND_DEV>>(&mut qa_done)
                        .file(files::DEVELOPER[9])?
                        .read_as::<QaReviseBackend>(&mut qa_revise);
                    let attached = ChoreoFsFiles::new(back_vm).file(file)?;
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(wash, GuestMemory::new(&mut memory), attached)?;
                    let (exit, attached) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut vm, (file, ())) = attached.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    vm.send::<Exit>(&status).await?;
                };
                async {
                    let file = qa_vm.recv::<Loan<QA_FILE>>().await?;
                    let mut result = vec![0; slab_bytes];
                    let mut result_0 = vec![0; slab_bytes];
                    let mut arguments_1 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; slab_bytes];
                    let mut arguments_2 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; slab_bytes];
                    let mut arguments_3 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_3 = vec![0; slab_bytes];
                    let file = ChoreoFsFile::new(files::QA[0], file)
                        .read_as::<ReadyForQa>(&mut result_0)
                        .file(files::QA[1])?
                        .write_as::<PullForQa>(&mut arguments_1)
                        .read_as::<QaPulled>(&mut result_1)
                        .file(files::QA[2])?
                        .write_as::<Integration>(&mut arguments_2)
                        .read_as::<Integrated>(&mut result_2)
                        .file(files::QA[3])?
                        .write_as::<E2e>(&mut arguments_3)
                        .read_as::<E2eTested>(&mut result_3)
                        .file(files::QA[4])?
                        .read_as::<QaFinished>(&mut result);
                    let attached = ChoreoFsFiles::new(qa_vm).file(file)?;
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(wash, GuestMemory::new(&mut memory), attached)?;
                    let (exit, attached) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut vm, (file, ())) = attached.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    vm.send::<Exit>(&status).await?;
                };
                async {
                    let file = release_vm.recv::<Loan<RELEASE_FILE>>().await?;
                    let mut result = vec![0; slab_bytes];
                    let mut result_0 = vec![0; slab_bytes];
                    let mut arguments_1 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_1 = vec![0; slab_bytes];
                    let mut arguments_2 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; slab_bytes];
                    let mut arguments_3 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_3 = vec![0; slab_bytes];
                    let mut result_4 = vec![0; slab_bytes];
                    let mut result_5 = vec![0; slab_bytes];
                    let mut arguments_6 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_6 = vec![0; slab_bytes];
                    let mut arguments_7 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_7 = vec![0; slab_bytes];
                    let mut staging_ready = vec![0; slab_bytes];
                    let file = ChoreoFsFile::new(files::RELEASE[0], file)
                        .read_as::<ReadyForRelease>(&mut result_0)
                        .file(files::RELEASE[1])?
                        .write_as::<PullForRelease>(&mut arguments_1)
                        .read_as::<ReleasePulled>(&mut result_1)
                        .file(files::RELEASE[2])?
                        .write_as::<Stage>(&mut arguments_2)
                        .read_as::<Staged>(&mut result_2)
                        .file(files::RELEASE[3])?
                        .write_as::<SmokeStaging>(&mut arguments_3)
                        .read_as::<StagingResult>(&mut result_3)
                        .file(files::RELEASE[4])?
                        .read_as::<Promote>(&mut result_4)
                        .file(files::RELEASE[5])?
                        .read_as::<StagingAborted>(&mut result_5)
                        .file(files::RELEASE[6])?
                        .write_as::<Deploy>(&mut arguments_6)
                        .read_as::<Deployed>(&mut result_6)
                        .file(files::RELEASE[7])?
                        .write_as::<SmokeProduction>(&mut arguments_7)
                        .read_as::<ProductionSmoke>(&mut result_7)
                        .file(files::RELEASE[8])?
                        .read_as::<StagingReady>(&mut staging_ready)
                        .file(files::RELEASE[9])?
                        .read_as::<ReleaseFinished>(&mut result);
                    let attached = ChoreoFsFiles::new(release_vm).file(file)?;
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(wash, GuestMemory::new(&mut memory), attached)?;
                    let (exit, attached) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut vm, (file, ())) = attached.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    vm.send::<Exit>(&status).await?;
                };
                async {
                    let file = master_vm.recv::<Loan<MASTER_FILE>>().await?;
                    let mut arguments_0 = vec![0; GUEST_MEMORY_BYTES];
                    let mut arguments_1 = vec![0; GUEST_MEMORY_BYTES];
                    let mut result_2 = vec![0; slab_bytes];
                    let mut report = vec![0; slab_bytes];
                    let mut repair = vec![0; GUEST_MEMORY_BYTES];
                    let file = ChoreoFsFile::new(files::MASTER[0], file)
                        .write_as::<ChooseFrontend>(&mut arguments_0)
                        .file(files::MASTER[1])?
                        .write_as::<ChooseBackend>(&mut arguments_1)
                        .file(files::MASTER[2])?
                        .read_as::<FinalResult>(&mut result_2)
                        .file(files::MASTER[3])?
                        .read_as::<QaReport>(&mut report)
                        .file(files::MASTER[4])?
                        .write_as::<RepairDecision>(&mut repair);
                    let attached = ChoreoFsFiles::new(master_vm).file(file)?;
                    let mut memory = vec![0; GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(wash, GuestMemory::new(&mut memory), attached)?;
                    let (exit, attached) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut vm, (file, ())) = attached.into_parts();
                    file.into_endpoint().send::<FileExit>(&status).await?;
                    vm.send::<Exit>(&status).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await;
            bus.observe(json!({
                "type":"protocol_checkpoint", "producer":0, "session":session.raw(),
                "checkpoint":observation.checkpoint()
            }));
            result
        },
        io.run(&bus),
    )
    .await;
    let verdict = match &result {
        Ok(()) => json!({"type":"run_result","status":"complete"}),
        Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
    };
    io.emit(&bus, verdict).await?;
    result
}
