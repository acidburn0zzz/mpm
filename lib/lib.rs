extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;

pub static MPM: Prog = Prog { name: "mpm", vers: "0.1.0", yr: "2015", };

use std::fs;
use std::env;
use std::io;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;

use rpf::*;
use tar::Archive;
use toml::{Value};
use rustc_serialize::{Encoder,Encodable};
use rustc_serialize::json::Json;
use rustc_serialize::json;

// This trait is temporary until `split` is considered stable in std
pub trait Splits<T> {
    fn split_frst(&self) -> Option<(&T, &[T])>;
    fn split_lst(&self) -> Option<(&T, &[T])>;
}

impl<T> Splits<T> for [T] {
    fn split_frst(&self) -> Option<(&T, &[T])> {
        if self.is_empty() { None } else { Some((&self[0], &self[1..]))}
    }
    fn split_lst(&self) -> Option<(&T, &[T])> {
        let len = self.len();
        if self.is_empty() { None } else {
            Some((&self[len - 1], &self[..(len -1)]))
        }
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct BuildFile {
    name: Option<String>,
    vers: Option<String>,
    build: Option<Vec<String>>,
    desc: Option<String>,
    prefix: Option<String>,
    install: Option<Vec<String>>,
    makedeps: Option<Vec<String>>,
    deps: Option<Vec<String>>,
    arch: Option<String>,
    url: Option<String>,
    source: Option<String>,
    license: Option<String>,
    provides: Option<String>,
    conflicts: Option<Vec<String>>,
    maintainers: Option<Vec<String>>,
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct PkgInfo {
    name: String,
    vers: String,
    builddate: String,
    url: String,
    size: u64,
    arch: String,
    license: String,
    conflicts: Vec<String>,
    provides: String,
}

impl BuildFile {
    pub fn new() -> BuildFile {
        Default::default()
    }

    pub fn from_file(file: &str) -> Result<BuildFile, json::DecoderError> {
        match json::decode(&parse_pkg_file(file).to_string()) {
            Ok(k) => { return Ok(k) },
            Err(e) => { return Err(e) }
        }
    }

    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }
}

pub trait Builder {
    fn create_pkg(&self);
    fn create_tar_file(&self) -> Result<File, io::Error>;
    fn set_env(&self);
    fn build(&self);
}

impl Builder for BuildFile {
    fn create_tar_file(&self) -> Result<File, io::Error> {
        let mut tar_name = self.name.clone().unwrap();
        tar_name.push_str(".pkg.tar");
        match File::create(&tar_name) {
            Ok(s) => { return Ok(s) },
            Err(e) => { return Err(e) }
        };
    }

    fn create_pkg(&self) {
        let tar = match self.create_tar_file() {
            Ok(s) => { s },
            Err(e) => {
                MPM.error(e.to_string(), ExitStatus::Error);
                panic!();
            }
        };
        let archive = Archive::new(tar);
        let dir_iter = match fs::read_dir("build") {
            Ok(s) => { s },
            Err(e) => { panic!(e); }
        };
        for entry in dir_iter {
            let entry = entry.unwrap();
            archive.append_path(entry.path());
        }
        match archive.finish() {
            Ok(_) => {
                println!("{}: package {} successfully built",
                         MPM.name.bold(),
                         &self.name.clone().unwrap().bold());
            },
            Err(e) => {
                println!("{}: package {} could not be build",
                         MPM.name.bold(),
                         &self.name.clone().unwrap().bold());
                MPM.error(e.to_string(), ExitStatus::Error);
            }
        }
    }

    fn set_env(&self) {
        let build = "BUILD";
        let prefix = "PREFIX";
        env::set_var(build, "build");
        env::set_var(prefix, self.prefix.clone().unwrap());
        match fs::create_dir("build") {
            Ok(_) => { },
            Err(e) => {
                MPM.error(e.to_string(), ExitStatus::Error);
                panic!();
            }
        };
    }

    fn build(&self) {
        &self.set_env();
        for line in self.build.clone().unwrap() {
            let parsed_line: Vec<&str> = line.split(' ').collect();
            match parsed_line.split_frst() {
                Some(s) => {
                    let command = Command::new(s.0)
                            .args(s.1)
                            .output().unwrap_or_else(|e| {
                                MPM.error(e.to_string(), ExitStatus::Error);
                                panic!();
                            });
                    println!("{}", String::from_utf8_lossy(&command.stdout));
                },
                None => { },
            };
        }
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
    let pkg = BuildFile::new();
    assert_eq!(pkg.name, None);
    assert_eq!(pkg.vers, None);
    assert_eq!(pkg.build, None);
}
