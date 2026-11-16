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

struct Effects {
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
        Ok(self.result.clone())
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
#[hibana_runtime::main]
#[test]
async fn both_actual_resource_requests_return_each_actual_positive_or_negative_result()
-> ShellResult<()> {
    for subject in 0..2 {
        for approved in [false, true] {
            for fume_first in [false, true] {
                let source = g::seq(
                    g::par(
                        g::send::<INIT, QC, SpecLoan>(),
                        g::send::<INIT, QC, FumeLoan>(),
                    ),
                    g::seq(
                        qc_operation(),
                        g::par(
                            g::send::<QC, INIT, SpecLoan>(),
                            g::send::<QC, INIT, FumeLoan>(),
                        ),
                    ),
                );
                let p_init = project::<INIT>(&source);
                let p_qc = project::<QC>(&source);
                let p_service = project::<QC_SERVICE>(&source);
                let p_spec = project::<SPECTROMETER>(&source);
                let p_fume = project::<FUME_HOOD>(&source);
                let session = SessionId::new(SESSION);
                let (bus, _, []) = Bus::open([], [(session, SPECTROMETER), (session, FUME_HOOD)]);
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
                let mut service = server.enter(session, &p_service)?;
                let spec = client.enter(session, &p_spec)?;
                let fume = client.enter(session, &p_fume)?;
                let materials = material();
                let args = arguments(subject);
                let inputs = imp::qc_inputs(&args, &materials)?;
                let actual = verdict(&args, &inputs, approved);
                let effects = Effects {
                    result: actual.clone(),
                    calls: Cell::new(Vec::new()),
                };
                let packet = imp::encode(&json!({"arguments":args,"materials":materials}))?;
                async {
                    async {
                        async {
                            let [mut a, mut b] = init.par()?;
                            async {
                                a.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                b.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        async {
                            let [mut a, mut b] = init.par()?;
                            async {
                                let spec = a.recv::<SpecLoan>().await?;
                                drop(spec);
                                Ok(())
                            };
                            async {
                                let fume = b.recv::<FumeLoan>().await?;
                                drop(fume);
                                Ok(())
                            };
                            Ok(())
                        }
                        .await?;
                    };
                    async {
                        localside::qc::run(&mut service, (&effects, &materials)).await?;
                    };
                    async {
                        let (mut spec, mut fume) = async {
                            let [mut a, mut b] = qc.par()?;
                            async { a.recv::<SpecLoan>().await };
                            async { b.recv::<FumeLoan>().await };
                        }
                        .await?;
                        qc.send::<QcAdmission>(&packet.as_slice()).await?;
                        if fume_first {
                            fume.send::<QcFumeRequest>(&packet.as_slice()).await?;
                            hibana_runtime::host::sleep(std::time::Duration::from_millis(20))
                                .await?;
                            assert!(
                                effects.calls.take().is_empty(),
                                "QC ran without the spectrometer request"
                            );
                            spec.send::<QcSpecRequest>(&packet.as_slice()).await?;
                        } else {
                            spec.send::<QcSpecRequest>(&packet.as_slice()).await?;
                            hibana_runtime::host::sleep(std::time::Duration::from_millis(20))
                                .await?;
                            assert!(
                                effects.calls.take().is_empty(),
                                "QC ran without the fume-hood request"
                            );
                            fume.send::<QcFumeRequest>(&packet.as_slice()).await?;
                        }
                        let (spec_result, fume_result) = async {
                            async { imp::decode(spec.recv::<QcSpecResult>().await?) };
                            async { imp::decode(fume.recv::<QcFumeResult>().await?) };
                        }
                        .await?;
                        let qc_result = imp::decode(qc.recv::<QcResult>().await?)?;
                        assert_eq!(spec_result, actual);
                        assert_eq!(fume_result, actual);
                        assert_eq!(qc_result, actual);
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
                    };
                    Ok(())
                }
                .await?;
                assert_eq!(
                    effects.calls.take(),
                    vec![json!({"actor":"QC_CONTROLLER","tool":"verify_result","arguments":args})]
                );
            }
        }
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn qc_rejects_substituted_requests_and_materials_before_the_effect() -> ShellResult<()> {
    for bad in 0..14 {
        let source = qc_operation();
        let p_qc = project::<QC>(&source);
        let p_service = project::<QC_SERVICE>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open([], [(session, SPECTROMETER), (session, FUME_HOOD)]);
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut qc = rv.enter(session, &p_qc)?;
        let mut service = rv.enter(session, &p_service)?;
        let mut spec = rv.enter(session, &p_spec)?;
        let mut fume = rv.enter(session, &p_fume)?;
        let mut materials = material();
        let args = arguments(if bad >= 9 { 1 } else { 0 });
        let inputs = imp::qc_inputs(&args, &materials)?;
        let effects = Effects {
            result: verdict(&args, &inputs, true),
            calls: Cell::new(Vec::new()),
        };
        let mut admitted = json!({"arguments":args,"materials":materials});
        match bad {
            4 => admitted["arguments"]["data_type"] = json!("wrong subject data"),
            5 => admitted["materials"]["synthesis"]["data"]["artifact_id"] = json!(90),
            6 => materials["synthesis"]["success"] = json!(false),
            7 => materials["initial_analysis"]["data"]["inputs"] = json!({"synthesis":80}),
            8 => materials["purification"]["data"]["compound"] = json!("different batch"),
            9 => materials["bioassay"]["success"] = json!(false),
            10 => materials["bioassay"]["data"]["inputs"]["purification"] = json!(80),
            11 => materials["cells"]["data"]["artifact_id"] = Value::Null,
            12 => materials["calibration"]["data"]["inputs"]["prepared_cells"] = json!(80),
            13 => materials["bioassay"]["data"]["compound"] = json!("foreign batch"),
            _ => {}
        }
        if bad >= 6 {
            admitted["materials"] = materials.clone();
        }
        let mut spec_packet = admitted.clone();
        let mut fume_packet = admitted.clone();
        match bad {
            0 => spec_packet["arguments"]["scientist"] = json!("BIOLOGIST"),
            1 => fume_packet["arguments"]["data_type"] = json!("bioassay"),
            2 => spec_packet["materials"]["analysis"]["data"]["artifact_id"] = json!(90),
            3 => fume_packet["materials"]["synthesis"]["data"]["compound"] = json!("foreign batch"),
            _ => {}
        }
        let admitted = imp::encode(&admitted)?;
        let spec_packet = imp::encode(&spec_packet)?;
        let fume_packet = imp::encode(&fume_packet)?;
        let error = hibana_runtime::race(
            hibana_runtime::race(
                localside::qc::run(&mut service, (&effects, &materials)),
                async {
                    qc.send::<QcAdmission>(&admitted.as_slice()).await?;
                    async {
                        async {
                            spec.send::<QcSpecRequest>(&spec_packet.as_slice()).await?;
                        };
                        async {
                            fume.send::<QcFumeRequest>(&fume_packet.as_slice()).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    core::future::pending::<ShellResult<()>>().await
                },
            ),
            async {
                hibana_runtime::host::sleep(std::time::Duration::from_secs(3)).await?;
                Err(ShellError::message("QC boundary fixture deadline"))
            },
        )
        .await
        .expect_err("substituted QC request succeeded");
        assert!(
            !error.to_string().contains("EndpointError") && !error.to_string().contains("deadline"),
            "{bad}: {error}"
        );
        assert!(
            effects.calls.take().is_empty(),
            "{bad}: rejected request invoked physical effect"
        );
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
