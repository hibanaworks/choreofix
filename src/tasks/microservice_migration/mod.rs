mod guests;
mod preparation;
use crate::{
    imp::projection,
    shell::{
        ShellResult,
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    },
};
use guests::*;
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    program::{RoleProgram, project},
};
use preparation::*;
use serde_json::{Value, json};
const SESSION: u32 = 404;
macro_rules! roles {
    ($($field:ident:$role:ident),+ $(,)?) => {
        pub(super) struct Programs { $($field:RoleProgram<$role>,)+ }
        impl Programs {
            pub(super) fn new() -> Self {
                let source=global();
                Self {$($field:project::<$role>(&source),)+}
            }
            pub(super) fn observation(&self) -> Value {
                let roles:serde_json::Map<String,Value>=[$((stringify!($role).to_owned(),Value::from($role)),)+].into_iter().collect();
                let programs=[$(projection::rows(&self.$field),)+];
                json!({"type":"compiled_projection","format_version":1,"task":"4H","session":SESSION,
                       "roles":roles,"programs":programs.as_slice()})
            }
        }
    }
}
roles!(architect:ARCHITECT,
developer_a:DEVELOPER_A,
reviewer_a:REVIEWER_A,
service_a:SERVICE_A,
tester_a:TESTER_A,
developer_b:DEVELOPER_B,
reviewer_b:REVIEWER_B,
service_b:SERVICE_B,
tester_b:TESTER_B,
developer_c:DEVELOPER_C,
reviewer_c:REVIEWER_C,
service_c:SERVICE_C,
tester_c:TESTER_C,
env_owner:ENV_OWNER,
environment:TEST_ENV,
architect_file:ARCHITECT_FILE,
architect_data:ARCHITECT_DATA,
architect_vm:ARCHITECT_VM,
architect_io:ARCHITECT_IO,
a_file:DEVELOPER_A_FILE,
a_data:A_DATA,
a_vm:A_VM,
a_io:A_IO,
b_file:DEVELOPER_B_FILE,
b_data:B_DATA,
b_vm:B_VM,
b_io:B_IO,
c_file:DEVELOPER_C_FILE,
c_data:C_DATA,
c_vm:C_VM,
c_io:C_IO,
review_file:REVIEW_FILE,
review_data:REVIEW_DATA,
review_vm:REVIEW_VM,
review_io:REVIEW_IO,
test_file:TEST_FILE,
test_data:TEST_DATA,
test_vm:TEST_VM,
test_io:TEST_IO,
devops:DEVOPS,
staging:STAGING_ENV,
devops_file:DEVOPS_FILE,
devops_vm:DEVOPS_VM,
devops_io:DEVOPS_IO,
devops_data:DEVOPS_DATA);

