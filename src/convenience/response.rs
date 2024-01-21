//! Response builder extensions

use crate::{
    bytetraits::IntoBytes,
    convenience::constants::{HEADER_CONTENTLENGTH, HEADER_CONTENTTYPE},
    Header, HeaderFields, HeaderStartLine,
};
use std::io::{self, BufReader, Error, Stdin};

/// A response builder extension trait
pub trait ResponseBuilder
where
    Self: Sized,
{
    /// Creates a new response
    fn new_response<S, R>(status: S, reason: R) -> Self
    where
        S: IntoBytes,
        R: IntoBytes;

    /// Sets an arbitrary header key-value pair
    fn set_header<K, V>(self, key: K, value: V) -> Self
    where
        K: IntoBytes,
        V: IntoBytes;

    /// Sets the status/reason tuple for `self`
    fn set_content_type<T>(self, type_: T) -> Self
    where
        T: IntoBytes,
    {
        self.set_header(HEADER_CONTENTTYPE, type_)
    }

    /// Sets the "Content-Length" header for `self`
    fn set_content_length<T>(self, length: T) -> Self
    where
        T: IntoBytes,
    {
        self.set_header(HEADER_CONTENTLENGTH, length)
    }
}
impl ResponseBuilder for Header {
    fn new_response<S, R>(status: S, reason: R) -> Self
    where
        S: IntoBytes,
        R: IntoBytes,
    {
        let start_line = HeaderStartLine::new_response(status, reason);
        let fields = HeaderFields::new();
        Header::new(start_line, fields)
    }

    fn set_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoBytes,
        V: IntoBytes,
    {
        self.fields_mut().set(key, value);
        self
    }
}

/// Adds convenience methods to access a header as response header
pub trait ResponseHeader
where
    Self: Sized,
{
    /// Reads `Self` from stdin (e.g. in CGI contexts)
    fn from_stdin() -> Result<(Self, BufReader<Stdin>), Error>;

    /// The response status code
    fn status(&self) -> &[u8];
    /// The response status reason
    fn reason(&self) -> &[u8];
    /// The response HTTP version
    fn version(&self) -> &[u8];
}
impl ResponseHeader for Header {
    fn from_stdin() -> Result<(Self, BufReader<Stdin>), Error> {
        let mut stdin = BufReader::new(io::stdin());
        let this = Header::read(&mut stdin)?;
        Ok((this, stdin))
    }

    fn status(&self) -> &[u8] {
        self.start_line().field1()
    }

    fn reason(&self) -> &[u8] {
        self.start_line().field2()
    }

    fn version(&self) -> &[u8] {
        self.start_line().field0()
    }
}
