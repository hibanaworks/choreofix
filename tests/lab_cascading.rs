//! Native component checks, not a full 13H Wasm/model qualification.
use choreofix::shell;
#[path = "../src/tasks/lab_cascading/mod.rs"]
mod lab_cascading;

use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
use lab_cascading::{arguments::InspectionArguments, global::*, imp, localside};
use serde_json::{Value, json};
use shell::{ShellResult, imp::bus::Bus, imp::service::Domain};
use std::cell::Cell;

struct Inspection {
    result: Value,
    calls: Cell<Vec<Value>>,
}

impl Domain for Inspection {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.calls.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.calls.set(calls);
        Ok(self.result.clone())
    }
}

#[test]
fn inspection_arguments_keep_optional_free_notes_without_workflow_fields() {
    for value in [
        json!({}),
        json!({"notes":""}),
        json!({"notes":"β notes\n\"arbitrary\" ".repeat(200)}),
    ] {
        let bytes = imp::encode(&value).unwrap();
        assert_eq!(
            InspectionArguments::try_from(bytes.as_slice())
                .unwrap()
                .into_value(),
            value
        );
    }
    for value in [
        Value::Null,
        json!([]),
        json!({"notes":7}),
        json!({"notes":null}),
        json!({"notes":"free", "stage":"inspected"}),
        json!({"approved":true}),
    ] {
        let bytes = imp::encode(&value).unwrap();
        assert!(InspectionArguments::try_from(bytes.as_slice()).is_err());
    }
}

#[test]
fn four_instrument_component_has_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&inspection()), None);
}

