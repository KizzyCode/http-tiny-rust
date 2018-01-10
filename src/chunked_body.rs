use std;

#[derive(Eq, PartialEq, Debug)]
enum ReaderChunkState {
	Header([u8; 18], usize),
	Payload(u64),
	Trailer([u8; 2], usize, bool),
	Done
}
/// Provides an API to receive and parse chunked HTTP-bodies
#[derive(Debug)]
pub struct ChunkedBodyReader {
	state: ReaderChunkState
}
impl ChunkedBodyReader {
	/// Creates a new `ChunkedBodyReader`
	pub fn new() -> Self {
		ChunkedBodyReader{ state: ReaderChunkState::Header([0u8; 18], 0) }
	}
	
	fn receive_chunk_size(&mut self, mut state: ([u8; 18], usize), connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Read until we receive the line-break, update state and check result
		if let Err(error) = connection.read_until(&mut state.0, &mut state.1, "\r\n".as_bytes(), timeout) {
			self.state = ReaderChunkState::Header(state.0, state.1);
			return Err(error)
		}
		
		// Decode length-string
		let decoded = if let Ok(decoded) = String::from_utf8(state.0[.. state.1 - 2].to_owned()) { decoded }
			else { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) };
		
		// Parse length-string
		let parsed = if let Ok(parsed) = u64::from_str_radix(&decoded, 16) { parsed }
			else { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) };
		
		// Check for finalizer
		if parsed == 0 { self.state = ReaderChunkState::Trailer([0u8; 2], 0, true) }
			else { self.state = ReaderChunkState::Payload(parsed) }
		Ok(())
	}
	
	fn receive_chunk_payload(&mut self, mut remaining: u64, buffer: &mut[u8], buffer_pos: &mut usize, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Determine the amount of bytes to receive
		let to_receive = if remaining > (buffer.len() - *buffer_pos) as u64 { buffer.len() - *buffer_pos }
			else { remaining as usize };
		
		// Receive payload
		let old_pos = *buffer_pos;
		let result = connection.read(&mut buffer[.. *buffer_pos + to_receive], buffer_pos, timeout);
		remaining -= (*buffer_pos - old_pos) as u64;
		
		// Update state
		self.state = if remaining > 0 { ReaderChunkState::Payload(remaining) }
			else { ReaderChunkState::Trailer([0u8; 2], 0, false) };
		result
	}
	
	fn receive_chunk_trailer(&mut self, mut state: ([u8; 2], usize), is_last: bool, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Read the line-break
		if let Err(error) = connection.read_until(&mut state.0, &mut state.1, "\r\n".as_bytes(), timeout) {
			self.state = ReaderChunkState::Trailer(state.0, state.1, is_last);
			return Err(error)
		}
		
		// Set state
		self.state = if is_last { ReaderChunkState::Done }
			else { ReaderChunkState::Header([0u8; 18], 0) };
		Ok(())
	}
}
impl super::ReadableBody for ChunkedBodyReader {
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error> {
		let timeout_point = std::time::Instant::now() + timeout;
		while *buffer_pos < buffer.len() {
			match self.state {
				ReaderChunkState::Header(buffer, pos) => self.receive_chunk_size((buffer, pos), connection, super::time_remaining(timeout_point))?,
				ReaderChunkState::Payload(remaining) => self.receive_chunk_payload(remaining, buffer, buffer_pos, connection, super::time_remaining(timeout_point))?,
				ReaderChunkState::Trailer(buffer, pos, is_last) => self.receive_chunk_trailer((buffer, pos), is_last, connection, super::time_remaining(timeout_point))?,
				ReaderChunkState::Done => return Ok(true)
			};
		}
		Ok(false)
	}
	fn size(&self) -> Option<u64> {
		None
	}
}



