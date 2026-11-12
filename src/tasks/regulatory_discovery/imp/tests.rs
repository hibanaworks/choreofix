use super::*;
use serde_json::json;

#[test]
fn analysis_contracts_keep_the_scientist_specimen_and_actual_verdict() {
    fn check<const STEP: u8>() {
        let (actor, tool, key, target) = analysis(STEP).unwrap();
        assert!(
            AnalysisArguments::<STEP>::from_bytes(&encode(&json!({key:target})).unwrap()).is_ok()
        );
        for wrong in [
            json!({key:"other"}),
            json!({key:target,"phase":STEP}),
            json!(null),
            json!([]),
        ] {
            assert!(AnalysisArguments::<STEP>::from_bytes(&encode(&wrong).unwrap()).is_err());
        }
        for success in [false, true] {
            let actual =
                json!({"tool_name":tool,"success":success,"data":{"scientist":actor,key:target}});
            let bytes = encode(&actual).unwrap();
            assert_eq!(
                AnalysisReply::<STEP, true>::from_bytes(&bytes).is_ok(),
                success
            );
            assert_eq!(
                AnalysisReply::<STEP, false>::from_bytes(&bytes).is_ok(),
                !success
            );
            for wrong in [
                {
                    let mut value = actual.clone();
                    value["data"]["scientist"] = json!("OTHER");
                    value
                },
                {
                    let mut value = actual.clone();
                    value["data"][key] = json!("other");
                    value
                },
                {
                    let mut value = actual.clone();
                    value["tool_name"] = json!("other");
                    value
                },
                {
                    let mut value = actual.clone();
                    value["success"] = Value::Null;
                    value
                },
            ] {
                let bytes = encode(&wrong).unwrap();
                assert!(AnalysisReply::<STEP, true>::from_bytes(&bytes).is_err());
                assert!(AnalysisReply::<STEP, false>::from_bytes(&bytes).is_err());
            }
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
fn all_four_reanalysis_choices_are_bound_to_the_actual_payload() {
    for biology in [false, true] {
        for toxicology in [false, true] {
            let bytes = encode(&json!({"biology":biology,"toxicology":toxicology})).unwrap();
            assert_eq!(
                ReanalysisPlan::<0, 1>::from_bytes(&bytes).is_ok(),
                !biology && !toxicology
            );
            assert_eq!(
                ReanalysisPlan::<1, 1>::from_bytes(&bytes).is_ok(),
                biology && !toxicology
            );
            assert_eq!(
                ReanalysisPlan::<2, 1>::from_bytes(&bytes).is_ok(),
                !biology && toxicology
            );
            assert_eq!(
                ReanalysisPlan::<3, 1>::from_bytes(&bytes).is_ok(),
                biology && toxicology
            );
        }
    }
    for value in [
        json!({"biology":null,"toxicology":false}),
        json!({"biology":false}),
        json!({"biology":false,"toxicology":false,"phase":0}),
    ] {
        assert!(ReanalysisPlan::<0, 1>::from_bytes(&encode(&value).unwrap()).is_err());
    }
}

#[test]
fn go_and_no_go_reviews_bind_all_completed_current_analysis_receipts() {
    for go in [false, true] {
        let result = |id| json!({"success":true,"data":{"artifact_id":id}});
        let candidate = json!({"go":go,"compound":{"identity":result(1)},
            "bio":{"assay":result(2),"stability":result(3)},"tox":{"cytotoxicity":result(4)},
            "analyses":{"structural":result(5),"characterization":result(6)}});
        let recorded = json!({"tool_name":"submit_decision","success":true,"data":{"go":go,"artifact_id":7,
            "inputs":{"compound":1,"bioassay":2,"stability":3,"toxicity":4,"structural":5,"characterization":6}}});
        decision_receipt(&candidate, &recorded).unwrap();
        for input in [
            "compound",
            "bioassay",
            "stability",
            "toxicity",
            "structural",
            "characterization",
        ] {
            for wrong in [json!(100), Value::Null] {
                let mut stale = recorded.clone();
                stale["data"]["inputs"][input] = wrong;
                assert!(decision_receipt(&candidate, &stale).is_err());
            }
        }
        let admitted = json!({"decision":candidate,"receipt":recorded});
        for success in [false, true] {
            let reviewed = json!({"tool_name":"verify_regulatory_compliance","success":success,
                "data":{"scientist":"REGULATORY_SPECIALIST","inputs":{"compound":1,"decision":7}}});
            regulatory_receipt(&admitted, &reviewed).unwrap();
            for input in ["compound", "decision"] {
                let mut stale = reviewed.clone();
                stale["data"]["inputs"][input] = json!(0);
                assert!(regulatory_receipt(&admitted, &stale).is_err());
            }
        }
    }
}
