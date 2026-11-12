use super as imp;
use serde_json::{Value, json};

#[test]
fn evidence_submission_decodes_only_the_admitted_file_and_exact_evidence() {
    use crate::shell::imp::file_wire::{ClosedFile, SERVICE_FD};
    use hibana_wasip1_runtime::protocol;
    for evidence in ["bioassay", "cytotoxicity", "regulatory"] {
        let mut file = ClosedFile {
            completion: protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
            path: b"service/submit_evidence".to_vec(),
            bytes: serde_json::to_vec(&json!({"evidence":evidence})).unwrap(),
        };
        imp::evidence_file(&file, evidence).unwrap();
        assert!(imp::evidence_file(&file, "another_evidence").is_err());
        file.path = b"service/run_cell_assay".to_vec();
        assert!(imp::evidence_file(&file, evidence).is_err());
        file.path = b"service/submit_evidence".to_vec();
        file.completion = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 28));
        assert!(imp::evidence_file(&file, evidence).is_err());
        file.completion = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0));
        for bytes in [
            b"not json".as_slice(),
            br#"{"evidence":"bioassay","extra":true}"#,
            br#"{"evidence":true}"#,
        ] {
            file.bytes = bytes.to_vec();
            assert!(imp::evidence_file(&file, evidence).is_err());
        }
    }
}

#[test]
fn biological_receipts_preserve_the_actual_material_and_sample_consumption() {
    let mut material = json!({
        "formulation":result(4,4,true),"cells":result(1,1,true),
        "baseline":result(2,2,true),"purity":result(7,7,true)
    });
    material["purity"]["data"]["inputs"] = json!({"formulation":4});
    for step in [5, 8] {
        let inputs = imp::assay_inputs(step, &material).unwrap();
        for (success, status, sample) in [
            (true, "complete", json!(0)),
            (false, "injected_failure", json!(2)),
            (false, "exhausted", Value::Null),
        ] {
            let mut actual = result(step, 42, success);
            actual["data"]["status"] = json!(status);
            actual["data"]["sample_unit"] = sample;
            actual["data"]["inputs"] = inputs.clone();
            imp::assay_receipt(step, &inputs, &actual).unwrap();
            let mut wrong = actual.clone();
            wrong["data"]["inputs"]["formulation"] = json!(40);
            assert!(imp::assay_receipt(step, &inputs, &wrong).is_err());
            let mut wrong = actual;
            wrong["data"]["sample_unit"] = if status == "exhausted" {
                json!(0)
            } else {
                Value::Null
            };
            assert!(imp::assay_receipt(step, &inputs, &wrong).is_err());
        }
    }
    material["purity"]["data"]["inputs"]["formulation"] = json!(40);
    assert!(imp::assay_inputs(8, &material).is_err());
    assert!(
        imp::assay_inputs(5, &material).is_ok(),
        "independent bioassay does not require purity"
    );
    material["cells"] = result(1, 1, false);
    assert!(imp::assay_inputs(5, &material).is_err());
    assert!(imp::assay_inputs(8, &material).is_err());
}

fn result(step: u8, event: u64, success: bool) -> Value {
    let (actor, tool, key, target) = imp::analysis(step).unwrap();
    json!({"tool_name":tool,"success":success,"data":{
        "scientist":actor,key:target.unwrap_or("compound-3"),"event_id":event,
        "artifact_id":if success{json!(event)}else{Value::Null},
        "status":if success{"complete"}else if step==10||step==11{"clinical_hold"}else{"injected_failure"}}})
}

