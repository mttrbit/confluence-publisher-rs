use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub fn read_file_content(path: &str) -> std::io::Result<String> {
    let file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    match buf_reader.read_to_string(&mut content) {
        Ok(_) => Ok(content),
        Err(e) => Err(e.into()),
    }
}
