//! Two production receivers share the real review and terminal choreography.
//! Other scientific data is supplied; this is not a common-world/Wasm run.
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

struct Effects<'a> {
    script: Vec<(u8, Value)>,
    journal: Cell<Vec<Value>>,
    source: Vec<TapEvent>,
    observations: &'a Cell<Vec<TapEvent>>,
}
impl Effects<'_> {
    fn assert_custody(&self, actor: u8, resources: &[u8]) {
        let records = self.observations.take();
        for resource in resources {
            let (mut received, mut sent) = (0, 0);
            for event in &records {
                if event.causal_key() >> 8 != u16::from(actor) {
                    continue;
                }
                let row = &self.source[usize::from(event.arg1() as u16)];
                if row.causal_key() >> 8 != 1 || row.arg1() != u32::from(*resource) {
                    continue;
                }
                match event.id() {
                    tap::ENDPOINT_RECV if (row.arg0() >> 16) as u8 == actor => received += 1,
                    tap::ENDPOINT_SEND if (row.arg0() >> 24) as u8 == actor => sent += 1,
                    _ => {}
                }
            }
            let initial = usize::from(actor == HPLC_SERVICE && *resource == HPLC);
            assert!(received > 0, "actual resource {resource} never arrived");
            assert_eq!(
                received + initial,
                sent + 1,
                "effect does not retain {resource}"
            );
        }
        self.observations.set(records);
    }
}
impl Domain for Effects<'_> {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        input: Value,
    ) -> Result<Value, String> {
        let mut journal = self.journal.take();
        let (step, actual) = self
            .script
            .get(journal.len())
            .ok_or("unexpected extra effect")?;
        let (expected_actor, expected_tool, key, _) =
            imp::analysis(*step).map_err(|e| e.to_string())?;
        if actor != expected_actor
            || tool != expected_tool
            || input != json!({key:actual["data"][key]})
        {
            return Err("effect differs from the supplied response script".into());
        }
        let (owner, resources): (_, &[u8]) = match step {
            0 => (HPLC_SERVICE, &[HPLC, QC]),
            7 => (HPLC_SERVICE, &[HPLC, PURITY]),
            6 => (HPLC_SERVICE, &[HPLC, STABILITY]),
            15 => (
                HPLC_SERVICE,
                &[
                    HPLC,
                    STUDY,
                    FORMULATION,
                    BIOASSAY,
                    STABILITY,
                    PURITY,
                    TOXICITY,
                ],
            ),
            11 => (STUDY_SERVICE, &[STUDY, HPLC, MASS]),
            10 | 12 | 14 => (STUDY_SERVICE, &[STUDY]),
            _ => return Err("unexpected effect kind".into()),
        };
        self.assert_custody(owner, resources);
        journal.push(json!({"step":step,"input":input,"result":actual}));
        self.journal.set(journal);
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
        self.assert_custody(owner, resources);
        Ok(actual.clone())
    }
}
fn result(step: u8, event: u64, success: bool, inputs: Value) -> Value {
    let (actor, tool, key, value) = imp::analysis(step).unwrap();
    json!({"tool_name":tool,"success":success,"data":{"scientist":actor,
        key:value.unwrap_or("compound-10"),"event_id":event,
        "artifact_id":if success{Some(event)}else{None},
        "status":if success{"complete"}else if matches!(step,10|11){"clinical_hold"}else{"injected_failure"},
        "inputs":inputs}})
}
fn source() -> impl Projectable {
    let preparation = g::seq(
        global::positive::<CHEMIST, HPLC_SERVICE, HPLC, QC, 0, FORMULATOR>(),
        g::seq(
            g::send::<HPLC_SERVICE, MASS_SERVICE, QcEvidence>(),
            g::seq(
                g::send::<CELL_SERVICE, HPLC_SERVICE, Cells<HPLC_SERVICE>>(),
                g::send::<FORM_SERVICE, HPLC_SERVICE, Formulation<HPLC_SERVICE>>(),
            ),
        ),
    );
    let analyses = g::seq(
        global::positive::<TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 7, CLINICIAN>(),
        g::seq(
            g::par(
                g::send::<TOXICOLOGIST, CELL_SERVICE, PurityEvidence<CELL_SERVICE>>(),
                g::send::<HPLC_SERVICE, MASS_SERVICE, PurityEvidence<MASS_SERVICE>>(),
            ),
            g::seq(
                global::positive::<BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 6, CLINICIAN>(),
                g::send::<HPLC_SERVICE, MASS_SERVICE, StabilityEvidence<MASS_SERVICE>>(),
            ),
        ),
    );
    let clinical = g::seq(
        g::par(
            g::send::<CLINICIAN, DIRECTOR, Specimen<STABILITY>>(),
            g::send::<CLINICIAN, DIRECTOR, Specimen<PURITY>>(),
        ),
        g::seq(
            g::send::<MASS_SERVICE, STUDY_SERVICE, Identity<STUDY_SERVICE>>(),
            g::seq(
                g::send::<FORM_SERVICE, STUDY_SERVICE, Formulation<STUDY_SERVICE>>(),
                g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
            ),
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
                    g::send::<STUDY_SERVICE, DIRECTOR, Specimen<STUDY>>(),
                ),
            ),
        ),
    );
    g::seq(
        preparation,
        g::seq(
            g::seq(
                g::par(
                    global::material::<BIOLOGIST, HPLC_SERVICE, STABILITY>(),
                    global::material::<TOXICOLOGIST, HPLC_SERVICE, PURITY>(),
                ),
                analyses,
            ),
            g::seq(
                clinical,
                g::seq(
                    global::review().roll(),
                    g::seq(decision, global::terminal()),
                ),
            ),
        ),
    )
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(crate::clock::Clock))]
#[test]
async fn chromatograph_and_study_retain_real_custody_through_review_and_documentation()
-> ShellResult<()> {
    #[cfg(miri)]
    use crate::clock::deadline;
    #[cfg(not(miri))]
    use hibana_runtime::host::deadline;
    let cases = [
        (0, 0, 0, 0, vec![10], "continue", 0),
        (0, 0, 0, 0, vec![11], "abort", 0),
        (1, 1, 1, 1, vec![10, 11], "continue", 0),
        (2, 0, 0, 2, vec![11, 10, 11], "abort", 0),
        (0, 2, 0, 0, vec![11, 11, 11, 11], "continue", 0),
        (0, 0, 2, 0, vec![10, 11], "continue", 1),
        (0, 0, 0, 0, vec![10, 11, 10, 11], "abort", 0),
        (0, 0, 0, 1, vec![10], "continue", 2),
    ];
    for (
        case,
        (qc_failures, purity_failures, stability_failures, doc_failures, plan, outcome, corrupt),
    ) in cases.into_iter().enumerate()
    {
        println!("14H HPLC/study condition {case}");
        let qc = result(0, 1 + qc_failures, true, json!({}));
        let mut identity = result(3, 10, true, json!({"qc":1+qc_failures}));
        identity["data"]["compound"] = json!("compound-10");
        let formulation = result(4, 20, true, json!({"compound":10}));
        let cells = result(1, 0, true, json!({}));
        let purity = result(7, 30 + purity_failures, true, json!({"formulation":20}));
        let stability = result(
            6,
            40 + stability_failures,
            true,
            json!({"formulation":20,"cells":0}),
        );
        let mut clinical = result(
            9,
            50,
            true,
            json!({"formulation":20,"bioassay":25,
            "purity":30+purity_failures,"stability":40+stability_failures,"toxicity":45}),
        );
        clinical["data"]["assessment"] = json!("supported");
        let review_step = *plan.last().unwrap();
        let regulatory = result(
            review_step,
            60 + plan.len() as u64 - 1,
            true,
            json!({"clinical":50}),
        );
        let decision_step = if outcome == "continue" { 12 } else { 14 };
        let decision = result(
            decision_step,
            70,
            true,
            json!({"compound":10,"formulation":20,"clinical":50,
            "regulatory":regulatory["data"]["event_id"]}),
        );
        let final_result = json!({"outcome":outcome,"compound":identity,"formulation":formulation,
            "clinical":clinical,"regulatory":regulatory,"decision":decision});
        let mut script = Vec::new();
        for (step, failures, start, inputs) in [
            (0, qc_failures, 1, json!({})),
            (7, purity_failures, 30, json!({"formulation":20})),
            (
                6,
                stability_failures,
                40,
                json!({"formulation":20,"cells":0}),
            ),
        ] {
            for i in 0..=failures {
                script.push((step, result(step, start + i, i == failures, inputs.clone())));
            }
        }
        for (i, step) in plan.iter().enumerate() {
            script.push((
                *step,
                result(
                    *step,
                    60 + i as u64,
                    i + 1 == plan.len(),
                    json!({"clinical":50}),
                ),
            ));
        }
        script.push((decision_step, decision.clone()));
        for i in 0..=doc_failures {
            script.push((15,result(15,80+i,i==doc_failures,json!({"compound":10,
            "formulation":20,"clinical":50,"regulatory":regulatory["data"]["event_id"],"decision":70}))));
        }
        let source = source();
        assert_eq!(g::diagnose(&source), None);
        let hp = project::<HPLC_SERVICE>(&source);
        let sp = project::<STUDY_SERVICE>(&source);
        let cp = project::<CHEMIST>(&source);
        let bp = project::<BIOLOGIST>(&source);
        let tp = project::<TOXICOLOGIST>(&source);
        let rp = project::<REGULATOR>(&source);
        let dp = project::<DIRECTOR>(&source);
        let lp = project::<CLINICIAN>(&source);
        let mp = project::<MASS_SERVICE>(&source);
        let csp = project::<CELL_SERVICE>(&source);
        let fp = project::<FORM_SERVICE>(&source);
        let fop = project::<FORMULATOR>(&source);
        let htp = project::<HPLC>(&source);
        let mtp = project::<MASS>(&source);
        let qp = project::<QC>(&source);
        let stp = project::<STUDY>(&source);
        let pp = project::<PURITY>(&source);
        let sap = project::<STABILITY>(&source);
        let fmp = project::<FORMULATION>(&source);
        let bap = project::<BIOASSAY>(&source);
        let txp = project::<TOXICITY>(&source);
        let observations = Cell::new(Vec::new());
        let observer = |event: TapEvent| {
            if matches!(event.causal_key() >> 8, 14 | 18)
                && matches!(event.id(), tap::ENDPOINT_SEND | tap::ENDPOINT_RECV)
            {
                let mut events = observations.take();
                events.push(event);
                observations.set(events);
            }
        };
        let effects = Effects {
            script,
            journal: Cell::new(Vec::new()),
            source: tap::projected(&hp)
                .filter(|row| row.id() == tap::PROJECTED_SOURCE)
                .collect(),
            observations: &observations,
        };
        let sid = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            [
                "CHEMIST",
                "BIOLOGIST",
                "TOXICOLOGIST",
                "REGULATORY_SPECIALIST",
                "PROJECT_DIRECTOR",
            ],
            [
                (sid, HPLC),
                (sid, MASS),
                (sid, QC),
                (sid, STUDY),
                (sid, PURITY),
                (sid, STABILITY),
                (sid, FORMULATION),
                (sid, BIOASSAY),
                (sid, TOXICITY),
            ],
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
        let mut hplc = owner.enter(sid, &hp)?;
        let instrument = owner.enter(sid, &htp)?;
        let mut study_service = owner.enter(sid, &sp)?;
        let mut chemist = client.enter(sid, &cp)?;
        let mut biologist = client.enter(sid, &bp)?;
        let mut toxicologist = client.enter(sid, &tp)?;
        let mut regulator = client.enter(sid, &rp)?;
        let mut director = client.enter(sid, &dp)?;
        let mut clinician = client.enter(sid, &lp)?;
        let mut mass = owner.enter(sid, &mp)?;
        let mass_instrument = owner.enter(sid, &mtp)?;
        let mut cell_service = owner.enter(sid, &csp)?;
        let mut form_service = owner.enter(sid, &fp)?;
        let mut formulator = client.enter(sid, &fop)?;
        let mut qc_cap = client.enter(sid, &qp)?;
        let mut study = client.enter(sid, &stp)?;
        let purity_cap = client.enter(sid, &pp)?;
        let stability_cap = client.enter(sid, &sap)?;
        let formulation_cap = client.enter(sid, &fmp)?;
        let bio_cap = client.enter(sid, &bap)?;
        let tox_cap = client.enter(sid, &txp)?;
        let actual = hibana_runtime::race(
            async {
                async {
                    super::hplc::run(&mut hplc, instrument, &effects).await?;
                };
                async {
                    super::study::run(&mut study_service, &effects).await?;
                };
                async {
                    let bytes = imp::encode(&json!({"sample":"synthesis_qc"}))?;
                    for i in 0..=qc_failures {
                        let input = imp::AnalysisArguments::<0>::from_bytes(&bytes)?;
                        qc_cap.send::<Intent<0>>(&input).await?;
                        let mut tool = chemist.recv::<Instrument<HPLC>>().await?;
                        tool.send::<Request<0>>(&input).await?;
                        chemist.send::<Instrument<HPLC>>(tool).await?;
                        chemist.send::<Specimen<QC>>(qc_cap).await?;
                        let branch = chemist.offer().await?;
                        if i == qc_failures {
                            let actual = imp::decode(branch.recv::<Passed<0>>().await?)?;
                            imp::same(&actual, &qc)?;
                            break;
                        }
                        imp::decode(branch.recv::<Failed<0>>().await?)?;
                        qc_cap = chemist.recv::<Specimen<QC>>().await?;
                    }
                    let mut study = chemist.recv::<Specimen<STUDY>>().await?;
                    chemist.recv::<Final<CHEMIST>>().await?;
                    let bytes = imp::encode(&json!({"sample":"documentation"}))?;
                    for i in 0..=doc_failures {
                        let input = imp::AnalysisArguments::<15>::from_bytes(&bytes)?;
                        study.send::<Intent<15>>(&input).await?;
                        let mut tool = chemist.recv::<Instrument<HPLC>>().await?;
                        tool.send::<Request<15>>(&input).await?;
                        chemist.send::<Instrument<HPLC>>(tool).await?;
                        chemist.send::<Specimen<STUDY>>(study).await?;
                        let branch = chemist.offer().await?;
                        if i == doc_failures {
                            imp::decode(branch.recv::<Passed<15>>().await?)?;
                            break;
                        }
                        imp::decode(branch.recv::<Failed<15>>().await?)?;
                        study = chemist.recv::<Specimen<STUDY>>().await?;
                    }
                    Ok(())
                };
                async {
                    let mut purity_cap = toxicologist.recv::<Specimen<PURITY>>().await?;
                    let bytes = imp::encode(&json!({"sample":"purity_check"}))?;
                    for i in 0..=purity_failures {
                        let input = imp::AnalysisArguments::<7>::from_bytes(&bytes)?;
                        purity_cap.send::<Intent<7>>(&input).await?;
                        let mut tool = toxicologist.recv::<Instrument<HPLC>>().await?;
                        tool.send::<Request<7>>(&input).await?;
                        toxicologist.send::<Instrument<HPLC>>(tool).await?;
                        toxicologist.send::<Specimen<PURITY>>(purity_cap).await?;
                        let branch = toxicologist.offer().await?;
                        if i == purity_failures {
                            let actual = imp::decode(branch.recv::<Passed<7>>().await?)?;
                            imp::same(&actual, &purity)?;
                            break;
                        }
                        imp::decode(branch.recv::<Failed<7>>().await?)?;
                        purity_cap = toxicologist.recv::<Specimen<PURITY>>().await?;
                    }
                    let purity_bytes = imp::encode(&purity)?;
                    toxicologist
                        .send::<PurityEvidence<CELL_SERVICE>>(
                            &imp::JsonTo::<CELL_SERVICE>::from_bytes(&purity_bytes)?,
                        )
                        .await?;
                    toxicologist.recv::<Final<TOXICOLOGIST>>().await?;
                    Ok(())
                };
                async {
                    let mut stability_cap = biologist.recv::<Specimen<STABILITY>>().await?;
                    let bytes = imp::encode(&json!({"sample":"stability"}))?;
                    for i in 0..=stability_failures {
                        let input = imp::AnalysisArguments::<6>::from_bytes(&bytes)?;
                        stability_cap.send::<Intent<6>>(&input).await?;
                        let mut tool = biologist.recv::<Instrument<HPLC>>().await?;
                        tool.send::<Request<6>>(&input).await?;
                        biologist.send::<Instrument<HPLC>>(tool).await?;
                        biologist.send::<Specimen<STABILITY>>(stability_cap).await?;
                        let branch = biologist.offer().await?;
                        if i == stability_failures {
                            let actual = imp::decode(branch.recv::<Passed<6>>().await?)?;
                            imp::same(&actual, &stability)?;
                            break;
                        }
                        imp::decode(branch.recv::<Failed<6>>().await?)?;
                        stability_cap = biologist.recv::<Specimen<STABILITY>>().await?;
                    }
                    biologist.recv::<Final<BIOLOGIST>>().await?;
                    Ok(())
                };
                async {
                    let purity = clinician.recv::<Specimen<PURITY>>().await?;
                    let stability = clinician.recv::<Specimen<STABILITY>>().await?;
                    async {
                        let [mut s, mut p] = clinician.par()?;
                        async { s.send::<Specimen<STABILITY>>(stability).await };
                        async { p.send::<Specimen<PURITY>>(purity).await };
                    }
                    .await?;
                    clinician.recv::<Final<CLINICIAN>>().await?;
                    Ok(())
                };
                async {
                    let actual = imp::decode(mass.recv::<QcEvidence>().await?)?;
                    imp::same(&actual, &qc)?;
                    let actual = imp::decode(mass.recv::<PurityEvidence<MASS_SERVICE>>().await?)?;
                    imp::same(&actual, &purity)?;
                    let actual =
                        imp::decode(mass.recv::<StabilityEvidence<MASS_SERVICE>>().await?)?;
                    imp::same(&actual, &stability)?;
                    let bytes = imp::encode(&identity)?;
                    mass.send::<Identity<STUDY_SERVICE>>(
                        &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                    let bytes = imp::encode(&clinical)?;
                    mass.send::<ClinicalEvidence<STUDY_SERVICE>>(
                        &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                    let mut instrument = mass_instrument;
                    for step in &plan {
                        if *step == 10 {
                            mass.recv::<RoutineReview>().await?;
                        } else {
                            mass.recv::<DeepIntent<MASS>>().await?;
                            mass.send::<Instrument<MASS>>(instrument).await?;
                            instrument = mass.recv::<Instrument<MASS>>().await?;
                        }
                    }
                    mass.recv::<Final<MASS_SERVICE>>().await?;
                    let _study = mass.recv::<Specimen<STUDY>>().await?;
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&cells)?;
                    cell_service
                        .send::<Cells<HPLC_SERVICE>>(&imp::JsonTo::<HPLC_SERVICE>::from_bytes(
                            &bytes,
                        )?)
                        .await?;
                    let actual =
                        imp::decode(cell_service.recv::<PurityEvidence<CELL_SERVICE>>().await?)?;
                    imp::same(&actual, &purity)?;
                    cell_service.recv::<Final<CELL_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&formulation)?;
                    form_service
                        .send::<Formulation<HPLC_SERVICE>>(
                            &imp::JsonTo::<HPLC_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    form_service
                        .send::<Formulation<STUDY_SERVICE>>(
                            &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    form_service.recv::<Final<FORM_SERVICE>>().await?;
                    Ok(())
                };
                async {
                    let _qc = formulator.recv::<Specimen<QC>>().await?;
                    formulator.recv::<Final<FORMULATOR>>().await?;
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
                            if branch.label() == Passed::<10>::LOGICAL_LABEL {
                                imp::decode(branch.recv::<Passed<10>>().await?)?;
                            } else {
                                imp::decode(branch.recv::<Failed<10>>().await?)?;
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
                            if branch.label() == Passed::<11>::LOGICAL_LABEL {
                                imp::decode(branch.recv::<Passed<11>>().await?)?;
                            } else {
                                imp::decode(branch.recv::<Failed<11>>().await?)?;
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
                    regulator.recv::<Final<REGULATOR>>().await?;
                    Ok(())
                };
                async {
                    async {
                        let [mut s, mut p] = director.par()?;
                        async { p.send::<Specimen<PURITY>>(purity_cap).await };
                        async { s.send::<Specimen<STABILITY>>(stability_cap).await };
                    }
                    .await?;
                    let (stability, purity) = async {
                        let [mut s, mut p] = director.par()?;
                        async { s.recv::<Specimen<STABILITY>>().await };
                        async { p.recv::<Specimen<PURITY>>().await };
                    }
                    .await?;
                    let actual =
                        imp::decode(director.recv::<RegulatoryEvidence<DIRECTOR>>().await?)?;
                    imp::same(&actual, &regulatory)?;
                    let bytes = imp::encode(&json!({"outcome":outcome}))?;
                    director.send::<DecisionRequest>(&bytes.as_slice()).await?;
                    let actual = imp::decode(director.recv::<DecisionEvidence<DIRECTOR>>().await?)?;
                    imp::same(&actual, &final_result)?;
                    let study = director.recv::<Specimen<STUDY>>().await?;
                    director.send::<Specimen<STUDY>>(study).await?;
                    async {
                        let [mut f, mut b, mut s, mut p, mut t] = director.par()?;
                        async { f.send::<Archived<FORMULATION>>(formulation_cap).await };
                        async { b.send::<Archived<BIOASSAY>>(bio_cap).await };
                        async { s.send::<Archived<STABILITY>>(stability).await };
                        async { p.send::<Archived<PURITY>>(purity).await };
                        async { t.send::<Archived<TOXICITY>>(tox_cap).await };
                    }
                    .await?;
                    let bytes = imp::encode(&final_result)?;
                    let mut altered = final_result.clone();
                    if corrupt == 1 {
                        altered["clinical"]["data"]["inputs"]["stability"] = json!(400);
                    }
                    if corrupt == 2 {
                        altered["compound"]["data"]["inputs"]["qc"] = json!(100);
                    }
                    let altered_bytes = imp::encode(&altered)?;
                    async {
                        let [
                            mut c,
                            mut b,
                            mut t,
                            mut f,
                            mut l,
                            mut r,
                            mut h,
                            mut m,
                            mut cs,
                            mut fs,
                            mut s,
                        ] = director.par()?;
                        async {
                            c.send::<Final<CHEMIST>>(&imp::FinalOutcome::<CHEMIST>::from_bytes(
                                &bytes,
                            )?)
                            .await
                        };
                        async {
                            b.send::<Final<BIOLOGIST>>(&imp::FinalOutcome::<BIOLOGIST>::from_bytes(
                                &bytes,
                            )?)
                            .await
                        };
                        async {
                            t.send::<Final<TOXICOLOGIST>>(
                                &imp::FinalOutcome::<TOXICOLOGIST>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                        async {
                            f.send::<Final<FORMULATOR>>(
                                &imp::FinalOutcome::<FORMULATOR>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                        async {
                            l.send::<Final<CLINICIAN>>(&imp::FinalOutcome::<CLINICIAN>::from_bytes(
                                &bytes,
                            )?)
                            .await
                        };
                        async {
                            r.send::<Final<REGULATOR>>(&imp::FinalOutcome::<REGULATOR>::from_bytes(
                                &bytes,
                            )?)
                            .await
                        };
                        async {
                            h.send::<Final<HPLC_SERVICE>>(
                                &imp::FinalOutcome::<HPLC_SERVICE>::from_bytes(&altered_bytes)?,
                            )
                            .await
                        };
                        async {
                            m.send::<Final<MASS_SERVICE>>(
                                &imp::FinalOutcome::<MASS_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                        async {
                            cs.send::<Final<CELL_SERVICE>>(
                                &imp::FinalOutcome::<CELL_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                        async {
                            fs.send::<Final<FORM_SERVICE>>(
                                &imp::FinalOutcome::<FORM_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                        async {
                            s.send::<Final<STUDY_SERVICE>>(
                                &imp::FinalOutcome::<STUDY_SERVICE>::from_bytes(&bytes)?,
                            )
                            .await
                        };
                    }
                    .await?;
                    Ok(())
                };
                Ok(())
            },
            deadline(Duration::from_secs(if cfg!(miri) { 1200 } else { 5 })),
        )
        .await;
        let journal = effects.journal.take();
        if corrupt != 0 {
            assert!(actual.is_err(), "changed final evidence must be rejected");
            assert!(
                journal.iter().all(|entry| entry["step"] != 15),
                "changed evidence reached documentation"
            );
        } else {
            if actual.is_err() {
                eprintln!("HPLC condition {case}: {actual:?}; effects={journal:?}");
                for row in bus.transport.take_observations() {
                    eprintln!("HPLC transport: {row}");
                }
            }
            actual?;
            assert_eq!(journal.len(), effects.script.len());
        }
    }
    Ok(())
}
