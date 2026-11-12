//! Projected request/reply connects each WASI role to its service owner.
pub mod global;
mod imp;
pub(crate) mod localside;

use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult, imp::service::Validation},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    bus: &crate::shell::imp::bus::Bus,
    terminals: [crate::shell::imp::bus::Terminal; 3],
) -> ShellResult<()> {
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let programs = programs();
    bus.observe(programs.observation());
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let capture = |event| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(event)]}));
    };
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(16);
    let result = async {
        rv.set_resolver(
            &programs.developer,
            ResolverRef::<COMPILE_CLOSE>::decision_state(&compile, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.tester,
            ResolverRef::<TEST_CLOSE>::decision_state(&test, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &programs.deployer,
            ResolverRef::<DEPLOY_CLOSE>::decision_state(&deploy, Validation::decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut avm = rv.enter(session, &programs.dev_wasi)?;
        let mut bvm = rv.enter(session, &programs.test_wasi)?;
        let mut cvm = rv.enter(session, &programs.deploy_wasi)?;
        let mut developer = rv.enter(session, &programs.developer)?;
        let mut tester = rv.enter(session, &programs.tester)?;
        let mut deployer = rv.enter(session, &programs.deployer)?;
        let mut build_owner = rv.enter(session, &programs.build_owner)?;
        let mut deploy_owner = rv.enter(session, &programs.deploy_owner)?;
        let build = rv.enter(session, &programs.build_server)?;
        let env = rv.enter(session, &programs.deploy_env)?;
        let mut aio = rv.enter(session, &programs.dev_io)?;
        let mut bio = rv.enter(session, &programs.test_io)?;
        let mut cio = rv.enter(session, &programs.deploy_io)?;
        let [aterm, bterm, cterm] = terminals;
        let afs = PrivateFs::new(aterm, bus);
        let bfs = PrivateFs::new(bterm, bus);
        let cfs = PrivateFs::new(cterm, bus);
        let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut astore = HibanaWasiGuestStorage::uninit();
        let mut bstore = HibanaWasiGuestStorage::uninit();
        let mut cstore = HibanaWasiGuestStorage::uninit();
        let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
        let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
        let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
        async {
            localside::developer::run(&mut developer, bus, &compile).await?;
        };
        async {
            localside::tester::run(&mut tester, bus, &test).await?;
        };
        async {
            localside::deployer::run(&mut deployer, bus, &deploy).await?;
        };
        async {
            localside::infrastructure::build_server(&mut build_owner, build).await?;
        };
        async {
            localside::infrastructure::deploy_env(&mut deploy_owner, env).await?;
        };
        async {
            localside::wasi::developer(aguest, &mut avm).await?;
        };
        async {
            private_io::run(&mut aio, afs).await?;
        };
        async {
            localside::wasi::tester(bguest, &mut bvm).await?;
        };
        async {
            private_io::run(&mut bio, bfs).await?;
        };
        async {
            localside::wasi::deployer(cguest, &mut cvm).await?;
        };
        async {
            private_io::run(&mut cio, cfs).await?;
        };
        Ok(())
    }
    .await;
    bus.observe(
        serde_json::json!({"type":"protocol_checkpoint","producer":0,
        "session":session.raw(),"checkpoint":observations.checkpoint()}),
    );
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        ["DEVELOPER", "TESTER", "DEPLOYER"],
        [
            (SessionId::new(16), global::BUILD_SERVER),
            (SessionId::new(16), global::DEPLOY_ENV),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result {
        Ok(()) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"complete"})).await?,
        Err(error) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
    }
    result
}

/// Static observations of the programs that the CI/CD runtime attaches.
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}

#[cfg(test)]
mod tests;
