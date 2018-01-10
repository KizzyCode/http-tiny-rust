use std;

/// A HTTP-request-header
#[derive(Clone, Eq, PartialEq)]
pub struct RequestHeader {
	/// The HTTP-version (default "HTTP/1.1")
	pub http_version: String,
	/// The HTTP-method (default "GET")
	pub http_method: String,
	/// The HTTP-request-URI (default "/")
	pub request_uri: String,
	/// The HTTP-header-fields (the keys do __not__ contain a trailing colon)
	pub header_fields: std::collections::HashMap<String, String>
}
impl RequestHeader {
	/// Initialize the `RequestHeader` with the data from a `RequestHeaderReader`
	pub fn from_reader(reader: RequestHeaderReader) -> Result<Self, std::io::Error> {
		let ((http_method, request_uri, http_version), header_fields) = super::parsers::http_header::parse(reader.data)?;
		if http_version != "HTTP/1.0" && http_version != "HTTP/1.1" { return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) }
		
		Ok(RequestHeader{ http_version, http_method, request_uri, header_fields })
	}
	
	/// Converts this `RequestHeader` into a `RequestHeaderWriter`
	pub fn into_writer(self) -> RequestHeaderWriter {
		// Serialize header-line
		let mut serialized = format!("{0} {1} {2}\r\n", &self.http_method, &self.request_uri, &self.http_version);
		// Serialize header-fields
		for (key, value) in self.header_fields.iter() { serialized += &format!("{0}: {1}\r\n", key, value); }
		// Append last line-break
		serialized += "\r\n";
		
		RequestHeaderWriter{ data: serialized.into_bytes(), position: 0 }
	}
}
impl Default for RequestHeader {
	fn default() -> Self {
		RequestHeader {
			http_version: "HTTP/1.1".to_owned(),
			http_method: "GET".to_owned(),
			request_uri: "/".to_owned(),
			header_fields: std::collections::HashMap::new()
		}
	}
}
impl std::fmt::Debug for RequestHeader {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		// Parse fields
		let mut fields = String::new();
		for (key, value) in self.header_fields.iter() { fields += &format!("\"{0}\": \"{1}\"\n", key, value) }
		
		write!(
			formatter,
			"Method: \"{0}\", URI: \"{1}\", HTTP-version: \"{2}\"\n{3}",
			self.http_method, self.request_uri, self.http_version, fields
		)
	}
}



/// The `RequestHeaderReader` can be used to receive an entire HTTP-request without parsing it
#[derive(Debug)]
pub struct RequestHeaderReader {
	pub data: Vec<u8>,
	position: usize
}
impl RequestHeaderReader {
	/// Creates a new `RequestHeaderReader` which accepts headers <= `max_header_size` bytes
	pub fn new(max_header_size: usize) -> Self {
		RequestHeaderReader{ data: vec![0u8; max_header_size], position: 0 }
	}
}
impl super::ReadableHeader for RequestHeaderReader {
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



/// The `RequestHeaderWriter` can be used to write an entire HTTP-request
#[derive(Debug)]
pub struct RequestHeaderWriter {
	pub data: Vec<u8>,
	position: usize
}
impl super::WriteableHeader for RequestHeaderWriter {
	fn write(&mut self, connection: &mut super::WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		connection.write(&self.data, &mut self.position, timeout)
	}
}