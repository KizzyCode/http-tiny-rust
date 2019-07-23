use crate::{
	HttpError,
	data::{
		Data,
		encodings::{ Ascii, HeaderFieldKey, Uri, Integer }
	},
	helpers::{
		iter_ext::IterExt,
		io_ext::{ ReadExt, WriteExt },
		slice_ext::{ ByteSliceExt, SliceExt }
	}
};
use std::{
	collections::HashMap,
	io::{ self, Read, Write, Cursor },
	convert::{ TryFrom, TryInto }
};


/// Reads from `source` into `buf` until a HTTP-header-end is matched or `buf` is filled completely
///
/// Returns either `Some(header_len)` if the header end has been matched or `None` if `buf` has been
/// filled completely without a match.
pub fn read(mut source: impl Read, buf: &mut[u8]) -> Result<Option<usize>, io::Error> {
	const END: &[u8] = b"\r\n\r\n";
	source.read_until(buf, &END)
}
/// Parses a HTTP request header from `bytes`
///
/// Returns the header and the remaining body data in `bytes` if any (`(header, body_data)`)
pub fn parse_request(bytes: &[u8]) -> Result<(RequestHeader, &[u8]), HttpError> {
	let (header, body) = Header::parse(bytes)?;
	Ok((RequestHeader(header), body))
}
/// Parses a HTTP response header from `bytes`
///
/// Returns the header and the remaining body data in `bytes` if any (`(header, body_data)`)
pub fn parse_response(bytes: &[u8]) -> Result<(ResponseHeader, &[u8]), HttpError> {
	let (header, body) = Header::parse(bytes)?;
	Ok((ResponseHeader(header), body))
}


/// An opaque HTTP header implementation
#[derive(Debug)]
pub(in crate::header) struct Header {
	pub header_line: (Vec<u8>, Vec<u8>, Vec<u8>),
	pub header_fields: HashMap<Data<HeaderFieldKey>, Data<Ascii>>
}
impl Header {
	fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), HttpError> {
		const SPACE: &[u8] = b" ";
		const SEPARATOR: &[u8] = b":";
		const NEWLINE: &[u8] = b"\r\n";
		const END: &[u8] = b"\r\n\r\n";
		
		// Split data into header and body
		let header_body = bytes.splitn_pat(2, &END)
			.collect_min(2).ok_or(HttpError::TruncatedData)?;
		let mut header = header_body[0].split_pat(&NEWLINE);
		let body = header_body[1];
		
		// Parse status line
		let status_line = header.next().ok_or(HttpError::ProtocolViolation)?
			.trim().split_pat(&SPACE)
			.collect_exact(3).ok_or(HttpError::ProtocolViolation)?;
		let status_line = (status_line[0].into(), status_line[1].into(), status_line[2].into());
		
		// Parse header fields
		let mut header_fields = HashMap::new();
		while let Some(line) = header.next() {
			let key_value = line.splitn_pat(2, &SEPARATOR)
				.collect_min(2).ok_or(HttpError::ProtocolViolation)?;
			header_fields.insert(
				Data::try_from(key_value[0])?,
				Data::try_from(key_value[1].trim())?
			);
		}
		Ok((Self{ header_line: status_line, header_fields }, body))
	}
	
	fn serialize(&self, mut sink: impl WriteExt) -> Result<usize, io::Error> {
		const SPACE: &[u8] = b" ";
		const SEPARATOR: &[u8] = b": ";
		const NEWLINE: &[u8] = b"\r\n";
		let mut written = 0;
		
		// Write header line
		sink.write(&self.header_line.0)?.write(SPACE)?
			.write(&self.header_line.1)?.write(SPACE)?
			.write(&self.header_line.2)?.write(NEWLINE)?;
		written += self.header_line.0.len() + SPACE.len()
			+ self.header_line.1.len() + SPACE.len()
			+ self.header_line.2.len() + NEWLINE.len();
		
		// Write header fields
		for (k, v) in self.header_fields.iter() {
			sink.write(k)?.write(SEPARATOR)?.write(v)?.write(NEWLINE)?;
			written += k.len() + SEPARATOR.len() + v.len() + NEWLINE.len();
		}
		
		// Write trailing newline
		sink.write(NEWLINE)?;
		written += NEWLINE.len();
		Ok(written)
	}
}


/// A HTTP request header
#[derive(Debug)]
pub struct RequestHeader(pub(in crate::header) Header);
impl RequestHeader {
	/// The request method
	pub fn method(&self) -> Result<Data<Ascii>, HttpError> {
		self.0.header_line.0.as_slice().try_into()
	}
	/// The requested URI
	pub fn uri(&self) -> Result<Data<Uri>, HttpError> {
		self.0.header_line.1.as_slice().try_into()
	}
	/// The HTTP version
	pub fn version(&self) -> Result<Data<Ascii>, HttpError> {
		self.0.header_line.2.as_slice().try_into()
	}
	
	/// Gets the field for `key` if any
	pub fn field(&self, key: Data<HeaderFieldKey>) -> Option<&Data<Ascii>> {
		self.0.header_fields.get(&key)
	}
	/// Returns an iterator over all header fields
	pub fn fields(&self) -> &HashMap<Data<HeaderFieldKey>, Data<Ascii>> {
		&self.0.header_fields
	}
	
	/// Serializes and writes the header to `sink` and returns the amount of bytes written
	pub fn write(&self, sink: &mut dyn Write) -> Result<usize, io::Error> {
		self.0.serialize(sink)
	}
	
	/// Serializes the header into a vector
	pub fn to_vec(&self) -> Vec<u8> {
		let mut sink = Cursor::new(Vec::new());
		self.write(&mut sink).unwrap();
		sink.into_inner()
	}
}


/// A HTTP response header
#[derive(Debug)]
pub struct ResponseHeader(pub(in crate::header) Header);
impl ResponseHeader {
	/// The HTTP version
	pub fn version(&self) -> Result<Data<Ascii>, HttpError> {
		self.0.header_line.0.as_slice().try_into()
	}
	/// The status code
	pub fn status(&self) -> Result<u16, HttpError> {
		let status = Data::<Integer>::try_from(self.0.header_line.1.as_slice())?;
		Ok(u16::try_from(status).map_err(|_| HttpError::ProtocolViolation)?)
	}
	/// The status reason
	pub fn reason(&self) -> Result<Data<Ascii>, HttpError> {
		self.0.header_line.2.as_slice().try_into()
	}
	
	/// Gets the field for `key` if any
	pub fn field(&self, key: Data<HeaderFieldKey>) -> Option<&Data<Ascii>> {
		self.0.header_fields.get(&key)
	}
	/// Returns an iterator over all header fields
	pub fn fields(&self) -> &HashMap<Data<HeaderFieldKey>, Data<Ascii>> {
		&self.0.header_fields
	}
	
	/// Serializes and writes the header to `sink` and returns the amount of bytes written
	pub fn write(&self, sink: &mut dyn Write) -> Result<usize, io::Error> {
		self.0.serialize(sink)
	}
	
	/// Serializes the header into a vector
	pub fn to_vec(&self) -> Vec<u8> {
		let mut sink = Cursor::new(Vec::new());
		self.write(&mut sink).unwrap();
		sink.into_inner()
	}
}