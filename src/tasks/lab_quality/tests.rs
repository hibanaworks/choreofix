use super::global;
#[cfg(miri)]
#[path = "../handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// This journal records actual fixture effects; it never authorizes a step.
struct Effects(std::cell::Cell<Vec<serde_json::Value>>);
impl crate::shell::imp::service::Domain for Effects {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut result = serde_json::json!({"tool_name":tool,"success":true,
            "data":{"scientist":actor,"arguments":arguments}});
        if actor == "CHEMIST" && tool == "run_separation" {
            result["data"]["compound"] = serde_json::json!("actual_compound");
        }
        if tool == "run_synthesis" {
            result["data"]["compound"] = result["data"]["arguments"]["compound"].clone();
        }
        if tool == "verify_result" {
            result["data"]["scientist"] = result["data"]["arguments"]["scientist"].clone();
            result["data"]["data_type"] = result["data"]["arguments"]["data_type"].clone();
        }
        let mut effects = self.0.take();
        effects.push(result.clone());
        self.0.set(effects);
        Ok(result)
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn centrifuge_compares_the_received_request_before_its_actual_effect()
-> crate::shell::ShellResult<()> {
    use crate::shell::{ShellError, imp::bus::Bus};
    use global::*;
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    use serde_json::{Value, json};
    use std::{cell::Cell, time::Duration};

    let p = programs();
    for (admitted, forwarded, expected_effects) in [
        (
            json!({"material":"unknown"}),
            json!({"material":"cell_cultures"}),
            0,
        ),
        (
            json!({"material":"cell_cultures"}),
            json!({"material":"compound_purification"}),
            0,
        ),
        (
            json!({"material":"cell_cultures"}),
            json!({"material":"cell_cultures"}),
            1,
        ),
    ] {
        let session = SessionId::new(SESSION + 200 + expected_effects);
        let (bus, _, _) = Bus::open([], [(session, CENTRIFUGE), (session, BIOLOGY)]);
        let actual = Cell::new(Value::Null);
        let effects = Effects(Cell::new(Vec::<Value>::new()));
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut scientist = rv.enter(session, &p.biologist)?;
        let [mut task, _private] = scientist.par()?;
        let mut owner = rv.enter(session, &p.cent_service)?;
        let instrument = rv.enter(session, &p.centrifuge)?;
        let candidate = rv.enter(session, &p.biology)?;
        let result = hibana_runtime::race(
            async {
                hibana_runtime::race(
                    super::localside::centrifuge::run(
                        &mut owner,
                        (instrument, candidate, &actual),
                        &effects,
                    ),
                    async {
                        let bytes = super::imp::encode(&admitted)?;
                        task.send::<Preparation>(&bytes.as_slice()).await?;
                        let mut instrument = task.recv::<CentLoan>().await?;
                        let bytes = super::imp::encode(&forwarded)?;
                        instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                        let result =
                            super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                        assert_eq!(
                            result,
                            json!({"tool_name":"run_separation","success":true,
                            "data":{"scientist":"BIOLOGIST","arguments":forwarded}})
                        );
                        task.send::<CentLoan>(instrument).await?;
                        Ok(())
                    },
                )
                .await
            },
            deadline(Duration::from_secs(10)),
        )
        .await;
        let effects = effects.0.take();
        assert_eq!(effects.len(), expected_effects as usize, "{result:?}");
        if expected_effects == 0 {
            assert!(result.as_ref().is_err_and(|error| matches!(error, ShellError::Message(message)
                if message.contains("assigned specimen") || message.contains("admitted actual data"))), "{result:?}");
        } else {
            result?;
            assert_eq!(effects[0]["data"]["arguments"], admitted);
        }
    }
    Ok(())
}

#[test]
fn biological_arguments_preserve_caller_data_and_bind_the_actual_compound() {
    use serde_json::json;
    let compound = json!("caller_selected_compound_".repeat(20));
    for arguments in [
        json!({"compound":compound}),
        json!({"compound":compound,"cell_type":"arbitrary culture".repeat(20)}),
    ] {
        super::imp::assay_arguments(&arguments, &compound).unwrap();
    }
    for arguments in [
        json!({}),
        json!({"compound":"another compound"}),
        json!({"compound":compound,"cell_type":3}),
        json!({"compound":compound,"extra":"not an argument"}),
    ] {
        assert!(super::imp::assay_arguments(&arguments, &compound).is_err());
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn synthesis_receiver_compares_the_actual_compound_before_effect()
-> crate::shell::ShellResult<()> {
    use crate::shell::{ShellError, imp::bus::Bus};
    use global::*;
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    use serde_json::{Value, json};
    use std::{cell::Cell, time::Duration};

    let p = programs();
    let compound = "caller_selected_compound_".repeat(20);
    for (admitted, forwarded, expected_effects) in [
        (json!({"compound":2}), json!({"compound":compound}), 0),
        (
            json!({"compound":compound}),
            json!({"compound":"substituted"}),
            0,
        ),
        (
            json!({"compound":compound}),
            json!({"compound":compound}),
            1,
        ),
    ] {
        let session = SessionId::new(SESSION + 300);
        let (bus, _, _) = Bus::open([], [(session, FUME_HOOD), (session, CHEMISTRY)]);
        let actual = Cell::new(Value::Null);
        let effects = Effects(Cell::new(Vec::<Value>::new()));
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut scientist = rv.enter(session, &p.chemist)?;
        let [mut task, _private] = scientist.par()?;
        let mut owner = rv.enter(session, &p.fume_service)?;
        let instrument = rv.enter(session, &p.fume_hood)?;
        let chemistry = rv.enter(session, &p.chemistry)?;
        let result = hibana_runtime::race(
            async {
                hibana_runtime::race(
                    super::localside::fume::run(
                        &mut owner,
                        (instrument, chemistry),
                        (&effects, &actual),
                    ),
                    async {
                        let mut chemistry = task.recv::<ChemAccess>().await?;
                        let bytes = super::imp::encode(&admitted)?;
                        chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
                        let mut instrument = chemistry.recv::<FumeLoan>().await?;
                        let bytes = super::imp::encode(&forwarded)?;
                        instrument
                            .send::<SynthesisRequest>(&bytes.as_slice())
                            .await?;
                        let result =
                            super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                        assert_eq!(result["data"]["compound"], forwarded["compound"]);
                        chemistry.send::<FumeLoan>(instrument).await?;
                        Ok(())
                    },
                )
                .await
            },
            deadline(Duration::from_secs(10)),
        )
        .await;
        let effects = effects.0.take();
        assert_eq!(effects.len(), expected_effects, "{result:?}");
        if expected_effects == 0 {
            assert!(result.as_ref().is_err_and(|error| matches!(error, ShellError::Message(message)
                if message.contains("synthesis compound") || message.contains("admitted actual data"))), "{result:?}");
        } else {
            result?;
            assert_eq!(effects[0]["data"]["arguments"], admitted);
        }
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn qc_effect_requires_both_received_requests_and_the_actual_measurement()
-> crate::shell::ShellResult<()> {
    use crate::shell::{ShellError, imp::bus::Bus};
    use global::*;
    use hibana::runtime::{
        SessionKitStorage,
        ids::SessionId,
        resolver::{DecisionArm, ResolverError, ResolverRef},
    };
    use serde_json::{Value, json};
    use std::{cell::Cell, time::Duration};

    let p = programs();
    for changed in ["admission", "spectrometer", "fume", "none"] {
        let session = SessionId::new(SESSION + 350);
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, SPECTROMETER),
                (session, CENTRIFUGE),
                (session, FUME_HOOD),
                (session, CHEMISTRY),
                (session, BIOLOGY),
            ],
        );
        let selected = json!({"scientist":"CHEMIST","data_type":"purification"});
        let selection = Cell::new(selected.clone());
        let actual_qc = Cell::new(Value::Null);
        let actual_bio = Cell::new(Value::Null);
        let effects = Effects(Cell::new(Vec::<Value>::new()));
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p.qc,
            ResolverRef::<SELECT_QC>::decision_state(&selection, |input: &Cell<Value>| {
                let value = input.take();
                let decision = match value["scientist"].as_str() {
                    Some("CHEMIST") => Ok(DecisionArm::Left),
                    Some("BIOLOGIST") => Ok(DecisionArm::Right),
                    _ => Err(ResolverError::reject()),
                };
                input.set(value);
                decision
            }),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        rv.set_resolver(
            &p.cent_service,
            ResolverRef::<BIO_SUCCESS>::decision_state(&actual_bio, super::imp::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        rv.set_resolver(
            &p.fume_service,
            ResolverRef::<QC_SUCCESS>::decision_state(&actual_qc, super::imp::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        rv.set_resolver(
            &p.fume_service,
            ResolverRef::<QC_SUBJECT>::decision_state(&actual_qc, |input: &Cell<Value>| {
                let value = input.take();
                let decision = match value["data"]["scientist"].as_str() {
                    Some("CHEMIST") => Ok(DecisionArm::Left),
                    Some("BIOLOGIST") => Ok(DecisionArm::Right),
                    _ => Err(ResolverError::reject()),
                };
                input.set(value);
                decision
            }),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let mut chemist = rv.enter(session, &p.chemist)?;
        let [mut c, _cp] = chemist.par()?;
        let mut biologist = rv.enter(session, &p.biologist)?;
        let [mut b, _bp] = biologist.par()?;
        let mut qc = rv.enter(session, &p.qc)?;
        let [mut q, _qp] = qc.par()?;
        let mut spec = rv.enter(session, &p.spec_service)?;
        let spectrometer = rv.enter(session, &p.spectrometer)?;
        let mut cent = rv.enter(session, &p.cent_service)?;
        let centrifuge = rv.enter(session, &p.centrifuge)?;
        let biology = rv.enter(session, &p.biology)?;
        let mut fume = rv.enter(session, &p.fume_service)?;
        let fume_hood = rv.enter(session, &p.fume_hood)?;
        let chemistry = rv.enter(session, &p.chemistry)?;
        let result = hibana_runtime::race(
            async {
                hibana_runtime::race(
                    async {
                        async { super::localside::spectrometer::run(&mut spec, spectrometer, &effects).await
                            .map_err(|error| ShellError::message(format!("spectrometer: {error}")))?; };
                        async { super::localside::centrifuge::run(&mut cent, (centrifuge, biology, &actual_bio), &effects).await
                            .map_err(|error| ShellError::message(format!("centrifuge: {error}")))?; };
                        async { super::localside::fume::run(&mut fume, (fume_hood, chemistry), (&effects, &actual_qc)).await
                            .map_err(|error| ShellError::message(format!("fume: {error}")))?; };
                        async {
                            let mut chemistry = c.recv::<ChemAccess>().await?;
                            let bytes = super::imp::encode(&json!({"compound":"actual_compound"}))?;
                            chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
                            let mut instrument = chemistry.recv::<FumeLoan>().await?;
                            instrument.send::<SynthesisRequest>(&bytes.as_slice()).await?;
                            let synthesis = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            chemistry.send::<FumeLoan>(instrument).await?;
                            let bytes = super::imp::encode(&json!({"sample":"synthesized_compound"}))?;
                            chemistry.send::<InitialAnalysis>(&bytes.as_slice()).await?;
                            let mut instrument = chemistry.recv::<SpecLoan>().await?;
                            instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                            let analysis = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            chemistry.send::<SpecLoan>(instrument).await?;
                            let bytes = super::imp::encode(&json!({"material":"compound_purification"}))?;
                            chemistry.send::<Purification>(&bytes.as_slice()).await?;
                            let mut instrument = chemistry.recv::<CentLoan>().await?;
                            instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                            let purification = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            chemistry.send::<CentLoan>(instrument).await?;
                            let sample = json!({"synthesis":synthesis,"analysis":analysis,"purification":purification});
                            let bytes = super::imp::encode(&sample)?;
                            c.send::<Sample>(&bytes.as_slice()).await?;
                            c.send::<ChemAccess>(chemistry).await?;
                            c.recv::<FirstChemist>().await?;
                            c.send::<ChemistReport>(&bytes.as_slice()).await?;
                            core::future::pending::<crate::shell::ShellResult<()>>().await
                        };
                        async {
                            let bytes = super::imp::encode(&json!({"material":"cell_cultures"}))?;
                            b.send::<Preparation>(&bytes.as_slice()).await?;
                            let mut instrument = b.recv::<CentLoan>().await?;
                            instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                            let preparation = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            b.send::<CentLoan>(instrument).await?;
                            let bytes = super::imp::encode(&json!({"sample":"fluorescence"}))?;
                            b.send::<Calibration>(&bytes.as_slice()).await?;
                            let mut instrument = b.recv::<SpecLoan>().await?;
                            instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                            let calibration = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            b.send::<SpecLoan>(instrument).await?;
                            let sample = super::imp::decode(b.recv::<Sample>().await?)?;
                            let mut biology = b.recv::<BioAccess>().await?;
                            b.recv::<FirstChemist>().await?;
                            let bytes = super::imp::encode(&json!({"arguments":{"compound":"actual_compound"},
                                "purification":sample["purification"],"preparation":preparation,"calibration":calibration}))?;
                            biology.send::<BioRequest>(&bytes.as_slice()).await?;
                            b.send::<BioAccess>(biology).await?;
                            b.recv::<BioPassed>().await?;
                            core::future::pending::<crate::shell::ShellResult<()>>().await
                        };
                        Ok(())
                    },
                    async {
                        let mut chemistry = q.recv::<ChemAccess>().await?;
                        let bytes = super::imp::encode(&selected)?;
                        q.send::<FirstChemist>(&bytes.as_slice()).await?;
                        q.send::<FirstChemist>(&bytes.as_slice()).await?;
                        q.send::<FirstChemist>(&bytes.as_slice()).await?;
                        q.send::<FirstChemist>(&bytes.as_slice()).await?;
                        let (_biology, _chemistry) = async {
                            let [mut trials, mut verification] = q.par()?;
                            async { Ok(trials.recv::<BioAccess>().await?) };
                            async {
                        let sample = super::imp::decode(verification.recv::<ChemistReport>().await?)?;
                        let mut admitted = json!({"arguments":selected,"sample":sample});
                        if changed == "admission" {
                            admitted["sample"]["analysis"] = json!({"forged":"measurement"});
                        }
                        let bytes = super::imp::encode(&admitted)?;
                        chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut spec, mut fume] = verification.par()?;
                            async { Ok(spec.recv::<SpecLoan>().await?) };
                            async { Ok(fume.recv::<FumeLoan>().await?) };
                        }.await?;
                        let mut spec_request = admitted.clone();
                        let mut fume_request = admitted;
                        if changed == "spectrometer" { spec_request["sample"]["analysis"] = json!({"forged":"spec"}); }
                        if changed == "fume" { fume_request["sample"]["analysis"] = json!({"forged":"fume"}); }
                        let ((spec, spec_result), (fume, fume_result)) = async {
                            async {
                                let bytes = super::imp::encode(&spec_request)?;
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                let result = super::imp::decode(spec.recv::<QcSpecResult>().await?)?;
                                Ok((spec, result))
                            };
                            async {
                                let bytes = super::imp::encode(&fume_request)?;
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                let result = super::imp::decode(fume.recv::<QcFumeResult>().await?)?;
                                Ok((fume, result))
                            };
                        }.await?;
                        assert_eq!(spec_result, fume_result);
                        assert_eq!(spec_result["tool_name"], "verify_result");
                        assert_eq!(spec_result["data"]["arguments"], selected);
                        async {
                            let [mut s, mut f] = verification.par()?;
                            async { s.send::<SpecLoan>(spec).await?; };
                            async { f.send::<FumeLoan>(fume).await?; };
                            Ok(())
                        }.await?;
                        Ok(chemistry)
                            };
                        }.await?;
                        Ok(())
                    },
                ).await
            },
            // This is an interpreter-time guard, not a changed native progress
            // contract. Keep all four receiver inputs and the same assertions.
            deadline(Duration::from_secs(if cfg!(miri) { 300 } else { 30 })),
        ).await;
        let effects = effects.0.take();
        let qc_effects: Vec<_> = effects
            .iter()
            .filter(|effect| effect["tool_name"] == "verify_result")
            .collect();
        if changed == "none" {
            result?;
            assert_eq!(qc_effects.len(), 1);
            assert_eq!(qc_effects[0]["data"]["arguments"], selected);
        } else {
            assert!(
                result
                    .as_ref()
                    .is_err_and(|error| matches!(error, ShellError::Message(message)
                if message.contains("admitted actual data"))),
                "{changed}: {result:?}"
            );
            assert_eq!(qc_effects.len(), 0, "{changed}: {effects:?}");
        }
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_biological_receiver_rejects_substituted_inputs_before_the_trial()
-> crate::shell::ShellResult<()> {
    use crate::shell::{ShellError, imp::bus::Bus};
    use global::*;
    use hibana::{
        g,
        runtime::{SessionKitStorage, ids::SessionId, program::project, resolver::ResolverRef},
    };
    use serde_json::{Value, json};
    use std::{cell::Cell, time::Duration};

    // Reuse the production separation and biological-retry fragments. The
    // fixture supplies a calibration receipt through its actual sender; it
    // does not claim to execute the entire synthesis/calibration application.
    let source = g::seq(
        g::send::<FUME_SERVICE, CHEMIST, ChemAccess>(),
        g::seq(
            global::separation::<BIOLOGIST, 13>(),
            g::seq(
                global::separation::<CHEMISTRY, 12>(),
                g::seq(
                    g::send::<CENT_SERVICE, FUME_SERVICE, PurificationProof>(),
                    g::seq(
                        g::send::<SPEC_SERVICE, CENT_SERVICE, CalibrationProof>(),
                        g::seq(
                            g::send::<CHEMIST, BIOLOGIST, Sample>(),
                            g::seq(
                                g::send::<CENT_SERVICE, BIOLOGIST, BioAccess>(),
                                global::initial_biology(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let p_c = project::<CHEMIST>(&source);
    let p_b = project::<BIOLOGIST>(&source);
    let p_service = project::<CENT_SERVICE>(&source);
    let p_spec = project::<SPEC_SERVICE>(&source);
    let p_fume = project::<FUME_SERVICE>(&source);
    let p_instrument = project::<CENTRIFUGE>(&source);
    let p_chemistry = project::<CHEMISTRY>(&source);
    let p_biology = project::<BIOLOGY>(&source);
    for (path, replacement) in [
        ("/arguments/compound", json!("another_compound")),
        ("/purification", json!({"forged":"purification"})),
        ("/preparation", json!({"forged":"preparation"})),
        ("/calibration", json!({"forged":"calibration"})),
    ] {
        let session = SessionId::new(SESSION + 250);
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, CENTRIFUGE),
                (session, CHEMISTRY),
                (session, BIOLOGY),
            ],
        );
        let actual = Cell::new(Value::Null);
        let effects = Effects(Cell::new(Vec::<Value>::new()));
        let calibration = json!({"tool_name":"run_analysis","success":true,
            "data":{"scientist":"BIOLOGIST","sample":"fluorescence"}});
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p_service,
            ResolverRef::<BIO_SUCCESS>::decision_state(&actual, super::imp::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
        let mut chemist = rv.enter(session, &p_c)?;
        let mut biologist = rv.enter(session, &p_b)?;
        let mut owner = rv.enter(session, &p_service)?;
        let mut spec = rv.enter(session, &p_spec)?;
        let mut fume = rv.enter(session, &p_fume)?;
        let instrument = rv.enter(session, &p_instrument)?;
        let chemistry = rv.enter(session, &p_chemistry)?;
        let biology = rv.enter(session, &p_biology)?;
        let result: crate::shell::ShellResult<()> = hibana_runtime::race(
            async {
                async {
                    super::localside::centrifuge::run(&mut owner, (instrument, biology, &actual), &effects).await?;
                };
                async {
                    fume.send::<ChemAccess>(chemistry).await?;
                    fume.recv::<PurificationProof>().await?;
                    fume.recv::<BioFumePassed>().await?;
                    Ok(())
                };
                async {
                    let bytes = super::imp::encode(&calibration)?;
                    spec.send::<CalibrationProof>(&bytes.as_slice()).await?;
                    spec.recv::<BioSpecPassed>().await?;
                    Ok(())
                };
                async {
                    let mut chemistry = chemist.recv::<ChemAccess>().await?;
                    let arguments = super::imp::encode(&json!({"material":"compound_purification"}))?;
                    chemistry.send::<Purification>(&arguments.as_slice()).await?;
                    let mut instrument = chemistry.recv::<CentLoan>().await?;
                    instrument.send::<CentRequest>(&arguments.as_slice()).await?;
                    let purification = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                    chemistry.send::<CentLoan>(instrument).await?;
                    let bytes = super::imp::encode(&json!({"purification":purification}))?;
                    chemist.send::<Sample>(&bytes.as_slice()).await?;
                    Ok(())
                };
                async {
                    let arguments = super::imp::encode(&json!({"material":"cell_cultures"}))?;
                    biologist.send::<Preparation>(&arguments.as_slice()).await?;
                    let mut instrument = biologist.recv::<CentLoan>().await?;
                    instrument.send::<CentRequest>(&arguments.as_slice()).await?;
                    let preparation = super::imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                    biologist.send::<CentLoan>(instrument).await?;
                    let sample = super::imp::decode(biologist.recv::<Sample>().await?)?;
                    let mut biology = biologist.recv::<BioAccess>().await?;
                    let mut request = json!({"arguments":{"compound":"actual_compound"},
                        "purification":sample["purification"],"preparation":preparation,"calibration":calibration});
                    *request.pointer_mut(path).unwrap() = replacement;
                    let bytes = super::imp::encode(&request)?;
                    biology.send::<BioRequest>(&bytes.as_slice()).await?;
                    biologist.send::<BioAccess>(biology).await?;
                    biologist.recv::<BioPassed>().await?;
                    Ok(())
                };
                Ok(())
            },
            deadline(Duration::from_secs(20)),
        ).await;
        assert!(result.as_ref().is_err_and(|error| matches!(error, ShellError::Message(message)
            if message.contains("admitted actual data") || message.contains("actual synthesized compound"))), "{path}: {result:?}");
        let effects = effects.0.take();
        assert_eq!(effects.len(), 2, "{path}: {effects:?}");
        assert!(
            effects
                .iter()
                .all(|effect| effect["tool_name"] == "run_separation")
        );
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn trial_retries_move_the_actual_continuation_and_cannot_restart_after_submission()
-> crate::shell::ShellResult<()> {
    use crate::shell::{ShellError, imp::bus::Bus, imp::review_reply::ReviewReply};
    use global::*;
    use hibana::{
        g,
        runtime::{
            SessionKitStorage,
            ids::SessionId,
            program::project,
            resolver::{DecisionArm, ResolverError, ResolverRef},
        },
    };
    use serde_json::{Value, json};
    use std::{cell::Cell, time::Duration};

    // Reuse the production source fragment, including its failed-reply roll
    // and candidate submission. These are scripted Endpoint cases, not AI or
    // a substitute for the complete Wasm task and its effect audit.
    let source = g::seq(
        g::send::<CENT_SERVICE, BIOLOGIST, BioAccess>(),
        g::seq(
            global::initial_biology(),
            g::seq(
                g::send::<QC, CENT_SERVICE, g::Msg<200, ()>>(),
                g::send::<QC, CENT_SERVICE, BioAccess>(),
            ),
        ),
    );
    let p_service = project::<CENT_SERVICE>(&source);
    let p_biologist = project::<BIOLOGIST>(&source);
    let p_candidate = project::<BIOLOGY>(&source);
    let p_qc = project::<QC>(&source);
    let p_fume = project::<FUME_SERVICE>(&source);
    let p_spec = project::<SPEC_SERVICE>(&source);
    for rejected in [1, 2, 4] {
        let session = SessionId::new(SESSION + 100 + rejected);
        let (bus, _, _) = Bus::open([], [(session, BIOLOGY)]);
        let actual = Cell::new(Value::Null);
        let mut service_slab = vec![0; 1024 * 1024];
        let mut service_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let service_rv = service_storage
            .init()
            .rendezvous(&mut service_slab, &bus.transport)?;
        let mut client_slab = vec![0; 1024 * 1024];
        let mut client_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let client_rv = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        service_rv
            .set_resolver(
                &p_service,
                ResolverRef::<BIO_SUCCESS>::decision_state(&actual, |input: &Cell<Value>| {
                    let result = input.take();
                    let decision = match result["success"].as_bool() {
                        Some(true) => Ok(DecisionArm::Left),
                        Some(false) => Ok(DecisionArm::Right),
                        None => Err(ResolverError::reject()),
                    };
                    input.set(result);
                    decision
                }),
            )
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut service = service_rv.enter(session, &p_service)?;
        let candidate = service_rv.enter(session, &p_candidate)?;
        let mut biologist = client_rv.enter(session, &p_biologist)?;
        let mut qc = client_rv.enter(session, &p_qc)?;
        let mut fume = client_rv.enter(session, &p_fume)?;
        let mut spec = client_rv.enter(session, &p_spec)?;
        let ((), (), (), (), submitted) = hibana_runtime::race(async {
            async {
                service.send::<BioAccess>(candidate).await?;
                for trial in 0..=rejected {
                    assert_eq!(service.recv::<BioRequest>().await?, b"actual specimen");
                    let candidate = service.recv::<BioAccess>().await?;
                    let result = json!({"tool_name":"run_bioassay","success":trial == rejected,"data":{"trial":trial}});
                    let bytes = serde_json::to_vec(&result).unwrap();
                    actual.set(result);
                    if trial == rejected {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                        service.send::<BioPassed>(&reply).await?;
                        async {
                            let [mut f, mut s] = service.par()?;
                            async { f.send::<BioFumePassed>(&reply).await?; };
                            async { s.send::<BioSpecPassed>(&reply).await?; };
                            Ok(())
                        }.await?;
                        service.send::<BioAccess>(candidate).await?;
                    } else {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                        service.send::<BioFailed>(&reply).await?;
                        async {
                            let [mut f, mut s] = service.par()?;
                            async { f.send::<BioFumeFailed>(&reply).await?; };
                            async { s.send::<BioSpecFailed>(&reply).await?; };
                            Ok(())
                        }.await?;
                        service.send::<BioAccess>(candidate).await?;
                    }
                }
                Ok(())
            };
            async {
                let mut candidate = biologist.recv::<BioAccess>().await?;
                loop {
                    candidate.send::<BioRequest>(&b"actual specimen".as_slice()).await?;
                    biologist.send::<BioAccess>(candidate).await?;
                    let branch = biologist.offer().await?;
                    match branch.label() {
                        25 => { branch.recv::<BioPassed>().await?; break; }
                        26 => { branch.recv::<BioFailed>().await?; candidate = biologist.recv::<BioAccess>().await?; }
                        label => return Err(ShellError::message(format!("unexpected trial reply {label}"))),
                    }
                }
                Ok(())
            };
            async {
                loop {
                    let branch = fume.offer().await?;
                    match branch.label() {
                        35 => { branch.recv::<BioFumePassed>().await?; break; }
                        36 => { branch.recv::<BioFumeFailed>().await?; }
                        label => return Err(ShellError::message(format!("unexpected fume proof {label}"))),
                    }
                }
                Ok(())
            };
            async {
                loop {
                    let branch = spec.offer().await?;
                    match branch.label() {
                        37 => { branch.recv::<BioSpecPassed>().await?; break; }
                        38 => { branch.recv::<BioSpecFailed>().await?; }
                        label => return Err(ShellError::message(format!("unexpected spectrometer proof {label}"))),
                    }
                }
                Ok(())
            };
            async { Ok(qc.recv::<BioAccess>().await?) };
        }, deadline(Duration::from_secs(30))).await?;
        // The attack follows actual foreign receive commit, not merely the
        // service's transport acceptance of the owned send.
        let offered = match hibana_runtime::race(
            async { service.offer().await.map_err(ShellError::from) },
            deadline(Duration::from_millis(10)),
        )
        .await
        {
            Ok(branch) => Some(branch.label()),
            Err(_) => None,
        };
        let false_result = json!({"tool_name":"run_bioassay","success":false});
        let bytes = serde_json::to_vec(&false_result).unwrap();
        actual.set(false_result);
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        drop(bus.transport.take_observations());
        let attempt: crate::shell::ShellResult<()> = hibana_runtime::race(
            async {
                service.send::<BioFailed>(&reply).await?;
                Ok(())
            },
            deadline(Duration::from_secs(5)),
        )
        .await;
        assert!(
            attempt
                .as_ref()
                .is_err_and(|e| matches!(e, ShellError::Endpoint(_))),
            "preview={offered:?}, following send={attempt:?}"
        );
        assert!(
            offered.is_none(),
            "submitted biology must not expose an old result: {offered:?}; following send={attempt:?}"
        );
        assert!(
            bus.transport
                .take_observations()
                .iter()
                .all(|event| event["action"] != "send" || event["source"] != CENT_SERVICE)
        );
        drop(submitted);
    }
    Ok(())
}

fn qc_payload<const WHO: u8, const APPROVED: bool>(scientist: &str, data_type: &str) {
    use super::imp::QcReply;
    use hibana::runtime::wire::WireEncode;
    use serde_json::json;
    let value = json!({"tool_name":"verify_result","success":APPROVED,
        "data":{"scientist":scientist,"data_type":data_type}});
    let bytes = serde_json::to_vec(&value).unwrap();
    let reply = QcReply::<WHO, APPROVED>::from_bytes(&bytes).unwrap();
    let mut encoded = vec![0; bytes.len()];
    assert_eq!(reply.encode_into(&mut encoded).unwrap(), bytes.len());
    assert_eq!(encoded, bytes);
    for (path, replacement) in [
        ("/success", json!(!APPROVED)),
        ("/tool_name", json!("run_bioassay")),
        (
            "/data/scientist",
            json!(if scientist == "CHEMIST" {
                "BIOLOGIST"
            } else {
                "CHEMIST"
            }),
        ),
        (
            "/data/data_type",
            json!(if data_type == "purification" {
                "bioassay"
            } else {
                "purification"
            }),
        ),
    ] {
        let mut changed = value.clone();
        *changed.pointer_mut(path).unwrap() = replacement;
        let changed = serde_json::to_vec(&changed).unwrap();
        assert!(QcReply::<WHO, APPROVED>::from_bytes(&changed).is_err());
    }
}

#[test]
fn qc_payloads_bind_subject_tool_and_verdict() {
    qc_payload::<{ global::CHEMIST }, true>("CHEMIST", "purification");
    qc_payload::<{ global::CHEMIST }, false>("CHEMIST", "purification");
    qc_payload::<{ global::BIOLOGIST }, true>("BIOLOGIST", "bioassay");
    qc_payload::<{ global::BIOLOGIST }, false>("BIOLOGIST", "bioassay");
}

#[test]
fn full_qc_source_projects_and_epf_checks_every_actor_snapshot() {
    let programs = global::programs();
    use hibana::runtime::tap;
    use hibana_epf::Protocol;
    let snapshots = [
        tap::projected(&programs.chemist).collect::<Vec<_>>(),
        tap::projected(&programs.biologist).collect(),
        tap::projected(&programs.analyst).collect(),
        tap::projected(&programs.qc).collect(),
        tap::projected(&programs.chemist_io).collect(),
        tap::projected(&programs.biologist_io).collect(),
        tap::projected(&programs.analyst_io).collect(),
        tap::projected(&programs.qc_io).collect(),
        tap::projected(&programs.spec_service).collect(),
        tap::projected(&programs.cent_service).collect(),
        tap::projected(&programs.fume_service).collect(),
        tap::projected(&programs.spectrometer).collect(),
        tap::projected(&programs.centrifuge).collect(),
        tap::projected(&programs.fume_hood).collect(),
        tap::projected(&programs.chemistry).collect(),
        tap::projected(&programs.biology).collect(),
    ];
    // This exercises actual projection/publication of the entire source. It
    // does not establish Endpoint execution or complete application effects.
    for records in snapshots {
        let mut storage = vec![0; Protocol::workspace_len(&records).unwrap()];
        let _protocol = Protocol::new(&records, &mut storage).unwrap();
    }
}
