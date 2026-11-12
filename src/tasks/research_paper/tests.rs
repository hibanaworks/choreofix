//! Data and actual two-resource preparation fixtures, not a full paper/Wasm run.
use super::{global::*, imp};
use serde_json::{Value, json};

fn research(author: u8) -> Value {
    json!({"tool_name":"research_topic","success":true,"data":{
        "agent":imp::AUTHORS[usize::from(author)],"topic":imp::TOPICS[usize::from(author)],
        "artifact":{"id":format!("research:{author}"),"content":format!("actual source {author}")}}})
}

fn material(author: u8, operation: u8, notes: &str) -> Value {
    json!({"tool_name":match operation {2=>"write_section",3=>"revise_section",_=>"update_references"},"success":true,
        "data":{"agent":imp::AUTHORS[usize::from(author)],"section":imp::SECTIONS[usize::from(author)],
        "artifact":{"id":format!("material:{author}:{operation}"),"content":notes,
        "research_inputs":{imp::TOPICS[usize::from(author)]:research(author)["data"]["artifact"]}}}})
}

#[test]
fn typed_paper_arguments_keep_free_notes_and_reject_other_sections() {
    for author in 0..3 {
        for operation in 1..=6 {
            let value = if operation == 1 {
                json!({"topic":"a freely chosen additional topic"})
            } else {
                json!({"section_name":imp::SECTIONS[usize::from(author)]})
            };
            imp::arguments(author, operation, &value).unwrap();
            if operation != 1 {
                let mut wrong = value.clone();
                wrong["section_name"] = json!(imp::SECTIONS[usize::from((author + 1) % 3)]);
                assert!(imp::arguments(author, operation, &wrong).is_err());
            }
            for bad in [
                Value::Null,
                json!([]),
                json!({}),
                json!({"topic":1}),
                json!({"section_name":true}),
            ] {
                assert!(imp::arguments(author, operation, &bad).is_err());
            }
        }
    }
    let notes = br#"{"section_name":"section_a","content_notes":"an independent claim"}"#;
    assert_eq!(
        imp::Arguments::<AUTHOR_A, 2>::from_bytes(notes)
            .unwrap()
            .into_value()["content_notes"],
        "an independent claim"
    );
    assert!(imp::Arguments::<AUTHOR_A, 3>::from_bytes(notes).is_err());
    assert!(imp::Arguments::<AUTHOR_B, 2>::from_bytes(notes).is_err());
}

#[test]
fn factual_and_editorial_evidence_bind_each_actual_section_without_a_full_paper_barrier() {
    for author in 0..3 {
        let research = research(author);
        let written = material(author, 2, "actual notes");
        let references = material(author, 4, "actual references");
        let arguments = json!({"section_name":imp::SECTIONS[usize::from(author)],"content_notes":"actual notes"});
        imp::material(author, 2, &written, (&arguments, &research)).unwrap();
        let snapshot = imp::snapshot(author, &written, &references).unwrap();
        let mut reused = references.clone();
        reused["data"]["artifact"]["id"] = written["data"]["artifact"]["id"].clone();
        assert!(imp::snapshot(author, &written, &reused).is_err());
        assert_eq!(snapshot["sections"].as_object().unwrap().len(), 1);
        let facts = json!({"tool_name":"fact_check","success":true,"data":{
            "agent":"FACT_CHECKER","section":imp::SECTIONS[usize::from(author)],
            "check_id":format!("fact:{author}"),"input_snapshot":snapshot}});
        let reviewed = json!({"tool_name":"review_sections","success":true,"data":{
            "agent":"EDITOR","scope":imp::SECTIONS[usize::from(author)],"review_id":format!("review:{author}"),
            "input_snapshot":snapshot,"fact_receipts":{imp::SECTIONS[usize::from(author)]:facts["data"]["check_id"]}}});
        imp::checked(author, &facts, &snapshot).unwrap();
        imp::reviewed(author, &reviewed, (&snapshot, &facts)).unwrap();
        for change in 0..10 {
            let mut current = snapshot.clone();
            let mut actual_facts = facts.clone();
            let mut actual_review = reviewed.clone();
            match change {
                0 => {
                    current["sections"][imp::SECTIONS[usize::from(author)]]["content"] =
                        json!("revised after checking")
                }
                1 => {
                    current["references"][imp::SECTIONS[usize::from(author)]]["id"] =
                        json!("replaced bibliography")
                }
                2 => actual_facts["success"] = json!(false),
                3 => actual_facts["data"]["agent"] = json!("EDITOR"),
                4 => actual_facts["data"]["section"] = json!("foreign section"),
                5 => actual_facts["data"]["check_id"] = json!("another occurrence"),
                6 => actual_review["data"]["input_snapshot"] = Value::Null,
                7 => actual_review["data"]["fact_receipts"] = json!({}),
                8 => {
                    current["references"][imp::SECTIONS[usize::from(author)]]["research_inputs"] =
                        Value::Null
                }
                _ => actual_review["data"]["review_id"] = Value::Null,
            }
            assert!(
                imp::reviewed(author, &actual_review, (&current, &actual_facts)).is_err(),
                "author={author} mutation={change}"
            );
        }
        for change in 0..5 {
            let mut wrong = written.clone();
            match change {
                0 => wrong["data"]["artifact"]["research_inputs"] = json!({}),
                1 => wrong["data"]["artifact"]["content"] = json!("changed admitted notes"),
                2 => wrong["data"]["agent"] = json!("another author"),
                3 => wrong["success"] = json!(false),
                _ => wrong["tool_name"] = json!("update_references"),
            }
            assert!(imp::material(author, 2, &wrong, (&arguments, &research)).is_err());
        }
    }
}

#[test]
fn the_actual_result_discriminant_cannot_be_sent_on_the_opposite_branch() {
    let snapshot = imp::snapshot(
        AUTHOR_A,
        &material(AUTHOR_A, 2, "actual notes"),
        &material(AUTHOR_A, 4, "actual references"),
    )
    .unwrap();
    for success in [false, true] {
        let facts = json!({"tool_name":"fact_check","success":success,"data":{"agent":"FACT_CHECKER","section":"section_a","check_id":"actual facts","input_snapshot":snapshot}});

        assert_eq!(
            imp::Verdict::<AUTHOR_A, 5, true>::new(facts.clone()).is_ok(),
            success
        );
        assert_eq!(
            imp::Verdict::<AUTHOR_A, 5, false>::new(facts.clone()).is_ok(),
            !success
        );
        assert!(imp::Verdict::<AUTHOR_B, 5, true>::new(facts.clone()).is_err());
        let review = json!({"tool_name":"review_sections","success":success,"data":{"agent":"EDITOR","scope":"section_a","review_id":"actual review","input_snapshot":snapshot,"fact_receipts":{"section_a":"actual facts"}}});

        assert_eq!(
            imp::Verdict::<AUTHOR_A, 6, true>::new(review.clone()).is_ok(),
            success
        );
        assert_eq!(
            imp::Verdict::<AUTHOR_A, 6, false>::new(review.clone()).is_ok(),
            !success
        );
        assert!(imp::Verdict::<AUTHOR_A, 5, true>::new(review.clone()).is_err());
    }
}
