use clap::builder::TypedValueParser as _;
use std::{
    fmt::Display,
    fs::{read_dir, File},
    io::{Read, Write},
    path::Path,
    str::FromStr,
};

use clap::Parser;
use regex::Regex;
use saphyr::{Yaml, YamlEmitter};

#[derive(Debug, Clone)]
enum Extension {
    JSON,
    YAML,
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::JSON => "JSON",
            Self::YAML => "YAML",
        };
        s.fmt(f)
    }
}

impl FromStr for Extension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JSON" => Ok(Self::JSON),
            "YAML" => Ok(Self::YAML),
            _ => Err(format!("Unknown extension type {s}")),
        }
    }
}

#[cfg(test)]
mod tests;

/// YAML template aggregator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Root template file
    template: String,

    #[arg(short, long, value_parser = clap::builder::PossibleValuesParser::new(["JSON", "YAML"])
        .map(|s|s.parse::<Extension>().unwrap()))]
    /// File type being parsed and generated
    extension: Extension,

    #[arg(short, long, default_value_t = String::from("output.yaml"))]
    /// Output name of generated YAML file
    output: String,
}

fn main() {
    let args = Args::parse();

    let template_file = args.template;
    
    // Read template
    let mut file = File::open(&template_file)
        .expect(&format!("check permissions/existence of '{template_file}'"));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    // Serialize it to a YAML mapping.
    let mut yamls = Yaml::load_from_str(&contents).unwrap();
    let yaml = &mut yamls[0];

    build_section(yaml, None);

    let mut dest = File::create(args.output).unwrap();
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(yaml).unwrap();

    out_str = out_str.strip_prefix("---\n").unwrap().to_string();

    dest.write_all(out_str.as_bytes()).unwrap();
    println!("Success!")
}

/// Reads a YAML file and returns a YAML object
fn yaml_from_file(file: &str) -> Yaml {
    let mut inner_file = File::open(file).expect(&format!("check permissions for {file}"));
    let mut inner_contents = String::new();
    inner_file.read_to_string(&mut inner_contents).unwrap();
    Yaml::load_from_str(&inner_contents).unwrap()[0].clone()
}

/// Returns a list of YAML objects from the directory provided.
/// Arg params can be "*" to indicate use directory to match section name
fn lookup(section_name: Option<&str>, arg: &str) -> Vec<Yaml> {
    let mut seq = vec![];
    let loc = match arg {
        "'*'" => {
            if let Some(name) = section_name {
                name
            } else {
                panic!("'*' cannot be used in this context")
            }
        }
        x => x.strip_prefix("'").unwrap().strip_suffix("'").unwrap(),
    };

    for dir in read_dir(loc).expect(&format!("check permissions/existence of {loc}")) {
        let dir = dir.unwrap();
        let v = yaml_from_file(dir.path().to_str().unwrap());
        if v.is_array() {
            seq.extend(v.as_vec().into_iter().flat_map(|f| f.to_owned()));
        } else {
            seq.push(v.to_owned());
        }
    }
    seq
}

/// If section is None, this indicates 'root' level
/// and will cause an error if '*' (auto-detect) is padded to a function
fn parse_function_for_section(section_name: Option<&str>, _str: &str) -> (bool, Vec<Yaml>) {
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
fn lookup_file(mut dir: &str, mut file_name: &str) -> Vec<Yaml> {
    dir = dir
        .trim()
        .strip_prefix("'")
        .unwrap()
        .strip_suffix("'")
        .unwrap();
    file_name = file_name
        .trim()
        .strip_prefix("'")
        .unwrap()
        .strip_suffix("'")
        .unwrap();
    let complete_path = &format!("{dir}/{file_name}");
    let path = Path::new(complete_path);
    // Just like template, we read in first
    let y = yaml_from_file(path.to_str().unwrap());
    if !y.is_array() {
        vec![y.to_owned()]
    } else {
        y.as_vec().unwrap().to_owned()
    }
}

/// Recuirsively modifies a YAML doucment
fn build_section(yaml: &mut Yaml, current_section: Option<&str>) {
    if let Some(arr) = yaml.as_mut_vec() {
        let mut new_vec = vec![];

        for item in arr.into_iter() {
            if let Some(item) = item.as_str() {
                let (updated, data) = parse_function_for_section(current_section, item);

                if updated {
                    new_vec.extend(data);
                } else {
                    new_vec.push(Yaml::String(item.to_owned()));
                }
            } else {
                new_vec.push(item.clone());
            }
        }
        *arr = new_vec;
    } else if let Some(mapping) = yaml.as_mut_hash() {
        for (_key, value) in mapping.iter_mut() {
            let section_name = _key.as_str().unwrap();
            if let Some(string_val) = value.as_str() {
                let (updated, data) = parse_function_for_section(Some(section_name), string_val);
                if updated && data.len() > 1 {
                    panic!(
                        "Use list syntax for add multiple object for section. '{}'",
                        section_name
                    )
                } else if updated && data.len() > 0 {
                    *value = Yaml::Hash(data.iter().next().unwrap().as_hash().unwrap().to_owned());
                }
            } else if value.is_array() {
                build_section(value, Some(section_name));
            } else if value.is_hash() {
                build_section(value, Some(section_name));
            }
        }
    }
}
