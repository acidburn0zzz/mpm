extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;
extern crate time;
extern crate walkdir;
extern crate hyper;
extern crate crypto;
extern crate flate2;

use ext::{parse_toml_file, strip_parent, assert_toml};
use error::BuildError;
use repo::clone_repo;

use std::env;
use std::fs;
use std::error;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use toml::decode;

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

use rpf::*;
use time::*;
use tar::Archive;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use rustc_serialize::json;
use walkdir::WalkDir;
use hyper::client::Client;
use crypto::sha2;
use crypto::digest::Digest;
use flate2::read::GzDecoder;

#[allow(non_camel_case_types)]
#[derive(PartialEq,PartialOrd,Debug,RustcEncodable,RustcDecodable,Clone,Eq,Hash)]
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

#[derive(Debug,Default,PartialEq,Clone)]
pub struct PackageBuild {
    package: Option<PackageDesc>,
    clean: Option<CleanDesc>,
}

impl PackageBuild {
    pub fn new() -> PackageBuild {
        Default::default()
    }

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

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq,Clone)]
pub struct CleanDesc {
    script: Option<Vec<String>>,
}

impl CleanDesc {
    pub fn new() -> CleanDesc {
        Default::default()
    }

    pub fn clean(&self) -> Result<(), Box<error::Error>> {
        println!("{}", "Cleaning build environment".bold());
        if let Some(script) = self.script.as_ref() {
            try!(self.exec(script));
        };
        Ok(println!("{}", "Clean succeeded".bold()))
    }
}

// Structure for describing a package to be built
#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq,Clone,Eq,Hash)]
pub struct PackageDesc {
    name: Option<String>,
    vers: Option<String>,
    rel: Option<String>,
    build: Option<Vec<String>>,
    builddate: Option<String>,
    desc: Option<String>,
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
    // Creates a new blank BuldFile
    pub fn new() -> PackageDesc {
        Default::default()
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
    // Creates a package from tar file
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>>;
    fn extract_tar(&self, path: &str) -> Result<(), Box<error::Error>>;
    // Creates a tar file
    fn create_tar_file(&self) -> Result<(File, String), Box<error::Error>>;
    // Sets the build environment for the package
    fn set_env(&self) -> Result<(), Box<error::Error>>;
    fn create_dirs(&self) -> Result<(), Box<error::Error>>;
    // Builds package
    fn build(&self) -> Result<(), Box<error::Error>>;
    // Installs to `pkg` dir
    fn package(&self) -> Result<(), Box<error::Error>>;
    // Gets size of directory before packaging
    fn pkg_size(&self) -> Result<u64, BuildError>;
    // Decides what to do for `source`
    fn handle_source(&self) -> Result<(), Box<error::Error>>;
    // Get sources from web, anything that is not prepended with 'git+' for
    // source is assumped to be a downloadable file
    fn web_get(&self, url: &str) -> Result<(), Box<error::Error>>;
    // Checks for '://', returns true if found in 'url'
    fn is_web_get(&self, url: &str) -> bool;
    // Computes 512 bit SHA2 hash for a file
    fn sha_512(&self, file: &str) -> Result<String, Box<error::Error>>;
    // Computes 256 bit SHA2 hash for a file
    fn sha_256(&self, file: &str) -> Result<String, Box<error::Error>>;
    fn match_hash(&self, index: usize, file: &str) -> Result<(), Box<error::Error>>;
}

impl Builder for PackageDesc {
    fn sha_512(&self, file: &str) -> Result<String, Box<error::Error>> {
        let mut hasher = sha2::Sha512::new();
        let mut buffer = Vec::new();
        try!(try!(File::open(file)).read_to_end(&mut buffer));
        hasher.input(&buffer);
        Ok(hasher.result_str())
    }

    fn sha_256(&self, file: &str) -> Result<String, Box<error::Error>> {
        let mut hasher = sha2::Sha256::new();
        let mut buffer = Vec::new();
        try!(try!(File::open(file)).read_to_end(&mut buffer));
        hasher.input(&buffer);
        Ok(hasher.result_str())
    }

