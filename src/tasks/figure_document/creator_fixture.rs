//! A Wasm fixture that supplies the same JSON an interactive guest can choose.
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

fn main() {
    {
        let mut create = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/create_figure/CHAPTER_1")
            .unwrap();
        create
            .write_all(br#"{"description":"Figure from the guest"}"#)
            .unwrap();
    }
    let created = fs::read_to_string("/service/create_figure/CHAPTER_1").unwrap();
    let figure = created
        .split_once("\"figure_id\":\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    {
        let mut insert = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/service/insert_figure/CHAPTER_1")
            .unwrap();
        write!(
            insert,
            r#"{{"section_name":"CHAPTER_1","figure_id":"{figure}"}}"#
        )
        .unwrap();
    }
    let inserted = fs::read_to_string("/service/insert_figure/CHAPTER_1").unwrap();
    assert!(inserted.contains(figure));
    assert!(inserted.contains("CHAPTER_1"));
    assert!(inserted.contains("\"success\":true"));
}
