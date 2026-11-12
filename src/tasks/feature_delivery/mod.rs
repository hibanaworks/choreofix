//! 6M: five wash guests, delegated feature authority and actual effect owners.
pub mod global;
pub(crate) mod imp;
pub mod localside;
use crate::shell::{ShellError, ShellResult};
use core::cell::Cell;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::Value;

fn actual_outcome(
    input: &Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = input.take();
    let result = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.set(value);
    result
}

#[hibana_runtime::scope]
async fn run_guests(
    module: &[u8],
    bus: &crate::shell::imp::bus::Bus,
    terminals: [crate::shell::imp::bus::Terminal; 5],
) -> ShellResult<()> {
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::private_io,
    };
    use global::{delivery::*, *};
    use hibana::runtime::resolver::ResolverRef;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let p = global::wasi::programs();
    bus.observe(p.observation());
    let capture = |row| {
        bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(row)]}))
    };
    let initial_a = Cell::new(Value::Null);
    let initial_b = Cell::new(Value::Null);
    let recovery_a = Cell::new(Value::Null);
    let recovery_b = Cell::new(Value::Null);
    let ci_a = Cell::new(Value::Null);
    let ci_b = Cell::new(Value::Null);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let session = SessionId::new(SESSION);
    let result = async {
        let mut a = rv.enter(session, &p.a)?;
        let mut b = rv.enter(session, &p.b)?;
        let mut writer = rv.enter(session, &p.writer)?;
        let mut reviewer = rv.enter(session, &p.reviewer)?;
        let mut runner = rv.enter(session, &p.runner)?;
        let mut repository = rv.enter(session, &p.repository)?;
        let mut review_a = rv.enter(session, &p.review_a)?;
        let mut review_b = rv.enter(session, &p.review_b)?;
        let mut data_a = rv.enter(session, &p.data_a)?;
        let mut data_b = rv.enter(session, &p.data_b)?;
        let mut ci = rv.enter(session, &p.ci)?;
        let repo = rv.enter(session, &p.repo)?;
        let environment = rv.enter(session, &p.environment)?;
        let build = rv.enter(session, &p.build)?;
        let feature_a = rv.enter(session, &p.feature_a)?;
        let feature_b = rv.enter(session, &p.feature_b)?;
        let mut a_io = rv.enter(session, &p.a_io)?;
        let mut b_io = rv.enter(session, &p.b_io)?;
        let mut writer_io = rv.enter(session, &p.writer_io)?;
        let mut reviewer_io = rv.enter(session, &p.reviewer_io)?;
        let mut runner_io = rv.enter(session, &p.runner_io)?;
        rv.set_resolver(
            &p.reviewer,
            ResolverRef::<REVIEW_DECISION_A>::decision_state(&initial_a, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        rv.set_resolver(
            &p.reviewer,
            ResolverRef::<REVIEW_DECISION_B>::decision_state(&initial_b, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        rv.set_resolver(
            &p.a,
            ResolverRef::<RECOVERY_REVIEW_A>::decision_state(&recovery_a, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        rv.set_resolver(
            &p.b,
            ResolverRef::<RECOVERY_REVIEW_B>::decision_state(&recovery_b, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        rv.set_resolver(
            &p.runner,
            ResolverRef::<CI_DECISION_A>::decision_state(&ci_a, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        rv.set_resolver(
            &p.runner,
            ResolverRef::<CI_DECISION_B>::decision_state(&ci_b, actual_outcome),
        )
        .map_err(|_| ShellError::message("could not bind the actual result resolver"))?;
        let [
            a_terminal,
            b_terminal,
            writer_terminal,
            reviewer_terminal,
            runner_terminal,
        ] = terminals;
        let a_fs = PrivateFs::new(a_terminal, bus);
        let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut a_store = HibanaWasiGuestStorage::uninit();
        let a_guest = a_store.init(module, GuestMemory::new(&mut a_memory), bindings())?;
        let b_fs = PrivateFs::new(b_terminal, bus);
        let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut b_store = HibanaWasiGuestStorage::uninit();
        let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
        let writer_fs = PrivateFs::new(writer_terminal, bus);
        let mut writer_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut writer_store = HibanaWasiGuestStorage::uninit();
        let writer_guest =
            writer_store.init(module, GuestMemory::new(&mut writer_memory), bindings())?;
        let reviewer_fs = PrivateFs::new(reviewer_terminal, bus);
        let mut reviewer_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut reviewer_store = HibanaWasiGuestStorage::uninit();
        let reviewer_guest =
            reviewer_store.init(module, GuestMemory::new(&mut reviewer_memory), bindings())?;
        let runner_fs = PrivateFs::new(runner_terminal, bus);
        let mut runner_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut runner_store = HibanaWasiGuestStorage::uninit();
        let runner_guest =
            runner_store.init(module, GuestMemory::new(&mut runner_memory), bindings())?;
        async {
            let _resources = localside::repository::run(
                &mut repository,
                (repo, environment, feature_a, feature_b),
                bus,
            )
            .await?;
        };
        async {
            localside::reviews::feature_a(&mut review_a, bus).await?;
        };
        async {
            localside::reviews::feature_b(&mut review_b, bus).await?;
        };
        async {
            localside::tests::feature_a(&mut data_a, bus).await?;
        };
        async {
            localside::tests::feature_b(&mut data_b, bus).await?;
        };
        async {
            localside::ci::run(&mut ci, build, bus).await?;
        };
        async {
            localside::developer_a_wasi::run(a_guest, &mut a, &recovery_a).await?;
        };
        async {
            localside::developer_b_wasi::run(b_guest, &mut b, &recovery_b).await?;
        };
        async {
            localside::test_writer_wasi::run(writer_guest, &mut writer).await?;
        };
        async {
            localside::reviewer_wasi::run(reviewer_guest, &mut reviewer, (&initial_a, &initial_b))
                .await?;
        };
        async {
            localside::ci_runner_wasi::run(runner_guest, &mut runner, (&ci_a, &ci_b)).await?;
        };
        async {
            private_io::run(&mut a_io, a_fs).await?;
        };
        async {
            private_io::run(&mut b_io, b_fs).await?;
        };
        async {
            private_io::run(&mut writer_io, writer_fs).await?;
        };
        async {
            private_io::run(&mut reviewer_io, reviewer_fs).await?;
        };
        async {
            private_io::run(&mut runner_io, runner_fs).await?;
        };
        Ok::<_, ShellError>(())
    }
    .await;
    bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use global::{delivery::*, *};
    let (bus, mut io, terminals) = crate::shell::imp::bus::Bus::open(
        [
            "DEVELOPER_A",
            "DEVELOPER_B",
            "TEST_WRITER",
            "REVIEWER",
            "CI_RUNNER",
        ],
        [
            (SessionId::new(SESSION), REPO),
            (SessionId::new(SESSION), TEST_ENV),
            (SessionId::new(SESSION), BUILD_SERVER),
            (SessionId::new(SESSION), AUTH_A),
            (SessionId::new(SESSION), AUTH_B),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), io.run(&bus)).await;
    match &result{
 Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
 Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
 }
    result
}
pub fn projection() -> Value {
    global::wasi::programs().observation()
}
