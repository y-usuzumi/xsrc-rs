#![feature(box_syntax)]

use maplit::hashmap;
use std::fmt;
use clap::{App, load_yaml};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use self::GenError::*;

struct LangInfo<'a> {
    ext: &'a str,
}

fn init_lang_infos() -> HashMap<&'static str, LangInfo<'static>> {
    return hashmap![
        "javascript" => LangInfo {
            ext: ".js"
        }
    ];
}

enum GenError {
    ParserError(xsrc::schema::ParserError),
    TransformerError(xsrc::transformer::TransformerError),
    UnsupportedLanguage(String),
    IOError(std::io::Error),
}

impl From<xsrc::schema::ParserError> for GenError {
    fn from(e: xsrc::schema::ParserError) -> Self {
        ParserError(e)
    }
}

impl From<xsrc::transformer::TransformerError> for GenError {
    fn from(e: xsrc::transformer::TransformerError) -> Self {
        TransformerError(e)
    }
}

impl From<std::io::Error> for GenError {
    fn from(e: std::io::Error) -> Self {
        IOError(e)
    }
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError(e) => write!(f, "Parser error: {}", e),
            TransformerError(e) => write!(f, "Transformer error: {}", e),
            UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            IOError(e) => write!(f, "IO error: {}", e)
        }
    }
}

fn gen<P: AsRef<Path> + Clone, Q: AsRef<Path> + Clone>(
    lang: &str,
    schema_file: P,
    output_file: Q,
) -> Result<PathBuf, GenError> {
    match lang {
        "javascript" => {
            let root_schema = xsrc::schema::parse_file(schema_file)?;
            let root = xsrc::transformer::transform(root_schema)?;
            let gen_ctx = Default::default();
            let code = xsrc::rewriter::javascript::gen(&root, &gen_ctx);
            let mut f = File::create(output_file.clone())?;
            f.write_all(&code.as_bytes())?;
            let p = output_file.as_ref().canonicalize()?;
            Ok(p)
        }
        _ => Err(GenError::UnsupportedLanguage(lang.to_string())),
    }
}

fn main() {
    let lang_infos = init_lang_infos();
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    let schema_file = matches.value_of("schema").unwrap();
    let lang = matches.value_of("lang").unwrap_or("javascript");
    let output_file = match matches.value_of("output") {
        Some(f) => f.to_string(),
        None => {
            let ext = match lang_infos.get(lang) {
                Some(li) => li.ext,
                _ => ".out",
            };
            format!("{}{}", "output", ext)
        }
    };
    match gen(lang, &schema_file, &output_file) {
        Ok(path) => {
            let path_str = path.to_str().unwrap();
            println!("Code file generated at {}", path_str);
        },
        Err(e) => eprintln!("Error: {}", e)
    }
}
