use crate::{ error::Result, helpers::BufReadExt };
use std::{
    slice, marker::PhantomData,
    io::{ BufRead, Write },
    ops::{ Deref, DerefMut }
};


/// The allowwed chars
const ALLOWED_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";


/// A buffer that can hold up to three bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Buf {
    /// The buffer
    buf: [u8; 3],
    /// The length
    len: usize
}
impl Buf {
    /// Creates a new 1-byte buffer
    pub const fn new1() -> Self {
        Self { buf: [0; 3], len: 1 }
    }
    /// Creates a new 3-byte buffer
    pub const fn new3() -> Self {
        Self { buf: [0; 3], len: 3 }
    }
}
impl From<u8> for Buf {
    fn from(byte: u8) -> Self {
        Self { buf: [byte, 0, 0], len: 1 }
    }
}
impl From<[u8; 3]> for Buf {
    fn from(bytes: [u8; 3]) -> Self {
        Self { buf: bytes, len: bytes.len() }
    }
}
impl Deref for Buf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}
impl DerefMut for Buf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf[..self.len]
    }
}


/// A percent encoder
#[derive(Debug, Default)]
pub struct PercentEncoder {
    _private: PhantomData<()>
}
impl PercentEncoder {
    /// Creates a new percent encoder
    pub const fn new() -> Self {
        Self { _private: PhantomData }
    }
    
    /// Percent-encodes some data
    pub fn copy<I, O>(&self, source: &mut I, sink: &mut O) -> Result<usize> where I: BufRead, O: Write {
        // Copy-encode all bytes
        let mut written = 0;
        while let Some(byte) = source.read_one()? {
            // Encode the byte
            let encoded = match ALLOWED_CHARS.contains(&byte) {
                true => Buf::from(byte),
                false => self.encode_byte(byte)
            };
            sink.write_all(&encoded)?;
            written += encoded.len();
        }
        Ok(written)
    }

    /// Encodes a nibble into a hex char
    fn encode_nibble(&self, nibble: u8) -> u8 {
        match nibble {
            0x0..=0x9 => nibble + b'0',
            0xA..=0xF => (nibble - 0xA) + b'A',
            nibble => unreachable!("Invalid nibble value: {nibble}")
        }
    }
    /// Encodes a byte
    fn encode_byte(&self, byte: u8) -> Buf {
        let (high, low) = (byte >> 4, byte & 0xF);
        Buf::from([b'%', self.encode_nibble(high), self.encode_nibble(low)])
    }
}


/// A percent decoder
#[derive(Debug, Default)]
pub struct PercentDecoder {
    _private: PhantomData<()>
}
impl PercentDecoder {
    /// Creates a new percent decoder
    pub const fn new() -> Self {
        Self { _private: PhantomData }
    }
    
    /// Percent-encodes some data
    pub fn copy<I, O>(&self, source: &mut I, sink: &mut O) -> Result<usize> where I: BufRead, O: Write {
        // Copy-decode all bytes
        let mut written = 0;
        while let Some(byte) = source.peek_one()? {
            // Fill the buffer
            let mut buf = match byte {
                b'%' => Buf::new3(),
                _ => Buf::new1()
            };
            source.read_exact(&mut buf)?;

            // Decode the buffer if necessary and write it
            let decoded = match byte {
                b'%' => self.decode_buf(&buf)?,
                _ => buf[0]
            };
            sink.write_all(slice::from_ref(&decoded))?;
            written += 1;
        }
        Ok(written)
    }

    /// Encodes a nibble into a hex char
    fn decode_nibble(&self, nibble: u8) -> Result<u8> {
        match nibble {
            b'0'..=b'9' => Ok(nibble - b'0'),
            b'a'..=b'f' => Ok((nibble - b'a') + 0xA),
            b'A'..=b'F' => Ok((nibble - b'A') + 0xA),
            nibble => Err(einval!("Invalid hex value: {nibble}"))
        }
    }
    /// Encodes a byte
    fn decode_buf(&self, buf: &Buf) -> Result<u8> {
        let (high, low) = (buf[1], buf[2]);
        Ok(self.decode_nibble(high)? << 4 | self.decode_nibble(low)?)
    }
}
