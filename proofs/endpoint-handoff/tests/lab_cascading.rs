//! Native component checks, not a full 13H Wasm/model qualification.
use choreofix::{imp, shell};
#[path = "../../../src/tasks/lab_cascading/mod.rs"]
pub mod lab_cascading;

use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
    },
};
use lab_cascading::{
    arguments::{Arguments, InspectionArguments},
    global::*,
    imp as lab_imp, localside,
};
use serde_json::{Value, json};
use shell::{ShellError, ShellResult, imp::bus::Bus, imp::service::Domain};
use std::cell::Cell;
fn quality() -> impl Projectable {
    g::seq(
        g::send::<BIOLOGIST, QC, BiologyReport>(),
        g::seq(
            g::send::<QC_FILE, QC, FileQuality>(),
            quality_exchange(
                g::seq(
                    g::send::<QC, BIOLOGIST, BiologyApproved>(),
                    g::send::<QC, QC_FILE, FileQualityResult>(),
                ),
                g::seq(
                    g::send::<QC, FUME_OWNER, Loan<REAGENT_SUPPLY>>(),
                    g::seq(
                        g::send::<QC, BIOLOGIST, BiologyRejected>(),
                        g::send::<QC, QC_FILE, FileQualityResult>(),
                    ),
                ),
            ),
        ),
    )
}

fn synthesis() -> impl Projectable {
    g::seq(
        g::send::<CHEM_DATA, FUME_OWNER, Loan<CHEM_FILE>>(),
        g::seq(
            synthesis_attempt().roll(),
            g::seq(
                g::route(
                    g::seq(
                        g::send::<FUME_OWNER, CHEMIST, SynthesisFinished>(),
                        g::send::<FUME_OWNER, QC, SynthesisFinished>(),
                    ),
                    g::seq(
                        g::send::<FUME_OWNER, CHEMIST, ReagentsExhausted>(),
                        g::send::<FUME_OWNER, QC, ReagentsExhausted>(),
                    ),
                ),
                g::send::<FUME_OWNER, QC, Loan<REAGENT_SUPPLY>>(),
            ),
        ),
    )
}
use lab_cascading::imp::SynthesisTerminal;
use shell::imp::review_reply::ReviewReply;

// Offline data checking only: this owns no Endpoint and drives no task protocol.
fn audit_native(
    captures: &[hibana_epf::Capture<'_>],
    snapshot: &[hibana::runtime::tap::TapEvent],
    (session, observed_producers): (u32, &[usize]),
) -> usize {
    let mut source_storage = vec![0; hibana_epf::Protocol::workspace_len(snapshot).unwrap()];
    let protocol = hibana_epf::Protocol::new(snapshot, &mut source_storage).unwrap();
    let count = captures
        .iter()
        .map(|capture| capture.records().len())
        .sum::<usize>();
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
    let mut scratch = vec![0; protocol.audit_workspace_len(captures, capacity).unwrap()];
    let audited = protocol
        .audit(
            captures,
            hibana_epf::AuditStorage {
                unfolding: &mut unfolding,
                execution: hibana_epf::ExecutionStorage {
                    session,
                    observed_producers,
                    order: &mut order,
                    occurrences: &mut occurrences,
                    acceptances: &mut acceptances,
                },
            },
            &mut scratch,
        )
        .expect("actual laboratory execution must have a checked source interpretation");
    assert_eq!(audited.execution().order.len(), count);
    count
}

struct Effect {
    result: Value,
    calls: Cell<Vec<Value>>,
}

impl Domain for Effect {
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
        let bytes = lab_imp::encode(&value).unwrap();
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
        let bytes = lab_imp::encode(&value).unwrap();
        assert!(InspectionArguments::try_from(bytes.as_slice()).is_err());
    }
}

#[test]
fn four_instrument_component_has_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&inspection()), None);
}

#[test]
fn scientific_workflow_has_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&science()), None);
}

#[test]
fn scientific_workflow_builds_every_actual_role_program() {
    use choreofix::imp::projection::rows;
    let source = science();
    let programs = [
        rows(&project::<CHEMIST>(&source)),
        rows(&project::<BIOLOGIST>(&source)),
        rows(&project::<ANALYST>(&source)),
        rows(&project::<QC>(&source)),
        rows(&project::<SAFETY_OFFICER>(&source)),
        rows(&project::<CHEM_CONTROL>(&source)),
        rows(&project::<BIO_CONTROL>(&source)),
        rows(&project::<INIT>(&source)),
        rows(&project::<SPEC_OWNER>(&source)),
        rows(&project::<CENT_OWNER>(&source)),
        rows(&project::<FUME_OWNER>(&source)),
        rows(&project::<CHROMA_OWNER>(&source)),
        rows(&project::<INSPECTION_SERVICE>(&source)),
        rows(&project::<SPECTROMETER>(&source)),
        rows(&project::<CENTRIFUGE>(&source)),
        rows(&project::<FUME_HOOD>(&source)),
        rows(&project::<CHROMATOGRAPH>(&source)),
        rows(&project::<REAGENT_SUPPLY>(&source)),
        rows(&project::<CHEM_FILE>(&source)),
        rows(&project::<BIO_FILE>(&source)),
        rows(&project::<ANALYST_FILE>(&source)),
        rows(&project::<QC_FILE>(&source)),
        rows(&project::<SAFETY_FILE>(&source)),
        rows(&project::<CHEM_DATA>(&source)),
        rows(&project::<BIO_DATA>(&source)),
        rows(&project::<ANALYST_DATA>(&source)),
        rows(&project::<QC_DATA>(&source)),
        rows(&project::<SAFETY_DATA>(&source)),
    ];
    // Each call above builds the real immutable image through mandatory const
    // structural and source-order checks. This is not a localside execution.
    if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("science-projection.json"),
            serde_json::to_vec_pretty(&json!({"programs":programs})).unwrap(),
        )
        .unwrap();
    }
    core::hint::black_box(programs);
}