#[test]
fn admitted_argument_files_keep_the_actual_tool_target_and_close_result() {
    use crate::shell::imp::file_wire::{ClosedFile, SERVICE_FD};
    use hibana_wasip1_runtime::protocol;
    fn check<const STEP: u8>() {
        let (_, tool, key, target) = imp::analysis(STEP).unwrap();
        let input = json!({key:target.unwrap_or("compound-3")});
        let path = format!("service/{tool}").into_bytes();
        let mut file = ClosedFile {
            completion: protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
            path: path.clone(),
            bytes: serde_json::to_vec(&input).unwrap(),
        };
        assert_eq!(imp::analysis_file::<STEP>(&file).unwrap(), input);
        file.path = b"service/another_tool".to_vec();
        assert!(imp::analysis_file::<STEP>(&file).is_err());
        file.path = path;
        file.completion = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 5));
        assert!(imp::analysis_file::<STEP>(&file).is_err());
        file.completion = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0));
        for wrong in [
            json!({}),
            json!({key:3}),
            json!({key:""}),
            json!({key:input[key],"phase":1}),
        ] {
            file.bytes = serde_json::to_vec(&wrong).unwrap();
            assert!(imp::analysis_file::<STEP>(&file).is_err());
        }
        if target.is_some() {
            file.bytes = serde_json::to_vec(&json!({key:"another_target"})).unwrap();
            assert!(imp::analysis_file::<STEP>(&file).is_err());
        }
    }
    check::<0>();
    check::<1>();
    check::<2>();
    check::<3>();
    check::<4>();
    check::<5>();
    check::<6>();
    check::<7>();
    check::<8>();
    check::<9>();
    check::<10>();
    check::<11>();
    check::<12>();
    check::<13>();
    check::<14>();
    check::<15>();
}

#[test]
fn scientific_occurrences_keep_negative_results_distinct_from_positive_artifacts() {
    for step in 0..16 {
        let (_, _, key, target) = imp::analysis(step).unwrap();
        let input = json!({key:target.unwrap_or("compound-3")});
        imp::arguments(step, &input).unwrap();
        for wrong in [
            json!({}),
            json!({key:7}),
            json!({key:""}),
            json!({key:input[key],"phase":0}),
        ] {
            assert!(imp::arguments(step, &wrong).is_err());
        }
        for success in [false, true] {
            let actual = result(step, 42, success);
            if !success && matches!(step, 2 | 3 | 9 | 12 | 13 | 14) {
                assert!(imp::reply(step, &actual).is_err());
                continue;
            }
            assert_eq!(imp::reply(step, &actual).unwrap(), 42);
            let mut wrong = actual.clone();
            wrong["data"]["event_id"] = Value::Null;
            assert!(imp::reply(step, &wrong).is_err());
            for (field, value) in [
                ("scientist", json!("OTHER")),
                ("event_id", json!(-1)),
                ("artifact_id", if success { json!(43) } else { json!(42) }),
                ("status", json!("pending")),
            ] {
                let mut wrong = actual.clone();
                wrong["data"][field] = value;
                assert!(imp::reply(step, &wrong).is_err());
            }
            let mut wrong = actual.clone();
            wrong["success"] = Value::Null;
            assert!(imp::reply(step, &wrong).is_err());
            let mut wrong = actual.clone();
            wrong["tool_name"] = json!("OTHER");
            assert!(imp::reply(step, &wrong).is_err());
        }
    }
    assert!(imp::analysis(16).is_err());
}

#[test]
fn formulation_uses_the_received_successful_compound() {
    let mut identity = result(3, 3, true);
    identity["data"]["compound"] = json!("compound-3");
    imp::formulation_input(&identity, &json!({"compound":"compound-3"})).unwrap();
    for success in [false, true] {
        let mut actual = result(4, 4, success);
        actual["data"]["inputs"] = json!({"compound":3});
        imp::formulation_receipt(&identity, &actual).unwrap();
        for (field, value) in [
            ("compound", json!("compound-2")),
            ("inputs", json!({"compound":2})),
        ] {
            let mut stale = actual.clone();
            stale["data"][field] = value;
            assert!(imp::formulation_receipt(&identity, &stale).is_err());
        }
    }
    for input in [
        json!({"compound":"compound-2"}),
        json!({"compound":"compound-3","phase":1}),
    ] {
        assert!(imp::formulation_input(&identity, &input).is_err());
    }
    identity["success"] = json!(false);
    assert!(imp::formulation_input(&identity, &json!({"compound":"compound-3"})).is_err());
}

