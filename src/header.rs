use crate::{
    error::Error,
    helpers::{
        self, BufReadExt,
        MatchConfig::{Required, Trim},
        SliceU8Ext,
    },
};
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
    iter::FromIterator,
    ops::Deref,
};

/// A HTTP header
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Header {
    /// The start line
    start_line: HeaderStartLine,
    /// The header fields
    fields: HeaderFields,
}
impl Header {
    /// Creates a new HTTP/1.1 header
    pub const fn new(start_line: HeaderStartLine, fields: HeaderFields) -> Self {
        Self { start_line, fields }
    }

    /// The start line
    pub fn start_line(&self) -> &HeaderStartLine {
        &self.start_line
    }
    /// The start line
    pub fn start_line_mut(&mut self) -> &mut HeaderStartLine {
        &mut self.start_line
    }

    /// The header fields
    pub fn fields(&self) -> &HeaderFields {
        &self.fields
    }
    /// The header fields
    pub fn fields_mut(&mut self) -> &mut HeaderFields {
        &mut self.fields
    }

    /// Loads a HTTP header from `bytes` and returns the remaining bytes
    ///
    /// # Warning:
    /// This function will read forever until there is either a `\r\n\r\n` or an EOF. To prevent DOS-attacks, it is
    /// strongly recommended to wrap the `source` into an [`IoLimiter<T>`](crate::IoLimiter) to limit the amount of data
    /// that will be read.
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        // Read the entire header
        let header = source.read_word("\r\n\r\n", [Required])?;
        let mut header = helpers::memreader(header);

        // Read start line and fields
        let start_line = HeaderStartLine::read(&mut header)?;
        let fields = HeaderFields::read(&mut header)?;
        Ok(Self { start_line, fields })
    }
    /// Writes the HTTP header
    pub fn write_all(&self, output: &mut dyn Write) -> Result<(), Error> {
        self.start_line.write_all(output)?;
        self.fields.write_all(output)?;
        output.flush()?;
        Ok(())
    }

    /// Tests whether `buf` contains a complete header
    ///
    /// ## Note
    /// This function tests only, if `buf` contains an end-of-header marker (`\r\n\r\n`) - it does not perform any further
    /// parsing attempts or sanity checks.
    pub fn is_complete<T>(buf: T) -> bool
    where
        T: AsRef<[u8]>,
    {
        buf.as_ref().windows(4).any(|p| p == b"\r\n\r\n")
    }
}

/// The start line
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct HeaderStartLine {
    field0: Vec<u8>,
    field1: Vec<u8>,
    field2: Vec<u8>,
}
impl HeaderStartLine {
    /// Creates a new HTTP/1.1 request
    pub fn new_request<T, U>(method: T, target: U) -> Self
    where
        T: Into<Vec<u8>>,
        U: Into<Vec<u8>>,
    {
        Self { field0: method.into(), field1: target.into(), field2: "HTTP/1.1".into() }
    }
    /// Creates a new HTTP/1.1 response
    pub fn new_response<T>(status: u16, reason: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self { field0: "HTTP/1.1".into(), field1: status.to_string().into(), field2: reason.into() }
    }

    /// Interprets the start line as request and returns the field containing the request method
    pub fn request_method(&self) -> &[u8] {
        &self.field0
    }
    /// Interprets the start line as request and returns the field containing the request method
    pub fn request_method_mut(&mut self) -> &mut Vec<u8> {
        &mut self.field0
    }
    /// Interprets the start line as request and returns the field containing the request target
    pub fn request_target(&self) -> &[u8] {
        &self.field1
    }
    /// Interprets the start line as request and returns the field containing the request target
    pub fn request_target_mut(&mut self) -> &mut Vec<u8> {
        &mut self.field1
    }

    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_binstatus(&self) -> &[u8] {
        &self.field1
    }
    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_binstatus_mut(&mut self) -> &mut Vec<u8> {
        &mut self.field1
    }
    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_reason(&self) -> &[u8] {
        &self.field2
    }
    pub fn response_reason_mut(&mut self) -> &mut Vec<u8> {
        &mut self.field2
    }

    /// Reads the start line from `source`
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        // Read the start line
        let line = source.read_word("\r\n", [Required, Trim])?;
        let mut line = helpers::memreader(line);
        let this = Self {
            field0: line.read_word(" ", [Required, Trim])?,
            field1: line.read_word(" ", [Required, Trim])?,
            field2: line.read_all([Required])?,
        };
        Ok(this)
    }
    /// Writes the HTTP start line
    pub fn write_all(&self, output: &mut dyn Write) -> Result<(), Error> {
        output.write_all(&self.field0)?;
        output.write_all(b" ")?;
        output.write_all(&self.field1)?;
        output.write_all(b" ")?;
        output.write_all(&self.field2)?;
        output.write_all(b"\r\n")?;
        Ok(())
    }
}

/// Some header fields
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct HeaderFields {
    /// The header fields
    fields: BTreeMap<Vec<u8>, Vec<u8>>,
}
impl HeaderFields {
    /// Creates a new header field map
    pub fn new() -> Self {
        Self { fields: BTreeMap::new() }
    }

    /// Gets the value for the field with the given name
    pub fn get<T>(&self, name: T) -> Option<&[u8]>
    where
        T: AsRef<[u8]>,
    {
        let name = name.as_ascii_lowercase();
        self.fields.get(name.as_ref()).map(|s| s.as_ref())
    }
    /// Sets the value for a fiels with the given name
    pub fn set<A, B>(&mut self, name: A, value: B)
    where
        A: AsRef<[u8]>,
        B: Into<Vec<u8>>,
    {
        let name = name.as_ascii_lowercase();
        self.fields.insert(name.into(), value.into());
    }

    /// Reads the header fields from `source`
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        let mut this = HeaderFields::new();
        'read_lines: loop {
            // Unwrap the next line and check for end of header
            let mut line = match source.read_word("\r\n", [Required, Trim])? {
                line if line.is_empty() => break 'read_lines,
                line => helpers::memreader(line),
            };
            let key = line.read_word(":", [Required, Trim])?;
            let value = line.read_all([Required])?;

            // Trim the leading spaces and insert the header pair
            let leading_whitespace = value.iter().take_while(|b| **b == b' ').count();
            let value = &value[leading_whitespace..];
            this.set(key, value);
        }

        Ok(this)
    }
    /// Writes the HTTP header fields
    pub fn write_all(&self, output: &mut dyn Write) -> Result<(), Error> {
        for (key, value) in self.fields.iter() {
            output.write_all(key)?;
            output.write_all(b": ")?;
            output.write_all(value)?;
            output.write_all(b"\r\n")?;
        }
        output.write_all(b"\r\n")?;
        Ok(())
    }
}
impl Deref for HeaderFields {
    type Target = BTreeMap<Vec<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
impl<K, V> FromIterator<(K, V)> for HeaderFields
where
    K: Into<Vec<u8>>,
    V: Into<Vec<u8>>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(pairs: T) -> Self {
        let fields =
            pairs.into_iter().map(|(k, v)| (k.into(), v.into())).map(|(k, v)| (k.to_ascii_lowercase(), v)).collect();
        Self { fields }
    }
}
impl IntoIterator for HeaderFields {
    type Item = <BTreeMap<Vec<u8>, Vec<u8>> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Vec<u8>, Vec<u8>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}
