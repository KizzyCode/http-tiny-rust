//! A HTTP 1/\*-header implementation

use crate::{
    bytetraits::{AsBytes, IntoBytes},
    iotraits::{
        BufReadExt,
        MatchConfig::{Required, Trim},
    },
};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{BufRead, BufReader, Error, Write},
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
    /// strongly recommended to wrap the `source` into a [`std::io::Take`] to limit the amount of data that will be read.
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        // Read the entire header
        let header = source.read_word("\r\n\r\n", [Required])?;
        let mut header = header.as_slice();
        let mut header = BufReader::new(&mut header);

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
    field0: Cow<'static, [u8]>,
    field1: Cow<'static, [u8]>,
    field2: Cow<'static, [u8]>,
}
impl HeaderStartLine {
    /// Creates a new HTTP/1.1 request
    pub fn new_request<T, U>(method: T, target: U) -> Self
    where
        T: IntoBytes,
        U: IntoBytes,
    {
        Self { field0: method.into_bytes(), field1: target.into_bytes(), field2: "HTTP/1.1".into_bytes() }
    }
    /// Creates a new HTTP/1.1 response
    pub fn new_response<T, U>(status: T, reason: U) -> Self
    where
        T: IntoBytes,
        U: IntoBytes,
    {
        let status = status.into_bytes();
        assert!(status.iter().all(u8::is_ascii_digit), "non-numeric HTTP status code");
        Self { field0: "HTTP/1.1".into_bytes(), field1: status, field2: reason.into_bytes() }
    }

    /// Interprets the start line as request and returns the field containing the request method
    pub fn request_method(&self) -> &[u8] {
        &self.field0
    }
    /// Interprets the start line as request and returns the field containing the request method
    pub fn request_method_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field0
    }
    /// Interprets the start line as request and returns the field containing the request target
    pub fn request_target(&self) -> &[u8] {
        &self.field1
    }
    /// Interprets the start line as request and returns the field containing the request target
    pub fn request_target_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field1
    }
    /// Interprets the start line as request and returns the field containing the HTTP version
    pub fn request_version(&self) -> &[u8] {
        &self.field2
    }
    /// Interprets the start line as request and returns the field containing the HTTP version
    pub fn request_version_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field2
    }

    /// Interprets the start line as response and returns the field containing the HTTP version
    pub fn response_version(&self) -> &[u8] {
        &self.field0
    }
    /// Interprets the start line as response and returns the field containing the HTTP version
    pub fn response_version_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field0
    }
    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_binstatus(&self) -> &[u8] {
        &self.field1
    }
    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_binstatus_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field1
    }
    /// Interprets the start line as response and returns the field containing the response status code
    pub fn response_reason(&self) -> &[u8] {
        &self.field2
    }
    pub fn response_reason_mut(&mut self) -> &mut Cow<'static, [u8]> {
        &mut self.field2
    }

    /// Reads the start line from `source`
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        // Read the start line
        let line = source.read_word("\r\n", [Required, Trim])?;
        let mut line = line.as_slice();
        let mut line = BufReader::new(&mut line);

        // Split the start line into its fields
        let this = Self {
            field0: line.read_word(" ", [Required, Trim])?.into_bytes(),
            field1: line.read_word(" ", [Required, Trim])?.into_bytes(),
            field2: line.read_all([Required])?.into_bytes(),
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
    fields: BTreeMap<Cow<'static, [u8]>, Cow<'static, [u8]>>,
}
impl HeaderFields {
    /// Creates a new header field map
    pub fn new() -> Self {
        Self { fields: BTreeMap::new() }
    }

    /// Gets the value for the field with the given name
    pub fn get<T>(&self, name: T) -> Option<&[u8]>
    where
        T: for<'a> AsBytes<'a>,
    {
        let name = name.into_ascii_lowercase();
        let value = self.fields.get(name.as_ref())?;
        Some(value.deref())
    }
    /// Sets the value for a fiels with the given name
    pub fn set<A, B>(&mut self, name: A, value: B)
    where
        A: IntoBytes,
        B: IntoBytes,
    {
        let name = name.into_ascii_lowercase();
        self.fields.insert(name, value.into_bytes());
    }

    /// Reads the header fields from `source`
    pub fn read<T>(source: &mut T) -> Result<Self, Error>
    where
        T: BufRead,
    {
        let mut this = HeaderFields::new();
        'read_lines: loop {
            // Unwrap the next line and check for end of header
            let line = source.read_word("\r\n", [Required, Trim])?;
            let mut line = line.as_slice();
            let mut line = match line {
                line if line.is_empty() => break 'read_lines,
                _ => BufReader::new(&mut line),
            };

            // Split the line into key-value
            let key = line.read_word(":", [Required, Trim])?;
            let mut value = line.read_all([Required])?;

            // Trim the leading spaces
            let leading_whitespace = value.iter().take_while(|b| **b == b' ').count();
            value.copy_within(leading_whitespace.., 0);
            value.truncate(value.len() - leading_whitespace);

            // Insert the pair
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
    type Target = BTreeMap<Cow<'static, [u8]>, Cow<'static, [u8]>>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
impl<K, V> FromIterator<(K, V)> for HeaderFields
where
    K: IntoBytes,
    V: IntoBytes,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(pairs: T) -> Self {
        let fields = pairs.into_iter().map(|(k, v)| (k.into_ascii_lowercase(), v.into_bytes())).collect();
        Self { fields }
    }
}
impl IntoIterator for HeaderFields {
    type Item = <BTreeMap<Cow<'static, [u8]>, Cow<'static, [u8]>> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Cow<'static, [u8]>, Cow<'static, [u8]>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}
