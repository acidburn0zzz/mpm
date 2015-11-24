pub mod build;
pub mod ext;

extern crate toml;
extern crate rustc_serialize;
extern crate rpf;
extern crate tar;
extern crate time;
extern crate walkdir;

use rpf::*;

pub static MPM: Prog = Prog { name: "mpm", vers: "0.1.0", yr: "2015", };
