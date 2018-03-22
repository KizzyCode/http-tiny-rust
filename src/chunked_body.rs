use std;

use super::io::{ self, WriteableBuffer };
use super::{ Error, HttpError };

#[derive(Debug, Clone)]
pub struct ChunkLength {
	len: u64
}
impl ChunkLength {
	/// Creates a new chunk-length
	pub fn new(length: u64) -> Self {
		ChunkLength{ len: length }
	}
	
	/// Parses a chunk-length-field
	pub fn parse(data: &[u8]) -> Result<Self, Error<HttpError>> {
		// Validate length and remove trailing b"\r\n"
		if data.len() < 3 || &data[data.len() - 2 ..] != b"\r\n" { throw_err!(HttpError::ProtocolError, "Invalid chunk-length") }
		let data = &data[.. data.len() - 2];
		
		// Decode and parse length-string
		let parsed = {
			let decoded = try_err_from!(String::from_utf8(data.to_owned()));
			try_err_from!(u64::from_str_radix(&decoded, 16))
		};
		
		Ok(ChunkLength{ len: parsed })
	}
	
	/// Receives a chunk-length-field into `buffer` (=> reads until either `b"\r\n"` is received or
	/// the buffer is full which causes an error) and then parses the buffer
	///
	/// _Note: If this function gets interrupted (fails with a non-fatal `IoError`) you can
	/// retry/continue reading by providing the same `input` and `buffer` again_
	pub fn read_length(input: &mut io::Reader, buffer: &mut io::WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<Self, Error<HttpError>> {
		// Try to receive the length
		try_err!(input.read_until(b"\r\n", buffer, timeout), HttpError::IoError);
		
		// Parse the buffer
		Ok(try_err!(ChunkLength::parse(buffer.processed())))
	}
	
	/// Returns the serialized length of this chunk-length
	pub fn serialized_len(&self) -> usize {
		format!("{:X}\r\n", self.len).len()
	}
	
	/// Returns `(payload_length, total_length)`
	///
	/// _Info: This function returns a tuple because a HTTP-chunk consists of `payload || b"\r\n"`.
	/// This means if you don't want to fuck up your stream you should not forget to read the
	/// trailing newline!_
	pub fn len(&self) -> (u64, u64) {
		(self.len, self.len + 2)
	}
	
	/// Serializes the chunk-length into `buffer`
	///
	/// Returns the serialized length
	pub fn serialize(&self, buffer: &mut[u8]) -> Result<usize, Error<HttpError>> {
		// Validate buffer-size and create IO-buffer
		if buffer.len() < self.serialized_len() { throw_err!(HttpError::ApiMisuse) }
		let mut buffer = io::MutableBackedBuffer::new(buffer);
		
		// Serialize length
		buffer.write(format!("{:X}\r\n", self.len).as_bytes());
		Ok(self.serialized_len())
	}
	
	/// Serializes the chunk-length into `buffer` and sends it
	///
	/// __Warning: The buffer must have the SAME length like the serialized chunk-length_
	///
	/// __Warning: If `buffer.pos()` greater than `0`, the sending will be resumed WITHOUT
	/// serializing the chunk-length__
	///
	/// _Note: If this function gets interrupted (fails with a non-fatal `IoError`) you can
	/// retry/continue writing by providing the same `output` and `buffer` again_
	pub fn write_length<T: io::ReadableBuffer<u8> + io::WriteableBuffer<u8>>(&self, output: &mut io::Writer, buffer: &mut T, timeout: std::time::Duration) -> Result<(), Error<HttpError>> {
		// Serialize if necessary
		if io::WriteableBuffer::pos(buffer) == 0 { try_err!(self.serialize(buffer.remaining_mut())); }
		
		// Send the (remaining) buffer
		Ok(try_err!(output.write_exact(buffer, timeout), HttpError::IoError))
	}
}