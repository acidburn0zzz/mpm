extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;
extern crate time;
extern crate walkdir;
extern crate git2;
extern crate hyper;

use ext::{parse_toml_file, strip_parent, assert_toml};
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
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use rustc_serialize::json;
use walkdir::WalkDir;
use git2::Repository;
use hyper::client::Client;

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
            "x86_64" => Arch::x86_64,
            "i686" => Arch::i686,
            "arm" => Arch::arm,
            "aarch64" => Arch::aarch64,
            "powerpc" => Arch::powerpc,
            _ => Arch::any,
        }
    }
}

#[derive(Debug,Default,PartialEq)]
pub struct PackageBuild {
    package: Option<PackageDesc>,
    clean: Option<CleanDesc>,
}

impl PackageBuild {
    pub fn from_file(file: &str) -> Result<PackageBuild, Vec<BuildError>> {
        try!(assert_toml(file));
        let mut pkg_build: PackageBuild = Default::default();
        parse_toml_file(file)
            .and_then(|toml| {
                for (key, table) in toml {
                    match key.as_ref() {
                        "package" => pkg_build.package = PackageDesc::from_toml_table(table).ok(),
                        "clean" => pkg_build.clean = CleanDesc::from_toml_table(table).ok(),
                        _ => (),
                    };
                }
                Ok(pkg_build)
            })
            .map_err(|err| err)
    }

    pub fn print_json(self) {
        println!("{}", "[package]".bold());
        self.package.unwrap_or(PackageDesc::new()).print_json();
        println!("{}", "[clean]".bold());
        self.clean.unwrap_or(CleanDesc::new()).print_json();
    }

    pub fn package(self) -> Option<PackageDesc> {
        self.package
    }

    pub fn clean(self) -> Option<CleanDesc> {
        self.clean
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct CleanDesc {
    script: Option<Vec<String>>,
}

impl CleanDesc {
    pub fn new() -> CleanDesc {
        Default::default()
    }

    pub fn from_toml_table(table: toml::Value) -> Result<CleanDesc, BuildError> {
        Ok(try!(Decodable::decode(&mut toml::Decoder::new(table))))
    }

    pub fn exec(&self) -> Result<(), Box<error::Error>> {
        println!("{}", "Cleaning build environment".bold());
        for line in &self.script.clone().unwrap_or(vec!["".to_string()]) {
            // Parse a line of commands from toml
            let parsed_line: Vec<&str> = line.split(' ').collect();
            if let Some(s) = parsed_line.split_first() {
                let mut command = try!(Command::new(s.0).args(s.1).spawn());
                let status = try!(command.wait());
                if let Some(child_output) = command.stdout.as_mut() {
                    // Child process has output
                    println!("{}", try!(child_output.read(&mut Vec::new())));
                };
                if let Some(code) = status.code() {
                    if code != 0 {
                        println!("'{}' terminated with code '{}'",
                                 s.0.bold(),
                                 code.to_string().bold());
                    }
                };
            }
        }
        Ok(println!("{}", "Clean succeeded".bold()))
    }

    // Prints the PackageDesc's serialized TOML as pretty JSON
    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }

    pub fn from_file(file: &str) -> Result<CleanDesc, Vec<BuildError>> {
        try!(assert_toml(file));
        parse_toml_file(file).and_then(|toml| {
            toml.get("clean")
                .ok_or(vec![BuildError::NoCleanDesc])
                .and_then(|desc| {
                    Decodable::decode(&mut toml::Decoder::new(desc.clone()))
                        .map_err(|err| vec![BuildError::TomlDecode(err)])
                })
        })
    }
}

// Structure for describing a package to be built
#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct PackageDesc {
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
    source: Option<Vec<String>>,
    sha256: Option<Vec<String>>,
    sha512: Option<Vec<String>>,
    license: Option<String>,
    provides: Option<String>,
    conflicts: Option<Vec<String>>,
    maintainers: Option<Vec<String>>,
}

