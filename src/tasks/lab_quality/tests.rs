//! Real Endpoint boundary tests with controlled scientific values and effects.
use super::{arguments::Arguments, global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use core::cell::Cell;
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};

#[test]
fn complete_direct_file_source_has_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&source()), None);
}

#[test]
fn whole_source_projects_all_views_and_the_scientists_file_operations() {
    use hibana::g::Message;
    use hibana::runtime::tap;
    let p = programs();
    let observed = p.observation();
    let images = observed["programs"].as_array().unwrap();
    assert_eq!(images.len(), 29);
    assert_eq!(images.len(), observed["roles"].as_object().unwrap().len());
    let events = images[0]["rows"][0][3].as_u64().unwrap() >> 16;
    assert!(events > 600);
    for image in images {
        assert_eq!(image["rows"][0][3].as_u64().unwrap() >> 16, events);
    }
    assert!(
        tap::projected(&p.chemist).any(|row| row.id() == tap::PROJECTED_SEND
            && row.arg1() >> 24 == u32::from(CHEM_FILE)
            && (row.arg0() & 255) == u32::from(FileAnalysis::LOGICAL_LABEL))
    );
    assert!(
        tap::projected(&p.chemist).any(|row| row.id() == tap::PROJECTED_RECV
            && row.arg1() >> 24 == u32::from(QC)
            && (row.arg0() & 255) == u32::from(ChemistRejected::LOGICAL_LABEL))
    );
    println!("13M direct source: views={} events={events}", images.len());
}

struct Effects {
    synthesis: Value,
    result: Value,
    calls: Cell<Vec<Value>>,
}
impl Domain for Effects {
    async fn execute(
        &self,
        actor: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.calls.take();
        calls.push(json!({"actor":actor,"tool":tool,"arguments":arguments}));
        self.calls.set(calls);
        match tool {
            "run_synthesis" => Ok(self.synthesis.clone()),
            "verify_result" => Ok(self.result.clone()),
            _ => Err("unexpected physical tool in QC receiver fixture".into()),
        }
    }
}
fn measured((actor, tool, arguments): (&str, &str, Value), (id, inputs): (u64, Value)) -> Value {
    let mut data = arguments;
    data["scientist"] = json!(actor);
    data["artifact_id"] = json!(id);
    data["inputs"] = inputs;
    data["status"] = json!("complete");
    data["missing_inputs"] = json!([]);
    json!({"tool_name":tool,"success":true,"data":data})
}
fn material() -> Value {
    let compound = "arbitrary β batch\nwith spaces";
    let mut purification = measured(
        (
            "CHEMIST",
            "run_separation",
            json!({"material":"compound_purification"}),
        ),
        (2, json!({"synthesis":0,"analysis":1})),
    );
    purification["data"]["compound"] = json!(compound);
    json!({
        "synthesis":measured(("CHEMIST","run_synthesis",json!({"compound":compound})),(0,json!({}))),
        "initial_analysis":measured(("CHEMIST","run_analysis",json!({"sample":"synthesized_compound"})),(1,json!({"synthesis":0}))),
        "purification":purification,
        "cells":measured(("BIOLOGIST","run_separation",json!({"material":"cell_cultures"})),(3,json!({}))),
        "calibration":measured(("BIOLOGIST","run_analysis",json!({"sample":"fluorescence"})),(4,json!({"prepared_cells":3}))),
        "bioassay":measured(("BIOLOGIST","run_bioassay",json!({"compound":compound,"cell_type":"free β culture"})),(5,json!({"synthesis":0,"purification":2,"prepared_cells":3,"calibration":4}))),
        "analysis":measured(("CHEMIST","run_analysis",json!({"sample":"synthesized_compound"})),(6,json!({"synthesis":0})))
    })
}
fn arguments(subject: usize) -> Value {
    match subject {
        0 => json!({"scientist":"CHEMIST","data_type":"purification"}),
        1 => json!({"scientist":"BIOLOGIST","data_type":"bioassay"}),
        _ => panic!("unknown fixture subject"),
    }
}
fn verdict(arguments: &Value, inputs: &Value, approved: bool) -> Value {
    let mut data = arguments.clone();
    data["inputs"] = inputs.clone();
    data["artifact_id"] = if approved { json!(7) } else { Value::Null };
    data["status"] = json!(if approved { "complete" } else { "rejected" });
    data["missing_inputs"] = json!([]);
    data["qc_agent"] = json!("QC_CONTROLLER");
    json!({"tool_name":"verify_result","success":approved,"data":data})
}
#[test]
fn scientific_arguments_preserve_free_batch_and_culture_text() {
    for batch in ["", "batch-23", "β batch\n\"quoted\""] {
        let synthesis = json!({"compound":batch});
        let bytes = imp::encode(&synthesis).unwrap();
        assert_eq!(
            Arguments::<0>::try_from(bytes.as_slice())
                .unwrap()
                .into_value(),
            synthesis
        );
        for args in [
            synthesis.clone(),
            json!({"compound":batch,"cell_type":"free β text".repeat(80)}),
        ] {
            let bytes = imp::encode(&args).unwrap();
            assert_eq!(
                Arguments::<5>::try_from(bytes.as_slice())
                    .unwrap()
                    .into_value(),
                args
            );
        }
    }
    for args in [
        json!({"compound":7}),
        json!({"compound":"batch","cell_type":7}),
        json!({"compound":"batch","phase":true}),
    ] {
        let bytes = imp::encode(&args).unwrap();
        assert!(Arguments::<5>::try_from(bytes.as_slice()).is_err());
    }
    for subject in 0..2 {
        let mut args = arguments(subject);
        let bytes = imp::encode(&args).unwrap();
        if subject == 0 {
            assert!(Arguments::<9>::try_from(bytes.as_slice()).is_ok());
        } else {
            assert!(Arguments::<10>::try_from(bytes.as_slice()).is_ok());
        }
        args["data_type"] = json!("other material");
        let bytes = imp::encode(&args).unwrap();
        assert!(Arguments::<9>::try_from(bytes.as_slice()).is_err());
        assert!(Arguments::<10>::try_from(bytes.as_slice()).is_err());
    }
}

