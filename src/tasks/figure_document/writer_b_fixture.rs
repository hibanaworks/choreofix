mod writer_fixture_io;
use writer_fixture_io::operation;

fn main() {
    let references = operation(
        "/service/update_references/CHAPTER_3",
        r#"{"section_name":"CHAPTER_3"}"#,
    );
    assert!(references.contains("refs:CHAPTER_3"));
    let draft = operation(
        "/service/write_section/CHAPTER_3",
        r#"{"section_name":"CHAPTER_3","content_notes":"Writer B's results"}"#,
    );
    assert!(draft.contains("Writer B's results"));
}