#[test]
fn publication_names_actual_artifact_without_workflow_fields() {
    for id in [0, 1, u64::MAX] {
        let value = json!({"artifact_id":id});
        let bytes = lab_imp::encode(&value).unwrap();
        assert_eq!(
            Arguments::<11>::try_from(bytes.as_slice())
                .unwrap()
                .into_value(),
            value,
        );
    }
    for value in [
        json!({}),
        json!({"artifact_id":-1}),
        json!({"artifact_id":"1"}),
        json!({"artifact_id":1,"approved":true}),
        json!({"artifact_id":1,"phase":"QC"}),
    ] {
        assert!(Arguments::<11>::try_from(lab_imp::encode(&value).unwrap().as_slice()).is_err());
    }
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
        let bytes = lab_imp::encode(&arguments)?;
        let request = InspectionArguments::try_from(bytes.as_slice())?;
        let actual = json!({"tool_name":"perform_inspection","success":success,"data":{
            "scientist":"SAFETY_OFFICER","notes":arguments["notes"],"inputs":{},
            "artifact_id":if success {json!(31)} else {Value::Null},
            "status":if success {"complete"} else {"missing_or_invalid_input"},
            "missing_inputs":[],
            "instruments":["CENTRIFUGE","CHROMATOGRAPH","FUME_HOOD","SPECTROMETER"]
        }});
        let inspection = Effect {
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
                    lab_imp::decode(file.recv::<FileInspectionResult>().await?)?,
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
        let count = captured.iter().map(Vec::len).sum::<usize>();
        let observed_producers = observed_producers.take();
        assert_eq!(
            audit_native(&captures, &snapshot, (session.raw(), &observed_producers)),
            count
        );
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
        let bytes = lab_imp::encode(&arguments)?;
        let request = InspectionArguments::try_from(bytes.as_slice())?;
        let actual = json!({"tool_name":"perform_inspection","success":true,"data":{
            "scientist":"SAFETY_OFFICER","notes":arguments["notes"],"artifact_id":19,
            "inputs":{},"status":"complete","missing_inputs":[],
            "instruments":["CENTRIFUGE","CHROMATOGRAPH","FUME_HOOD","SPECTROMETER"]
        }});
        let inspection = Effect {
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
                        let bytes = lab_imp::encode(&input)?;
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
                        async { lab_imp::decode(spec.recv::<SpecResult>().await?) };
                        async { lab_imp::decode(cent.recv::<CentResult>().await?) };
                        async { lab_imp::decode(fume.recv::<FumeResult>().await?) };
                        async { lab_imp::decode(chroma.recv::<ChromaResult>().await?) };
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
                    let bytes = lab_imp::encode(&results.0)?;
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
                        lab_imp::decode(file.recv::<FileInspectionResult>().await?)?,
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

// Plain fixture values, not another material store or application progress state.
fn synthesized(compound: &str, (status, remaining, artifact): (&str, u64, Option<u64>)) -> Value {
    json!({"tool_name":"run_synthesis","success":status=="complete","data":{
        "scientist":"CHEMIST","compound":compound,"inputs":{},"missing_inputs":[],
        "status":status,"artifact_id":artifact,"reagent_units_remaining":remaining
    }})
}

struct Synthesis {
    results: Vec<Value>,
    calls: Cell<Vec<Value>>,
}

impl Domain for Synthesis {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.calls.take();
        let result = self.results.get(calls.len()).cloned();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.calls.set(calls);
        result.ok_or_else(|| "unexpected additional physical synthesis in fixture".into())
    }
}

#[test]
fn scientific_arguments_preserve_original_data_and_reject_workflow_fields() {
    fn roundtrip<const OP: u8>(arguments: Value) {
        let bytes = lab_imp::encode(&arguments).unwrap();
        assert_eq!(
            Arguments::<OP>::try_from(bytes.as_slice())
                .unwrap()
                .into_value(),
            arguments
        );
        let mut unrelated = arguments;
        unrelated["stage"] = json!("host-selected operation");
        let bytes = lab_imp::encode(&unrelated).unwrap();
        assert!(Arguments::<OP>::try_from(bytes.as_slice()).is_err());
    }
    roundtrip::<0>(json!({"compound":"free β compound\n\"not a fixed batch name\""}));
    roundtrip::<1>(json!({"sample":"synthesized_compound"}));
    roundtrip::<2>(json!({"material":"compound_purification"}));
    roundtrip::<3>(json!({"material":"cell_cultures"}));
    roundtrip::<4>(json!({"sample":"fluorescence"}));
    roundtrip::<5>(json!({"compound":"free β compound"}));
    roundtrip::<5>(json!({"compound":"free β compound","cell_type":"free β culture".repeat(90)}));
    roundtrip::<6>(json!({"sample":"free β compound"}));
    roundtrip::<7>(json!({"sample":"molecular_detail"}));
    roundtrip::<8>(json!({"material":"verification"}));
    roundtrip::<9>(json!({"scientist":"BIOLOGIST","data_type":"bioassay"}));
    roundtrip::<10>(json!({"sample":"final_validation"}));
    for arguments in [
        json!({"compound":""}),
        json!({"compound":" \n\t"}),
        json!({"compound":7}),
    ] {
        let bytes = lab_imp::encode(&arguments).unwrap();
        assert!(Arguments::<0>::try_from(bytes.as_slice()).is_err());
    }
    let bytes =
        lab_imp::encode(&json!({"scientist":"CHEMIST","data_type":"purification"})).unwrap();
    assert!(Arguments::<9>::try_from(bytes.as_slice()).is_err());
}

#[test]
fn synthesis_terminal_payload_cannot_turn_retry_or_exhaustion_into_success() {
    for result in [
        synthesized("β", ("complete", 2, Some(0))),
        synthesized("β", ("synthesis_failed", 0, None)),
        synthesized("β", ("reagent_exhausted", 0, None)),
    ] {
        let bytes = lab_imp::encode(&result).unwrap();
        assert_eq!(
            lab_imp::SynthesisTerminal::<true>::try_from(bytes.as_slice()).is_ok(),
            result["success"] == true
        );
        assert_eq!(
            lab_imp::SynthesisTerminal::<false>::try_from(bytes.as_slice()).is_ok(),
            result["data"]["status"] == "reagent_exhausted"
        );
    }
    let mut result = synthesized("β", ("reagent_exhausted", 0, None));
    result["data"]["reagent_units_remaining"] = json!(1);
    let bytes = lab_imp::encode(&result).unwrap();
    assert!(lab_imp::SynthesisTerminal::<false>::try_from(bytes.as_slice()).is_err());
}

#[test]
fn synthesis_retry_and_terminal_handoff_have_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&synthesis()), None);
}

