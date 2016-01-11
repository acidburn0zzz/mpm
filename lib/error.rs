extern crate toml;
extern crate time;
extern crate walkdir;
extern crate rustc_serialize;
extern crate git2;
extern crate hyper;

use std::io;
use std::io::prelude::*;
use std::error::*;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum BuildError {
    Io(io::Error),
    Time(time::ParseError),
    WalkDir(walkdir::Error),
    TomlParse(toml::ParserError),
    TomlDecode(toml::DecodeError),
    JsonEncode(rustc_serialize::json::EncoderError),
    Git(git2::Error),
    Hyper(hyper::error::Error),
    NonToml(String),
    NoDesc(String),
    HttpNoFile(hyper::status::StatusCode, String),
    HashMismatch(String, String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildError::Io(ref err) => write!(f, "i/o error, {}", err),
            BuildError::Time(ref err) => write!(f, "time error, {}", err),
            BuildError::WalkDir(ref err) => write!(f, "directory walking error, {}", err),
            BuildError::TomlParse(ref err) => write!(f, "toml parsing error, {}", err),
            BuildError::TomlDecode(ref err) => write!(f, "toml decoding error, {}", err),
            BuildError::JsonEncode(ref err) => write!(f, "json encoding error {}", err),
            BuildError::Git(ref err) => write!(f, "git error {}", err),
            BuildError::Hyper(ref err) => write!(f, "hyper error {}", err),
            BuildError::NonToml(ref file) => write!(f, "'{}' is not a toml file", file),
            BuildError::NoDesc(ref name) => write!(f, "no '{}' section found in PKG.toml", name),
            BuildError::HttpNoFile(ref status_code, ref url) => {
                write!(f, "HTTP: {} from '{}'", status_code, url)
            }
            BuildError::HashMismatch(ref file, ref hash) => {
                write!(f, "hash mismatch: '{}' : '{}'", file, hash)
            }
        }
    }
}

impl Error for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::Io(ref err) => err.description(),
            BuildError::Time(ref err) => err.description(),
            BuildError::WalkDir(ref err) => err.description(),
            BuildError::TomlParse(ref err) => err.description(),
            BuildError::TomlDecode(ref err) => err.description(),
            BuildError::JsonEncode(ref err) => err.description(),
            BuildError::Git(ref err) => err.description(),
            BuildError::Hyper(ref err) => err.description(),
            BuildError::NonToml(..) => "toml file error",
            BuildError::NoDesc(..) => "no 'package' nor 'clean' section found in PKG.toml",
            BuildError::HttpNoFile(..) => "http file error",
            BuildError::HashMismatch(..) => "hash mismatch",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            BuildError::Io(ref err) => Some(err),
            BuildError::Time(ref err) => Some(err),
            BuildError::WalkDir(ref err) => Some(err),
            BuildError::TomlParse(ref err) => Some(err),
            BuildError::TomlDecode(ref err) => Some(err),
            BuildError::JsonEncode(ref err) => Some(err),
            BuildError::Git(ref err) => Some(err),
            BuildError::Hyper(ref err) => Some(err),
            BuildError::NonToml(..) => None,
            BuildError::NoDesc(..) => None,
            BuildError::HttpNoFile(..) => None,
            BuildError::HashMismatch(..) => None,
        }
    }
}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> BuildError {
        BuildError::Io(err)
    }
}

impl From<time::ParseError> for BuildError {
    fn from(err: time::ParseError) -> BuildError {
        BuildError::Time(err)
    }
}

impl From<walkdir::Error> for BuildError {
    fn from(err: walkdir::Error) -> BuildError {
        BuildError::WalkDir(err)
    }
}

impl From<toml::ParserError> for BuildError {
    fn from(err: toml::ParserError) -> BuildError {
        BuildError::TomlParse(err)
    }
}

impl From<toml::DecodeError> for BuildError {
    fn from(err: toml::DecodeError) -> BuildError {
        BuildError::TomlDecode(err)
    }
}

impl From<rustc_serialize::json::EncoderError> for BuildError {
    fn from(err: rustc_serialize::json::EncoderError) -> BuildError {
        BuildError::JsonEncode(err)
    }
}

impl From<git2::Error> for BuildError {
    fn from(err: git2::Error) -> BuildError {
        BuildError::Git(err)
    }
}

impl From<hyper::error::Error> for BuildError {
    fn from(err: hyper::error::Error) -> BuildError {
        BuildError::Hyper(err)
    }
}

impl From<BuildError> for Vec<BuildError> {
    fn from(err: BuildError) -> Vec<BuildError> {
        vec![err]
    }
}
