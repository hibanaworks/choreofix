//! Shared file calls for the two scripted Wasm writers.
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

pub fn operation(path: &str, json: &str) -> String {
    {
        let mut request = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        request.write_all(json.as_bytes()).unwrap();
    }
    fs::read_to_string(path).unwrap()
}
