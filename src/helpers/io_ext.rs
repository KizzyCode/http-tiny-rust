use std::io::{ self, Read, Write };


/// An extension of the `Write` trait
pub trait ReadExt {
	/// Reads until either `pat` is matched or `buf` is filled completely
	///
	/// Returns either `Some(bytes_read)` if the pattern has been matched or `None` if `buf` has
	/// been filled completely without matching the pattern.
	fn read_until(&mut self, buf: &mut[u8], pat: impl AsRef<[u8]>)
		-> Result<Option<usize>, io::Error>;
}
impl<T: Read> ReadExt for T {
	fn read_until(&mut self, buf: &mut[u8], pat: impl AsRef<[u8]>)
		-> Result<Option<usize>, io::Error>
	{
		let pat = pat.as_ref();

		// Read the input byte-per-byte and check for the pattern
		let mut pos = 0;
		while pos < buf.len() {
			// Read the next byte and adjust `pos`
			match self.read_exact(&mut buf[pos .. pos + 1]) {
				Ok(_) => pos += 1,
				Err(e) => match e.kind() {
					io::ErrorKind::UnexpectedEof => return Ok(None),
					e => Err(e)?
				}
			}
			
			// Check for pattern
			if pos >= pat.len() && &buf[pos - pat.len() .. pos] == pat {
				return Ok(Some(pos))
			}
		}
		Ok(None)
	}
}


/// An extension of the `Write` trait
pub trait WriteExt {
	/// Writes all of `data`
	fn write(&mut self, data: impl AsRef<[u8]>) -> Result<&mut Self, io::Error>;
}
impl<T: Write> WriteExt for T {
	fn write(&mut self, data: impl AsRef<[u8]>) -> Result<&mut Self, io::Error> {
		self.write_all(data.as_ref())?;
		Ok(self)
	}
}