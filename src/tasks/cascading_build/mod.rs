//! 10M: overlapping affine build inputs, actual tests and data-bound rebuilding.
pub mod global;
pub(crate) mod imp;
pub mod localside;
#[cfg(test)]
mod tests;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    resolver::{DecisionArm, ResolverRef},
};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8], order: [u8; 3]) -> ShellResult<()> {
    use crate::shell::{
        imp::{
            bus::Bus,
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    let mut members = order;
    members.sort_unstable();
    if members != [A, B, C] {
        return Err(ShellError::message(
            "initial build order must contain A, B and C once",
        ));
    }
    let (
        bus,
        mut io,
        [
            a_terminal,
            b_terminal,
            c_terminal,
            integrator_terminal,
            tester_terminal,
        ],
    ) = Bus::open(
        [
            "BUILDER_A",
            "BUILDER_B",
            "BUILDER_C",
            "INTEGRATOR",
            "TEST_RUNNER",
        ],
        [
            (SessionId::new(SESSION), CORE),
            (SessionId::new(SESSION), CONFIG),
            (SessionId::new(SESSION), MODELS),
        ],
    );
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.observe(p.observation());
            let capture = |row| {
                bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
                "records":[crate::imp::projection::record(row)]}))
            };
            let input = core::cell::Cell::new(serde_json::Value::Null);
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let session = SessionId::new(SESSION);
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut c = rv.enter(session, &p.c)?;
                let mut integrator = rv.enter(session, &p.integrator)?;
                let mut tester = rv.enter(session, &p.tester)?;
                let mut compiler = rv.enter(session, &p.compiler)?;
                let mut testing = rv.enter(session, &p.testing)?;
                let core = rv.enter(session, &p.core)?;
                let config = rv.enter(session, &p.config)?;
                let models = rv.enter(session, &p.models)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut c_io = rv.enter(session, &p.c_io)?;
                let mut integrator_io = rv.enter(session, &p.integrator_io)?;
                let mut tester_io = rv.enter(session, &p.tester_io)?;
                rv.set_resolver(
                    &p.compiler,
                    ResolverRef::<FIRST_A>::decision_state(&order, |order| {
                        Ok(if order[0] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .map_err(|_| ShellError::message("could not bind first A allocation"))?;
                rv.set_resolver(
                    &p.compiler,
                    ResolverRef::<FIRST_B>::decision_state(&order, |order| {
                        Ok(if order[0] == B {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .map_err(|_| ShellError::message("could not bind first B allocation"))?;
                rv.set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_A>::decision_state(&order, |order| {
                        Ok(if order[1] == B {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .map_err(|_| ShellError::message("could not bind A's following allocation"))?;
                rv.set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_B>::decision_state(&order, |order| {
                        Ok(if order[1] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .map_err(|_| ShellError::message("could not bind B's following allocation"))?;
                rv.set_resolver(
                    &p.compiler,
                    ResolverRef::<SECOND_C>::decision_state(&order, |order| {
                        Ok(if order[1] == A {
                            DecisionArm::Left
                        } else {
                            DecisionArm::Right
                        })
                    }),
                )
                .map_err(|_| ShellError::message("could not bind C's following allocation"))?;
                rv.set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_A>::decision_state(&input, imp::selected::<0>),
                )
                .map_err(|_| ShellError::message("could not bind actual A candidate input"))?;
                rv.set_resolver(
                    &p.integrator,
                    ResolverRef::<REBUILD_B>::decision_state(&input, imp::selected::<1>),
                )
                .map_err(|_| ShellError::message("could not bind actual B candidate input"))?;
                let a_fs = PrivateFs::new(a_terminal, &bus);
                let b_fs = PrivateFs::new(b_terminal, &bus);
                let c_fs = PrivateFs::new(c_terminal, &bus);
                let integrator_fs = PrivateFs::new(integrator_terminal, &bus);
                let tester_fs = PrivateFs::new(tester_terminal, &bus);
                let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut c_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut integrator_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut tester_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut a_storage = HibanaWasiGuestStorage::uninit();
                let mut b_storage = HibanaWasiGuestStorage::uninit();
                let mut c_storage = HibanaWasiGuestStorage::uninit();
                let mut integrator_storage = HibanaWasiGuestStorage::uninit();
                let mut tester_storage = HibanaWasiGuestStorage::uninit();
                let a_guest =
                    a_storage.init(module, GuestMemory::new(&mut a_memory), bindings())?;
                let b_guest =
                    b_storage.init(module, GuestMemory::new(&mut b_memory), bindings())?;
                let c_guest =
                    c_storage.init(module, GuestMemory::new(&mut c_memory), bindings())?;
                let integrator_guest = integrator_storage.init(
                    module,
                    GuestMemory::new(&mut integrator_memory),
                    bindings(),
                )?;
                let tester_guest = tester_storage.init(
                    module,
                    GuestMemory::new(&mut tester_memory),
                    bindings(),
                )?;
                async {
                    localside::compiler::run(&mut compiler, (core, config, models), &bus).await?;
                };
                async {
                    localside::testing::run(&mut testing, &bus).await?;
                };
                async {
                    localside::builder::run(a_guest, &mut a).await?;
                };
                async {
                    localside::builder::run(b_guest, &mut b).await?;
                };
                async {
                    localside::builder::run(c_guest, &mut c).await?;
                };
                async {
                    localside::integrator::run(integrator_guest, &mut integrator, &input).await?;
                };
                async {
                    localside::tester::run(tester_guest, &mut tester).await?;
                };
                async {
                    private_io::run(&mut a_io, a_fs).await?;
                };
                async {
                    private_io::run(&mut b_io, b_fs).await?;
                };
                async {
                    private_io::run(&mut c_io, c_fs).await?;
                };
                async {
                    private_io::run(&mut integrator_io, integrator_fs).await?;
                };
                async {
                    private_io::run(&mut tester_io, tester_fs).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await;
            bus.observe(
                serde_json::json!({"type":"protocol_checkpoint","producer":0,
                "session":SESSION,"checkpoint":observations.checkpoint()}),
            );
            result
        },
        io.run(&bus),
    )
    .await;
    match &result {
        Ok(()) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
