extern crate libpm;
extern crate rpf;
extern crate pgetopts;

use std::env;
use rpf::*;
use pgetopts::Options;
use libpm::build::*;

pub static MPM: Prog = Prog {
    name: "mpm",
    vers: "0.1.0",
    yr: "2015",
};

fn print_usage(opts: Options) {
    print!("{0}: {1} {2} {3}",
           "Usage".bold(),
           MPM.name.bold(),
           "[OPTION]".underline(),
           "BUILD FILE".underline());
    println!("{}", opts.options());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("p", "print", "Print package file in JSON");
    opts.optflag("b", "build", "Build package");
    opts.optflag("c", "clean", "Clean build environment");
    opts.optflag("h", "help", "Print help information");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            MPM.error(e.to_string(), ExitStatus::OptError);
            panic!();
        }
    };

    if matches.opt_present("h") {
        print_usage(opts);
    } else if matches.opt_present("p") {
        for item in matches.free {
            match PackageBuild::from_file(&*item) {
                Ok(pkg) => pkg.print_json(),
                Err(e) => {
                    for error in e {
                        println!("{}", error.to_string().paint(Color::Red));
                    }
                    MPM.exit(ExitStatus::Error);
                    panic!();
                }
            };
        }
    } else if matches.opt_present("b") {
        for item in matches.free {
            match PackageBuild::from_file(&*item) {
                Ok(pkg_build) => {
                    if let Some(mut package) = pkg_build.package() {
                        if let Err(e) = package.create_pkg() {
                            MPM.error(e.to_string(), ExitStatus::Error);
                        };
                    };
                }
                Err(e) => {
                    for error in e {
                        println!("{}", error.to_string().paint(Color::Red));
                    }
                    MPM.exit(ExitStatus::Error);
                    panic!();
                }
            };
        }
    } else if matches.opt_present("c") {
        for item in matches.free {
            match PackageBuild::from_file(&*item) {
                Ok(pkg_build) => {
                    if let Some(package) = pkg_build.clone().package() {
                        package.set_env().unwrap();
                    };
                    if let Some(clean) = pkg_build.clone().clean() {
                        if let Err(e) = clean.clean() {
                            MPM.error(e.to_string(), ExitStatus::Error);
                        };
                    };
                }
                Err(e) => {
                    for error in e {
                        println!("{}", error.to_string().paint(Color::Red));
                    }
                    MPM.exit(ExitStatus::Error);
                    panic!();
                }
            };
        }
    }
}
