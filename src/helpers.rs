//! Some helper functions

use crate::error::Error;
use std::{
    borrow::Cow,
    io::{BufRead, BufReader, Cursor},
    slice,
};

/// Config for pattern matching
#[derive(Debug, PartialEq, Eq)]
pub enum MatchConfig {
    /// Treat early EOF as an error
    Required,
    /// Whether the matched pattern should be trimmed or not
    Trim,
}

/// Creates a readable data source over a slice
pub fn memreader<T>(data: T) -> impl BufRead
where
    T: AsRef<[u8]>,
{
    BufReader::new(Cursor::new(data))
}

/// A convenience extension for `BufRead`
pub trait BufReadExt
where
    Self: BufRead,
{
    /// Takes a peek at the next byte or returns `None` in case of EOF
    fn peek_one(&mut self) -> Result<Option<u8>, Error>;

    /// Reads the next byte or returns `None` in case of EOF
    fn read_one(&mut self) -> Result<Option<u8>, Error>;

    /// Reads a single "word" terminated by `delimiter` (not included)
    fn read_word<T, F>(&mut self, delimiter: T, flags: F) -> Result<Vec<u8>, Error>
    where
        T: AsRef<[u8]>,
        F: AsRef<[MatchConfig]>;

    /// Reads the remaining bytes from `source`
    fn read_all<T>(&mut self, flags: T) -> Result<Vec<u8>, Error>
    where
        T: AsRef<[MatchConfig]>;
}
impl<T> BufReadExt for T
where
    T: BufRead,
{
    fn peek_one(&mut self) -> Result<Option<u8>, Error> {
        let buf = self.fill_buf()?;
        Ok(buf.first().copied())
    }

    fn read_one(&mut self) -> Result<Option<u8>, Error> {
        // Check if we are EOF
        if self.fill_buf()?.is_empty() {
            return Ok(None);
        }

        // Read the next byte
        let mut byte = 0;
        self.read_exact(slice::from_mut(&mut byte))?;
        Ok(Some(byte))
    }

    fn read_word<D, F>(&mut self, delimiter: D, flags: F) -> Result<Vec<u8>, Error>
    where
        D: AsRef<[u8]>,
        F: AsRef<[MatchConfig]>,
    {
        // Deconstruct delimiter
        let delimiter = delimiter.as_ref();
        let flags = flags.as_ref();
        let mut line = Vec::new();

        // Read the word
        'read_loop: while !line.ends_with(delimiter) {
            match self.read_one()? {
                Some(next) => line.push(next),
                None => break 'read_loop,
            }
        }

        // Assert that the delimiter exists if required
        if flags.contains(&MatchConfig::Required) && !line.ends_with(delimiter) {
            return Err(Error::Http);
        }

        // Trim the match if required
        if flags.contains(&MatchConfig::Trim) && line.ends_with(delimiter) {
            let len_without_delimiter = line.len() - delimiter.len();
            line.resize(len_without_delimiter, 0);
        }
        Ok(line)
    }

    fn read_all<F>(&mut self, flags: F) -> Result<Vec<u8>, Error>
    where
        F: AsRef<[MatchConfig]>,
    {
        // Load the flags and read all data
        let flags = flags.as_ref();
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;

        // Check for early EOF
        if flags.contains(&MatchConfig::Required) && buf.is_empty() {
            return Err(Error::Http);
        }
        Ok(buf)
    }
}

/// A convenience extension for `[u8]`
pub trait SliceU8Ext
where
    Self: AsRef<[u8]>,
{
    /// Converts slice to ASCII-lowercase (zero copy if the slice is already lowercase only)
    fn as_ascii_lowercase(&self) -> Cow<[u8]>;
}
impl<T> SliceU8Ext for T
where
    T: AsRef<[u8]>,
{
    fn as_ascii_lowercase(&self) -> Cow<[u8]> {
        let slice = self.as_ref();
        match slice.iter().all(|b| b.is_ascii_lowercase()) {
            true => Cow::Borrowed(slice),
            false => Cow::Owned(slice.to_ascii_lowercase()),
        }
    }
}
