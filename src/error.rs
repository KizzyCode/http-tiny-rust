use ebacktrace::define_error;
use std::{
    error, io, result,
    fmt::{ self, Display, Formatter }
};


/// Creates a new variant
#[macro_export] macro_rules! e {
    ($kind:expr, $($arg:tt)*) => ({ $crate::error::ErrorImpl::new($kind, format!($($arg)*)) })
}
/// Creates a new `ErrorImpl::InOutError` kind
#[macro_export] macro_rules! eio {
    ($($arg:tt)*) => ({ e!($crate::error::ErrorKind::InOutError, $($arg)*) });
}
/// Creates a new `ErrorImpl::InvalidValue` kind
#[macro_export] macro_rules! einval {
    ($($arg:tt)*) => ({ e!($crate::error::ErrorKind::InvalidValue, $($arg)*) });
}


/// The error kind
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// An I/O-related error occurred
    InOutError,
    /// A value is invalid
    InvalidValue
}
impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InOutError => write!(f, "An I/O-error occurred"),
            Self::InvalidValue => write!(f, "A value is invalid")
        }
    }
}
impl error::Error for ErrorKind {
    /* No members to implement */
}


// Define our custom error type
define_error!(ErrorImpl);
impl From<io::Error> for ErrorImpl<ErrorKind> {
    fn from(underlying: io::Error) -> Self {
        ErrorImpl::new(ErrorKind::InOutError, underlying.to_string())
    }
}


/// A nice typealias for our custom error
pub type Error = ErrorImpl<ErrorKind>;
/// A nice typealias for a `Result` with our custom error
pub type Result<T = ()> = result::Result<T, ErrorImpl<ErrorKind>>;
