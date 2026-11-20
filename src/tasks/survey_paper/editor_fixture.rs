//! Scripted Wasm guest for the editor ChoreoFS boundary.
use std::{fs::OpenOptions, io::Write};

fn main() {
    let admitted = std::fs::read_to_string("/service/accepted_sections").unwrap();
    assert!(admitted.contains("\"section_d\""));
    assert!(admitted.contains("\"review_receipts\""));
    {
        let mut decision = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/make_editorial_decision")
            .unwrap();
        decision
            .write_all(br#"{"review_summary":"Use the four approved drafts."}"#)
            .unwrap();
    }
    let actual = std::fs::read_to_string("/service/make_editorial_decision").unwrap();
    assert!(actual.contains("editorial:1"));
    {
        let mut combine = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/combine_sections")
            .unwrap();
        combine
            .write_all(br#"{"sections":"section_d,section_b, section_a, section_c"}"#)
            .unwrap();
    }
    let actual = std::fs::read_to_string("/service/combine_sections").unwrap();
    assert!(actual.contains("combined:2"));
}
