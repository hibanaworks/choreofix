//! Real owned Endpoint transfers through the production formulation receiver.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::{
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

// Only an owned effect journal and fault injection. It grants no Endpoint or
// protocol progress and never inspects the receiver's cursor.
struct Formulations {
    failures: usize,
    journal: Cell<Vec<Value>>,
}
impl Domain for Formulations {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        if agent != "FORMULATION_SCIENTIST"
            || tool != "prepare_formulation"
            || arguments != json!({"compound":"compound-7"})
        {
            return Err("unexpected formulation effect".into());
        }
        let mut journal = self.journal.take();
        let event = 20 + journal.len();
        let success = journal.len() >= self.failures;
        let result = json!({"tool_name":tool,"success":success,"data":{
            "scientist":agent,"compound":"compound-7","formulation":format!("formulation-{event}"),
            "event_id":event,"artifact_id":if success{Some(event)}else{None},
            "status":if success{"complete"}else{"injected_failure"},"inputs":{"compound":7}}});
        journal.push(result.clone());
        self.journal.set(journal);
        Ok(result)
    }
}
fn identity() -> Value {
    json!({"tool_name":"run_mass_spec_analysis","success":true,"data":{
        "scientist":"CHEMIST","sample":"molecular_id","compound":"compound-7",
        "event_id":7,"artifact_id":7,"status":"complete","inputs":{"qc":1}}})
}
// Supplied terminal data isolates this receiver. The remaining scientific
// effects are not executed or certified by this native component fixture.
fn terminal(formulation: &Value) -> Value {
    json!({"outcome":"continue","compound":identity(),"formulation":formulation,
        "clinical":{"tool_name":"review_clinical_data","success":true,"data":{
            "scientist":"CLINICAL_LEAD","data_type":"combined_review","event_id":100,
            "artifact_id":100,"status":"complete","assessment":"supported",
            "inputs":{"formulation":formulation["data"]["event_id"]}}},
        "regulatory":{"tool_name":"verify_regulatory_compliance","success":true,"data":{
            "scientist":"REGULATORY_SPECIALIST","review_type":"initial_review","event_id":101,
            "artifact_id":101,"status":"complete","inputs":{"clinical":100}}},
        "decision":{"tool_name":"submit_project_decision","success":true,"data":{
            "scientist":"PROJECT_DIRECTOR","outcome":"continue","event_id":102,
            "artifact_id":102,"status":"complete","inputs":{"compound":7,
                "formulation":formulation["data"]["event_id"],"clinical":100,"regulatory":101}}}})
}
fn boundary_round() -> impl Projectable {
    g::seq(
        super::super::global::formulation(),
        g::par(
            g::send::<CLINICIAN, DIRECTOR, Specimen<FORMULATION>>(),
            g::send::<STUDY_SERVICE, DIRECTOR, DecisionEvidence<DIRECTOR>>(),
        ),
    )
}
fn boundary_entry() -> impl Projectable {
    g::seq(
        g::send::<DIRECTOR, FORMULATOR, Specimen<FORMULATION>>(),
        g::send::<FORMULATION, FORM_SERVICE, Intent<4>>(),
    )
}
fn boundary_finish() -> impl Projectable {
    g::seq(
        g::send::<DIRECTOR, HPLC_SERVICE, Archived<FORMULATION>>(),
        g::par(
            g::send::<DIRECTOR, FORMULATOR, Final<FORMULATOR>>(),
            g::par(
                g::send::<DIRECTOR, CLINICIAN, Final<CLINICIAN>>(),
                g::par(
                    g::send::<DIRECTOR, HPLC_SERVICE, Final<HPLC_SERVICE>>(),
                    g::par(
                        g::send::<DIRECTOR, CELL_SERVICE, Final<CELL_SERVICE>>(),
                        g::par(
                            g::send::<DIRECTOR, MASS_SERVICE, Final<MASS_SERVICE>>(),
                            g::par(
                                g::send::<DIRECTOR, STUDY_SERVICE, Final<STUDY_SERVICE>>(),
                                g::send::<DIRECTOR, FORM_SERVICE, Final<FORM_SERVICE>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn source() -> impl Projectable {
    g::seq(
        g::send::<MASS_SERVICE, FORM_SERVICE, Identity<FORM_SERVICE>>(),
        g::seq(
            g::seq(boundary_entry(), boundary_round()).roll(),
            boundary_finish(),
        ),
    )
}

#[hibana_runtime::main]
#[test]
async fn formulation_retries_actual_effects_and_retires_the_moved_material() -> ShellResult<()> {
    for (failures, attack_retired_material) in
        (0..=3).flat_map(|failures| [(failures, false), (failures, true)])
    {
        let source = source();
        let formulator_program = project::<FORMULATOR>(&source);
        let material_program = project::<FORMULATION>(&source);
        let service_program = project::<FORM_SERVICE>(&source);
        let instrument_program = project::<FORM>(&source);
        let mass_program = project::<MASS_SERVICE>(&source);
        let hplc_program = project::<HPLC_SERVICE>(&source);
        let cells_program = project::<CELL_SERVICE>(&source);
        let study_program = project::<STUDY_SERVICE>(&source);
        let clinician_program = project::<CLINICIAN>(&source);
        let director_program = project::<DIRECTOR>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["FORMULATION_SCIENTIST"],
            [(session, FORM), (session, FORMULATION)],
        );
        let mut client_slab = vec![0; 1024 * 1024];
        let mut service_slab = vec![0; 1024 * 1024];
        let mut client_storage = SessionKitStorage::uninit();
        let mut service_storage = SessionKitStorage::uninit();
        let client_runtime = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let service_runtime = service_storage
            .init()
            .rendezvous(&mut service_slab, &bus.transport)?;
        let mut formulator = client_runtime.enter(session, &formulator_program)?;
        let mut director = service_runtime.enter(session, &director_program)?;
        let material = service_runtime.enter(session, &material_program)?;
        let mut service = service_runtime.enter(session, &service_program)?;
        let instrument = service_runtime.enter(session, &instrument_program)?;
        let mut mass = service_runtime.enter(session, &mass_program)?;
        let mut hplc = service_runtime.enter(session, &hplc_program)?;
        let mut cells = service_runtime.enter(session, &cells_program)?;
        let mut study = service_runtime.enter(session, &study_program)?;
        let mut clinician = service_runtime.enter(session, &clinician_program)?;
        let effects = Formulations {
            failures,
            journal: Cell::new(Vec::new()),
        };
        let result = hibana_runtime::race(
            async {
                async {
                    super::formulation::run(&mut service, instrument, &effects).await?;
                };
                async {
                    let mut material = formulator.recv::<Specimen<FORMULATION>>().await?;
                    let bytes = imp::encode(&json!({"compound":"compound-7"}))?;
                    loop {
                        let input = imp::AnalysisArguments::<4>::from_bytes(&bytes)?;
                        material.send::<Intent<4>>(&input).await?;
                        let mut instrument = formulator.recv::<Instrument<FORM>>().await?;
                        instrument.send::<Request<4>>(&input).await?;
                        formulator.send::<Instrument<FORM>>(instrument).await?;
                        formulator.send::<Specimen<FORMULATION>>(material).await?;
                        let branch = formulator.offer().await?;
                        match branch.label() {
                            label if label == Passed::<4>::LOGICAL_LABEL => {
                                let actual = imp::decode(branch.recv::<Passed<4>>().await?)?;
                                assert_eq!(actual["success"], true);
                                break;
                            }
                            label if label == Failed::<4>::LOGICAL_LABEL => {
                                let actual = imp::decode(branch.recv::<Failed<4>>().await?)?;
                                assert_eq!(actual["success"], false);
                                material = formulator.recv::<Specimen<FORMULATION>>().await?;
                            }
                            _ => return Err(ShellError::message("unexpected formulation result")),
                        }
                    }
                    formulator.recv::<Final<FORMULATOR>>().await?;
                    Ok(())
                };
                async {
                    let mut retired = clinician.recv::<Specimen<FORMULATION>>().await?;
                    if attack_retired_material {
                        let bytes = imp::encode(&json!({"compound":"compound-7"}))?;
                        let input = imp::AnalysisArguments::<4>::from_bytes(&bytes)?;
                        drop(bus.transport.take_observations());
                        let result = hibana_runtime::race(
                            async {
                                Ok::<_, ShellError>(Some(retired.send::<Intent<4>>(&input).await))
                            },
                            async { Ok(None) },
                        )
                        .await?;
                        assert!(
                            matches!(result, Some(Err(_))),
                            "retired material must reject on its first poll"
                        );
                        assert!(
                            bus.transport
                                .take_observations()
                                .iter()
                                .all(|row| row["action"] != "send")
                        );
                        // This is a terminal rejection, not a recoverable call.
                        // End this negative run instead of continuing a faulted
                        // session; the success path uses its own fresh runtimes.
                        return Err(ShellError::message("retired material attack rejected"));
                    }
                    clinician.send::<Specimen<FORMULATION>>(retired).await?;
                    clinician.recv::<Final<CLINICIAN>>().await?;
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&identity())?;
                    mass.send::<Identity<FORM_SERVICE>>(&imp::JsonTo::<FORM_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
                    let actual = imp::decode(mass.recv::<Formulation<MASS_SERVICE>>().await?)?;
                    assert_eq!(actual["data"]["event_id"], 20 + failures);
                    assert_eq!(actual["success"], true);
                    mass.recv::<Final<MASS_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    let actual = imp::decode(hplc.recv::<Formulation<HPLC_SERVICE>>().await?)?;
                    assert_eq!(actual["data"]["event_id"], 20 + failures);
                    assert_eq!(actual["success"], true);
                    let _archived = hplc.recv::<Archived<FORMULATION>>().await?;
                    hplc.recv::<Final<HPLC_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    let actual = imp::decode(cells.recv::<Formulation<CELL_SERVICE>>().await?)?;
                    assert_eq!(actual["data"]["event_id"], 20 + failures);
                    assert_eq!(actual["success"], true);
                    cells.recv::<Final<CELL_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    let actual = imp::decode(study.recv::<Formulation<STUDY_SERVICE>>().await?)?;
                    assert_eq!(actual["data"]["event_id"], 20 + failures);
                    assert_eq!(actual["success"], true);
                    let final_result = terminal(&actual);
                    let bytes = imp::encode(&final_result)?;
                    study
                        .send::<DecisionEvidence<DIRECTOR>>(&imp::JsonTo::<DIRECTOR>::from_bytes(
                            &bytes,
                        )?)
                        .await?;
                    study.recv::<Final<STUDY_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    director.send::<Specimen<FORMULATION>>(material).await?;
                    let (material, bytes) = async {
                        let [mut custody, mut data] = director.par()?;
                        async { custody.recv::<Specimen<FORMULATION>>().await };
                        async { Ok(data.recv::<DecisionEvidence<DIRECTOR>>().await?.to_vec()) };
                    }
                    .await?;
                    director.send::<Archived<FORMULATION>>(material).await?;
                    async {
                        let [
                            mut formulator,
                            mut clinician,
                            mut hplc,
                            mut cells,
                            mut mass,
                            mut study,
                            mut form,
                        ] = director.par()?;
                        async {
                            formulator
                                .send::<Final<FORMULATOR>>(
                                    &imp::FinalOutcome::<FORMULATOR>::from_bytes(&bytes)?,
                                )
                                .await?;
                        };
                        async {
                            clinician
                                .send::<Final<CLINICIAN>>(
                                    &imp::FinalOutcome::<CLINICIAN>::from_bytes(&bytes)?,
                                )
                                .await?;
                        };
                        async {
                            hplc.send::<Final<HPLC_SERVICE>>(
                                &imp::FinalOutcome::<HPLC_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await?;
                        };
                        async {
                            cells
                                .send::<Final<CELL_SERVICE>>(
                                    &imp::FinalOutcome::<CELL_SERVICE>::from_bytes(&bytes)?,
                                )
                                .await?;
                        };
                        async {
                            mass.send::<Final<MASS_SERVICE>>(
                                &imp::FinalOutcome::<MASS_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await?;
                        };
                        async {
                            study
                                .send::<Final<STUDY_SERVICE>>(
                                    &imp::FinalOutcome::<STUDY_SERVICE>::from_bytes(&bytes)?,
                                )
                                .await?;
                        };
                        async {
                            form.send::<Final<FORM_SERVICE>>(
                                &imp::FinalOutcome::<FORM_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await?;
                        };
                        Ok(())
                    }
                    .await?;
                    Ok(())
                };
                Ok(())
            },
            hibana_runtime::host::deadline(Duration::from_secs(5)),
        )
        .await;
        if attack_retired_material {
            assert!(result.is_err_and(|error| {
                error
                    .to_string()
                    .contains("retired material attack rejected")
            }));
        } else {
            result?;
        }
        let journal = effects.journal.take();
        assert_eq!(journal.len(), failures + 1);
        assert_eq!(
            journal.iter().filter(|row| row["success"] == true).count(),
            1
        );
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn formulation_receiver_rejects_changed_admission_or_resource_request_before_effect()
-> ShellResult<()> {
    for change_admission in [false, true] {
        let source = source();
        let fp = project::<FORMULATOR>(&source);
        let cp = project::<FORMULATION>(&source);
        let sp = project::<FORM_SERVICE>(&source);
        let ip = project::<FORM>(&source);
        let mp = project::<MASS_SERVICE>(&source);
        let dp = project::<DIRECTOR>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["FORMULATION_SCIENTIST"],
            [(session, FORM), (session, FORMULATION)],
        );
        let mut a_slab = vec![0; 1024 * 1024];
        let mut b_slab = vec![0; 1024 * 1024];
        let mut a_storage = SessionKitStorage::uninit();
        let mut b_storage = SessionKitStorage::uninit();
        let a = a_storage.init().rendezvous(&mut a_slab, &bus.transport)?;
        let b = b_storage.init().rendezvous(&mut b_slab, &bus.transport)?;
        let mut formulator = a.enter(session, &fp)?;
        let mut director = b.enter(session, &dp)?;
        let material = b.enter(session, &cp)?;
        let mut service = b.enter(session, &sp)?;
        let instrument = b.enter(session, &ip)?;
        let mut mass = b.enter(session, &mp)?;
        let effects = Formulations {
            failures: 0,
            journal: Cell::new(Vec::new()),
        };
        let (result, _, _, _) = hibana_runtime::race(
            async {
                async {
                    Ok::<_, ShellError>(
                        super::formulation::run(&mut service, instrument, &effects).await,
                    )
                };
                async {
                    let mut material = formulator.recv::<Specimen<FORMULATION>>().await?;
                    let bytes = imp::encode(
                        &json!({"compound":if change_admission{"compound-8"}else{"compound-7"}}),
                    )?;
                    material
                        .send::<Intent<4>>(&imp::AnalysisArguments::<4>::from_bytes(&bytes)?)
                        .await?;
                    if !change_admission {
                        let mut instrument = formulator.recv::<Instrument<FORM>>().await?;
                        let substituted = imp::encode(&json!({"compound":"compound-8"}))?;
                        instrument
                            .send::<Request<4>>(&imp::AnalysisArguments::<4>::from_bytes(
                                &substituted,
                            )?)
                            .await?;
                    }
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&identity())?;
                    mass.send::<Identity<FORM_SERVICE>>(&imp::JsonTo::<FORM_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
                    Ok(())
                };
                async {
                    director.send::<Specimen<FORMULATION>>(material).await?;
                    Ok(())
                };
            },
            hibana_runtime::host::deadline(Duration::from_secs(5)),
        )
        .await?;
        assert!(result.is_err());
        assert!(effects.journal.take().is_empty());
    }
    Ok(())
}
