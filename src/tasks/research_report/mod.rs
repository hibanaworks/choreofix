//! 3E drives actual WASI guests with owned DOCUMENT and DATABASE Endpoints.
pub mod global;
mod imp;
pub(crate) mod localside;
#[cfg(test)]
mod tests;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult},
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
    use imp::{ReviewInput, decide};
    let p = programs();
    bus.observe(p.observation());
    let review_a_result = ReviewInput::default();
    let review_b_result = ReviewInput::default();
    let selected_review = imp::ReviewTarget::default();
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
    let session = SessionId::new(SESSION);
    let result = async {
        rv.set_resolver(
            &p.review_a,
            ResolverRef::<DECIDE_A>::decision_state(&review_a_result, decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_b,
            ResolverRef::<DECIDE_B>::decision_state(&review_b_result, decide),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &p.review_wasi,
            ResolverRef::<SELECT_REVIEW>::decision_state(
                &selected_review,
                imp::ReviewTarget::decide,
            ),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut document_service = rv.enter(session, &p.document_service)?;
        let document = rv.enter(session, &p.document)?;
        let mut database_service = rv.enter(session, &p.database_service)?;
        let database = rv.enter(session, &p.database)?;
        let mut research_a = rv.enter(session, &p.research_a)?;
        let mut research_b = rv.enter(session, &p.research_b)?;
        let mut a_wasi = rv.enter(session, &p.a_wasi)?;
        let mut b_wasi = rv.enter(session, &p.b_wasi)?;
        let mut review_wasi = rv.enter(session, &p.review_wasi)?;
        let mut review_a = rv.enter(session, &p.review_a)?;
        let mut review_b = rv.enter(session, &p.review_b)?;
        let mut a_io = rv.enter(session, &p.a_io)?;
        let mut b_io = rv.enter(session, &p.b_io)?;
        let mut review_io = rv.enter(session, &p.review_io)?;
        let [a_terminal, b_terminal, review_terminal] = terminals;
        let a_fs = PrivateFs::new(a_terminal, bus);
        let mut a_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut a_store = HibanaWasiGuestStorage::uninit();
        let a_guest = a_store.init(module, GuestMemory::new(&mut a_memory), bindings())?;
        let b_fs = PrivateFs::new(b_terminal, bus);
        let mut b_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut b_store = HibanaWasiGuestStorage::uninit();
        let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
        let review_fs = PrivateFs::new(review_terminal, bus);
        let mut review_memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
        let mut review_store = HibanaWasiGuestStorage::uninit();
        let review_guest =
            review_store.init(module, GuestMemory::new(&mut review_memory), bindings())?;
        async {
            localside::reviewer_a::run(&mut review_a, bus, &review_a_result).await?;
        };
        async {
            localside::reviewer_b::run(&mut review_b, bus, &review_b_result).await?;
        };
        async {
            localside::resources::document(&mut document_service, document, bus).await?;
        };
        async {
            localside::resources::database(&mut database_service, database, bus).await?;
        };
        async {
            localside::resources::research::<RESEARCH_A>(&mut research_a, bus).await?;
        };
        async {
            localside::resources::research::<RESEARCH_B>(&mut research_b, bus).await?;
        };
        async {
            localside::author::run::<A_WASI>(a_guest, &mut a_wasi).await?;
        };
        async {
            private_io::run(&mut a_io, a_fs).await?;
        };
        async {
            localside::author::run::<B_WASI>(b_guest, &mut b_wasi).await?;
        };
        async {
            private_io::run(&mut b_io, b_fs).await?;
        };
        async {
            localside::editor::run(review_guest, &mut review_wasi, &selected_review).await?;
        };
        async {
            private_io::run(&mut review_io, review_fs).await?;
        };
        Ok(())
    }
    .await;
    // Read the producer's actual head independently of the drained prefix.
    // A lost ring prefix/suffix or exhausted ordinal cannot pass EPF capture.
    bus.observe(
        serde_json::json!({"type":"protocol_checkpoint","producer":0,
        "session":session.raw(),"checkpoint":observations.checkpoint()}),
    );
    result
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open(
        ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
        [
            (SessionId::new(global::SESSION), global::DOCUMENT),
            (SessionId::new(global::SESSION), global::DATABASE),
        ],
    );
    let result = hibana_runtime::race(run_guests(module, &bus, terminals), reader.run(&bus)).await;
    match &result {
        Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}

/// Compiled communication metadata for the standalone audit artifact.
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
