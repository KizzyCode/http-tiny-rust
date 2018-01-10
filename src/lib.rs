pub mod parsers;
pub mod request_header;
pub mod response_header;
pub mod legacy_body;
pub mod sized_body;
pub mod chunked_body;

pub type RequestHeader = request_header::RequestHeader;
pub type RequestHeaderReader = request_header::RequestHeaderReader;
pub type RequestHeaderWriter = request_header::RequestHeaderWriter;

pub type ResponseHeader = response_header::ResponseHeader;
pub type ResponseHeaderReader = response_header::ResponseHeaderReader;
pub type ResponseHeaderWriter = response_header::ResponseHeaderWriter;

pub type LegacyBodyReader = legacy_body::LegacyBodyReader;
pub type LegacyBodyWriter = legacy_body::LegacyBodyWriter;

pub type SizedBodyReader = sized_body::SizedBodyReader;
pub type SizedBodyWriter = sized_body::SizedBodyWriter;

pub type ChunkedBodyReader = chunked_body::ChunkedBodyReader;
pub type ChunkedBodyWriter = chunked_body::ChunkedBodyWriter;

/// A readable stream (probably a TCP-stream)
pub trait ReadableStream {
	/// Read `buffer.len() - buffer_pos` bytes from the stream into `buffer` starting at `buffer_pos`
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, timeout: std::time::Duration) -> Result<(), std::io::Error>;
	
	/// Read up to `buffer.len() - buffer_pos` bytes from the stream into `buffer` starting at
	/// `buffer_pos` until `pattern` is matched (pattern must be included in `buffer`)
	fn read_until(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, pattern: &[u8], timeout: std::time::Duration) -> Result<(), std::io::Error>;
}
pub trait WriteableStream {
	/// Write `buffer.len() - buffer_pos` bytes starting at `data_pos` into the stream
	fn write(&mut self, data: &[u8], data_pos: &mut usize, timeout: std::time::Duration) -> Result<(), std::io::Error>;
}


pub trait ReadableHeader {
	/// Reads a HTTP-header from `connection`
	///
	/// On non-fatal errors like `std::io::ErrorKind::TimedOut` you can try to call `read` again to complete the operation.
	fn read(&mut self, connection: &mut ReadableStream, timeout: std::time::Duration) -> Result<(), std::io::Error>;
}
pub trait WriteableHeader {
	/// Sends the HTTP-header over `connection`
	///
	/// On non-fatal errors like `std::io::ErrorKind::TimedOut` you can try to call `write` again to complete the operation.
	fn write(&mut self, connection: &mut WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error>;
}

pub trait ReadableBody {
	/// Reads the HTTP-body from `connection` into `buffer[buffer_pos ..]`
	///
	/// Returns `true` if the entire body was read or `false` if there are pending bytes.
	/// `buffer_pos` will be incremented by the amount of received bytes.
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, connection: &mut ReadableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error>;
	
	/// Returns the size of the body or `None` if the size is indetermined
	fn size(&self) -> Option<u64>;
}
pub trait WriteableBody {
	/// Send `buffer[buffer_pos ..]` as HTTP-body
	///
	/// On non-fatal errors like `std::io::ErrorKind::TimedOut` you can try to call `write` again to write more bytes.
	/// `buffer_pos` will be incremented by the amount of sent bytes.
	fn write(&mut self, buffer: &[u8], buffer_pos: &mut usize, connection: &mut WriteableStream, timeout: std::time::Duration) -> Result<(), std::io::Error>;
	
	/// Finalizes an unsized HTTP-body (e.g. chunked or legacy)
	///
	/// Returns `true` if the body could be finalized or `false` if there are pending bytes.
	fn finalize(&mut self, connection: &mut WriteableStream, timeout: std::time::Duration) -> Result<bool, std::io::Error>;
}

/// Underflow-safe computation of the remaining time
pub fn time_remaining(timeout_point: std::time::Instant) -> std::time::Duration {
	let now = std::time::Instant::now();
	if now > timeout_point { std::time::Duration::default() } else { timeout_point - now }
}