#[derive(Eq, PartialEq, Debug)]
enum WriterChunkState {
	Waiting,
	Header(usize, usize), // to_send, buffer_pos
	Payload(usize),
	Trailer([u8; 2], usize, bool),
	Finalized
}
/// Provides an API to write chunked HTTP-bodies
#[derive(Debug)]
pub struct ChunkedBodyWriter {
	state: WriterChunkState
}
impl ChunkedBodyWriter {
	/// Creates a new `ChunkedBodyWriter`
	pub fn new() -> Self {
		ChunkedBodyWriter{ state: WriterChunkState::Waiting }
	}
	
	fn init_chunk(&mut self, buffer: &[u8], buffer_pos: &mut usize) {
		// Determine the amount of bytes to send and update state
		let to_send = buffer.len() - *buffer_pos;
		self.state = WriterChunkState::Header(to_send, 0);
	}
	
	fn send_chunk_size(&mut self, mut state: (usize, usize), connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Encode length
		let encoded = format!("{0:x}\r\n", state.0);
		
		// Try to send the encoded length
		if let Err(error) = connection.write(encoded.as_bytes(), &mut state.1, timeout) {
			self.state = WriterChunkState::Header(state.0, state.1);
			return Err(error);
		}
		
		// Update state
		self.state = if state.0 == 0 { WriterChunkState::Trailer([0xD, 0xA], 0, true) }
			else { WriterChunkState::Payload(state.0) };
		Ok(())
	}
	
	fn send_chunk_payload(&mut self, mut remaining: usize, buffer: &[u8], buffer_pos: &mut usize, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Determine the amount of data to send
		let to_send = if remaining > (buffer.len() - *buffer_pos) { buffer.len() - *buffer_pos }
			else { remaining };
		
		// Send payload
		let old_pos = *buffer_pos;
		let result = connection.write(&buffer[.. *buffer_pos + to_send], buffer_pos, timeout);
		remaining -= *buffer_pos - old_pos;
		
		// Update state
		self.state = if remaining > 0 { WriterChunkState::Payload(remaining) }
			else { WriterChunkState::Trailer([0u8; 2], 0, false) };
		result
	}
	
	fn send_chunk_trailer(&mut self, mut state: ([u8; 2], usize), is_last: bool, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Write the line-break
		if let Err(error) = connection.write(&mut state.0, &mut state.1, timeout) {
			self.state = WriterChunkState::Trailer(state.0, state.1, is_last);
			return Err(error)
		}
		
		// Set state
		self.state = if is_last { WriterChunkState::Finalized }
			else { WriterChunkState::Waiting };
		Ok(())
	}
	
	fn write_chunk(&mut self, buffer: &[u8], buffer_pos: &mut usize, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		let timeout_point = std::time::Instant::now() + timeout;
		while self.state != WriterChunkState::Finalized {
			match self.state {
				WriterChunkState::Waiting => self.init_chunk(buffer, buffer_pos),
				WriterChunkState::Header(to_send, pos) => self.send_chunk_size((to_send, pos), connection, super::time_remaining(timeout_point))?,
				WriterChunkState::Payload(pending) => self.send_chunk_payload(pending, buffer, buffer_pos, connection, super::time_remaining(timeout_point))?,
				WriterChunkState::Trailer(buffer, pos, is_last) => self.send_chunk_trailer((buffer, pos), is_last, connection, super::time_remaining(timeout_point))?,
				_ => ()
			}
		}
		Ok(())
	}
}
impl super::WriteableBody for ChunkedBodyWriter {
	fn write(&mut self, buffer: &[u8], buffer_pos: &mut usize, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		// Empty chunks are not allowed in chunked bodies (except as finalizer) so we ignore them
		if *buffer_pos < buffer.len() { self.write_chunk(buffer, buffer_pos, connection, timeout) }
			else if let WriterChunkState::Finalized = self.state { return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)) }
			else { Ok(()) }
	}
	
	fn finalize(&mut self, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error> {
		// Send an empty chunk as finalizer
		if self.state != WriterChunkState::Finalized { self.write_chunk(&[], &mut 0, connection, timeout)? }
		Ok(true)
	}
}