#[test]
fn clinical_inputs_use_one_material_chain_and_keep_actual_negative_assays() {
    for bio_success in [false, true] {
        for tox_success in [false, true] {
            let mut candidate = json!({"formulation":result(4,4,true),"bioassay":result(5,5,bio_success),
                "stability":result(6,6,true),"purity":result(7,7,true),"toxicity":result(8,8,tox_success)});
            for name in ["bioassay", "stability", "purity", "toxicity"] {
                candidate[name]["data"]["inputs"] = json!({"formulation":4});
                if name != "purity" {
                    candidate[name]["data"]["inputs"]["cells"] = json!(1);
                }
            }
            candidate["toxicity"]["data"]["inputs"]["baseline"] = json!(2);
            candidate["toxicity"]["data"]["inputs"]["purity"] = json!(7);
            let baseline = result(2, 2, true);
            imp::clinical_inputs(&candidate, &baseline).unwrap();
            for (name, key) in [
                ("bioassay", "formulation"),
                ("bioassay", "cells"),
                ("stability", "formulation"),
                ("stability", "cells"),
                ("purity", "formulation"),
                ("toxicity", "formulation"),
                ("toxicity", "cells"),
                ("toxicity", "baseline"),
                ("toxicity", "purity"),
            ] {
                for replacement in [Value::Null, json!(100)] {
                    let mut wrong = candidate.clone();
                    wrong[name]["data"]["inputs"][key] = replacement;
                    assert!(imp::clinical_inputs(&wrong, &baseline).is_err());
                }
            }
            let mut exhausted = candidate.clone();
            if !bio_success {
                exhausted["bioassay"]["data"]["status"] = json!("exhausted");
            }
            if !tox_success {
                exhausted["toxicity"]["data"]["status"] = json!("exhausted");
            }
            imp::clinical_inputs(&exhausted, &baseline).unwrap();
        }
    }
}

#[test]
fn clinical_data_cites_failed_and_exhausted_occurrences_without_inventing_artifacts() {
    for bio_success in [false, true] {
        for tox_success in [false, true] {
            let candidate = json!({"formulation":result(4,4,true),"bioassay":result(5,5,bio_success),
                "stability":result(6,6,true),"purity":result(7,7,true),"toxicity":result(8,8,tox_success)});
            let mut actual = result(9, 9, true);
            actual["data"]["inputs"] =
                json!({"formulation":4,"bioassay":5,"stability":6,"purity":7,"toxicity":8});
            actual["data"]["assessment"] = json!(if bio_success && tox_success {
                "supported"
            } else {
                "insufficient_data"
            });
            imp::clinical_receipt(&candidate, &actual).unwrap();
            for input in ["formulation", "bioassay", "stability", "purity", "toxicity"] {
                let mut stale = actual.clone();
                stale["data"]["inputs"][input] = Value::Null;
                assert!(imp::clinical_receipt(&candidate, &stale).is_err());
                let mut stale = actual.clone();
                stale["data"]["inputs"][input] = json!(100);
                assert!(imp::clinical_receipt(&candidate, &stale).is_err());
            }
            let mut false_positive = actual.clone();
            false_positive["data"]["assessment"] = json!("supported");
            assert_eq!(
                imp::clinical_receipt(&candidate, &false_positive).is_ok(),
                bio_success && tox_success
            );
            for status in ["exhausted", "injected_failure"] {
                let mut exhausted = candidate.clone();
                if !bio_success {
                    exhausted["bioassay"]["data"]["status"] = json!(status);
                }
                if !tox_success {
                    exhausted["toxicity"]["data"]["status"] = json!(status);
                }
                imp::clinical_receipt(&exhausted, &actual).unwrap();
            }
        }
    }
}

