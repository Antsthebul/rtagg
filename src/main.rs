use std::{
    fs::{read_dir, File},
    io::{Read, Write}, path::Path,
};

use regex::Regex;
use clap::Parser;
use saphyr::{LoadableYamlNode, Yaml, YamlEmitter, YamlLoader};

#[cfg(test)]
mod tests{
    use std::collections::BTreeMap;

    use saphyr::Hash;

    use super::*;

    fn get_test_yaml_string(file_name:&str)-> String{
        let mut inner_file = File::open(file_name).expect(&format!("check permissions for '{file_name}'"));
        let mut yaml_as_str = String::new();
        inner_file.read_to_string(&mut yaml_as_str).unwrap();
        yaml_as_str
    }

    #[test]
    fn test_yaml_is_create(){
        let yaml_as_str = get_test_yaml_string("./src/test__no_inject.yaml");

        let docs =  Yaml::load_from_str(&yaml_as_str).unwrap();
        let doc = &docs[0];

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); 

        out_str = out_str.strip_prefix("---\n").unwrap().to_string();
        assert_eq!(out_str, yaml_as_str)
    }
    
    #[test]
    fn test_can_inject_yaml(){
        let yaml_template = get_test_yaml_string("./src/test__inject_simple_template.yaml");
        let child_template = get_test_yaml_string("./src/test__inject_simple__child_template.yaml");

        let expected_template = get_test_yaml_string("./src/test__inject_simple_result.yaml");

        let mut  docs =  Yaml::load_from_str(&yaml_template).unwrap();
        let doc = &mut docs[0];

        let mut ct_docs = Yaml::load_from_str(&child_template).unwrap();
        let ct_doc = &mut ct_docs[0];
        let ct_map = Yaml::Hash(ct_doc.as_hash().unwrap().clone());

        for (_key, value) in doc.as_mut_hash().unwrap().iter_mut(){
            if let Some(array) = value.as_mut_vec(){
                array.push(ct_map.clone());
            }
        }



        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); 
        // let mut dest = File::create("test.yaml").unwrap();
        // dest.write_all(&mut out_str.as_ref()).unwrap();
        out_str = out_str.strip_prefix("---\n").unwrap().to_string();
        assert_eq!(out_str, expected_template)

    }

    #[test]
    fn test_yaml_is_injected(){
        let yaml_template = get_test_yaml_string("./example/main.yaml");
        let expected_yaml = get_test_yaml_string("./src/test__inject.yaml");

        let docs =  Yaml::load_from_str(&yaml_template).unwrap();
        let doc = &docs[0];

        if (doc.is_hash()){
            for (key, value) in doc.as_hash().into_iter().flat_map(|f|f){
                println!("\ndata => {:?}", key);
                if value.is_array(){
                    for item in value.as_vec(){
                        // if item.is_string() {
                        //     match func_regex.captures(item.as_str().unwrap()){
                        //         Some(v)=>{
                        //             let func = v.get(1).unwrap();
                        //             let arg = v.get(2).unwrap().as_str();
                        //             let mut args = arg.split(",");
           
                        //             let lookups = match func.as_str(){
                        //                 "lookup"=>lookup(section_name, args.next().unwrap()),
                        //                 "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap())],
                        //                 x=>panic!("{x} is not a function.")
                        //             };
                        //             seq.extend(lookups.into_iter());
                
                        //         },
                        //         None=>{seq.push(item.clone())}
                        //     }
                        // } else {
                        //     seq.push(item.clone());
                        // }
                    }
                }
            }
        }
        assert_eq!(yaml_template, expected_yaml)

    }

    #[test]
    fn test_build_section(){
        let yaml_template = get_test_yaml_string("./example/main.yaml");
        let docs =  Yaml::load_from_str(&yaml_template).unwrap();
        let doc = &docs[0];
 

        build_body_v2(doc.to_owned());
    }
}

fn build_new_array(yaml:&Yaml, section_name:Option<&str>)->Vec<Yaml>{
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();

    let mut new_arr = vec![];
    for item in yaml.as_vec().unwrap(){
        if item.is_string() {
            match func_regex.captures(item.as_str().unwrap()){
                Some(v)=>{
                    let func = v.get(1).unwrap();
                    let arg = v.get(2).unwrap().as_str();
                    let mut args = arg.split(",");

                    let lookups = match func.as_str(){
                        "lookup"=>lookup(section_name, args.next().unwrap()),
                        "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap())],
                        x=>panic!("{x} is not a function.")
                    };
                    new_arr.extend(lookups.into_iter());

                },
                None=>{new_arr.push(item.clone())}
            }
        } else {
            new_arr.push(item.clone());
        }
    }
    new_arr
}

