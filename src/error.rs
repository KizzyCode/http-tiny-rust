//! Implements the crate's error type

use std::io;

/// An error
#[derive(Debug, Error)]
pub enum Error {
    /// The HTTP header is invalid
    #[error("invalid HTTP header")]
    Http,
    /// An in-/out-error
    #[error("in/out error: {source}")]
    InOut {
        /// The underlying in-out-error
        #[from]
        #[source]
        source: io::Error,
    },
}
