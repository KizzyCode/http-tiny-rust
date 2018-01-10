use std;

/// The `LegacyBodyReader` provides an API to receive a legacy/unsized HTTP-bodies
#[derive(Debug)]
pub struct LegacyBodyReader {
	completed: bool
}
impl LegacyBodyReader {
	/// Creates a new `LegacyBodyReader`
	pub fn new() -> Self {
		LegacyBodyReader{ completed: false }
	}
}
impl super::ReadableBody for LegacyBodyReader {
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error> {
		// Check if the connection is completed
		if self.completed { return Ok(true) }
		
		// Read data
		match connection.read(buffer, buffer_pos, timeout) {
			Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
				self.completed = true;
				Ok(true)
			},
			Err(error) => Err(error),
			Ok(_) => Ok(false)
		}
	}
	
	fn size(&self) -> Option<u64> {
		None
	}
}



/// Provides an API to write a legacy/unsized HTTP-body
#[derive(Debug)]
pub struct LegacyBodyWriter {
	completed: bool
}
impl LegacyBodyWriter {
	/// Creates a new `LegacyBodyWriter`
	pub fn new() -> Self {
		LegacyBodyWriter{ completed: false }
	}
}
impl super::WriteableBody for LegacyBodyWriter {
	fn write(&mut self, buffer: &[u8], buffer_pos: &mut usize, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Check if the body was finalized
		if self.completed { Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)) }
			else { connection.write(buffer, buffer_pos, timeout) }
	}
	
	fn finalize(&mut self, _: &mut super::WriteableStream, _: std::time::Duration) -> Result<bool, std::io::Error> {
		self.completed = true;
		Ok(true)
	}
}