//! # About
//! `http_header` is a small, dependency-less crate to create, serialize, read and parse
//! HTTP/1.1-headers.
//!
//! It is not designed to be the fastest crate out there, but it's easy to understand and read and
//! flexible enough to be useful as general-purpose HTTP-header crate.

/// Implements error types with support for `Backtrace` and some additional helpers
#[macro_use] pub mod error;
/// A HTTP header implementation
mod header;
// A URL request target implementation
mod request_target;
/// Some internal helpers
mod helpers;
/// A wrapper to limit IO
mod io_limiter;

// Re-export public types
pub use crate::{
    io_limiter::IoLimiter,
    header::{ Header, HeaderStartLine, HeaderFields, HeaderFieldsExt },
    request_target::{ RequestTarget, RequestTargetPath, QueryString }
};