fn quality_packet() -> Value {
    let recorded = |(tool, scientist, arguments): (&str, &str, Value), inputs: Value, id: u64| {
        let mut data = json!({"scientist":scientist,"inputs":inputs,"missing_inputs":[],
            "status":"complete","artifact_id":id});
        for (key, value) in arguments.as_object().unwrap() {
            data[key] = value.clone();
        }
        json!({"tool_name":tool,"success":true,"data":data})
    };
    let synthesis = synthesized("β compound", ("complete", 2, Some(0)));
    let analysis = recorded(
        (
            "run_analysis",
            "CHEMIST",
            json!({"sample":"synthesized_compound"}),
        ),
        json!({"synthesis":0}),
        1,
    );
    let purification = recorded(
        (
            "run_separation",
            "CHEMIST",
            json!({"material":"compound_purification","compound":"β compound"}),
        ),
        json!({"synthesis":0,"analysis":1}),
        2,
    );
    let cells = recorded(
        (
            "run_separation",
            "BIOLOGIST",
            json!({"material":"cell_cultures"}),
        ),
        json!({}),
        3,
    );
    let calibration = recorded(
        (
            "run_analysis",
            "BIOLOGIST",
            json!({"sample":"fluorescence"}),
        ),
        json!({"prepared_cells":3}),
        4,
    );
    let bioassay = recorded(
        (
            "run_bioassay",
            "BIOLOGIST",
            json!({"compound":"β compound","cell_type":"free β culture"}),
        ),
        json!({"synthesis":0,"purification":2,"prepared_cells":3,"calibration":4}),
        5,
    );
    json!({"arguments":{"scientist":"BIOLOGIST","data_type":"bioassay"},
        "material":{"synthesis":synthesis,"analysis":analysis,"purification":purification,
        "cells":cells,"calibration":calibration,"bioassay":bioassay}})
}

fn quality_verdict(approved: bool) -> Value {
    json!({"tool_name":"verify_result","success":approved,"data":{
        "scientist":"BIOLOGIST","data_type":"bioassay","qc_agent":"QC_CONTROLLER",
        "inputs":{"synthesis":0,"purification":2,"bioassay":5},"missing_inputs":[],
        "status":if approved {"complete"} else {"rejected"},
        "artifact_id":if approved {json!(6)} else {Value::Null}
    }})
}

#[test]
fn quality_uses_actual_material_records_and_distinct_approval_and_rejection() {
    let packet = quality_packet();
    assert_eq!(
        lab_imp::quality_input(&packet).unwrap(),
        json!({"synthesis":0,"purification":2,"bioassay":5})
    );
    for approved in [false, true] {
        let result = quality_verdict(approved);
        lab_imp::quality_result(&result, &packet).unwrap();
        for key in ["synthesis", "purification", "bioassay"] {
            let mut old = result.clone();
            old["data"]["inputs"][key] = json!(99);
            assert!(lab_imp::quality_result(&old, &packet).is_err(), "{key}");
        }
        let mut wrong_subject = result;
        wrong_subject["data"]["scientist"] = json!("CHEMIST");
        assert!(lab_imp::quality_result(&wrong_subject, &packet).is_err());
    }
    for record in [
        "synthesis",
        "analysis",
        "purification",
        "cells",
        "calibration",
    ] {
        let mut old = packet.clone();
        old["material"][record]["data"]["artifact_id"] = json!(99);
        assert!(lab_imp::quality_input(&old).is_err(), "{record}");
    }
    for input in ["synthesis", "purification", "prepared_cells", "calibration"] {
        let mut old = packet.clone();
        old["material"]["bioassay"]["data"]["inputs"][input] = json!(99);
        assert!(lab_imp::quality_input(&old).is_err(), "{input}");
    }
}

#[test]
fn cascading_quality_has_no_projection_diagnostic() {
    assert_eq!(hibana::g::diagnose(&quality()), None);
    assert_eq!(
        hibana::g::diagnose(&hibana::g::seq(synthesis(), quality())),
        None
    );
}

