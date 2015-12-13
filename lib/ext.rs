extern crate toml;
extern crate rustc_serialize;
extern crate rpf;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use toml::{Value};
use rustc_serialize::json::Json;

// Parses a toml file.
pub fn parse_toml_file<T: AsRef<Path>>
(file: T) -> Result<toml::Table, Vec<toml::ParserError>> {
    let mut buff = String::new();
    let mut file = match File::open(file) {
        Ok(s) => { s },
        Err(_) => { panic!(); }
    };
    match file.read_to_string(&mut buff) {
        Ok(s) => { s },
        Err(_) => { panic!(); }
    };
    let mut parser = toml::Parser::new(&buff);
    match parser.parse() {
        Some(s) => { return Ok(s) },
        None => { return Err(parser.errors) }
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

#[test]
fn test_parse_toml_file() {
    let json = parse_toml_file("example/PKG.toml");
}

#[test]
fn test_convert() {
    let toml = "foo = 'bar'";
    let val: toml::Value = toml.parse().unwrap();
    let json = convert(val);
}
