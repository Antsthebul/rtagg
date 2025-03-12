use std::{
    fs::{read_dir, File},
    io::{Read, Write}, path::Path, thread::panicking,
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
        assert_eq!(out_str, yaml_as_str);
    }
    


    // #[test]
    // fn test_yaml_is_injected(){
    //     let yaml_template = get_test_yaml_string("./example/main.yaml");
    //     let expected_yaml = get_test_yaml_string("./src/test__inject.yaml");

    //     let docs =  Yaml::load_from_str(&yaml_template).unwrap();
    //     let doc = &docs[0];

    //     if (doc.is_hash()){
    //         for (key, value) in doc.as_hash().into_iter().flat_map(|f|f){
    //             println!("\ndata => {:?}", key);
    //             if value.is_array(){
    //                 for item in value.as_vec(){
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
        //             }
        //         }
        //     }
        // }
        // assert_eq!(yaml_template, expected_yaml)

    // }

    #[test]
    fn test_build_section_list(){
        let yaml_template = get_test_yaml_string("./src/test__inject_list_template.yaml");
        let expected_output = get_test_yaml_string("./src/test__inject_list_result.yaml");
        
        let mut docs =  Yaml::load_from_str(&yaml_template).unwrap();
        let doc = &mut docs[0];
 

        build_body_v2(doc, None);
        println!("omg {:?}", doc);
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); 

        out_str = out_str.strip_prefix("---\n").unwrap().to_string();

        
        let mut des = File::create("lis_result.yaml").unwrap();
        des.write_all(out_str.as_bytes()).unwrap();
        assert_eq!(expected_output, out_str);

    }

    #[test]
    fn test_build_section_map(){
        let yaml_template = get_test_yaml_string("./src/test__inject_map_template.yaml");
        let expected_output = get_test_yaml_string("./src/test__inject_map_result.yaml");
        
        let mut docs =  Yaml::load_from_str(&yaml_template).unwrap();
        let doc = &mut docs[0];
 

        build_body_v2(doc, None);

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); 

        out_str = out_str.strip_prefix("---\n").unwrap().to_string();

        let mut des = File::create("mapper_result.yaml").unwrap();
        des.write_all(out_str.as_bytes()).unwrap();

        assert_eq!(expected_output, out_str);

    }
}


fn build_body_v2(yaml:&mut Yaml, current_section:Option<&str>){

    if let Some(arr) = yaml.as_mut_vec(){
        let mut total = 0;
        let mut new_vec = vec![];

        for item in arr.into_iter(){
            println!("Runing {:?}", item);
            if let Some(item) = item.as_str(){
                let (updated, data) = parse_function_for_section(current_section, item);
                println!("we update date outer => {:?}", data);
                if updated{
                    new_vec.extend(data);
                }else{
                    new_vec.push(Yaml::String(item.to_owned()));

                }
            
            }else{
          
                new_vec.push(item.clone());
            }
        }
        *arr = new_vec;
        println!("Im done {:?}", arr);
    }
    else if let Some(mapping) = yaml.as_mut_hash(){
        for (_key, value) in mapping.iter_mut(){
            let section_name = _key.as_str().unwrap();
            if let Some(string_val) = value.as_str(){
                let (updated, data) = parse_function_for_section(Some(section_name), string_val);
                if updated && data.len() > 1{
                    panic!("Use list syntax for add multiple object for section. '{}'", section_name)
                }else if updated && data.len() > 0{
                    *value = Yaml::Hash(data.iter().next().unwrap().as_hash().unwrap().to_owned());
                    println!("Its updated {:?}", value);
                }
           }else if value.is_array(){
            build_body_v2(value  , current_section);
           }
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
    let mut yamls = Yaml::load_from_str(&contents).unwrap();
    let yaml = &mut yamls[0];

    build_body_v2(yaml, None);

    let mut dest = File::create( args.output).unwrap();
    println!("Final => {:?}",&result);

    dest.write_all(&mut result.as_ref()).unwrap();

}

fn yaml_from_file(file:&str) ->  Yaml {
    let mut inner_file = File::open(file).expect(&format!("check permissions for {file}"));
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
        let v = yaml_from_file(dir.path().to_str().unwrap());
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
    match func_regex.captures(_str.trim()){
        Some(v)=>{
            updated = true;
            let func = v.get(1).unwrap();
            let arg = v.get(2).unwrap().as_str();
            let mut args = arg.split(",");

            let lookups = match func.as_str(){
                "lookup"=>lookup(section_name, args.next().unwrap()),
                "lookup_file"=>lookup_file(args.next().unwrap(), args.next().unwrap()),
                x=>panic!("{x} is not a function.")
            };
            seq.extend(lookups.into_iter());

        },
        None=>{()}
    }
    println!("kill {} {:?}", updated, seq);
    (updated, seq)
}
/// Used to lookup single object from file and return YAML
fn lookup_file(mut dir:&str, mut file_name:&str) -> Vec<Yaml>{
    let mut result = String::new();
    println!("erm {} {}", dir, file_name);
    dir = dir.trim().strip_prefix("'").unwrap().strip_suffix("'").unwrap();
    file_name = file_name.trim().strip_prefix("'").unwrap().strip_suffix("'").unwrap();
    let complete_path = &format!("{dir}/{file_name}");
    println!("wow {}",complete_path);
    let path = Path::new(complete_path);
    // Just like template, we read in first
    let y = yaml_from_file(path.to_str().unwrap());
    if !y.is_array(){
        vec![y.to_owned()]
    }else{
        y.as_vec().unwrap().to_owned()
    }
    // build out any necessary params
    // result = build_section(v, result);

}



