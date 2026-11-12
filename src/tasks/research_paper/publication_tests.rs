//! Final-boundary fixtures; prior chapter work is supplied data, not executed.
use super::imp;
use serde_json::{Value, json};

fn inputs() -> (Value, Value, Value) {
    let mut sections = json!({});
    let mut references = json!({});
    let mut reviews = json!({"facts":{},"reviews":{}});
    for author in 0u8..3 {
        let name = imp::SECTIONS[usize::from(author)];
        let research = json!({imp::TOPICS[usize::from(author)]:{"id":format!("research:{author}"),"content":format!("sources for {name}")}});
        sections[name] = json!({"id":format!("document:{author}"),"content":format!("actual chapter {name}"),"research_inputs":research});
        references[name] = json!({"id":format!("references:{author}"),"content":format!("actual references {name}"),"research_inputs":research});
        let snapshot =
            json!({"sections":{name:sections[name]},"references":{name:references[name]}});
        reviews["facts"][name] = json!({"tool_name":"fact_check","success":true,"data":{
            "agent":"FACT_CHECKER","section":name,"check_id":format!("facts:{author}"),"input_snapshot":snapshot}});
        reviews["reviews"][name] = json!({"tool_name":"review_sections","success":true,"data":{
            "agent":"EDITOR","scope":name,"review_id":format!("review:{author}"),"input_snapshot":snapshot,
            "fact_receipts":{name:format!("facts:{author}")}}});
    }
    (sections, references, reviews)
}

pub(super) fn reply(
    arguments: &Value,
    (sections, references, reviews): (&Value, &Value, &Value),
) -> Value {
    let sections_requested: Vec<_> = arguments["sections"]
        .as_str()
        .unwrap()
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    let facts: serde_json::Map<String, Value> = imp::SECTIONS
        .iter()
        .map(|name| {
            (
                name.to_string(),
                reviews["facts"][*name]["data"]["check_id"].clone(),
            )
        })
        .collect();
    let checks: serde_json::Map<String, Value> = imp::SECTIONS
        .iter()
        .map(|name| {
            (
                name.to_string(),
                reviews["reviews"][*name]["data"]["review_id"].clone(),
            )
        })
        .collect();
    json!({"tool_name":"combine_sections","success":true,"data":{"agent":"EDITOR","status":"combined",
        "sections":sections_requested,"artifact_id":"combined:actual","input_snapshot":{"sections":sections,"references":references},
        "fact_receipts":facts,"review_receipts":checks}})
}

#[test]
fn paper_publication_requires_current_positive_reviews_for_all_three_sections() {
    let (sections, references, reviews) = inputs();
    for order in [
        "section_a,section_b,section_c",
        "section_a,section_c,section_b",
        "section_b,section_a,section_c",
        "section_b,section_c,section_a",
        "section_c,section_a,section_b",
        "section_c,section_b,section_a",
        " section_c,, section_a, section_b ",
    ] {
        let arguments = json!({"sections":order});
        let snapshot = imp::publication(&arguments, (&sections, &references), &reviews).unwrap();
        imp::published(
            &reply(&arguments, (&sections, &references, &reviews)),
            (&arguments, &snapshot),
            &reviews,
        )
        .unwrap();
    }
    for change in 0..12 {
        let (mut sections, mut references, mut reviews) = inputs();
        let mut arguments = json!({"sections":"section_a,section_b,section_c"});
        match change {
            0 => {
                reviews["facts"]
                    .as_object_mut()
                    .unwrap()
                    .remove("section_b");
            }
            1 => reviews["facts"]["section_b"]["success"] = json!(false),
            2 => sections["section_b"]["content"] = json!("changed after review"),
            3 => references["section_b"]["id"] = json!("new references"),
            4 => reviews["reviews"]["section_b"]["success"] = json!(false),
            5 => {
                reviews["reviews"]["section_b"]["data"]["fact_receipts"]["section_b"] =
                    json!("another check")
            }
            6 => reviews["reviews"]["section_b"]["data"]["input_snapshot"] = Value::Null,
            7 => {
                reviews["facts"]["section_b"]["data"]["check_id"] = json!("facts:0");
                reviews["reviews"]["section_b"]["data"]["fact_receipts"]["section_b"] =
                    json!("facts:0");
            }
            8 => arguments["sections"] = json!("section_a,section_b,section_b"),
            9 => {
                sections["section_b"]["id"] = json!("document:0");
                reviews["facts"]["section_b"]["data"]["input_snapshot"]["sections"]["section_b"] =
                    sections["section_b"].clone();
                reviews["reviews"]["section_b"]["data"]["input_snapshot"]["sections"]["section_b"] =
                    sections["section_b"].clone();
            }
            10 => {
                references.as_object_mut().unwrap().remove("section_b");
            }
            _ => sections["another_section"] = json!({}),
        }
        assert!(
            imp::publication(&arguments, (&sections, &references), &reviews).is_err(),
            "change={change}"
        );
    }
    let arguments = json!({"sections":"section_a,section_b,section_c"});
    let snapshot = imp::publication(&arguments, (&sections, &references), &reviews).unwrap();
    for change in 0..6 {
        let mut actual = reply(&arguments, (&sections, &references, &reviews));
        match change {
            0 => {
                actual["data"]["input_snapshot"]["sections"]["section_a"]["id"] = json!("old draft")
            }
            1 => actual["data"]["fact_receipts"]["section_a"] = json!("old facts"),
            2 => actual["data"]["review_receipts"]["section_a"] = json!("old review"),
            3 => actual["success"] = json!(false),
            4 => actual["data"]["sections"] = json!(["section_a"]),
            _ => actual["violations"] = json!([{"type":"unreviewed_document"}]),
        }
        assert!(imp::published(&actual, (&arguments, &snapshot), &reviews).is_err());
    }
}
