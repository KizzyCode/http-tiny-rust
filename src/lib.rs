#![doc = include_str!("../README.md")]

pub mod bytetraits;
mod header;
mod iotraits;

// Re-export public header types
pub use crate::header::{Header, HeaderFields, HeaderStartLine};
