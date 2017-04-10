#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate libpm;
extern crate rpf;
extern crate pgetopts;
extern crate ansi_term;

use std::process::exit;
use clap::App;
use libpm::build::*;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};
use ansi_term::Colour;

pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), libpm::error::PkgError> {
        Ok(log::set_logger(|max_log_level| {
                               max_log_level.set(LogLevelFilter::Info);
                               Box::new(Logger)
                           })?)
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            match record.level() {
                LogLevel::Error => {
                    println!("{}: {}", Colour::Red.bold().paint("error"), record.args());
                }
                LogLevel::Warn => {}
                LogLevel::Info => {
                    println!("{}: {}", Colour::Green.bold().paint("yabs"), record.args());
                }
                LogLevel::Debug => {}
                LogLevel::Trace => {}
            };
        }
    }
}

fn run() -> i32 {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    if let Some(matches) = matches.subcommand_matches("build") {
        match PackageBuild::from_file("PKG.toml") {
            Ok(pkg_build) => {
                if let Some(mut package) = pkg_build.clone().package() {
                    if matches.is_present("clean") {
                        if let Err(err) = package.set_env() {
                            error!("{}", err.to_string());
                            return -1;
                        }
                        if let Some(clean) = pkg_build.clean() {
                            if let Err(err) = clean.clean() {
                                error!("{}", err.to_string());
                                return -1;
                            }
                        }
                        return 0;
                    } else {
                        if let Err(err) = package.create_pkg() {
                            error!("{}", err.to_string());
                            return -1;
                        }
                        return 0;
                    }
                }
            }
            Err(err) => {
                error!("{}", err.to_string());
                return -1;
            }
        }
    }
    return 0;
}

fn main() {
    match run() {
        error @ 1...10 => exit(error),
        _ => (),
    }
}
