//! Direct production receivers and all six independent CT/MRI operation orders.
use super::{global::*, localside};
use crate::imp::{json, patient};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::g::Message;
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

/// Actual fixture requests/results only; no protocol permission or next-step table.
struct Effects(Cell<Vec<Value>>, Cell<Vec<Value>>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut requests = self.1.take();
        requests.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.1.set(requests);
        let mut effects = self.0.take();
        let result = (|| {
            let latest =
                |actor: &str, tool: &str, modality: Option<&str>| -> Result<&Value, String> {
                    effects
                        .iter()
                        .rev()
                        .find(|r| {
                            r["data"]["agent"] == actor
                                && r["tool_name"] == tool
                                && modality.is_none_or(|m| r["data"]["modality"] == m)
                        })
                        .ok_or_else(|| format!("missing actual {actor} {tool}"))
                };
            let mut inputs = json!({});
            if tool != "assess_patient" && tool != "review_prescriptions" {
                inputs["assessment"] =
                    latest("ER_DOCTOR", "assess_patient", None)?["data"]["artifact_id"].clone();
            }
            if tool == "interpret_imaging" {
                inputs["image"] = latest(
                    "RADIOLOGIST",
                    "perform_imaging",
                    arguments["modality"].as_str(),
                )?["data"]["artifact_id"]
                    .clone();
            }
            if tool == "diagnose" {
                let expected = if agent == "CARDIOLOGIST" {
                    ["ecg", "cardiac_enzymes"]
                } else {
                    ["neuro_exam", "nerve_conduction"]
                };
                for test in expected {
                    let actual = effects
                        .iter()
                        .rev()
                        .find(|r| {
                            r["data"]["agent"] == agent
                                && r["tool_name"] == "order_tests"
                                && r["data"]["tests"]
                                    .as_array()
                                    .is_some_and(|ts| ts.iter().any(|t| t == test))
                        })
                        .ok_or_else(|| format!("missing actual test {test}"))?;
                    inputs[format!("test:{test}")] = actual["data"]["artifact_id"].clone();
                }
                for modality in ["CT", "MRI"] {
                    inputs[format!("imaging:{modality}")] = latest(
                        "RADIOLOGIST",
                        "interpret_imaging",
                        Some(modality),
                    )?["data"]["artifact_id"]
                        .clone();
                }
            }
            if tool == "conduct_case_conference" {
                for actor in ["CARDIOLOGIST", "NEUROLOGIST"] {
                    inputs[format!("diagnosis:{actor}")] =
                        latest(actor, "diagnose", None)?["data"]["artifact_id"].clone();
                }
            }
            let tests = arguments["test_type"]
                .as_str()
                .map(|s| s.split(',').map(str::trim).collect::<Vec<_>>())
                .unwrap_or_default();
            Ok(
                json!({"success":true,"tool_name":tool,"data":{"patient_id":arguments["patient_id"],
                "agent":agent,"artifact_id":format!("{tool}:{}",effects.len()),"inputs":inputs,
                "tests":tests,"modality":arguments["modality"],"arguments":arguments}}),
            )
        })();
        if let Ok(value) = &result {
            effects.push(value.clone());
        }
        self.0.set(effects);
        result
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn independent_image_orders_and_partial_panels_reach_the_actual_conference() -> ShellResult<()>
{
    let p = programs();
    for order in [
        [("CT", true), ("CT", false), ("MRI", true), ("MRI", false)],
        [("CT", true), ("MRI", true), ("CT", false), ("MRI", false)],
        [("CT", true), ("MRI", true), ("MRI", false), ("CT", false)],
        [("MRI", true), ("MRI", false), ("CT", true), ("CT", false)],
        [("MRI", true), ("CT", true), ("MRI", false), ("CT", false)],
        [("MRI", true), ("CT", true), ("CT", false), ("MRI", false)],
    ] {
        let session = SessionId::new(SESSION + 200);
        let (bus, _, _) = Bus::open(
            [],
            [
                (session, GENERAL_PANEL),
                (session, CARDIAC_PANEL),
                (session, NEURO_PANEL),
                (session, CT_SCANNER),
                (session, MRI),
                (session, CONSULTATION_ROOM),
            ],
        );
        let general_input = Cell::new(Value::Null);
        let cardiac_input = Cell::new(Value::Null);
        let neuro_input = Cell::new(Value::Null);
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &p.general,
            ResolverRef::<GENERAL_COMPLETE>::decision_state(&general_input, |x: &Cell<Value>| {
                patient::panel_arm(x, &["vitals", "basic_labs"])
            }),
        )
        .unwrap();
        rv.set_resolver(
            &p.cardiac,
            ResolverRef::<CARDIAC_COMPLETE>::decision_state(&cardiac_input, |x: &Cell<Value>| {
                patient::panel_arm(x, &["ecg", "cardiac_enzymes"])
            }),
        )
        .unwrap();
        rv.set_resolver(
            &p.neurology,
            ResolverRef::<NEURO_COMPLETE>::decision_state(&neuro_input, |x: &Cell<Value>| {
                patient::panel_arm(x, &["neuro_exam", "nerve_conduction"])
            }),
        )
        .unwrap();
        let mut er = rv.enter(session, &p.er)?;
        let mut cardio = rv.enter(session, &p.cardio)?;
        let mut neuro = rv.enter(session, &p.neuro)?;
        let mut radiologist = rv.enter(session, &p.radiologist)?;
        let mut pharmacist = rv.enter(session, &p.pharmacist)?;
        let mut er_io = rv.enter(session, &p.er_io)?;
        let mut cardio_io = rv.enter(session, &p.cardio_io)?;
        let mut neuro_io = rv.enter(session, &p.neuro_io)?;
        let mut radiology_io = rv.enter(session, &p.radiology_io)?;
        let mut pharmacy_io = rv.enter(session, &p.pharmacy_io)?;
        let mut general = rv.enter(session, &p.general)?;
        let mut cardiac = rv.enter(session, &p.cardiac)?;
        let mut neurology = rv.enter(session, &p.neurology)?;
        let mut ct_service = rv.enter(session, &p.ct_service)?;
        let mut mri_service = rv.enter(session, &p.mri_service)?;
        let mut pharmacy = rv.enter(session, &p.pharmacy)?;
        let mut conference = rv.enter(session, &p.conference)?;
        let general_panel = rv.enter(session, &p.general_panel)?;
        let cardiac_panel = rv.enter(session, &p.cardiac_panel)?;
        let neuro_panel = rv.enter(session, &p.neuro_panel)?;
        let ct = rv.enter(session, &p.ct)?;
        let mri = rv.enter(session, &p.mri)?;
        let room = rv.enter(session, &p.room)?;
        let effects = Effects(Cell::new(Vec::new()), Cell::new(Vec::new()));
        let general_requests = &["vitals", "basic_labs"];
        let cardiac_requests = &["ecg", "cardiac_enzymes"];
        let neuro_requests = &["nerve_conduction", "neuro_exam"];
        hibana_runtime::race(async {
            async{localside::services::general(&mut general,(general_panel,&general_input),&effects).await?;};
            async{localside::services::cardiac(&mut cardiac,(cardiac_panel,&cardiac_input),&effects).await?;};
            async{localside::services::neurology(&mut neurology,(neuro_panel,&neuro_input),&effects).await?;};
            async{localside::services::ct(&mut ct_service,ct,&effects).await?;};
            async{localside::services::mri(&mut mri_service,mri,&effects).await?;};
            async{localside::services::pharmacy(&mut pharmacy,&effects).await?;};
            async{localside::services::conference(&mut conference,room,&effects).await?;};
            async{assert_eq!(er_io.recv::<crate::shell::global::Exit>().await?,0);Ok(())};
            async{assert_eq!(cardio_io.recv::<crate::shell::global::Exit>().await?,0);Ok(())};
            async{assert_eq!(neuro_io.recv::<crate::shell::global::Exit>().await?,0);Ok(())};
            async{assert_eq!(radiology_io.recv::<crate::shell::global::Exit>().await?,0);Ok(())};
            async{assert_eq!(pharmacy_io.recv::<crate::shell::global::Exit>().await?,0);Ok(())};
            async{
                let [mut business,mut private]=er.par()?;
                let (mut case,review)=async {
                    let [mut clinical,mut review]=business.par()?;
                    async{
                        let bytes=json::encode(&json!({"patient_id":"patient_1"}))?;
                        clinical.send::<AssessmentRequest>(&bytes.as_slice()).await?;
                        let assessment=json::decode(clinical.recv::<AssessmentResult>().await?)?;
                        let ((receipts,_general_panel),_card_panel,_neuro_panel)=async {
                            let [mut own,mut card,mut neuro]=clinical.par()?;
                            async{{
let mut requests=general_requests.iter();
loop {
  let mut panel=own.recv::<PanelLoan<GENERAL_PANEL>>().await?;
  let test=requests.next().expect("unexpected panel repetition");
  let bytes=json::encode(&json!({"patient_id":"patient_1","test_type":test}))?;
  panel.send::<TestRequest>(&bytes.as_slice()).await?;
  patient::actual_result(&json::decode(panel.recv::<TestResult>().await?)?)?;
  own.send::<PanelReturn<GENERAL_PANEL>>(panel).await?;
  let branch=own.offer().await?;
  match branch.label() {
    PanelCompleted::<GENERAL_PANEL>::LOGICAL_LABEL =>{
      let panel=branch.recv::<PanelCompleted<GENERAL_PANEL>>().await?;
      let receipts=json::decode(own.recv::<PanelComplete>().await?)?;
      assert!(requests.next().is_none());
      break Ok::<_,ShellError>((receipts,panel));
    }
    PanelPartial::LOGICAL_LABEL =>{let _receipts=json::decode(branch.recv::<PanelPartial>().await?)?;}
    _=>return Err(ShellError::message("unexpected panel result")),
  }
}
}};
                            async{
                                let bytes=json::encode(&assessment)?;
                                card.send::<CardReferral>(&bytes.as_slice()).await?;
                                Ok(card.recv::<PanelCompleted<CARDIAC_PANEL>>().await?)
                            };
                            async{
                                let bytes=json::encode(&assessment)?;
                                neuro.send::<NeuroReferral>(&bytes.as_slice()).await?;
                                Ok(neuro.recv::<PanelCompleted<NEURO_PANEL>>().await?)
                            };
                        }.await?;
                        let (ct,mri)=async{
                            let [mut c,mut m]=clinical.par()?;
                            async{json::decode(c.recv::<CtForER>().await?)};
                            async{json::decode(m.recv::<MriForER>().await?)};
                        }.await?;
                        let (cardiac,neuro)=async{
                            let [mut c,mut n]=clinical.par()?;
                            async{json::decode(c.recv::<CardDiagnosisForER>().await?)};
                            async{json::decode(n.recv::<NeuroDiagnosisForER>().await?)};
                        }.await?;
                        Ok(json!({"assessment":assessment,"general":receipts,"CT":ct,"MRI":mri,
                            "cardiac":cardiac,"neurology":neuro}))
                    };
                    async{json::decode(review.recv::<ReviewForER>().await?)};
                }.await?;
                case["review"]=review;
                let admitted=json!({"case":case,"arguments":{"patient_id":"patient_1","attendees":"actual fixture attendees"}});
                let bytes=json::encode(&admitted)?;
                business.send::<ConferenceIntent>(&bytes.as_slice()).await?;
                let mut room=business.recv::<RoomLoan>().await?;
                room.send::<RoomRequest>(&bytes.as_slice()).await?;
                let result=json::decode(room.recv::<RoomResult>().await?)?;
                business.send::<RoomLoan>(room).await?;
                let final_case=json::decode(business.recv::<FinalForER>().await?)?;
                assert_eq!(final_case,json!({"case":case,"conference":result}));
                drop(business);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
                Ok(())
            };
            async {
  let [mut task,mut private]=cardio.par()?;
  let assessment=json::decode(task.recv::<CardReferral>().await?)?;
  let bytes=json::encode(&assessment)?;
  async {
    let [mut panel_owner,mut radiologist,mut image_owner]=task.par()?;
    async {panel_owner.send::<ReferralForOwner>(&bytes.as_slice()).await?;};
    async {radiologist.send::<CtOrder>(&bytes.as_slice()).await?;};
    async {image_owner.send::<CtOrderForOwner>(&bytes.as_slice()).await?;};
    Ok(())
  }.await?;
  let receipts={
    let mut requests=cardiac_requests.iter();
    loop {
      let mut panel=task.recv::<PanelLoan<CARDIAC_PANEL>>().await?;
      let test=requests.next().expect("unexpected specialist repetition");
      let bytes=json::encode(&json!({"patient_id":"patient_1","test_type":test}))?;
      panel.send::<TestRequest>(&bytes.as_slice()).await?;
      patient::actual_result(&json::decode(panel.recv::<TestResult>().await?)?)?;
      task.send::<PanelReturn<CARDIAC_PANEL>>(panel).await?;
      let branch=task.offer().await?;
      match branch.label() {
        PanelPartial::LOGICAL_LABEL =>{let _receipts=json::decode(branch.recv::<PanelPartial>().await?)?;}
        PanelComplete::LOGICAL_LABEL =>{
          let receipts=json::decode(branch.recv::<PanelComplete>().await?)?;
          assert!(patient::panel_complete(&receipts,&["ecg","cardiac_enzymes"])?);
          assert!(requests.next().is_none());
          break receipts;
        }
        _=>return Err(ShellError::message("unexpected specialist result")),
      }
    }
  };
  let (general,cross,ct,mri)=async {
    let [mut g,mut x,mut c,mut m]=task.par()?;
    async{json::decode(g.recv::<GeneralForCard>().await?)};
    async{json::decode(x.recv::<NeuroPanelForCard>().await?)};
    async{json::decode(c.recv::<CtForCard>().await?)};
    async{json::decode(m.recv::<MriForCard>().await?)};
  }.await?;
  json::same(&general["assessment"],&assessment)?;
  assert!(patient::panel_complete(&cross,&["neuro_exam","nerve_conduction"])?);
  for image in [&ct,&mri]{patient::actual_result(image)?;}
  let bytes=json::encode(&json!({"patient_id":"patient_1","symptoms":"actual fixture symptom data"}))?;
  task.send::<CardDiagnosisRequest>(&bytes.as_slice()).await?;
  let diagnosis=json::decode(task.recv::<CardDiagnosisResult>().await?)?;
  let final_case=json::decode(task.recv::<FinalForCard>().await?)?;
  assert_eq!(final_case["case"]["cardiac"]["result"],diagnosis);
  assert_eq!(final_case["case"]["cardiac"]["tests"],receipts);
  drop(task);
  private.send::<crate::shell::global::Exit>(&0_i32).await?;
  Ok(())
};
            async {
  let [mut task,mut private]=neuro.par()?;
  let assessment=json::decode(task.recv::<NeuroReferral>().await?)?;
  let bytes=json::encode(&assessment)?;
  async {
    let [mut panel_owner,mut radiologist,mut image_owner]=task.par()?;
    async {panel_owner.send::<ReferralForOwner>(&bytes.as_slice()).await?;};
    async {radiologist.send::<MriOrder>(&bytes.as_slice()).await?;};
    async {image_owner.send::<MriOrderForOwner>(&bytes.as_slice()).await?;};
    Ok(())
  }.await?;
  let receipts={
    let mut requests=neuro_requests.iter();
    loop {
      let mut panel=task.recv::<PanelLoan<NEURO_PANEL>>().await?;
      let test=requests.next().expect("unexpected specialist repetition");
      let bytes=json::encode(&json!({"patient_id":"patient_1","test_type":test}))?;
      panel.send::<TestRequest>(&bytes.as_slice()).await?;
      patient::actual_result(&json::decode(panel.recv::<TestResult>().await?)?)?;
      task.send::<PanelReturn<NEURO_PANEL>>(panel).await?;
      let branch=task.offer().await?;
      match branch.label() {
        PanelPartial::LOGICAL_LABEL =>{let _receipts=json::decode(branch.recv::<PanelPartial>().await?)?;}
        PanelComplete::LOGICAL_LABEL =>{
          let receipts=json::decode(branch.recv::<PanelComplete>().await?)?;
          assert!(patient::panel_complete(&receipts,&["neuro_exam","nerve_conduction"])?);
          assert!(requests.next().is_none());
          break receipts;
        }
        _=>return Err(ShellError::message("unexpected specialist result")),
      }
    }
  };
  let (general,cross,ct,mri)=async {
    let [mut g,mut x,mut c,mut m]=task.par()?;
    async{json::decode(g.recv::<GeneralForNeuro>().await?)};
    async{json::decode(x.recv::<CardPanelForNeuro>().await?)};
    async{json::decode(c.recv::<CtForNeuro>().await?)};
    async{json::decode(m.recv::<MriForNeuro>().await?)};
  }.await?;
  json::same(&general["assessment"],&assessment)?;
  assert!(patient::panel_complete(&cross,&["ecg","cardiac_enzymes"])?);
  for image in [&ct,&mri]{patient::actual_result(image)?;}
  let bytes=json::encode(&json!({"patient_id":"patient_1","symptoms":"actual fixture symptom data"}))?;
  task.send::<NeuroDiagnosisRequest>(&bytes.as_slice()).await?;
  let diagnosis=json::decode(task.recv::<NeuroDiagnosisResult>().await?)?;
  let final_case=json::decode(task.recv::<FinalForNeuro>().await?)?;
  assert_eq!(final_case["case"]["neurology"]["result"],diagnosis);
  assert_eq!(final_case["case"]["neurology"]["tests"],receipts);
  drop(task);
  private.send::<crate::shell::global::Exit>(&0_i32).await?;
  Ok(())
};
            async{
                let [mut task,mut private]=radiologist.par()?;
                async{
                    let [mut ct,mut mri]=task.par()?;
                    for (modality,perform) in order {
                        let bytes=json::encode(&json!({"patient_id":"patient_1","modality":modality}))?;
                        match (modality,perform) {
                            ("CT",true)=>{
                                patient::actual_result(&json::decode(ct.recv::<CtOrder>().await?)?)?;
                                ct.send::<CtPerformIntent>(&bytes.as_slice()).await?;
                                let mut resource=ct.recv::<CtLoan>().await?;
                                resource.send::<CtPerformRequest>(&bytes.as_slice()).await?;
                                patient::actual_result(&json::decode(resource.recv::<CtImageResult>().await?)?)?;
                                ct.send::<CtLoan>(resource).await?;
                            }
                            ("CT",false)=>{
                                ct.send::<CtInterpretIntent>(&bytes.as_slice()).await?;
                                let mut resource=ct.recv::<CtLoan>().await?;
                                resource.send::<CtInterpretRequest>(&bytes.as_slice()).await?;
                                patient::actual_result(&json::decode(resource.recv::<CtReportResult>().await?)?)?;
                                ct.send::<CtLoan>(resource).await?;
                            }
                            ("MRI",true)=>{
                                patient::actual_result(&json::decode(mri.recv::<MriOrder>().await?)?)?;
                                mri.send::<MriPerformIntent>(&bytes.as_slice()).await?;
                                let mut resource=mri.recv::<MriLoan>().await?;
                                resource.send::<MriPerformRequest>(&bytes.as_slice()).await?;
                                patient::actual_result(&json::decode(resource.recv::<MriImageResult>().await?)?)?;
                                mri.send::<MriLoan>(resource).await?;
                            }
                            ("MRI",false)=>{
                                mri.send::<MriInterpretIntent>(&bytes.as_slice()).await?;
                                let mut resource=mri.recv::<MriLoan>().await?;
                                resource.send::<MriInterpretRequest>(&bytes.as_slice()).await?;
                                patient::actual_result(&json::decode(resource.recv::<MriReportResult>().await?)?)?;
                                mri.send::<MriLoan>(resource).await?;
                            }
                            _=>unreachable!(),
                        }
                    }
                    Ok::<_,ShellError>(())
                }.await?;
                let final_case=json::decode(task.recv::<FinalForRad>().await?)?;
                patient::actual_result(&final_case["conference"])?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
                Ok(())
            };
            async{
                let [mut task,mut private]=pharmacist.par()?;
                let bytes=json::encode(&json!({"patient_id":"patient_1","medications":"actual caller data"}))?;
                task.send::<ReviewRequest>(&bytes.as_slice()).await?;
                patient::actual_result(&json::decode(task.recv::<ReviewResult>().await?)?)?;
                drop(task);
                private.send::<crate::shell::global::Exit>(&0_i32).await?;
                Ok(())
            };
            Ok(())
        },deadline(std::time::Duration::from_secs(if cfg!(miri){600}else{45}))).await?;
        let results = effects.0.take();
        assert_eq!(results.len(), 15);
        assert_eq!(
            results
                .iter()
                .filter(|r| r["tool_name"] == "conduct_case_conference")
                .count(),
            1
        );
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_ct_receiver_rejects_changed_arguments_before_the_effect() -> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<CARDIOLOGIST, CT_SERVICE, CtOrderForOwner>(),
        g::seq(
            g::send::<RADIOLOGIST, CT_SERVICE, CtPerformIntent>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequest>(),
            ),
        ),
    );
    let owner_program = project::<CT_SERVICE>(&source);
    let doctor_program = project::<CARDIOLOGIST>(&source);
    let radiologist_program = project::<RADIOLOGIST>(&source);
    let scanner_program = project::<CT_SCANNER>(&source);
    for changed in [
        json!({"patient_id":"another_patient","modality":"CT"}),
        json!({"patient_id":"patient_1","modality":"MRI"}),
    ] {
        let sid = SessionId::new(SESSION + 100);
        let (bus, _, _) = Bus::open([], [(sid, CT_SCANNER)]);
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
        let scanner = owner_rv.enter(sid, &scanner_program)?;
        let mut doctor = client_rv.enter(sid, &doctor_program)?;
        let mut radiologist = client_rv.enter(sid, &radiologist_program)?;
        async {
            async {
                assert!(matches!(localside::services::ct(&mut owner,scanner,&effects).await,Err(ShellError::Message(_))));
                Ok::<_,ShellError>(())
            };
            async {
                let assessment=json::encode(&json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"actual assessment"}}))?;
                doctor.send::<CtOrderForOwner>(&assessment.as_slice()).await?;
                let admitted=json::encode(&json!({"patient_id":"patient_1","modality":"CT"}))?;
                radiologist.send::<CtPerformIntent>(&admitted.as_slice()).await?;
                let mut scanner=radiologist.recv::<CtLoan>().await?;
                let changed=json::encode(&changed)?;
                scanner.send::<CtPerformRequest>(&changed.as_slice()).await?;
            };
            Ok(())
        }.await?;
        assert!(effects.0.take().is_empty());
        assert!(
            effects.1.take().is_empty(),
            "the effect boundary was invoked"
        );
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_mri_receiver_rejects_changed_arguments_before_the_effect() -> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let source = g::seq(
        g::send::<NEUROLOGIST, MRI_SERVICE, MriOrderForOwner>(),
        g::seq(
            g::send::<RADIOLOGIST, MRI_SERVICE, MriPerformIntent>(),
            g::seq(
                g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
            ),
        ),
    );
    let owner_program = project::<MRI_SERVICE>(&source);
    let doctor_program = project::<NEUROLOGIST>(&source);
    let radiologist_program = project::<RADIOLOGIST>(&source);
    let scanner_program = project::<MRI>(&source);
    for changed in [
        json!({"patient_id":"another_patient","modality":"MRI"}),
        json!({"patient_id":"patient_1","modality":"CT"}),
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
        let scanner = owner_rv.enter(sid, &scanner_program)?;
        let mut doctor = client_rv.enter(sid, &doctor_program)?;
        let mut radiologist = client_rv.enter(sid, &radiologist_program)?;
        async {
            async {
                assert!(matches!(localside::services::mri(&mut owner,scanner,&effects).await,Err(ShellError::Message(_))));
                Ok::<_,ShellError>(())
            };
            async {
                let assessment=json::encode(&json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"actual assessment"}}))?;
                doctor.send::<MriOrderForOwner>(&assessment.as_slice()).await?;
                let admitted=json::encode(&json!({"patient_id":"patient_1","modality":"MRI"}))?;
                radiologist.send::<MriPerformIntent>(&admitted.as_slice()).await?;
                let mut scanner=radiologist.recv::<MriLoan>().await?;
                let changed=json::encode(&changed)?;
                scanner.send::<MriPerformRequest>(&changed.as_slice()).await?;
            };
            Ok(())
        }.await?;
        assert!(effects.0.take().is_empty());
        assert!(
            effects.1.take().is_empty(),
            "the effect boundary was invoked"
        );
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn actual_conference_receiver_rejects_changed_request_before_the_effect() -> ShellResult<()> {
    use hibana::{g, runtime::program::project};
    let inputs = g::par(
        g::seq(
            g::par(
                g::send::<GENERAL_SERVICE, CONFERENCE_SERVICE, GeneralForConference>(),
                g::par(
                    g::send::<CT_SERVICE, CONFERENCE_SERVICE, CtForConference>(),
                    g::send::<MRI_SERVICE, CONFERENCE_SERVICE, MriForConference>(),
                ),
            ),
            g::par(
                g::send::<CARDIAC_SERVICE, CONFERENCE_SERVICE, CardDiagnosisForConference>(),
                g::send::<NEURO_SERVICE, CONFERENCE_SERVICE, NeuroDiagnosisForConference>(),
            ),
        ),
        g::send::<PHARMACY_SERVICE, CONFERENCE_SERVICE, ReviewForConference>(),
    );
    let source = g::seq(
        inputs,
        g::seq(
            g::send::<ER_DOCTOR, CONFERENCE_SERVICE, ConferenceIntent>(),
            g::seq(
                g::send::<CONFERENCE_SERVICE, ER_DOCTOR, RoomLoan>(),
                g::send::<CONSULTATION_ROOM, CONFERENCE_SERVICE, RoomRequest>(),
            ),
        ),
    );
    let owner_program = project::<CONFERENCE_SERVICE>(&source);
    let er_program = project::<ER_DOCTOR>(&source);
    let room_program = project::<CONSULTATION_ROOM>(&source);
    let general_program = project::<GENERAL_SERVICE>(&source);
    let ct_program = project::<CT_SERVICE>(&source);
    let mri_program = project::<MRI_SERVICE>(&source);
    let cardiac_program = project::<CARDIAC_SERVICE>(&source);
    let neuro_program = project::<NEURO_SERVICE>(&source);
    let pharmacy_program = project::<PHARMACY_SERVICE>(&source);
    let assessment = json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"received assessment"}});
    let general = json!({"assessment":assessment,"tests":{}});
    let ct = json!({"success":true,"data":{"patient_id":"patient_1","modality":"CT","artifact_id":"received CT report"}});
    let mri = json!({"success":true,"data":{"patient_id":"patient_1","modality":"MRI","artifact_id":"received MRI report"}});
    let cardiac = json!({"result":{"success":true,"data":{"patient_id":"patient_1","artifact_id":"received cardiac diagnosis"}},"tests":{}});
    let neuro = json!({"result":{"success":true,"data":{"patient_id":"patient_1","artifact_id":"received neuro diagnosis"}},"tests":{}});
    let review = json!({"success":true,"data":{"patient_id":"patient_1","artifact_id":"received medication review"}});
    let admitted = json!({"arguments":{"patient_id":"patient_1","attendees":"actual submitted attendees"},"case":{"assessment":assessment,"general":{},"CT":ct,"MRI":mri,"cardiac":cardiac,"neurology":neuro,"review":review}});
    let mut another_patient = admitted.clone();
    another_patient["arguments"]["patient_id"] = json!("another_patient");
    let mut another_argument = admitted.clone();
    another_argument["arguments"]["attendees"] = json!("not the admitted attendees");
    let mut another_diagnosis = admitted.clone();
    another_diagnosis["case"]["cardiac"]["result"]["data"]["artifact_id"] =
        json!("stale diagnosis");
    for changed in [another_patient, another_argument, another_diagnosis] {
        let sid = SessionId::new(SESSION + 102);
        let (bus, _, _) = Bus::open([], [(sid, CONSULTATION_ROOM)]);
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
        let mut er = client_rv.enter(sid, &er_program)?;
        let mut general_sender = client_rv.enter(sid, &general_program)?;
        let mut ct_sender = client_rv.enter(sid, &ct_program)?;
        let mut mri_sender = client_rv.enter(sid, &mri_program)?;
        let mut cardiac_sender = client_rv.enter(sid, &cardiac_program)?;
        let mut neuro_sender = client_rv.enter(sid, &neuro_program)?;
        let mut pharmacy_sender = client_rv.enter(sid, &pharmacy_program)?;
        async {
            async {
                assert!(matches!(
                    localside::services::conference(&mut owner, room, &effects).await,
                    Err(ShellError::Message(_))
                ));
                Ok::<_, ShellError>(())
            };
            async {
                let bytes = json::encode(&general)?;
                general_sender
                    .send::<GeneralForConference>(&bytes.as_slice())
                    .await?;
                let bytes = json::encode(&ct)?;
                ct_sender.send::<CtForConference>(&bytes.as_slice()).await?;
                let bytes = json::encode(&mri)?;
                mri_sender
                    .send::<MriForConference>(&bytes.as_slice())
                    .await?;
                let bytes = json::encode(&cardiac)?;
                cardiac_sender
                    .send::<CardDiagnosisForConference>(&bytes.as_slice())
                    .await?;
                let bytes = json::encode(&neuro)?;
                neuro_sender
                    .send::<NeuroDiagnosisForConference>(&bytes.as_slice())
                    .await?;
                let bytes = json::encode(&review)?;
                pharmacy_sender
                    .send::<ReviewForConference>(&bytes.as_slice())
                    .await?;
                let bytes = json::encode(&admitted)?;
                er.send::<ConferenceIntent>(&bytes.as_slice()).await?;
                let mut room = er.recv::<RoomLoan>().await?;
                let bytes = json::encode(&changed)?;
                room.send::<RoomRequest>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
        assert!(effects.0.take().is_empty());
        assert!(
            effects.1.take().is_empty(),
            "the conference effect boundary was invoked"
        );
    }
    Ok(())
}
