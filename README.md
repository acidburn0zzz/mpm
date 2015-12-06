#mpm
'Modern package manager'* (mpm) is a package manager written in Rust. This is
mostly and primarily a side project, for fun.

Current Status:
- [x] Builds a thing
	- [ ] Creates a proper build environment
- [x] Packages a thing
	- [ ] Includes metadata
	- [ ] Signed packages
- [ ] Installs a thing

# Creating a Package
`mpm` uses toml files to describe a package. The following is an example
schema:
```toml
maintainers = [ "Alberto Corona<ac@albertocorona.com>" ]
desc = "Package file for testing mpm"
name = "hello-mpm"
vers = "0.0.1"
rel = "1"
arch = "any"
url = "https://github.com"
license = "BSD"
makedeps = ["make", "git"]
deps = [ ]
provides = "test"
prefix = "/usr"
conflicts = ["test-git"]
source = "example"
build = [
	'make',
	'make DEST=build install'
]
```

`mpm` is heavily influences by `pacman`

# Why?
Because why not and also because why not.

<sub>*Nothing about `mpm` is particularly modern</sub>
