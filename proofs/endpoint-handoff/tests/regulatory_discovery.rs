//! The production 14M scientific protocol, isolated from unrelated globals.
#[path = "../../../src/imp/json.rs"]
pub(crate) mod json;
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{json, projection, transport};
}
#[path = "../../../src/tasks/regulatory_discovery/mod.rs"]
pub mod regulatory_discovery;

use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use regulatory_discovery::{global::*, imp as data, localside};
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use std::{cell::Cell, time::Duration};
#[cfg(miri)]
#[path = "../../../src/tasks/handoff/tests/clock.rs"]
mod clock;

// Controlled results with an owned append-only effect journal. It does not
// inspect Endpoint state or grant an instrument or scientific continuation.
struct QcEffects {
    rejections: usize,
    journal: Cell<Vec<Value>>,
}
impl Domain for QcEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        if agent != "CHEMIST"
            || tool != "run_hplc_analysis"
            || arguments != json!({"sample":"synthesis_qc"})
        {
            return Err("unexpected effect in the synthesis QC fixture".into());
        }
        let mut journal = self.journal.take();
        let success = journal.len() >= self.rejections;
        let result = json!({"tool_name":tool,"success":success,"data":{
            "scientist":agent,"sample":"synthesis_qc","inputs":{},
            "artifact_id":if success {Some(journal.len())} else {None},
            "status":if success {"complete"} else {"injected_failure"}}});
        journal.push(json!({"agent":agent,"arguments":arguments,"result":result}));
        self.journal.set(journal);
        Ok(result)
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn synthesis_qc_retries_actual_results_then_retires_its_continuation() -> ShellResult<()> {
    #[cfg(miri)]
    use clock::deadline;
    use hibana::{g::Message, runtime::program::project};
    #[cfg(not(miri))]
    use hibana_runtime::host::deadline;
    for rejected in 0..=3 {
        let source = source();
        let chemist_program = project::<CHEMIST>(&source);
        let qc_program = project::<QC>(&source);
        let service_program = project::<HPLC_SERVICE>(&source);
        let instrument_program = project::<HPLC>(&source);
        let lead_program = project::<LEAD>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["CHEMIST", "LEAD_SCIENTIST"],
            [(session, HPLC), (session, QC)],
        );
        let mut chemist_slab = vec![0; 1024 * 1024];
        let mut service_slab = vec![0; 1024 * 1024];
        let mut lead_slab = vec![0; 1024 * 1024];
        let actual = Cell::new(Value::Null);
        let mut chemist_storage = SessionKitStorage::uninit();
        let mut service_storage = SessionKitStorage::uninit();
        let mut lead_storage = SessionKitStorage::uninit();
        let chemist_runtime = chemist_storage
            .init()
            .rendezvous(&mut chemist_slab, &bus.transport)?;
        let service_runtime = service_storage
            .init()
            .rendezvous(&mut service_slab, &bus.transport)?;
        let lead_runtime = lead_storage
            .init()
            .rendezvous(&mut lead_slab, &bus.transport)?;
        service_runtime
            .set_resolver(
                &service_program,
                ResolverRef::<HPLC_RESULT>::decision_state(&actual, data::decide),
            )
            .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let mut chemist = chemist_runtime.enter(session, &chemist_program)?;
        let mut qc = chemist_runtime.enter(session, &qc_program)?;
        let mut service = service_runtime.enter(session, &service_program)?;
        let instrument = service_runtime.enter(session, &instrument_program)?;
        let mut lead = lead_runtime.enter(session, &lead_program)?;
        let effects = QcEffects {
            rejections: rejected,
            journal: Cell::new(Vec::new()),
        };
        let bytes = data::encode(&json!({"sample":"synthesis_qc"}))?;
        let arguments = data::AnalysisArguments::<0>::from_bytes(&bytes)?;
        hibana_runtime::race(
            async {
                async {
                    async {
                        let mut failures = 0;
                        loop {
                            qc.send::<Intent<0>>(&arguments).await?;
                            let mut instrument = chemist.recv::<Instrument<HPLC>>().await?;
                            instrument.send::<Request<0>>(&arguments).await?;
                            chemist.send::<Instrument<HPLC>>(instrument).await?;
                            chemist.send::<Specimen<QC>>(qc).await?;
                            let branch = chemist.offer().await?;
                            match branch.label() {
                                label if label == Passed::<0>::LOGICAL_LABEL => {
                                    let result = data::decode(branch.recv::<Passed<0>>().await?)?;
                                    assert_eq!(result["success"], true);
                                    assert_eq!(failures, rejected);
                                    break;
                                }
                                label if label == Failed::<0>::LOGICAL_LABEL => {
                                    let result = data::decode(branch.recv::<Failed<0>>().await?)?;
                                    assert_eq!(result["success"], false);
                                    failures += 1;
                                    qc = chemist.recv::<Specimen<QC>>().await?;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected synthesis result {label}"
                                    )));
                                }
                            }
                        }
                    };
                    async {
                        let mut retired = lead.recv::<Specimen<QC>>().await?;
                        let _ = bus.transport.take_observations();
                        let result = hibana_runtime::race(
                            async {
                                Ok::<_, ShellError>(Some(
                                    retired.send::<Intent<0>>(&arguments).await,
                                ))
                            },
                            async { Ok(None) },
                        )
                        .await?;
                        assert!(
                            matches!(result, Some(Err(_))),
                            "retired analysis must reject on its first poll"
                        );
                        assert!(
                            bus.transport
                                .take_observations()
                                .iter()
                                .all(|row| row["action"] != "send"),
                            "a retired analysis must not emit a frame"
                        );
                        Ok::<_, ShellError>(())
                    };
                    Ok::<_, ShellError>(())
                }
                .await?;
                Ok::<_, ShellError>(())
            },
            async {
                hibana_runtime::race(
                    localside::hplc::run(&mut service, (instrument, &actual), &effects),
                    deadline(Duration::from_secs(if cfg!(miri) { 600 } else { 5 })),
                )
                .await?;
                Err(ShellError::message(
                    "full HPLC service unexpectedly completed during synthesis-only probe",
                ))
            },
        )
        .await?;
        let journal = effects.journal.take();
        assert_eq!(journal.len(), rejected + 1);
        assert_eq!(
            journal
                .iter()
                .filter(|row| row["result"]["success"] == true)
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn complete_study_has_no_projection_diagnostic() {
    assert_eq!(
        hibana::g::diagnose(&regulatory_discovery::global::source()),
        None
    );
}

#[test]
fn all_nine_participants_compile_the_same_complete_study() {
    use hibana::runtime::program::project;
    use regulatory_discovery::global::*;
    let source = source();
    let _ = project::<CHEMIST>(&source);
    let _ = project::<BIOLOGIST>(&source);
    let _ = project::<TOXICOLOGIST>(&source);
    let _ = project::<LEAD>(&source);
    let _ = project::<REGULATOR>(&source);
    let _ = project::<HPLC_SERVICE>(&source);
    let _ = project::<MASS_SERVICE>(&source);
    let _ = project::<CELL_SERVICE>(&source);
    let _ = project::<REVIEW_SERVICE>(&source);
}
