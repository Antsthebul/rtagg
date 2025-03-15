use serde_json::Value;

use crate::build_json::build_section_json;

use std::{fs::File, 
    io::Read,
    path::PathBuf};


/// Helper function to return file as a string
fn get_test_json_string(file_name: &str) -> String {
    let mut path = PathBuf::new();
    path.push("src/tests");
    path.push(file_name);
    let mut inner_file = File::open(path).expect(&format!("check permissions for '{file_name}'"));
    let mut json_as_str = String::new();
    inner_file.read_to_string(&mut json_as_str).unwrap();
    json_as_str
}

/// Returns a JSON string with whitespace removed
fn simplify_json_string(text:&str) -> String{
    serde_json::to_string(&serde_json::from_str::<Value>(text).unwrap()).unwrap()
}


#[test]
fn test_build_section_list() {
    let json_template = get_test_json_string("test__inject_list_template.json");
    let mut expected_output = get_test_json_string("test__inject_list_result.json");
    expected_output = simplify_json_string(&expected_output);
    let json = &mut serde_json::from_str(&json_template).unwrap();

    build_section_json(json, None);
     
    let out_str = serde_json::to_string(json).unwrap();
    assert_eq!(expected_output, out_str);
}

#[test]
fn test_build_section_map() {
    let json_template = get_test_json_string("test__inject_map_template.json");
    let mut expected_output = get_test_json_string("test__inject_map_result.json");
    expected_output = simplify_json_string(&expected_output);

    let json = &mut serde_json::from_str(&json_template).unwrap();

    build_section_json(json, None);

    let out_str = serde_json::to_string(json).unwrap();

    assert_eq!(expected_output, out_str);
}
