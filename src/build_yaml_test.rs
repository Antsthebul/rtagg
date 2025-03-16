use saphyr::{Yaml, YamlEmitter};

use crate:: build_yaml::build_section_yaml;

use std::{fs::File, 
    io::{Read, Write},
    path::PathBuf};


/// Helper function to return file as a string
fn get_test_yaml_string(file_name: &str) -> String {
    let mut path = PathBuf::new();
    path.push("src/tests/yaml_files");
    path.push(file_name);
    let mut inner_file = File::open(path).expect(&format!("check permissions for '{file_name}'"));
    let mut yaml_as_str = String::new();
    inner_file.read_to_string(&mut yaml_as_str).unwrap();
    yaml_as_str
}


#[test]
fn test_build_section_list() {
    let yaml_template = get_test_yaml_string("test__inject_list_template.yaml");
    let expected_output = get_test_yaml_string("test__inject_list_result.yaml");

    let mut docs = Yaml::load_from_str(&yaml_template).unwrap();
    let doc = &mut docs[0];

    build_section_yaml(doc, None);
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(doc).unwrap();

    out_str = out_str.strip_prefix("---\n").unwrap().to_string();

    assert_eq!(expected_output, out_str);
}

#[test]
fn test_build_section_map() {
    let yaml_template = get_test_yaml_string("test__inject_map_template.yaml");
    let expected_output = get_test_yaml_string("test__inject_map_result.yaml");

    let mut docs = Yaml::load_from_str(&yaml_template).unwrap();
    let doc = &mut docs[0];

    build_section_yaml(doc, None);

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(doc).unwrap();

    out_str = out_str.strip_prefix("---\n").unwrap().to_string();

    assert_eq!(expected_output, out_str);
}