    fn match_hash(&self, index: usize, file: &str) -> Result<(), Box<error::Error>> {
        if self.sha512.is_some() {
            if (index >= self.sha512.as_ref().unwrap().len()) ||
               (self.sha512.as_ref().unwrap()[index].is_empty()) {
                return Err(Box::new(BuildError::NoHash(file.to_owned())));
            }
            let hash = try!(self.sha_512(&file));
            if (self.sha512.as_ref().unwrap()[index] != hash) &&
               (self.sha512.as_ref().unwrap()[index] != "SKIP") {
                return Err(Box::new(BuildError::HashMismatch(file.to_owned(), hash)));
            }
        } else if self.sha256.is_some() {
            if (index >= self.sha256.as_ref().unwrap().len()) ||
               (self.sha256.as_ref().unwrap().is_empty()) {
                return Err(Box::new(BuildError::NoHash(file.to_owned())));
            }
            let hash = try!(self.sha_256(&file));
            if (self.sha256.as_ref().unwrap()[index] != hash) &&
               (self.sha256.as_ref().unwrap()[index] != "SKIP") {
                return Err(Box::new(BuildError::HashMismatch(file.to_owned(), hash)));
            }
        }
        Ok(())
    }

    // 'touches' a tarball file using the string in 'name' as a file name
    fn create_tar_file(&self) -> Result<(File, String), Box<error::Error>> {
        let mut current_dir = try!(env::current_dir());
        let mut tar_name = self.name.clone().unwrap_or("Unkown".to_owned());
        if let Some(arch) = self.arch.clone() {
            if let Some(pkg_vers) = self.vers.as_ref() {
                if let Some(pkg_rel) = self.rel.as_ref() {
                    if let Some(first) = arch.first() {
                        tar_name.push_str(&format!("-{}-{}", pkg_vers, pkg_rel));
                        tar_name.push_str(&format!("-{:?}", first));
                    };
                };
            } else {
                if let Some(first) = arch.first() {
                    tar_name.push_str(&format!("-{:?}", first));
                };
            }
        };
        tar_name.push_str(".pkg.tar");
        current_dir.push(&tar_name);
        Ok((try!(File::create(current_dir)), tar_name))
    }

