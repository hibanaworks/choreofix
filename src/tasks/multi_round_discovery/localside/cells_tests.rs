//! Cell-lab component qualification. The scripted scientific responses do not
//! claim to execute the common world, the other effect owners, Wasm or models.
use super::super::{global, global::*, imp};
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
        tap::{self, TapEvent},
    },
};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

struct Measurements<'a> {
    script: Vec<Value>,
    journal: Cell<Vec<Value>>,
    source: Vec<TapEvent>,
    observations: &'a Cell<Vec<TapEvent>>,
}
impl Measurements<'_> {
    fn assert_custody(&self, assay: &str) {
        let events = self.observations.take();
        let resources: &[u8] = match assay {
            "culture_prep" => &[CELL, CULTURE],
            "bioassay" => &[CELL, SAMPLES, BIOASSAY],
            "cytotoxicity" => &[CELL, SAMPLES, TOXICITY],
            _ => panic!("unexpected assay"),
        };
        for resource in resources {
            let mut balance = 0i32;
            let mut received = 0;
            for event in &events {
                let row = self.source.get(usize::from(event.arg1() as u16)).unwrap();
                assert_eq!(row.ts(), u32::from(event.arg1() as u16));
                if row.causal_key() >> 8 == 1 && row.arg1() == u32::from(*resource) {
                    match event.id() {
                        tap::ENDPOINT_RECV if (row.arg0() >> 16) as u8 == CELL_SERVICE => {
                            received += 1;
                            balance += 1
                        }
                        tap::ENDPOINT_SEND if (row.arg0() >> 24) as u8 == CELL_SERVICE => {
                            balance -= 1
                        }
                        _ => {}
                    }
                }
            }
            // CELL and SAMPLES start here. A specimen must actually arrive.
            let expected = if matches!(*resource, CELL | SAMPLES) {
                0
            } else {
                1
            };
            assert_eq!(
                balance, expected,
                "effect does not retain resource {resource}"
            );
            assert!(received > 0, "effect never received resource {resource}");
        }
        if assay == "cytotoxicity" {
            assert!(
                events.iter().any(|event| {
                    let row = &self.source[usize::from(event.arg1() as u16)];
                    event.id() == tap::ENDPOINT_RECV
                        && row.arg0() as u8 == PurityEvidence::<CELL_SERVICE>::LOGICAL_LABEL
                        && (row.arg0() >> 24) as u8 == TOXICOLOGIST
                }),
                "toxicity must receive the toxicologist's actual purity evidence first"
            );
        }
        self.observations.set(events);
    }
}
impl Domain for Measurements<'_> {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        input: Value,
    ) -> Result<Value, String> {
        let mut journal = self.journal.take();
        let actual = self
            .script
            .get(journal.len())
            .ok_or("unexpected extra cell effect")?
            .clone();
        if actual["data"]["scientist"] != actor
            || actual["tool_name"] != tool
            || input != json!({"assay_type":actual["data"]["assay_type"]})
        {
            return Err("cell effect differs from the supplied response fixture".into());
        }
        self.assert_custody(input["assay_type"].as_str().unwrap());
        journal.push(actual.clone());
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
        self.assert_custody(input["assay_type"].as_str().unwrap());
        Ok(actual)
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
fn terminal(formulation: &Value, bio: &Value, tox: &Value) -> Value {
    let mut compound = result(3, 3, true, json!({"qc":0}));
    compound["data"]["compound"] = json!("compound-3");
    let supported = bio["success"] == true && tox["success"] == true;
    let mut clinical = result(
        9,
        100,
        true,
        json!({"formulation":4,
        "bioassay":bio["data"]["event_id"],"toxicity":tox["data"]["event_id"]}),
    );
    clinical["data"]["assessment"] = json!(if supported {
        "supported"
    } else {
        "insufficient_data"
    });
    let review = result(10, 101, supported, json!({"clinical":100}));
    let step = if supported { 12 } else { 14 };
    json!({"outcome":if supported{"continue"}else{"abort"},"compound":compound,
        "formulation":formulation,"clinical":clinical,"regulatory":review,
        "decision":result(step,102,true,json!({"compound":3,"formulation":4,"clinical":100,"regulatory":101}))})
}
fn source() -> impl Projectable {
    let preparation = g::seq(
        global::positive::<BIOLOGIST, CELL_SERVICE, CELL, CULTURE, 1, CLINICIAN>(),
        g::seq(
            g::send::<CELL_SERVICE, HPLC_SERVICE, Cells<HPLC_SERVICE>>(),
            g::seq(
                g::send::<MASS_SERVICE, CELL_SERVICE, Baseline<CELL_SERVICE>>(),
                g::send::<FORM_SERVICE, CELL_SERVICE, Formulation<CELL_SERVICE>>(),
            ),
        ),
    );
    let assays = g::seq(
        global::assay::<BIOLOGIST, BIOASSAY, 5>(),
        g::seq(
            g::send::<CELL_SERVICE, MASS_SERVICE, BioEvidence<MASS_SERVICE>>(),
            g::seq(
                g::send::<TOXICOLOGIST, CELL_SERVICE, PurityEvidence<CELL_SERVICE>>(),
                g::seq(
                    global::assay::<TOXICOLOGIST, TOXICITY, 8>(),
                    g::send::<CELL_SERVICE, MASS_SERVICE, ToxEvidence<MASS_SERVICE>>(),
                ),
            ),
        ),
    );
    g::seq(
        preparation,
        g::seq(
            g::seq(
                g::par(
                    global::material::<BIOLOGIST, CELL_SERVICE, BIOASSAY>(),
                    global::material::<TOXICOLOGIST, CELL_SERVICE, TOXICITY>(),
                ),
                assays,
            ),
            g::seq(
                g::send::<MASS_SERVICE, DIRECTOR, DecisionEvidence<DIRECTOR>>(),
                g::send::<DIRECTOR, CELL_SERVICE, Final<CELL_SERVICE>>(),
            ),
        ),
    )
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(crate::clock::Clock))]
#[test]
async fn cell_owner_retries_the_same_material_retains_both_resources_and_retires_the_specimen()
-> ShellResult<()> {
    #[cfg(miri)]
    use crate::clock::deadline;
    #[cfg(not(miri))]
    use hibana_runtime::host::deadline;
    // Explicit fixture responses: null consumes no sample after the three
    // actual units have been used. This does not implement the world physics.
    let cases = [
        (
            vec![(true, Some(0))],
            vec![(true, Some(1))],
            0,
            false,
            false,
        ),
        (
            vec![(false, Some(0)), (true, Some(1))],
            vec![(true, Some(2))],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0))],
            vec![(false, Some(1)), (true, Some(2))],
            0,
            false,
            false,
        ),
        (
            vec![(false, Some(0)), (false, Some(1)), (true, Some(2))],
            vec![(false, None)],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0))],
            vec![(false, Some(1)), (false, Some(2)), (false, None)],
            0,
            false,
            false,
        ),
        (
            vec![(false, Some(0)), (false, Some(1)), (false, Some(2))],
            vec![(false, None)],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0)), (true, Some(1))],
            vec![(true, Some(2))],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0)), (false, Some(1))],
            vec![(true, Some(2))],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0)), (false, Some(1)), (true, Some(2))],
            vec![(false, None)],
            0,
            false,
            false,
        ),
        (
            vec![(true, Some(0))],
            vec![(true, Some(1))],
            2,
            false,
            false,
        ),
        (vec![(true, Some(0))], vec![(true, Some(1))], 0, true, false),
        (vec![(true, Some(0))], vec![(true, Some(1))], 0, false, true),
    ];
    for (case, (bio_plan, tox_plan, culture_failures, changed_purity, retired_attack)) in
        cases.into_iter().enumerate()
    {
        println!("14H cell receiver condition {case}");
        let cells_id = 10 + culture_failures;
        let formulation = result(4, 4, true, json!({"compound":3}));
        let baseline = result(2, 2, true, json!({}));
        let mut purity = result(7, 7, true, json!({"formulation":4}));
        if changed_purity {
            purity["data"]["inputs"]["formulation"] = json!(40);
        }
        let mut script = (0..=culture_failures)
            .map(|i| result(1, 10 + i, i == culture_failures, json!({})))
            .collect::<Vec<_>>();
        let bio_results = bio_plan
            .iter()
            .enumerate()
            .map(|(i, (ok, unit))| {
                let mut actual = result(
                    5,
                    20 + i as u64,
                    *ok,
                    json!({"formulation":4,"cells":cells_id}),
                );
                actual["data"]["sample_unit"] = json!(unit);
                if unit.is_none() {
                    actual["data"]["status"] = json!("exhausted");
                }
                actual
            })
            .collect::<Vec<_>>();
        let tox_results = tox_plan
            .iter()
            .enumerate()
            .map(|(i, (ok, unit))| {
                let mut actual = result(
                    8,
                    40 + i as u64,
                    *ok,
                    json!({"formulation":4,"cells":cells_id,"baseline":2,"purity":7}),
                );
                actual["data"]["sample_unit"] = json!(unit);
                if unit.is_none() {
                    actual["data"]["status"] = json!("exhausted");
                }
                actual
            })
            .collect::<Vec<_>>();
        script.extend(bio_results.iter().cloned());
        script.extend(tox_results.iter().cloned());
        let source = source();
        assert_eq!(g::diagnose(&source), None);
        let bp = project::<BIOLOGIST>(&source);
        let tp = project::<TOXICOLOGIST>(&source);
        let cp = project::<CELL_SERVICE>(&source);
        let hp = project::<HPLC_SERVICE>(&source);
        let mp = project::<MASS_SERVICE>(&source);
        let fp = project::<FORM_SERVICE>(&source);
        let dp = project::<DIRECTOR>(&source);
        let lp = project::<CLINICIAN>(&source);
        let clp = project::<CELL>(&source);
        let sp = project::<SAMPLES>(&source);
        let cup = project::<CULTURE>(&source);
        let bap = project::<BIOASSAY>(&source);
        let txp = project::<TOXICITY>(&source);
        let observations = Cell::new(Vec::new());
        let observer = |event: TapEvent| {
            if event.causal_key() >> 8 == u16::from(CELL_SERVICE)
                && matches!(event.id(), tap::ENDPOINT_SEND | tap::ENDPOINT_RECV)
            {
                let mut events = observations.take();
                events.push(event);
                observations.set(events);
            }
        };
        let effects = Measurements {
            script,
            journal: Cell::new(Vec::new()),
            source: tap::projected(&cp)
                .filter(|row| row.id() == tap::PROJECTED_SOURCE)
                .collect(),
            observations: &observations,
        };
        let sid = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["BIOLOGIST", "TOXICOLOGIST"],
            [
                (sid, CELL),
                (sid, SAMPLES),
                (sid, CULTURE),
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
        let mut service = owner.enter(sid, &cp)?;
        let cell = owner.enter(sid, &clp)?;
        let samples = owner.enter(sid, &sp)?;
        let mut biologist = client.enter(sid, &bp)?;
        let mut toxicologist = client.enter(sid, &tp)?;
        let mut culture = client.enter(sid, &cup)?;
        let mut hplc = owner.enter(sid, &hp)?;
        let mut mass = owner.enter(sid, &mp)?;
        let mut form = owner.enter(sid, &fp)?;
        let mut director = owner.enter(sid, &dp)?;
        let bio = owner.enter(sid, &bap)?;
        let tox = owner.enter(sid, &txp)?;
        let mut clinician = owner.enter(sid, &lp)?;
        let actual = hibana_runtime::race(
            async {
                async {
                    super::cells::run(&mut service, (cell, samples), &effects).await?;
                };
                async {
                    let bytes = imp::encode(&json!({"assay_type":"culture_prep"}))?;
                    for attempt in 0..=culture_failures {
                        let input = imp::AnalysisArguments::<1>::from_bytes(&bytes)?;
                        culture.send::<Intent<1>>(&input).await?;
                        let mut instrument = biologist.recv::<Instrument<CELL>>().await?;
                        instrument.send::<Request<1>>(&input).await?;
                        biologist.send::<Instrument<CELL>>(instrument).await?;
                        biologist.send::<Specimen<CULTURE>>(culture).await?;
                        let branch = biologist.offer().await?;
                        if attempt == culture_failures {
                            imp::decode(branch.recv::<Passed<1>>().await?)?;
                            break;
                        }
                        imp::decode(branch.recv::<Failed<1>>().await?)?;
                        culture = biologist.recv::<Specimen<CULTURE>>().await?;
                    }
                    let mut bio = biologist.recv::<Specimen<BIOASSAY>>().await?;
                    let bytes = imp::encode(&json!({"assay_type":"bioassay"}))?;
                    for expected in &bio_results {
                        let input = imp::AnalysisArguments::<5>::from_bytes(&bytes)?;
                        bio.send::<Intent<5>>(&input).await?;
                        let (mut cell, mut stock) = async {
                            let [mut c, mut s] = biologist.par()?;
                            async { c.recv::<Instrument<CELL>>().await };
                            async { s.recv::<Instrument<SAMPLES>>().await };
                        }
                        .await?;
                        async {
                            async { cell.send::<Request<5>>(&input).await };
                            async { stock.send::<SampleRequest<5>>(&input).await };
                        }
                        .await?;
                        async {
                            let [mut s, mut c] = biologist.par()?;
                            async { s.send::<Instrument<SAMPLES>>(stock).await };
                            async { c.send::<Instrument<CELL>>(cell).await };
                        }
                        .await?;
                        biologist.send::<Specimen<BIOASSAY>>(bio).await?;
                        let branch = biologist.offer().await?;
                        let actual = if branch.label() == Passed::<5>::LOGICAL_LABEL {
                            imp::decode(branch.recv::<Passed<5>>().await?)?
                        } else {
                            imp::decode(branch.recv::<Failed<5>>().await?)?
                        };
                        imp::same(&actual, expected)?;
                        bio = biologist.recv::<Specimen<BIOASSAY>>().await?;
                    }
                    biologist.send::<Specimen<BIOASSAY>>(bio).await?;
                    Ok(())
                };
                async {
                    let mut tox = toxicologist.recv::<Specimen<TOXICITY>>().await?;
                    let purity_bytes = imp::encode(&purity)?;
                    toxicologist
                        .send::<PurityEvidence<CELL_SERVICE>>(
                            &imp::JsonTo::<CELL_SERVICE>::from_bytes(&purity_bytes)?,
                        )
                        .await?;
                    let bytes = imp::encode(&json!({"assay_type":"cytotoxicity"}))?;
                    for expected in &tox_results {
                        let input = imp::AnalysisArguments::<8>::from_bytes(&bytes)?;
                        tox.send::<Intent<8>>(&input).await?;
                        let (mut cell, mut stock) = async {
                            let [mut c, mut s] = toxicologist.par()?;
                            async { c.recv::<Instrument<CELL>>().await };
                            async { s.recv::<Instrument<SAMPLES>>().await };
                        }
                        .await?;
                        async {
                            async { cell.send::<Request<8>>(&input).await };
                            async { stock.send::<SampleRequest<8>>(&input).await };
                        }
                        .await?;
                        async {
                            let [mut s, mut c] = toxicologist.par()?;
                            async { s.send::<Instrument<SAMPLES>>(stock).await };
                            async { c.send::<Instrument<CELL>>(cell).await };
                        }
                        .await?;
                        toxicologist.send::<Specimen<TOXICITY>>(tox).await?;
                        let branch = toxicologist.offer().await?;
                        let actual = if branch.label() == Passed::<8>::LOGICAL_LABEL {
                            imp::decode(branch.recv::<Passed<8>>().await?)?
                        } else {
                            imp::decode(branch.recv::<Failed<8>>().await?)?
                        };
                        imp::same(&actual, expected)?;
                        tox = toxicologist.recv::<Specimen<TOXICITY>>().await?;
                    }
                    toxicologist.send::<Specimen<TOXICITY>>(tox).await?;
                    Ok(())
                };
                async {
                    let actual = imp::decode(hplc.recv::<Cells<HPLC_SERVICE>>().await?)?;
                    assert_eq!(actual["data"]["event_id"], cells_id);
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&baseline)?;
                    mass.send::<Baseline<CELL_SERVICE>>(&imp::JsonTo::<CELL_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
                    let bio = imp::decode(mass.recv::<BioEvidence<MASS_SERVICE>>().await?)?;
                    let tox = imp::decode(mass.recv::<ToxEvidence<MASS_SERVICE>>().await?)?;
                    imp::same(&bio, bio_results.last().unwrap())?;
                    imp::same(&tox, tox_results.last().unwrap())?;
                    let bytes = imp::encode(&terminal(&formulation, &bio, &tox))?;
                    mass.send::<DecisionEvidence<DIRECTOR>>(&imp::JsonTo::<DIRECTOR>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
                    Ok(())
                };
                async {
                    let bytes = imp::encode(&formulation)?;
                    form.send::<Formulation<CELL_SERVICE>>(
                        &imp::JsonTo::<CELL_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                    Ok(())
                };
                async {
                    let _culture = clinician.recv::<Specimen<CULTURE>>().await?;
                    let mut bio = clinician.recv::<Specimen<BIOASSAY>>().await?;
                    if retired_attack {
                        drop(bus.transport.take_observations());
                        let bytes = imp::encode(&json!({"assay_type":"bioassay"}))?;
                        let input = imp::AnalysisArguments::<5>::from_bytes(&bytes)?;
                        let attempt = hibana_runtime::race(
                            async {
                                Ok::<_, ShellError>(Some(bio.send::<Intent<5>>(&input).await))
                            },
                            async { Ok(None) },
                        )
                        .await?;
                        assert!(
                            matches!(attempt, Some(Err(_))),
                            "retired assay must reject on the first poll"
                        );
                        assert!(
                            bus.transport
                                .take_observations()
                                .iter()
                                .all(|row| row["action"] != "send")
                        );
                        return Err(ShellError::message("retired assay attack rejected"));
                    }
                    let _tox = clinician.recv::<Specimen<TOXICITY>>().await?;
                    Ok(())
                };
                async {
                    async {
                        let [mut b, mut t] = director.par()?;
                        async { b.send::<Specimen<BIOASSAY>>(bio).await };
                        async { t.send::<Specimen<TOXICITY>>(tox).await };
                    }
                    .await?;
                    let actual = imp::decode(director.recv::<DecisionEvidence<DIRECTOR>>().await?)?;
                    let bytes = imp::encode(&actual)?;
                    director
                        .send::<Final<CELL_SERVICE>>(
                            &imp::FinalOutcome::<CELL_SERVICE>::from_bytes(&bytes)?,
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
        if actual.is_err() && !changed_purity && !retired_attack {
            eprintln!("cell condition {case}: {actual:?}; effects={journal:?}");
            for row in bus.transport.take_observations() {
                eprintln!("cell transport: {row}");
            }
            for row in observations.take() {
                eprintln!("cell commit: {row:?}");
            }
        }
        if changed_purity {
            assert_eq!(
                actual.unwrap_err().to_string(),
                "toxicity requires the actual baseline and purity"
            );
            assert_eq!(
                journal.len(),
                1 + culture_failures as usize + bio_results.len()
            );
        } else if retired_attack {
            assert_eq!(
                actual.unwrap_err().to_string(),
                "retired assay attack rejected"
            );
        } else {
            actual?;
            assert_eq!(journal, effects.script);
        }
    }
    Ok(())
}
