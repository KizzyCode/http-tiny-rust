use std;

/// A HTTP-response-header
#[derive(Clone, Eq, PartialEq)]
pub struct ResponseHeader {
	pub http_version: String,
	pub http_status_code_reason: (u16, String),
	pub header_fields: std::collections::HashMap<String, String>
}
impl ResponseHeader {
	/// Initialize the `ResponseHeader` with the data from a `ResponseHeaderReader`
	pub fn from_reader(reader: ResponseHeaderReader) -> Result<Self, std::io::Error> {
		let ((http_version, http_status_code, http_status_reason), header_fields) = super::parsers::http_header::parse(reader.data)?;
		// Validate HTTP-version
		if http_version != "HTTP/1.0" && http_version != "HTTP/1.1" { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) }
		// Parse status-code
		let http_status_code = if let Ok(code) = http_status_code.parse::<u16>() { code }
			else { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) };
		Ok(ResponseHeader{ http_version, http_status_code_reason: (http_status_code, http_status_reason), header_fields })
	}
	
	/// Converts this `ResponseHeader` into a `ResponseHeaderWriter`
	pub fn into_writer(self) -> ResponseHeaderWriter {
		// Serialize header-line
		let mut serialized = format!("{0} {1} {2}\r\n", self.http_version, self.http_status_code_reason.0, self.http_status_code_reason.1);
		// Serialize header-fields
		for (key, value) in self.header_fields.iter() { serialized += &format!("{0}: {1}\r\n", key, value) };
		// Append last line-break
		serialized += "\r\n";
		
		ResponseHeaderWriter{ data: serialized.into_bytes(), position: 0 }
	}
}
impl Default for ResponseHeader {
	fn default() -> Self {
		ResponseHeader {
			http_version: "HTTP/1.1".to_owned(),
			http_status_code_reason: (200, "OK".to_owned()),
			header_fields: std::collections::HashMap::new()
		}
	}
}
impl std::fmt::Debug for ResponseHeader {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		// Parse fields
		let mut fields = String::new();
		for (key, value) in self.header_fields.iter() { fields += &format!("\"{0}\": \"{1}\"\n", key, value) }
		
		write!(
			formatter,
			"HTTP-Version: \"{0}\", Status: \"{1}\", Reason: \"{2}\"\n{3}",
			self.http_version, self.http_status_code_reason.0, self.http_status_code_reason.1, fields
		)
	}
}



/// The `ResponseHeaderReader` can be used to receive an entire HTTP-request without parsing it
#[derive(Debug)]
pub struct ResponseHeaderReader {
	pub data: Vec<u8>,
	position: usize
}
impl ResponseHeaderReader {
	/// Creates a new `ResponseHeaderReader` which accepts headers <= `max_header_size` bytes
	pub fn new(max_header_size: usize) -> Self {
		ResponseHeaderReader{ data: vec![0u8; max_header_size], position: 0 }
	}
}
impl super::ReadableHeader for ResponseHeaderReader {
	fn read(&mut self, connection: &mut super::ReadableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		match connection.read_until(&mut self.data, &mut self.position, "\r\n\r\n".as_bytes(), timeout) {
			Err(ref error) if error.kind() == std::io::ErrorKind::NotFound => { Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) },
			Err(error) => Err(error),
			Ok(_) => {
				self.data.truncate(self.position);
				Ok(())
			}
		}
	}
}



/// The `ResponseHeaderWriter` can be used to write an entire HTTP-request
#[derive(Debug)]
pub struct ResponseHeaderWriter {
	pub data: Vec<u8>,
	position: usize
}
impl super::WriteableHeader for ResponseHeaderWriter {
	fn write(&mut self, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		connection.write(&self.data, &mut self.position, timeout)
	}
}