impl PackageDesc {
    pub fn from_toml_table(table: toml::Value) -> Result<PackageDesc, BuildError> {
        Ok(try!(Decodable::decode(&mut toml::Decoder::new(table))))
    }

    // Creates a new blank BuldFile
    pub fn new() -> PackageDesc {
        Default::default()
    }

    // Creates a BuldFile struct from a TOML file
    pub fn from_file(file: &str) -> Result<PackageDesc, Vec<BuildError>> {
        try!(assert_toml(file));
        parse_toml_file(file).and_then(|toml| {
            toml.get("package")
                .ok_or(vec![BuildError::NoBuildDesc])
                .and_then(|desc| {
                    Decodable::decode(&mut toml::Decoder::new(desc.clone()))
                        .map_err(|err| vec![BuildError::TomlDecode(err)])
                })
        })
    }

    // Prints the PackageDesc's serialized TOML as pretty JSON
    pub fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }

    pub fn set_builddate(&mut self) -> Result<(), Box<error::Error>> {
        match time::strftime("%m%d%Y%H%M%S", &time::now_utc()) {
            Ok(s) => self.builddate = Some(s),
            Err(e) => return Err(Box::new(e)),
        };
        Ok(())
    }
}

// Trait for performing a build
pub trait Builder {
    fn assign_host_arch(&mut self);
    // Creates a package from tar file
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>>;
    // Creates a tar file
    fn create_tar_file(&self) -> Result<(File, String), Box<error::Error>>;
    // Sets the build environment for the package
    fn set_env(&self) -> io::Result<()>;
    // Builds pacakge
    fn build(&self) -> Result<(), Box<error::Error>>;
    // Gets size of directory before packaging
    fn pkg_size(&self) -> Result<u64, BuildError>;
    fn handle_status(&self, cmd: &str, code: Option<i32>);
    fn handle_source(&self) -> Result<(), Box<error::Error>>;
    fn clone_repo(&self, url: &str) -> Result<Repository, BuildError>;
    fn web_get(&self, url: &str) -> Result<(), Box<error::Error>>;
}


impl Builder for PackageDesc {
    fn assign_host_arch(&mut self) {
        if self.arch.is_none() {
            match env::consts::ARCH {
                "x86_64" => self.arch = Some(vec![Arch::x86_64]),
                "i686" => self.arch = Some(vec![Arch::i686]),
                "arm" => self.arch = Some(vec![Arch::arm]),
                _ => self.arch = Some(vec![Default::default()]),
            }
        }
    }

    // 'touches' a tarball file using the string in 'name' as a file name
    fn create_tar_file(&self) -> Result<(File, String), Box<error::Error>> {
        let mut tar_name = self.name.clone().unwrap_or("Unkown".to_owned());
        tar_name.push_str(&format!("-{:?}", self.arch.clone().unwrap().first().unwrap()));
        tar_name.push_str(".pkg.tar");
        Ok((try!(File::create(&tar_name)), tar_name))
    }

    // Creates a package tarball
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>> {
        try!(self.build());
        let tar = try!(self.create_tar_file());
        let archive = Archive::new(tar.0);
        try!(self.set_builddate());
        try!(PkgInfo::new(&self).write("build/PKGINFO"));
        print!("{}", "Compressing package..".bold());
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
        if let Err(e) = archive.finish() {
            return Err(Box::new(BuildError::Io(e)));
        } else {
            print!("{}\n", "OK".paint(Color::Green))
        }
        Ok(println!("{} '{}' {}",
                    "Package".bold(),
                    &tar.1.paint(Color::Green),
                    "successfully built".bold()))
    }

    // This should ideally create a build environment from PKG.toml
    fn set_env(&self) -> io::Result<()> {
        unimplemented!();
    }

    fn handle_status(&self, cmd: &str, code: Option<i32>) {
        if let Some(status) = code {
            if status != 0 {
                println!("'{}' terminated with code '{}'",
                         cmd.bold(),
                         status.to_string().bold());
            }
        };
    }

