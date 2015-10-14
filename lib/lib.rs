extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;
extern crate time;

pub static MPM: Prog = Prog { name: "mpm", vers: "0.1.0", yr: "2015", };

use std::fs;
use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;

use rpf::*;
use time::*;
use tar::Archive;
use toml::{Value};
use rustc_serialize::{Encoder,Encodable};
use rustc_serialize::json::Json;
use rustc_serialize::json;

// Parses a toml file and converts it to json
fn parse_pkg_file<T: AsRef<Path>>(file_name: T) -> Json {
    let mut buff = String::new();
    let mut file = match File::open(file_name) {
        Ok(s) => { s },
        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error); panic!(); }
    };
    match file.read_to_string(&mut buff) {
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

// Converts a given toml value to json
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

// Structure for describing a package to be built
#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct BuildFile {
    name: Option<String>,
    vers: Option<String>,
    build: Option<Vec<String>>,
    desc: Option<String>,
    prefix: Option<String>,
    package: Option<Vec<String>>,
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

impl BuildFile {
    // Creates a new blank BuldFile
    pub fn new() -> BuildFile {
        Default::default()
    }

    // Creates a BuldFile struct from a TOML file
    pub fn from_file(file: &str) -> Result<BuildFile, json::DecoderError> {
        match json::decode(&parse_pkg_file(file).to_string()) {
            Ok(k) => { return Ok(k) },
            Err(e) => { return Err(e) }
        }
    }

    // Prints the BuildFile's serialized TOML as pretty JSON
    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }
}

// Trait for performing a build
pub trait Builder {
    // Creates a package from tar file
    fn create_pkg(&self) -> Result<(), io::Error>;
    // Creates a tar file
    fn create_tar_file(&self) -> Result<File, io::Error>;
    // Sets the build environment for the package
    fn set_env(&self);
    // Builds pacakge
    fn build(&self)-> Result<(), io::Error>;
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

    fn create_pkg(&self) -> Result<(), io::Error> {
        let tar = match self.create_tar_file() {
            Ok(s) => { s },
            Err(e) => { return Err(e) }
        };
        let archive = Archive::new(tar);
        let dir_iter = match fs::read_dir("build") {
            Ok(s) => { s },
            Err(e) => { return Err(e) }
        };
        for entry in dir_iter {
            let entry = entry.unwrap();
            match archive.append_path(entry.path()) {
                Ok(s) => { s },
                Err(e) => { return Err(e) }
            }
        }
        match archive.finish() {
            Ok(_) => {
                println!("{}: package '{}' successfully built", "mpm", &self.name.clone().unwrap());
                return Ok(());
            },
            Err(e) => { return Err(e) }
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

    fn build(&self) -> Result<(), io::Error> {
        &self.set_env();
        for line in self.build.clone().unwrap() {
            let parsed_line: Vec<&str> = line.split(' ').collect();
            match parsed_line.split_frst() {
                Some(s) => {
                    let command = match Command::new(s.0).args(s.1).output() {
                        Ok(s) => { s },
                        Err(e) => { return Err(e) }
                    };
                    println!("{}", String::from_utf8_lossy(&command.stdout));
                },
                None => { () },
            };
        }
        return Ok(());
    }
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

impl PkgInfo {
    pub fn new(build_file: &BuildFile) -> PkgInfo {
        let mut info: PkgInfo = Default::default();
        info.name = build_file.name.clone().unwrap();
        info.builddate = time::strftime("%m%d%Y%H%M%S", &time::now()).unwrap();
        return info;
    }
}

#[test]
fn test_new_empty_build_file() {
    let build_file = BuildFile::new();
    assert_eq!(build_file.name, None);
    assert_eq!(build_file.vers, None);
    assert_eq!(build_file.build, None);
}

#[test]
fn test_from_file() {
    let build_file = match BuildFile::from_file("example/PKG.toml") {
        Ok(s) => { s },
        Err(_) => { panic!() }
    };
    assert_eq!(build_file.name.unwrap(), "hello-mpm".to_owned());
}

#[test]
fn test_print_json() {
    let build_file = BuildFile::from_file("example/PKG.toml").unwrap();
    build_file.print_json();
}

#[test]
fn test_new_empty_pkginfo() {
    let mut build_file = BuildFile::new();
    build_file.name = Some("test".to_owned());
    let info = PkgInfo::new(&build_file);
}

#[test]
fn test_parse_pkg_file() {
    let json = parse_pkg_file("example/PKG.toml");
}

#[test]
fn test_convert() {
    let toml = "foo = 'bar'";
    let val: toml::Value = toml.parse().unwrap();
    let json = convert(val);
}
