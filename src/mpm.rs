extern crate util;
extern crate rpf;
extern crate pgetopts;

use std::env;
use rpf::*;
use pgetopts::Options;
use util::build::*;

pub static MPM: Prog = Prog { name: "mpm", vers: "0.1.0", yr: "2015", };

fn print_usage(opts: Options) {
    print!("{0}: {1} ", "Usage".bold(), MPM.name.bold());
    println!("{}", opts.options());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("p", "print", "Print package file");
    opts.optflag("b", "build", "Build package");
    opts.optflag("h", "help", "Print help information");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(e) => {
            MPM.error(e.to_string(), ExitStatus::OptError);
            panic!();
        }
    };

    if matches.opt_present("h") {
        print_usage(opts);
     } else if matches.opt_present("p") {
        for item in matches.free {
            let pkg = match BuildFile::from_file(&*item) {
                Some(s) => {
                    match s.assert_toml(&*item) {
                        Ok(_) => { },
                        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error) },
                    }
                    s
                }
                // This is a weak and undetailed error
                None => { MPM.error("empty build file", ExitStatus::Error);
                    let s = BuildFile::new();
                    match s.assert_toml(&*item) {
                        Ok(_) => { },
                        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error) },
                    }
                    s
                }
            };
            pkg.print_json();
        }
    } else if matches.opt_present("b") {
        for item in matches.free {
            let pkg = match BuildFile::from_file(&*item) {
                Some(s) => {
                    match s.assert_toml(&*item) {
                        Ok(_) => { },
                        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error) },
                    }
                    s
                }
                // This is a weak and undetailed error
                None => {
                    let s = BuildFile::new();
                    match s.assert_toml(&*item) {
                        Ok(_) => { },
                        Err(e) => { MPM.error(e.to_string(), ExitStatus::Error) },
                    }
                    s
                }
            };
            match pkg.build() {
                Ok(s) => { s },
                Err(e) => { MPM.error(e.to_string(), ExitStatus::Error) }
            };
        }
    }
}
