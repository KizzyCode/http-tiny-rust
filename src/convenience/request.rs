//! Request builder extensions

use crate::{
    bytetraits::IntoBytes, convenience::constants::HEADER_CONTENTLENGTH, Header, HeaderFields, HeaderStartLine,
};
use std::io::{self, BufReader, Error, Stdin};

/// A request builder extension trait
pub trait RequestBuilder
where
    Self: Sized,
{
    /// Creates a new request
    fn new_request<M, T>(method: M, target: T) -> Self
    where
        M: IntoBytes,
        T: IntoBytes;

    /// Sets an arbitrary header key-value pair
    fn set_header<K, V>(self, key: K, value: V) -> Self
    where
        K: IntoBytes,
        V: IntoBytes;

    /// Sets the "Content-Length" header for `self`
    fn set_content_length<T>(self, length: T) -> Self
    where
        T: IntoBytes,
    {
        self.set_header(HEADER_CONTENTLENGTH, length)
    }
}
impl RequestBuilder for Header {
    fn new_request<M, T>(method: M, target: T) -> Self
    where
        M: IntoBytes,
        T: IntoBytes,
    {
        let start_line = HeaderStartLine::new_request(method, target);
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

/// Adds convenience methods to access a header as request header
pub trait RequestHeader
where
    Self: Sized,
{
    /// Reads `Self` from stdin (e.g. in CGI contexts)
    fn from_stdin() -> Result<(Self, BufReader<Stdin>), Error>;

    /// The request method
    fn method(&self) -> &[u8];
    /// The request target URL
    fn target(&self) -> &[u8];
    /// The request HTTP version
    fn version(&self) -> &[u8];
}
impl RequestHeader for Header {
    fn from_stdin() -> Result<(Self, BufReader<Stdin>), Error> {
        let mut stdin = BufReader::new(io::stdin());
        let this = Header::read(&mut stdin)?;
        Ok((this, stdin))
    }

    fn method(&self) -> &[u8] {
        self.start_line().field0()
    }

    fn target(&self) -> &[u8] {
        self.start_line().field1()
    }

    fn version(&self) -> &[u8] {
        self.start_line().field2()
    }
}
