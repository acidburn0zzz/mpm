extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;
extern crate time;
extern crate walkdir;

use ext::{parse_toml_file,Splits,strip_parent};
use error::BuildError;

use std::env;
use std::fs;
use std::io;
use std::error;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use toml::decode;

use rpf::*;
use time::*;
use tar::Archive;
use rustc_serialize::{Encoder,Encodable};
use rustc_serialize::json;
use walkdir::WalkDir;

#[allow(non_camel_case_types)]
#[derive(PartialEq,PartialOrd,Debug,RustcEncodable,RustcDecodable,Clone)]
pub enum Arch {
    x86_64,
    i686,
    arm,
    aarch64,
    powerpc,
    any,
}

impl Default for Arch {
    fn default() -> Arch {
        match env::consts::ARCH {
            "x86_64" => { Arch::x86_64 },
            "i686" => { Arch::i686 },
            "arm" => { Arch::arm },
            "aarch64" => { Arch::aarch64 },
            "powerpc" => { Arch::powerpc },
            _ => { Arch::any },
        }
    }
}

// Structure for describing a package to be built
#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct BuildFile {
    name: Option<String>,
    vers: Option<String>,
    build: Option<Vec<String>>,
    builddate: Option<String>,
    desc: Option<String>,
    prefix: Option<String>,
    package: Option<Vec<String>>,
    makedeps: Option<Vec<String>>,
    deps: Option<Vec<String>>,
    arch: Option<Vec<Arch>>,
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
        let mut bf: BuildFile = Default::default();
        bf.builddate = match bf.set_builddate() {
            Some(s) => Some(s),
            None => None,
        };
        return bf;
    }

    // Creates a BuldFile struct from a TOML file
    pub fn from_file(file: &str) -> Option<BuildFile> {
        let toml = match parse_toml_file(file) {
            Ok(s) => { s },
            Err(_) => { return None }
        };
        match toml::decode(toml::Value::Table(toml)) {
            Some(s) => { return s },
            None => { return None },
        };
    }

    // Prints the BuildFile's serialized TOML as pretty JSON
    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }

    pub fn set_builddate(&self) -> Option<String> {
        match time::strftime("%m%d%Y%H%M%S", &time::now()) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }
}

// Trait for performing a build
pub trait Builder {
    fn assign_host_arch(&mut self);
    // Creates a package from tar file
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>>;
    // Creates a tar file
    fn create_tar_file(&self) -> Result<File, Box<error::Error>>;
    // Sets the build environment for the package
    fn set_env(&self) -> io::Result<()>;
    // Builds pacakge
    fn build(&self) -> io::Result<()>;
    // Gets size of directory before packaging
    fn pkg_size(&self) -> Result<u64, BuildError>;
}


impl Builder for BuildFile {
    fn assign_host_arch(&mut self) {
        if self.arch.is_none() {
            match env::consts::ARCH {
                "x86_64" => { self.arch = Some(vec![Arch::x86_64]) },
                "i686" => { self.arch = Some(vec![Arch::i686]) },
                "arm" => { self.arch = Some(vec![Arch::arm]) },
                _ => { self.arch = Some(vec![Default::default()]) },
            }
        }
    }

    // 'touches' a tarball file using the string in 'name' as a file name
    fn create_tar_file(&self) -> Result<File, Box<error::Error>> {
        let mut tar_name = self.name.clone().unwrap();
        tar_name.push_str(".pkg.tar");
        Ok(try!(File::create(&tar_name)))
    }

    // Creates a package tarball
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>> {
        let tar = try!(self.create_tar_file());
        let archive = Archive::new(tar);
        self.set_builddate();
        for entry in WalkDir::new("build") {
            let entry = try!(entry);
            let file_name = strip_parent(entry.path().to_path_buf());
            let metadata = try!(fs::metadata(entry.path()));
            if metadata.is_file() {
                let mut file = try!(File::open(entry.path()));
                try!(archive.append_file(file_name, &mut file));
            } else if metadata.is_dir() {
                if entry.path() != "build".as_path() {
                    try!(archive.append_dir(file_name, entry.path()));
                }
            }
        }
        // Wrap this turd up
        match archive.finish() {
            Ok(_) => {
                println!("{}: package '{}' successfully built", "mpm", &self.name.clone().unwrap());
            },
            Err(e) => { return Err(Box::new(BuildError::Io(e))) }
        }
        Ok(())
    }

    // This should ideally create a build environment from PKG.toml
    fn set_env(&self) -> io::Result<()> {
        unimplemented!();
    }

    fn build(&self) -> io::Result<()> {
        for line in self.build.clone().unwrap() {
            let parsed_line: Vec<&str> = line.split(' ').collect();
            match parsed_line.split_frst() {
                Some(s) => {
                    let command = try!(Command::new(s.0).args(s.1).output());
                    println!("{}", String::from_utf8_lossy(&command.stdout));
                },
                None => { () },
            };
        }
        return Ok(());
    }

    fn pkg_size(&self) -> Result<u64, BuildError> {
        let mut size: u64 = 0;
        for entry in WalkDir::new("build") {
            let entry = try!(entry);
            if entry.path() != "build".as_path() {
                match fs::metadata(entry.path()) {
                    Ok(s) => size += s.len(),
                    Err(e) => { return Err(BuildError::Io(e)) },
                };
            }
        }
        return Ok(size);
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct PkgInfo {
    name: String,
    vers: String,
    builddate: String,
    url: String,
    size: u64,
    arch: Vec<Arch>,
    license: String,
    conflicts: Vec<String>,
    provides: String,
}

impl PkgInfo {
    pub fn new(build_file: &BuildFile) -> PkgInfo {
        let mut info: PkgInfo = Default::default();
        info.name = build_file.name.clone().unwrap_or("Unknown".to_owned());
        info.vers = build_file.vers.clone().unwrap_or("Unknown".to_owned());
        info.builddate = build_file.builddate.clone().unwrap_or("Unkown".to_owned());
        info.url = build_file.url.clone().unwrap_or("Uknown".to_owned());
        info.size = build_file.pkg_size().unwrap_or(0);
        info.arch = build_file.arch.clone().unwrap_or(vec![Default::default()]);;
        info.license = build_file.license.clone().unwrap_or("Unkown".to_owned());
        info.conflicts = build_file.conflicts.clone().unwrap_or(vec!["Unkown".to_owned()]);
        info.provides = build_file.provides.clone().unwrap_or("Uknown".to_owned());
        return info;
    }

    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }

    pub fn update_size(&mut self, build_file: &BuildFile) {
        self.size = build_file.pkg_size().unwrap_or(0);
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
        Some(s) => { s },
        None => { panic!() }
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
    let build_file = BuildFile::from_file("example/PKG.toml").unwrap();
    let info = PkgInfo::new(&build_file);
}
