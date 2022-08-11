#![doc = include_str!("../README.md")]

/// Implements error types with support for `Backtrace` and some additional helpers
#[macro_use]
pub mod error;

/// A HTTP header implementation
mod header;
/// Some internal helpers
mod helpers;
/// A wrapper to limit IO
mod limiter;

// Re-export public types
pub use crate::{
    header::{Header, HeaderFields, HeaderStartLine},
    limiter::Limiter,
};
