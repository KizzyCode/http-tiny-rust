use std;

/// Provides an API to receive a sized HTTP-response-body
#[derive(Debug)]
pub struct SizedBodyReader {
	body_size: u64,
	remaining: u64
}
impl SizedBodyReader {
	/// Creates a new `SizedBodyReader`
	pub fn new(size: u64) -> Self {
		SizedBodyReader{ body_size: size, remaining: size }
	}
}
impl super::ReadableBody for SizedBodyReader {
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error> {
		// Determine the amount of bytes to read
		let to_receive = if self.remaining > (buffer.len() - *buffer_pos) as u64 { buffer.len() - *buffer_pos }
			else { self.remaining as usize };
		
		// Read bytes and decrement remaining
		let old_pos = *buffer_pos;
		let read_result = connection.read(&mut buffer[.. *buffer_pos + to_receive], buffer_pos, timeout);
		self.remaining -= (*buffer_pos - old_pos) as u64;
		
		read_result?;
		Ok(self.remaining == 0)
	}
	
	fn size(&self) -> Option<u64> {
		Some(self.body_size)
	}
}



/// Provides an API to write a sized HTTP-body
#[derive(Debug)]
pub struct SizedBodyWriter {
	remaining: u64
}
impl SizedBodyWriter {
	/// Creates a new `SizedBodyWriter`
	pub fn new(size: u64) -> Self {
		SizedBodyWriter{ remaining: size }
	}
}
impl super::WriteableBody for SizedBodyWriter {
	fn write(&mut self, buffer: &[u8], buffer_pos: &mut usize, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Check if the body has been finalized
		if self.remaining == 0 { return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)) }
		
		// Determine the amount of bytes to write
		let to_write = if self.remaining > (buffer.len() - *buffer_pos) as u64 { buffer.len() - *buffer_pos }
			else { self.remaining as usize };
		
		// Write bytes and update `remaining`
		let old_pos = *buffer_pos;
		let write_result = connection.write(&buffer[.. *buffer_pos + to_write], buffer_pos, timeout);
		self.remaining -= (*buffer_pos - old_pos) as u64;
		
		write_result
	}
	
	fn finalize(&mut self, _: &mut super::WriteableStream, _: std::time::Duration) -> Result<bool, std::io::Error> {
		if self.remaining > 0 { Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)) }
			else { Ok(true) }
	}
}