#[test]
fn director_can_abort_or_retry_negative_evidence_but_cannot_call_it_positive_approval() {
    for review in [10, 11] {
        for approved in [false, true] {
            let mut candidate = json!({"compound":result(3,3,true),"formulation":result(4,4,true),
                "clinical":result(9,9,true),"regulatory":result(review,10,approved)});
            candidate["formulation"]["data"]["inputs"] = json!({"compound":3});
            candidate["clinical"]["data"]["inputs"] = json!({"formulation":4});
            candidate["regulatory"]["data"]["inputs"] = json!({"clinical":9});
            for step in [12, 13, 14] {
                candidate["outcome"] = json!(imp::analysis(step).unwrap().3.unwrap());
                let mut actual = result(step, 11, true);
                actual["data"]["inputs"] =
                    json!({"compound":3,"formulation":4,"clinical":9,"regulatory":10});
                assert_eq!(
                    imp::decision_receipt(&candidate, &actual).is_ok(),
                    approved || step != 12
                );
                for input in ["compound", "formulation", "clinical", "regulatory"] {
                    let mut stale = actual.clone();
                    stale["data"]["inputs"][input] = json!(100);
                    assert!(imp::decision_receipt(&candidate, &stale).is_err());
                }
                for (result, input) in [
                    ("formulation", "compound"),
                    ("clinical", "formulation"),
                    ("regulatory", "clinical"),
                ] {
                    let mut stale = candidate.clone();
                    stale[result]["data"]["inputs"][input] = json!(100);
                    assert!(imp::decision_receipt(&stale, &actual).is_err());
                }
                let mut changed = actual.clone();
                changed["data"]["outcome"] = json!(if step == 14 { "retry" } else { "abort" });
                assert!(imp::decision_receipt(&candidate, &changed).is_err());
            }
        }
    }
}

#[test]
fn final_documentation_archives_the_actual_terminal_decision_and_all_current_inputs() {
    for step in [12, 13, 14] {
        for approved in [false, true] {
            let mut candidate = json!({"outcome":imp::analysis(step).unwrap().3.unwrap(),
                "compound":result(3,3,true),"formulation":result(4,4,true),
                "clinical":result(9,9,true),"regulatory":result(11,10,approved),"decision":result(step,11,true)});
            candidate["formulation"]["data"]["inputs"] = json!({"compound":3});
            candidate["clinical"]["data"]["inputs"] = json!({"formulation":4});
            candidate["regulatory"]["data"]["inputs"] = json!({"clinical":9});
            candidate["decision"]["data"]["inputs"] =
                json!({"compound":3,"formulation":4,"clinical":9,"regulatory":10});
            let mut actual = result(15, 12, true);
            actual["data"]["inputs"] =
                json!({"compound":3,"formulation":4,"clinical":9,"regulatory":10,"decision":11});
            assert_eq!(
                imp::documentation_receipt(&candidate, &actual).is_ok(),
                step == 14 || step == 12 && approved
            );
            for name in [
                "compound",
                "formulation",
                "clinical",
                "regulatory",
                "decision",
            ] {
                let mut stale = actual.clone();
                stale["data"]["inputs"][name] = json!(100);
                assert!(imp::documentation_receipt(&candidate, &stale).is_err());
            }
            let mut failed = actual.clone();
            failed["success"] = json!(false);
            failed["data"]["artifact_id"] = Value::Null;
            failed["data"]["status"] = json!("injected_failure");
            assert!(imp::documentation_receipt(&candidate, &failed).is_err());
        }
    }
}