#[hibana_runtime::scope]
pub(super) async fn run_stdio(wash: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            architect_terminal,
            a_terminal,
            b_terminal,
            c_terminal,
            review_terminal,
            test_terminal,
            devops_terminal,
        ],
    ) = Bus::open(
        [
            "ARCHITECT",
            "DEVELOPER_A",
            "DEVELOPER_B",
            "DEVELOPER_C",
            "REVIEWER",
            "TESTER",
            "DEVOPS",
        ],
        [
            SERVICE_A,
            SERVICE_B,
            SERVICE_C,
            TEST_ENV,
            STAGING_ENV,
            ARCHITECT_FILE,
            DEVELOPER_A_FILE,
            DEVELOPER_B_FILE,
            DEVELOPER_C_FILE,
            REVIEW_FILE,
            TEST_FILE,
            DEVOPS_FILE,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let trial = async {
        let programs = Programs::new();
        bus.projected(programs.observation());
        let capture = |event| bus.capture(event);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let tap = rv.tap();
        let result = async {
            let mut architect_side = rv.enter(session, &programs.architect)?;
            let staging = rv.enter(session, &programs.staging)?;
            let mut developer_a = rv.enter(session, &programs.developer_a)?;
            let mut reviewer_a = rv.enter(session, &programs.reviewer_a)?;
            let service_a = rv.enter(session, &programs.service_a)?;
            let mut tester_a = rv.enter(session, &programs.tester_a)?;
            let mut developer_b = rv.enter(session, &programs.developer_b)?;
            let mut reviewer_b = rv.enter(session, &programs.reviewer_b)?;
            let service_b = rv.enter(session, &programs.service_b)?;
            let mut tester_b = rv.enter(session, &programs.tester_b)?;
            let mut developer_c = rv.enter(session, &programs.developer_c)?;
            let mut reviewer_c = rv.enter(session, &programs.reviewer_c)?;
            let service_c = rv.enter(session, &programs.service_c)?;
            let mut tester_c = rv.enter(session, &programs.tester_c)?;
            let mut env_owner = rv.enter(session, &programs.env_owner)?;
            let environment = rv.enter(session, &programs.environment)?;
            let mut architect_data = rv.enter(session, &programs.architect_data)?;
            let architect_vm = rv.enter(session, &programs.architect_vm)?;
            let mut architect_io = rv.enter(session, &programs.architect_io)?;
            let mut a_data = rv.enter(session, &programs.a_data)?;
            let a_vm = rv.enter(session, &programs.a_vm)?;
            let mut a_io = rv.enter(session, &programs.a_io)?;
            let mut b_data = rv.enter(session, &programs.b_data)?;
            let b_vm = rv.enter(session, &programs.b_vm)?;
            let mut b_io = rv.enter(session, &programs.b_io)?;
            let mut c_data = rv.enter(session, &programs.c_data)?;
            let c_vm = rv.enter(session, &programs.c_vm)?;
            let mut c_io = rv.enter(session, &programs.c_io)?;
            let mut review_data = rv.enter(session, &programs.review_data)?;
            let review_vm = rv.enter(session, &programs.review_vm)?;
            let mut review_io = rv.enter(session, &programs.review_io)?;
            let mut test_data = rv.enter(session, &programs.test_data)?;
            let test_vm = rv.enter(session, &programs.test_vm)?;
            let mut test_io = rv.enter(session, &programs.test_io)?;
            // Each file's initial issuer is its metadata owner.
            let architect_file = rv.enter(session, &programs.architect_file)?;
            let a_file = rv.enter(session, &programs.a_file)?;
            let b_file = rv.enter(session, &programs.b_file)?;
            let c_file = rv.enter(session, &programs.c_file)?;
            let review_file = rv.enter(session, &programs.review_file)?;
            let test_file = rv.enter(session, &programs.test_file)?;
            let mut devops_side = rv.enter(session, &programs.devops)?;
            let mut devops_data = rv.enter(session, &programs.devops_data)?;
            let devops_file = rv.enter(session, &programs.devops_file)?;
            let devops_vm = rv.enter(session, &programs.devops_vm)?;
            let mut devops_io = rv.enter(session, &programs.devops_io)?;
            async { architect(&mut architect_side, staging, &simulator).await };
            async { devops(&mut devops_side, &simulator).await };
            async { file_metadata::run(&mut devops_data, devops_file, DEVOPS_FILES).await };
            async {
                private_io::run(
                    &mut devops_io,
                    PrivateFs::new(devops_terminal, bus, &DEVOPS_FILES),
                )
                .await
            };
            async { devops_guest(wash, devops_vm).await };
            async { developer(&mut developer_a, service_a, &simulator).await };
            async {
                reviewer::<REVIEWER_A, DEVELOPER_A, SERVICE_A>(&mut reviewer_a, &simulator).await
            };
            async {
                let _tested = tester::<TESTER_A, SERVICE_A>(&mut tester_a, &simulator).await?;
            };
            async { developer(&mut developer_b, service_b, &simulator).await };
            async {
                reviewer::<REVIEWER_B, DEVELOPER_B, SERVICE_B>(&mut reviewer_b, &simulator).await
            };
            async {
                let _tested = tester::<TESTER_B, SERVICE_B>(&mut tester_b, &simulator).await?;
            };
            async { developer(&mut developer_c, service_c, &simulator).await };
            async {
                reviewer::<REVIEWER_C, DEVELOPER_C, SERVICE_C>(&mut reviewer_c, &simulator).await
            };
            async {
                let _tested = tester::<TESTER_C, SERVICE_C>(&mut tester_c, &simulator).await?;
            };
            async {
                let _environment = test_environment(&mut env_owner, environment).await?;
            };
            async {
                file_metadata::run(&mut architect_data, architect_file, ARCHITECT_FILES).await
            };
            async {
                private_io::run(
                    &mut architect_io,
                    PrivateFs::new(architect_terminal, bus, &ARCHITECT_FILES),
                )
                .await
            };
            async { architect_guest(wash, architect_vm).await };
            async { file_metadata::run(&mut a_data, a_file, DEVELOPER_FILES).await };
            async {
                private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &DEVELOPER_FILES)).await
            };
            async { developer_guest::<A_VM, DEVELOPER_A_FILE, DEVELOPER_A>(wash, a_vm).await };
            async { file_metadata::run(&mut b_data, b_file, DEVELOPER_FILES).await };
            async {
                private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &DEVELOPER_FILES)).await
            };
            async { developer_guest::<B_VM, DEVELOPER_B_FILE, DEVELOPER_B>(wash, b_vm).await };
            async { file_metadata::run(&mut c_data, c_file, DEVELOPER_FILES).await };
            async {
                private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &DEVELOPER_FILES)).await
            };
            async { developer_guest::<C_VM, DEVELOPER_C_FILE, DEVELOPER_C>(wash, c_vm).await };
            async { file_metadata::run(&mut review_data, review_file, REVIEW_FILES).await };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(review_terminal, bus, &REVIEW_FILES),
                )
                .await
            };
            async { review_guest(wash, review_vm).await };
            async { file_metadata::run(&mut test_data, test_file, TEST_FILES).await };
            async {
                private_io::run(
                    &mut test_io,
                    PrivateFs::new(test_terminal, bus, &TEST_FILES),
                )
                .await
            };
            async { test_guest(wash, test_vm).await };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, tap.checkpoint());
        result
    };
    let result = hibana_runtime::race(trial, io.run(bus)).await;
    io.finish(bus, result, Some("preparation_tests_and_schema"))
        .await
}