#[hibana_runtime::main]
#[test]
async fn officer_returns_all_four_endpoints_and_preserves_the_actual_result() -> ShellResult<()> {
    for success in [true, false] {
        let source = inspection();
        let p_spec_owner = project::<SPEC_OWNER>(&source);
        let p_cent_owner = project::<CENT_OWNER>(&source);
        let p_fume_owner = project::<FUME_OWNER>(&source);
        let p_chroma_owner = project::<CHROMA_OWNER>(&source);
        let p_safety = project::<SAFETY_OFFICER>(&source);
        let p_file = project::<SAFETY_FILE>(&source);
        let p_service = project::<INSPECTION_SERVICE>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_cent = project::<CENTRIFUGE>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_chroma = project::<CHROMATOGRAPH>(&source);
        let session = SessionId::new(132);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, SPECTROMETER),
                (session, CENTRIFUGE),
                (session, FUME_HOOD),
                (session, CHROMATOGRAPH),
            ],
        );
        let records: [Cell<Vec<hibana::runtime::tap::TapEvent>>; 3] =
            core::array::from_fn(|_| Cell::new(Vec::new()));
        let observed_producers = Cell::new(Vec::new());
        let observers = [0, 1, 2].map(|producer| {
            let records = &records;
            let observed_producers = &observed_producers;
            move |event| {
                let mut order = observed_producers.take();
                order.push(producer);
                observed_producers.set(order);
                let mut capture = records[producer].take();
                capture.push(event);
                records[producer].set(capture);
            }
        });
        let mut owner_slab = vec![0; 128 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let owners = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut scientist_slab = vec![0; 128 * 1024];
        let mut scientist_storage = SessionKitStorage::uninit();
        let scientist = scientist_storage
            .init()
            .rendezvous(&mut scientist_slab, &bus.transport)?;
        let mut effect_slab = vec![0; 128 * 1024];
        let mut effect_storage = SessionKitStorage::uninit();
        let effect = effect_storage
            .init()
            .rendezvous(&mut effect_slab, &bus.transport)?;
        owners.observe(&observers[0]);
        scientist.observe(&observers[1]);
        effect.observe(&observers[2]);
        let mut spec_owner = owners.enter(session, &p_spec_owner)?;
        let mut cent_owner = owners.enter(session, &p_cent_owner)?;
        let mut fume_owner = owners.enter(session, &p_fume_owner)?;
        let mut chroma_owner = owners.enter(session, &p_chroma_owner)?;
        let spec = owners.enter(session, &p_spec)?;
        let cent = owners.enter(session, &p_cent)?;
        let fume = owners.enter(session, &p_fume)?;
        let chroma = owners.enter(session, &p_chroma)?;
        let mut safety = scientist.enter(session, &p_safety)?;
        let mut file = scientist.enter(session, &p_file)?;
        let mut service = effect.enter(session, &p_service)?;
        let arguments = json!({"notes":"inspect real instruments β\nwithout a command recipe"});
        let bytes = imp::encode(&arguments)?;
        let request = InspectionArguments::try_from(bytes.as_slice())?;
        let actual = json!({"tool_name":"perform_inspection","success":success,"data":{
            "scientist":"SAFETY_OFFICER","notes":arguments["notes"],"inputs":{},
            "artifact_id":if success {json!(31)} else {Value::Null},
            "status":if success {"complete"} else {"missing_or_invalid_input"},
            "missing_inputs":[],
            "instruments":["CENTRIFUGE","CHROMATOGRAPH","FUME_HOOD","SPECTROMETER"]
        }});
        let inspection = Inspection {
            result: actual.clone(),
            calls: Cell::new(Vec::new()),
        };
        async {
            async {
                assert_eq!(
                    spec_owner.recv::<SpecInspection>().await?.into_value(),
                    arguments
                );
                spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
                drop(spec_owner.recv::<Loan<SPECTROMETER>>().await?);
                Ok(())
            };
            async {
                assert_eq!(
                    cent_owner.recv::<CentInspection>().await?.into_value(),
                    arguments
                );
                cent_owner.send::<Loan<CENTRIFUGE>>(cent).await?;
                drop(cent_owner.recv::<Loan<CENTRIFUGE>>().await?);
                Ok(())
            };
            async {
                assert_eq!(
                    fume_owner.recv::<FumeInspection>().await?.into_value(),
                    arguments
                );
                fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                drop(fume_owner.recv::<Loan<FUME_HOOD>>().await?);
                Ok(())
            };
            async {
                assert_eq!(
                    chroma_owner.recv::<ChromaInspection>().await?.into_value(),
                    arguments
                );
                chroma_owner.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                drop(chroma_owner.recv::<Loan<CHROMATOGRAPH>>().await?);
                Ok(())
            };
            async {
                localside::inspection::run(&mut service, &inspection).await?;
            };
            async {
                localside::safety::run(&mut safety).await?;
            };
            async {
                file.send::<FileInspection>(&request).await?;
                assert_eq!(
                    imp::decode(file.recv::<FileInspectionResult>().await?)?,
                    actual
                );
                Ok(())
            };
            Ok::<(), shell::ShellError>(())
        }
        .await?;
        let calls = inspection.calls.take();
        assert_eq!(
            calls,
            vec![json!({
                "agent":"SAFETY_OFFICER","tool":"perform_inspection","arguments":arguments
            })]
        );
        let checkpoints = [
            owners.tap().checkpoint(),
            scientist.tap().checkpoint(),
            effect.tap().checkpoint(),
        ];
        let captured = records.each_ref().map(|records| records.take());
        let captures: [hibana_epf::Capture; 3] = core::array::from_fn(|producer| {
            hibana_epf::Capture::new(&captured[producer], checkpoints[producer])
                .expect("complete inspection producer capture")
        });
        let snapshot: Vec<_> = hibana::runtime::tap::projected(&p_service).collect();
        assert_eq!(snapshot[0].arg0() >> 16, 23);
        let mut source_storage = vec![0; hibana_epf::Protocol::workspace_len(&snapshot).unwrap()];
        let protocol = hibana_epf::Protocol::new(&snapshot, &mut source_storage).unwrap();
        let count = captured.iter().map(Vec::len).sum::<usize>();
        let capacity = count.checked_mul(2).and_then(|n| n.checked_add(1)).unwrap();
        let mut unfolding = vec![hibana_epf::Unfold::Source; capacity];
        let record_id = hibana_epf::RecordId {
            producer: 0,
            ordinal: 0,
        };
        let mut order = vec![record_id; count];
        let mut occurrences = vec![None; count];
        let mut acceptances = vec![
            hibana_epf::Acceptance {
                first_poll: record_id,
                before: 0
            };
            count
        ];
        let observed_producers = observed_producers.take();
        let mut scratch = vec![0; protocol.audit_workspace_len(&captures, capacity).unwrap()];
        let audited = protocol
            .audit(
                &captures,
                hibana_epf::AuditStorage {
                    unfolding: &mut unfolding,
                    execution: hibana_epf::ExecutionStorage {
                        session: session.raw(),
                        observed_producers: &observed_producers,
                        order: &mut order,
                        occurrences: &mut occurrences,
                        acceptances: &mut acceptances,
                    },
                },
                &mut scratch,
            )
            .expect("actual inspection must have a checked source execution");
        assert_eq!(audited.execution().order.len(), count);
        println!(
            "inspection success={success}: EPF checked {count} complete records from three runtimes"
        );
        if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory)?;
            let frames: Vec<_> = bus.transport.take_observations().collect();
            let evidence = json!({
                "scope":"native four-instrument inspection component; controlled effect fixture, not full 13H qualification",
                "session":session.raw(),
                "source":snapshot.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>(),
                "captures":captured.iter().map(|producer| producer.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "checkpoints":checkpoints,
                "observed_producers":observed_producers,
                "frames":frames,
                "effects":calls,
                "result":actual,
                "epf_checked_records":count,
                "application_effects_certified_by_epf":false
            });
            std::fs::write(
                directory.join(if success {
                    "positive.json"
                } else {
                    "negative.json"
                }),
                serde_json::to_vec_pretty(&evidence)
                    .map_err(|error| shell::ShellError::message(error.to_string()))?,
            )?;
        }
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn inspection_requires_each_received_request_and_rejects_argument_substitution()
-> ShellResult<()> {
    // Exercise every order of the actual resource sends. In every good case,
    // withhold its last request, then resume the same Endpoints. No reattach.
    let mut cases = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if a != b && a != c && a != d && b != c && b != d && c != d {
                        cases.push(([a, b, c, d], None));
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 24);
    for resource in 0..4 {
        cases.push(([0, 1, 2, 3], Some(resource)));
    }
    for (order, substituted) in cases {
        let source = inspection();
        let p_spec_owner = project::<SPEC_OWNER>(&source);
        let p_cent_owner = project::<CENT_OWNER>(&source);
        let p_fume_owner = project::<FUME_OWNER>(&source);
        let p_chroma_owner = project::<CHROMA_OWNER>(&source);
        let p_safety = project::<SAFETY_OFFICER>(&source);
        let p_file = project::<SAFETY_FILE>(&source);
        let p_service = project::<INSPECTION_SERVICE>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_cent = project::<CENTRIFUGE>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_chroma = project::<CHROMATOGRAPH>(&source);
        let session = SessionId::new(132);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, SPECTROMETER),
                (session, CENTRIFUGE),
                (session, FUME_HOOD),
                (session, CHROMATOGRAPH),
            ],
        );
        let mut owner_slab = vec![0; 128 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let owners = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut scientist_slab = vec![0; 128 * 1024];
        let mut scientist_storage = SessionKitStorage::uninit();
        let scientist = scientist_storage
            .init()
            .rendezvous(&mut scientist_slab, &bus.transport)?;
        let mut effect_slab = vec![0; 128 * 1024];
        let mut effect_storage = SessionKitStorage::uninit();
        let effect = effect_storage
            .init()
            .rendezvous(&mut effect_slab, &bus.transport)?;
        let mut spec_owner = owners.enter(session, &p_spec_owner)?;
        let mut cent_owner = owners.enter(session, &p_cent_owner)?;
        let mut fume_owner = owners.enter(session, &p_fume_owner)?;
        let mut chroma_owner = owners.enter(session, &p_chroma_owner)?;
        let spec = owners.enter(session, &p_spec)?;
        let cent = owners.enter(session, &p_cent)?;
        let fume = owners.enter(session, &p_fume)?;
        let chroma = owners.enter(session, &p_chroma)?;
        let mut safety = scientist.enter(session, &p_safety)?;
        let mut file = scientist.enter(session, &p_file)?;
        let mut service = effect.enter(session, &p_service)?;
        let arguments = json!({"notes":"inspect batch β\nexact admitted notes"});
        let bytes = imp::encode(&arguments)?;
        let request = InspectionArguments::try_from(bytes.as_slice())?;
        let actual = json!({"tool_name":"perform_inspection","success":true,"data":{
            "scientist":"SAFETY_OFFICER","notes":arguments["notes"],"artifact_id":19,
            "inputs":{},"status":"complete","missing_inputs":[],
            "instruments":["CENTRIFUGE","CHROMATOGRAPH","FUME_HOOD","SPECTROMETER"]
        }});
        let inspection = Inspection {
            result: actual.clone(),
            calls: Cell::new(Vec::new()),
        };
        let execution = hibana_runtime::race(
            async {
                async {
                    assert_eq!(
                        spec_owner.recv::<SpecInspection>().await?.into_value(),
                        arguments
                    );
                    spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
                    drop(spec_owner.recv::<Loan<SPECTROMETER>>().await?);
                    Ok(())
                };
                async {
                    assert_eq!(
                        cent_owner.recv::<CentInspection>().await?.into_value(),
                        arguments
                    );
                    cent_owner.send::<Loan<CENTRIFUGE>>(cent).await?;
                    drop(cent_owner.recv::<Loan<CENTRIFUGE>>().await?);
                    Ok(())
                };
                async {
                    assert_eq!(
                        fume_owner.recv::<FumeInspection>().await?.into_value(),
                        arguments
                    );
                    fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                    drop(fume_owner.recv::<Loan<FUME_HOOD>>().await?);
                    Ok(())
                };
                async {
                    assert_eq!(
                        chroma_owner.recv::<ChromaInspection>().await?.into_value(),
                        arguments
                    );
                    chroma_owner.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                    drop(chroma_owner.recv::<Loan<CHROMATOGRAPH>>().await?);
                    Ok(())
                };
                async {
                    localside::inspection::run(&mut service, &inspection).await?;
                };
                async {
                    let admitted = safety.recv::<FileInspection>().await?;
                    safety.send::<InspectionAdmission>(&admitted).await?;
                    let (mut spec, mut cent, mut fume, mut chroma) = async {
                        let [mut a, mut b, mut c, mut d] = safety.par()?;
                        async {
                            a.send::<SpecInspection>(&admitted).await?;
                            a.recv::<Loan<SPECTROMETER>>().await
                        };
                        async {
                            b.send::<CentInspection>(&admitted).await?;
                            b.recv::<Loan<CENTRIFUGE>>().await
                        };
                        async {
                            c.send::<FumeInspection>(&admitted).await?;
                            c.recv::<Loan<FUME_HOOD>>().await
                        };
                        async {
                            d.send::<ChromaInspection>(&admitted).await?;
                            d.recv::<Loan<CHROMATOGRAPH>>().await
                        };
                    }
                    .await?;
                    for (position, resource) in order.into_iter().enumerate() {
                        if position == 3 {
                            hibana_runtime::host::sleep(std::time::Duration::from_millis(1))
                                .await?;
                            let calls = inspection.calls.take();
                            assert!(calls.is_empty(), "inspection without resource {resource}");
                            inspection.calls.set(calls);
                        }
                        let input = if substituted == Some(resource) {
                            json!({"notes":"a different, otherwise valid inspection request"})
                        } else {
                            arguments.clone()
                        };
                        let bytes = imp::encode(&input)?;
                        let input = InspectionArguments::try_from(bytes.as_slice())?;
                        match resource {
                            0 => spec.send::<SpecInspection>(&input).await?,
                            1 => cent.send::<CentInspection>(&input).await?,
                            2 => fume.send::<FumeInspection>(&input).await?,
                            3 => chroma.send::<ChromaInspection>(&input).await?,
                            _ => unreachable!(),
                        }
                    }
                    let results = async {
                        async { imp::decode(spec.recv::<SpecResult>().await?) };
                        async { imp::decode(cent.recv::<CentResult>().await?) };
                        async { imp::decode(fume.recv::<FumeResult>().await?) };
                        async { imp::decode(chroma.recv::<ChromaResult>().await?) };
                    }
                    .await?;
                    assert_eq!(
                        results,
                        (
                            actual.clone(),
                            actual.clone(),
                            actual.clone(),
                            actual.clone()
                        )
                    );
                    let bytes = imp::encode(&results.0)?;
                    async {
                        let [mut a, mut b, mut c, mut d, mut file] = safety.par()?;
                        async {
                            a.send::<Loan<SPECTROMETER>>(spec).await?;
                        };
                        async {
                            b.send::<Loan<CENTRIFUGE>>(cent).await?;
                        };
                        async {
                            c.send::<Loan<FUME_HOOD>>(fume).await?;
                        };
                        async {
                            d.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
                        };
                        async {
                            file.send::<FileInspectionResult>(&bytes.as_slice()).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    Ok(())
                };
                async {
                    file.send::<FileInspection>(&request).await?;
                    assert_eq!(
                        imp::decode(file.recv::<FileInspectionResult>().await?)?,
                        actual
                    );
                    Ok(())
                };
                Ok::<(), shell::ShellError>(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(20)),
        )
        .await;
        let calls = inspection.calls.take();
        if substituted.is_some() {
            let error = execution.expect_err("different instrument input reached the effect");
            assert!(
                error
                    .to_string()
                    .contains("differs from the admitted file input"),
                "{error}"
            );
            assert!(calls.is_empty());
        } else {
            execution?;
            assert_eq!(
                calls,
                vec![json!({
                    "agent":"SAFETY_OFFICER","tool":"perform_inspection","arguments":arguments
                })]
            );
        }
    }
    Ok(())
}