#[test]
fn wire_ingress_keeps_the_step_target_actual_verdict_and_recorded_outcome() {
    use hibana::runtime::wire::{CodecError, Payload, WireEncode, WirePayload};
    let bytes = serde_json::to_vec(&json!({"sample":"synthesis_qc"})).unwrap();
    let input = imp::AnalysisArguments::<0>::from_bytes(&bytes).unwrap();
    imp::AnalysisArguments::<0>::validate_payload(Payload::new(&bytes)).unwrap();
    let mut short = vec![0xcc; bytes.len() - 1];
    assert_eq!(input.encode_into(&mut short), Err(CodecError::Truncated));
    assert!(short.iter().all(|byte| *byte == 0xcc));
    let mut encoded = vec![0xcc; bytes.len() + 4];
    assert_eq!(input.encode_into(&mut encoded).unwrap(), bytes.len());
    assert_eq!(&encoded[..bytes.len()], &bytes);
    assert_eq!(&encoded[bytes.len()..], &[0xcc; 4]);
    assert!(imp::AnalysisArguments::<15>::validate_payload(Payload::new(&bytes)).is_err());
    let mut exhausted = result(5, 42, false);
    exhausted["data"]["status"] = json!("exhausted");
    let bytes = serde_json::to_vec(&exhausted).unwrap();
    imp::AnalysisReply::<5, false>::validate_payload(Payload::new(&bytes)).unwrap();
    assert!(imp::AnalysisReply::<5, true>::validate_payload(Payload::new(&bytes)).is_err());
    assert_ne!(
        imp::AnalysisReply::<5, true>::SCHEMA_ID,
        imp::AnalysisReply::<5, false>::SCHEMA_ID
    );
    imp::JsonTo::<1>::from_bytes(&bytes).unwrap();
    assert_ne!(imp::JsonTo::<1>::SCHEMA_ID, imp::JsonTo::<2>::SCHEMA_ID);
    assert!(imp::JsonTo::<1>::validate_payload(Payload::new(b"[]")).is_err());
    for step in [12, 13, 14] {
        let mut bundle = json!({"outcome":imp::analysis(step).unwrap().3.unwrap(),
            "compound":result(3,3,true),"formulation":result(4,4,true),"clinical":result(9,9,true),
            "regulatory":result(11,10,true),"decision":result(step,11,true)});
        bundle["formulation"]["data"]["inputs"] = json!({"compound":3});
        bundle["clinical"]["data"]["inputs"] = json!({"formulation":4});
        bundle["regulatory"]["data"]["inputs"] = json!({"clinical":9});
        bundle["decision"]["data"]["inputs"] =
            json!({"compound":3,"formulation":4,"clinical":9,"regulatory":10});
        let bytes = serde_json::to_vec(&bundle).unwrap();
        assert_eq!(
            imp::FinalOutcome::<1>::validate_payload(Payload::new(&bytes)).is_ok(),
            step != 13
        );
        bundle["outcome"] = json!(if step == 14 { "retry" } else { "abort" });
        let bytes = serde_json::to_vec(&bundle).unwrap();
        assert!(imp::FinalOutcome::<1>::validate_payload(Payload::new(&bytes)).is_err());
    }
}

#[test]
fn regulatory_results_bind_the_requested_review_and_actual_clinical_assessment() {
    for step in [10, 11] {
        for supported in [false, true] {
            for approved in [false, true] {
                let input = json!({"review_type":imp::analysis(step).unwrap().3.unwrap()});
                let mut clinical = result(9, 9, true);
                clinical["data"]["assessment"] = json!(if supported {
                    "supported"
                } else {
                    "insufficient_data"
                });
                let mut actual = result(step, 10, approved);
                actual["data"]["inputs"] = json!({"clinical":9});
                assert_eq!(
                    imp::regulatory_receipt(&input, &clinical, &actual).is_ok(),
                    supported || !approved
                );
                for wrong in [json!(100), Value::Null] {
                    let mut stale = actual.clone();
                    stale["data"]["inputs"]["clinical"] = wrong;
                    assert!(imp::regulatory_receipt(&input, &clinical, &stale).is_err());
                }
                let mut changed = actual.clone();
                changed["data"]["review_type"] = json!(if step == 10 {
                    "hold_review"
                } else {
                    "initial_review"
                });
                assert!(imp::regulatory_receipt(&input, &clinical, &changed).is_err());
            }
        }
    }
}