fn build_body_v2(mut yaml:&Yaml, current_section:Option<&str>){
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();
 
    if let Some(arr) = yaml.as_mut_vec(){
        let mut ix = 0;

        while ix < arr.len(){
            
            if let Some(item) = arr[ix].as_str(){
                match func_regex.captures(item){
                    Some(v)=>{
                        let func = v.get(1).unwrap();
                        let arg = v.get(2).unwrap().as_str();
                        let mut args = arg.split(",");
    
                        let lookups = match func.as_str(){
                            "lookup"=>lookup(current_section, args.next().unwrap()),
                            "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap())],
                            x=>panic!("{x} is not a function.")
                        };

                        arr.splice(ix..=ix, lookups);

                    },
                    None=>()
                };
            }
            ix += 1;
        }
    }
    // if let Some(mapping) = yaml.as_mut_hash() {
    //     for (_key, value) in mapping.iter_mut(){
    //         let section_name = _key.as_str().unwrap();
    //         if value.is_array(){
    //             let new_arr = build_new_array(&value, Some(section_name));
    //             *value = Yaml::Array(new_arr);
    //         }else if value.is_hash(){
    //             build_body_v2(Yaml::Hash(value.as_mut_hash().unwrap().to_owned()));
    //             *value = Yaml::Hash(value.as_hash().unwrap().to_owned());
    //         }
            
    //     }
    // } else if let Some(array) = yaml.as_mut_vec(){
    //     for item in array{

    //     }
    //     let new_arr = build_new_array(&Yaml::Array(array.to_vec()), None);
    //     *array = vec![Yaml::Array(new_arr)];
        
    // } else if yaml.is_string(){
    //     let converted = parse_function_for_section(None, _str);
    //     if converted.0{

    //         *yaml = converted.1; 
    //     }
    // }
}
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
    let yamls = Yaml::load_from_str(&contents).unwrap();
    let yaml = &yamls[0];

    result = build_section(yaml.clone(), result);

    let mut dest = File::create( args.output).unwrap();
    println!("Final => {:?}",&result);

    dest.write_all(&mut result.as_ref()).unwrap();

}

fn yaml_from_file<P>(file:P) ->  Yaml
    where P:AsRef<Path> {
    let mut inner_file = File::open(file).unwrap();
    let mut inner_contents = String::new();
    inner_file.read_to_string(&mut inner_contents).unwrap();
    Yaml::load_from_str(&inner_contents).unwrap()[0].clone()
}

/// Returns a list of YAMl objects from the directory provided.
/// Arg params can be "*" to indicate use directory to match section name
fn lookup(section_name:Option<&str>, arg:&str)-> Vec<Yaml>{
    // println!("Level is {} for section_name {}", lvl, section_name);
    let mut seq = vec![];
    let loc = match arg{
        "'*'" => {
            if let Some(name) = section_name{
                name
            }else{
                panic!("'*' cannot be used in this context")
            }},
        x=>x.strip_prefix("'").unwrap().strip_suffix("'").unwrap()
    };
    
    for dir in read_dir(loc).expect(&format!("check permissions/existence of {loc}")) {
        let dir = dir.unwrap();
        let v = yaml_from_file(dir.path());
        // if v.is_sequence(){
        //     seq.extend(v.as_sequence().into_iter().flat_map(|f|f.to_owned()));

        // }else{
        //     seq.push(v.to_owned());
        // }      
    };
    seq
}

/// If section is None, this indicates 'root' level
/// and will cause an error if '*' (auto-detect) is padded to a function
fn parse_function_for_section(section_name:Option<&str>,_str:&str)-> (bool,Vec<Yaml>){
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();
    let mut updated = false;
    let mut seq =  vec![];
    match func_regex.captures(_str){
        Some(v)=>{
            updated = true;
            let func = v.get(1).unwrap();
            let arg = v.get(2).unwrap().as_str();
            let mut args = arg.split(",");

            let lookups = match func.as_str(){
                "lookup"=>lookup(section_name, args.next().unwrap()),
                "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap())],
                x=>panic!("{x} is not a function.")
            };
            seq.extend(lookups.into_iter());

        },
        None=>{()}
    }
    (updated, seq)
}
/// Used to lookup single object from file and return YAML
fn lookup_file(dir:&str, file_name:&str) -> Yaml{
    let mut result = String::new();
    let complete_path = &format!("./{dir}/{file_name}");
    let path = Path::new(complete_path);
    // Just like template, we read in first
    yaml_from_file(path)
    // build out any necessary params
    // result = build_section(v, result);

}



fn build_section(yaml:Yaml, mut result:String,)-> String{
    let func_regex = Regex::new(r"(\w+)\((.+)\)").unwrap();
    // let mut emitter = YamlEmitter::new(&mut out_str);
    for (key, section) in yaml.as_hash().iter().flat_map(|d| d.iter()) {
        let section_name = key.as_str().unwrap();
        println!("sec => {}", section_name);
        result.push_str(&format! {"{section_name}:\n"});
        if section.is_hash() {
            result = build_section(section.clone(), result.clone());
        } else if section.is_array() {
            // Iterate over items
            let mut seq = vec![];
            for item in section.as_vec().iter().flat_map(|f| f.iter()) {
                // if str then check if its a function
                if item.is_string() {
                    match func_regex.captures(item.as_str().unwrap()){
                        Some(v)=>{
                            let func = v.get(1).unwrap();
                            let arg = v.get(2).unwrap().as_str();
                            let mut args = arg.split(",");
   
                            let lookups = match func.as_str(){
                                "lookup"=>lookup(Some(section_name), args.next().unwrap()),
                                "lookup_file"=>vec![lookup_file(args.next().unwrap(), args.next().unwrap())],
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
       
        }
    }
    result

}