#[hibana_runtime::main]
#[test]
async fn actual_synthesis_failures_retry_and_exhaustion_returns_normally_with_one_stock_endpoint()
-> ShellResult<()> {
    use hibana::g::Message;
    let cases = [
        vec![("complete", 2, Some(0))],
        vec![("synthesis_failed", 2, None), ("complete", 1, Some(1))],
        vec![
            ("synthesis_failed", 2, None),
            ("synthesis_failed", 1, None),
            ("complete", 0, Some(2)),
        ],
        vec![
            ("synthesis_failed", 2, None),
            ("synthesis_failed", 1, None),
            ("synthesis_failed", 0, None),
            ("reagent_exhausted", 0, None),
        ],
        vec![("reagent_exhausted", 0, None)],
    ];
    for (case, outcomes) in cases.into_iter().enumerate() {
        let source = synthesis();
        let p_owner = project::<FUME_OWNER>(&source);
        let p_chemist = project::<CHEMIST>(&source);
        let p_file = project::<CHEM_FILE>(&source);
        let p_data = project::<CHEM_DATA>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_stock = project::<REAGENT_SUPPLY>(&source);
        let p_qc = project::<QC>(&source);
        let session = SessionId::new(133);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, FUME_HOOD),
                (session, REAGENT_SUPPLY),
                (session, CHEM_FILE),
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
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut scientist_slab = vec![0; 128 * 1024];
        let mut scientist_storage = SessionKitStorage::uninit();
        let scientist_rv = scientist_storage
            .init()
            .rendezvous(&mut scientist_slab, &bus.transport)?;
        let mut qc_slab = vec![0; 128 * 1024];
        let mut qc_storage = SessionKitStorage::uninit();
        let qc_rv = qc_storage.init().rendezvous(&mut qc_slab, &bus.transport)?;
        owner_rv.observe(&observers[0]);
        scientist_rv.observe(&observers[1]);
        qc_rv.observe(&observers[2]);
        let mut owner = owner_rv.enter(session, &p_owner)?;
        let fume = owner_rv.enter(session, &p_fume)?;
        let stock = owner_rv.enter(session, &p_stock)?;
        let mut chemist = scientist_rv.enter(session, &p_chemist)?;
        let mut data = owner_rv.enter(session, &p_data)?;
        let file = owner_rv.enter(session, &p_file)?;
        let mut qc = qc_rv.enter(session, &p_qc)?;
        data.send::<Loan<CHEM_FILE>>(file).await?;
        let mut file = owner.recv::<Loan<CHEM_FILE>>().await?;
        let results: Vec<_> = outcomes
            .into_iter()
            .enumerate()
            .map(|(attempt, outcome)| {
                synthesized(
                    &format!("free β compound case {case} attempt {attempt}\n"),
                    outcome,
                )
            })
            .collect();
        let expected = results.last().unwrap().clone();
        let domain = Synthesis {
            results: results.clone(),
            calls: Cell::new(Vec::new()),
        };
        let execution = hibana_runtime::race(
            async {
                async {
async {
let endpoint = &mut owner;
let domain = &domain;
let mut fume = fume;
let mut stock = stock;

    loop {
        let admitted = endpoint.recv::<SynthesisAdmission>().await?.into_value();
        async {
            let [mut fume_owner, mut stock_owner] = endpoint.par()?;
            async {
                fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
            };
            async {
                stock_owner.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            };
            Ok(())
        }
        .await?;
        let (fume_request, stock_request) = async {
            let [mut fume_request, mut stock_request] = endpoint.par()?;
            async { Ok(fume_request.recv::<FumeSynthesis>().await?.into_value()) };
            async { Ok(stock_request.recv::<ReagentSynthesis>().await?.into_value()) };
        }
        .await?;
        if fume_request != admitted || stock_request != admitted {
            return Err(ShellError::message(
                "synthesis resource request differs from admitted file arguments",
            ));
        }
        let actual = domain
            .execute("CHEMIST", "run_synthesis", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        lab_imp::synthesis_result(&actual, &admitted)?;
        let bytes = lab_imp::encode(&actual)?;
        async {
            let [mut fume_result, mut stock_result] = endpoint.par()?;
            async {
                fume_result
                    .send::<FumeSynthesisResult>(&bytes.as_slice())
                    .await?;
            };
            async {
                stock_result
                    .send::<ReagentSynthesisResult>(&bytes.as_slice())
                    .await?;
            };
            Ok(())
        }
        .await?;
        (fume, stock) = async {
            let [mut fume_owner, mut stock_owner] = endpoint.par()?;
            async { fume_owner.recv::<Loan<FUME_HOOD>>().await };
            async { stock_owner.recv::<Loan<REAGENT_SUPPLY>>().await };
        }
        .await?;
        if actual["success"] == true {
            let terminal = SynthesisTerminal::<true>::try_from(bytes.as_slice())?;
            endpoint.send::<SynthesisFinished>(&terminal).await?;
            endpoint.send::<SynthesisFinished>(&terminal).await?;
            endpoint.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            return Ok(fume);
        }
        if actual["data"]["status"] == "reagent_exhausted" {
            let terminal = SynthesisTerminal::<false>::try_from(bytes.as_slice())?;
            endpoint.send::<ReagentsExhausted>(&terminal).await?;
            endpoint.send::<ReagentsExhausted>(&terminal).await?;
            endpoint.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            return Ok(fume);
        }
    }

}.await
                        .map_err(|error| {
                            shell::ShellError::message(format!("synthesis owner: {error}"))
                        })
                };
                async {
                    let observed =
async {
let endpoint = &mut chemist;

    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            FileSynthesis::LOGICAL_LABEL => {
                let admitted = branch.recv::<FileSynthesis>().await?;
                endpoint.send::<SynthesisAdmission>(&admitted).await?;
                let (mut fume, mut stock) = async {
                    let [mut fume, mut stock] = endpoint.par()?;
                    async { fume.recv::<Loan<FUME_HOOD>>().await };
                    async { stock.recv::<Loan<REAGENT_SUPPLY>>().await };
                }
                .await?;
                let (fume_result, stock_result) = async {
                    async {
                        fume.send::<FumeSynthesis>(&admitted).await?;
                        lab_imp::decode(fume.recv::<FumeSynthesisResult>().await?)
                    };
                    async {
                        stock.send::<ReagentSynthesis>(&admitted).await?;
                        lab_imp::decode(stock.recv::<ReagentSynthesisResult>().await?)
                    };
                }
                .await?;
                if fume_result != stock_result {
                    return Err(ShellError::message(
                        "synthesis resources returned different actual results",
                    ));
                }
                lab_imp::synthesis_result(&fume_result, &admitted.into_value())?;
                let bytes = lab_imp::encode(&fume_result)?;
                async {
                    let [mut fume_owner, mut stock_owner, mut file] = endpoint.par()?;
                    async {
                        fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                    };
                    async {
                        stock_owner.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                    };
                    async {
                        file.send::<FileSynthesisResult>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            SynthesisFinished::LOGICAL_LABEL => {
                return Ok(branch.recv::<SynthesisFinished>().await?.into_value());
            }
            ReagentsExhausted::LOGICAL_LABEL => {
                return Ok(branch.recv::<ReagentsExhausted>().await?.into_value());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected synthesis operation {label}"
                )));
            }
        }
    }

}.await
                            .map_err(|error| shell::ShellError::message(format!(
                                "synthesis chemist: {error}"
                            )))?;
                    assert_eq!(observed, expected);
                    Ok::<(), shell::ShellError>(())
                };
                async {
                    for (attempt, result) in results.iter().enumerate() {
                        let arguments = json!({"compound":result["data"]["compound"]});
                        let bytes = lab_imp::encode(&arguments)?;
                        let arguments = Arguments::<0>::try_from(bytes.as_slice())?;
                        file.send::<FileSynthesis>(&arguments)
                            .await
                            .map_err(|error| {
                                shell::ShellError::message(format!(
                                        "synthesis file request case={case} attempt={attempt}: {error:?}"
                                ))
                            })?;
                        assert_eq!(
                            lab_imp::decode(file.recv::<FileSynthesisResult>().await?)?,
                            *result
                        );
                    }
                    Ok::<(), shell::ShellError>(())
                };
                async {
                    let branch = qc.offer().await?;
                    let actual = match branch.label() {
                        SynthesisFinished::LOGICAL_LABEL => {
                            branch.recv::<SynthesisFinished>().await?.into_value()
                        }
                        ReagentsExhausted::LOGICAL_LABEL => {
                            branch.recv::<ReagentsExhausted>().await?.into_value()
                        }
                        label => {
                            return Err(shell::ShellError::message(format!(
                                "unexpected QC synthesis outcome {label}"
                            )));
                        }
                    };
                    assert_eq!(actual, expected);
                    Ok::<_, shell::ShellError>(qc.recv::<Loan<REAGENT_SUPPLY>>().await?)
                };
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(20)),
        )
        .await;
        if execution.is_err() {
            let captured = records.each_ref().map(|records| records.take());
            for (producer, events) in captured.iter().enumerate() {
                for event in events.iter().rev().take(12).rev() {
                    eprintln!(
                        "synthesis failure producer={producer} record={:?}",
                        choreofix::imp::projection::record(*event)
                    );
                }
            }
            if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory)?;
                let snapshot: Vec<_> = hibana::runtime::tap::projected(&p_owner)
                    .map(choreofix::imp::projection::record)
                    .collect();
                let evidence = json!({"case":case,"source":snapshot,
                    "captures":captured.iter().map(|events|events.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>()).collect::<Vec<_>>(),
                    "frames":bus.transport.take_observations().collect::<Vec<_>>(),"calls":domain.calls.take(),
                    "expected_results":results});
                std::fs::write(
                    directory.join(format!("failure-case-{case}.json")),
                    serde_json::to_vec_pretty(&evidence)
                        .map_err(|error| shell::ShellError::message(error.to_string()))?,
                )?;
            }
        }
        let (returned_fume, (), (), returned_stock) = execution?;
        let frames_before_probe: Vec<_> = bus.transport.take_observations().collect();
        {
            // The production localsides have returned after the terminal
            // handoff. A fresh file request must not publish work to them.
            let bytes = lab_imp::encode(&json!({"compound":"a later unrequested batch"}))?;
            let arguments = Arguments::<0>::try_from(bytes.as_slice())?;
            let before = owner_rv.tap().checkpoint();
            let retry = hibana_runtime::race(
                async { Ok::<_, shell::ShellError>(file.send::<FileSynthesis>(&arguments).await) },
                hibana_runtime::host::deadline(std::time::Duration::from_millis(100)),
            )
            .await?;
            let after = owner_rv.tap().checkpoint();
            let published: Vec<_> = bus.transport.take_observations().collect();
            if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory)?;
                std::fs::write(
                    directory.join(format!("post-terminal-request-{case}.json")),
                    serde_json::to_vec_pretty(&json!({
                        "scope":"actual Endpoint probe after terminal resource handoff",
                        "case":case,"resource_endpoints_retained":true,
                        "request":format!("{retry:?}"),
                        "checkpoint_before":before,"checkpoint_after":after,
                        "additional_transport_observations":published,
                        "recoverable_refusal":retry.as_ref().err().is_some_and(|error| error.is_unavailable()),
                        "core_changed_for_probe":false
                    }))
                    .map_err(|error| shell::ShellError::message(error.to_string()))?,
                )?;
            }
            // No continuation remains in this component. This checks terminal
            // rejection, not recovery of another legal Wasm/file operation.
            // Recoverable completed-send rejection excludes repeating bodies.
            retry.expect_err("terminal synthesis must close further file requests");
            assert_eq!(before, after, "a refused request must publish no operation");
            assert!(
                published.is_empty(),
                "a refused request published {published:?}"
            );
        }
        drop(returned_stock);
        drop(returned_fume);
        let calls = domain.calls.take();
        let expected_calls: Vec<_> = results.iter().map(|result| json!({
            "agent":"CHEMIST","tool":"run_synthesis","arguments":{"compound":result["data"]["compound"]}
        })).collect();
        assert_eq!(calls, expected_calls);
        let checkpoints = [
            owner_rv.tap().checkpoint(),
            scientist_rv.tap().checkpoint(),
            qc_rv.tap().checkpoint(),
        ];
        let captured = records.each_ref().map(|records| records.take());
        let captures: [hibana_epf::Capture; 3] = core::array::from_fn(|producer| {
            hibana_epf::Capture::new(&captured[producer], checkpoints[producer])
                .expect("complete synthesis producer capture")
        });
        let snapshot: Vec<_> = hibana::runtime::tap::projected(&p_owner).collect();
        assert_eq!(snapshot[0].arg0() >> 16, 17);
        let observed_producers = observed_producers.take();
        let count = audit_native(&captures, &snapshot, (session.raw(), &observed_producers));
        println!(
            "synthesis case={case}: {} actual effects, terminal={}, EPF checked {count} complete records",
            calls.len(),
            expected["data"]["status"]
        );
        if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory)?;
            let frames: Vec<_> = frames_before_probe
                .into_iter()
                .chain(bus.transport.take_observations())
                .collect();
            let evidence = json!({
                "scope":"native synthesis/reagent component; controlled actual-result fixture, not full 13H qualification",
                "session":session.raw(),
                "source":snapshot.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>(),
                "captures":captured.iter().map(|producer| producer.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "checkpoints":checkpoints,"observed_producers":observed_producers,
                "frames":frames,"effects":calls,"results":results,"terminal":expected,
                "epf_checked_records":count,"application_effects_certified_by_epf":false
            });
            std::fs::write(
                directory.join(format!("synthesis-{case}.json")),
                serde_json::to_vec_pretty(&evidence)
                    .map_err(|error| shell::ShellError::message(error.to_string()))?,
            )?;
        }
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn actual_quality_uses_both_requests_and_returns_stock_only_after_rejection()
-> ShellResult<()> {
    use hibana::g::Message;
    for (approved, substitute) in [
        (true, None),
        (false, None),
        (true, Some("fume")),
        (true, Some("spectrometer")),
    ] {
        let source = quality();
        let p_owner = project::<FUME_OWNER>(&source);
        let p_spec_owner = project::<SPEC_OWNER>(&source);
        let p_qc = project::<QC>(&source);
        let p_biologist = project::<BIOLOGIST>(&source);
        let p_file = project::<QC_FILE>(&source);
        let p_fume = project::<FUME_HOOD>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_stock = project::<REAGENT_SUPPLY>(&source);
        let session = SessionId::new(134);
        let (bus, _, []) = Bus::open(
            [],
            [
                (session, FUME_HOOD),
                (session, SPECTROMETER),
                (session, REAGENT_SUPPLY),
            ],
        );
        let quality_actual = Cell::new(Value::Null);
        let records: [Cell<Vec<hibana::runtime::tap::TapEvent>>; 5] =
            core::array::from_fn(|_| Cell::new(Vec::new()));
        let observed_producers = Cell::new(Vec::new());
        let observers = [0, 1, 2, 3, 4].map(|producer| {
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
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut spec_slab = vec![0; 128 * 1024];
        let mut spec_storage = SessionKitStorage::uninit();
        let spec_rv = spec_storage
            .init()
            .rendezvous(&mut spec_slab, &bus.transport)?;
        let mut qc_slab = vec![0; 128 * 1024];
        let mut qc_storage = SessionKitStorage::uninit();
        let qc_rv = qc_storage.init().rendezvous(&mut qc_slab, &bus.transport)?;
        let mut bio_slab = vec![0; 128 * 1024];
        let mut bio_storage = SessionKitStorage::uninit();
        let bio_rv = bio_storage
            .init()
            .rendezvous(&mut bio_slab, &bus.transport)?;
        let mut file_slab = vec![0; 128 * 1024];
        let mut file_storage = SessionKitStorage::uninit();
        let file_rv = file_storage
            .init()
            .rendezvous(&mut file_slab, &bus.transport)?;
        owner_rv.observe(&observers[0]);
        spec_rv.observe(&observers[1]);
        qc_rv.observe(&observers[2]);
        bio_rv.observe(&observers[3]);
        file_rv.observe(&observers[4]);
        owner_rv
            .set_resolver(
                &p_owner,
                hibana::runtime::resolver::ResolverRef::<QUALITY_OUTCOME>::decision_state(
                    &quality_actual,
                    |actual| {
                        use hibana::runtime::resolver::{DecisionArm, ResolverError};
                        let value = actual.take();
                        let decision = match value["success"].as_bool() {
                            Some(true) => Ok(DecisionArm::Left),
                            Some(false) => Ok(DecisionArm::Right),
                            None => Err(ResolverError::reject()),
                        };
                        actual.set(value);
                        decision
                    },
                ),
            )
            .map_err(|error| shell::ShellError::message(format!("{error:?}")))?;
        let mut owner = owner_rv.enter(session, &p_owner)?;
        let fume = owner_rv.enter(session, &p_fume)?;
        let mut spec_owner = spec_rv.enter(session, &p_spec_owner)?;
        let spec = spec_rv.enter(session, &p_spec)?;
        let mut qc = qc_rv.enter(session, &p_qc)?;
        let stock = qc_rv.enter(session, &p_stock)?;
        let mut biologist = bio_rv.enter(session, &p_biologist)?;
        let mut file = file_rv.enter(session, &p_file)?;
        let packet = quality_packet();
        let expected = quality_verdict(approved);
        let domain = Effect {
            result: expected.clone(),
            calls: Cell::new(Vec::new()),
        };
        let execution = hibana_runtime::race(
            async {
                async {
                    if let Some(resource) = substitute {
                        // These are valid, different material bytes sent through
                        // the real resource view, rather than an invalid codec.
                        let material = lab_imp::decode(qc.recv::<BiologyReport>().await?)?;
                        let arguments = qc.recv::<FileQuality>().await?.into_value();
                        let admitted = json!({"material":material,"arguments":arguments});
                        assert_eq!(admitted, packet);
                        let bytes = lab_imp::encode(&admitted)?;
                        async {
                            let [mut fume, mut spec] = qc.par()?;
                            async {
                                fume.send::<QualityAdmission>(&bytes.as_slice()).await?;
                            };
                            async {
                                spec.send::<SpecQualityAdmission>(&bytes.as_slice()).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        let (mut fume, mut spec) = async {
                            let [mut fume, mut spec] = qc.par()?;
                            async { fume.recv::<Loan<FUME_HOOD>>().await };
                            async { spec.recv::<Loan<SPECTROMETER>>().await };
                        }
                        .await?;
                        let mut changed = admitted;
                        changed["material"]["bioassay"]["data"]["cell_type"] =
                            json!("another actual material claim");
                        lab_imp::quality_input(&changed)?;
                        let changed = lab_imp::encode(&changed)?;
                        async {
                            async {
                                fume.send::<FumeQuality>(&if resource == "fume" {
                                    changed.as_slice()
                                } else {
                                    bytes.as_slice()
                                })
                                .await?;
                            };
                            async {
                                spec.send::<SpecQuality>(&if resource == "spectrometer" {
                                    changed.as_slice()
                                } else {
                                    bytes.as_slice()
                                })
                                .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        drop(stock);
                        Ok(Value::Null)
                    } else {
                        async {
                            let endpoint = &mut qc;

                            let material =
                                lab_imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                            let arguments = endpoint.recv::<FileQuality>().await?.into_value();
                            let packet = json!({"arguments":arguments,"material":material});
                            lab_imp::quality_input(&packet)?;
                            let bytes = lab_imp::encode(&packet)?;
                            async {
                                let [mut fume, mut spec] = endpoint.par()?;
                                async {
                                    fume.send::<QualityAdmission>(&bytes.as_slice()).await?;
                                };
                                async {
                                    spec.send::<SpecQualityAdmission>(&bytes.as_slice()).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            let (mut fume, mut spec) = async {
                                let [mut fume, mut spec] = endpoint.par()?;
                                async { fume.recv::<Loan<FUME_HOOD>>().await };
                                async { spec.recv::<Loan<SPECTROMETER>>().await };
                            }
                            .await?;
                            let (from_fume, from_spec) = async {
                                async {
                                    fume.send::<FumeQuality>(&bytes.as_slice()).await?;
                                    let branch = fume.offer().await?;
                                    match branch.label() {
                                        FumeQualityApproved::LOGICAL_LABEL => lab_imp::decode(
                                            branch.recv::<FumeQualityApproved>().await?,
                                        ),
                                        FumeQualityRejected::LOGICAL_LABEL => lab_imp::decode(
                                            branch.recv::<FumeQualityRejected>().await?,
                                        ),
                                        label => Err(ShellError::message(format!(
                                            "unexpected fume QC result {label}"
                                        ))),
                                    }
                                };
                                async {
                                    spec.send::<SpecQuality>(&bytes.as_slice()).await?;
                                    let branch = spec.offer().await?;
                                    match branch.label() {
                                        SpecQualityApproved::LOGICAL_LABEL => lab_imp::decode(
                                            branch.recv::<SpecQualityApproved>().await?,
                                        ),
                                        SpecQualityRejected::LOGICAL_LABEL => lab_imp::decode(
                                            branch.recv::<SpecQualityRejected>().await?,
                                        ),
                                        label => Err(ShellError::message(format!(
                                            "unexpected spectrometer QC result {label}"
                                        ))),
                                    }
                                };
                            }
                            .await?;
                            if from_fume != from_spec {
                                return Err(ShellError::message(
                                    "QC instruments returned different actual decisions",
                                ));
                            }
                            lab_imp::quality_result(&from_fume, &packet)?;
                            async {
                                let [mut fume_owner, mut spec_owner] = endpoint.par()?;
                                async {
                                    fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                                };
                                async {
                                    spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            let bytes = lab_imp::encode(&from_fume)?;
                            if from_fume["success"] == true {
                                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                                endpoint.send::<BiologyApproved>(&reply).await?;
                                endpoint
                                    .send::<FileQualityResult>(&bytes.as_slice())
                                    .await?;
                                drop(stock);
                            } else {
                                endpoint.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                                endpoint.send::<BiologyRejected>(&reply).await?;
                                endpoint
                                    .send::<FileQualityResult>(&bytes.as_slice())
                                    .await?;
                            }
                            Ok(from_fume)
                        }
                        .await
                    }
                };
                async {
                    async {
                        let endpoint = &mut owner;
                        let domain = &domain;
                        let actual_result = &quality_actual;

                        let admitted = lab_imp::decode(endpoint.recv::<QualityAdmission>().await?)?;
                        lab_imp::quality_input(&admitted)?;
                        endpoint.send::<Loan<FUME_HOOD>>(fume).await?;
                        let (from_fume, from_spec) = async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async { lab_imp::decode(fume.recv::<FumeQuality>().await?) };
                            async { lab_imp::decode(spec.recv::<SpecQuality>().await?) };
                        }
                        .await?;
                        if from_fume != admitted || from_spec != admitted {
                            return Err(ShellError::message(
                                "QC resource request differs from admitted material",
                            ));
                        }
                        let actual = domain
                            .execute(
                                "QC_CONTROLLER",
                                "verify_result",
                                admitted["arguments"].clone(),
                            )
                            .await
                            .map_err(ShellError::message)?;
                        lab_imp::quality_result(&actual, &admitted)?;
                        // The resolver reads the actual effect result. par borrows an already
                        // selected branch; it does not decide the scientific outcome itself.
                        actual_result.set(actual);
                        drop(endpoint.offer().await?);
                        let actual = actual_result.take();
                        let bytes = lab_imp::encode(&actual)?;
                        if actual["success"] == true {
                            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                            async {
                                let [mut fume, mut spec] = endpoint.par()?;
                                async {
                                    fume.send::<FumeQualityApproved>(&reply).await?;
                                };
                                async {
                                    spec.send::<SpecQualityApproved>(&reply).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            let fume = endpoint.recv::<Loan<FUME_HOOD>>().await?;
                            Ok((fume, None))
                        } else {
                            let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                            async {
                                let [mut fume, mut spec] = endpoint.par()?;
                                async {
                                    fume.send::<FumeQualityRejected>(&reply).await?;
                                };
                                async {
                                    spec.send::<SpecQualityRejected>(&reply).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            let fume = endpoint.recv::<Loan<FUME_HOOD>>().await?;
                            let stock = endpoint.recv::<Loan<REAGENT_SUPPLY>>().await?;
                            Ok((fume, Some(stock)))
                        }
                    }
                    .await
                };
                async {
                    let admitted =
                        lab_imp::decode(spec_owner.recv::<SpecQualityAdmission>().await?)?;
                    assert_eq!(admitted, packet);
                    spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
                    if substitute.is_some() {
                        Ok(None)
                    } else {
                        Ok(Some(spec_owner.recv::<Loan<SPECTROMETER>>().await?))
                    }
                };
                async {
                    let bytes = lab_imp::encode(&packet["material"])?;
                    biologist.send::<BiologyReport>(&bytes.as_slice()).await?;
                    if substitute.is_none() {
                        let branch = biologist.offer().await?;
                        let actual = match branch.label() {
                            BiologyApproved::LOGICAL_LABEL => {
                                lab_imp::decode(branch.recv::<BiologyApproved>().await?)?
                            }
                            BiologyRejected::LOGICAL_LABEL => {
                                lab_imp::decode(branch.recv::<BiologyRejected>().await?)?
                            }
                            label => panic!("unexpected actual biology verdict {label}"),
                        };
                        assert_eq!(actual, expected);
                    }
                    Ok(())
                };
                async {
                    let bytes = lab_imp::encode(&packet["arguments"])?;
                    let arguments = Arguments::<9>::try_from(bytes.as_slice())?;
                    file.send::<FileQuality>(&arguments).await?;
                    if substitute.is_none() {
                        assert_eq!(
                            lab_imp::decode(file.recv::<FileQualityResult>().await?)?,
                            expected
                        );
                    }
                    Ok(())
                };
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(20)),
        )
        .await;
        let calls = domain.calls.take();
        let checkpoints = [
            owner_rv.tap().checkpoint(),
            spec_rv.tap().checkpoint(),
            qc_rv.tap().checkpoint(),
            bio_rv.tap().checkpoint(),
            file_rv.tap().checkpoint(),
        ];
        let captured = records.each_ref().map(|producer| producer.take());
        if let Err(error) = &execution
            && let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE")
        {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory)?;
            let evidence = json!({
                "scope":"native QC component failure, not a successful qualification",
                "approved":approved,"substituted_resource":substitute,"error":error.to_string(),
                "source":hibana::runtime::tap::projected(&p_owner).map(choreofix::imp::projection::record).collect::<Vec<_>>(),
                "captures":captured.iter().map(|producer|producer.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "checkpoints":checkpoints,"effects":calls
            });
            std::fs::write(
                directory.join(format!("quality-failure-{approved}-{substitute:?}.json")),
                serde_json::to_vec_pretty(&evidence)
                    .map_err(|error| shell::ShellError::message(error.to_string()))?,
            )?;
        }
        if substitute.is_some() {
            let error = match execution {
                Err(error) => error,
                Ok(_) => panic!("substituted material was accepted"),
            };
            // A deadline is Io and a protocol failure is Endpoint. Neither
            // demonstrates rejection by the actual argument/effect boundary.
            assert!(matches!(&error, shell::ShellError::Message(_)), "{error}");
            assert!(calls.is_empty(), "a substituted request reached the effect");
        } else {
            let (actual, (fume, stock), spec, (), ()) = execution?;
            assert_eq!(actual, expected);
            assert_eq!(stock.is_some(), !approved);
            assert!(spec.is_some());
            assert_eq!(
                calls,
                vec![json!({"agent":"QC_CONTROLLER","tool":"verify_result",
                    "arguments":packet["arguments"]})]
            );
            drop((fume, stock, spec));
        }
        let captures: [hibana_epf::Capture; 5] = core::array::from_fn(|producer| {
            hibana_epf::Capture::new(&captured[producer], checkpoints[producer])
                .expect("complete native quality capture")
        });
        let snapshot: Vec<_> = hibana::runtime::tap::projected(&p_owner).collect();
        let observed_producers = observed_producers.take();
        let count = audit_native(&captures, &snapshot, (session.raw(), &observed_producers));
        println!(
            "quality approved={approved} substituted={substitute:?}: {} actual effects, EPF checked {count} complete records",
            calls.len()
        );
        if let Some(directory) = std::env::var_os("CHOREOFIX_LAB_CASCADING_EVIDENCE") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory)?;
            let evidence = json!({
                "scope":"native QC component with controlled actual results; not full 13H/Wasm/model qualification",
                "session":session.raw(),"approved":approved,"substituted_resource":substitute,
                "source":snapshot.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>(),
                "captures":captured.iter().map(|producer|producer.iter().copied().map(choreofix::imp::projection::record).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "checkpoints":checkpoints,"observed_producers":observed_producers,
                "frames":bus.transport.take_observations().collect::<Vec<_>>(),
                "admitted":packet,"actual_result":expected,"effects":calls,
                "epf_checked_records":count,"application_effects_certified_by_epf":false
            });
            std::fs::write(
                directory.join(format!("quality-{approved}-{substitute:?}.json")),
                serde_json::to_vec_pretty(&evidence)
                    .map_err(|error| shell::ShellError::message(error.to_string()))?,
            )?;
        }
    }
    Ok(())
}
