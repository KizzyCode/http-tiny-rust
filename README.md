[![docs.rs](https://docs.rs/http_tiny/badge.svg)](https://docs.rs/http_tiny)
[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/http_tiny.svg)](https://crates.io/crates/http_tiny)
[![Download numbers](https://img.shields.io/crates/d/http_tiny.svg)](https://crates.io/crates/http_tiny)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/http_tiny-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/http-tiny-rust)
[![dependency status](https://deps.rs/crate/http_tiny/latest/status.svg)](https://deps.rs/crate/http_tiny)


# About
`http_tiny` is a small, nearly dependency-less crate to create, serialize, read and parse HTTP/1.1-headers.

It is not designed to be the fastest crate out there, but it's easy to understand and read and flexible enough to be
useful as general-purpose HTTP-header crate.

## Query strings
Please note that query string parsing and percent encoding has been
[moved to a different crate](https://crates.io/crates/querystring_tiny) for ease of maintainance.
