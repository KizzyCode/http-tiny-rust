//! Some I/O helper traits

use std::{
    io::{BufRead, Error, ErrorKind},
    slice,
};

/// Config for pattern matching
#[derive(Debug, PartialEq, Eq)]
pub enum MatchConfig {
    /// Treat an EOF before a match occurrs as error
    Required,
    /// Whether the matched pattern should be trimmed or not
    Trim,
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
            return Err(ErrorKind::UnexpectedEof.into());
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
            return Err(ErrorKind::UnexpectedEof.into());
        }
        Ok(buf)
    }
}
