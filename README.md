# mpm

'Modern package manager'* (mpm) is a package manager written in Rust. This is
mostly and primarily a side project, for fun.

## Building mpm
`mpm` builds on stable rustc (as of 1.5). To build run `cargo build`.

[**Get `cargo/rustc`**](https://www.rust-lang.org/downloads.html)

## Current Status:
- [x] Builds a thing
	- [ ] Creates a proper build environment
- [x] Packages a thing
	- [ ] Includes metadata
	- [ ] Signed packages
- [ ] Installs a thing

## Creating a Package
`mpm` uses toml files to describe a package. The following is an example
schema (found in the `example/tar` directory), please note that this is subject to
change:
```toml
[package]
maintainers = [ "Alberto Corona<ac@albertocorona.com>" ]
desc = "Package file for testing mpm"
name = "hello-mpm"
vers = "0.0.1"
rel = "1"
arch = [""]
url = "https://github.com"
license = "BSD"
makedeps = ["make", "git"]
deps = [ ]
provides = "test"
prefix = "/usr"
conflicts = ["test-git"]
source = ["https://github.com/0X1A/hello-mpm/releases/download/0.0.0/hello-mpm.tar.gz"]
build = [
	'make',
	'make DEST=build install'
]

[clean]
script = [
	'rm -v hello-mpm.tar.gz',
	'rm -v hello-mpm-x86_64.pkg.tar',
	'rm -rv build',
]
```

`mpm` is heavily influences by `pacman`

## Building a Package
For general help and options run `mpm -h`. To build a package run `mpm -b
PKG.toml`

## Why?
Because why not and also because why not.

<sub>*Nothing about `mpm` is particularly modern</sub>
