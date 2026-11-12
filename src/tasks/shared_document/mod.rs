//! 7E: one document authority, actual chapter material and fresh full reviews.
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
pub async fn run_stdio(module: &[u8], first: DecisionArm) -> ShellResult<()> {
    use crate::shell::{
        imp::{
            bus::Bus,
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};

    let (bus, mut io, [a_terminal, b_terminal, editor_terminal]) = Bus::open(
        ["WRITER_A", "WRITER_B", "EDITOR"],
        [(SessionId::new(SESSION), DOCUMENT)],
    );
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.observe(p.observation());
            let capture = |row| {
                bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(row)]}))
            };
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let session = SessionId::new(SESSION);
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut editor = rv.enter(session, &p.editor)?;
                let mut service = rv.enter(session, &p.service)?;
                let document = rv.enter(session, &p.document)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut editor_io = rv.enter(session, &p.editor_io)?;
                rv.set_resolver(
                    &p.service,
                    ResolverRef::<FIRST_WRITER>::decision_state(&first, |arm| Ok(*arm)),
                )
                .map_err(|_| {
                    ShellError::message("could not bind the initial document allocation")
                })?;
                let a_fs = PrivateFs::new(a_terminal, &bus);
                let b_fs = PrivateFs::new(b_terminal, &bus);
                let editor_fs = PrivateFs::new(editor_terminal, &bus);
                let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut editor_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                let mut a_store = HibanaWasiGuestStorage::uninit();
                let mut b_store = HibanaWasiGuestStorage::uninit();
                let mut editor_store = HibanaWasiGuestStorage::uninit();
                let a_guest = a_store.init(module, GuestMemory::new(&mut a_memory), bindings())?;
                let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
                let editor_guest =
                    editor_store.init(module, GuestMemory::new(&mut editor_memory), bindings())?;
                async {
                    let _document = localside::document::run(&mut service, (document, first), &bus)
                        .await
                        .map_err(|error| {
                            ShellError::message(format!("document service: {error}"))
                        })?;
                };
                async {
                    localside::writer::run(a_guest, &mut a)
                        .await
                        .map_err(|error| ShellError::message(format!("writer A: {error}")))?;
                };
                async {
                    localside::writer::run(b_guest, &mut b)
                        .await
                        .map_err(|error| ShellError::message(format!("writer B: {error}")))?;
                };
                async {
                    localside::editor::run(editor_guest, &mut editor)
                        .await
                        .map_err(|error| ShellError::message(format!("editor: {error}")))?;
                };
                async {
                    private_io::run(&mut a_io, a_fs).await?;
                };
                async {
                    private_io::run(&mut b_io, b_fs).await?;
                };
                async {
                    private_io::run(&mut editor_io, editor_fs).await?;
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
