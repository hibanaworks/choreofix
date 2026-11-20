//! Free shell calls use their Endpoint-bound ChoreoFS projections directly.
pub mod global;
mod imp;
pub(crate) mod localside;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::PrivateFs,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    (bus, simulator): (&Bus, &impl crate::shell::imp::service::Domain),
    terminals: [Terminal; 3],
) -> ShellResult<()> {
    use global::*;
    let p = programs();
    bus.projected(p.observation());
    let capture = |event| bus.capture(event);
    let mut slab = vec![0; 2 * 1024 * 1024];
    let incoming_capacity = slab.len();
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let result = async {
        let mut build_owner = rv.enter(session, &p.build_owner)?;
        let mut deploy_owner = rv.enter(session, &p.deploy_owner)?;
        let resource = rv.enter(session, &p.build_server)?;
        let environment = rv.enter(session, &p.deploy_env)?;
        let mut developer = rv.enter(session, &p.developer)?;
        let developer_close = rv.enter(session, &p.dev_close)?;
        let developer_file = rv.enter(session, &p.dev_file)?;
        let developer_vm = rv.enter(session, &p.dev_wasi)?;
        let mut developer_io = rv.enter(session, &p.dev_io)?;
        let mut tester = rv.enter(session, &p.tester)?;
        let tester_close = rv.enter(session, &p.test_close)?;
        let tester_file = rv.enter(session, &p.test_file)?;
        let tester_vm = rv.enter(session, &p.test_wasi)?;
        let mut tester_io = rv.enter(session, &p.test_io)?;
        let mut deployer = rv.enter(session, &p.deployer)?;
        let deployer_close = rv.enter(session, &p.deploy_close)?;
        let deployer_file = rv.enter(session, &p.deploy_file)?;
        let deployer_vm = rv.enter(session, &p.deploy_wasi)?;
        let mut deployer_io = rv.enter(session, &p.deploy_io)?;
        let [developer_terminal, tester_terminal, deployer_terminal] = terminals;
        async {
            localside::developer::run(&mut developer, developer_file).await?;
        };
        async {
            localside::tester::run(&mut tester, tester_file).await?;
        };
        async {
            localside::deployer::run(&mut deployer, deployer_file).await?;
        };
        async {
            localside::infrastructure::build_server(
                &mut build_owner,
                (resource, developer_close, tester_close),
                simulator,
            )
            .await?;
        };
        async {
            localside::infrastructure::deploy_env(
                &mut deploy_owner,
                (environment, deployer_close),
                simulator,
            )
            .await?;
        };
        async {
            let mut endpoint = developer_vm;
            let file = endpoint.recv::<Loan<DEV_FILE>>().await?;
            let mut incoming = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(COMPILE_OBJECT, file).read_as::<Notice>(&mut incoming))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<FileExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut developer_io,
                PrivateFs::new(developer_terminal, bus, &[COMPILE_OBJECT]),
            )
            .await?;
        };
        async {
            let mut endpoint = tester_vm;
            let file = endpoint.recv::<Loan<TEST_FILE>>().await?;
            let mut incoming = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(TEST_OBJECT, file).read_as::<Notice>(&mut incoming))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<FileExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut tester_io,
                PrivateFs::new(tester_terminal, bus, &[TEST_OBJECT]),
            )
            .await?;
        };
        async {
            let mut endpoint = deployer_vm;
            let file = endpoint.recv::<Loan<DEPLOY_FILE>>().await?;
            let mut incoming = vec![0; incoming_capacity];
            let files = ChoreoFsFiles::new(endpoint)
                .file(ChoreoFsFile::new(DEPLOY_OBJECT, file).read_as::<Notice>(&mut incoming))?;
            let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
            let mut storage = HibanaWasiGuestStorage::uninit();
            let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut endpoint, (file, ())) = files.into_parts();
            let mut file = file.into_endpoint();
            file.send::<FileExit>(&status).await?;
            endpoint.send::<crate::shell::global::Exit>(&status).await?;
        };
        async {
            private_io::run(
                &mut deployer_io,
                PrivateFs::new(deployer_terminal, bus, &[DEPLOY_OBJECT]),
            )
            .await?;
        };
        Ok(())
    }
    .await;
    bus.checkpoint(session.raw(), observations.checkpoint());
    result
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::*;
    let session = SessionId::new(SESSION);
    let (mut connection, mut reader, terminals) = Bus::open(
        ["DEVELOPER", "TESTER", "DEPLOYER"],
        [
            (session, BUILD_SERVER),
            (session, DEPLOY_ENV),
            (session, DEV_FILE),
            (session, DEV_CLOSE),
            (session, TEST_FILE),
            (session, TEST_CLOSE),
            (session, DEPLOY_FILE),
            (session, DEPLOY_CLOSE),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        run_guests(module, (bus, &simulator), terminals),
        reader.run(bus),
    )
    .await;
    match &result {
  Ok(())=>reader.emit(bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
  Err(error)=>reader.emit(bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
 }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
#[cfg(test)]
mod tests;
