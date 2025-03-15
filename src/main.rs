mod build_json;
mod build_yaml;
mod generator;
mod readers;
mod utils;

use clap::builder::TypedValueParser as _;
use std::{fmt::Display, str::FromStr};

use clap::Parser;
use generator::{Extension, Generator};

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

    #[arg(short, long, default_value_t = String::from("output"))]
    /// Output name of generated YAML file
    output: String,
}

fn main() {
    let args = Args::parse();

    let template_file = args.template;
    let suffix = match args.extension {
        Extension::JSON => ".json",
        Extension::YAML => ".yaml",
    };
    let output = format!("{}{}", args.output, suffix);
    let file_gen = &mut Generator::new(args.extension, template_file, output);

    file_gen.generate_file();

    println!("Success!")
}
