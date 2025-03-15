use std::{fs::File, io::Read};

/// Reads a YAML file and returns a YAML object
pub fn read_file(file: &str) -> String {
    let mut inner_file = File::open(file).expect(&format!("check permissions for {file}"));
    let mut inner_contents = String::new();
    inner_file.read_to_string(&mut inner_contents).unwrap();
    inner_contents
}
