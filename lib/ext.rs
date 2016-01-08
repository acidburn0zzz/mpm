extern crate toml;
extern crate rustc_serialize;
extern crate rpf;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use rpf::*;

use toml::Value;
use rustc_serialize::json::Json;
use std::fs;
use error::BuildError;

pub fn assert_toml(file: &str) -> Result<(), BuildError> {
    let metadata = try!(fs::metadata(file));
    if metadata.is_dir() {
        return Err(BuildError::NonToml(file.to_string()));
    } else if metadata.is_file() {
        match file.as_path().extension() {
            Some(ext) => {
                if ext != "toml" && ext != "tml" {
                    return Err(BuildError::NonToml(file.to_string()));
                }
            }
            None => {
                return Err(BuildError::NonToml(file.to_string()));
            }
        }
    }
    Ok(())
}

// Strip 'build' from path's as using this directory is currently hard coded
// behaviour
// FIXME: name is extremely misleading
pub fn strip_parent(path: PathBuf) -> PathBuf {
    let mut new_path = PathBuf::new();
    for component in path.components() {
        if component.as_ref() != "pkg" {
            new_path.push(component.as_ref());
        }
    }
    new_path
}

// Parses a toml file.
pub fn parse_toml_file<T: AsRef<Path>>(file: T) -> Result<toml::Table, Vec<BuildError>> {
    let mut buff = String::new();
    let mut error_vec = Vec::new();
    let mut file = match File::open(file) {
        Ok(s) => s,
        Err(e) => {
            error_vec.push(BuildError::Io(e));
            return Err(error_vec);
        }
    };
    match file.read_to_string(&mut buff) {
        Ok(s) => s,
        Err(e) => {
            error_vec.push(BuildError::Io(e));
            return Err(error_vec);
        }
    };
    let mut parser = toml::Parser::new(&buff);
    match parser.parse() {
        Some(s) => return Ok(s),
        None => {
            for err in parser.errors {
                error_vec.push(BuildError::TomlParse(err));
            }
            return Err(error_vec);
        }
    };
}

// Converts a given toml value to json
pub fn convert(toml: toml::Value) -> Json {
    match toml {
        Value::String(s) => Json::String(s),
        Value::Integer(i) => Json::I64(i),
        Value::Float(f) => Json::F64(f),
        Value::Boolean(b) => Json::Boolean(b),
        Value::Array(a) => Json::Array(a.into_iter().map(convert).collect()),
        Value::Table(t) => {
            Json::Object(t.into_iter()
                          .map(|(k, v)| (k, convert(v)))
                          .collect())
        }
        Value::Datetime(d) => Json::String(d),
    }
}

#[test]
fn test_strip_parent() {
    let test_path = PathBuf::from("test/build/dir");
    assert_eq!(strip_parent(test_path), PathBuf::from("test/dir"));
}

#[test]
#[should_panic]
fn test_parse_toml_file() {
    let table_from_file = parse_toml_file("example/PKG.toml").unwrap();
    let toml_table = toml::Table::new();
    assert_eq!(table_from_file, toml_table);
}

#[test]
fn test_convert() {
    let toml = "foo = 'bar'";
    let val: toml::Value = toml.parse().unwrap();
    let json_from_toml = convert(val);
    let encoded_json = Json::from_str("{\"foo\": \"bar\"}").unwrap();
    assert_eq!(json_from_toml, encoded_json);
}
