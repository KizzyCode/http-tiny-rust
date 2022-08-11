//! Implements the crate's error type

use std::{
    error,
    fmt::{self, Display, Formatter},
    io,
};

/// Creates a new error
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error, file!(), line!())
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The underlying I/O error
    error: String,
    /// The underlying source error
    source: Option<Box<dyn std::error::Error>>,
    /// The file where the error occurred
    file: &'static str,
    /// The line where the error occurred
    line: u32,
}
impl Error {
    /// Creates a new error
    #[doc(hidden)]
    pub fn new<T>(desc: T, file: &'static str, line: u32) -> Self
    where
        T: ToString,
    {
        Self { error: desc.to_string(), source: None, file, line }
    }
    /// Creates a new error with a source error
    #[doc(hidden)]
    pub fn with_error<T>(error: T, file: &'static str, line: u32) -> Self
    where
        T: error::Error + 'static,
    {
        let error = Box::new(error);
        Self { error: format!("{error}"), source: Some(error), file, line }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}:{}", self.error, self.file, self.line)
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        let source = self.source.as_ref()?;
        Some(source.as_ref())
    }
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::with_error(error, file!(), line!())
    }
}
