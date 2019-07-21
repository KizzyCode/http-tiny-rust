[![docs.rs](https://docs.rs/http_header/badge.svg)](https://docs.rs/http_header)
[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/http_header.svg)](https://crates.io/crates/http_header)
[![Download numbers](https://img.shields.io/crates/d/http_header.svg)](https://crates.io/crates/http_header)
[![Travis CI](https://travis-ci.org/KizzyCode/http_header.svg?branch=master)](https://travis-ci.org/KizzyCode/http_header)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/http_header?svg=true)](https://ci.appveyor.com/project/KizzyCode/http-header)
[![dependency status](https://deps.rs/crate/http_header/0.4.0/status.svg)](https://deps.rs/crate/http_header/0.4.0)


# About
`http_header` is a small, dependency-less crate to create, serialize, read and parse 
HTTP/1.*-headers.

It is not designed to be the fastest crate out there, but it's easy to understand and read and
flexible enough to be useful as general-purpose HTTP-header crate.


# Build Library and Documentation
To build the documentation, go into the projects root-directory and run `cargo doc --release`; to
open the documentation in your web-browser, run `cargo doc --open`.

To build the library, go into the projects root-directory and run `cargo build --release`; you can
find the build in target/release.