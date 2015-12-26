extern crate toml;
extern crate walkdir;
extern crate rustc_serialize;

use std::io;
use std::io::prelude::*;
use std::error::*;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum BuildError {
    Io(io::Error),
    WalkDir(walkdir::Error),
    TomlParse(toml::ParserError),
    TomlDecode(toml::DecodeError),
    JsonEncode(rustc_serialize::json::EncoderError),
    NonToml(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildError::Io(ref err) => write!(f, "i/o error, {}", err),
            BuildError::WalkDir(ref err) => write!(f, "directory walking error, {}", err),
            BuildError::TomlParse(ref err) => write!(f, "toml parsing error, {}", err),
            BuildError::TomlDecode(ref err) => write!(f, "toml decoding error, {}", err),
            BuildError::JsonEncode(ref err) => write!(f, "json encoding error {}", err),
            BuildError::NonToml(ref file) => write!(f, "'{}' is not a toml file", file),
        }
    }
}

impl Error for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::Io(ref err) => err.description(),
            BuildError::WalkDir(ref err) => err.description(),
            BuildError::TomlParse(ref err) => err.description(),
            BuildError::TomlDecode(ref err) => err.description(),
            BuildError::JsonEncode(ref err) => err.description(),
            BuildError::NonToml(..) => "toml file error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            BuildError::Io(ref err) => Some(err),
            BuildError::WalkDir(ref err) => Some(err),
            BuildError::TomlParse(ref err) => Some(err),
            BuildError::TomlDecode(ref err) => Some(err),
            BuildError::JsonEncode(ref err) => Some(err),
            BuildError::NonToml(..) => None,
        }
    }
}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> BuildError {
        BuildError::Io(err)
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
