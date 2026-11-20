//! A scripted editor for the typed review and assembly files.
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

fn main() {
    let material = fs::read_to_string("/service/review_material").unwrap();
    assert!(material.contains("draft:3:current"));
    assert!(material.contains("fact_receipts"));
    for chapter in ["CHAPTER_3", "CHAPTER_1", "CHAPTER_2"] {
        let path = format!("/service/review_sections/{chapter}");
        {
            let mut review = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .unwrap();
            write!(review, r#"{{"section_name":"{chapter}"}}"#).unwrap();
        }
        let reviewed = fs::read_to_string(path).unwrap();
        assert!(reviewed.contains("\"success\":true"));
        assert!(reviewed.contains("fact_receipts"));
        assert!(reviewed.contains("draft:3:current"));
    }
    {
        let mut combine = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/combine_sections")
            .unwrap();
        combine
            .write_all(br#"{"sections":"CHAPTER_3, CHAPTER_1, CHAPTER_2"}"#)
            .unwrap();
    }
    let combined = fs::read_to_string("/service/combine_sections").unwrap();
    assert!(combined.contains("\"artifact_id\":\"publication:1\""));
    assert!(combined.contains("\"sections\":[\"CHAPTER_3\",\"CHAPTER_1\",\"CHAPTER_2\"]"));
}
