//! Production receivers, all 90 imaging orders, patient progress and altered resource requests.
use super::{global::*, imp, localside};
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

/// Immutable actual-call journal owned by the fixture environment, not permissions.
struct Effects(Cell<Vec<Value>>, Cell<Vec<Value>>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.1.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.1.set(calls);
        let mut effects = self.0.take();
        let result = (|| {
            let patient = arguments["patient_id"].as_str().ok_or("missing patient")?;
            let latest = |tool: &str, modality: Option<&str>| -> Result<&Value, String> {
                effects
                    .iter()
                    .rev()
                    .find(|result| {
                        result["tool_name"] == tool
                            && result["data"]["patient_id"] == patient
                            && modality.is_none_or(|m| result["data"]["modality"] == m)
                    })
                    .ok_or_else(|| format!("missing actual {tool} for {patient}"))
            };
            let mut inputs = json!({});
            if !["assess_patient", "review_prescriptions"].contains(&tool) {
                inputs["assessment"] =
                    latest("assess_patient", None)?["data"]["artifact_id"].clone();
            }
            if tool == "interpret_imaging" {
                inputs["image"] = latest("perform_imaging", arguments["modality"].as_str())?["data"]["artifact_id"].clone();
            }
            if tool == "diagnose" {
                let expected: &[&str] = if patient == "patient_a" {
                    &["trauma_panel", "blood_type", "coag"]
                } else {
                    &["stroke_panel", "ct_angio"]
                };
                for test in expected {
                    let receipt = effects
                        .iter()
                        .rev()
                        .find(|result| {
                            result["tool_name"] == "order_tests"
                                && result["data"]["patient_id"] == patient
                                && result["data"]["tests"]
                                    .as_array()
                                    .is_some_and(|tests| tests.iter().any(|name| name == test))
                        })
                        .ok_or_else(|| format!("missing actual test {test}"))?;
                    inputs[format!("test:{test}")] = receipt["data"]["artifact_id"].clone();
                }
                for modality in if patient == "patient_a" {
                    &["CT"][..]
                } else {
                    &["CT", "MRI"][..]
                } {
                    inputs[format!("imaging:{modality}")] =
                        latest("interpret_imaging", Some(modality))?["data"]["artifact_id"].clone();
                }
            }
            if ["prepare_anesthesia", "perform_surgery", "manage_icu"].contains(&tool) {
                let actor = if patient == "patient_a" {
                    "ER_DOCTOR_A"
                } else {
                    "ER_DOCTOR_B"
                };
                inputs[format!("diagnosis:{actor}")] =
                    latest("diagnose", None)?["data"]["artifact_id"].clone();
            }
            if tool == "perform_surgery" {
                inputs["anesthesia"] =
                    latest("prepare_anesthesia", None)?["data"]["artifact_id"].clone();
            }
            let tests = arguments["test_type"]
                .as_str()
                .map(|v| v.split(',').map(str::trim).collect::<Vec<_>>())
                .unwrap_or_default();
            Ok(
                json!({"success":true,"tool_name":tool,"data":{"patient_id":patient,
                "agent":agent,"artifact_id":format!("{tool}:{}",effects.len()),"inputs":inputs,
                "tests":tests,"modality":arguments["modality"],"arguments":arguments}}),
            )
        })();
        if let Ok(result) = &result {
            effects.push(result.clone());
        }
        self.0.set(effects);
        result
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn all_imaging_orders_and_either_patient_finishing_first() -> ShellResult<()> {
    let p = programs();
    let mut cases = Vec::new();
    for ct in [
        [0, 1, 2, 3],
        [0, 2, 1, 3],
        [0, 2, 3, 1],
        [2, 3, 0, 1],
        [2, 0, 3, 1],
        [2, 0, 1, 3],
    ] {
        for first_mri in 0..5 {
            for second_mri in first_mri + 1..6 {
                let mut ct = ct.into_iter();
                let mut order = [0; 6];
                for (index, value) in order.iter_mut().enumerate() {
                    *value = if index == first_mri {
                        4
                    } else if index == second_mri {
                        5
                    } else {
                        ct.next().expect("four CT positions")
                    };
                }
                cases.push((order, 0));
            }
        }
    }
    assert_eq!(cases.len(), 90);
    // Delay one patient's assessment until the other patient's actual care effect.
    cases.push(([0, 1, 2, 3, 4, 5], 1));
    cases.push(([2, 3, 4, 5, 0, 1], 2));
    for (case, (order, delayed)) in cases.into_iter().enumerate() {
        let session = SessionId::new(SESSION + 200);
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, PANEL_A),
                (session, PANEL_B),
                (session, CT_SCANNER),
                (session, MRI),
                (session, OPERATING_ROOM),
                (session, ICU_BEDS),
                (session, VENTILATOR),
            ],
        );
        let actual_a = Cell::new(Value::Null);
        let actual_b = Cell::new(Value::Null);
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p.patient_a,
            ResolverRef::<PANEL_A_COMPLETE>::decision_state(&actual_a, |value: &Cell<Value>| {
                imp::panel_arm(
                    value,
                    ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                )
            }),
        )
        .unwrap();
        rv.set_resolver(
            &p.patient_b,
            ResolverRef::<PANEL_B_COMPLETE>::decision_state(&actual_b, |value: &Cell<Value>| {
                imp::panel_arm(value, ("patient_b", &["stroke_panel", "ct_angio"]))
            }),
        )
        .unwrap();
        let mut patient_a = rv.enter(session, &p.patient_a)?;
        let mut patient_b = rv.enter(session, &p.patient_b)?;
        let panel_a = rv.enter(session, &p.panel_a)?;
        let panel_b = rv.enter(session, &p.panel_b)?;
        let mut ct_service = rv.enter(session, &p.ct_service)?;
        let ct = rv.enter(session, &p.ct)?;
        let mut mri_service = rv.enter(session, &p.mri_service)?;
        let mri = rv.enter(session, &p.mri)?;
        let mut anesthesia = rv.enter(session, &p.anesthesia)?;
        let room = rv.enter(session, &p.room)?;
        let mut surgery = rv.enter(session, &p.surgery)?;
        let mut icu = rv.enter(session, &p.icu)?;
        let bed = rv.enter(session, &p.bed)?;
        let ventilator = rv.enter(session, &p.ventilator)?;
        let mut pharmacy = rv.enter(session, &p.pharmacy)?;
        let mut er_a = rv.enter(session, &p.er_a)?;
        let mut er_b = rv.enter(session, &p.er_b)?;
        let mut radiologist = rv.enter(session, &p.radiologist)?;
        let mut anesthesiologist = rv.enter(session, &p.anesthesiologist)?;
        let mut surgeon = rv.enter(session, &p.surgeon)?;
        let mut icu_coordinator = rv.enter(session, &p.icu_coordinator)?;
        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut er_a_io = rv.enter(session, &p.er_a_io)?;
        let mut er_b_io = rv.enter(session, &p.er_b_io)?;
        let mut radiology_io = rv.enter(session, &p.radiology_io)?;
        let mut surgery_io = rv.enter(session, &p.surgery_io)?;
        let mut anesthesia_io = rv.enter(session, &p.anesthesia_io)?;
        let mut icu_io = rv.enter(session, &p.icu_io)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let result = hibana_runtime::race(async {
            async { localside::patients::patient_a(&mut patient_a,(panel_a,&actual_a),&effects).await.map_err(|error|ShellError::message(format!("patient_a owner: {error:?}")))?; };
            async { localside::patients::patient_b(&mut patient_b,(panel_b,&actual_b),&effects).await.map_err(|error|ShellError::message(format!("patient_b owner: {error:?}")))?; };
            async { localside::imaging::ct(&mut ct_service,ct,&effects).await.map_err(|error|ShellError::message(format!("CT owner: {error:?}")))?; };
            async { localside::imaging::mri(&mut mri_service,mri,&effects).await.map_err(|error|ShellError::message(format!("MRI owner: {error:?}")))?; };
            async { localside::care::anesthesia(&mut anesthesia,room,&effects).await.map_err(|error|ShellError::message(format!("anesthesia owner: {error:?}")))?; };
            async { localside::care::surgery(&mut surgery,&effects).await.map_err(|error|ShellError::message(format!("surgery owner: {error:?}")))?; };
            async { localside::care::icu(&mut icu,(bed,ventilator),&effects).await.map_err(|error|ShellError::message(format!("ICU owner: {error:?}")))?; };
            async { localside::care::pharmacy(&mut pharmacy,&effects).await.map_err(|error|ShellError::message(format!("pharmacy owner: {error:?}")))?; };
            async { assert_eq!(er_a_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(er_b_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(radiology_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(surgery_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(anesthesia_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(icu_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async { assert_eq!(pharmacy_io.recv::<crate::shell::global::Exit>().await?,0); Ok::<(),ShellError>(()) };
            async {
                let [mut clinical, mut review, mut private] = er_a.par().map_err(|error|ShellError::message(format!("ER A initial split: {error:?}")))?;
                async {
                async {
                    let result = imp::decode(review.recv::<ReviewForA>().await?)?;
                    imp::actual_result(&result,"patient_A_and_B")?;
                };
                async {
                    if delayed == 2 {
                        core::future::poll_fn(|cx| {
                            let journal = effects.0.take();
                            let completed = journal.iter().any(|r| r["tool_name"] == "manage_icu");
                            effects.0.set(journal);
                            if completed { core::task::Poll::Ready(()) }
                            else { cx.waker().wake_by_ref(); core::task::Poll::Pending }
                        }).await;
                    }
                    let bytes = imp::encode(&json!({"patient_id":"patient_a"}))?;
                    clinical.send::<AssessmentRequestA>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(clinical.recv::<AssessmentResultA>().await?)?,"patient_a")?;
                    let mut panel = clinical.recv::<PanelLoan<PANEL_A>>().await?;
                    let bytes = imp::encode(&json!({"patient_id":"patient_a","test_type":"trauma_panel,blood_type,coag"}))?;
                    panel.send::<TestRequest>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(panel.recv::<TestResult>().await?)?,"patient_a")?;
                    clinical.send::<PanelReturn<PANEL_A>>(panel).await?;
                    let _panel = clinical.recv::<PanelCompleted<PANEL_A>>().await?;
                    let _ = imp::decode(clinical.recv::<PanelComplete>().await?)?;
                    let findings = imp::decode(clinical.recv::<FindingsA>().await?)?;
                    imp::actual_result(&findings["ct"],"patient_a")?;
                    let bytes = imp::encode(&json!({"patient_id":"patient_a","symptoms":"caller observation"}))?;
                    clinical.send::<DiagnosisRequestA>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(clinical.recv::<DiagnosisResultA>().await?)?,"patient_a")?;
                    imp::actual_result(&imp::decode(clinical.recv::<SurgeryForER>().await?)?,"patient_a")?;
                };
                    Ok::<(),ShellError>(())
                }.await?;
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut clinical, mut review, mut private] = er_b.par().map_err(|error|ShellError::message(format!("ER B initial split: {error:?}")))?;
                async {
                async {
                    let result = imp::decode(review.recv::<ReviewForB>().await?)?;
                    imp::actual_result(&result,"patient_A_and_B")?;
                };
                async {
                    if delayed == 1 {
                        core::future::poll_fn(|cx| {
                            let journal = effects.0.take();
                            let completed = journal.iter().any(|r| r["tool_name"] == "perform_surgery");
                            effects.0.set(journal);
                            if completed { core::task::Poll::Ready(()) }
                            else { cx.waker().wake_by_ref(); core::task::Poll::Pending }
                        }).await;
                    }
                    let bytes = imp::encode(&json!({"patient_id":"patient_b"}))?;
                    clinical.send::<AssessmentRequestB>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(clinical.recv::<AssessmentResultB>().await?)?,"patient_b")?;
                    let mut panel = clinical.recv::<PanelLoan<PANEL_B>>().await?;
                    let bytes = imp::encode(&json!({"patient_id":"patient_b","test_type":"stroke_panel,ct_angio"}))?;
                    panel.send::<TestRequest>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(panel.recv::<TestResult>().await?)?,"patient_b")?;
                    clinical.send::<PanelReturn<PANEL_B>>(panel).await?;
                    let _panel = clinical.recv::<PanelCompleted<PANEL_B>>().await?;
                    let _ = imp::decode(clinical.recv::<PanelComplete>().await?)?;
                    let findings = imp::decode(clinical.recv::<FindingsB>().await?)?;
                    imp::actual_result(&findings["ct"],"patient_b")?;
                    let bytes = imp::encode(&json!({"patient_id":"patient_b","symptoms":"caller observation"}))?;
                    clinical.send::<DiagnosisRequestB>(&bytes.as_slice()).await?;
                    imp::actual_result(&imp::decode(clinical.recv::<DiagnosisResultB>().await?)?,"patient_b")?;
                    imp::actual_result(&imp::decode(clinical.recv::<IcuForER>().await?)?,"patient_b")?;
                };
                    Ok::<(),ShellError>(())
                }.await?;
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut task, mut private] = radiologist.par()?;
                async {
                    let [mut ct,mut mri] = task.par()?;
                for operation in order {
                    match operation {
                        0 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_a","modality":"CT"}))?;
                            ct.send::<CtPerformIntentA>(&bytes.as_slice()).await?;
                            let mut instrument = ct.recv::<CtLoan>().await?;
                            instrument.send::<CtPerformRequestA>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<CtImageA>().await?)?,"patient_a")?;
                            ct.send::<CtLoan>(instrument).await?;
                        }
                        1 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_a","modality":"CT"}))?;
                            ct.send::<CtInterpretIntentA>(&bytes.as_slice()).await?;
                            let mut instrument = ct.recv::<CtLoan>().await?;
                            instrument.send::<CtInterpretRequestA>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<CtReportA>().await?)?,"patient_a")?;
                            ct.send::<CtLoan>(instrument).await?;
                        }
                        2 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_b","modality":"CT"}))?;
                            ct.send::<CtPerformIntentB>(&bytes.as_slice()).await?;
                            let mut instrument = ct.recv::<CtLoan>().await?;
                            instrument.send::<CtPerformRequestB>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<CtImageB>().await?)?,"patient_b")?;
                            ct.send::<CtLoan>(instrument).await?;
                        }
                        3 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_b","modality":"CT"}))?;
                            ct.send::<CtInterpretIntentB>(&bytes.as_slice()).await?;
                            let mut instrument = ct.recv::<CtLoan>().await?;
                            instrument.send::<CtInterpretRequestB>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<CtReportB>().await?)?,"patient_b")?;
                            ct.send::<CtLoan>(instrument).await?;
                        }
                        4 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_b","modality":"MRI"}))?;
                            mri.send::<MriPerformIntent>(&bytes.as_slice()).await?;
                            let mut instrument = mri.recv::<MriLoan>().await?;
                            instrument.send::<MriPerformRequest>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<MriImage>().await?)?,"patient_b")?;
                            mri.send::<MriLoan>(instrument).await?;
                        }
                        5 => {
                            let bytes = imp::encode(&json!({"patient_id":"patient_b","modality":"MRI"}))?;
                            mri.send::<MriInterpretIntent>(&bytes.as_slice()).await?;
                            let mut instrument = mri.recv::<MriLoan>().await?;
                            instrument.send::<MriInterpretRequest>(&bytes.as_slice()).await?;
                            imp::actual_result(&imp::decode(instrument.recv::<MriReport>().await?)?,"patient_b")?;
                            mri.send::<MriLoan>(instrument).await?;
                        }
                        _ => unreachable!(),
                    }
                }
                    Ok::<(),ShellError>(())
                }.await?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut task, mut private] = anesthesiologist.par()?;
                let bytes = imp::encode(&json!({"patient_id":"patient_a","type":"caller anesthesia"}))?;
                task.send::<AnesthesiaIntent>(&bytes.as_slice()).await?;
                let mut room = task.recv::<AnesthesiaLoan>().await?;
                room.send::<AnesthesiaRequest>(&bytes.as_slice()).await?;
                imp::actual_result(&imp::decode(room.recv::<AnesthesiaResult>().await?)?,"patient_a")?;
                task.send::<AnesthesiaLoan>(room).await?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut task, mut private] = surgeon.par()?;
                let bytes = imp::encode(&json!({"patient_id":"patient_a","procedure":"trauma_surgery"}))?;
                task.send::<SurgeryIntent>(&bytes.as_slice()).await?;
                let mut room = task.recv::<SurgeryLoan>().await?;
                room.send::<SurgeryRequest>(&bytes.as_slice()).await?;
                imp::actual_result(&imp::decode(room.recv::<SurgeryResult>().await?)?,"patient_a")?;
                task.send::<SurgeryLoan>(room).await?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut task, mut private] = icu_coordinator.par()?;
                let bytes = imp::encode(&json!({"patient_id":"patient_b"}))?;
                task.send::<IcuIntent>(&bytes.as_slice()).await?;
                let (mut bed, mut ventilator) = async {
                    let [mut bed, mut vent] = task.par().map_err(|error|ShellError::message(format!("ICU carrier split: {error:?}")))?;
                    async { bed.recv::<ResourceLoan<ICU_BEDS>>().await };
                    async { vent.recv::<ResourceLoan<VENTILATOR>>().await };
                }.await?;
                async {
                    async { bed.send::<BedRequest>(&bytes.as_slice()).await?; };
                    async { ventilator.send::<VentilatorRequest>(&bytes.as_slice()).await?; };
                    Ok::<(),ShellError>(())
                }.await?;
                let (bed_result, vent_result) = async {
                    async { imp::decode(bed.recv::<BedResult>().await?) };
                    async { imp::decode(ventilator.recv::<VentilatorResult>().await?) };
                }.await?;
                imp::same(&bed_result,&vent_result)?;
                async {
                    let [mut bed_branch, mut vent_branch] = task.par().map_err(|error|ShellError::message(format!("ICU carrier split: {error:?}")))?;
                    async { bed_branch.send::<ResourceReturn<ICU_BEDS>>(bed).await?; };
                    async { vent_branch.send::<ResourceReturn<VENTILATOR>>(ventilator).await?; };
                    Ok::<(),ShellError>(())
                }.await?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            async {
                let [mut task, mut private] = pharmacist.par()?;
                let bytes = imp::encode(&json!({"patient_id":"patient_A_and_B","medications":"caller medication review"}))?;
                task.send::<ReviewRequest>(&bytes.as_slice()).await?;
                imp::actual_result(&imp::decode(task.recv::<ReviewResult>().await?)?,"patient_A_and_B")?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
            };
            Ok::<_,ShellError>(())
        }, deadline(std::time::Duration::from_secs(5))).await;
        result.map_err(|error| {
            ShellError::message(format!(
                "case {case}, order {order:?}, delayed {delayed}: {error:?}"
            ))
        })?;
        let journal = effects.0.take();
        assert_eq!(journal.len(), 16, "case {case}");
        if delayed == 1 {
            assert!(
                journal
                    .iter()
                    .position(|r| r["tool_name"] == "perform_surgery")
                    .unwrap()
                    < journal
                        .iter()
                        .position(|r| r["tool_name"] == "assess_patient"
                            && r["data"]["patient_id"] == "patient_b")
                        .unwrap()
            );
        }
        if delayed == 2 {
            assert!(
                journal
                    .iter()
                    .position(|r| r["tool_name"] == "manage_icu")
                    .unwrap()
                    < journal
                        .iter()
                        .position(|r| r["tool_name"] == "assess_patient"
                            && r["data"]["patient_id"] == "patient_a")
                        .unwrap()
            );
        }
    }
    Ok::<(), ShellError>(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_ct_receiver_rejects_changed_arguments_before_the_effect() -> ShellResult<()> {
    // Use the actual production image: offer() needs its real intrinsic choice,
    // and that choice cannot be replaced by a plain receive or an incomplete arm.
    let p = programs();
    for changed in [
        json!({"patient_id":"patient_b","modality":"CT"}),
        json!({"patient_id":"patient_a","modality":"MRI"}),
    ] {
        let sid = SessionId::new(SESSION + 101);
        let (bus, _, _) = Bus::open([], [(sid, CT_SCANNER), (sid, PANEL_A)]);
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let actual_a = Cell::new(Value::Null);
        let mut owner_slab = vec![0; 256 * 1024];
        let mut owner_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut client_slab = vec![0; 256 * 1024];
        let mut client_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let client_rv = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        client_rv
            .set_resolver(
                &p.patient_a,
                ResolverRef::<PANEL_A_COMPLETE>::decision_state(
                    &actual_a,
                    |value: &Cell<Value>| {
                        imp::panel_arm(
                            value,
                            ("patient_a", &["trauma_panel", "blood_type", "coag"]),
                        )
                    },
                ),
            )
            .unwrap();
        let mut owner = owner_rv.enter(sid, &p.ct_service)?;
        let resource = owner_rv.enter(sid, &p.ct)?;
        let mut donor = client_rv.enter(sid, &p.patient_a)?;
        let _panel = client_rv.enter(sid, &p.panel_a)?;
        let mut doctor = client_rv.enter(sid, &p.er_a)?;
        let mut caller = client_rv.enter(sid, &p.radiologist)?;
        hibana_runtime::race(
            async {
                async {
                    let result = localside::imaging::ct(&mut owner, resource, &effects).await;
                    assert!(
                        matches!(result,Err(ShellError::Message(ref message))
                    if message == "request differs from its actual admitted data"),
                        "{result:?}"
                    );
                    Ok::<(), ShellError>(())
                };
                async {
                    let request = imp::encode(&json!({"patient_id":"patient_a"}))?;
                    doctor
                        .send::<AssessmentRequestA>(&request.as_slice())
                        .await?;
                    imp::same(
                        &imp::decode(donor.recv::<AssessmentRequestA>().await?)?,
                        &json!({"patient_id":"patient_a"}),
                    )?;
                    let assessment = json!({"success":true,
                    "data":{"patient_id":"patient_a","artifact_id":"actual assessment"}});
                    let bytes = imp::encode(&assessment)?;
                    donor.send::<AssessmentResultA>(&bytes.as_slice()).await?;
                    imp::same(
                        &imp::decode(doctor.recv::<AssessmentResultA>().await?)?,
                        &assessment,
                    )?;
                    let admitted = json!({"patient_id":"patient_a","modality":"CT"});
                    let bytes = imp::encode(&admitted)?;
                    caller.send::<CtPerformIntentA>(&bytes.as_slice()).await?;
                    imp::same(
                        &imp::decode(donor.recv::<CtAssessmentQueryA>().await?)?,
                        &admitted,
                    )?;
                    let bytes = imp::encode(&assessment)?;
                    donor.send::<CtAssessmentA>(&bytes.as_slice()).await?;
                    let mut resource = caller.recv::<CtLoan>().await?;
                    let changed = imp::encode(&changed)?;
                    resource
                        .send::<CtPerformRequestA>(&changed.as_slice())
                        .await?;
                    // The runtime keeps this owned result alive until its sibling
                    // has checked the receiver's rejection.
                    Ok::<_, ShellError>(resource)
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                15
            })),
        )
        .await?;
        assert!(effects.0.take().is_empty());
        assert!(effects.1.take().is_empty(), "Domain was invoked");
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_mri_receiver_rejects_changed_arguments_before_the_effect() -> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<RADIOLOGIST, MRI_SERVICE, MriPerformIntent>(),
        g::seq(
            g::send::<MRI_SERVICE, PATIENT_B_SERVICE, MriAssessmentQuery>(),
            g::seq(
                g::send::<PATIENT_B_SERVICE, MRI_SERVICE, MriAssessment>(),
                g::seq(
                    g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                    g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
                ),
            ),
        ),
    );
    let owner_program = project::<MRI_SERVICE>(&source);
    let donor_program = project::<PATIENT_B_SERVICE>(&source);
    let caller_program = project::<RADIOLOGIST>(&source);
    let resource_program = project::<MRI>(&source);
    for changed in [
        json!({"patient_id":"patient_a","modality":"MRI"}),
        json!({"patient_id":"patient_b","modality":"CT"}),
    ] {
        let sid = SessionId::new(SESSION + 101);
        let (bus, _, _) = Bus::open([], [(sid, MRI)]);
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let mut owner_slab = vec![0; 128 * 1024];
        let mut owner_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut client_slab = vec![0; 128 * 1024];
        let mut client_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let client_rv = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let mut owner = owner_rv.enter(sid, &owner_program)?;
        let resource = owner_rv.enter(sid, &resource_program)?;
        let mut donor = client_rv.enter(sid, &donor_program)?;
        let mut caller = client_rv.enter(sid, &caller_program)?;
        hibana_runtime::race(
            async {
                async {
                    let result = localside::imaging::mri(&mut owner, resource, &effects).await;
                    assert!(
                        matches!(result,Err(ShellError::Message(ref message))
                    if message == "request differs from its actual admitted data"),
                        "{result:?}"
                    );
                    Ok::<(), ShellError>(())
                };
                async {
                    let admitted = json!({"patient_id":"patient_b","modality":"MRI"});
                    let bytes = imp::encode(&admitted)?;
                    caller.send::<MriPerformIntent>(&bytes.as_slice()).await?;
                    imp::same(
                        &imp::decode(donor.recv::<MriAssessmentQuery>().await?)?,
                        &admitted,
                    )?;
                    let assessment = imp::encode(&json!({"success":true,
                    "data":{"patient_id":"patient_b","artifact_id":"actual assessment"}}))?;
                    donor.send::<MriAssessment>(&assessment.as_slice()).await?;
                    let mut resource = caller.recv::<MriLoan>().await?;
                    let changed = imp::encode(&changed)?;
                    resource
                        .send::<MriPerformRequest>(&changed.as_slice())
                        .await?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                15
            })),
        )
        .await?;
        assert!(effects.0.take().is_empty());
        assert!(effects.1.take().is_empty(), "Domain was invoked");
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_anesthesia_receiver_rejects_changed_room_arguments_before_the_effect()
-> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<PATIENT_A_SERVICE, ANESTHESIA_SERVICE, DiagnosisForAnesthesia>(),
        g::seq(
            g::send::<ANESTHESIOLOGIST, ANESTHESIA_SERVICE, AnesthesiaIntent>(),
            g::seq(
                g::send::<ANESTHESIA_SERVICE, ANESTHESIOLOGIST, AnesthesiaLoan>(),
                g::send::<OPERATING_ROOM, ANESTHESIA_SERVICE, AnesthesiaRequest>(),
            ),
        ),
    );
    let owner_program = project::<ANESTHESIA_SERVICE>(&source);
    let donor_program = project::<PATIENT_A_SERVICE>(&source);
    let caller_program = project::<ANESTHESIOLOGIST>(&source);
    let room_program = project::<OPERATING_ROOM>(&source);
    for changed in [
        json!({"patient_id":"patient_b","type":"admitted"}),
        json!({"patient_id":"patient_a","type":"substituted"}),
    ] {
        let sid = SessionId::new(SESSION + 102);
        let (bus, _, _) = Bus::open([], [(sid, OPERATING_ROOM)]);
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let mut owner_slab = vec![0; 128 * 1024];
        let mut owner_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut client_slab = vec![0; 128 * 1024];
        let mut client_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let client_rv = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let mut owner = owner_rv.enter(sid, &owner_program)?;
        let room = owner_rv.enter(sid, &room_program)?;
        let mut donor = client_rv.enter(sid, &donor_program)?;
        let mut caller = client_rv.enter(sid, &caller_program)?;
        hibana_runtime::race(
            async {
                async {
                    let result = localside::care::anesthesia(&mut owner, room, &effects).await;
                    assert!(
                        matches!(result,Err(ShellError::Message(ref message))
                    if message == "request differs from its actual admitted data"),
                        "{result:?}"
                    );
                    Ok::<(), ShellError>(())
                };
                async {
                    let diagnosis = imp::encode(&json!({"result":{"success":true,
                    "data":{"patient_id":"patient_a","artifact_id":"actual diagnosis"}}}))?;
                    donor
                        .send::<DiagnosisForAnesthesia>(&diagnosis.as_slice())
                        .await?;
                    let admitted =
                        imp::encode(&json!({"patient_id":"patient_a","type":"admitted"}))?;
                    caller
                        .send::<AnesthesiaIntent>(&admitted.as_slice())
                        .await?;
                    let mut room = caller.recv::<AnesthesiaLoan>().await?;
                    let changed = imp::encode(&changed)?;
                    room.send::<AnesthesiaRequest>(&changed.as_slice()).await?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                15
            })),
        )
        .await?;
        assert!(effects.0.take().is_empty());
        assert!(effects.1.take().is_empty(), "Domain was invoked");
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_surgery_receiver_rejects_changed_room_arguments_before_the_effect()
-> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<PATIENT_A_SERVICE, SURGERY_SERVICE, DiagnosisForSurgery>(),
        g::seq(
            g::par(
                g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparedRoom>(),
                g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparationForSurgery>(),
            ),
            g::seq(
                g::send::<SURGEON, SURGERY_SERVICE, SurgeryIntent>(),
                g::seq(
                    g::send::<SURGERY_SERVICE, SURGEON, SurgeryLoan>(),
                    g::send::<OPERATING_ROOM, SURGERY_SERVICE, SurgeryRequest>(),
                ),
            ),
        ),
    );
    let owner_program = project::<SURGERY_SERVICE>(&source);
    let donor_program = project::<PATIENT_A_SERVICE>(&source);
    let anesthesia_program = project::<ANESTHESIA_SERVICE>(&source);
    let caller_program = project::<SURGEON>(&source);
    let room_program = project::<OPERATING_ROOM>(&source);
    for changed in [
        json!({"patient_id":"patient_b","procedure":"trauma_surgery"}),
        json!({"patient_id":"patient_a","procedure":"another_procedure"}),
    ] {
        let sid = SessionId::new(SESSION + 103);
        let (bus, _, _) = Bus::open([], [(sid, OPERATING_ROOM)]);
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let mut owner_slab = vec![0; 128 * 1024];
        let mut owner_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let owner_rv = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut client_slab = vec![0; 128 * 1024];
        let mut client_storage =
            SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let client_rv = client_storage
            .init()
            .rendezvous(&mut client_slab, &bus.transport)?;
        let mut owner = owner_rv.enter(sid, &owner_program)?;
        let mut donor = client_rv.enter(sid, &donor_program)?;
        let mut anesthesia = client_rv.enter(sid, &anesthesia_program)?;
        let room = client_rv.enter(sid, &room_program)?;
        let mut caller = client_rv.enter(sid, &caller_program)?;
        hibana_runtime::race(
            async {
                async {
                    let result = localside::care::surgery(&mut owner, &effects).await;
                    assert!(
                        matches!(result,Err(ShellError::Message(ref message))
                    if message == "request differs from its actual admitted data"),
                        "{result:?}"
                    );
                    Ok::<(), ShellError>(())
                };
                async {
                    let diagnosis = imp::encode(&json!({"result":{"success":true,
                    "data":{"patient_id":"patient_a","artifact_id":"actual diagnosis"}}}))?;
                    donor
                        .send::<DiagnosisForSurgery>(&diagnosis.as_slice())
                        .await?;
                    let preparation = imp::encode(&json!({"success":true,"data":{
                    "patient_id":"patient_a","artifact_id":"actual preparation",
                    "inputs":{"diagnosis:ER_DOCTOR_A":"actual diagnosis"}}}))?;
                    async {
                        let [mut custody, mut actual] = anesthesia.par()?;
                        async {
                            custody.send::<PreparedRoom>(room).await?;
                        };
                        async {
                            actual
                                .send::<PreparationForSurgery>(&preparation.as_slice())
                                .await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    let admitted = imp::encode(
                        &json!({"patient_id":"patient_a","procedure":"trauma_surgery"}),
                    )?;
                    caller.send::<SurgeryIntent>(&admitted.as_slice()).await?;
                    let mut room = caller.recv::<SurgeryLoan>().await?;
                    let changed = imp::encode(&changed)?;
                    room.send::<SurgeryRequest>(&changed.as_slice()).await?;
                };
                Ok::<(), ShellError>(())
            },
            deadline(std::time::Duration::from_secs(if cfg!(miri) {
                300
            } else {
                15
            })),
        )
        .await?;
        assert!(effects.0.take().is_empty());
        assert!(effects.1.take().is_empty(), "Domain was invoked");
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_icu_receiver_requires_both_unchanged_requests_before_the_effect() -> ShellResult<()>
{
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<PATIENT_B_SERVICE, ICU_SERVICE, DiagnosisForIcu>(),
        g::seq(
            g::send::<ICU_COORDINATOR, ICU_SERVICE, IcuIntent>(),
            g::seq(
                g::par(
                    g::send::<ICU_SERVICE, ICU_COORDINATOR, ResourceLoan<ICU_BEDS>>(),
                    g::send::<ICU_SERVICE, ICU_COORDINATOR, ResourceLoan<VENTILATOR>>(),
                ),
                g::par(
                    g::send::<ICU_BEDS, ICU_SERVICE, BedRequest>(),
                    g::send::<VENTILATOR, ICU_SERVICE, VentilatorRequest>(),
                ),
            ),
        ),
    );
    let owner_program = project::<ICU_SERVICE>(&source);
    let donor_program = project::<PATIENT_B_SERVICE>(&source);
    let caller_program = project::<ICU_COORDINATOR>(&source);
    let bed_program = project::<ICU_BEDS>(&source);
    let vent_program = project::<VENTILATOR>(&source);
    for change_bed in [true, false] {
        for changed in [
            json!({"patient_id":"patient_a"}),
            json!({"patient_id":"patient_b","unreviewed":true}),
        ] {
            let sid = SessionId::new(SESSION + 104);
            let (bus, _, _) = Bus::open([], [(sid, ICU_BEDS), (sid, VENTILATOR)]);
            let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
            let mut owner_slab = vec![0; 128 * 1024];
            let mut owner_storage =
                SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
            let owner_rv = owner_storage
                .init()
                .rendezvous(&mut owner_slab, &bus.transport)?;
            let mut client_slab = vec![0; 128 * 1024];
            let mut client_storage =
                SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
            let client_rv = client_storage
                .init()
                .rendezvous(&mut client_slab, &bus.transport)?;
            let mut owner = owner_rv.enter(sid, &owner_program)?;
            let bed = owner_rv.enter(sid, &bed_program)?;
            let ventilator = owner_rv.enter(sid, &vent_program)?;
            let mut donor = client_rv.enter(sid, &donor_program)?;
            let mut caller = client_rv.enter(sid, &caller_program)?;
            hibana_runtime::race(
                async {
                    async {
                        let result =
                            localside::care::icu(&mut owner, (bed, ventilator), &effects).await;
                        assert!(
                            matches!(result,Err(ShellError::Message(ref message))
                        if message == "request differs from its actual admitted data"),
                            "{result:?}"
                        );
                        Ok::<(), ShellError>(())
                    };
                    async {
                        let diagnosis = imp::encode(&json!({"result":{"success":true,
                        "data":{"patient_id":"patient_b","artifact_id":"actual diagnosis"}}}))?;
                        donor.send::<DiagnosisForIcu>(&diagnosis.as_slice()).await?;
                        let admitted = json!({"patient_id":"patient_b"});
                        let bytes = imp::encode(&admitted)?;
                        caller.send::<IcuIntent>(&bytes.as_slice()).await?;
                        let (mut bed, mut ventilator) = async {
                            let [mut b, mut v] = caller.par()?;
                            async { b.recv::<ResourceLoan<ICU_BEDS>>().await };
                            async { v.recv::<ResourceLoan<VENTILATOR>>().await };
                        }
                        .await?;
                        let bed_bytes = imp::encode(if change_bed { &changed } else { &admitted })?;
                        let vent_bytes =
                            imp::encode(if change_bed { &admitted } else { &changed })?;
                        async {
                            async {
                                bed.send::<BedRequest>(&bed_bytes.as_slice()).await?;
                            };
                            async {
                                ventilator
                                    .send::<VentilatorRequest>(&vent_bytes.as_slice())
                                    .await?;
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;
                    };
                    Ok::<(), ShellError>(())
                },
                deadline(std::time::Duration::from_secs(if cfg!(miri) {
                    300
                } else {
                    15
                })),
            )
            .await?;
            assert!(effects.0.take().is_empty());
            assert!(effects.1.take().is_empty(), "Domain was invoked");
        }
    }
    Ok(())
}
