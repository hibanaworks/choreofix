//! Native instrument fixture. Other scientists' supplied records are not
//! executions of their effects, Wasm qualification, or scored model work.
use super::super::{global, global::*, imp};
use crate::shell::{
    ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

struct Measurements {
    actual: Value,
    journal: Cell<Vec<Value>>,
}
impl Domain for Measurements {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        input: Value,
    ) -> Result<Value, String> {
        for step in [2, 3, 9] {
            let (expected_actor, expected_tool, key, value) =
                imp::analysis(step).map_err(|e| e.to_string())?;
            if actor == expected_actor
                && tool == expected_tool
                && input == json!({key:value.unwrap()})
            {
                let actual = self.actual[step.to_string()].clone();
                let mut journal = self.journal.take();
                journal.push(json!({"actor":actor,"tool":tool,"input":input,"result":actual}));
                self.journal.set(journal);
                return Ok(actual);
            }
        }
        Err("unexpected instrument request".into())
    }
}
fn result(step: u8, event: u64, success: bool, inputs: Value) -> Value {
    let (actor, tool, key, value) = imp::analysis(step).unwrap();
    json!({"tool_name":tool,"success":success,"data":{
        "scientist":actor,key:value.unwrap_or("compound-3"),"event_id":event,
        "artifact_id":if success{Some(event)}else{None},
        "status":if success{"complete"}else if step==10{"clinical_hold"}else{"injected_failure"},
        "inputs":inputs}})
}
fn source() -> impl Projectable {
    let identity = g::seq(
        global::mass::<CHEMIST, 3>(),
        g::par(
            g::send::<MASS_SERVICE, FORMULATOR, Identity<FORMULATOR>>(),
            g::par(
                g::send::<MASS_SERVICE, FORM_SERVICE, Identity<FORM_SERVICE>>(),
                g::send::<MASS_SERVICE, STUDY_SERVICE, Identity<STUDY_SERVICE>>(),
            ),
        ),
    );
    let initial = g::seq(
        global::mass::<TOXICOLOGIST, 2>(),
        g::seq(
            g::send::<MASS_SERVICE, CELL_SERVICE, Baseline<CELL_SERVICE>>(),
            g::seq(
                g::send::<HPLC_SERVICE, MASS_SERVICE, QcEvidence>(),
                identity,
            ),
        ),
    );
    let inputs = g::seq(
        g::send::<FORM_SERVICE, MASS_SERVICE, Formulation<MASS_SERVICE>>(),
        g::seq(
            g::send::<HPLC_SERVICE, MASS_SERVICE, PurityEvidence<MASS_SERVICE>>(),
            g::par(
                g::seq(
                    g::send::<CELL_SERVICE, MASS_SERVICE, BioEvidence<MASS_SERVICE>>(),
                    g::send::<CELL_SERVICE, MASS_SERVICE, ToxEvidence<MASS_SERVICE>>(),
                ),
                g::send::<HPLC_SERVICE, MASS_SERVICE, StabilityEvidence<MASS_SERVICE>>(),
            ),
        ),
    );
    let clinical = g::seq(
        global::trial::<CLINICIAN, MASS_SERVICE, MASS, STUDY, 9>(),
        g::seq(
            global::passed::<CLINICIAN, MASS_SERVICE, STUDY, 9, REGULATOR>(),
            g::par(
                g::send::<MASS_SERVICE, REGULATOR, ClinicalEvidence<REGULATOR>>(),
                g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
            ),
        ),
    );
    let terminal = g::seq(
        g::send::<DIRECTOR, MASS_SERVICE, Final<MASS_SERVICE>>(),
        g::send::<REGULATOR, MASS_SERVICE, Specimen<STUDY>>(),
    );
    g::seq(initial, g::seq(inputs, g::seq(clinical, terminal)))
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(crate::clock::Clock))]
#[test]
async fn clinical_owner_executes_actual_requests_and_rejects_mixed_material_before_the_effect()
-> ShellResult<()> {
    #[cfg(miri)]
    use crate::clock::deadline;
    #[cfg(not(miri))]
    use hibana_runtime::host::deadline;
    for bio_ok in [false, true] {
        for tox_ok in [false, true] {
            for mixed_material in [false, true] {
                let baseline = result(2, 2, true, json!({}));
                let qc = result(0, 1, true, json!({}));
                let mut identity = result(3, 3, true, json!({"qc":1}));
                identity["data"]["compound"] = json!("compound-3");
                let formulation = result(4, 4, true, json!({"compound":3}));
                let bio = result(5, 5, bio_ok, json!({"formulation":4,"cells":0}));
                let purity = result(7, 7, true, json!({"formulation":4}));
                let stability = result(
                    6,
                    6,
                    true,
                    json!({"formulation":if mixed_material{40}else{4},"cells":0}),
                );
                let tox = result(
                    8,
                    8,
                    tox_ok,
                    json!({"formulation":4,"cells":0,"baseline":2,"purity":7}),
                );
                let mut clinical = result(
                    9,
                    9,
                    true,
                    json!({"formulation":4,"bioassay":5,"purity":7,"stability":6,"toxicity":8}),
                );
                clinical["data"]["assessment"] = json!(if bio_ok && tox_ok {
                    "supported"
                } else {
                    "insufficient_data"
                });
                let regulatory = result(10, 10, bio_ok && tox_ok, json!({"clinical":9}));
                let outcome = if bio_ok && tox_ok {
                    "continue"
                } else {
                    "abort"
                };
                let decision = result(
                    if bio_ok && tox_ok { 12 } else { 14 },
                    11,
                    true,
                    json!({"compound":3,"formulation":4,"clinical":9,"regulatory":10}),
                );
                let final_result = json!({"outcome":outcome,"compound":identity,"formulation":formulation,
                    "clinical":clinical,"regulatory":regulatory,"decision":decision});
                let effects = Measurements {
                    actual: json!({"2":baseline,"3":identity,"9":clinical}),
                    journal: Cell::new(Vec::new()),
                };

                let source = source();
                let service_program = project::<MASS_SERVICE>(&source);
                let instrument_program = project::<MASS>(&source);
                let chemist_program = project::<CHEMIST>(&source);
                let tox_program = project::<TOXICOLOGIST>(&source);
                let formulator_program = project::<FORMULATOR>(&source);
                let form_program = project::<FORM_SERVICE>(&source);
                let hplc_program = project::<HPLC_SERVICE>(&source);
                let cells_program = project::<CELL_SERVICE>(&source);
                let clinical_program = project::<CLINICIAN>(&source);
                let specimen_program = project::<STUDY>(&source);
                let regulator_program = project::<REGULATOR>(&source);
                let director_program = project::<DIRECTOR>(&source);
                let study_program = project::<STUDY_SERVICE>(&source);
                let sid = SessionId::new(SESSION);
                let (bus, _, _) = Bus::open(
                    ["CHEMIST", "TOXICOLOGIST", "CLINICAL_LEAD"],
                    [(sid, MASS), (sid, STUDY)],
                );
                let mut client_slab = vec![0; 1024 * 1024];
                let mut owner_slab = vec![0; 1024 * 1024];
                let mut client_storage = SessionKitStorage::uninit();
                let mut owner_storage = SessionKitStorage::uninit();
                let client = client_storage
                    .init()
                    .rendezvous(&mut client_slab, &bus.transport)?;
                let owner = owner_storage
                    .init()
                    .rendezvous(&mut owner_slab, &bus.transport)?;
                let mut service = owner.enter(sid, &service_program)?;
                let instrument = owner.enter(sid, &instrument_program)?;
                let mut chemist = client.enter(sid, &chemist_program)?;
                let mut toxicologist = client.enter(sid, &tox_program)?;
                let mut formulator = client.enter(sid, &formulator_program)?;
                let mut form = client.enter(sid, &form_program)?;
                let mut hplc = client.enter(sid, &hplc_program)?;
                let mut cells = client.enter(sid, &cells_program)?;
                let mut clinician = client.enter(sid, &clinical_program)?;
                let mut specimen = client.enter(sid, &specimen_program)?;
                let mut regulator = client.enter(sid, &regulator_program)?;
                let mut director = client.enter(sid, &director_program)?;
                let mut study = client.enter(sid, &study_program)?;
                let actual = hibana_runtime::race(
                    async {
                        async {
                            super::mass::run(&mut service, instrument, &effects).await?;
                        };
                        async {
                            let bytes = imp::encode(&json!({"sample":"baseline_markers"}))?;
                            let input = imp::AnalysisArguments::<2>::from_bytes(&bytes)?;
                            toxicologist.send::<Intent<2>>(&input).await?;
                            let mut instrument = toxicologist.recv::<Instrument<MASS>>().await?;
                            instrument.send::<Request<2>>(&input).await?;
                            let actual = imp::decode(instrument.recv::<Passed<2>>().await?)?;
                            imp::same(&actual, &baseline)?;
                            toxicologist.send::<Instrument<MASS>>(instrument).await?;
                            Ok(())
                        };
                        async {
                            let bytes = imp::encode(&json!({"sample":"molecular_id"}))?;
                            let input = imp::AnalysisArguments::<3>::from_bytes(&bytes)?;
                            chemist.send::<Intent<3>>(&input).await?;
                            let mut instrument = chemist.recv::<Instrument<MASS>>().await?;
                            instrument.send::<Request<3>>(&input).await?;
                            let actual = imp::decode(instrument.recv::<Passed<3>>().await?)?;
                            imp::same(&actual, &identity)?;
                            chemist.send::<Instrument<MASS>>(instrument).await?;
                            Ok(())
                        };
                        async {
                            let actual =
                                imp::decode(formulator.recv::<Identity<FORMULATOR>>().await?)?;
                            imp::same(&actual, &identity)?;
                            Ok(())
                        };
                        async {
                            let actual = imp::decode(form.recv::<Identity<FORM_SERVICE>>().await?)?;
                            imp::same(&actual, &identity)?;
                            let bytes = imp::encode(&formulation)?;
                            form.send::<Formulation<MASS_SERVICE>>(
                                &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await?;
                            Ok(())
                        };
                        async {
                            let bytes = imp::encode(&qc)?;
                            hplc.send::<QcEvidence>(&imp::JsonTo::<MASS_SERVICE>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                            let bytes = imp::encode(&purity)?;
                            hplc.send::<PurityEvidence<MASS_SERVICE>>(
                                &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await?;
                            let bytes = imp::encode(&stability)?;
                            hplc.send::<StabilityEvidence<MASS_SERVICE>>(&imp::JsonTo::<
                                MASS_SERVICE,
                            >::from_bytes(
                                &bytes
                            )?)
                            .await?;
                            Ok(())
                        };
                        async {
                            let actual =
                                imp::decode(cells.recv::<Baseline<CELL_SERVICE>>().await?)?;
                            imp::same(&actual, &baseline)?;
                            let bytes = imp::encode(&bio)?;
                            cells
                                .send::<BioEvidence<MASS_SERVICE>>(
                                    &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                                )
                                .await?;
                            let bytes = imp::encode(&tox)?;
                            cells
                                .send::<ToxEvidence<MASS_SERVICE>>(
                                    &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                                )
                                .await?;
                            Ok(())
                        };
                        async {
                            let bytes = imp::encode(&json!({"data_type":"combined_review"}))?;
                            let input = imp::AnalysisArguments::<9>::from_bytes(&bytes)?;
                            specimen.send::<Intent<9>>(&input).await?;
                            let mut instrument = clinician.recv::<Instrument<MASS>>().await?;
                            instrument.send::<Request<9>>(&input).await?;
                            clinician.send::<Instrument<MASS>>(instrument).await?;
                            clinician.send::<Specimen<STUDY>>(specimen).await?;
                            let actual = imp::decode(clinician.recv::<Passed<9>>().await?)?;
                            imp::same(&actual, &clinical)?;
                            Ok(())
                        };
                        async {
                            let specimen = regulator.recv::<Specimen<STUDY>>().await?;
                            let actual = imp::decode(
                                regulator.recv::<ClinicalEvidence<REGULATOR>>().await?,
                            )?;
                            imp::same(&actual, &clinical)?;
                            regulator.send::<Specimen<STUDY>>(specimen).await?;
                            Ok(())
                        };
                        async {
                            let bytes = imp::encode(&final_result)?;
                            director
                                .send::<Final<MASS_SERVICE>>(
                                    &imp::FinalOutcome::<MASS_SERVICE>::from_bytes(&bytes)?,
                                )
                                .await?;
                            Ok(())
                        };
                        async {
                            let actual =
                                imp::decode(study.recv::<Identity<STUDY_SERVICE>>().await?)?;
                            imp::same(&actual, &identity)?;
                            let actual = imp::decode(
                                study.recv::<ClinicalEvidence<STUDY_SERVICE>>().await?,
                            )?;
                            imp::same(&actual, &clinical)?;
                            Ok(())
                        };
                        Ok(())
                    },
                    deadline(Duration::from_secs(if cfg!(miri) { 600 } else { 5 })),
                )
                .await;
                let journal = effects.journal.take();
                if mixed_material {
                    assert_eq!(
                        actual.unwrap_err().to_string(),
                        "clinical evidence uses different materials"
                    );
                    assert_eq!(
                        journal.len(),
                        2,
                        "mixed materials must not execute clinical review"
                    );
                } else {
                    actual?;
                    assert_eq!(journal.len(), 3);
                }
            }
        }
    }
    Ok(())
}
