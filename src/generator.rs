use std::{
    fs::File,
    io::{Read, Write},
};

use saphyr::{Yaml, YamlEmitter};

use crate::{build_json::build_section_json, build_yaml::build_section_yaml};

#[derive(Debug, Clone)]
pub enum Extension {
    JSON,
    YAML,
}

pub struct Generator {
    ext: Extension,
    file_contents: String,
    output: String,
}

impl Generator {
    /// Returns a Generator about with template file contents read as a string
    pub fn new(ext: Extension, template: String, output: String) -> Self {
        let mut file =
            File::open(&template).expect(&format!("check permissions/existence of '{template}'"));

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        Self {
            ext: ext,
            file_contents: contents,
            output: output,
        }
    }

    /// Dispatch fn to generate output file based on
    /// ext provided
    pub fn generate_file(&self) {
        match self.ext {
            Extension::JSON => {
                self.generate_json();
            }
            Extension::YAML => {
                self.generate_yaml();
            }
        }
    }

    fn generate_json(&self) {
        let json = &mut serde_json::from_str(&self.file_contents).unwrap();
        build_section_json(json, None);

        let dest = File::create(&self.output).unwrap();

        serde_json::to_writer_pretty(dest, json).unwrap()
    }

    fn generate_yaml(&self) {
        let mut yamls = Yaml::load_from_str(&self.file_contents).unwrap();
        let yaml = &mut yamls[0];

        build_section_yaml(yaml, None);
        let mut dest = File::create(&self.output).unwrap();
        let mut out_str = String::new();

        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(yaml).unwrap();
        out_str = out_str.strip_prefix("---\n").unwrap().to_string();
        dest.write_all(out_str.as_bytes()).unwrap();
    }
}
