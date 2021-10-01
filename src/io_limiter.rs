use std::{
    cmp,
    io::{ Read, BufRead, Write, Result }
};


/// An I/O-limiter
///
/// This wrapper is useful if e.g. you want to ensure that `Header` will not read more than say 4 kilobyte to prevent DOS
/// attacks
pub struct IoLimiter<T> {
    /// The underlying I/O element
    inner: T,
    /// The amount of bytes left to read
    read_left: usize,
    /// The amount of bytes left to write
    write_left: usize
}
impl<T> IoLimiter<T> {
    /// Creates a new I/O-limiter
    pub const fn new(io: T, read_max: usize, write_max: usize) -> Self {
        Self { inner: io, read_left: read_max, write_left: write_max }
    }

    /// Returns the underlying I/O-element
    pub fn into_inner(self) -> T {
        self.inner
    }
}
impl<T> Read for IoLimiter<T> where T: Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Read into `buf`
        let to_read = cmp::min(self.read_left, buf.len());
        let read = self.inner.read(&mut buf[..to_read])?;
        
        // Update the counter
        self.read_left -= read;
        Ok(read)
    }
}
impl<T> BufRead for IoLimiter<T> where T: BufRead {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}
impl<T> Write for IoLimiter<T> where T: Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // Write from `buf`
        let to_write = cmp::min(self.write_left, buf.len());
        let written = self.inner.write(&buf[..to_write])?;

        // Update the counter
        self.write_left -= written;
        Ok(written)
    }
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}
