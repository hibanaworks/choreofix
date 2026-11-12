//! Real Endpoint flow and receiver effects, including separate panel requests.
use super::{global::*, localside};
use crate::imp::{json, patient};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};
use std::cell::Cell;
#[cfg(miri)]
#[path = "../handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// Actual fixture effect receipts only; this cannot admit a protocol operation.
struct Effects(Cell<Vec<Value>>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut effects = self.0.take();
        let assessment = effects
            .iter()
            .rev()
            .find(|effect| effect["tool_name"] == "assess_patient")
            .map(|effect| effect["data"]["artifact_id"].clone());
        let tests = arguments["test_type"]
            .as_str()
            .map(|tests| tests.split(',').map(str::trim).collect::<Vec<_>>())
            .unwrap_or_default();
        let result = json!({"success":true,"tool_name":tool,"data":{
            "agent":agent,"patient_id":arguments["patient_id"],"arguments":arguments,
            "artifact_id":format!("{tool}:{}",effects.len()),"tests":tests,
            "inputs":{"assessment":assessment}}});
        effects.push(result.clone());
        self.0.set(effects);
        Ok(result)
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn consultation_preserves_separate_panel_requests_and_actual_ct_custody() -> ShellResult<()> {
    let p = programs();
    for (general_requests, cardiac_requests) in [
        (&["vitals,basic_labs"][..], &["ecg,cardiac_enzymes"][..]),
        (
            &["vitals", "basic_labs"][..],
            &["ecg", "cardiac_enzymes"][..],
        ),
        (
            &["basic_labs", "vitals"][..],
            &["cardiac_enzymes", "ecg"][..],
        ),
        (
            &["vitals", "vitals", "vitals", "basic_labs"][..],
            &[
                "cardiac_enzymes",
                "cardiac_enzymes",
                "cardiac_enzymes",
                "ecg",
            ][..],
        ),
    ] {
        let session = SessionId::new(SESSION + 200 + general_requests.len() as u32);
        // Each binding is one actual object: scanner and two disjoint test panels.
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, CT_SCANNER),
                (session, GENERAL_PANEL),
                (session, CARDIAC_PANEL),
            ],
        );
        let general_input = Cell::new(Value::Null);
        let cardiac_input = Cell::new(Value::Null);
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p.general_service,
            ResolverRef::<GENERAL_COMPLETE>::decision_state(
                &general_input,
                |input: &Cell<Value>| patient::panel_arm(input, &["vitals", "basic_labs"]),
            ),
        )
        .unwrap();
        rv.set_resolver(
            &p.cardiac_service,
            ResolverRef::<CARDIAC_COMPLETE>::decision_state(
                &cardiac_input,
                |input: &Cell<Value>| patient::panel_arm(input, &["ecg", "cardiac_enzymes"]),
            ),
        )
        .unwrap();
        let mut er = rv.enter(session, &p.er)?;
        let mut cardiologist = rv.enter(session, &p.cardiologist)?;
        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut er_io = rv.enter(session, &p.er_io)?;
        let mut cardio_io = rv.enter(session, &p.cardio_io)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let mut general = rv.enter(session, &p.general_service)?;
        let mut cardiac = rv.enter(session, &p.cardiac_service)?;
        let mut pharmacy = rv.enter(session, &p.pharmacy_service)?;
        let mut imaging = rv.enter(session, &p.imaging_service)?;
        let ct = rv.enter(session, &p.ct)?;
        let general_panel = rv.enter(session, &p.general_panel)?;
        let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
        let effects = Effects(Cell::new(Vec::new()));

        hibana_runtime::race(
            async {
                async {
                    localside::services::general(
                        &mut general,
                        (general_panel, &general_input),
                        &effects,
                    )
                    .await?;
                };
                async {
                    localside::services::cardiac(
                        &mut cardiac,
                        (cardiac_panel, &cardiac_input),
                        &effects,
                    )
                    .await?;
                };
                async {
                    localside::services::pharmacy(&mut pharmacy, &effects).await?;
                };
                async {
                    localside::services::imaging(&mut imaging, ct, &effects).await?;
                };
                async {
                    assert_eq!(er_io.recv::<crate::shell::global::Exit>().await?, 0);
                    Ok(())
                };
                async {
                    assert_eq!(cardio_io.recv::<crate::shell::global::Exit>().await?, 0);
                    Ok(())
                };
                async {
                    assert_eq!(pharmacy_io.recv::<crate::shell::global::Exit>().await?, 0);
                    Ok(())
                };
                async {
                    let [mut clinical, mut review, mut private] = er.par().map_err(|error| {
                        ShellError::message(format!("ER clinical/review/private: {error:?}"))
                    })?;
                    async {
                        async {
                            let bytes = json::encode(&json!({"patient_id":"patient_1"}))?;
                            clinical
                                .send::<AssessmentRequest>(&bytes.as_slice())
                                .await?;
                            let assessment =
                                json::decode(clinical.recv::<AssessmentResult>().await?)?;
                            let ((tests, _general_panel), imaging, _cardiac_panel) = async {
                            let [mut tests, mut scan, mut referral] =
                                clinical.par().map_err(|error| {
                                    ShellError::message(format!(
                                        "ER tests/imaging/referral: {error:?}"
                                    ))
                                })?;
                            async {
                                let mut requests = general_requests.iter();
                                loop {
                                    let mut panel = tests.recv::<GeneralPanelLoan>().await?;
                                    let requested =
                                        requests.next().expect("unexpected extra test attempt");
                                    let bytes = json::encode(
                                        &json!({"patient_id":"patient_1","test_type":requested}),
                                    )?;
                                    panel.send::<GeneralTests>(&bytes.as_slice()).await?;
                                    let attempt =
                                        json::decode(panel.recv::<GeneralTestResults>().await?)?;
                                    patient::actual_result(&attempt)?;
                                    tests.send::<GeneralPanelReturn>(panel).await?;
                                    let branch = tests.offer().await?;
                                    match branch.label() {
                                        25 => {
                                            let panel =
                                                branch.recv::<GeneralPanelCompleted>().await?;
                                            let results = json::decode(
                                                tests.recv::<GeneralTestsComplete>().await?,
                                            )?;
                                            assert!(patient::panel_complete(
                                                &results,
                                                &["vitals", "basic_labs"]
                                            )?);
                                            assert!(requests.next().is_none());
                                            break Ok::<_, ShellError>((results, panel));
                                        }
                                        22 => {
                                            let results = json::decode(
                                                branch.recv::<GeneralTestsPartial>().await?,
                                            )?;
                                            assert!(!patient::panel_complete(
                                                &results,
                                                &["vitals", "basic_labs"]
                                            )?);
                                        }
                                        _ => {
                                            return Err(ShellError::message(
                                                "unexpected panel result",
                                            ));
                                        }
                                    }
                                }
                            };
                            async {
                                let bytes = json::encode(
                                    &json!({"patient_id":"patient_1","modality":"CT"}),
                                )?;
                                scan.send::<ImagingIntent>(&bytes.as_slice()).await?;
                                let mut ct = scan.recv::<CtLoan>().await?;
                                ct.send::<ImagingRequest>(&bytes.as_slice()).await?;
                                let result = json::decode(ct.recv::<ImagingResult>().await?)?;
                                scan.send::<CtLoan>(ct).await?;
                                Ok::<_, ShellError>(result)
                            };
                            async {
                                async {
                                    let [mut doctor, mut owner] =
                                        referral.par().map_err(|error| {
                                            ShellError::message(format!(
                                                "ER referral recipients: {error:?}"
                                            ))
                                        })?;
                                    let bytes = json::encode(&assessment)?;
                                    async {
                                        doctor.send::<Referral>(&bytes.as_slice()).await?;
                                    };
                                    async {
                                        owner.send::<ReferralForOwner>(&bytes.as_slice()).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                                referral
                                    .recv::<CardiacPanelCompleted>()
                                    .await
                                    .map_err(ShellError::from)
                            };
                        }
                        .await?;
                            let findings =
                                json!({"assessment":assessment,"tests":tests,"imaging":imaging});
                            async {
                                let [mut doctor, mut owner] = clinical.par().map_err(|error| {
                                    ShellError::message(format!("ER clinical: {error:?}"))
                                })?;
                                let bytes = json::encode(&findings)?;
                                async {
                                    doctor.send::<GeneralFindings>(&bytes.as_slice()).await?;
                                };
                                async {
                                    owner.send::<FindingsForOwner>(&bytes.as_slice()).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            let actual = json::decode(clinical.recv::<CardiacFindings>().await?)?;
                            patient::actual_result(&actual)?;
                            Ok(())
                        };
                        async {
                            let result = json::decode(review.recv::<MedicationReview>().await?)?;
                            patient::actual_result(&result)?;
                            Ok(())
                        };
                        Ok(())
                    }
                    .await?;
                    private.send::<crate::shell::global::Exit>(&0).await?;
                    Ok(())
                };
                async {
                    let [mut task, mut private] = cardiologist.par().map_err(|error| {
                        ShellError::message(format!("cardiologist business/private: {error:?}"))
                    })?;
                    let assessment = json::decode(task.recv::<Referral>().await?)?;
                    patient::actual_result(&assessment)?;
                    let mut requests = cardiac_requests.iter();
                    loop {
                        let mut panel = task.recv::<CardiacPanelLoan>().await?;
                        let requested = requests
                            .next()
                            .expect("unexpected extra cardiac test attempt");
                        let bytes =
                            json::encode(&json!({"patient_id":"patient_1","test_type":requested}))?;
                        panel.send::<CardiacTests>(&bytes.as_slice()).await?;
                        let result = json::decode(panel.recv::<CardiacTestResults>().await?)?;
                        patient::actual_result(&result)?;
                        task.send::<CardiacPanelReturn>(panel).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            32 => {
                                let results =
                                    json::decode(branch.recv::<CardiacTestsComplete>().await?)?;
                                assert!(patient::panel_complete(
                                    &results,
                                    &["ecg", "cardiac_enzymes"]
                                )?);
                                assert!(requests.next().is_none());
                                break;
                            }
                            30 => {
                                let results =
                                    json::decode(branch.recv::<CardiacTestsPartial>().await?)?;
                                assert!(!patient::panel_complete(
                                    &results,
                                    &["ecg", "cardiac_enzymes"]
                                )?);
                            }
                            _ => {
                                return Err(ShellError::message("unexpected cardiac panel result"));
                            }
                        }
                    }
                    let findings = json::decode(task.recv::<GeneralFindings>().await?)?;
                    json::same(&findings["assessment"], &assessment)?;
                    let bytes = json::encode(
                        &json!({"patient_id":"patient_1","symptoms":"actual caller symptoms"}),
                    )?;
                    task.send::<DiagnosisRequest>(&bytes.as_slice()).await?;
                    let diagnosis = json::decode(task.recv::<DiagnosisResult>().await?)?;
                    let bytes = json::encode(&diagnosis)?;
                    task.send::<CardiacFindings>(&bytes.as_slice()).await?;
                    private.send::<crate::shell::global::Exit>(&0).await?;
                    Ok(())
                };
                async {
                    let [mut task, mut private] = pharmacist.par().map_err(|error| {
                        ShellError::message(format!("pharmacist business/private: {error:?}"))
                    })?;
                    let arguments =
                        json!({"patient_id":"patient_1","medications":"caller supplied list"});
                    let bytes = json::encode(&arguments)?;
                    task.send::<ReviewRequest>(&bytes.as_slice()).await?;
                    let result = json::decode(task.recv::<ReviewResult>().await?)?;
                    json::same(&result["data"]["arguments"], &arguments)?;
                    let bytes = json::encode(&result)?;
                    task.send::<MedicationReview>(&bytes.as_slice()).await?;
                    private.send::<crate::shell::global::Exit>(&0).await?;
                    Ok(())
                };
                Ok(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                30
            })),
        )
        .await?;
        assert_eq!(
            effects.0.take().len(),
            4 + general_requests.len() + cardiac_requests.len()
        );
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn imaging_compares_the_actual_resource_request_before_its_effect() -> ShellResult<()> {
    let p = programs();
    for (admitted, forwarded, expected) in [
        (
            json!({"patient_id":"patient_1","modality":"MRI"}),
            json!({"patient_id":"patient_1","modality":"CT"}),
            0,
        ),
        (
            json!({"patient_id":"patient_1","modality":"CT"}),
            json!({"patient_id":"patient_other","modality":"CT"}),
            0,
        ),
        (
            json!({"patient_id":"patient_1","modality":"CT"}),
            json!({"patient_id":"patient_1","modality":"CT"}),
            1,
        ),
    ] {
        let session = SessionId::new(SESSION + 300);
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, CT_SCANNER),
                (session, GENERAL_PANEL),
                (session, CARDIAC_PANEL),
            ],
        );
        let general_input = Cell::new(Value::Null);
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p.general_service,
            ResolverRef::<GENERAL_COMPLETE>::decision_state(
                &general_input,
                |input: &Cell<Value>| patient::panel_arm(input, &["vitals", "basic_labs"]),
            ),
        )
        .unwrap();
        let mut doctor = rv.enter(session, &p.er)?;
        let mut general = rv.enter(session, &p.general_service)?;
        let mut imaging = rv.enter(session, &p.imaging_service)?;
        let ct = rv.enter(session, &p.ct)?;
        let panel = rv.enter(session, &p.general_panel)?;
        let effects = Effects(Cell::new(Vec::new()));
        let result: ShellResult<()> = hibana_runtime::race(
            async {
                async {
                    localside::services::general(&mut general, (panel, &general_input), &effects)
                        .await?;
                };
                async {
                    localside::services::imaging(&mut imaging, ct, &effects).await?;
                };
                async {
                    let [mut clinical, _review, _private] = doctor.par()?;
                    let bytes = json::encode(&json!({"patient_id":"patient_1"}))?;
                    clinical
                        .send::<AssessmentRequest>(&bytes.as_slice())
                        .await?;
                    clinical.recv::<AssessmentResult>().await?;
                    let [_tests, mut scan, _referral] = clinical.par()?;
                    let bytes = json::encode(&admitted)?;
                    scan.send::<ImagingIntent>(&bytes.as_slice()).await?;
                    let mut ct = scan.recv::<CtLoan>().await?;
                    let bytes = json::encode(&forwarded)?;
                    ct.send::<ImagingRequest>(&bytes.as_slice()).await?;
                    let result = json::decode(ct.recv::<ImagingResult>().await?)?;
                    json::same(&result["data"]["arguments"], &admitted)?;
                    scan.send::<CtLoan>(ct).await?;
                    // End this measured prefix through normal scoped cancellation.
                    // This test does not claim whole-protocol completion.
                    Err(ShellError::message("recorded CT prefix"))
                };
                Ok(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                30
            })),
        )
        .await;
        let records = effects.0.take();
        assert_eq!(
            records
                .iter()
                .filter(|record| record["tool_name"] == "perform_imaging")
                .count(),
            expected,
            "{result:?}"
        );
        assert!(result.as_ref().is_err_and(|error|matches!(error,ShellError::Message(message)
            if message.contains("expected CT") || message.contains("differs from") || message=="recorded CT prefix")),"{result:?}");
        assert_eq!(
            records
                .iter()
                .filter(|record| record["tool_name"] == "assess_patient")
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn panel_receipts_retain_current_data_and_reject_partial_updates() {
    let receipt = |identity: &str, tests: Value, assessment: &str| {
        json!({"success":true,"data":{
        "patient_id":"patient_1","artifact_id":identity,"tests":tests,
        "inputs":{"assessment":assessment}}})
    };
    let mut panel = Value::Null;
    for attempt in 0..200 {
        patient::record_panel(
            &mut panel,
            &receipt(&attempt.to_string(), json!(["vitals"]), "assessment:0"),
            &["vitals", "basic_labs"],
        )
        .unwrap();
        assert_eq!(panel.as_object().unwrap().len(), 1);
    }
    assert!(!patient::panel_complete(&panel, &["vitals", "basic_labs"]).unwrap());
    patient::record_panel(
        &mut panel,
        &receipt("basic:0", json!(["basic_labs"]), "assessment:0"),
        &["vitals", "basic_labs"],
    )
    .unwrap();
    assert_eq!(panel["vitals"]["data"]["artifact_id"], "199");
    assert!(patient::panel_complete(&panel, &["vitals", "basic_labs"]).unwrap());
    assert_eq!(panel.as_object().unwrap().len(), 2);
    let previous = panel.clone();
    assert!(
        patient::record_panel(
            &mut panel,
            &receipt(
                "foreign:0",
                json!(["vitals", "unknown_test"]),
                "assessment:0"
            ),
            &["vitals", "basic_labs"]
        )
        .is_err()
    );
    assert_eq!(panel, previous);
    assert!(
        patient::record_panel(
            &mut panel,
            &receipt("stale:0", json!(["vitals"]), "another_assessment"),
            &["vitals", "basic_labs"]
        )
        .is_err()
    );
    assert_eq!(panel, previous);
}
