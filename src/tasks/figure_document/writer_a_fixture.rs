mod writer_fixture_io;
use std::{fs::OpenOptions, io::Write};
use writer_fixture_io::operation;

fn main() {
    {
        let mut invalid = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/update_references/CHAPTER_1")
            .unwrap();
        invalid
            .write_all(br#"{"section_name":"CHAPTER_3"}"#)
            .unwrap();
    }
    let first_references = operation(
        "/service/update_references/CHAPTER_1",
        r#"{"section_name":"CHAPTER_1"}"#,
    );
    assert!(first_references.contains("refs:CHAPTER_1"));
    let second_draft = operation(
        "/service/write_section/CHAPTER_2",
        r#"{"section_name":"CHAPTER_2","content_notes":"Writer A's methodology"}"#,
    );
    assert!(second_draft.contains("Writer A's methodology"));
    let snapshot = operation("/service/read_section", r#"{"section_name":"CHAPTER_3"}"#);
    assert!(snapshot.contains("\"section\":\"CHAPTER_3\""));
    let first_draft = operation(
        "/service/write_section/CHAPTER_1",
        r#"{"section_name":"CHAPTER_1","content_notes":"Writer A's introduction"}"#,
    );
    assert!(first_draft.contains("Writer A's introduction"));
    let second_references = operation(
        "/service/update_references/CHAPTER_2",
        r#"{"section_name":"CHAPTER_2"}"#,
    );
    assert!(second_references.contains("refs:CHAPTER_2"));
}
