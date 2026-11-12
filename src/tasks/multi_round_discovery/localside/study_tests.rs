//! Actual owned tool transfers through the study receiver. Scientific inputs
//! are supplied fixture records; no Wasm, model or common-world score is claimed.
use super::super::{global, global::*, imp};
use crate::shell::{
    ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::{
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
        tap::{self, TapEvent},
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

struct Reviews<'a> {
    clinical: Value,
    rejections: usize,
    journal: Cell<Vec<Value>>,
    source: Vec<TapEvent>,
    observations: &'a Cell<Vec<TapEvent>>,
}
impl Reviews<'_> {
    // Observation only: no Endpoint or domain permission is derived here.
    fn assert_custody(&self, resources: &[u8]) {
        let records = self.observations.take();
        for resource in resources {
            let mut received = 0;
            let mut returned = 0;
            for event in &records {
                if event.causal_key() >> 8 != u16::from(STUDY_SERVICE)
                    || !matches!(event.id(), tap::ENDPOINT_SEND | tap::ENDPOINT_RECV)
                {
                    continue;
                }
                let source = self.source.get(usize::from(event.arg1() as u16)).unwrap();
                assert_eq!(source.ts(), u32::from(event.arg1() as u16));
                if source.causal_key() >> 8 != 1 || source.arg1() != u32::from(*resource) {
                    continue;
                }
                if event.id() == tap::ENDPOINT_RECV && (source.arg0() >> 16) as u8 == STUDY_SERVICE
                {
                    received += 1;
                }
                if event.id() == tap::ENDPOINT_SEND && (source.arg0() >> 24) as u8 == STUDY_SERVICE
                {
                    returned += 1;
                }
            }
            assert_eq!(
                received,
                returned + 1,
                "effect must retain the actual resource {resource}"
            );
        }
        self.observations.set(records);
    }
}
impl Domain for Reviews<'_> {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        input: Value,
    ) -> Result<Value, String> {
        let resources = if input["review_type"] == "hold_review" {
            &[STUDY, HPLC, MASS][..]
        } else {
            &[STUDY][..]
        };
        self.assert_custody(resources);
        let mut journal = self.journal.take();
        let event = 20 + journal.len();
        let actual = match (actor, tool) {
            ("REGULATORY_SPECIALIST", "verify_regulatory_compliance") => {
                let step = if input["review_type"] == "initial_review" {
                    10
                } else {
                    11
                };
                imp::arguments(step, &input).map_err(|e| e.to_string())?;
                let success = self.clinical["data"]["assessment"] == "supported"
                    && journal.len() >= self.rejections;
                result(step, event, success, json!({"clinical":9}))
            }
            ("PROJECT_DIRECTOR", "submit_project_decision") => {
                let step = if input["outcome"] == "continue" {
                    12
                } else {
                    14
                };
                imp::arguments(step, &input).map_err(|e| e.to_string())?;
                let last = journal.last().ok_or("missing actual review")?;
                result(
                    step,
                    event,
                    true,
                    json!({"compound":3,"formulation":4,"clinical":9,
                    "regulatory":last["result"]["data"]["event_id"]}),
                )
            }
            _ => return Err("unexpected study effect".into()),
        };
        journal.push(json!({"actor":actor,"tool_name":tool,"arguments":input,"result":actual}));
        self.journal.set(journal);
        // One Pending poll tests that all required resources survive the await.
        std::future::poll_fn({
            let mut first = true;
            move |cx| {
                if std::mem::take(&mut first) {
                    cx.waker().wake_by_ref();
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            }
        })
        .await;
        self.assert_custody(resources);
        Ok(actual)
    }
}
fn result(step: u8, event: usize, success: bool, inputs: Value) -> Value {
    let (actor, tool, key, value) = imp::analysis(step).unwrap();
    json!({"tool_name":tool,"success":success,"data":{"scientist":actor,
        key:value.unwrap_or("compound-3"),"event_id":event,"artifact_id":if success{Some(event)}else{None},
        "status":if success{"complete"}else{"clinical_hold"},"inputs":inputs}})
}
fn source() -> impl Projectable {
    let inputs = g::seq(
        g::send::<MASS_SERVICE, STUDY_SERVICE, Identity<STUDY_SERVICE>>(),
        g::seq(
            g::send::<FORM_SERVICE, STUDY_SERVICE, Formulation<STUDY_SERVICE>>(),
            g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
        ),
    );
    let decision = g::seq(
        g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
        g::seq(
            g::send::<STUDY, DIRECTOR, RegulatoryEvidence<DIRECTOR>>(),
            g::seq(
                g::send::<DIRECTOR, STUDY_SERVICE, DecisionRequest>(),
                g::seq(
                    g::send::<STUDY, DIRECTOR, DecisionEvidence<DIRECTOR>>(),
                    g::seq(
                        g::send::<STUDY_SERVICE, DIRECTOR, Specimen<STUDY>>(),
                        g::seq(
                            g::send::<DIRECTOR, STUDY_SERVICE, Final<STUDY_SERVICE>>(),
                            g::send::<STUDY_SERVICE, HPLC_SERVICE, DecisionEvidence<HPLC_SERVICE>>(
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(inputs, g::seq(global::review().roll(), decision))
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(crate::clock::Clock))]
#[test]
async fn study_repeats_actual_reviews_holds_both_tools_and_binds_the_director_effect()
-> ShellResult<()> {
    #[cfg(miri)]
    use crate::clock::deadline;
    #[cfg(not(miri))]
    use hibana_runtime::host::deadline;
    let cases = [
        (vec![10], true, false, false),
        (vec![11], true, false, false),
        (vec![10, 11], true, false, false),
        (vec![11, 10], true, false, false),
        (vec![11, 10, 11], false, false, false),
        (vec![10, 11, 10], false, false, false),
        (vec![11, 11, 11, 11], true, false, false),
        (vec![11], true, true, false),
        (vec![11], false, false, true),
    ];
    for (plan, supported, changed_clinical, early_continue) in cases {
        let mut identity = result(3, 3, true, json!({"qc":1}));
        identity["data"]["compound"] = json!("compound-3");
        let formulation = result(4, 4, true, json!({"compound":3}));
        let mut clinical = result(9, 9, true, json!({"formulation":4}));
        clinical["data"]["assessment"] = json!(if supported {
            "supported"
        } else {
            "insufficient_data"
        });
        let source = source();
        let sp = project::<STUDY_SERVICE>(&source);
        let rp = project::<REGULATOR>(&source);
        let dp = project::<DIRECTOR>(&source);
        let fp = project::<FORM_SERVICE>(&source);
        let hp = project::<HPLC_SERVICE>(&source);
        let mp = project::<MASS_SERVICE>(&source);
        let tp = project::<STUDY>(&source);
        let htp = project::<HPLC>(&source);
        let mtp = project::<MASS>(&source);
        let observations = Cell::new(Vec::new());
        let observer = |event: TapEvent| {
            if event.causal_key() >> 8 != u16::from(STUDY_SERVICE)
                || !matches!(event.id(), tap::ENDPOINT_SEND | tap::ENDPOINT_RECV)
            {
                return;
            }
            let mut rows = observations.take();
            rows.push(event);
            observations.set(rows);
        };
        let effects = Reviews {
            clinical: clinical.clone(),
            rejections: plan.len() - 1,
            journal: Cell::new(Vec::new()),
            source: tap::projected(&sp)
                .filter(|row| row.id() == tap::PROJECTED_SOURCE)
                .collect(),
            observations: &observations,
        };
        let sid = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["REGULATORY_SPECIALIST", "PROJECT_DIRECTOR"],
            [(sid, STUDY), (sid, HPLC), (sid, MASS)],
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
        owner.observe(&observer);
        let mut service = owner.enter(sid, &sp)?;
        let mut regulator = client.enter(sid, &rp)?;
        let mut study = client.enter(sid, &tp)?;
        let mut hplc_owner = client.enter(sid, &hp)?;
        let hplc = client.enter(sid, &htp)?;
        let mut mass_owner = client.enter(sid, &mp)?;
        let mass = client.enter(sid, &mtp)?;
        let mut director = client.enter(sid, &dp)?;
        let mut form = client.enter(sid, &fp)?;
        let actual = hibana_runtime::race(
            async {
                async {
                    super::study::run(&mut service, &effects).await?;
                };
                async {
                    let mut instrument = hplc;
                    for step in &plan {
                        if *step == 10 {
                            hplc_owner.recv::<RoutineReview>().await?;
                        } else {
                            hplc_owner.recv::<DeepIntent<HPLC>>().await?;
                            hplc_owner.send::<Instrument<HPLC>>(instrument).await?;
                            instrument = hplc_owner.recv::<Instrument<HPLC>>().await?;
                        }
                    }
                    let actual =
                        imp::decode(hplc_owner.recv::<DecisionEvidence<HPLC_SERVICE>>().await?)?;
                    imp::reply(if supported { 12 } else { 14 }, &actual)?;
                    assert_eq!(actual["data"]["event_id"], 20 + plan.len());
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&identity)?;
                    mass_owner
                        .send::<Identity<STUDY_SERVICE>>(&imp::JsonTo::<STUDY_SERVICE>::from_bytes(
                            &bytes,
                        )?)
                        .await?;
                    let mut sent = clinical.clone();
                    if changed_clinical {
                        sent["data"]["inputs"]["formulation"] = json!(40);
                    }
                    let bytes = imp::encode(&sent)?;
                    mass_owner
                        .send::<ClinicalEvidence<STUDY_SERVICE>>(
                            &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    let mut instrument = mass;
                    for step in &plan {
                        if *step == 10 {
                            mass_owner.recv::<RoutineReview>().await?;
                        } else {
                            mass_owner.recv::<DeepIntent<MASS>>().await?;
                            mass_owner.send::<Instrument<MASS>>(instrument).await?;
                            instrument = mass_owner.recv::<Instrument<MASS>>().await?;
                        }
                    }
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&formulation)?;
                    form.send::<Formulation<STUDY_SERVICE>>(
                        &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                    Ok(())
                };
                async {
                    for step in &plan {
                        if *step == 10 {
                            let bytes = imp::encode(&json!({"review_type":"initial_review"}))?;
                            let input = imp::AnalysisArguments::<10>::from_bytes(&bytes)?;
                            study.send::<RoutineReview>(&input).await?;
                            study.send::<RoutineReview>(&input).await?;
                            study.send::<RoutineReview>(&input).await?;
                            regulator.send::<Specimen<STUDY>>(study).await?;
                            let branch = regulator.offer().await?;
                            match branch.label() {
                                label if label == Passed::<10>::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<Passed<10>>().await?)?;
                                }
                                label if label == Failed::<10>::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<Failed<10>>().await?)?;
                                }
                                _ => panic!("wrong routine verdict"),
                            }
                            study = regulator.recv::<Specimen<STUDY>>().await?;
                        } else {
                            let bytes = imp::encode(&json!({"review_type":"hold_review"}))?;
                            let input = imp::AnalysisArguments::<11>::from_bytes(&bytes)?;
                            study
                                .send::<DeepIntent<HPLC>>(&imp::JsonTo::<HPLC>::from_bytes(&bytes)?)
                                .await?;
                            let mut hplc = regulator.recv::<Instrument<HPLC>>().await?;
                            study
                                .send::<DeepIntent<MASS>>(&imp::JsonTo::<MASS>::from_bytes(&bytes)?)
                                .await?;
                            let mut mass = regulator.recv::<Instrument<MASS>>().await?;
                            study.send::<DeepReview>(&input).await?;
                            async {
                                async { hplc.send::<Request<11>>(&input).await };
                                async { mass.send::<Request<11>>(&input).await };
                            }
                            .await?;
                            async {
                                let [mut h, mut m] = regulator.par()?;
                                async { h.send::<Instrument<HPLC>>(hplc).await };
                                async { m.send::<Instrument<MASS>>(mass).await };
                            }
                            .await?;
                            regulator.send::<Specimen<STUDY>>(study).await?;
                            let branch = regulator.offer().await?;
                            match branch.label() {
                                label if label == Passed::<11>::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<Passed<11>>().await?)?;
                                }
                                label if label == Failed::<11>::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<Failed<11>>().await?)?;
                                }
                                _ => panic!("wrong deep verdict"),
                            }
                            study = regulator.recv::<Specimen<STUDY>>().await?;
                            let (hplc, mass) = async {
                                let [mut h, mut m] = regulator.par()?;
                                async { h.recv::<Instrument<HPLC>>().await };
                                async { m.recv::<Instrument<MASS>>().await };
                            }
                            .await?;
                            regulator.send::<Instrument<MASS>>(mass).await?;
                            regulator.send::<Instrument<HPLC>>(hplc).await?;
                        }
                    }
                    regulator.send::<Specimen<STUDY>>(study).await?;
                    Ok(())
                };
                async {
                    let regulatory =
                        imp::decode(director.recv::<RegulatoryEvidence<DIRECTOR>>().await?)?;
                    let outcome = if supported || early_continue {
                        "continue"
                    } else {
                        "abort"
                    };
                    let bytes = imp::encode(&json!({"outcome":outcome}))?;
                    director.send::<DecisionRequest>(&bytes.as_slice()).await?;
                    let final_outcome =
                        imp::decode(director.recv::<DecisionEvidence<DIRECTOR>>().await?)?;
                    imp::same(&final_outcome["compound"], &identity)?;
                    imp::same(&final_outcome["formulation"], &formulation)?;
                    imp::same(&final_outcome["clinical"], &clinical)?;
                    imp::same(&final_outcome["regulatory"], &regulatory)?;
                    assert_eq!(final_outcome["outcome"], outcome);
                    let _study = director.recv::<Specimen<STUDY>>().await?;
                    let bytes = imp::encode(&final_outcome)?;
                    director
                        .send::<Final<STUDY_SERVICE>>(
                            &imp::FinalOutcome::<STUDY_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    Ok(())
                };
                Ok(())
            },
            deadline(Duration::from_secs(if cfg!(miri) { 600 } else { 5 })),
        )
        .await;
        let journal = effects.journal.take();
        if changed_clinical {
            assert_eq!(
                actual.unwrap_err().to_string(),
                "clinical assessment belongs to another formulation"
            );
            assert!(journal.is_empty());
        } else if early_continue {
            assert_eq!(
                actual.unwrap_err().to_string(),
                "continuation requires actual positive approval"
            );
            assert_eq!(journal.len(), plan.len());
        } else {
            actual?;
            assert_eq!(journal.len(), plan.len() + 1);
        }
    }
    Ok(())
}
