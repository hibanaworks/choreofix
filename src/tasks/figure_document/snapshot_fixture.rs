//! A writer exercising the actual ChoreoFS snapshot file.
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

fn snapshot() -> String {
    {
        let mut query = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/read_section")
            .unwrap();
        query.write_all(br#"{"section_name":"CHAPTER_3"}"#).unwrap();
    }
    fs::read_to_string("/service/read_section").unwrap()
}

fn main() {
    let pending = snapshot();
    assert!(pending.contains("\"success\":false"));
    assert!(pending.contains("\"artifact\":null"));
    let first = (0..16)
        .map(|_| snapshot())
        .find(|value| value.contains("\"id\":\"draft:1\""))
        .expect("completed first version");
    let current = (0..16)
        .map(|_| snapshot())
        .find(|value| value.contains("\"id\":\"draft:2\""))
        .expect("completed revision");
    assert!(first.contains("\"content\":\"original text\""));
    assert!(current.contains("\"content\":\"revised wording\""));
    assert_ne!(first, current);

    let version = current
        .split_once("\"id\":\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    let quote = current
        .split_once("\"content\":\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    {
        let mut draft = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/write_section")
            .unwrap();
        write!(
            draft,
            r#"{{"section_name":"CHAPTER_1","content_notes":"Chapter 3 ({version}): {quote}"}}"#
        )
        .unwrap();
    }
    let written = fs::read_to_string("/service/write_section").unwrap();
    assert!(written.contains("\"success\":true"));
    assert!(written.contains("draft:2"));
    assert!(written.contains("revised wording"));
}
