use std;

use super::io::{ self, WriteableBuffer };
use super::{ Error, HttpError };



pub struct Header {
	pub header_line: (String, String, String),
	pub fields: std::collections::BTreeMap<String, String>
}
impl Header {
	/// Creates a new header
	pub fn new<S: ToString>(header_line: (S, S, S), fields: &[(S, S)]) -> Self {
		// Map fields
		let mut fields_map = std::collections::BTreeMap::new();
		for key_value in fields { fields_map.insert(key_value.0.to_string(), key_value.1.to_string()); }
		
		Header { header_line: (header_line.0.to_string(), header_line.1.to_string(), header_line.2.to_string()), fields: fields_map }
	}
	
	/// Parses a header
	pub fn parse(data: &[u8]) -> Result<Self, Error<HttpError>> {
		// Convert header to text
		let header = try_err_from!(String::from_utf8(data.to_vec()), "Invalid header-encoding");
		
		// Create line-iterator
		let mut lines = header.trim_right().split("\r\n");
		
		// Parse request-line
		let header_line = {
			let segments = if let Some(line) = lines.next() { line.split(" ").collect::<Vec<&str>>() }
				else { throw_err!(HttpError::ProtocolError, "No request-line given") };
			
			if segments.len() == 3 { (segments[0].to_string(), segments[1].to_string(), segments[2].to_string()) }
				else { throw_err!(HttpError::ProtocolError, "Invalid request-line") }
		};
		
		// Parse fields
		let mut fields = std::collections::BTreeMap::new();
		for line in lines {
			let key_value = line.split(" ").collect::<Vec<&str>>();
			if key_value.len() == 2 { fields.insert(key_value[0].to_string(), key_value[1].trim_left().to_string()); }
				else { throw_err!(HttpError::ProtocolError, "Invalid header-field") }
		}
		
		Ok(Header { header_line, fields })
	}
	
	/// Receives a header into `buffer` (=> reads until either `b"\r\n\r\n"` is received or the
	/// buffer is full which causes an error) and then parses the buffer
	///
	/// _Note: If this function gets interrupted (fails with a non-fatal `IoError`) you can
	/// retry/continue reading by providing the same `input` and `buffer` again_
	pub fn read_header(input: &mut io::Reader, buffer: &mut io::WriteableBuffer<u8>, timeout: std::time::Duration) -> Result<Self, Error<HttpError>> {
		// Try to receive the header
		try_err!(input.read_until(b"\r\n\r\n", buffer, timeout), HttpError::IoError);
		
		// Parse the buffer
		Ok(try_err!(Header::parse(buffer.processed())))
	}
	
	
	
	/// Returns the serialized length of this header
	pub fn serialized_len(&self) -> usize {
		let mut len = self.header_line.0.len() + 1 + self.header_line.1.len() + 1 + self.header_line.2.len() + 2;
		for (key, value) in self.fields.iter() { len += key.len() + 2 + value.len() + 2 }
		len + 4
	}
	
	/// Serializes the header into `buffer`
	///
	/// Returns the serialized length
	pub fn serialize(&self, buffer: &mut[u8]) -> Result<usize, Error<HttpError>> {
		// Validate buffer-size and create IO-buffer
		if buffer.len() < self.serialized_len() { throw_err!(HttpError::ApiMisuse) }
		let mut buffer = io::MutableBackedBuffer::new(buffer);
		
		// Serialize request-line, fields and append newline
		buffer.write(format!("{} {} {}\r\n", self.header_line.0, self.header_line.1, self.header_line.2).as_bytes());
		for (key, value) in self.fields.iter() { buffer.write(format!("{}: {}\r\n", key, value).as_bytes()) }
		buffer.write(b"\r\n");
		
		Ok(self.serialized_len())
	}
	
	/// Serializes the header into `buffer` and sends it
	///
	/// __Warning: The buffer must have the SAME length like the serialized header__
	///
	/// __Warning: If `buffer.pos()` greater than `0`, the sending will be resumed WITHOUT
	/// serializing the header__
	///
	/// _Note: If this function gets interrupted (fails with a non-fatal `IoError`) you can
	/// retry/continue writing by providing the same `output` and `buffer` again_
	pub fn write_header<T: io::ReadableBuffer<u8> + io::WriteableBuffer<u8>>(&self, output: &mut io::Writer, buffer: &mut T, timeout: std::time::Duration) -> Result<(), Error<HttpError>> {
		// Serialize if necessary
		if *io::WriteableBuffer::pos(buffer) == 0 { try_err!(self.serialize(buffer.remaining())); }
		
		// Send the (remaining) buffer
		Ok(try_err!(output.write_exact(buffer, timeout), HttpError::IoError))
	}
	
	/// View this header as request-header
	pub fn as_request(&mut self) -> RequestHeaderView {
		RequestHeaderView{ method: &mut self.header_line.0, uri: &mut self.header_line.1, version: &mut self.header_line.2, fields: &mut self.fields }
	}
	
	/// View this header as response-header
	pub fn as_response(&mut self) -> ResponseHeaderView {
		ResponseHeaderView{ version: &mut self.header_line.0, code: &mut self.header_line.1, message: &mut self.header_line.2, fields: &mut self.fields }
	}
}



pub struct RequestHeaderView<'a> {
	pub method: &'a mut String,
	pub uri: &'a mut String,
	pub version: &'a mut String,
	pub fields: &'a mut std::collections::BTreeMap<String, String>
}

pub struct ResponseHeaderView<'a> {
	pub version: &'a mut String,
	pub code: &'a mut String,
	pub message: &'a mut String,
	pub fields: &'a mut std::collections::BTreeMap<String, String>
}