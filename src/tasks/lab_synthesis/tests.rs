use super::{global::*, imp, localside};
use crate::shell::{
    ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};
use std::cell::Cell;
// Observation only: this count never authorizes a step.
struct Effects(Cell<usize>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        self.0.set(self.0.get() + 1);
        Ok(
            json!({"tool_name":tool,"success":true,"data":{"scientist":agent,"arguments":arguments}}),
        )
    }
}
#[hibana_runtime::main]
#[test]
async fn spectrometer_rejects_an_unassigned_specimen_before_effect() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION + 1);
    let (bus, _reader, _terminals) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "ANALYST"],
        [(session, SPECTROMETER), (session, CENTRIFUGE)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut scientist = rv.enter(session, &p.chemist)?;
    let [mut task, _private] = scientist.par()?;
    let mut owner = rv.enter(session, &p.spec_service)?;
    let instrument = rv.enter(session, &p.spectrometer)?;
    let effects = Effects(Cell::new(0));
    let result: ShellResult<()> = hibana_runtime::race(
        localside::instruments::spectrometer(&mut owner, instrument, &effects),
        async {
            let admitted = imp::encode(&json!({"sample":"final_validation"}))?;
            task.send::<SpecIntent>(&admitted.as_slice()).await?;
            let mut instrument = task.recv::<SpecLoan>().await?;
            let changed = imp::encode(&json!({"sample":"raw_materials"}))?;
            instrument.send::<SpecRequest>(&changed.as_slice()).await?;
            instrument.recv::<SpecResult>().await?;
            Ok(())
        },
    )
    .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("assigned specimen")),
        "{result:?}"
    );
    assert_eq!(effects.0.get(), 0);
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn spectrometer_rejects_changed_resource_arguments_before_effect() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION + 2);
    let (bus, _reader, _terminals) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "ANALYST"],
        [(session, SPECTROMETER), (session, CENTRIFUGE)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut scientist = rv.enter(session, &p.chemist)?;
    let [mut task, _private] = scientist.par()?;
    let mut owner = rv.enter(session, &p.spec_service)?;
    let instrument = rv.enter(session, &p.spectrometer)?;
    let effects = Effects(Cell::new(0));
    let result: ShellResult<()> = hibana_runtime::race(
        localside::instruments::spectrometer(&mut owner, instrument, &effects),
        async {
            let admitted = imp::encode(&json!({"sample":"raw_materials"}))?;
            task.send::<SpecIntent>(&admitted.as_slice()).await?;
            let mut instrument = task.recv::<SpecLoan>().await?;
            let changed = imp::encode(&json!({"sample":"final_validation"}))?;
            instrument.send::<SpecRequest>(&changed.as_slice()).await?;
            instrument.recv::<SpecResult>().await?;
            Ok(())
        },
    )
    .await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("admitted actual data")),
        "{result:?}"
    );
    assert_eq!(effects.0.get(), 0);
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn bioassay_rejects_a_forged_purified_specimen_after_real_preparation() -> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(SESSION + 3);
    let (bus, _reader, _terminals) = Bus::open(
        ["CHEMIST", "BIOLOGIST", "ANALYST"],
        [(session, SPECTROMETER), (session, CENTRIFUGE)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut chemist = rv.enter(session, &p.chemist)?;
    let [mut c, _cp] = chemist.par()?;
    let mut biologist = rv.enter(session, &p.biologist)?;
    let [mut b, _bp] = biologist.par()?;
    let mut spec = rv.enter(session, &p.spec_service)?;
    let spectrometer = rv.enter(session, &p.spectrometer)?;
    let mut cent = rv.enter(session, &p.cent_service)?;
    let centrifuge = rv.enter(session, &p.centrifuge)?;
    let effects = Effects(Cell::new(0));
    let result:ShellResult<()>=hibana_runtime::race(
  async {
   async {localside::instruments::spectrometer(&mut spec,spectrometer,&effects).await?;};
   async {localside::instruments::centrifuge(&mut cent,centrifuge,&effects).await?;};
   Ok(())
  },
  async {
   async {
    let args=imp::encode(&json!({"sample":"raw_materials"}))?;
    c.send::<SpecIntent>(&args.as_slice()).await?;
    let mut instrument=c.recv::<SpecLoan>().await?;
    instrument.send::<SpecRequest>(&args.as_slice()).await?;
    let raw=imp::decode(instrument.recv::<SpecResult>().await?)?;
    c.send::<SpecLoan>(instrument).await?;
    let args=imp::encode(&json!({"material":"compound_purification"}))?;
    c.send::<CentIntent>(&args.as_slice()).await?;
    let mut instrument=c.recv::<CentLoan>().await?;
    instrument.send::<CentRequest>(&args.as_slice()).await?;
    let purification=imp::decode(instrument.recv::<CentResult>().await?)?;
    c.send::<CentLoan>(instrument).await?;
    let bytes=imp::encode(&json!({"raw":raw,"purification":purification}))?;
    c.send::<Sample>(&bytes.as_slice()).await?;
    c.recv::<Quality>().await?;Ok(())
   };
   async {
    let args=imp::encode(&json!({"material":"cell_cultures"}))?;
    b.send::<CentIntent>(&args.as_slice()).await?;
    let mut instrument=b.recv::<CentLoan>().await?;
    instrument.send::<CentRequest>(&args.as_slice()).await?;
    let prep=imp::decode(instrument.recv::<CentResult>().await?)?;
    b.send::<CentLoan>(instrument).await?;
    let args=imp::encode(&json!({"sample":"fluorescence"}))?;
    b.send::<SpecIntent>(&args.as_slice()).await?;
    let mut instrument=b.recv::<SpecLoan>().await?;
    instrument.send::<SpecRequest>(&args.as_slice()).await?;
    instrument.recv::<SpecResult>().await?;
    b.send::<SpecLoan>(instrument).await?;
    b.recv::<Sample>().await?;
    let forged=imp::encode(&json!({"arguments":{"compound":"compound_purification"},"prep":prep,"purification":{"forged":true}}))?;
    b.send::<BioRequest>(&forged.as_slice()).await?;
    b.recv::<BioResult>().await?;Ok(())
   };
   Ok(())
  }
 ).await;
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("admitted actual data")),
        "{result:?}"
    );
    assert_eq!(
        effects.0.get(),
        4,
        "only the four real preparation effects may run"
    );
    Ok(())
}
