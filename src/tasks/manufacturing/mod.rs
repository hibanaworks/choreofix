//! 11E development source; not yet an accepted Wasm/model-qualified task.
pub mod global;
pub(crate) mod imp;
pub mod localside;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{
            bus::Bus,
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    use std::cell::Cell;
    let session = SessionId::new(SESSION);
    let (bus, mut io, [a_terminal, b_terminal, supervisor_terminal]) = Bus::open(
        ["TECHNICIAN_A", "TECHNICIAN_B", "SUPERVISOR"],
        [
            (session, LATHE),
            (session, MILL),
            (session, LATHE_REQUEST_A),
            (session, LATHE_REQUEST_B),
            (session, MILL_REQUEST_A),
            (session, MILL_REQUEST_B),
        ],
    );
    let result = hibana_runtime::race(
        async {
            let p = global::programs();
            bus.observe(p.observation());
            let capture = |row| bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
                "records":[crate::imp::projection::record(row)]}));
            let first_lathe = Cell::new(serde_json::Value::Null);
            let first_mill = Cell::new(serde_json::Value::Null);
            let arrival_lathe = Cell::new(serde_json::Value::Null);
            let arrival_mill = Cell::new(serde_json::Value::Null);
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut supervisor = rv.enter(session, &p.supervisor)?;
                let mut lathe_service = rv.enter(session, &p.lathe_service)?;
                let mut mill_service = rv.enter(session, &p.mill_service)?;
                let mut report_service = rv.enter(session, &p.report_service)?;
                let lathe = rv.enter(session, &p.lathe)?;
                let mill = rv.enter(session, &p.mill)?;
                let lathe_request_a = rv.enter(session, &p.lathe_request_a)?;
                let lathe_request_b = rv.enter(session, &p.lathe_request_b)?;
                let mill_request_a = rv.enter(session, &p.mill_request_a)?;
                let mill_request_b = rv.enter(session, &p.mill_request_b)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut supervisor_io = rv.enter(session, &p.supervisor_io)?;
                rv.set_resolver(&p.lathe_service, ResolverRef::<FIRST_LATHE>::decision_state(&first_lathe, imp::first_request)).map_err(|e| ShellError::message(format!("{e:?}")))?;
                rv.set_resolver(&p.mill_service, ResolverRef::<FIRST_MILL>::decision_state(&first_mill, imp::first_request)).map_err(|e| ShellError::message(format!("{e:?}")))?;
                rv.set_resolver(&p.lathe_service, ResolverRef::<LATHE_ARRIVAL>::decision_state(&arrival_lathe, imp::arrival)).map_err(|e| ShellError::message(format!("{e:?}")))?;
                rv.set_resolver(&p.mill_service, ResolverRef::<MILL_ARRIVAL>::decision_state(&arrival_mill, imp::arrival)).map_err(|e| ShellError::message(format!("{e:?}")))?;
                let a_fs = PrivateFs::new(a_terminal, &bus);
                let b_fs = PrivateFs::new(b_terminal, &bus);
                let supervisor_fs = PrivateFs::new(supervisor_terminal, &bus);
                let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut supervisor_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut a_storage = HibanaWasiGuestStorage::uninit();
                let mut b_storage = HibanaWasiGuestStorage::uninit();
                let mut supervisor_storage = HibanaWasiGuestStorage::uninit();
                let a_guest = a_storage.init(module, GuestMemory::new(&mut a_memory), bindings())?;
                let b_guest = b_storage.init(module, GuestMemory::new(&mut b_memory), bindings())?;
                let supervisor_guest = supervisor_storage.init(module, GuestMemory::new(&mut supervisor_memory), bindings())?;
                async { localside::lathe::run(&mut lathe_service, (lathe, lathe_request_a, lathe_request_b), (&bus, &first_lathe, &arrival_lathe)).await?; };
                async { localside::mill::run(&mut mill_service, (mill, mill_request_a, mill_request_b), (&bus, &first_mill, &arrival_mill)).await?; };
                async { localside::reports::run(&mut report_service, &bus).await?; };
                async { localside::technician_a::run(a_guest, &mut a).await?; };
                async { localside::technician_b::run(b_guest, &mut b).await?; };
                async { localside::supervisor::run(supervisor_guest, &mut supervisor).await?; };
                async { private_io::run(&mut a_io, a_fs).await?; };
                async { private_io::run(&mut b_io, b_fs).await?; };
                async { private_io::run(&mut supervisor_io, supervisor_fs).await?; };
                Ok::<(), ShellError>(())
            }.await;
            bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
            result
        },
        io.run(&bus),
    ).await;
    match &result {
        Ok(()) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(&bus, serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