    fn handle_source(&self) -> Result<(), Box<error::Error>> {
        if let Some(source) = self.source.clone() {
            for item in source {
                let pos = item.find('+').unwrap_or(item.len());
                let (cvs, url) = item.split_at(pos);
                if cvs == "git" {
                    try!(self.clone_repo(url));
                } else {
                    try!(self.web_get(cvs));
                }
            }
        };
        Ok(())
    }

    fn web_get(&self, url: &str) -> Result<(), Box<error::Error>> {
        if let Some(file_name) = url.rsplit('/').nth(0) {
            println!("Downloading: {}", url);
            let mut res = try!(Client::new().get(url).send());
            let mut buffer = Vec::new();
            try!(res.read_to_end(&mut buffer));
            if res.status != hyper::Ok {
                return Err(Box::new(BuildError::HttpNoFile(res.status, url.to_owned())))
            } else {
                try!(try!(File::create(file_name)).write(&mut buffer));
            }
        }
        Ok(())
    }

    fn clone_repo(&self, url: &str) -> Result<Repository, BuildError> {
        Repository::clone(url, "build")
            .map_err(|err| BuildError::Git(err))
            .map(|repo|{
                println!("{} {}", "Cloning".bold(), url.bold());
                repo
            })
    }


    fn build(&self) -> Result<(), Box<error::Error>> {
        println!("{}", "Beginning package build".bold());
        try!(self.handle_source());
        for line in self.build.clone().unwrap() {
            // Parse a line of commands from toml
            let parsed_line: Vec<&str> = line.split(' ').collect();
            if let Some(s) = parsed_line.split_first() {
                let mut command = try!(Command::new(s.0).args(s.1).spawn());
                let status = try!(command.wait());
                if let Some(child_output) = command.stdout.as_mut() {
                    // Child process has output
                    let mut buff = String::new();
                    println!("{}", try!(child_output.read_to_string(&mut buff)));
                };
                &self.handle_status(s.0, status.code());
            };
        }
        Ok(println!("{}", "Build succeeded".bold()))
    }

    fn pkg_size(&self) -> Result<u64, BuildError> {
        let mut size: u64 = 0;
        for entry in WalkDir::new("build") {
            let entry = try!(entry);
            if entry.path() != "build".as_path() {
                match fs::metadata(entry.path()) {
                    Ok(s) => size += s.len(),
                    Err(e) => return Err(BuildError::Io(e)),
                };
            }
        }
        Ok(size)
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
    pub fn new(build_file: &PackageDesc) -> PkgInfo {
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

    pub fn update_size(&mut self, build_file: &PackageDesc) {
        self.size = build_file.pkg_size().unwrap_or(0);
    }

    pub fn write(&self, path: &str) -> Result<(), BuildError> {
        print!("{}", "Generating PKGINFO...".bold());
        try!(try!(File::create(path)).write_all(try!(json::encode(&self)).as_bytes()));
        Ok(print!("{}\n", "OK".paint(Color::Green)))
    }
}

#[test]
fn test_default_arch() {
    let system_arch = env::consts::ARCH;
    let default_arch: Arch = Default::default();
    assert_eq!(system_arch, format!("{:?}", default_arch));
}

#[test]
fn test_new_empty_build_file() {
    let build_file = PackageDesc::new();
    assert_eq!(build_file.name, None);
    assert_eq!(build_file.vers, None);
    assert_eq!(build_file.build, None);
}

#[test]
#[should_panic]
fn test_from_file_fail() {
    let build_file = match PackageDesc::from_file("none.toml") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(build_file, Ok(()));
}

#[test]
fn test_from_file_success() {
    let build_file = match PackageDesc::from_file("example/PKG.toml") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(build_file, Ok(()));
}

#[test]
fn test_print_json() {
    let build_file = PackageDesc::from_file("example/PKG.toml").unwrap();
    build_file.print_json();
}

#[test]
fn test_new_empty_pkginfo() {
    let build_file = PackageDesc::from_file("example/PKG.toml").unwrap();
    let info = PkgInfo::new(&build_file);
    assert_eq!(info.name, build_file.name.unwrap_or("Unknown".to_owned()));
}
