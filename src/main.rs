extern crate util;
extern crate rpf;
extern crate pgetopts;

use util::*;
use rpf::*;
use pgetopts::{Options};
use std::env;

fn print_usage(opts: Options) {
    print!("{0}: {1} ", "Usage".bold(), util::MPM.name.bold());
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
            util::MPM.error(e.to_string(), ExitStatus::OptError);
            panic!();
        }
    };

    if matches.opt_present("h") {
        print_usage(opts);
     } else if matches.opt_present("p") {
        for item in matches.free {
            let pkg = util::BuildFile::from_file(&*item).unwrap();
            pkg.print_json();
        }
    } else if matches.opt_present("b") {
        for item in matches.free {
            let pkg = util::BuildFile::from_file(&*item).unwrap();
            pkg.build();
            pkg.create_pkg();
        }
    }
}