#[test]
fn biological_input_refusal_does_not_impersonate_a_physical_trial_failure() {
    let materials = material();
    let actual_compound = materials["synthesis"]["data"]["compound"].clone();
    for wrong_batch in [false, true] {
        let compound = if wrong_batch {
            json!("another batch")
        } else {
            actual_compound.clone()
        };
        let arguments = json!({"compound":compound,"cell_type":"free β culture"});
        assert_eq!(
            imp::bio_arguments(&arguments, &actual_compound).is_err(),
            wrong_batch
        );
        for input_refusal in [false, true] {
            let mut result = materials["bioassay"].clone();
            result["success"] = json!(false);
            result["data"]["compound"] = arguments["compound"].clone();
            result["data"]["artifact_id"] = Value::Null;
            result["data"]["status"] = json!(if input_refusal {
                "invalid_arguments"
            } else {
                "bioassay_failed"
            });
            assert_eq!(
                imp::bio_result(&result, &arguments, &materials).is_ok(),
                wrong_batch == input_refusal
            );
            result["data"]["inputs"]["purification"] = json!(90);
            assert!(imp::bio_result(&result, &arguments, &materials).is_err());
        }
    }
}

#[test]
fn typed_qc_replies_bind_both_their_actual_subject_and_verdict() {
    let materials = material();
    for subject in 0..2 {
        let arguments = arguments(subject);
        let inputs = imp::qc_inputs(&arguments, &materials).unwrap();
        for approved in [false, true] {
            let result = verdict(&arguments, &inputs, approved);
            let bytes = imp::encode(&result).unwrap();
            assert_eq!(
                imp::QcReply::<CHEMIST, true>::from_bytes(&bytes).is_ok(),
                subject == 0 && approved
            );
            assert_eq!(
                imp::QcReply::<CHEMIST, false>::from_bytes(&bytes).is_ok(),
                subject == 0 && !approved
            );
            assert_eq!(
                imp::QcReply::<BIOLOGIST, true>::from_bytes(&bytes).is_ok(),
                subject == 1 && approved
            );
            assert_eq!(
                imp::QcReply::<BIOLOGIST, false>::from_bytes(&bytes).is_ok(),
                subject == 1 && !approved
            );
        }
    }
}
#[hibana_runtime::main]
#[test]
async fn chemistry_qc_prefix_requires_both_requests_and_refuses_substitution() -> ShellResult<()> {
    use crate::shell::imp::review_reply::ReviewReply;
    let subject = 0;
    for case in 0..13 {
        let (approved, fume_first, bad) = if case < 4 {
            (case >= 2, case % 2 == 1, None)
        } else {
            (true, false, Some(case - 4))
        };
        // Run the synthesis and custody prefix of the production owner. Stop
        // after returning both instruments, before submitting the candidates.
        // This is a finite prefix test; whole-Wasm tests cover final retirement.
        let source = g::par(
            g::send::<INIT, QC_SERVICE, FumeLoan>(),
            g::send::<INIT, QC, SpecLoan>(),
        );
        let source = g::seq(source, g::send::<QC_SERVICE, CHEMIST, ChemAccess>());
        let source = g::seq(source, g::send::<CHEMISTRY, QC_SERVICE, Synthesis>());
        let source = g::seq(source, g::send::<QC_SERVICE, CHEMISTRY, FumeLoan>());
        let source = g::seq(source, g::send::<FUME_HOOD, QC_SERVICE, SynthesisRequest>());
        let source = g::seq(source, g::send::<QC_SERVICE, FUME_HOOD, InstrumentResult>());
        let source = g::seq(source, g::send::<CHEMISTRY, QC_SERVICE, FumeLoan>());
        let source = g::seq(source, g::send::<SPEC_SERVICE, QC_SERVICE, AnalysisProof>());
        let source = g::seq(
            source,
            g::send::<CENT_SERVICE, QC_SERVICE, PurificationProof>(),
        );
        let source = g::seq(source, g::send::<CHEMIST, QC, ChemAccess>());
        let resource_roundtrip = || {
            g::seq(
                g::send::<QC_SERVICE, QC, FumeLoan>(),
                g::seq(
                    g::par(
                        g::send::<SPECTROMETER, QC_SERVICE, QcSpecRequest>(),
                        g::send::<FUME_HOOD, QC_SERVICE, QcFumeRequest>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<QC_SERVICE, SPECTROMETER, QcSpecResult>(),
                            g::send::<QC_SERVICE, FUME_HOOD, QcFumeResult>(),
                        ),
                        g::par(
                            g::send::<QC, INIT, SpecLoan>(),
                            g::send::<QC, QC_SERVICE, FumeLoan>(),
                        ),
                    ),
                ),
            )
        };
        let source = g::seq(
            source,
            g::seq(
                g::send::<QC, QC_SERVICE, FirstChemist>(),
                g::par(
                    g::seq(
                        g::send::<CENT_SERVICE, QC_SERVICE, BioFumePassed>(),
                        g::send::<CENT_SERVICE, QC, BioAccess>(),
                    ),
                    g::seq(
                        g::send::<CHEMISTRY, QC_SERVICE, QcAdmission>(),
                        resource_roundtrip(),
                    ),
                ),
            ),
        );
        let source = g::seq(
            source,
            g::par(
                g::send::<QC, QC_SERVICE, ChemAccess>(),
                g::send::<QC, QC_SERVICE, BioAccess>(),
            ),
        );
        let p_init = project::<INIT>(&source);
        let p_qc = project::<QC>(&source);
        let p_chemist = project::<CHEMIST>(&source);
        let p_service = project::<QC_SERVICE>(&source);
        let p_spec_service = project::<SPEC_SERVICE>(&source);
        let p_cent_service = project::<CENT_SERVICE>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_chemistry = project::<CHEMISTRY>(&source);
        let p_biology = project::<BIOLOGY>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, SPECTROMETER),
                (session, FUME_HOOD),
                (session, CHEMISTRY),
                (session, BIOLOGY),
            ],
        );
        let mut client_slab = vec![0; 128 * 1024];
        let mut server_slab = vec![0; 128 * 1024];
        let mut client_storage = SessionKitStorage::uninit();
        let mut server_storage = SessionKitStorage::uninit();
        let client = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let server = server_storage
            .init()
            .rendezvous(&mut server_slab, &bus.transport)?;
        let mut init = client.enter(session, &p_init)?;
        let mut qc = client.enter(session, &p_qc)?;
        let mut chemist = client.enter(session, &p_chemist)?;
        let mut spec_owner = client.enter(session, &p_spec_service)?;
        let mut cent_owner = client.enter(session, &p_cent_service)?;
        let mut service = server.enter(session, &p_service)?;
        let spec = client.enter(session, &p_spec)?;
        let fume = client.enter(session, &p_fume)?;
        let chemistry = server.enter(session, &p_chemistry)?;
        let biology = client.enter(session, &p_biology)?;
        let mut materials = material();
        materials["analysis"] = materials["initial_analysis"].clone();
        let args = arguments(subject);
        let inputs = imp::qc_inputs(&args, &materials)?;
        let actual = verdict(&args, &inputs, approved);
        let effects = Effects {
            synthesis: materials["synthesis"].clone(),
            result: actual.clone(),
            calls: Cell::new(Vec::new()),
        };
        let decision = Cell::new(Value::Null);
        let mut reported = materials.clone();
        if subject == 0 {
            for key in ["cells", "calibration", "bioassay"] {
                reported.as_object_mut().unwrap().remove(key);
            }
        }
        let mut admitted = json!({"arguments":args,"materials":reported});
        match bad {
            Some(4) => admitted["arguments"]["data_type"] = json!("wrong subject data"),
            Some(5) => admitted["materials"]["synthesis"]["data"]["artifact_id"] = json!(90),
            Some(6) => admitted["materials"]["synthesis"]["success"] = json!(false),
            Some(7) => {
                admitted["materials"]["initial_analysis"]["data"]["inputs"] =
                    json!({"synthesis":80})
            }
            Some(8) => {
                admitted["materials"]["purification"]["data"]["compound"] = json!("different batch")
            }
            Some(9) => admitted["materials"]["bioassay"]["success"] = json!(false),
            Some(10) => {
                admitted["materials"]["bioassay"]["data"]["inputs"]["purification"] = json!(80)
            }
            Some(11) => admitted["materials"]["cells"]["data"]["artifact_id"] = Value::Null,
            Some(12) => {
                admitted["materials"]["calibration"]["data"]["inputs"]["prepared_cells"] = json!(80)
            }
            Some(13) => {
                admitted["materials"]["bioassay"]["data"]["compound"] = json!("foreign batch")
            }
            _ => {}
        }
        let mut spec_packet = admitted.clone();
        let mut fume_packet = admitted.clone();
        match bad {
            Some(0) => spec_packet["arguments"]["scientist"] = json!("BIOLOGIST"),
            Some(1) => fume_packet["arguments"]["data_type"] = json!("bioassay"),
            Some(2) => spec_packet["materials"]["analysis"]["data"]["artifact_id"] = json!(90),
            Some(3) => {
                fume_packet["materials"]["synthesis"]["data"]["compound"] = json!("foreign batch")
            }
            _ => {}
        }
        let mut sample = materials.clone();
        for key in ["cells", "calibration", "bioassay"] {
            sample.as_object_mut().unwrap().remove(key);
        }
        let proof = json!({"success":true,"arguments":{
            "compound":materials["synthesis"]["data"]["compound"],"cell_type":"free β culture"},
            "result":materials["bioassay"],"materials":materials});
        let result = hibana_runtime::race(
            hibana_runtime::race(
                localside::qc::run(&mut service, (chemistry, &effects, &decision)),
                async {
                    async {
                        let [mut a, mut b] = init.par()?;
                        async {
                            a.send::<FumeLoan>(fume).await?;
                        };
                        async {
                            b.send::<SpecLoan>(spec).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let mut spec = qc.recv::<SpecLoan>().await?;
                    let mut chemistry = chemist.recv::<ChemAccess>().await?;
                    let synthesis_args =
                        json!({"compound":materials["synthesis"]["data"]["compound"]});
                    let bytes = imp::encode(&synthesis_args)?;
                    chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
                    let mut fume = chemistry.recv::<FumeLoan>().await?;
                    fume.send::<SynthesisRequest>(&bytes.as_slice()).await?;
                    assert_eq!(
                        imp::decode(fume.recv::<InstrumentResult>().await?)?,
                        materials["synthesis"]
                    );
                    chemistry.send::<FumeLoan>(fume).await?;
                    let bytes = imp::encode(&materials["initial_analysis"])?;
                    spec_owner.send::<AnalysisProof>(&bytes.as_slice()).await?;
                    let bytes = imp::encode(&sample)?;
                    cent_owner
                        .send::<PurificationProof>(&bytes.as_slice())
                        .await?;
                    chemist.send::<ChemAccess>(chemistry).await?;
                    let mut chemistry = qc.recv::<ChemAccess>().await?;
                    let bytes = imp::encode(&args)?;
                    if subject == 0 {
                        qc.send::<FirstChemist>(&bytes.as_slice()).await?;
                    } else {
                        qc.send::<FirstBiologist>(&bytes.as_slice()).await?;
                    }
                    let proof_bytes = imp::encode(&proof)?;
                    let proof = ReviewReply::<true>::from_bytes(&proof_bytes)?;
                    let bytes = imp::encode(&admitted)?;
                    let _prefix_receipts = async {
                        async {
                            cent_owner.send::<BioFumePassed>(&proof).await?;
                            cent_owner.send::<BioAccess>(biology).await?;
                        };
                        async {
                            let [mut trials, mut verify] = qc.par()?;
                            async { trials.recv::<BioAccess>().await };
                            async {
                                chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                                let mut fume = verify.recv::<FumeLoan>().await?;
                                let spec_bytes = imp::encode(&spec_packet)?;
                                let fume_bytes = imp::encode(&fume_packet)?;
                                if fume_first {
                                    fume.send::<QcFumeRequest>(&fume_bytes.as_slice()).await?;
                                } else {
                                    spec.send::<QcSpecRequest>(&spec_bytes.as_slice()).await?;
                                }
                                hibana_runtime::host::sleep(std::time::Duration::from_millis(20))
                                    .await?;
                                let calls = effects.calls.take();
                                assert!(
                                    calls.iter().all(|call| call["tool"] != "verify_result"),
                                    "QC invoked without both actual instrument requests"
                                );
                                effects.calls.set(calls);
                                if fume_first {
                                    spec.send::<QcSpecRequest>(&spec_bytes.as_slice()).await?;
                                } else {
                                    fume.send::<QcFumeRequest>(&fume_bytes.as_slice()).await?;
                                }
                                let (spec_result, fume_result) = async {
                                    async { imp::decode(spec.recv::<QcSpecResult>().await?) };
                                    async { imp::decode(fume.recv::<QcFumeResult>().await?) };
                                }
                                .await?;
                                assert_eq!(spec_result, actual);
                                assert_eq!(fume_result, actual);
                                async {
                                    let [mut a, mut b] = verify.par()?;
                                    async {
                                        a.send::<SpecLoan>(spec).await?;
                                    };
                                    async {
                                        b.send::<FumeLoan>(fume).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                                let returned = init.recv::<SpecLoan>().await?;
                                drop(returned);
                            };
                        };
                    }
                    .await?;
                    // Withholding both candidate submissions keeps the actual
                    // owner Pending at the next source operation, without an ack.
                    Ok(())
                },
            ),
            async {
                hibana_runtime::host::sleep(std::time::Duration::from_secs(10)).await?;
                Err(ShellError::message("QC boundary fixture deadline"))
            },
        )
        .await;
        if let Some(bad) = bad {
            let error = result.expect_err("substituted QC request succeeded");
            assert!(
                !error.to_string().contains("EndpointError")
                    && !error.to_string().contains("deadline"),
                "{bad}: {error}"
            );
        } else {
            result?;
        }
        let calls = effects.calls.take();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call["tool"] == "verify_result")
                .count(),
            usize::from(bad.is_none())
        );
        assert_eq!(
            calls[0],
            json!({"actor":"CHEMIST","tool":"run_synthesis","arguments":{"compound":materials["synthesis"]["data"]["compound"]}})
        );
        if bad.is_none() {
            assert_eq!(
                calls[1],
                json!({"actor":"QC_CONTROLLER","tool":"verify_result","arguments":args})
            );
        }
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn biology_qc_prefix_requires_both_requests_and_refuses_substitution() -> ShellResult<()> {
    use crate::shell::imp::review_reply::ReviewReply;
    let subject = 1;
    for case in 0..9 {
        let (approved, fume_first, bad) = if case < 4 {
            (case >= 2, case % 2 == 1, None)
        } else {
            (true, false, Some(case - 4 + 9))
        };
        // Run the synthesis and custody prefix of the production owner. Stop
        // after returning both instruments, before submitting the candidates.
        // This is a finite prefix test; whole-Wasm tests cover final retirement.
        let source = g::par(
            g::send::<INIT, QC_SERVICE, FumeLoan>(),
            g::send::<INIT, QC, SpecLoan>(),
        );
        let source = g::seq(source, g::send::<QC_SERVICE, CHEMIST, ChemAccess>());
        let source = g::seq(source, g::send::<CHEMISTRY, QC_SERVICE, Synthesis>());
        let source = g::seq(source, g::send::<QC_SERVICE, CHEMISTRY, FumeLoan>());
        let source = g::seq(source, g::send::<FUME_HOOD, QC_SERVICE, SynthesisRequest>());
        let source = g::seq(source, g::send::<QC_SERVICE, FUME_HOOD, InstrumentResult>());
        let source = g::seq(source, g::send::<CHEMISTRY, QC_SERVICE, FumeLoan>());
        let source = g::seq(source, g::send::<SPEC_SERVICE, QC_SERVICE, AnalysisProof>());
        let source = g::seq(
            source,
            g::send::<CENT_SERVICE, QC_SERVICE, PurificationProof>(),
        );
        let source = g::seq(source, g::send::<CHEMIST, QC, ChemAccess>());
        let resource_roundtrip = || {
            g::seq(
                g::send::<QC_SERVICE, QC, FumeLoan>(),
                g::seq(
                    g::par(
                        g::send::<SPECTROMETER, QC_SERVICE, QcSpecRequest>(),
                        g::send::<FUME_HOOD, QC_SERVICE, QcFumeRequest>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<QC_SERVICE, SPECTROMETER, QcSpecResult>(),
                            g::send::<QC_SERVICE, FUME_HOOD, QcFumeResult>(),
                        ),
                        g::par(
                            g::send::<QC, INIT, SpecLoan>(),
                            g::send::<QC, QC_SERVICE, FumeLoan>(),
                        ),
                    ),
                ),
            )
        };
        let source = g::seq(
            source,
            g::seq(
                g::send::<QC, QC_SERVICE, FirstBiologist>(),
                g::seq(
                    g::seq(
                        g::send::<CENT_SERVICE, QC_SERVICE, BioFumePassed>(),
                        g::send::<CENT_SERVICE, QC, BioAccess>(),
                    ),
                    g::seq(
                        g::send::<BIOLOGY, QC_SERVICE, QcAdmission>(),
                        resource_roundtrip(),
                    ),
                ),
            ),
        );
        let source = g::seq(
            source,
            g::par(
                g::send::<QC, QC_SERVICE, ChemAccess>(),
                g::send::<QC, QC_SERVICE, BioAccess>(),
            ),
        );
        let p_init = project::<INIT>(&source);
        let p_qc = project::<QC>(&source);
        let p_chemist = project::<CHEMIST>(&source);
        let p_service = project::<QC_SERVICE>(&source);
        let p_spec_service = project::<SPEC_SERVICE>(&source);
        let p_cent_service = project::<CENT_SERVICE>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_chemistry = project::<CHEMISTRY>(&source);
        let p_biology = project::<BIOLOGY>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, SPECTROMETER),
                (session, FUME_HOOD),
                (session, CHEMISTRY),
                (session, BIOLOGY),
            ],
        );
        let mut client_slab = vec![0; 128 * 1024];
        let mut server_slab = vec![0; 128 * 1024];
        let mut client_storage = SessionKitStorage::uninit();
        let mut server_storage = SessionKitStorage::uninit();
        let client = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let server = server_storage
            .init()
            .rendezvous(&mut server_slab, &bus.transport)?;
        let mut init = client.enter(session, &p_init)?;
        let mut qc = client.enter(session, &p_qc)?;
        let mut chemist = client.enter(session, &p_chemist)?;
        let mut spec_owner = client.enter(session, &p_spec_service)?;
        let mut cent_owner = client.enter(session, &p_cent_service)?;
        let mut service = server.enter(session, &p_service)?;
        let spec = client.enter(session, &p_spec)?;
        let fume = client.enter(session, &p_fume)?;
        let chemistry = server.enter(session, &p_chemistry)?;
        let biology = client.enter(session, &p_biology)?;
        let mut materials = material();
        materials["analysis"] = materials["initial_analysis"].clone();
        let args = arguments(subject);
        let inputs = imp::qc_inputs(&args, &materials)?;
        let actual = verdict(&args, &inputs, approved);
        let effects = Effects {
            synthesis: materials["synthesis"].clone(),
            result: actual.clone(),
            calls: Cell::new(Vec::new()),
        };
        let decision = Cell::new(Value::Null);
        let mut reported = materials.clone();
        if subject == 0 {
            for key in ["cells", "calibration", "bioassay"] {
                reported.as_object_mut().unwrap().remove(key);
            }
        }
        let mut admitted = json!({"arguments":args,"materials":reported});
        match bad {
            Some(4) => admitted["arguments"]["data_type"] = json!("wrong subject data"),
            Some(5) => admitted["materials"]["synthesis"]["data"]["artifact_id"] = json!(90),
            Some(6) => admitted["materials"]["synthesis"]["success"] = json!(false),
            Some(7) => {
                admitted["materials"]["initial_analysis"]["data"]["inputs"] =
                    json!({"synthesis":80})
            }
            Some(8) => {
                admitted["materials"]["purification"]["data"]["compound"] = json!("different batch")
            }
            Some(9) => admitted["materials"]["bioassay"]["success"] = json!(false),
            Some(10) => {
                admitted["materials"]["bioassay"]["data"]["inputs"]["purification"] = json!(80)
            }
            Some(11) => admitted["materials"]["cells"]["data"]["artifact_id"] = Value::Null,
            Some(12) => {
                admitted["materials"]["calibration"]["data"]["inputs"]["prepared_cells"] = json!(80)
            }
            Some(13) => {
                admitted["materials"]["bioassay"]["data"]["compound"] = json!("foreign batch")
            }
            _ => {}
        }
        let mut spec_packet = admitted.clone();
        let mut fume_packet = admitted.clone();
        match bad {
            Some(0) => spec_packet["arguments"]["scientist"] = json!("BIOLOGIST"),
            Some(1) => fume_packet["arguments"]["data_type"] = json!("bioassay"),
            Some(2) => spec_packet["materials"]["analysis"]["data"]["artifact_id"] = json!(90),
            Some(3) => {
                fume_packet["materials"]["synthesis"]["data"]["compound"] = json!("foreign batch")
            }
            _ => {}
        }
        let mut sample = materials.clone();
        for key in ["cells", "calibration", "bioassay"] {
            sample.as_object_mut().unwrap().remove(key);
        }
        let proof = json!({"success":true,"arguments":{
            "compound":materials["synthesis"]["data"]["compound"],"cell_type":"free β culture"},
            "result":materials["bioassay"],"materials":materials});
        let result = hibana_runtime::race(
            hibana_runtime::race(
                localside::qc::run(&mut service, (chemistry, &effects, &decision)),
                async {
                    async {
                        let [mut a, mut b] = init.par()?;
                        async {
                            a.send::<FumeLoan>(fume).await?;
                        };
                        async {
                            b.send::<SpecLoan>(spec).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let mut spec = qc.recv::<SpecLoan>().await?;
                    let mut chemistry = chemist.recv::<ChemAccess>().await?;
                    let synthesis_args =
                        json!({"compound":materials["synthesis"]["data"]["compound"]});
                    let bytes = imp::encode(&synthesis_args)?;
                    chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
                    let mut fume = chemistry.recv::<FumeLoan>().await?;
                    fume.send::<SynthesisRequest>(&bytes.as_slice()).await?;
                    assert_eq!(
                        imp::decode(fume.recv::<InstrumentResult>().await?)?,
                        materials["synthesis"]
                    );
                    chemistry.send::<FumeLoan>(fume).await?;
                    let bytes = imp::encode(&materials["initial_analysis"])?;
                    spec_owner.send::<AnalysisProof>(&bytes.as_slice()).await?;
                    let bytes = imp::encode(&sample)?;
                    cent_owner
                        .send::<PurificationProof>(&bytes.as_slice())
                        .await?;
                    chemist.send::<ChemAccess>(chemistry).await?;
                    let mut chemistry = qc.recv::<ChemAccess>().await?;
                    let bytes = imp::encode(&args)?;
                    if subject == 0 {
                        qc.send::<FirstChemist>(&bytes.as_slice()).await?;
                    } else {
                        qc.send::<FirstBiologist>(&bytes.as_slice()).await?;
                    }
                    let proof_bytes = imp::encode(&proof)?;
                    let proof = ReviewReply::<true>::from_bytes(&proof_bytes)?;
                    let bytes = imp::encode(&admitted)?;
                    let _biology = if subject == 0 {
                        async {
                            async {
                                cent_owner.send::<BioFumePassed>(&proof).await?;
                                cent_owner.send::<BioAccess>(biology).await?;
                            };
                            async {
                                chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        qc.recv::<BioAccess>().await?
                    } else {
                        cent_owner.send::<BioFumePassed>(&proof).await?;
                        cent_owner.send::<BioAccess>(biology).await?;
                        let mut biology = qc.recv::<BioAccess>().await?;
                        biology.send::<QcAdmission>(&bytes.as_slice()).await?;
                        biology
                    };
                    let mut fume = qc.recv::<FumeLoan>().await?;
                    let spec_bytes = imp::encode(&spec_packet)?;
                    let fume_bytes = imp::encode(&fume_packet)?;
                    if fume_first {
                        fume.send::<QcFumeRequest>(&fume_bytes.as_slice()).await?;
                    } else {
                        spec.send::<QcSpecRequest>(&spec_bytes.as_slice()).await?;
                    }
                    hibana_runtime::host::sleep(std::time::Duration::from_millis(20)).await?;
                    let calls = effects.calls.take();
                    assert!(
                        calls.iter().all(|call| call["tool"] != "verify_result"),
                        "QC invoked without both actual instrument requests"
                    );
                    effects.calls.set(calls);
                    if fume_first {
                        spec.send::<QcSpecRequest>(&spec_bytes.as_slice()).await?;
                    } else {
                        fume.send::<QcFumeRequest>(&fume_bytes.as_slice()).await?;
                    }
                    let (spec_result, fume_result) = async {
                        async { imp::decode(spec.recv::<QcSpecResult>().await?) };
                        async { imp::decode(fume.recv::<QcFumeResult>().await?) };
                    }
                    .await?;
                    assert_eq!(spec_result, actual);
                    assert_eq!(fume_result, actual);
                    async {
                        let [mut a, mut b] = qc.par()?;
                        async {
                            a.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            b.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let returned = init.recv::<SpecLoan>().await?;
                    drop(returned);
                    // Withholding both candidate submissions keeps the actual
                    // owner Pending at the next source operation, without an ack.
                    Ok(())
                },
            ),
            async {
                hibana_runtime::host::sleep(std::time::Duration::from_secs(10)).await?;
                Err(ShellError::message("QC boundary fixture deadline"))
            },
        )
        .await;
        if let Some(bad) = bad {
            let error = result.expect_err("substituted QC request succeeded");
            assert!(
                !error.to_string().contains("EndpointError")
                    && !error.to_string().contains("deadline"),
                "{bad}: {error}"
            );
        } else {
            result?;
        }
        let calls = effects.calls.take();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call["tool"] == "verify_result")
                .count(),
            usize::from(bad.is_none())
        );
        assert_eq!(
            calls[0],
            json!({"actor":"CHEMIST","tool":"run_synthesis","arguments":{"compound":materials["synthesis"]["data"]["compound"]}})
        );
        if bad.is_none() {
            assert_eq!(
                calls[1],
                json!({"actor":"QC_CONTROLLER","tool":"verify_result","arguments":args})
            );
        }
    }
    Ok(())
}
#[test]
fn qc_reply_cannot_relabel_rejection_as_approval_or_change_its_subject_or_material() {
    let materials = material();
    for subject in 0..2 {
        let args = arguments(subject);
        let inputs = imp::qc_inputs(&args, &materials).unwrap();
        for approved in [false, true] {
            let result = verdict(&args, &inputs, approved);
            imp::qc_result(&result, &args, &inputs).unwrap();
            for bad in 0..7 {
                let mut result = result.clone();
                match bad {
                    0 => result["success"] = json!(!approved),
                    1 => result["data"]["scientist"] = json!("other scientist"),
                    2 => result["data"]["data_type"] = json!("other data"),
                    3 => result["data"]["qc_agent"] = json!("CHEMIST"),
                    4 => result["data"]["inputs"]["purification"] = json!(90),
                    5 => result["data"]["status"] = json!("missing_or_invalid_input"),
                    _ => result["data"]["missing_inputs"] = json!(["synthesis"]),
                }
                assert!(
                    imp::qc_result(&result, &args, &inputs).is_err(),
                    "{subject}/{approved}/{bad}"
                );
            }
        }
    }
}

#[test]
fn final_validation_binds_both_approvals_and_quality_to_the_same_actual_batch_and_trial() {
    let materials = material();
    let chem_args = arguments(0);
    let bio_args = arguments(1);
    let chem_inputs = imp::qc_inputs(&chem_args, &materials).unwrap();
    let bio_inputs = imp::qc_inputs(&bio_args, &materials).unwrap();
    let mut bio_qc = verdict(&bio_args, &bio_inputs, true);
    bio_qc["data"]["artifact_id"] = json!(8);
    let inputs = json!({"synthesis":0,"purification":2,"bioassay":5});
    let quality = json!({"materials":materials,
        "chemist_qc":verdict(&chem_args,&chem_inputs,true),"biologist_qc":bio_qc,
        "molecular":measured(("ANALYST","run_analysis",json!({"sample":"molecular_detail"})),(9,inputs.clone())),
        "verification":measured(("ANALYST","run_separation",json!({"material":"verification"})),(10,inputs))});
    assert_eq!(
        imp::final_inputs(&quality).unwrap(),
        json!({"synthesis":0,"purification":2,"bioassay":5,
        "purification_qc":7,"bioassay_qc":8,"molecular":9,"verification":10})
    );
    for bad in 0..12 {
        let mut changed = quality.clone();
        match bad {
            0 => changed["chemist_qc"] = verdict(&chem_args, &chem_inputs, false),
            1 => changed["biologist_qc"] = verdict(&bio_args, &bio_inputs, false),
            2 => changed["chemist_qc"]["data"]["inputs"]["analysis"] = json!(1),
            3 => changed["biologist_qc"]["data"]["inputs"]["bioassay"] = json!(90),
            4 => changed["materials"]["synthesis"]["data"]["compound"] = json!("other batch"),
            5 => changed["materials"]["bioassay"]["data"]["compound"] = json!("other batch"),
            6 => changed["molecular"]["data"]["inputs"]["bioassay"] = json!(90),
            7 => changed["verification"]["data"]["inputs"]["purification"] = json!(90),
            8 => changed["molecular"]["success"] = json!(false),
            9 => changed["verification"]["success"] = json!(false),
            10 => changed["chemist_qc"]["data"]["artifact_id"] = Value::Null,
            _ => changed["biologist_qc"]["data"]["artifact_id"] = Value::Null,
        }
        assert!(
            imp::final_inputs(&changed).is_err(),
            "{bad}: substituted or negative final input"
        );
    }
}
