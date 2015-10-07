extern crate toml;
extern crate rustc_serialize;
extern crate rpf;

pub static MPM: Prog = Prog { name: "mpm", vers: "0.1.0", yr: "2015", };

use rpf::*;
use std::fs::File;
use std::io::prelude::*;
use toml::{Value};
use rustc_serialize::Encoder;
use rustc_serialize::Encodable;
use rustc_serialize::json::Json;
use rustc_serialize::json;

#[derive(RustcDecodable,RustcEncodable,Debug,Default)]
pub struct Package {
    name: Option<String>,
    vers: Option<String>,
    build: Option<Vec<String>>,
    install: Option<Vec<String>>,
    deps: Option<Vec<String>>,
    arch: Option<String>,
    files: Option<Vec<String>>,
}

impl Package {
    pub fn new(file: &str) -> Package {
        let pkg: Package = match json::decode(&parse_pkg_file(file).to_string()) {
            Ok(k) => { k },
            Err(e) => {
                MPM.error(e.to_string(), ExitStatus::Error);
                panic!();
            }
        };
        return pkg;
    }

    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }
}

fn parse_pkg_file(file: &str) -> Json {
    let mut buff = String::new();
    let mut f = match File::open(file) {
        Ok(s) => { s },
        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error); panic!(); }
    };
    match f.read_to_string(&mut buff) {
        Ok(s) => { s },
        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error); panic!(); }
    };
    let mut parser = toml::Parser::new(&buff);
    let toml = match parser.parse() {
        Some(s) => { s },
        None => { panic!() }
    };
    convert(toml::Value::Table(toml))
}

fn convert(toml: toml::Value) -> Json {
    match toml {
        Value::String(s) => Json::String(s),
        Value::Integer(i) => Json::I64(i),
        Value::Float(f) => Json::F64(f),
        Value::Boolean(b) => Json::Boolean(b),
        Value::Array(a) => Json::Array(a.into_iter().map(convert).collect()),
        Value::Table(t) => Json::Object(t.into_iter().map(|(k, v)| {
            (k, convert(v))
        }).collect()),
        Value::Datetime(d) => Json::String(d),
    }
}

#[test]
fn new_empty_package() {
    let pkg: Package = Default::default();
    assert_eq!(pkg.name, None);
    assert_eq!(pkg.vers, None);
    assert_eq!(pkg.build, None);
}
