use super::{global::*, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::cell::Cell;

// Append-only observation count; it never grants a protocol operation.
struct Effects(Cell<usize>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        self.0.set(self.0.get() + 1);
        Ok(json!({"tool_name":tool,"success":true,"data":{"agent":agent,"arguments":arguments}}))
    }
}
#[hibana_runtime::main]
#[test]
async fn hplc_rejects_an_unassigned_sample_before_a_domain_call() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION + 1);
    let (bus, _reader, _terminals) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "TOXICOLOGIST", "LEAD_SCIENTIST"],
        [(session, HPLC), (session, MASS_SPEC), (session, CELL_LAB)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut scientist = rv.enter(session, &p.chemist)?;
    let [mut task, _private] = scientist.par()?;
    let mut owner = rv.enter(session, &p.hplc_service)?;
    let instrument = rv.enter(session, &p.hplc)?;
    let effects = Effects(Cell::new(0));
    let result: ShellResult<()> = hibana_runtime::race(
        localside::instruments::hplc(&mut owner, instrument, &effects),
        async {
            let admitted = serde_json::to_vec(&json!({"sample":"purity_check"}))
                .map_err(|e| ShellError::message(e.to_string()))?;
            task.send::<HplcIntent>(&admitted.as_slice()).await?;
            let mut instrument = task.recv::<HplcLoan>().await?;
            let substituted = serde_json::to_vec(&json!({"sample":"documentation"}))
                .map_err(|e| ShellError::message(e.to_string()))?;
            instrument
                .send::<HplcRequest>(&substituted.as_slice())
                .await?;
            instrument.recv::<HplcResult>().await?;
            Ok(())
        },
    )
    .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("assigned sample")),
        "{result:?}"
    );
    assert_eq!(effects.0.get(), 0);
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn hplc_rejects_substituted_resource_arguments_before_a_domain_call() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION + 2);
    let (bus, _reader, _terminals) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "TOXICOLOGIST", "LEAD_SCIENTIST"],
        [(session, HPLC), (session, MASS_SPEC), (session, CELL_LAB)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut scientist = rv.enter(session, &p.chemist)?;
    let [mut task, _private] = scientist.par()?;
    let mut owner = rv.enter(session, &p.hplc_service)?;
    let instrument = rv.enter(session, &p.hplc)?;
    let effects = Effects(Cell::new(0));
    let result: ShellResult<()> = hibana_runtime::race(
        localside::instruments::hplc(&mut owner, instrument, &effects),
        async {
            let admitted = serde_json::to_vec(&json!({"sample":"synthesis_qc"}))
                .map_err(|e| ShellError::message(e.to_string()))?;
            task.send::<HplcIntent>(&admitted.as_slice()).await?;
            let mut instrument = task.recv::<HplcLoan>().await?;
            let substituted = serde_json::to_vec(&json!({"sample":"documentation"}))
                .map_err(|e| ShellError::message(e.to_string()))?;
            instrument
                .send::<HplcRequest>(&substituted.as_slice())
                .await?;
            instrument.recv::<HplcResult>().await?;
            Ok(())
        },
    )
    .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("differs from admitted")),
        "{result:?}"
    );
    assert_eq!(effects.0.get(), 0);
    Ok(())
}
