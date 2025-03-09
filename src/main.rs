use std::{
    fs::{read_dir, File},
    io::{Read, Write}, path::Path,
};

use regex::Regex;
use serde_yaml::Value;
use clap::Parser;

/// YAML template aggregator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Root YAML template file
    template: String,

    #[arg(short, long, default_value_t = String::from("output.yaml"))]
    /// Output name of generated YAML file
    output:String,
}


fn main() {
    let args = Args::parse();

    let template_file = args.template;
    let mut result = String::new();
    // Read template
    let mut file = File::open(&template_file)
        .expect(&format!("check permissions/existence of '{template_file}'"));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    // Serialize it to a YAML mapping.
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&contents).unwrap();
    
    result = build_section(yaml, result, 0);

    let mut dest = File::create( args.output).unwrap();
    println!("Final => {:?}",&result);

    dest.write_all(&mut result.as_ref()).unwrap();

}

fn yaml_from_file<P>(file:P, lvl:u8) ->  Value
    where P:AsRef<Path> {
    let mut inner_file = File::open(file).unwrap();
    let mut inner_contents = String::new();
    inner_file.read_to_string(&mut inner_contents).unwrap();
    serde_yaml::from_str(&inner_contents).unwrap()
}

/// Returns a list of YAMl objects from the directory provided.
/// Arg params can be "*" to indicate use directory to match section name
fn lookup(section_name:&str, arg:&str, lvl:u8)->Vec<Value>{
    // println!("Level is {} for section_name {}", lvl, section_name);
    let mut seq = vec![];
    let loc = match arg{
        "'*'" => section_name,
        x=>x.strip_prefix("'").unwrap().strip_suffix("'").unwrap()
    };
    
    for dir in read_dir(loc).expect(&format!("check permissions/existence of {loc}")) {
        let dir = dir.unwrap();
        let v = yaml_from_file(dir.path(), lvl);
        if v.is_sequence(){
            seq.extend(v.as_sequence().into_iter().flat_map(|f|f.to_owned()));

        }else{
            seq.push(v.to_owned());
        }      
    };
    seq
}

/// Used to lookup single object from file and return YAML
fn lookup_file(dir:&str, file_name:&str, lvl:u8)->Value{
    let mut result = String::new();
    let complete_path = &format!("./{dir}/{file_name}");
    let path = Path::new(complete_path);
    // Just like template, we read in first
    let v = yaml_from_file(path, lvl);
    // build out any necessary params
    // result = build_section(v, result);
    serde_yaml::from_str(&result).unwrap()
}

fn build_section(yaml:Value, mut result:String, lvl:u8)-> String{
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();
   
    for (key, section) in yaml.as_mapping().iter().flat_map(|d| d.iter()) {
        let section_name = key.as_str().unwrap();
        result.push_str(&format! {"{section_name}:\n"});
        if section.is_mapping() {
            result = build_section(section.clone(), result.clone(), lvl +1);
        } else if section.is_sequence() {
            // Iterate over items
            let mut seq = vec![];
            for item in section.as_sequence().iter().flat_map(|f| f.iter()) {
                // if str then check if its a function
                if item.is_string() {
                    match func_regex.captures(item.as_str().unwrap()){
                        Some(v)=>{
                            let func = v.get(1).unwrap();
                            let arg = v.get(2).unwrap().as_str();
                            let mut args = arg.split(",");
                            let lookups = match func.as_str(){
                                "lookup"=>lookup(section_name, args.next().unwrap(), lvl),
                                "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap(), lvl)],
                                x=>panic!("{x} is not a function.")
                            };
                            seq.extend(lookups.into_iter());
        
                        },
                        None=>{seq.push(item.clone())}
                    }
                } else {
                    seq.push(item.clone());
                }
            }
            result.push_str(&serde_yaml::to_string(&seq).unwrap());
        }
    }
    result

}
