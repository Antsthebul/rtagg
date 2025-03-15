use std::fs::read_dir;

use regex::Regex;
use serde_json::Value;

use crate::{
    readers::read_file,
    utils::{compile_path, strip_quotes},
};

pub fn json_from_file(file: &str) -> Value {
    let json_contents = read_file(file);
    serde_json::from_str(json_contents.as_str()).unwrap()
}
/// Recursively modifies a YAML doucment
pub fn build_section_json(json: &mut Value, current_section: Option<&str>) {
    if let Some(arr) = json.as_array_mut() {
        let mut new_vec = vec![];

        for item in arr.into_iter() {
            if let Some(item) = item.as_str() {
                let (updated, data) = parse_function_for_section(current_section, item);

                if updated {
                    new_vec.extend(data);
                } else {
                    new_vec.push(item.into());
                }
            } else if item.is_object() {
                build_section_json(item, None);
            } else {
                new_vec.push(item.clone());
            }
        }
        *arr = new_vec;
    } else if let Some(mapping) = json.as_object_mut() {
        for (_key, value) in mapping.iter_mut() {
            let section_name = _key.as_str();

            if let Some(string_val) = value.as_str() {
                let (updated, data) = parse_function_for_section(Some(section_name), string_val);
                if updated && data.len() > 1 {
                    panic!(
                        "Use list syntax for add multiple object for section. '{}'",
                        section_name
                    )
                } else if updated && data.len() > 0 {
                    *value = data
                        .iter()
                        .next()
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .to_owned()
                        .into();
                }
            } else if value.is_array() {
                build_section_json(value, Some(section_name));
            } else if value.is_object() {
                build_section_json(value, Some(section_name));
            }
        }
    }
}

/// Returns a list of YAML objects from the directory provided.
/// Arg params can be "*" to indicate use directory to match section name
fn lookup(section_name: Option<&str>, arg: &str) -> Vec<Value> {
    let mut seq = vec![];
    let loc = match arg {
        "'*'" => {
            if let Some(name) = section_name {
                name
            } else {
                panic!("'*' cannot be used in this context")
            }
        }
        x => &strip_quotes(x),
    };

    for dir in read_dir(loc).expect(&format!("check permissions/existence of {loc}")) {
        let dir = dir.unwrap();
        let v = json_from_file(dir.path().to_str().unwrap());
        if v.is_array() {
            seq.extend(v.as_array().into_iter().flat_map(|f| f.to_owned()));
        } else {
            seq.push(v.to_owned());
        }
    }
    seq
}

/// If section is None, this indicates 'root' level
/// and will cause an error if '*' (auto-detect) is padded to a function
fn parse_function_for_section(section_name: Option<&str>, _str: &str) -> (bool, Vec<Value>) {
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();
    let mut updated = false;
    let mut seq = vec![];
    match func_regex.captures(_str.trim()) {
        Some(v) => {
            updated = true;
            let func = v.get(1).unwrap();
            let arg = v.get(2).unwrap().as_str();
            let mut args = arg.split(",");

            let lookups = match func.as_str() {
                "lookup" => lookup(section_name, args.next().unwrap()),
                "lookup_file" => lookup_file(args.next().unwrap(), args.next().unwrap()),
                x => panic!("{x} is not a function."),
            };
            seq.extend(lookups.into_iter());
        }
        None => (),
    }
    (updated, seq)
}

/// Used to lookup single object from file and return YAML
fn lookup_file(dir: &str, file_name: &str) -> Vec<Value> {
    let path = compile_path(dir, file_name);
    // Just like template, we read in first
    let y = json_from_file(path.to_str().unwrap());
    if !y.is_array() {
        vec![y.to_owned()]
    } else {
        y.as_array().unwrap().to_owned()
    }
}