    // Creates a package tarball
    fn create_pkg(&mut self) -> Result<(), Box<error::Error>> {
        let current_dir = try!(env::current_dir());
        try!(self.set_env());
        try!(self.create_dirs());
        try!(self.handle_source());
        try!(self.build());
        try!(self.package());
        try!(self.set_builddate());
        try!(env::set_current_dir(current_dir));
        try!(PkgInfo::new(&self).write("pkg/PKGINFO"));
        try!(try!(MTree::from_dir("pkg")).write("pkg/MTREE"));
        let tar = try!(self.create_tar_file());
        let archive = Archive::new(tar.0);
        print!("{}", "Compressing package..".bold());
        for entry in WalkDir::new("pkg") {
            let entry = try!(entry);
            let file_name = strip_parent(entry.path().to_path_buf());
            let metadata = try!(fs::metadata(entry.path()));
            if metadata.is_file() {
                let mut file = try!(File::open(entry.path()));
                try!(archive.append_file(file_name, &mut file));
            } else if metadata.is_dir() {
                if entry.path() != "pkg".as_path() {
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

    fn set_env(&self) -> Result<(), Box<error::Error>> {
        let current_dir = try!(env::current_dir());
        let mut pkg_dir = current_dir.clone();
        let mut src_dir = current_dir.clone();
        pkg_dir.push("pkg");
        src_dir.push("src");
        env::set_var("pkg_dir", pkg_dir);
        env::set_var("src_dir", src_dir);
        if let Some(pkg_vers) = self.vers.as_ref() {
            env::set_var("pkg_vers", pkg_vers);
        }
        if let Some(pkg_rel) = self.rel.as_ref() {
            env::set_var("pkg_rel", pkg_rel);
        };
        Ok(())
    }

    fn create_dirs(&self) -> Result<(), Box<error::Error>> {
        let current_dir = try!(env::current_dir());
        let mut pkg_dir = current_dir.clone();
        let mut src_dir = current_dir.clone();
        pkg_dir.push("pkg");
        src_dir.push("src");
        // Don't create src_dir since it should be created by 'handle_source'
        if !pkg_dir.exists() {
            try!(fs::create_dir(&pkg_dir));
        }
        Ok(())
    }

    fn handle_source(&self) -> Result<(), Box<error::Error>> {
        if let Some(source) = self.source.as_ref() {
            let mut index: usize = 0;
            for item in source {
                let pos = item.find('+').unwrap_or(item.len());
                let (cvs, url) = item.split_at(pos);
                if cvs == "git" {
                    let url = url.replace("+", "");
                    println!("{} {}", "Cloning".bold(), &url.bold());
                    try!(clone_repo(&url));
                } else {
                    if self.is_web_get(cvs) {
                        try!(self.web_get(cvs));
                        if let Some(file_name) = cvs.rsplit('/').nth(0) {
                            try!(self.match_hash(index, file_name));
                            try!(self.extract_tar(file_name));
                        };
                    } else {
                        try!(self.match_hash(index, item));
                    }
                }
                index += 1;
            }
        };
        Ok(())
    }

    fn is_web_get(&self, url: &str) -> bool {
        if let Some(_) = url.find("://") {
            return true;
        } else {
            return false;
        }
    }

    fn web_get(&self, url: &str) -> Result<(), Box<error::Error>> {
        if let Some(file_name) = url.rsplit('/').nth(0) {
            println!("Downloading: {}", url);
            let mut res = try!(Client::new().get(url).send());
            let mut buffer = Vec::new();
            try!(res.read_to_end(&mut buffer));
            if res.status != hyper::Ok {
                return Err(Box::new(BuildError::HttpNoFile(res.status, url.to_owned())));
            } else {
                try!(try!(File::create(file_name)).write(&mut buffer));
            }
        }
        Ok(())
    }

    fn extract_tar(&self, path: &str) -> Result<(), Box<error::Error>> {
        if let Ok(decomp) = GzDecoder::new(try!(File::open(path))) {
            let mut archive = Archive::new(decomp);
            try!(archive.unpack("src"));
        } else {
            let mut archive = Archive::new(try!(File::open(path)));
            try!(archive.unpack("src"));
        };
        Ok(())
    }

    fn build(&self) -> Result<(), Box<error::Error>> {
        try!(env::set_current_dir("src"));
        println!("{}", "Beginning build".bold());
        if let Some(script) = self.build.as_ref() {
            try!(self.exec(script));
        };
        Ok(println!("{}", "Build succeeded".bold()))
    }

    fn package(&self) -> Result<(), Box<error::Error>> {
        println!("{}", "Beginning package".bold());
        if let Some(script) = self.package.as_ref() {
            try!(self.exec(script));
        };
        Ok(println!("{}", "Package succeeded".bold()))
    }

    fn pkg_size(&self) -> Result<u64, BuildError> {
        let mut size: u64 = 0;
        for entry in WalkDir::new("pkg") {
            let entry = try!(entry);
            if entry.path() != "pkg".as_path() {
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
pub struct MTree {
    entries: Vec<MTreeEntry>,
}

impl MTree {
    fn new() -> MTree {
        Default::default()
    }

    fn write(&self, path: &str) -> Result<(), BuildError> {
        print!("{}", "Generating MTREE...".bold());
        try!(try!(File::create(path)).write_all(&format!("{}", toml::encode(&self)).as_bytes()));
        Ok(print!("{}\n", "OK".paint(Color::Green)))
    }

    fn add(&mut self, entry: MTreeEntry) {
        self.entries.push(entry);
    }

    #[cfg(target_family = "unix")]
    fn from_dir(path: &str) -> Result<MTree, BuildError> {
        let mut mtree = MTree::new();
        for entry in WalkDir::new(path) {
            let entry = try!(entry);
            if entry.path() != "pkg".as_path() {
                let mut mtree_entry = MTreeEntry::new();
                let file_name = strip_parent(entry.path().to_path_buf());
                mtree_entry.set_path(file_name.as_str());
                if !entry.path().is_dir() {
                    try!(mtree_entry.set_checksum(entry.path().as_str()));
                    try!(mtree_entry.set_size(entry.path().as_str()));
                }
                try!(mtree_entry.set_time());
                try!(mtree_entry.set_mode(entry.path().as_str()));
                mtree.add(mtree_entry);
            }
        }
        Ok(mtree)
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq)]
pub struct MTreeEntry {
    path: String,
    time: String,
    size: u64,
    checksum: String,
    mode: u32,
}

impl MTreeEntry {
    fn new() -> MTreeEntry {
        Default::default()
    }

    fn set_path(&mut self, path: &str) {
        self.path = self.prepend_path(path);
    }

    fn set_time(&mut self) -> Result<(), BuildError> {
        match time::strftime("%m%d%Y%H%M%S", &time::now_utc()) {
            Ok(s) => self.time = s,
            Err(e) => return Err(BuildError::Time(e)),
        };
        Ok(())
    }

    fn set_mode(&mut self, path: &str) -> Result<(), BuildError> {
        let metadata = try!(fs::metadata(path));
        self.mode = metadata.permissions().mode();
        Ok(())
    }

    fn set_size(&mut self, path: &str) -> Result<(), BuildError> {
        match fs::metadata(path) {
            Ok(s) => self.size = s.len(),
            Err(e) => return Err(BuildError::Io(e)),
        };
        Ok(())
    }

    fn set_checksum(&mut self, path: &str) -> Result<(), BuildError> {
        let mut hasher = sha2::Sha256::new();
        let mut buffer = Vec::new();
        try!(try!(File::open(path)).read_to_end(&mut buffer));
        hasher.input(&buffer);
        Ok(self.checksum = hasher.result_str())
    }

    fn prepend_path(&self, path: &str) -> String {
        let mut prepend: PathBuf = PathBuf::from("./");
        prepend.push(path);
        prepend.as_string()
    }
}

#[derive(RustcDecodable,RustcEncodable,Debug,Default,PartialEq,Eq,Hash,Clone)]
pub struct PkgInfo {
    pub name: String,
    vers: String,
    desc: String,
    builddate: String,
    url: String,
    size: u64,
    arch: Vec<Arch>,
    license: String,
    conflicts: Vec<String>,
    provides: String,
    deps: Vec<String>,
    maintainers: Vec<String>,
}

impl PkgInfo {
    pub fn new(build_file: &PackageDesc) -> PkgInfo {
        let mut info: PkgInfo = Default::default();
        info.name = build_file.name.clone().unwrap_or("Unknown".to_owned());
        info.vers = build_file.vers.clone().unwrap_or("Unknown".to_owned());
        info.desc = build_file.desc.clone().unwrap_or("".to_owned());
        info.builddate = build_file.builddate.clone().unwrap_or("Unkown".to_owned());
        info.url = build_file.url.clone().unwrap_or("Uknown".to_owned());
        info.size = build_file.pkg_size().unwrap_or(0);
        info.arch = build_file.arch.clone().unwrap_or(vec![Default::default()]);;
        info.license = build_file.license.clone().unwrap_or("Unkown".to_owned());
        info.conflicts = build_file.conflicts.clone().unwrap_or(vec!["Unkown".to_owned()]);
        info.provides = build_file.provides.clone().unwrap_or("Uknown".to_owned());
        info.deps = build_file.deps.clone().unwrap_or(vec![Default::default()]);
        info.maintainers = build_file.maintainers.clone().unwrap_or(vec!["Unknown".to_owned()]);
        return info;
    }

    pub fn update_size(&mut self, build_file: &PackageDesc) {
        self.size = build_file.pkg_size().unwrap_or(0);
    }

    pub fn write(&self, path: &str) -> Result<(), BuildError> {
        print!("{}", "Generating PKGINFO...".bold());
        try!(try!(File::create(path)).write_all(&format!("{}", toml::encode(&self)).as_bytes()));
        Ok(print!("{}\n", "OK".paint(Color::Green)))
    }
}

pub trait Desc<T> {
    fn from_file(file: &str, name: &str) -> Result<T, Vec<BuildError>>;
    fn from_toml_table(table: toml::Value) -> Result<T, BuildError>;
    fn exec(&self, script: &Vec<String>) -> Result<(), Box<error::Error>>;
    fn print_json(&self);
}

impl<T: Encodable + Decodable> Desc<T> for T {
    fn from_file(file: &str, name: &str) -> Result<T, Vec<BuildError>> {
        try!(assert_toml(file));
        parse_toml_file(file).and_then(|toml| {
            toml.get(name)
                .ok_or(vec![BuildError::NoDesc(name.to_owned())])
                .and_then(|desc| {
                    Decodable::decode(&mut toml::Decoder::new(desc.clone()))
                        .map_err(|err| vec![BuildError::TomlDecode(err)])
                })
        })
    }

    fn from_toml_table(table: toml::Value) -> Result<T, BuildError> {
        Ok(try!(Decodable::decode(&mut toml::Decoder::new(table))))
    }

    fn exec(&self, script: &Vec<String>) -> Result<(), Box<error::Error>> {
        let mut script_clone = script.clone();
        for mut line in &mut script_clone {
            line.push_str(";");
        }
        let mut command = try!(Command::new("sh")
                                   .arg("-c")
                                   .arg(script_clone.join(" "))
                                   .spawn());
        try!(command.wait());
        if let Some(child_output) = command.stdout.as_mut() {
            // Child process has output
            let mut buff = String::new();
            println!("{}", try!(child_output.read_to_string(&mut buff)));
        };
        Ok(())
    }

    fn print_json(&self) {
        println!("{}", json::as_pretty_json(&self));
    }
}

#[test]
fn test_default_arch() {
    let system_arch = env::consts::ARCH;
    let default_arch: Arch = Default::default();
    assert_eq!(system_arch, format!("{:?}", default_arch));
}

#[test]
fn test_new_pkgbuild() {
    let pkg_build = PackageBuild::new();
    assert_eq!(pkg_build.package, None);
    assert_eq!(pkg_build.clean, None);
}

#[test]
#[should_panic]
fn test_pkgbuild_from_file_fail() {
    let pkg_build = match PackageBuild::from_file("none.toml") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(pkg_build, Ok(()));
}

#[test]
fn test_pkgbuild_from_file_success() {
    let pkg_build = match PackageBuild::from_file("example/tar/PKG.toml") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(pkg_build, Ok(()));
}

#[test]
#[should_panic]
fn test_pkgbuild_package() {
    let pkg_build = match PackageBuild::from_file("example/tar/PKG.toml") {
        Ok(pkg) => pkg,
        Err(_) => PackageBuild::new(),
    };
    assert_eq!(pkg_build.package(), None);
}

#[test]
#[should_panic]
fn test_pkgbuild_clean() {
    let pkg_build = match PackageBuild::from_file("example/tar/PKG.toml") {
        Ok(pkg) => pkg,
        Err(_) => PackageBuild::new(),
    };
    assert_eq!(pkg_build.clean(), None);
}

#[test]
fn test_new_empty_pkgdesc() {
    let build_file = PackageDesc::new();
    assert_eq!(build_file.name, None);
    assert_eq!(build_file.vers, None);
    assert_eq!(build_file.build, None);
}

#[test]
#[should_panic]
fn test_pkgdesc_from_file_fail() {
    let build_file = match PackageDesc::from_file("none.toml", "package") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(build_file, Ok(()));
}

#[test]
fn test_pkgdesc_from_file_success() {
    let build_file = match PackageDesc::from_file("example/tar/PKG.toml", "package") {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    assert_eq!(build_file, Ok(()));
}

#[test]
fn test_pkgdesc_from_toml_table_success() {
    let test_desc = PackageDesc::from_file("example/tar/PKG.toml", "package").unwrap();
    if let Ok(toml) = parse_toml_file("example/tar/PKG.toml") {
        if let Some(pkg) = toml.get("package") {
            let desc = PackageDesc::from_toml_table(pkg.clone()).unwrap();
            assert_eq!(test_desc, desc);
        };
    };
}

#[test]
fn test_pkgdesc_print_json() {
    let build_file = PackageDesc::from_file("example/tar/PKG.toml", "package").unwrap();
    build_file.print_json();
}

#[test]
fn test_new_empty_pkginfo() {
    let build_file = PackageDesc::from_file("example/tar/PKG.toml", "package").unwrap();
    let info = PkgInfo::new(&build_file);
    assert_eq!(info.name, build_file.name.unwrap_or("Unknown".to_owned()));
}
